//! Channel-aware presets of the params bridge (a `#[path]` child of
//! `motion_bridge_params.rs`, split off for the 600-LOC shell file cap). "What a
//! behaviour's magnitude MEANS on the channel it drives" — the reset presets and the
//! Rotation option they hinge on. The widget-range WIDENING (`contain` /
//! `channel_range_override`) stays with `build_params_snapshot`, its only caller.

use crate::motion_state::MotionState;
use ph2d_node_registry::ParamUnit;

/// Reset a behaviour node's magnitude params to a sensible default for the newly
/// selected channel (#10 consistency). Switching what a stagger/oscillator drives
/// — X/Y position (world units) vs Rotation (degrees) vs Size (scale delta) —
/// rewrites the range so a `±1` meant for position doesn't read as a barely-there
/// ±1° / ±huge-scale on the other channels. Editor UX (not node math): it
/// runs on the channel switch inside the same undo step, so Ctrl+Z restores the
/// artist's previous values. Non-behaviour node types are a no-op.
pub(in crate::render_loop::motion_bridge) fn apply_channel_presets(
    motion: &mut MotionState,
    nid: ph2d_nodegraph::graph::NodeId,
    type_name: &str,
    channel: f32,
) {
    let ch = channel.round() as i32;
    match type_name {
        "motion.stagger" => {
            let (min, max) = stagger_channel_range(ch);
            motion.doc.graph.set_param(nid, "min", min);
            motion.doc.graph.set_param(nid, "max", max);
        }
        // Both wave behaviours carry the same `channel` enum + `amplitude`.
        "motion.oscillator" | "motion.wiggle" => {
            motion
                .doc
                .graph
                .set_param(nid, "amplitude", wave_channel_amplitude(ch));
            // The oscillator's DC `offset` shares the amplitude's unit, so its
            // range is channel-dependent too (`channel_range_override`). Any param
            // whose RANGE moves with the channel must have its VALUE reset with it
            // — otherwise a 300° offset dialled on Rotation survives into a
            // ±10-world-unit position channel, outside the range it will be shown
            // in. Zero is the neutral offset and is legal on every channel.
            if type_name == "motion.oscillator" {
                motion.doc.graph.set_param(nid, "offset", 0.0);
            }
        }
        _ => {}
    }
    // ⚠️ **E depois a regra DERIVADA, que é a que não apodrece.** Todo param cuja
    // FAIXA segue o canal (a declaração do nó) tem de ter o VALOR trazido para
    // dentro da faixa do canal NOVO — senão um `scale` de 360 graus dialado em
    // Rotation sobrevive para o canal X, onde ele é 360 unidades de mundo e joga
    // as instâncias para fora do quadro, mostrado num slider que não o alcança.
    //
    // Os presets acima escolhem um valor BONITO para os três nós que os têm; isto
    // só garante o mínimo para TODOS — e para aqueles três é no-op, porque o valor
    // que eles escrevem já está dentro (gate).
    clamp_channel_ranged_params(motion, nid, type_name, ch);
}

/// Traz cada param de faixa-por-canal para dentro da faixa que o canal NOVO
/// implica: a declarada, se o canal é angular; a do hint, caso contrário — pela
/// MESMA porta que a row do painel usa, senão a faixa em que o valor é guardado e
/// a faixa em que ele é mostrado divergiriam.
fn clamp_channel_ranged_params(
    motion: &mut MotionState,
    nid: ph2d_nodegraph::graph::NodeId,
    type_name: &str,
    ch: i32,
) {
    let id = ph2d_nodegraph::node::NodeTypeId::of(type_name);
    let Some(decls) = motion
        .registry
        .channel_ranged_types()
        .find(|(k, _)| *k == id)
        .map(|(_, v)| v)
    else {
        return;
    };
    let hints = motion.registry.param_ui(id).unwrap_or(&[]);
    for d in decls {
        let (lo, hi) = if channel_unit(ch) == ParamUnit::Angle {
            (d.min, d.max)
        } else if let Some(h) = hints.iter().find(|h| h.param == d.param) {
            (h.min, h.max)
        } else {
            continue;
        };
        let v = motion
            .doc
            .graph
            .node_param_overrides(nid)
            .and_then(|m| m.get(d.param).copied())
            .unwrap_or_else(|| {
                motion
                    .registry
                    .manifests()
                    .find(|m| m.id == id)
                    .and_then(|m| m.params.iter().find(|p| p.name == d.param))
                    .map_or(0.0, |p| p.default)
            });
        let c = v.clamp(lo, hi);
        if c != v {
            motion.doc.graph.set_param(nid, d.param, c);
        }
    }
}

