//! Canonical "label + horizontal slider track + numeric chip"
//! composite — the form the BlenderColorPicker channel rows use,
//! extracted so the Inspector and any other slider site can share
//! the exact same visual + interaction surface.
//!
//! **Use this anywhere a slider with a value chip is needed.**
//! Don't roll a one-off `Slider + paint_number_input_with_buffer`
//! pair like the Inspector did pre-M13 — that was a recurring
//! source of "the slider in panel X looks different from the one
//! in panel Y" bugs. See `docs/UI_Bugs/README.md` §6.1.
//!
//! Two pieces:
//! - [`paint_slider_with_chip`] — the full row (label + track + chip),
//!   reads the slider's state + the chip's NumberInput state straight
//!   from the [`crate::interaction::WidgetStore`] and registers both
//!   sub-rect hits in the [`crate::interaction::HitIndex`].
//! - [`paint_number_chip`] — the standalone chip (interactive
//!   NumberInput-style: focus border + caret + buffer + selection +
//!   centered text). Used by `paint_slider_with_chip` and directly
//!   by callers that just want the chip on its own (e.g. the
//!   color-picker hue/V channels).
//!
//! Default layout: label_w=70 left, track in the middle, chip_w=60
//! right with a `Spacing::Sm.px()` gap on each side. Override via
//! [`paint_slider_with_chip_layout`] if a particular row needs a
//! wider chip or label.

use crate::icons::IconId;
use crate::interaction::{HitIndex, InteractiveState, WidgetStore};
use crate::paint::{fill_rounded_rect, paint_icon, paint_text_centered, resolve};
use crate::text_elide::paint_text_elided;
use crate::widget::TextInputState;
use crate::widget::number_input::{stepper_down_rect, stepper_up_rect, stepper_width};
use crate::widget::property_box::{PropertyBox, PropertyBoxState, paint_property_box};

mod classic;
mod number_chip;
use crate::zones::Rect;
pub use number_chip::{paint_number_chip, paint_number_chip_flat};
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::VectorScene;

pub const DEFAULT_LABEL_W: f32 = 70.0; // LITERAL-PX-OK: slider-with-chip default label column width (chrome-specific)
/// Default chip width — bumped to the canonical number-input minimum
/// post-2026-05-24 (vide [`crate::widget::number_input::MIN_W_PX`]).
/// Sized to fit 7 digits at Sm font + padding + stepper column.
pub const DEFAULT_CHIP_W: f32 = crate::widget::number_input::MIN_W_PX;

/// Paint a label + slider track + numeric chip composite using the
/// canonical layout. Both `slider_id` and `chip_id` register in the
/// hit index so the dispatch can route drag (slider) and click /
/// type (chip) separately.
///
/// `value` is the slider value in `[0..1]`; the chip displays the
/// same value formatted via [`crate::interaction::format_number`].
/// For displays that diverge from the slider's normalised value
/// (e.g. Inspector fields that show "160" for a slider at 0.62),
/// use [`paint_slider_with_chip_layout`] and pass `chip_value` /
/// `display_override` separately.
/// Returns the total vertical extent used: `rect.h` in the normal
/// (one-row) layout, or taller when the panel is narrow and the label
/// demotes to its own row above (see
/// [`paint_slider_with_chip_layout_adaptive`]). **Callers must advance
/// their y-cursor by the returned value**, not a fixed row height, so
/// the demoted label doesn't overlap the next control.
#[allow(clippy::too_many_arguments)]
pub fn paint_slider_with_chip(
    rect: Rect,
    label: &str,
    value: f32,
    slider_id: NodeId,
    chip_id: NodeId,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) -> f32 {
    paint_slider_with_chip_layout_adaptive(
        rect,
        label,
        value,
        value as f64,
        None,
        slider_id,
        chip_id,
        DEFAULT_LABEL_W,
        DEFAULT_CHIP_W,
        store,
        hit_index,
        scene,
        text_system,
        theme,
    )
}

