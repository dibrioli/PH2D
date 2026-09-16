//! ⭐⭐⭐ **AS LINHAS DO OSSO INTELIGENTE** — irmão do [`super::section`] pelo tecto de LOC dos
//! painéis (600), com o corte por RESPONSABILIDADE: ali moram as linhas que dizem **o que um osso
//! É** (os números dele, a curvatura, o limite, a âncora), aqui as que dizem **o que ele DISPARA**.
//!
//! ⚠️ A lista de acções é a mesma que o [`crate::state`] publica, e o popover dela vive aqui pela
//! mesma razão que a linha que o abre: *o chip e a lista que ele abre são um controlo só.*

use crate::state::{self, SmartBoneView};
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::panel::{LABEL_COL_W, PaintCtx, RowCtx};
use ph2d_editor_core::widget::{
    DROPDOWN_SCROLLBAR_ID, Dropdown, DropdownOption, paint_dropdown_chip,
    paint_dropdown_popover_scrolled, scrollbar_is_needed, scrollbar_track_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::{ColorToken, Spacing, Theme};

use crate::section::ANGLE_STEP;

/// ⭐⭐⭐ **O OSSO INTELIGENTE** — girar este osso percorre uma acção inteira.
///
/// ⚠️⚠️ **A linha *Action* vem PRIMEIRO, e ela é a wave de 2026-09-08.** Até esse dia esta
/// função pintava *Remove* mais dois números e mais nada — o dono carregava em *Add Smart Bone*
/// e ficava com dois campos de graus **sem sujeito** (report: *«não há meios de selecionar nem o
/// objeto alvo nem a animação»*). *Um controlo cujo sujeito é invisível lê-se exactamente como
/// um controlo morto*, e a única resposta na casa era um `eprintln!` que o artista nunca vê.
///
/// ⇒ o chip é o **readout e o gesto**: ele diz o nome da acção ligada e abre a lista das que o
/// documento tem.
pub(crate) fn smart_rows(r: &mut RowCtx, y: f32) -> f32 {
    let Some(sb) = state::current_bone_smart() else {
        return r.action_button(
            ids::VECTOR_BONE_SMART_ADD,
            tr("panel.vector.bone.smart.add"),
            y,
        );
    };
    // ⚠️ **O OBJECTO vem ANTES da acção**, e a ordem diz o porquê: ele **decide o que a linha
    // seguinte mostra** (só as acções que animam este objecto). Ler isso depois de já ter
    // escolhido na lista é tarde — é a mesma lei do par *Criar × Transformar* no topo da secção.
    let mut y = smart_object_row(r, &sb, y);
    y = smart_action_row(r, &sb, y);
    y = r.action_button(
        ids::VECTOR_BONE_SMART_REMOVE,
        tr("panel.vector.bone.smart.remove"),
        y,
    );
    let campos: [(ph2d_a11y::NodeId, &str); 2] = [
        (
            ids::VECTOR_BONE_SMART_FROM,
            tr("panel.vector.bone.smart.from"),
        ),
        (ids::VECTOR_BONE_SMART_TO, tr("panel.vector.bone.smart.to")),
    ];
    for (id, label) in campos {
        y = r.labeled_number_field(label, id, ANGLE_STEP, y);
    }
    y
}

/// ⭐⭐⭐ **QUAL ACÇÃO** — o chip que a nomeia e abre a lista. Espelho exacto da linha de mistura
/// de um degrau de filtro (`paint_filters::filter_blend_row`).
///
/// ⚠️ Ele vem DEPOIS da linha do objecto de propósito (ver [`Self::smart_object_row`]), e a 1.ª
/// redacção deste doc ficou por cima dela — *um item novo colado a um comentário fica
/// documentado por ele* (auditoria de 2026-09-08).
///
/// ⚠️ **Vazio mostra o traço**, e não uma cadeia vazia: uma célula em branco lê-se como um
/// controlo por carregar, e o traço diz *«nenhuma»* em voz alta — a mesma lei da tecla de uma
/// forma do Morph.
/// ⭐⭐⭐ **DE QUE OBJECTO ESTE CONTROLO TRATA** — o botão que arma o *Pick Object*.
///
/// ⚠️⚠️ **Report do dono (2026-09-08):** *«é necessário um botão de picker para selecionar o
/// objeto seja no canvas ou seja na hierarquia … só deve aparecer as animações relacionadas ao
/// objeto selecionado»*. As duas superfícies saem de graça porque o pick resolve pela
/// **SELECÇÃO** — canvas e hierarquia escrevem a mesma —, e não por um segundo caminho de
/// acerto que divergiria do primeiro.
///
/// ⚠️ **É o READOUT e o gesto**: o rótulo é o nome do objecto, ou o convite quando não há. E
/// **armado ele diz o que espera** — um botão que fica igual depois do clique lê-se como um
/// botão que não fez nada.
fn smart_object_row(r: &mut RowCtx, sb: &SmartBoneView, y: f32) -> f32 {
    let rotulo = if sb.picking {
        tr("panel.vector.bone.smart.picking")
    } else if sb.target.is_empty() {
        tr("panel.vector.bone.smart.pick")
    } else {
        sb.target.as_str()
    };
    r.labeled_action_button(
        tr("panel.vector.bone.smart.object"),
        ids::VECTOR_BONE_SMART_PICK,
        rotulo,
        sb.picking,
        y,
    )
}

fn smart_action_row(r: &mut RowCtx, sb: &SmartBoneView, y: f32) -> f32 {
    let gap = Spacing::Xs.px();
    let id = crate::ids::VECTOR_BONE_SMART_CLIP;
    paint_text(
        r.text_system,
        r.scene,
        tr("panel.vector.bone.smart.action"),
        r.inner_x,
        y + (r.row_h - r.font) * 0.5,
        r.font,
        LABEL_COL_W,
        resolve(ColorToken::Text1, r.theme),
    );
    let rotulo = if sb.clip.is_empty() {
        tr("panel.vector.bone.smart.none")
    } else {
        sb.clip.as_str()
    };
    let chip = Rect::new(
        r.inner_x + LABEL_COL_W + gap,
        y,
        (r.inner_w - LABEL_COL_W - gap).max(1.0),
        r.row_h,
    );
    let open = matches!(
        r.store.get(id),
        Some(InteractiveState::Dropdown { open: true, .. })
    );
    let dd = Dropdown::new(id, "", vec![DropdownOption::new(id, (), rotulo)])
        .selected(())
        .open(open)
        .visual(r.store.dropdown_visual(id));
    paint_dropdown_chip(&dd, chip, r.scene, r.text_system, r.theme);
    r.hit_index.register(id, chip);
    if open {
        state::set_pending_bone_action_dd(Some(chip));
    }
    y + r.row_h + r.row_gap
}

/// ⭐⭐⭐ **A LISTA DAS ACÇÕES** do osso inteligente — pintada no passe DIFERIDO de `paint.rs`, POR
/// CIMA de todas as seções. Espelho exacto do `paint_filters_blend::paint_blend_popover`.
///
/// ⚠️ **A seção ROLA**, então sem o passe diferido a lista seria cortada na borda dela — foi o que
/// obrigou o Morph e a mistura de filtro a fazerem o mesmo.
///
/// ⚠️ **As acções saem da lista PUBLICADA pela shell**, nunca de uma leitura do painel: elas são
/// conteúdo do documento, e uma segunda leitura aqui envelheceria na primeira que ele criasse.
///
/// ⚠️ **O corte é o POOL de ids** ([`ids::VECTOR_BONE_SMART_CLIP_IDS`]) e não um número escrito
/// aqui: o chrome não cunha um id em tempo de execução, e uma opção sem id nasceria **morta sob o
/// dedo**. O gate da shell mantém o pool do tamanho do tecto do documento.
pub(crate) fn paint_action_popover(ctx: &mut PaintCtx, chip: Rect, theme: Theme) {
    let id = crate::ids::VECTOR_BONE_SMART_CLIP;
    let nomes = state::bone_actions();
    let n = nomes.len().min(ids::VECTOR_BONE_SMART_CLIP_IDS.len());
    if n == 0 {
        return;
    }
    let ligada = state::current_bone_smart()
        .map(|s| s.clip)
        .unwrap_or_default();
    let sel = nomes.iter().take(n).position(|c| *c == ligada).unwrap_or(0);
    let options: Vec<DropdownOption<usize>> = nomes
        .iter()
        .take(n)
        .enumerate()
        .map(|(i, nome)| DropdownOption::new(ids::VECTOR_BONE_SMART_CLIP_IDS[i], i, nome.as_str()))
        .collect();
    let dd = Dropdown::new(id, "", options).selected(sel).open(true);

    let panel = dd.popover_rect_clamped(chip, ctx.layout.popover_region());
    let content_h = dd.content_height(chip.h);
    let visible_h = panel.h;
    let max_scroll = (content_h - visible_h).max(0.0);
    {
        let store = ctx.host.store_mut();
        store.set_dropdown_popover(id, panel);
        store.set_panel_content_h(id, content_h);
        store.set_panel_visible_h(id, visible_h);
        if store.panel_scroll(id) > max_scroll {
            store.set_panel_scroll(id, max_scroll);
        }
    }
    let scroll = ctx.host.store().panel_scroll(id).clamp(0.0, max_scroll); // CLAMP-OK: 0.0 literal; max_scroll is a non-negative px extent
    paint_dropdown_popover_scrolled(
        &dd,
        chip,
        panel,
        scroll,
        ctx.host
            .store()
            .scrollbar_visual_for(DROPDOWN_SCROLLBAR_ID, Some(id)),
        ctx.scene,
        ctx.text_system,
        theme,
    );

    // Hit-register só a parte VISÍVEL de cada linha (a barra de rolagem é o alvo do arrasto).
    let hit_index = ctx.host.hit_index_mut();
    for i in 0..n {
        let r = dd.option_rect_in_scrolled(chip, panel, i, scroll);
        let top = r.y.max(panel.y);
        let bot = (r.y + r.h).min(panel.y + panel.h);
        if bot - top >= 1.0 {
            hit_index.register(
                ids::VECTOR_BONE_SMART_CLIP_IDS[i],
                Rect::new(r.x, top, r.w, bot - top),
            );
        }
    }
    if scrollbar_is_needed(content_h, visible_h) {
        ctx.host
            .hit_index_mut()
            .register(DROPDOWN_SCROLLBAR_ID, scrollbar_track_rect(panel));
    }
}
