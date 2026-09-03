//! ⭐⭐⭐ **A CAIXA ÚNICA** — o pintor de uma linha de propriedade, e o padrão do app desde
//! 2026-09-02.
//!
//! Uma linha deixa de ser `rótulo | trilho | caixa numérica` (`154 px` de cromo fixo, medido em
//! `docs/UI_New_and_Simple/pesquisa/07` §2) e passa a ser **uma caixa**: rótulo à esquerda dentro,
//! valor à direita dentro, e o preenchimento a dizer a fracção.
//!
//! O desenho, o raio e a altura vêm do [`SliderStyle`](ph2d_tokens::SliderStyle) — a aparência que
//! o artista escolhe, publicada uma vez por quadro como o [`TextRendering`](ph2d_tokens::TextRendering).
//!
//! # ⚠️ Uma PORTA, não uma cópia
//!
//! Este ficheiro é o **único** sítio que sabe desenhar a caixa. O Widget Lab pinta as amostras dele
//! chamando aqui; a linha do produto passa por aqui. ⛔ *Um segundo pintor «só para a bancada» faria
//! o estudo divergir do produto sem ninguém notar* — que é literalmente o bug que criou o
//! [`slider_with_chip`](super::slider_with_chip) (*"the slider in panel X looks different from the
//! one in panel Y"*).
//!
//! # A lei do rótulo
//!
//! ⚠️ **O rótulo é o que CEDE.** Se não couber, trunca; o valor **nunca** trunca, porque um número
//! cortado é um número errado. É a inversão exacta do widget antigo, onde o rótulo tinha `70 px`
//! fixos e o trilho encolhia até desaparecer.

use crate::paint::{fill_rounded_rect, paint_text, resolve, stroke_rounded_rect};
use crate::zones::Rect;
use ph2d_a11y::{Action, Node, NodeBuilder, Role};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, SliderDesign, SliderStyle, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// Em que estado a caixa é pintada.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum PropertyBoxState {
    #[default]
    Normal,
    Hovered,
    Dragging,
    Disabled,
    /// A escrever — a caixa virou campo de texto (clicar edita, como no Blender).
    Editing,
}

impl PropertyBoxState {
    pub const ALL: [PropertyBoxState; 5] = [
        PropertyBoxState::Normal,
        PropertyBoxState::Hovered,
        PropertyBoxState::Dragging,
        PropertyBoxState::Disabled,
        PropertyBoxState::Editing,
    ];

    /// O nome que aparece no ecrã (inglês — regra do app).
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            PropertyBoxState::Normal => "normal",
            PropertyBoxState::Hovered => "hover",
            PropertyBoxState::Dragging => "drag",
            PropertyBoxState::Disabled => "disabled",
            PropertyBoxState::Editing => "typing",
        }
    }
}

/// A largura da coluna de animação — o *decorator* do Blender.
///
/// ⭐ Enio, 2026-09-01: *"em todas as propriedades que podem ser animadas, e nessa engine vou querer
/// animar tudo"* ⇒ ela é permanente, e sai de **todas** as linhas. Por isso é medida aqui, uma vez,
/// e não escolhida em cada sítio.
pub const DECORATOR_W: f32 = 14.0; // LITERAL-PX-OK: coluna de animacao (o decorator do Blender)

/// Quantos `pad` separam o fim do rótulo do início do valor: um de cada bordo, mais **um** entre os
/// dois. ⚠️ É uma CONTAGEM de folgas, não uma medida — o px vem do `Spacing::Md`.
const PAD_UNITS_BETWEEN_LABEL_AND_VALUE: f32 = 3.0; // LITERAL-PX-OK: CONTAGEM de folgas, nao px

/// Tudo o que o pintor precisa e que **não** é geometria.
#[derive(Copy, Clone, Debug)]
pub struct PropertyBox<'a> {
    pub label: &'a str,
    /// O valor já formatado, com unidade se houver (`"0.10 m"`, `"62%"`).
    pub value: &'a str,
    /// A fracção `0..1` que o preenchimento mostra.
    pub t: f32,
    pub state: PropertyBoxState,
    /// A cor do preenchimento.
    pub accent: ColorToken,
    /// Desenha a coluna de animação à direita.
    pub decorator: bool,
}

