// compile-flags: -Zmir-emit-retag

fn cast_to_pointer(x: &mut i32, y: &mut i32) -> i32 {
    *x = 42;
    let z = x as *mut i32;
    *y = 7;
    return *x;
}

fn main() {
    let mut x = 1;
    let mut y = 2;
    cast_to_pointer(&mut x, &mut y);
}
