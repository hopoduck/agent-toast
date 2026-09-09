//! 메인 스레드 응답성 감시.
//!
//! tao 의 이벤트 러너는 이벤트 핸들러가 실행 중인 동안 도착한 이벤트를 버퍼에 쌓고,
//! 그 핸들러가 리턴할 때만 버퍼를 비운다 (tao `event_loop/runner.rs`의 `should_buffer`).
//! wry 가 WebView2 생성을 기다리며 중첩 메시지 루프(`webview2_com::wait_with_pump`)에
//! 갇히면 이 버퍼가 영영 비워지지 않는다. 그러면 Win32 메시지 펌프는 계속 돌아서
//! 트레이 메뉴는 뜨는데, Tauri 이벤트는 하나도 처리되지 않는 상태가 된다.
//! (메뉴 항목을 눌러도 아무 일이 없고, 워커 스레드의 `primary_monitor()` 같은
//! 메인 스레드 왕복 호출은 응답을 영영 못 받는다.)
//!
//! `run_on_main_thread` 도 같은 러너를 거치므로, 주기적으로 ACK 를 요청해 두고
//! 돌아오는지 보면 이 상태를 정확히 겨냥해 잡아낼 수 있다.
//!
//! 감시 스레드는 메인 스레드에 의존하는 호출을 절대 하지 않는다. 공유 상태 조회는
//! 전부 `try_lock` 이고, 실패하면 해당 항목을 비운 채로 보고한다.

use crate::notification::NotificationManagerState;
use once_cell::sync::Lazy;
use std::cell::Cell;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime};
use tauri::{AppHandle, Manager};

/// ACK 요청 주기.
const HEARTBEAT: Duration = Duration::from_secs(5);
/// ACK 요청을 던진 뒤 응답을 기다려 주는 여유. 던지자마자 읽으면 메인 스레드가
/// 아무리 건강해도 직전 사이클의 ACK 를 읽게 되어, 무응답 시간이 늘 하트비트 주기
/// 하나만큼 부풀려진다.
const ACK_GRACE: Duration = Duration::from_millis(200);
/// 이 시간 이상 ACK 가 없으면 먹통으로 판정한다. 정상 동작 중에도 메인 스레드가
/// 몇 초 막히는 구간(웹뷰 생성, 폰트 열거 등)이 있어 여유를 뒀다.
const STALL_THRESHOLD: Duration = Duration::from_secs(20);
/// 먹통이 계속될 때 보고를 반복하는 주기.
const REPORT_INTERVAL: Duration = Duration::from_secs(60);
/// 감시가 살아 있음을 확인하기 위한 debug 로그 주기.
const ALIVE_LOG_INTERVAL: Duration = Duration::from_secs(300);
/// 정상일 때 헬스 파일을 갱신하는 주기. 하트비트마다 쓰면 하루 만 몇 천 번의
/// 무의미한 디스크 쓰기가 되므로 늦춘다. 먹통이거나 상태가 바뀐 순간은 즉시 쓴다.
const HEALTH_WRITE_INTERVAL: Duration = Duration::from_secs(30);
/// 잠들기로 한 시간보다 이만큼 더 잤으면 시스템 절전으로 간주한다.
const SLEEP_SLACK: Duration = Duration::from_secs(10);
/// 중첩 루프를 깨우는 시도를 이 횟수까지만 반복하고, 그 뒤엔 재시작으로 올린다.
/// 깨워도 다음 웹뷰 생성이 또 갇히면 그 프로세스의 WebView2 환경이 회복 불능이라고
/// 보는 편이 낫다.
const MAX_UNSTICKS: u32 = 3;
/// 마지막으로 깨운 뒤 이만큼 조용하면 시도 횟수를 초기화한다. 몇 시간에 한 번씩
/// 걸리는 것과 몇 분 사이에 연달아 걸리는 것은 다른 상황이다.
const UNSTICK_RESET: Duration = Duration::from_secs(600);
/// 갇힌 자리가 웹뷰 생성이 아닐 때 재시작까지 참아 주는 보고 횟수. 메인 스레드가
/// 잠깐 오래 걸리는 것뿐일 수도 있어 한 번 더 지켜본다.
const NON_BUILD_PATIENCE: u32 = 2;
/// 재시작으로 되살아난 인스턴스가 스스로를 알아보는 표식.
pub const RECOVERED_ENV: &str = "AGENT_TOAST_RECOVERED_MS";

