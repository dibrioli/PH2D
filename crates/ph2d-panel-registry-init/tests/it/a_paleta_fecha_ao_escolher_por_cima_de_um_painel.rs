//! ⭐⭐⭐ **ESCOLHER UM ITEM NA PALETA FECHA-A, MESMO QUANDO UM PAINEL CONHECE O MESMO ID.**
//!
//! # ⛔ O report que isto fecha (Enio, 2026-09-20)
//!
//! *«AO selecionar o pincel, o modal deveria se fechar automaticamente.»* A causa era a ROTA: um
//! painel registado que reconhecia o id do item **consumia** o clique antes do chrome, e a paleta
//! ficava aberta por cima de uma troca que acontecia por baixo dela. A cura pôs a paleta no
//! `pre_dispatch` — *enquanto um modal de ecrã inteiro está aberto, o ponteiro é dele.*
//!
//! # ⚠️ Porque ele vive AQUI, e porque o item é uma célula da FÍSICA
//!
//! Ele precisa do `HeroScreen` (da `ph2d-editor-core`) **e** de um painel REGISTADO que consome um
//! clique sem ser armado — e esta é a crate mais barata que enxerga os dois. A célula da matriz de
//! colisão (`PHYSICS_LAYER_CELL`) é consumida pelo painel de física incondicionalmente, logo um
//! item da paleta com esse id é exactamente o sujeito do defeito: *uma fixtura que não põe um
//! painel interessado por baixo mede o caminho onde o defeito não existe.* (Até ao ADR-0179 o
//! sujeito era a paleta de pincéis da escultura 3D, que saiu do produto.)

use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::widget::command_palette::{
    PaletteGroup, PaletteItem, PaletteModel, PaletteSub,
};

/// Uma paleta de UM item cujo id é o de uma célula viva do painel de física.
fn paleta_sobre_a_fisica() -> (PaletteModel, ph2d_editor_core::NodeId) {
    let id = ph2d_panel_physics::ids::PHYSICS_LAYER_CELL[0];
    let rotulo = ph2d_i18n::tr("panel.physics.section.layers").to_string();
    let modelo = PaletteModel {
        title: rotulo.clone(),
        toggle: None,
        groups: vec![PaletteGroup {
            title: rotulo.clone(),
            color: ph2d_tokens::ColorToken::NodeCatSource,
            subs: vec![PaletteSub {
                title: None,
                items: vec![PaletteItem { label: rotulo, id }],
            }],
        }],
    };
    (modelo, id)
}

/// ⭐⭐⭐ **Um clique no item: a paleta FECHA, o pick fica lá, e o painel de baixo NÃO age.**
///
/// *Mutação que sangra:* tirar o `command_palette_pointer` do `pre_dispatch` (o painel de física
/// consome o clique e a paleta fica aberta), ou o `apply` da paleta deixar de chamar
/// `close_command_palette` no ramo do item.
#[test]
fn escolher_um_item_fecha_a_paleta_e_o_painel_de_baixo_nao_age() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    // ⛔ Fila limpa ANTES: o que outro teste deixou na mesma thread não é deste clique.
    let _ = ph2d_panel_physics::drain_intents();

    let mut hero = HeroScreen::new(ph2d_editor_core::NodeId(1));
    let (modelo, item) = paleta_sobre_a_fisica();
    hero.store.open_command_palette(modelo);
    assert!(
        hero.store.command_palette_open(),
        "a paleta tinha de abrir — sem isso o resto deste gate não mede nada",
    );

    // ⛔⛔⛔ **PELA PORTA DO QUADRO — `HeroScreen::apply_event` — E NÃO PELO CHROME.** A rota é
    //    `pre_dispatch → os PAINÉIS (Consumed ⇒ return) → showcase → chrome::dispatch_all`; entrar
    //    pelo chrome é entrar ABAIXO da rotura e chamar-lhe a porta real.
    let consumiu = hero.apply_event(WidgetEvent::Click(item));
    let do_painel = ph2d_panel_physics::drain_intents();

    assert!(
        consumiu,
        "o clique num item tem de ser consumido pela paleta"
    );
    assert!(
        !hero.store.command_palette_open(),
        "a paleta ficou ABERTA depois de escolher — é o report do dono de 2026-09-20",
    );
    assert_eq!(
        hero.store.take_command_pick(),
        Some(item),
        "o pick não ficou registado, logo a ponte não teria o que aplicar",
    );
    assert!(
        do_painel.is_empty(),
        "o painel POR BAIXO da paleta agiu sobre o clique ({do_painel:?}) — o modal não é dono do \
         ponteiro",
    );
}
