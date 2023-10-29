// lowering tests.

use super::*;
use crate::commons::skip_validation;
use crate::middle_end::lir;
use crate::interpreter::{interpret, RuntimeError};

mod part1_basic;
mod part1_second_point;

// This is the student version.  It does not do AST validation.
fn parse_and_validate(code: &str) -> Result<Valid<Program>, String> {
    let program = parse(code).map_err(|err| err.0)?;

    Ok(skip_validation(program))
}

// Parse given program, skip validation, lower to LIR, validate, run and return
// what `main` returns.
fn lower_and_run(code: &str) -> Result<i64, String> {
    let program = parse(code).map_err(|err| err.0)?;

    let lowered = lower(&skip_validation(program));
    lir::validate(&lowered).expect("The generated LIR program is not valid.");

    interpret(lowered).map_err(|RuntimeError(s)| format!("runtime error: {s}"))
}
