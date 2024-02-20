# HEY! 
You're probably reading this to figure out how to get everything working!  
If so then click [HERE!](#Setup).  

## Table Of Contents
1. [Installation](#Setup)
2. [Compiling](#Compiling)
3. [Cross-Compiling](#Cross-Compiling)

# Setup
I'm not good at writing guides so it's probably best you look at the [official installation guide](https://www.rust-lang.org/tools/install)
# Compiling
For this application, the compiling process can be shortened to two commands. These commands differ in if the outputted file is a debug file
or a release file.  
Whats the difference?  
Release builds are optimised, are faster to execute, but take longer to produce.  
Debug builds are unoptimised, run slower, but are quicker to produce.  
## Debug
If the program is being used for debugging then the following may be used:
```bash
cargo build
```
The output can be found in the Debug folder.
## Release
For a release build, use the following:
```bash
cargo build --release
```
The output can be found in the Release folder.
# Cross-Compiling
Quick guide on compiling for an operating system that differs from the one you are currently using (e.g. Compiling for Windows whilst using Linux).  
## Installation
### Windows 
#### x86-64 
```bash
rustup target add x86_64-pc-windows-gnu
```
OR 
```bash
rustup target add x86_64-pc-windows-msvc
```
#### ARM
```bash
rustup target add aarch64-pc-windows-msvc
```
#### Differences?
Typically desktops are x86-64, windows tablets are likely to be ARM.  

GNU, the first option, is going to be similar to a Linux environment, whilst MSVC is the most compatible with Windows. 
### Linux
```bash
rustup target add x86_64-unknown-linux-gnu
```
### Mac
#### Non-ARM
The following should be ran to install the tools for compiling for pre-M1 Macs
```bash
rustup target add x86_64-apple-darwin
```
### ARM
Anything M1 and newer
```bash
rustup target add aarch64-apple-darwin
```
### Other
Any other targets may be found [here](https://forge.rust-lang.org/infra/other-installation-methods.html#other-ways-to-install-rustup)
```bash
rustup target add <TARGET>
```
## Building / Compiling 
Cross-compiling can be performed with a single command, with the final arguement changing depending on your target.  
```bash
cargo build --target <TARGET>
```
With the <TARGET> being the arguement after the "add" when installing with `rustup target add <TARGET>`
### Examples
#### ARM Mac 
Building for an ARM Mac from a non-arm Mac can be performed with: 
```bash
cargo build --target aarch64-apple-darwin
```
#### x86-64 Windows GNU 
```bash
crgo build --target x86_64-pc-windows-gnu
```
