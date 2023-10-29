# C-Flat

A soundly-typed C-like language. This library contains the language definition,
compiler, and analyzer.

# Programming Assignments 3 and 4: The IR generator

**See Canvas for the due dates**

- Part 1 is assignment 3.
- Part 2 is assignment 4.
- You need to make part 1 to work as a pre-requisite for part 2.
- You'll submit the same repository to both assignments, but they will be
  separate assignments on Gradescope.  In short: 1 repo, 2 gradescope
  assignments.

In this assignment, you are going to write the lowering pass for the C♭ (CFlat)
programming language.

The goals of this assignment are:

1. To build the core of the compiler.
2. To understand how syntax-driven translation works.
3. To see how to traverse ASTs recursively to analyze and generate code.

You are already given a lexer that uses the `Logos` library, so you don't need
assignment 1 solution to work on this assignment.  However, you need to either
use Linux or your solution to the parser assignment.

**Copy your parser to `parser.rs` before starting the assignment.  If you don't
have a parser, you need to edit the `parse` function to execute my parser binary
(`~memre/parse` on vlab, which you need to download) and deserialize the JSON it
produces.**  I will release a version of `parse` that does this by the end of
Fall break.  Until then, you can see how to deserialize the ASTs in `cfc.rs`.

## Compiler correctness

You should consult the C♭ semantics document to make sure that your lowering
implementation produces a correct LIR program (i.e., it preserves the
semantics).

### Validation