/// The `channel` enum's Rotation option (see the behaviours' `channel` hint:
/// `0` X, `1` Y, `2` Rotation, `3` Size).
pub(super) const CHANNEL_ROTATION: i32 = 2;

/// Os dois eixos do tamanho do `motion.drive` (`CH_SIZE_X`/`CH_SIZE_Y`, apendados
/// em 2026-08-18). ⚠️ **Eles são a MESMA grandeza que o `Size` (3)** — um eixo de
/// um tamanho não muda de unidade por ser um eixo —, e é por isso que os três
/// entram no mesmo braço em vez de o par cair no `_`.
const CHANNEL_SIZE: i32 = 3;
const CHANNEL_SIZE_X: i32 = 10;
const CHANNEL_SIZE_Y: i32 = 11;

/// **What a magnitude MEANS on the channel it drives** (doc 88) — the resolution
/// of [`ParamUnit::FromChannel`], living next to the ranges and presets that
/// already answer the same question for their own halves.
///
/// This is the whole reason the `FromChannel` variant exists: a stagger's `min`
/// is metres on Position, DEGREES on Rotation and a bare scale factor on Size,
/// and a boundary that converted all three by `pixels_per_meter` would turn the
/// `±90` preset into `±9000`. The three answers were already written down here
/// twice (once as a range, once as a preset); this is the third face of the one
/// fact, deliberately in the same file so they cannot drift.
///
/// An unknown index falls back to [`ParamUnit::None`] — a visible gap is worth
/// more than a wrong scale.
pub(super) fn channel_unit(channel: i32) -> ParamUnit {
    match channel {
        0 | 1 => ParamUnit::Length,           // Position X / Y: world metres
        CHANNEL_ROTATION => ParamUnit::Angle, // the `rot` column: degrees
        // Size e os DOIS eixos dele: um delta de escala, adimensional.
        CHANNEL_SIZE | CHANNEL_SIZE_X | CHANNEL_SIZE_Y => ParamUnit::Ratio,
        _ => ParamUnit::None,
    }
}

/// Stagger `(min, max)` ramp endpoints per channel. The Rotation channel writes
/// the `rot` stream column, whose unit is **degrees** (the app's authored-angle
/// unit); Position is world units, Size a scale delta.
fn stagger_channel_range(channel: i32) -> (f32, f32) {
    match channel {
        CHANNEL_ROTATION => (-90.0, 90.0), // ±90 degrees
        3 => (-0.5, 0.5),                  // Size: ±0.5 scale
        _ => (-1.0, 1.0),                  // Position (X/Y): ±1 world unit
    }
}

/// Peak `amplitude` per channel for the wave behaviours (oscillator / wiggle) —
/// same unit logic as the stagger range.
fn wave_channel_amplitude(channel: i32) -> f32 {
    match channel {
        CHANNEL_ROTATION => 30.0, // ±30 degrees
        3 => 0.3,                 // Size: ±0.3 scale
        _ => 1.0,                 // Position: ±1 world unit
    }
}

