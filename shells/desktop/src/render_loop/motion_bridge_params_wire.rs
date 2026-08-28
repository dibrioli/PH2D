//! **A ROUPA DE QUEM CONDUZ** — a face e a faixa que um nó de valor veste quando o fio
//! dele cai num param (doc 58 + doc 88).
//!
//! Irmã de `params_range` (até onde a row alcança), `params_channel` (o que a row É) e
//! `params_stream` (o que os fios carregam). Cortada aqui pelo teto de 600 LOC da shell,
//! pelo critério de sempre — por ASSUNTO.
//!
//! # O report
//!
//! *"number em 0,94 imprime em shape:size 94px. melhor corrigir isso."* — Enio, 2026-08-27.
//!
//! O número **não** estava errado: o `source.shape::size` é um `ParamUnit::Length`, guardado
//! em metros, e `0,94 m` **são** `94 px` a `pixels_per_meter = 100`. As duas rows estavam
//! honestas cada uma sobre a própria unidade — e é exactamente por isso que o artista não
//! conseguia reconciliá-las: **um fio, dois números**.
//!
//! # A lei
//!
//! *Um nó que conduz um param veste a roupa daquele param* — a face **e** a faixa. Não é
//! gosto: é a única leitura em que arrastar o `Number` e arrastar o `Size` fazem a mesma
//! coisa, que é o que ligar um ao outro promete.
//!
//! ⚠️ **E o `display_face` mora aqui** desde 2026-08-27: *em que face esta row se lê?* é uma
//! pergunta só, e as duas metades dela (o que o destino impõe · a conversão para o número que
//! o artista vê) viviam em arquivos diferentes. A mudança foi forçada pelo teto de LOC da
//! shell e o corte é o que a pergunta desenha — não onde a linha 600 calhou.
//!
//! ⚠️ **E ela NÃO contradiz a cerca do `ParamUnit::None`** — completa-a. A cerca diz que a
//! magnitude de um `value.*` *"não tem unidade própria: ela significa o que a coluna em que
//! o artista a liga significa"*, e conclui que uma lacuna visível vale mais que um número
//! errado. A conclusão vale enquanto o destino é uma COLUNA (que pode ser qualquer coisa).
//! Um param **dirigido** não é uma coluna: é um param declarado, com unidade declarada, e o
//! grafo sabe qual. ⇒ nos casos de que a cerca falava (nenhum destino, destinos que
//! DISCORDAM, destino que também é `FromWire`) esta porta devolve `None` e a lacuna fica.

use super::params_channel::channel_unit;
use crate::motion_state::MotionState;
use ph2d_node_registry::{ParamUnit, ParamWidget};
use ph2d_nodegraph::cook::OpResolver;
use ph2d_nodegraph::graph::NodeId;
use ph2d_panel_motion_params::RowDisplay;

/// A face que o destino impõe a quem o conduz.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(super) struct WireFace {
    /// A unidade do param conduzido — nunca `FromWire` nem `FromChannel` (as duas são
    /// perguntas, e esta struct só carrega respostas).
    pub(super) unit: ParamUnit,
    /// `(min, max, step)` do destino, quando ele declara um hint. `None` quando nenhum
    /// destino tem faixa declarada — a face ainda vale, e é só a faixa que fica com a do
    /// próprio condutor.
    pub(super) range: Option<(f32, f32, f32)>,
}

/// **O que o nó `driver` conduz, resolvido numa face só** — ou `None` quando não há
/// resposta honesta.
///
/// ⚠️ **A varredura é sobre `all_param_sources`, não sobre as arestas.** Um param dirigido
/// não é uma aresta (`NodeManifest.inputs` é congelado, ADR-0039): ele vive no mapa próprio
/// do documento, e é lá que está a única lista de quem este nó alimenta.
///
/// As três recusas, cada uma a cerca original do `ParamUnit::None`:
/// - **não conduz nada** ⇒ a magnitude não tem destino, logo não tem unidade;
/// - **destinos que DISCORDAM** (um `Length` e um `Angle`) ⇒ uma face qualquer seria certa
///   metade do tempo, e a metade errada escala por `pixels_per_meter`;
/// - **um destino que é ele próprio `FromWire`** ⇒ a resposta ainda não é um facto.
///   ⚠️ **Não é sobre terminação** — o `Graph::drive_param` já recusa fechar ciclo
///   (`EdgeError::WouldCycle`, porque um param dirigido *é* uma dependência), então a recursão
///   terminaria. É sobre o que o artista LÊ: com recursão, a face de um nó viria de um destino
///   a três saltos e nada na tela o diria. *Uma face que se resolve num salto pode ser
///   apontada; uma que se resolve numa cadeia é adivinhação com procedimento.*
pub(super) fn wire_face(motion: &MotionState, driver: NodeId) -> Option<WireFace> {
    let mut acc: Option<WireFace> = None;
    for (dst, sources) in motion.doc.graph.all_param_sources() {
        for (param, (src, _port)) in sources {
            if *src != driver {
                continue;
            }
            let WireFace { unit, range } = destination_face(motion, *dst, param)?;
            acc = Some(match acc {
                None => WireFace { unit, range },
                // Discordância de UNIDADE é recusa; discordância de FAIXA não é — dois
                // params da mesma unidade são a mesma grandeza, e a união das faixas
                // alcança os dois sem mentir sobre nenhum.
                Some(a) if a.unit == unit => WireFace {
                    unit,
                    range: match (a.range, range) {
                        (Some((a0, a1, a2)), Some((b0, b1, b2))) => {
                            Some((a0.min(b0), a1.max(b1), a2.min(b2)))
                        }
                        (some, None) | (None, some) => some,
                    },
                },
                Some(_) => return None,
            });
        }
    }
    acc
}

