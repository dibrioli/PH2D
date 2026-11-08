//! A janela do arranque **em cartões, dentro do app** (P12 S): um cartão por sessão recuperada —
//! nome, quando («Today 14:32»), tamanho, `Restore` · `Discard` — e o rodapé `Later` (fecha, as
//! cópias ficam). É modal: enquanto aberta, todo clique é dela ([`hit`] nunca devolve «passa») e o
//! `Esc` = `Later` ([`key_escape`]). A shell só compõe: pinta com [`paint`] por cima de tudo, e
//! manda o ponteiro e o `Esc` aqui antes de qualquer outro consumidor.

use ph2d_a11y::NodeId;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text, paint_text_block, resolve};
use ph2d_editor_core::widget::{Button, paint_button};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

use crate::{Choice, Orphan, Prompt};

/// A largura da janela, em módulos `Xl4` (9 × 48 = 432 px: nome + duas linhas + dois botões).
const DIALOG_COLS: f32 = 9.0; // LITERAL-PX-OK: a count of Xl4 modules, not pixels
/// A largura de um botão, em módulos `Xl3` (3 × 32 = 96 px, a do `Modal`).
const BUTTON_COLS: f32 = 3.0; // LITERAL-PX-OK: a count of Xl3 modules, not pixels

/// Onde cada peça da janela está neste quadro.
#[derive(Clone, Debug, PartialEq)]
pub struct Layout {
    /// A janela inteira (o cartão de fora).
    pub dialog: Rect,
    /// Um cartão por sessão que coube (as que não couberam aparecem depois de responder às de cima).
    pub rows: Vec<RowRects>,
    /// O botão `Later` do rodapé.
    pub later: Rect,
}

/// As peças de um cartão de sessão.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RowRects {
    /// O cartão.
    pub card: Rect,
    /// `Restore`.
    pub restore: Rect,
    /// `Discard`.
    pub discard: Rect,
}

/// O que um clique fez.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hit {
    /// Um botão de um cartão.
    Answer(usize, Choice),
    /// `Later`: fecha, as cópias ficam.
    Later,
    /// Na janela ou no véu, fora dos botões: comido (a janela é modal).
    Swallowed,
    /// Fecho: grava e fecha.
    Save,
    /// Fecho: fecha sem gravar.
    DontSave,
    /// Fecho: não fecha.
    Cancel,
}

/// A pergunta do fecho: o corpo e `Cancel` · `Don't Save` · `Save` no rodapé.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct QuitLayout {
    /// A janela.
    pub dialog: Rect,
    /// `Cancel`.
    pub cancel: Rect,
    /// `Don't Save`.
    pub dont_save: Rect,
    /// `Save`.
    pub save: Rect,
}

/// A disposição da pergunta do fecho de `r` (o corpo à altura medida no último quadro).
#[must_use]
pub fn quit_rects(r: &crate::Recovery, viewport: Rect) -> QuitLayout {
    quit_layout(viewport, r.quit_body_h.unwrap_or_else(row_h))
}

/// A largura do texto dentro da janela de largura `w`.
fn body_w(w: f32) -> f32 {
    (w - Spacing::Lg.px() * 2.0).max(0.0)
}

/// A disposição da pergunta do fecho, centrada em `viewport`, com o corpo de altura `body_h`.
#[must_use]
pub fn quit_layout(viewport: Rect, body_h: f32) -> QuitLayout {
    let (pad, gap) = (Spacing::Lg.px(), Spacing::Md.px());
    let w = (Spacing::Xl4.px() * DIALOG_COLS).min(viewport.w);
    let h = header_h() + body_h + footer_h();
    let dialog = Rect::new(
        viewport.x + (viewport.w - w) * 0.5,
        viewport.y + (viewport.h - h).max(0.0) * 0.5,
        w,
        h,
    );
    let (bw, bh) = button_size();
    let by = dialog.y + dialog.h - footer_h() + (footer_h() - bh) * 0.5;
    let save = Rect::new(dialog.x + dialog.w - pad - bw, by, bw, bh);
    let dont_save = Rect::new(save.x - gap - bw, by, bw, bh);
    let cancel = Rect::new(dont_save.x - gap - bw, by, bw, bh);
    QuitLayout {
        dialog,
        cancel,
        dont_save,
        save,
    }
}

