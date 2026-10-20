//! A página `globals` (assinaturas e exemplos; o texto em `ph2d-i18n/src/code_help/globals.rs`).

use super::Entry;
#[allow(unused_imports)]
use crate::Kind::{Constant, Function};

pub(crate) const ENTRIES: &[Entry] = &[
    Entry {
        name: "_G",
        kind: Constant,
        sigs: &["any"],
        params: &[],
        returns: false,
        example: r#"print(_G.math == math) --> true
local ok = pcall(function() _G.score = 10 end)
print(ok) --> false"#,
    },
    Entry {
        name: "_VERSION",
        kind: Constant,
        sigs: &["string"],
        params: &[],
        returns: false,
        example: r#"print(string.sub(_VERSION, 1, 4)) --> Luau"#,
    },
    Entry {
        name: "assert",
        kind: Function,
        sigs: &["<T>(value: T, errorMessage: string?): T"],
        params: &["value", "errorMessage"],
        returns: true,
        example: r#"local hero = { name = "Hero", hp = 10 }
local h = assert(hero, "no hero in the scene")
print(h.name) --> Hero
print(pcall(assert, nil, "no enemy in the scene")) --> false, no enemy in the scene"#,
    },
    Entry {
        name: "collectgarbage",
        kind: Function,
        sigs: &["(option: \"count\"): number"],
        params: &["option"],
        returns: true,
        example: r#"local kb = collectgarbage("count")
print(kb > 0) --> true
print(pcall(collectgarbage, "collect")) --> false, collectgarbage called with invalid option"#,
    },
    Entry {
        name: "error",
        kind: Function,
        sigs: &["<T>(message: T, level: number?): never"],
        params: &["message", "level"],
        returns: false,
        example: r#"local function damage(hp, amount)
  if amount < 0 then error("damage cannot be negative", 0) end
  return hp - amount
end
print(pcall(damage, 10, -5)) --> false, damage cannot be negative"#,
    },
    Entry {
        name: "gcinfo",
        kind: Function,
        sigs: &["(): number"],
        params: &[],
        returns: true,
        example: r#"local kb = gcinfo()
print(kb > 0, kb == math.floor(kb)) --> true, true"#,
    },
    Entry {
        name: "getfenv",
        kind: Function,
        sigs: &["(target: any): { [string]: any }"],
        params: &["target"],
        returns: true,
        example: r#"local env = getfenv()
print(env.math == math) --> true
print(env == _G) --> false"#,
    },
    Entry {
        name: "getmetatable",
        kind: Function,
        sigs: &["<MT>(object: setmetatable<{  }, MT>): MT"],
        params: &["object"],
        returns: true,
        example: r#"local mt = { __index = { speed = 5 } }
local enemy = setmetatable({}, mt)
print(getmetatable(enemy) == mt) --> true
print(getmetatable({ name = "Bat" })) --> nil"#,
    },
    Entry {
        name: "ipairs",
        kind: Function,
        sigs: &["<V>(tab: {V}): (({V}, number) -> (number?, V), {V}, number)"],
        params: &["tab"],
        returns: true,
        example: r#"local names = { "Hero", "Slime", nil, "Bat" }
local seen = {}
for i, name in ipairs(names) do seen[#seen + 1] = i .. ":" .. name end
print(table.concat(seen, " ")) --> 1:Hero 2:Slime"#,
    },
    Entry {
        name: "loadstring",
        kind: Function,
        sigs: &["<A...>(src: string, chunkname: string?): (((A...) -> any)?, string?)"],
        params: &["src", "chunkname"],
        returns: true,
        example: r#"local f = loadstring("return 2 + 3")
print(f()) --> 5
local g, err = loadstring("return +")
print(g) --> nil"#,
    },
    Entry {
        name: "newproxy",
        kind: Function,
        sigs: &["(mt: boolean?): any"],
        params: &["mt"],
        returns: true,
        example: r#"local chest = newproxy(true)
getmetatable(chest).__tostring = function() return "Treasure chest" end
print(type(chest), tostring(chest)) --> userdata, Treasure chest"#,
    },
    Entry {
        name: "next",
        kind: Function,
        sigs: &["<K, V>(t: { [K]: V }, key: K?): (K?, V)"],
        params: &["t", "key"],
        returns: true,
        example: r#"local inventory = {}
print(next(inventory) == nil) --> true
local items = { "sword", "shield" }
print(next(items)) --> 1, sword
print(next(items, 1)) --> 2, shield"#,
    },
    Entry {
        name: "pairs",
        kind: Function,
        sigs: &["<K, V>(t: { [K]: V }): (({ [K]: V }, K?) -> (K?, V), { [K]: V }, nil)"],
        params: &["t"],
        returns: true,
        example: r#"local stats = { hp = 10, mp = 4, stamina = 6 }
local total = 0
for name, value in pairs(stats) do total += value end
print(total) --> 20"#,
    },
    Entry {
        name: "pcall",
        kind: Function,
        sigs: &["<A..., R...>(f: (A...) -> (R...), ...: A...): boolean, R..."],
        params: &["f", "..."],
        returns: true,
        example: r#"local function double(x) return x * 2 end
print(pcall(double, 21)) --> true, 42
print(pcall(function() error("out of ammo", 0) end)) --> false, out of ammo"#,
    },
    Entry {
        name: "print",
        kind: Function,
        sigs: &["<T...>(...: T...)"],
        params: &["..."],
        returns: false,
        example: r#"local name, hp = "Hero", 10
print(name, hp, hp > 5) --> Hero, 10, true
print("Lives left: " .. 3) --> Lives left: 3"#,
    },
    Entry {
        name: "rawequal",
        kind: Function,
        sigs: &["<T1, T2>(a: T1, b: T2): boolean"],
        params: &["a", "b"],
        returns: true,
        example: r#"local a, b = { x = 1 }, { x = 1 }
print(rawequal(a, b), rawequal(a, a)) --> false, true"#,
    },
    Entry {
        name: "rawget",
        kind: Function,
        sigs: &["<K, V>(tab: { [K]: V }, k: K): V?"],
        params: &["tab", "k"],
        returns: true,
        example: r#"local defaults = { speed = 5 }
local enemy = setmetatable({}, { __index = defaults })
print(enemy.speed, rawget(enemy, "speed")) --> 5, nil"#,
    },
    Entry {
        name: "rawlen",
        kind: Function,
        sigs: &["<K, V>(obj: string | { [K]: V }): number"],
        params: &["obj"],
        returns: true,
        example: r#"print(rawlen({ 10, 20, 30 }), rawlen("Hero")) --> 3, 4"#,
    },
    Entry {
        name: "rawset",
        kind: Function,
        sigs: &["<K, V>(tab: { [K]: V }, k: K, v: V): { [K]: V }"],
        params: &["tab", "k", "v"],
        returns: true,
        example: r#"local guarded = setmetatable({}, { __newindex = function() error("locked", 0) end })
print(pcall(function() guarded.level = 1 end)) --> false, locked
rawset(guarded, "level", 1)
print(guarded.level) --> 1"#,
    },
    Entry {
        name: "require",
        kind: Function,
        sigs: &["(target: any): any"],
        params: &["target"],
        returns: true,
        example: r#"local ok = pcall(require, "./enemy")
print(ok) --> false"#,
    },
    Entry {
        name: "select",
        kind: Function,
        sigs: &["<A...>(i: number | string, ...: A...): ...any"],
        params: &["i", "..."],
        returns: true,
        example: r##"print(select("#", "a", nil, "c")) --> 3
print(select(2, "Hero", "Slime", "Bat")) --> Slime, Bat
print(select(-1, "Hero", "Slime", "Bat")) --> Bat"##,
    },
    Entry {
        name: "setfenv",
        kind: Function,
        sigs: &[
            "<T..., R...>(target: ((T...) -> (R...)) | number, env: { [string]: any }): ((T...) -> (R...))?",
        ],
        params: &["target", "env"],
        returns: true,
        example: r#"local function greet() return greeting end
setfenv(greet, { greeting = "Hello" })
print(greet()) --> Hello"#,
    },
    Entry {
        name: "setmetatable",
        kind: Function,
        sigs: &["<MT>(t: {  }, metatable: MT): setmetatable<{  }, MT>"],
        params: &["t", "metatable"],
        returns: true,
        example: r#"local Enemy = { __index = { speed = 5 } }
local bat = setmetatable({ name = "Bat" }, Enemy)
print(bat.name, bat.speed) --> Bat, 5
print(pcall(setmetatable, math, {})) --> false, attempt to modify a readonly table"#,
    },
    Entry {
        name: "tonumber",
        kind: Function,
        sigs: &["<T>(value: T, radix: number?): number?"],
        params: &["value", "radix"],
        returns: true,
        example: r#"print(tonumber("42"), tonumber("3.5")) --> 42, 3.5
print(tonumber("ff", 16), tonumber("five")) --> 255, nil"#,
    },
    Entry {
        name: "tostring",
        kind: Function,
        sigs: &["<T>(value: T): string"],
        params: &["value"],
        returns: true,
        example: r#"local score = 1500
print("Score: " .. tostring(score)) --> Score: 1500
print(tostring(nil), tostring(0.1 + 0.2)) --> nil, 0.30000000000000004"#,
    },
    Entry {
        name: "type",
        kind: Function,
        sigs: &["<T>(value: T): string"],
        params: &["value"],
        returns: true,
        example: r#"print(type(10), type("Hero"), type(nil), type({})) --> number, string, nil, table
print(type(print), type(vector.create(1, 2))) --> function, vector"#,
    },
    Entry {
        name: "typeof",
        kind: Function,
        sigs: &["<T>(value: T): string"],
        params: &["value"],
        returns: true,
        example: r#"print(typeof(5), typeof(vector.create(1, 2))) --> number, vector
print(typeof(ph2d.vec2(1, 2))) --> table"#,
    },
    Entry {
        name: "unpack",
        kind: Function,
        sigs: &["<V>(tab: {V}, i: number?, j: number?): ...V"],
        params: &["tab", "i", "j"],
        returns: true,
        example: r#"local pos = { 120, 80 }
local x, y = unpack(pos)
print(x, y) --> 120, 80
print(unpack({ "a", "b", "c", "d" }, 2, 3)) --> b, c"#,
    },
    Entry {
        name: "xpcall",
        kind: Function,
        sigs: &[
            "<E, A..., R1..., R2...>(f: (A...) -> (R1...), err: (E) -> (R2...), ...: A...): boolean, R1...",
        ],
        params: &["f", "err", "..."],
        returns: true,
        example: r#"local function onError(message) return "caught: " .. message end
print(xpcall(function() error("bad jump", 0) end, onError)) --> false, caught: bad jump
print(xpcall(function(a, b) return a + b end, onError, 2, 3)) --> true, 5"#,
    },
];
