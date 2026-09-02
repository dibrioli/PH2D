//! ⭐⭐⭐ **A BANCADA** — o corpo do laboratório, secção a secção.
//!
//! O Enio pediu (2026-09-01): *"podemos colocar no painel várias amostras e várias cores e
//! comportamentos para testar"*. As cinco secções abaixo são exactamente isso, e a ordem delas é
//! a ordem em que uma decisão de desenho se toma:
//!
//! | § | secção | a pergunta que ela responde |
//! |---|---|---|
//! | 1 | **os seis desenhos** | qual é o *look* |
//! | 2 | **a régua de largura** | ⭐ ele aguenta um painel estreito? — a razão de existir de tudo isto |
//! | 3 | **os estados** | dá para ver que é interactivo |
//! | 4 | **as cores** | o acento funciona em todos |
//! | 5 | **o de hoje, lado a lado** | ⭐⭐ estamos mesmo a melhorar? |
//!
//! ⚠️ **A §2 é a que decide.** As outras quatro são gosto; essa é medição — o desenho escolhido
//! tem de continuar legível a `110 px`, que é abaixo do `PANEL_MIN_W = 220` de hoje e é onde uma
//! coluna de tablet vai parar.

use crate::design::{BoxDesign, BoxState, BoxStyle, DECORATOR_W, paint_box};
use crate::state::WidgetLabState;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::HitIndex;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text, resolve, stroke_rounded_rect};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// As larguras da régua da §2. ⭐ **Contadas, não escolhidas:**
/// `268` é o corpo do Inspector de hoje · `184` é o corpo dele à largura mínima que o app permite
/// (`PANEL_MIN_W = 220`), onde **todo slider de hoje já está em duas linhas** · `140` e `110` são
/// onde uma coluna de tablet vai parar quando as duas estiverem abertas.
const RULER_WIDTHS: [f32; 4] = [268.0, 184.0, 140.0, 110.0]; // LITERAL-PX-OK: reguas medidas (pesquisa 07 §2)

/// As cores que a §4 percorre. ⚠️ **`pub(crate)` de propósito** — o `event.rs` conta o
/// comprimento DESTA tabela em vez de declarar o seu; ver a nota lá.
pub(crate) const ACCENTS: [ColorToken; 6] = [
    ColorToken::Accent,
    ColorToken::Info,
    ColorToken::Success,
    ColorToken::Warn,
    ColorToken::Danger,
    ColorToken::AccentPress,
];

/// Os raios que o `LAB_RADIUS_CYCLE` percorre — `16` é o de hoje, `4` é o do Godot.
pub const RADII: [f32; 4] = [16.0, 8.0, 4.0, 0.0]; // LITERAL-PX-OK: eixo de estudo do raio

/// A escada de densidade que o `LAB_DENSITY_CYCLE` percorre.
///
/// ⚠️ **Começa no `Compact`** — ao contrário do app, cujo default é `Comfortable` por causa do
/// Pencil. A bancada quer mostrar o caso apertado primeiro, porque é o que a decisão precisa de
/// ver; o produto quer o alvo de toque grande. *São perguntas diferentes e por isso defaults
/// diferentes.*
pub const DENSITIES: [ph2d_tokens::Density; 3] = [
    ph2d_tokens::Density::Compact,
    ph2d_tokens::Density::Cozy,
    ph2d_tokens::Density::Comfortable,
];

/// A amostra: um rótulo comprido de propósito, para a truncagem se ver.
const SAMPLE_LABEL: &str = "Geometry Offset";
const SAMPLE_VALUE: &str = "0.10 m";

/// A fracção que todas as amostras mostram.
///
/// ⚠️ **Uma só, e nem perto de meio.** Amostras a `0,5` fazem o preenchimento cair exactamente no
/// centro da caixa, onde ele encosta no valor em metade dos desenhos e em nenhum se percebe se a
/// borda está no sítio certo. E tem de ser a **mesma** nas cinco secções: duas fracções diferentes
/// fariam dois desenhos parecer distintos por causa do valor, não do desenho.
pub(crate) const SAMPLE_T: f32 = 0.62; // LITERAL-PX-OK: fraccao da amostra, nao e' medida de UI

/// Para escrever a fracção como percentagem na caixa viva.
const PERCENT: f32 = 100.0; // LITERAL-PX-OK: conversao de fraccao para percentagem, nao e' medida

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
}

