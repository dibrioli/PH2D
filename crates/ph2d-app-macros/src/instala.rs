//! **O objecto fica com a `Macro` e o clipe** — o fim do Parar, e a ressincronia de uma edição.

use crate::Cena;
use crate::etapas::Injectadas;
use ph2d_ecs::{Entity, Macro, ModoDaMacro};
use ph2d_macro::MacroDoc;
use ph2d_timeline::{PropKind, TimelineDoc, TimelineIntent, TimelineState};

/// O nome do clipe que o Parar cria (conteúdo do artista depois: ele renomeia-o).
pub const NOME_DO_CLIPE: &str = "app.macros.clip_name";

/// O clipe da macro de `dono`, se a animação dele já o tem.
#[must_use]
pub fn clipe_de(doc: &TimelineDoc, dono: u64) -> Option<usize> {
    doc.clip_named(dono, ph2d_i18n::tr(NOME_DO_CLIPE))
}

/// **A duração da macro QUE TOCA** (s). ⚠️ Não é `MacroDoc::duration` (a gravação imutável): uma
/// espera editada muda o que toca, e o clipe acaba onde a reprodução acaba.
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
/// # Errors
/// A chave da frase: o clipe não coube na animação.
pub fn instala(
    c: &mut Cena<'_>,
    outras: Injectadas<'_>,
    dono: u64,
    doc: &MacroDoc,
    modo: ModoDaMacro,
) -> Result<usize, &'static str> {
    // O clipe é UM passo da pilha da Timeline; a `Macro` entra no diff do mundo. O clipe fica ACTIVO.
    c.timeline.history.begin(&c.timeline.doc);
    let r = instala_nascido(c, outras, dono, doc, modo);
    c.timeline.history.commit_if_changed(&c.timeline.doc);
    r
}

/// [`instala`] num objecto Macro nascido NESTE gesto: o clipe sem passo próprio (vive e morre com o
/// dono); o aplicar inteiro é UM Ctrl+Z.
/// # Errors
/// A chave da frase: o clipe não coube na animação.
pub fn instala_nascido(
    c: &mut Cena<'_>,
    outras: Injectadas<'_>,
    dono: u64,
    doc: &MacroDoc,
    modo: ModoDaMacro,
) -> Result<usize, &'static str> {
    let d = segundos(doc, outras);
    if let Ok(mut e) = c.sim.world_mut().get_entity_mut(Entity::from_bits(dono)) {
        e.insert(Macro {
            doc: ph2d_macro::codec::write(doc),
            modo,
        });
    }
    let r = clipe(&mut c.timeline.doc, dono, d);
    rearma_o_laco(c.timeline, c.intents);
    r
}

/// **O laço que TOCA re-arma-se**: a cópia viva no relógio só se arma por intenção (`SetLoop`, a porta
/// da régua) com o laço do documento. É o da aba Keys (na Arrange, `sync_transport_loop`).
pub(crate) fn rearma_o_laco(tl: &TimelineState, intents: &mut Vec<TimelineIntent>) {
    if !tl.keys_mode {
        return;
    }
    intents.push(TimelineIntent::SetLoop {
        range: tl.doc.active_loop_for(true),
        ping_pong: tl.doc.active_ping_pong_for(true),
    });
}

fn clipe(tl: &mut TimelineDoc, dono: u64, d: f64) -> Result<usize, &'static str> {
    if let Some(clip) = clipe_de(tl, dono) {
        tl.retime_macro_clip(clip, dono, d);
        tl.set_active(clip);
        entra_se_ninguem_entra(tl, dono, clip);
        return Ok(clip);
    }
    let clip = tl
        .add_macro_clip(ph2d_i18n::tr(NOME_DO_CLIPE).to_owned(), dono, d)
        .map_err(ph2d_timeline::ClipRefusal::message_key)?;
    tl.set_active(clip);
    tl.set_active_loop_for(true, Some((0.0, d)));
    entra_se_ninguem_entra(tl, dono, clip);
    Ok(clip)
}

/// O clipe de uma cena de smoke: activo, laço `[0, d]`, Autoplay salvo [`crate::a_cena_abre_parada`]
/// (`false` = não coube).
pub fn clipe_da_cena(tl: &mut TimelineDoc, dono: u64, d: f64, parada: bool) -> bool {
    let Ok(clip) = tl.add_macro_clip(ph2d_i18n::tr(NOME_DO_CLIPE).to_owned(), dono, d) else {
        return false;
    };
    tl.set_active(clip);
    tl.set_active_loop_for(true, Some((0.0, d)));
    tl.set_autoplay(clip, !parada);
    true
}

/// **O Autoplay do Parar não rouba a entrada do artista**: o clipe da macro só é a entrada se nenhum
/// OUTRO clipe do objecto o é; um que toca a macro (tomada antiga) não é escolha do artista e cede.
fn entra_se_ninguem_entra(tl: &mut TimelineDoc, dono: u64, clip: usize) {
    if tl
        .entry_clip_of(dono)
        .is_none_or(|e| e == clip || toca_a_macro(tl, e, dono))
    {
        tl.set_autoplay(clip, true);
    }
}

/// O clipe `c` tem a faixa do relógio da macro de `dono`.
fn toca_a_macro(tl: &TimelineDoc, c: usize, dono: u64) -> bool {
    tl.binding_for(dono, PropKind::MacroTime).is_some_and(|b| {
        tl.clips()
            .get(c)
            .is_some_and(|nc| nc.clip.track(b.target).is_some())
    })
}

/// As durações que tocam, por objecto — refeitas só quando os bytes da `Macro` mudam.
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
