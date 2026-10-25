//! ⭐⭐ **A fileira de cada ITEM de lista tem canal** (F30 §A1) — o Inspector armado inteiro, uma
//! pintura, e cada lista regista as chaves do item ABERTO pelo `ItemId` dele (o da fixtura em
//! `o_inspector_armado`), nunca pela posição.

use ph2d_component_desc::lista::{self, codigo};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::widget::key_mark;
use ph2d_ui_testkit::MockPanelHost;

use super::quantas_entradas_tem_cada_painel::{VIEWPORT, abre_tudo};

/// As chaves que a pintura do Inspector armado registou.
fn registadas() -> Vec<[u16; 4]> {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut out = Vec::new();
    ph2d_editor_core::panel::with_registry(|reg| {
        let painel = reg
            .panels_mut()
            .iter_mut()
            .find(|p| p.manifest.id == "inspector")
            .expect("inspector");
        super::o_inspector_armado::arma_tudo();
        key_mark::set_key_marks(std::collections::BTreeMap::new());
        let mut host = MockPanelHost::new();
        painel.populate(host.store_mut());
        abre_tudo(host.store_mut());
        let _ = host.medindo_a_pintura_do_registo(painel, VIEWPORT);
        out = key_mark::canais_registados()
            .into_iter()
            .flat_map(|(_, ks)| ks)
            .collect();
        super::o_inspector_armado::desarma_tudo();
    });
    out
}

fn tem(lista_: u16, campos: &[u16], item: u32) {
    let r = registadas();
    for &f in campos {
        let k = lista::chaves(lista_, f, item);
        assert!(!k.is_empty(), "lista {lista_} #{f} não é canal");
        assert!(
            k.iter().all(|k| r.contains(k)),
            "a fileira do campo #{f} da lista {lista_} (item {item}) não registou o canal"
        );
    }
}

#[test]
fn a_fileira_do_timer_tem_canal() {
    tem(codigo::TIMERS, &[1, 2, 3, 4, 5], 101);
}

#[test]
fn a_fileira_do_tween_tem_canal() {
    tem(codigo::TWEENS, &[1, 2, 3, 4, 5, 6, 7], 102);
}

/// O relógio do tween e o do seguidor de caminho são o TIMER deles (o 101).
#[test]
fn as_linhas_do_relogio_sao_canal_do_timer() {
    tem(codigo::TIMERS, &[2, 3, 4], 101);
}

#[test]
fn a_fileira_do_estado_tem_canal() {
    tem(codigo::ESTADOS, &[1, 2, 3], 103);
}

#[test]
fn a_fileira_da_seta_tem_canal() {
    tem(codigo::SETAS, &[4, 5, 6], 104);
}

#[test]
fn a_fileira_da_regra_da_vigia_tem_canal() {
    tem(codigo::VIGIA, &[1, 2, 3, 4, 5, 6], 105);
}

#[test]
fn a_fileira_da_acao_tem_canal() {
    tem(codigo::ACOES, &[1, 2, 3, 4, 5, 6], 106);
}

#[test]
fn a_fileira_do_gatilho_tem_canal() {
    tem(codigo::GATILHOS, &[1, 2, 3], 107);
}

#[test]
fn a_fileira_da_fonte_de_abanao_tem_canal() {
    tem(codigo::ABANAO, &[0, 1, 2, 3, 4], 108);
}

#[test]
fn a_fileira_da_resistencia_tem_canal() {
    tem(codigo::RESISTENCIAS, &[1, 2, 3], 109);
}

#[test]
fn a_fileira_da_ancora_tem_canal() {
    tem(codigo::ANCORAS, &[1, 2, 3, 4, 5], 110);
}
