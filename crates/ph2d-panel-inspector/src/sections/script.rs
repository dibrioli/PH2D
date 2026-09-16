//! ⭐⭐⭐ **A secção SCRIPT** — o ficheiro que um objecto corre e os NÚMEROS que ele oferece
//! (TOP-20 #16, W3). Plano: `docs/Components/13_plano_script_properties.md`.
//!
//! # A forma é a do `@export` do Godot, e não a do cérebro
//!
//! **Uma linha por propriedade declarada, cada uma com o seu controlo** — o artista vê todos os
//! números do script de uma vez, que é o que toda engine madura faz. O tipo de cada controlo é o do
//! DEFAULT declarado (`number` → campo numérico · `boolean` → caixa · `string` → campo de texto).
//!
//! # ⭐⭐ O que a linha DIZ
//!
//! - **O NOME em cor de acento** quando o valor é PRÓPRIO — o artista pô-lo neste objecto — e um
//!   `Reset` ao lado que o larga. ⚠️ É a divergência D1 do oráculo: próprio é quem PÔS, nunca *«difere
//!   do default»*, então um número igual ao default também tem `Reset`.
//! - **Os ÓRFÃOS por baixo** — valores próprios cuja propriedade saiu do script (ou mudou de tipo),
//!   cada um com `Remove`. ⚠️ A divergência D2: o alvo apaga-os em silêncio ao re-gravar.
//! - **Com o ficheiro sumido ou partido, os valores ficam e NÃO se oferecem para apagar** — *não sei o
//!   que o script declara* não é *o script não declara nada*.

use super::*;
use ph2d_editor_core::script_edits::{
    InspectorScriptInfo, InspectorScriptProp, InspectorScriptStatus, InspectorScriptValue,
};
use ph2d_editor_core::widget::SectionFold;

const BTN_H: f32 = 30.0; // LITERAL-PX-OK: altura de botão do Inspector, igual à das irmãs
/// A altura do controlo de uma linha — a do campo de número das irmãs.
const FIELD_H: f32 = 22.0; // LITERAL-PX-OK: altura do NumberInput, a das secções irmãs
/// A largura do `Reset`, em múltiplos do passo de espaçamento — o botão diz uma palavra curta.
const RESET_W_STEPS: f32 = 8.0; // LITERAL-PX-OK: múltiplo do token de espaçamento

/// Um valor pronto a ler numa linha de órfão.
fn legivel(v: &InspectorScriptValue) -> String {
    match v {
        InspectorScriptValue::Number(n) => ph2d_editor_core::interaction::format_number(*n),
        InspectorScriptValue::Bool(b) => b.to_string(),
        InspectorScriptValue::Text(t) => format!("\u{201c}{t}\u{201d}"),
    }
}

/// Uma nota de uma linha. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
fn nota(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    x: f32,
    w: f32,
    y: f32,
    texto: &str,
    token: ColorToken,
) -> f32 {
    let font = TypeToken::Sm.px();
    paint_text(
        text_system,
        scene,
        texto,
        x,
        y,
        font,
        w,
        resolve(token, theme),
    );
    y + font + ph2d_tokens::control_gap_px()
}

/// Um botão de largura inteira ou à direita. Regista e pinta.
#[allow(clippy::too_many_arguments)]
fn botao(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    rect: Rect,
    id: NodeId,
    rotulo: &str,
) {
    hit_index.register(id, rect);
    paint_button(
        &Button::new(id, rotulo)
            .kind(ButtonKind::Default)
            .visual(store.button_visual(id)),
        rect,
        scene,
        text_system,
        theme,
    );
}

