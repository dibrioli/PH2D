//! ⭐⭐⭐ **A secção HEALTH BAR é PINTADA, está VIVA sob o dedo, mostra os números DO OBJECTO e leva
//! a cor escolhida ao DOCUMENTO** (plano 28, W4).
//!
//! Irmã do [`super::a_seccao_vida_esta_viva`] — ficheiro próprio porque a barra é a primeira secção
//! da família com AMOSTRAS de cor, e as amostras têm o regime delas: ⚠️ o despacho abre o selector e
//! **não emite evento nenhum**, logo a régua de um clique numa amostra é o EFEITO (o selector
//! apontado, a semente certa), nunca um `WidgetEvent`.

use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::vida_edits::{
    BarraAlvo, InspectorBarInfo, InspectorHealthInfo, InspectorVidaInfo, VidaFieldEdit as E,
};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_inspector::{InspectorPanel, InspectorState, ids, set_current_inspector_vida};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 3200.0,
};
const SEC: u128 = 1_000_000_000;
const BITS: u64 = 0x00BA_7700;

/// ⭐ Uma barra que **NÃO** é o `HealthBar::default()` em nenhum campo — *um corpus no neutro de um
/// knob não testa esse knob*.
fn barra() -> InspectorBarInfo {
    InspectorBarInfo {
        target: "Heroi".into(),
        width: 2.5,
        height: 0.3,
        offset_x: -0.4,
        offset_y: 1.25,
        fill: [0.2, 0.4, 0.6, 1.0],
        trail: [1.0, 0.8, 0.0, 1.0],
        back: [0.0, 0.0, 0.2, 0.5],
        trail_delay_s: 0.9,
        trail_speed: 2.5,
        hide_when_full: false,
        alvo: BarraAlvo::Mostra {
            pontos: 30.0,
            max: 100.0,
        },
    }
}

/// Um objecto que só tem a barra — a do PLACAR.
fn info(b: InspectorBarInfo) -> InspectorVidaInfo {
    InspectorVidaInfo {
        entity_bits: BITS,
        health: None,
        damage: None,
        bar: Some(b),
        has_body: false,
        clock_playing: true,
        selected_count: 1,
    }
}

fn host(i: InspectorVidaInfo) -> (MockPanelHost, InspectorState) {
    let h = MockPanelHost::with_panel::<InspectorPanel>();
    set_current_inspector_vida(Some(i));
    (h, InspectorState::default())
}

fn rect_de(rects: &[(ph2d_a11y::NodeId, Rect)], id: ph2d_a11y::NodeId) -> Option<Rect> {
    rects.iter().find(|(n, _)| *n == id).map(|(_, r)| *r)
}

fn pointer(kind: PointerKind, x: f32, y: f32, t: u128) -> PointerEvent {
    PointerEvent {
        kind,
        x,
        y,
        button: PointerButton::Primary,
        source: PointerSource::Mouse,
        pressure: 1.0,
        timestamp_ns: t,
    }
}

fn edicoes(acoes: Vec<EditorAction>) -> Vec<E> {
    acoes
        .into_iter()
        .filter_map(|a| match a {
            EditorAction::InspectorComponentEdit {
                entity_bits,
                edit: ComponentEdit::Vida(e),
            } => {
                assert_eq!(entity_bits, BITS);
                Some(e)
            }
            _ => None,
        })
        .collect()
}

/// Os SEIS números da barra, com o valor da fixtura.
const NUMEROS: [(ph2d_a11y::NodeId, f64); 6] = [
    (ids::INSP_BARRA_WIDTH, 2.5),
    (ids::INSP_BARRA_HEIGHT, 0.3),
    (ids::INSP_BARRA_OFFSET_X, -0.4),
    (ids::INSP_BARRA_OFFSET_Y, 1.25),
    (ids::INSP_BARRA_TRAIL_DELAY, 0.9),
    (ids::INSP_BARRA_TRAIL_SPEED, 2.5),
];

