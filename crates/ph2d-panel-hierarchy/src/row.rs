//! `paint_hierarchy_row` — per-row painter for the hierarchy panel.
//! Ported from `ph2d_editor_core::screens::hero::hierarchy::row_painter`
//! in Phase C.2; logic unchanged.

use crate::row_badge::badge_tone;
use crate::row_ink::RowInk;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::HitIndex;
use ph2d_editor_core::paint::{paint_icon, paint_text};
use ph2d_editor_core::screens::hero::fixture;
use ph2d_editor_core::widget::{Tag, TagState, paint_tag};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ICON_BTN_SIZE_PX, INLINE_ICON_PX, Spacing, StrokeToken, TypeToken};
use ph2d_vector::VectorScene;

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
    ink: RowInk,
) {
    // Num cartão o fundo é o da linha INTEIRA, recuo incluído: quem o pinta é o `cards.rs`.
    let RowInk::Panel(theme) = ink else {
        return;
    };
    // ⭐ **O REALCE DE PROVENIÊNCIA** (estudo de UI viva, C2): o objecto sob o ponteiro acende a
    // linha dele, venha o ponteiro do canvas ou desta mesma lista.
    //
    // ⚠️ **E é um tom SÓ, sem eixo** — a cerca do estudo §6.2: *o realce de uma lista OBEDECE ao
    // cursor*. Oito linhas meio-acesas ao mesmo tempo é rasto, não vida.
    //
    // ⭐⭐⭐ **A LEI vive na porta desde a wave 21** — `widget::paint_row_highlight` —, e ela
    // decide a precedência (escolhida vence apontada), o tom (o *pressed* do modelo, nunca o
    // repouso de um botão), o sangramento e a AUSÊNCIA de quinas.
    //
    // ⛔ **Este sítio perdeu o raio e a moldura de acento, e isso é a padronização** (report do
    // dono sobre a cadeia de efeitos, 2026-09-07): a hierarquia dizia *«esta é a linha em mãos»*
    // com uma caixa arredondada e contornada, e a cadeia dizia o mesmo com uma faixa a sangrar.
    // *Duas superfícies que dizem a mesma coisa de maneiras diferentes ensinam ao artista que elas
    // são coisas diferentes.* ⚠️ E era a moldura que fazia a linha escolhida ler-se como um botão,
    // que é o report que a cadeia levou **três** waves a fechar.
    ph2d_editor_core::widget::paint_row_highlight(
        scene,
        rect,
        theme,
        if entity.selected {
            ph2d_editor_core::widget::RowHighlight::Selected
        } else if entity.hovered {
            ph2d_editor_core::widget::RowHighlight::Hovered
        } else {
            ph2d_editor_core::widget::RowHighlight::None
        },
        // A hierarquia ocupa a largura toda do corpo: não há folga de cartão para transbordar.
        0.0,
    );
}

/// ⭐⭐⭐ **O alvo de clique de um companheiro tem a altura DA LINHA — nunca mais que ela.**
///
/// Os quatro companheiros desta linha (o galo, o olho, o grupo, o cadeado) inflavam o rectângulo
/// do ícone com uma folga própria: `16 + 2·4 = 24 px` para os três da direita, `12 + 12 = 24` para
/// o galo. Com a linha a **32 px** isso cabia; com a linha do app (**22**, wave 17) cada um passa
/// a transbordar **2 px** para a linha de cima — e no `HitIndex` **quem regista depois ganha**, ou
/// seja, a fatia de baixo da linha `N−1` passa a comandar o olho da linha `N`.
///
/// ⚠️ **Não é um caso limite: é o modo de falha que o dono reportaria como «o olho errado apagou»**
/// — e nenhum gate de registo o veria, porque o companheiro está vivo e registado; só está grande
/// demais. ⇒ a folga fica na HORIZONTAL (é lá que os ícones respiram uns dos outros) e a vertical
/// é a da linha, por construção. É o que o `Tree` do Godot faz: a célula de um botão **é** a linha.
pub(crate) fn row_tall(row: Rect, x: f32, w: f32) -> Rect {
    Rect::new(x, row.y, w, row.h)
}

