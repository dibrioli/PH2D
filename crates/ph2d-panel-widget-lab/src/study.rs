//! ⭐⭐⭐ **A BANCADA** — o corpo do laboratório, secção a secção.
//!
//! ⚠️ **Toda string que o artista LÊ é inglês** (regra do app, e o Enio apanhou-a na 1.ª foto:
//! *"obviamente tudo em inglês"*). Os comentários e docs continuam em português — o leitor deles é
//! a próxima LLM.
//!
//! ⭐ **Desde 2026-09-02 a bancada é também a CUSTOMIZAÇÃO:** os três primeiros chips escrevem no
//! [`SliderStyle`](ph2d_tokens::SliderStyle) que o app inteiro lê. Ela deixou de ser só um estudo.
//!
//! | § | secção | a pergunta |
//! |---|---|---|
//! | 1 | os **quatro desenhos** | qual é o *look* |
//! | 2 | a **régua de largura** | ⭐ aguenta um painel estreito? — a razão de existir de tudo isto |
//! | 3 | os **estados** | vê-se que é interactivo |
//! | 4 | as **cores** | o acento funciona em todos |
//! | 5 | o **widget antigo**, lado a lado | estamos mesmo a melhorar |

use crate::state::WidgetLabState;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::HitIndex;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text, resolve, stroke_rounded_rect};
use ph2d_editor_core::widget::{PropertyBox, PropertyBoxState, paint_property_box};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{
    ColorToken, Radius, SliderDesign, SliderStyle, Spacing, StrokeToken, Theme, TypeToken,
};
use ph2d_vector::VectorScene;

/// As larguras da régua da §2. ⭐ **Contadas, não escolhidas:** `268` é o corpo do Inspector de
/// hoje · `184` é o corpo dele à largura mínima que o app permite (`PANEL_MIN_W = 220`), onde todo
/// slider antigo já está em duas linhas · `140` e `110` são onde uma coluna de tablet vai parar.
const RULER_WIDTHS: [f32; 4] = [268.0, 184.0, 140.0, 110.0]; // LITERAL-PX-OK: reguas medidas (pesquisa 07 §2)

/// As cores que a §4 percorre. ⚠️ `pub(crate)` — o `event.rs` conta o comprimento DESTA tabela em
/// vez de declarar o seu.
pub(crate) const ACCENTS: [ColorToken; 6] = [
    ColorToken::Accent,
    ColorToken::Info,
    ColorToken::Success,
    ColorToken::Warn,
    ColorToken::Danger,
    ColorToken::AccentPress,
];

/// A amostra: um rótulo comprido de propósito, para a truncagem se ver.
const SAMPLE_LABEL: &str = "Geometry Offset";
const SAMPLE_VALUE: &str = "0.10 m";

/// A fracção que todas as amostras mostram.
///
/// ⚠️ **Uma só, e nem perto de meio.** A `0,5` o preenchimento cai no centro da caixa, onde encosta
/// no valor em metade dos desenhos e em nenhum se percebe se a borda está no sítio certo. E tem de
/// ser a **mesma** nas cinco secções: duas fracções fariam dois desenhos parecer distintos por causa
/// do valor, não do desenho.
pub(crate) const SAMPLE_T: f32 = 0.62; // LITERAL-PX-OK: fraccao da amostra, nao e' medida de UI

/// Fracção → percentagem, para a caixa viva.
const PERCENT: f32 = 100.0; // LITERAL-PX-OK: conversao, nao e' medida

/// A coluna do valor que a bancada reserva — **a mesma que a linha do produto**
/// (`slider_with_chip::DEFAULT_CHIP_W`). ⛔ Não é um número desta bancada: se ela reservasse outra,
/// as cinco secções mostrariam um widget que o app não tem.
const VALUE_W: f32 = ph2d_editor_core::widget::DEFAULT_CHIP_W;

pub(crate) struct Bench<'a> {
    pub scene: &'a mut VectorScene,
    pub text: &'a mut TextSystem,
    pub hit: &'a mut HitIndex,
    pub theme: Theme,
    pub x: f32,
    pub w: f32,
    pub y: f32,
}

