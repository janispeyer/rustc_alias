// compile-flags: -Zmir-emit-retag

fn eliminate_reads<'a>(x: &mut &'a i32, y: &mut &'a i32, z: &mut &'a i32) -> &'a i32 {
    let tmp = *x;
    *x = *z;
    *y = tmp;
    return *x;
}

fn main() {
    let a = 1;
    let b = 2;
    let c = 3;
    let mut x = &a;
    let mut y = &b;
    let mut z = &c;
    eliminate_reads(&mut x, &mut y, &mut z);
}