/// A unidade e a faixa de UM param de UM nó — a mesma resolução que o construtor de rows
/// faz para o nó selecionado (`unit_of` primeiro, a tabela declarada depois).
///
/// ⚠️ **O widget responde primeiro, e é o que torna isto seguro:** um `Angle`/`IntSlider`/
/// `Enum` fixa a própria unidade e nenhuma declaração a contradiz, então um destino desses
/// nunca pede conversão de comprimento.
fn destination_face(motion: &MotionState, dst: NodeId, param: &str) -> Option<WireFace> {
    let inst = motion.doc.graph.node(dst)?;
    let type_id = inst.type_id();
    let hint = motion
        .registry
        .param_ui(type_id)
        .and_then(|hs| hs.iter().find(|h| h.param == param));
    let unit = ph2d_node_registry::unit_of(
        hint.map_or(ParamWidget::Slider, |h| h.widget),
        motion.registry.param_unit_declared(type_id, param),
    );
    let unit = match unit {
        // O destino é ele próprio uma pergunta sobre o canal: responde-se com o canal DELE,
        // pela mesma porta que a row dele usa. Um nó sem `channel` não pode responder, e uma
        // unidade sem resposta é `None` — nunca um palpite.
        ParamUnit::FromChannel => has_channel(motion, dst)
            .map(channel_unit)
            .unwrap_or_default(),
        // A recusa que TERMINA a recursão (ver o doc da função pública).
        ParamUnit::FromWire => return None,
        other => other,
    };
    Some(WireFace {
        unit,
        range: hint.map(|h| (h.min, h.max, h.step)),
    })
}

/// O canal que `node` conduz agora, se ele tiver um param `channel` — a MESMA derivação do
/// construtor de rows (`manifest.params.any(name == "channel")` e depois o valor), nunca uma
/// segunda: um destino cuja unidade dependesse de um canal lido de outra maneira poria a
/// face do condutor e a do destino em desacordo, que é o defeito que este arquivo cura.
fn has_channel(motion: &MotionState, node: NodeId) -> Option<i32> {
    let inst = motion.doc.graph.node(node)?;
    let manifest = motion.registry.resolve(inst.type_id())?.manifest();
    manifest
        .params
        .iter()
        .any(|p| p.name == "channel")
        .then(|| super::param_value(motion, node, "channel").round() as i32)
}

/// **The display face for one param** (doc 88, Wave A) — the single place a
/// declared [`ParamUnit`] becomes the number the artist reads.
///
/// [`ParamUnit::Length`] is the only unit that CONVERTS, and it converts through
/// the project's setting, never a constant of its own: the same
/// `pixels_per_meter` the sprite importer and the gizmo readouts use. Everything
/// else is stored in the unit it is shown in, so it gets a suffix and a scale of
/// exactly `1.0` — the neutral face, byte-identical to before this wave.
///
/// [`ParamUnit::FromChannel`] is resolved first, by asking the channel the node
/// currently drives; a node with no `channel` param cannot answer, and an
/// unanswerable unit is [`ParamUnit::None`] rather than a guess.
///
/// [`ParamUnit::FromWire`] is the same shape one step further out: the unit is the
/// one of the param this node's output DRIVES (`params_wire`). Unresolvable for the
/// same reason and with the same answer — `None`, never a guess.
pub(super) fn display_face(
    unit: ParamUnit,
    channel: Option<i32>,
    wire: Option<ParamUnit>,
    project: ph2d_editor::ProjectSettings,
) -> RowDisplay {
    let unit = match unit {
        ParamUnit::FromChannel => channel.map(channel_unit).unwrap_or_default(),
        ParamUnit::FromWire => wire.unwrap_or_default(),
        other => other,
    };
    if let Some(fixed) = unit.fixed_suffix() {
        return RowDisplay::new(1.0, fixed);
    }
    if !unit.converts() {
        return RowDisplay::default();
    }
    // A world LENGTH: stored in metres, shown in the project's unit.
    RowDisplay::new(
        f64::from(
            project
                .display_unit
                .from_meters(1.0, project.pixels_per_meter),
        ),
        project.display_unit.suffix(),
    )
}

#[cfg(test)]
#[path = "motion_bridge_params_wire_tests.rs"]
mod tests;