/// ⭐⭐ **O RECUO INTERNO de uma linha — e ele era escrito em DOIS ficheiros.**
///
/// A linha declarava-o aqui e o desenhador das linhas de parentesco declarava-o outra vez em
/// `paint.rs`, cada um com metade de um comentário a mandar sincronizar (*«MUST match»* /
/// *«sync with row.rs»*). ⚠️ **Um par sincronizado à mão não é uma lei — é duas leis que hoje
/// concordam**, e a que derivasse punha a linha de parentesco ao lado da seta em vez de por
/// baixo dela, que é um report que este painel já pagou (Enio, 2026-05-26).
///
/// ⛔⛔ **O modelo discorda do VALOR, e o dono DECIDIU FICAR NOS 2 px** (2026-09-07, perguntado
/// com a divergência na mão). O Godot Modern dá `Tree.inner_item_margin_left = base_margin` =
/// **4 px**; esta casa fica nos **2**, e a divergência é deliberada, datada e gateada
/// (`the_hierarchy_row_inset_is_the_owners_two`).
///
/// ⚠️ **Não é teimosia contra o modelo — é a segunda vez que o dono aperta esta linha.** Em
/// 2026-05-24 ele já tinha mandado colar a seta ao ícone (`Xs`→`Xxs`) e o nome ao ícone
/// (`Md`→`Xs`, estendido a todo o app na wave 25). *Um veredito de produto medido no ecrã ganha de
/// um número portado, e três vereditos na mesma direcção são uma preferência, não um acaso.*
pub(crate) fn row_inset_px() -> f32 {
    Spacing::Xxs.px()
}

/// Onde começa o nome numa linha em `row_x`: seta, ícone e os vãos deles.
pub(crate) fn name_x(row_x: f32) -> f32 {
    row_x
        + row_inset_px()
        + ph2d_tokens::tree_chevron_col_px()
        + Spacing::Xxs.px()
        + Spacing::Xl.px()
        + ph2d_tokens::icon_label_gap_px()
}

