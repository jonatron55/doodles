Miscellaneous Rusty Doodles
===========================

This repository implements several small terminal-based graphical programs in Rust.

`bubble`
--------

A bubble sort animation with customizable colors and rendering styles.

`conway`
--------

An implementation of [Conway's Game of Life] (cellular automata) with coloured cells. In addition to the usual rules of
Conway's Game of Life, this implementation adds colour to the cells and modifies the rules slightly:

1. Any living cell with fewer than two live neighbours **of the same colour** dies, as if by underpopulation.
2. Any living cell with two or three live neighbours **of the same colour** survives.
3. Any living cell with more than three live neighbours **of any colour** dies, as if by overpopulation.
4. Any dead cell with exactly three live neighbours **of the same colour** becomes a live cell, as if by reproduction.

Boards can be loaded from a file. If no file is provided, a random board will be generated.

`digirain`
----------

A "Matrix"-style digital rain animation. Colours can be specified as can the character set and length of the trails.

`maze`
------

Generates random mazes using a randomized depth-first search. Once the maze is generated, multiple agents will attempt
to solve it using their own depth-first search.

Different rendering styles are available for the maze and the solving agents.

Building and Running
--------------------

- To install Rust, first install [rustup], and then run `rustup update` to get the latest stable version of Rust. You
  may need to close and reopen your terminal before new commands are available.
- To build the project, navigate to the project directory and run `cargo build --all-targets` (for debug builds) or
  `cargo build --all-targets --release`  (for optimized builds).
- The output will be placed in the `target/debug` or `target/release` directory, respectively.
- To run a specific target, use `cargo run --release --bin <bin>`, replacing `<bin>` with the name of the desired
  target.
- For detailed usage information and command-line options for each program, run
  `cargo run --release --bin <bin> -- --help`.

[Conway's Game of Life]: https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life
[rustup]: https://rustup.rs/
