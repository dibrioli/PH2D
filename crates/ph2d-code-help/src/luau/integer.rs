//! A página `integer` (assinaturas e exemplos; o texto em `ph2d-i18n/src/code_help/integer.rs`).

use super::Entry;
#[allow(unused_imports)]
use crate::Kind::{Constant, Function};

pub(crate) const ENTRIES: &[Entry] = &[
    Entry {
        name: "add",
        kind: Function,
        sigs: &["(x: integer, y: integer): integer"],
        params: &["x", "y"],
        returns: true,
        example: r#"local frames = integer.create(250)
print(integer.add(frames, integer.create(1000))) --> 1250
local past = integer.add(integer.maxsigned, integer.create(1))
print(tostring(past)) --> -9223372036854775808"#,
    },
    Entry {
        name: "arshift",
        kind: Function,
        sigs: &["(x: integer, numBitPositions: integer): integer"],
        params: &["x", "numBitPositions"],
        returns: true,
        example: r#"print(integer.arshift(integer.create(256), integer.create(4))) --> 16
print(integer.arshift(integer.create(-16), integer.create(2))) --> -4"#,
    },
    Entry {
        name: "band",
        kind: Function,
        sigs: &["(...: integer): integer"],
        params: &["..."],
        returns: true,
        example: r#"local flags = integer.create(13) -- bits 1101
print(integer.band(flags, integer.create(4))) --> 4
print(integer.band(flags, integer.create(2))) --> 0"#,
    },
    Entry {
        name: "bnot",
        kind: Function,
        sigs: &["(x: integer): integer"],
        params: &["x"],
        returns: true,
        example: r#"print(integer.bnot(integer.create(0))) --> -1
print(integer.bnot(integer.create(5))) --> -6"#,
    },
    Entry {
        name: "bor",
        kind: Function,
        sigs: &["(...: integer): integer"],
        params: &["..."],
        returns: true,
        example: r#"local VISIBLE = integer.create(1)
local LOCKED = integer.create(4)
print(integer.bor(VISIBLE, LOCKED)) --> 5"#,
    },
    Entry {
        name: "bswap",
        kind: Function,
        sigs: &["(x: integer): integer"],
        params: &["x"],
        returns: true,
        example: r#"print(tostring(integer.bswap(integer.create(1)))) --> 72057594037927936
local id = integer.create(1234)
print(integer.bswap(integer.bswap(id))) --> 1234"#,
    },
    Entry {
        name: "btest",
        kind: Function,
        sigs: &["(...: integer): boolean"],
        params: &["..."],
        returns: true,
        example: r#"local flags = integer.create(5) -- VISIBLE (1) and LOCKED (4)
print(integer.btest(flags, integer.create(4))) --> true
print(integer.btest(flags, integer.create(2))) --> false"#,
    },
    Entry {
        name: "bxor",
        kind: Function,
        sigs: &["(...: integer): integer"],
        params: &["..."],
        returns: true,
        example: r#"local flags = integer.create(5)
flags = integer.bxor(flags, integer.create(1))
print(flags) --> 4
flags = integer.bxor(flags, integer.create(1))
print(flags) --> 5"#,
    },
    Entry {
        name: "clamp",
        kind: Function,
        sigs: &["(value: integer, min: integer, max: integer): integer"],
        params: &["value", "min", "max"],
        returns: true,
        example: r#"local lo, hi = integer.create(0), integer.create(10)
print(integer.clamp(integer.create(15), lo, hi)) --> 10
print(pcall(integer.clamp, integer.create(1), hi, lo)) --> false, invalid argument #3 to 'clamp' (max must be greater than or equal to min)"#,
    },
    Entry {
        name: "countlz",
        kind: Function,
        sigs: &["(x: integer): integer"],
        params: &["x"],
        returns: true,
        example: r#"print(integer.countlz(integer.create(1))) --> 63
print(integer.countlz(integer.create(0))) --> 64"#,
    },
    Entry {
        name: "countrz",
        kind: Function,
        sigs: &["(x: integer): integer"],
        params: &["x"],
        returns: true,
        example: r#"print(integer.countrz(integer.create(8))) --> 3
print(integer.countrz(integer.create(0))) --> 64"#,
    },
    Entry {
        name: "create",
        kind: Function,
        sigs: &["(x: number): integer"],
        params: &["x"],
        returns: true,
        example: r#"local id = integer.create(42)
print(id, typeof(id)) --> 42, integer
print(id == integer.create(42), id == 42) --> true, false
print(integer.create(1.5)) --> nil"#,
    },
    Entry {
        name: "div",
        kind: Function,
        sigs: &["(x: integer, y: integer): integer"],
        params: &["x", "y"],
        returns: true,
        example: r#"print(integer.div(integer.create(7), integer.create(2))) --> 3
print(integer.div(integer.create(-7), integer.create(2))) --> -3
print(pcall(integer.div, integer.create(7), integer.create(0))) --> false, division by zero"#,
    },
    Entry {
        name: "extract",
        kind: Function,
        sigs: &["(value: integer, bitPosition: integer, numBits: integer?): integer"],
        params: &["value", "bitPosition", "numBits"],
        returns: true,
        example: r#"local colour = integer.fromstring("FF8800", 16)
print(integer.extract(colour, integer.create(16), integer.create(8))) --> 255
print(integer.extract(colour, integer.create(8), integer.create(8))) --> 136
print(integer.extract(colour, integer.create(0))) --> 0"#,
    },
    Entry {
        name: "fromstring",
        kind: Function,
        sigs: &["(str: string, base: number?): integer"],
        params: &["str", "base"],
        returns: true,
        example: r#"local id = integer.fromstring("9007199254740993")
print(tostring(id)) --> 9007199254740993
print(integer.fromstring("ff", 16)) --> 255
print(integer.fromstring("12.5")) --> nil"#,
    },
    Entry {
        name: "ge",
        kind: Function,
        sigs: &["(x: integer, y: integer): boolean"],
        params: &["x", "y"],
        returns: true,
        example: r#"local score, needed = integer.create(300), integer.create(300)
print(integer.ge(score, needed)) --> true"#,
    },
    Entry {
        name: "gt",
        kind: Function,
        sigs: &["(x: integer, y: integer): boolean"],
        params: &["x", "y"],
        returns: true,
        example: r#"local score, best = integer.create(1200), integer.create(950)
print(integer.gt(score, best)) --> true
print(integer.gt(best, best)) --> false"#,
    },
    Entry {
        name: "idiv",
        kind: Function,
        sigs: &["(x: integer, y: integer): integer"],
        params: &["x", "y"],
        returns: true,
        example: r#"print(integer.idiv(integer.create(7), integer.create(2))) --> 3
print(integer.idiv(integer.create(-7), integer.create(2))) --> -4"#,
    },
    Entry {
        name: "le",
        kind: Function,
        sigs: &["(x: integer, y: integer): boolean"],
        params: &["x", "y"],
        returns: true,
        example: r#"local used, limit = integer.create(64), integer.create(64)
print(integer.le(used, limit)) --> true"#,
    },
    Entry {
        name: "lrotate",
        kind: Function,
        sigs: &["(x: integer, numBitPositions: integer): integer"],
        params: &["x", "numBitPositions"],
        returns: true,
        example: r#"print(integer.lrotate(integer.create(1), integer.create(4))) --> 16
print(tostring(integer.lrotate(integer.create(1), integer.create(63)))) --> -9223372036854775808
print(integer.lrotate(integer.create(1), integer.create(64))) --> 1"#,
    },
    Entry {
        name: "lshift",
        kind: Function,
        sigs: &["(x: integer, numBitPositions: integer): integer"],
        params: &["x", "numBitPositions"],
        returns: true,
        example: r#"print(integer.lshift(integer.create(1), integer.create(4))) --> 16
print(integer.lshift(integer.create(1), integer.create(64))) --> 0"#,
    },
    Entry {
        name: "lt",
        kind: Function,
        sigs: &["(x: integer, y: integer): boolean"],
        params: &["x", "y"],
        returns: true,
        example: r#"local a, b = integer.create(-1), integer.create(1)
print(integer.lt(a, b)) --> true
print(pcall(integer.lt, a, 1)) --> false, invalid argument #2 to 'lt' (integer expected, got number)"#,
    },
    Entry {
        name: "max",
        kind: Function,
        sigs: &["(first: integer, ...: integer): integer"],
        params: &["first", "..."],
        returns: true,
        example: r#"local a, b, c = integer.create(3), integer.create(-1), integer.create(7)
print(integer.max(a, b, c)) --> 7"#,
    },
    Entry {
        name: "maxsigned",
        kind: Constant,
        sigs: &["integer"],
        params: &[],
        returns: false,
        example: r#"print(tostring(integer.maxsigned)) --> 9223372036854775807"#,
    },
    Entry {
        name: "min",
        kind: Function,
        sigs: &["(first: integer, ...: integer): integer"],
        params: &["first", "..."],
        returns: true,
        example: r#"local a, b, c = integer.create(3), integer.create(-1), integer.create(7)
print(integer.min(a, b, c)) --> -1"#,
    },
    Entry {
        name: "minsigned",
        kind: Constant,
        sigs: &["integer"],
        params: &[],
        returns: false,
        example: r#"print(tostring(integer.minsigned)) --> -9223372036854775808"#,
    },
    Entry {
        name: "mod",
        kind: Function,
        sigs: &["(x: integer, y: integer): integer"],
        params: &["x", "y"],
        returns: true,
        example: r#"local frame = integer.create(-1)
print(integer.mod(frame, integer.create(8))) --> 7
print(integer.mod(integer.create(7), integer.create(-2))) --> -1"#,
    },
    Entry {
        name: "mul",
        kind: Function,
        sigs: &["(x: integer, y: integer): integer"],
        params: &["x", "y"],
        returns: true,
        example: r#"print(integer.mul(integer.create(1920), integer.create(1080))) --> 2073600
print(tostring(integer.mul(integer.maxsigned, integer.create(2)))) --> -2"#,
    },
    Entry {
        name: "neg",
        kind: Function,
        sigs: &["(value: integer): integer"],
        params: &["value"],
        returns: true,
        example: r#"print(integer.neg(integer.create(5))) --> -5
print(integer.neg(integer.minsigned) == integer.minsigned) --> true"#,
    },
    Entry {
        name: "rem",
        kind: Function,
        sigs: &["(x: integer, y: integer): integer"],
        params: &["x", "y"],
        returns: true,
        example: r#"local x, y = integer.create(-7), integer.create(2)
print(integer.rem(x, y)) --> -1
print(integer.mod(x, y)) --> 1"#,
    },
    Entry {
        name: "replace",
        kind: Function,
        sigs: &[
            "(value: integer, replacement: integer, bitPosition: integer, numBits: integer?): integer",
        ],
        params: &["value", "replacement", "bitPosition", "numBits"],
        returns: true,
        example: r#"local eight = integer.create(8)
local colour = integer.replace(integer.create(0), integer.create(255), integer.create(16), eight)
colour = integer.replace(colour, integer.create(136), eight, eight)
print(string.format("%x", colour)) --> ff8800"#,
    },
    Entry {
        name: "rrotate",
        kind: Function,
        sigs: &["(x: integer, numBitPositions: integer): integer"],
        params: &["x", "numBitPositions"],
        returns: true,
        example: r#"print(integer.rrotate(integer.create(16), integer.create(4))) --> 1
print(tostring(integer.rrotate(integer.create(1), integer.create(1)))) --> -9223372036854775808"#,
    },
    Entry {
        name: "rshift",
        kind: Function,
        sigs: &["(x: integer, numBitPositions: integer): integer"],
        params: &["x", "numBitPositions"],
        returns: true,
        example: r#"print(integer.rshift(integer.create(256), integer.create(4))) --> 16
print(integer.rshift(integer.create(-1), integer.create(60))) --> 15"#,
    },
    Entry {
        name: "sub",
        kind: Function,
        sigs: &["(x: integer, y: integer): integer"],
        params: &["x", "y"],
        returns: true,
        example: r#"local lives, hits = integer.create(3), integer.create(10)
print(integer.sub(lives, hits)) --> -7"#,
    },
    Entry {
        name: "tonumber",
        kind: Function,
        sigs: &["(x: integer): number"],
        params: &["x"],
        returns: true,
        example: r#"local n = integer.tonumber(integer.create(42))
print(n + 0.5, typeof(n)) --> 42.5, number
local big = integer.fromstring("9007199254740993")
print(integer.tonumber(big)) --> 9007199254740992"#,
    },
    Entry {
        name: "udiv",
        kind: Function,
        sigs: &["(x: integer, y: integer): integer"],
        params: &["x", "y"],
        returns: true,
        example: r#"print(integer.udiv(integer.create(7), integer.create(2))) --> 3
print(tostring(integer.udiv(integer.create(-2), integer.create(2)))) --> 9223372036854775807"#,
    },
    Entry {
        name: "uge",
        kind: Function,
        sigs: &["(x: integer, y: integer): boolean"],
        params: &["x", "y"],
        returns: true,
        example: r#"local a, b = integer.create(-1), integer.create(1)
print(integer.uge(a, b), integer.ge(a, b)) --> true, false"#,
    },
    Entry {
        name: "ugt",
        kind: Function,
        sigs: &["(x: integer, y: integer): boolean"],
        params: &["x", "y"],
        returns: true,
        example: r#"local a, b = integer.create(-1), integer.create(1)
print(integer.ugt(a, b), integer.gt(a, b)) --> true, false"#,
    },
    Entry {
        name: "ule",
        kind: Function,
        sigs: &["(x: integer, y: integer): boolean"],
        params: &["x", "y"],
        returns: true,
        example: r#"local a, b = integer.create(1), integer.create(-1)
print(integer.ule(a, b), integer.le(a, b)) --> true, false"#,
    },
    Entry {
        name: "ult",
        kind: Function,
        sigs: &["(x: integer, y: integer): boolean"],
        params: &["x", "y"],
        returns: true,
        example: r#"local a, b = integer.create(-1), integer.create(1)
print(integer.ult(a, b), integer.lt(a, b)) --> false, true"#,
    },
    Entry {
        name: "urem",
        kind: Function,
        sigs: &["(x: integer, y: integer): integer"],
        params: &["x", "y"],
        returns: true,
        example: r#"print(integer.urem(integer.create(7), integer.create(2))) --> 1
print(integer.urem(integer.create(-1), integer.create(10))) --> 5"#,
    },
];
