//! ⭐⭐ **F44 — Fill e Stroke no layout do Figma** (dono 01/11: *«o MESMO padrão de Effects/Filters»*
//! e *«modernizar Cap, Join, Align num modo moderno como Figma, com ícones inteligíveis»*).
//!
//! Cada gate CLICA pelo ponteiro (pintar → rect → Down/Up → dispatcher → painel): o «+» do título,
//! as linhas (base e camadas extra da pilha), a caixa que abre o painel à ESQUERDA, a linha fixa
//! Position · Weight · ⚙ e o «Stroke settings».

use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_vector::state::{self, Appearance, PaintRow, VectorPanelState};
use ph2d_panel_vector::{VectorPanel, ids};
use ph2d_tool_vector::ids as tv;
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 4000.0,
};

fn pintado(h: &mut MockPanelHost, id: NodeId) -> Option<Rect> {
    h.painted_rect::<VectorPanel>(&mut VectorPanelState, VIEWPORT, id)
}

fn camada(is_fill: bool) -> PaintRow {
    PaintRow {
        is_fill,
        color: [0x33, 0x66, 0xCC, 0xFF],
        width: 2.0,
        enabled: true,
        opacity: 0.5,
        blend: ph2d_vec_scene::BlendMode::Normal,
        offset: [0.0, 0.0],
        dilate: 0.0,
        dilate_join: 1,
        mixed: Default::default(),
    }
}

/// Uma forma selecionada com uma camada de PREENCHIMENTO (0) e uma de TRAÇO (1) por cima.
fn forma() {
    state::set_current_appearance(Some(Appearance {
        opacity: 1.0,
        blend: ph2d_vec_scene::BlendMode::Normal,
        layers: vec![camada(true), camada(false)],
        mixed_opacity: false,
        mixed_blend: false,
        layers_differ: false,
    }));
    state::set_stroke_present(Some(true));
    state::set_current_fill(Some(ph2d_panel_vector::FillKind::Linear), None);
}

fn limpa() {
    state::set_current_appearance(None);
    state::set_stroke_present(None);
    state::set_current_fill(None, None);
    state::close_open_layer();
}

fn cliques(h: &mut MockPanelHost) -> Vec<NodeId> {
    h.drained_actions()
        .into_iter()
        .filter_map(|a| match a {
            EditorAction::ToolPanelEvent(PanelEvent::Click(id)) => Some(id),
            _ => None,
        })
        .collect()
}

/// ⭐ **Cada camada mora na secção da ESPÉCIE dela**, e a Aparência ficou só com o objecto: a camada
/// de preenchimento é uma linha do Fill (acima da base), a de traço uma do Stroke. *Mutação: pintar
/// as camadas na Aparência de novo, ou trocar o filtro `is_fill` ⇒ vermelho.*
#[test]
fn cada_camada_mora_na_seccao_da_especie_dela() {
    forma();
    let mut h = crate::host_aberto();
    let fill_base = pintado(&mut h, tv::VECTOR_FILL_SWATCH).expect("a linha de base do Fill");
    let stroke_base = pintado(&mut h, tv::VECTOR_STROKE_SWATCH).expect("a linha de base do traço");
    let tinta = pintado(&mut h, ids::vector_paint_swatch_id(0)).expect("a camada de tinta");
    let traco = pintado(&mut h, ids::vector_paint_swatch_id(1)).expect("a camada de traço");
    assert!(
        tinta.y < fill_base.y && fill_base.y < traco.y && traco.y < stroke_base.y,
        "a ordem é Fill(camada, base) → Stroke(camada, base): {tinta:?} {fill_base:?} \
         {traco:?} {stroke_base:?}"
    );
    let obj = pintado(&mut h, ids::VECTOR_OBJ_OPACITY).expect("a opacidade do objecto");
    assert!(
        obj.y < tinta.y,
        "a Aparência (o objecto) vem antes das tintas"
    );
    limpa();
}

/// ⭐ **O «+» do título faz a pilha crescer**: o do Fill pede `+ Fill`; o do Stroke, `+ Stroke` —
/// e, numa forma SEM traço, dá o traço de base (`VECTOR_STROKE_PRESENT`). O «−» da linha de base do
/// traço tira-o. *Mutação: trocar o id do «+» do Stroke ⇒ vermelho.*
#[test]
fn o_mais_do_titulo_e_o_menos_da_base() {
    forma();
    for (id, what) in [
        (ids::VECTOR_PAINT_ADD_FILL, "o «+» do Fill"),
        (ids::VECTOR_PAINT_ADD_STROKE, "o «+» do Stroke"),
        (ids::VECTOR_STROKE_PRESENT, "o «−» do traço de base"),
    ] {
        let mut h = crate::host_aberto();
        crate::clica(&mut h, id);
        assert!(cliques(&mut h).contains(&id), "{what} não chegou ao bus");
    }
    state::set_stroke_present(Some(false));
    let mut h = crate::host_aberto();
    assert!(
        pintado(&mut h, ids::VECTOR_PAINT_ADD_STROKE).is_none(),
        "sem traço o «+» dá o traço de base, não uma camada"
    );
    crate::clica(&mut h, ids::VECTOR_STROKE_PRESENT);
    assert!(cliques(&mut h).contains(&ids::VECTOR_STROKE_PRESENT));
    limpa();
}

