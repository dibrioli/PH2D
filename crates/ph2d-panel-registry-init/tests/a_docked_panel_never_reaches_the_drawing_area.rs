//! ⭐⭐⭐ **UM PAINEL QUE NÃO DECLARA FLUTUAR NUNCA CHEGA À ÁREA DE DESENHO** — a decisão **D1**,
//! medida sobre o quadro real.
//!
//! > *«Painéis de propriedade declaram que **não flutuam** ⇒ nunca chegam perto de uma viewport nem
//! > de uma régua. É um `Constraint` (o gesto errado torna-se inexprimível), não uma verificação.»*
//! > — `docs/UI_New_and_Simple/00_DECISOES_DO_ENIO.md`, D1
//!
//! # Por que o gate mora AQUI
//!
//! Ele pinta um quadro com **todos** os painéis abertos e lê os rects que eles **publicaram**. Isso
//! precisa do registry de painéis, que vive nesta crate — na `ph2d-editor-core` o
//! `test_support::ensure_panel_registry` é um `{}` e a varredura correria sobre zero painéis.
//!
//! # ⚠️ O quadro tem de ser pintado MAIS DE UMA VEZ, e isso é uma propriedade do desenho
//!
//! O `DockSides::from_published` lê os rects do quadro **anterior** — no primeiro quadro nenhuma
//! coluna está reservada e a área de desenho ocupa a **largura toda** (medido: `x=0, w=1366` no
//! 1.º, `x=308, w=754` a partir do 2.º). ⇒ um gate de um quadro só mediria o estado transitório e
//! acusaria toda a gente.
//!
//! # ⛔ A catraca, e o que ela nomeia
//!
//! [`REACHES_PENDING`] é a lista dos que **hoje** violam, cada um com o mecanismo. Ela só encolhe,
//! e a metade de baixo recusa uma entrada que já não descreve nada — *uma catraca sem censo de
//! obsolescência não desce: vira licença* (`CLAUDE.md` §5.0).

use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::screens::slot::Slot;
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;

/// ⛔ Os que ainda alcançam a área de desenho sem declarar que flutuam, **com o mecanismo**.
///
/// ⭐ **VAZIA desde 2026-08-30.** A única entrada era o `audio_editor`, que se encaixava a oeste do
/// Inspector para poder estar aberto ao lado do Mixer; hoje os dois são **abas** do mesmo encaixe
/// (`screens::hero::slot_tabs`) e ele ocupa a coluna como toda a gente. *A catraca desceu, e desceu
/// porque a metade de obsolescência a acusou primeiro.*
const REACHES_PENDING: &[(&str, &str)] = &[];

fn overlap(a: Rect, b: Rect) -> f32 {
    let w = (a.x + a.w).min(b.x + b.w) - a.x.max(b.x);
    let h = (a.y + a.h).min(b.y + b.h) - a.y.max(b.y);
    if w <= 0.0 || h <= 0.0 { 0.0 } else { w * h }
}

fn all_panel_ids() -> Vec<&'static str> {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut v = Vec::new();
    ph2d_editor_core::panel::with_registry_ref(|reg| {
        for p in reg.panels() {
            v.push(p.manifest.id);
        }
    });
    v
}

/// Pinta três quadros com **exactamente** este painel visível.
///
/// ⚠️⚠️ **Era «tudo aberto», e a feature das ABAS esvaziou-o.** Com todos visíveis, treze painéis
/// ocupam a coluna da direita e **doze ficam escondidos por abas** — a varredura passou de 18
/// medidos para **2**, e continuaria verde. *Um gate cuja população uma feature nova esvazia
/// passa a medir nada sem uma linha de erro*; o controlo `measured >= 10` foi o que o disse.
fn settled_one(only: &str) -> HeroScreen {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut h = HeroScreen::new(ph2d_editor_core::NodeId(1));
    for n in all_panel_ids() {
        h.panel_visibility.insert(n, n == only);
    }
    let mut scene = ph2d_vector::VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    let viewport = Rect::new(0.0, 0.0, 1366.0, 1024.0);
    for _ in 0..3 {
        ph2d_editor_core::screens::hero::paint_hero_screen(&mut h, viewport, &mut scene, &mut text);
    }
    h
}

