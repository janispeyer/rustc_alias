// compile-flags: -Zmir-emit-retag

fn eliminate_reads(x: &mut Option<i32>, y: &mut Option<i32>) -> Option<i32> {
    *x = Some(42);
    *y = None;
    return *x;
}

fn main() {
    let mut x = Some(1);
    let mut y = Some(2);
    eliminate_reads(&mut x, &mut y);
}