/// **Uma linha de propriedade** — o nome por cima, o controlo e o `Reset` por baixo.
#[allow(clippy::too_many_arguments)]
fn linha(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    i: usize,
    p: &InspectorScriptProp,
) -> f32 {
    let font = TypeToken::Sm.px();
    let label_h = font + Spacing::Xs.px();
    let cor = if p.own {
        ColorToken::Accent
    } else {
        ColorToken::Text2
    };
    paint_text(
        text_system,
        scene,
        &p.name,
        x,
        y,
        font,
        w,
        resolve(cor, theme),
    );
    let row_y = y + label_h;
    let gap = Spacing::Xs.px();
    let reset_w = Spacing::Xs.px() * RESET_W_STEPS;
    // ⚠️ **A coluna de animação reserva-se como em toda linha de formulário** (gate
    // `every_form_row_reserves_the_animation_column`): o controlo e o `Reset` cabem em `control_w`.
    let (control_w, dot) = ph2d_editor_core::widget::form_row_columns(x, w, row_y, FIELD_H);
    let ctrl_w = if p.own {
        control_w - reset_w - gap
    } else {
        control_w
    }
    .max(0.0);
    let ctrl = Rect::new(x, row_y, ctrl_w, FIELD_H);
    match &p.value {
        InspectorScriptValue::Number(_) => {
            let id = ids::INSP_SCRIPT_NUM[i];
            hit_index.register(id, ctrl);
            let (state, value, buffer, caret, anchor) = read_number_input(store, id);
            let input = NumberInput::new(id, "", value)
                .step(p.step.unwrap_or(1.0))
                .visual((state, store.hover_live(id)));
            paint_number_input_with_buffer(
                &input,
                Some(buffer),
                caret,
                anchor,
                ctrl,
                scene,
                text_system,
                theme,
            );
        }
        InspectorScriptValue::Bool(_) => {
            let id = ids::INSP_SCRIPT_BOOL[i];
            hit_index.register(id, ctrl);
            let (_, value) = store
                .checkbox(id)
                .unwrap_or((CheckboxState::Normal, CheckboxValue::Unchecked));
            paint_checkbox(
                &Checkbox::new(id, "")
                    .visual(store.checkbox_visual(id))
                    .value(value),
                ctrl,
                scene,
                text_system,
                theme,
            );
        }
        InspectorScriptValue::Text(_) => {
            let id = ids::INSP_SCRIPT_TEXT[i];
            hit_index.register(id, ctrl);
            let (state, text, caret, anchor) = match store.get(id) {
                Some(InteractiveState::TextInput {
                    state,
                    text,
                    caret,
                    selection_anchor,
                }) => (*state, Some(text.as_str()), *caret, *selection_anchor),
                _ => (TextInputState::Normal, None, 0, None),
            };
            let input = TextInput::new(id, "").visual((state, store.hover_live(id)));
            paint_text_input_with_buffer(
                &input,
                text,
                Some(caret),
                anchor,
                ctrl,
                scene,
                text_system,
                theme,
            );
        }
    }
    if p.own {
        let rect = Rect::new(x + ctrl_w + gap, row_y, reset_w, FIELD_H);
        botao(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            rect,
            ids::INSP_SCRIPT_RESET[i],
            "Reset",
        );
    }
    ph2d_editor_core::widget::paint_decorator_dot(scene, theme, dot);
    row_y + FIELD_H + Spacing::Sm.px()
}

/// Os avisos do ficheiro e da corrida. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
fn avisos(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorScriptInfo,
) -> f32 {
    let mut y = y;
    // ⚠️ **Os avisos vêm ANTES dos números**, pela razão da secção do som: quem não vê nada a mexer
    // não quer afinar um número — quer saber porquê.
    match &info.status {
        InspectorScriptStatus::NoFile => {
            y = nota(
                scene,
                text_system,
                theme,
                x,
                w,
                y,
                "No script file yet \u{2014} use Browse to pick a .luau file.",
                ColorToken::Text3,
            );
        }
        InspectorScriptStatus::Unavailable => {
            y = nota(
                scene,
                text_system,
                theme,
                x,
                w,
                y,
                "Scripting is not available in this session.",
                ColorToken::Danger,
            );
        }
        InspectorScriptStatus::Loading => {
            y = nota(
                scene,
                text_system,
                theme,
                x,
                w,
                y,
                "Reading the file\u{2026}",
                ColorToken::Text3,
            );
        }
        InspectorScriptStatus::Missing => {
            y = nota(
                scene,
                text_system,
                theme,
                x,
                w,
                y,
                "That file is gone \u{2014} pick it again.",
                ColorToken::Danger,
            );
        }
        InspectorScriptStatus::Broken(msg) => {
            y = nota(
                scene,
                text_system,
                theme,
                x,
                w,
                y,
                &format!("The script has an error: {msg}"),
                ColorToken::Danger,
            );
        }
        InspectorScriptStatus::Ready => {
            if info.props.is_empty() {
                y = nota(
                    scene,
                    text_system,
                    theme,
                    x,
                    w,
                    y,
                    "This script offers no properties.",
                    ColorToken::Text3,
                );
            }
        }
    }
    if let Some(msg) = &info.failure {
        y = nota(
            scene,
            text_system,
            theme,
            x,
            w,
            y,
            &format!("Stopped: {msg} \u{2014} fix the script and save it."),
            ColorToken::Danger,
        );
    }
    if info.kept > 0 {
        y = nota(
            scene,
            text_system,
            theme,
            x,
            w,
            y,
            &format!("{} value(s) kept until the script loads again.", info.kept),
            ColorToken::Text3,
        );
    }
    if info.also_physics {
        y = nota(
            scene,
            text_system,
            theme,
            x,
            w,
            y,
            "This object is also moved by physics \u{2014} the two fight.",
            ColorToken::Warn,
        );
    }
    if !info.clock_playing {
        y = nota(
            scene,
            text_system,
            theme,
            x,
            w,
            y,
            "The clock is stopped \u{2014} scripts only run while the scene plays.",
            ColorToken::Text3,
        );
    }
    y
}