#[test]
fn a_panel_that_does_not_declare_floating_never_reaches_the_drawing_area() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut offenders = Vec::new();
    let mut measured = 0usize;
    let mut floating = 0usize;
    let mut floats = std::collections::BTreeSet::new();
    ph2d_editor_core::panel::with_registry_ref(|reg| {
        for p in reg.panels() {
            if p.manifest.can_float {
                floating += 1;
                floats.insert(p.manifest.id);
            }
        }
    });

    for name in all_panel_ids() {
        if floats.contains(name) {
            continue;
        }
        let h = settled_one(name);
        let draw = h.last_layout.expect("o quadro publicou o layout").draw_area;
        assert!(
            draw.w > 0.0,
            "{name}: a área de desenho não assentou ({draw:?})"
        );
        let node = ph2d_editor_core::panel::with_registry_ref(|reg| {
            reg.panels()
                .iter()
                .find(|p| p.manifest.id == name)
                .map(|p| p.manifest.panel_node_id)
                .expect("registado")
        });
        let Some(rect) = h.store.panel_rect(node) else {
            continue; // não desenha sem a ferramenta dele activa
        };
        measured += 1;
        let o = overlap(rect, draw);
        if o > 0.0 && !REACHES_PENDING.iter().any(|(n, _)| *n == name) {
            offenders.push(format!(
                "{name} ({o:.0} px² sobre o desenho, rect={rect:?})"
            ));
        }
    }

    assert!(
        measured >= 10,
        "só {measured} painéis ancorados publicaram rect — a varredura ficou vazia e passaria \
         sobre nada"
    );
    assert!(
        floating >= 3,
        "só {floating} painéis declaram flutuar; o controlo de que a declaração é lida caiu"
    );
    assert!(
        offenders.is_empty(),
        "painéis que NÃO declaram flutuar e mesmo assim publicam rect sobre a área de desenho — \
         é a foto 2 da D1, e o artista vê um painel por cima do que está a desenhar:\n  {}",
        offenders.join("\n  ")
    );
}

/// ⭐ **A metade que impede a catraca de virar licença.**
#[test]
fn no_pending_entry_still_describes_nothing() {
    let mut cured = Vec::new();
    for (name, _) in REACHES_PENDING {
        let found = ph2d_editor_core::panel::with_registry_ref(|reg| {
            reg.panels()
                .iter()
                .find(|p| p.manifest.id == *name)
                .map(|p| (p.manifest.can_float, p.manifest.panel_node_id))
        });
        let Some((can_float, node)) = found else {
            cured.push(format!("{name} (já não é um painel registado)"));
            continue;
        };
        if can_float {
            cured.push(format!("{name} (passou a DECLARAR que flutua)"));
            continue;
        }
        let h = settled_one(name);
        let draw = h.last_layout.expect("layout").draw_area;
        match h.store.panel_rect(node) {
            None => cured.push(format!("{name} (já não publica rect)")),
            Some(r) if overlap(r, draw) == 0.0 => {
                cured.push(format!("{name} (já não alcança a área de desenho)"));
            }
            Some(_) => {}
        }
    }
    assert!(
        cured.is_empty(),
        "estas entradas da catraca já não descrevem nada — apague-as: {cured:?}"
    );
}

/// **E o que cada painel DECLARA é coerente consigo mesmo** — nasce onde pode estar.
#[test]
fn every_panel_is_born_inside_its_own_allowed_slots() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut bad = Vec::new();
    let mut center_claimants = Vec::new();
    let mut seen = 0usize;
    ph2d_editor_core::panel::with_registry_ref(|reg| {
        for p in reg.panels() {
            let m = &p.manifest;
            seen += 1;
            if m.allowed_slots.is_empty() {
                bad.push(format!(
                    "{}: `allowed_slots` VAZIO — não tem onde estar",
                    m.id
                ));
            } else if !m.allowed_slots.contains(m.default_slot) {
                bad.push(format!(
                    "{}: nasce em {:?}, que não está nos encaixes que ele permite",
                    m.id, m.default_slot
                ));
            }
            // ⛔⛔ **O centro admite UM ocupante, e ele não pode encaixar em mais lado nenhum.**
            //
            // ⚠️ Esta regra era *«ninguém declara o CENTER»*, e estava errada por omissão: o
            // `motion_graph` **é** o centro — ele parte a área de desenho em duas regiões IRMÃS
            // (`CenterSplit`), que é literalmente o que a decisão D5 diz que uma região é. Ele
            // declarava `RightTop` só porque o `DEFAULT_SLOT` tinha default, e as abas iriam
            // lê-lo como ocupante da coluna da direita — onde ele nunca esteve.
            //
            // ⇒ o que a regra defende de facto é *nenhum painel FLUTUA sobre a viewport*: quem
            // declara o centro declara **só** o centro, e por isso não pode docar em lado nenhum.
            if m.allowed_slots.contains(Slot::Center) {
                center_claimants.push(m.id);
                if m.allowed_slots != ph2d_editor_core::screens::slot::SlotSet::CENTER {
                    bad.push(format!(
                        "{}: declara o CENTER **e** um encaixe — o centro não é uma alternativa, \
                         é a região que ele PARTE",
                        m.id
                    ));
                }
            }
        }
    });
    assert!(
        seen >= 15,
        "só {seen} painéis vistos — o registry não é o do boot"
    );
    assert!(
        center_claimants.len() <= 1,
        "mais de um painel reclama o CENTRO, e ele não se divide entre docas: {center_claimants:?}"
    );
    assert!(bad.is_empty(), "declarações incoerentes: {bad:?}");
}
