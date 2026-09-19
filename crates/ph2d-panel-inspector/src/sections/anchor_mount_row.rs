//! **A linha «Rides Parent Anchor»** da §12 — o único controlo do Inspector cujo dono é OUTRA
//! entidade ([ADR-0072] §2.6).
//!
//! ⚠️ **Irmão de [`super::anchors`] por CAP de LOC** — mesmo padrão do
//! [`super::slice_grid`] em relação ao [`super::slice_nine`].
//!
//! # As duas metades da seção têm donos diferentes, e a UI diz qual
//!
//! Abaixo desta linha está a lista de âncoras **deste** objeto; esta linha diz de que âncora **do
//! pai** ele parte. Os dois vivem na mesma seção de propósito — *socket* é a palavra que o
//! artista procura quando quer prender uma espada a uma mão —, e cada bloco leva o seu rótulo a
//! dizer de quem é. Espalhá-los por duas seções faria procurar em duas.
//!
//! # ⛔ A linha NÃO se pinta quando não há o que escolher
//!
//! Sem pai, ou com um pai sem âncoras, não há escolha nenhuma a oferecer — e um controlo com uma
//! opção só é a mesma afordância a mentir que o botão `Simple` do 9-slice era (Enio, 2026-08-22).
//! A decisão vive no modelo (`InspectorAnchorInfo::mount_pick_is_useful`), não aqui, porque é a
//! mesma pergunta que o gate de alcance faz.
//!
//! # ⚠️ Um vínculo PENDURADO aparece no chip, não some
//!
//! Quando o pai já não tem a âncora que este objeto montava, o índice não resolve — e um chip que
//! mostrasse «—» estaria a dizer que o objeto não monta em nada, que é falso. O nome perdido vai
//! ao **placeholder** e uma linha de aviso explica-o. Escolher qualquer coisa (incluindo «—»)
//! resolve; é a diferença entre um estado mau e um estado preso.
//!
//! [ADR-0072]: ../../../../docs/architecture/decisions/0072-named-anchor-unification.md

use super::*;
use ph2d_editor_core::screens::hero::InspectorAnchorInfo;
use ph2d_editor_core::widget::{Dropdown, DropdownOption, paint_dropdown_chip};
use ph2d_i18n::TextKey;
use ph2d_i18n::tr;
use ph2d_i18n::tr_with;

/// O rótulo da opção «não montar». ⚠️ É também o que o chip mostra quando nada está montado.
pub(crate) const MOUNT_NONE_LABEL: &str = "\u{2014}";

/// Altura de botão do Inspector, igual à de [`super::anchors`].
const BTN_H: f32 = 30.0; // LITERAL-PX-OK: altura de botão do Inspector
/// Altura visual de uma checkbox, igual à de [`super::anchors`].
const CHECK_H: f32 = ph2d_tokens::ROW_H_PX; // ⛔ era `18.0`, o MESMO literal em TREZE sitios: a linha de marcar e' uma linha de propriedade, e a altura dela e' a do app (report do dono 2026-09-15: a marca enchia a caixa toda)

/// **As opções do seletor**: «—» mais uma por âncora do pai.
///
/// O valor é `Option<usize>` — o ÍNDICE na lista do pai, e `None` para «não montar». ⚠️ O índice
/// serve para o widget saber o que está escolhido **neste quadro**; o que viaja na edição é o
/// NOME (`AnchorFieldEdit::Mount`), porque um índice envelheceria à primeira reordenação.
pub(crate) fn mount_options(info: &InspectorAnchorInfo) -> Vec<DropdownOption<Option<usize>>> {
    let mut out = Vec::with_capacity(info.parent_anchors.len() + 1);
    out.push(DropdownOption::new(
        ids::INSP_MOUNT_NONE_OPT,
        None,
        MOUNT_NONE_LABEL,
    ));
    // ⚠️ `zip` com o array de ids: um pai com mais âncoras do que ids (impossível enquanto o gate
    // `the_mount_option_ids_cover_the_model_cap` viver) perde as excedentes em vez de as pintar
    // umas sobre as outras.
    for (i, (name, &id)) in info
        .parent_anchors
        .iter()
        .zip(ids::INSP_MOUNT_OPT.iter())
        .enumerate()
    {
        out.push(DropdownOption::new(id, Some(i), name.clone()));
    }
    out
}

/// O que o chip mostra quando **nada** está escolhido — «—», ou o nome perdido.
pub(crate) fn mount_placeholder(info: &InspectorAnchorInfo) -> String {
    match (&info.mount, info.mount_dangling()) {
        (Some(name), true) => tr_with("panel.inspector.anchors.mount_missing", &[("name", &name)]),
        _ => String::from(MOUNT_NONE_LABEL),
    }
}

