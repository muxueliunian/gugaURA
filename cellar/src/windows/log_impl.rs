use windows::core::PCSTR;
use windows::Win32::System::Diagnostics::Debug::OutputDebugStringA;

pub fn init(filter_level: log::LevelFilter) {
    if let Some(level) = filter_level.to_level() {
        windebug_logger::init_with_level(level).ok();
    }
}

pub fn raw_debug_output(message: &str) {
    let mut bytes: Vec<u8> = message
        .as_bytes()
        .iter()
        .copied()
        .filter(|b| *b != 0)
        .collect();
    bytes.push(b'\n');
    bytes.push(0);
    unsafe {
        OutputDebugStringA(PCSTR(bytes.as_ptr()));
    }
}
