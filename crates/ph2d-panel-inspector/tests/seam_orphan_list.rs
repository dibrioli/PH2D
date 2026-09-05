//! ⛔⛔ **AS EXCEPÇÕES SEM ALVO CHEGAM A PIXEL** (ADR-0164 / F5, critério 3).
//!
//! # Porque este gate mede GLIFOS, e não uma altura
//!
//! O critério pede que a excepção *«apareça na secção»*. Uma altura reservada (`y += line`) não é
//! pintura: o achado §4.2 da auditoria do `source.lsystem` mediu exactamente isso — um gate que
//! prometia *«a queixa chega a pixel»* lia o `content_h`, e **apagar a pintura inteira deixava-o
//! verde**. O arnês ganhou por causa disso um contador de glifos
//! ([`MockPanelHost::paint_and_count_geometry`]), e é ele que este ficheiro usa.
//!
//! ⚠️ **A régua tem de encher o balde nos dois sentidos:** mais órfãos ⇒ mais glifos, e um órfão
//! que NOMEIA a peça ⇒ mais glifos do que um que não a nomeia. *Uma régua que devolve o mesmo
//! número dos dois lados não distingue «não pintou» de «não vejo o que ele pintou».*

use ph2d_editor_core::screens::hero::{InspectorInstanceInfo, InspectorNameInfo, OrphanRow};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::{
    InspectorPanel, InspectorState, set_current_inspector_instance, set_current_inspector_name,
};
use ph2d_ui_testkit::MockPanelHost;

const ENTITY: u64 = 0x5EED_00B1;
const ROOT: u64 = 0x5EED_00B2;
const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 320.0,
    h: 2400.0,
};

fn info(rows: Vec<OrphanRow>) -> InspectorInstanceInfo {
    InspectorInstanceInfo {
        entity_bits: ENTITY,
        master_name: "Ragdoll".into(),
        overridden: Vec::new(),
        orphan_rows: rows,
        root_bits: ROOT,
        is_variant: false,
        apply_levels: Vec::new(),
        apply_levels_beyond: 0,
    }
}

fn orphan(component: &str, piece: &str) -> OrphanRow {
    OrphanRow {
        component: component.into(),
        piece: piece.into(),
    }
}

/// Pinta o cartão com estes órfãos e devolve quantos glifos foram encomendados.
fn glyphs(rows: Vec<OrphanRow>) -> u32 {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_name(Some(InspectorNameInfo {
        entity_bits: ENTITY,
        name: "Ragdoll".into(),
    }));
    set_current_inspector_instance(Some(info(rows)));
    let (g, _) = host.paint_and_count_geometry::<InspectorPanel>(&mut state, VIEWPORT);
    set_current_inspector_instance(None);
    set_current_inspector_name(None);
    g
}

/// ⭐⭐⭐ **Cada excepção sem alvo é uma LINHA que se vê** — e uma a mais dá mais glifos.
///
/// **Mutação que deve sangrar:** apagar o laço que pinta `info.orphan_rows` no cartão.
#[test]
fn every_unused_override_is_painted_as_its_own_line() {
    let none = glyphs(Vec::new());
    let one = glyphs(vec![orphan("Sprite", "Arm")]);
    let two = glyphs(vec![orphan("Sprite", "Arm"), orphan("Transform", "Leg")]);
    assert!(
        one > none,
        "um orfao nao acrescentou glifo nenhum: {none} -> {one} \u{2014} a linha nao chega a pixel"
    );
    assert!(
        two > one,
        "o segundo orfao nao acrescentou nada: {one} -> {two}"
    );
}

/// ⭐⭐ **E a linha NOMEIA a peça** — sem a metade que diz *de onde*, duas peças apagadas leem-se
/// `Sprite · Sprite` e o artista não sabe o que o botão *Clear* vai levar.
///
/// **Mutação que deve sangrar:** o `OrphanRow::label` devolver só o componente.
#[test]
fn the_line_names_the_piece_that_died() {
    let named = glyphs(vec![orphan("Sprite", "Arm")]);
    let nameless = glyphs(vec![orphan("Sprite", "")]);
    assert!(
        named > nameless,
        "nomear a peca nao mudou um glifo: {nameless} -> {named}"
    );
    assert_eq!(
        orphan("Sprite", "Arm").label(),
        "Sprite \u{2014} was on \u{201c}Arm\u{201d}"
    );
    assert_eq!(
        orphan("Sprite", "").label(),
        "Sprite",
        "sem peca a frase nao pode dizer que ela se chamava vazio"
    );
}

/// ⛔ **O botão promete o que a lista mostra** — o número dele é DERIVADO das linhas, e não uma
/// segunda contagem que discorda no dia em que uma entrada for saltada.
#[test]
fn the_clear_button_counts_exactly_the_lines_it_will_erase() {
    let i = info(vec![orphan("Sprite", "Arm"), orphan("Transform", "Leg")]);
    assert_eq!(i.orphans(), i.orphan_rows.len());
    assert_eq!(i.summary(), "Follows the component \u{b7} 2 unused");
}
