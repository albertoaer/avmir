# Avmir
**A**nother **v**irtual **m**achine **i**n **r**ust

This project is only intended to be a toy VM to write languages and programs on. The purpose is to play with memory, instructions and performance while having fun.

## Design

The `machine` can run an arbitrary number of `processes`. Processes are abstracted away from the machine through the `process supervisor` which provides memory units, and the capability to be forked. A process is instantiated from a single `program`.

A single `process` contains:
  - the `program` itself
  - the program counter (`pc`)
  - `stack` with *32* elements capacity
  - `registers`: from *[0, 9]* shared with ffi, *[10, 13]* reserved for flags, *[14, 23]* for the process private usage
  - the name of the ffi function prepared to invoke (`invoke_target`)

The `program` contains not only a vector of `instructions` but metadata and a initital memory state to provide for example *strings* in the compilation process.

## Parser

Currently it is implemented a `Parser` for very [simple](src\parser\simple.rs) Assembly like source files. Since features still coming it's yet on develop. However the structure won't get to be as complex as a full compiler with type inference and error detection, beyond the very basic syntax support.

### Hello World Program

```
#std_println
#hello world!

RegSet 10 1
RegSet 0 $1
RegSet 1 @1

FastInvoke $0 @0
```