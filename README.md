# mvts: like mv but for TypeScript files

mvts is a a tool for moving TypeScript files and updating their relative imports and affected files. This is WIP but works on my machine :).


## Usage

mvts takes two arguments: source_file_path and target_file_path.

`mvts source_file.ts some_folder/another_folder/moved_source_file.ts`

mvts moves source file to target_file_path and edits it's imports so that they are correct in the new location. mvts finds all files that import moved file and update them accordingly.

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
