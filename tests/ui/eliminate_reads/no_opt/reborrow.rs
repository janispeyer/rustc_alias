// compile-flags: -Zmir-emit-retag

fn reborrow(x: &mut i32, y: &mut i32) -> i32 {
    *x = 42;
    let z = &mut *x;
    *y = 7;
    return *x;
}

fn main() {
    let mut x = 1;
    let mut y = 2;
    reborrow(&mut x, &mut y);
}
