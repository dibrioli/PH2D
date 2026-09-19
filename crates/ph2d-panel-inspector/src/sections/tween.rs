//! ⭐⭐⭐ **A secção TWEEN** (suplente #22) — *«esta propriedade vai de A a B»*.
//!
//! ⚠️ **Irmã da [`super::timers`], e o molde é o dela pela mesma razão:** um `Tweens` é uma LISTA
//! cujo índice **é a identidade** (é ele que liga o tween ao timer do mesmo índice), e cada tween
//! tem cinco campos. ⇒ a **lista** escolhe qual está aberto, e um editor só, abaixo dela, mostra os
//! campos desse.
//!
//! # ⭐⭐ A QUEIXA vem antes dos números
//!
//! | aviso | o que se passa |
//! |---|---|
//! | `no timer at this slot` | ⛔ **o tween não tem relógio** — ele nem chega a correr |
//! | `this object has no sprite` | o canal escreve num campo do `Sprite`, e não há nenhum |
//! | `from and to are the same` | ele corre e **não move nada** |
//! | `a silhouette that holds stays lit` | ⚠️ funciona, e quase de certeza não é o que se quer |
//!
//! ⚠️ **Os dois primeiros são de outra espécie que os dois últimos:** ali o tween **não corre**,
//! aqui ele corre. *Dizer «ele não move nada» a quem não tem relógio é mandá-lo resolver a metade
//! errada* — a lei da recusa dos pincéis.
//!
//! ⛔⛔ **A ordem NÃO vive aqui**, e é isso que a torna testável: ela é a porta
//! [`InspectorTweenRow::queixa`], e o gate dela corre **sem um device**.
//!
//! # ⚠️ A curva é onze botões, e por isso são TRÊS fileiras
//!
//! A coluna do Inspector tem ~300 px; onze numa fileira dão ~25 px cada, e um rótulo que não cabe é
//! um chip que o artista não lê. *O corte não é do modelo — as `33` curvas do motor continuam todas
//! alcançáveis —, é da LARGURA.*

use super::*;
use ph2d_editor_core::tween_edits::{InspectorTweenInfo, InspectorTweenRow, TweenQueixa};
use ph2d_editor_core::widget::SectionFold;
use ph2d_i18n::{tr, tr_with};
use ph2d_tween::{AoAcabar, Canal};

const BTN_H: f32 = 30.0; // LITERAL-PX-OK: altura de botão do Inspector, igual à da irmã
/// A linha de uma lista é a linha do app — pela porta, nunca por um literal que coincide.
const ROW_H: f32 = ph2d_tokens::ROW_H_PX;
/// Quantos chips cabem numa fileira da coluna do Inspector — ver o cabeçalho.
const CHIPS_POR_FILEIRA: usize = 4;

/// **A CHAVE de cada queixa — a PORTA, e não um `match` dentro do pintor.**
///
/// ⛔ Ela traduz o enum da lei numa chave de i18n, e é o único sítio onde as duas coisas se tocam:
/// a lei não conhece a língua e o pintor não decide a ordem.
#[must_use]
const fn chave_da_queixa(q: TweenQueixa) -> &'static str {
    match q {
        TweenQueixa::SemRelogio => "panel.inspector.tween.no_timer_at_this_slot",
        TweenQueixa::SemSprite => "panel.inspector.tween.this_object_has_no_sprite",
        TweenQueixa::Inerte => "panel.inspector.tween.from_and_to_are_the_same",
        TweenQueixa::SilhuetaQueFica => "panel.inspector.tween.a_silhouette_that_holds_stays_lit",
    }
}

