mod library;
mod parsers;

use chumsky::{prelude::*, primitive::Choice, Parser};
use library::Types::{Instruction, Object};
use parsers::instructions::{yazdir, Kit};
use clap::{Parser as ClapParser, Subcommand};

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
    },
}

/* fn Lexer() -> Choice<[Box<dyn Parser<char, Object, Error = Simple<char>>>; 2], Simple<char>>{
    choice([
        Object::parser()
    ])
} */ 

fn process(AST: Instruction){
    let instruction = AST.0.as_str();
    let args = AST.1;
    match instruction {
        "yazdır" => {
            PrintVec!(args);
        }
        _ => {}
    }
}

fn run(file: String, verbose: bool) {
    let input = ReadFile!(file);
    let input = input.split("\n").collect::<Vec<&str>>()[0];

    match Kit::parser().parse(input) {
        Ok(result) => {
            process(result)
        },
        Err(errors) => {
            for error in errors {
                println!("Hata: {:?}", error);
            }
        }
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { file, verbose } => {
            run(file, verbose);
        }
    }
}