/// Devolve onde o nome tem de acabar (o cacho da direita começa aí).
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_hierarchy_row(
    entity: &fixture::HierarchyEntity,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    ink: RowInk,
    row_id: Option<ph2d_a11y::NodeId>,
    mut hit_index: Option<&mut HitIndex>,
    has_children: bool,
    is_collapsed: bool,
    direct_match: bool,
) -> f32 {
    paint_row_background(entity, rect, scene, ink);
    // As DUAS medidas vêm de uma porta cada (wave 23) — antes eram cópias sincronizadas à mão
    // com `paint.rs`, e a que derivasse deslocava a linha de parentesco para fora da seta.
    let pad = row_inset_px();
    let chev_w = ph2d_tokens::tree_chevron_col_px();
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
            ink.color(ColorToken::Text2),
            StrokeToken::Default.px(),
        );
        if let (Some(row_id), Some(idx)) = (row_id, hit_index.as_mut()) {
            let hit_rect = row_tall(
                rect,
                chev_rect.x - Spacing::Sm.px(),
                chev_w + Spacing::Lg.px(),
            );
            idx.register(ids::hier_expand_companion(row_id), hit_rect);
        }
    }
    let icon_w = Spacing::Xl.px();
    let icon_x = chev_x + chev_w + chev_pad;
    let icon_rect = Rect::new(icon_x, rect.y + (rect.h - icon_w) * 0.5, icon_w, icon_w);
    let icon_color = icon_color(entity);
    paint_icon(
        scene,
        entity.icon,
        icon_rect,
        ink.color(icon_color),
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
        idx.register(crate::ids::hier_icon_companion(row_id), icon_rect);
    }

    // Right-side icon cluster — eye colada na borda direita (pad 0)
    // e gap inter-icon Xxs (2 px) pra ficarem "bem juntos" (Enio
    // 2026-05-26). Antes: pad=2 + gap=Sm (6).
    let icon_cluster_pad = Spacing::Sm.px(); // 8 px da borda direita (Enio 2026-05-26 round 2: "separe os olhos mais 4 pixels da lateral direita (desloque 4 pixels para esquerda)" — antes Xs=4)
    let icon_cluster_gap = 0.0_f32; // LITERAL-PX-OK: Enio 2026-05-26 "reduza a distância entre os ícones"
    let mut right_x = rect.x + rect.w - icon_cluster_pad;
    let eye_size = Spacing::Xl.px();
    // ⭐ Só um OBJECTO tem olho e travas: a Cena e o conteúdo não se escondem nem se trancam.
    if entity.role == ph2d_editor_core::hier_rows::RowRole::Object {
        let todas = crate::row_name::toggles_all_shown(entity);
        if todas || !entity.visible {
            right_x = paint_row_eye(
                entity,
                rect,
                scene,
                ink,
                row_id,
                hit_index.as_deref_mut(),
                right_x,
            );
        }
        right_x = paint_row_locks(
            entity,
            rect,
            scene,
            ink,
            row_id,
            hit_index.as_deref_mut(),
            (eye_size, icon_cluster_gap, todas),
            right_x,
        );
    }
    // Os ícones de componente seguem os interruptores: em repouso, nenhum (dono 11/11).
    if crate::row_name::toggles_all_shown(entity) {
        right_x = crate::row_components::paint_component_icons(
            entity, rect, scene, ink, row_id, hit_index, right_x,
        );
    }
    if let Some(swatch) = entity.swatch {
        let sw = INLINE_ICON_PX;
        let sw_rect = Rect::new(right_x - sw, rect.y + (rect.h - sw) * 0.5, sw, sw);
        // Canonical color swatch painter (single source of truth).
        let cs = ph2d_editor_core::widget::ColorSwatch::new(
            row_id.unwrap_or(ph2d_a11y::NodeId(0)),
            "",
            swatch,
        );
        ph2d_editor_core::widget::paint_color_swatch(&cs, sw_rect, scene, ink.theme());
        right_x -= sw + icon_cluster_gap;
    }
    if let Some(badge) = &entity.badge {
        let badge_h = TypeToken::Lg.px();
        let tone = badge_tone(badge);
        let tag = Tag::new(ph2d_a11y::NodeId(0), badge)
            .tone(tone)
            .state(if entity.muted {
                TagState::Disabled
            } else {
                TagState::Normal
            });
        // ⛔⛔⛔ **O SELO ERA CORTADO, e é uma linha que o dono vê em toda sessão.** Medido em
        //    2026-09-19: a caixa era o slot de ícone (`36 px`), que dá ao rótulo um orçamento de
        //    `20,00` — e `ENT` mede `22,13`, `CAM` `25,66`, `GRP` `22,41`, `LNK` `22,10`, `PRF`
        //    `20,67`, `SPR` `21,29`. **Seis dos sete selos deste painel saíam com reticência**, e
        //    só `ISO` (`18,54`) cabia. ⚠️ A varredura de elisões do app **não podia vê-lo**: um
        //    painel de fábrica não tem objecto nenhum, logo não pinta selo nenhum.
        //
        // ⭐ O **PISO** é o slot do ícone, e não um número escolhido: o selo é uma das ranhuras do
        //    cacho da direita, logo nunca fica mais estreito do que elas — ele só CRESCE quando a
        //    palavra que carrega pede mais.
        let badge_w = tag
            .natural_width(text_system, badge_h)
            .max(ICON_BTN_SIZE_PX);
        // O selo é adorno e o nome é por onde se acha a linha: sem lugar para os dois, sai o selo.
        let nome_x = icon_rect.x + icon_w + ph2d_tokens::icon_label_gap_px();
        let sobra = right_x - badge_w - icon_cluster_gap - Spacing::Xxs.px() - nome_x;
        if text_system.prefix_width(&entity.name, ink.name_px()) <= sobra {
            let badge_rect = Rect::new(
                right_x - badge_w,
                rect.y + (rect.h - badge_h) * 0.5,
                badge_w,
                badge_h,
            );
            paint_tag(&tag, badge_rect, scene, text_system, ink.theme());
            right_x -= badge_w + icon_cluster_gap;
        }
    }

    // Icon → name gap tightened Md (8) → Xs (4) 2026-05-24 per user:
    // "nome mais próximos dos ícones".
    let name_x = icon_rect.x + icon_w + ph2d_tokens::icon_label_gap_px();
    let name_color = if entity.muted {
        ColorToken::TextDisabled
    } else if direct_match {
        ColorToken::Accent
    } else {
        ColorToken::Text1
    };
    let cortado = paint_row_name(
        scene,
        text_system,
        ink,
        entity,
        rect,
        name_x,
        right_x,
        name_color,
    );
    if let Some(id) = row_id.filter(|_| cortado) {
        crate::row_name::mark_cut(id);
    }
    right_x - Spacing::Xxs.px()
}