/// Uma linha de aviso. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
fn warn(
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

/// **Uma fileira de chips**, de `ids[de..ate]`. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
fn fileira(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    ids_: &[NodeId],
    labels: &[&str],
    sel: usize,
    base: usize,
) -> f32 {
    let gap = Spacing::Xs.px();
    #[allow(clippy::cast_precision_loss)]
    let n = ids_.len().max(1) as f32;
    let cw = ((w - gap * (n - 1.0)) / n).max(0.0);
    for (i, (&id, text)) in ids_.iter().zip(labels.iter()).enumerate() {
        #[allow(clippy::cast_precision_loss)]
        let rect = Rect::new(x + (cw + gap) * i as f32, y, cw, ROW_H);
        hit_index.register(id, rect);
        // ⚠️ **A selecção vem do SNAPSHOT**, nunca do store: o store guarda o visual do botão, e
        // ler dali qual está aceso faria o realce sobreviver à troca de objecto.
        let kind = if base + i == sel {
            ButtonKind::Accent
        } else {
            ButtonKind::Default
        };
        paint_button(
            &Button::new(id, *text)
                .kind(kind)
                .visual(store.button_visual(id)),
            rect,
            scene,
            text_system,
            theme,
        );
    }
    // ⚠️ **O passo de uma linha vem da PORTA** — há gate contra a segunda resposta.
    y + ph2d_tokens::row_pitch_px()
}

/// **Um grupo de chips com rótulo**, partido em fileiras de [`CHIPS_POR_FILEIRA`].
#[allow(clippy::too_many_arguments)]
fn grupo(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    label: &str,
    ids_: &[NodeId],
    labels: &[&str],
    sel: usize,
) -> f32 {
    let font = TypeToken::Sm.px();
    paint_text(
        text_system,
        scene,
        label,
        x,
        y,
        font,
        w,
        resolve(ColorToken::Text2, theme),
    );
    let mut cur_y = y + font + Spacing::Xs.px();
    for (bloco, chunk) in ids_.chunks(CHIPS_POR_FILEIRA).enumerate() {
        let base = bloco * CHIPS_POR_FILEIRA;
        cur_y = fileira(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            chunk,
            &labels[base..base + chunk.len()],
            sel,
            base,
        );
    }
    // ⚠️ **A cauda de um bloco vem da PORTA**, e a última fileira já trouxe o passo dela.
    cur_y + ph2d_tokens::control_gap_px()
}

/// A lista. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
fn list(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorTweenInfo,
    selected: usize,
) -> f32 {
    let font = TypeToken::Sm.px();
    let mut cur_y = y;
    // ⚠️ **`zip` com o array de ids**: uma lista com mais tweens do que ids perde os excedentes em
    // vez de os pintar uns sobre os outros (e há gate a prender os dois comprimentos).
    for (i, (row, &id)) in info
        .rows
        .iter()
        .zip(crate::ids::INSP_TWEEN_ROW.iter())
        .enumerate()
    {
        let rect = Rect::new(x, cur_y, w, ROW_H);
        hit_index.register(id, rect);
        ph2d_editor_core::widget::paint_row_stripe(
            scene,
            rect,
            theme,
            ph2d_editor_core::widget::section_cards::CardDepth::Section.token(),
            i,
        );
        // ⚠️ **O `Hovered` está aqui de propósito:** uma linha que só acende ao ser escolhida não
        // diz que é clicável — e isso lê-se exactamente como um controlo morto sob o dedo.
        ph2d_editor_core::widget::paint_row_highlight(
            scene,
            rect,
            theme,
            if i == selected {
                ph2d_editor_core::widget::RowHighlight::Selected
            } else if store.hover_live(id) > 0.0 {
                ph2d_editor_core::widget::RowHighlight::Hovered
            } else {
                ph2d_editor_core::widget::RowHighlight::None
            },
            0.0,
        );
        // ⚠️ **Um tween com QUEIXA escreve-se em WARN**, e a razão é a mesma do irmão: *«nada
        // acontece»* é o sintoma mais caro deste componente, e as causas autoráveis dele são
        // invisíveis num resumo.
        let color = if row.queixa(info.tem_sprite).is_some() {
            resolve(ColorToken::Warn, theme)
        } else if i == selected {
            resolve(ColorToken::Text1, theme)
        } else {
            resolve(ColorToken::Text2, theme)
        };
        paint_text(
            text_system,
            scene,
            Canal::from_tag(row.canal).label(),
            x + Spacing::Sm.px(),
            cur_y + (ROW_H - font) * 0.5,
            font,
            w,
            color,
        );
        cur_y += ROW_H;
    }
    // ⚠️ **A cauda da lista vem da PORTA** — há gate contra a segunda resposta.
    cur_y + ph2d_tokens::control_gap_px()
}

/// Os dois botões da lista, lado a lado. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
fn buttons(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorTweenInfo,
) -> f32 {
    // ⚠️ **O `+` DESAPARECE no tecto**, e não fica cinzento a mentir: um botão que aceita o clique
    // e não faz nada é o defeito que a DIRETIVA §2 nomeia.
    let can_add = info.rows.len() < crate::ids::INSP_TWEEN_ROW.len();
    let can_remove = !info.rows.is_empty();
    let n = usize::from(can_add) + usize::from(can_remove);
    if n == 0 {
        return y;
    }
    // ⭐⭐ **`+ Add | x Remove` é UM par**, e por isso passa pela porta do grupo — há gate
    // (`no_panel_lays_a_button_row_out_by_hand`).
    let seg = ph2d_editor_core::widget::segment_rects(Rect::new(x, y, w, BTN_H), n);
    let mut cell = 0usize;
    if can_add {
        let (rect, group) = seg[cell];
        cell += 1;
        hit_index.register(ids::INSP_TWEEN_ADD, rect);
        paint_button(
            &Button::new(
                ids::INSP_TWEEN_ADD,
                tr("panel.inspector.tween.plus_add_tween"),
            )
            .kind(ButtonKind::Default)
            .visual(store.button_visual(ids::INSP_TWEEN_ADD))
            .in_group(group),
            rect,
            scene,
            text_system,
            theme,
        );
    }
    if can_remove {
        let (rect, group) = seg[cell];
        hit_index.register(ids::INSP_TWEEN_REMOVE, rect);
        paint_button(
            &Button::new(
                ids::INSP_TWEEN_REMOVE,
                tr("panel.inspector.tween.x_remove_tween"),
            )
            .kind(ButtonKind::Default)
            .visual(store.button_visual(ids::INSP_TWEEN_REMOVE))
            .in_group(group),
            rect,
            scene,
            text_system,
            theme,
        );
    }
    y + BTN_H + ph2d_tokens::control_gap_px()
}

/// O editor do tween aberto. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
fn editor(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    row: &InspectorTweenRow,
    tem_sprite: bool,
) -> f32 {
    let mut cur_y = y;
    // ⚠️ **A QUEIXA primeiro** — quem não vê nada mexer não quer afinar uma curva.
    if let Some(q) = row.queixa(tem_sprite) {
        cur_y = warn(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr(chave_da_queixa(q)),
            ColorToken::Text3,
        );
    }
    let canal = Canal::from_tag(row.canal);
    let canais: Vec<&str> = Canal::ALL.iter().map(|c| c.label()).collect();
    cur_y = grupo(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.tween.channel"),
        &crate::ids::INSP_TWEEN_CANAL,
        &canais,
        canal.tag() as usize,
    );

    // ⭐ **Uma componente ou quatro — DERIVADO do canal**, nunca uma segunda lista.
    let n = canal.aridade();
    let seccao = ph2d_editor_core::property_row::Seccao::medida(
        text_system,
        n,
        &[
            tr("panel.inspector.tween.from"),
            tr("panel.inspector.tween.to"),
        ],
    );
    for (label, ids_) in [
        (tr("panel.inspector.tween.from"), &crate::ids::INSP_TWEEN_DE),
        (tr("panel.inspector.tween.to"), &crate::ids::INSP_TWEEN_PARA),
    ] {
        cur_y = super::rows::fields_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            label,
            &ids_[..n],
            0.05, // LITERAL-PX-OK: passo de arrasto — adimensional numa cor, metros numa pose
            None,
            seccao,
        );
    }

    let familias: Vec<&str> = ph2d_anim::EasingFamily::ALL
        .iter()
        .map(|f| f.label())
        .collect();
    cur_y = grupo(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.tween.curve"),
        &crate::ids::INSP_TWEEN_FAMILIA,
        &familias,
        row.familia as usize,
    );
    let modos: Vec<&str> = ph2d_anim::EasingMode::ALL
        .iter()
        .map(|m| m.label())
        .collect();
    cur_y = grupo(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.tween.ease"),
        &crate::ids::INSP_TWEEN_MODO,
        &modos,
        row.modo as usize,
    );
    let fins: Vec<&str> = AoAcabar::ALL.iter().map(|a| a.label()).collect();
    grupo(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.tween.when_done"),
        &crate::ids::INSP_TWEEN_AO_ACABAR,
        &fins,
        row.ao_acabar as usize,
    )
}

