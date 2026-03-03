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

The grammar syntax is derived from [Augmented Backus-Naur form](https://en.wikipedia.org/wiki/Augmented_Backus%E2%80%93Naur_form), but made a bit simpler and cleaner.

The grammar language used in this project, that will just be called CFG here for simplicity, features 4 types of elements: rules, terminals, non-terminals, and operators.

### Rules
Rules are a reusable sequence of values.
You define them as such:
```
rule:
    | value
    ;
```
The identifier, 'rule' in this case, has to be lowercase, as to avoid confusion with terminal values.
Then to use a rule in another rule, simply use its identifier. e.g:
```
rule2:
    | rule1
    ;
```

### Terminal values

A value is terminal if it doesn't refer to any other rule, as they match with literals.
An example of a terminal value is a string `"foo"`, or a number `91`, or ranges `[0-9]`.
There are also builtin terminal values, such as `IDENT` for an identifier. 
(You can also specify your own terminals with the `-T` flag)

### Operators

#### OR
The OR operator picks


## ROADMAP
- [ ] Limit recursion/depth of generation
- [x] Add custom term declarations
- [x] Range support to CFG
- [x] Fix comments in CFG
- [ ] Add documentation for CFG syntax
- [ ] Fix inconsistent spacing between generated strings
    - [ ] Decide the rule for spacing
- [ ] More robust syntax and errors
    - [ ] Lexer errors
    - [ ] Error on no ';' after last rule
    - [x] Show actual row and column in error message
- [ ] Set up tests
    - [ ] Lexer tests
    - [ ] Parser tests
    - [ ] Generation tests
- [ ] Metadata
    - [ ] Set probability in ors
    - [ ] Set number of repititions
    - [ ] Set 'no-space'
- [ ] Add examples of CFG's
