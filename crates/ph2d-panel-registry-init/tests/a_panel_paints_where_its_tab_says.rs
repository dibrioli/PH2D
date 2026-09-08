//! ⭐⭐⭐ **O CORPO DE UM PAINEL VAI ONDE A ABA DELE FOI** — o censo que faltava desde que os
//! encaixes existem.
//!
//! # O report que o obrigou
//!
//! > *«se arrastar um para a área da Hierarquia e colapsar a hierarquia (puxando para esquerda) as
//! > abas do painel esquerdo ficam travadas»* — Enio, 2026-09-08.
//!
//! ⛔⛔ **Dois painéis — o Inspector e a Hierarquia — publicavam o rect a partir do `hero::paint`,
//! de FORA, com `layout.inspector` / `layout.hierarchy`**: *a coluna da direita* e *a coluna da
//! esquerda* **por nome**. Os outros vinte leem o `ctx.slot`, que é o encaixe **resolvido** — o
//! `slot_of` honra o encaixe que o artista arrumou por cima do que o painel declara.
//!
//! ⇒ arrastar a aba de um deles para a outra coluna movia a **aba** e deixava o **corpo** onde
//! sempre esteve. E como o `DockSides::from_published` responde *«esta coluna está ocupada?»*
//! cruzando os rects PUBLICADOS com o rect da coluna, a coluna de destino lia-se **VAZIA**: a fila
//! de abas ficava a flutuar sobre a área de desenho, com a alça de reabertura armada por baixo
//! dela. *Abas que se vêem, se clicam, e não trazem corpo nenhum.*
//!
//! # ⛔ Por que 22 200 testes verdes não viram isto
//!
//! O gate que existia — `the_slot_a_panel_declares_is_where_it_paints` — mede o `DEFAULT_SLOT`,
//! **um painel de cada vez, no sítio dele**. Ali `layout.inspector` e `slot_rects[RightTop]` são o
//! MESMO rect, então ler o campo errado é indistinguível de ler o certo. *A pergunta que separa os
//! dois só existe depois de alguém MOVER o painel* — e nenhum gate movia nenhum.
//!
//! # ⚠️ A fixtura tem de PRODUZIR o fenómeno, e para um painel ela não produz
//!
//! O `sculpt3d` não publica rect nenhum **nem no encaixe de fábrica** (o módulo 3D não está armado
//! sem a env var, e o `AppGfx.sculpt3d` é `None`). Medi-lo aqui seria medir silêncio, e acusá-lo
//! seria acusar um inocente. ⇒ ele é **saltado com a razão medida** — a condição é *«não publica no
//! próprio default»*, derivada, e não uma lista de nomes.
//!
//! ⛔ E é por isso que o gate **NOMEIA os dois** que tinham o defeito: sem esse controlo, um dia em
//! que todo painel ficasse silencioso deixaria a varredura verde sobre o vazio.

use ph2d_editor_core::screens::hero::{HeroScreen, slot_tabs};
use ph2d_editor_core::screens::slot::{Slot, SlotSet};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1366.0,
    h: 1024.0,
};

/// Quanto dois rects podem diferir e ainda ser «o mesmo sítio» — meio pixel.
const SAME_PLACE_PX: f32 = 0.5; // LITERAL-PX-OK: tolerância de comparação, não medida de desenho

struct Docked {
    id: &'static str,
    node: ph2d_editor_core::NodeId,
    allowed: SlotSet,
    default_slot: Slot,
}

/// Os painéis que ocupam encaixe — os `CAN_FLOAT` têm rect próprio e não respondem a esta lei.
fn docked() -> Vec<Docked> {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut v = Vec::new();
    ph2d_editor_core::panel::with_registry_ref(|reg| {
        for p in reg.panels() {
            let m = &p.manifest;
            if m.can_float {
                continue;
            }
            v.push(Docked {
                id: m.id,
                node: m.panel_node_id,
                allowed: m.allowed_slots,
                default_slot: m.default_slot,
            });
        }
    });
    v
}