/// ⭐⭐⭐ **A COLUNA desta secção — os TRÊS nomes que ela pinta.**
///
/// ⛔⛔ Medido em 2026-09-15: os dois nomes das caixas medem `115,4` e `135,8 px` a `Sm` e a
/// **metade cega** dá `104,6` na largura do dono — *os dois saíam cortados*, e a linha de escolher
/// ao lado deles pedia a mesma metade por outra conta. ⇒ uma declaração, três linhas.
fn seccao(text_system: &mut TextSystem) -> ph2d_editor_core::property_row::Seccao {
    ph2d_editor_core::property_row::Seccao::medida(
        text_system,
        1,
        &[
            tr("panel.inspector.anchors.rides_parent_anchor"),
            tr("panel.inspector.anchors.always_show_anchors"),
            RUNTIME_BOX_LABEL.tr(),
        ],
    )
}

/// Pinta a linha e devolve o `y` seguinte. Não pinta nada — nem consome altura — quando o modelo
/// diz que não há o que escolher.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_mount_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorAnchorInfo,
) -> f32 {
    if !info.mount_pick_is_useful() {
        return y;
    }
    let h = ROW_H_PX;
    // ⭐ A coluna do rótulo sai da porta (14/09) — era a `LABEL_COL_W` deste ficheiro, cujo próprio
    // comentário dizia *«igual à da §7»*: uma cópia a prometer que acompanharia outra.
    // ⭐⭐ E desde 15/09 é a coluna da SECÇÃO — ver [`seccao`].
    let font = TypeToken::Sm.px();
    let row = ph2d_editor_core::property_row::colunas_da_linha(x, w, y, h, seccao(text_system));
    ph2d_editor_core::widget::paint_property_label(
        text_system,
        scene,
        tr("panel.inspector.anchors.rides_parent_anchor"),
        row.label.x,
        row.label.y + (h - font) * 0.5,
        font,
        row.label.w,
        resolve(ColorToken::Text2, theme),
    );
    let (dot, rect) = (row.dot, row.control);
    hit_index.register(ids::INSP_MOUNT_PICK, rect);

    // ⚠️ **A verdade é do MODELO; o store só guarda se o popover está aberto.** Um índice vindo do
    // store sobreviveria à troca de seleção e mostraria a montagem do objeto anterior — a mesma
    // lei que o `open_anchor_row` já paga: *o seed é dono do VALOR, o dispatch é dono do ESTADO*.
    let open = matches!(
        store.get(ids::INSP_MOUNT_PICK),
        Some(InteractiveState::Dropdown { open: true, .. })
    );
    let mut dd = Dropdown::new(ids::INSP_MOUNT_PICK, "", mount_options(info))
        .open(open)
        .visual(store.dropdown_visual(ids::INSP_MOUNT_PICK))
        .placeholder(mount_placeholder(info));
    if let Some(i) = info.mount_index() {
        dd.select(Some(i));
    }
    paint_dropdown_chip(&dd, rect, scene, text_system, theme);
    if open {
        crate::state_popovers::set_pending_mount_dd(Some(rect));
    }
    ph2d_editor_core::widget::paint_decorator_dot(scene, theme, dot);
    let mut cur_y = y + h;

    if info.mount_dangling() {
        cur_y += Spacing::Xs.px();
        paint_text(
            text_system,
            scene,
            tr("panel.inspector.anchors.the_parent_has_no_anchor"),
            x,
            cur_y,
            font,
            w,
            resolve(ColorToken::Danger, theme),
        );
        cur_y += font;
    }

    // **O deslocamento, e a saída dele** (Enio, 2026-08-23).
    //
    // ⛔ Os dois só existem quando o objeto está FORA da âncora. Montar já o pousa em cima dela,
    // então o estado normal é não haver nada aqui — e um botão permanentemente sem efeito seria a
    // terceira ação morta desta família.
    //
    // ⚠️ **O número vem antes do botão de propósito.** «Reset» sozinho não diz de quanto se está a
    // falar; com o deslocamento ao lado, o artista vê se vale a pena carregar.
    if info.is_off_anchor() {
        cur_y += Spacing::Xs.px();
        let [ox, oy] = info.mount_offset;
        let label = tr_with(
            "panel.inspector.anchors.off_anchor_by",
            &[("ox", &format!("{ox:.0}")), ("oy", &format!("{oy:.0}"))],
        );
        paint_text(
            text_system,
            scene,
            &label,
            x,
            cur_y,
            font,
            w,
            resolve(ColorToken::Text3, theme),
        );
        cur_y += font + ph2d_tokens::control_gap_px();
        let btn = Rect::new(x, cur_y, w, BTN_H);
        hit_index.register(ids::INSP_MOUNT_SNAP, btn);
        paint_button(
            &Button::new(
                ids::INSP_MOUNT_SNAP,
                tr("panel.inspector.anchors.reset_to_anchor"),
            )
            .kind(ButtonKind::Default)
            .visual(store.button_visual(ids::INSP_MOUNT_SNAP)),
            btn,
            scene,
            text_system,
            theme,
        );
        cur_y += BTN_H;
    }
    cur_y + ph2d_tokens::control_gap_px()
}

