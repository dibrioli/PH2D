//! **A trilha do Motion noutro documento** (W18, `docs/Macros/16_W18_plano.md` §2.5 e §2.7): o mesmo
//! gesto gravado, com o estado inicial reescrito sobre OUTRO documento e outro dono.
//!
//! - [`so_do_dono`] — exportar: o inicial leva só o grafo do dono (o resto do projecto não viaja).
//! - [`aplica`] — aplicar (novo ou por cima): os nós da macro juntam-se aos do dono novo no documento
//!   vivo, e o fim da reprodução é o documento que o editor passa a mostrar.
//!
//! Uma porta só reescreve ([`Mapa`]): cada `Alvo::Existente` pelo mapa do `import_owner`, a base dos
//! criados pelo `next_id` novo, os grupos e as molduras criados a gravar pelo deslocamento do próximo
//! id, e cada `Passo::Documento` reduzido ao grafo do dono com esses ids. O que não se reescreve AO BIT
//! recusa-se pelo nome ([`ErroAoTranspor`]) — nunca se larga um passo em silêncio. Os grupos e as
//! molduras que já existiam ao armar não viajam (são da vista, a lei do `import_owner`).

use super::FAMILIA;
use super::etapas::LeitorDoMotion;
use super::inicial::{self, ErroDoInicial, Inicial};
use super::tocar::{ErroDaMacro, Modo, TocadorDoMotion};
use ph2d_macro::{CodecError, LeitorDaFamilia, Track};
use ph2d_motion_doc::MotionDoc;
use ph2d_motion_doc::subgraph;
use ph2d_nodegraph::graph::NodeId;
use std::collections::BTreeSet;

/// Porque a trilha não se transpõe.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErroAoTranspor {
    /// A trilha não é do Motion.
    NaoEDoMotion,
    /// A trilha (ou o destino) não tem objecto dono (`StableId` `0`).
    SemDono,
    Inicial(ErroDoInicial),
    Passo(CodecError),
    /// Um passo, intenção ou edição de param de uma versão mais nova: os ids dele não se conhecem.
    Desconhecido(u64),
    /// Um passo (ou o documento de um passo) nomeia um nó ou uma moldura de OUTRO objecto.
    DoutroDono(u32),
    /// Um fio ou um param dirigido entre um nó do dono e um de fora.
    FioDeFora,
    /// Um grupo ou uma moldura que já existia ao armar (a vista não viaja).
    VistaDeAntes(u32),
    /// Nós do dono dentro de um grupo DESLIGADO que não viaja: o cozimento mudaria.
    GrupoDesligado,
    /// O documento de um passo é de antes de armar (`next_id` abaixo da base, ou a vista de antes mudou).
    DocumentoDeAntes,
    /// O documento de um passo não se lê.
    Documento(ErroDaMacro),
    /// O documento reduzido não volta ao bit pelo texto.
    DocumentoSemTexto,
    /// A reprodução da trilha transposta parou.
    Tocar(ErroDaMacro),
    /// A reprodução mudou um nó que o documento vivo já tinha.
    TocouOsNosDeAntes,
}

type E = ErroAoTranspor;

#[path = "transpor_mapa.rs"]
mod mapa;
use mapa::Mapa;

/// **A trilha a exportar**: a mesma, com o inicial reduzido aos nós de `track.target` (importados para
/// um documento vazio, sob o mesmo dono).
///
/// # Errors
/// [`ErroAoTranspor`].
pub fn so_do_dono(track: &Track) -> Result<Track, ErroAoTranspor> {
    let ini = le(track)?;
    let mut vazio = MotionDoc::new();
    vazio.base_z = ini.doc.base_z;
    transpoe(track, &ini, &vazio, track.target)
}

