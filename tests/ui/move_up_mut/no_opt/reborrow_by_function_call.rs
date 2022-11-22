// compile-flags: -Zmir-emit-retag

fn access(x: &mut i32) {
    *x = 5;
}

fn reborrow_by_function_call(x: &mut i32, y: &mut i32) -> i32 {
    *x = 42;
    *y = 7;
    access(x);
    return *x;
}

fn main() {
    let mut x = 1;
    let mut y = 2;
    reborrow_by_function_call(&mut x, &mut y);
}
