//! ⭐⭐⭐ **A secção TAGS** (TOP-20 #9, W3a) — a que o objecto É, e não o que ele tem.
//!
//! # ⚠️ Ela nasce COM a wave, e isso é a lição que o `Timers` custou
//!
//! O `Timers` shipou anexável e sem linha de edição, e o report do dono foi *«timer sumiu do modal
//! de componente»*. *Um componente anexável sem painel é indistinguível de um que não foi anexado.*
//!
//! # ⭐⭐ UM campo, DUAS perguntas
//!
//! O campo de texto é a BUSCA **e** o nome da tag nova: escrever `Fly` estreita a lista e arma o
//! `Create "Fly"`. Dois campos fariam o artista escrever o nome duas vezes só para descobrir que a
//! tag já existia — que é exactamente o gesto que a busca existe para evitar.
//!
//! ⚠️ **A busca dobra pela porta da ÁRVORE** ([`ph2d_label_fold::fold`]) — a decisão do dono é
//! *«maiúscula não importa, letra acentuada não importa»*, e um `to_lowercase` aqui seria a segunda
//! resposta à mesma pergunta: `Énemy` deixaria de ser encontrado por `enemy` no painel enquanto a
//! árvore continuava a tratá-los como a mesma tag.
//!
//! # ⚠️ O que a lista NÃO oferece
//!
//! As tags que o objecto **já tem**. Escolher uma delas seria um gesto que o `Tags::insert` recusa,
//! e o painel nunca oferece o que vai ser recusado — é a mesma lei que esconde a caixa toda quando
//! o objecto chega ao [cap](ph2d_panel_inspector_cap) e diz porquê no lugar dela.
//!
//! [ph2d_panel_inspector_cap]: crate::ids::INSP_TAGS_CHIP
//!
//! # ⛔ E o que sobra da lista é DITO
//!
//! A lista mostra no máximo [`crate::ids::INSP_TAGS_OPT`] opções (`64`, medido). Quando a busca
//! encontra mais, a linha por baixo conta as que ficaram de fora — *um chooser que esconde metade
//! dos resultados em silêncio ensina que a tag não existe*.

use super::*;
use ph2d_editor_core::screens::hero::{InspectorTagRow, InspectorTagsInfo};
use ph2d_editor_core::widget::{
    Dropdown, DropdownOption, SectionFold, Tag, TagState, TagTone, paint_dropdown_chip, paint_tag,
};
use ph2d_i18n::{tr, tr_with};

const BTN_H: f32 = 30.0; // LITERAL-PX-OK: altura de botão do Inspector, igual à das irmãs
/// A linha de uma lista é a linha do app — pela porta, nunca por um literal que coincide.
const ROW_H: f32 = ph2d_tokens::ROW_H_PX;

/// **A nuvem de chips**, quebrada em linhas. Devolve o `y` seguinte.
///
/// ⚠️ **`zip` com o array de ids**: um objecto com mais tags do que ids (impossível enquanto o gate
/// `the_tag_chip_ids_cover_the_model_cap` viver) perde os excedentes em vez de os pintar uns sobre
/// os outros.
#[allow(clippy::too_many_arguments)]
fn chips(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    rows: &[InspectorTagRow],
) -> f32 {
    let gap = Spacing::Xs.px();
    let mut cx = x;
    let mut cy = y;
    for (row, &id) in rows.iter().zip(crate::ids::INSP_TAGS_CHIP.iter()) {
        let state = match store.get(id) {
            Some(InteractiveState::Tag { state }) => *state,
            _ => TagState::Normal,
        };
        // ⚠️ **O par `(estado, hover)` pela porta `visual`**, e não dois setters: o `WidgetStore`
        // não tem `tag_visual()` como tem `button_visual()` — o `Tag` foi escrito com a porta e
        // nunca teve consumidor fora do showcase para lhe pedir o atalho.
        let chip = Tag::new(id, row.label.clone())
            .tone(TagTone::Accent)
            .removable(true)
            .visual((state, store.hover_live(id)));
        // ⚠️ **O chip mostra o RÓTULO e o balão o caminho inteiro** — `Flying` cabe numa nuvem,
        // `Enemy/Ground/Flying` não; e é o rótulo que o artista reconhece.
        //
        // ⛔⛔ **A largura sai da PÍLULA, e esta linha era a segunda cópia da geometria dela.**
        // A cópia daqui somava `2,5·pad + ×` ao texto e o pintor descontava, por cima disso, o
        // respiro de uma caixa de rótulo ⇒ **todo chip desta secção era cortado a metade**
        // (`Ground` pede `38,95` e recebia `22,95`). ⚠️ E a varredura de elisões do app **não o
        // via**: um Inspector de fábrica não tem objecto seleccionado, logo não pinta um único
        // chip. *Uma lei escrita em dois sítios ainda não é uma lei — só uma PORTA é.*
        let cw = chip.natural_width(text_system, ROW_H).min(w);
        if cx > x && cx + cw > x + w {
            cx = x;
            cy += ROW_H + gap;
        }
        let rect = Rect::new(cx, cy, cw, ROW_H);
        paint_tag(&chip, rect, scene, text_system, theme);
        // ⭐⭐ **Só o `×` é hit-registado, e é isso que separa os dois gestos**: carregar no corpo
        // do chip não faz nada (ainda não há *ir para a tag*), carregar no `×` tira-a. Registar a
        // pílula inteira faria todo clique distraído desmarcar o objecto.
        if let Some(close_r) = chip.close_rect(rect) {
            hit_index.register(id, close_r);
        }
        cx += cw + gap;
    }
    cy + ph2d_tokens::row_pitch_px()
}

