// compile-flags: -Zmir-emit-retag

fn access(value: &mut i32) {
    println!("{}", value);
}

fn regain_top_of_stack(x: &mut i32, y: &mut i32) -> i32 {
    access(&mut *x); // lose top of stack here
    *x = 42; // regain top of stack (not implemtned yet, so it's in the no_opt folder)
    *y = 7;
    return *x;
}

fn main() {
    let mut x = 1;
    let mut y = 2;
    regain_top_of_stack(&mut x, &mut y);
}