/// O clique do ponteiro (píxeis FÍSICOS do ecrã) numa janela aberta, pela disposição do último
/// quadro pintado. `Later`, `Cancel` e `Don't Save` já se executam aqui; o resto devolve-se à shell. `None` = não
/// há janela aberta (o clique é do editor). A posição é a do último [`crate::Recovery::hover`]
/// (`x, y` só sem nenhum movimento visto).
pub fn click(r: &mut crate::Recovery, x: f32, y: f32) -> Option<Hit> {
    if !r.is_modal() {
        return None;
    }
    let (x, y) = r.cursor.unwrap_or((x, y));
    let Some((screen, map)) = r.screen else {
        return Some(Hit::Swallowed);
    };
    let vp = map.rect_to_logical(screen);
    let (lx, ly) = map.to_logical((x, y));
    let got = if r.prompt.is_open() {
        hit(&layout(vp, r.prompt.entries.len()), lx, ly)
    } else {
        let q = quit_rects(r, vp);
        if q.save.contains(lx, ly) {
            Hit::Save
        } else if q.dont_save.contains(lx, ly) {
            Hit::DontSave
        } else if q.cancel.contains(lx, ly) {
            Hit::Cancel
        } else {
            Hit::Swallowed
        }
    };
    match got {
        Hit::Later => r.prompt.later_all(),
        Hit::Cancel => r.quit = crate::Quit::Idle,
        // `Don't Save` = descartar também a cópia: o fecho normal apaga a sessão inteira.
        Hit::DontSave => r.quit = crate::Quit::Go,
        _ => {}
    }
    Some(got)
}

fn header_h() -> f32 {
    TypeToken::Xl3.px()
}

fn row_h() -> f32 {
    TypeToken::Md.px() + TypeToken::Sm.px() + Spacing::Md.px() * 3.0
}

fn footer_h() -> f32 {
    Spacing::Xl3.px() + Spacing::Lg.px() * 2.0
}

fn button_size() -> (f32, f32) {
    (Spacing::Xl3.px() * BUTTON_COLS, Spacing::Xl2.px())
}

/// A disposição para `n` sessões, centrada em `viewport`; só entram os cartões que cabem.
#[must_use]
pub fn layout(viewport: Rect, n: usize) -> Layout {
    let pad = Spacing::Lg.px();
    let gap = Spacing::Md.px();
    let w = (Spacing::Xl4.px() * DIALOG_COLS).min(viewport.w);
    let chrome = header_h() + footer_h() + pad;
    let room = ((viewport.h - chrome - pad * 2.0) / (row_h() + gap))
        .floor()
        .max(1.0);
    let shown = n.min(room as usize).max(1);
    let h = chrome + shown as f32 * (row_h() + gap);
    let dialog = Rect::new(
        viewport.x + (viewport.w - w) * 0.5,
        viewport.y + (viewport.h - h).max(0.0) * 0.5,
        w,
        h,
    );
    let (bw, bh) = button_size();
    let rows = (0..n.min(shown))
        .map(|i| {
            let card = Rect::new(
                dialog.x + pad,
                dialog.y + header_h() + i as f32 * (row_h() + gap),
                dialog.w - pad * 2.0,
                row_h(),
            );
            let by = card.y + (card.h - bh) * 0.5;
            let restore = Rect::new(card.x + card.w - Spacing::Md.px() - bw, by, bw, bh);
            let discard = Rect::new(restore.x - gap - bw, by, bw, bh);
            RowRects {
                card,
                restore,
                discard,
            }
        })
        .collect();
    let later = Rect::new(
        dialog.x + dialog.w - pad - bw,
        dialog.y + dialog.h - footer_h() + (footer_h() - bh) * 0.5,
        bw,
        bh,
    );
    Layout {
        dialog,
        rows,
        later,
    }
}

