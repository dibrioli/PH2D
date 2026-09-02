//! ⭐⭐⭐ **O CATÁLOGO DE DESENHOS da caixa única** — e o pintor que os desenha.
//!
//! O Enio decidiu em 2026-09-01: *"A caixa única é o alvo!"* — uma linha de propriedade deixa de
//! ser `rótulo | trilho | caixa numérica` (154 px de cromo fixo, medido em
//! `docs/UI_New_and_Simple/pesquisa/07`) e passa a ser **uma caixa** com o rótulo à esquerda
//! dentro, o valor à direita dentro, e o preenchimento a dizer a fracção.
//!
//! ⚠️ **Isto é a BANCADA, não o produto.** Nada aqui é chamado pelo app: o laboratório existe
//! para o desenho ser escolhido **antes** de os 162 sítios de chamada serem tocados. É por isso
//! que este pintor é auto-contido e não reaproveita o `slider_with_chip` — ele é justamente o que
//! está a ser substituído, e herdar a geometria dele importaria a decisão que se quer refazer.
//!
//! # As seis famílias, e o que cada uma TROCA
//!
//! Nenhuma é melhor em abstracto; cada uma paga por uma coisa diferente. A coluna que interessa é
//! a última — *o que se perde*.
//!
//! | # | nome | o preenchimento é | ganha | perde |
//! |---|---|---|---|---|
//! | 1 | `Bar` | **o fundo inteiro** (Blender) | leitura de relance a qualquer largura | o valor exacto lê-se mal quando a barra passa por baixo do número |
//! | 2 | `Underline` | uma linha de 2 px em baixo | o texto nunca compete com a barra | a fracção fica discreta — pior de relance |
//! | 3 | `Inset` | uma cápsula dentro de um sulco | parece um controlo, não um rótulo | gasta 6 px de altura em molduras |
//! | 4 | `Ghost` | fundo a 12 % + linha de base | o mais plano de todos; some numa lista longa | quase não se vê que é arrastável |
//! | 5 | `Notch` | fundo + **marca vertical** no valor | recupera a precisão que a `Bar` perde | mais um elemento a desenhar por linha |
//! | 6 | `Split` | fundo, com o rótulo **FORA** à esquerda | rótulo nunca colide com valor | volta a ter coluna fixa — é o meio-termo, não a caixa única |
//!
//! ⚠️ **A `Split` está aqui de propósito como CONTROLO NEGATIVO.** Ela é o desenho do Blender (duas
//! colunas), e a decisão do Enio foi ir mais longe. Tê-la lado a lado é o que torna a decisão
//! verificável em vez de lembrada.

use ph2d_editor_core::paint::{fill_rounded_rect, paint_text, resolve, stroke_rounded_rect};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// Quantas caixas de estudo o laboratório conhece.
pub const DESIGN_COUNT: usize = 6;

/// Um desenho da caixa única.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum BoxDesign {
    /// O preenchimento é o fundo inteiro. É o do manual do Blender.
    #[default]
    Bar,
    /// Fundo chapado; o preenchimento é uma linha fina no bordo de baixo.
    Underline,
    /// Uma cápsula de preenchimento dentro de um sulco recuado.
    Inset,
    /// Sem caixa: preenchimento muito ténue e uma linha de base.
    Ghost,
    /// `Bar` mais uma marca vertical na posição exacta do valor.
    Notch,
    /// Rótulo FORA à esquerda; a caixa carrega só o número. ⚠️ Controlo negativo.
    Split,
}

