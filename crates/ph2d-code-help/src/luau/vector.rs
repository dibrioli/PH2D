//! A página `vector` (assinaturas e exemplos; o texto em `ph2d-i18n/src/code_help/vector.rs`).

use super::Entry;
#[allow(unused_imports)]
use crate::Kind::{Constant, Function};

pub(crate) const ENTRIES: &[Entry] = &[
    Entry {
        name: "abs",
        kind: Function,
        sigs: &["(vec: vector): vector"],
        params: &["vec"],
        returns: true,
        example: r#"local push = vector.create(-3, 4, -2)
print(vector.abs(push)) --> 3, 4, 2"#,
    },
    Entry {
        name: "angle",
        kind: Function,
        sigs: &["(vec1: vector, vec2: vector, axis: vector?): number"],
        params: &["vec1", "vec2", "axis"],
        returns: true,
        example: r#"local right = vector.create(1, 0)
local down = vector.create(0, -1)
print(vector.angle(right, down)) --> 1.5707963267948966
print(vector.angle(right, down, vector.create(0, 0, 1))) --> -1.5707963267948966"#,
    },
    Entry {
        name: "ceil",
        kind: Function,
        sigs: &["(vec: vector): vector"],
        params: &["vec"],
        returns: true,
        example: r#"local pos = vector.create(1.2, -1.7)
print(vector.ceil(pos)) --> 2, -1, 0"#,
    },
    Entry {
        name: "clamp",
        kind: Function,
        sigs: &["(vec: vector, min: vector, max: vector): vector"],
        params: &["vec", "min", "max"],
        returns: true,
        example: r#"local pos = vector.create(15, -5)
local corner = vector.create(10, 10)
print(vector.clamp(pos, vector.zero, corner)) --> 10, 0, 0
print(pcall(vector.clamp, pos, corner, vector.zero)) --> false, invalid argument #3 to 'clamp' (max.x must be greater than or equal to min.x)"#,
    },
    Entry {
        name: "create",
        kind: Function,
        sigs: &["(x: number, y: number, z: number?): vector"],
        params: &["x", "y", "z"],
        returns: true,
        example: r#"local pos = vector.create(120, 45)
print(pos) --> 120, 45, 0
print(pos.x, pos.y) --> 120, 45"#,
    },
    Entry {
        name: "cross",
        kind: Function,
        sigs: &["(vec1: vector, vec2: vector): vector"],
        params: &["vec1", "vec2"],
        returns: true,
        example: r#"local x = vector.create(1, 0, 0)
local y = vector.create(0, 1, 0)
print(vector.cross(x, y)) --> 0, 0, 1"#,
    },
    Entry {
        name: "dot",
        kind: Function,
        sigs: &["(vec1: vector, vec2: vector): number"],
        params: &["vec1", "vec2"],
        returns: true,
        example: r#"local facing = vector.create(1, 0)
print(vector.dot(facing, vector.create(3, 4))) --> 3
print(vector.dot(facing, vector.create(-2, 1))) --> -2"#,
    },
    Entry {
        name: "floor",
        kind: Function,
        sigs: &["(vec: vector): vector"],
        params: &["vec"],
        returns: true,
        example: r#"local pos = vector.create(1.7, -1.2)
print(vector.floor(pos)) --> 1, -2, 0"#,
    },
    Entry {
        name: "lerp",
        kind: Function,
        sigs: &["(vec1: vector, vec2: vector, t: number): vector"],
        params: &["vec1", "vec2", "t"],
        returns: true,
        example: r#"local start = vector.create(0, 0)
local goal = vector.create(10, 20)
print(vector.lerp(start, goal, 0.25)) --> 2.5, 5, 0
print(vector.lerp(start, goal, 2)) --> 20, 40, 0"#,
    },
    Entry {
        name: "magnitude",
        kind: Function,
        sigs: &["(vec: vector): number"],
        params: &["vec"],
        returns: true,
        example: r#"local hero = vector.create(4, 8)
local coin = vector.create(10, 0)
print(vector.magnitude(coin - hero)) --> 10"#,
    },
    Entry {
        name: "max",
        kind: Function,
        sigs: &["(vec: vector, ...: vector): vector"],
        params: &["vec", "..."],
        returns: true,
        example: r#"local a = vector.create(1, 5)
local b = vector.create(4, 2)
print(vector.max(a, b, vector.create(0, 9))) --> 4, 9, 0"#,
    },
    Entry {
        name: "min",
        kind: Function,
        sigs: &["(vec: vector, ...: vector): vector"],
        params: &["vec", "..."],
        returns: true,
        example: r#"local a = vector.create(1, 5)
local b = vector.create(4, 2)
print(vector.min(a, b, vector.create(0, 9))) --> 0, 2, 0"#,
    },
    Entry {
        name: "normalize",
        kind: Function,
        sigs: &["(vec: vector): vector"],
        params: &["vec"],
        returns: true,
        example: r#"local dir = vector.normalize(vector.create(3, 4))
print(dir) --> 0.6000000238418579, 0.800000011920929, 0
print(vector.normalize(vector.zero)) --> nan, nan, nan"#,
    },
    Entry {
        name: "one",
        kind: Constant,
        sigs: &["vector"],
        params: &[],
        returns: false,
        example: r#"print(vector.one) --> 1, 1, 1
print(vector.one * 5) --> 5, 5, 5"#,
    },
    Entry {
        name: "sign",
        kind: Function,
        sigs: &["(vec: vector): vector"],
        params: &["vec"],
        returns: true,
        example: r#"local speed = vector.create(-0.5, 0, 7)
print(vector.sign(speed)) --> -1, 0, 1"#,
    },
    Entry {
        name: "zero",
        kind: Constant,
        sigs: &["vector"],
        params: &[],
        returns: false,
        example: r#"print(vector.zero) --> 0, 0, 0
local pos = vector.create(3, 4)
print(pos + vector.zero == pos) --> true"#,
    },
];
