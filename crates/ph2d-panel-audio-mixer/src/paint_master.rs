//! Audio Mixer master-section footer painter — Play Test · loudness (LUFS) ·
//! Limiter, then the collapsible master-effect groups (EQ · Reverb · Delay ·
//! Comp · Ducking). Split out of `paint.rs` to keep it under the panel LOC cap.
//!
//! Each effect group is a canonical collapsible [`SectionHeader`]: its id is
//! `mark_collapsible_section`-registered in `populate`, so the dispatch folds it
//! on click (no `apply_event` arm needed); the body paints only when open.

use crate::paint::MUTE_H;
use crate::paint_widgets::{paint_labeled_slider, paint_toggle};
use crate::{
    AMIX_DELAY, AMIX_DELAY_FEEDBACK, AMIX_DELAY_MIX, AMIX_DELAY_TIME, AMIX_DUCK, AMIX_DUCK_DEPTH,
    AMIX_DUCK_KEY_BUS, AMIX_EQ_HIGH, AMIX_EQ_LOW, AMIX_EQ_MID, AMIX_LIMITER, AMIX_PLAY,
    AMIX_REVERB, AMIX_REVERB_MIX, AMIX_REVERB_SIZE, AMIX_SEC_COMP, AMIX_SEC_DELAY, AMIX_SEC_DUCK,
    AMIX_SEC_EQ, AMIX_SEC_REVERB, SUB_BUS_COUNT, SUB_BUS_LABELS, SUB_COMP, SUB_DELAY_SEND,
    SUB_SEND, snapshot,
};
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::paint::{paint_text_centered, resolve};
use ph2d_editor_core::widget::{SectionFold, SectionHeader, paint_section_header};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{TextKey, tr};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

const LUFS_SILENCE_DISPLAY: f32 = -70.0; // LITERAL-PX-OK: below this the loudness reads "-inf" (audio domain)

/// The shared paint context threaded through the footer section painters —
/// bundles the borrows so each section fn takes just `(&mut Ctx, y)`.
struct Ctx<'a> {
    scene: &'a mut VectorScene,
    text_system: &'a mut TextSystem,
    hit_index: &'a mut HitIndex,
    store: &'a WidgetStore,
    theme: Theme,
    /// Content left edge + width (the panel's padded inner column).
    x: f32,
    w: f32,
    /// ⭐ **A coluna dos nomes das barras, MEDIDA uma vez** — ver
    /// [`crate::paint_widgets::coluna_dos_nomes`]. Ela vive aqui e não em cada secção porque o
    /// bloco de efeitos do master lê-se como UMA pilha: *as labels alinhadas todas à direita*
    /// (ordem do dono) pára de ser verdade no dia em que cada secção medir a sua.
    col: f32,
    /// ⭐ **A coluna do nome das CAIXAS DE MARCAR e da escolha do barramento-chave, medida uma vez
    /// sobre os NOMES DELAS** — os quatro liga/desliga do master (*Limiter* · *Reverb* · *Delay* ·
    /// *Ducking*) e o *Key*. ⚠️ Não é a `col` das barras: as barras são a CAIXA ÚNICA da casa (o
    /// nome vive DENTRO da caixa), logo não há coluna de nome com quem alinhar — e a coluna de
    /// omissão (metade da linha) cortava o nome no degrau estreito, que é o que a Física pagou no
    /// mesmo dia com o *Show Colliders*.
    caixas: ph2d_editor_core::widget::Seccao,
}

