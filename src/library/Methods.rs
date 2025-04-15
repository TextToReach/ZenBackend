#![allow(non_snake_case, dead_code)]

use super::Types::ZenError;
use colored::Colorize;

pub fn Str(val: &str) -> String {
    String::from(val)
}

#[macro_export]
macro_rules! Print {
    () => {
        println!();

        
    };

    ($($arg:expr),*) => {
        let mut acc: Vec<String> = Vec::new();

        $(
            acc.push(format!("{}", $arg));
        )*

        println!("{}", acc.join(" "));
    };
}

#[macro_export]
macro_rules! PrintVec {
    () => {
        println!();
    };

    ($($arg:expr),*) => {
        let mut acc: Vec<String> = Vec::new();

        $(
            for x in $arg {
                acc.push(format!("{}", x));
            }
        )*

        println!("{}", acc.join(" "));
    };
}

#[macro_export]
macro_rules! Debug {
    () => {
        println!();
    };

    ($($arg:expr),*) => {
        let mut acc: Vec<String> = Vec::new();

        $(
            acc.push(format!("{:?}", $arg));
        )*

        println!("{}", acc.join(" "));
    };
}

#[macro_export]
macro_rules! ReadFile {
    ($path:expr) => {{
        use std::fs;
        match fs::read_to_string($path) {
            Ok(content) => content,
            Err(e) => panic!("Dosya okunurken bir hata oluştu! Hata: {}", e),
        }
    }};
}

#[macro_export]
macro_rules! Input {
    () => {{
        use std::io::{self, Write};
        let mut input = String::new();
        print!("");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut input).unwrap();
        input.trim().to_string()
    }};

    ($prompt:expr) => {{
        use std::io::{self, Write};
        let mut input = String::new();
        print!("{}", $prompt);
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut input).unwrap();
        input.trim().to_string()
    }};

    (<$t:ty>) => {{
        use std::io::{self, Write};
        let mut input = String::new();
        print!("Enter input: "); // Optional prompt
        io::stdout().flush().unwrap(); // Ensure the prompt is printed immediately
        io::stdin().read_line(&mut input).unwrap();
        input
            .trim()
            .parse::<$t>()
            .unwrap_or_else(|_| panic!("Failed to parse input into the specified type."))
    }};

    // Case when a prompt is passed, with a specified type
    ($prompt:expr, <$t:ty>) => {{
        use std::io::{self, Write};
        let mut input = String::new();
        print!("{}", $prompt); // Print the custom prompt
        io::stdout().flush().unwrap(); // Ensure the prompt is printed immediately
        io::stdin().read_line(&mut input).unwrap();
        input
            .trim()
            .parse::<$t>()
            .unwrap_or_else(|_| panic!("Failed to parse input into the specified type."))
    }};
}

#[derive(Clone)]
pub struct FileAndLineInformation(pub u16, pub String);
pub fn Throw(
    description: String,
    errortype: ZenError,
    file_and_line: Option<FileAndLineInformation>,
    unreachable_error: Option<bool>,
) {
    println!(
        "\n{err1}\n[{line}] ve [{file}] noktasında bir hatayla karşılaşıldı.\n\n...\n\n{err2}: {desc}{extra}",
        extra = if unreachable_error.unwrap_or(false) { "\nBu hata normal kullanımda karşılaşılamaması gereken bir hatadır. Hata yüksek ihtimalle kullandığınız Zen sürümünden kaynaklıdır.\nLütfen Zen yüklemenizi güncelleyiniz. Eğer hata devam ederse lütfen giderilebilmesi için geliştiricilere domain@mail.com adresinden iletiniz." } else { "" },
        err1 = format!("Bir hata ile karşılaşıldı. ({})", format!("{:?}", errortype).underline().italic()).red(),
        err2 = format!("{:?}", errortype).underline().italic().red(),
        file = format!("Satır: {}", if let Some(FileAndLine) = file_and_line.clone() { FileAndLine.0.to_string() } else { "Bilinmiyor".to_string() } ).green(),
        line = format!("Dosya: {}", if let Some(FileAndLine) = file_and_line.clone() { FileAndLine.1.to_string() } else { "Bilinmiyor".to_string() }).cyan(),
        desc = description
    );
}
