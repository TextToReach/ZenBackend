use super::ScopeManager::{ConditionBlock, ConditionStructure, Scope};
use crate::features::tokenizer::{AssignmentMethod, CheckTokenVec, ConditionBlockType, RemoveQuotes};
use crate::library::Error::GirintiHatası;
use crate::parsers::Parsers::Expression;
use crate::{
	DebugVec, Print, PrintVec,
	features::tokenizer::{InstructionEnum, TokenData, TokenTable, tokenize},
	library::Types::CutFromStart,
	parsers::Parsers::{self, ParserOutput},
	util::ScopeManager::{ScopeAction, ScopeManager},
};
use chumsky::prelude::*;
use colored::Colorize;
use defer::defer;
use miette::{Error, NamedSource, SourceSpan};
use std::collections::{HashMap, HashSet};
use std::ptr::null;

#[derive(Debug, Clone, PartialEq)]
pub enum BlockOutput {
	Break,
	Continue,
	None,
}

pub fn ExecuteBlock(scope_id: usize, manager: &mut ScopeManager) -> BlockOutput {
	let scope = manager.get_scope(scope_id).expect(format!("Scope {scope_id} does not exist.").as_str());
	let block = scope.block.clone();
	let mut result = BlockOutput::None;

	for line in block.clone() {
		match line.clone() {
			InstructionEnum::Print(expr) => {
				PrintVec!(expr.iter().map(|x| x.evaluate(scope_id, manager)).collect::<Vec<_>>());
			}
			InstructionEnum::VariableDeclaration(name, value, method) => {
				let evaluated_value = value.evaluate(scope_id, manager);
				let new_value = match method {
					AssignmentMethod::Set => evaluated_value,
					AssignmentMethod::Add => manager.get_var(scope_id, name.clone()).expect("No previous value") + evaluated_value,
					AssignmentMethod::Sub => manager.get_var(scope_id, name.clone()).expect("No previous value") - evaluated_value,
					AssignmentMethod::Mul => manager.get_var(scope_id, name.clone()).expect("No previous value") * evaluated_value,
					AssignmentMethod::Div => manager.get_var(scope_id, name.clone()).expect("No previous value") / evaluated_value,
				};

				let mut temp_scope_id = scope_id.clone();
				loop {
					let current_scope_var = manager.does_var_exists(scope_id, name.clone());
					if current_scope_var {
						manager.set_var(temp_scope_id, name.clone(), new_value.clone());
						break;
					} else {
						if let Some(parent) = manager.get_parent(temp_scope_id) {
							temp_scope_id = parent;
						} else {
							manager.set_var(temp_scope_id, name.clone(), new_value.clone());
							break;
						}
					}
				}
			}
			InstructionEnum::Repeat { repeat_count, scope_pointer } => {
				for _ in 0..repeat_count.floor() as i64 {
					match ExecuteBlock(scope_pointer, manager) {
						BlockOutput::Break => break,
						BlockOutput::Continue => continue,
						BlockOutput::None => {}
					}
				}
			}
			InstructionEnum::Function { name, args, scope_pointer } => {
				manager.declare_function(scope_id, name.clone(), args.clone(), scope_pointer.clone());
			}
			InstructionEnum::CallFunction { name, args } => {
				ExecuteBlock(manager.get_function(scope_id, name.clone()).unwrap().scope_pointer, manager);
			}
			InstructionEnum::Break => {
				result = BlockOutput::Break;
				break;
			}
			InstructionEnum::Continue => {
				result = BlockOutput::Continue;
				break;
			}
			InstructionEnum::Condition(condition) => {
				// Evaluate the main condition
				if condition.If.condition.isTruthy(scope_id, manager) {
					match ExecuteBlock(condition.If.scope_pointer, manager) {
						BlockOutput::Break => {
							result = BlockOutput::Break;
							break;
						}
						BlockOutput::Continue => {
							result = BlockOutput::Continue;
							break;
						}
						BlockOutput::None => {}
					}
				} else {
					// Check elifs
					let mut executed = false;
					for elif in &condition.Elif {
						if elif.condition.isTruthy(scope_id, manager) {
							match ExecuteBlock(elif.scope_pointer, manager) {
								BlockOutput::Break => {
									result = BlockOutput::Break;
									break;
								}
								BlockOutput::Continue => {
									result = BlockOutput::Continue;
									break;
								}
								BlockOutput::None => {}
							}
							executed = true;
							break;
						}
					}
					// Else block
					if !executed {
						match ExecuteBlock(condition.Else.scope_pointer, manager) {
							BlockOutput::Break => {
								result = BlockOutput::Break;
								break;
							}
							BlockOutput::Continue => {
								result = BlockOutput::Continue;
								break;
							}
							BlockOutput::None => {}
						}
					}
				}
			}
			_ => todo!(),
		}
	}
	result
}