/// ⭐ Um interruptor da linha (olho, grupo, cadeado) é pintável por arrasto — vertical (outras linhas)
/// e horizontal (as outras funções da mesma linha): um grupo só para os três (dono 01/11).
/// ⭐ **A cor do ícone da linha** (F42): a do TIPO num objecto — também seleccionado, é a identidade
/// dele —, cinzenta se escondido; a Cena e o conteúdo ficam como eram (acento na selecção).
pub(crate) fn icon_color(entity: &fixture::HierarchyEntity) -> ColorToken {
    use ph2d_editor_core::hier_types::ObjectType;
    let objecto = entity.role == ph2d_editor_core::hier_rows::RowRole::Object;
    let tipo = objecto.then(|| ObjectType::of_icon(entity.icon)).flatten();
    if entity.muted {
        ColorToken::TextDisabled
    } else if let Some(t) = tipo {
        t.color()
    } else if entity.selected {
        ColorToken::Accent
    } else {
        ColorToken::Text3
    }
}

fn pintavel(id: ph2d_a11y::NodeId, ligado: bool, rect: Rect) {
    ph2d_editor_core::interaction::toggle_paint::celula(
        id,
        crate::ids::HIER_TOGGLE_PAINT_GROUP,
        ligado,
        rect,
    );
}

/// **O OLHO** — o companheiro mais à direita. Saiu do [`paint_hierarchy_row`] pelo tecto de 200
/// LOC quando a linha ganhou os ícones de componente: recebe o `right_x` e devolve onde acaba.
fn paint_row_eye(
    entity: &fixture::HierarchyEntity,
    rect: Rect,
    scene: &mut VectorScene,
    ink: RowInk,
    row_id: Option<ph2d_a11y::NodeId>,
    hit_index: Option<&mut HitIndex>,
    right_x: f32,
) -> f32 {
    let (eye_icon, eye_color) = if entity.visible {
        (IconId::Eye, ColorToken::Text2)
    } else {
        (IconId::EyeClosed, ColorToken::TextDisabled)
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
        ink.color(eye_color),
        StrokeToken::Default.px(),
    );
    if let (Some(row_id), Some(idx)) = (row_id, hit_index) {
        let hit_pad = Spacing::Xs.px();
        let hit_rect = row_tall(rect, eye_rect.x - hit_pad, eye_rect.w + hit_pad * 2.0);
        idx.register(ids::hier_eye_companion(row_id), hit_rect);
        pintavel(ids::hier_eye_companion(row_id), entity.visible, hit_rect);
    }
    right_x - eye_size
}

/// ⭐ **O GRUPO e o CADEADO** — os dois companheiros de trava à esquerda do olho, com a cor a dizer o
/// estado. Destravados só se pintam com `todas` ([`crate::row_name::toggles_all_shown`]); travados,
/// sempre. Recebe o `right_x` onde o olho acabou e devolve onde o último pintado acaba.
#[allow(clippy::too_many_arguments)]
fn paint_row_locks(
    entity: &fixture::HierarchyEntity,
    rect: Rect,
    scene: &mut VectorScene,
    ink: RowInk,
    row_id: Option<ph2d_a11y::NodeId>,
    mut hit_index: Option<&mut HitIndex>,
    (eye_size, icon_cluster_gap, todas): (f32, f32, bool),
    mut right_x: f32,
) -> f32 {
    if todas || entity.group_locked {
        right_x = paint_row_group(
            entity,
            rect,
            scene,
            ink,
            (row_id, hit_index.as_deref_mut()),
            eye_size,
            right_x,
        ) - icon_cluster_gap;
    }
    if !(todas || entity.locked) {
        return right_x;
    }
    let icon_btn = eye_size;
    // ── Lock individual (cadeado) — Enio: "Cadeado trava apenas o
    // objeto. Se este objeto tiver filhos, os filhos podem ser
    // manipulados".
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
        ink.color(lock_color),
        StrokeToken::Default.px(),
    );
    if let (Some(row_id), Some(idx)) = (row_id, hit_index.as_mut()) {
        let hit_pad = Spacing::Xs.px();
        let hit_rect = row_tall(rect, lock_rect.x - hit_pad, lock_rect.w + hit_pad * 2.0);
        idx.register(crate::ids::hier_lock_companion(row_id), hit_rect);
        pintavel(
            crate::ids::hier_lock_companion(row_id),
            entity.locked,
            hit_rect,
        );
    }
    right_x -= icon_btn + icon_cluster_gap;
    right_x
}