impl BoxDesign {
    pub const ALL: [BoxDesign; DESIGN_COUNT] = [
        BoxDesign::Bar,
        BoxDesign::Underline,
        BoxDesign::Inset,
        BoxDesign::Ghost,
        BoxDesign::Notch,
        BoxDesign::Split,
    ];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            BoxDesign::Bar => "Bar",
            BoxDesign::Underline => "Underline",
            BoxDesign::Inset => "Inset",
            BoxDesign::Ghost => "Ghost",
            BoxDesign::Notch => "Notch",
            BoxDesign::Split => "Split",
        }
    }

    /// A linha que o laboratório imprime ao lado do desenho — **o que ele troca**, não o que ele é.
    #[must_use]
    pub const fn blurb(self) -> &'static str {
        match self {
            BoxDesign::Bar => {
                "o fundo e' a barra \u{b7} le-se de relance \u{b7} o numero compete com ela"
            }
            BoxDesign::Underline => {
                "barra de 2 px em baixo \u{b7} texto limpo \u{b7} fraccao discreta"
            }
            BoxDesign::Inset => "capsula num sulco \u{b7} parece controlo \u{b7} gasta altura",
            BoxDesign::Ghost => "o mais plano \u{b7} some numa lista \u{b7} mal se ve' que arrasta",
            BoxDesign::Notch => {
                "barra + marca no valor \u{b7} recupera a precisao \u{b7} +1 elemento"
            }
            BoxDesign::Split => "rotulo FORA \u{b7} nunca colide \u{b7} volta a ter coluna fixa",
        }
    }

    #[must_use]
    pub fn next(self) -> Self {
        let i = Self::ALL.iter().position(|d| *d == self).unwrap_or(0);
        Self::ALL[(i + 1) % DESIGN_COUNT]
    }

    #[must_use]
    pub fn prev(self) -> Self {
        let i = Self::ALL.iter().position(|d| *d == self).unwrap_or(0);
        Self::ALL[(i + DESIGN_COUNT - 1) % DESIGN_COUNT]
    }

    /// **A largura que o rótulo gasta FORA da caixa.** Zero para toda a família da caixa única —
    /// é literalmente a grandeza que a decisão do Enio põe a zero, e a `Split` é a única que não.
    #[must_use]
    pub const fn outer_label_w(self) -> f32 {
        match self {
            BoxDesign::Split => 56.0, // LITERAL-PX-OK: coluna de rotulo do controlo negativo
            _ => 0.0,
        }
    }
}

/// Em que estado a amostra é pintada. ⚠️ Estes são os estados que o Enio pediu para **ver**
/// ("vários comportamentos"), não uma máquina de estados — o laboratório pinta-os lado a lado.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum BoxState {
    #[default]
    Normal,
    Hovered,
    Dragging,
    Disabled,
    /// A escrever — a caixa virou campo de texto (o gesto do Blender: clicar edita).
    Editing,
}

impl BoxState {
    pub const ALL: [BoxState; 5] = [
        BoxState::Normal,
        BoxState::Hovered,
        BoxState::Dragging,
        BoxState::Disabled,
        BoxState::Editing,
    ];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            BoxState::Normal => "normal",
            BoxState::Hovered => "hover",
            BoxState::Dragging => "drag",
            BoxState::Disabled => "off",
            BoxState::Editing => "typing",
        }
    }
}

/// Tudo o que o pintor precisa de saber que **não** é a geometria. Um struct em vez de dez
/// argumentos: o `clippy::too_many_arguments` já mordeu esta linha duas vezes.
#[derive(Copy, Clone, Debug)]
pub struct BoxStyle {
    pub design: BoxDesign,
    pub state: BoxState,
    /// A cor do preenchimento.
    pub accent: ColorToken,
    /// Raio do canto, em px. O laboratório cicla-o de propósito — `16` é o de hoje, `4` o do Godot.
    pub radius: f32,
    /// Desenha a coluna de animação à direita (o *decorator*).
    pub decorator: bool,
}

/// A largura da coluna de animação. ⭐ Enio: *"em todas as propriedades que podem ser animadas, e
/// nessa engine vou querer animar tudo"* — logo ela é permanente, e a largura dela sai de todas as
/// linhas do painel. É por isso que ela é MEDIDA aqui e não escolhida em cada sítio.
pub const DECORATOR_W: f32 = 14.0; // LITERAL-PX-OK: coluna de animacao (o decorator do Blender)

/// Abaixo disto a `Split` **desiste da coluna de rótulo** e comporta-se como as outras cinco.
///
/// ⚠️ É o degrau 1 da escada do estreito aplicado ao próprio controlo negativo: mesmo o desenho
/// que existe para gastar uma coluna tem de a largar quando não sobra caixa. *Sem isto a `Split`
/// desenha um rótulo de 56 px ao lado de uma caixa de 4 e a comparação da §2 mede um artefacto.*
const SPLIT_MIN_BOX_W: f32 = 40.0; // LITERAL-PX-OK: piso da caixa antes de a Split largar a coluna

/// Quantos `pad` separam o fim do rótulo do início do valor: um de cada bordo, mais **um** entre
/// os dois. ⚠️ É uma CONTAGEM de folgas, não uma medida — o px vem do `Spacing::Md`.
const PAD_UNITS_BETWEEN_LABEL_AND_VALUE: f32 = 3.0; // LITERAL-PX-OK: CONTAGEM de folgas, nao px