/// **As opções da caixa** — as tags do projecto que o objecto ainda não tem, filtradas por `filtro`
/// (já DOBRADO pelo chamador).
///
/// ⚠️ **`pub(crate)` porque o passe diferido a re-deriva**: o popover pinta-se depois de todas as
/// secções, e a lei dele é *guardar o que não se pode rederivar, rederivar o resto*. As opções saem
/// do snapshot e do store, que ele tem — só o rect do chip é que não.
///
/// ⚠️ **A indentação é a PROFUNDIDADE**, e é o que faz a lista ler-se como a árvore que é.
pub(crate) fn pick_options(
    arvore: &[InspectorTagRow],
    no_objecto: &[InspectorTagRow],
    filtro: &str,
) -> Vec<DropdownOption<u64>> {
    tags_que_faltam(arvore, no_objecto, filtro)
        .zip(crate::ids::INSP_TAGS_OPT.iter())
        .map(|(row, &id)| {
            let recuo = "    ".repeat(row.depth);
            DropdownOption::new(id, row.id, format!("{recuo}{}", row.label))
        })
        .collect()
}

/// As candidatas, na ordem da árvore: as que o objecto não tem e que a busca aceita.
///
/// ⚠️ **A comparação é sobre o CAMINHO inteiro**, não sobre o rótulo: escrever `enemy` tem de
/// trazer `Enemy/Flying`, senão procurar por uma família não acha os membros dela.
fn tags_que_faltam<'a>(
    arvore: &'a [InspectorTagRow],
    no_objecto: &'a [InspectorTagRow],
    filtro: &'a str,
) -> impl Iterator<Item = &'a InspectorTagRow> {
    arvore.iter().filter(move |row| {
        !no_objecto.iter().any(|t| t.id == row.id)
            && (filtro.is_empty() || ph2d_label_fold::fold(&row.path).contains(filtro))
    })
}

/// Quantas candidatas a busca encontrou — para a linha que conta as que não couberam.
fn quantas_faltam(
    arvore: &[InspectorTagRow],
    no_objecto: &[InspectorTagRow],
    filtro: &str,
) -> usize {
    tags_que_faltam(arvore, no_objecto, filtro).count()
}

/// **O texto que o artista escreveu no campo**, tal como está.
fn texto(store: &WidgetStore) -> String {
    match store.get(crate::ids::INSP_TAGS_NEW) {
        Some(InteractiveState::TextInput { text, .. }) => text.clone(),
        _ => String::new(),
    }
}

/// A linha da caixa de escolha. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
fn pick_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    arvore: &[InspectorTagRow],
    no_objecto: &[InspectorTagRow],
    filtro: &str,
) -> f32 {
    let (control_w, dot) = ph2d_editor_core::widget::form_row_columns(x, w, y, ROW_H);
    let rect = Rect::new(x, y, control_w, ROW_H);
    hit_index.register(crate::ids::INSP_TAGS_PICK, rect);
    let open = matches!(
        store.get(crate::ids::INSP_TAGS_PICK),
        Some(InteractiveState::Dropdown { open: true, .. })
    );
    let dd = Dropdown::new(
        crate::ids::INSP_TAGS_PICK,
        "",
        pick_options(arvore, no_objecto, filtro),
    )
    .placeholder(ph2d_i18n::tr("panel.tags.pick"))
    .open(open)
    .visual(store.dropdown_visual(crate::ids::INSP_TAGS_PICK));
    paint_dropdown_chip(&dd, rect, scene, text_system, theme);
    // ⚠️ **O popover NÃO se pinta aqui** — ele sairia debaixo da secção seguinte. O rect vai ao
    // slot e o passe diferido do painel desenha-o por cima de tudo.
    if open {
        crate::state_popovers::set_pending_tags_dd(Some(rect));
    }
    ph2d_editor_core::widget::paint_decorator_dot(scene, theme, dot);
    y + ph2d_tokens::row_pitch_px()
}

