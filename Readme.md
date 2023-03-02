I'm learning Rust and porting my previous work of the [TiledMMO](https://github.com/Joey92/TiledMMO) from Typescript to Rust and its Bevy game engine.

# TiledMMO

This project is a client and server combo tiled map multiplayer/MMO game engine. You only need your tiled maps, the rest is handled by the server and client.

Some features (Compared with the port of the [TiledMMO](https://github.com/Joey92/TiledMMO) project) include:

- [x] Multiplayer support
- [x] NPCs via object layer
  - [] properties on the objects configure how the NPC behaves
- [] NPC navigation via navmesh
- [] Portals to other maps
- [] Full scripting support

Upcoming features:

- Combat system
- Quests
- Inventory system
- Dungeons

# Setup

To start the server run `cargo run --bin server`
And for the client run `cargo run --bin client`

The ports are not really configurable yet, and the server and client are both assumed to be running on localhost.

## Server / Client

- The server does not need any database storage, up to now.
- The server and client both use the Bevy ECS

## Maps

The maps are created via the Tiled map editor. This Software comes with some example map that uses the features of the server/client.
