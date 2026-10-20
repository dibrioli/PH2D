//! O texto da página `table` do editor de scripts (as assinaturas e os exemplos em
//! `ph2d-code-help/src/luau/table.rs`).

pub(super) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "code.help.table" => {
            "Lists and tables: adding, removing, finding, sorting, copying and locking."
        }
        "code.help.table.about" => {
            "A list is a table whose items sit at positions 1, 2, 3… with no gaps. Positions start at 1, not 0, and `#list` gives the number of items. Most functions here only look at that numbered part and stop at the first gap (a `nil`).\n\nIn a PH2D script the built-in libraries such as `math`, `string` and `table` are frozen: you can read them but not change them. Tables your script creates are not frozen unless you call [table.freeze]."
        }
        "code.help.table.clear" => {
            "Empties a table, removing every item and every key.\n\nThe same table stays, so every variable that points to it now sees it empty. Faster than making a new table when you refill a list every frame. A frozen table raises an error."
        }
        "code.help.table.clear.table" => "The table to empty.",
        "code.help.table.clone" => {
            "Makes a copy of a table, with the same keys and values.\n\nThe copy is shallow: a table stored inside is shared, not copied. The copy keeps the metatable, and it is never frozen, even when the original is, so cloning is how you get an editable version of a frozen table."
        }
        "code.help.table.clone.t" => "The table to copy.",
        "code.help.table.clone.returns" => "A new table with the same contents.",
        "code.help.table.concat" => {
            "Joins the items of a list into one text, with a separator between them.\n\nThe items must be text or numbers; a `true` or a table inside raises an error. An empty list gives an empty text."
        }
        "code.help.table.concat.t" => "The list to join.",
        "code.help.table.concat.sep" => "The text placed between items. Defaults to nothing.",
        "code.help.table.concat.i" => "The first position to join. Defaults to 1.",
        "code.help.table.concat.j" => "The last position to join. Defaults to `#t`.",
        "code.help.table.concat.returns" => "The joined text.",
        "code.help.table.create" => {
            "Makes a new list with room for a number of items, optionally filled with one value.\n\nWith a value, every slot holds it and `#` is the count. Without a value the room is reserved but the slots stay empty, so `#` is 0. Use it to start a grid or a score board of a known size."
        }
        "code.help.table.create.count" => "How many items. A negative number is an error.",
        "code.help.table.create.value" => {
            "The value to put in every slot. Leave it out to keep the slots empty."
        }
        "code.help.table.create.returns" => "The new list.",
        "code.help.table.find" => {
            "Looks for a value in a list and gives its position.\n\nIt returns `nil` when the value is not there, so check the result before using it. It compares with `==`, so two different tables with the same contents do not match."
        }
        "code.help.table.find.haystack" => "The list to search.",
        "code.help.table.find.needle" => "The value to look for.",
        "code.help.table.find.init" => "The position to start from. Defaults to 1.",
        "code.help.table.find.returns" => "The position of the first match, or `nil`.",
        "code.help.table.foreach" => {
            "Calls a function once for every key and value of a table.\n\nAn older form kept for compatibility; a `for key, value in pairs(t) do` loop is clearer. If the function returns anything other than `nil`, the walk stops and that value is returned."
        }
        "code.help.table.foreach.t" => "The table to walk through.",
        "code.help.table.foreach.f" => "The function to call with each key and value.",
        "code.help.table.foreachi" => {
            "Calls a function once for every item of a list, in order.\n\nAn older form kept for compatibility; a `for i, value in ipairs(list) do` loop is clearer. Returning anything other than `nil` from the function stops the walk."
        }
        "code.help.table.foreachi.t" => "The list to walk through.",
        "code.help.table.foreachi.f" => "The function to call with each position and item.",
        "code.help.table.freeze" => {
            "Locks a table so nothing can be added, changed or removed.\n\nAfter freezing, any change raises an error (catch it with [pcall]). The lock is shallow: tables stored inside can still change. Freezing cannot be undone, but [table.clone] gives an editable copy. Freezing a table twice is an error.\n\nUse it for settings that must never change by accident."
        }
        "code.help.table.freeze.t" => "The table to lock.",
        "code.help.table.freeze.returns" => "The same table, now frozen.",
        "code.help.table.getn" => {
            "Counts the items of a list, the same as `#t`.\n\nAn older form kept for compatibility; write `#t`."
        }
        "code.help.table.getn.t" => "The list to count.",
        "code.help.table.getn.returns" => "The number of items.",
        "code.help.table.insert" => {
            "Adds an item to a list, at the end or at a chosen position.\n\nWith a position, the items from there onwards move up by one to make room. Note: a position beyond `#t + 1` is not an error, but the item lands after a gap and `#t` does not count it."
        }
        "code.help.table.insert.t" => "The list to add to.",
        "code.help.table.insert.value" => "The item to add.",
        "code.help.table.insert.pos" => {
            "Where to put it: 1 puts it first. Leave it out to add at the end."
        }
        "code.help.table.isfrozen" => {
            "Tells whether a table is locked by [table.freeze].\n\nThe built-in libraries, such as `math`, are frozen in PH2D scripts."
        }
        "code.help.table.isfrozen.t" => "The table to check.",
        "code.help.table.isfrozen.returns" => "`true` if the table is frozen, `false` otherwise.",
        "code.help.table.maxn" => {
            "Gives the largest positive number used as a key, even after gaps.\n\nUnlike `#t`, it sees items placed past a gap. It walks through the whole table, so it is slower than `#t`. An empty table gives 0."
        }
        "code.help.table.maxn.t" => "The table to check.",
        "code.help.table.maxn.returns" => "The largest positive number key, or 0.",
        "code.help.table.move" => {
            "Copies a run of items from one list into another, or to another place in the same list.\n\nIt copies the items at positions `a` to `b` so that the first lands at position `t`. Without a target list it writes into the source."
        }
        "code.help.table.move.src" => "The list to copy from.",
        "code.help.table.move.a" => "The first position to copy.",
        "code.help.table.move.b" => "The last position to copy.",
        "code.help.table.move.t" => "The position in the target where the first item lands.",
        "code.help.table.move.dst" => "The list to copy into. Defaults to `src`.",
        "code.help.table.move.returns" => "The target list.",
        "code.help.table.pack" => {
            "Puts any number of values into a new list, and records how many in `n`.\n\nThe count `n` includes `nil` values, which `#` could miss. Useful to keep all the extra values a function received with `...`."
        }
        "code.help.table.pack.vararg" => "The values to collect.",
        "code.help.table.pack.returns" => {
            "A list of the values, with the field `n` holding how many there were."
        }
        "code.help.table.remove" => {
            "Takes an item out of a list and gives it back.\n\nWithout a position it removes the last item. The items after it move down by one, so there is no gap. An empty list gives `nil`."
        }
        "code.help.table.remove.t" => "The list to remove from.",
        "code.help.table.remove.pos" => "The position to remove. Defaults to the last item.",
        "code.help.table.remove.returns" => "The removed item, or `nil` if there was none.",
        "code.help.table.sort" => {
            "Puts the items of a list in order, changing the list itself.\n\nWithout a function, numbers go smallest first and text alphabetically, with capitals before small letters. Mixing numbers and text is an error.\n\nThe optional function receives two items and returns `true` when the first should come before the second. Note: it must answer `false` for two equal items, so use `<` or `>`, never `<=`. Items that are equal may end in any order."
        }
        "code.help.table.sort.t" => "The list to sort.",
        "code.help.table.sort.comp" => {
            "A function `(a, b)` returning `true` when `a` goes before `b`. Defaults to `a < b`."
        }
        "code.help.table.unpack" => {
            "Gives the items of a list as separate values.\n\nUseful to pass a list to a function that expects separate arguments, or to read several items into variables at once."
        }
        "code.help.table.unpack.list" => "The list to read.",
        "code.help.table.unpack.i" => "The first position. Defaults to 1.",
        "code.help.table.unpack.j" => "The last position. Defaults to `#list`.",
        "code.help.table.unpack.returns" => "The items from `i` to `j`, as separate values.",
        _ => return None,
    })
}
