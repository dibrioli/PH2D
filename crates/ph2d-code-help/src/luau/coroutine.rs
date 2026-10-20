//! A página `coroutine` (assinaturas e exemplos; o texto em `ph2d-i18n/src/code_help/coroutine.rs`).

use super::Entry;
#[allow(unused_imports)]
use crate::Kind::{Constant, Function};

pub(crate) const ENTRIES: &[Entry] = &[
    Entry {
        name: "close",
        kind: Function,
        sigs: &["(co: thread): (boolean, any)"],
        params: &["co"],
        returns: true,
        example: r#"local walk = coroutine.create(function() coroutine.yield("step") end)
coroutine.resume(walk)
print(coroutine.close(walk), coroutine.status(walk)) --> true, dead"#,
    },
    Entry {
        name: "create",
        kind: Function,
        sigs: &["<A..., R...>(f: (A...) -> (R...)): thread"],
        params: &["f"],
        returns: true,
        example: r#"local patrol = coroutine.create(function(name)
  coroutine.yield(name .. " goes left")
  return name .. " goes right"
end)
print(coroutine.resume(patrol, "Guard")) --> true, Guard goes left"#,
    },
    Entry {
        name: "isyieldable",
        kind: Function,
        sigs: &["(): boolean"],
        params: &[],
        returns: true,
        example: r#"print(coroutine.isyieldable()) --> false
local co = coroutine.create(function() return coroutine.isyieldable() end)
print(coroutine.resume(co)) --> true, true"#,
    },
    Entry {
        name: "resume",
        kind: Function,
        sigs: &["<A..., R...>(co: thread, ...: A...): boolean, R..."],
        params: &["co", "..."],
        returns: true,
        example: r#"local counter = coroutine.create(function(start) local step = coroutine.yield(start) return start + step end)
print(coroutine.resume(counter, 10)) --> true, 10
print(coroutine.resume(counter, 5)) --> true, 15
print(coroutine.resume(counter)) --> false, cannot resume dead coroutine"#,
    },
    Entry {
        name: "running",
        kind: Function,
        sigs: &["(): thread"],
        params: &[],
        returns: true,
        example: r#"print(coroutine.running()) --> nil
local co
co = coroutine.create(function() return coroutine.running() == co end)
print(coroutine.resume(co)) --> true, true"#,
    },
    Entry {
        name: "status",
        kind: Function,
        sigs: &["(co: thread): \"dead\" | \"normal\" | \"running\" | \"suspended\""],
        params: &["co"],
        returns: true,
        example: r#"local co = coroutine.create(function() coroutine.yield() end)
print(coroutine.status(co)) --> suspended
coroutine.resume(co)
coroutine.resume(co)
print(coroutine.status(co)) --> dead"#,
    },
    Entry {
        name: "wrap",
        kind: Function,
        sigs: &["<A..., R...>(f: (A...) -> (R...)): (A...) -> (R...)"],
        params: &["f"],
        returns: true,
        example: r#"local nextFrame = coroutine.wrap(function()
  for frame = 1, 3 do coroutine.yield(frame) end
end)
print(nextFrame(), nextFrame(), nextFrame()) --> 1, 2, 3"#,
    },
    Entry {
        name: "yield",
        kind: Function,
        sigs: &["<A..., R...>(...: A...): R..."],
        params: &["..."],
        returns: true,
        example: r#"local blink = coroutine.wrap(function()
  while true do coroutine.yield("on") coroutine.yield("off") end
end)
print(blink(), blink(), blink()) --> on, off, on
print(pcall(coroutine.yield)) --> false, attempt to yield across metamethod/C-call boundary"#,
    },
];
