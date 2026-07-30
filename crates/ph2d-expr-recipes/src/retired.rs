//! **As receitas APOSENTADAS** — o que saiu do catálogo, por que, e onde a capacidade
//! foi parar (FASE A do plano 12).
//!
//! O Enio reprovou o catálogo com *"não vejo o menor sentido para artistas na seção
//! logic"* e a ordem de **eliminar as expressões similares umas às outras**. A auditoria
//! de 2026-07-29 mediu a matriz de redundância (doc 13 §3) e o corte saiu dela: **50 → 31**.
//!
//! ⚠️ **O tamanho NUNCA foi o defeito, e a meta "~21" do plano foi abandonada** — a
//! [Cavalry shipa 40+ Behaviours](https://docs.cavalry.scenegroup.co/nodes/behaviours/) e o
//! [Blender ship 9 F-Modifiers](https://docs.blender.org/manual/en/3.3/editors/graph_editor/fcurves/modifiers.html)
//! empilháveis; o que o artista encontrou primeiro foi RUÍDO. Então o critério é
//! **inerte ou programação**, e 31 é o RESTO da regra, não uma cota (plano 12 §0, D-0.2).
//!
//! ## Duas razões para cortar, e cada uma tem a sua resposta
//!
//! | razão | o que acontece com a palavra do artista |
//! |---|---|
//! | **REDUNDANTE** (medido: existe ajuste do sobrevivente que a reproduz) | o sobrevivente **herda o rótulo e os apelidos** — a capacidade continua lá |
//! | **PROGRAMAÇÃO / COMPOSIÇÃO** (não há sobrevivente) | uma **recusa com roteamento** (`REFUSALS`) diz onde aquilo mora |
//!
//! ⚠️ **Cortar sem herdar é ESCONDER capacidade**, e a auditoria mediu isso acontecendo:
//! das 5 receitas cortadas na jornada anterior, os SINÔNIMOS foram herdados (`"sawtooth"`
//! → Pulse ✓) e os **RÓTULOS não** — `"ramp"`, `"ramp loop"` e `"sway cosine"` davam
//! **zero hits** (doc 13 §7.3). É por isso que esta tabela existe em vez de uma nota: ela
//! é o que o gate [`crate::retired`] varre, e um corte novo sem entrada aqui nasce
//! VERMELHO.

use crate::recipe::RecipeId;

/// Para onde a palavra de uma receita aposentada leva.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Answer {
    /// Outra receita faz o mesmo — medido, com o delta ao lado no `why`. Ela herdou o
    /// rótulo e os apelidos desta.
    Survivor(RecipeId),
    /// Não há sobrevivente: a ideia mora fora do catálogo (ou é a PILHA). A chave é a de
    /// um [`crate::refusal::Refusal`], que a busca já responde.
    Refusal(&'static str),
}

/// Uma receita que saiu, e o que a palavra dela faz agora.
#[derive(Clone, Copy, Debug)]
pub struct Retired {
    /// O id que ela tinha (é o que o `git log` guarda, e o que um save antigo pode citar).
    pub id: RecipeId,
    /// O RÓTULO que o artista lia na galeria — a palavra que ele vai digitar.
    pub label: &'static str,
    /// Para onde a palavra leva.
    pub answer: Answer,
    /// **A medição**, não a opinião.
    pub why: &'static str,
}