/// ⭐⭐ **A caixa da linha de base do Fill abre o painel à ESQUERDA com o tipo e o degradê**, e a
/// secção «Fill Type» deixou de existir. *Mutação: pintar o tipo no corpo, ou o painel à direita ⇒
/// vermelho.*
#[test]
fn a_linha_do_fill_abre_o_tipo_a_esquerda() {
    forma();
    let mut h = crate::host_aberto();
    assert!(
        pintado(&mut h, tv::VECTOR_FILL_KIND_LINEAR).is_none(),
        "o tipo à mostra com o painel fechado"
    );
    let linha = pintado(&mut h, ids::VECTOR_FILL_ROW).expect("a caixa do Fill de base");
    crate::clica(&mut h, ids::VECTOR_FILL_ROW);
    assert_eq!(h.dropdown_is_open(ids::VECTOR_FILL_ROW), Some(true));
    let chip = pintado(&mut h, tv::VECTOR_FILL_KIND_LINEAR).expect("o tipo no painel");
    let painel = h
        .store()
        .panel_rect(ph2d_editor_core::ids::VECTOR_PANEL)
        .expect("o rect do Inspector");
    assert!(
        chip.x + chip.w <= painel.x,
        "o painel do Fill abre à ESQUERDA do Inspector: {chip:?} × {painel:?}"
    );
    assert!(chip.y >= linha.y - 1.0, "o painel alinha-se à linha");
    crate::clica(&mut h, ids::VECTOR_FILL_PROPS_CLOSE);
    assert_eq!(h.dropdown_is_open(ids::VECTOR_FILL_ROW), Some(false));
    limpa();
}

/// ⭐⭐ **O ⚙ abre o «Stroke settings»: Cap e Join em GLIFOS** — e eles não estão no corpo. *Mutação:
/// voltar Cap/Join para o corpo ⇒ vermelho.*
#[test]
fn o_ajuste_abre_o_stroke_settings() {
    forma();
    let mut h = crate::host_aberto();
    for id in [tv::VECTOR_CAP_BUTT, tv::VECTOR_JOIN_MITER] {
        assert!(pintado(&mut h, id).is_none(), "Cap/Join à mostra no corpo");
    }
    crate::clica(&mut h, ids::VECTOR_STROKE_SETTINGS);
    for id in [
        tv::VECTOR_CAP_BUTT,
        tv::VECTOR_CAP_ROUND,
        tv::VECTOR_CAP_SQUARE,
        tv::VECTOR_JOIN_MITER,
        tv::VECTOR_JOIN_ROUND,
        tv::VECTOR_JOIN_BEVEL,
    ] {
        crate::clica(&mut h, id);
        assert!(
            cliques(&mut h).contains(&id),
            "{id:?} pintado no «Stroke settings» e morto"
        );
        assert_eq!(
            h.dropdown_is_open(ids::VECTOR_STROKE_SETTINGS),
            Some(true),
            "escolher no painel fechou-o"
        );
    }
    limpa();
}

/// ⭐ **Cap, Join e Position são GLIFOS, não rótulos** — a lei do dono, no fonte: os segmentos passam
/// pela porta de ícones e o corpo não escreve as palavras. *Mutação: voltar ao `segmented3` de texto
/// ⇒ vermelho.*
#[test]
fn cap_join_e_position_sao_glifos() {
    const SETTINGS: &str = include_str!("../../src/paint_stroke_settings.rs");
    const LINHAS: &str = include_str!("../../src/paint_fill_stroke.rs");
    for glifo in [
        "IconId::CapButt",
        "IconId::CapRound",
        "IconId::CapSquare",
        "IconId::JoinMiter",
        "IconId::JoinRound",
        "IconId::JoinBevel",
    ] {
        assert!(SETTINGS.contains(glifo), "falta o glifo {glifo}");
    }
    for glifo in [
        "IconId::StrokeInside",
        "IconId::StrokeCenter",
        "IconId::StrokeOutside",
    ] {
        assert!(LINHAS.contains(glifo), "falta o glifo {glifo}");
    }
    for palavra in [
        "panel.vector.stroke.butt",
        "panel.vector.stroke.miter",
        "panel.vector.stroke.centre",
    ] {
        assert!(
            !SETTINGS.contains(palavra) && !LINHAS.contains(palavra),
            "um segmento voltou a ser TEXTO: {palavra}"
        );
    }
}

