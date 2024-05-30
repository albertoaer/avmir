# Avmir
**A**nother **v**irtual **m**achine **i**n **r**ust

This project is only intended to be a toy VM to write languages and programs on. The purpose is to play with memory, instructions and performance while having fun.

## Features

- `Assembly style like parser supporting memory and instructions tagging`
- `Safe concurrency`
- `Shared Memory`
- `Memory mapped files for IO and interprocess operations`

## Usage

**avmir *[OPTIONS]* *[FILES]*...**

Options:
- `-m size` shared memory
- `-m size:path` shared memory mapped file as memory
- `-l library` load a ffi library

Every file will be parsed as an independent program and run in a different thread

## Design

The `machine` can run an arbitrary number of `processes`. Processes are abstracted away from the machine through the `process supervisor` which provides memory units, and the capability to be forked. A process is instantiated from a single `program`.

A single `process` contains:
  - the `program` itself
  - the parsed `instructions` mapped from the `program`
  - the program counter (`pc`)
  - `stack` with *32* elements capacity
  - `registers`: from *[0, 9]* shared with ffi, *[10, 13]* reserved for flags, *[14, 23]* for the process private usage
  - the name of the ffi function prepared to invoke (`invoke_target`)

The `program` contains not only a vector of `instructions` but metadata and a initital memory state to provide for example *strings* in the compilation process.

The `process supervisor` is in charge of providing ffi functions and memory to the process. There is one memory prepared for every process and a variable number of memories that can be accessed by many process to read/write concurrently.

## Parser

Currently there is implemented a `Parser` for very simple Assembly like source files. The current implementation is the [v2](src\parser\simple_v2.rs) supporting tags for both memory chunks and instructions that can be used as operands.

### Hello World Program

```
; declare ffi function name and message in memory
print #std_println
message #hello world!

; enable memory share in special registry 10
RegSet 10 1

; set address and size of the message
RegSet 0 $message
RegSet 1 @message

; perform the invocation with the address and size of the function name
FastInvoke $print @print
```

## Run

First build all in order to get the std dynamic library compiled
```
$ cargo build --all
```

Then run the hello world using the compiled library under the `-l` flag
```
$ cargo run examples/hello_world.txt -l avmir_std
```

## More examples

Unde the [examples](/examples/) directory there are several examples. The following will run a process writing a number in memory (*loop_write.txt*) and a process reading that number from memory (*loop_read.txt*). Note that they have a limit.

In order to share memory, the vm will be created with a shared memory `-m 1024`. There can be as many memories as you want.
```
$ cargo run examples/concurrent_rw/loop_read.txt examples/concurrent_rw/loop_write.txt -m 1024
```

If you want to run them not only in separated threads but in separated processes, you could consider using `-m 1024:a_common_file.dump` so they use a memory mapped file as the shared memory and that way achive interprocess communication. In this case run the reading process in first place so it does not close.

```
$ cargo run examples/concurrent_rw/loop_read.txt -m 1024:a_common_file.dump
```

```
$ cargo run examples/concurrent_rw/loop_write.txt -m 1024:a_common_file.dump
```

Beware that the file will remain in disk, so if you try to run again, as the last value in the file (also the memory) will be the expected value to exit the loop, the program will have no effect but outputing one value to console.