// ── breadcrumb ──────────────────────────────────────────────────────────

/// 감시 대상 스레드. 배열 인덱스로 쓰이므로 값이 곧 슬롯 번호다.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(usize)]
pub enum Thread {
    Main = 0,
    Pipe = 1,
    Http = 2,
    Foreground = 3,
}

const SLOTS: usize = 4;
const SLOT_NAMES: [&str; SLOTS] = ["main", "pipe", "http", "foreground"];

/// 각 스레드가 지금 어느 단계에 있는지. 문자열 대신 열거형을 쓰는 이유는 값 하나를
/// 원자적으로 저장하기 위해서다. `&'static str` 은 팻 포인터라 원자 저장이 안 되고,
/// 포인터와 길이를 따로 저장하면 찢어진 읽기가 생긴다.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Step {
    Idle = 0,
    PipeWaitConnect,
    PipeRead,
    HttpRecv,
    ForegroundChange,
    NotifyWin32Lookup,
    NotifyCalcPosition,
    NotifyBuildWindow,
    NotifyAfterBuild,
    NotifyClose,
    NotifyResize,
    SetupBuildWindow,
}

impl Step {
    fn as_str(self) -> &'static str {
        match self {
            Step::Idle => "idle",
            Step::PipeWaitConnect => "pipe-wait-connect",
            Step::PipeRead => "pipe-read",
            Step::HttpRecv => "http-recv",
            Step::ForegroundChange => "foreground-change",
            Step::NotifyWin32Lookup => "notify-win32-lookup",
            Step::NotifyCalcPosition => "notify-calc-position",
            Step::NotifyBuildWindow => "notify-build-window",
            Step::NotifyAfterBuild => "notify-after-build",
            Step::NotifyClose => "notify-close",
            Step::NotifyResize => "notify-resize",
            Step::SetupBuildWindow => "setup-build-window",
        }
    }

    fn from_u8(v: u8) -> Step {
        match v {
            1 => Step::PipeWaitConnect,
            2 => Step::PipeRead,
            3 => Step::HttpRecv,
            4 => Step::ForegroundChange,
            5 => Step::NotifyWin32Lookup,
            6 => Step::NotifyCalcPosition,
            7 => Step::NotifyBuildWindow,
            8 => Step::NotifyAfterBuild,
            9 => Step::NotifyClose,
            10 => Step::NotifyResize,
            11 => Step::SetupBuildWindow,
            _ => Step::Idle,
        }
    }
}

/// 상위 8비트에 단계, 하위 56비트에 기록 시각(ms)을 담는다. 56비트면 220만 년치라
/// 넘칠 일이 없고, 원자 하나에 담기므로 단계와 시각이 어긋나 읽히지 않는다.
const MS_BITS: u32 = 56;
const MS_MASK: u64 = (1u64 << MS_BITS) - 1;

fn pack(step: Step, ms: u64) -> u64 {
    ((step as u64) << MS_BITS) | (ms & MS_MASK)
}

fn unpack(v: u64) -> (Step, u64) {
    (Step::from_u8((v >> MS_BITS) as u8), v & MS_MASK)
}

static CRUMBS: [AtomicU64; SLOTS] = [
    AtomicU64::new(0),
    AtomicU64::new(0),
    AtomicU64::new(0),
    AtomicU64::new(0),
];

/// 마지막으로 메인 스레드가 ACK 한 시각(ms).
static LAST_ACK_MS: AtomicU64 = AtomicU64::new(0);

/// 메인 스레드의 OS 스레드 id. 중첩 메시지 루프를 깨우려면 창이 아니라 스레드로
/// 메시지를 보내야 해서 필요하다. 0 이면 아직 확보 전.
static MAIN_TID: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

static EPOCH: Lazy<Instant> = Lazy::new(Instant::now);

fn now_ms() -> u64 {
    EPOCH.elapsed().as_millis() as u64
}

thread_local! {
    /// 이 스레드가 쓸 breadcrumb 슬롯. 등록하지 않은 스레드의 `mark` 는 무시된다.
    static SLOT: Cell<usize> = const { Cell::new(usize::MAX) };
}