/// ⭐ **A linha fixa: Position (três glifos) · Weight · ⚙**, e cada glifo chega ao bus. *Mutação:
/// trocar o id de um glifo ⇒ vermelho.*
#[test]
fn a_linha_fixa_position_weight_ajustes() {
    forma();
    for id in [
        tv::VECTOR_ALIGN_INNER,
        tv::VECTOR_ALIGN_CENTRE,
        tv::VECTOR_ALIGN_OUTER,
    ] {
        let mut h = crate::host_aberto();
        crate::clica(&mut h, id);
        assert!(cliques(&mut h).contains(&id), "{id:?} morto");
    }
    let mut h = crate::host_aberto();
    let pos = pintado(&mut h, tv::VECTOR_ALIGN_OUTER).expect("Position");
    let peso = pintado(&mut h, tv::VECTOR_WIDTH_NUM).expect("Weight");
    let ajuste = pintado(&mut h, ids::VECTOR_STROKE_SETTINGS).expect("⚙");
    assert!(
        (pos.y - peso.y).abs() < 0.5 && (peso.y - ajuste.y).abs() < 0.5,
        "Position, Weight e ⚙ na MESMA linha"
    );
    assert!(pos.x < peso.x && peso.x < ajuste.x);
    limpa();
}

/// ⭐⭐ **A opacidade na LINHA de uma camada chega ao bus como a da camada, e aponta-a.** O campo
/// mostra `%`; o barramento leva a fracção. *Mutação: esquecer o `set_open_layer` ⇒ a aberta fica
/// `None` e a shell não grava.*
#[test]
fn a_opacidade_da_linha_aponta_a_camada() {
    forma();
    let mut h = crate::host_aberto();
    let id = ids::vector_paint_row_opacity_id(1);
    assert!(pintado(&mut h, id).is_some(), "o % da camada de traço");
    h.set_number_value(id, 40.0);
    h.apply_panel_event::<VectorPanel>(&mut VectorPanelState, WidgetEvent::ValueChanged(id));
    let v: Vec<f64> = h
        .drained_actions()
        .into_iter()
        .filter_map(|a| match a {
            EditorAction::ToolPanelEvent(PanelEvent::SetValue(x, v))
                if x == ids::VECTOR_PAINT_OPACITY =>
            {
                Some(v)
            }
            _ => None,
        })
        .collect();
    assert_eq!(v.len(), 1);
    assert!((v[0] - 0.4).abs() < 1e-9, "40 % viaja como 0,4: {}", v[0]);
    assert_eq!(state::open_layer_index(), Some(1));
    limpa();
}

/// ⭐ **A caixa de uma camada abre o painel dela à esquerda** (largura, deslocamento, mistura) e a
/// camada aberta passa a ser ESSA — a que a shell grava. *Mutação: não derivar a aberta do
/// `Dropdown` ⇒ vermelho.*
#[test]
fn a_caixa_da_camada_abre_o_painel_dela() {
    forma();
    let mut h = crate::host_aberto();
    assert!(pintado(&mut h, ids::VECTOR_PAINT_WIDTH).is_none());
    crate::clica(&mut h, ids::vector_paint_row_id(1));
    assert!(
        pintado(&mut h, ids::VECTOR_PAINT_WIDTH).is_some(),
        "a largura no painel"
    );
    assert_eq!(state::open_layer_index(), Some(1));
    crate::clica(&mut h, ids::VECTOR_PAINT_PROPS_CLOSE);
    let _ = pintado(&mut h, ids::VECTOR_PAINT_ADD_FILL);
    assert_eq!(state::open_layer_index(), None, "o X fechou a camada");
    limpa();
}

/// ⭐ **O painel do traço da barra ② pinta o MESMO conteúdo**: as linhas e o «Stroke settings» (sem o
/// ⚙ — ele já está aberto por baixo). *Mutação: a barra sem o `stroke_settings_body` ⇒ vermelho.*
#[test]
fn a_barra_abre_o_mesmo_conteudo() {
    forma();
    let mut h = crate::host_com(&[ids::VECTOR_BAR_STROKE]);
    {
        use ph2d_editor_core::panel::PanelHostInternal;
        // O chip «Stroke ▾» da barra ② (a barra pinta-se no hero; aqui só a âncora).
        h.store_mut().set_tool_row_slots(vec![(
            ids::VECTOR_BAR_STROKE,
            Rect::new(400.0, 40.0, 80.0, 24.0),
        )]);
    }
    for id in [
        tv::VECTOR_CAP_ROUND,
        tv::VECTOR_JOIN_BEVEL,
        tv::VECTOR_ALIGN_INNER,
        ids::vector_paint_row_opacity_id(1),
    ] {
        assert!(
            pintado(&mut h, id).is_some(),
            "{id:?} falta no painel da barra"
        );
    }
    limpa();
}