/// O clique em `(x, y)`. A janela é modal: tudo o que não é botão é [`Hit::Swallowed`].
#[must_use]
pub fn hit(l: &Layout, x: f32, y: f32) -> Hit {
    if l.later.contains(x, y) {
        return Hit::Later;
    }
    for (i, r) in l.rows.iter().enumerate() {
        if r.restore.contains(x, y) {
            return Hit::Answer(i, Choice::Restore);
        }
        if r.discard.contains(x, y) {
            return Hit::Answer(i, Choice::Discard);
        }
    }
    Hit::Swallowed
}

/// O `Esc`: `Later` em todas (nada se perde por fechar a janela) ou `Cancel` do fecho.
pub fn key_escape(r: &mut crate::Recovery) {
    if r.prompt.is_open() {
        r.prompt.later_all();
    } else {
        r.quit = crate::Quit::Idle;
    }
}

/// «Today 14:32» · «Yesterday 09:10» · «Nov 3, 14:32», no fuso do sistema.
#[must_use]
pub fn when(saved: u64, now: u64) -> String {
    when_in(saved, now, &jiff::tz::TimeZone::system())
}

/// [`when`] num fuso dado (os testes usam UTC).
#[must_use]
pub fn when_in(saved: u64, now: u64, tz: &jiff::tz::TimeZone) -> String {
    let z = |s: u64| {
        jiff::Timestamp::from_second(i64::try_from(s).unwrap_or(0))
            .unwrap_or(jiff::Timestamp::UNIX_EPOCH)
            .to_zoned(tz.clone())
    };
    let (s, n) = (z(saved), z(now));
    let time = s.strftime("%H:%M").to_string();
    let days = (n.date() - s.date()).get_days();
    match days {
        0 => tr_with("recovery.today_at", &[("time", &time)]),
        1 => tr_with("recovery.yesterday_at", &[("time", &time)]),
        _ => tr_with(
            "recovery.date_at",
            &[("date", &s.strftime("%b %-d").to_string()), ("time", &time)],
        ),
    }
}

/// A segunda linha de um cartão: «Today 14:32 · 3 MB».
#[must_use]
pub fn detail(o: &Orphan, now: u64) -> String {
    tr_with(
        "recovery.detail",
        &[
            ("when", &when(o.meta.saved, now)),
            ("size", &crate::size(o.size)),
        ],
    )
}

const ID_BASE: u64 = 0x5245_434f_5645_0000;

fn id(n: u64) -> NodeId {
    NodeId(ID_BASE | n)
}

fn frame(dialog: Rect, title: &str, scene: &mut VectorScene, text: &mut TextSystem, theme: Theme) {
    fill_rounded_rect(
        scene,
        dialog,
        Radius::Lg.px(),
        resolve(ColorToken::BgElev, theme),
    );
    let pad = Spacing::Lg.px();
    let size = TypeToken::Md.px();
    paint_text(
        text,
        scene,
        title,
        dialog.x + pad,
        dialog.y + (header_h() - size) * 0.5,
        size,
        dialog.w - pad * 2.0,
        resolve(ColorToken::Text1, theme),
    );
}