/// The master-section footer below the strips, top-down: Play Test · loudness ·
/// Limiter · EQ · Reverb · Delay · Comp · Ducking. Returns the final `y` (the
/// bottom of the painted content) so the caller can size the scroll region.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_master_section(
    y0: f32,
    content_x: f32,
    content_w: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
) -> f32 {
    // ⚠️ A coluna mede-se ANTES do `Ctx` nascer: ela precisa do sistema de texto, e o `Ctx`
    //    toma-o emprestado por inteiro.
    let col = crate::paint_widgets::coluna_dos_nomes(text_system, content_w);
    let caixas = ph2d_editor_core::widget::Seccao::medida(
        text_system,
        1,
        &[
            tr("panel.audio_mixer.master.limiter"),
            tr("panel.audio_mixer.master.reverb"),
            tr("panel.audio_mixer.master.delay"),
            tr("panel.audio_mixer.master.ducking"),
            tr("panel.audio_mixer.master.key"),
        ],
    );
    let mut ctx = Ctx {
        scene,
        text_system,
        hit_index,
        store,
        theme,
        x: content_x,
        w: content_w,
        col,
        caixas,
    };
    let mut y = y0;
    y = paint_play_test(&mut ctx, y);
    y = paint_loudness(&mut ctx, y);
    y = paint_limiter(&mut ctx, y);
    y = paint_eq(&mut ctx, y);
    y = paint_reverb(&mut ctx, y);
    y = paint_delay(&mut ctx, y);
    y = paint_comp(&mut ctx, y);
    paint_ducking(&mut ctx, y)
}

/// ⭐ **Um liga/desliga de efeito do master, pela porta da casa**
/// ([`ph2d_editor_core::property_row::paint_check_row`]) — o nome na coluna das caixas e a marca
/// na do valor. Ordem do dono (2026-09-24, *«siga»* depois da Física): eram botões acesos a toda
/// a largura. ⚠️ O valor é o `bool` do RETRATO. Devolve o `y` seguinte.
fn check_row(ctx: &mut Ctx, y: f32, label: &str, on: bool, id: NodeId) -> f32 {
    ph2d_editor_core::property_row::paint_check_row(
        ctx.scene,
        ctx.text_system,
        ctx.theme,
        ctx.hit_index,
        ctx.store,
        ctx.x,
        ctx.w,
        y,
        (id, label, on),
        ctx.caixas,
    )
}

/// A labeled thin-slider row; returns the next `y`.
fn slider_row(ctx: &mut Ctx, y: f32, label: &str, id: NodeId, value: f32) -> f32 {
    paint_labeled_slider(
        y,
        ctx.col,
        label,
        id,
        value,
        ctx.x,
        ctx.w,
        ctx.scene,
        ctx.text_system,
        ctx.theme,
        ctx.store,
        ctx.hit_index,
    )
}

/// Paint a collapsible section header (chevron + uppercase label). Returns
/// `(open, next_y)`; the dispatch flips `is_collapsed` on click.
/// ⚠️⚠️ **O rótulo entra TIPADO (`TextKey`), e isso é a cerca.**
///
/// Até 2026-09-18 era um `&str` e o cabeçalho da secção `EQ` estava escrito **cru** ao lado de dois
/// irmãos que já vinham da tabela — *um estranho numa lista de chaves*. O censo desta crate ficava
/// VERDE porque a régua só conta uma palavra GRITADA a partir de TRÊS letras (senão `UV` e `RGBA16`
/// seriam língua). ⇒ com este parâmetro, escrever `"EQ"` aqui deixa de compilar.
fn section_header(ctx: &mut Ctx, y: f32, id: NodeId, label: TextKey) -> (Option<SectionFold>, f32) {
    let open = !ctx.store.is_collapsed(id);
    let rect = Rect::new(ctx.x, y, ctx.w, MUTE_H);
    let header = SectionHeader::new(id, label.tr())
        .collapsible(open)
        .open_t(ctx.store.section_open_live(id));
    paint_section_header(&header, rect, ctx.scene, ctx.text_system, ctx.theme);
    ctx.hit_index.register(id, rect);
    let body_top = y + MUTE_H + Spacing::Sm.px();
    let fold = SectionFold::begin(
        ctx.store,
        id,
        ctx.x,
        ctx.w,
        body_top,
        ctx.scene,
        ctx.hit_index,
    );
    (fold, body_top)
}

/// Fecha a dobra aberta pelo [`section_header`] e devolve o `y` de saida.
fn end_fold(ctx: &mut Ctx, fold: SectionFold, y: f32) -> f32 {
    fold.finish(ctx.store, ctx.scene, ctx.hit_index, y)
}

/// Per-sub-bus labeled rows (used by the Reverb/Delay sends + Comp knobs).
fn sub_bus_rows(
    ctx: &mut Ctx,
    mut y: f32,
    ids: &[NodeId; SUB_BUS_COUNT],
    vals: [f32; SUB_BUS_COUNT],
) -> f32 {
    for i in 0..SUB_BUS_COUNT {
        y = slider_row(ctx, y, SUB_BUS_LABELS[i].tr(), ids[i], vals[i]);
    }
    y
}

