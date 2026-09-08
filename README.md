# rust-snake

cli snake game built with rust with a mini game engine I built to practice core gamedev and rust fundamentals.

## Setup

have the necessary deps at [mise.toml](mise.toml), you can also run:

```bash
just bootstrap # if necessary, installs box deps
just setup
```

## Playing

to play:

```bash
just play
```

controls:

| Action | Key |
| -------- | ----- |
| movement | arrow keys |
| pause/unpause | P |
| reset | R |
| quit | Q |

## Notes

The initial goal was to build a snake game without any deps, but rendering on the TUI with colors and with cross platform support (bc I worked on and off on both a linux box and a windows box) proved to be too time consuming, so I used crossterm and a few other crates.

some primitives I needed to built, like a [signal bus](crates/snake-core/src/signal.rs) to handle reactive state and an [ascii renderer, using crossterm](crates/snake-game/src/infra/crossterm_renderer.rs) could obviously be much much simpler by using existing tooling.
for those 2, Bevy to handle the game loop state could be excellent, or ratatui for the terminal renderer.

still, it was a really fun experiment to build it. might add a few other things to it in the future.