/// ⭐⭐⭐ **O PINTOR DA CAIXA ÚNICA.**
///
/// `t` é a fracção `0..1`; `value` é o texto já formatado (com unidade, se houver — a decisão do
/// §5.1 da pesquisa: a unidade entra no texto do valor, não numa coluna).
///
/// ⚠️ **O rótulo é o que CEDE.** Se não couber, ele trunca; o valor **nunca** trunca, porque um
/// número cortado é um número errado (pesquisa §6.2). É a inversão exacta do widget de hoje, onde
/// o rótulo tem 70 px fixos e o trilho é que encolhe até desaparecer.
#[allow(clippy::too_many_arguments)]
pub fn paint_box(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    rect: Rect,
    label: &str,
    value: &str,
    t: f32,
    style: BoxStyle,
) {
    let t = t.clamp(0.0, 1.0);
    let disabled = style.state == BoxState::Disabled;
    let pad = Spacing::Md.px();

    // A coluna de animação sai da direita ANTES de tudo — ela é do formulário, não da caixa.
    let (rect, deco) = if style.decorator {
        let w = rect.w - DECORATOR_W;
        (
            Rect::new(rect.x, rect.y, w.max(1.0), rect.h),
            Some(Rect::new(rect.x + w, rect.y, DECORATOR_W, rect.h)),
        )
    } else {
        (rect, None)
    };

    // A `Split` reserva a coluna de rótulo por fora; as outras cinco não.
    let outer = style.design.outer_label_w();
    let (label_out, box_rect) = if outer > 0.0 && rect.w > outer + SPLIT_MIN_BOX_W {
        (
            Some(Rect::new(rect.x, rect.y, outer - Spacing::Xs.px(), rect.h)),
            Rect::new(rect.x + outer, rect.y, rect.w - outer, rect.h),
        )
    } else {
        (None, rect)
    };

    let fg = if disabled {
        ColorToken::TextDisabled
    } else {
        ColorToken::Text1
    };
    let accent = if disabled {
        ColorToken::Border
    } else {
        style.accent
    };

    paint_surface(scene, theme, box_rect, t, style, accent);

    // ── O texto ────────────────────────────────────────────────────────────
    let size = TypeToken::Sm.px();
    let value_w = text_system.layout(value, size, f32::INFINITY).width();
    let text_h = text_system.layout(value, size, f32::INFINITY).height();
    let ty = box_rect.y + (box_rect.h - text_h) * 0.5;

    // O valor, encostado à direita. NUNCA trunca.
    let vx = box_rect.x + box_rect.w - pad - value_w;
    paint_text(
        text_system,
        scene,
        value,
        vx,
        ty,
        size,
        f32::INFINITY,
        resolve(fg, theme),
    );

    // O rótulo: dentro (as cinco) ou fora (a `Split`).
    if let Some(lo) = label_out {
        let cut = fit_label(text_system, label, size, lo.w);
        paint_text(
            text_system,
            scene,
            &cut,
            lo.x,
            ty,
            size,
            f32::INFINITY,
            resolve(ColorToken::Text2, theme),
        );
    } else {
        // ⚠️ O orçamento do rótulo é o que sobra DEPOIS do valor — e é aqui que a lei do estreito
        // vive. Uma folga de `pad` entre os dois impede que se toquem antes de truncar.
        let budget = (box_rect.w - pad * PAD_UNITS_BETWEEN_LABEL_AND_VALUE - value_w).max(0.0);
        let cut = fit_label(text_system, label, size, budget);
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
    }

    if style.state == BoxState::Editing {
        // O cursor de escrita, colado ao valor.
        let caret = Rect::new(vx + value_w + 1.0, box_rect.y + 5.0, 1.0, box_rect.h - 10.0); // LITERAL-PX-OK: caret do estudo
        fill_rounded_rect(scene, caret, 0.0, resolve(ColorToken::Accent, theme));
    }

    if let Some(d) = deco {
        paint_decorator(scene, theme, d, disabled);
    }
}