/// O GRUPO (pasta) à esquerda do olho: clique alterna `GroupedChildren` (Enio 2026-05-26: «Agrupar: vc
/// pode manipular o pai mas não os filhos»). Ligado = Accent; desligado = Text3. Devolve onde acaba.
fn paint_row_group(
    entity: &fixture::HierarchyEntity,
    rect: Rect,
    scene: &mut VectorScene,
    ink: RowInk,
    (row_id, hit_index): (Option<ph2d_a11y::NodeId>, Option<&mut HitIndex>),
    icon_btn: f32,
    right_x: f32,
) -> f32 {
    let group_rect = Rect::new(
        right_x - icon_btn,
        rect.y + (rect.h - icon_btn) * 0.5,
        icon_btn,
        icon_btn,
    );
    let (group_icon, group_color) = if entity.group_locked {
        (IconId::Group, ColorToken::Accent)
    } else {
        (IconId::Ungroup, ColorToken::Text3)
    };
    paint_icon(
        scene,
        group_icon,
        group_rect,
        ink.color(group_color),
        StrokeToken::Default.px(),
    );
    if let (Some(row_id), Some(idx)) = (row_id, hit_index) {
        let hit_pad = Spacing::Xs.px();
        let hit_rect = row_tall(rect, group_rect.x - hit_pad, group_rect.w + hit_pad * 2.0);
        idx.register(crate::ids::hier_group_companion(row_id), hit_rect);
        pintavel(
            crate::ids::hier_group_companion(row_id),
            entity.group_locked,
            hit_rect,
        );
    }
    right_x - icon_btn
}

/// ⭐⭐ **O NOME da linha** — o que o artista lê, recortado ao espaço que sobra.
///
/// ⚠️ **Função irmã, e não um bloco no meio do pintor** (teto de 200 LOC por função): o rótulo
/// deixou de ser o `Name` cru quando as propriedades passaram a viajar entre chaves, e a lei disso
/// precisa de um sítio onde seja lida por quem for mexer nela.
///
/// # ⭐⭐⭐ A linha mostra o que o objecto É, não as propriedades dele
///
/// Enio, 2026-08-30: *«os nomes ficam grandes demais e nem cabem direito na hierarquia»*.
/// `Casa {Size=Small, State=Idle} (1)` desenha-se **`Casa (1) *²`** — o nome curto, o sufixo de
/// cópia intacto, e o selo do que ficou escondido.
///
/// ⚠️ **Derivado, e o documento guarda o nome INTEIRO** — é isso que mantém a renomeação a editar
/// as chaves (ela semeia do `Name` da entidade, não desta linha) e a busca a encontrar por valor de
/// propriedade. *O que se guarda é a autoria; o que se mostra é uma leitura dela.*
///
/// ⚠️ **E o `*²` diz QUANTAS, nunca QUAIS** — quem responde *quais* é o cartão de propriedades do
/// Inspector (`ph2d-panel-inspector/src/sections/properties.rs`), que é onde a pergunta pertence.
#[allow(clippy::too_many_arguments)]
fn paint_row_name(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    ink: RowInk,
    entity: &fixture::HierarchyEntity,
    rect: Rect,
    name_x: f32,
    right_x: f32,
    name_color: ColorToken,
) -> bool {
    // Hard clip ao espaço entre name_x e o icon cluster — sem clip,
    // `paint_text` faz wrap quando a string excede a largura (nomes
    // longos quebravam em 2 linhas E invadiam os ícones). Com clip,
    // letras à direita simplesmente não aparecem (Enio 2026-05-26:
    // "As letras da direita do nome que invadirão os ícones devem
    // simplesmente não aparecer. Não podem sobrepor os ícones").
    // Reservar gap pequeno antes do primeiro ícone (não colado).
    let name_right_limit = right_x - Spacing::Xxs.px();
    let name_w = (name_right_limit - name_x).max(0.0);
    if name_w <= 0.0 {
        return !entity.name.is_empty();
    }
    let name_clip = ph2d_vector::Rect::new(
        name_x as f64,
        rect.y as f64,
        (name_x + name_w) as f64,
        (rect.y + rect.h) as f64,
    );
    scene.push_clip(&name_clip);
    paint_text(
        text_system,
        scene,
        &entity.name,
        name_x,
        rect.y + (rect.h - ink.name_px()) * 0.5,
        ink.name_px(),
        name_w,
        ink.color(name_color),
    );
    scene.pop_layer();
    text_system.prefix_width(&entity.name, ink.name_px()) > name_w
}

#[cfg(test)]
#[path = "row_icon_color_tests.rs"]
mod icon_color_tests;
