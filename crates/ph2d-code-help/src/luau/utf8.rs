//! A página `utf8` (assinaturas e exemplos; o texto em `ph2d-i18n/src/code_help/utf8.rs`).

use super::Entry;
#[allow(unused_imports)]
use crate::Kind::{Constant, Function};

pub(crate) const ENTRIES: &[Entry] = &[
    Entry {
        name: "char",
        kind: Function,
        sigs: &["(...: number): string"],
        params: &["..."],
        returns: true,
        example: r#"print(utf8.char(72, 233, 108, 108, 111)) --> Héllo"#,
    },
    Entry {
        name: "charpattern",
        kind: Constant,
        sigs: &["string"],
        params: &[],
        returns: false,
        example: r#"local count = 0
for letter in string.gmatch("héllo", utf8.charpattern) do count += 1 end
print(count, #"héllo") --> 5, 6"#,
    },
    Entry {
        name: "codepoint",
        kind: Function,
        sigs: &["(str: string, i: number?, j: number?): ...number"],
        params: &["str", "i", "j"],
        returns: true,
        example: r#"print(utf8.codepoint("é")) --> 233
print(utf8.codepoint("héllo", 1, -1)) --> 104, 233, 108, 108, 111"#,
    },
    Entry {
        name: "codes",
        kind: Function,
        sigs: &["(str: string): ((string, number) -> (number, number), string, number)"],
        params: &["str"],
        returns: true,
        example: r#"for pos, code in utf8.codes("hé!") do
	print(pos, code)
end
--> 1, 104
--> 2, 233
--> 4, 33"#,
    },
    Entry {
        name: "len",
        kind: Function,
        sigs: &["(s: string, i: number?, j: number?): (number?, number?)"],
        params: &["s", "i", "j"],
        returns: true,
        example: r#"local name = "Zoë"
print(#name, utf8.len(name)) --> 4, 3
print(utf8.len("ab\255")) --> nil, 3"#,
    },
    Entry {
        name: "offset",
        kind: Function,
        sigs: &["(s: string, n: number?, i: number?): number"],
        params: &["s", "n", "i"],
        returns: true,
        example: r#"local word = "héllo"
print(utf8.offset(word, 3)) --> 4
print(utf8.offset(word, -1)) --> 6
print(utf8.offset(word, 10)) --> nil"#,
    },
];
