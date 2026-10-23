//! **A etapa que anda sozinha** (W16, `docs/Macros/14_W16_plano.md`): o losango arrastado na Timeline
//! leva a etapa `k` para outro instante — as outras ficam. Sobrepor-se a outra só entre pinceladas
//! simples do mesmo meio (dois pincéis na mesma tinta); uma etapa levada para fora do sítio dela pinta com
//! o pincel dela, e o meio do Wet Paint (a água armada, que é da tela) tem de ser o dela.

use super::{ErroDeEdicaoDoPainter, LeitorDoPainter, tocadas};
use crate::macro_track::tocar::ErroDaMacro;
use ph2d_macro::{LeitorDaFamilia, Track};

/// **Desloca a etapa `k`** `us` µs a partir do sítio onde a gravação (e as esperas) a põe — absoluto:
/// editar outra vez substitui. Uma edição que não cabe deixa a trilha como estava.
///
/// # Errors
/// [`ErroDeEdicaoDoPainter`].
pub fn desloca(track: &mut Track, k: usize, us: i64) -> Result<(), ErroDeEdicaoDoPainter> {
    let e = *LeitorDoPainter
        .etapas(&track.events)
        .get(k)
        .ok_or(ErroDeEdicaoDoPainter::SemEtapa(k))?;
    let antes = track.edicoes.clone();
    track.edicao_mut(e.inicio).desloca_us = us;
    // Uma edição vazia não se guarda: o ficheiro sem deslocamentos continua o de antes.
    let vazia = ph2d_macro::EdicaoDeEtapa {
        ancora: e.inicio,
        ..Default::default()
    };
    track.edicoes.retain(|x| *x != vazia);
    let r = confere(track, k);
    if r.is_err() {
        track.edicoes = antes;
    }
    r
}

fn confere(track: &Track, k: usize) -> Result<(), ErroDeEdicaoDoPainter> {
    let m = |x: ErroDaMacro| ErroDeEdicaoDoPainter::Macro(x);
    let tocavel = track
        .tocavel(&LeitorDoPainter)
        .map_err(|x| m(ErroDaMacro::Edicao(x)))?;
    let ts = tocadas(track, &tocavel).map_err(m)?;
    let eu = ts
        .iter()
        .find(|t| t.k == k)
        .ok_or(ErroDeEdicaoDoPainter::SemEtapa(k))?;
    let vao = |t: &super::Tocada| {
        (
            tocavel.events[t.tocada.inicio].t,
            tocavel.events[t.tocada.fim].t,
        )
    };
    let (a0, a1) = vao(eu);
    for o in ts.iter().filter(|o| o.k != k) {
        let (b0, b1) = vao(o);
        if a0 > b1 || b0 > a1 {
            continue;
        }
        let (Some(p), Some(q)) = (eu.no_toque, o.no_toque) else {
            return Err(ErroDeEdicaoDoPainter::SoPinceladasSeSobrepoem(k));
        };
        if !(p.simples && q.simples) {
            return Err(ErroDeEdicaoDoPainter::SoPinceladasSeSobrepoem(k));
        }
        if p.meio != q.meio {
            return Err(ErroDeEdicaoDoPainter::OutroMeio(k));
        }
    }
    // O meio de uma etapa que vai para fora do sítio dela: o do pincel viaja com ela, o do Wet Paint é da
    // tela — o do instante novo tem de ser o da gravação.
    let gravado = super::etapas(&Track {
        edicoes: Vec::new(),
        ..track.clone()
    })
    .map_err(m)?
    .into_iter()
    .find(|g| g.k == k)
    .map(|g| g.meio);
    if eu.no_toque.is_some_and(|p| Some(p.meio) != gravado) {
        return Err(ErroDeEdicaoDoPainter::OutroMeio(k));
    }
    Ok(())
}
