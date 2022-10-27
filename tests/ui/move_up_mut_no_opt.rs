// compile-flags: -Zmir-emit-retag

fn access(x: &mut i32) {
    *x = 5;
}

fn move_up_mut_no_opt(x: &mut i32, y: &mut i32) -> i32 {
    *x = 42;
    *y = 7;
    access(x);
    return *x;
}

fn main() {
    let mut x = 1;
    let mut y = 2;
    move_up_mut_no_opt(&mut x, &mut y);
}