impl Bench<'_> {
    fn head(&mut self, s: &str) {
        self.y += Spacing::Lg.px();
        paint_text(
            self.text,
            self.scene,
            s,
            self.x,
            self.y,
            TypeToken::Xs.px(),
            self.w,
            resolve(ColorToken::Text3, self.theme),
        );
        self.y += TypeToken::Xs.px() + Spacing::Xs.px();
        let line = Rect::new(self.x, self.y, self.w, StrokeToken::Hairline.px());
        fill_rounded_rect(
            self.scene,
            line,
            0.0,
            resolve(ColorToken::Border, self.theme),
        );
        self.y += Spacing::Sm.px();
    }

    fn caption(&mut self, s: &str) {
        paint_text(
            self.text,
            self.scene,
            s,
            self.x,
            self.y,
            TypeToken::Xxs.px(),
            self.w,
            resolve(ColorToken::Text3, self.theme),
        );
        self.y += TypeToken::Xxs.px() + Spacing::Xs.px();
    }

    /// Uma amostra da caixa, com a aparência dada.
    ///
    /// ⚠️ Recebe o [`PropertyBox`] inteiro em vez dos seis campos soltos — a 1.ª redacção tinha
    /// **nove** argumentos e o clippy apanhou-a. *Quando uma função de conveniência re-lista os
    /// campos de um struct que já existe, ela é a segunda declaração do mesmo modelo.*
    fn box_row(&mut self, r: Rect, b: PropertyBox<'_>, style: SliderStyle) {
        paint_property_box(self.scene, self.text, self.theme, r, b, style);
    }
}

/// Pinta a bancada e devolve a altura usada.
///
/// ⚠️ `live` é `(fracção, está-a-arrastar)` e vem do **store**: a caixa viva é um
/// `InteractiveState::Slider` de verdade, conduzido pelo despacho de ponteiro do produto.
pub(crate) fn paint_study(b: &mut Bench<'_>, st: &WidgetLabState, live: (f32, bool)) -> f32 {
    let top = b.y;
    let style = st.style;
    let row_h = style.row_h_px();
    let accent = ACCENTS[st.accent % ACCENTS.len()];

    paint_controls(b, st);

    // ── §1 — os quatro desenhos ────────────────────────────────────────────
    b.head("1 \u{b7} THE FOUR DESIGNS");
    for d in SliderDesign::ALL {
        if d == style.design {
            let mark = Rect::new(b.x - Spacing::Sm.px(), b.y, StrokeToken::Thick.px(), row_h);
            fill_rounded_rect(b.scene, mark, 0.0, resolve(ColorToken::Accent, b.theme));
        }
        let r = Rect::new(b.x, b.y, b.w, row_h);
        b.box_row(
            r,
            PropertyBox {
                label: SAMPLE_LABEL,
                value: SAMPLE_VALUE,
                t: SAMPLE_T,
                state: PropertyBoxState::Normal,
                accent,
                decorator: st.decorator,
                // ⚠️ A MESMA coluna que a linha do produto reserva. Medi-la pelo texto faria a
                // régua da §2 mentir: a `110 px` a amostra sobraria espaço que uma linha real não
                // tem, e a bancada aprovaria um desenho que o app não consegue mostrar.
                value_w: Some(VALUE_W),
            },
            SliderStyle { design: d, ..style },
        );
        b.y += row_h + Spacing::Xxs.px();
        let blurb = format!("{} \u{b7} {}", d.label(), d.blurb());
        b.caption(&blurb);
        b.y += Spacing::Xs.px();
    }

    // ── §2 — a régua de largura ────────────────────────────────────────────
    b.head("2 \u{b7} WIDTH RULER \u{2014} the chosen design, squeezed");
    b.caption(
        "268 = today's Inspector \u{b7} 184 = the app's MINIMUM column \u{b7} 140 and 110 = tablet",
    );
    for w in RULER_WIDTHS {
        let w = w.min(b.w);
        b.box_row(
            Rect::new(b.x, b.y, w, row_h),
            PropertyBox {
                label: SAMPLE_LABEL,
                value: SAMPLE_VALUE,
                t: SAMPLE_T,
                state: PropertyBoxState::Normal,
                accent,
                decorator: st.decorator,
                // ⚠️ A MESMA coluna que a linha do produto reserva. Medi-la pelo texto faria a
                // régua da §2 mentir: a `110 px` a amostra sobraria espaço que uma linha real não
                // tem, e a bancada aprovaria um desenho que o app não consegue mostrar.
                value_w: Some(VALUE_W),
            },
            style,
        );
        paint_text(
            b.text,
            b.scene,
            &format!("{w:.0}"),
            b.x + w + Spacing::Sm.px(),
            b.y + (row_h - TypeToken::Xxs.px()) * 0.5,
            TypeToken::Xxs.px(),
            f32::INFINITY,
            resolve(ColorToken::Text3, b.theme),
        );
        b.y += row_h + Spacing::Xs.px();
    }

    // ── §3 — os estados ────────────────────────────────────────────────────
    b.head("3 \u{b7} STATES");
    for s in PropertyBoxState::ALL {
        b.box_row(
            Rect::new(b.x, b.y, b.w, row_h),
            PropertyBox {
                label: s.label(),
                value: SAMPLE_VALUE,
                t: SAMPLE_T,
                state: s,
                accent,
                decorator: st.decorator,
                // ⚠️ A MESMA coluna que a linha do produto reserva. Medi-la pelo texto faria a
                // régua da §2 mentir: a `110 px` a amostra sobraria espaço que uma linha real não
                // tem, e a bancada aprovaria um desenho que o app não consegue mostrar.
                value_w: Some(VALUE_W),
            },
            style,
        );
        b.y += row_h + Spacing::Xs.px();
    }

    // ── §4 — as cores ──────────────────────────────────────────────────────
    b.head("4 \u{b7} ACCENTS");
    let half = (b.w - Spacing::Sm.px()) * 0.5;
    for (i, a) in ACCENTS.iter().enumerate() {
        let col = i % 2;
        let r = Rect::new(
            b.x + col as f32 * (half + Spacing::Sm.px()),
            b.y,
            half,
            row_h,
        );
        b.box_row(
            r,
            PropertyBox {
                label: a.key(),
                value: "62%",
                t: SAMPLE_T,
                state: PropertyBoxState::Normal,
                accent: *a,
                decorator: st.decorator,
                // ⚠️ A MESMA coluna que a linha do produto reserva. Medi-la pelo texto faria a
                // régua da §2 mentir: a `110 px` a amostra sobraria espaço que uma linha real não
                // tem, e a bancada aprovaria um desenho que o app não consegue mostrar.
                value_w: Some(VALUE_W),
            },
            style,
        );
        if col == 1 || i == ACCENTS.len() - 1 {
            b.y += row_h + Spacing::Xs.px();
        }
    }

    // ── §5 — o widget antigo ───────────────────────────────────────────────
    if st.compare {
        b.head("5 \u{b7} THE OLD WIDGET, SIDE BY SIDE");
        b.caption("label 70 + gap 6 + track + gap 6 + box 72 = 154 px of fixed chrome");
        for w in RULER_WIDTHS {
            let w = w.min(b.w);
            let h = paint_old_widget(b, w, row_h);
            b.y += h + Spacing::Xs.px();
        }
    }

    // ── A caixa VIVA ───────────────────────────────────────────────────────
    b.head("LIVE BOX \u{2014} drag it");
    let live_rect = Rect::new(b.x, b.y, b.w, row_h);
    b.hit.register(ids::LAB_LIVE_BOX, live_rect);
    let value = format!("{:.0}%", live.0 * PERCENT);
    let state = if live.1 {
        PropertyBoxState::Dragging
    } else {
        PropertyBoxState::Normal
    };
    b.box_row(
        live_rect,
        PropertyBox {
            label: "Opacity",
            value: &value,
            t: live.0,
            state,
            accent,
            decorator: st.decorator,
            value_w: Some(VALUE_W),
        },
        style,
    );
    b.y += row_h + Spacing::Xl.px();

    b.y - top
}

