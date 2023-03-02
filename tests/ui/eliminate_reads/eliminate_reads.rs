// compile-flags: -Zmir-emit-retag

fn eliminate_reads(x: &mut i32, y: &mut i32) -> i32 {
    *x = 42;
    *y = 7;
    return *x;
}

fn main() {
    let mut x = 1;
    let mut y = 2;
    eliminate_reads(&mut x, &mut y);
}
