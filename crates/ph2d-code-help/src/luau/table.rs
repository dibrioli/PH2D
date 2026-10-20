//! A página `table` (assinaturas e exemplos; o texto em `ph2d-i18n/src/code_help/table.rs`).

use super::Entry;
#[allow(unused_imports)]
use crate::Kind::{Constant, Function};

pub(crate) const ENTRIES: &[Entry] = &[
    Entry {
        name: "clear",
        kind: Function,
        sigs: &["(table: {  })"],
        params: &["table"],
        returns: false,
        example: r#"local enemies = {"bat", "slime", "ghost"}
table.clear(enemies)
print(#enemies, enemies[1]) --> 0, nil"#,
    },
    Entry {
        name: "clone",
        kind: Function,
        sigs: &["(t: {  }): {  }"],
        params: &["t"],
        returns: true,
        example: r#"local start = {x = 10, y = 20}
local pos = table.clone(start)
pos.x = 99
print(start.x, pos.x) --> 10, 99"#,
    },
    Entry {
        name: "concat",
        kind: Function,
        sigs: &["<V>(t: {V}, sep: string?, i: number?, j: number?): string"],
        params: &["t", "sep", "i", "j"],
        returns: true,
        example: r#"local items = {"sword", "shield", "potion"}
print(table.concat(items, " + ")) --> sword + shield + potion
print(table.concat(items, "/", 2, 3)) --> shield/potion
print(table.concat({1, 2, 3})) --> 123"#,
    },
    Entry {
        name: "create",
        kind: Function,
        sigs: &["<V>(count: number, value: V?): {V}"],
        params: &["count", "value"],
        returns: true,
        example: r#"local scores = table.create(3, 0)
print(#scores, scores[3]) --> 3, 0
local empty = table.create(100)
print(#empty) --> 0"#,
    },
    Entry {
        name: "find",
        kind: Function,
        sigs: &["<V>(haystack: {V}, needle: V, init: number?): number?"],
        params: &["haystack", "needle", "init"],
        returns: true,
        example: r#"local names = {"bat", "slime", "ghost", "slime"}
print(table.find(names, "slime")) --> 2
print(table.find(names, "slime", 3)) --> 4
print(table.find(names, "dragon")) --> nil"#,
    },
    Entry {
        name: "foreach",
        kind: Function,
        sigs: &["<K, V>(t: { [K]: V }, f: (K, V) -> ())"],
        params: &["t", "f"],
        returns: false,
        example: r#"table.foreach({hp = 10}, function(key, value)
	print(key, value) --> hp, 10
end)"#,
    },
    Entry {
        name: "foreachi",
        kind: Function,
        sigs: &["<V>(t: {V}, f: (number, V) -> ())"],
        params: &["t", "f"],
        returns: false,
        example: r#"table.foreachi({"idle", "walk"}, function(i, name)
	print(i, name)
end)
--> 1, idle
--> 2, walk"#,
    },
    Entry {
        name: "freeze",
        kind: Function,
        sigs: &["(t: {  }): {  }"],
        params: &["t"],
        returns: true,
        example: r#"local config = table.freeze({speed = 5})
local ok = pcall(function() config.speed = 9 end)
print(ok, config.speed) --> false, 5"#,
    },
    Entry {
        name: "getn",
        kind: Function,
        sigs: &["<V>(t: {V}): number"],
        params: &["t"],
        returns: true,
        example: r#"local frames = {"f1", "f2", "f3"}
print(table.getn(frames), #frames) --> 3, 3"#,
    },
    Entry {
        name: "insert",
        kind: Function,
        sigs: &[
            "<V>(t: {V}, value: V)",
            "<V>(t: {V}, pos: number, value: V)",
        ],
        params: &["t", "value", "pos"],
        returns: false,
        example: r#"local queue = {"walk", "jump"}
table.insert(queue, "attack")
table.insert(queue, 1, "idle")
print(table.concat(queue, " ")) --> idle walk jump attack"#,
    },
    Entry {
        name: "isfrozen",
        kind: Function,
        sigs: &["(t: {  }): boolean"],
        params: &["t"],
        returns: true,
        example: r#"local stats = {hp = 10}
print(table.isfrozen(stats)) --> false
table.freeze(stats)
print(table.isfrozen(stats), table.isfrozen(math)) --> true, true"#,
    },
    Entry {
        name: "maxn",
        kind: Function,
        sigs: &["<V>(t: {V}): number"],
        params: &["t"],
        returns: true,
        example: r#"local slots = {"sword", "bow", [10] = "key"}
print(table.maxn(slots), #slots) --> 10, 2"#,
    },
    Entry {
        name: "move",
        kind: Function,
        sigs: &["<V>(src: {V}, a: number, b: number, t: number, dst: {V}?): {V}"],
        params: &["src", "a", "b", "t", "dst"],
        returns: true,
        example: r#"local walk = {"w1", "w2", "w3", "w4"}
local lastTwo = table.move(walk, 3, 4, 1, {})
print(table.concat(lastTwo, ",")) --> w3,w4"#,
    },
    Entry {
        name: "pack",
        kind: Function,
        sigs: &["<V>(...: V): { [number]: V, n: number }"],
        params: &["..."],
        returns: true,
        example: r#"local args = table.pack("x", nil, "z")
print(args.n, args[1], args[3]) --> 3, x, z"#,
    },
    Entry {
        name: "remove",
        kind: Function,
        sigs: &["<V>(t: {V}, pos: number?): V?"],
        params: &["t", "pos"],
        returns: true,
        example: r#"local stack = {"red", "green", "blue"}
print(table.remove(stack)) --> blue
print(table.remove(stack, 1)) --> red
print(#stack, stack[1]) --> 1, green"#,
    },
    Entry {
        name: "sort",
        kind: Function,
        sigs: &["<V>(t: {V}, comp: ((V, V) -> boolean)?)"],
        params: &["t", "comp"],
        returns: false,
        example: r#"local scores = {50, 120, 80}
table.sort(scores)
print(table.concat(scores, " ")) --> 50 80 120
table.sort(scores, function(a, b) return a > b end)
print(table.concat(scores, " ")) --> 120 80 50"#,
    },
    Entry {
        name: "unpack",
        kind: Function,
        sigs: &["<V>(list: {V}, i: number?, j: number?): ...V"],
        params: &["list", "i", "j"],
        returns: true,
        example: r#"local pos = {640, 360}
local x, y = table.unpack(pos)
print(x, y) --> 640, 360
print(table.unpack({"a", "b", "c"}, 2)) --> b, c"#,
    },
];
