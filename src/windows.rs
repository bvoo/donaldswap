use anyhow::{Context, Result};
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
use windows::Win32::System::ProcessStatus::GetModuleFileNameExW;
use windows::Win32::System::Threading::{
    AttachThreadInput, GetCurrentThreadId, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{keybd_event, SetFocus, KEYEVENTF_KEYUP};
use windows::Win32::UI::WindowsAndMessaging::{
    BringWindowToTop, EnumWindows, GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW,
    GetWindowThreadProcessId, IsWindowVisible, SetForegroundWindow, ShowWindow, SW_RESTORE,
};

#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub hwnd: isize,
    pub title: String,
    pub exe_name: String,
}

pub fn enumerate_windows() -> Result<Vec<WindowInfo>> {
    let mut windows: Vec<WindowInfo> = Vec::new();

    unsafe {
        EnumWindows(
            Some(enum_windows_callback),
            LPARAM(&mut windows as *mut Vec<WindowInfo> as isize),
        )
        .context("EnumWindows failed")?;
    }

    Ok(windows)
}

unsafe extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let windows = &mut *(lparam.0 as *mut Vec<WindowInfo>);

    if !IsWindowVisible(hwnd).as_bool() {
        return BOOL(1);
    }

    let title = get_window_title(hwnd);
    if title.is_empty() {
        return BOOL(1);
    }

    let exe_name = get_window_exe(hwnd);
    if exe_name.is_empty() {
        return BOOL(1);
    }

    windows.push(WindowInfo {
        hwnd: hwnd.0,
        title,
        exe_name,
    });

    BOOL(1)
}

unsafe fn get_window_title(hwnd: HWND) -> String {
    let length = GetWindowTextLengthW(hwnd);
    if length == 0 {
        return String::new();
    }

    let mut buffer = vec![0u16; (length + 1) as usize];
    GetWindowTextW(hwnd, &mut buffer);

    let len = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
    if len > 0 {
        String::from_utf16_lossy(&buffer[..len])
    } else {
        String::new()
    }
}

unsafe fn get_window_exe(hwnd: HWND) -> String {
    let mut process_id: u32 = 0;
    GetWindowThreadProcessId(hwnd, Some(&mut process_id));

    if process_id == 0 {
        return String::new();
    }

    let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id);

    let Ok(process) = process else {
        return String::new();
    };

    let mut buffer = [0u16; 260];
    let result = GetModuleFileNameExW(process, None, &mut buffer);

    if result == 0 {
        return String::new();
    }

    let exe_path = OsString::from_wide(&buffer[..result as usize]);
    let exe_path = exe_path.to_string_lossy();

    exe_path
        .rsplit('\\')
        .next()
        .map(|s| s.to_string())
        .unwrap_or_default()
}

pub fn find_window_by_exe(exe_name: &str) -> Option<isize> {
    let windows = enumerate_windows().ok()?;
    windows
        .iter()
        .find(|w| w.exe_name.eq_ignore_ascii_case(exe_name))
        .map(|w| w.hwnd)
}

pub fn focus_window(hwnd: isize) -> Result<()> {
    const MAX_RETRIES: u32 = 5;

    for attempt in 0..MAX_RETRIES {
        if focus_window_once(hwnd)? {
            return Ok(());
        }

        if attempt < MAX_RETRIES - 1 {
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    }

    anyhow::bail!(
        "Failed to set foreground window after {} attempts",
        MAX_RETRIES
    )
}

fn focus_window_once(hwnd: isize) -> Result<bool> {
    unsafe {
        let hwnd = HWND(hwnd);

        // Simulate an ALT keypress to bypass Windows foreground lock
        // Virtual-Key code for ALT is VK_MENU (0x12)
        keybd_event(0x12, 0, Default::default(), 0);
        keybd_event(0x12, 0, KEYEVENTF_KEYUP, 0);

        ShowWindow(hwnd, SW_RESTORE);

        let foreground_hwnd = GetForegroundWindow();
        let current_thread_id = GetCurrentThreadId();
        let mut foreground_thread_id: u32 = 0;
        let mut attached = false;

        if foreground_hwnd.0 != 0 && foreground_hwnd != hwnd {
            GetWindowThreadProcessId(foreground_hwnd, Some(&mut foreground_thread_id));

            if foreground_thread_id != current_thread_id {
                AttachThreadInput(current_thread_id, foreground_thread_id, true);
                attached = true;
            }
        }

        SetForegroundWindow(hwnd);
        let _ = BringWindowToTop(hwnd);
        SetFocus(hwnd);

        if attached {
            AttachThreadInput(current_thread_id, foreground_thread_id, false);
        }

        let new_foreground = GetForegroundWindow();
        Ok(new_foreground == hwnd)
    }
}
