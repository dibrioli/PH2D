//! A página `math` (assinaturas e exemplos; o texto em `ph2d-i18n/src/code_help/math.rs`).

use super::Entry;
#[allow(unused_imports)]
use crate::Kind::{Constant, Function};

pub(crate) const ENTRIES: &[Entry] = &[
    Entry {
        name: "abs",
        kind: Function,
        sigs: &["(n: number): number"],
        params: &["n"],
        returns: true,
        example: r#"local speed = -12.5
print(math.abs(speed)) --> 12.5"#,
    },
    Entry {
        name: "acos",
        kind: Function,
        sigs: &["(n: number): number"],
        params: &["n"],
        returns: true,
        example: r#"print(math.acos(1)) --> 0
print(math.round(math.deg(math.acos(0.5)))) --> 60
print(math.acos(2)) --> nan"#,
    },
    Entry {
        name: "asin",
        kind: Function,
        sigs: &["(n: number): number"],
        params: &["n"],
        returns: true,
        example: r#"print(math.asin(0)) --> 0
print(math.deg(math.asin(1))) --> 90"#,
    },
    Entry {
        name: "atan",
        kind: Function,
        sigs: &["(n: number): number"],
        params: &["n"],
        returns: true,
        example: r#"local slope = 1 -- climbs 1 for every 1 across
print(math.deg(math.atan(slope))) --> 45"#,
    },
    Entry {
        name: "atan2",
        kind: Function,
        sigs: &["(y: number, x: number): number"],
        params: &["y", "x"],
        returns: true,
        example: r#"local dx, dy = -10, 0 -- the target is straight to the left
print(math.deg(math.atan2(dy, dx))) --> 180
print(math.deg(math.atan2(10, 0))) --> 90"#,
    },
    Entry {
        name: "ceil",
        kind: Function,
        sigs: &["(n: number): number"],
        params: &["n"],
        returns: true,
        example: r#"local pages = math.ceil(25 / 10) -- 25 items, 10 per page
print(pages) --> 3
print(math.ceil(-2.5)) --> -2"#,
    },
    Entry {
        name: "clamp",
        kind: Function,
        sigs: &["(n: number, min: number, max: number): number"],
        params: &["n", "min", "max"],
        returns: true,
        example: r#"local health = 80
health = math.clamp(health - 35, 0, 100)
print(health) --> 45
print(math.clamp(150, 0, 100)) --> 100
print(pcall(math.clamp, 5, 10, 0)) --> false, invalid argument #3 to 'clamp' (max must be greater than or equal to min)"#,
    },
    Entry {
        name: "cos",
        kind: Function,
        sigs: &["(n: number): number"],
        params: &["n"],
        returns: true,
        example: r#"print(math.cos(0)) --> 1
print(math.cos(math.pi)) --> -1
local x = 100 + math.cos(math.rad(90)) * 50 -- a point on a circle
print(math.round(x)) --> 100"#,
    },
    Entry {
        name: "cosh",
        kind: Function,
        sigs: &["(n: number): number"],
        params: &["n"],
        returns: true,
        example: r#"print(math.cosh(0)) --> 1
print(math.cosh(1)) --> 1.5430806348152437"#,
    },
    Entry {
        name: "deg",
        kind: Function,
        sigs: &["(n: number): number"],
        params: &["n"],
        returns: true,
        example: r#"print(math.deg(math.pi)) --> 180
print(math.deg(math.pi / 2)) --> 90"#,
    },
    Entry {
        name: "e",
        kind: Constant,
        sigs: &["number"],
        params: &[],
        returns: false,
        example: r#"print(math.e) --> 2.718281828459045"#,
    },
    Entry {
        name: "exp",
        kind: Function,
        sigs: &["(n: number): number"],
        params: &["n"],
        returns: true,
        example: r#"print(math.exp(0)) --> 1
print(math.exp(1)) --> 2.718281828459045"#,
    },
    Entry {
        name: "floor",
        kind: Function,
        sigs: &["(n: number): number"],
        params: &["n"],
        returns: true,
        example: r#"local seconds = 125
print(math.floor(seconds / 60), seconds % 60) --> 2, 5
print(math.floor(-2.5)) --> -3"#,
    },
    Entry {
        name: "fmod",
        kind: Function,
        sigs: &["(x: number, y: number): number"],
        params: &["x", "y"],
        returns: true,
        example: r#"print(math.fmod(7, 3)) --> 1
print(math.fmod(-7, 3), -7 % 3) --> -1, 2
print(math.fmod(5.5, 2)) --> 1.5"#,
    },
    Entry {
        name: "frexp",
        kind: Function,
        sigs: &["(n: number): (number, number)"],
        params: &["n"],
        returns: true,
        example: r#"print(math.frexp(8)) --> 0.5, 4
print(math.frexp(0)) --> 0, 0"#,
    },
    Entry {
        name: "huge",
        kind: Constant,
        sigs: &["number"],
        params: &[],
        returns: false,
        example: r#"print(math.huge, -math.huge) --> inf, -inf
local closest = math.huge -- start bigger than any real distance
closest = math.min(closest, 42)
print(closest) --> 42"#,
    },
    Entry {
        name: "isfinite",
        kind: Function,
        sigs: &["(x: number): boolean"],
        params: &["x"],
        returns: true,
        example: r#"print(math.isfinite(3)) --> true
print(math.isfinite(1 / 0), math.isfinite(0 / 0)) --> false, false"#,
    },
    Entry {
        name: "isinf",
        kind: Function,
        sigs: &["(x: number): boolean"],
        params: &["x"],
        returns: true,
        example: r#"local ratio = 10 / 0
print(ratio, math.isinf(ratio)) --> inf, true
print(math.isinf(-math.huge), math.isinf(5)) --> true, false"#,
    },
    Entry {
        name: "isnan",
        kind: Function,
        sigs: &["(x: number): boolean"],
        params: &["x"],
        returns: true,
        example: r#"local scale = 0 / 0
print(math.isnan(scale), math.isnan(1)) --> true, false
print(scale == scale) --> false"#,
    },
    Entry {
        name: "ldexp",
        kind: Function,
        sigs: &["(s: number, e: number): number"],
        params: &["s", "e"],
        returns: true,
        example: r#"print(math.ldexp(1, 10)) --> 1024
print(math.ldexp(0.75, 2)) --> 3"#,
    },
    Entry {
        name: "lerp",
        kind: Function,
        sigs: &["(a: number, b: number, t: number): number"],
        params: &["a", "b", "t"],
        returns: true,
        example: r#"local startX, endX = 0, 200
print(math.lerp(startX, endX, 0.25)) --> 50
print(math.lerp(startX, endX, 1)) --> 200
print(math.lerp(0, 10, 1.5)) --> 15"#,
    },
    Entry {
        name: "log",
        kind: Function,
        sigs: &["(n: number, base: number?): number"],
        params: &["n", "base"],
        returns: true,
        example: r#"print(math.log(math.e)) --> 1
print(math.log(8, 2), math.log(100, 10)) --> 3, 2
print(math.log(0), math.log(-1)) --> -inf, nan"#,
    },
    Entry {
        name: "log10",
        kind: Function,
        sigs: &["(n: number): number"],
        params: &["n"],
        returns: true,
        example: r#"local score = 4500
print(math.floor(math.log10(score)) + 1) --> 4"#,
    },
    Entry {
        name: "map",
        kind: Function,
        sigs: &[
            "(x: number, inmin: number, inmax: number, outmin: number, outmax: number): number",
        ],
        params: &["x", "inmin", "inmax", "outmin", "outmax"],
        returns: true,
        example: r#"local volume = 0.5 -- a slider from 0 to 1
print(math.map(volume, 0, 1, 0, 255)) --> 127.5
print(math.map(5, 0, 10, 100, 0)) --> 50
print(math.map(15, 0, 10, 0, 100)) --> 150"#,
    },
    Entry {
        name: "max",
        kind: Function,
        sigs: &["(x: number, ...: number): number"],
        params: &["x", "..."],
        returns: true,
        example: r#"print(math.max(3, 9, 2)) --> 9
local damage = math.max(10 - 15, 0) -- never heal by accident
print(damage) --> 0"#,
    },
    Entry {
        name: "min",
        kind: Function,
        sigs: &["(x: number, ...: number): number"],
        params: &["x", "..."],
        returns: true,
        example: r#"print(math.min(3, 1, 2)) --> 1
local ammo = math.min(25 + 10, 30) -- the clip holds 30
print(ammo) --> 30"#,
    },
    Entry {
        name: "modf",
        kind: Function,
        sigs: &["(n: number): (number, number)"],
        params: &["n"],
        returns: true,
        example: r#"local whole, part = math.modf(3.25)
print(whole, part) --> 3, 0.25
print(math.modf(-2.5)) --> -2, -0.5"#,
    },
    Entry {
        name: "nan",
        kind: Constant,
        sigs: &["number"],
        params: &[],
        returns: false,
        example: r#"print(math.nan) --> nan
print(math.nan == math.nan, math.isnan(math.nan)) --> false, true"#,
    },
    Entry {
        name: "noise",
        kind: Function,
        sigs: &["(x: number, y: number?, z: number?): number"],
        params: &["x", "y", "z"],
        returns: true,
        example: r#"print(math.noise(0.25)) --> 0.146484375
print(math.noise(0.25) == math.noise(0.25)) --> true
print(math.noise(1), math.noise(2, 3)) --> 0, 0"#,
    },
    Entry {
        name: "phi",
        kind: Constant,
        sigs: &["number"],
        params: &[],
        returns: false,
        example: r#"print(math.phi) --> 1.618033988749895
local width = 300
print(math.round(width / math.phi)) --> 185"#,
    },
    Entry {
        name: "pi",
        kind: Constant,
        sigs: &["number"],
        params: &[],
        returns: false,
        example: r#"print(math.pi) --> 3.141592653589793
local half_turn = math.pi -- 180 degrees, in radians
print(math.deg(half_turn)) --> 180"#,
    },
    Entry {
        name: "pow",
        kind: Function,
        sigs: &["(x: number, y: number): number"],
        params: &["x", "y"],
        returns: true,
        example: r#"print(math.pow(2, 10), 2 ^ 10) --> 1024, 1024
print(math.pow(9, 0.5)) --> 3"#,
    },
    Entry {
        name: "rad",
        kind: Function,
        sigs: &["(n: number): number"],
        params: &["n"],
        returns: true,
        example: r#"print(math.rad(180)) --> 3.141592653589793
print(math.sin(math.rad(90))) --> 1"#,
    },
    Entry {
        name: "random",
        kind: Function,
        sigs: &["(m: number?, n: number?): number"],
        params: &["m", "n"],
        returns: true,
        example: r#"local roll = math.random(6) -- a dice: 1, 2, 3, 4, 5 or 6
print(roll >= 1 and roll <= 6, roll == math.floor(roll)) --> true, true
local chance = math.random() -- from 0 up to (not including) 1
print(chance >= 0 and chance < 1) --> true
print(pcall(math.random, 5, 1)) --> false, invalid argument #2 to 'random' (interval is empty)"#,
    },
    Entry {
        name: "randomseed",
        kind: Function,
        sigs: &["(seed: number)"],
        params: &["seed"],
        returns: false,
        example: r#"math.randomseed(42)
local first = math.random(1000)
math.randomseed(42)
print(math.random(1000) == first) --> true"#,
    },
    Entry {
        name: "round",
        kind: Function,
        sigs: &["(n: number): number"],
        params: &["n"],
        returns: true,
        example: r#"print(math.round(2.4), math.round(2.5)) --> 2, 3
print(math.round(-2.5)) --> -3
print(math.round(3.14159 * 100) / 100) --> 3.14"#,
    },
    Entry {
        name: "sign",
        kind: Function,
        sigs: &["(n: number): number"],
        params: &["n"],
        returns: true,
        example: r#"local velocity = -4.5
print(math.sign(velocity)) --> -1
print(math.sign(0), math.sign(12)) --> 0, 1"#,
    },
    Entry {
        name: "sin",
        kind: Function,
        sigs: &["(n: number): number"],
        params: &["n"],
        returns: true,
        example: r#"print(math.sin(math.pi / 2)) --> 1
local time = math.pi * 1.5 -- seconds since the start
local y = 100 + math.sin(time) * 10 -- bob 10 pixels around y = 100
print(y) --> 90"#,
    },
    Entry {
        name: "sinh",
        kind: Function,
        sigs: &["(n: number): number"],
        params: &["n"],
        returns: true,
        example: r#"print(math.sinh(0)) --> 0
print(math.sinh(1)) --> 1.1752011936438014"#,
    },
    Entry {
        name: "sqrt",
        kind: Function,
        sigs: &["(n: number): number"],
        params: &["n"],
        returns: true,
        example: r#"local dx, dy = 3, 4
print(math.sqrt(dx * dx + dy * dy)) --> 5
print(math.sqrt(-1)) --> nan"#,
    },
    Entry {
        name: "sqrt2",
        kind: Constant,
        sigs: &["number"],
        params: &[],
        returns: false,
        example: r#"print(math.sqrt2) --> 1.4142135623730951
print(math.sqrt2 == math.sqrt(2)) --> true"#,
    },
    Entry {
        name: "tan",
        kind: Function,
        sigs: &["(n: number): number"],
        params: &["n"],
        returns: true,
        example: r#"print(math.tan(0)) --> 0
print(math.round(math.tan(math.rad(45)) * 1000) / 1000) --> 1"#,
    },
    Entry {
        name: "tanh",
        kind: Function,
        sigs: &["(n: number): number"],
        params: &["n"],
        returns: true,
        example: r#"print(math.tanh(0)) --> 0
print(math.tanh(10)) --> 0.9999999958776927"#,
    },
    Entry {
        name: "tau",
        kind: Constant,
        sigs: &["number"],
        params: &[],
        returns: false,
        example: r#"print(math.tau) --> 6.283185307179586
print(math.tau == math.pi * 2) --> true"#,
    },
];
