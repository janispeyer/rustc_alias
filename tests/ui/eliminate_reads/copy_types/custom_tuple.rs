// compile-flags: -Zmir-emit-retag
// Tuples are not yet affected by eliminate-read optimisations.
// That's because tuple assignments create MIR that is hard to handle.
// We're showing that tuples work conceptially by implementing eliminate-reads
// for custom types of the form `struct CustomType { f1: Type1, f2: Type2, ..., fN: TypeN }`.

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
