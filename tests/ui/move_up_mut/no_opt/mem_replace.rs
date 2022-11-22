// compile-flags: -Zmir-emit-retag

fn reborrow(x: &mut i32, y: &mut i32) -> i32 {
    *x = 7;
    let z = std::mem::replace(x, 42);
    *y = z + 1;
    return *x;
}

fn main() {
    let mut x = 1;
    let mut y = 2;
    reborrow(&mut x, &mut y);
}
