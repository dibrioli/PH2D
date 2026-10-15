//! **O objecto fica com a `Macro` e o clipe** — o fim do Gravar ▸ Parar, e o que uma edição que muda
//! a duração ressincroniza.

use crate::Cena;
use ph2d_ecs::{Entity, Macro, ModoDaMacro};
use ph2d_macro::MacroDoc;
use ph2d_timeline::TimelineDoc;

/// O nome do clipe que o Parar cria (conteúdo do artista depois: ele renomeia-o).
pub const NOME_DO_CLIPE: &str = "app.macros.clip_name";

/// O clipe da macro de `dono`, se a animação dele já o tem.
#[must_use]
pub fn clipe_de(doc: &TimelineDoc, dono: u64) -> Option<usize> {
    doc.clip_named(dono, ph2d_i18n::tr(NOME_DO_CLIPE))
}

#[must_use]
pub fn segundos(doc: &MacroDoc) -> f64 {
    #[expect(clippy::cast_precision_loss, reason = "µs de uma macro cabem num f64")]
    let s = doc.duration().0 as f64 / 1e6;
    s
}

/// Grava `doc` no objecto `dono` (substitui a `Macro` que ele tinha) e cria ou re-cronometra o clipe.
///
/// # Errors
/// A chave da frase: o clipe não coube na animação.
pub fn instala(
    c: &mut Cena<'_>,
    dono: u64,
    doc: &MacroDoc,
    modo: ModoDaMacro,
) -> Result<usize, &'static str> {
    let bytes = ph2d_macro::codec::write(doc);
    let d = segundos(doc);
    if let Ok(mut e) = c.sim.world_mut().get_entity_mut(Entity::from_bits(dono)) {
        e.insert(Macro { doc: bytes, modo });
    }
    let tl = &mut *c.timeline;
    if let Some(clip) = clipe_de(tl, dono) {
        tl.retime_macro_clip(clip, dono, d);
        return Ok(clip);
    }
    let clip = tl
        .add_macro_clip(ph2d_i18n::tr(NOME_DO_CLIPE).to_owned(), dono, d)
        .map_err(ph2d_timeline::ClipRefusal::message_key)?;
    let antes = tl.active_index();
    tl.set_active(clip);
    tl.set_active_loop_for(true, Some((0.0, d)));
    tl.set_active(antes);
    tl.set_autoplay(clip, true);
    Ok(clip)
}
