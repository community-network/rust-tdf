# Rust TDF
###### This version is not the release yet and can be unstable

Public Open Source library to serialize and deserialize TDF format binary tree into Rust structs/primitives or any other format.

This library is based on work made by other authors and reverse engineering, key features are:

* Better Labels
* Support for negative numbers
* Written fully in Rust
* Direct to-rust serialization and deserialization 

In core of it's implementation lies Tokens and stream of tokens which indicate about startings and endings of collections, followed types, and primitive typed data containers. By flattering data, we improve performance and avoiding problems caused by nested types. It also gives us possibility to ser/des data not only into Rust components but in formats like JSON, XML, etc.

#### What is TDF format?

TDF is an Electronic Arts own binary format to transmit data between games made via Frostbite and so called Blaze Hubs, main backend servers for games.

#### Does it include pre-defined game packets?

Shortly no. Packets are different depending on game, and they are considered to be EA intellectual property, so we are unable to share them with everyone. You have to implement them your self by listening "in the middle". You also would need to reverse engineer packet headers yourself, they are usually 16 bytes long.

#### With what games it is working?

We tested it on Battlefield 1 and 5, but I suppouse it should work on any game under frostbite3 engine.

### Under Apache 2.0 License
