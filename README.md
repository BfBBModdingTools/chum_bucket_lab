# BfBB Modloader

This repository contains a WIP mod loader for the 2003 3D platformer SpongeBob SquarePants: Battle for Bikini Bottom. It currently has some ability to automatically apply a custom set of IPS patches given a clean base `default.xbe` executable from the Xbox version of the game. Once sufficient progress is made, this README will be updated and pre-release versions will be build and distributed.

The Goal is to make modding the game as easy and painless as possible for the user and to support a more vibrant ecosystem of modding within the game's community.

## Features

### Current

- Support for .xbe mods.
- Automatic updating of mod list through the internet.
- Automatically apply a set of IPS patches based on a selection of mods given by the user.

### Planned

- Support for .dol mods (For Gamecube and Dolphin Emulator).
- Support for asset file mods (HIP/HOP mods).
- Support for more advanced ASM mods with custom code and data to coexist through a basemod compatibility layer.

## Usage
This section will be updated once the tool is ready to be distributed.

## Building

### Requirements

- [Rust](https://www.rust-lang.org/tools/install)
- For Linux: Druid requires gtk+3. [See Here](https://linebender.org/druid/setup.html)

### Compiling

- If you have satisfied the requirements above, you should simply be able to build and run the program with the `cargo run` command.

## Contributions
Contributions are generally welcomed, though until the project reaches a more advanced state likely will not be encouraged. Code quality PRs are the most likely to be considered at this stage of the project. At this point, the overall vision of the project is still being solidified so contributions in the form of planning and design recommendations are welcome.