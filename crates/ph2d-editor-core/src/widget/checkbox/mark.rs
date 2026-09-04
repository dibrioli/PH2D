//! **O PINTOR de uma marca booleana** — a caixa de verificação e o interruptor, um desenho só.
//!
//! ⚠️ Este ficheiro nasceu de um **tecto de LOC**: o `checkbox.rs` passou os `500` quando a coluna
//! de animação entrou. O corte é por **responsabilidade**, não por linha: aqui mora *como um
//! booleano se desenha*; ao lado (`mod.rs`) mora *o que um booleano É* — estado, valor, construtor
//! e o nó de acessibilidade. ⛔ A casa não tolera folga de tecto: parte-se para um irmão.

use super::{CHECKBOX_BOX_PX, Checkbox, CheckboxState, CheckboxValue};
use crate::icons::IconId;
use crate::paint::{fill_rounded_rect, paint_icon, paint_text, resolve, stroke_rounded_rect};
use crate::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// ⭐⭐⭐ **O pintor de uma MARCA BOOLEANA — e desde 2026-09-03 há um só no app.**
///
/// Pinta a superfície da linha (que acende) e a caixa com o glifo, **encostada à direita**.
/// Devolve o rectângulo da caixa, para quem quiser desenhar à volta dela.
///
/// ⚠️ **Existe porque o INTERRUPTOR DESLIZANTE saiu** (decisão do Enio: *«as pílulas e o
/// interruptor deslizante podem sair»*). ⛔ Ele **não se apaga**: o `WidgetKind::Toggle` tem
/// `code() == 2` e esse número **viaja em documento** (`skin/kind.rs`), então um painel autorado
/// gravado ontem tem de continuar a abrir. ⇒ a fusão é **de PINTURA**, exactamente como a pesquisa
/// `07` §5.5 mandava: *o código velho fica a apontar para o pintor novo*.
///
/// ⚠️ **A semântica NÃO se funde:** o `Toggle` continua a ser `Role::Switch` para quem não vê, e o
/// `Checkbox` continua `Role::CheckBox`. *Fundir a tinta de dois controlos não os torna o mesmo
/// controlo* — e um leitor de ecrã que passasse a anunciar «caixa de verificação» onde o documento
/// diz «interruptor» estaria a mentir sobre o modelo.
/// O que a marca precisa de saber, e que **não** é geometria — o irmão exacto do
/// [`crate::widget::PropertyBox`].
///
/// ⚠️ Ele existe porque o clippy contou **oito** argumentos: *«too many arguments»* não é um limite
/// de estilo, é a pergunta *«estes parâmetros não serão um modelo?»* — e cinco destes descrevem o
/// mesmo booleano. O `Checkbox` e o `Toggle` preenchem-no cada um à sua maneira, que é o que os
/// mantém dois widgets com um pintor só.
#[derive(Copy, Clone, Debug)]
pub(crate) struct BooleanMark {
    pub value: CheckboxValue,
    pub state: CheckboxState,
    pub hover_t: f32,
    /// A aresta da caixa. `None` = o token — ver [`Checkbox::box_px`].
    pub box_px: Option<f32>,
    /// Reserva e desenha a coluna de animação. Ver [`Checkbox::decorator`].
    pub decorator: bool,
}

