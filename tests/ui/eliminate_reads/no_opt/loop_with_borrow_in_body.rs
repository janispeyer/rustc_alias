// compile-flags: -Zmir-emit-retag

fn some_fn() {
    println!("something");
}

fn loop_with_borrow_in_body(x: &mut i32) -> i32 {
    *x = 42;

    for _ in 0..3 {
        // x is on top of the stack in the first iteration.
        // In the consecutive iterations x is not on top of the stack.
        // So the analysis should consider x to be unkown.
        some_fn();
        let y = &mut *x;
        *y = 5;
    }

    return *x;
}

fn main() {
    let mut x = 1;
    loop_with_borrow_in_body(&mut x);
}
