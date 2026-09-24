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
    InspectorScriptInfo, InspectorScriptProp, InspectorScriptValue,
};
use ph2d_editor_core::widget::SectionFold;
use ph2d_i18n::{tr, tr_with};

/// **A largura de um botão que DIZ o rótulo inteiro** — o texto medido no tamanho do botão e o recuo
/// dos dois lados.
///
/// ⛔ **Nasceu de uma FOTO da cena de smoke**, não de um gate: a 1.ª redacção dava ao `Reset` uma
/// largura fixa em passos de espaçamento, e no ecrã o botão lia-se `…` — o pintor elide o que não
/// cabe, e um botão sem palavra é um botão que ninguém sabe para que serve. ⚠️ A medida é a do
/// `title_elided_width` (o peso mais largo), pela lei escrita ao lado dela: *quem decide se cabe
/// chama isto, nunca o `prefix_width` cru*.
fn largura_do_botao(text_system: &mut TextSystem, id: NodeId, rotulo: &str) -> f32 {
    let b = Button::new(id, rotulo);
    ph2d_editor_core::text_elide::title_elided_width(text_system, rotulo, b.font_size())
        + 2.0 * b.padding()
}

/// Um valor pronto a ler numa linha de órfão.
fn legivel(v: &InspectorScriptValue) -> String {
    match v {
        InspectorScriptValue::Number(n) => ph2d_editor_core::interaction::format_number(*n),
        InspectorScriptValue::Bool(b) => b.to_string(),
        InspectorScriptValue::Text(t) => format!("\u{201c}{t}\u{201d}"),
    }
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

/// **Uma linha de propriedade** — o nome à esquerda, o controlo e o `Reset` à direita.
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
    seccao: ph2d_editor_core::property_row::Seccao,
) -> f32 {
    // ⭐⭐ **O nome fica À ESQUERDA do controlo, não por cima** (`line/UIUX`, report do dono de
    //    2026-09-14 e 15). ⚠️ A coluna é da SECÇÃO: os nomes vêm do script do artista, logo a
    //    lista mede-se do INFO — uma coluna derivada só desta linha saltaria de linha para linha.
    let row = super::rows::property_label_row(
        scene,
        text_system,
        theme,
        x,
        w,
        y,
        ALTURA_DE_CAMPO,
        &p.name,
        seccao,
    );
    let row_y = row.control.y;
    let gap = Spacing::Xs.px();
    let reset_w = largura_do_botao(
        text_system,
        ids::INSP_SCRIPT_RESET[i],
        tr("panel.inspector.script.reset"),
    );
    // ⚠️ **A coluna de animação reserva-se como em toda linha de formulário** (gate
    // `every_form_row_reserves_the_animation_column`): o controlo e o `Reset` cabem em `control_w`.
    //
    // ⚠️ **A CAIXA não leva o ponto daqui**: o `paint_checkbox` reserva e desenha a coluna dele
    // sozinho (é por isso que o gate não o lista entre os pintores de linha), e a 1.ª redacção
    // mostrava DOIS pontos na linha de um `boolean` — foi a foto da cena que o disse.
    // ⚠️ **A CAIXA não leva o ponto daqui**: o `paint_checkbox` reserva e desenha a coluna dele
    //    sozinho, e a 1.ª redacção mostrava DOIS pontos na linha de um `boolean`.
    let caixa = matches!(p.value, InspectorScriptValue::Bool(_));
    let (control_w, dot) = if caixa {
        (row.control.w + row.dot.w, None)
    } else {
        (row.control.w, Some(row.dot))
    };
    // ⛔⛔⛔ **O CAMPO e servido ANTES do `Reset` e, quando os dois nao cabem, o `Reset`
    // DESCE.**
    //
    // A 1.ª redaccao escrevia `control_w - reset_w - gap` com piso ZERO: o botao levava a largura
    // natural dele e o campo ficava com o resto. Medido em 2026-09-19 pela escada da varredura de
    // elisoes, com a coluna no minimo (`220 px`, onde o dono trabalha em cinco dos seis espacos) o
    // campo ficava com **`0,0 px`** e o valor (`"2"`) saia VAZIO — *um numero que nao se le
    // e um numero que nao existe dao o mesmo report*.
    //
    // ⚠️⚠️ **E dar o piso ao campo sozinho TROCOU DE VITIMA** (medido na mesma corrida): o
    // `Reset` passou a `1,0`–`2,0 px` e ficou ele em branco. *Repartir bem uma fileira MAL FORMADA
    // so troca de vitima* — a lei que esta casa pagou na grelha de enum do Motion.
    //
    // ⭐ **A saida e a lei da casa: o controlo REFLUI.** Quando `campo + vao + Reset` nao
    // cabe, o campo fica com a linha inteira e o botao desce para a seguinte. ⚠️ A altura e
    // devolvida por esta funcao (ela acumula), logo nao ha moldura pre-medida a contradizer.
    //
    // ⛔ **Esconder o `Reset` esta FORA:** um controlo cortado e mau, um controlo
    // inalcancavel e pior — e a capacidade de repor um valor proprio nao tem segunda porta.
    let piso_do_campo = ph2d_editor_core::widget::NUMBER_INPUT_MIN_W_PX.min(control_w);
    let reset_desce = p.own && control_w - reset_w - gap < piso_do_campo;
    let ctrl_w = if p.own && !reset_desce {
        control_w - reset_w - gap
    } else {
        control_w
    }
    .max(0.0);
    let reset_w = reset_w.min(control_w);
    let ctrl = Rect::new(row.control.x, row_y, ctrl_w, ALTURA_DE_CAMPO);
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
        let rect = if reset_desce {
            Rect::new(
                row.control.x,
                row_y + ALTURA_DE_CAMPO + gap,
                reset_w,
                ALTURA_DE_CAMPO,
            )
        } else {
            Rect::new(
                row.control.x + ctrl_w + gap,
                row_y,
                reset_w,
                ALTURA_DE_CAMPO,
            )
        };
        botao(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            rect,
            ids::INSP_SCRIPT_RESET[i],
            tr("panel.inspector.script.reset"),
        );
    }
    if let Some(dot) = dot {
        ph2d_editor_core::widget::paint_decorator_dot(scene, theme, dot);
    }
    // ⚠️ A altura conta a fileira que o refluxo de facto produziu — ver o bloco do `reset_desce`.
    let alturas = if reset_desce {
        ALTURA_DE_CAMPO * 2.0 + gap
    } else {
        ALTURA_DE_CAMPO
    };
    row_y + alturas + Spacing::Sm.px()
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
    let header = section_header(
        store,
        core_ids::INSP_LIVE_SCRIPT_SECTION,
        tr("panel.inspector.script.script"),
    )
    .color(rgba);
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
    // ── O FICHEIRO ───────────────────────────────────────────────────────────
    let seccao_do_ficheiro = ph2d_editor_core::property_row::Seccao::medida(
        text_system,
        1,
        &[tr("panel.inspector.script.source_label")],
    );
    cur_y = super::anim_rows::text_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.script.source_label"),
        ids::INSP_SCRIPT_SOURCE,
        TextInput::new(ids::INSP_SCRIPT_SOURCE, "")
            .placeholder(tr("panel.inspector.script.script_file_u")),
        seccao_do_ficheiro,
    );
    botao(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        Rect::new(x, cur_y, w, ALTURA_DE_CAMPO),
        ids::INSP_SCRIPT_BROWSE,
        tr("panel.inspector.script.browse"),
    );
    cur_y += ALTURA_DE_CAMPO + ph2d_tokens::control_gap_px();
    cur_y = super::script_avisos::avisos(scene, text_system, theme, x, w, cur_y, info);

    // ── OS NÚMEROS ───────────────────────────────────────────────────────────
    // ⭐⭐ **A coluna do nome é da SECÇÃO, e aqui os nomes são do SCRIPT DO ARTISTA** — mede-se a
    //    lista inteira uma vez; uma medida por linha seria uma coluna por linha (`line/UIUX`).
    let nomes: Vec<&str> = info.props.iter().map(|p| p.name.as_str()).collect();
    let seccao = if nomes.is_empty() {
        ph2d_editor_core::property_row::Seccao::apenas_campos(1)
    } else {
        ph2d_editor_core::property_row::Seccao::medida(text_system, 1, &nomes)
    };
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
            seccao,
        );
    }

    // ── OS ÓRFÃOS ────────────────────────────────────────────────────────────
    if !info.orphans.is_empty() {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr("panel.inspector.script.no_longer_in_the_script"),
            ColorToken::Warn,
        );
    }
    for (o, &id) in info
        .orphans
        .iter()
        .zip(ids::INSP_SCRIPT_ORPHAN_REMOVE.iter())
    {
        let porque = match o.wants {
            None => String::from(tr("panel.inspector.script.not_in_the_script")),
            Some(tipo) => tr_with(
                "panel.inspector.script.the_script_now_wants_a",
                &[("tipo", &tipo)],
            ),
        };
        let reset_w = largura_do_botao(text_system, id, tr("panel.inspector.script.remove"));
        let gap = Spacing::Xs.px();
        let texto = format!("{} = {} \u{2014} {porque}", o.name, legivel(&o.value));
        paint_text(
            text_system,
            scene,
            &texto,
            x,
            cur_y + (ALTURA_DE_CAMPO - TypeToken::Sm.px()) * 0.5,
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
            Rect::new(x + w - reset_w, cur_y, reset_w, ALTURA_DE_CAMPO),
            id,
            tr("panel.inspector.script.remove"),
        );
        cur_y += ALTURA_DE_CAMPO + ph2d_tokens::control_gap_px();
    }
    fold.finish(store, scene, hit_index, cur_y + SECTION_BOTTOM_PAD_PX)
}