impl PropertyBox<'_> {
    /// **O nó de acessibilidade da caixa** — irmão exacto do [`super::Slider::build_a11y`].
    ///
    /// ⚠️ **A caixa única é um SLIDER para quem não a vê**, e é aqui que isso fica dito: o rótulo
    /// que o vidente lê à esquerda e o valor que ele lê à direita são, para o leitor de ecrã, o
    /// `label` e o `numeric_value` do mesmo nó. ⛔ Sem isto a fusão das três colunas numa **apagava
    /// a semântica** junto com o cromo — o widget antigo tinha um nó de slider e um de campo, e
    /// perder os dois em silêncio seria o preço escondido de um redesenho que se anuncia como
    /// visual.
    ///
    /// ⏳ **BURACO NOMEADO, não fingido:** o `t` viaja como `numeric_value` (a fracção `0..1`, como
    /// no `Slider`), e o **texto** do valor — `"0.10 m"` — **não tem slot**: o nosso
    /// [`NodeBuilder`] só tem `label` e os três `numeric_value*`. ⇒ quem não vê ouve *«Speed, 62 %»*
    /// e **perde a unidade**. ⛔ Não o dobrei dentro do `label` (`"Speed 0.10 m"`) porque isso
    /// mistura duas grandezas num campo com dono, e o dia em que o builder ganhar um `value` de
    /// texto deixaria dois sítios a dizer a mesma coisa. *A cura é um campo no `ph2d-a11y`, que é
    /// foundational de outra gente e não cabe nesta wave.*
    #[must_use]
    pub fn a11y_node(&self, rect: Rect) -> Node {
        NodeBuilder::new(Role::Slider)
            .label(self.label)
            .bounds(rect.x as f64, rect.y as f64, rect.w as f64, rect.h as f64)
            .focusable(self.state != PropertyBoxState::Disabled)
            .action(Action::Click)
            .numeric_value(f64::from(self.t.clamp(0.0, 1.0)))
            .numeric_value_min(0.0)
            .numeric_value_max(1.0)
            .build()
    }
}

/// ⭐⭐⭐ **Pinta a caixa.** `style` é a aparência escolhida; `rect` é a linha inteira, decorator
/// incluído.
///
/// Devolve o rectângulo do **valor** — a região que aceita clique-para-escrever. Quem regista hits
/// precisa dela, e derivá-la duas vezes poria o alvo num sítio e a tinta noutro.
pub fn paint_property_box(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    rect: Rect,
    b: PropertyBox<'_>,
    style: SliderStyle,
) -> Rect {
    let t = b.t.clamp(0.0, 1.0);
    let disabled = b.state == PropertyBoxState::Disabled;
    let pad = Spacing::Md.px();

    // A coluna de animação sai da direita ANTES de tudo — ela é do FORMULÁRIO, não da caixa.
    let (box_rect, deco) = if b.decorator {
        let w = (rect.w - DECORATOR_W).max(1.0);
        (
            Rect::new(rect.x, rect.y, w, rect.h),
            Some(Rect::new(rect.x + w, rect.y, DECORATOR_W, rect.h)),
        )
    } else {
        (rect, None)
    };

    let fg = if disabled {
        ColorToken::TextDisabled
    } else {
        ColorToken::Text1
    };
    let accent = if disabled {
        ColorToken::Border
    } else {
        b.accent
    };

    paint_surface(scene, theme, box_rect, t, b.state, accent, style);

    let size = TypeToken::Sm.px();
    let value_w = text_system.layout(b.value, size, f32::INFINITY).width();
    let text_h = text_system.layout(b.value, size, f32::INFINITY).height();
    let ty = box_rect.y + (box_rect.h - text_h) * 0.5;

    // O valor, encostado à direita. NUNCA trunca.
    let vx = box_rect.x + box_rect.w - pad - value_w;
    paint_text(
        text_system,
        scene,
        b.value,
        vx,
        ty,
        size,
        f32::INFINITY,
        resolve(fg, theme),
    );

    // O rótulo, dentro à esquerda, com o que sobra depois do valor.
    let budget = (box_rect.w - pad * PAD_UNITS_BETWEEN_LABEL_AND_VALUE - value_w).max(0.0);
    let cut = fit_label(text_system, b.label, size, budget);
    if !cut.is_empty() {
        paint_text(
            text_system,
            scene,
            &cut,
            box_rect.x + pad,
            ty,
            size,
            f32::INFINITY,
            resolve(fg, theme),
        );
    }

    if b.state == PropertyBoxState::Editing {
        let caret = Rect::new(
            vx + value_w + StrokeToken::Thin.px(),
            box_rect.y + Spacing::Xs.px(),
            StrokeToken::Thin.px(),
            (box_rect.h - Spacing::Md.px()).max(1.0),
        );
        fill_rounded_rect(scene, caret, 0.0, resolve(ColorToken::Accent, theme));
    }

    if let Some(d) = deco {
        paint_decorator(scene, theme, d, disabled);
    }

    // A região do valor: do início da folga antes do número até ao bordo da caixa.
    Rect::new(
        (vx - pad).max(box_rect.x),
        box_rect.y,
        (box_rect.x + box_rect.w - (vx - pad)).max(1.0),
        box_rect.h,
    )
}

