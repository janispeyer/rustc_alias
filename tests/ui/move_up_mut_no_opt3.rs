// compile-flags: -Zmir-emit-retag

fn move_up_mut_no_opt3(x: &mut i32, y: &mut i32) -> i32 {
    *x = 42;
    let z = x as *mut i32;
    *y = 7;
    return *x;
}

fn main() {
    let mut x = 1;
    let mut y = 2;
    move_up_mut_no_opt3(&mut x, &mut y);
}