/// **Aplicar** a trilha a `novo_dono` em `vivo` — um objecto acabado de nascer (sem nós) ou um objecto
/// Motion que já tem os seus (os da macro juntam-se-lhes). Devolve a trilha a instalar (inicial = `vivo`
/// + os nós da macro sob `novo_dono`) e o documento do FIM dela, que substitui o vivo.
///
/// # Errors
/// [`ErroAoTranspor`].
pub fn aplica(
    track: &Track,
    vivo: &MotionDoc,
    novo_dono: u64,
) -> Result<(Track, MotionDoc), ErroAoTranspor> {
    if novo_dono == 0 {
        return Err(E::SemDono);
    }
    let ini = le(track)?;
    let t = transpoe(track, &ini, vivo, novo_dono)?;
    let doc = fim(&t).map_err(E::Tocar)?;
    if !intactos(vivo, &doc) {
        return Err(E::TocouOsNosDeAntes);
    }
    Ok((t, doc))
}

/// **O documento do fim da trilha** — o tocador ([`TocadorDoMotion`]) passo a passo, sem cozer os
/// quadros (o relógio não muda o grafo; a mesma escolha de [`super::etapas`]).
///
/// # Errors
/// [`ErroDaMacro`] do passo que parou.
pub fn fim(track: &Track) -> Result<MotionDoc, ErroDaMacro> {
    let t = track
        .tocavel(&LeitorDoMotion)
        .map_err(ErroDaMacro::Edicao)?;
    let mut tocador = TocadorDoMotion::da_trilha(&t, Modo::Explicativo)?;
    for e in &t.events {
        if !LeitorDoMotion.e_relogio(&e.payload) {
            tocador.aplica(&e.payload, None)?;
        }
    }
    Ok(tocador.palco().doc.clone())
}

fn le(track: &Track) -> Result<Inicial, E> {
    if track.family != FAMILIA {
        return Err(E::NaoEDoMotion);
    }
    if track.target == 0 {
        return Err(E::SemDono);
    }
    inicial::le(&track.initial).map_err(E::Inicial)
}

fn transpoe(track: &Track, ini: &Inicial, destino: &MotionDoc, novo: u64) -> Result<Track, E> {
    let dono = track.target;
    let i = &ini.doc;
    fios_so_do_dono(i, |n| i.owner_of(n) == Some(dono))?;
    if i.nodes_of(dono)
        .iter()
        .any(|n| em_grupo_desligado(i, *n, u32::MAX))
    {
        return Err(E::GrupoDesligado);
    }
    let mut j = destino.clone();
    let nos = j.import_owner(i, dono, novo);
    if i.arrumados_a_mao.contains(&dono) {
        j.arrumados_a_mao.insert(novo);
    }
    let mapa = Mapa {
        ini: i,
        destino,
        dono,
        novo,
        nos,
        base: ini.base,
        base_nova: j.graph.next_id(),
        grupos: (
            subgraph::next_id(&i.subgraphs),
            subgraph::next_id(&destino.subgraphs),
        ),
        fundos: (proxima_moldura(i), proxima_moldura(destino)),
    };
    let initial = inicial::escreve(&Inicial {
        doc: j,
        base: mapa.base_nova,
        lei: ini.lei,
        uv: ini.uv,
        tamanho: ini.tamanho,
    });
    let mut events = track.events.clone();
    for e in &mut events {
        e.payload = mapa.carga(&e.payload)?;
    }
    let mut edicoes = track.edicoes.clone();
    for ed in &mut edicoes {
        for (_, c) in ed.trocas.iter_mut().chain(ed.insercoes.iter_mut()) {
            *c = mapa.carga(c)?;
        }
    }
    Ok(Track {
        family: track.family.clone(),
        target: novo,
        initial,
        events,
        edicoes,
        nasce: track.nasce.clone(),
        continua: track.continua,
    })
}

/// A lei do `next_id` das molduras (`motion_bridge_backdrops`).
fn proxima_moldura(d: &MotionDoc) -> u32 {
    d.backdrops
        .iter()
        .map(|b| b.id)
        .max()
        .map_or(0, |m| m.saturating_add(1))
}

/// Nenhum fio nem param dirigido atravessa a fronteira do dono.
fn fios_so_do_dono(d: &MotionDoc, e_do_dono: impl Fn(NodeId) -> bool) -> Result<(), E> {
    let cruza = |a: NodeId, b: NodeId| e_do_dono(a) != e_do_dono(b);
    if d.graph.edges().iter().any(|e| cruza(e.from.0, e.to.0))
        || d.graph
            .all_param_sources()
            .iter()
            .any(|(n, s)| s.values().any(|(src, _)| cruza(*n, *src)))
    {
        return Err(E::FioDeFora);
    }
    Ok(())
}

