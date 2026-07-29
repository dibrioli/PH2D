//! **Logic** — conditionals over the value, another object, or the clock.

use crate::knob::Knob;
use crate::recipe::{ClockUse, Combine, Family, Neutrality, Recipe, RowKind};

pub const IF_GREATER: Recipe = Recipe {
    id: "if-greater",
    family: Family::Logic,
    label: "If Greater",
    blurb: "Pick one of two values depending on whether the value is above a mark.",
    aliases: &[
        "if",
        "greater than",
        "above",
        "over",
        "condition",
        "compare",
    ],
    knobs: &[
        Knob::num("threshold", "Threshold", 0.5, (-40.0, 40.0)),
        Knob::num("then", "Then", 1.0, (-40.0, 40.0)),
        Knob::num("else", "Else", 0.0, (-40.0, 40.0)),
    ],
    kind: RowKind::Value,
    combine: None,
    clock: ClockUse::None,
    neutral: Neutrality::NoNeutral,
    pair: None,
    emit: |c| format!("select({} > {}, {}, {})", c.tight(), c.n(0), c.n(1), c.n(2)),
};

pub const IF_LESS: Recipe = Recipe {
    id: "if-less",
    family: Family::Logic,
    label: "If Less",
    blurb: "Pick one of two values depending on whether the value is below a mark.",
    aliases: &["less than", "below", "under", "condition", "compare"],
    knobs: &[
        Knob::num("threshold", "Threshold", 0.5, (-40.0, 40.0)),
        Knob::num("then", "Then", 1.0, (-40.0, 40.0)),
        Knob::num("else", "Else", 0.0, (-40.0, 40.0)),
    ],
    kind: RowKind::Value,
    combine: None,
    clock: ClockUse::None,
    neutral: Neutrality::NoNeutral,
    pair: None,
    emit: |c| format!("select({} < {}, {}, {})", c.tight(), c.n(0), c.n(1), c.n(2)),
};

pub const IF_EQUAL: Recipe = Recipe {
    id: "if-equal",
    family: Family::Logic,
    label: "If Near",
    blurb: "Pick one of two values depending on whether the value comes close to a mark.",
    aliases: &[
        "equals",
        "is",
        "match",
        "same as",
        "near",
        "close to",
        "condition",
    ],
    knobs: &[
        Knob::num("threshold", "Near", 0.5, (-40.0, 40.0)),
        Knob::num("then", "Then", 1.0, (-40.0, 40.0)),
        Knob::num("else", "Else", 0.0, (-40.0, 40.0)),
        // ⚠️ **A BANDA, e é ela que faz a receita existir.** Isto emitia
        // `select(x == k, ..)` — igualdade EXATA de `f32` sobre um sinal contínuo,
        // que praticamente nunca dispara: medido no censo, era a única receita que
        // continuava parada mesmo sob um gerador. Um rótulo tem de prometer o que o
        // modelo entrega, então ela deixou de dizer "Equal" e ganhou a tolerância que
        // a torna verdadeira.
        //
        // ⚠️ O default saiu de uma MEDIÇÃO, não de gosto — fração do tempo em que um
        // `Sway` de 1 m passa dentro de ±tol de uma marca no meio da faixa:
        // `0,01 m → 1,7%` (um lampejo que lê como nada) · `0,05 → 7,5%` ·
        // **`0,10 → 13,3%`** (pulso inequívoco, e ainda significa *perto*) ·
        // `0,20 → 26,2%` · `0,40 → 61,2%` (a banda é a maior parte do tempo, e a
        // condição deixa de dizer coisa alguma).
        //
        // ⚠️ A comparação é `<` ESTRITO porque é o único que existe: o `ph2d-expr` é
        // CONGELADO (ADR-0039) e só tem `Lt`/`Gt`, então `<=` não desce — o gate
        // `every_recipe_emits_a_formula_the_one_parser_accepts` pegou isso na primeira
        // rodada, com o erro `unexpected char '='`, que é exatamente para o que ele
        // existe. Num sinal contínuo a fronteira tem medida zero, então estrito e
        // não-estrito são indistinguíveis na tela.
        //
        // Corolário que o `<` traz e que fica escrito porque é o que o CÓDIGO faz:
        // tolerância **0 = banda vazia**, logo a linha toma sempre o Else. É o
        // degenerado que o artista alcança digitando zero (uma faixa de knob é faixa de
        // ARRASTO, não afirmação de validade), e é visível — o readout mostra a
        // constante. Ainda assim é melhor que o `==` de antes, que também nunca
        // disparava e se vendia como feature.
        Knob::num("tolerance", "Tolerance", 0.1, (0.0, 4.0)),
    ],
    kind: RowKind::Value,
    combine: None,
    clock: ClockUse::None,
    neutral: Neutrality::NoNeutral,
    pair: None,
    emit: |c| {
        format!(
            "select(abs({} - {}) < {}, {}, {})",
            c.tight(),
            c.n(0),
            c.n(3),
            c.n(1),
            c.n(2)
        )
    },
};

