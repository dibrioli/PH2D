//! ⭐⭐⭐ **A secção WEAPON é PINTADA, está VIVA sob o dedo, e mostra os números DO OBJECTO.**
//!
//! Irmã do [`super::a_seccao_ray_sensor_esta_viva`], e nasce no MESMO commit que a secção **por
//! causa dela**: aquela shipou com os seis campos pintados, registados e clicáveis — e a mostrar os
//! valores de **FÁBRICA**. Quem a apanhou foi uma FOTO, não um gate.
//!
//! ⚠️⚠️ *Números plausíveis são a pior forma deste defeito:* eles não parecem partidos, e o artista
//! que escreva por cima de UM **grava o default nos outros sete**.
//!
//! ⛔ **Nenhum gate de LEI pode vê-lo:** os da lei e do dreno entram **abaixo** do `WidgetStore`.
//! *A semente é a única metade da secção cujo sujeito é o widget*, e por isso ela precisa de um
//! gate que PINTE.

use ph2d_editor_core::weapon_edits::InspectorWeaponInfo;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::{InspectorPanel, InspectorState, ids, set_current_inspector_weapon};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 2400.0,
};

/// ⭐ Uma arma que **NÃO** é o `WeaponFire::default()` em nenhum dos oito campos — é isso que faz o
/// gate da semente discriminar: *um corpus no NEUTRO de um knob não testa esse knob*.
fn arma() -> InspectorWeaponInfo {
    InspectorWeaponInfo {
        entity_bits: 0x00AB_CDEF,
        on_signal: "fire".into(),
        cooldown_ms: 250,
        ammo_counter: "ammo".into(),
        reload_ms: 800,
        reload_on: "recarrega".into(),
        on_fire: "tiro".into(),
        on_empty: "seco".into(),
        on_reloaded: "cheio".into(),
        municao: Some(4),
        pente: 6,
        recarregando: false,
        clock_playing: true,
        selected_count: 1,
    }
}

fn host(i: InspectorWeaponInfo) -> (MockPanelHost, InspectorState) {
    let h = MockPanelHost::with_panel::<InspectorPanel>();
    set_current_inspector_weapon(Some(i));
    (h, InspectorState::default())
}

/// Os dois números da secção, com o valor que a fixtura tem.
const NUMEROS: [(ph2d_a11y::NodeId, f64); 2] = [
    (ids::INSP_WEAPON_COOLDOWN, 250.0),
    (ids::INSP_WEAPON_RELOAD_MS, 800.0),
];

/// Os seis nomes, com o texto que a fixtura tem.
const NOMES: [(ph2d_a11y::NodeId, &str); 6] = [
    (ids::INSP_WEAPON_ON_SIGNAL, "fire"),
    (ids::INSP_WEAPON_ON_FIRE, "tiro"),
    (ids::INSP_WEAPON_AMMO, "ammo"),
    (ids::INSP_WEAPON_RELOAD_ON, "recarrega"),
    (ids::INSP_WEAPON_ON_EMPTY, "seco"),
    (ids::INSP_WEAPON_ON_RELOADED, "cheio"),
];

/// ⭐⭐ **TODO campo da secção é pintado com área clicável.**
///
/// **Mutação que deve sangrar:** tirar qualquer linha do pintor da secção.
#[test]
fn todo_campo_da_seccao_e_pintado() {
    let (mut h, mut st) = host(arma());
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    for (id, _) in NUMEROS {
        let r = rects
            .iter()
            .find(|(n, _)| *n == id)
            .map(|(_, r)| *r)
            .unwrap_or_else(|| panic!("o campo {id:?} nao foi PINTADO com area clicavel"));
        assert!(r.w > 0.0 && r.h > 0.0, "campo {id:?} sem area: {r:?}");
    }
    for (id, _) in NOMES {
        assert!(
            rects.iter().any(|(n, _)| *n == id),
            "o campo de texto {id:?} nao foi pintado"
        );
    }
    set_current_inspector_weapon(None);
}

