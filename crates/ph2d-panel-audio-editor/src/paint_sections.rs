//! The Audio Editor panel's **sections**: the collapsible blocks the body is made of,
//! and the chrome that separates them.
//!
//! The panel had grown into one unbroken column of controls. The app's canonical answer
//! is a [`SectionHeader`](ph2d_editor_core::widget::SectionHeader) (chevron + uppercase label, darker plate when folded) with
//! `paint_section_separator` — the 1 px **accent-coloured** rule — between blocks. Both
//! come from the Widget Gallery, which is the single source of truth for chrome
//! (DIRETRIZ §5.2); this file exists so the panel wears the app's clothes rather than
//! its own.
//!
//! Split out of `paint.rs` to keep that file (and its `paint` fn) under the panel LOC
//! caps.
//!
//! ⭐ Desde 2026-09-30 a fronteira entre dois blocos é a borda de um CARTÃO e quem a pinta é o
//! plano das secções ([`paint_body`]), que também dá a cada uma a PEGA de arrasto e o TEMA que o
//! artista lhe escolhe pelo botão direito no título.

use crate::paint::{ClippedHits, button_in_group, fmt_time, toggle_in_group};
use crate::{
    AEDIT_BATCH_LUFS, AEDIT_EXPORT, AEDIT_LOAD, AEDIT_LOOP, AEDIT_NAME, AEDIT_PLAY,
    AEDIT_SEC_DELIVERY, AEDIT_SEC_EDIT, AEDIT_SEC_FX, AEDIT_SEC_LOOP, AEDIT_SEC_MARKERS,
    AEDIT_SEC_SPECTRAL, AEDIT_SEC_TRANSPORT, AEDIT_SEC_VARIATIONS, AEDIT_STOP,
};
use ph2d_a11y::NodeId;
use ph2d_editor_core::paint::{paint_text, paint_text_centered, rect_to_vello, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::panel::section_plan_ctx::PlanoCtx;
use ph2d_editor_core::widget::section_cards::skip_section_header;
use ph2d_editor_core::widget::{
    SectionFold, TextInput, TextInputState, block_cells, grid_height, paint_section_header,
    paint_text_input_with_buffer,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

use crate::paint::ROW_H;

/// Everything the body needs for one frame, bundled so the walk fits one arg list.
pub(crate) struct Body {
    /// Fold state per section, in [`SECTIONS`] order: **o par** `(aberta?, t VIVO)`.
    ///
    /// ⚠️ **Um array de PARES e não dois arrays paralelos** — duas listas indexadas pela mesma
    /// posição são duas cópias do mesmo facto, e a segunda é a que alguém esquece de reordenar.
    pub open: [(bool, f32); 8],
    pub loaded: bool,
    pub undo_ok: bool,
    pub redo_ok: bool,
    pub has_sel: bool,
    pub transport: Transport,
    pub name: NameBox,
}

impl Body {
    /// O par que o cabeçalho veste: o estado semântico e o `t` VIVO da dobra.
    ///
    /// **By id, never by index**: `open[2]` means whatever the third entry of [`SECTIONS`]
    /// happens to be today, so reordering the panel would silently hand every section somebody
    /// else's fold state — a bug that paints perfectly and is invisible until a user clicks the
    /// wrong chevron.
    fn fold(&self, id: NodeId) -> (bool, f32) {
        SECTIONS
            .iter()
            .position(|s| *s == id)
            .map_or((false, 0.0), |i| self.open[i])
    }
}

#[path = "paint_sections_chrome.rs"]
mod chrome;
#[cfg(test)]
use chrome::section_h;
use chrome::{end_fold, section};

/// ⭐⭐⭐ **O corpo é uma LISTA** (ordem do dono, 2026-09-30: *«siga com os outros painéis»*,
/// depois de o menu de tema no título e a pega de dez pontos nascerem no Inspector e chegarem ao
/// Vector e ao Painter). As oito secções declaram-se pela ordem NATURAL de [`SECTIONS`] e o
/// [`PlanoCtx`] pinta-as pela ordem do ARTISTA, cada uma no TEMA que ele lhe deu pelo botão direito
/// no título — a lei da ordem, do tema, do corredor do cartão, da marca de queda e do fantasma é a
/// partilhada; aqui só mora QUEM é o quê.
///
/// ⚠️ **As oito ARRASTAM-SE, nenhuma é fixa, e isso é uma decisão:** nenhuma reinterpreta as que
/// estão abaixo dela (a razão por que a Máscara e o meio da tinta ficam presos no Painter). O
/// Transporte é o primeiro por omissão — é o que se toca em todas as passagens — mas o `Load` dele
/// não governa a pintura das outras: cada uma lê o `loaded` do retrato, esteja onde estiver.
///
/// ⚠️ **O separador entre secções saiu do passo e não se perdeu:** o [`PlanoCtx`] fecha o cartão
/// da anterior ANTES de cada secção que se segue a uma que pintou
/// ([`ph2d_editor_core::panel::section_plan::Corredor`]) — exactamente o que o `separator` fazia.
///
/// ⚠️ E o cartão da ÚLTIMA fecha-o o próprio plano — o `end_section_cards` de quem chama só pinta
/// os cartões que o livro FECHOU, e antes do plano isso deixava a última secção aberta sem cartão.
///
/// ⚠️ Cada tarefa constrói o seu [`ClippedHits`] a partir do contexto: o recorte do corpo rolado é
/// o mesmo `clip` para as oito, e o empréstimo store/hit não pode atravessar o laço do plano (ele
/// precisa do `ctx` inteiro entre duas secções, para pintar a marca de queda e o fantasma).
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_body(
    ctx: &mut PaintCtx<'_>,
    y: f32,
    x: f32,
    w: f32,
    b: &Body,
    clip: Rect,
    theme: Theme,
) -> f32 {
    let mut plano = PlanoCtx::new();
    for id in SECTIONS {
        plano.seccao(id, move |ctx, tema, y| {
            let (scene, text_system) = (&mut *ctx.scene, &mut *ctx.text_system);
            let (store, hits) = ctx.host.store_and_hit_index_mut();
            let hit_index = &mut ClippedHits::new(store, hits, clip);
            paint_one(id, y, x, w, b, scene, text_system, tema, hit_index)
        });
    }
    plano.corre(ctx, theme, x, w, chrome::section_h(), y)
}

/// **A mesma pilha pela ordem NATURAL, sem plano** — o corpo dos gates de altura e de duplicados
/// deste módulo, que não têm um `PaintCtx` à mão.
///
/// ⚠️ Não é um sucedâneo do produto: cada secção pinta-se pela MESMA [`paint_one`] e o corredor é
/// o MESMO [`ph2d_editor_core::panel::section_plan::Corredor`] que o [`PlanoCtx`] usa — só a ORDEM
/// fica a natural. Quem mede a ordem do artista, o tema e o arrasto é o gate de costura
/// `tests/it/as_seccoes_arrastam_e_tem_tema.rs`, pelo painel inteiro.
#[cfg(test)]
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_body_em_serie(
    y: f32,
    x: f32,
    w: f32,
    b: &Body,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut ClippedHits,
) -> f32 {
    ph2d_editor_core::widget::section_cards::with_section_cards(scene, theme, y, |scene| {
        let mut corredor = ph2d_editor_core::panel::section_plan::Corredor::default();
        let mut y = y;
        for id in SECTIONS {
            y = corredor.antes(scene, theme, x, w, y);
            let y0 = y;
            y = paint_one(id, y, x, w, b, scene, text_system, theme, hit_index);
            corredor.depois(y0, y);
        }
        y
    })
}

/// O rótulo e o resumo do cabeçalho da secção `id` — o resumo vive NO cabeçalho porque
/// "Variations" e "3 clips" são um facto só, e uma secção dobrada ainda tem de dizer o que guarda.
fn header_of(id: NodeId) -> (&'static str, Option<String>) {
    match id {
        AEDIT_SEC_LOOP => (
            tr("panel.audio_editor.transport.loop"),
            Some(crate::paint_loop::loop_readout()),
        ),
        AEDIT_SEC_EDIT => (tr("panel.audio_editor.transport.edit"), None),
        AEDIT_SEC_SPECTRAL => (
            tr("panel.audio_editor.transport.spectral"),
            Some(crate::paint_spectral::spectral_readout().to_string()),
        ),
        AEDIT_SEC_FX => (tr("panel.audio_editor.transport.effects"), None),
        AEDIT_SEC_MARKERS => (
            tr("panel.audio_editor.transport.markers"),
            Some(crate::paint_loop::markers_readout()),
        ),
        AEDIT_SEC_VARIATIONS => (
            tr("panel.audio_editor.transport.variations"),
            Some(crate::paint_variation::variation_readout()),
        ),
        AEDIT_SEC_DELIVERY => (
            tr("panel.audio_editor.transport.delivery"),
            Some(crate::paint_delivery::delivery_readout()),
        ),
        _ => (tr("panel.audio_editor.transport.transport"), None),
    }
}

/// **Uma secção inteira** — o cabeçalho (fora do cartão), e o bloco se ela está aberta. Devolve o
/// `y` debaixo do que pintou; o corredor até à seguinte é do chamador.
///
/// ⭐ **O que cada uma é, pela ordem natural:** *trabalhar o som* — o transporte, o loop (logo
/// debaixo dele: é uma coisa de AUDIÇÃO, Loop ligado + Play é como se ouve; Enio, 2026-07-12), a
/// edição, o espectral (entre a edição e os efeitos: É edição, destrutiva e desfazível, num
/// domínio que a onda não mostra) e o rack de efeitos; depois *preparar o asset* — marcadores,
/// variações, entrega —, que se tocam uma vez por asset e por isso nascem DOBRADAS.
#[allow(clippy::too_many_arguments)]
fn paint_one(
    id: NodeId,
    y: f32,
    x: f32,
    w: f32,
    b: &Body,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut ClippedHits,
) -> f32 {
    let (label, readout) = header_of(id);
    let (fold, y) = section(
        y,
        x,
        w,
        id,
        label,
        readout.as_deref(),
        b.fold(id),
        scene,
        text_system,
        theme,
        hit_index,
    );
    // ⚠️ O título fica FORA do cartão — o cursor do livro salta-o.
    skip_section_header(y);
    let Some(fold) = fold else {
        return y;
    };
    let (ts, th) = (text_system, theme);
    let y = match id {
        AEDIT_SEC_TRANSPORT => {
            paint_transport_section(y, x, w, b.transport, &b.name, scene, ts, th, hit_index)
        }
        AEDIT_SEC_LOOP => crate::paint_loop::paint_loop_section(
            y, x, w, b.loaded, b.has_sel, ROW_H, scene, ts, th, hit_index,
        ),
        AEDIT_SEC_EDIT => crate::paint_edit::paint_edit_section(
            y, x, w, b.loaded, b.undo_ok, b.redo_ok, b.has_sel, scene, ts, th, hit_index,
        ),
        AEDIT_SEC_SPECTRAL => crate::paint_spectral::paint_spectral_section(
            y, x, w, b.loaded, b.has_sel, ROW_H, scene, ts, th, hit_index,
        ),
        AEDIT_SEC_FX => {
            crate::paint_fx::paint_fx_section(y, x, w, b.loaded, ROW_H, scene, ts, th, hit_index)
        }
        AEDIT_SEC_MARKERS => crate::paint_loop::paint_markers_section(
            y, x, w, b.loaded, ROW_H, scene, ts, th, hit_index,
        ),
        AEDIT_SEC_VARIATIONS => crate::paint_variation::paint_variation_section(
            y, x, w, ROW_H, scene, ts, th, hit_index,
        ),
        AEDIT_SEC_DELIVERY => crate::paint_delivery::paint_delivery_section(
            y, x, w, b.loaded, ROW_H, scene, ts, th, hit_index,
        ),
        _ => y,
    };
    end_fold(fold, y, scene, hit_index)
}

/// Every collapsible section, in paint order. The fold state is read from the store as
/// one array before the paint borrows, so the body never has to reach back into it.
pub(crate) const SECTIONS: [NodeId; 8] = [
    AEDIT_SEC_TRANSPORT,
    AEDIT_SEC_LOOP,
    AEDIT_SEC_EDIT,
    AEDIT_SEC_SPECTRAL,
    AEDIT_SEC_FX,
    AEDIT_SEC_MARKERS,
    AEDIT_SEC_VARIATIONS,
    AEDIT_SEC_DELIVERY,
];

/// The shell's live transport readout, bundled so the section fits one arg list.
#[derive(Clone, Copy)]
pub(crate) struct Transport {
    pub loaded: bool,
    pub playing: bool,
    pub looping: bool,
    pub pos: f64,
    pub dur: f64,
}

/// The clip-name `TextInput`'s live buffer, cloned out of the store so the scene
/// borrow below is free of it.
pub(crate) struct NameBox {
    pub state: TextInputState,
    /// ⚠️ **Quanto do hover está presente.** Vem no snapshot e não é lido aqui do store porque
    /// este pintor não tem um — quem sabe é quem monta o `NameBox`.
    pub hover_t: f32,
    pub text: String,
    pub caret: usize,
    pub anchor: Option<usize>,
}

/// Clip name · position/duration readout · Play/Pause · Stop | Loop · Load | Export.
/// Returns the `y` below the block.
#[allow(clippy::too_many_arguments)]
fn paint_transport_section(
    mut y: f32,
    x: f32,
    w: f32,
    t: Transport,
    name: &NameBox,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut ClippedHits,
) -> f32 {
    // Clip name — an editable TextInput (mirror of the sprite name box). The widget
    // clips its own overflow to the field, so a long filename no longer wraps/crams
    // the header.
    let name_h = TypeToken::Sm.px() + Spacing::Sm.px() * 2.0;
    let name_rect = Rect::new(x, y, w, name_h);
    hit_index.register(AEDIT_NAME, name_rect);
    let input = TextInput::new(AEDIT_NAME, "")
        .placeholder(tr("panel.audio_editor.transport.no_clip_loaded"))
        .visual((name.state, name.hover_t));
    // Clip to the field: the TextInput lays its text out with word-wrap at the inner
    // width, so a long filename spills onto a 2nd line below the box. A clip to the
    // single-line box crops that overflow instead of letting it extrapolate.
    scene.push_clip(&rect_to_vello(name_rect));
    paint_text_input_with_buffer(
        &input,
        Some(name.text.as_str()),
        Some(name.caret),
        name.anchor,
        name_rect,
        scene,
        text_system,
        theme,
    );
    scene.pop_layer();
    y += name_h + ph2d_tokens::control_gap_px();

    // Position / duration readout.
    let time_line = format!("{} / {}", fmt_time(t.pos), fmt_time(t.dur));
    paint_text_centered(
        text_system,
        scene,
        &time_line,
        Rect::new(x, y, w, TypeToken::Xs.px()),
        TypeToken::Xs.px(),
        resolve(ColorToken::Text2, theme),
    );
    y += TypeToken::Xs.px() + ph2d_tokens::control_gap_px();

    // ⭐⭐ **As QUATRO fileiras do transporte são um corpo só** (`1 · 2 · 2 · 1`) — Enio,
    //    2026-09-06: *«na vertical ainda tem muito espaço ainda»*. Elas fazem a mesma coisa
    //    (comandar o clipe), logo encostam, e só os quatro cantos do BLOCO arredondam.
    let block = block_cells(Rect::new(x, y, w, 0.0), &[1, 2, 2, 1], ROW_H);
    // Transport: Play/Pause (full width toggle, active while playing).
    let play_label = if t.playing {
        tr("panel.audio_editor.transport.pause")
    } else {
        tr("panel.audio_editor.transport.play")
    };
    toggle_in_group(
        block[0][0].0,
        block[0][0].1,
        play_label,
        t.playing,
        t.loaded,
        AEDIT_PLAY,
        scene,
        text_system,
        theme,
        hit_index,
    );

    // Stop | Loop side by side.
    let seg = &block[1];
    button_in_group(
        seg[0].0,
        tr("panel.audio_editor.transport.stop"),
        t.loaded,
        AEDIT_STOP,
        seg[0].1,
        scene,
        text_system,
        theme,
        hit_index,
    );
    toggle_in_group(
        seg[1].0,
        seg[1].1,
        tr("panel.audio_editor.transport.loop"),
        t.looping,
        true,
        AEDIT_LOOP,
        scene,
        text_system,
        theme,
        hit_index,
    );

    // Load | Export WAV side by side.
    let seg = &block[2];
    button_in_group(
        seg[0].0,
        tr("panel.audio_editor.transport.load"),
        true,
        AEDIT_LOAD,
        seg[0].1,
        scene,
        text_system,
        theme,
        hit_index,
    );
    button_in_group(
        seg[1].0,
        tr("panel.audio_editor.transport.export_wav"),
        t.loaded,
        AEDIT_EXPORT,
        seg[1].1,
        scene,
        text_system,
        theme,
        hit_index,
    );

    // Batch LUFS — a FOLDER op (independent of the loaded clip), so always enabled.
    button_in_group(
        block[3][0].0,
        tr("panel.audio_editor.transport.batch_lufs"),
        true,
        AEDIT_BATCH_LUFS,
        block[3][0].1,
        scene,
        text_system,
        theme,
        hit_index,
    );
    y + grid_height(4, ROW_H) + ph2d_tokens::control_gap_px()
}

#[cfg(test)]
mod tests;
