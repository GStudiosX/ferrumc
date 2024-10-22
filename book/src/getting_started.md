# Getting Started

How about we get started shall we?

First of all before we can start using FerrumC! We need to install the prerequisites if you are compiling from source.

## Prerequisites

- Rust Compiler (latest nightly version)

- Cargo

If you don't know how to install rust this isn't really the guide for that.
[See This Instead](https://www.rust-lang.org/tools/install)

Make sure that you customize the installation and get nightly Rust as that is required.

## Installation

Now we get to the fun part installing and running FerrumC.

There are a few ways we can install FerrumC the simplest way is to get builds from our [Github Actions](https://github.com/ferrumc-rs/ferrumc/actions?query=branch%3Arewrite%2Fv3).

### Option 2 (Compile From Source)

The second option is to compile from source this may take a while but the first step is to clone the FerrumC[^1] repository.

```bash
# Clone the repository
git clone https://github.com/Sweattypalms/ferrumc
cd ferrumc
```

After that we can then compile the source with cargo
[make sure you have installed the prerequisites](#prerequisites)

```bash
# Build the project
cargo build --release
```

Now wait for that to compile this can take a few minutes.

### Option 3 ([Docker](https://www.docker.com/))

The final option of running FerrumC this one is pretty simple if you already have docker installed!

You just have to run this single command to get started.

```bash
docker run -d -p 25565:25565 -v ferrumc/ferrumc-example:latest
```

##### Wow almost done

Well that seemed a little easy but don't fret there is still one last thing
if you didn't choose the docker option.

## Running FerrumC

Theres not much to do if you use Option 3 but if you downloaded the build from our Github Actions you can just
run it from your command line and that will create the config and everything for you.

If you choose to compile from source copy the executable file `target/release/ferrumc` (.exe on Windows)
into any folder you want to start the server in then run the excutable.

Thats it your done. But there's still other chapters to learn more or configure FerrumC how you want!

[^1]: As the time of writing this the current branch is `rewrite/v3` make sure you switch to it. Use `git checkout rewrite/v3`