/// ⭐⭐⭐ **UM MOLDE ESCOLHIDO ESCREVE AS DUAS CAIXAS DE TEXTO** — a resposta ao *"Axiom e
/// Rules não são nada intuitivos"* (Enio, 2026-08-28).
///
/// ⚠️ **A resposta NÃO foi inventar uma sintaxe amigável**, e a razão é medida em vez de
/// estética: `F[+F]F` é a notação de Lindenmayer, e é ela que está no ABOP, nos tutoriais e em
/// todo exemplo que o artista vai encontrar. Trocá-la tornaria este nó **incompatível com o
/// conhecimento do mundo** — ele deixaria de aceitar o que se copia de qualquer lado. ⇒ o que
/// se dá é um sítio por onde COMEÇAR: escolhe-se um molde, vê-se a planta, e edita-se.
///
/// ⚠️ **Ele vive AQUI, ao lado do preset de canal, e não no crate do nó** — pela mesma razão
/// que aquele: um param que reescreve OUTROS é edição de EDITOR, não matemática de nó. O nó
/// possui a tabela (é vocabulário dele); quem a aplica é a ponte, dentro do mesmo passo de
/// undo, para o `Ctrl+Z` devolver o texto anterior.
///
/// ⚠️ **Ele SOBRESCREVE o que o artista escreveu, e é isso que um molde é.** O undo é a rede,
/// e é por isso que ele corre dentro do passo — sem isso, um clique acidental apagaria uma
/// gramática autorada sem volta.
pub(in crate::render_loop::motion_bridge) fn apply_lsystem_preset(
    motion: &mut MotionState,
    nid: ph2d_nodegraph::graph::NodeId,
    type_name: &str,
    preset: f32,
) {
    if type_name != ph2d_node_source_lsystem::MANIFEST.name {
        return;
    }
    let Some(p) = ph2d_node_source_lsystem::PRESETS.get(preset.round().max(0.0) as usize) else {
        // ⚠️ O `PRESET_CUSTOM` cai aqui, e o silêncio é a lei: *"nenhum destes"* não escreve
        // nada. Ele é o rótulo em que o selector aterra quando outra coisa mexeu no texto.
        return;
    };
    motion
        .doc
        .graph
        .set_text_param(nid, ph2d_node_source_lsystem::AXIOM_PARAM, p.axiom);
    motion
        .doc
        .graph
        .set_text_param(nid, ph2d_node_source_lsystem::RULES_PARAM, p.rules);
    // ⭐⭐⭐ **E O ENQUADRAMENTO** — auditoria de 2026-08-29. Um molde que escrevesse só o
    // texto entregava a curva de Koch a **25°** (ela é 90 por definição) e a **1290 unidades**
    // de mundo, numa coluna de ~4: o dono do produto viu «resultado questionável» em sete dos
    // oito, e era isto. Os quatro números vivem na tabela, medidos, e são escritos no MESMO
    // passo de undo que o próprio molde.
    for (name, v) in [
        (ph2d_node_source_lsystem::param::ANGLE, p.angle),
        (ph2d_node_source_lsystem::param::GENERATIONS, p.generations),
        (ph2d_node_source_lsystem::param::STEP, p.step),
        (ph2d_node_source_lsystem::param::WIDTH, p.width),
    ] {
        motion.doc.graph.set_param(nid, name, v);
    }
}

/// **O selector aterra em `Custom`** — chamado por todo escritor de texto que NÃO é um molde.
///
/// ⚠️ Sem isto o `Preset` mente. Ele é um `ParamSpec` persistido que o `build` **nunca lê**:
/// todo o efeito de um molde vive nesta shell, então o número guardado é o eco de um gesto
/// passado. Três escritores mudam o texto sem lhe tocar — o [`bake_lsystem_grammar`], a edição
/// à mão da caixa, e uma cena —, e o estado de chegada NORMAL (abrir em `Guided`, converter)
/// deixava o selector a dizer «Tree» sobre uma planta **76 % mais alta**. Pior: clicar em
/// «Tree» era mudo, porque o despacho exige que o valor MUDE para correr o molde.
///
/// ⇒ *Um selector só pode nomear um molde enquanto o texto for o daquele molde.*
pub(in crate::render_loop::motion_bridge) fn mark_lsystem_custom(
    motion: &mut MotionState,
    nid: ph2d_nodegraph::graph::NodeId,
    type_name: &str,
) {
    if type_name != ph2d_node_source_lsystem::MANIFEST.name {
        return;
    }
    motion.doc.graph.set_param(
        nid,
        ph2d_node_source_lsystem::param::PRESET,
        ph2d_node_source_lsystem::PRESET_CUSTOM as f32,
    );
}

