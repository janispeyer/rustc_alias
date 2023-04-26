// compile-flags: -Zmir-emit-retag

#[derive(Copy, Clone)]
struct SomeTuple(i32);

#[derive(Copy, Clone)]
struct SomeStruct {
    a: i32,
    b: i32,
}

fn eliminate_reads(x: &mut SomeTuple, y: &mut SomeTuple) -> i32 {
    *x = SomeTuple(42);
    *y = SomeTuple(7);
    return x.0;
}

fn eliminate_reads_struct(x: &mut SomeStruct, y: &mut SomeStruct) -> i32 {
    *x = SomeStruct { a: 42, b: 24 };
    *y = SomeStruct { a: 17, b: 7 };
    return x.a;
}

fn main() {
    let mut x = SomeTuple(0);
    let mut y = SomeTuple(0);
    eliminate_reads(&mut x, &mut y);

    let mut a = SomeStruct { a: 0, b: 0 };
    let mut b = SomeStruct { a: 0, b: 0 };
    eliminate_reads_struct(&mut a, &mut b);
}