/// 현재 스레드를 breadcrumb 슬롯에 등록한다. 각 워커 스레드 진입부에서 한 번 호출.
pub fn register(thread: Thread) {
    SLOT.with(|s| s.set(thread as usize));
    mark(Step::Idle);
}

/// 현재 스레드의 진행 단계를 기록한다. 락도 할당도 없다.
pub fn mark(step: Step) {
    let ms = now_ms();
    SLOT.with(|s| {
        let i = s.get();
        if i < SLOTS {
            CRUMBS[i].store(pack(step, ms), Ordering::Relaxed);
        }
    });
}

// ── 먹통 판정 ───────────────────────────────────────────────────────────

#[derive(Debug, PartialEq, Eq)]
enum Action {
    Quiet,
    Report,
    Recovered { stalled_for_ms: u64 },
}

/// 먹통 진입/지속/복귀를 추적한다. 시간은 전부 인자로 받아 순수하게 판정한다.
struct StallTracker {
    /// 먹통이 시작된 것으로 보이는 시각(= 마지막 ACK 시각).
    stalled_since_ms: Option<u64>,
    last_report_ms: u64,
}

impl StallTracker {
    fn new() -> Self {
        Self {
            stalled_since_ms: None,
            last_report_ms: 0,
        }
    }

    fn is_stalled(&self) -> bool {
        self.stalled_since_ms.is_some()
    }

    /// `stalled_ms` 는 마지막 ACK 이후 흐른 시간.
    fn step(&mut self, stalled_ms: u64, now: u64, threshold_ms: u64, report_ms: u64) -> Action {
        if stalled_ms >= threshold_ms {
            if self.stalled_since_ms.is_none() {
                self.stalled_since_ms = Some(now.saturating_sub(stalled_ms));
                self.last_report_ms = now;
                return Action::Report;
            }
            if now.saturating_sub(self.last_report_ms) >= report_ms {
                self.last_report_ms = now;
                return Action::Report;
            }
            Action::Quiet
        } else if let Some(since) = self.stalled_since_ms.take() {
            Action::Recovered {
                stalled_for_ms: now.saturating_sub(since),
            }
        } else {
            Action::Quiet
        }
    }
}

/// 감시 스레드 자신이 예정보다 훨씬 오래 잤다면 시스템이 절전에 들어갔던 것이다.
/// 그동안은 메인 스레드도 멈춰 있었으므로 먹통으로 판정하면 안 된다.
fn is_resume_from_sleep(intended_ms: u64, actual_ms: u64) -> bool {
    actual_ms > intended_ms + SLEEP_SLACK.as_millis() as u64
}

// ── 복구 ────────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Recovery {
    /// 아직 개입하지 않고 한 번 더 지켜본다.
    Wait,
    /// 웹뷰 생성 중첩 루프에 갇힌 게 확실하다. 그 루프만 깨운다.
    Unstick,
    /// 깨워도 안 풀리거나, 갇힌 자리가 웹뷰 생성이 아니다. 프로세스를 다시 띄운다.
    Restart,
}

/// 메인 스레드가 지금 웹뷰를 만드는 중인가. 이때만 `WM_QUIT` 이 안전하다.
///
/// 중첩 루프 안이면 그 루프의 `GetMessage` 가 메시지를 집어 0 을 돌려주고, wry 는
/// 생성 실패로 빠져나온다. 반대로 메인 스레드가 단순히 오래 걸리는 중이면 같은
/// 메시지를 tao 이벤트 루프가 받아서 앱이 통째로 종료된다. 그래서 자리를 본다.
fn is_inside_webview_build(step: Step) -> bool {
    matches!(step, Step::NotifyBuildWindow | Step::SetupBuildWindow)
}

/// 깨우기 시도 횟수를 기억해 재시작으로 올릴 시점을 정한다. 시각을 인자로 받아
/// 판정만 하므로 테스트에서 시계 없이 검증할 수 있다.
struct RecoveryState {
    unsticks: u32,
    last_unstick_ms: Option<u64>,
}

impl RecoveryState {
    fn new() -> Self {
        Self {
            unsticks: 0,
            last_unstick_ms: None,
        }
    }

