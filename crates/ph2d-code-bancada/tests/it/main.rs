//! Um binário de teste para a crate (convenção do repo: teste novo = ficheiro aqui + linha `mod`).
//! A Bancada (plano 13 Onda B) sobre um `World` real: quem corre o script, as colunas do TEXTO com o
//! tipo das declarações, as células, a edição que um clique pede e o que a aba pede à cena.

mod acto;
mod quem;
mod seguir;

use ph2d_code_bancada::TextFacts;
use ph2d_code_buffer::Text;
use ph2d_code_syntax::Syntax;
use ph2d_ecs::{Entity, Name, StableId, World};
use ph2d_script::{LuauScript, PropDecl, PropHint, ScriptId, ScriptValue};

/// Sete propriedades no texto (linhas 0–6) e uma função (linha 7); a última, `novo`, só o texto a
/// tem (escrita e por salvar: a cena não a conhece).
pub const TEXTO: &str = r#"ph2d.property("speed", 220, { min = 0, max = 500 })
ph2d.property("active", true)
ph2d.property("mode", "walk", { options = { "walk", "run" } })
ph2d.property("aim", ph2d.vec2(0, 1))
ph2d.property("tint", ph2d.color(1, 0, 0))
ph2d.property("label", "bob")
ph2d.property("novo", 1)
function on_update(self, dt)
end
"#;

pub const SCRIPT: ScriptId = ScriptId(7);
pub const OUTRO: ScriptId = ScriptId(8);

fn decl(name: &str, default: ScriptValue, hint: PropHint) -> PropDecl {
    PropDecl {
        name: name.to_owned(),
        default,
        hint,
    }
}

/// As declarações que a cena conhece do [`TEXTO`]: todas menos `novo`.
pub fn decls() -> Vec<PropDecl> {
    let livre = PropHint::default;
    vec![
        decl(
            "speed",
            ScriptValue::Number(220.0),
            PropHint {
                min: Some(0.0),
                max: Some(500.0),
                ..PropHint::default()
            },
        ),
        decl("active", ScriptValue::Bool(true), livre()),
        decl(
            "mode",
            ScriptValue::Text("walk".into()),
            PropHint {
                options: vec!["walk".into(), "run".into()],
                ..PropHint::default()
            },
        ),
        decl("aim", ScriptValue::Vec2([0.0, 1.0]), livre()),
        decl("tint", ScriptValue::Color([1.0, 0.0, 0.0, 1.0]), livre()),
        decl("label", ScriptValue::Text("bob".into()), livre()),
    ]
}

/// Os factos do [`TEXTO`], pela porta que a Bancada usa.
pub fn factos() -> TextFacts {
    let text = Text::new(TEXTO);
    TextFacts::read(&Syntax::new(&text), &text)
}

/// Um objecto com a `StableId` `sid` a correr `id`, com nome (ou sem) e os valores próprios `own`.
pub fn objecto(
    w: &mut World,
    sid: u64,
    nome: Option<&str>,
    id: ScriptId,
    own: &[(&str, ScriptValue)],
) -> Entity {
    let mut s = LuauScript::library(id);
    for (k, v) in own {
        s.own.insert((*k).to_owned(), v.clone());
    }
    let mut e = w.spawn((StableId(sid), s));
    if let Some(n) = nome {
        e.insert(Name(n.to_owned()));
    }
    e.id()
}