fn paint_play_test(ctx: &mut Ctx, y: f32) -> f32 {
    // Play Test first — the primary "make sound" control stays reachable.
    //
    // ⭐ É uma ACÇÃO (liga e desliga o sinal de teste), e por isso fica botão — mas pela porta da
    //    casa ([`ph2d_editor_core::property_row::caixa_do_botao`]): na coluna do valor quando o
    //    rótulo cabe, a toda a largura quando não. O tom aceso fica: ele é o que diz «a tocar».
    let playing = snapshot::play_test();
    let label = if playing {
        tr("panel.audio_mixer.master.stop")
    } else {
        tr("panel.audio_mixer.master.play_test")
    };
    let rect =
        ph2d_editor_core::property_row::caixa_do_botao(ctx.text_system, ctx.x, ctx.w, y, label);
    paint_toggle(
        rect,
        label,
        playing,
        ColorToken::Accent,
        AMIX_PLAY,
        ph2d_editor_core::widget::GroupCell {
            col: ph2d_editor_core::widget::GroupPos::Only,
            row: ph2d_editor_core::widget::GroupPos::Only,
        },
        ctx.scene,
        ctx.text_system,
        ctx.theme,
        ctx.store,
        ctx.hit_index,
    );
    ph2d_editor_core::property_row::abaixo_do_botao(rect)
}

fn paint_loudness(ctx: &mut Ctx, y: f32) -> f32 {
    // Momentary loudness (LUFS, BS.1770); "-inf" when effectively silent.
    let lufs = snapshot::loudness();
    let text = if lufs <= LUFS_SILENCE_DISPLAY {
        tr("panel.audio_mixer.master.inf_lufs").to_string()
    } else {
        ph2d_i18n::tr_with(
            "panel.audio_mixer.master.lufs_value",
            &[("lufs", &format!("{lufs:.1}"))],
        )
    };
    paint_text_centered(
        ctx.text_system,
        ctx.scene,
        &text,
        Rect::new(ctx.x, y, ctx.w, TypeToken::Xs.px()),
        TypeToken::Xs.px(),
        resolve(ColorToken::Text2, ctx.theme),
    );
    y + TypeToken::Xs.px() + ph2d_tokens::control_gap_px()
}

fn paint_limiter(ctx: &mut Ctx, y: f32) -> f32 {
    // Master output limiter — tames peaks below the clip ceiling.
    //
    // ⚠️ Havia um `+ Spacing::Sm` escrito à mão depois dele (o vão que um botão aceso a toda a
    //    largura pedia para não colar no cabeçalho do EQ). Uma linha de marcar já avança o passo da
    //    casa (`row_pitch_px`), como todas as outras — e o vão à mão saiu (2026-09-24), com a
    //    altura de abertura MEDIDA na catraca.
    check_row(
        ctx,
        y,
        tr("panel.audio_mixer.master.limiter"),
        snapshot::limiter(),
        AMIX_LIMITER,
    )
}

fn paint_eq(ctx: &mut Ctx, y: f32) -> f32 {
    let (fold, mut y) = section_header(
        ctx,
        y,
        AMIX_SEC_EQ,
        TextKey::new("panel.audio_mixer.master.eq"),
    );
    if let Some(fold) = fold {
        let eq = snapshot::eq();
        y = slider_row(
            ctx,
            y,
            tr("panel.audio_mixer.master.low"),
            AMIX_EQ_LOW,
            eq[0],
        );
        y = slider_row(
            ctx,
            y,
            tr("panel.audio_mixer.master.mid"),
            AMIX_EQ_MID,
            eq[1],
        );
        y = slider_row(
            ctx,
            y,
            tr("panel.audio_mixer.master.high"),
            AMIX_EQ_HIGH,
            eq[2],
        );
        y = end_fold(ctx, fold, y);
    }
    y + ph2d_tokens::control_gap_px()
}

