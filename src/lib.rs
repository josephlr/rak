//! Library common to tests and bare-metal
#![no_std]
#![feature(
    const_in_array_repeat_expressions,
    const_mut_refs,
    const_ptr_offset,
    const_fn,
    const_panic
)]
#![feature(asm, naked_functions)]

use core::{
    fmt::{self, Write},
    mem::size_of,
};
use getrandom::getrandom;

pub mod gdt;
pub mod paging;

pub trait Plat {
    fn logger(&mut self) -> &mut dyn Write;
}

fn random_usize() -> usize {
    let mut arr = [0; size_of::<usize>()];
    getrandom(&mut arr).unwrap();
    usize::from_ne_bytes(arr)
}

pub fn print_random_val(plat: &mut dyn Plat, a: &[usize]) -> fmt::Result {
    let idx = random_usize() % a.len();
    let val = a[idx];
    writeln!(plat.logger(), "{}", val)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lib_unit_test() {
        assert_ne!(random_usize(), random_usize());
    }
}
