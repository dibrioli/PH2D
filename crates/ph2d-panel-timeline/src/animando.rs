//! ⭐ **«Animating · Done»** (doc 17 item 9, maquete B7, o idioma do Construct): na aba Keys dos
//! cartões, enquanto a Timeline mostra um clipe na cena, ir e vir com a agulha é só OLHAR — não entra
//! no desfazer, que guarda só as chaves. O **Done** devolve a cena como estava e pára a prévia até ao
//! próximo gesto na Timeline. A lei mora na shell e no ledger (`declare_timeline_writes_animating`);
//! aqui ficam a pílula, o contorno e as duas perguntas que a shell lê ([`animating`], [`parked`]).

use std::cell::Cell;

use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_icon, resolve, stroke_rounded_rect};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::panel::cards::banda::metrics as m;
use ph2d_editor_core::zones::Rect;
use ph2d_timeline::TimelineViewSnapshot;
use ph2d_tokens::{ColorToken, StrokeToken, Theme};

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

/// **O Done foi clicado desde a última pergunta?** Se sim, a agulha pára e volta ao início (dono
/// 11/11 e 12/11) e a shell desliga o Auto Key: acabou de animar, arrastar o objecto volta a ser mexer
/// na cena. ⚠️ Antes da [`sessao`]: a agulha a tocar é um gesto, e desfaria o Done no mesmo quadro.
pub fn take_done(agulha: &mut ph2d_core::Playhead) -> bool {
    let feito = FEITO.with(|c| c.replace(false));
    if feito {
        agulha.pause();
        agulha.rewind();
    }
    feito
}

/// O clique no ■ Stop (dono 13/11): pára, volta ao início e desliga o Auto Key como o Done, mas não
/// estaciona a sessão.
pub(crate) fn para() {
    FEITO.with(|c| c.set(true));
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

/// O Done redondo dentro da ponta direita do chip `chip`, com a folga de um segmento.
pub(crate) fn botao_done_no_chip(ctx: &mut PaintCtx, chip: Rect) {
    let p = m::SEG_PAD * 2.0;
    let d = chip.h - p * 2.0;
    let b = Rect::new(chip.x + chip.w - p - d, chip.y + p, d, d);
    botao_done(ctx, ctx.host.theme(), b, d * 0.5);
}

/// ⭐ **O Done é um botão VERDE com ✓, colado ao clipe** (dono 13/11): no fim do chip escolhido,
/// depois do «⋯», ou à direita do «⋯» da lista suspensa. `r` é a caixa do botão; o raio é o de quem o
/// leva (o chip é pílula, a lista é campo).
pub(crate) fn botao_done(ctx: &mut PaintCtx, theme: Theme, r: Rect, raio: f32) {
    let sob = ctx.host.store().hover_live(TIMELINE_ANIMATING_DONE) > 0.0;
    fill_rounded_rect(ctx.scene, r, raio, resolve(ColorToken::Success, theme));
    let fg = resolve(ColorToken::AccentFg, theme);
    if sob {
        // FRAME-RAW-OK: o anel claro É o realce do botão sob o rato.
        stroke_rounded_rect(ctx.scene, r, raio, StrokeToken::Default.px(), fg);
    }
    let g = m::ICON_GLYPH;
    paint_icon(
        ctx.scene,
        IconId::Check,
        Rect::new(r.x + (r.w - g) * 0.5, r.y + (r.h - g) * 0.5, g, g),
        fg,
        StrokeToken::Thick.px(),
    );
    ctx.host
        .hit_index_mut()
        .register(TIMELINE_ANIMATING_DONE, r);
    ph2d_editor_core::panel::cards::dicas::icone(TIMELINE_ANIMATING_DONE);
}

/// A moldura da fita fica AZUL enquanto se anima (B7) — por cima do vermelho do Auto Key, cujo
/// ponto na barra continua a dizer que grava.
pub(crate) fn cor_da_moldura(theme: Theme) -> Option<ph2d_vector::Color> {
    animating().then(|| resolve(ColorToken::Accent, theme))
}

#[cfg(test)]
#[path = "animando_tests.rs"]
mod tests;
