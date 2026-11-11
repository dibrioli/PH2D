//! ⭐ **«Animating · Done»** (doc 17 item 9, maquete B7, o idioma do Construct): na aba Keys dos
//! cartões, enquanto a Timeline mostra um clipe na cena, ir e vir com a agulha é só OLHAR — não entra
//! no desfazer, que guarda só as chaves. O **Done** devolve a cena como estava e pára a prévia até ao
//! próximo gesto na Timeline. A lei mora na shell e no ledger (`declare_timeline_writes_animating`);
//! aqui ficam a pílula, o contorno e as duas perguntas que a shell lê ([`animating`], [`parked`]).

use std::cell::Cell;

use ph2d_editor_core::paint::{
    fill_rounded_rect, fill_rounded_rect_radii, folga_do_texto, label_column_width,
    paint_text_elided, resolve, stroke_rounded_rect,
};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::panel::cards::banda::metrics as m;
use ph2d_editor_core::zones::Rect;
use ph2d_timeline::TimelineViewSnapshot;
use ph2d_tokens::{ColorToken, Spacing, StrokeToken, Theme, TypeToken};

use crate::ids::TIMELINE_ANIMATING_DONE;

thread_local! {
    /// Cartões e aba Keys visível: onde uma sessão PODE existir (o `paint` publica).
    static ANIMAVEL: Cell<bool> = const { Cell::new(false) };
    static PARADO: Cell<bool> = const { Cell::new(false) };
    /// O que a última [`sessao`] respondeu, para ver a descida.
    static ANTES: Cell<bool> = const { Cell::new(false) };
    static ACABOU: Cell<bool> = const { Cell::new(false) };
    static FEITO: Cell<bool> = const { Cell::new(false) };
}

/// **A Timeline está a animar a cena?** Lido pela shell a cada quadro: enquanto `true`, o que as
/// curvas escrevem é pré-visualização segura (a agulha não grava passo); a descida para `false`
/// devolve a cena.
#[must_use]
pub fn animating() -> bool {
    ANIMAVEL.with(Cell::get) && !parked()
}

/// **O Done foi carregado e nada mexeu na Timeline desde então?** A shell não aplica o clipe à cena.
#[must_use]
pub fn parked() -> bool {
    PARADO.with(Cell::get)
}

/// A shell viu um gesto que volta a animar (tocar, ir a um instante): a prévia volta.
pub fn resume() {
    PARADO.with(|c| c.set(false));
}

/// ⭐ **A porta da shell, uma vez por quadro, antes de aplicar a Timeline:** `(a animar, parada)`
/// na vista `keys` da shell. `gesto` = a agulha toca ou alguém pediu um instante: depois do Done,
/// isso volta a animar. A descida da sessão fica para [`take_ended`].
pub fn sessao(keys: bool, gesto: bool) -> (bool, bool) {
    if gesto && parked() {
        resume();
    }
    let anim = keys && animating();
    if ANTES.with(|a| a.replace(anim)) && !anim {
        ACABOU.with(|c| c.set(true));
    }
    (anim, keys && parked())
}

/// **A sessão acabou desde a última pergunta** (Done, outra aba, painel escondido, o jogo): a shell
/// devolve a cena à base do undo, que é a pose de entrada.
pub fn take_ended() -> bool {
    ACABOU.with(|c| c.replace(false))
}

/// **O Done foi clicado desde a última pergunta?** A shell desliga o Auto Key (dono 11/11): acabou de
/// animar, arrastar o objecto volta a ser mexer na cena, não gravar chave.
pub fn take_done() -> bool {
    FEITO.with(|c| c.replace(false))
}

/// O clique no Done.
pub(crate) fn done() {
    PARADO.with(|c| c.set(true));
    FEITO.with(|c| c.set(true));
}

/// Publicado pelo `paint`: só nos cartões, só na aba Keys. Fora dela não há sessão nem paragem.
pub(crate) fn publica(keys: bool) {
    let cartoes = ph2d_editor_core::panel::cards::ferramenta::cards_on();
    if !(cartoes && keys) {
        PARADO.with(|c| c.set(false));
    }
    ANIMAVEL.with(|c| c.set(cartoes && keys));
}