/// A secção inteira — cabeçalho, dobra e corpo. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_script_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorScriptInfo,
) -> f32 {
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: banda do cabeçalho
    let color_id = core_ids::INSP_LIVE_SCRIPT_COLOR;
    let rgba = store
        .widget_color(color_id)
        .unwrap_or([0x88, 0x88, 0x88, 0xff]); // LITERAL-COLOR-OK: acento neutro por omissão
    let header = section_header(store, core_ids::INSP_LIVE_SCRIPT_SECTION, "Script").color(rgba);
    let header_rect = Rect::new(x, y, w, header_h);
    paint_section_header(&header, header_rect, scene, text_system, theme);
    if let Some(circle_rect) = ph2d_editor_core::widget::color_circle_hit_rect(&header, header_rect)
    {
        hit_index.register(color_id, circle_rect);
    }
    let Some(fold) = SectionFold::begin(
        store,
        core_ids::INSP_LIVE_SCRIPT_SECTION,
        x,
        w,
        y + header_h,
        scene,
        hit_index,
    ) else {
        return y + header_h;
    };
    let mut cur_y = y + header_h;
    if info.selected_count > 1 {
        cur_y = nota(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            "Multiple selected \u{b7} edits apply to the active object only.",
            ColorToken::Warn,
        );
    }
    // ── O FICHEIRO ───────────────────────────────────────────────────────────
    cur_y = super::anim_rows::text_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        ids::INSP_SCRIPT_SOURCE,
        TextInput::new(ids::INSP_SCRIPT_SOURCE, "").placeholder("script file\u{2026}"),
    );
    botao(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        Rect::new(x, cur_y, w, BTN_H),
        ids::INSP_SCRIPT_BROWSE,
        "Browse",
    );
    cur_y += BTN_H + ph2d_tokens::control_gap_px();
    cur_y = avisos(scene, text_system, theme, x, w, cur_y, info);

    // ── OS NÚMEROS ───────────────────────────────────────────────────────────
    // ⚠️ `zip` com a tabela de ids: o script não pode declarar mais do que ela tem (o
    // `ph2d_script::PROPS_MAX`), e o gate da shell afirma que os dois são o mesmo número.
    for (i, p) in info
        .props
        .iter()
        .enumerate()
        .take(ids::INSP_SCRIPT_NUM.len())
    {
        cur_y = linha(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            i,
            p,
        );
    }

    // ── OS ÓRFÃOS ────────────────────────────────────────────────────────────
    if !info.orphans.is_empty() {
        cur_y = nota(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            "Values this object keeps that the script no longer offers:",
            ColorToken::Warn,
        );
    }
    for (o, &id) in info
        .orphans
        .iter()
        .zip(ids::INSP_SCRIPT_ORPHAN_REMOVE.iter())
    {
        let porque = match o.wants {
            None => String::from("not in the script"),
            Some(tipo) => format!("the script now wants a {tipo}"),
        };
        let reset_w = Spacing::Xs.px() * RESET_W_STEPS;
        let gap = Spacing::Xs.px();
        let texto = format!("{} = {} \u{2014} {porque}", o.name, legivel(&o.value));
        paint_text(
            text_system,
            scene,
            &texto,
            x,
            cur_y + (BTN_H - TypeToken::Sm.px()) * 0.5,
            TypeToken::Sm.px(),
            (w - reset_w - gap).max(0.0),
            resolve(ColorToken::Text2, theme),
        );
        botao(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            Rect::new(x + w - reset_w, cur_y, reset_w, BTN_H),
            id,
            "Remove",
        );
        cur_y += BTN_H + ph2d_tokens::control_gap_px();
    }
    fold.finish(store, scene, hit_index, cur_y + SECTION_BOTTOM_PAD_PX)
}
