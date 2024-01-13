extern crate alloc;
use crate::common::jit_common::X86jitError;
use alloc::vec::Vec;
use core::ptr::write_unaligned;
use reloaded_hooks_portable::api::jit::return_operation::ReturnOperation;

pub(crate) fn encode_return<TRegister>(
    x: &ReturnOperation,
    pc: &mut usize,
    buf: &mut Vec<u8>,
) -> Result<(), X86jitError<TRegister>> {
    unsafe {
        let len = if x.offset == 0 { 1 } else { 3 };
        let old_len = buf.len();
        buf.reserve(len);
        let ptr = buf.as_mut_ptr().add(old_len);

        if len == 1 {
            ptr.write(0xC3);
        } else {
            ptr.write(0xC2);
            // Append the 16-bit offset as little-endian using write_unaligned
            write_unaligned(ptr.add(1) as *mut u16, (x.offset as u16).to_le());
        }
        buf.set_len(old_len + len);
        *pc += len;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::util::test_utilities::assert_encode;
    use rstest::rstest;

    #[rstest]
    #[case(ReturnOperation::new(0), "c3")]
    #[case(ReturnOperation::new(4), "c20400")]
    #[case(ReturnOperation::new(0x1234), "c23412")]
    fn test_encode_return(#[case] operation: ReturnOperation, #[case] expected_encoded: &str) {
        let mut buf = Vec::new();
        let mut pc = 0;
        encode_return::<crate::x64::Register>(&operation, &mut pc, &mut buf).unwrap();
        assert_encode(expected_encoded, &buf, pc);
    }
}
