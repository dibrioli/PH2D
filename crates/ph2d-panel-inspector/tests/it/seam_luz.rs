//! **G8 — a costura da secção LUZ** (Luz 2D, W4 — `docs/Luz2D/01_plano.md` §4/§5): todo controlo
//! pintado chega ao barramento com a SUA edição, pelo gesto REAL (`click_at`, `drag_at`, digitar,
//! o seletor). O lado do mundo (a edição muda a `Luz`, o `Ctrl+Z` volta) está em
//! `ph2d-app-light2d` e na shell.

use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::luz_edits::{AvisoDaLuz, InspectorLuzInfo, LuzFieldEdit as E, TIPOS};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_inspector::{InspectorPanel, InspectorState, ids, set_current_inspector_luz};
use ph2d_ui_testkit::MockPanelHost;

const ENTITY: u64 = 0x1_0000_002D;
const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 320.0,
    h: 2400.0,
};

/// ⚠️ O tipo da fixtura é o ÚLTIMO (Ambiente) só onde o gate o pede; aqui é Ponto com alcance
/// FINITO, para os cinco controlos estarem todos pintados.
fn ponto() -> InspectorLuzInfo {
    InspectorLuzInfo {
        entity_bits: ENTITY,
        cor_srgb: [255, 200, 150],
        intensidade: 6.25,
        tamanho_m: 0.25,
        alcance_m: 10.0,
        tipo: 0,
        luz_a_1m: 1.0,
        avisos: vec![],
        qualidade: Some((
            ph2d_editor_core::luz_edits::NivelDaLuz::Alta,
            true,
            Some(3.2),
        )),
        selected_count: 1,
    }
}

fn pinta(
    info: InspectorLuzInfo,
) -> (
    MockPanelHost,
    InspectorState,
    Vec<(ph2d_a11y::NodeId, Rect)>,
) {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_luz(Some(info));
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    (host, state, rects)
}

fn ponteiro(kind: PointerKind, x: f32, y: f32, t: u128) -> PointerEvent {
    PointerEvent {
        kind,
        x,
        y,
        button: PointerButton::Primary,
        source: PointerSource::Mouse,
        pressure: 1.0,
        timestamp_ns: t * 1_000_000_000,
    }
}

fn rect_de(rects: &[(ph2d_a11y::NodeId, Rect)], id: ph2d_a11y::NodeId) -> Option<Rect> {
    rects.iter().find(|(n, _)| *n == id).map(|(_, r)| *r)
}

fn aplica(host: &mut MockPanelHost, state: &mut InspectorState, evs: Vec<WidgetEvent>) {
    for ev in evs {
        let _ = host.apply_panel_event::<InspectorPanel>(state, ev);
    }
}

fn edicoes(acoes: Vec<EditorAction>) -> Vec<E> {
    acoes
        .into_iter()
        .filter_map(|a| match a {
            EditorAction::InspectorComponentEdit {
                entity_bits,
                edit: ComponentEdit::Luz(e),
            } if entity_bits == ENTITY => Some(e),
            _ => None,
        })
        .collect()
}

fn clica(info: InspectorLuzInfo, id: ph2d_a11y::NodeId) -> Vec<E> {
    let (mut host, mut state, rects) = pinta(info);
    let r = rect_de(&rects, id).unwrap_or_else(|| panic!("a secção LUZ não pintou {id:?}"));
    let evs = host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5);
    assert!(
        !evs.is_empty(),
        "{id:?} está MORTO sob o dedo: o clique não produziu evento"
    );
    aplica(&mut host, &mut state, evs);
    let out = edicoes(host.drained_actions());
    set_current_inspector_luz(None);
    out
}

