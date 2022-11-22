// compile-flags: -Zmir-emit-retag
use std::arch::asm;

fn inline_asm(x: &mut i32, y: &mut i32) -> i32 {
    *x = 42;
    unsafe {
        asm!(
            "mov {tmp}, 7",
            tmp = out(reg) _,
        );
    }
    *y = 7;
    return *x;
}

fn main() {
    let mut x = 1;
    let mut y = 2;
    inline_asm(&mut x, &mut y);
}