/// Tudo que saiu, com o número que justificou.
///
/// ⚠️ **Inclui as cinco da jornada ANTERIOR**, que viviam numa const local dentro de um
/// gate. Duas listas do mesmo fato é a falha que esta sessão passou o dia consertando: o
/// gate agora LÊ esta tabela, então cortar uma receita e esquecer a linha aqui é vermelho.
pub const RETIRED: &[Retired] = &[
    // ── A jornada anterior (5 duplicatas MEDIDAS) ───────────────────────────────
    Retired {
        id: "sway-cosine",
        label: "Sway (Cosine)",
        answer: Answer::Survivor("sway"),
        why: "= Sway com Phase em um quarto de período.",
    },
    Retired {
        id: "ramp-loop",
        label: "Ramp Loop",
        answer: Answer::Survivor("pulse"),
        why: "= Pulse com Decay 1 e On/Off trocados.",
    },
    Retired {
        id: "mirror",
        label: "Mirror",
        answer: Answer::Survivor("follow"),
        why: "= Opposite com Pivot 0 — e ⚠️ o `opposite` foi aposentado DEPOIS (FASE A), \
              então o alvo aqui é o sobrevivente VIVO, não o intermediário. Uma \
              aposentadoria em cadeia não resolvida mandaria o artista a um card que não \
              existe (gate `every_survivor_is_a_recipe_that_still_exists`).",
    },
    Retired {
        id: "midpoint",
        label: "Midpoint",
        answer: Answer::Survivor("blend-two"),
        why: "= Blend Two em 0.5.",
    },
    Retired {
        id: "negate",
        label: "Negate",
        answer: Answer::Survivor("multiply-add"),
        why: "= Multiply/Add com −1.",
    },
    // ── Redundantes: o sobrevivente herdou rótulo + apelidos ────────────────────
    Retired {
        id: "turbulence",
        label: "Turbulence",
        answer: Answer::Survivor("shake"),
        why: "`turbulence ~> shake` delta 0,000000 — turbulence É shake com octaves. O \
              Shake ABSORVEU os knobs Detail/Roughness, e com Detail = 1 o `wiggle` de 4 \
              argumentos é byte-idêntico ao de 2 (contrato do parser), então o default do \
              Shake não se moveu.",
    },
    Retired {
        id: "opposite",
        label: "Opposite",
        answer: Answer::Survivor("follow"),
        why: "`follow ~> opposite` delta 0,000000 — opposite é follow com multiplicador −1. \
              Não previsto pelo plano.",
    },
    Retired {
        id: "offset-copy",
        label: "Offset Copy",
        answer: Answer::Survivor("follow"),
        why: "`offset-copy ~> follow` delta 0,000000 — e ⚠️ **eu li isso errado na \
              auditoria**: escrevi *\"a espinha do Link não é o follow\"* como se \
              offset-copy fosse a mais geral. Não é. A matriz compara contra o DEFAULT de \
              B, e o default do Follow (`link*1 + 0`) é reproduzível por offset-copy com \
              Offset 0 — mas o Follow **já tem os dois knobs** (Multiply E Offset), então \
              ele contém offset-copy no espaço INTEIRO. Nada foi absorvido: só os apelidos.",
    },
    Retired {
        id: "floor-at",
        label: "Floor At",
        answer: Answer::Survivor("limit"),
        why: "`limit ~> floor-at` delta 0,000000 — Limit é o mesmo com os dois lados.",
    },
    Retired {
        id: "ceiling-at",
        label: "Ceiling At",
        answer: Answer::Survivor("limit"),
        why: "`limit ~> ceiling-at` delta 0,000000 — idem.",
    },
    Retired {
        id: "remap-clamped",
        label: "Remap (Clamped)",
        answer: Answer::Survivor("limit"),
        why: "`limit ~> remap-clamped` E `remap-clamped ~> limit`, delta 0,000000 — MÚTUA, \
              são a mesma receita. ⚠️ E é por isso que NÃO ganhamos um checkbox `Clamp` no \
              Remap: um remap clampado é **Remap + Limit**, duas linhas, e a PILHA é \
              exatamente o modelo que já temos para composição.",
    },
    Retired {
        id: "invert-range",
        label: "Invert in Range",
        answer: Answer::Survivor("multiply-add"),
        why: "`remap ~> invert-range` (1e-7) E `multiply-add ~> invert-range` (0,000000) — \
              subsumida por DUAS receitas.",
    },
    Retired {
        id: "reverse-time",
        label: "Reverse Time",
        answer: Answer::Survivor("speed"),
        why: "`speed ~> reverse-time` delta 0,000000, em Speed = −1. ⚠️ A grade uniforme do \
              plano NÃO achava isto: 11 passos sobre (−10, 10) pulam o −1 (doc 13 §3).",
    },
    Retired {
        id: "free-fall",
        label: "Free Fall",
        answer: Answer::Survivor("throw"),
        why: "`throw ~> free-fall` delta 0,000000 — free-fall É throw com velocidade 0.",
    },
    // ── Programação: a família Logic inteira + o Switch disfarçado de Link ───────
    Retired {
        id: "if-greater",
        label: "If Greater",
        answer: Answer::Refusal("condition"),
        why: "D3 — *\"não vejo o menor sentido para artistas na seção logic\"* (Enio). E a \
              medição concorda: `if-greater ~> if-less` é MÚTUA e `gate-and`/`gate-or ~> \
              switch`, então 5 das 6 colapsam em 2 formas — a família era redundante \
              consigo mesma antes de ser programação.",
    },
    Retired {
        id: "if-less",
        label: "If Less",
        answer: Answer::Refusal("condition"),
        why: "MÚTUA com `if-greater` (delta 0,000000): a mesma receita com as saídas \
              trocadas.",
    },
    Retired {
        id: "if-equal",
        label: "If Near",
        answer: Answer::Refusal("condition"),
        why: "D3 — um limiar numérico com tolerância é programação, não animação.",
    },
    Retired {
        id: "gate-and",
        label: "Gate (Both)",
        answer: Answer::Refusal("condition"),
        why: "`gate-and ~> switch` delta 0,000000.",
    },
    Retired {
        id: "gate-or",
        label: "Gate (Either)",
        answer: Answer::Refusal("condition"),
        why: "`gate-or ~> switch` delta 0,000000.",
    },
    Retired {
        id: "after-time",
        label: "After Time",
        answer: Answer::Refusal("condition"),
        why: "D3, e o caso de uso legítimo — *\"acontece a partir de tal segundo\"* — é um \
              KEYFRAME, e este app tem uma timeline.",
    },
    Retired {
        id: "switch",
        label: "Switch",
        answer: Answer::Refusal("condition"),
        why: "Estava em Link e é Logic disfarçado (escolhe entre dois valores por um \
              limiar). `gate-and`/`gate-or ~> switch` mede a mesma coisa por outro lado.",
    },
    // ── Composições: a família Field é Remap(link), que a PILHA já expressa ──────
    Retired {
        id: "fade-by-distance",
        label: "Fade by Distance",
        answer: Answer::Refusal("compose"),
        why: "`fade-by-distance ~> distance-2d` (6e-8) — é Remap(Distance), duas linhas. E \
              ela sozinha comia **9 dos 12 slots** do sheet (doc 13 §4-bis).",
    },
    Retired {
        id: "scale-by-proximity",
        label: "Scale by Proximity",
        answer: Answer::Refusal("compose"),
        why: "`scale-by-proximity ~> distance-1d` (7e-7) — idem, sobre um eixo.",
    },
    Retired {
        id: "gradient-by-value",
        label: "Driven by Another",
        answer: Answer::Refusal("compose"),
        why: "`gradient-by-value ~> follow` / `~> opposite` (1e-7) — é Remap(Follow).",
    },
];

/// A receita aposentada com este id, se houver.
#[must_use]
pub fn retired_by_id(id: RecipeId) -> Option<&'static Retired> {
    RETIRED.iter().find(|r| r.id == id)
}
