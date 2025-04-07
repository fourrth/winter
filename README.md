# winter

A general OpenGL wrapping framework

## Usage

The crate is currently being reworked, so tags and refs may change in the future

First, clone the primary branch of the repository and cd into the dir,
```
git clone https://github.com/fourrth/winter.git
cd winter
```

Now you may use cargo like any other project, for example:

```
cargo r --example snake-framework -- 1200 1200
```

Currently, winter is not very usable for other crates. The only supported use is to use the crate through the provided examples.
Please refer to this readme for more details on building/usage

### OpenGL

This crate targets OpenGL gles 3.2 core, however the bindings will almost certainly work 
if you have OpenGL gl 4+

## Features

This crate uses features to switch on and off certain parts of the crate.
The following is a comprehensive list of all the features this crate offers:

### raw

This feature simply switches everything off, other than the raw submodule of this crate

## Examples

This crate uses examples to provide a good overview of what is possible with this crate. This includes more complete projects, along with some examples for starting projects

Examples may be ran in the usual way by running:
```
cargo run --example example1
```

The following are some of the *more interesting* examples:

### snake-framework

This is a pretty simple implementation of the game snake

Here are some quick notes and features:

 - *KEY_ESCAPE* closes the window
 - *KEY_ENTER* starts the game
 - *KEY_W*, *KEY_A*, *KEY_S*, *KEY_D* (along with the arrow keys) move the snake around
 - ~~Allows you to run into your self if you go in the opposite direction you currently going~~. I actually run into this on accident a lot, so I actually bothered to fix this one
 - Infinite bounds, so no running into walls
 - Secret debug action if you find the key to do it
 - Simple move buffer

### game_of_life-framework

This is Conway's Game of Life

Not very complete, but has more features than snake:

 - *KEY_ESCAPE* closes the window
 - *KEY_ENTER* resets the current state
 - *KEY_BACKSLASH* saves the current state
 - *KEY_BACKSPACE* changes how the game will reset
   - basically, if you press *KEY_ENTER* normally, it generates the new state randomly. However, if you press backspace, you will load the state you saved previously
   - Note this also provides the great feature that it will crash if you have not saved before trying to load a saved state
 - *KEY_LEFT* and *KEY_RIGHT* decrease and increase the generations per second respectively 

### v1.5

This update is currently almost complete with an estimated release of
some time in the end of April 2025.
This will make the crate more generic and it will
allow more access to the inner workings of the framework.
Not only that, serialization and deserialization will be completed
so that one may export and import data from other sources.

### v2.0 and Beyond

v2.0 and anything past it will more-or-less be the final form of the crate.
The end goal of this crate is to split it up into multiple different ones
which fill various users' needs.

For example, the main crate here will be the engine and raw code,
which will mostly be traits. However, the main context will reside here 
and will utilize the aforementioned traits. This allows easy access
to the engine code, which would allow someone to build their framework
with specific access to particular engine code.

Various implementation crates will be provided which allow the user
easy access to use the engine. The user may also give their own
implementation to more better suit their needs.

## Compatibility

This crate is used and tested with Rust 2021 and `rustc` 1.60+

## License

This crate — along with any subsequent additions or revisions — are all dual licensed under [MIT License](LICENSE-MIT) or [Apache License](LICENSE-APACHE) at your option.