// compile-flags: -Zmir-emit-retag

// Prevent inlining
#[inline(never)]
fn lib(x: &u32) {
    // Prevent the LLVM inferring that the argument is unused
    println!("{}", x);
}

#[inline(never)]
fn mid(x: &mut u32, y: &mut u32) -> bool {
    *x = 7;
    *y = 42;
    lib(x);
    lib(y);
    return *x == 7 && *y == 42;
}

fn main() {
    let mut x = 1;
    let mut y = 2;
    mid(&mut x, &mut y);
}
