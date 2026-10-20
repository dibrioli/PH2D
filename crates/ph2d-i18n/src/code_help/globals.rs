//! O texto da página `globals` do editor de scripts (as assinaturas e os exemplos em
//! `ph2d-code-help/src/luau/globals.rs`).

pub(super) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "code.help.globals.title" => "Global functions",
        "code.help.globals" => {
            "The functions you call by their name alone, with no library in front: `print`, `type`, `pairs` and friends."
        }
        "code.help.globals.about" => {
            "Each PH2D script runs in its own space. A global variable you create (`score = 10`) belongs to that script only, so two scripts never overwrite each other. The built-in libraries (`math`, `string`, `ph2d`…) are shared and locked: trying to change them stops the script with `attempt to modify a readonly table`.\n\nWhen something goes wrong, the error message starts with the script's name and the line number, and appears in the Console tab of the script editor."
        }
        "code.help.globals._G" => {
            "The table of shared, built-in names: `_G.math` is the same as `math`.\n\nNote: in PH2D this table is locked. Writing to it (`_G.score = 10`) stops the script with an error, and the globals your script creates are not in it. To see your own script's globals, use [getfenv]."
        }
        "code.help.globals._VERSION" => {
            "The name and version of the scripting language, as text.\n\nIt starts with `Luau` followed by a version number, which changes when PH2D updates its Luau."
        }
        "code.help.globals.assert" => {
            "Stops the script with an error when `value` is `false` or `nil`; otherwise hands `value` back.\n\nUse it to check something you are counting on, such as an object found by name: `local hero = assert(ph2d.find_by_name(\"Hero\"), \"no Hero\")`. Without a message the error says `assertion failed!`."
        }
        "code.help.globals.assert.value" => "The value to check.",
        "code.help.globals.assert.errorMessage" => {
            "Optional. The message of the error when the check fails."
        }
        "code.help.globals.assert.returns" => {
            "`value` itself (and any extra arguments) when the check passes."
        }
        "code.help.globals.collectgarbage" => {
            "Tells how much memory the scripts are using, in kilobytes.\n\nNote: in PH2D only `collectgarbage(\"count\")` works. Every other option (`\"collect\"`, `\"step\"`, `\"stop\"`…), and calling it with nothing, stops the script with `collectgarbage called with invalid option`. Memory is cleaned up automatically."
        }
        "code.help.globals.collectgarbage.option" => "Must be `\"count\"`.",
        "code.help.globals.collectgarbage.returns" => {
            "The memory in use, in kilobytes, with a fractional part."
        }
        "code.help.globals.error" => {
            "Stops the running code with a message, like a crash you cause on purpose.\n\nIf nothing catches it with [pcall] or [xpcall], the script stops and the message appears in the Console. A text message gets the script name and line in front (`player:12: message`); pass `level` 0 to leave them out."
        }
        "code.help.globals.error.message" => {
            "What went wrong. Usually text, but any value works; [pcall] hands it back untouched."
        }
        "code.help.globals.error.level" => {
            "Optional. Which line the message points to: 1 (the default) is where `error` was called, 2 is the code that called that function, 0 adds no position at all."
        }
        "code.help.globals.gcinfo" => {
            "Tells how much memory the scripts are using, in whole kilobytes.\n\nThe same reading as `collectgarbage(\"count\")`, rounded down."
        }
        "code.help.globals.gcinfo.returns" => "The memory in use, in whole kilobytes.",
        "code.help.globals.getfenv" => {
            "Gives the table that holds a script's global variables.\n\nIn PH2D each script has its own: the globals the script creates live in it, and looking up a name it does not have falls through to the shared built-ins. So `getfenv().math` is `math`, but `getfenv()` is not [_G].\n\nNote: using `getfenv` or [setfenv] makes Luau switch off some speed-ups for that script. You rarely need them."
        }
        "code.help.globals.getfenv.target" => {
            "Optional. A function, or a level number (1 = the current function, 2 = its caller). Leave it out for the current script."
        }
        "code.help.globals.getfenv.returns" => "The table of global variables.",
        "code.help.globals.getmetatable" => {
            "Gives the metatable of a value, or `nil` if it has none.\n\nIf the metatable has a `__metatable` field, you get that field instead of the real table: a way to keep a metatable private. Text values share one metatable, whose `__index` is the [string] library."
        }
        "code.help.globals.getmetatable.object" => "The value whose metatable you want.",
        "code.help.globals.getmetatable.returns" => {
            "The metatable, its `__metatable` field, or `nil`."
        }
        "code.help.globals.ipairs" => {
            "Goes through a list in order, giving the position and the value of each item.\n\nIt starts at 1 and stops at the first missing (`nil`) item, so anything after a gap is skipped. For tables with names as keys, use [pairs]."
        }
        "code.help.globals.ipairs.tab" => {
            "The list to go through. Anything other than a table stops the script with an error."
        }
        "code.help.globals.ipairs.returns" => {
            "What a `for` loop needs; you normally write `for i, v in ipairs(list) do`."
        }
        "code.help.globals.loadstring" => {
            "Turns a piece of Luau code written as text into a function you can call.\n\nNote: in PH2D a mistake in the text stops the script with a `syntax error`, instead of giving `nil`; wrap the call in [pcall] if the text might be broken.\n\nNote: the new code sees the built-in names but not the global variables of your script, and the globals it creates are not your script's either. Hand it values as arguments instead: they arrive as `...`, as in `loadstring(\"local n = ... return n * 2\")(21)`."
        }
        "code.help.globals.loadstring.src" => "The Luau code, as text.",
        "code.help.globals.loadstring.chunkname" => {
            "Optional. The name shown in front of its error messages. Start it with `=` to use it exactly as written."
        }
        "code.help.globals.loadstring.returns" => "The new function.",
        "code.help.globals.newproxy" => {
            "Makes an empty value of type `userdata`, optionally with its own metatable.\n\nUseful only for advanced tricks: unlike a table, nothing can be stored in it directly, so its behaviour comes entirely from the metatable."
        }
        "code.help.globals.newproxy.mt" => {
            "Optional. `true` to give the value a fresh metatable you can fill with [getmetatable]."
        }
        "code.help.globals.newproxy.returns" => "The new value.",
        "code.help.globals.next" => {
            "Gives the key and value that come after `key` in a table.\n\nWith no `key` you get the first pair; after the last pair you get `nil`. `next(t) == nil` is the quick way to ask whether a table is empty. The order of named keys is not fixed."
        }
        "code.help.globals.next.t" => "The table to look in.",
        "code.help.globals.next.key" => "Optional. The key you are at now; leave it out to start.",
        "code.help.globals.next.returns" => "The next key and its value, or `nil` at the end.",
        "code.help.globals.pairs" => {
            "Goes through every key and value of a table.\n\nNote: the order is not guaranteed for named keys, so do not count on it. In Luau you can also write `for k, v in t do` directly."
        }
        "code.help.globals.pairs.t" => {
            "The table to go through. Anything other than a table stops the script with an error."
        }
        "code.help.globals.pairs.returns" => {
            "What a `for` loop needs; you normally write `for key, value in pairs(t) do`."
        }
        "code.help.globals.pcall" => {
            "Calls a function in protected mode: if it fails, your script keeps going.\n\nThe first result tells whether it worked. On success the function's own results follow; on failure the error message follows, and nothing is printed to the Console."
        }
        "code.help.globals.pcall.f" => "The function to call.",
        "code.help.globals.pcall.vararg" => "The arguments to pass to `f`.",
        "code.help.globals.pcall.returns" => {
            "`true` and the results of `f`, or `false` and the error."
        }
        "code.help.globals.print" => {
            "Writes values as a line in the Console tab of the script editor.\n\nEach value is turned into text with [tostring], and they are separated by a tab. A very long line is cut short. Use it to check what your script is doing while you work."
        }
        "code.help.globals.print.vararg" => "The values to write; any number of them, of any type.",
        "code.help.globals.rawequal" => {
            "Checks whether two values are the very same thing, ignoring any `__eq` in their metatables.\n\nTwo tables are only equal when they are the same table, even if they hold the same contents."
        }
        "code.help.globals.rawequal.a" => "The first value.",
        "code.help.globals.rawequal.b" => "The second value.",
        "code.help.globals.rawequal.returns" => "`true` when they are the same.",
        "code.help.globals.rawget" => {
            "Reads a key straight from a table, ignoring its metatable.\n\nUnlike `t[k]`, it does not fall back to `__index`, so it tells you what the table itself holds."
        }
        "code.help.globals.rawget.tab" => "The table to read.",
        "code.help.globals.rawget.k" => "The key to read.",
        "code.help.globals.rawget.returns" => "The value stored at that key, or `nil`.",
        "code.help.globals.rawlen" => {
            "The length of a table or a text, ignoring any `__len` in the metatable.\n\nAny other kind of value stops the script with an error."
        }
        "code.help.globals.rawlen.obj" => "A table or a text.",
        "code.help.globals.rawlen.returns" => {
            "The length: items in the list, or bytes in the text."
        }
        "code.help.globals.rawset" => {
            "Writes a key straight into a table, ignoring its metatable.\n\nUnlike `t[k] = v`, it skips `__newindex`. Note: it cannot write into the locked built-in libraries."
        }
        "code.help.globals.rawset.tab" => "The table to write into.",
        "code.help.globals.rawset.k" => "The key to write.",
        "code.help.globals.rawset.v" => "The value to store.",
        "code.help.globals.rawset.returns" => "The same table.",
        "code.help.globals.require" => {
            "Loads another code file as a module.\n\nNote: this does not work in PH2D. Scripts live inside the project, not as files on disk, so every call stops with an error. To share values between objects, use [ph2d.emit] signals or [ph2d.state_set]."
        }
        "code.help.globals.require.target" => "The path of the module.",
        "code.help.globals.require.returns" => "Nothing in PH2D: the call always fails.",
        "code.help.globals.select" => {
            "Picks values out of a list of arguments, or counts them.\n\nWith a number `i`, you get the `i`-th value and all after it; a negative number counts from the end. With `\"#\"`, you get how many values there are, `nil` ones included. `0` stops the script with an error."
        }
        "code.help.globals.select.i" => {
            "A position (from 1, or negative from the end), or `\"#\"` to count."
        }
        "code.help.globals.select.vararg" => "The values to choose from.",
        "code.help.globals.select.returns" => "The values from position `i` on, or the count.",
        "code.help.globals.setfenv" => {
            "Replaces the table of global variables that a function sees.\n\nAfter it, every global the function reads or writes goes to `env` instead. Note: like [getfenv], it makes Luau switch off some speed-ups for that script."
        }
        "code.help.globals.setfenv.target" => {
            "The function to change, or a level number (1 = the current function)."
        }
        "code.help.globals.setfenv.env" => "The new table of globals.",
        "code.help.globals.setfenv.returns" => "The function that was changed.",
        "code.help.globals.setmetatable" => {
            "Attaches a metatable to a table, giving it extra behaviour; hands the same table back.\n\nThe usual case is `__index`, which supplies default values or shared methods. Pass `nil` to remove the metatable.\n\nNote: the built-in libraries and [_G] are locked, so they cannot get a new metatable. A metatable with a `__metatable` field cannot be replaced either."
        }
        "code.help.globals.setmetatable.t" => "The table to change.",
        "code.help.globals.setmetatable.metatable" => {
            "The metatable to attach, or `nil` to remove it."
        }
        "code.help.globals.setmetatable.returns" => "The same table `t`.",
        "code.help.globals.tonumber" => {
            "Turns text into a number, or gives `nil` when the text is not a number.\n\nSpaces around the number are fine, and `0x` means hexadecimal. Extra words are not: `\"5 apples\"` gives `nil`. A value that is already a number comes back as it is."
        }
        "code.help.globals.tonumber.value" => "The text (or number) to convert.",
        "code.help.globals.tonumber.radix" => {
            "Optional. The base of the digits, from 2 to 36 (16 for hexadecimal). Anything else stops the script with an error."
        }
        "code.help.globals.tonumber.returns" => "The number, or `nil`.",
        "code.help.globals.tostring" => {
            "Turns any value into text.\n\nNumbers are written so they read back exactly, which is why `0.1 + 0.2` shows `0.30000000000000004`. Tables and functions show an address such as `table: 0x…`, unless their metatable has `__tostring`. To round a number for display, use [string.format]."
        }
        "code.help.globals.tostring.value" => "The value to convert.",
        "code.help.globals.tostring.returns" => "The text.",
        "code.help.globals.type" => {
            "Tells what kind of value something is, as text.\n\nThe answers are `nil`, `boolean`, `number`, `string`, `table`, `function`, `thread`, `userdata`, `vector` and `buffer`. Good for checking an argument before using it."
        }
        "code.help.globals.type.value" => "The value to look at.",
        "code.help.globals.type.returns" => "The kind of value, such as `\"number\"`.",
        "code.help.globals.typeof" => {
            "Like [type], but can also name special values made by the engine.\n\nIn PH2D scripts it gives the same answer as [type] in practice: PH2D's own values, such as [ph2d.vec2] positions, are tables."
        }
        "code.help.globals.typeof.value" => "The value to look at.",
        "code.help.globals.typeof.returns" => "The kind of value, such as `\"vector\"`.",
        "code.help.globals.unpack" => {
            "Spreads the items of a list out as separate values.\n\nHandy to pass a list as arguments, or to take a position apart: `local x, y = unpack(pos)`. It stops at the list's length, so items after a gap may be lost. The same as [table.unpack]."
        }
        "code.help.globals.unpack.tab" => "The list.",
        "code.help.globals.unpack.i" => "Optional. The first position to take; 1 when left out.",
        "code.help.globals.unpack.j" => {
            "Optional. The last position to take; the length of the list when left out."
        }
        "code.help.globals.unpack.returns" => "The items, one value each.",
        "code.help.globals.xpcall" => {
            "Like [pcall], but lets you handle the error with a function of your own.\n\nThe handler gets the error and its result replaces the message. Pass [debug.traceback] as the handler to get the list of calls that led to the error."
        }
        "code.help.globals.xpcall.f" => "The function to call.",
        "code.help.globals.xpcall.err" => "The function called with the error, if `f` fails.",
        "code.help.globals.xpcall.vararg" => "The arguments to pass to `f`.",
        "code.help.globals.xpcall.returns" => {
            "`true` and the results of `f`, or `false` and what the handler returned."
        }
        _ => return None,
    })
}