The tests I give you are valid C♭ programs.  You can use my compiler on vlab
computers to check for validity of your own tests (if it is valid, then you
won't get a compiler error).

When given a valid C♭ program, lowering should produce a valid LIR program.  The
test suite checks this before running the program.

## Directory structure

This is the directory structure we will use in programming assignments:

```
├── Cargo.toml
├── README.md
└── src
    ├── bin
    │   └── ...
    ├── commons.rs
    ├── front_end
    ├── middle_end
    ├── back_end
    └── lib.rs
```

For this project, we will have `front_end` and `middle_end`, and it looks like
so:

```
src
├── bin
│   ├── parse.rs                  // This program runs the parser on a C-flat
│   │                             // program.  It outputs the AST as a JSON file.
│   └── prettify.rs               // This program runs the parser on a C-flat
│                                 // program, then "un-parses" the AST.  It can
│                                 // be a useful testing tool to see the parser
│                                 // output.
├── commons.rs
├── front_end
│   ├── ast                       // Various trait & associated implementations for the AST.
│   │   ├── arbitrary_impl.rs
│   │   ├── associated_impl.rs
│   │   ├── display_impl.rs
│   │   └── fromstr_impl.rs
│   ├── ast.rs                    // Main AST definitions. You'll need these.
│   ├── lexer.rs
│   ├── mod.rs
│   ├── parser.rs                 // This file contains the parser.
│   ├── lower.rs                  // This file contains the lowering pass.
│   │                             // All your implementation goes here
│   ├── tests
│   │   └── lex_tests.rs
│   │   └── parse_tests.rs
│   │   └── parse_tests
│   │   └── lower_tests.rs        // This file contains the lowering tests
│   │   └── lower_tests           // This directory contains the lowering tests
│   └── tests.rs
├── lib.rs
└── middle_end
    ├── lir                       // This directory includes the IR.
    │   │                         // You need it because that's the output of your assignment.
    │   ├── associated_impl.rs    // YOU WILL NEED SOME FUNCTIONS FROM HERE.
    │   ├── display_impl.rs
    │   ├── fromstr_impl.rs
    │   ├── id_type_factories.rs
    │   └── misc_impl.rs
    ├── lir.rs
    └── mod.rs
```

- Read all the comments above, and familiarize yourself with the project
  structure.  You will need to understand data structures that are spread over
  different files.

- Cargo.toml is there to tell Cargo how to build the project.

**Do not delete or change any of the files in the root of the project directory,
unless I explicitly told you so.  That may cause your project not to build.**

### JSON and printing ASTs

The `parse` program outputs the AST in a JSON format.  We use this format so
that the parser can save the output in a way that other tools can load.  If you
aren't familiar with JSON, you can learn it from [its official
site](https://www.json.org/json-en.html).  JSON is an extremely common
serialization format that you are probably going to encounter in your
professional life.

You can serialize any data type to JSON that implements the `Serialize` trait
(all of our AST nodes do) to JSON by calling
`serde_json::to_string_pretty(&YOUR_VALUE).unwrap()`.

The structure of the JSON values follow the structure of the Rust types used to
store the AST.  For example, here is a small C♭ program:

```
fn main() -> int {
  print(42);
  return 0;
}
```

Here is the JSON serialization of its AST:

```json
{
  "globals": [],
  "typedefs": [],
  "externs": [],
  "functions": [
    {
      "name": "main",
      "params": [],
      "rettyp": "Int",
      "body": {
        "decls": [],
        "stmts": [
          {
            "Call": {
              "callee": {
                "Id": "print"
              },
              "args": [
                {
                  "Num": 42
                }
              ]
            }
          },
          {
            "Return": {
              "Num": 0
            }
          }
        ]
      }
    }
  ]
}
```

You can see that each enum is converted to a map where the key is the enum tag,
and the value is the values stored inside.  Each enum/struct data is also a map
where the keys are field names and the values are the values of each field.
Vectors are converted to arrays.

**You can use the JSON files as inputs to your lowering program (`cfc`) even if
you don't have a working parser.**

#### Pretty-printing

You can also pretty-print a whole AST (a `Program` object) to "un-parse" it, and
get a string representing the program.  The `prettify` program does that.
That's a good way to test your parser.  If your parser is implemented correctly,
`prettify` should be idempotent (applying it once should do the same thing as
applying it twice to the same input).


### Lvalues

The `lval` nonterminal corresponds to the **lvalues** in the program. An lvalue
is a value that can be on the _left_-hand side of an assignment (that's why they
are called lvalues).  These are expressions that have an actual location in
memory that we can store data in.  That's why they look a lot like some of the
expression cases.

When implementing lvalues, you need to be careful about a few things:
- The LIR code for an lvalue is similar to the code for an expression, except
  that *you skip the last load.*
- lvalues are used in assignments, and the compiler should emit different
  instructions for different lvalues:
  1. if the lvalue is lowered to just a variable (the variable case), then you
     emit a `$copy` instruction for the assignment.
  2. if the lvalue is not a variable, then it is lowered to an address (you
     skipped the last load), in this case we want to _store the data at this
     memory address_, so you should emit a `$store` instruction.

### Rhs

The `rhs` nonterminal stores all the possible right-hand sides for an
assignment. This includes all expressions as well as `new`.  So, unlike Java,
`new` in C♭ can only be used in the right-hand side of an expression.  This
isn't a huge restriction (we can use temporary variables to emulate Java's
behavior).

### Types & LIR

**For the LIR reference, see the IR generation lecture notes.**

All of the type definitions in our AST are actually defined in LIR, our
intermediate representation (because we use the same type definitions in both).
Converting the AST to LIR is the topic of the next assignment.  We create types
only using the factory functions (see below).  This allows us to implement
hash consing: the factory functions cache each type, so we have a single `int`
type for example, and the type objects are just pointers.  So, we can create new
type objects cheaply (just look it up in the cache), use less memory, and
compare & hash type objects cheaply too (just compare/hash the pointers).

Also, you will need to use `struct_id`, which hash-conses struct names.  Without
it, you cannot create struct types.  We will use similar hash consing functions
also when building the IR generator in the next assignment.

Note that `func_ty` takes the return type first, then the list of parameter
types.

### LIR identifiers

