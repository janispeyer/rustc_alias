// compile-flags: -Zmir-emit-retag

fn read() -> &'static mut i32 {
    Box::leak(Box::new(42))
}

fn call_return(mut x: &mut i32) -> i32 {
    *x = 7;
    x = read();
    return *x;
}

fn main() {
    let mut x = 1;
    call_return(&mut x);
}
