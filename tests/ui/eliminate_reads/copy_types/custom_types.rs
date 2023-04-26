// compile-flags: -Zmir-emit-retag

#[derive(Copy, Clone)]
enum SomeEnum {
    A,
    B(i32),
    C { a: i32 },
}

#[derive(Copy, Clone)]
struct SomeStruct {
    a: i32,
    b: i32,
}

fn eliminate_reads(x: &mut SomeEnum, y: &mut SomeEnum) -> SomeEnum {
    *x = SomeEnum::B(42);
    *y = SomeEnum::C { a: 7 };
    return *x;
}

fn eliminate_reads_struct(x: &mut SomeStruct, y: &mut SomeStruct) -> SomeStruct {
    *x = SomeStruct { a: 42, b: 24 };
    *y = SomeStruct { a: 17, b: 7 };
    return *x;
}

fn main() {
    let mut x = SomeEnum::A;
    let mut y = SomeEnum::A;
    eliminate_reads(&mut x, &mut y);

    let mut a = SomeStruct { a: 0, b: 0 };
    let mut b = SomeStruct { a: 0, b: 0 };
    eliminate_reads_struct(&mut a, &mut b);
}
