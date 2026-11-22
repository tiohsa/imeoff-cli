use std::process::ExitCode;

/// 引数から IME 状態を決定する。なし・`0`・その他は OFF(0)、`1` は ON(1)。
fn parse_status(arg: Option<&str>) -> isize {
    match arg {
        Some("1") => 1,
        _ => 0,
    }
}

#[cfg(windows)]
fn set_ime_open_status(status: isize) -> Result<(), ()> {
    use std::mem::{size_of, zeroed};
    use windows_sys::Win32::UI::Input::Ime::ImmGetDefaultIMEWnd;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GUITHREADINFO, GetForegroundWindow, GetGUIThreadInfo, GetWindowThreadProcessId,
        SendMessageW,
    };

    const WM_IME_CONTROL: u32 = 0x0283;
    const IMC_SETOPENSTATUS: usize = 0x0006;

    unsafe {
        let fg = GetForegroundWindow();
        if fg.is_null() {
            return Err(());
        }

        let mut target = fg;
        let tid = GetWindowThreadProcessId(fg, std::ptr::null_mut());
        if tid != 0 {
            let mut info: GUITHREADINFO = zeroed();
            info.cbSize = size_of::<GUITHREADINFO>() as u32;
            if GetGUIThreadInfo(tid, &mut info) != 0 && !info.hwndFocus.is_null() {
                target = info.hwndFocus;
            }
        }

        let ime = ImmGetDefaultIMEWnd(target);
        if ime.is_null() {
            return Err(());
        }
        SendMessageW(ime, WM_IME_CONTROL, IMC_SETOPENSTATUS, status);
    }
    Ok(())
}

#[cfg(not(windows))]
fn set_ime_open_status(_status: isize) -> Result<(), ()> {
    eprintln!("imeoff: Windows only");
    Err(())
}

fn main() -> ExitCode {
    let arg = std::env::args().nth(1);
    match set_ime_open_status(parse_status(arg.as_deref())) {
        Ok(()) => ExitCode::SUCCESS,
        Err(()) => ExitCode::from(1),
    }
}

#[cfg(test)]
mod tests {
    use super::parse_status;

    #[test]
    fn status_parsing() {
        assert_eq!(parse_status(None), 0);
        assert_eq!(parse_status(Some("0")), 0);
        assert_eq!(parse_status(Some("1")), 1);
        assert_eq!(parse_status(Some("x")), 0);
    }
}