/// A secção inteira — cabeçalho, dobra e corpo. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_tags_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorTagsInfo,
) -> f32 {
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: banda do cabeçalho
    let color_id = core_ids::INSP_LIVE_TAGS_COLOR;
    let rgba = store
        .widget_color(color_id)
        .unwrap_or([0x88, 0x88, 0x88, 0xff]); // LITERAL-COLOR-OK: acento neutro por omissão
    let title = if info.on_object.is_empty() {
        String::from(tr("panel.inspector.tags.tags"))
    } else {
        tr_with(
            "panel.inspector.tags.tags_count",
            &[("n", &info.on_object.len())],
        )
    };
    let header = section_header(store, core_ids::INSP_LIVE_TAGS_SECTION, &title).color(rgba);
    let header_rect = Rect::new(x, y, w, header_h);
    paint_section_header(&header, header_rect, scene, text_system, theme);
    if let Some(circle_rect) = ph2d_editor_core::widget::color_circle_hit_rect(&header, header_rect)
    {
        hit_index.register(color_id, circle_rect);
    }
    let Some(fold) = SectionFold::begin(
        store,
        core_ids::INSP_LIVE_TAGS_SECTION,
        x,
        w,
        y + header_h,
        scene,
        hit_index,
    ) else {
        return y + header_h;
    };
    let mut cur_y = y + header_h;
    let font = TypeToken::Sm.px();

    // ⚠️ **A SELEÇÃO MÚLTIPLA tem de se dizer** — a lei das irmãs. Aqui a razão é outra que a
    // delas (um `TagId` significa o mesmo em toda a cena, logo espalhar seria exprimível): ela
    // não se espalha porque *marcar N objectos de uma vez* é um gesto que o painel não desenha,
    // e um efeito invisível é pior que um botão ausente.
    if info.selected_count > 1 {
        cur_y = aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            font,
            tr(
                "panel.inspector.tags.multiple_selected_u_tag_edits_apply_to_the_active_object_only",
            ),
            ColorToken::Warn,
        );
    }

    if info.on_object.is_empty() {
        cur_y = aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            font,
            tr("panel.inspector.tags.no_tags_yet"),
            ColorToken::Text3,
        );
    } else {
        cur_y = chips(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            &info.on_object,
        );
    }

    if info.full {
        // ⛔ **Cheio ⇒ a caixa SAI e o porquê fica no lugar dela.** Deixá-la ali a recusar cada
        // escolha ensinaria que o painel está avariado.
        cur_y = aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            font,
            tr(
                "panel.inspector.tags.this_object_holds_the_most_tags_the_section_can_show_remove_one_to_add_another",
            ),
            ColorToken::Warn,
        );
        return fold.finish(store, scene, hit_index, cur_y + SECTION_BOTTOM_PAD_PX);
    }

    let escrito = texto(store);
    let filtro = ph2d_label_fold::fold(&escrito);
    // ⭐ A árvore vem da porta do DOCUMENTO, não deste instantâneo — ver o cabeçalho do modelo.
    let arvore = crate::state::current_tag_tree();
    cur_y = pick_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        &arvore,
        &info.on_object,
        &filtro,
    );
    // ⛔ **O que não coube é CONTADO** — ver o cabeçalho.
    let achadas = quantas_faltam(&arvore, &info.on_object, &filtro);
    if achadas > crate::ids::INSP_TAGS_OPT.len() {
        cur_y = aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            font,
            &tr_with(
                "panel.inspector.tags.showing_of_type_to_narrow",
                &[
                    ("n", &crate::ids::INSP_TAGS_OPT.len()),
                    ("achadas", &achadas),
                ],
            ),
            ColorToken::Text3,
        );
    }

    cur_y = super::anim_rows::text_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        crate::ids::INSP_TAGS_NEW,
        TextInput::new(crate::ids::INSP_TAGS_NEW, "")
            .placeholder(ph2d_i18n::tr("panel.tags.new_or_search")),
    );

    // ⭐ O `Create` só existe quando há um nome que ainda não é de ninguém — oferecê-lo sobre uma
    // tag que já existe faria o artista pensar que criou uma segunda com o mesmo nome.
    let ja_existe = !filtro.is_empty()
        && arvore
            .iter()
            .any(|r| ph2d_label_fold::fold(&r.path) == filtro);
    if !escrito.trim().is_empty() && !ja_existe {
        let rect = Rect::new(x, cur_y, w, BTN_H);
        hit_index.register(crate::ids::INSP_TAGS_CREATE, rect);
        paint_button(
            &Button::new(
                crate::ids::INSP_TAGS_CREATE,
                tr_with(
                    "panel.inspector.tags.create_named",
                    &[("nome", &escrito.trim())],
                ),
            )
            .kind(ButtonKind::Default)
            .visual(store.button_visual(crate::ids::INSP_TAGS_CREATE)),
            rect,
            scene,
            text_system,
            theme,
        );
        cur_y += BTN_H + ph2d_tokens::control_gap_px();
    }

    fold.finish(store, scene, hit_index, cur_y + SECTION_BOTTOM_PAD_PX)
}

/// Uma linha de texto de aviso. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
fn aviso(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    x: f32,
    w: f32,
    y: f32,
    font: f32,
    texto: &str,
    cor: ColorToken,
) -> f32 {
    paint_text(
        text_system,
        scene,
        texto,
        x,
        y,
        font,
        w,
        resolve(cor, theme),
    );
    y + font + ph2d_tokens::control_gap_px()
}