/// O widget ANTIGO, redesenhado aqui em miniatura para a comparação da §5.
///
/// ⚠️ **Redesenhado, não chamado.** O `paint_slider_with_chip` real precisa do `WidgetStore` e
/// regista hits — dentro de uma bancada isso poria alvos invisíveis a competir com os controlos. O
/// que interessa comparar é a **geometria**, e ela está reproduzida à letra: 70 / 6 / 6 / 72, com o
/// mesmo limiar de empilhamento.
fn paint_old_widget(b: &mut Bench<'_>, w: f32, row_h: f32) -> f32 {
    const LABEL_W: f32 = 70.0; // LITERAL-PX-OK: DEFAULT_LABEL_W do slider_with_chip
    const CHIP_W: f32 = 72.0; // LITERAL-PX-OK: number_input::MIN_W_PX
    const MIN_TRACK: f32 = 60.0; // LITERAL-PX-OK: SLIDER_CHIP_MIN_SLIDER_W
    let gap = Spacing::Sm.px();
    let stacked = w < LABEL_W + CHIP_W + gap * 2.0 + MIN_TRACK;
    let theme = b.theme;

    let (label_r, row_y, used_h) = if stacked {
        (
            Rect::new(b.x, b.y, w, row_h),
            b.y + row_h,
            row_h * 2.0 + Spacing::Xxs.px(),
        )
    } else {
        (Rect::new(b.x, b.y, LABEL_W, row_h), b.y, row_h)
    };

    paint_text(
        b.text,
        b.scene,
        SAMPLE_LABEL,
        label_r.x,
        label_r.y + (row_h - TypeToken::Sm.px()) * 0.5,
        TypeToken::Sm.px(),
        label_r.w,
        resolve(ColorToken::Text1, theme),
    );

    let track_x = if stacked { b.x } else { b.x + LABEL_W + gap };
    let track_w = (b.x + w - CHIP_W - gap - track_x).max(0.0);
    // A politica de linha do slider ANTIGO, reproduzida a` letra: a §5 tem de medir o widget que
    // existia, nao uma aproximacao dele.
    let track_h = (row_h * 0.25).min(Spacing::Sm.px()); // LITERAL-PX-OK: 25% da moldura, do slider antigo
    let track = Rect::new(track_x, row_y + (row_h - track_h) * 0.5, track_w, track_h);
    fill_rounded_rect(
        b.scene,
        track,
        track_h * 0.5,
        resolve(ColorToken::Bg3, theme),
    );
    let fill = Rect::new(track.x, track.y, track.w * SAMPLE_T, track.h);
    fill_rounded_rect(
        b.scene,
        fill,
        track_h * 0.5,
        resolve(ColorToken::Accent, theme),
    );

    let chip = Rect::new(b.x + w - CHIP_W, row_y, CHIP_W, row_h);
    fill_rounded_rect(
        b.scene,
        chip,
        Radius::Sm.px(),
        resolve(ColorToken::Bg3, theme),
    );
    stroke_rounded_rect(
        b.scene,
        chip,
        Radius::Sm.px(),
        StrokeToken::Hairline.px(),
        resolve(ColorToken::Border, theme),
    );
    paint_text(
        b.text,
        b.scene,
        SAMPLE_VALUE,
        chip.x + Spacing::Sm.px(),
        chip.y + (row_h - TypeToken::Sm.px()) * 0.5,
        TypeToken::Sm.px(),
        chip.w,
        resolve(ColorToken::Text1, theme),
    );

    if stacked {
        paint_text(
            b.text,
            b.scene,
            "\u{2191} STACKED: 2 rows",
            b.x + w + Spacing::Sm.px(),
            b.y + (row_h - TypeToken::Xxs.px()) * 0.5,
            TypeToken::Xxs.px(),
            f32::INFINITY,
            resolve(ColorToken::Warn, theme),
        );
    }
    used_h
}

