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

/// O rótulo da opção «não montar». ⚠️ É também o que o chip mostra quando nada está montado.
pub(crate) const MOUNT_NONE_LABEL: &str = "\u{2014}";

/// A largura da coluna do rótulo, igual à da §7 Ordering — as duas linhas de rótulo-mais-chip do
/// Inspector alinham entre si.
const LABEL_COL_W: f32 = 96.0; // LITERAL-PX-OK: coluna de rótulo, igual à da §7
/// Altura de botão do Inspector, igual à de [`super::anchors`].
const BTN_H: f32 = 30.0; // LITERAL-PX-OK: altura de botão do Inspector
/// Altura visual de uma checkbox, igual à de [`super::anchors`].
const CHECK_H: f32 = 18.0; // LITERAL-PX-OK: altura visual do Checkbox

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
        (Some(name), true) => format!("{name}  (missing)"),
        _ => String::from(MOUNT_NONE_LABEL),
    }
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
    let font = TypeToken::Sm.px();
    paint_text(
        text_system,
        scene,
        "Rides Parent Anchor",
        x,
        y + (h - font) * 0.5,
        font,
        LABEL_COL_W,
        resolve(ColorToken::Text2, theme),
    );
    let chip_x = x + LABEL_COL_W + Spacing::Md.px();
    let chip_w = (w - LABEL_COL_W - Spacing::Md.px()).max(0.0);
    let rect = Rect::new(chip_x, y, chip_w, h);
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
        crate::state::set_pending_mount_dd(Some(rect));
    }
    let mut cur_y = y + h;

    if info.mount_dangling() {
        cur_y += Spacing::Xs.px();
        paint_text(
            text_system,
            scene,
            "The parent has no anchor with that name.",
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
        let label = format!("Off anchor by {ox:.0}, {oy:.0} px");
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
        cur_y += font + Spacing::Xs.px();
        let btn = Rect::new(x, cur_y, w, BTN_H);
        hit_index.register(ids::INSP_MOUNT_SNAP, btn);
        paint_button(
            &Button::new(ids::INSP_MOUNT_SNAP, "Reset to Anchor")
                .kind(ButtonKind::Default)
                .visual(store.button_visual(ids::INSP_MOUNT_SNAP)),
            btn,
            scene,
            text_system,
            theme,
        );
        cur_y += BTN_H;
    }
    cur_y + Spacing::Sm.px()
}

/// ⛔⛔ **O rótulo da caixa PARADA, e o bloqueador vai NELE.**
///
/// A caixa «Show anchors at runtime» gravava, viajava no `.ph2dproj` e **não tinha um único
/// leitor** — o próprio `mount_smoke.rs` o declarava desde 2026-08-22. O bloqueador tem nome:
/// **não existe modo de jogo** (`shells/game` / Runtime R1, adiado por decisão do dono do
/// produto), e sem ele não há «runtime» onde uma âncora se possa mostrar.
///
/// ⚠️ **A razão vai no RÓTULO e não só na dica de hover**: uma dica só aparece a quem já pousou o
/// rato, e quem lê «Show anchors at runtime» a cinzento sem explicação conclui que o app está
/// avariado. *Um controlo parado sem a razão à vista é a mesma promessa por outras palavras.*
///
/// ⛔ **O campo FICA no modelo** (`ph2d_ecs::AnchorVisibility::at_runtime`): apagá-lo partiria
/// todo ficheiro já gravado. O que sai é a **promessa**, não o dado.
pub(crate) const RUNTIME_BOX_LABEL: &str = "Show anchors at runtime (no game runtime yet)";

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
    for (id, label, on) in [
        (
            ids::INSP_ANCHOR_VIS_EDITOR,
            "Always show anchors",
            info.vis_in_editor,
        ),
        (
            ids::INSP_ANCHOR_VIS_RUNTIME,
            RUNTIME_BOX_LABEL,
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
            });
        if id == ids::INSP_ANCHOR_VIS_RUNTIME {
            cb = cb.state(CheckboxState::Disabled);
        }
        paint_checkbox(&cb, rect, scene, text_system, theme);
        cur_y += cb_h + Spacing::Xs.px();
    }
    cur_y + Spacing::Xs.px()
}
