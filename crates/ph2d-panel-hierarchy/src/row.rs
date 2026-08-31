//! `paint_hierarchy_row` — per-row painter for the hierarchy panel.
//! Ported from `ph2d_editor_core::screens::hero::hierarchy::row_painter`
//! in Phase C.2; logic unchanged.

use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::interaction::HitIndex;
use ph2d_editor_core::paint::{
    fill_rounded_rect, paint_icon, paint_text, resolve, stroke_rounded_rect,
};
use ph2d_editor_core::screens::hero::fixture;
use ph2d_editor_core::screens::hero::ids;
use ph2d_editor_core::widget::{Tag, TagState, TagTone, paint_tag};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{
    ColorToken, ICON_BTN_SIZE_PX, Radius, SECTION_GAP_PX, Spacing, StrokeToken, Theme, TypeToken,
};
use ph2d_vector::VectorScene;

/// **O TOM de cada selo de linha.** Porta única: o selo é um código, e o tom é o que o olho lê
/// antes de decifrar as três letras.
///
/// ⚠️ Ela saiu de dentro do `paint_hierarchy_row` em 2026-08-22, quando os selos do papel booleano
/// empurraram aquela função para fora do teto de LOC. ⛔ **Split, nunca allowlist** — e o número
/// do allowlist desceu junto, que é a única direção em que ele pode andar.
fn badge_tone(badge: &str) -> TagTone {
    match badge {
        "PRF" | "SPR" => TagTone::Accent,
        "OUT" | "LGT" => TagTone::Warn,
        "CAM" => TagTone::Success,
        "TRG" => TagTone::Danger,
        // O PAPEL de uma forma dentro de uma booleana viva (2026-08-22). O tom separa **o que
        // acrescenta** do **o que tira** — a leitura que o olho faz ao percorrer a receita.
        //
        // ⚠️ `UNI` cai no Neutral por já existir acima com outro significado, e ⛔ **não se muda o
        // tom daquele** para acomodar este: seria repintar um selo de outra família.
        "SUB" => TagTone::Warn,
        // ⭐ **O vínculo ao desenho** (W57): a forma muda quando a curva muda. `Success` porque é
        // uma capacidade a mais, nunca um aviso — o oposto de uma forma que perdeu a fonte.
        "LNK" => TagTone::Success,
        // ⭐⭐ **O ISOLAMENTO** (2026-08-25): esta linha é a única que está a ser desenhada, e as
        // outras estão escondidas por causa dela. `Warn` porque é um **estado da vista** que
        // explica uma ausência — o artista que não o vê conclui que perdeu a peça.
        "ISO" => TagTone::Warn,
        "INT" | "EXC" => TagTone::Accent,
        // A BASE não tem verbo e a RECEITA é do grupo inteiro: nenhum dos dois é escolha daquela
        // linha, e o neutro é o que os separa dos que são.
        _ => TagTone::Neutral,
    }
}

