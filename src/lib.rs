//! # Some Cool Reloaded Library
//! Here's the crate documentation.
pub mod exports;

pub mod common {
    pub mod move_operation;

    pub mod graph {
        pub mod node;
    }
}

pub mod x64 {
    pub mod function_attribute;
    pub mod preset_calling_convention;
    pub mod register;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