pub const GATE_AND: Recipe = Recipe {
    id: "gate-and",
    family: Family::Logic,
    label: "Gate (Both)",
    blurb: "One value only while BOTH watched objects are past their marks.",
    aliases: &["and", "both", "all", "gate", "two conditions"],
    knobs: &[
        Knob::link("a", "Watch A"),
        Knob::num("a_min", "A Over", 0.5, (-40.0, 40.0)),
        Knob::link("b", "Watch B"),
        Knob::num("b_min", "B Over", 0.5, (-40.0, 40.0)),
        Knob::num("on", "On", 1.0, (-40.0, 40.0)),
        Knob::num("off", "Off", 0.0, (-40.0, 40.0)),
    ],
    kind: RowKind::Value,
    combine: Some(Combine::Replace),
    clock: ClockUse::None,
    neutral: Neutrality::NoNeutral,
    pair: None,
    emit: |c| {
        format!(
            "select(({} > {}) && ({} > {}), {}, {})",
            c.link(0),
            c.n(1),
            c.link(2),
            c.n(3),
            c.n(4),
            c.n(5)
        )
    },
};

pub const GATE_OR: Recipe = Recipe {
    id: "gate-or",
    family: Family::Logic,
    label: "Gate (Either)",
    blurb: "One value while EITHER watched object is past its mark.",
    aliases: &["or", "either", "any", "gate", "two conditions"],
    knobs: &[
        Knob::link("a", "Watch A"),
        Knob::num("a_min", "A Over", 0.5, (-40.0, 40.0)),
        Knob::link("b", "Watch B"),
        Knob::num("b_min", "B Over", 0.5, (-40.0, 40.0)),
        Knob::num("on", "On", 1.0, (-40.0, 40.0)),
        Knob::num("off", "Off", 0.0, (-40.0, 40.0)),
    ],
    kind: RowKind::Value,
    combine: Some(Combine::Replace),
    clock: ClockUse::None,
    neutral: Neutrality::NoNeutral,
    pair: None,
    emit: |c| {
        format!(
            "select(({} > {}) || ({} > {}), {}, {})",
            c.link(0),
            c.n(1),
            c.link(2),
            c.n(3),
            c.n(4),
            c.n(5)
        )
    },
};

pub const AFTER_TIME: Recipe = Recipe {
    id: "after-time",
    family: Family::Logic,
    label: "After Time",
    blurb: "Switch from one value to another at a moment.",
    aliases: &[
        "at time",
        "after",
        "cue",
        "when",
        "trigger time",
        "step time",
    ],
    knobs: &[
        Knob::num("at", "At (s)", 1.5, (0.0, 600.0)),
        Knob::num("before", "Before", 0.0, (-40.0, 40.0)),
        Knob::num("after", "After", 1.0, (-40.0, 40.0)),
    ],
    kind: RowKind::Value,
    combine: Some(Combine::Replace),
    clock: ClockUse::Explicit,
    neutral: Neutrality::NoNeutral,
    pair: None,
    emit: |c| format!("select({} > {}, {}, {})", c.clock, c.n(0), c.n(2), c.n(1)),
};
