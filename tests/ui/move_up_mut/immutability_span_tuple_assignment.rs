// compile-flags: -Zmir-emit-retag

fn tuple_assignment(x: &mut i32, y: &mut i32) -> i32 {
    (*x, *y) = (42, 7);
    return *x;
}

fn main() {
    let mut x = 1;
    let mut y = 2;
    tuple_assignment(&mut x, &mut y);
}
