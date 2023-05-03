// compile-flags: -Zmir-emit-retag
// Tuples are not yet affected by eliminate-read optimisations.
// That's because tuple assignments create MIR that is hard to handle.
// We're showing that tuples work conceptially by implementing eliminate-reads
// for custom types of the form `struct CustomTuple(Type1, Type2, ..., TypeN)`.

fn eliminate_reads(x: &mut (i32, i32), y: &mut (i32, i32)) -> (i32, i32) {
    *x = (42, 1337);
    *y = (7, 13);
    return *x;
}

fn main() {
    let mut x = (1, 2);
    let mut y = (3, 4);
    eliminate_reads(&mut x, &mut y);
}
