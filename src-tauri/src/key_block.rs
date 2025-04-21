use core_foundation::runloop::{
    kCFRunLoopCommonModes, CFRunLoopAddSource, CFRunLoopGetCurrent, CFRunLoopRun, CFRunLoopStop,
    CFRunLoopWakeUp,
};
use std::sync::atomic::{AtomicPtr, Ordering};
use std::{
    os::raw::c_void,
    ptr,
    sync::{Mutex, OnceLock},
};
use tauri::{AppHandle, Emitter};
use tracing::debug;
use anyhow::{anyhow, Ok};

#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    pub fn CGEventTapCreate(
        tap: u32,
        place: u32,
        options: u32,
        eventsOfInterest: u64,
        callback: extern "C" fn(
            proxy: *mut c_void,
            type_: u32,
            event: *mut c_void,
            refcon: *mut c_void,
        ) -> *mut c_void,
        userdebug: *mut c_void,
    ) -> *mut c_void;

    pub fn CFMachPortCreateRunLoopSource(
        allocator: *const c_void,
        port: *mut c_void,
        order: i32,
    ) -> *mut c_void;

    pub fn CGEventCreateKeyboardEvent(
        source: *const c_void,
        virtualKey: u16,
        keyDown: bool,
    ) -> *mut c_void;
    pub fn CGEventPost(tap: u32, event: *mut c_void);

    pub fn AXIsProcessTrusted() -> bool;
}

const K_CG_HID_EVENT_TAP: u32 = 0;
const K_CG_HEAD_INSERT_EVENT_TAP: u32 = 0;
const K_CG_EVENT_TAP_OPTION_DEFAULT: u32 = 0;
const K_CG_EVENT_KEY_DOWN: u32 = 10;
const K_CG_EVENT_KEY_UP: u32 = 11;

fn cg_event_mask_bit(event_type: u32) -> u64 {
    1u64 << event_type
}

extern "C" fn keyboard_callback(
    _proxy: *mut c_void,
    _type: u32,
    _event: *mut c_void,
    _user_data: *mut c_void,
) -> *mut c_void {
    debug!("키보드 이벤트가 발생했습니다.");
    ptr::null_mut()
}

fn is_accessibility_trusted() -> bool {
    unsafe { AXIsProcessTrusted() }
}

static HANDLE: OnceLock<Mutex<Option<std::thread::JoinHandle<()>>>> = OnceLock::new();
static RUNLOOP: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

pub fn start() -> anyhow::Result<()> {
    debug!("start");
    if !is_accessibility_trusted() {
        debug!("접근성 권한이 없습니다. start()를 중단합니다.");

        // macOS 접근성 설정 페이지 자동으로 열기
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            let _ = Command::new("open")
                .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
                .spawn();
        }

        return Err(anyhow!("Accessibility permission not granted"));
    }

    let event_mask = cg_event_mask_bit(K_CG_EVENT_KEY_DOWN) | cg_event_mask_bit(K_CG_EVENT_KEY_UP);

    let thread = std::thread::spawn(move || unsafe {
        let event_tap = CGEventTapCreate(
            K_CG_HID_EVENT_TAP,
            K_CG_HEAD_INSERT_EVENT_TAP,
            K_CG_EVENT_TAP_OPTION_DEFAULT,
            event_mask,
            keyboard_callback,
            ptr::null_mut(),
        );
        assert!(!event_tap.is_null(), "Failed to create event tap");

        let runloop_source = CFMachPortCreateRunLoopSource(ptr::null(), event_tap, 0);
        assert!(
            !runloop_source.is_null(),
            "Failed to create run loop source"
        );

        let runloop = CFRunLoopGetCurrent();
        RUNLOOP.store(runloop as *mut c_void, Ordering::SeqCst);

        CFRunLoopAddSource(runloop, runloop_source as *mut _, kCFRunLoopCommonModes);
        CFRunLoopRun();
    });

    HANDLE.get_or_init(|| Mutex::new(None));
    let mut handle_guard = HANDLE.get().unwrap().lock().unwrap();
    *handle_guard = Some(thread);

    Ok(())
}

pub fn stop() -> anyhow::Result<()> {
    debug!("stop");
    let runloop = RUNLOOP.load(Ordering::SeqCst);
    if !runloop.is_null() {
        unsafe {
            CFRunLoopStop(runloop as *mut _);
            CFRunLoopWakeUp(runloop as *mut _);

            // 더미 키 이벤트 생성 및 전송 (예: virtualKey 0, keyDown false)
            let dummy_event = CGEventCreateKeyboardEvent(ptr::null(), 0, false);
            if !dummy_event.is_null() {
                CGEventPost(0, dummy_event);
            }
            debug!("Dummy event posted");
        }
    }
    if let Some(handle_mutex) = HANDLE.get() {
        let mut handle_guard = handle_mutex.lock().unwrap();
        if let Some(thread) = handle_guard.take() {
            let _ = thread.join();
            debug!("Stopping thread");
        }
    }
    RUNLOOP.store(std::ptr::null_mut(), Ordering::SeqCst);

    Ok(())
}
