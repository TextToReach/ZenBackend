#![allow(non_snake_case)]

mod features;
mod library;
mod parsers;
mod test;
mod util;

use std::{
	cell::RefCell,
	fs::{File, read_to_string},
	rc::Rc,
};

use chumsky::{Parser, prelude::*, primitive::Choice};
use clap::{Parser as ClapParser, Subcommand};
use colored::Colorize;
use features::{
	preprocessor,
	tokenizer::{self, TokenData, TokenTable},
};
use library::{
	Methods::Throw,
	Types::{Severity, ZenError},
};
use test::test::run_tests;
use util::process;

/// Ana CLI aracı
#[derive(ClapParser, Debug)]
#[command(author = "...", version = "0.0.1+ALPHA", about = "Zen CLI")]
struct Cli {
	#[command(subcommand)]
	command: Commands,
}

/// Alt komutlar
#[derive(Subcommand, Debug)]
enum Commands {
	/// Dosya çalıştırma komutu
	Run {
		/// İşlenecek dosya adı
		file: String,

		/// Ayrıntılı çıktı göster
		#[arg(short, long, default_value_t = false)]
		verbose: bool,

		/// AST çıktısını göster
		#[arg(long, default_value_t = false)]
		printast: bool,

		#[arg(long, default_value_t = false)]
		printpreprocessoutput: bool,

		#[arg(long, default_value_t = false)]
		noexecute: bool,

		#[arg(long, default_value_t = true)]
		strict: bool,
	},

	Test,
}

fn run_zen_file(file: String, verbose: bool, printAst: bool, printPreprocessOutput: bool, noexecute: bool, strict: bool) -> miette::Result<()> {
	let full_src: String;
	let contents = match File::open(&file) { Ok(res) => { let lines = read_to_string(&file); match lines { Ok(lines) => { full_src = lines.clone(); let mut buffr = Vec::new(); for line in lines.lines() { buffr.push(line.to_string()); } buffr } Err(_) => { Throw( "Dosya okunmaya çalışırken bir hatayla karşılaşıldı.".to_owned(), library::Types::ZenError::GeneralError, None, None, Severity::High, ); unreachable!() } } } Err(_) => { Throw( "Dosya okunmaya çalışırken bir hatayla karşılaşıldı.".to_owned(), library::Types::ZenError::GeneralError, None, None, Severity::High, ); unreachable!() } };
	process::index(&mut contents.clone(), full_src, verbose, strict, &file)?;

	Ok(())
}

fn main() -> miette::Result<()> {
	let cli = Cli::parse();
	ctrlc::set_handler(|| {
        println!("\nProgram sonlandırılıyor...");
        std::process::exit(0);
    }).unwrap_or_else(|_| {
        Throw(
            format!("Uyarı: Zen {} sinyalini yakalamaya çalışırken bir sorun yaşandı. Program içerisinde bu sinyalle karşılaşılırsa tanımsız durumlarla karşılaşılabilir.\nNot: Bu uyarıyı susturmak için programınızı \"zen --silenced\" ile başlatmayı deneyebilirsiniz.", "(Ctrl+C / Interrupt)".red().italic()),
            ZenError::UnknownError,
            None,
            None,
            Severity::Low
        );
    });

	match cli.command {
		Commands::Run {
			file,
			verbose,
			printast,
			printpreprocessoutput,
			noexecute,
			strict,
		} => {
			run_zen_file(file, verbose, printast, printpreprocessoutput, noexecute, strict)?;
		}
		Commands::Test => {
			run_tests();
		}
	}
	Ok(())
}