/// Pinta a janela aberta (o arranque ou o fecho) por cima de tudo, no espaço do chrome sob `map`,
/// e lembra onde (o [`click`] lê daqui). `screen` = o ecrã em píxeis físicos.
pub fn paint(
    r: &mut crate::Recovery,
    screen: Rect,
    map: ph2d_editor_core::ui_scale::UiScaleMap,
    scene: &mut VectorScene,
    text: &mut TextSystem,
    theme: Theme,
) {
    if !r.is_modal() {
        r.screen = None;
        return;
    }
    r.screen = Some((screen, map));
    let now = crate::now_secs();
    // O corpo mede-se (quebrado) ANTES de dispor a janela: ela cresce até caber o texto todo.
    r.quit_body_h = match &r.quit {
        crate::Quit::Asking(name) if !r.prompt.is_open() => {
            let vp = map.rect_to_logical(screen);
            let w = body_w((Spacing::Xl4.px() * DIALOG_COLS).min(vp.w));
            let h = paint_text_block(
                text,
                &mut VectorScene::new(),
                &quit_body(name),
                0.0,
                0.0,
                TypeToken::Base.px(),
                w,
                resolve(ColorToken::Text2, theme),
            );
            Some(h + Spacing::Md.px())
        }
        _ => None,
    };
    let body_h = r.quit_body_h.unwrap_or_else(row_h);
    ph2d_editor_core::ui_scale::pintar_no_chrome(map, screen, scene, |vp, cena| {
        cena.fill_rect(
            ph2d_editor_core::paint::rect_to_vello(vp),
            resolve(ColorToken::BgScrim, theme),
        );
        if r.prompt.is_open() {
            paint_prompt(&r.prompt, vp, now, cena, text, theme);
        } else if let crate::Quit::Asking(name) = &r.quit {
            paint_quit(name, vp, body_h, cena, text, theme);
        }
    });
}

fn quit_body(name: &str) -> String {
    tr_with("recovery.quit_body", &[("name", &name)])
}

fn paint_quit(
    name: &str,
    vp: Rect,
    body_h: f32,
    scene: &mut VectorScene,
    text: &mut TextSystem,
    theme: Theme,
) {
    let q = quit_layout(vp, body_h);
    frame(q.dialog, &tr("recovery.quit_title"), scene, text, theme);
    // Quebra em linhas (como o corpo de toda janela do app): nunca elide uma frase.
    paint_text_block(
        text,
        scene,
        &quit_body(name),
        q.dialog.x + Spacing::Lg.px(),
        q.dialog.y + header_h(),
        TypeToken::Base.px(),
        body_w(q.dialog.w),
        resolve(ColorToken::Text2, theme),
    );
    let buttons = [
        (q.cancel, Button::new(id(0x100), tr("recovery.cancel"))),
        (
            q.dont_save,
            Button::new(id(0x101), tr("recovery.dont_save")).danger(),
        ),
        (q.save, Button::new(id(0x102), tr("recovery.save")).accent()),
    ];
    for (rect, b) in buttons {
        paint_button(&b.on_elevated(), rect, scene, text, theme);
    }
}

fn paint_prompt(
    prompt: &Prompt,
    viewport: Rect,
    now: u64,
    scene: &mut VectorScene,
    text: &mut TextSystem,
    theme: Theme,
) {
    let l = layout(viewport, prompt.entries.len());
    frame(l.dialog, &tr("recovery.title"), scene, text, theme);
    for (i, (row, o)) in l.rows.iter().zip(&prompt.entries).enumerate() {
        fill_rounded_rect(
            scene,
            row.card,
            Radius::Md.px(),
            resolve(ColorToken::Bg2, theme),
        );
        let tx = row.card.x + Spacing::Md.px();
        let tw = (row.discard.x - tx - Spacing::Md.px()).max(0.0);
        let y0 = row.card.y + Spacing::Md.px();
        paint_text(
            text,
            scene,
            &o.meta.name,
            tx,
            y0,
            TypeToken::Md.px(),
            tw,
            resolve(ColorToken::Text1, theme),
        );
        paint_text(
            text,
            scene,
            &detail(o, now),
            tx,
            y0 + TypeToken::Md.px() + Spacing::Md.px(),
            TypeToken::Sm.px(),
            tw,
            resolve(ColorToken::Text2, theme),
        );
        let k = i as u64 * 2;
        let discard = Button::new(id(k + 1), tr("recovery.discard")).on_elevated();
        let restore = Button::new(id(k + 2), tr("recovery.restore"))
            .accent()
            .on_elevated();
        paint_button(&discard, row.discard, scene, text, theme);
        paint_button(&restore, row.restore, scene, text, theme);
    }
    let later = Button::new(id(0), tr("recovery.later")).on_elevated();
    paint_button(&later, l.later, scene, text, theme);
}