All identifiers in LIR are hash-consed, so you need to youse factory functions
to create them.  These are defined in `id_type_factories.rs`.

## What you need to implement

Part 1:

- All cases of `lower_exp_to_operand` **except `And`, `Or`, `Call`.**
- `lower_lval`.
- `lower_assign`.
- `lower_if`.
- `lower_while`.
- `eliminate_inits`.  This pass converts `let x: int = e, y: int = f;` to `let
  x: int, y: int; x = e; y = f;`.  So, it removes the initializers in local
  variable declarations.  Then, the rest of lowering doesn't need to worry about
  this.
  
Part 2:

- `lower_call`.
- remaining cases of `lower_exp_to_operand`.
- `eliminate_multiple_ret`.  Converts functions with `$ret` instructions to a
  functions with a single `$ret` at the end.

## Helpers available to you

- You need to use the type factory functions `func_ty`, `int_ty`, etc. to create
  `Type` objects.  These objects use a particular optimization called **hash
  consing** to make them cheap to create and compare.
- Similarly, you need the factory functions `func_id`, `var_id`, `field_id`.
  Note that variable and field IDs also store the type.
- The `Lowering` data structure has a few helper methods you can use:
    + `reset` resets all local counters and information.  This needs to be done
      only at the beginning of a function.
    + `create_tmp` creates a temporary variable with given prefix.  My compiler
      uses the `_t` prefix for temporary variables.  This is similar to the
      `fresh` method for the arithmetic language.
    + `create_bb` creates a new basic block ID.
    + `name_to_var` looks up a name.  This is similar to how we did lookup in
      the arithmetic language.
    + `is_extern` checks whether a name is an `extern` function.
    + `is_internal_func` checks whether a name is a function defined in the
      program (rather than a local variable or a parameter).
    + `get_field_by_name` looks up a struct field in the type definitions,
      ignoring the type.
- Other helpers are:
    + `add_inst` adds an instruction to the end of the given block.
    + `set_terminal` sets the terminal instruction of a block.
    + `reset_terminal` resets the terminal instruction of a block to a sentinel
      value.
- Associated functions for LIR objects.  These are defined in
  `associated_impl.rs`.  One important example is `get_deref_type()` which
  returns the pointee type for a given pointeer type (it returns `t` when called
  on `&t`).
      
### Sentinel values

A sentinel value is a value we put somewhere to mark it as a special case.  In
this case, there is a special basic block name we reserve (`_SENTINEL`).  This
block is never created, so a `$jump _SENTINEL` instruction is something we put
to create a basic block without knowing the terminal instruction.  After filling
in the basic block, you need to figure out what the terminal is, and then you
can set the terminal.

## Eliminating initializers in declarations

`eliminate_inits` converts declarations with initializers like:

```rust
fn main() -> int {
  let x: int = 3, y: int, z: int = x;
  ...
}
```

into assignment statements like:

```rust
fn main() -> int {
  let x: int, y: int, z: int;
  x = 3;
  z = x;
  ...
}
```

## Lowering simple expressions (everything except `and`, `or`, and calls)

`lower_exp_to_operand` works similarly to our in-class exercise with the
arithmetic language.  However, there are a few things to keep in mind:

- lowering expressions can create new basic blocks, so this function takes the
  current basic block, and returns the basic block *after the code for this
  expression*.
- for part 1, you won't create new basic blocks, but you need to assume that the
  subexpressions you recursively generate code for can create new basic blocks
  (as they will do this in part 2).

## Lowering statements

Each statement lowering function takes the parts of the relevant statement's AST
node, the function body (so that it can insert instructions), and the current
basic block's label.  Things to note:

- these functions return a basic block label because they **may** create new
  basic blocks (similar to the reasoning in `lower_exp_to_operand`, we did
  several examples of this in class).
- for part 2, you may skip generating some blocks when all branches of execution
  just return from the function--so that you don't generate dead code.  in this
  case, lowering the statement would not return a basic block to continue
  emitting the rest of the code.

