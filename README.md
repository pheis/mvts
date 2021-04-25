# TSmv: like mv but for TypeScript files

TSmv is a a tool for moving TypeScript files and updating their relative imports. This is WIP but works on my machine :).


## Cloning the repo
This repo depends on tree-sitter-typescript repo. When cloning remember to also clone submodules with '--recursive-submodules'-switch.

`git clone --recursive-submodules https://github.com/pheis/tsmv`

## Building

`cargo build`

## usage

TSmv takes two arguments: source_file_path and target_file_path.

`tsmv source_file.ts some_folder/another_folder/moved_source_file.ts`

TSmv moves source file to target_file_path and edits it's imports so that they are correct in the new location.
