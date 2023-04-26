// compile-flags: -Zmir-emit-retag

#[derive(Copy, Clone)]
enum SomeEnum {
    A,
    B,
    C,
}

fn eliminate_reads(x: &mut SomeEnum, y: &mut SomeEnum) -> SomeEnum {
    *x = SomeEnum::B;
    *y = SomeEnum::C;
    return *x;
}

fn main() {
    let mut x = SomeEnum::A;
    let mut y = SomeEnum::A;
    eliminate_reads(&mut x, &mut y);
}
