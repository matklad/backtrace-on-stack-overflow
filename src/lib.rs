use nix::sys::signal;

/// Best effort printing of backtrace on stack overflow.
///
/// Works on my machine, may summon laundry-eating nasal daemons.
///
/// PRs to make this more robust are welcome
pub unsafe fn enable<T, F: FnOnce() -> T>(f: F) -> T {
    let buf = vec![0u128; 4096];
    let buf = Vec::leak(buf);
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
    f()
}

extern "C" fn handle_sigsegv(_: i32) {
    eprintln!("Stack Overflow:\n{:?}", backtrace::Backtrace::new());
    std::process::abort();
}
