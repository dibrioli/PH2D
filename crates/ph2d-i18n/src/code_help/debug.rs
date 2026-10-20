//! O texto da página `debug` do editor de scripts (as assinaturas e os exemplos em
//! `ph2d-code-help/src/luau/debug.rs`).

pub(super) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "code.help.debug" => "Two tools for finding out where your code is and how it got there.",
        "code.help.debug.about" => {
            "Use them while hunting a problem: to print which function called which, or the line a message comes from. Line numbers are those of your script, and names are the script's name in the project."
        }
        "code.help.debug.info" => {
            "Answers questions about a running function, or about a function you hand it.\n\n`options` is a short text, one letter per question, and you get one answer per letter, in the same order: `s` the script name, `l` the line, `n` the function's name, `a` how many named arguments it takes and whether it also accepts `...` (two answers), `f` the function itself. Any other letter stops the script with an error.\n\nPass a level to ask about the call stack: 1 is the function you are in, 2 the one that called it, and so on. A level deeper than the stack gives nothing back."
        }
        "code.help.debug.info.level" => {
            "Which function on the call stack: 1 = this one, 2 = its caller, …"
        }
        "code.help.debug.info.options" => "The letters of the questions, such as `\"sln\"`.",
        "code.help.debug.info.thread" => "A coroutine whose own call stack you want to inspect.",
        "code.help.debug.info.func" => "A function to describe, instead of a level.",
        "code.help.debug.info.returns" => {
            "One answer per letter. A name it cannot find comes back as an empty text, and a line it cannot know as -1."
        }
        "code.help.debug.traceback" => {
            "Gives the list of calls that led to this point, as text: your message first, then one line per call with the script name and line number.\n\nPass it to [xpcall] as the error handler to see where an error came from. Print it while testing; its exact layout is not meant to be read by your code."
        }
        "code.help.debug.traceback.message" => {
            "Optional. A text to put on the first line. A number is turned into text; anything else stops the script with an error."
        }
        "code.help.debug.traceback.level" => {
            "Optional. Where to start the list: 1 (the default) is the function calling `traceback`, 2 skips it and starts at its caller."
        }
        "code.help.debug.traceback.thread" => {
            "A coroutine whose call stack you want instead of the current one."
        }
        "code.help.debug.traceback.returns" => "The text of the list, one call per line.",
        _ => return None,
    })
}
