# backtrace-on-stack-overflow

By default, Rust aborts on stackoverflow without printing a backtrace:

```console
λ bat src/main.rs
fn main() {
    f(92)
}

fn f(x: u64) {
    f(x)
}
λ cargo run
    Finished dev [unoptimized + debuginfo] target(s) in 0.00s
     Running `target/debug/so`

thread 'main' has overflowed its stack
fatal runtime error: stack overflow
fish: Job 1, 'cargo run' terminated by signal SIGABRT (Abort)
```

This crate fixes this:

```console
λ bat src/main.rs
fn main() {
    unsafe { backtrace_on_stack_overflow::enable() };
    f(92)
}

fn f(x: u64) {
    f(x)
}
λ cargo run
    Finished dev [unoptimized + debuginfo] target(s) in 0.01s
     Running `target/debug/so`
Stack Overflow:
   0: backtrace_on_stack_overflow::handle_sigsegv
             at /home/matklad/p/backtrace-on-stack-overflow/src/lib.rs:33:40
   1: <unknown>
   2: so::f
             at src/main.rs:6
   3: so::f
             at src/main.rs:7:5
   4: so::f
             at src/main.rs:7:5
   5: so::f
             at src/main.rs:7:5
   6: so::f
             at src/main.rs:7:5
   7: so::f
             at src/main.rs:7:5
   8: so::f
             at src/main.rs:7:5
   9: so::f
             at src/main.rs:7:5
  10: so::f
             at src/main.rs:7:5
```

This crate works for debugging, but is unsuited for being enabled in production.
