//! **As portas da face VAZIA do §11** — o que se oferece a um objeto que ainda
//! **não** é físico.
//!
//! Irmão do `physics.rs` pelo cap de 600 LOC, cortado na linha que o doc daquele
//! já desenhava: *o que este corpo É* × **o que CRIAR aqui**. São três portas, e
//! cada uma responde a uma pergunta diferente:
//!
//! - **Add Physics Body** — torna ESTE objeto um corpo;
//! - **Add Shape to X** (W-Compound) — torna esta forma mais uma peça do corpo
//!   ancestral, em vez de um corpo novo;
//! - **Rig N Parts** (W-Rig) — torna o personagem INTEIRO físico, lendo a árvore
//!   que o artista já desenhou.
//!
//! ⚠️ **A face vazia não é uma borda, é o caso NORMAL de duas delas.** Um
//! personagem nasce como sprites parenteados, sem corpo em lugar nenhum — as
//! rotas de LIGAR (join/chain) não aparecem aqui porque precisam de corpos, e
//! estas aparecem porque os criam.

use super::*;

/// Pinta as portas e devolve o `y` final da seção.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_empty_face(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    info: &ph2d_editor_core::screens::hero::InspectorPhysicsInfo,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let mut yy = y;
    let h = ROW_H_PX;
    let label_font = TypeToken::Sm.px();
    // The door. One line of explanation, because "Add Physics Body" on a
    // sprite that is about to start falling deserves to say so.
    paint_text(
        text_system,
        scene,
        "Not simulated \u{00b7} add a body to make it fall and collide",
        x,
        yy + (h - label_font) * 0.5,
        label_font,
        w,
        resolve(ColorToken::Text3, theme),
    );
    yy += h;
    let btn_rect = Rect::new(x, yy, w, h);
    let btn = Button::new(ids::INSP_PHYS_ADD, "Add Physics Body")
        .kind(ButtonKind::Default)
        .visual(store.button_visual(ids::INSP_PHYS_ADD));
    paint_button(&btn, btn_rect, scene, text_system, theme);
    hit_index.register(ids::INSP_PHYS_ADD, btn_rect);
    yy += h;
    // **A TERCEIRA porta** (W-Compound): as duas acima fazem um CORPO; esta faz
    // desta forma mais uma peça do corpo que já existe acima na árvore. Só é
    // oferecida quando há um — uma peça pendurada em nada não é nada, e o
    // rótulo NOMEIA o dono porque um collider é invisível e a hierarquia pode
    // ter um grupo no meio.
    if !info.part_owner.is_empty() {
        let rect = Rect::new(x, yy, w, h);
        let btn = Button::new(
            ids::INSP_PHYS_ADD_SHAPE,
            format!("Add Shape to {}", info.part_owner),
        )
        .kind(ButtonKind::Default)
        .visual(store.button_visual(ids::INSP_PHYS_ADD_SHAPE));
        paint_button(&btn, rect, scene, text_system, theme);
        hit_index.register(ids::INSP_PHYS_ADD_SHAPE, rect);
        yy += h;
    }
    // **E a segunda resposta à mesma pergunta** (W-Rig): a porta acima torna
    // ESTE objeto físico; o rig torna o personagem INTEIRO físico e o monta,
    // lendo a árvore que o artista já desenhou.
    //
    // ⚠️ **A face vazia é o caso NORMAL do rig, não uma borda** — um
    // personagem nasce como sprites parenteados, sem corpo em lugar nenhum.
    // Deixá-lo só na face com corpo faria o gerador exigir o passo manual que
    // ele existe para remover (é por isso que as rotas de LIGAR não aparecem
    // aqui e esta aparece: elas precisam de corpos, esta os cria).
    if info.rig_parts > 0 {
        let rect = Rect::new(x, yy, w, h);
        let btn = Button::new(
            ids::INSP_PHYS_RIG,
            super::physics_join_rows::rig_button_label(info.rig_parts),
        )
        .kind(ButtonKind::Default)
        .visual(store.button_visual(ids::INSP_PHYS_RIG));
        paint_button(&btn, rect, scene, text_system, theme);
        hit_index.register(ids::INSP_PHYS_RIG, rect);
        yy += h;
    }
    yy + SECTION_BOTTOM_PAD_PX
}

/// As portas da face de **PEÇA** (W-PartFace): as duas saídas de um `Collider`
/// que não é corpo.
///
/// ⚠️ **São os MESMOS dois ids das outras faces**, e de propósito — cada um faz
/// exatamente a mesma coisa ao ECS (`INSP_PHYS_ADD` anexa um `RigidBody`,
/// `INSP_PHYS_REMOVE` tira corpo e collider) e o que muda é a **consequência**,
/// que é o que o rótulo tem de nomear: numa peça, ganhar um corpo é *deixar de
/// ser peça*, e perder o collider é *voltar a ser só desenho*. Dois ids novos
/// seriam duas rotas para a mesma operação, e é assim que elas divergem.
///
/// ⚠️ **NÃO se oferece *Add Shape to X* aqui** — foi o que a wave veio corrigir:
/// a porta que CRIA a peça, clicada de novo, reescreve o collider com os defaults
/// e apaga a forma autorada em silêncio (medido: barra `0,17 × 0,91` → caixa do
/// sprite `0,20 × 1,00`, offset/densidade/camada zerados). A recusa mora nas duas
/// metades: o painter não a pinta e o handler não a honra.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_part_doors(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let mut yy = y;
    let h = ROW_H_PX;
    for (id, label) in [
        (ids::INSP_PHYS_ADD, "Make Independent Body"),
        (ids::INSP_PHYS_REMOVE, "Remove Shape"),
    ] {
        let rect = Rect::new(x, yy, w, h);
        let btn = Button::new(id, label)
            .kind(ButtonKind::Default)
            .visual(store.button_visual(id));
        paint_button(&btn, rect, scene, text_system, theme);
        hit_index.register(id, rect);
        yy += h;
    }
    yy + SECTION_BOTTOM_PAD_PX
}
