#[cfg(windows)]
pub fn play_notification_sound() {
    use windows::core::w;
    use windows::Win32::Media::Audio::{PlaySoundW, SND_ALIAS, SND_ASYNC};
    unsafe {
        let _ = PlaySoundW(w!("SystemNotification"), None, SND_ALIAS | SND_ASYNC);
    }
}

#[cfg(not(windows))]
pub fn play_notification_sound() {}
