// compile-flags: -Zmir-emit-retag

fn access(x: &mut i32) {
    *x = 5;
}

fn immutability_span(x: &mut i32, y: &mut i32, flag: bool) -> i32 {
    *x = 42;

    if flag {
        let mut z;
        for _ in 0..3 {
            z = *x;
        }
        let _ = z;

        *x = 5;

        if *y == 0 {
            *y = 1;
        }
    } else {
        *y = 7;
        access(x);

        for _ in 0..5 {
            *y = 11;
            *x = 17;
        }

        let z = *x;
        let _ = z;
    }

    *y = 12;

    return *x;
}

fn main() {
    let mut x = 1;
    let mut y = 2;
    immutability_span(&mut x, &mut y, false);
}
