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

The TDF Data transmitted in packet body:

| Type Byte | Name       |
| --------- |:---------- |
| 0_u8      | Uint       | 
| 1_u8      | String     | 
| 2_u8      | Blob       |
| 3_u8      | Map        | 
| 4_u8      | List       | 
| 5_u8      | PairList   | 
| 6_u8      | Union      | 
| 7_u8      | IntList    | 
| 8_u8      | ObjectType | 
| 9_u8      | ObjectId   | 
| A_u8      | Float      | 
| B_u8      | Time*      | 
| C_u8      | Generic    | 


\* Not implemented


##### Uint primitive type

Unsigned integer, from 1 up to 8 bytes in length (encoded to u64).
Compressible number. 

Note: There might be an error in previous reverse engeneering work, so that the second 
(left to right) controll bit might be used as a sign. This should be researched in future.

The leading 2 bits (left to right) in first byte considered to be controll bits. 
If first bit is set to 1, we should continue reading next byte, otherwise - it is the last byte. 
Simmilar to all other bytes, but with only one controll bit. After we read the last byte, we should
concatinate them without controll bits, so the last byte we received will be the first byte in the be 
number. So, the first byte is masked by 0x3F, second byte is masked by 0x7F and shifted by 6 bits to left.
All the next bytes masked by 0x7F and shifted by 7 bits to left.

##### String primitive (zero-teminated):
Consist of
* Length (compressed Int) of string, including terminating zero. 
* ASCII string (bytes maybe invaid)
* Zero-byte termination

##### Blob, or raw byte array, consist of:
* Length (compressed Int)
* Raw bytes of the length

##### Map - named sequence.
Consist of zero, one or multiple elements and terminated by zero byte at the end.
Each element has such a structure:
* Label (3 bytes)
* Type byte
* Data of type

**Important:**
It is required to peek 1 byte before each Label and check if it is not 2. 
First label byte can't be a 2, so if it is 2 - This struct considered to be a union-like struct, 
made for transmitting network details, and is supposedly always set. 
In general, we should ignore this byte and continue from the next, where label starts. 
This byte might be only before 1st field, and have to be stored somewhere as a map marker.

##### List
Sequence of elements strictly one typed. 
* Type byte
* Length (compressed int)
* N Elements, where N - is a Length. No margins, no termination.

##### Pair list
List of paired types, where first one is a key, and the second one is a value.
Key can be any primitive data type. Pair list looks like this:
* Type byte of key
* Type byte of value
* Length (compressed int)
* Sequence of key data and value data going together

##### Union - network enum
Typed enum, that is used to transmit network data.
Can be None or Some (depends on game, not sure exact ids of each one).
None has id of 127, the data in this case is not sent (so no bytes are read).
* Type byte: None(127), Some(...)
* Nothing or labeled data in this struct:
    - Label (3 bytes)
    - Type byte
    - Data of type
* Termination byte

##### Generic
Looks like a Union, but covers general data types.
* Valid: bool as 1 or 0 byte
* ID: Compressed int
* Nothing or labeled data in this struct:
    - Label (3 bytes)
    - Type byte
    - Data of type
* Termination byte

##### IntList - list of compressed integers
Shorter on 1 byte than normal list.
* Length (compressed int)
* Given length of Compressed Integers

##### ObjectType
Tuple of 2 compressed integers representing blaze object type.

##### ObjectId
Tuple of 3 compressed integers representing blaze object id.

##### Float - f32 value
Float value in big-endian. 4 bytes, so f32.

#### Does it include pre-defined game packets?

Shortly no. Packets are different depending on game, and they are considered to be EA intellectual property, so we are unable to share them with everyone. You have to implement them your self by listening "in the middle". You also would need to reverse engineer packet headers yourself, they are usually 16 bytes long.

#### With what games is it working?

We tested it on Battlefield 1 and 5, but I suppouse it should work on any game under frostbite3 engine.

### Under Apache 2.0 License