/// `n` está (a qualquer profundidade) num grupo desligado de id abaixo de `limite`.
fn em_grupo_desligado(d: &MotionDoc, n: NodeId, limite: u32) -> bool {
    let mut s = d.members.get(&n).copied();
    let mut vistos = BTreeSet::new();
    while let Some(id) = s {
        if !vistos.insert(id) {
            return false;
        }
        if id < limite && d.bypassed_subgraphs.contains(&id) {
            return true;
        }
        s = d
            .subgraphs
            .iter()
            .find(|x| x.id == id)
            .and_then(|x| x.parent);
    }
    false
}

/// A vista que já existia ao armar: os grupos, os desligados e as molduras abaixo dos próximos ids.
fn vista_de_antes(d: &MotionDoc, grupos: u32, fundos: u32) -> [BTreeSet<u32>; 3] {
    [
        d.subgraphs
            .iter()
            .map(|s| s.id)
            .filter(|s| *s < grupos)
            .collect(),
        d.bypassed_subgraphs
            .iter()
            .copied()
            .filter(|s| *s < grupos)
            .collect(),
        d.backdrops
            .iter()
            .map(|b| b.id)
            .filter(|b| *b < fundos)
            .collect(),
    ]
}

/// Cada nó que `vivo` tinha está no `fim` como estava, e os fios dele também.
fn intactos(vivo: &MotionDoc, fim: &MotionDoc) -> bool {
    let (a, b) = (&vivo.graph, &fim.graph);
    a.nodes().iter().all(|n| {
        let id = n.id;
        b.node(id).map(|x| &x.type_name) == Some(&n.type_name)
            && vivo.owner_of(id) == fim.owner_of(id)
            && a.node_param_overrides(id) == b.node_param_overrides(id)
            && a.node_text_param_overrides(id) == b.node_text_param_overrides(id)
            && a.label(id) == b.label(id)
            && a.node_bypassed(id) == b.node_bypassed(id)
            && a.pos(id) == b.pos(id)
            && a.param_sources(id) == b.param_sources(id)
    }) && a.edges().iter().all(|e| b.edges().contains(e))
}

/// A frase do porquê a trilha não viaja.
#[must_use]
pub fn erro_view(e: &ErroAoTranspor) -> ph2d_editor_core::macro_edits::ErroDaMacroView {
    use ph2d_editor_core::macro_edits::MACRO_LIB_NAO_TOCA;
    match e {
        E::Tocar(x) | E::Documento(x) => super::vistas::do_motion(x),
        outro => ph2d_editor_core::macro_etapas::erros::erro_com(
            MACRO_LIB_NAO_TOCA,
            format!("{outro:?}"),
        ),
    }
}

/// **A família MACROS leva a trilha por esta porta** (W18): exportar = só o grafo do dono, tocado até ao
/// fim; aplicar = os nós da macro no documento vivo, que passa ao fim dela num passo de undo.
impl ph2d_editor_core::macro_tutorial::TranspoeDaFamilia for crate::motion_state::MotionState {
    fn exporta(
        &self,
        track: &Track,
    ) -> Result<Track, ph2d_editor_core::macro_edits::ErroDaMacroView> {
        let t = so_do_dono(track).map_err(|e| erro_view(&e))?;
        fim(&t).map_err(|e| super::vistas::do_motion(&e))?;
        Ok(t)
    }

    fn aplica(
        &mut self,
        track: &Track,
        dono: u64,
        vida: u64,
    ) -> Result<Track, ph2d_editor_core::macro_edits::ErroDaMacroView> {
        let (t, doc) = aplica(track, &self.doc, dono).map_err(|e| erro_view(&e))?;
        if doc != self.doc {
            let pre = std::mem::replace(&mut self.doc, doc);
            self.objeto
                .atados
                .ata(&mut self.history, vida, pre, &self.doc);
            self.pump.mark_dirty();
        }
        Ok(t)
    }
}

#[cfg(all(test, feature = "panel-motion-graph"))]
#[path = "transpor_tests.rs"]
pub(crate) mod tests;
