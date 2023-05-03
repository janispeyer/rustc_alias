// compile-flags: -Zmir-emit-retag

#[derive(Copy, Clone)]
struct CustomTuple(i32, i32);

fn eliminate_reads(x: &mut CustomTuple, y: &mut CustomTuple) -> CustomTuple {
    *x = CustomTuple(42, 1337);
    *y = CustomTuple(7, 13);
    return *x;
}

fn main() {
    let mut x = CustomTuple(1, 2);
    let mut y = CustomTuple(3, 4);
    eliminate_reads(&mut x, &mut y);
}