pub(crate) fn paint_boolean_mark(
    rect: Rect,
    m: BooleanMark,
    scene: &mut VectorScene,
    theme: Theme,
) -> Rect {
    let BooleanMark {
        value,
        state,
        hover_t,
        box_px,
        decorator,
    } = m;
    // A moldura é o TETO em qualquer dos dois casos (a caixa não transborda o que a contém);
    // o que `box_px` troca é a BASE — o token, ou o que o chamador mediu. `None` reduz à
    // expressão que shipava, ao bit.
    let box_size = box_px.unwrap_or(CHECKBOX_BOX_PX).min(rect.h);
    let box_y = rect.y + (rect.h - box_size) * 0.5;

    // ⭐⭐⭐ **A LINHA INTEIRA É O ALVO, e a marca vai para a DIREITA** (Enio, redesenho de
    // 2026-09; pesquisa `07` §5.2 e §15). O padrão é o do Blender/GNOME, e a razão aqui é o
    // ALINHAMENTO: a marca cai na **mesma coluna** em que a linha de propriedade põe o número
    // (`property_box::value_column`), então um formulário passa a ter **uma** margem direita em vez
    // de duas linguagens empilhadas.
    //
    // ⭐ **Isto é só PINTURA — nenhum chamador muda.** Medido em 2026-09-03: 17 de 19 chamadores
    // amostrados já registam `Rect::new(x, y, w, h)` — *a linha inteira já era o alvo de clique há
    // muito tempo; o que faltava era o desenho dizê-lo.* Os 2 restantes passam meia-linha (dois
    // checkboxes lado a lado), e encostar à direita da meia-linha é igualmente correcto.
    // ⚠️ **A coluna de animação vale para a linha de checkbox também** — senão o alinhamento que a
    // §16.1 comprou desfaz-se: as linhas de propriedade recuariam `14 px` e as de marcar não, e o
    // formulário voltaria a ter duas margens direitas.
    //
    // ⭐ **Quem NÃO a leva é a pele de canvas** — ali a moldura é o que o *artista* desenhou, não
    // uma linha de formulário, e uma bolinha de animação não significa nada. Ela di-lo por um campo
    // PRÓPRIO: ⛔ a 1.ª tentativa derivava-o de `box_px.is_none()` e partia o contrato daquele
    // campo (*«pedir o token é igual a não pedir nada»*), apanhada pelo gate na 1.ª corrida.
    // ⭐⭐⭐ **A APARÊNCIA escolhe a âncora** (Enio, 2026-09-03: *«por enquanto permanece a
    // antiga»*). No clássico a marca fica em `rect.x` e não há coluna de animação — é a linha que
    // o app pinta desde sempre, e é o caminho de OMISSÃO.
    let redesign = crate::paint::ui_is_redesign();
    let decorator = decorator && redesign;
    let box_rect = if redesign {
        Rect::new(
            crate::widget::property_box::value_column(rect, box_size, decorator).x,
            box_y,
            box_size,
            box_size,
        )
    } else {
        Rect::new(rect.x, box_y, box_size, box_size)
    };

    // ⭐ **A superfície da linha ACENDE**, e é ela que ensina que o alvo é largo. Emerge do nada
    // (o `hover_axis` com repouso `None` faz *fade*, como o botão *ghost*) ⇒ em repouso a linha
    // continua a não desenhar caixa nenhuma, que é a lei do §5.3: *dentro de um painel, molduras
    // não se desenham*.
    let row_hot = redesign
        && matches!(
            state,
            CheckboxState::Hovered | CheckboxState::Focused | CheckboxState::Pressed
        );
    if row_hot || (redesign && hover_t < crate::motion::SETTLED) {
        if let Some(c) = crate::motion::hover_axis(
            matches!(state, CheckboxState::Normal | CheckboxState::Hovered),
            hover_t,
            None,
            Some(ColorToken::Bg2.resolve(theme)),
        ) {
            fill_rounded_rect(
                scene,
                rect,
                crate::paint::slider_style().radius_px(),
                crate::paint::token_to_vello(c),
            );
        } else if row_hot {
            fill_rounded_rect(
                scene,
                rect,
                crate::paint::slider_style().radius_px(),
                resolve(ColorToken::Bg2, theme),
            );
        }
    }

    let radius = Radius::Xs.px();
    let (bg_token, border_token) = match (state, value) {
        (CheckboxState::Disabled, _) => (ColorToken::Bg2, ColorToken::Border),
        (_, CheckboxValue::Checked | CheckboxValue::Indeterminate) => {
            (ColorToken::Accent, ColorToken::Accent)
        }
        (CheckboxState::Hovered | CheckboxState::Focused, _) => {
            (ColorToken::Bg2, ColorToken::BorderEmph)
        }
        _ => (ColorToken::Bg1, ColorToken::Border),
    };
    // ⚠️ **O eixo do hover é o par NÃO-MARCADO** (`Bg1 → Bg2`, `Border → BorderEmph`): uma caixa
    //    MARCADA é `Accent` em qualquer estado, então ali não há eixo nenhum a percorrer. O
    //    `Focused` fica de fora com o `Disabled` — é estado duro, e o traço dele mede 2 px.
    let soft = matches!(state, CheckboxState::Normal | CheckboxState::Hovered)
        && matches!(value, CheckboxValue::Unchecked);
    let bg = crate::motion::hover_axis(
        soft,
        hover_t,
        Some(ColorToken::Bg1.resolve(theme)),
        Some(ColorToken::Bg2.resolve(theme)),
    )
    .map_or_else(|| resolve(bg_token, theme), crate::paint::token_to_vello);
    let border = crate::motion::hover_axis(
        soft,
        hover_t,
        Some(ColorToken::Border.resolve(theme)),
        Some(ColorToken::BorderEmph.resolve(theme)),
    )
    .map_or_else(
        || resolve(border_token, theme),
        crate::paint::token_to_vello,
    );
    fill_rounded_rect(scene, box_rect, radius, bg);
    stroke_rounded_rect(
        scene,
        box_rect,
        radius,
        if state == CheckboxState::Focused {
            2.0
        } else {
            1.0
        },
        border,
    );

    let glyph_color = if state == CheckboxState::Disabled {
        ColorToken::TextDisabled
    } else {
        ColorToken::AccentFg
    };
    let g = resolve(glyph_color, theme);
    match value {
        CheckboxValue::Checked => {
            paint_icon(scene, IconId::Check, box_rect, g, StrokeToken::Default.px())
        }
        // Indeterminate paints a horizontal dash (Minus glyph) — the
        // platform convention for "some children selected". Previous
        // code used `Plus` here, which painted a `+` and read as
        // "add" rather than "mixed".
        CheckboxValue::Indeterminate => paint_icon(scene, IconId::Minus, box_rect, g, 2.0),
        CheckboxValue::Unchecked => {}
    }

    if decorator {
        crate::widget::property_box::paint_decorator(
            scene,
            theme,
            crate::widget::property_box::decorator_rect(rect),
            state == CheckboxState::Disabled,
        );
    }

    box_rect
}

