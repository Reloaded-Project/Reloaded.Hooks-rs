pub(crate) trait ToZydis {
    fn to_zydis(&self) -> zydis::Register;
}
