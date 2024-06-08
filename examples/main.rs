fn main() {
    unsafe {
        backtrace_on_stack_overflow::enable_with_limit(20);
    }

    f(1)
}

fn f(x: u64) {
    if x == 0 {
        return;
    }
    f(x)
}