## Lowering `and` and `or`

We lower them like if statements.  The cases for them have a sketch of what the
code for them should look like.

## Lowering function calls

You need to handle several things:
- evaluate arguments, this may create basic blocks
- handle extern vs non-extern functions separately.  non-extern functions create
  a new basic block due to LIR's design.
- handle direct vs indirect calls.
- make sure to store the result to a left-hand side variable when needed.

## Eliminating multiple return instructions

A correct LIR function has to have **exactly 1 return instruction.**
`eliminate_multiple_ret` should rewrite the return instructions into jumps to a
special basic block named `exit` to give you this guarantee.  You need it only
for part 2.

## Tips & gotchas

Recommended path for part 1:
- arithmetic expressions.
- assignment with basic lvalues (just variables) and expressions.
- comparison expressions.
- `eliminate_inits`.
- assignment involving `new` on the right-hand side.
- pointer-related expressions.
- remaining lvalues.
- conditionals.
- loops.

- You need to make sure that the types you generate for any temporaries are
  correct.  This is especially important when you are dealing with generating
  `$alloc` instructions.
- You need to be careful with managing the basic block you're adding to.
- Use `set_terminal` for setting the terminal, that helps catch some bugs where
  you overwrite a terminal.
- You should use helpers to look up variable names, figuring out direct,
  indirect, vs. extern calls, etc.
- (part 2) You should be careful with handling multiple returns, returning from
  a loop, which loop exit to jump to for `break` and `continue`.
- To get a `LirType` from a `Type` object `t`, use `&*t.0`.

## Test cases

You are given a lot of tests.  The ones relevant to grading are `lower_tests`.
Other tests are there to test the template and the lexer.

### Part 1

You need to pass `part1_basic` for 1/2 points, `part1_second_point` for the
second point.  There are no hidden or generated tests for part 1.

### Part 2

You need to pass `part2_basic` for 1/2 points, `part2_second_point` and the
tests on the autograder for the second point.

**These tests will be released along with part 2's official release.**

### Test helpers

The tests uses the `lower_and_run` helper. `lower_and_run(code, result)` does
the following:
1. It calls the parser to

3 functions to check how your parser should behave:
- `prettifies_to(input, output)`, checks that when we run the parser run on the
  input program, then pretty print the result, it should be the same as the
  given output.
- `parse_and_prettify(code)` basically acts as `prettifies_to(code, code)`, so
  pretty-printing the parser's result should be the same as the input.
- `fails_to_parse(code)` checks whether the parser has failed to parse the input.

## Testing your code

You can run all tests you are given by running `cargo test`.  You can run
specific tests by giving a substring to `cargo test`.  For example,

- you can run the `accept1` test using the command `cargo test ::accept1`.
- you can run all `accept` and `run_till_accept` tests using the command `cargo
  test accept`.

- See the Grading section for how grading works.

## Running your code on an input file

You can run `cfc`, `parse` and `prettify` programs using Cargo (the latter two
require your parser).  To run `cfc` on an input program, use `cargo run --bin
parse -- my-cflat-program my-lir-program`.  For example, if your program's AST
is in afile called `test.json` and you want to save the LIR program to
`test.lir`, you can run

```
cargo run --bin cfc -- test.json test.lir
```

### Hooking up your parser

If you have your parser, you can copy it to `parse.rs`, then you can use your
compiler just like the reference compiler (`cfc`) on the vlab machines.  That
is, you can run `cfc` on normal C♭ programs (not just JSON files).  In this
case, you can run your compiler like so:

```
cargo run --bin cfc -- test.cb test.lir
```

**Notice the file extension!** `cfc` looks at the extension of the input and the
output to determine whether to run the parser.

### Using my parser (only for Linux or WSL)

Download the `parse` program from `~memre/parse` on vlab machines.  Then, you
can run my parser to generate a JSON file for the AST which you can use with
your compiler:

```
./parse my-program.cb my-program.json
cargo run --bin cfc my-program.json my-program.lir
```

If you don't have Linux, you can run the parser on the vlab machines and
download the JSON file.

## Running the LIR interpreter

You can run `liri` using Cargo to execute a LIR program.  I'll use a modified
interpreter in the autograder.  Here is an example:

```
cargo run --bin liri -- test.lir
```

`liri` will interpret the LIR program.  It supports all LIR instructions except
`$addrof` and `$phi` which we don't need for this class.  It supports only a
couple of external functions to call.

## Reference compiler

There is a reference implementation on vlab machines that you can use and
compare against your interpreter.  It is `~memre/cfc`.  You can run it like how
you run your programs (except there is no `--`).  See assignment 1 readme or
Homework 4 for how to run such tools.

### Why am I seeing all these warnings?

The helpers aren't used without your implementation.  As you implement the
project, you will use them and the warnings will go away.  If you come up with a
solution that doesn't use some of the helpers, you can remove the helpers that
you don't use.

## Debugging your program

If you build your program in debug mode (which is the default), then you can use
a debugger like GDB and LLDB.

I also recommend writing regression tests as you discover bugs.

## rustfmt

Starting with this assignment, your code must be properly formatted (otherwise,
you get an automatic 0).  You can do this easily by running `cargo fmt` **after
you save your changes, and before committing them.** So, you don't need to
format anything manually.  The autograder will check if there are any formatting
issues in your code.

This is standard practice in software engineering, to make sure there is one
consistent style across different people's contributions to the same code base.
You are starting to have a large code base, so this is useful for making your
code readable by the instructional staff, as well as you one week later.

## Grading

### Part 1

To get 1/2, your code needs to:

- Pass all the tests in `part1_basic.rs`.
- There are other tests in the assignment template, you can ignore them.
- Be properly formatted (see the rustfmt section).

To get 2/2, your code needs to do the above, and:

- Pass all the tests in `part1_second_point.rs`.

### Part 2

To get 1/2, your code needs to:

- Pass all part 1 tests.
- Pass all the tests in `part2_basic.rs`.
- There are other tests in the assignment template, you can ignore them.
- Be properly formatted (see the rustfmt section).

To get 2/2, your code needs to do the above, and:

- Pass all the tests in `part2_second_point.rs`.
- Pass 80% of the random tests (hidden from you).
- Pass the hidden handwritten tests.

### Automatic 0

Under the following circumstances, you'll automatically get a 0:
- If your code does not compile.
- If your code does not finish running within the allocated time limit (20
  minutes) on the Docker container.  **This includes the programs your compiler
  produces.  So, if your compiler generates a program with an infinite loop you
  get a 0.**
- If you encode the test cases in your solution rather than fixing the bugs they
  expose.

### The autograder

**The autograder for part 1 will be released after Fall break.**

The autograder will use only `front_end/lower.rs` and it will throw
away any other changes you made to the program.  So make sure not to change
anything else.

Also, the autograder will run the program in **release mode** for performance,
so that your solution is less likely to time out.  Normally, the debug vs
release mode shouldn't affect your program's behavior (except that unintended
integer overflows crash the program in debug mode), but I recommend testing your
program in release mode.  See the Cargo manual on how to build in release mode.

### Clippy & warnings

In order to get 2/2, your code should compile with no warnings either from
rustc or from clippy.  You can check this by running `cargo clippy`.

### Interactive grading

After you are done with the assignment, you can schedule a 15 minute grading
session with me, and you will explain your solution along with my follow-up
questions:
- If your answers are satisfactory, you will pass interactive grading.
- If your answer is unsatisfactory (you cannot explain _why_ your code works,
  and the alternatives you explored), you will _not_ pass interactive grading.

**See the syllabus for the number of interactive grading sessions to pass for
each grade level.**