/// Layout-flexible variant of [`paint_slider_with_chip`].
///
/// `chip_value` is the f64 the chip displays when not focused (and
/// what the painter formats via `format_number`). `display_override`
/// wins over `chip_value` when present — useful for Inspector fields
/// that display "160" instead of "0.62" for a slider at 62%.
#[allow(clippy::too_many_arguments)]
pub fn paint_slider_with_chip_layout(
    rect: Rect,
    label: &str,
    value: f32,
    chip_value: f64,
    display_override: Option<&str>,
    slider_id: NodeId,
    chip_id: NodeId,
    label_w: f32,
    chip_w: f32,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    // ⭐⭐⭐ **A CAIXA ÚNICA** (Enio, 2026-09-02) — rótulo à esquerda DENTRO, valor à direita DENTRO,
    // preenchimento a dizer a fracção. As três colunas de antes (rótulo `70` | trilho | caixa `72`)
    // eram `154 px` de cromo fixo, e a `PANEL_MIN_W = 220` a linha **empilhava**.
    //
    // ⚠️ **`label_w` deixou de ter consumidor, e é a decisão.** Ele era a coluna EXTERNA de rótulo —
    // a grandeza que a caixa única põe a zero. Os ~50 chamadores continuam a passá-lo e ele é
    // **ignorado de propósito**: mudar-lhes a assinatura para o apagar seria 50 diffs para exprimir
    // uma decisão que já está exprimida aqui. ⛔ Não o reaproveite para outra coisa — um parâmetro
    // que muda de significado é pior que um que não faz nada.
    // ⭐⭐⭐ **A APARÊNCIA escolhe o pintor** (Enio, 2026-09-03: *«por enquanto permanece a
    // antiga»*). ⚠️ O `Classic` é o caminho de OMISSÃO — quem não liga `PH2D_UI_NEW=1` vê a linha
    // de sempre, e é isso que deixa esta linha entrar no `main` com o redesenho a meio.
    if !crate::paint::ui_is_redesign() {
        classic::paint_classic_row(
            rect,
            label,
            value,
            chip_value,
            display_override,
            slider_id,
            chip_id,
            label_w,
            chip_w,
            store,
            hit_index,
            scene,
            text_system,
            theme,
        );
        return;
    }
    let _ = label_w;
    let style = crate::paint::slider_style();

    // O estado da caixa sai de DOIS sítios, e a ordem é a lei: **escrever ganha a arrastar**.
    // Enquanto o campo tem foco, a caixa é um campo de texto — pintar `Dragging` por baixo de um
    // cursor a piscar diria duas coisas ao mesmo tempo.
    let (chip_state, chip_buffer, chip_caret, chip_anchor) = match store.get(chip_id) {
        Some(InteractiveState::NumberInput {
            state,
            buffer,
            caret,
            selection_anchor,
            ..
        }) => (*state, Some(buffer.as_str()), *caret, *selection_anchor),
        _ => (TextInputState::Normal, None, 0, None),
    };
    let (slider_state, _) = store.slider_visual(slider_id);
    let box_state = if chip_state == TextInputState::Focused {
        PropertyBoxState::Editing
    } else {
        match slider_state {
            crate::widget::SliderState::Dragging => PropertyBoxState::Dragging,
            crate::widget::SliderState::Hovered => PropertyBoxState::Hovered,
            crate::widget::SliderState::Disabled => PropertyBoxState::Disabled,
            _ => PropertyBoxState::Normal,
        }
    };

    // ⚠️ `value: ""` + `value_w: Some(chip_w)` ⇒ a caixa **reserva e não pinta** a coluna do valor.
    // Quem pinta lá é o campo numérico REAL, logo abaixo: cursor, selecção, recorte do texto e as
    // **setinhas** continuam a ser os que o app sempre teve. ⛔ Reimplementá-los aqui seria a
    // segunda cópia de um campo de texto.
    let b = PropertyBox {
        label,
        value: "",
        t: value,
        state: box_state,
        accent: ColorToken::Accent,
        // ⭐⭐⭐ **LIGADA** (Enio, 2026-09-03: *«a bolinha de animação — só desenhá-la»*).
        //
        // ⚠️ A premissa que a mantinha desligada — *«esta função não sabe se a propriedade é
        // animável»* — pressupunha que a resposta VARIA. A decisão do dono é a outra
        // (*«vou querer animar tudo»*), e uma constante não precisa de ser consultada.
        //
        // ⚠️ **Custa 14 px em toda linha do app**, e é por isso que esperou por ele.
        decorator: crate::widget::property_box::FORM_ROWS_SHOW_DECORATOR,
        value_w: Some(chip_w),
    };

    if slider_id.0 != 0 {
        // ⭐⭐⭐ **O alvo do arrasto é a SUPERFÍCIE — o MESMO rect que o preenchimento atravessa.**
        //
        // ⚠️ A 1.ª redacção registava *a caixa menos a coluna do valor*, por um raciocínio que
        // parecia óbvio (*«ali em cima o clique é para escrever, logo não é para arrastar»*) e que
        // está errado: o despacho não usa este rect só para decidir **se** o gesto é meu — ele
        // divide por `rect.w` para saber **quanto** vale. Registar um rect mais estreito do que o
        // que se pinta multiplica todo valor por `w/(w−84)` ⇒ a tinta corre à frente do dedo.
        // Mecanismo e a tabela: [`property_box::surface_rect`].
        //
        // ⭐ O clique-para-escrever continua a funcionar sem carvar nada, e por uma propriedade do
        // `HitIndex`: ele resolve em **ordem inversa de registo**, então o chip — registado LOGO
        // ABAIXO — ganha dentro da coluna do valor. ⚠️ A ordem é load-bearing; trocá-la faz o
        // número deixar de ser editável em todo o app.
        //
        // ⚠️ E o topo da faixa continua alcançável: durante o arrasto o ponteiro pode sair pela
        // direita e o `clamp` entrega `1.0` — é o que o Blender faz.
        hit_index.register(
            slider_id,
            crate::widget::property_box::surface_rect(rect, b.decorator),
        );
    }

    let chip_rect = paint_property_box(scene, text_system, theme, rect, b, style);

    // O campo numérico REAL na coluna que a caixa reservou — **sem superfície própria**, senão
    // seria uma caixa dentro da caixa, que é exactamente o cromo que este redesenho apaga.
    paint_number_chip_flat(
        chip_rect,
        chip_state,
        chip_value,
        display_override,
        chip_buffer,
        chip_caret,
        chip_anchor,
        scene,
        text_system,
        theme,
    );
    if chip_id.0 != 0 {
        hit_index.register(chip_id, chip_rect);
    }
}