/// ⭐ Todo chip do TIPO manda a SUA posição, e os chips cobrem os tipos.
///
/// **Mutações que devem sangrar:** tirar o array do `populate_luz` · tirar o braço `Click` · mandar
/// sempre `0`.
#[test]
fn todo_chip_do_tipo_chega_ao_barramento_com_a_sua_tag() {
    assert_eq!(
        ids::INSP_LUZ_TIPO.len(),
        TIPOS.len(),
        "um tipo sem chip é inalcançável"
    );
    for (i, &id) in ids::INSP_LUZ_TIPO.iter().enumerate() {
        assert_eq!(clica(ponto(), id), vec![E::Tipo(u8::try_from(i).unwrap())]);
    }
}

/// ⭐⭐ Digitar em cada caixa chega ao barramento com a variante DELA.
///
/// **Mutações que devem sangrar:** trocar dois braços do `ValueChanged` · tirar um número do
/// `populate_luz`.
#[test]
fn cada_numero_digitado_chega_com_a_variante_dele() {
    for (id, texto, esperado) in [
        (ids::INSP_LUZ_INTENSIDADE, "3.5", E::Intensidade(3.5)),
        (ids::INSP_LUZ_TAMANHO, "0.75", E::Tamanho(0.75)),
        (ids::INSP_LUZ_ALCANCE, "4", E::Alcance(4.0)),
    ] {
        let (mut host, mut state, _) = pinta(ponto());
        let evs = host.type_into_number(id, texto);
        aplica(&mut host, &mut state, evs);
        assert_eq!(edicoes(host.drained_actions()), vec![esperado], "{id:?}");
        set_current_inspector_luz(None);
    }
}

/// ⭐⭐⭐ **ARRASTAR a Intensidade** publica a edição (o gesto do plano §4: «arrastar Intensidade
/// muda a luz no mesmo quadro») — com o valor NOVO, nunca o de partida.
#[test]
fn arrastar_a_intensidade_publica_o_valor_novo() {
    let (mut host, mut state, rects) = pinta(ponto());
    let r = rect_de(&rects, ids::INSP_LUZ_INTENSIDADE).expect("a Intensidade não foi pintada");
    let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    // O gesto da mão: o 1.º movimento passa o limiar (vira arrasto), o 2.º arrasta o valor.
    let mut evs = Vec::new();
    for (t, (kind, dx)) in [
        (PointerKind::Down, 0.0),
        (PointerKind::Move, 20.0), // LITERAL-PX-OK: além do limiar de arrasto
        (PointerKind::Move, 60.0), // LITERAL-PX-OK: o arrasto
        (PointerKind::Up, 60.0),   // LITERAL-PX-OK: idem
    ]
    .into_iter()
    .enumerate()
    {
        evs.extend(host.dispatch_pointer_event(ponteiro(kind, x + dx, y, t as u128 + 1)));
    }
    aplica(&mut host, &mut state, evs);
    let e = edicoes(host.drained_actions());
    set_current_inspector_luz(None);
    let Some(E::Intensidade(v)) = e.last() else {
        panic!("arrastar a Intensidade não publicou nada: {e:?}");
    };
    assert!(
        *v > ponto().intensidade,
        "arrastar para a direita sobe: {v}"
    );
}

/// ⭐⭐ A caixa «Limit Range»: com alcance, desligá-la manda `∞`; sem alcance, ligá-la manda o
/// número da caixa — e o NÚMERO só é pintado com o alcance ligado.
#[test]
fn a_caixa_do_alcance_alterna_entre_infinito_e_o_numero() {
    assert_eq!(
        clica(ponto(), ids::INSP_LUZ_ALCANCE_LIGADO),
        vec![E::Alcance(f32::INFINITY)]
    );
    let livre = InspectorLuzInfo {
        alcance_m: f32::INFINITY,
        ..ponto()
    };
    let (_, _, rects) = pinta(livre.clone());
    set_current_inspector_luz(None);
    assert!(
        rect_de(&rects, ids::INSP_LUZ_ALCANCE).is_none(),
        "sem alcance, a caixa do número não existe (∞ não se escreve)"
    );
    let e = clica(livre, ids::INSP_LUZ_ALCANCE_LIGADO);
    assert!(
        matches!(e.as_slice(), [E::Alcance(v)] if v.is_finite() && *v > 0.0),
        "ligar o alcance tem de mandar um número: {e:?}"
    );
}

