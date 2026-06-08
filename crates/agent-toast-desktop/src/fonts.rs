//! 시스템 설치 폰트 열거 (Windows GDI). 알림 폰트 커스터마이즈용.

#[cfg(windows)]
use windows::Win32::Foundation::LPARAM;
#[cfg(windows)]
use windows::Win32::Graphics::Gdi::{
    EnumFontFamiliesExW, GetDC, ReleaseDC, DEFAULT_CHARSET, LOGFONTW, TEXTMETRICW,
};

/// EnumFontFamiliesExW 콜백 — 각 패밀리명을 lparam이 가리키는 Vec<String>에 수집.
#[cfg(windows)]
unsafe extern "system" fn enum_font_proc(
    lf: *const LOGFONTW,
    _tm: *const TEXTMETRICW,
    _font_type: u32,
    lparam: LPARAM,
) -> i32 {
    if lf.is_null() {
        return 1;
    }
    let face = &(*lf).lfFaceName;
    let len = face.iter().position(|&c| c == 0).unwrap_or(face.len());
    let name = String::from_utf16_lossy(&face[..len]);
    let names = &mut *(lparam.0 as *mut Vec<String>);
    names.push(name);
    1 // 계속 열거
}

/// 설치된 폰트 패밀리명을 모두 수집한다(원시, 중복·세로쓰기 포함).
#[cfg(windows)]
fn enumerate_installed_fonts() -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    unsafe {
        let hdc = GetDC(None);
        let lf = LOGFONTW {
            lfCharSet: DEFAULT_CHARSET,
            ..Default::default()
        };
        // lfFaceName 비움 = 모든 패밀리를 1회씩 열거
        EnumFontFamiliesExW(
            hdc,
            &lf,
            Some(enum_font_proc),
            LPARAM(&mut names as *mut Vec<String> as isize),
            0,
        );
        ReleaseDC(None, hdc);
    }
    names
}

#[cfg(not(windows))]
fn enumerate_installed_fonts() -> Vec<String> {
    Vec::new()
}

/// 알림 폰트 설정 드롭다운용 시스템 폰트 목록.
#[tauri::command]
pub fn list_system_fonts() -> Vec<String> {
    normalize_font_families(enumerate_installed_fonts())
}

/// 열거 원시 결과를 UI용으로 정리: 빈 문자열·세로쓰기(@) 폰트 제외,
/// 대소문자 무시 정렬 후 중복 제거.
fn normalize_font_families(mut names: Vec<String>) -> Vec<String> {
    names.retain(|n| !n.is_empty() && !n.starts_with('@'));
    names.sort_by_key(|a| a.to_lowercase());
    names.dedup();
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_filters_vertical_and_empty() {
        let input = vec![
            "Arial".to_string(),
            "@Malgun Gothic".to_string(),
            "".to_string(),
            "Consolas".to_string(),
        ];
        let out = normalize_font_families(input);
        assert_eq!(out, vec!["Arial".to_string(), "Consolas".to_string()]);
    }

    #[test]
    fn normalize_sorts_case_insensitive_and_dedups() {
        let input = vec![
            "consolas".to_string(),
            "Arial".to_string(),
            "consolas".to_string(),
            "Batang".to_string(),
        ];
        let out = normalize_font_families(input);
        assert_eq!(
            out,
            vec![
                "Arial".to_string(),
                "Batang".to_string(),
                "consolas".to_string()
            ]
        );
    }
}
