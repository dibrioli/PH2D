//! **O RASTRO RE-COZIDO** — a cauda montada de um LEQUE DE TEMPO em vez de um
//! ring ([ADR-0163](../../../docs/architecture/decisions/0163-a-node-may-cook-its-own-input-at-n-instants-a-time-fan.md);
//! doc 89, folha 07, `SUPERAR:` S1).
//!
//! Um ring contém o passado porque passado é o que um ring é. Re-cozinhar a
//! entrada em `t ± k·spacing` desenha os dois lados — e é exacto sob scrub,
//! porque nada aqui é estado.
//!
//! Separado do `lib.rs` pelo tecto de LOC (HR-18), na costura que o próprio modo
//! desenha: acima fica o ring e a aritmética que os dois modos partilham; aqui, a
//! lei das gerações e o seu construtor de mapas.

use super::{
    AGE, Column, Decay, MANIFEST, MAX_LENGTH, NodeOp, Stream, concat, materialize_render_columns,
    spacing_of,
};

/// A chave do param **de onde o eco vem**.
pub const SOURCE: &str = "source";
/// A chave do param **quantos ecos vêm da frente**.
pub const FORWARD: &str = "forward";
/// O valor de [`SOURCE`] que pede a entrada RE-COZIDA em vez do ring.
pub const SOURCE_RESAMPLED: f32 = 1.0;

/// **AS GERAÇÕES, EM ORDEM DE DESENHO** — o deslocamento de cada linha do rastro,
/// em TICKS, com sinal: **negativo = passado**, **positivo = futuro**, `0` = a
/// cabeça viva.
///
/// ⚠️ **Esta função é a ÚNICA fonte da lei, e tem dois leitores** — o
/// [`time_fans`], que a converte nos mapas de tempo que o cook aplica, e o
/// `eval`, que tira dela a IDADE de cada geração (`|g|`) para desbotar. Escrever
/// a mesma escada nos dois sítios seria a receita de a alça sair de baixo do
/// dedo: o desenho pousaria num instante e a cor viria de outro.
///
/// A ordem é **cauda primeiro, cabeça por último** (a linha viva pinta sobre os
/// próprios ecos), com as idades a descer e o passado à frente do futuro num
/// empate — arbitrário, fixo e escrito.
///
/// ⚠️ **`forward = 0` devolve exactamente `[-(L−1)s, …, −s, 0]`**, que é a cauda
/// que o ring produz. É essa redução que faz o modo novo nascer no ponto neutro.
#[must_use]
pub fn echo_offsets(length: usize, spacing: usize, forward: usize) -> Vec<i32> {
    if length == 0 {
        return Vec::new();
    }
    let s = spacing.max(1) as i32;
    let echoes = length - 1;
    let f = forward.min(echoes);
    let back = echoes - f;
    let mut out: Vec<i32> = Vec::with_capacity(length);
    for k in (1..=back).rev() {
        out.push(-(k as i32) * s);
    }
    for k in (1..=f).rev() {
        out.push(k as i32 * s);
    }
    // ⚠️ Ordena por IDADE decrescente e mantém o passado antes do futuro num
    // empate — `sort_by_key` é estável, então a regra do empate é a ordem em que
    // os dois laços acima empurraram.
    out.sort_by_key(|g| std::cmp::Reverse(g.unsigned_abs()));
    out.push(0);
    out
}

/// Quantos ecos vêm da frente, clampado ao que a cauda tem para dar.
///
/// ⚠️ Um `forward` maior que os ecos que existem **não é erro nem teto novo**: ele
/// simplesmente vira "todos à frente", que é o que o número pede. Clampar aqui é
/// o que faz o slider e o campo digitável concordarem sem um `ParamHardMax` a
/// mentir sobre o que o nó honra.
pub(super) fn forward_of(forward: f32, length: usize) -> usize {
    if !forward.is_finite() || forward < 0.5 {
        return 0;
    }
    (forward.round() as usize).min(length.saturating_sub(1))
}

/// **O vão dos alvos autorados**, em ticks: a idade que o eco mais VELHO
/// alcançaria se todos estivessem atrás.
///
/// ⚠️ Ele NÃO é o maior `|g|` do leque, e isso é produto: com `forward` a meio, o
/// eco mais distante fica a metade do caminho, e um vão derivado dele faria o
/// `Fade` significar *"o alvo é atingido no eco mais distante que houver"* —
/// o número mudaria de sentido ao arrastar o Forward. Fixo em `(L−1)·s`, o
/// `Fade` é sempre *"o que um eco a `L−1` passos de distância parece"*, de que
/// lado for.
pub(super) fn authored_span(length: usize, spacing: usize) -> u32 {
    u32::try_from(length.saturating_sub(1) * spacing.max(1)).unwrap_or(u32::MAX)
}

