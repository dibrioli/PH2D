//! **Onde cada etapa da gravação toca** (W16): a porta única de quem lê as etapas na trilha que toca.

use super::{LeitorDoPainter, NoToque, TipoDaEtapa, apagada, e_um_colordrop, passo, pinceis};
use crate::macro_track::passo::Passo;
use crate::macro_track::tocar::ErroDaMacro;
use ph2d_macro::{Etapa, LeitorDaFamilia, Track};
use std::collections::{BTreeMap, VecDeque};

/// Uma etapa VIVA da gravação e onde ela toca.
pub(crate) struct Tocada {
    /// A etapa a contar da gravação.
    pub k: usize,
    pub tipo: TipoDaEtapa,
    pub gravada: Etapa,
    /// Os eventos dela na trilha que toca (`tocavel`), do primeiro ao último.
    pub tocada: Etapa,
    /// A faixa dela (`0` = a da gravação; W16: a etapa deslocada toca na `k + 1`).
    pub faixa: u32,
    /// O pincel no toque (uma pincelada ou um ColorDrop).
    pub no_toque: Option<NoToque>,
}

/// **Cada etapa viva de `track`, pela ordem da gravação, e onde ela toca em `tocavel`** (a trilha que
/// toca dela) — a porta única de quem lê as etapas na trilha que toca: a deslocada (W16) toca na faixa
/// dela, entrelaçada ou noutra ordem; as outras, pela ordem.
pub(crate) fn tocadas(track: &Track, tocavel: &Track) -> Result<Vec<Tocada>, ErroDaMacro> {
    let gravadas = LeitorDoPainter.etapas(&track.events);
    let faixa_de = |e: &Etapa| crate::macro_track::passo::faixa_de(&tocavel.events[e.inicio].payload);
    let tocaveis = LeitorDoPainter.etapas(&tocavel.events);
    let mut na_ordem = tocaveis.iter().filter(|e| faixa_de(e) == 0);
    let mut por_faixa: BTreeMap<u32, Etapa> = tocaveis
        .iter()
        .filter(|e| faixa_de(e) != 0)
        .map(|e| (faixa_de(e), *e))
        .collect();
    // O pincel de cada toque, por faixa, pela ordem dos toques.
    let mut no_toque: BTreeMap<u32, VecDeque<NoToque>> = BTreeMap::new();
    for (n, p) in pinceis(tocavel)? {
        no_toque.entry(n).or_default().push_back(p);
    }
    let deslocadas: Vec<usize> = track
        .edicoes
        .iter()
        .filter(|e| e.desloca_us != 0 && !e.apagada)
        .map(|e| e.ancora)
        .collect();
    let (mut n, mut f) = (0, 0);
    let mut out = Vec::with_capacity(gravadas.len());
    for (k, g) in gravadas.iter().enumerate() {
        let tipo = match passo(&track.events[g.fim].payload) {
            Some(Passo::Recusada(porta)) => TipoDaEtapa::Recusada(porta),
            _ if e_um_colordrop(&track.events[..=g.inicio]) => {
                f += 1;
                TipoDaEtapa::Preenchimento(f)
            }
            _ => {
                n += 1;
                TipoDaEtapa::Pincelada(n)
            }
        };
        if apagada(track, g) {
            continue;
        }
        let faixa = if deslocadas.contains(&g.inicio) {
            u32::try_from(k + 1).unwrap_or(u32::MAX)
        } else {
            0
        };
        let tocada = if faixa == 0 {
            na_ordem.next().copied()
        } else {
            por_faixa.remove(&faixa)
        };
        let Some(tocada) = tocada else { break };
        let no_toque = match tipo {
            TipoDaEtapa::Pincelada(_) | TipoDaEtapa::Preenchimento(_) => {
                no_toque.get_mut(&faixa).and_then(VecDeque::pop_front)
            }
            TipoDaEtapa::Recusada(_) => None,
        };
        out.push(Tocada {
            k,
            tipo,
            gravada: *g,
            tocada,
            faixa,
            no_toque,
        });
    }
    Ok(out)
}