fn paint_reverb(ctx: &mut Ctx, y: f32) -> f32 {
    let (fold, mut y) = section_header(
        ctx,
        y,
        AMIX_SEC_REVERB,
        TextKey::new("panel.audio_mixer.master.reverb"),
    );
    if let Some(fold) = fold {
        y = check_row(
            ctx,
            y,
            tr("panel.audio_mixer.master.reverb"),
            snapshot::reverb_on(),
            AMIX_REVERB,
        );
        y = slider_row(
            ctx,
            y,
            tr("panel.audio_mixer.master.size"),
            AMIX_REVERB_SIZE,
            snapshot::reverb_size(),
        );
        y = slider_row(
            ctx,
            y,
            tr("panel.audio_mixer.master.return"),
            AMIX_REVERB_MIX,
            snapshot::reverb_mix(),
        );
        y = sub_bus_rows(ctx, y, &SUB_SEND, snapshot::sub_send());
        y = end_fold(ctx, fold, y);
    }
    y + ph2d_tokens::control_gap_px()
}

fn paint_delay(ctx: &mut Ctx, y: f32) -> f32 {
    let (fold, mut y) = section_header(
        ctx,
        y,
        AMIX_SEC_DELAY,
        TextKey::new("panel.audio_mixer.master.delay"),
    );
    if let Some(fold) = fold {
        y = check_row(
            ctx,
            y,
            tr("panel.audio_mixer.master.delay"),
            snapshot::delay_on(),
            AMIX_DELAY,
        );
        y = slider_row(
            ctx,
            y,
            tr("panel.audio_mixer.master.time"),
            AMIX_DELAY_TIME,
            snapshot::delay_time(),
        );
        y = slider_row(
            ctx,
            y,
            tr("panel.audio_mixer.master.fbk"),
            AMIX_DELAY_FEEDBACK,
            snapshot::delay_feedback(),
        );
        y = slider_row(
            ctx,
            y,
            tr("panel.audio_mixer.master.return"),
            AMIX_DELAY_MIX,
            snapshot::delay_mix(),
        );
        y = sub_bus_rows(ctx, y, &SUB_DELAY_SEND, snapshot::sub_delay_send());
        y = end_fold(ctx, fold, y);
    }
    y + ph2d_tokens::control_gap_px()
}

fn paint_comp(ctx: &mut Ctx, y: f32) -> f32 {
    let (fold, mut y) = section_header(
        ctx,
        y,
        AMIX_SEC_COMP,
        TextKey::new("panel.audio_mixer.master.comp"),
    );
    if let Some(fold) = fold {
        y = sub_bus_rows(ctx, y, &SUB_COMP, snapshot::sub_comp());
        y = end_fold(ctx, fold, y);
    }
    y + ph2d_tokens::control_gap_px()
}

fn paint_ducking(ctx: &mut Ctx, y: f32) -> f32 {
    let (fold, mut y) = section_header(
        ctx,
        y,
        AMIX_SEC_DUCK,
        TextKey::new("panel.audio_mixer.master.ducking"),
    );
    if let Some(fold) = fold {
        y = check_row(
            ctx,
            y,
            tr("panel.audio_mixer.master.ducking"),
            snapshot::ducking(),
            AMIX_DUCK,
        );
        // ⭐ The sidechain key is a CHOICE of sub-bus, painted by the house choice door with the
        //    four buses in view (it used to be one button that CYCLED `Key: Music → SFX → …`).
        let key = snapshot::ducking_key();
        let rotulos: [&str; SUB_BUS_COUNT] = std::array::from_fn(|i| SUB_BUS_LABELS[i].tr());
        let segmentos: [(&str, bool, NodeId); SUB_BUS_COUNT] =
            std::array::from_fn(|i| (rotulos[i], i == key, AMIX_DUCK_KEY_BUS[i]));
        y = ph2d_editor_core::property_row::paint_choice_row(
            ctx.scene,
            ctx.text_system,
            ctx.theme,
            ctx.hit_index,
            ctx.store,
            ctx.x,
            ctx.w,
            y,
            tr("panel.audio_mixer.master.key"),
            &segmentos,
            ctx.caixas,
        );
        y = slider_row(
            ctx,
            y,
            tr("panel.audio_mixer.master.depth"),
            AMIX_DUCK_DEPTH,
            snapshot::duck_depth(),
        );
        y = end_fold(ctx, fold, y);
    }
    y + ph2d_tokens::control_gap_px()
}