/// Square box + label. Box fills with `Accent` when Checked, paints a check glyph; Indeterminate
/// paints a dash glyph.
///
/// ⚠️ **A marca vai à DIREITA e o rótulo à esquerda** desde o redesenho de 2026-09 — ver
/// [`paint_boolean_mark`], que é onde a geometria vive.
pub fn paint_checkbox(
    cb: &Checkbox,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let box_rect = paint_boolean_mark(
        rect,
        BooleanMark {
            value: cb.value,
            state: cb.state,
            hover_t: cb.hover_t,
            box_px: cb.box_px,
            decorator: cb.decorator,
        },
        scene,
        theme,
    );

    if !cb.label.is_empty() {
        let label_color = if cb.state == CheckboxState::Disabled {
            ColorToken::TextDisabled
        } else {
            ColorToken::Text1
        };
        // ⚠️ **`Sm`, e não `Base`** — medido em 2026-09-03: a linha de propriedade escreve o
        // rótulo a `12 px` e esta escrevia a `13`. Num formulário as duas alternam, e **1 px de
        // corpo de letra entre linhas vizinhas lê-se como desalinho**, não como ênfase.
        // ⚠️ **No clássico volta a `Base`, à DIREITA da caixa** — é a linha de sempre.
        let redesign = crate::paint::ui_is_redesign();
        let font_size = if redesign {
            TypeToken::Sm.px()
        } else {
            TypeToken::Base.px()
        };
        let ly = rect.y + (rect.h - font_size) * 0.5;
        if redesign {
            let lx = rect.x + Spacing::Md.px();
            // O rótulo é o que CEDE — a mesma lei da caixa única, pela mesma função (§6.2).
            // ⛔ Nunca uma cópia: metade das linhas a truncar e a outra metade a transbordar é
            // pior que nenhuma das duas.
            let budget = (box_rect.x - lx - Spacing::Md.px()).max(0.0);
            let cut =
                crate::widget::property_box::fit_label(text_system, &cb.label, font_size, budget);
            if !cut.is_empty() {
                paint_text(
                    text_system,
                    scene,
                    &cut,
                    lx,
                    ly,
                    font_size,
                    f32::INFINITY,
                    resolve(label_color, theme),
                );
            }
        } else if rect.w > box_rect.w + Spacing::Md.px() {
            let lx = rect.x + box_rect.w + Spacing::Md.px();
            paint_text(
                text_system,
                scene,
                &cb.label,
                lx,
                ly,
                font_size,
                (rect.x + rect.w - lx).max(0.0),
                resolve(label_color, theme),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_a11y::NodeId;
    use ph2d_tokens::CHECKBOX_BOX_PX as CHROME_CHECKBOX_BOX;

    /// A caixa de partida dos testes de tinta — um sítio só.
    fn fixture() -> Checkbox {
        Checkbox::new(NodeId(1), "Snap to grid")
    }

    fn smoke(c: Checkbox, theme: Theme) {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        paint_checkbox(
            &c,
            Rect::new(0.0, 0.0, 200.0, 18.0),
            &mut scene,
            &mut text,
            theme,
        );
    }

    #[test]
    fn half_a_hover_moves_the_unchecked_box_between_the_two_ends() {
        use super::*;
        let theme = Theme::Forge;
        let rest = ColorToken::Bg1.resolve(theme);
        let hot = ColorToken::Bg2.resolve(theme);
        let mid = crate::motion::hover_axis(true, 0.5, Some(rest), Some(hot))
            .expect("o eixo macio mistura");
        assert_ne!(mid, rest);
        assert_ne!(mid, hot);
        // O neutro sai como `None` ⇒ o chamador cai no token DURO.
        assert!(
            crate::motion::hover_axis(true, crate::motion::SETTLED, Some(rest), Some(hot))
                .is_none()
        );
        // Estado duro (ou caixa MARCADA) não é uma quantidade: fora do eixo.
        assert!(crate::motion::hover_axis(false, 0.5, Some(rest), Some(hot)).is_none());
    }

    /// **Sem override, a caixa é o TOKEN** — a lei de todo painel do app, ao bit
    /// (BUGS_vector #26).
    ///
    /// ⚠️ `box_px: None` não é "um default razoável": é o que faz cada checkbox do app ter
    /// exactamente o mesmo tamanho, e é a razão de um formulário ler como formulário. Este gate
    /// existe para que mexer nisso exija mexer nele.
    #[test]
    fn without_an_override_the_box_is_the_token() {
        let c = fixture();
        assert_eq!(c.box_px, None, "o default deixou de ser o token");

        let tall = Rect::new(0.0, 0.0, 200.0, CHROME_CHECKBOX_BOX * 8.0);
        let mut a = VectorScene::new();
        let mut ts = TextSystem::without_system_fonts();
        paint_checkbox(&c, tall, &mut a, &mut ts, Theme::Forge);

        let mut explicit = c.clone();
        explicit.box_px = Some(CHROME_CHECKBOX_BOX);
        let mut b = VectorScene::new();
        paint_checkbox(&explicit, tall, &mut b, &mut ts, Theme::Forge);

        let (ea, eb) = (a.inner().encoding(), b.inner().encoding());
        assert_eq!(
            (ea.n_paths, ea.path_data.clone()),
            (eb.n_paths, eb.path_data.clone()),
            "pedir o proprio token divergiu de nao pedir nada — o canal nao e' neutro"
        );
    }

    #[test]
    fn paint_smoke_normal_unchecked() {
        smoke(fixture(), Theme::Forge);
    }

    #[test]
    fn paint_smoke_hovered_checked() {
        smoke(
            fixture()
                .value(CheckboxValue::Checked)
                .state(CheckboxState::Hovered),
            Theme::Sunstone,
        );
    }

    #[test]
    fn paint_smoke_pressed_indeterminate() {
        smoke(
            fixture()
                .value(CheckboxValue::Indeterminate)
                .state(CheckboxState::Pressed),
            Theme::Blueprint,
        );
    }

    #[test]
    fn paint_smoke_focused_unchecked() {
        smoke(fixture().state(CheckboxState::Focused), Theme::Workshop);
    }

    #[test]
    fn paint_smoke_disabled_checked() {
        smoke(
            fixture()
                .value(CheckboxValue::Checked)
                .state(CheckboxState::Disabled),
            Theme::Forge,
        );
    }
}
