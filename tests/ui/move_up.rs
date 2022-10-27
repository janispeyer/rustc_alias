// compile-flags: -Zmir-emit-retag

fn move_up(x: &i32, mut f: impl FnMut(&i32, i32)) -> i32 {
    let val = *x / 3;
    f(x, val);
    return *x / 3;
}

fn main() {
    let x = 42;
    move_up(&x, |_, _| {});
}
