// compile-flags: -Zmir-emit-retag

fn access(value: &mut i32) {
    *value = 77;
}

fn eliminate_reads(x: &mut (i32, i32), y: &mut (i32, i32)) -> (i32, i32) {
    *x = (42, 1337);
    *y = (7, 13);
    access(&mut x.0);
    return *x;
}

fn main() {
    let mut x = (1, 2);
    let mut y = (3, 4);
    eliminate_reads(&mut x, &mut y);
}
