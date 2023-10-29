// The compiler for CFlat code.

use clap::Parser;
use derive_more::Display;
use lowering::commons::skip_validation;
use lowering::front_end::*;
use lowering::middle_end::lir;
use std::str::FromStr;

// Input/output file types
#[derive(Display, Clone, Copy, PartialEq, Eq)]
enum FileType {
    CFlat,
    Ast,
    Lir,
}

// File names with associated file types.  This is used for determining input
// and output file types from file names.  The actual functionality is
// implemented in the `from_str` trait function.
#[derive(Clone)]
struct File {
    typ: FileType,
    name: String,
}

impl FromStr for File {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use FileType::*;

        let name = String::from(s);
        let typ = s.rsplit_once('.').and_then(|(_, extension)| match extension {
            "lir" => Some(Lir),
            "json" => Some(Ast),
            "cf" | "cb" => Some(CFlat),
            _ => None,
        }).ok_or_else(|| format!("Expected a file name with one of the following extensions: json, lir, cf, cb. Got {}", s))?;

        Ok(File { typ, name })
    }
}

// Command-line arguments
#[derive(Parser)]
#[command(version, about)]
struct Args {
    input_file: File,
    output_file: File,
}

pub fn main() {
    let args = Args::parse();
    let input_file = args.input_file.name.as_str();
    let output_file = args.output_file.name.as_str();

    let input_string = String::from_utf8(
        std::fs::read(input_file)
            .unwrap_or_else(|_| panic!("Could not read the input file {}", input_file)),
    )
    .expect("The input file does not contain valid utf-8 text");

    let cf_program: ast::Program;
    let program: lir::Program = match args.input_file.typ {
        FileType::Lir => panic!("The input file must be a CFlat program, not an LIR program."),
        FileType::Ast => {
	    cf_program = serde_json::from_str(&input_string).unwrap_or_else(|e| panic!("AST JSON file is not valid: {e}"));
            lower(&skip_validation(cf_program.clone()))
	}
        FileType::CFlat => {
            cf_program =
                parse(&input_string).unwrap_or_else(|e| panic!("Syntax error: {e}"));
            lower(&skip_validation(cf_program.clone()))
        }
    };

    let output = match args.output_file.typ {
        FileType::Lir => program.to_string().into_bytes(),
        FileType::Ast => serde_json::to_string_pretty(&cf_program).unwrap().into_bytes(),
        FileType::CFlat => panic!("Cannot output a .cb file"),
    };

    std::fs::write(output_file, output).unwrap_or_else(|_| {
        panic!(
            "Failed to write to the optimized program to the output file: {}",
            output_file
        )
    });
}
