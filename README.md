# winter

A general opengl wrapping framework

## Usage

The library is currently being reworked, so tags and refs may change in the future

First, clone the primary branch of the repository,
```
git clone https://github.com/fourrth/winter.git
```
It is reccomended that you delete the remote after cloning due to frequent structure changes. You can do this by running:
```
git remote remove origin
```

Currently, winter is not very usable for other crates. The only supported use is to use the library through the provided examples.
Another thing is that currently the function loader is bundled in the repo, however this may change in the future. Please refer to this readme for more deatils on building/usage.

## Examples

This crate uses examples to provide a good overview of what is possible with this library. This includes more complete projects, along with some examples for starting projects

Examples may be ran in the usual way by running:
```
cargo run --example example1
```

The following are some of the *more interesting* examples:

### snake-framework

This is a pretty simple implimentation of the game snake

Here are some quick notes and features:

 - *KEY_ESCAPE* closes the window
 - *KEY_ENTER* starts the game
 - *KEY_W*, *KEY_A*, *KEY_S*, *KEY_D* (along with the arrow keys) move the snake around
 - Allows you to run into your self if you go in the opposite direction you currently going
 - Infinite bounds, so no running into walls
 - Secret debug action if you find the key to do it

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

## Compatibility

This crate is used and tested with Rust 2021 and `rustc` 1.60+

## License

This crate — along with any subsequent additions or revisions — are all dual licensed under [MIT License](LICENSE-MIT) or [Apache License](LICENSE-APACHE) at your option.