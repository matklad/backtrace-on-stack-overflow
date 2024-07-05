//! By default, Rust aborts on stackoverflow without printing a backtrace:
//!
//! ```console
//! λ bat src/main.rs
//! fn main() {
//!     f(92)
//! }
//!
//! fn f(x: u64) {
//!     f(x)
//! }
//! λ cargo run
//!     Finished dev [unoptimized + debuginfo] target(s) in 0.00s
//!      Running `target/debug/so`
//!
//! thread 'main' has overflowed its stack
//! fatal runtime error: stack overflow
//! fish: Job 1, 'cargo run' terminated by signal SIGABRT (Abort)
//! ```
//!
//! This crate fixes this:
//!
//! ```console
//! λ bat src/main.rs
//! fn main() {
//!     unsafe { backtrace_on_stack_overflow::enable_with_limit(10) };
//!     f(92)
//! }
//!
//! fn f(x: u64) {
//!     f(x)
//! }
//! λ cargo run
//!     Finished dev [unoptimized + debuginfo] target(s) in 0.01s
//!      Running `target/debug/so`
//! Stack Overflow:
//!    0: backtrace_on_stack_overflow::handle_sigsegv
//!              at /home/matklad/p/backtrace-on-stack-overflow/src/lib.rs:33:40
//!    1: <unknown>
//!    2: so::f
//!              at src/main.rs:6
//!    3: so::f
//!              at src/main.rs:7:5
//!    4: so::f
//!              at src/main.rs:7:5
//!    5: so::f
//!              at src/main.rs:7:5
//!    6: so::f
//!              at src/main.rs:7:5
//!    7: so::f
//!              at src/main.rs:7:5
//!    8: so::f
//!              at src/main.rs:7:5
//!    9: so::f
//!              at src/main.rs:7:5
//!   10: so::f
//!              at src/main.rs:7:5
//!
//! ... and 261982 more
//! ```
//!
//! This crate works for debugging, but is unsuited for being enabled in production.
use std::sync::atomic::{AtomicBool, Ordering};

use nix::sys::signal;

use backtrace::{Backtrace, BacktraceFrame};

static mut COUNT: usize = 10;

/// Best effort printing of backtrace on stack overflow.
///
/// Works on my machine, may summon laundry-eating nasal daemons.
///
/// PRs to make this more robust are welcome
pub unsafe fn enable() {
    static ONCE: std::sync::Once = std::sync::Once::new();

    ONCE.call_once(|| {
        // Use u128 for alignment.
        let buf = Vec::leak(vec![0u128; 4096]);
        let stack = libc::stack_t {
            ss_sp: buf.as_ptr() as *mut libc::c_void,
            ss_flags: 0,
            ss_size: buf.len() * std::mem::size_of::<u128>(),
        };
        let mut old = libc::stack_t { ss_sp: std::ptr::null_mut(), ss_flags: 0, ss_size: 0 };
        let ret = libc::sigaltstack(&stack, &mut old);
        assert_eq!(ret, 0, "sigaltstack failed");

        let sig_action = signal::SigAction::new(
            signal::SigHandler::Handler(handle_sigsegv),
            signal::SaFlags::SA_NODEFER | signal::SaFlags::SA_ONSTACK,
            signal::SigSet::empty(),
        );
        signal::sigaction(signal::SIGSEGV, &sig_action).unwrap();
        signal::sigaction(signal::SIGABRT, &sig_action).unwrap();
    })
}

/// Like [`enable`], but only print the first `count` frames of the backtrace.
pub unsafe fn enable_with_limit(count: usize) {
    COUNT = count;
    enable();
}

extern "C" fn handle_sigsegv(_: i32) {
    // not using Once to avoid adding extra frames
    static HANDLED: AtomicBool = AtomicBool::new(false);
    if HANDLED.swap(true, Ordering::SeqCst) {
        return;
    }
    let backtrace: Vec<BacktraceFrame> = Backtrace::new().into();
    let count = unsafe { COUNT };
    let truncated = if backtrace.len() > count { backtrace.len() - count } else { 0 };
    let backtrace: Backtrace = backtrace.into_iter().take(count).collect::<Vec<_>>().into();
    eprintln!("Stack Overflow:\n{:?}", backtrace);
    if truncated > 0 {
        eprintln!("... and {} more", truncated);
    }
    std::process::exit(1);
}