/// ⭐⭐⭐ **TODO campo está VIVO SOB O DEDO** — o gesto REAL, e não um `WidgetEvent` sintético.
///
/// ⚠️⚠️ **É este o gate que apanha o defeito que esta crate pagou SETE vezes:** um id que o
/// `populate` não regista é **pintado, hit-registado e MORTO** — o despachante decide pelo
/// `is_focusable`, e o ramo `None => false` engole o clique em silêncio. *Um `WidgetEvent`
/// sintético salta essa verificação, logo passaria com o `populate_weapon` vazio.*
///
/// **Mutação que deve sangrar:** tirar qualquer id do `populate_weapon`.
#[test]
fn todo_campo_esta_vivo_sob_o_dedo() {
    for (id, _) in NUMEROS
        .iter()
        .map(|(i, _)| (*i, ""))
        .chain(NOMES.iter().map(|(i, t)| (*i, *t)))
    {
        let (mut h, mut st) = host(arma());
        let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
        let r = rects
            .iter()
            .find(|(n, _)| *n == id)
            .map(|(_, r)| *r)
            .unwrap_or_else(|| panic!("{id:?} nao foi pintado"));
        let _ = h.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5);
        assert_eq!(
            h.store().focus_id(),
            Some(id),
            "clicar no meio de {id:?} nao lhe deu o foco — ele e' pintado e hit-registado, mas o \
             despachante engoliu o clique: falta o registo no `populate_weapon`, e ele esta' MORTO \
             SOB O DEDO"
        );
        set_current_inspector_weapon(None);
    }
}

/// ⭐⭐⭐ **OS CAMPOS MOSTRAM O QUE O OBJECTO TEM, e não os valores de fábrica** — o gate que a foto
/// da secção do RAIO encomendou.
///
/// **Mutação que deve sangrar:** tirar a chamada do `sync_weapon` do `sync_sections`.
#[test]
fn os_campos_mostram_o_que_o_objecto_tem() {
    let (mut h, mut st) = host(arma());
    let _ = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    for (id, esperado) in NUMEROS {
        let lido = h
            .store()
            .number_value(id)
            .unwrap_or_else(|| panic!("o campo {id:?} nem sequer esta' registado"));
        assert!(
            (lido - esperado).abs() < 1.0e-6,
            "o campo {id:?} mostra {lido} e o objecto tem {esperado} — o painel esta' a mostrar os \
             valores de FABRICA do `populate_weapon`"
        );
    }
    for (id, esperado) in NOMES {
        assert_eq!(
            h.store().text(id),
            Some(esperado),
            "o campo de texto {id:?} nao traz o nome que o objecto tem"
        );
    }
    set_current_inspector_weapon(None);
}

/// ⭐⭐ **E a MÃO do artista ganha ao instantâneo** — com o campo em foco, a semente não lhe toca.
///
/// **Mutação que deve sangrar:** tirar o `continue` do `focus == Some(id)`.
#[test]
fn a_mao_do_artista_ganha_a_semente() {
    let (mut h, mut st) = host(arma());
    let _ = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let id = ids::INSP_WEAPON_COOLDOWN;

    use ph2d_editor_core::panel::PanelHostInternal as _;
    h.store_mut().set_focus(Some(id));
    h.store_mut().set_number_value(id, 42.0);
    // E o instantâneo muda por baixo dele (outro objecto, outra aresta).
    let mut outra = arma();
    outra.entity_bits = 0x00FF_0001;
    outra.cooldown_ms = 900;
    set_current_inspector_weapon(Some(outra));
    let _ = h.paint::<InspectorPanel>(&mut st, VIEWPORT);

    assert!(
        (h.store().number_value(id).unwrap_or_default() - 42.0).abs() < 1.0e-6,
        "a semente escreveu por cima do campo em FOCO — o artista perde o que estava a escrever"
    );
    set_current_inspector_weapon(None);
}

/// ⭐⭐⭐ **A secção só existe para quem TEM a arma** (ADR-0166).
///
/// ⚠️ Metade NEGATIVA, e ela vale metade do gate: uma secção que aparecesse sempre poria oito
/// campos mortos em todo objecto da cena.
#[test]
fn sem_o_componente_a_seccao_nao_e_pintada() {
    set_current_inspector_weapon(None);
    let mut h = MockPanelHost::with_panel::<InspectorPanel>();
    let mut st = InspectorState::default();
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    for (id, _) in NUMEROS {
        assert!(
            !rects.iter().any(|(n, _)| *n == id),
            "o campo {id:?} foi pintado num objecto SEM arma"
        );
    }
}
