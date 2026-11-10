//! **Editar uma pincelada da trilha do Painter** (W5, `docs/Macros/02_plano_W5.md` §3): apagar, a cor, o
//! tamanho, o caminho e a espera de uma etapa — cada edição toca só nela. Partido de `etapas.rs` (B52.R24).

use super::{EtapaDoPainter, LeitorDoPainter, TipoDaEtapa, etapas, passo};
use crate::macro_track::passo::Passo;
use crate::macro_track::tocar::ErroDaMacro;
use ph2d_editor_core::tool::PointerPhase;
use ph2d_macro::{Etapa, LeitorDaFamilia, Track};

/// **Apaga a etapa `k`**: o tempo dela fecha-se e a água passa ao relógio fixo ali.
///
/// # Errors
/// [`ErroDeEdicaoDoPainter::SemEtapa`].
pub fn apaga(track: &mut Track, k: usize) -> Result<(), ErroDeEdicaoDoPainter> {
    if track.apaga_etapa(&LeitorDoPainter, k) {
        Ok(())
    } else {
        Err(ErroDeEdicaoDoPainter::SemEtapa(k))
    }
}

/// Uma edição pedida a uma etapa que não a aceita.
#[derive(Clone, Debug, PartialEq)]
pub enum ErroDeEdicaoDoPainter {
    SemEtapa(usize),
    NaoEPincelada(usize),
    Macro(ErroDaMacro),
    /// Só duas pinceladas simples tocam ao mesmo tempo (W16): um preenchimento, uma forma, uma seleção
    /// ou uma porta recusada não se sobrepõem a outra etapa.
    SoPinceladasSeSobrepoem(usize),
    /// A etapa tocaria noutro meio (duas sobrepostas de meios diferentes, ou a água do Wet Paint que a
    /// tela tem no instante novo).
    OutroMeio(usize),
    /// Um preenchimento sem Done (o modal ficou aberto) e o toque que desceu ainda em Fill dependem um
    /// do outro (W17): nenhum dos dois anda sozinho.
    ColorDropAberto(usize),
}

fn pincelada(
    track: &Track,
    k: usize,
) -> Result<(Etapa, usize, EtapaDoPainter), ErroDeEdicaoDoPainter> {
    let e = etapas(track)
        .map_err(ErroDeEdicaoDoPainter::Macro)?
        .into_iter()
        .find(|e| e.k == k)
        .ok_or(ErroDeEdicaoDoPainter::SemEtapa(k))?;
    // O ColorDrop tem a cor e o sítio onde a cor caiu; o tamanho do pincel não lhe chega.
    if !matches!(
        e.tipo,
        TipoDaEtapa::Pincelada(_) | TipoDaEtapa::Preenchimento(_)
    ) {
        return Err(ErroDeEdicaoDoPainter::NaoEPincelada(k));
    }
    let down = (e.etapa.inicio..=e.etapa.fim)
        .find(|&i| {
            matches!(
                passo(&track.events[i].payload),
                Some(Passo::Ponteiro {
                    fase: PointerPhase::Down,
                    ..
                })
            )
        })
        .expect("uma pincelada tem o seu toque");
    Ok((e.etapa, down, e))
}

/// Põe `novo` antes do toque e o que a pincelada tinha (`antigo`) depois do levantar — as inserções
/// anteriores da MESMA propriedade nesta etapa saem.
fn troca_a_volta(
    track: &mut Track,
    etapa: Etapa,
    down: usize,
    mesma: impl Fn(&Passo) -> bool,
    novo: Passo,
    antigo: Passo,
) {
    let depois = etapa.fim + 1;
    let tem_depois = depois < track.events.len();
    let ed = track.edicao_mut(etapa.inicio);
    ed.insercoes
        .retain(|(i, c)| !((*i == down || *i == depois) && passo(c).is_some_and(|p| mesma(&p))));
    ed.insercoes.push((down, novo.codifica()));
    if tem_depois {
        ed.insercoes.push((depois, antigo.codifica()));
    }
}

/// **A cor da pincelada `k`** (só dela).
///
/// # Errors
/// [`ErroDeEdicaoDoPainter`].
pub fn edita_cor(track: &mut Track, k: usize, cor: [u8; 3]) -> Result<(), ErroDeEdicaoDoPainter> {
    let (etapa, down, _) = pincelada(track, k)?;
    let original = etapas(&Track {
        edicoes: Vec::new(),
        nasce: None,
        continua: None,
        ..track.clone()
    })
    .map_err(ErroDeEdicaoDoPainter::Macro)?[k]
        .cor;
    troca_a_volta(
        track,
        etapa,
        down,
        |p| matches!(p, Passo::Cor(_)),
        Passo::Cor(cor),
        Passo::Cor(original),
    );
    Ok(())
}

/// **O tamanho da pincelada `k`**, em píxeis (só dela).
///
/// # Errors
/// [`ErroDeEdicaoDoPainter`].
pub fn edita_tamanho(track: &mut Track, k: usize, px: f32) -> Result<(), ErroDeEdicaoDoPainter> {
    let (etapa, down, e) = pincelada(track, k)?;
    if !matches!(e.tipo, TipoDaEtapa::Pincelada(_)) {
        return Err(ErroDeEdicaoDoPainter::NaoEPincelada(k));
    }
    let original = etapas(&Track {
        edicoes: Vec::new(),
        nasce: None,
        continua: None,
        ..track.clone()
    })
    .map_err(ErroDeEdicaoDoPainter::Macro)?[k]
        .tamanho_px;
    troca_a_volta(
        track,
        etapa,
        down,
        |p| matches!(p, Passo::TamanhoPx(_)),
        Passo::TamanhoPx(px),
        Passo::TamanhoPx(original),
    );
    Ok(())
}

/// **O caminho da pincelada `k` deslocado** `d` (fracções da tela de referência), a contar do
/// GRAVADO — editar outra vez substitui, não acumula.
///
/// # Errors
/// [`ErroDeEdicaoDoPainter`].
pub fn desloca_caminho(
    track: &mut Track,
    k: usize,
    d: [f64; 2],
) -> Result<(), ErroDeEdicaoDoPainter> {
    let (etapa, _, _) = pincelada(track, k)?;
    let novos: Vec<(usize, Vec<u8>)> = (etapa.inicio..=etapa.fim)
        .filter_map(|i| match passo(&track.events[i].payload)? {
            Passo::Ponteiro {
                pos,
                pressao,
                tilt,
                fase,
            } => Some((
                i,
                Passo::Ponteiro {
                    pos: [pos[0] + d[0], pos[1] + d[1]],
                    pressao,
                    tilt,
                    fase,
                }
                .codifica(),
            )),
            _ => None,
        })
        .collect();
    let ed = track.edicao_mut(etapa.inicio);
    ed.trocas
        .retain(|(i, _)| !(etapa.inicio..=etapa.fim).contains(i));
    if d != [0.0, 0.0] {
        ed.trocas.extend(novos);
    }
    Ok(())
}

/// **A espera antes da etapa `k`**, em µs (+ abre, − encurta); a etapa e as seguintes andam com ela.
///
/// # Errors
/// [`ErroDeEdicaoDoPainter::SemEtapa`].
pub fn espera(track: &mut Track, k: usize, us: i64) -> Result<(), ErroDeEdicaoDoPainter> {
    let e = *LeitorDoPainter
        .etapas(&track.events)
        .get(k)
        .ok_or(ErroDeEdicaoDoPainter::SemEtapa(k))?;
    track.edicao_mut(e.inicio).espera_us = us;
    Ok(())
}
