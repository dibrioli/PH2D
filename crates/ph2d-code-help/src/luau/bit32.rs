//! A página `bit32` (assinaturas e exemplos; o texto em `ph2d-i18n/src/code_help/bit32.rs`).

use super::Entry;
#[allow(unused_imports)]
use crate::Kind::{Constant, Function};

pub(crate) const ENTRIES: &[Entry] = &[
    Entry {
        name: "arshift",
        kind: Function,
        sigs: &["(x: number, disp: number): number"],
        params: &["x", "disp"],
        returns: true,
        example: r#"print(bit32.arshift(16, 2)) --> 4
print(bit32.arshift(-16, 2)) --> 4294967292
print(bit32.rshift(-16, 2)) --> 1073741820"#,
    },
    Entry {
        name: "band",
        kind: Function,
        sigs: &["(...: number): number"],
        params: &["..."],
        returns: true,
        example: r#"local ON_GROUND, IS_HURT, HAS_KEY = 1, 2, 4
local state = bit32.bor(ON_GROUND, HAS_KEY)
print(bit32.band(state, HAS_KEY)) --> 4
print(bit32.band(state, IS_HURT)) --> 0
print(bit32.band(0xFF00, 0x0FF0)) --> 3840"#,
    },
    Entry {
        name: "bnot",
        kind: Function,
        sigs: &["(x: number): number"],
        params: &["x"],
        returns: true,
        example: r#"print(bit32.bnot(0)) --> 4294967295
print(bit32.bnot(5)) --> 4294967290
print(bit32.band(7, bit32.bnot(2))) --> 5"#,
    },
    Entry {
        name: "bor",
        kind: Function,
        sigs: &["(...: number): number"],
        params: &["..."],
        returns: true,
        example: r#"local ON_GROUND, HAS_KEY = 1, 4
print(bit32.bor(ON_GROUND, HAS_KEY)) --> 5
print(bit32.bor(1, 2, 4, 8)) --> 15"#,
    },
    Entry {
        name: "btest",
        kind: Function,
        sigs: &["(x: number, ...: number): boolean"],
        params: &["x", "..."],
        returns: true,
        example: r#"local HAS_KEY, IS_HURT = 4, 2
local state = bit32.bor(1, HAS_KEY)
print(bit32.btest(state, HAS_KEY), bit32.btest(state, IS_HURT)) --> true, false"#,
    },
    Entry {
        name: "bxor",
        kind: Function,
        sigs: &["(...: number): number"],
        params: &["..."],
        returns: true,
        example: r#"local LIGHT_ON = 4
local state = bit32.bxor(1, LIGHT_ON)
print(state) --> 5
print(bit32.bxor(state, LIGHT_ON)) --> 1"#,
    },
    Entry {
        name: "byteswap",
        kind: Function,
        sigs: &["(n: number): number"],
        params: &["n"],
        returns: true,
        example: r#"local swapped = bit32.byteswap(0x11223344)
print(string.format("%08x", swapped)) --> 44332211
print(bit32.byteswap(1)) --> 16777216"#,
    },
    Entry {
        name: "countlz",
        kind: Function,
        sigs: &["(n: number): number"],
        params: &["n"],
        returns: true,
        example: r#"print(bit32.countlz(1), bit32.countlz(255), bit32.countlz(0)) --> 31, 24, 32"#,
    },
    Entry {
        name: "countrz",
        kind: Function,
        sigs: &["(n: number): number"],
        params: &["n"],
        returns: true,
        example: r#"print(bit32.countrz(8), bit32.countrz(12), bit32.countrz(0)) --> 3, 2, 32"#,
    },
    Entry {
        name: "extract",
        kind: Function,
        sigs: &["(n: number, field: number, width: number?): number"],
        params: &["n", "field", "width"],
        returns: true,
        example: r#"local orange = 0xFF8800
local r = bit32.extract(orange, 16, 8)
local g = bit32.extract(orange, 8, 8)
print(r, g, bit32.extract(orange, 0, 8)) --> 255, 136, 0
print(pcall(bit32.extract, orange, 30, 4)) --> false, trying to access non-existent bits"#,
    },
    Entry {
        name: "lrotate",
        kind: Function,
        sigs: &["(x: number, disp: number): number"],
        params: &["x", "disp"],
        returns: true,
        example: r#"print(bit32.lrotate(0x80000001, 1)) --> 3
print(bit32.lshift(0x80000001, 1)) --> 2"#,
    },
    Entry {
        name: "lshift",
        kind: Function,
        sigs: &["(x: number, disp: number): number"],
        params: &["x", "disp"],
        returns: true,
        example: r#"print(bit32.lshift(1, 4)) --> 16
print(bit32.lshift(255, 16)) --> 16711680
print(bit32.lshift(1, 32)) --> 0"#,
    },
    Entry {
        name: "replace",
        kind: Function,
        sigs: &["(n: number, v: number, field: number, width: number?): number"],
        params: &["n", "v", "field", "width"],
        returns: true,
        example: r#"local orange = 0xFF8800
local red = bit32.replace(orange, 0, 8, 8)
print(string.format("%06x", red)) --> ff0000
print(bit32.replace(0, 5, 4, 3)) --> 80"#,
    },
    Entry {
        name: "rrotate",
        kind: Function,
        sigs: &["(x: number, disp: number): number"],
        params: &["x", "disp"],
        returns: true,
        example: r#"print(bit32.rrotate(0xF0, 4)) --> 15
print(bit32.rrotate(1, 1)) --> 2147483648"#,
    },
    Entry {
        name: "rshift",
        kind: Function,
        sigs: &["(x: number, disp: number): number"],
        params: &["x", "disp"],
        returns: true,
        example: r#"print(bit32.rshift(256, 4)) --> 16
print(bit32.rshift(0xFF8800, 16)) --> 255
print(bit32.rshift(-1, 28)) --> 15"#,
    },
];
