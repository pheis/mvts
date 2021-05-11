# mvts: like mv but for TypeScript files

mvts is a a tool for moving TypeScript files and updating their relative imports and affected files. This is WIP but works on my machine :).


## Usage

mvts takes two arguments: source_file_path and target_file_path.

`mvts source_file.ts some_folder/another_folder/moved_source_file.ts`

mvts moves source file to target_file_path and edits it's imports so that they are correct in the new location. mvts finds all files that import moved file and updates them accordingly.

### Screenshot of git status after using mvts
![A screenshot of a sample move with mvts](screenshot.png?raw=true "Screenshot of git status after using mvts")

## Installation

mvts can be installed with cargo

`cargo install mvts`

## Building

```
$ git clone https://github.com/pheis/mvts
$ cd mvts
$ cargo build --release
$ ./target/release/mvts --version
0.2.0
```

## Features and missing features

- [x] Handling relateive node import statements (no .ts/tsx suffix)
- [x] Moving single ts/tsx file and updating it's imports
- [x] Updatating affected files imports to moved file
- [x] Parallel processing
- [x] Renaming folder containing multiple files
- [x] Handling Deno imports (with .ts/tsx suffix)
- [x] Handling .svg etc imports
- [x] export * statements
- [ ] require ( ) functions
- [ ] import ( ) functions
- [ ] Handling glob patters in argumentsts
- [ ] Finding root based on git, package.json etc patterns if feasible