/// **O FUNDO DE UMA LINHA** — o que ela diz sobre si antes de qualquer conteúdo.
///
/// ⚠️ Ela saiu de dentro do `paint_hierarchy_row` em 2026-08-23, quando o realce de proveniência
/// empurrou aquela função para fora do teto de 200 LOC. ⛔ **Split, nunca allowlist** — e o número
/// do allowlist desce junto, que é a única direção em que ele pode andar (a mesma decisão que o
/// `badge_tone` pagou em 22/08).
fn paint_row_background(
    entity: &fixture::HierarchyEntity,
    rect: Rect,
    scene: &mut VectorScene,
    theme: Theme,
) {
    // ⭐ **O REALCE DE PROVENIÊNCIA** (estudo de UI viva, C2): o objecto sob o ponteiro acende a
    // linha dele, venha o ponteiro do canvas ou desta mesma lista.
    //
    // ⚠️ **A seleção VENCE, e não é ordem de desenho — é significado.** Selecionado é um facto do
    // documento (*estas são as formas em mãos*); apontado é um facto do ponteiro, que dura o que a
    // mão durar. Pintar o hover por cima faria a linha selecionada mudar de cor por alguém passar
    // o rato, e o artista perderia de vista o que tem em mãos.
    //
    // ⚠️ **E é um tom SÓ, sem eixo** — a cerca do estudo §6.2: *o realce de uma lista OBEDECE ao
    // cursor*. Oito linhas meio-acesas ao mesmo tempo é rasto, não vida.
    if !entity.selected && entity.hovered {
        fill_rounded_rect(
            scene,
            rect,
            Radius::Sm.px(),
            resolve(ColorToken::Bg2, theme),
        );
    }
    if entity.selected {
        fill_rounded_rect(
            scene,
            rect,
            Radius::Sm.px(),
            resolve(ColorToken::AccentSoft, theme),
        );
        stroke_rounded_rect(
            scene,
            rect,
            Radius::Sm.px(),
            1.0,
            resolve(ColorToken::Accent, theme),
        );
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_hierarchy_row(
    entity: &fixture::HierarchyEntity,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    row_id: Option<ph2d_a11y::NodeId>,
    mut hit_index: Option<&mut HitIndex>,
    has_children: bool,
    is_collapsed: bool,
    direct_match: bool,
) {
    paint_row_background(entity, rect, scene, theme);
    // Internal row inset. MUST match `paint.rs::row_inner_pad` — the
    // parentesco tree lines in `paint.rs` are drawn at
    // `col_chev_x + half_chev` using row_inner_pad as the inner offset.
    // If pad ≠ row_inner_pad, the vertical guide line drifts horizontally
    // away from the chevron of the parent row (Enio 2026-05-26: "A linha
    // que mostra parentesco deveria sair exatamente abaixo da setinha
    // mas está deslocada. Para corrigir coloque mais para a esquerda
    // a setinha, o ícone e o nome dos objetos").
    let pad = Spacing::Xxs.px(); // sync with paint.rs::row_inner_pad
    let chev_w = Spacing::Lg.px();
    // Chev → icon gap tightened Xs (4) → Xxs (2) 2026-05-24 per user:
    // "quero os ícones mais próximos das setas". Godot's Scene panel
    // packs them flush together; we keep a 2-px hairline so click
    // targets don't visually merge.
    let chev_pad = Spacing::Xxs.px();
    let chev_x = rect.x + pad;
    if has_children {
        let chev_rect = Rect::new(chev_x, rect.y + (rect.h - chev_w) * 0.5, chev_w, chev_w);
        let chev_icon = if is_collapsed {
            IconId::ChevronRight
        } else {
            IconId::ChevronDown
        };
        paint_icon(
            scene,
            chev_icon,
            chev_rect,
            resolve(ColorToken::Text2, theme),
            StrokeToken::Default.px(),
        );
        if let (Some(row_id), Some(idx)) = (row_id, hit_index.as_mut()) {
            let hit_rect = Rect::new(
                chev_rect.x - Spacing::Sm.px(),
                chev_rect.y - Spacing::Sm.px(),
                chev_w + Spacing::Lg.px(),
                chev_w + Spacing::Lg.px(),
            );
            idx.register(ids::hier_expand_companion(row_id), hit_rect);
        }
    }
    let icon_w = Spacing::Xl.px();
    let icon_x = chev_x + chev_w + chev_pad;
    let icon_rect = Rect::new(icon_x, rect.y + (rect.h - icon_w) * 0.5, icon_w, icon_w);
    let icon_color = if entity.selected {
        ColorToken::Accent
    } else if entity.muted {
        ColorToken::TextDisabled
    } else {
        ColorToken::Text3
    };
    paint_icon(
        scene,
        entity.icon,
        icon_rect,
        resolve(icon_color, theme),
        StrokeToken::Default.px(),
    );
    // Hit-register the entity icon as its own companion (2026-05-26):
    // double-click on the icon focuses the view; double-click on the
    // name body (row body) triggers rename. Hit area = icon glyph
    // EXACTLY (zero padding) — Enio 2026-05-26 round 2: "a área
    // sensível ao duplo clique sobre o ícone à esquerda do nome do
    // objeto em hierarquia ficou muito grande e atrapalha clicar
    // na seta à esquerda do ícone. Ajuste ao tamanho do ícone".
    // Any positive pad would extend left past the icon edge into
    // the chev_pad gap (2 px) and onto the chevron itself, since
    // companion ids registered later in HitIndex win.
    if let (Some(row_id), Some(idx)) = (row_id, hit_index.as_mut()) {
        idx.register(ids::hier_icon_companion(row_id), icon_rect);
    }

    // Right-side icon cluster — eye colada na borda direita (pad 0)
    // e gap inter-icon Xxs (2 px) pra ficarem "bem juntos" (Enio
    // 2026-05-26). Antes: pad=2 + gap=Sm (6).
    let icon_cluster_pad = Spacing::Sm.px(); // 8 px da borda direita (Enio 2026-05-26 round 2: "separe os olhos mais 4 pixels da lateral direita (desloque 4 pixels para esquerda)" — antes Xs=4)
    let icon_cluster_gap = 0.0_f32; // LITERAL-PX-OK: Enio 2026-05-26 "reduza a distância entre os ícones"
    let mut right_x = rect.x + rect.w - icon_cluster_pad;
    let eye_icon = if entity.visible {
        IconId::Eye
    } else {
        IconId::EyeClosed
    };
    let eye_color = if entity.visible {
        ColorToken::Text2
    } else {
        ColorToken::TextDisabled
    };
    let eye_size = Spacing::Xl.px();
    let eye_rect = Rect::new(
        right_x - eye_size,
        rect.y + (rect.h - eye_size) * 0.5,
        eye_size,
        eye_size,
    );
    paint_icon(
        scene,
        eye_icon,
        eye_rect,
        resolve(eye_color, theme),
        StrokeToken::Default.px(),
    );
    if let (Some(row_id), Some(idx)) = (row_id, hit_index.as_mut()) {
        let hit_pad = Spacing::Xs.px();
        let hit_rect = Rect::new(
            eye_rect.x - hit_pad,
            eye_rect.y - hit_pad,
            eye_rect.w + hit_pad * 2.0,
            eye_rect.h + hit_pad * 2.0,
        );
        idx.register(ids::hier_eye_companion(row_id), hit_rect);
    }
    right_x -= eye_size + icon_cluster_gap;
    // ── Group lock (folder icon) — pinta SE locked, sempre clicável.
    // À esquerda do olho. Click toggla `GroupedChildren` em SimWorld
    // via EditorAction::HierToggleGroup (handler no shell). Enio
    // 2026-05-26: "Agrupar: vc pode manipular o pai mas não os filhos".
    let icon_btn = eye_size;
    let group_rect = Rect::new(
        right_x - icon_btn,
        rect.y + (rect.h - icon_btn) * 0.5,
        icon_btn,
        icon_btn,
    );
    // Enio 2026-05-26: checked → Accent (cor de destaque do tema);
    // unchecked → Text3 (cinza). Olho NÃO usa accent — segue o seu
    // próprio padrão.
    let group_color = if entity.group_locked {
        ColorToken::Accent
    } else {
        ColorToken::Text3
    };
    let group_icon = if entity.group_locked {
        IconId::Group
    } else {
        IconId::Ungroup
    };
    paint_icon(
        scene,
        group_icon,
        group_rect,
        resolve(group_color, theme),
        StrokeToken::Default.px(),
    );
    if let (Some(row_id), Some(idx)) = (row_id, hit_index.as_mut()) {
        let hit_pad = Spacing::Xs.px();
        let hit_rect = Rect::new(
            group_rect.x - hit_pad,
            group_rect.y - hit_pad,
            group_rect.w + hit_pad * 2.0,
            group_rect.h + hit_pad * 2.0,
        );
        idx.register(ids::hier_group_companion(row_id), hit_rect);
    }
    right_x -= icon_btn + icon_cluster_gap;
    // ── Lock individual (cadeado) — Enio: "Cadeado trava apenas o
    // objeto. Se este objeto tiver filhos, os filhos podem ser
    // manipulados". Sempre pintado, cor reflete estado.
    let lock_rect = Rect::new(
        right_x - icon_btn,
        rect.y + (rect.h - icon_btn) * 0.5,
        icon_btn,
        icon_btn,
    );
    // Enio 2026-05-26: ver comentário do group_color acima.
    let lock_color = if entity.locked {
        ColorToken::Accent
    } else {
        ColorToken::Text3
    };
    let lock_icon = if entity.locked {
        IconId::LockKeyhole
    } else {
        IconId::LockKeyholeOpen
    };
    paint_icon(
        scene,
        lock_icon,
        lock_rect,
        resolve(lock_color, theme),
        StrokeToken::Default.px(),
    );
    if let (Some(row_id), Some(idx)) = (row_id, hit_index.as_mut()) {
        let hit_pad = Spacing::Xs.px();
        let hit_rect = Rect::new(
            lock_rect.x - hit_pad,
            lock_rect.y - hit_pad,
            lock_rect.w + hit_pad * 2.0,
            lock_rect.h + hit_pad * 2.0,
        );
        idx.register(ids::hier_lock_companion(row_id), hit_rect);
    }
    right_x -= icon_btn + icon_cluster_gap;
    if let Some(swatch) = entity.swatch {
        let sw = SECTION_GAP_PX;
        let sw_rect = Rect::new(right_x - sw, rect.y + (rect.h - sw) * 0.5, sw, sw);
        // Canonical color swatch painter (single source of truth).
        let cs = ph2d_editor_core::widget::ColorSwatch::new(
            row_id.unwrap_or(ph2d_a11y::NodeId(0)),
            "",
            swatch,
        );
        ph2d_editor_core::widget::paint_color_swatch(&cs, sw_rect, scene, theme);
        right_x -= sw + icon_cluster_gap;
    }
    if let Some(badge) = &entity.badge {
        let badge_w = ICON_BTN_SIZE_PX;
        let badge_h = TypeToken::Lg.px();
        let badge_rect = Rect::new(
            right_x - badge_w,
            rect.y + (rect.h - badge_h) * 0.5,
            badge_w,
            badge_h,
        );
        let tone = badge_tone(badge);
        let tag = Tag::new(ph2d_a11y::NodeId(0), badge)
            .tone(tone)
            .state(if entity.muted {
                TagState::Disabled
            } else {
                TagState::Normal
            });
        paint_tag(&tag, badge_rect, scene, text_system, theme);
        right_x -= badge_w + icon_cluster_gap;
    }

    // Icon → name gap tightened Md (8) → Xs (4) 2026-05-24 per user:
    // "nome mais próximos dos ícones".
    let name_x = icon_rect.x + icon_w + Spacing::Xs.px();
    let name_color = if entity.muted {
        ColorToken::TextDisabled
    } else if direct_match {
        ColorToken::Accent
    } else {
        ColorToken::Text1
    };
    // Hard clip ao espaço entre name_x e o icon cluster — sem clip,
    // `paint_text` faz wrap quando a string excede a largura (nomes
    // longos quebravam em 2 linhas E invadiam os ícones). Com clip,
    // letras à direita simplesmente não aparecem (Enio 2026-05-26:
    // "As letras da direita do nome que invadirão os ícones devem
    // simplesmente não aparecer. Não podem sobrepor os ícones").
    // Reservar gap pequeno antes do primeiro ícone (não colado).
    let name_right_limit = right_x - Spacing::Xxs.px();
    let name_w = (name_right_limit - name_x).max(0.0);
    if name_w > 0.0 {
        let name_clip = ph2d_vector::Rect::new(
            name_x as f64,
            rect.y as f64,
            (name_x + name_w) as f64,
            (rect.y + rect.h) as f64,
        );
        scene.push_clip(&name_clip);
        // ⭐⭐⭐ **A linha mostra o que o objecto É, não as propriedades dele** (Enio, 2026-08-30:
        // *«os nomes ficam grandes demais e nem cabem direito na hierarquia»*).
        //
        // `Casa {Size=Small, State=Idle}` desenha-se **`Casa`**. ⚠️ **Derivado, e o documento
        // guarda o nome INTEIRO** — é isso que mantém a renomeação a editar as chaves (ela semeia
        // do `Name` da entidade, não desta linha) e a busca a encontrar por valor de propriedade.
        // *O que se guarda é a autoria; o que se mostra é uma leitura dela.*
        paint_text(
            text_system,
            scene,
            ph2d_editor_core::screens::hero::variant_axes::display_name(&entity.name),
            name_x,
            rect.y + (rect.h - TypeToken::Sm.px()) * 0.5,
            TypeToken::Sm.px(),
            name_w,
            resolve(name_color, theme),
        );
        scene.pop_layer();
    }
}
