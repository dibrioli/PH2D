//! O texto da página `os` do editor de scripts (as assinaturas e os exemplos em
//! `ph2d-code-help/src/luau/os.rs`).

pub(super) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "code.help.os" => "Clocks and calendar dates: measuring time and writing dates as text.",
        "code.help.os.about" => {
            "Times are counted in seconds since 1 January 1970 at midnight, UTC. This library only reads clocks: it cannot open files or run programs.\n\nNote: for anything that happens inside the scene, such as timers and animations, count the `dt` your `update` receives. The computer's clock keeps running when the scene is paused."
        }
        "code.help.os.clock" => {
            "A precise timer, in seconds, for measuring how long something takes.\n\nThe number on its own means nothing: take one reading before and one after, and subtract. It has fractions of a second, unlike [os.time]."
        }
        "code.help.os.clock.returns" => "Seconds, with a fractional part.",
        "code.help.os.date" => {
            "Writes a date and time as text, or splits it into its parts.\n\nThe format uses `%` codes: `%Y` year, `%m` month, `%d` day, `%H` hour, `%M` minutes, `%S` seconds, `%A` weekday name, `%B` month name. An unknown code stops the script with an error. Use `*t` to get a table of parts instead of text (`year`, `month`, `day`, `hour`, `min`, `sec`, `wday` with 1 = Sunday, `yday`, `isdst`).\n\nNote: without a leading `!` the answer is in the computer's own time zone, so it differs from one machine to another. Start the format with `!` (`\"!%H:%M\"`, `\"!*t\"`) to get UTC."
        }
        "code.help.os.date.formatString" => {
            "Optional. The format, as described above; a full date and time in the local time zone when left out."
        }
        "code.help.os.date.time" => {
            "Optional. The moment to describe, in seconds like [os.time] gives; now when left out."
        }
        "code.help.os.date.returns" => "The text, or a table of parts with `*t`.",
        "code.help.os.difftime" => {
            "How many seconds passed from `t1` to `t2`: simply `t2 - t1`.\n\nNote: in PH2D it only accepts numbers (a date table stops the script with an error; convert it with [os.time] first), and it drops fractions of a second, so `os.difftime(1.5, 1)` gives 0."
        }
        "code.help.os.difftime.t2" => "The later moment, in seconds.",
        "code.help.os.difftime.t1" => "The earlier moment, in seconds.",
        "code.help.os.difftime.returns" => {
            "The difference in seconds; negative if `t2` comes first."
        }
        "code.help.os.time" => {
            "The current date and time as a number of whole seconds, or the number for a date you describe.\n\nA date table needs `year`, `month` and `day`; `hour` is noon when left out, `min` and `sec` are 0. The table is read as UTC. Values out of range roll over: day 32 of January is 1 February.\n\nNote: dates before 1970 give `nil`."
        }
        "code.help.os.time.time" => {
            "Optional. A table with `year`, `month`, `day` and, if you like, `hour`, `min`, `sec`. Leave it out for now."
        }
        "code.help.os.time.returns" => "Seconds since 1 January 1970 UTC.",
        _ => return None,
    })
}
