//! O texto da página `utf8` do editor de scripts (as assinaturas e os exemplos em
//! `ph2d-code-help/src/luau/utf8.rs`).

pub(super) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "code.help.utf8" => {
            "Text letter by letter, for letters beyond plain English such as accents and emoji."
        }
        "code.help.utf8.about" => {
            "Text is stored as bytes. Plain English letters take one byte each, but `é` takes two and an emoji four, so `#text` and the `string` functions count bytes, not letters. The functions here understand those multi-byte letters.\n\nPositions here are still byte positions, starting at 1. A letter's number code (its code point) is the same everywhere: `é` is 233."
        }
        "code.help.utf8.char" => {
            "Builds text from letter codes, one letter per number.\n\nUnlike [string.char], it accepts any letter code, so accents and emoji work. A negative number or one above 1114111 is an error."
        }
        "code.help.utf8.char.vararg" => "Any number of letter codes.",
        "code.help.utf8.char.returns" => "The text made of those letters.",
        "code.help.utf8.charpattern" => {
            "A pattern that matches exactly one letter, however many bytes it takes.\n\nUse it with [string.gmatch] to walk through the letters of a text. Printing it shows odd symbols; it is meant for searching, not reading."
        }
        "code.help.utf8.codepoint" => {
            "Gives the letter code of the letters between two byte positions.\n\nNote: the start position must be the first byte of a letter. Pointing into the middle of a letter such as `é`, or at bytes that are not valid text, raises an error."
        }
        "code.help.utf8.codepoint.str" => "The text to read.",
        "code.help.utf8.codepoint.i" => "The byte position of the first letter. Defaults to 1.",
        "code.help.utf8.codepoint.j" => {
            "The byte position of the last letter. Defaults to `i`, so only one letter is read."
        }
        "code.help.utf8.codepoint.returns" => "One number for each letter read.",
        "code.help.utf8.codes" => {
            "Walks through the letters of a text in a `for` loop.\n\nEach turn gives the byte position where the letter starts and its letter code. Turn the code back into text with [utf8.char]. Bytes that are not valid text raise an error during the loop."
        }
        "code.help.utf8.codes.str" => "The text to walk through.",
        "code.help.utf8.codes.returns" => {
            "What a `for` loop needs; each turn gives a byte position and a letter code."
        }
        "code.help.utf8.len" => {
            "Counts the letters in a text, where `#` would count bytes.\n\nIf the text holds bytes that are not valid letters, it returns `nil` and the position of the first bad byte. Starting in the middle of a letter counts as bad too."
        }
        "code.help.utf8.len.s" => "The text to count.",
        "code.help.utf8.len.i" => "The byte position to start counting from. Defaults to 1.",
        "code.help.utf8.len.j" => "The byte position to stop at. Defaults to the end.",
        "code.help.utf8.len.returns" => {
            "The number of letters; or `nil` and the position of the first bad byte."
        }
        "code.help.utf8.offset" => {
            "Finds the byte position where a given letter starts.\n\nUse it with [string.sub] to cut text by letters instead of bytes. A negative `n` counts from the end. It returns `nil` when the text has fewer letters; asking for one more than the count gives the position just after the end."
        }
        "code.help.utf8.offset.s" => "The text to look in.",
        "code.help.utf8.offset.n" => {
            "Which letter: 1 is the first, -1 the last. With 0 you get the start of the letter that contains byte `i`."
        }
        "code.help.utf8.offset.i" => {
            "The byte position to count from. Defaults to 1, or to the end when `n` is negative."
        }
        "code.help.utf8.offset.returns" => "The byte position where that letter starts, or `nil`.",
        _ => return None,
    })
}
