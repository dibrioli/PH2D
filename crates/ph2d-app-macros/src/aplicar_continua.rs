//! **As continuações ao aplicar** (W19, `Track::continua`): o artista voltou a um objecto que uma
//! trilha anterior já gravou — o Motion não junta os nós outra vez; o Painter, por cima, começa onde a
//! anterior acabou sobre o alvo.

use super::{Entity, erro_com, fim, inicial, m, nao_toca, pm};
use super::{
    ErroDaMacroView, FamiliaDaMacro, Familias, MacroDoc, PaintedDocument, SimWorld, Track,
};

/// A continuação `k` do Motion: os nós da cadeia (`orig` como veio, `doc` aplicada) já estão no
/// objecto — a porta da família só reescreve os ids dos passos.
pub(super) fn continuacao_do_motion(
    orig: &MacroDoc,
    doc: &MacroDoc,
    k: usize,
    sid: u64,
    vida: u64,
    f: &mut Familias<'_>,
) -> Result<Track, ErroDaMacroView> {
    let o = orig.tracks[k].continua;
    let cadeia: Vec<(&Track, &Track)> = (0..k)
        .filter(|j| u32::try_from(*j).ok() == o || orig.tracks[*j].continua == o)
        .map(|j| (&orig.tracks[j], &doc.tracks[j]))
        .collect();
    let mut n = f.motion.continua(&orig.tracks[k], &cadeia, sid, vida)?;
    n.nasce.clone_from(&orig.tracks[k].nasce);
    n.continua = o;
    Ok(n)
}

/// Por cima: cada trilha do Painter tem o alvo como tela; uma continuação começa no FIM da anterior já
/// rebaseada (com o encaixe do objecto Macro `raiz` sobre o `alvo`), nunca na imagem de antes.
pub(super) fn rebaseia_o_painter(
    doc: &mut MacroDoc,
    fams: &[FamiliaDaMacro],
    base: &PaintedDocument,
    sid: u64,
    sim: &SimWorld,
    (raiz, alvo): (u64, u64),
) -> Result<(), ErroDaMacroView> {
    let pintor = |k: usize| fams[k] == FamiliaDaMacro::Painter;
    for k in (0..doc.tracks.len()).filter(|k| pintor(*k)) {
        let fim_da_anterior = match doc.tracks[k].continua {
            None => None,
            Some(o) => {
                let ultima = (0..k)
                    .rev()
                    .filter(|j| pintor(*j))
                    .find(|j| {
                        u32::try_from(*j).ok() == Some(o) || doc.tracks[*j].continua == Some(o)
                    })
                    .ok_or_else(|| erro_com(m::MACRO_APLICAR_NAO_NASCE, format!("#{o}")))?;
                let t = &doc.tracks[ultima];
                let encaixe = inicial::tamanho_gravado(&t.initial)
                    .and_then(|g| {
                        ph2d_sprite_screen::gravacao_no_alvo::gravacao_no_alvo(
                            sim.world(),
                            Entity::from_bits(raiz),
                            Entity::from_bits(alvo),
                            base.size,
                            g,
                        )
                    })
                    .map(|afim| pm::encaixe::Encaixe { afim });
                Some(
                    fim::documento_no_fim(t, encaixe)
                        .map_err(|e| nao_toca(&crate::erros::do_painter(&e)))?,
                )
            }
        };
        let t = &mut doc.tracks[k];
        t.initial = inicial::sobre_documento(&t.initial, fim_da_anterior.as_ref().unwrap_or(base))
            .map_err(|e| {
                nao_toca(&crate::erros::do_painter(&pm::tocar::ErroDaMacro::Inicial(
                    e,
                )))
            })?;
        t.target = sid;
        t.nasce = None;
    }
    Ok(())
}
