use iced_x86::Code;

pub(crate) fn get_instruction_length(code: Code) -> usize {
    match code {
        // Core x86 Instructions
        Code::Mov_rm64_r64 => 3,
        Code::Mov_r64_rm64 => 3,
        Code::Mov_rm32_r32 => 2,
        Code::Mov_r32_rm32 => 2,
        Code::Mov_rm16_r16 => 3,
        Code::Mov_r16_rm16 => 3,
        Code::Mov_rm8_r8 => 2,
        Code::Mov_r8_rm8 => 2,

        Code::Lea_r64_m => 3,
        Code::Lea_r32_m => 2,
        Code::Lea_r16_m => 3,

        Code::Xchg_rm64_r64 => 3,
        Code::Xchg_rm32_r32 => 2,
        Code::Xchg_rm16_r16 => 3,
        Code::Xchg_rm8_r8 => 2,

        Code::Add_rm64_r64 => 3,
        Code::Add_r64_rm64 => 3,
        Code::Add_rm32_r32 => 2,
        Code::Add_r32_rm32 => 2,
        Code::Add_rm16_r16 => 3,
        Code::Add_r16_rm16 => 3,
        Code::Add_rm8_r8 => 2,
        Code::Add_r8_rm8 => 2,

        Code::Adc_rm64_r64 => 3,
        Code::Adc_r64_rm64 => 3,
        Code::Adc_rm32_r32 => 2,
        Code::Adc_r32_rm32 => 2,
        Code::Adc_rm16_r16 => 3,
        Code::Adc_r16_rm16 => 3,
        Code::Adc_rm8_r8 => 2,
        Code::Adc_r8_rm8 => 2,

        Code::Or_rm64_r64 => 3,
        Code::Or_r64_rm64 => 3,
        Code::Or_rm32_r32 => 2,
        Code::Or_r32_rm32 => 2,
        Code::Or_rm16_r16 => 3,
        Code::Or_r16_rm16 => 3,
        Code::Or_rm8_r8 => 2,
        Code::Or_r8_rm8 => 2,

        Code::Sbb_rm64_r64 => 3,
        Code::Sbb_r64_rm64 => 3,
        Code::Sbb_rm32_r32 => 2,
        Code::Sbb_r32_rm32 => 2,
        Code::Sbb_rm16_r16 => 3,
        Code::Sbb_r16_rm16 => 3,
        Code::Sbb_rm8_r8 => 2,
        Code::Sbb_r8_rm8 => 2,

        Code::And_rm64_r64 => 3,
        Code::And_r64_rm64 => 3,
        Code::And_rm32_r32 => 2,
        Code::And_r32_rm32 => 2,
        Code::And_rm16_r16 => 3,
        Code::And_r16_rm16 => 3,
        Code::And_rm8_r8 => 2,
        Code::And_r8_rm8 => 2,

        Code::Sub_rm64_r64 => 3,
        Code::Sub_r64_rm64 => 3,
        Code::Sub_rm32_r32 => 2,
        Code::Sub_r32_rm32 => 2,
        Code::Sub_rm16_r16 => 3,
        Code::Sub_r16_rm16 => 3,
        Code::Sub_rm8_r8 => 2,
        Code::Sub_r8_rm8 => 2,

        Code::Xor_rm64_r64 => 3,
        Code::Xor_r64_rm64 => 3,
        Code::Xor_rm32_r32 => 2,
        Code::Xor_r32_rm32 => 2,
        Code::Xor_rm16_r16 => 3,
        Code::Xor_r16_rm16 => 3,
        Code::Xor_rm8_r8 => 2,
        Code::Xor_r8_rm8 => 2,

        Code::Cmp_rm64_r64 => 3,
        Code::Cmp_r64_rm64 => 3,
        Code::Cmp_rm32_r32 => 2,
        Code::Cmp_r32_rm32 => 2,
        Code::Cmp_rm16_r16 => 3,
        Code::Cmp_r16_rm16 => 3,
        Code::Cmp_rm8_r8 => 2,
        Code::Cmp_r8_rm8 => 2,

        Code::Test_rm64_r64 => 3,
        Code::Test_rm32_r32 => 2,
        Code::Test_rm16_r16 => 3,
        Code::Test_rm8_r8 => 2,

        // Multi-byte extensions (0F)
        Code::Imul_r64_rm64 => 4,
        Code::Imul_r32_rm32 => 3,
        Code::Imul_r16_rm16 => 4,

        Code::Crc32_r64_rm64 => 5,
        Code::Crc32_r64_rm8 => 5,
        Code::Crc32_r32_rm32 => 4,
        Code::Crc32_r32_rm16 => 5,
        Code::Crc32_r32_rm8 => 4,

        Code::Cmovo_r64_rm64 => 4,
        Code::Cmovo_r32_rm32 => 3,
        Code::Cmovo_r16_rm16 => 4,

        Code::Cmovno_r64_rm64 => 4,
        Code::Cmovno_r32_rm32 => 3,
        Code::Cmovno_r16_rm16 => 4,

        Code::Cmovb_r64_rm64 => 4,
        Code::Cmovb_r32_rm32 => 3,
        Code::Cmovb_r16_rm16 => 4,

        // CMOVNB
        Code::Cmovae_r64_rm64 => 4,
        Code::Cmovae_r32_rm32 => 3,
        Code::Cmovae_r16_rm16 => 4,

        // CMOVZ (Zero / Equal) / CMOVE
        Code::Cmove_r64_rm64 => 4,
        Code::Cmove_r32_rm32 => 3,
        Code::Cmove_r16_rm16 => 4,

        // CMOVNZ (Not Zero / Not Equal)
        Code::Cmovne_r64_rm64 => 4,
        Code::Cmovne_r32_rm32 => 3,
        Code::Cmovne_r16_rm16 => 4,

        // CMOVBE (Below or Equal / Not Above)
        Code::Cmovbe_r64_rm64 => 4,
        Code::Cmovbe_r32_rm32 => 3,
        Code::Cmovbe_r16_rm16 => 4,

        // CMOVNBE (Not Below or Equal / Above)
        Code::Cmova_r64_rm64 => 4,
        Code::Cmova_r32_rm32 => 3,
        Code::Cmova_r16_rm16 => 4,

        // CMOVS (Sign)
        Code::Cmovs_r64_rm64 => 4,
        Code::Cmovs_r32_rm32 => 3,
        Code::Cmovs_r16_rm16 => 4,

        // CMOVNS (Not Sign)
        Code::Cmovns_r64_rm64 => 4,
        Code::Cmovns_r32_rm32 => 3,
        Code::Cmovns_r16_rm16 => 4,

        // CMOVP (Parity / Parity Even)
        Code::Cmovp_r64_rm64 => 4,
        Code::Cmovp_r32_rm32 => 3,
        Code::Cmovp_r16_rm16 => 4,

        // CMOVNP (Not Parity / Parity Odd)
        Code::Cmovnp_r64_rm64 => 4,
        Code::Cmovnp_r32_rm32 => 3,
        Code::Cmovnp_r16_rm16 => 4,

        // CMOVL (Less / Not Greater)
        Code::Cmovl_r64_rm64 => 4,
        Code::Cmovl_r32_rm32 => 3,
        Code::Cmovl_r16_rm16 => 4,

        // CMOVNL (Not Less / Greater or Equal)
        Code::Cmovge_r64_rm64 => 4,
        Code::Cmovge_r32_rm32 => 3,
        Code::Cmovge_r16_rm16 => 4,

        // CMOVLE (Less or Equal / Not Greater)
        Code::Cmovle_r64_rm64 => 4,
        Code::Cmovle_r32_rm32 => 3,
        Code::Cmovle_r16_rm16 => 4,

        // CMOVNLE (Not Less or Equal / Greater)
        Code::Cmovg_r64_rm64 => 4,
        Code::Cmovg_r32_rm32 => 3,
        Code::Cmovg_r16_rm16 => 4,

        Code::Bt_rm64_r64 => 4,
        Code::Bt_rm32_r32 => 3,
        Code::Bt_rm16_r16 => 4,

        Code::Bts_rm64_r64 => 4,
        Code::Bts_rm32_r32 => 3,
        Code::Bts_rm16_r16 => 4,

        Code::Shld_rm64_r64_CL => 4,
        Code::Shld_rm32_r32_CL => 3,
        Code::Shld_rm16_r16_CL => 4,

        Code::Shld_rm64_r64_imm8 => 5,
        Code::Shld_rm32_r32_imm8 => 4,
        Code::Shld_rm16_r16_imm8 => 5,

        Code::Shrd_rm64_r64_CL => 4,
        Code::Shrd_rm32_r32_CL => 3,
        Code::Shrd_rm16_r16_CL => 4,

        Code::Shrd_rm64_r64_imm8 => 5,
        Code::Shrd_rm32_r32_imm8 => 4,
        Code::Shrd_rm16_r16_imm8 => 5,

        Code::Cmpxchg_rm64_r64 => 4,
        Code::Cmpxchg_rm32_r32 => 3,
        Code::Cmpxchg_rm16_r16 => 4,
        Code::Cmpxchg_rm8_r8 => 3,

        Code::Btr_rm64_r64 => 4,
        Code::Btr_rm32_r32 => 3,
        Code::Btr_rm16_r16 => 4,

        Code::Popcnt_r64_rm64 => 5,
        Code::Popcnt_r32_rm32 => 4,
        Code::Popcnt_r16_rm16 => 5,

        Code::Btc_rm64_r64 => 4,
        Code::Btc_rm32_r32 => 3,
        Code::Btc_rm16_r16 => 4,

        Code::Bt_rm64_imm8 => 5,
        Code::Bt_rm32_imm8 => 4,
        Code::Bt_rm16_imm8 => 5,

        Code::Bts_rm64_imm8 => 5,
        Code::Bts_rm32_imm8 => 4,
        Code::Bts_rm16_imm8 => 5,

        Code::Btr_rm64_imm8 => 5,
        Code::Btr_rm32_imm8 => 4,
        Code::Btr_rm16_imm8 => 5,

        Code::Btc_rm64_imm8 => 5,
        Code::Btc_rm32_imm8 => 4,
        Code::Btc_rm16_imm8 => 5,

        Code::Bsf_r64_rm64 => 4,
        Code::Bsf_r32_rm32 => 3,
        Code::Bsf_r16_rm16 => 4,

        Code::Bsr_r64_rm64 => 4,
        Code::Bsr_r32_rm32 => 3,
        Code::Bsr_r16_rm16 => 4,

        Code::Xadd_rm64_r64 => 4,
        Code::Xadd_rm32_r32 => 3,
        Code::Xadd_rm16_r16 => 4,

        // More Obscure Tech
        Code::Adcx_r64_rm64 => 6,
        Code::Adcx_r32_rm32 => 5,

        // Alternate instructions (untested)
        _ => panic!("Unknown instruction"),
    }
}

// Instructions for adding lengths
// Check instruction/opcode docs for length.

// If you see 'o64' it means, '48' prefix, and thus +1 byte
// If you see 'o16' it means, '66' prefix, and thus +1 byte