/// ⭐⭐ **TODO campo da secção é pintado com área clicável** — os seis números, o alvo, a caixa e
/// as três amostras.
///
/// **Mutação que deve sangrar:** tirar qualquer linha do `corpo_barra`.
#[test]
fn todo_campo_da_barra_e_pintado() {
    let (mut h, mut st) = host(info(barra()));
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let todos = NUMEROS
        .iter()
        .map(|(i, _)| *i)
        .chain([ids::INSP_BARRA_TARGET, ids::INSP_BARRA_HIDE_FULL])
        .chain(ids::INSP_BARRA_CORES);
    for id in todos {
        let r = rect_de(&rects, id)
            .unwrap_or_else(|| panic!("o campo {id:?} nao foi PINTADO com area clicavel"));
        assert!(r.w > 0.0 && r.h > 0.0, "campo {id:?} sem area: {r:?}");
    }
    // ⚠️ E um objecto SÓ com barra não pinta a secção da vida — ADR-0166.
    assert!(
        rect_de(&rects, ids::INSP_VIDA_MAX).is_none(),
        "a secção HEALTH foi pintada num placar sem vida"
    );
    set_current_inspector_vida(None);
}

/// ⭐⭐⭐ **Os números e o alvo estão VIVOS sob o dedo** — o gesto REAL.
///
/// **Mutação que deve sangrar:** tirar qualquer id da barra do `populate_vida`.
#[test]
fn os_numeros_e_o_alvo_estao_vivos_sob_o_dedo() {
    for id in NUMEROS
        .iter()
        .map(|(i, _)| *i)
        .chain([ids::INSP_BARRA_TARGET])
    {
        let (mut h, mut st) = host(info(barra()));
        let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
        let r = rect_de(&rects, id).unwrap_or_else(|| panic!("{id:?} nao foi pintado"));
        let _ = h.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5);
        assert_eq!(
            h.store().focus_id(),
            Some(id),
            "clicar no meio de {id:?} nao lhe deu o foco — ele esta' MORTO SOB O DEDO"
        );
        set_current_inspector_vida(None);
    }
}

/// ⭐⭐⭐ **OS CAMPOS MOSTRAM O QUE O OBJECTO TEM, e não os valores de fábrica** — e as amostras
/// também (fora do selector, a amostra segue o documento).
///
/// **Mutações que devem sangrar:** tirar a barra do `numeros`/`textos` do `sync_vida` · tirar a
/// re-semente das amostras.
#[test]
fn os_campos_e_as_amostras_mostram_o_que_o_objecto_tem() {
    let (mut h, mut st) = host(info(barra()));
    let _ = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    for (id, esperado) in NUMEROS {
        let lido = h.store().number_value(id).unwrap();
        assert!(
            (lido - esperado).abs() < 1.0e-5,
            "o campo {id:?} mostra {lido} e o objecto tem {esperado}"
        );
    }
    assert_eq!(h.store().text(ids::INSP_BARRA_TARGET), Some("Heroi"));
    let [fill, trail, back] = ids::INSP_BARRA_CORES;
    assert_eq!(h.store().widget_color(fill), Some([51, 102, 153, 255]));
    assert_eq!(h.store().widget_color(trail), Some([255, 204, 0, 255]));
    assert_eq!(h.store().widget_color(back), Some([0, 0, 51, 128]));
    set_current_inspector_vida(None);
}

/// ⭐⭐⭐ **Carregar numa amostra APONTA o selector a ela e não escreve no documento** — a régua é o
/// EFEITO, porque o despacho curto-circuita e não emite evento.
///
/// **Mutação que deve sangrar:** tirar o `register_picker_swatch` das amostras da barra.
#[test]
fn uma_amostra_abre_o_selector_e_o_clique_nao_escreve_nada() {
    for (k, id) in ids::INSP_BARRA_CORES.into_iter().enumerate() {
        let (mut h, mut st) = host(info(barra()));
        let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
        let r = rect_de(&rects, id).expect("a amostra nao foi pintada");
        let _ = h.drained_actions();
        let _ = h.dispatch_pointer_event(pointer(
            PointerKind::Down,
            r.x + r.w * 0.5,
            r.y + r.h * 0.5,
            SEC,
        ));
        assert_eq!(
            h.store().picker_target(),
            Some(id),
            "a amostra {k} nao apontou o selector a ela"
        );
        assert_eq!(
            edicoes(h.drained_actions()),
            Vec::<E>::new(),
            "abrir o selector escreveu no documento"
        );
        set_current_inspector_vida(None);
    }
}