/// A superfície — é aqui que os quatro desenhos divergem, e **só** aqui.
fn paint_surface(
    scene: &mut VectorScene,
    theme: Theme,
    r: Rect,
    t: f32,
    state: PropertyBoxState,
    accent: ColorToken,
    style: SliderStyle,
) {
    let rad = style.radius_px();
    let hot = matches!(
        state,
        PropertyBoxState::Hovered | PropertyBoxState::Dragging
    );
    let trough = if hot {
        ColorToken::Bg2
    } else {
        ColorToken::Bg3
    };
    let fill_w = (r.w * t).max(0.0);
    // A espessura da linha de valor do `Underline`. `Thick` é o token de 2 px.
    let base_h = StrokeToken::Thick.px();

    match style.design {
        SliderDesign::Underline => {
            fill_rounded_rect(scene, r, rad, resolve(trough, theme));
            let base = Rect::new(r.x, r.y + r.h - base_h, r.w, base_h);
            fill_rounded_rect(scene, base, 0.0, resolve(ColorToken::Border, theme));
            if fill_w > 0.5 {
                let f = Rect::new(r.x, r.y + r.h - base_h, fill_w, base_h);
                fill_rounded_rect(scene, f, 0.0, resolve(accent, theme));
            }
        }
        SliderDesign::Bar => {
            fill_rounded_rect(scene, r, rad, resolve(trough, theme));
            if fill_w > 0.5 {
                fill_rounded_rect(
                    scene,
                    Rect::new(r.x, r.y, fill_w, r.h),
                    rad,
                    resolve(accent, theme),
                );
            }
        }
        SliderDesign::Inset => {
            fill_rounded_rect(scene, r, rad, resolve(trough, theme));
            let m = Spacing::Xxs.px();
            let inner = Rect::new(
                r.x + m,
                r.y + m,
                (r.w - m * 2.0).max(0.0),
                (r.h - m * 2.0).max(0.0),
            );
            let iw = inner.w * t;
            if iw > 0.5 {
                fill_rounded_rect(
                    scene,
                    Rect::new(inner.x, inner.y, iw, inner.h),
                    (rad - m).max(0.0),
                    resolve(accent, theme),
                );
            }
        }
        SliderDesign::Ghost => {
            if fill_w > 0.5 {
                fill_rounded_rect(
                    scene,
                    Rect::new(r.x, r.y, fill_w, r.h),
                    rad,
                    resolve(ColorToken::AccentSoft, theme),
                );
            }
            let h = StrokeToken::Thin.px();
            let base = Rect::new(r.x, r.y + r.h - h, r.w, h);
            fill_rounded_rect(scene, base, 0.0, resolve(ColorToken::Border, theme));
        }
    }

    if state == PropertyBoxState::Editing {
        stroke_rounded_rect(
            scene,
            r,
            rad,
            StrokeToken::Thin.px(),
            resolve(ColorToken::Accent, theme),
        );
    } else if state == PropertyBoxState::Hovered {
        stroke_rounded_rect(
            scene,
            r,
            rad,
            StrokeToken::Hairline.px(),
            resolve(ColorToken::BorderStrong, theme),
        );
    }
}

/// A coluna de animação.
///
/// ⚠️ Aqui ela é só o estado «animável, sem chave» (o ponto vazio). Os outros do Blender — losango
/// cheio (chave neste quadro), losango vazio (chave noutro), ícone de driver — são trabalho a
/// seguir e precisam da **timeline**, não de desenho.
fn paint_decorator(scene: &mut VectorScene, theme: Theme, r: Rect, disabled: bool) {
    let d = Spacing::Xs.px();
    let dot = Rect::new(r.x + (r.w - d) * 0.5, r.y + (r.h - d) * 0.5, d, d);
    let c = if disabled {
        ColorToken::TextDisabled
    } else {
        ColorToken::Text3
    };
    fill_rounded_rect(scene, dot, d * 0.5, resolve(c, theme));
}

/// Trunca o rótulo para caber, com reticências.
///
/// ⚠️ Devolve string VAZIA quando nem duas letras cabem — e isso é uma resposta, não uma falha: a
/// caixa fica só com o número, que é o degrau seguinte da escada do estreito (pesquisa §6.1).
///
/// ⏳ A alternativa é o **esbatimento** (`Scene::push_luminance_mask_layer`, zero consumidores
/// hoje): em vez de `…`, o rótulo desvanece nos últimos px. É mais bonito e não come letras —
/// nomeado na pesquisa §7.3, com o custo por medir.
fn fit_label(text_system: &mut TextSystem, label: &str, size: f32, budget: f32) -> String {
    if budget <= 0.0 {
        return String::new();
    }
    if text_system.layout(label, size, f32::INFINITY).width() <= budget {
        return label.to_string();
    }
    let ell = "\u{2026}";
    let mut chars: Vec<char> = label.chars().collect();
    while !chars.is_empty() {
        chars.pop();
        let cand: String = chars.iter().collect::<String>() + ell;
        if text_system.layout(&cand, size, f32::INFINITY).width() <= budget {
            return cand;
        }
    }
    String::new()
}
