use std::fs;

// ============================================
// ANTI-DEBUGGING & HARDENING
// ============================================

#[cfg(target_os = "linux")]
pub fn detect_debugger() -> bool {
  // Check TracerPid in /proc/self/status
  if let Ok(status) = fs::read_to_string("/proc/self/status") {
    for line in status.lines() {
      if line.starts_with("TracerPid:") {
          let pid: i32 = line.split_whitespace()
            .nth(1)
            .unwrap_or("0")
            .parse()
            .unwrap_or(0);
          return pid != 0;
        }
      }
  }
  false
}

#[cfg(target_os = "linux")]
pub fn check_ptrace() {
  // If we can't ptrace ourselves, another debugger is attached
  unsafe {
    if libc::ptrace(libc::PTRACE_TRACEME, 0, 0, 0) == -1 {
      eprintln!("Debugger detected!");
      std::process::exit(1);
    }
  }
}
