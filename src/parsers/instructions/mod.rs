pub mod yazdir;
pub mod Kit {
    use chumsky::prelude::*;
    use crate::library::Types::{Instruction, Object};

    use super::yazdir;

    pub fn parser<'a>() -> Box<dyn Parser<char, Instruction, Error = Simple<char>> + 'a> {
        Box::new(choice([
            yazdir::parser()
        ]))
    }
}