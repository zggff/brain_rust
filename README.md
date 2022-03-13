# BrainRust
Simple brainfuck compiler written in Rust.

## Goals
- [x] interpretation
- [ ] compilation to assembly
    - [x] compilation to arm64 assembly
    - [ ] compilation to x86_64 assembly
- [ ] native compilation via llvm

### Compilation tutorial
Currently compilation is only viable on apple silicon.
To compile the generated assembly use the following commands:
```console
$ as main.s -o main.o
$ ld main.o -o main -lSystem -syslibroot `xcrun -sdk macosx --show-sdk-path` -e _start -arch arm64  
```
