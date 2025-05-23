use super::Parsers;
use crate::{
	Debug, Print, PrintVec,
	features::tokenizer::{TokenData, InstructionEnum, TokenTable},
};
use chumsky::prelude::*;

pub fn parser() -> Box<dyn Parser<TokenData, InstructionEnum, Error = Simple<TokenData>>> {
	let out = Parsers::number()
		.then_ignore(just(TokenTable::KeywordNDefaTekrarla.asTokenData())) 
		.map(|a| InstructionEnum::Repeat{ repeat_count: a.asNumberLiteral(), scope_pointer: 0 } );

	return Box::new(out);
}