/// A superfície — é aqui que os seis desenhos divergem, e **só** aqui.
fn paint_surface(
    scene: &mut VectorScene,
    theme: Theme,
    r: Rect,
    t: f32,
    style: BoxStyle,
    accent: ColorToken,
) {
    let rad = style.radius;
    let hot = matches!(style.state, BoxState::Hovered | BoxState::Dragging);
    let trough = if hot {
        ColorToken::Bg2
    } else {
        ColorToken::Bg3
    };
    let fill_w = (r.w * t).max(0.0);

    match style.design {
        BoxDesign::Bar | BoxDesign::Notch | BoxDesign::Split => {
            fill_rounded_rect(scene, r, rad, resolve(trough, theme));
            if fill_w > 0.5 {
                let f = Rect::new(r.x, r.y, fill_w, r.h);
                fill_rounded_rect(scene, f, rad, resolve(accent, theme));
            }
            if style.design == BoxDesign::Notch && t > 0.0 {
                let x = r.x + fill_w - 1.0;
                let n = Rect::new(x, r.y, 2.0, r.h); // LITERAL-PX-OK: marca do valor, 2 px
                fill_rounded_rect(scene, n, 0.0, resolve(ColorToken::Text1, theme));
            }
        }
        BoxDesign::Underline => {
            fill_rounded_rect(scene, r, rad, resolve(trough, theme));
            let h = 2.0; // LITERAL-PX-OK: espessura da linha de valor
            let base = Rect::new(r.x, r.y + r.h - h, r.w, h);
            fill_rounded_rect(scene, base, 0.0, resolve(ColorToken::Border, theme));
            if fill_w > 0.5 {
                let f = Rect::new(r.x, r.y + r.h - h, fill_w, h);
                fill_rounded_rect(scene, f, 0.0, resolve(accent, theme));
            }
        }
        BoxDesign::Inset => {
            fill_rounded_rect(scene, r, rad, resolve(trough, theme));
            let m = 3.0; // LITERAL-PX-OK: recuo do sulco
            let inner = Rect::new(
                r.x + m,
                r.y + m,
                (r.w - m * 2.0).max(0.0),
                (r.h - m * 2.0).max(0.0),
            );
            let iw = inner.w * t;
            if iw > 0.5 {
                let f = Rect::new(inner.x, inner.y, iw, inner.h);
                fill_rounded_rect(scene, f, (rad - m).max(0.0), resolve(accent, theme));
            }
        }
        BoxDesign::Ghost => {
            if fill_w > 0.5 {
                let f = Rect::new(r.x, r.y, fill_w, r.h);
                fill_rounded_rect(scene, f, rad, resolve(ColorToken::AccentSoft, theme));
            }
            let h = 1.0; // LITERAL-PX-OK: linha de base do desenho mais plano
            let base = Rect::new(r.x, r.y + r.h - h, r.w, h);
            fill_rounded_rect(scene, base, 0.0, resolve(ColorToken::Border, theme));
        }
    }

    if style.state == BoxState::Editing {
        stroke_rounded_rect(
            scene,
            r,
            rad,
            StrokeToken::Thin.px(),
            resolve(ColorToken::Accent, theme),
        );
    } else if style.state == BoxState::Hovered {
        stroke_rounded_rect(
            scene,
            r,
            rad,
            StrokeToken::Hairline.px(),
            resolve(ColorToken::BorderStrong, theme),
        );
    }
}

/// A coluna de animação — o *decorator* do Blender, que o Enio pediu para **todas** as
/// propriedades animáveis.
///
/// ⚠️ Aqui ela é só o estado «animável, sem chave» (o ponto vazio). Os outros estados do Blender
/// — losango cheio (chave neste quadro), losango vazio (chave noutro), ícone de driver — são
/// trabalho a seguir, e **precisam da timeline**, não de desenho.
fn paint_decorator(scene: &mut VectorScene, theme: Theme, r: Rect, disabled: bool) {
    let d = 4.0; // LITERAL-PX-OK: diametro do ponto de animacao
    let dot = Rect::new(r.x + (r.w - d) * 0.5, r.y + (r.h - d) * 0.5, d, d);
    let c = if disabled {
        ColorToken::TextDisabled
    } else {
        ColorToken::Text3
    };
    fill_rounded_rect(scene, dot, d * 0.5, resolve(c, theme));
}

/// **Trunca o rótulo para caber, com reticências.**
///
/// ⚠️ Devolve string VAZIA quando nem duas letras cabem — e isso é uma resposta, não uma falha: a
/// caixa fica só com o número, que é o degrau seguinte da escada do estreito (pesquisa §6.1).
///
/// ⏳ **A alternativa é o ESBATIMENTO** (`Scene::push_luminance_mask_layer`, zero consumidores hoje):
/// em vez de `…`, o rótulo desvanece nos últimos px. É mais bonito e não come letras — e está
/// nomeado na pesquisa §7.3 como o próximo eixo desta bancada, com o custo por medir.
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
