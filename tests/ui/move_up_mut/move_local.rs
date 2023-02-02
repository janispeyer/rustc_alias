// compile-flags: -Zmir-emit-retag

pub fn f(x: &mut u32, y: &mut u32) -> u32 {
    let mut a = 42;
    let b = &mut a;
    *x = *b; // gets translated to: `tmp = copy (*b); *x = move tmp;`
    *y = 7;
    return *x;
}

fn main() {
    let mut x = 1;
    let mut y = 2;
    println!("{}", f(&mut x, &mut y));
}