/// A fileira de controlos. Cada chip regista o próprio hit e é lido pelo `event.rs`.
///
/// ⭐ Os três primeiros mudam o **app**; os três últimos mudam só a bancada.
fn paint_controls(b: &mut Bench<'_>, st: &WidgetLabState) {
    let s = st.style;
    let chips: [(ph2d_a11y::NodeId, String); 7] = [
        (ids::LAB_VARIANT_PREV, "\u{2039}".into()),
        (
            ids::LAB_VARIANT_NEXT,
            format!("{} \u{203a}", s.design.label()),
        ),
        (
            ids::LAB_RADIUS_CYCLE,
            format!("radius {:.0}", s.radius_px()),
        ),
        (ids::LAB_DENSITY_CYCLE, format!("row {:.0}", s.row_h_px())),
        (
            ids::LAB_ACCENT_CYCLE,
            ACCENTS[st.accent % ACCENTS.len()].key().into(),
        ),
        (
            ids::LAB_DECORATOR_TOGGLE,
            format!("animate {}", if st.decorator { "ON" } else { "off" }),
        ),
        (
            ids::LAB_COMPARE_TOGGLE,
            format!("old {}", if st.compare { "ON" } else { "off" }),
        ),
    ];
    let h = s.row_h_px();
    let mut cx = b.x;
    let mut cy = b.y;
    for (id, text) in chips {
        let tw = b
            .text
            .layout(&text, TypeToken::Xxs.px(), f32::INFINITY)
            .width();
        let cw = tw + Spacing::Md.px() * 2.0;
        if cx + cw > b.x + b.w {
            cx = b.x;
            cy += h + Spacing::Xs.px();
        }
        let r = Rect::new(cx, cy, cw, h);
        b.hit.register(id, r);
        fill_rounded_rect(
            b.scene,
            r,
            Radius::Xs.px(),
            resolve(ColorToken::Bg2, b.theme),
        );
        stroke_rounded_rect(
            b.scene,
            r,
            Radius::Xs.px(),
            StrokeToken::Hairline.px(),
            resolve(ColorToken::Border, b.theme),
        );
        paint_text(
            b.text,
            b.scene,
            &text,
            r.x + Spacing::Md.px(),
            r.y + (h - TypeToken::Xxs.px()) * 0.5,
            TypeToken::Xxs.px(),
            f32::INFINITY,
            resolve(ColorToken::Text2, b.theme),
        );
        cx += cw + Spacing::Xs.px();
    }
    b.y = cy + h + Spacing::Sm.px();
}
