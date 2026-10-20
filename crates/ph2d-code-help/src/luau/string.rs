//! A página `string` (assinaturas e exemplos; o texto em `ph2d-i18n/src/code_help/string.rs`).

use super::Entry;
#[allow(unused_imports)]
use crate::Kind::{Constant, Function};

pub(crate) const ENTRIES: &[Entry] = &[
    Entry {
        name: "byte",
        kind: Function,
        sigs: &["(s: string, i: number?, j: number?): ...number"],
        params: &["s", "i", "j"],
        returns: true,
        example: r#"local name = "Ana"
print(string.byte(name)) --> 65
print(name:byte(1, 3)) --> 65, 110, 97"#,
    },
    Entry {
        name: "char",
        kind: Function,
        sigs: &["(...: number): string"],
        params: &["..."],
        returns: true,
        example: r#"print(string.char(72, 105)) --> Hi"#,
    },
    Entry {
        name: "find",
        kind: Function,
        sigs: &[
            "(s: string, pattern: string, init: number?, plain: boolean?): (number?, number?, ...string)",
        ],
        params: &["s", "pattern", "init", "plain"],
        returns: true,
        example: r#"local file = "hero_walk_01.png"
print(string.find(file, "walk")) --> 6, 9
print(string.find(file, ".", 1, true)) --> 13, 13
print(string.find("Score: 120", "(%d+)")) --> 8, 10, 120
print(string.find(file, "jump")) --> nil"#,
    },
    Entry {
        name: "format",
        kind: Function,
        sigs: &["(format: string, ...: any): string"],
        params: &["format", "..."],
        returns: true,
        example: r#"print(string.format("Score: %d", 1500)) --> Score: 1500
print(string.format("Time: %.1f s", 7.456)) --> Time: 7.5 s
print(string.format("%02d:%02d", 3, 7)) --> 03:07
print(string.format("%* is %*", "door", true)) --> door is true"#,
    },
    Entry {
        name: "gmatch",
        kind: Function,
        sigs: &["(s: string, pattern: string): () -> (...string)"],
        params: &["s", "pattern"],
        returns: true,
        example: r#"local stats = {}
for key, value in string.gmatch("hp=10, mp=5", "(%a+)=(%d+)") do
	stats[key] = tonumber(value)
end
print(stats.hp, stats.mp) --> 10, 5"#,
    },
    Entry {
        name: "gsub",
        kind: Function,
        sigs: &[
            "(s: string, pattern: string, repl: ((string) -> string) | string | { [string]: string }, n: number?): (string, number)",
        ],
        params: &["s", "pattern", "repl", "n"],
        returns: true,
        example: r#"local text, count = string.gsub("Hello, $name!", "%$name", "Mia")
print(text) --> Hello, Mia!
print(count) --> 1
local loud = string.gsub("red cat", "%a+", function(word) return word:upper() end)
print(loud) --> RED CAT"#,
    },
    Entry {
        name: "len",
        kind: Function,
        sigs: &["(s: string): number"],
        params: &["s"],
        returns: true,
        example: r#"print(string.len("Hero")) --> 4
print(("olá"):len(), utf8.len("olá")) --> 4, 3"#,
    },
    Entry {
        name: "lower",
        kind: Function,
        sigs: &["(s: string): string"],
        params: &["s"],
        returns: true,
        example: r#"print(string.lower("GAME OVER")) --> game over
print(("ÉPICO"):lower()) --> Épico"#,
    },
    Entry {
        name: "match",
        kind: Function,
        sigs: &["(s: string, pattern: string, init: number?): ...string"],
        params: &["s", "pattern", "init"],
        returns: true,
        example: r#"local label = "Level 12"
print(string.match(label, "%d+")) --> 12
print(string.match(label, "(%a+) (%d+)")) --> Level, 12
print(string.match(label, "boss")) --> nil"#,
    },
    Entry {
        name: "pack",
        kind: Function,
        sigs: &["(format: string, ...: any): string"],
        params: &["format", "..."],
        returns: true,
        example: r#"local data = string.pack("<i2 f", 300, 1.5)
print(#data) --> 6
print(string.unpack("<i2 f", data)) --> 300, 1.5, 7"#,
    },
    Entry {
        name: "packsize",
        kind: Function,
        sigs: &["(s: string): number"],
        params: &["s"],
        returns: true,
        example: r#"print(string.packsize("<i2 f")) --> 6
print((pcall(string.packsize, "s"))) --> false"#,
    },
    Entry {
        name: "rep",
        kind: Function,
        sigs: &["(s: string, n: number): string"],
        params: &["s", "n"],
        returns: true,
        example: r#"print(string.rep("=", 10)) --> ==========
print(("ab"):rep(3)) --> ababab"#,
    },
    Entry {
        name: "reverse",
        kind: Function,
        sigs: &["(s: string): string"],
        params: &["s"],
        returns: true,
        example: r#"print(string.reverse("stressed")) --> desserts"#,
    },
    Entry {
        name: "split",
        kind: Function,
        sigs: &["(s: string, separator: string?): {string}"],
        params: &["s", "separator"],
        returns: true,
        example: r#"local states = string.split("idle,walk,run", ",")
print(#states, states[2]) --> 3, walk
local cells = ("a,,b"):split(",")
print(#cells, cells[2] == "") --> 3, true"#,
    },
    Entry {
        name: "sub",
        kind: Function,
        sigs: &["(s: string, i: number, j: number?): string"],
        params: &["s", "i", "j"],
        returns: true,
        example: r#"local name = "PlayerOne"
print(string.sub(name, 1, 6)) --> Player
print(name:sub(7)) --> One
print(name:sub(-3)) --> One"#,
    },
    Entry {
        name: "unpack",
        kind: Function,
        sigs: &["(format: string, data: string, pos: number?): ...any"],
        params: &["format", "data", "pos"],
        returns: true,
        example: r#"local data = string.pack("<i2 i2", 640, 480)
local width, height, nextPos = string.unpack("<i2 i2", data)
print(width, height, nextPos) --> 640, 480, 5"#,
    },
    Entry {
        name: "upper",
        kind: Function,
        sigs: &["(s: string): string"],
        params: &["s"],
        returns: true,
        example: r#"print(string.upper("game over")) --> GAME OVER
print(("café"):upper()) --> CAFé"#,
    },
];
