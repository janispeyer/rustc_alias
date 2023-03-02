// compile-flags: -Zmir-emit-retag
use std::cell::Cell;
use std::rc::Rc;

fn access(x: &Rc<Cell<i32>>) {
    x.set(13);
}

fn interior_mutability(x: &mut Rc<Cell<i32>>, y: &mut Rc<Cell<i32>>) -> i32 {
    *x = Rc::new(Cell::new(42));
    y.set(7);
    access(x); // <- because of this (interior mutability),
    x.get() // we cannot optimise this line
}

fn main() {
    let mut x = Rc::new(Cell::new(1));
    let mut y = x.clone();
    interior_mutability(&mut x, &mut y);
}
