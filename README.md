# CFG-FUZZER

>⚠️ THIS IS IN EARLY DEVELOPMENT, HAS BUGS, AND IS NOT FEATURE COMPLETE ⚠️

This program aims to create random, but parsable, code.
It can do this for any language that can be described in a Context-Free-Grammar (CFG).

If the CFG is done correctly, the program assures that the generated code can be parsed. This can help to test a lexer and parser for toy languages.
The program does not, however, guarantee that the generated code can be compiled or interpreted.
So this may not work for languages where there is a tight relationship between the parser and static analysis/codegen/interpreter, including languages like C that use the [Lexer hack](https://en.wikipedia.org/wiki/Lexer_hack).

## USAGE

To use this program, first create a CFG file (docs for syntax is further down), then call the program, with the CFG file as an argument. You also need to specify the starting rule with the `--start` flag.
An example is:
```sh
$ cfg-fuzzer c.cfg --start code
```

## INSTALLATION

To install the program, clone the repo locally
```sh
$ git clone https://github.com/AtleSkaanes/cfg-fuzzer.git
```
And then compile it with Cargo
```sh
$ cargo build --release
```
Alternatively, you can install it on your machine through cargo with
```sh
$ cargo install --git https://github.com/AtleSkaanes/cfg-fuzzer.git
```

## CFG-SYNTAX

**🚧 UNDER CONSTRUCTION 🚧**

## ROADMAP
- [ ] Limit recursion/depth of generation
- [ ] Add custom term declarations
- [x] Range support to CFG
- [ ] Fix comments
- [ ] Add documentation for CFG syntax
- [ ] Fix inconsistent spacing between generated strings
    - [ ] Decide the rule for spacing
- [ ] More robust syntax and errors
    - [ ] Lexer errors
    - [ ] Error on no ';' after last rule
    - [ ] Show actual row and column in error message
- [ ] Set up tests
    - [ ] Lexer tests
    - [ ] Parser tests
    - [ ] Generation tests
- [ ] Metadata
    - [ ] Set probability in ors
    - [ ] Set number of repititions
    - [ ] Set 'no-space'
