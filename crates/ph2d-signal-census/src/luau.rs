//! Os sinais dos scripts Luau: `ph2d.emit("x")` emite, `ph2d.connect(obj, "x", …)` ouve, e o gancho
//! `on_signal` ouve QUALQUER nome. Um nome que não é literal liga a flag «dinâmico» da direcção.

use ph2d_ecs::World;
use ph2d_script::{LuauScript, ProjectScripts, ScriptSource};
use ph2d_signal_ends::{Role, SignalCensus};

/// Soma os scripts de cada objecto que corre um (um script em dois objectos conta duas vezes).
pub(crate) fn scripts(world: &World, out: &mut SignalCensus) {
    let Some(lib) = world.get_resource::<ProjectScripts>() else {
        return;
    };
    let Some(mut q) = world.try_query::<&LuauScript>() else {
        return;
    };
    for s in q.iter(world) {
        if let ScriptSource::Library(id) = &s.source
            && let Some(doc) = lib.0.get(*id)
        {
            source(&doc.text, out);
        }
    }
}

/// O que muda o censo dos scripts: a revisão da biblioteca e que script corre em cada objecto.
pub(crate) fn fingerprint(world: &World, put: &mut dyn FnMut(u64)) {
    let Some(lib) = world.get_resource::<ProjectScripts>() else {
        return;
    };
    put(lib.0.revision());
    if let Some(mut q) = world.try_query::<&LuauScript>() {
        for s in q.iter(world) {
            put(match &s.source {
                ScriptSource::Library(id) => id.0,
                _ => u64::MAX,
            });
        }
    }
}

/// O censo de um texto Luau.
pub fn source(text: &str, out: &mut SignalCensus) {
    for (i, _) in text.match_indices("ph2d.emit") {
        let rest = &text[i + "ph2d.emit".len()..];
        match first_arg_literal(rest, 0) {
            Some(Some(n)) => out.add(n, Role::Emit),
            Some(None) => out.dynamic_emit = true,
            None => {}
        }
    }
    for (i, _) in text.match_indices("ph2d.connect") {
        let rest = &text[i + "ph2d.connect".len()..];
        match first_arg_literal(rest, 1) {
            Some(Some(n)) => out.add(n, Role::Listen),
            Some(None) => out.dynamic_listen = true,
            None => {}
        }
    }
    if text
        .split(|c: char| !(c.is_alphanumeric() || c == '_'))
        .any(|w| w == "on_signal")
    {
        out.dynamic_listen = true;
    }
}

/// O argumento `skip` de uma chamada que começa em `rest` (`(` primeiro): `Some(Some(nome))` se é um
/// literal de string, `Some(None)` se é outra expressão, `None` se não é uma chamada.
fn first_arg_literal(rest: &str, skip: usize) -> Option<Option<&str>> {
    let rest = rest.trim_start().strip_prefix('(')?;
    let mut arg = rest;
    for _ in 0..skip {
        let mut depth = 0i32;
        let mut cut = None;
        for (j, ch) in arg.char_indices() {
            match ch {
                '(' | '{' | '[' => depth += 1,
                ')' | '}' | ']' => depth -= 1,
                ',' if depth == 0 => {
                    cut = Some(j);
                    break;
                }
                _ => {}
            }
            if depth < 0 {
                return Some(None);
            }
        }
        arg = &arg[cut? + 1..];
    }
    let arg = arg.trim_start();
    let q = arg.chars().next()?;
    if q != '"' && q != '\'' {
        return Some(None);
    }
    let body = &arg[1..];
    let end = body.find(q)?;
    let after = body[end + 1..].trim_start();
    if after.starts_with(',') || after.starts_with(')') {
        Some(Some(&body[..end]))
    } else {
        Some(None)
    }
}