/// Pinta quatro quadros com **só** este painel aberto, arrumado neste encaixe, e devolve o rect que
/// ele publicou e o rect do encaixe.
///
/// ⚠️ **Um painel de cada vez**: com vários abertos os outros ocupantes do encaixe ficam escondidos
/// por abas e não publicam nada — a varredura mediria dois e leria como aprovada.
fn place(p: &Docked, slot: Slot, all: &[Docked]) -> (Option<Rect>, Rect) {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut h = HeroScreen::new(ph2d_editor_core::NodeId(1));
    ph2d_editor_core::panel::with_registry_ref(|reg| {
        for q in reg.panels() {
            h.panel_visibility
                .insert(q.manifest.id, q.manifest.id == p.id);
        }
    });
    let _ = all;
    h.store.set_panel_slot(p.node, slot);
    let mut scene = ph2d_vector::VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    for _ in 0..4 {
        ph2d_editor_core::screens::hero::paint_hero_screen(&mut h, VIEWPORT, &mut scene, &mut text);
    }
    let layout = h.last_layout.expect("o quadro publicou o layout");
    let want = layout.slot_rects(slot_tabs::occupied(&h)).get(slot);
    (h.store.panel_rect(p.node), want)
}

fn same_place(a: Rect, b: Rect) -> bool {
    (a.x - b.x).abs() < SAME_PLACE_PX && (a.w - b.w).abs() < SAME_PLACE_PX
}

/// ⭐⭐⭐ **Todo painel encaixado publica o rect do encaixe em que ELE está** — em cada encaixe que
/// ele próprio declara aceitar.
#[test]
fn every_docked_panel_paints_where_its_tab_says() {
    let all = docked();
    let mut wrong: Vec<String> = Vec::new();
    let mut silent: Vec<&'static str> = Vec::new();
    let mut measured: Vec<&'static str> = Vec::new();

    for p in &all {
        // ⚠️ **O controlo por painel vem primeiro:** quem não publica no próprio encaixe de fábrica
        //    não tem fenómeno para esta régua ver, e medi-lo produziria uma acusação fabricada.
        if place(p, p.default_slot, &all).0.is_none() {
            silent.push(p.id);
            continue;
        }
        measured.push(p.id);
        for slot in Slot::ALL {
            if !p.allowed.contains(slot) || slot.dock_side().is_none() {
                continue;
            }
            let (got, want) = place(p, slot, &all);
            match got {
                Some(r) if same_place(r, want) => {}
                Some(r) => wrong.push(format!(
                    "{} arrumado em {slot:?}: o encaixe está em x={:.0} w={:.0} e ele publicou \
                     x={:.0} w={:.0}",
                    p.id, want.x, want.w, r.x, r.w
                )),
                None => wrong.push(format!(
                    "{} arrumado em {slot:?} não publicou rect nenhum (publica no default)",
                    p.id
                )),
            }
        }
    }

    // ⛔ **O controlo da POPULAÇÃO, e ele nomeia os dois que tinham o defeito.** Sem isto, um dia em
    //    que todo painel ficasse silencioso deixaria a varredura verde sobre zero células.
    for anchor in ["inspector", "hierarchy"] {
        assert!(
            measured.contains(&anchor),
            "«{anchor}» deixou de publicar rect no encaixe de fábrica, e ele é um dos DOIS que \
             tinham o defeito de 2026-09-08 — sem ele a varredura mede outra coisa. Silenciosos: \
             {silent:?}"
        );
    }

    assert!(
        wrong.is_empty(),
        "painéis cujo CORPO não segue a ABA — é o report de 2026-09-08 («as abas do painel \
         esquerdo ficam travadas»):\n  {}\n\n({} painéis medidos, {} saltados por não publicarem \
         nem no default: {silent:?})",
        wrong.join("\n  "),
        measured.len(),
        silent.len(),
    );
}
