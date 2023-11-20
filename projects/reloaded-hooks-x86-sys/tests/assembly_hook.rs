mod asm;
use asm::calculator::CALCULATOR_ADD_MSFT_X64;

#[test]
#[cfg(target_arch = "x86_64")]
fn calculator_add_works() {
    use crate::asm::calculator::Add;
    use asm::assemble_function::alloc_function;
    use core::mem::transmute;

    let add: Add = unsafe { transmute(alloc_function(&CALCULATOR_ADD_MSFT_X64).unwrap()) };
    for x in 0..100 {
        for y in 0..100 {
            assert_eq!(x + y, add(x, y));
        }
    }
}