/// **Onde o CHIP numérico desta row cai** — o mesmo retângulo que o painter usa e regista.
///
/// ⚠️ Existe para quem precisa DESENHAR POR CIMA do chip (a rachura de *"um token cobre este
/// valor"*, plano UI/UX W4c.4) sem re-derivar a conta. Uma segunda expressão para *"onde está o
/// chip?"* divergiria no dia em que a row empilhasse — e a marca apareceria ao lado do número em
/// vez de sobre ele, num painel estreito.
#[must_use]
pub fn slider_with_chip_chip_rect(rect: Rect, label_w: f32, chip_w: f32) -> Rect {
    // ⚠️ **A aparência escolhe, como no pintor** — senão a marca de *"um token cobre este valor"*
    // aparece onde o número NÃO está. É a mesma pergunta, e ela tem duas respostas desde que o
    // redesenho passou a ser opcional.
    if !crate::paint::ui_is_redesign() {
        let row = if slider_with_chip_is_stacked(rect.w, label_w, chip_w) {
            Rect::new(
                rect.x,
                rect.y + rect.h + crate::widget::panel_chrome::SECTION_LABEL_TO_CONTROL_PX,
                rect.w,
                rect.h,
            )
        } else {
            rect
        };
        return Rect::new(row.x + row.w - chip_w, row.y, chip_w, row.h);
    }
    let _ = label_w;
    crate::widget::property_box::value_column(
        rect,
        chip_w,
        crate::widget::property_box::FORM_ROWS_SHOW_DECORATOR,
    )
}

/// Whether [`paint_slider_with_chip_layout_adaptive`] will STACK (demote the label to its own row) at
/// `content_w` for the given `label_w` / `chip_w` — the same threshold the painter uses. Lets a container
/// that must size a background BEFORE painting the adaptive rows (e.g. the Jitter card) agree exactly.
#[must_use]
pub fn slider_with_chip_is_stacked(content_w: f32, label_w: f32, chip_w: f32) -> bool {
    // ⚠️ **O CLÁSSICO empilha, o redesenho nunca.** Nove sítios perguntam isto para saber a altura
    // que uma linha vai ocupar — se a resposta não seguir a aparência, as linhas do caminho de
    // omissão desenham-se **umas por cima das outras**.
    if !crate::paint::ui_is_redesign() {
        let needed = label_w + chip_w + Spacing::Sm.px() * 2.0 + CLASSIC_MIN_TRACK_W;
        return content_w < needed;
    }
    let _ = (content_w, label_w, chip_w);
    false
}

/// A trilha mínima antes de o rótulo descer — o piso de cromo da linha CLÁSSICA.
const CLASSIC_MIN_TRACK_W: f32 = 60.0; // LITERAL-PX-OK: piso de cromo da linha classica

