//! A página `os` (assinaturas e exemplos; o texto em `ph2d-i18n/src/code_help/os.rs`).

use super::Entry;
#[allow(unused_imports)]
use crate::Kind::{Constant, Function};

pub(crate) const ENTRIES: &[Entry] = &[
    Entry {
        name: "clock",
        kind: Function,
        sigs: &["(): number"],
        params: &[],
        returns: true,
        example: r#"local start = os.clock()
local sum = 0
for i = 1, 1000 do sum += i end
local elapsed = os.clock() - start
print(sum, elapsed >= 0) --> 500500, true"#,
    },
    Entry {
        name: "date",
        kind: Function,
        sigs: &[
            "(formatString: string?, time: number?): string",
            "(formatString: \"!*t\" | \"*t\", time: number?): { day: number, hour: number, isdst: boolean, min: number, month: number, sec: number, wday: number, yday: number, year: number }",
        ],
        params: &["formatString", "time"],
        returns: true,
        example: r#"print(os.date("!%Y-%m-%d %H:%M", 0)) --> 1970-01-01 00:00
local d = os.date("!*t", 86400)
print(d.year, d.month, d.day, d.wday) --> 1970, 1, 2, 6"#,
    },
    Entry {
        name: "difftime",
        kind: Function,
        sigs: &["(t2: DateTypeResult | number, t1: DateTypeResult | number): number"],
        params: &["t2", "t1"],
        returns: true,
        example: r#"local start = os.time({ year = 2024, month = 1, day = 1, hour = 10 })
local finish = os.time({ year = 2024, month = 1, day = 1, hour = 11, min = 30 })
print(os.difftime(finish, start)) --> 5400"#,
    },
    Entry {
        name: "time",
        kind: Function,
        sigs: &["(time: DateTypeArg?): number"],
        params: &["time"],
        returns: true,
        example: r#"local newYear = os.time({ year = 2024, month = 1, day = 1, hour = 0 })
print(newYear) --> 1704067200
print(os.time() > newYear) --> true"#,
    },
];