    fn decide(&mut self, now: u64, main_step: Step, report_count: u32) -> Recovery {
        // 한동안 조용했으면 지난 시도는 잊는다.
        if let Some(at) = self.last_unstick_ms {
            if now.saturating_sub(at) >= UNSTICK_RESET.as_millis() as u64 {
                self.unsticks = 0;
                self.last_unstick_ms = None;
            }
        }

        if is_inside_webview_build(main_step) {
            if self.unsticks < MAX_UNSTICKS {
                self.unsticks += 1;
                self.last_unstick_ms = Some(now);
                return Recovery::Unstick;
            }
            return Recovery::Restart;
        }

        if report_count >= NON_BUILD_PATIENCE {
            Recovery::Restart
        } else {
            Recovery::Wait
        }
    }
}

/// 메인 스레드가 갇힌 중첩 메시지 루프를 깨운다.
///
/// `wait_with_pump` 은 `GetMessage` 가 0 을 돌려줄 때만 빠져나온다. 스레드 큐에
/// `WM_QUIT` 을 넣으면 그 조건이 만들어지고, wry 는 `TaskCanceled` 로 실패를
/// 반환한다. 창 하나가 실패할 뿐 프로세스는 살아남는다.
#[cfg(windows)]
fn unstick_main_thread() -> bool {
    use windows::Win32::Foundation::{LPARAM, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{PostThreadMessageW, WM_QUIT};

    let tid = MAIN_TID.load(Ordering::Relaxed);
    if tid == 0 {
        log::error!("[WATCHDOG] main thread id unknown, cannot unstick");
        return false;
    }
    let sent = unsafe { PostThreadMessageW(tid, WM_QUIT, WPARAM(0), LPARAM(0)) };
    match sent {
        Ok(()) => true,
        Err(e) => {
            log::error!("[WATCHDOG] PostThreadMessage(WM_QUIT) failed: {e}");
            false
        }
    }
}

#[cfg(not(windows))]
fn unstick_main_thread() -> bool {
    false
}

/// 메인 스레드 도움 없이 프로세스를 다시 띄운다.
///
/// Tauri 의 `restart()` 는 종료 절차가 메인 스레드를 거치므로 먹통일 때 쓸 수 없다.
/// 여기서는 감시 스레드가 직접 새 프로세스를 띄우고 자기 프로세스를 끝낸다.
fn hard_restart(app: &AppHandle, stalled_ms: u64) -> ! {
    if let Some(stats) = app.try_state::<crate::stats::StatsState>() {
        crate::stats::flush(&stats.inner().clone());
    }

    match std::env::current_exe() {
        Ok(exe) => {
            let spawned = std::process::Command::new(&exe)
                .env(RECOVERED_ENV, stalled_ms.to_string())
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn();
            match spawned {
                Ok(child) => log::error!(
                    "[WATCHDOG] restarting: spawned pid={} after {:.1}s stall",
                    child.id(),
                    stalled_ms as f64 / 1000.0
                ),
                Err(e) => log::error!("[WATCHDOG] restart failed to spawn: {e}"),
            }
        }
        Err(e) => log::error!("[WATCHDOG] restart failed, current_exe: {e}"),
    }

    // 새 인스턴스는 싱글턴 뮤텍스가 풀리기를 잠시 기다린다(main.rs). 여기서는
    // 정리 훅을 태우지 않고 즉시 끝낸다. 메인 스레드가 죽어 있어 정상 종료 경로는
    // 어차피 끝까지 가지 못한다.
    std::process::exit(0);
}

// ── 스냅샷 ──────────────────────────────────────────────────────────────

struct Snapshot {
    now_ms: u64,
    stalled: bool,
    stalled_ms: u64,
    /// (스레드 이름, 단계, 기록된 지 지난 시간 ms)
    crumbs: Vec<(&'static str, &'static str, u64)>,
    /// 메인 스레드가 멈춰 선 자리. 복구 방법을 고르는 근거라 문자열이 아닌 값으로 둔다.
    main_step: Step,
    /// `None` 이면 알림 매니저 뮤텍스를 잡지 못한 것. 그 자체가 단서다.
    live_toasts: Option<Vec<String>>,
}

impl Snapshot {
    fn capture(app: &AppHandle, stalled: bool, stalled_ms: u64, now: u64) -> Snapshot {
        let crumbs = (0..SLOTS)
            .map(|i| {
                let (step, at) = unpack(CRUMBS[i].load(Ordering::Relaxed));
                (SLOT_NAMES[i], step.as_str(), now.saturating_sub(at))
            })
            .collect();

        // 먹통의 원인이 이 뮤텍스일 수도 있으므로 절대 기다리지 않는다.
        let live_toasts = app
            .try_state::<NotificationManagerState>()
            .and_then(|state| state.inner().try_lock().ok().map(|m| m.live_ids()));

        let (main_step, _) = unpack(CRUMBS[Thread::Main as usize].load(Ordering::Relaxed));

        Snapshot {
            now_ms: now,
            stalled,
            stalled_ms,
            crumbs,
            main_step,
            live_toasts,
        }
    }

    fn report_lines(&self) -> Vec<String> {
        let mut out = vec![format!(
            "[WATCHDOG] main thread unresponsive for {:.1}s (health: {})",
            self.stalled_ms as f64 / 1000.0,
            health_path().display()
        )];
        for (thread, step, ms_ago) in &self.crumbs {
            out.push(format!(
                "[WATCHDOG]   {thread}: {step} ({:.1}s ago)",
                *ms_ago as f64 / 1000.0
            ));
        }
        out.push(match &self.live_toasts {
            Some(ids) if ids.is_empty() => "[WATCHDOG]   live toasts: none".to_string(),
            Some(ids) => format!(
                "[WATCHDOG]   live toasts: {} [{}]",
                ids.len(),
                ids.join(", ")
            ),
            None => "[WATCHDOG]   live toasts: unavailable (manager lock held)".to_string(),
        });
        out
    }
}

/// 로거와 무관한 두 번째 증거 경로. 로거는 내부 뮤텍스가 poison 되면 통째로 죽고,
/// 로그 파일은 1MB 로테이션에 잘릴 수 있어서 최초 진입 기록을 잃을 수 있다.
fn health_path() -> PathBuf {
    std::env::temp_dir().join("agent-toast-health.json")
}

fn write_health_file(snapshot: &Snapshot) {
    let crumbs: serde_json::Map<String, serde_json::Value> = snapshot
        .crumbs
        .iter()
        .map(|(thread, step, ms_ago)| {
            (
                (*thread).to_string(),
                serde_json::json!({ "step": step, "ms_ago": ms_ago }),
            )
        })
        .collect();

    let doc = serde_json::json!({
        "pid": std::process::id(),
        "written_at": chrono::Local::now().to_rfc3339(),
        "uptime_ms": snapshot.now_ms,
        "state": if snapshot.stalled { "STALLED" } else { "OK" },
        "main_thread_stalled_ms": snapshot.stalled_ms,
        "breadcrumbs": crumbs,
        "live_toasts": snapshot.live_toasts,
    });

    let Ok(bytes) = serde_json::to_vec_pretty(&doc) else {
        return;
    };
    // 부분 기록된 파일을 읽는 일이 없도록 임시 파일에 쓴 뒤 교체한다.
    let path = health_path();
    let tmp = path.with_extension("json.tmp");
    if std::fs::write(&tmp, bytes).is_ok() {
        let _ = std::fs::rename(&tmp, &path);
    }
}

// ── 구동 ────────────────────────────────────────────────────────────────

/// 워치독 재시작으로 되살아난 인스턴스라면 그 사실을 한 줄로 알린다.
///
/// 조용히 되살아나면 설정 창이 사라지고 토스트가 끊긴 이유를 알 길이 없다. 반대로
/// 중첩 루프만 깨워서 복구한 경우는 사용자가 겪는 변화가 없으므로 알리지 않는다.
pub fn notify_if_recovered(app: &AppHandle, state: &NotificationManagerState) {
    let Ok(raw) = std::env::var(RECOVERED_ENV) else {
        return;
    };
    let stalled_secs = raw.parse::<u64>().unwrap_or(0) as f64 / 1000.0;
    log::warn!("[WATCHDOG] this instance was restarted after a {stalled_secs:.0}s stall");

    let app = app.clone();
    let state = state.clone();
    std::thread::spawn(move || {
        // 창과 트레이가 자리를 잡은 뒤에 띄운다.
        std::thread::sleep(Duration::from_secs(2));
        let message = match crate::setup::read_locale().as_str() {
            "en" => format!(
                "Agent Toast froze for {stalled_secs:.0}s and restarted itself. Notifications are working again."
            ),
            _ => format!(
                "알림이 {stalled_secs:.0}초간 멈춰 자동으로 재시작했습니다. 지금은 정상입니다."
            ),
        };
        let req = crate::cli::NotifyRequest {
            pid: 0,
            event: "error".to_string(),
            message: Some(message),
            title_hint: Some("Agent Toast".to_string()),
            alt_title_hint: None,
            process_tree: Some(vec![]),
            source: "watchdog".into(),
            hostname: None,
            orca_terminal_handle: None,
            orca_tab_id: None,
        };
        crate::notification::show_notification(&app, &state, req);
    });
}

/// 감시를 시작한다. Tauri `setup` 에서 한 번 호출.
pub fn start(app: &AppHandle) {
    Lazy::force(&EPOCH);
    LAST_ACK_MS.store(now_ms(), Ordering::Relaxed);

    // 메인 스레드도 슬롯을 갖게 해서, 창 생성처럼 메인 스레드에서 도는 구간이
    // breadcrumb 에 남도록 한다. 스레드 id 도 여기서 확보한다. 나중에는 메인
    // 스레드가 먹통이라 물어볼 방법이 없다.
    let _ = app.run_on_main_thread(|| {
        register(Thread::Main);
        #[cfg(windows)]
        MAIN_TID.store(
            unsafe { windows::Win32::System::Threading::GetCurrentThreadId() },
            Ordering::Relaxed,
        );
    });

    let app = app.clone();
    if let Err(e) = std::thread::Builder::new()
        .name("agent-toast-watchdog".into())
        .spawn(move || run(app))
    {
        log::error!("[WATCHDOG] failed to spawn: {e}");
    }
}

fn run(app: AppHandle) {
    log::info!(
        "[WATCHDOG] started (heartbeat {}s, stall threshold {}s, health: {})",
        HEARTBEAT.as_secs(),
        STALL_THRESHOLD.as_secs(),
        health_path().display()
    );

    let threshold_ms = STALL_THRESHOLD.as_millis() as u64;
    let report_ms = REPORT_INTERVAL.as_millis() as u64;
    let alive_ms = ALIVE_LOG_INTERVAL.as_millis() as u64;
    let health_ms = HEALTH_WRITE_INTERVAL.as_millis() as u64;

    let mut tracker = StallTracker::new();
    let mut recovery = RecoveryState::new();
    let mut report_count: u32 = 0;
    let mut last_alive_log = now_ms();
    // None = 아직 한 번도 안 씀. 첫 사이클에 바로 써서 파일 존재 자체가 감시 동작의
    // 증거가 되게 한다.
    let mut last_health_write: Option<u64> = None;

    loop {
        // 절전 감지는 두 시계를 함께 본다. `Instant` 는 Windows 에서 QPC 인데 절전
        // 구간을 시간에 포함하는지가 하드웨어/전원 상태에 따라 갈린다. 절전을 확실히
        // 반영하는 벽시계와 비교해 더 큰 쪽을 취하면 어느 환경에서도 잡힌다.
        let wall_before = SystemTime::now();
        let before = Instant::now();
        std::thread::sleep(HEARTBEAT);
        let wall_slept = SystemTime::now()
            .duration_since(wall_before)
            // 시계가 뒤로 갔으면(수동 변경/NTP 역보정) 정상 수면으로 취급해 오탐을 막는다.
            .unwrap_or(HEARTBEAT);
        let slept = before.elapsed().max(wall_slept);

        if is_resume_from_sleep(HEARTBEAT.as_millis() as u64, slept.as_millis() as u64) {
            let now = now_ms();
            log::info!(
                "[WATCHDOG] watchdog thread slept {:.0}s (system suspend?), skipping this cycle",
                slept.as_secs_f64()
            );
            LAST_ACK_MS.store(now, Ordering::Relaxed);
            tracker = StallTracker::new();
            last_alive_log = now;
            continue;
        }

        // 먹통이면 이 태스크는 tao 이벤트 버퍼에 들어간 채 실행되지 않는다.
        let _ = app.run_on_main_thread(|| {
            LAST_ACK_MS.store(now_ms(), Ordering::Relaxed);
        });
        // 응답이 도착할 여유를 준 뒤에 측정한다. 건강하면 여기서 이미 갱신돼 있다.
        std::thread::sleep(ACK_GRACE);

        let now = now_ms();
        let stalled_ms = now.saturating_sub(LAST_ACK_MS.load(Ordering::Relaxed));
        let action = tracker.step(stalled_ms, now, threshold_ms, report_ms);
        let snapshot = Snapshot::capture(&app, tracker.is_stalled(), stalled_ms, now);

        // 먹통 중이거나 방금 상태가 바뀐 순간은 헬스 파일을 즉시 갱신한다.
        let mut write_health_now = snapshot.stalled;

        match action {
            Action::Report => {
                for line in snapshot.report_lines() {
                    log::error!("{line}");
                }
                write_health_now = true;
                report_count += 1;

                // 증거를 먼저 남기고 개입한다. 재시작하면 이 프로세스는 사라진다.
                write_health_file(&snapshot);
                last_health_write = Some(now);

                let main_step = snapshot.main_step;
                match recovery.decide(now, main_step, report_count) {
                    Recovery::Wait => {
                        log::warn!(
                            "[WATCHDOG] stalled at '{}' (not a webview build), watching one more cycle",
                            main_step.as_str()
                        );
                    }
                    Recovery::Unstick => {
                        log::error!(
                            "[WATCHDOG] stalled inside webview build, waking the nested loop (attempt {})",
                            recovery.unsticks
                        );
                        if !unstick_main_thread() {
                            log::error!("[WATCHDOG] unstick failed, restarting instead");
                            hard_restart(&app, stalled_ms);
                        }
                    }
                    Recovery::Restart => {
                        log::error!(
                            "[WATCHDOG] unrecoverable (main at '{}'), restarting",
                            main_step.as_str()
                        );
                        hard_restart(&app, stalled_ms);
                    }
                }
            }
            Action::Recovered { stalled_for_ms } => {
                log::warn!(
                    "[WATCHDOG] main thread recovered (was unresponsive for {:.1}s)",
                    stalled_for_ms as f64 / 1000.0
                );
                report_count = 0;
                last_alive_log = now;
                write_health_now = true;
            }
            Action::Quiet => {
                if now.saturating_sub(last_alive_log) >= alive_ms {
                    last_alive_log = now;
                    log::debug!("[WATCHDOG] alive, main thread ok ({stalled_ms}ms since last ack)");
                }
            }
        }

        let health_due = match last_health_write {
            None => true,
            Some(at) => now.saturating_sub(at) >= health_ms,
        };
        if write_health_now || health_due {
            last_health_write = Some(now);
            write_health_file(&snapshot);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const THRESHOLD: u64 = 20_000;
    const REPORT: u64 = 60_000;

    // ── 복구 판정 ──

    #[test]
    fn unsticks_while_stalled_inside_webview_build() {
        let mut r = RecoveryState::new();
        assert_eq!(
            r.decide(1_000, Step::NotifyBuildWindow, 1),
            Recovery::Unstick
        );
        assert_eq!(r.unsticks, 1);
    }

    #[test]
    fn setup_window_build_also_counts_as_webview_build() {
        let mut r = RecoveryState::new();
        assert_eq!(
            r.decide(1_000, Step::SetupBuildWindow, 1),
            Recovery::Unstick
        );
    }

    #[test]
    fn escalates_to_restart_after_max_unsticks() {
        let mut r = RecoveryState::new();
        for i in 0..MAX_UNSTICKS {
            assert_eq!(
                r.decide(1_000 + i as u64, Step::NotifyBuildWindow, 1),
                Recovery::Unstick,
                "attempt {i} should still unstick"
            );
        }
        assert_eq!(
            r.decide(2_000, Step::NotifyBuildWindow, 1),
            Recovery::Restart
        );
    }

    #[test]
    fn forgets_attempts_after_a_quiet_period() {
        let mut r = RecoveryState::new();
        for _ in 0..MAX_UNSTICKS {
            r.decide(1_000, Step::NotifyBuildWindow, 1);
        }
        let later = 1_000 + UNSTICK_RESET.as_millis() as u64;
        assert_eq!(
            r.decide(later, Step::NotifyBuildWindow, 1),
            Recovery::Unstick,
            "a long quiet stretch should reset the escalation"
        );
        assert_eq!(r.unsticks, 1);
    }

    #[test]
    fn waits_once_then_restarts_when_not_inside_a_build() {
        let mut r = RecoveryState::new();
        // 폰트 열거처럼 메인 스레드가 그냥 오래 걸리는 자리에는 WM_QUIT 을 쏘면 안 된다.
        assert_eq!(r.decide(1_000, Step::NotifyCalcPosition, 1), Recovery::Wait);
        assert_eq!(
            r.decide(2_000, Step::NotifyCalcPosition, 2),
            Recovery::Restart
        );
        assert_eq!(r.unsticks, 0, "no unstick should have been attempted");
    }

    #[test]
    fn idle_main_thread_is_not_treated_as_a_build() {
        assert!(!is_inside_webview_build(Step::Idle));
        assert!(is_inside_webview_build(Step::NotifyBuildWindow));
    }

    // ── pack / unpack ──

    #[test]
    fn pack_roundtrip_preserves_step_and_time() {
        let (step, ms) = unpack(pack(Step::NotifyCalcPosition, 1_234_567));
        assert_eq!(step, Step::NotifyCalcPosition);
        assert_eq!(ms, 1_234_567);
    }

    #[test]
    fn pack_zero_is_idle_at_zero() {
        // 초기화되지 않은 슬롯(0)은 "부팅 직후 idle" 로 읽혀야 한다.
        let (step, ms) = unpack(0);
        assert_eq!(step, Step::Idle);
        assert_eq!(ms, 0);
    }

    #[test]
    fn pack_survives_large_timestamps() {
        // 56비트 경계 근처에서도 단계가 시각에 침범당하지 않는다.
        let big = MS_MASK;
        let (step, ms) = unpack(pack(Step::SetupBuildWindow, big));
        assert_eq!(step, Step::SetupBuildWindow);
        assert_eq!(ms, big);
    }

    // ── 절전 복귀 ──

    #[test]
    fn normal_sleep_is_not_resume() {
        assert!(!is_resume_from_sleep(5_000, 5_030));
    }

    #[test]
    fn oversleeping_past_slack_is_resume() {
        assert!(is_resume_from_sleep(5_000, 1_800_000));
    }

    #[test]
    fn slack_boundary_is_not_resume() {
        // 정확히 여유분만큼 늦은 것은 절전으로 보지 않는다 (스케줄러 지연 허용).
        assert!(!is_resume_from_sleep(5_000, 15_000));
    }

    // ── 먹통 판정 ──

    #[test]
    fn below_threshold_stays_quiet() {
        let mut t = StallTracker::new();
        assert_eq!(t.step(19_999, 100_000, THRESHOLD, REPORT), Action::Quiet);
        assert!(!t.is_stalled());
    }

    #[test]
    fn crossing_threshold_reports_once() {
        let mut t = StallTracker::new();
        assert_eq!(t.step(20_000, 100_000, THRESHOLD, REPORT), Action::Report);
        assert!(t.is_stalled());
        // 아직 보고 주기가 안 됐으므로 조용해야 한다.
        assert_eq!(t.step(25_000, 105_000, THRESHOLD, REPORT), Action::Quiet);
    }

    #[test]
    fn continued_stall_reports_again_after_interval() {
        let mut t = StallTracker::new();
        t.step(20_000, 100_000, THRESHOLD, REPORT);
        assert_eq!(t.step(75_000, 155_000, THRESHOLD, REPORT), Action::Quiet);
        assert_eq!(t.step(80_000, 160_000, THRESHOLD, REPORT), Action::Report);
    }

    #[test]
    fn recovery_reports_total_stall_duration() {
        let mut t = StallTracker::new();
        // 100초 시점에 20초째 무응답 → 먹통 시작은 80초 시점.
        t.step(20_000, 100_000, THRESHOLD, REPORT);
        // 140초 시점에 응답이 돌아왔다면 총 60초간 먹통이었다.
        assert_eq!(
            t.step(30, 140_000, THRESHOLD, REPORT),
            Action::Recovered {
                stalled_for_ms: 60_000
            }
        );
        assert!(!t.is_stalled());
    }

    #[test]
    fn recovery_only_fires_once() {
        let mut t = StallTracker::new();
        t.step(20_000, 100_000, THRESHOLD, REPORT);
        t.step(30, 140_000, THRESHOLD, REPORT);
        assert_eq!(t.step(30, 145_000, THRESHOLD, REPORT), Action::Quiet);
    }
}