/// **O RASTRO RE-COZIDO** — a cauda montada do leque em vez do ring.
///
/// Cada geração é a sub-árvore de entrada cozida no seu instante, envelhecida de
/// uma vez por [`Decay::at_age`] em vez de uma vez por tick. Nada aqui é estado,
/// e é isso que dá as quatro coisas que um ring não dá: o eco para a FRENTE, o
/// `length` sem tecto de memória, o espaçamento não-uniforme, e o scrub exacto.
pub(super) fn step_resampled(
    ctx_fan: &[&Stream],
    offsets: &[i32],
    decay: Decay,
    span: u32,
) -> Stream {
    let mut out: Option<Stream> = None;
    for (k, &g) in offsets.iter().enumerate() {
        let Some(src) = ctx_fan.get(k) else { continue };
        if src.count() == 0 {
            continue;
        }
        let mut row = (*src).clone();
        let age = g.unsigned_abs();
        row.set(AGE, Column::Scalar(vec![age as f32; row.count()]));
        materialize_render_columns(&mut row);
        if age > 0 {
            decay.at_age(span, age).apply(&mut row);
        }
        out = Some(match out {
            None => row,
            Some(acc) => concat(&acc, &row),
        });
    }
    out.unwrap_or_else(Stream::empty)
}

/// **OS LEQUES DE TEMPO desta família** — o que a camada de domínio pousa no cook
/// para um rastro `Resampled` (`ph2d_nodegraph::cook::TimeFans`).
///
/// Espelho exacto do `ph2d_node_motion_time_remap::time_scopes`: o substrato
/// chaveia por `NodeId` e não conhece tipo nenhum, então quem sabe o que
/// `motion.trail` é somos nós. Um grafo sem rastro re-cozido devolve o mapa
/// vazio, e o cook toma o caminho de sempre.
///
/// ⚠️ **`tick_seconds` é a duração de um tique do relógio EXTERNO**, e ela não
/// sai do grafo — é do shell, que é quem faz o playhead andar. O `spacing` deste
/// nó sempre contou TIQUES (o ring envelhecia um por cozedura), e é essa
/// conversão que mantém o número do slider a significar a mesma coisa nos dois
/// modos.
///
/// ⚠️ **A geração `0` é a IDENTIDADE**, e de propósito: um mapa identidade
/// partilha a faixa de memo não-escopada, então a cabeça viva de um rastro
/// re-cozido custa **zero** — é a mesma cozedura que o resto do grafo já pediu.
#[must_use]
pub fn time_fans(
    graph: &ph2d_nodegraph::graph::Graph,
    ops: &dyn ph2d_nodegraph::cook::OpResolver,
    tick_seconds: f64,
) -> ph2d_nodegraph::cook::TimeFans {
    use ph2d_nodegraph::time::{TimeMap, TimeMode};
    let mut fans = ph2d_nodegraph::cook::TimeFans::new();
    for inst in graph.nodes() {
        if inst.type_name != MANIFEST.name {
            continue;
        }
        let Some(manifest) = ops.resolve(inst.type_id()).map(NodeOp::manifest) else {
            continue;
        };
        let overrides = graph.node_param_overrides(inst.id);
        let p = |name: &str| {
            overrides
                .and_then(|o| o.get(name).copied())
                .or_else(|| manifest.param_default(name))
                .unwrap_or(0.0)
        };
        if p(SOURCE) < 0.5 {
            continue;
        }
        // ⚠️ O `k` aqui NÃO passa pelo tecto de instâncias (o `generations` do
        // `eval` conhece a contagem viva e este não): o leque é montado antes de
        // haver stream nenhuma. Um leque maior que a cauda que o `eval` monta
        // custa cozeduras a mais e desenha o mesmo; um MENOR truncaria a cauda —
        // e é por isso que o erro admissível é para cima.
        let k = (p("length").round().max(1.0) as usize).min(MAX_LENGTH);
        let s = spacing_of(p("spacing"));
        let maps: Vec<TimeMap> = echo_offsets(k, s, forward_of(p(FORWARD), k))
            .into_iter()
            .map(|g| TimeMap {
                mode: TimeMode::Scale,
                scale: 1.0,
                offset: f64::from(g) * tick_seconds,
                ..TimeMap::default()
            })
            .collect();
        if !maps.is_empty() {
            fans.insert(inst.id, maps);
        }
    }
    fans
}