/// **A largura mínima em que a caixa única ainda diz alguma coisa.**
///
/// ⚠️ Ela existe porque a pergunta que o `is_stacked` respondia — *«cabe?»* — continua legítima; o
/// que mudou foi a resposta a *«e se não couber?»*: antes empilhava, agora o **rótulo trunca** e,
/// no limite, desaparece e fica só o número (a escada do estreito, `pesquisa/07` §6.1).
///
/// ⇒ o piso é *a coluna do valor mais uma folga de cada lado*. Abaixo disto a linha já não é uma
/// linha de propriedade — é um número solto.
#[must_use]
pub fn slider_with_chip_min_w(chip_w: f32) -> f32 {
    // ⛔⛔ **A APARÊNCIA decide, e esta função é a que quase escapou.** Ela substituiu, no
    // `chrome::input_map`, a fórmula que o `main` usava (`cw + Sm*2 + 60`), e sob a UI clássica
    // devolvia `cw + Md*2` — **56 px mais estreita**. A janela do Input Map encolhia sem ninguém
    // ter pedido, no caminho de omissão.
    //
    // ⚠️ **O censo dos pintores não a apanhou**, porque ela não pinta: é uma RÉGUA, e uma régua
    // trocada muda o produto sem passar por um pintor. *Trocar a régua de um caminho é mudá-lo,
    // mesmo sem lhe tocar na tinta.*
    if !crate::paint::ui_is_redesign() {
        return chip_w + Spacing::Sm.px() * 2.0 + CLASSIC_MIN_TRACK_W;
    }
    chip_w + Spacing::Md.px() * 2.0
}

/// The vertical extent [`paint_slider_with_chip`] will use at `content_w` — `row_h` on one row, else the
/// stacked (label-demoted) height. Uses the default label/chip widths (the `paint_slider_with_chip` form).
#[must_use]
pub fn slider_with_chip_height(row_h: f32, content_w: f32) -> f32 {
    // ⭐ **UMA linha, sempre.** Era este o preço que a estreiteza cobrava: à `PANEL_MIN_W = 220` a
    // linha antiga empilhava e gastava `2 × row_h`, e a altura é o recurso mais escasso de um
    // painel de tablet. ⚠️ Derivado do `is_stacked` de propósito — se um dia a caixa voltar a ter
    // um modo empilhado, esta função segue-o sem ninguém se lembrar dela.
    if slider_with_chip_is_stacked(content_w, DEFAULT_LABEL_W, DEFAULT_CHIP_W) {
        row_h + crate::widget::panel_chrome::SECTION_LABEL_TO_CONTROL_PX + row_h
    } else {
        row_h
    }
}

/// Adaptive variant of [`paint_slider_with_chip_layout`] — when the
/// label + slider + chip won't fit horizontally inside `rect.w`, the
/// label demotes to its own row ABOVE the slider+chip row (the
/// slider+chip then takes the full width on the lower row).
///
/// UI canon 2026-05-24 (user: "vamos tornar todos os sliders
/// adaptáveis à largura do painel. Na largura padrão fica como está
/// mas no momento em que o painel fica mais estreito, a label dos
/// slider passa para linha de cima. Como fizemos com as caixas de
/// texto"). Mirrors the demote-on-narrow pattern in
/// [`crate::widget::panel_chrome::paint_segmented_group_adaptive`].
///
/// Returns the total vertical extent used (`rect.h` in horizontal
/// mode; `rect.h * 2 + SECTION_LABEL_TO_CONTROL_PX` in stacked mode).
/// Callers advance their y-cursor by the returned value.
#[allow(clippy::too_many_arguments)]
pub fn paint_slider_with_chip_layout_adaptive(
    rect: Rect,
    label: &str,
    value: f32,
    chip_value: f64,
    display_override: Option<&str>,
    slider_id: NodeId,
    chip_id: NodeId,
    label_w: f32,
    chip_w: f32,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) -> f32 {
    // Demote the label when the row would crush the slider below its chrome floor — shared with
    // `slider_with_chip_height` so a container pre-sizing a background agrees exactly.
    if !slider_with_chip_is_stacked(rect.w, label_w, chip_w) {
        paint_slider_with_chip_layout(
            rect,
            label,
            value,
            chip_value,
            display_override,
            slider_id,
            chip_id,
            label_w,
            chip_w,
            store,
            hit_index,
            scene,
            text_system,
            theme,
        );
        return rect.h;
    }
    // Stacked: label on top row, slider+chip on bottom row at full width.
    let font = TypeToken::Base.px();
    let label_row_h = rect.h;
    paint_text_elided(
        text_system,
        scene,
        label,
        rect.x,
        rect.y + (label_row_h - font) * 0.5,
        font,
        rect.w,
        resolve(ColorToken::Text1, theme),
    );
    let lower_y = rect.y + label_row_h + crate::widget::panel_chrome::SECTION_LABEL_TO_CONTROL_PX;
    let lower_rect = Rect::new(rect.x, lower_y, rect.w, rect.h);
    paint_slider_with_chip_layout(
        lower_rect,
        "", // label already painted above
        value,
        chip_value,
        display_override,
        slider_id,
        chip_id,
        0.0, // give all width to slider+chip
        chip_w,
        store,
        hit_index,
        scene,
        text_system,
        theme,
    );
    label_row_h + crate::widget::panel_chrome::SECTION_LABEL_TO_CONTROL_PX + rect.h
}