/// ⭐⭐ A COR escolhida no seletor chega ao barramento em sRGB de 8 bits.
#[test]
fn a_cor_escolhida_chega_ao_barramento() {
    let (mut host, mut state, rects) = pinta(ponto());
    assert_eq!(
        host.store().widget_color(ids::INSP_LUZ_COR),
        Some([255, 200, 150, 255]),
        "a amostra não mostra a cor da lâmpada"
    );
    let r = rect_de(&rects, ids::INSP_LUZ_COR).expect("a amostra da cor não foi pintada");
    let _ = host.dispatch_pointer_event(ponteiro(
        PointerKind::Down,
        r.x + r.w * 0.5,
        r.y + r.h * 0.5,
        1,
    ));
    assert_eq!(
        host.store().picker_target(),
        Some(ids::INSP_LUZ_COR),
        "o seletor não abriu"
    );
    host.pick_colour_in_the_open_picker([10, 20, 30, 255]);
    let _ = host.drained_actions();
    let _ = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    let e = edicoes(host.drained_actions());
    set_current_inspector_luz(None);
    assert_eq!(e, vec![E::Cor([10, 20, 30])]);
}

/// ⭐ A AMBIENTE não pinta tamanho nem alcance (campos que não fariam nada).
#[test]
fn a_ambiente_nao_pinta_tamanho_nem_alcance() {
    let (_, _, rects) = pinta(InspectorLuzInfo { tipo: 1, ..ponto() });
    set_current_inspector_luz(None);
    assert!(rect_de(&rects, ids::INSP_LUZ_INTENSIDADE).is_some());
    for id in [
        ids::INSP_LUZ_TAMANHO,
        ids::INSP_LUZ_ALCANCE_LIGADO,
        ids::INSP_LUZ_ALCANCE,
    ] {
        assert!(rect_de(&rects, id).is_none(), "{id:?} pintado na Ambiente");
    }
}

/// ⭐ Cada AVISO pinta-se sem roubar o lugar dos controlos (a secção cresce, os campos continuam
/// alcançáveis) — a recusa que só o terminal via chega à tela.
#[test]
fn os_avisos_pintam_e_os_controlos_continuam_vivos() {
    let todos = vec![
        AvisoDaLuz::GradeGrandeDemais {
            lado: 9000,
            limite: 8192,
        },
        AvisoDaLuz::SolverIndisponivel,
        AvisoDaLuz::SemMemoria { mb: 4162 },
        AvisoDaLuz::SombraEngrossada { vezes: 2 },
        AvisoDaLuz::PontoSemTamanho,
        AvisoDaLuz::PequenaDemaisParaOZoom,
        AvisoDaLuz::ForaDaVista,
        AvisoDaLuz::SpritesPeloQuad {
            com_quad: 2,
            total: 5,
        },
        AvisoDaLuz::CantosArredondados { blocos: 3 },
    ];
    let (_, _, sem) = pinta(ponto());
    let (_, _, com) = pinta(InspectorLuzInfo {
        avisos: todos,
        selected_count: 3,
        ..ponto()
    });
    set_current_inspector_luz(None);
    let y = |r: &[(ph2d_a11y::NodeId, Rect)]| rect_de(r, ids::INSP_LUZ_INTENSIDADE).unwrap().y;
    assert!(
        y(&com) > y(&sem),
        "os avisos não ocuparam lugar: a Intensidade ficou em {} (sem avisos: {})",
        y(&com),
        y(&sem)
    );
    assert_eq!(
        clica(
            InspectorLuzInfo {
                avisos: vec![AvisoDaLuz::PequenaDemaisParaOZoom],
                ..ponto()
            },
            ids::INSP_LUZ_TIPO[1]
        ),
        vec![E::Tipo(1)]
    );
}
