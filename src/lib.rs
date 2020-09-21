//! Library common to tests and bare-metal
#![no_std]

#![feature(const_in_array_repeat_expressions, const_mut_refs)]

use core::{mem::size_of, fmt::Write};
use getrandom::getrandom;

pub mod tables;

pub trait Plat {
    fn logger(&mut self) -> &mut dyn Write;
    fn print_random_val(&mut self, a: &[usize]) {
        let idx = random_usize() % a.len();
        let val = a[idx];
        writeln!(self.logger(), "{}", val).unwrap();
    }
}

fn random_usize() -> usize {
    let mut arr = [0; size_of::<usize>()];
    getrandom(&mut arr).unwrap();
    usize::from_ne_bytes(arr)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lib_unit_test() {
        assert_ne!(random_usize(), random_usize());
    }
}
