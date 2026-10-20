//! O texto da página `coroutine` do editor de scripts (as assinaturas e os exemplos em
//! `ph2d-code-help/src/luau/coroutine.rs`).

pub(super) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "code.help.coroutine" => {
            "Functions that can pause halfway and carry on later from the same spot."
        }
        "code.help.coroutine.about" => {
            "A coroutine is a function you can pause with [coroutine.yield] and continue with [coroutine.resume]; its local variables are kept while it waits. That makes step-by-step behaviour easy to write: a patrol, a dialogue, a sequence of moves.\n\nNote: PH2D's hooks such as `update` are not coroutines, so pausing directly inside them stops the script with an error. Create your own coroutine, keep it in a variable, and resume it from `update` whenever the next step is due. [ph2d.wait] pauses a coroutine in the same way, handing the number of seconds to whoever resumes it; nothing resumes it for you."
        }
        "code.help.coroutine.close" => {
            "Shuts down a paused or finished coroutine for good; its status becomes `dead`.\n\nUse it to cancel a sequence you no longer need. It cannot close the coroutine you are running in."
        }
        "code.help.coroutine.close.co" => "The coroutine to close.",
        "code.help.coroutine.close.returns" => {
            "`true`, or `false` and the error if the coroutine had stopped because of one."
        }
        "code.help.coroutine.create" => {
            "Makes a new coroutine from a function, without starting it.\n\nIt starts running at the first [coroutine.resume]; the arguments of that call become the function's arguments."
        }
        "code.help.coroutine.create.f" => "The function the coroutine will run.",
        "code.help.coroutine.create.returns" => "The new coroutine, with status `suspended`.",
        "code.help.coroutine.isyieldable" => {
            "Tells whether the code running now is allowed to pause.\n\nIt is `true` inside a coroutine and `false` in a hook such as `update` or at the top of the script."
        }
        "code.help.coroutine.isyieldable.returns" => {
            "`true` when [coroutine.yield] would work here."
        }
        "code.help.coroutine.resume" => {
            "Starts a coroutine, or continues it from where it paused.\n\nIt runs until the coroutine pauses again or finishes. Values you pass become the results of the [coroutine.yield] it was paused on (or the function's arguments on the first run). Errors inside do not stop your script: they come back as `false` and the message. Resuming a finished coroutine gives `false, cannot resume dead coroutine`."
        }
        "code.help.coroutine.resume.co" => "The coroutine to run.",
        "code.help.coroutine.resume.vararg" => "Values to hand to the coroutine.",
        "code.help.coroutine.resume.returns" => {
            "`true` and the values it yielded or returned, or `false` and the error."
        }
        "code.help.coroutine.running" => {
            "Gives the coroutine that is running the current code.\n\nNote: outside any coroutine (in a hook or at the top of the script) it gives `nil` in PH2D."
        }
        "code.help.coroutine.running.returns" => "The running coroutine, or `nil` outside one.",
        "code.help.coroutine.status" => {
            "Tells what state a coroutine is in, as text.\n\n`suspended`: paused, or not started yet. `running`: it is the one asking. `normal`: it is waiting for another coroutine it resumed. `dead`: finished, failed or closed."
        }
        "code.help.coroutine.status.co" => "The coroutine to check.",
        "code.help.coroutine.status.returns" => {
            "`\"suspended\"`, `\"running\"`, `\"normal\"` or `\"dead\"`."
        }
        "code.help.coroutine.wrap" => {
            "Makes a coroutine that you run simply by calling it, like a function.\n\nEach call continues it and gives back what it yields. Unlike [coroutine.resume], there is no `true`/`false` first: an error inside, or calling it after it has finished, stops your script."
        }
        "code.help.coroutine.wrap.f" => "The function the coroutine will run.",
        "code.help.coroutine.wrap.returns" => {
            "A function that continues the coroutine each time it is called."
        }
        "code.help.coroutine.yield" => {
            "Pauses the coroutine you are in and hands values back to whoever resumed it.\n\nThe coroutine stays paused until the next [coroutine.resume], whose extra values become the results of `yield`. Outside a coroutine it stops the script with an error."
        }
        "code.help.coroutine.yield.vararg" => "Values to hand back to [coroutine.resume].",
        "code.help.coroutine.yield.returns" => "The values passed to the next [coroutine.resume].",
        _ => return None,
    })
}
