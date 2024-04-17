# Monkey Language

Interpreter for Monkey Language based on the [Writing An Interpreter In Go](https://interpreterbook.com/) book. This repository contains two implementations - one in Rust and one in Zig.

## Project Structure

This repository contains two implementations of the interpreter. The primary implementation is the one in Rust, which implements more features of the language.

The implementation in Zig was done for learning purposes as my first non-hello world project in Zig. It implements only the basic language features.

## Performance

During the implementation of the interpreter in Zig, I started wondering how the original Go implementation from the book's author compares to the Rust and Zig implementations.

Rust and Zig are not garbage collected, which makes the implementation of the AST evaluation different from the Go implementation since you have to also implement your own garbage collector.

I tested the performance of all three interpreters on a simple program that calculates Fibonacci numbers. It was tested on an M1 MacBook Pro. Here are the results:

| Language | Average Time [ms] | Relative Performance |
| -------- | ----------------- | -------------------- |
| Go       | 4.303             | 1.0                  |
| Rust     | 5.730             | 1.33                 |
| Zig      | 6.397             | 1.49                 |

> [!NOTE]
> The benchmark is not very detailed and was done just to satisfy my curiosity, so it should be taken with a grain of salt.

## License

Project is licensed under the [MIT License](LICENSE).
