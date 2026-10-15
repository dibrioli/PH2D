//! **O objecto fica com a `Macro` e o clipe** — o fim do Gravar ▸ Parar, e o que uma edição que muda
//! a duração ressincroniza.

use crate::Cena;
use crate::etapas::Injectadas;
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

/// **A duração da macro QUE TOCA**, em segundos: o fim da trilha efectiva mais longa. ⚠️ Não é o
/// `MacroDoc::duration`, que lê a gravação imutável — uma espera editada muda o que toca, não o que
/// se gravou, e o clipe tem de acabar onde a reprodução acaba.
#[must_use]
pub fn segundos(doc: &MacroDoc, outras: Injectadas<'_>) -> f64 {
    let fim = doc
        .tracks
        .iter()
        .map(|t| {
            crate::etapas::leitor(outras, t)
                .and_then(|l| t.tocavel(l).ok())
                .map_or_else(
                    || t.events.last().map_or(0, |e| e.t.0),
                    |e| e.events.last().map_or(0, |e| e.t.0),
                )
        })
        .max()
        .unwrap_or(0);
    #[expect(clippy::cast_precision_loss, reason = "µs de uma macro cabem num f64")]
    let s = fim as f64 / 1e6;
    s
}

/// Grava `doc` no objecto `dono` (substitui a `Macro` que ele tinha) e cria ou re-cronometra o clipe.
///
/// # Errors
/// A chave da frase: o clipe não coube na animação.
pub fn instala(
    c: &mut Cena<'_>,
    outras: Injectadas<'_>,
    dono: u64,
    doc: &MacroDoc,
    modo: ModoDaMacro,
) -> Result<usize, &'static str> {
    let bytes = ph2d_macro::codec::write(doc);
    let d = segundos(doc, outras);
    if let Ok(mut e) = c.sim.world_mut().get_entity_mut(Entity::from_bits(dono)) {
        e.insert(Macro { doc: bytes, modo });
    }
    // Criar ou re-cronometrar o clipe é UM passo da pilha da Timeline (o undo dos clipes é o dela);
    // a `Macro` entra no diff do mundo. O clipe da macro fica ACTIVO: a linha «Macro steps» mostra-se.
    let st = &mut *c.timeline;
    st.history.begin(&st.doc);
    let r = clipe(&mut st.doc, dono, d);
    st.history.commit_if_changed(&st.doc);
    r
}

fn clipe(tl: &mut TimelineDoc, dono: u64, d: f64) -> Result<usize, &'static str> {
    if let Some(clip) = clipe_de(tl, dono) {
        tl.retime_macro_clip(clip, dono, d);
        tl.set_active(clip);
        return Ok(clip);
    }
    let clip = tl
        .add_macro_clip(ph2d_i18n::tr(NOME_DO_CLIPE).to_owned(), dono, d)
        .map_err(ph2d_timeline::ClipRefusal::message_key)?;
    tl.set_active(clip);
    tl.set_active_loop_for(true, Some((0.0, d)));
    tl.set_autoplay(clip, true);
    Ok(clip)
}

/// As durações que tocam, por objecto — refeitas só quando os bytes da `Macro` mudam (a sincronização
/// do clipe corre a cada quadro).
#[derive(Default)]
pub struct Duracoes(std::collections::BTreeMap<u64, (Vec<u8>, f64)>);

impl Duracoes {
    pub fn de(&mut self, bits: u64, bytes: &[u8], outras: Injectadas<'_>) -> f64 {
        if let Some((b, d)) = self.0.get(&bits)
            && b == bytes
        {
            return *d;
        }
        let d = ph2d_macro::codec::read(bytes).map_or(0.0, |doc| segundos(&doc, outras));
        self.0.insert(bits, (bytes.to_vec(), d));
        d
    }
}