pub fn ProcessLine(
	full_source: String,
	raw_line_feed_string: String,
	line_feed: Vec<TokenData>,
	instr: (ParserOutput, InstructionEnum),
	current_scope_id: &mut usize,
	manager: &mut ScopeManager,
	opts: &Runopts,
	fileandline: (&str, u32)
	// conditional_grup:
) -> miette::Result<()> {
	let line_feed_tab_count = line_feed.count_from_start(|x| x.token == TokenTable::Tab);
	let inferred_scope_depth = manager.get_depth(current_scope_id.clone());

	if line_feed_tab_count != inferred_scope_depth {
		if line_feed_tab_count > inferred_scope_depth && opts.strict {
			return Err(GirintiHatası {
				src: NamedSource::new(fileandline.0, full_source).with_language("Zen"),
				bad_bit: SourceSpan::new(
					inferred_scope_depth.into(),
					(line_feed_tab_count - inferred_scope_depth) as usize
				)
			})?;
		} else {
			let temp = current_scope_id.clone();
			*current_scope_id = manager.get_parent(current_scope_id.clone()).unwrap();
		}
	}

	if instr.0.indent {
		let mut instr_enum = instr.clone().1;
		let new_scope = match instr_enum {
			InstructionEnum::IfBlock { .. } => manager.create_transparent_scope(*current_scope_id, Some(instr_enum.as_block_action())),
			InstructionEnum::ElifBlock { .. } => manager.create_transparent_scope(*current_scope_id, Some(instr_enum.as_block_action())),
			InstructionEnum::ElseBlock { .. } => manager.create_transparent_scope(*current_scope_id, Some(instr_enum.as_block_action())),
			InstructionEnum::Function { .. } => manager.create_isolated_scope(*current_scope_id, Some(instr_enum.as_block_action())),
			_ => manager.create_scope(Some(*current_scope_id), Some(instr_enum.as_block_action())),
		};

		match instr_enum {
			InstructionEnum::IfBlock { .. } => {
				instr_enum = InstructionEnum::Condition(
					ConditionBlock::new(ConditionStructure { scope_pointer: new_scope, condition: instr_enum.as_expression() })
				);
				manager.push_code_to_scope(*current_scope_id, &instr_enum);
				*current_scope_id = new_scope;
			}
			InstructionEnum::ElifBlock { .. } => {
				if let Some(last_instr) = manager.get_scope_mut(*current_scope_id).unwrap().block.last_mut() {
					if let InstructionEnum::Condition(con) = last_instr {
						con.push_elif(
							ConditionStructure { scope_pointer: new_scope, condition: instr_enum.as_expression() }
						);
					}
				}
				*current_scope_id = new_scope;
			}
			InstructionEnum::ElseBlock { .. } => {
				if let Some(last_instr) = manager.get_scope_mut(*current_scope_id).unwrap().block.last_mut() {
					if let InstructionEnum::Condition(con) = last_instr {
						con.push_else(
							ConditionStructure { scope_pointer: new_scope, condition: instr_enum.as_expression() }
						);
					}
				}
				*current_scope_id = new_scope;
				
			}
			_ => {				
				instr_enum.set_block_pointer(new_scope);
				manager.push_code_to_scope(*current_scope_id, &instr_enum);
				*current_scope_id = new_scope;
			}
		}

	} else {
		manager.push_code_to_scope(*current_scope_id, &instr.1);
	}

	Ok(())
}

pub struct Runopts { verbose: bool, strict:bool }

pub fn index(input: &mut Vec<String>, full_source: String, verbose: bool, strict: bool, filename: &str) -> miette::Result<()> {
	let mut manager = ScopeManager::new();
	let root_scope = manager.create_scope(None, None);
	let mut currentScope = root_scope;
	let mut line_index = 0;
	let opts = Runopts {
		verbose,
		strict
	};

	for line in input.iter_mut() {
		line_index += 1;
		for chunk in line.split(";") {
			let raw_line_feed = tokenize(chunk);
			if !raw_line_feed.is_all_ok() {
				continue;
			}
			let line_feed_without_tabs = raw_line_feed.iter().filter(|x| x.token != TokenTable::Tab).cloned().collect::<Vec<_>>();

			if !line_feed_without_tabs.starts_with(&[TokenTable::Comment.asTokenData()]) {
				match Parsers::parser().parse(line_feed_without_tabs.clone()) {
					Ok(res) => {
						match ProcessLine(chunk.to_owned(), full_source.clone(), raw_line_feed, res.clone(), &mut currentScope, &mut manager, &opts, (filename, line_index)) {
							Err(e) => {
								return Err(e);
							}
							_ => {}
						}
					}
					Err(e) => {
						panic!("Error happened: {:#?}", e)
					}
				}
			}
		}
	}

	
	// println!("{:#?}\n-----------------------------------", manager.get_scope(0));
	// println!("{:#?}\n-----------------------------------", manager.get_scope(1));
	// println!("{:#?}\n-----------------------------------", manager.get_scope(2));
	// println!("{:#?}\n-----------------------------------", manager.get_scope(3));
	
	ExecuteBlock(root_scope, &mut manager);
	Ok(())
}