/// ⭐⭐⭐ **A cor ESCOLHIDA chega ao documento, na variante DA amostra** — a segunda metade, e a que
/// morre em silêncio: sem o braço do selector na semente, o fio acaba no `widget_color`.
///
/// **Mutações que devem sangrar:** apagar o `push` do `cores` · trocar duas variantes.
#[test]
fn a_cor_escolhida_chega_ao_documento_na_variante_da_amostra() {
    let verde = [0, 255, 0, 255];
    let esperado = [
        E::BarFill([0.0, 1.0, 0.0, 1.0]),
        E::BarTrail([0.0, 1.0, 0.0, 1.0]),
        E::BarBack([0.0, 1.0, 0.0, 1.0]),
    ];
    for (id, e) in ids::INSP_BARRA_CORES.into_iter().zip(esperado) {
        let (mut h, mut st) = host(info(barra()));
        let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
        let r = rect_de(&rects, id).unwrap();
        let _ = h.dispatch_pointer_event(pointer(
            PointerKind::Down,
            r.x + r.w * 0.5,
            r.y + r.h * 0.5,
            SEC,
        ));
        h.pick_colour_in_the_open_picker(verde);
        let _ = h.drained_actions();
        let _ = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
        assert_eq!(edicoes(h.drained_actions()), vec![e]);
        set_current_inspector_vida(None);
    }
}

/// ⭐⭐⭐ **Escrever num campo chega ao BARRAMENTO com a variante DELE** — números, alvo e caixa.
///
/// **Mutações que devem sangrar:** tirar qualquer braço da barra do `event_vida` · a caixa deixar
/// de inverter.
#[test]
fn cada_campo_chega_ao_barramento_com_a_variante_dele() {
    let esperado = [
        (ids::INSP_BARRA_WIDTH, E::BarWidth(7.0)),
        (ids::INSP_BARRA_HEIGHT, E::BarHeight(7.0)),
        (ids::INSP_BARRA_OFFSET_X, E::BarOffsetX(7.0)),
        (ids::INSP_BARRA_OFFSET_Y, E::BarOffsetY(7.0)),
        (ids::INSP_BARRA_TRAIL_DELAY, E::BarTrailDelayS(7.0)),
        (ids::INSP_BARRA_TRAIL_SPEED, E::BarTrailSpeed(7.0)),
    ];
    for (id, e) in esperado {
        let (mut h, mut st) = host(info(barra()));
        let _ = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
        h.set_number_value(id, 7.0);
        let _ = h.apply_panel_event::<InspectorPanel>(&mut st, WidgetEvent::ValueChanged(id));
        assert_eq!(edicoes(h.drained_actions()), vec![e], "o número {id:?}");
        set_current_inspector_vida(None);
    }
    let (mut h, mut st) = host(info(barra()));
    let _ = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    h.set_text(ids::INSP_BARRA_TARGET, "Chefe");
    let _ = h.apply_panel_event::<InspectorPanel>(
        &mut st,
        WidgetEvent::TextChanged(ids::INSP_BARRA_TARGET),
    );
    assert_eq!(
        edicoes(h.drained_actions()),
        vec![E::BarTarget("Chefe".into())]
    );
    let _ = h.apply_panel_event::<InspectorPanel>(
        &mut st,
        WidgetEvent::Toggled(ids::INSP_BARRA_HIDE_FULL),
    );
    assert_eq!(
        edicoes(h.drained_actions()),
        vec![E::BarHideWhenFull(true)],
        "a caixa pede o CONTRÁRIO do que o objecto tem"
    );
    set_current_inspector_vida(None);
}

/// ⚠️ **Com vida E barra no mesmo objecto (o inimigo) as duas secções pintam-se**, e a barra não
/// rouba o lugar da vida — o CONTROLO de que a terceira secção não é um `else` da primeira.
#[test]
fn a_barra_do_inimigo_mora_ao_lado_da_vida_dele() {
    let vida = InspectorHealthInfo {
        max: 30.0,
        start: 30.0,
        invincible_s: 0.0,
        overheal: false,
        regen: 0.0,
        regen_delay_s: 0.0,
        shield_start: 0.0,
        shield_max: 0.0,
        shield_duration_s: 5.0,
        shield_regen: 0.0,
        shield_regen_delay_s: 0.0,
        shield_blocks_excess: false,
        armor_flat: 0.0,
        armor_percent: 0.0,
        dodge: 0.0,
        team: String::new(),
        on_damage: String::new(),
        on_heal: String::new(),
        on_death: String::new(),
        seed: 0,
        agora: None,
    };
    let mut i = info(barra());
    i.health = Some(vida);
    i.has_body = true;
    let (mut h, mut st) = host(i);
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let vida_y = rect_de(&rects, ids::INSP_VIDA_MAX).expect("a vida sumiu").y;
    let barra_y = rect_de(&rects, ids::INSP_BARRA_WIDTH)
        .expect("a barra sumiu")
        .y;
    assert!(barra_y > vida_y, "a barra pinta-se DEPOIS da vida");
    set_current_inspector_vida(None);
}
