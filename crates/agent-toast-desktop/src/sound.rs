//! 알림 소리 재생. 커스텀 파일(설정)이 있으면 WinRT MediaPlayer로,
//! 없으면 시스템 알림음(PlaySoundW)으로 재생한다.

/// 재생 종료 콜백 (미리듣기 토글 버튼 복귀용)
#[cfg(windows)]
type EndedCallback = std::sync::Arc<dyn Fn() + Send + Sync>;

/// 설정을 읽어 알림 소리 재생 (알림 표시 경로에서 호출)
#[cfg(windows)]
pub fn play_notification_sound() {
    play_path_or_default(crate::setup::load_notification_sound_file().as_deref());
}

/// 주어진 경로(없으면 시스템 기본음)를 재생. 실제 알림 경로의 진입점.
#[cfg(windows)]
pub fn play_path_or_default(path: Option<&str>) {
    match path {
        Some(p) if !p.is_empty() => play_custom(p.to_string(), None),
        _ => play_default(),
    }
}

/// 미리듣기 재생. 재생이 끝까지 가면 on_ended 호출 (stop_playback으로 끊기면 호출 안 됨).
#[cfg(windows)]
pub fn preview(path: Option<&str>, on_ended: impl Fn() + Send + Sync + 'static) {
    match path {
        Some(p) if !p.is_empty() => play_custom(p.to_string(), Some(std::sync::Arc::new(on_ended))),
        _ => {
            // 시스템 기본음은 종료 이벤트가 없으므로 동기 재생 후 콜백
            std::thread::spawn(move || {
                play_default_sync();
                on_ended();
            });
        }
    }
}

/// 재생 중인 소리 정지 (미리듣기 정지 버튼, 설정 창 닫힘 시 호출)
#[cfg(windows)]
pub fn stop_playback() {
    use windows::core::PCWSTR;
    use windows::Win32::Media::Audio::{PlaySoundW, SND_FLAGS};
    // MediaPlayer는 drop되면 재생이 끊긴다
    *PLAYER.lock().unwrap() = None;
    // PlaySoundW 비동기 재생도 취소 (pszSound=null)
    unsafe {
        let _ = PlaySoundW(PCWSTR::null(), None, SND_FLAGS(0));
    }
}

#[cfg(windows)]
fn play_default() {
    use windows::core::w;
    use windows::Win32::Media::Audio::{PlaySoundW, SND_ALIAS, SND_ASYNC};
    unsafe {
        let _ = PlaySoundW(w!("SystemNotification"), None, SND_ALIAS | SND_ASYNC);
    }
}

/// 기본음 동기 재생 (미리듣기 전용: 재생이 끝나야 리턴해 종료 콜백 시점을 알 수 있다)
#[cfg(windows)]
fn play_default_sync() {
    use windows::core::w;
    use windows::Win32::Media::Audio::{PlaySoundW, SND_ALIAS};
    unsafe {
        let _ = PlaySoundW(w!("SystemNotification"), None, SND_ALIAS);
    }
}

/// 커스텀 파일 재생. 파일이 없거나 재생 준비에 실패하면 시스템 기본음으로 폴백.
#[cfg(windows)]
fn play_custom(path: String, on_ended: Option<EndedCallback>) {
    // 호출 스레드(파이프 서버 등)는 WinRT 아파트먼트가 없을 수 있으므로 전용 스레드에서 재생
    std::thread::spawn(move || {
        if !std::path::Path::new(&path).exists() {
            log::warn!("[SOUND] 커스텀 사운드 없음, 기본음 폴백: {path}");
            fallback_default(on_ended.as_ref());
            return;
        }
        if let Err(e) = try_play_media(&path, on_ended.as_ref()) {
            log::warn!("[SOUND] 커스텀 사운드 재생 실패({e}), 기본음 폴백: {path}");
            fallback_default(on_ended.as_ref());
        }
    });
}

/// 폴백 기본음: 미리듣기(콜백 있음)면 동기 재생 후 종료 콜백, 알림 경로면 비동기
#[cfg(windows)]
fn fallback_default(on_ended: Option<&EndedCallback>) {
    match on_ended {
        Some(f) => {
            play_default_sync();
            f();
        }
        None => play_default(),
    }
}

/// 재생 중인 MediaPlayer 보관소. drop되면 재생이 끊기므로 함수 리턴 후에도 유지하고,
/// 새 재생이 오면 교체해 이전 소리를 멈춘다 (중첩 재생 방지).
#[cfg(windows)]
static PLAYER: once_cell::sync::Lazy<
    std::sync::Mutex<Option<windows::Media::Playback::MediaPlayer>>,
> = once_cell::sync::Lazy::new(|| std::sync::Mutex::new(None));

#[cfg(windows)]
fn try_play_media(path: &str, on_ended: Option<&EndedCallback>) -> Result<(), String> {
    use windows::core::HSTRING;
    use windows::Foundation::{TypedEventHandler, Uri};
    use windows::Media::Core::MediaSource;
    use windows::Media::Playback::MediaPlayer;
    use windows::Win32::System::WinRT::{RoInitialize, RO_INIT_MULTITHREADED};

    // 이 스레드에 WinRT 아파트먼트 보장 (이미 초기화된 경우의 오류는 무시)
    unsafe {
        let _ = RoInitialize(RO_INIT_MULTITHREADED);
    }

    let url = url::Url::from_file_path(path).map_err(|_| format!("잘못된 경로: {path}"))?;
    let uri = Uri::CreateUri(&HSTRING::from(url.as_str())).map_err(|e| e.to_string())?;
    let source = MediaSource::CreateFromUri(&uri).map_err(|e| e.to_string())?;
    let player = MediaPlayer::new().map_err(|e| e.to_string())?;
    player.SetSource(&source).map_err(|e| e.to_string())?;
    if let Some(f) = on_ended {
        let f = f.clone();
        player
            .MediaEnded(&TypedEventHandler::new(move |_, _| {
                f();
                Ok(())
            }))
            .map_err(|e| e.to_string())?;
    }
    player.Play().map_err(|e| e.to_string())?;
    *PLAYER.lock().unwrap() = Some(player);
    Ok(())
}

#[cfg(not(windows))]
pub fn play_notification_sound() {}

#[cfg(not(windows))]
pub fn play_path_or_default(_path: Option<&str>) {}

#[cfg(not(windows))]
pub fn preview(_path: Option<&str>, _on_ended: impl Fn() + Send + Sync + 'static) {}

#[cfg(not(windows))]
pub fn stop_playback() {}
