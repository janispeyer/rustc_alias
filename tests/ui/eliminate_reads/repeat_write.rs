// compile-flags: -Zmir-emit-retag

fn repeat_write(x: &mut i32, y: &mut i32) -> i32 {
    *x = 42;
    *y = 7;
    *x = *x;
    *y = 8;
    *x = *x + 1;
    *y = 9;
    return *x;
}

fn main() {
    let mut x = 1;
    let mut y = 2;
    repeat_write(&mut x, &mut y);
}