/// ⛔⛔ **O rótulo da caixa PARADA, e o bloqueador vai NELE.**
///
/// A caixa «Show anchors at runtime» gravava, viajava no `.ph2dproj` e **não tinha um único
/// leitor** — o próprio `mount_smoke.rs` o declarava desde 2026-08-22. O bloqueador tem nome:
/// **não existe modo de jogo** (`shells/game` / Runtime R1, adiado por decisão do dono do
/// produto), e sem ele não há «runtime» onde uma âncora se possa mostrar.
///
/// ⛔⛔⛔ **ESTA LINHA DIZIA O CONTRÁRIO ATÉ 2026-09-19, e foi o DONO que a virou.** Ela
/// argumentava que *«a razão vai no RÓTULO e não só na dica de hover»* — porque uma dica só aparece
/// a quem já pousou o rato. O argumento continua de pé; o que mudou foi o **preço**, medido pela
/// varredura de elisões com o Inspector armado: com o parêntesis, o rótulo pede `~190 px` numa
/// coluna de `174` e sai `Show anchors at runtime (n…` — *uma razão cortada a meio da palavra não
/// é uma razão à vista*. Postas as três saídas à frente dele, ele escolheu **encurtar o nome**.
///
/// ⚠️ A razão inteira vive no balão (`parked_this_app_has_no`, registado no `populate_anchor`), e
/// há gate a exigir que ela lá esteja: *cumprir só a metade que REMOVE apaga a explicação em
/// silêncio* (`as_caixas_que_encurtaram_guardam_a_explicacao`).
///
/// ⛔ **O campo FICA no modelo** (`ph2d_ecs::AnchorVisibility::at_runtime`): apagá-lo partiria
/// todo ficheiro já gravado. O que sai é a **promessa**, não o dado.
pub(crate) const RUNTIME_BOX_LABEL: TextKey =
    TextKey::new("panel.inspector.anchors.show_anchors_at_runtime_no");

/// **As duas caixas de VISIBILIDADE**, do dono das âncoras (Enio, 2026-08-23). Devolve o `y`.
///
/// ⚠️ **Elas são do PAI, e por isso ficam ao pé da lista dele** — não da linha «Rides Parent
/// Anchor», que fala do avô. As duas metades desta seção já têm donos diferentes; misturar as
/// caixas na metade errada faria três.
///
/// ⛔ Não se pintam quando não há âncoras nenhumas: não há o que manter visível.
///
/// ⚠️ **A segunda caixa nasce PARADA** (`CheckboxState::Disabled`) — ver [`RUNTIME_BOX_LABEL`].
/// ⛔ **A IRMÃ está VIVA e não se lhe toca:** «Always show anchors» tem consumidor
/// (`render_loop::anchor_overlay`, `PlanMode::AlwaysVisible`) e é o que faz as âncoras aparecerem
/// no editor. Parar as duas por simetria apagaria uma feature que funciona.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_visibility_rows(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorAnchorInfo,
) -> f32 {
    if info.rows.is_empty() {
        return y;
    }
    let mut cur_y = y;
    // ⚠️ **Estas duas NÃO passam pela [`ph2d_editor_core::property_row::paint_check_row`]**, e a
    // razão é a segunda: ela nasce **PARADA** (ver [`RUNTIME_BOX_LABEL`]), e o estado dela não vem
    // do store. A porta serve quem lê o par visual da loja; aqui o painel sobrepõe-se a ele.
    // ⭐ O que a porta dá — **a coluna da SECÇÃO** — chega na mesma, pelo `.seccao(..)`.
    let sec = seccao(text_system);
    for (id, label, on) in [
        (
            ids::INSP_ANCHOR_VIS_EDITOR,
            tr("panel.inspector.anchors.always_show_anchors"),
            info.vis_in_editor,
        ),
        (
            ids::INSP_ANCHOR_VIS_RUNTIME,
            RUNTIME_BOX_LABEL.tr(),
            info.vis_at_runtime,
        ),
    ] {
        let cb_h = CHECK_H;
        let rect = Rect::new(x, cur_y, w, cb_h);
        hit_index.register(id, rect);
        let mut cb = Checkbox::new(id, label)
            .visual(store.checkbox_visual(id))
            .value(if on {
                CheckboxValue::Checked
            } else {
                CheckboxValue::Unchecked
            })
            .seccao(sec);
        if id == ids::INSP_ANCHOR_VIS_RUNTIME {
            cb = cb.state(CheckboxState::Disabled);
        }
        paint_checkbox(&cb, rect, scene, text_system, theme);
        cur_y += cb_h + ph2d_tokens::control_gap_px();
    }
    cur_y + ph2d_tokens::control_gap_px()
}
