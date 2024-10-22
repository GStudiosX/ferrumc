# Introduction

FerrumC is a reimplementation of the Minecraft Server written in rust.

it is completely multithreaded; and offers high performance as well as amazing memory efficiency!

## Key Features

##### TODO: Add images

- 🧠 Highly efficient memory usage!

- Highly Customizable configuration ⚙️

    ```bash
    host = "0.0.0.0"
    port = 25565
    motd = ["A supersonic FerrumC server."]
    max_players = 20
    network_tick_rate = 0

    world = "world"
    network_compression_threshold = 256

    [database]
    cache_size = 1024
    compression = "fast"

    [velocity]
    enabled = false
    secret = ""
    ```

- 💪 Powerful [Entity Component System](https://en.m.wikipedia.org/wiki/Entity_component_system) or [ECS](https://en.m.wikipedia.org/wiki/Entity_component_system) for short!

- Custom Network and [NBT](https://wiki.vg/NBT) encoding/decoding system. 🔄

- 📦 Full multithreading and proxy support ([Velocity](https://papermc.io/software/velocity))

## Chapters

1. [Getting Started With FerrumC](./getting_started.md)