/// Pinta a bancada inteira e devolve a altura de conteúdo usada.
///
/// ⚠️ `live` é `(fracção, está-a-arrastar)` e vem do **store**, nunca do estado do painel: a caixa
/// viva é um `InteractiveState::Slider` de verdade, então quem manda no valor dela é o mesmo
/// despacho de ponteiro que manda em todos os sliders do app. *Guardar aqui uma cópia do valor
/// seria a segunda fonte de verdade que a auditoria da §11 do Sprite já cobrou.*
pub(crate) fn paint_study(
    b: &mut Bench<'_>,
    st: &WidgetLabState,
    row_h: f32,
    live: (f32, bool),
) -> f32 {
    let top = b.y;
    let style = BoxStyle {
        design: st.design,
        state: BoxState::Normal,
        accent: ACCENTS[st.accent % ACCENTS.len()],
        radius: RADII[st.radius % RADII.len()],
        decorator: st.decorator,
    };

    paint_controls(b, st, row_h);

    // ── §1 — os seis desenhos ──────────────────────────────────────────────
    b.head("1 \u{b7} OS SEIS DESENHOS");
    for d in BoxDesign::ALL {
        let chosen = d == st.design;
        if chosen {
            let mark = Rect::new(b.x - Spacing::Sm.px(), b.y, 2.0, row_h); // LITERAL-PX-OK: marca do escolhido
            fill_rounded_rect(b.scene, mark, 0.0, resolve(ColorToken::Accent, b.theme));
        }
        let r = Rect::new(b.x, b.y, b.w, row_h);
        paint_box(
            b.scene,
            b.text,
            b.theme,
            r,
            SAMPLE_LABEL,
            SAMPLE_VALUE,
            SAMPLE_T,
            BoxStyle { design: d, ..style },
        );
        b.y += row_h + Spacing::Xxs.px();
        b.caption(&format!("{} \u{b7} {}", d.label(), d.blurb()));
        b.y += Spacing::Xs.px();
    }

    // ── §2 — a régua de largura ────────────────────────────────────────────
    b.head("2 \u{b7} A REGUA DE LARGURA \u{2014} o desenho escolhido, encolhido");
    b.caption("268 = Inspector de hoje \u{b7} 184 = a largura MINIMA do app (hoje empilha) \u{b7} 140 e 110 = coluna de tablet");
    for w in RULER_WIDTHS {
        let w = w.min(b.w);
        let r = Rect::new(b.x, b.y, w, row_h);
        paint_box(
            b.scene,
            b.text,
            b.theme,
            r,
            SAMPLE_LABEL,
            SAMPLE_VALUE,
            SAMPLE_T,
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
    b.head("3 \u{b7} OS ESTADOS");
    for s in BoxState::ALL {
        let r = Rect::new(b.x, b.y, b.w, row_h);
        paint_box(
            b.scene,
            b.text,
            b.theme,
            r,
            s.label(),
            SAMPLE_VALUE,
            SAMPLE_T,
            BoxStyle { state: s, ..style },
        );
        b.y += row_h + Spacing::Xs.px();
    }

    // ── §4 — as cores ──────────────────────────────────────────────────────
    b.head("4 \u{b7} AS CORES");
    let half = (b.w - Spacing::Sm.px()) * 0.5;
    for (i, a) in ACCENTS.iter().enumerate() {
        let col = i % 2;
        let r = Rect::new(
            b.x + col as f32 * (half + Spacing::Sm.px()),
            b.y,
            half,
            row_h,
        );
        paint_box(
            b.scene,
            b.text,
            b.theme,
            r,
            a.key(),
            "62%",
            SAMPLE_T,
            BoxStyle {
                accent: *a,
                ..style
            },
        );
        if col == 1 || i == ACCENTS.len() - 1 {
            b.y += row_h + Spacing::Xs.px();
        }
    }

    // ── §5 — o de hoje ─────────────────────────────────────────────────────
    if st.compare {
        b.head("5 \u{b7} O DE HOJE, LADO A LADO");
        b.caption("rotulo 70 + folga 6 + trilho + folga 6 + caixa 72 = 154 px de cromo fixo");
        for w in RULER_WIDTHS {
            let w = w.min(b.w);
            let h = paint_today(b, w, row_h);
            b.y += h + Spacing::Xs.px();
        }
    }

    // ── A caixa VIVA ───────────────────────────────────────────────────────
    b.head("A CAIXA VIVA \u{2014} arraste-a");
    let live_rect = Rect::new(b.x, b.y, b.w, row_h);
    b.hit.register(ids::LAB_LIVE_BOX, live_rect);
    paint_box(
        b.scene,
        b.text,
        b.theme,
        live_rect,
        "Opacity",
        &format!("{:.0}%", live.0 * PERCENT),
        live.0,
        BoxStyle {
            state: if live.1 {
                BoxState::Dragging
            } else {
                BoxState::Normal
            },
            ..style
        },
    );
    b.y += row_h + Spacing::Xl.px();

    b.y - top
}

/// O widget de HOJE, redesenhado aqui em miniatura para a comparação da §5.
///
/// ⚠️ **Redesenhado, não chamado.** O `paint_slider_with_chip` real precisa do `WidgetStore` e
/// regista hits — dentro de uma bancada de comparação isso poria 8 alvos invisíveis a competir com
/// os controlos. O que interessa comparar é a **geometria**, e ela está reproduzida à letra: 70 /
/// 6 / 6 / 72, com o mesmo limiar de empilhamento.
fn paint_today(b: &mut Bench<'_>, w: f32, row_h: f32) -> f32 {
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
    let track_h = (row_h * 0.25).min(6.0); // LITERAL-PX-OK: a politica de linha do slider de hoje
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
            "\u{2191} EMPILHOU: 2 linhas",
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
fn paint_controls(b: &mut Bench<'_>, st: &WidgetLabState, row_h: f32) {
    let chips: [(ph2d_a11y::NodeId, String); 7] = [
        (ids::LAB_VARIANT_PREV, "\u{2039}".into()),
        (
            ids::LAB_VARIANT_NEXT,
            format!("{} \u{203a}", st.design.label()),
        ),
        (
            ids::LAB_RADIUS_CYCLE,
            format!("raio {:.0}", RADII[st.radius % RADII.len()]),
        ),
        (
            ids::LAB_ACCENT_CYCLE,
            ACCENTS[st.accent % ACCENTS.len()].key().into(),
        ),
        (ids::LAB_DENSITY_CYCLE, format!("linha {row_h:.0}")),
        (
            ids::LAB_DECORATOR_TOGGLE,
            format!("animar {}", if st.decorator { "ON" } else { "off" }),
        ),
        (
            ids::LAB_COMPARE_TOGGLE,
            format!("hoje {}", if st.compare { "ON" } else { "off" }),
        ),
    ];
    let h = row_h.min(24.0); // LITERAL-PX-OK: altura do chip de controlo
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
    let _ = DECORATOR_W;
}
