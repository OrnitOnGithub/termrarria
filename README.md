# Termrarria (temporary name)

This is a terraria-like game made to be played in the terminal.

## Build and Play

(This was never tested outside of Linux)

You will need Cargo to compile this project..

- `cargo run` to build and run
- `cargo build` to just build. Find the executable at `target/debug/termrarria`

### Controls

Move around using W, A, S, D keys.

### Customisation

You can change the top-level constants and statics in `src/main.rs` to change things such as:
- Game "window" size
- World size (requires generating a world of such size using `mkworld.py`)
- Extra debug info on/off
- etc...

You can also play around with `mkworld.py` to generate a custom world.

### Contribute

Any help would be greatly appreciated! The game currently needs help with the following:
- Ideas for game/gameplay
- Better controls and keypress handling (while maintaining zero external dependency philosophy)
- A proper name and identity