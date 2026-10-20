//! A página `buffer` (assinaturas e exemplos; o texto em `ph2d-i18n/src/code_help/buffer.rs`).

use super::Entry;
#[allow(unused_imports)]
use crate::Kind::{Constant, Function};

pub(crate) const ENTRIES: &[Entry] = &[
    Entry {
        name: "copy",
        kind: Function,
        sigs: &[
            "(target: buffer, targetOffset: number, source: buffer, sourceOffset: number?, count: number?)",
        ],
        params: &["target", "targetOffset", "source", "sourceOffset", "count"],
        returns: false,
        example: r#"local name = buffer.fromstring("Hero")
local save = buffer.create(6)
buffer.copy(save, 1, name)
print(buffer.readstring(save, 1, 4)) --> Hero
print(pcall(buffer.copy, buffer.create(2), 0, name)) --> false, buffer access out of bounds"#,
    },
    Entry {
        name: "create",
        kind: Function,
        sigs: &["(size: number): buffer"],
        params: &["size"],
        returns: true,
        example: r#"local save = buffer.create(4)
print(buffer.len(save), buffer.readu8(save, 0)) --> 4, 0"#,
    },
    Entry {
        name: "fill",
        kind: Function,
        sigs: &["(b: buffer, offset: number, value: number, count: number?)"],
        params: &["b", "offset", "value", "count"],
        returns: false,
        example: r#"local row = buffer.create(5)
buffer.fill(row, 0, 46)
buffer.fill(row, 1, 35, 3)
print(buffer.tostring(row)) --> .###."#,
    },
    Entry {
        name: "fromstring",
        kind: Function,
        sigs: &["(str: string): buffer"],
        params: &["str"],
        returns: true,
        example: r#"local name = buffer.fromstring("Hero")
print(buffer.len(name), buffer.readu8(name, 0)) --> 4, 72"#,
    },
    Entry {
        name: "len",
        kind: Function,
        sigs: &["(b: buffer): number"],
        params: &["b"],
        returns: true,
        example: r#"local name = buffer.fromstring("Hero")
local last = buffer.len(name) - 1
print(buffer.len(name), buffer.readstring(name, last, 1)) --> 4, o"#,
    },
    Entry {
        name: "readbits",
        kind: Function,
        sigs: &["(b: buffer, bitOffset: number, bitCount: number): number"],
        params: &["b", "bitOffset", "bitCount"],
        returns: true,
        example: r#"local flags = buffer.create(1)
buffer.writebits(flags, 3, 4, 15)
print(buffer.readbits(flags, 3, 4)) --> 15
print(buffer.readbits(flags, 0, 8)) --> 120"#,
    },
    Entry {
        name: "readf32",
        kind: Function,
        sigs: &["(b: buffer, offset: number): number"],
        params: &["b", "offset"],
        returns: true,
        example: r#"local pos = buffer.create(8)
buffer.writef32(pos, 0, 12.5)
buffer.writef32(pos, 4, 0.1)
print(buffer.readf32(pos, 0), buffer.readf32(pos, 4)) --> 12.5, 0.10000000149011612"#,
    },
    Entry {
        name: "readf64",
        kind: Function,
        sigs: &["(b: buffer, offset: number): number"],
        params: &["b", "offset"],
        returns: true,
        example: r#"local b = buffer.create(8)
buffer.writef64(b, 0, 0.1)
print(buffer.readf64(b, 0)) --> 0.1"#,
    },
    Entry {
        name: "readi16",
        kind: Function,
        sigs: &["(b: buffer, offset: number): number"],
        params: &["b", "offset"],
        returns: true,
        example: r#"local b = buffer.create(2)
buffer.writei16(b, 0, -2)
print(buffer.readi16(b, 0), buffer.readu16(b, 0)) --> -2, 65534"#,
    },
    Entry {
        name: "readi32",
        kind: Function,
        sigs: &["(b: buffer, offset: number): number"],
        params: &["b", "offset"],
        returns: true,
        example: r#"local b = buffer.create(4)
buffer.writei32(b, 0, -5)
print(buffer.readi32(b, 0), buffer.readu32(b, 0)) --> -5, 4294967291"#,
    },
    Entry {
        name: "readi8",
        kind: Function,
        sigs: &["(b: buffer, offset: number): number"],
        params: &["b", "offset"],
        returns: true,
        example: r#"local b = buffer.create(1)
buffer.writeu8(b, 0, 255)
print(buffer.readi8(b, 0), buffer.readu8(b, 0)) --> -1, 255"#,
    },
    Entry {
        name: "readstring",
        kind: Function,
        sigs: &["(b: buffer, offset: number, count: number): string"],
        params: &["b", "offset", "count"],
        returns: true,
        example: r#"local save = buffer.fromstring("Hero:42")
print(buffer.readstring(save, 0, 4)) --> Hero
print(buffer.readstring(save, 5, 2)) --> 42
print(pcall(buffer.readstring, save, 5, 4)) --> false, buffer access out of bounds"#,
    },
    Entry {
        name: "readu16",
        kind: Function,
        sigs: &["(b: buffer, offset: number): number"],
        params: &["b", "offset"],
        returns: true,
        example: r#"local b = buffer.create(2)
buffer.writeu16(b, 0, 1500)
print(buffer.readu16(b, 0)) --> 1500
print(buffer.readu8(b, 0), buffer.readu8(b, 1)) --> 220, 5"#,
    },
    Entry {
        name: "readu32",
        kind: Function,
        sigs: &["(b: buffer, offset: number): number"],
        params: &["b", "offset"],
        returns: true,
        example: r#"local b = buffer.create(4)
buffer.writeu32(b, 0, 4294967295)
print(buffer.readu32(b, 0), buffer.readi32(b, 0)) --> 4294967295, -1"#,
    },
    Entry {
        name: "readu8",
        kind: Function,
        sigs: &["(b: buffer, offset: number): number"],
        params: &["b", "offset"],
        returns: true,
        example: r#"local name = buffer.fromstring("Hero")
print(buffer.readu8(name, 0), buffer.readu8(name, 3)) --> 72, 111
print(pcall(buffer.readu8, name, 4)) --> false, buffer access out of bounds"#,
    },
    Entry {
        name: "tostring",
        kind: Function,
        sigs: &["(b: buffer): string"],
        params: &["b"],
        returns: true,
        example: r#"local word = buffer.create(3)
buffer.writeu8(word, 0, 72)
buffer.writeu8(word, 1, 105)
buffer.writeu8(word, 2, 33)
print(buffer.tostring(word)) --> Hi!"#,
    },
    Entry {
        name: "writebits",
        kind: Function,
        sigs: &["(b: buffer, bitOffset: number, bitCount: number, value: number)"],
        params: &["b", "bitOffset", "bitCount", "value"],
        returns: false,
        example: r#"local flags = buffer.create(1)
buffer.writebits(flags, 0, 3, 5)
print(buffer.readbits(flags, 0, 3), buffer.readu8(flags, 0)) --> 5, 5
print(pcall(buffer.writebits, flags, 0, 33, 1)) --> false, bit count is out of range of [0; 32]"#,
    },
    Entry {
        name: "writef32",
        kind: Function,
        sigs: &["(b: buffer, offset: number, value: number)"],
        params: &["b", "offset", "value"],
        returns: false,
        example: r#"local pos = buffer.create(8)
buffer.writef32(pos, 0, 12.5)
buffer.writef32(pos, 4, -3.25)
print(buffer.readf32(pos, 0), buffer.readf32(pos, 4)) --> 12.5, -3.25"#,
    },
    Entry {
        name: "writef64",
        kind: Function,
        sigs: &["(b: buffer, offset: number, value: number)"],
        params: &["b", "offset", "value"],
        returns: false,
        example: r#"local b = buffer.create(8)
buffer.writef64(b, 0, 1 / 3)
print(buffer.readf64(b, 0)) --> 0.3333333333333333"#,
    },
    Entry {
        name: "writei16",
        kind: Function,
        sigs: &["(b: buffer, offset: number, value: number)"],
        params: &["b", "offset", "value"],
        returns: false,
        example: r#"local b = buffer.create(2)
buffer.writei16(b, 0, -300)
print(buffer.readi16(b, 0)) --> -300"#,
    },
    Entry {
        name: "writei32",
        kind: Function,
        sigs: &["(b: buffer, offset: number, value: number)"],
        params: &["b", "offset", "value"],
        returns: false,
        example: r#"local b = buffer.create(4)
buffer.writei32(b, 0, -100000)
print(buffer.readi32(b, 0)) --> -100000"#,
    },
    Entry {
        name: "writei8",
        kind: Function,
        sigs: &["(b: buffer, offset: number, value: number)"],
        params: &["b", "offset", "value"],
        returns: false,
        example: r#"local b = buffer.create(2)
buffer.writei8(b, 0, -20)
buffer.writei8(b, 1, 200)
print(buffer.readi8(b, 0), buffer.readi8(b, 1)) --> -20, -56"#,
    },
    Entry {
        name: "writestring",
        kind: Function,
        sigs: &["(b: buffer, offset: number, value: string, count: number?)"],
        params: &["b", "offset", "value", "count"],
        returns: false,
        example: r#"local name = buffer.fromstring("Hero")
buffer.writestring(name, 0, "Zed")
print(buffer.tostring(name)) --> Zedo
buffer.writestring(name, 0, "Bobby", 2)
print(buffer.tostring(name)) --> Bodo"#,
    },
    Entry {
        name: "writeu16",
        kind: Function,
        sigs: &["(b: buffer, offset: number, value: number)"],
        params: &["b", "offset", "value"],
        returns: false,
        example: r#"local b = buffer.create(2)
buffer.writeu16(b, 0, 1500)
print(buffer.readu16(b, 0)) --> 1500
buffer.writeu16(b, 0, 70000)
print(buffer.readu16(b, 0)) --> 4464"#,
    },
    Entry {
        name: "writeu32",
        kind: Function,
        sigs: &["(b: buffer, offset: number, value: number)"],
        params: &["b", "offset", "value"],
        returns: false,
        example: r#"local b = buffer.create(4)
buffer.writeu32(b, 0, 1)
print(buffer.readu8(b, 0), buffer.readu8(b, 3)) --> 1, 0
print(buffer.readu32(b, 0)) --> 1"#,
    },
    Entry {
        name: "writeu8",
        kind: Function,
        sigs: &["(b: buffer, offset: number, value: number)"],
        params: &["b", "offset", "value"],
        returns: false,
        example: r#"local b = buffer.create(3)
buffer.writeu8(b, 0, 200)
buffer.writeu8(b, 1, 300)
buffer.writeu8(b, 2, 2.9)
print(buffer.readu8(b, 0), buffer.readu8(b, 1), buffer.readu8(b, 2)) --> 200, 44, 2"#,
    },
];