/// Pinta a secção. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_tween_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorTweenInfo,
    selected: usize,
) -> f32 {
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: banda do cabeçalho
    let color_id = core_ids::INSP_LIVE_TWEEN_COLOR;
    let rgba = store
        .widget_color(color_id)
        .unwrap_or([0x88, 0x88, 0x88, 0xff]); // LITERAL-COLOR-OK: acento neutro por omissão
    let title = if info.rows.is_empty() {
        String::from(tr("panel.inspector.tween.tween"))
    } else {
        tr_with(
            "panel.inspector.tween.title_count",
            &[("n", &info.rows.len())],
        )
    };
    let header = section_header(store, core_ids::INSP_LIVE_TWEEN_SECTION, &title).color(rgba);
    let header_rect = Rect::new(x, y, w, header_h);
    paint_section_header(&header, header_rect, scene, text_system, theme);
    if let Some(circle_rect) = ph2d_editor_core::widget::color_circle_hit_rect(&header, header_rect)
    {
        hit_index.register(color_id, circle_rect);
    }
    let Some(fold) = SectionFold::begin(
        store,
        core_ids::INSP_LIVE_TWEEN_SECTION,
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

    // ⚠️ **A SELECÇÃO MÚLTIPLA tem de se dizer** — o índice que uma edição carrega só significa
    // alguma coisa na lista da primária. Vem antes de tudo, porque *«em quem é que isto pega?»* é
    // anterior a qualquer controlo.
    if info.selected_count > 1 {
        cur_y = warn(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr("panel.inspector.tween.multiple_selected_tween_edits_apply"),
            ColorToken::Warn,
        );
    }
    if info.rows.is_empty() {
        paint_text(
            text_system,
            scene,
            tr("panel.inspector.tween.no_tweens_yet"),
            x,
            cur_y,
            font,
            w,
            resolve(ColorToken::Text3, theme),
        );
        cur_y += font + ph2d_tokens::control_gap_px();
    } else {
        cur_y = list(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            info,
            selected,
        );
    }
    cur_y = buttons(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        info,
    );
    if let Some(row) = info.rows.get(selected) {
        cur_y = editor(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            row,
            info.tem_sprite,
        );
    }
    fold.finish(store, scene, hit_index, cur_y + SECTION_BOTTOM_PAD_PX)
}