/// O painel escondido não anima nada.
pub(crate) fn larga() {
    PARADO.with(|c| c.set(false));
    ANIMAVEL.with(|c| c.set(false));
}

/// O nome do clipe em edição, se a pílula se pinta neste quadro.
pub(crate) fn rotulo(snap: &TimelineViewSnapshot) -> Option<String> {
    if !animating() {
        return None;
    }
    let linha = snap
        .clip_index
        .iter()
        .position(|&i| i == snap.active_clip)?;
    let clip = snap.clips.get(linha)?;
    Some(ph2d_i18n::tr("panel.timeline.animating").replace("{clip}", clip))
}

fn medidas() -> (f32, f32, f32) {
    (TypeToken::Sm.px(), Spacing::Sm.px(), Spacing::Xs.px())
}

/// A largura do segmento «Done», encostado à direita da pílula, de cima a baixo.
fn done_w(ctx: &mut PaintCtx, done: &str) -> f32 {
    let (font, pad, _) = medidas();
    pad + label_column_width(ctx.text_system, font, [done]) + pad
}

/// A largura da pílula «● Animating jump [Done]».
pub(crate) fn pilula_w(ctx: &mut PaintCtx, texto: &str) -> f32 {
    let (font, pad, gap) = medidas();
    let done = ph2d_i18n::tr("panel.timeline.animating.done");
    let tw = label_column_width(ctx.text_system, font, [texto]);
    pad + gap * 2.0 + gap + tw + gap + done_w(ctx, done)
}

/// Pinta a pílula em `r` (o raio dos campos do cartão, B7) e regista o Done: o segmento cheio
/// encostado à direita, da altura toda, redondo só do lado de fora.
pub(crate) fn pilula(ctx: &mut PaintCtx, theme: Theme, r: Rect, texto: &str) {
    let (font, pad, gap) = medidas();
    let done = ph2d_i18n::tr("panel.timeline.animating.done");
    let azul = resolve(ColorToken::Accent, theme);
    let raio = m::FIELD_RADIUS;
    fill_rounded_rect(ctx.scene, r, raio, resolve(ColorToken::AccentSoft, theme));
    let ponto = gap * 2.0;
    let cy = r.y + r.h * 0.5;
    let bolinha = Rect::new(r.x + pad, cy - ponto * 0.5, ponto, ponto);
    fill_rounded_rect(ctx.scene, bolinha, ponto * 0.5, azul);
    let dw = done_w(ctx, done);
    let botao = Rect::new(r.x + r.w - dw, r.y, dw, r.h);
    let tx = bolinha.x + ponto + gap;
    let tw = label_column_width(ctx.text_system, font, [texto]).min((botao.x - tx).max(0.0));
    let ty = r.y + folga_do_texto(r.h, font);
    let claro = resolve(ColorToken::Text1, theme);
    paint_text_elided(ctx.text_system, ctx.scene, texto, tx, ty, font, tw, claro);
    let sob = ctx.host.store().hover_live(TIMELINE_ANIMATING_DONE) > 0.0;
    let fundo = if sob {
        ColorToken::AccentHover
    } else {
        ColorToken::Accent
    };
    let raios = (0.0, raio, raio, 0.0);
    fill_rounded_rect_radii(ctx.scene, botao, raios, resolve(fundo, theme));
    stroke_rounded_rect(ctx.scene, r, raio, StrokeToken::Thin.px(), azul);
    let fg = resolve(ColorToken::AccentFg, theme);
    let texto_done = botao.w - pad * 2.0;
    let dy = botao.y + folga_do_texto(botao.h, font);
    paint_text_elided(
        ctx.text_system,
        ctx.scene,
        done,
        botao.x + pad,
        dy,
        font,
        texto_done,
        fg,
    );
    ctx.host
        .hit_index_mut()
        .register(TIMELINE_ANIMATING_DONE, botao);
}

/// A moldura da fita fica AZUL enquanto se anima (B7) — por cima do vermelho do Auto Key, cujo
/// ponto na barra continua a dizer que grava.
pub(crate) fn cor_da_moldura(theme: Theme) -> Option<ph2d_vector::Color> {
    animating().then(|| resolve(ColorToken::Accent, theme))
}

#[cfg(test)]
#[path = "animando_tests.rs"]
mod tests;
