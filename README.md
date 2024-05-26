# Avmir
**A**nother **v**irtual **m**achine **i**n **r**ust

This project is only intended to be a toy VM to write languages and programs on. The purpose is to play with memory, instructions and performance while having fun.

## Design

The `machine` can run an arbitrary number of `processes`. Processes are abstracted away from the machine through the `process supervisor` which provides memory units, and the capability to be forked.

A single `process` contains:
  - `registers` *[0, 10)* elements
  - `stack` *[0, 32)* elements
  - `pc` program counter
  - `program`

The `program` contains not only a vector of `instructions` but metadata and a initital memory state to provide for example *strings* in the compilation process.

## Parser

Currently it is implemented a `Parser` for very [simple](src\parser\simple.rs) Assembly like source files. Since features still coming it's yet on develop. However the structure won't get to be as complex as a full compiler with type inference and error detection, beyond the very basic syntax support.