/// ⭐⭐⭐ **ASSAR o modo guiado no texto** — o que acontece quando o artista muda `Mode` de
/// `Guided` para `Grammar`.
///
/// # Por que a conversão ESCREVE, em vez de o texto ficar como estava
///
/// É a lei da casa para fonte-vs-cozido (ADR-0121) com o verbo `Detach` do ADR-0164: enquanto
/// o modo é guiado, os sliders são a FONTE e a gramática é derivada; a partir do momento em
/// que o artista vai para `Grammar`, a fonte passa a ser o texto — e ele tem de encontrar lá
/// **a planta que estava a ver**, não a de fábrica nem a que escreveu há meia hora.
///
/// ⭐ **É também o único sítio onde alguém APRENDE a notação**: a gramática que aparece é a
/// que os sliders dele faziam, com os nomes dos params lá dentro (`s*length_scale`, `+`,
/// `+(angle*0.500)`). Foi essa a resposta que faltava ao report de 2026-08-29.
///
/// ⚠️ **Só na TRANSIÇÃO, e é o call site que a mede.** Se isto corresse a cada edição com o
/// modo já em `Grammar`, cada mexida num slider escondido reescreveria por cima do que o
/// artista digitou.
///
/// ⚠️⚠️ **E a nota que aqui estava MENTIA** (auditoria de 2026-08-29): ela dizia que *"o
/// caminho de volta não apaga nada: o texto fica intacto, e voltar a `Grammar` devolve-o"*.
/// Não devolve — o `bake` dispara em **toda** transição `Guided → Grammar`, então
/// `Grammar → Guided → Grammar` reescreve por cima da gramática autorada.
///
/// ⇒ **E é assim que tem de ser, não é o defeito.** Enquanto o artista esteve em `Guided`, o
/// que ele viu no ecrã foi a planta dos sliders; devolver-lhe um texto que desenha OUTRA
/// coisa faria a conversão mudar a peça, que é precisamente o que uma conversão não pode
/// fazer. O preço é a gramática autorada, e a rede é o undo (**um** `Ctrl+Z` por transição —
/// a escrita cai no mesmo laço de intents e o registo é por diff pós-quadro).
/// ⛳ *Se o dono do produto preferir o contrário, é decisão dele e não desta função.*
///
/// ⚠️ **Corre dentro do MESMO passo de undo** que o `set_param` do modo — o `Ctrl+Z` devolve
/// o par (modo, texto) de uma vez. É a mesma nota do [`apply_lsystem_preset`].
pub(in crate::render_loop::motion_bridge) fn bake_lsystem_grammar(
    motion: &mut MotionState,
    nid: ph2d_nodegraph::graph::NodeId,
    type_name: &str,
) {
    if type_name != ph2d_node_source_lsystem::MANIFEST.name {
        return;
    }
    use ph2d_node_source_lsystem::param;
    let v = |p: &str| crate::render_loop::motion_bridge::params::param_value(motion, nid, p);
    let (axiom, rules) = ph2d_node_source_lsystem::grammar_for(
        v(param::BRANCHES),
        v(param::SEGMENTS),
        v(param::VARIATION),
        v(param::BEND),
    );
    motion
        .doc
        .graph
        .set_text_param(nid, ph2d_node_source_lsystem::AXIOM_PARAM, axiom);
    motion
        .doc
        .graph
        .set_text_param(nid, ph2d_node_source_lsystem::RULES_PARAM, &rules);
    // O texto assado não é molde nenhum — ver [`mark_lsystem_custom`].
    mark_lsystem_custom(motion, nid, type_name);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Os dois eixos de um tamanho são a MESMA grandeza que o tamanho.**
    ///
    /// ⚠️ Esta tabela **ENUMERA canais**, e um canal novo cai no `_` em silêncio — foi
    /// exactamente o que aconteceu com a faixa por-canal em 2026-08-14, e o par
    /// `Size X`/`Size Y` de 18/08 é a segunda oportunidade de o repetir. O gate não
    /// afirma *qual* é a unidade (isso é decisão de produto e muda com ela): afirma que
    /// os três **concordam**, que é a propriedade que um eixo novo tem de herdar.
    #[test]
    fn the_two_size_axes_carry_the_same_unit_as_the_size_itself() {
        let size = channel_unit(CHANNEL_SIZE);
        assert_eq!(
            channel_unit(CHANNEL_SIZE_X),
            size,
            "o eixo X de um tamanho tem de ler a unidade do tamanho"
        );
        assert_eq!(
            channel_unit(CHANNEL_SIZE_Y),
            size,
            "o eixo Y de um tamanho tem de ler a unidade do tamanho"
        );
        // E o CONTROLE: a tabela ainda DISCRIMINA. Sem isto, uma tabela que
        // devolvesse a mesma coisa para tudo satisfaria as duas linhas acima.
        assert_ne!(
            channel_unit(CHANNEL_ROTATION),
            size,
            "a rotacao nao pode ler a mesma unidade que o tamanho"
        );
    }

    /// ⚠️ **Os dois consumidores desta tabela perguntam UMA coisa** — *este canal é
    /// ANGULAR?* — e um eixo de tamanho não é. O gate existe porque a resposta certa
    /// aqui é a mesma com `Ratio` e com o antigo `None`, então a linha acima poderia
    /// mudar sem ninguém notar que ela alcança a conversão de graus.
    #[test]
    fn no_size_channel_is_angular() {
        for ch in [CHANNEL_SIZE, CHANNEL_SIZE_X, CHANNEL_SIZE_Y] {
            assert_ne!(channel_unit(ch), ParamUnit::Angle, "canal {ch}");
        }
        assert_eq!(channel_unit(CHANNEL_ROTATION), ParamUnit::Angle);
    }
}
