//! O texto da página `string` do editor de scripts (as assinaturas e os exemplos em
//! `ph2d-code-help/src/luau/string.rs`).

pub(super) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "code.help.string" => {
            "Text: searching, cutting, replacing, changing case and building text from numbers."
        }
        "code.help.string.about" => {
            "Every function here can also be called on the text itself, with a colon: `name:upper()` is the same as `string.upper(name)`.\n\nPositions count from 1, and a negative position counts back from the end (`-1` is the last character). Positions and `#text` count bytes, not letters: plain English letters are one byte each, but `é` takes two and an emoji four. To count letters use [utf8.len].\n\n[string.find], [string.match], [string.gmatch] and [string.gsub] search with Luau patterns, a small search language that is NOT the same as regular expressions. `.` means any character, `%d` a digit, `%a` a letter, `%w` a letter or digit, `%s` a space; `+` after one of those means one or more, `*` zero or more, `-` as few as possible; `^` pins the search to the start and `$` to the end; parentheses `( )` capture the part you want back. To search for a symbol that has a meaning, such as `.`, `%`, `+`, `-` or `$`, put `%` before it: `%.`."
        }
        "code.help.string.byte" => {
            "Gives the number code of one or more characters.\n\nWith no positions you get the code of the first byte. A letter like `é` is made of two bytes, so it gives two numbers; use [utf8.codepoint] for one number per letter. If the position is past the end, you get nothing back. [string.char] goes the other way."
        }
        "code.help.string.byte.s" => "The text to read.",
        "code.help.string.byte.i" => "The first position to read. Defaults to 1.",
        "code.help.string.byte.j" => {
            "The last position to read. Defaults to `i`, so only one byte is read."
        }
        "code.help.string.byte.returns" => "One number from 0 to 255 for each byte read.",
        "code.help.string.char" => {
            "Builds text from number codes, one byte per number.\n\nEach number must be from 0 to 255. For letters beyond plain English, such as `é` or emoji, use [utf8.char] instead."
        }
        "code.help.string.char.vararg" => "Any number of byte codes, from 0 to 255.",
        "code.help.string.char.returns" => "The text made of those bytes.",
        "code.help.string.find" => {
            "Looks for a piece of text and tells you where it starts and ends.\n\nIt returns `nil` when nothing is found, so check the result before using it. The search is a pattern (see the top of this page): to look for a dot or another symbol as plain text, pass `true` as the fourth argument. If the pattern has captures in parentheses, the captured pieces come after the two positions."
        }
        "code.help.string.find.s" => "The text to search in.",
        "code.help.string.find.pattern" => {
            "What to look for, as a pattern (or plain text when `plain` is `true`)."
        }
        "code.help.string.find.init" => {
            "The position where the search begins. Defaults to 1; a negative value counts from the end."
        }
        "code.help.string.find.plain" => {
            "`true` to search for the exact text, ignoring pattern symbols. Defaults to `false`."
        }
        "code.help.string.find.returns" => {
            "The start and end positions of the first match, then any captures; `nil` if not found."
        }
        "code.help.string.format" => {
            "Builds text by placing values into a template, such as `\"Score: %d\"`.\n\nEach `%` code in the template takes the next value: `%d` a whole number (a fraction is cut off, so `3.9` gives `3`), `%.2f` a number with 2 decimals, `%02d` a number padded with zeros to 2 digits, `%s` text, `%*` any value written as [tostring] would, and `%%` a percent sign.\n\nNote: `%s` only accepts text and numbers; giving it `true`, `nil` or a table is an error. Use `%*` when the value might be something else."
        }
        "code.help.string.format.format" => "The template text with `%` codes where the values go.",
        "code.help.string.format.vararg" => "The values, in the same order as the `%` codes.",
        "code.help.string.format.returns" => "The finished text.",
        "code.help.string.gmatch" => {
            "Walks through every match of a pattern, for use in a `for` loop.\n\nEach turn of the loop gets the next match. If the pattern has captures in parentheses, the loop gets one variable per capture. Use it to read every number in a line, or every `name=value` pair."
        }
        "code.help.string.gmatch.s" => "The text to search in.",
        "code.help.string.gmatch.pattern" => "The pattern to look for (see the top of this page).",
        "code.help.string.gmatch.returns" => {
            "A function for the `for` loop; each call gives the next match or its captures."
        }
        "code.help.string.gsub" => {
            "Replaces every match of a pattern and tells you how many it replaced.\n\nThe replacement can be text, where `%1` stands for the first capture and `%0` for the whole match; a table, where the match is looked up as a key; or a function, which receives the match and returns its replacement. If the table or function gives `nil` or `false`, the match is left as it was.\n\nNote: it returns two values. Wrap the call in extra parentheses, `(string.gsub(...))`, when you only want the text."
        }
        "code.help.string.gsub.s" => {
            "The original text. It is not changed; you get a new text back."
        }
        "code.help.string.gsub.pattern" => {
            "What to replace, as a pattern. Write `%$` or `%.` to match those symbols as themselves."
        }
        "code.help.string.gsub.repl" => {
            "The replacement: text, a table of replacements, or a function."
        }
        "code.help.string.gsub.n" => "The most replacements to make. Defaults to all of them.",
        "code.help.string.gsub.returns" => "The new text, then the number of replacements made.",
        "code.help.string.len" => {
            "Counts the bytes in a text, exactly like `#text`.\n\nNote: this is bytes, not letters, so `\"olá\"` gives 4. Use [utf8.len] to count letters."
        }
        "code.help.string.len.s" => "The text to measure.",
        "code.help.string.len.returns" => "The number of bytes.",
        "code.help.string.lower" => {
            "Turns capital letters into small letters.\n\nOnly the plain English letters A to Z change; accented letters such as `É` stay as they are. Handy for comparing names without caring about case."
        }
        "code.help.string.lower.s" => "The text to change.",
        "code.help.string.lower.returns" => "A new text in small letters.",
        "code.help.string.match" => {
            "Gives back the first part of a text that matches a pattern.\n\nIt returns `nil` when nothing matches. With captures in parentheses you get one value per capture instead of the whole match. The result is always text: use [tonumber] to turn `\"12\"` into the number 12."
        }
        "code.help.string.match.s" => "The text to search in.",
        "code.help.string.match.pattern" => "The pattern to look for (see the top of this page).",
        "code.help.string.match.init" => "The position where the search begins. Defaults to 1.",
        "code.help.string.match.returns" => {
            "The matched text or its captures; `nil` if there is no match."
        }
        "code.help.string.pack" => {
            "Packs numbers and text into a compact binary text, following a format.\n\nThe format lists one code per value, for example `i2` a 2-byte whole number, `f` a 4-byte decimal number, `d` an 8-byte one, `B` one byte, and `<` or `>` for the byte order. Read the result back with [string.unpack] and the same format. You rarely need this in an animation script; the `buffer` library is usually easier."
        }
        "code.help.string.pack.format" => "The list of codes describing each value.",
        "code.help.string.pack.vararg" => "The values to pack, one per code.",
        "code.help.string.pack.returns" => "The packed bytes, as a text.",
        "code.help.string.packsize" => {
            "Tells how many bytes [string.pack] would produce for a format.\n\nNote: formats with a piece of variable size, such as `s` (text with its length), have no fixed size and raise an error."
        }
        "code.help.string.packsize.s" => "The format, written as for [string.pack].",
        "code.help.string.packsize.returns" => "The size in bytes.",
        "code.help.string.rep" => {
            "Repeats a text a number of times.\n\nA count of 0 or less gives an empty text. There is no separator argument: an extra value after the count is ignored."
        }
        "code.help.string.rep.s" => "The text to repeat.",
        "code.help.string.rep.n" => "How many copies to make.",
        "code.help.string.rep.returns" => "The repeated text.",
        "code.help.string.reverse" => {
            "Writes a text backwards.\n\nNote: it reverses bytes, so letters made of several bytes, such as `é` or emoji, come out broken."
        }
        "code.help.string.reverse.s" => "The text to reverse.",
        "code.help.string.reverse.returns" => "The reversed text.",
        "code.help.string.split" => {
            "Cuts a text into a list of pieces at every separator.\n\nThe separator is plain text, not a pattern, so `\".\"` really splits at dots. Two separators side by side give an empty piece `\"\"` between them. Without a separator, the text is cut at commas."
        }
        "code.help.string.split.s" => "The text to cut.",
        "code.help.string.split.separator" => {
            "The text that marks where to cut. Defaults to `\",\"`."
        }
        "code.help.string.split.returns" => "A list of the pieces, in order.",
        "code.help.string.sub" => {
            "Cuts out the part of a text between two positions, both included.\n\nNegative positions count from the end, so `name:sub(-3)` is the last three bytes. If the start is after the end, you get an empty text."
        }
        "code.help.string.sub.s" => "The text to cut from.",
        "code.help.string.sub.i" => "The first position to keep.",
        "code.help.string.sub.j" => "The last position to keep. Defaults to the end of the text.",
        "code.help.string.sub.returns" => "The piece of text.",
        "code.help.string.unpack" => {
            "Reads values back out of a binary text made by [string.pack].\n\nUse the same format that packed it. After the values you also get the position just after the last byte read, useful to keep reading from there. Too little data for the format is an error."
        }
        "code.help.string.unpack.format" => "The list of codes, as given to [string.pack].",
        "code.help.string.unpack.data" => "The packed bytes.",
        "code.help.string.unpack.pos" => "Where to start reading. Defaults to 1.",
        "code.help.string.unpack.returns" => "Each value read, then the next position to read.",
        "code.help.string.upper" => {
            "Turns small letters into capital letters.\n\nOnly the plain English letters a to z change; accented letters such as `é` stay as they are."
        }
        "code.help.string.upper.s" => "The text to change.",
        "code.help.string.upper.returns" => "A new text in capital letters.",
        _ => return None,
    })
}
