//! ⭐⭐ **O painel Vector em cartões, fatia G5** (`docs/CartoesUI/12_VECTOR_PLANO.md`): Pattern (as duas
//! tintas), Brush, Connector e Vertex. Cada id é provado pelo gesto REAL, nos rects pintados, e o que
//! chega ao barramento é o MESMO do painel de antes.

use std::collections::BTreeMap;

use super::g4_os_cartoes::Mesa;
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::cards::censo;
use ph2d_editor_core::panel::cards::dicas::field_hint;
use ph2d_editor_core::widget::key_mark::{KeyMark, key_mark_id, set_key_marks};
use ph2d_panel_vector::ids::TexPatKnob as K;
use ph2d_panel_vector::texture_pattern::kid;
use ph2d_panel_vector::{BrushRow, ConnectorSnapshot, TexturePatternRow, ids as p};
use ph2d_tool_vector::ids as t;
use ph2d_tool_vector::{VertexSel, VertexType};

fn padrao() -> TexturePatternRow {
    TexturePatternRow {
        kind: 1,
        offset_denom: 3.0,
        size: [2.0, 1.5],
        lock_aspect: true,
        gap: [0.25, -0.25],
        link_gap: false,
        angle_deg: 30.0,
        shift_pct: [10.0, 20.0],
        mode: 0,
        wrap_seam_visible: true,
        art: ph2d_vec_pattern::PatternArt::Ready,
    }
}

/// Uma forma com tudo à vista: dois nós, conector ortogonal, pincel com arte, os dois padrões.
fn forma() {
    ph2d_panel_vector::set_current_text_visible(false);
    ph2d_panel_vector::set_selected_vertex_type(Some(VertexSel::Uniform(VertexType::Smooth)));
    ph2d_panel_vector::set_current_vertex_pos(Some([1.5, -2.5]));
    ph2d_panel_vector::set_current_vertex_count(2);
    ph2d_panel_vector::set_current_connector(Some(ConnectorSnapshot {
        route: 1,
        jetty: 0.25,
        spread: 0.5,
        corner: 0.125,
        curve: 0.75,
    }));
    ph2d_panel_vector::set_current_brush(Some(BrushRow {
        has_art: true,
        spacing: 1.5,
        scale: 2.5,
        offset: -0.5,
        rotation_deg: 45.0,
        flip: false,
    }));
    ph2d_panel_vector::set_current_texture_pattern(0, Some(padrao()));
    ph2d_panel_vector::set_current_texture_pattern(1, Some(padrao()));
    set_key_marks(BTreeMap::new());
}

fn cliques() -> Vec<NodeId> {
    let mut v = vec![
        t::VECTOR_VERT_CORNER,
        t::VECTOR_VERT_SMOOTH,
        t::VECTOR_VERT_SYMMETRIC,
        p::VECTOR_VERT_SEL_SUBPATH,
        p::VECTOR_VERT_SEL_SAME,
        p::VECTOR_VERT_AVERAGE,
        p::VECTOR_VERT_DELETE,
        t::VECTOR_CONNECTOR_ROUTE,
        p::VECTOR_BRUSH_PICK_SHAPE,
        p::VECTOR_BRUSH_FLIP,
    ];
    for slot in 0..2 {
        v.extend(
            [
                K::Source,
                K::PickShape,
                K::Tile(0),
                K::Tile(1),
                K::Tile(2),
                K::Tile(3),
                K::Mode(0),
                K::Mode(1),
                K::Mode(2),
                K::Lock,
                K::GapLink,
            ]
            .map(|k| kid(slot, k)),
        );
    }
    v
}

/// `(chip, barra)` dos números ligados a uma barra.
fn numeros() -> Vec<(NodeId, NodeId)> {
    let mut v = vec![
        (p::VECTOR_BRUSH_SCALE_NUM, p::VECTOR_BRUSH_SCALE),
        (p::VECTOR_BRUSH_SPACING_NUM, p::VECTOR_BRUSH_SPACING),
        (p::VECTOR_BRUSH_ROTATION_NUM, p::VECTOR_BRUSH_ROTATION),
        (p::VECTOR_BRUSH_OFFSET_NUM, p::VECTOR_BRUSH_OFFSET),
    ];
    for slot in 0..2 {
        v.extend(
            [
                (K::OffsetNum, K::Offset),
                (K::WidthNum, K::Width),
                (K::HeightNum, K::Height),
                (K::GapNum, K::Gap),
                (K::GapYNum, K::GapY),
                (K::ShiftXNum, K::ShiftX),
                (K::ShiftYNum, K::ShiftY),
                (K::AngleNum, K::Angle),
            ]
            .map(|(n, b)| (kid(slot, n), kid(slot, b))),
        );
    }
    v
}

/// Os números que não têm barra: falam pelo próprio id.
fn soltos() -> [NodeId; 5] {
    [
        p::VECTOR_VERT_X,
        p::VECTOR_VERT_Y,
        t::VECTOR_CONNECTOR_JETTY,
        t::VECTOR_CONNECTOR_SPREAD,
        t::VECTOR_CONNECTOR_CORNER,
    ]
}

#[test]
fn every_id_of_the_slice_is_painted_in_cards_and_in_the_old_panel() {
    for cartoes in [true, false] {
        let mesa = Mesa::com(cartoes, forma);
        for id in cliques().into_iter().chain(soltos()) {
            assert!(mesa.rect(id).is_some(), "{id:?} (cartões {cartoes})");
        }
        for (chip, _) in numeros() {
            assert!(mesa.rect(chip).is_some(), "{chip:?} (cartões {cartoes})");
        }
    }
}

/// Nos cartões as barras não se pintam: o campo é a cara delas.
#[test]
fn no_old_bar_is_painted_in_the_slice() {
    let mesa = Mesa::com(true, forma);
    for (_, barra) in numeros() {
        assert!(mesa.rect(barra).is_none(), "barra {barra:?}");
    }
}

/// ⭐⭐ **Mesmo id, mesmo evento**. *Mutação: trocar dois ids num corpo ⇒ vermelho.*
#[test]
fn every_click_sends_the_same_action_as_the_old_panel() {
    for id in cliques() {
        let novo = Mesa::com(true, forma).clica(id);
        assert_eq!(novo, Mesa::com(false, forma).clica(id), "{id:?}");
    }
}

/// ⭐ **Scrub de cada número ligado**: a ferramenta ouve pela barra, a semente a meio não o puxa, e o Esc
/// devolve o valor do documento. *Mutação: tirar `mark_card_field` ou `semeia` ⇒ vermelho.*
#[test]
fn scrubbing_each_number_reaches_the_tool_and_escape_gives_it_back() {
    for (chip, barra) in numeros() {
        let mut mesa = Mesa::com(true, forma);
        let antes = mesa.host.store().number_value(chip).expect("semeado");
        // O Offset é inteiro, de 2 a 8 em 250 px: 40 px não chegavam a meia unidade.
        let ev = mesa.arrasta_a_pega(chip, 120.0);
        assert!(
            ev.contains(&WidgetEvent::ValueChanged(barra)),
            "{chip:?} não falou pela barra: {ev:?}"
        );
        let durante = mesa.host.store().number_value(chip).expect("campo");
        assert!((durante - antes).abs() > 1e-6, "{chip:?} parado em {antes}");
        mesa.esc();
        let voltou = mesa.host.store().number_value(chip).expect("campo");
        assert!(
            (voltou - antes).abs() < 1e-6,
            "{chip:?}: o Esc deu {voltou}"
        );
    }
    for id in soltos() {
        let mut mesa = Mesa::com(true, forma);
        let ev = mesa.arrasta_a_pega(id, 40.0);
        assert!(
            ev.contains(&WidgetEvent::ValueChanged(id)),
            "{id:?}: {ev:?}"
        );
    }
}

/// O campo mostra o valor do DOCUMENTO: o corpo antigo recebia-o ao pintar.
#[test]
fn each_field_shows_the_document_value() {
    let mesa = Mesa::com(true, forma);
    for (id, valor) in [
        (p::VECTOR_BRUSH_SCALE_NUM, 2.5),
        (p::VECTOR_BRUSH_SPACING_NUM, 1.5),
        (p::VECTOR_BRUSH_ROTATION_NUM, 45.0),
        (p::VECTOR_BRUSH_OFFSET_NUM, -0.5),
        (kid(1, K::OffsetNum), 3.0),
        (kid(1, K::WidthNum), 2.0),
        (kid(1, K::HeightNum), 1.5),
        (kid(1, K::GapNum), 0.25),
        (kid(1, K::GapYNum), -0.25),
        (kid(1, K::ShiftXNum), 10.0),
        (kid(1, K::ShiftYNum), 20.0),
        (kid(1, K::AngleNum), 30.0),
        (p::VECTOR_VERT_X, 1.5),
        (t::VECTOR_CONNECTOR_JETTY, 0.25),
    ] {
        let v = mesa.host.store().number_value(id).expect("campo");
        assert!((v - valor).abs() < 1e-6, "{id:?} mostra {v}");
    }
}

/// Escrever o número manda à ferramenta o MESMO que no painel de antes.
#[test]
fn typing_a_number_sends_the_same_as_the_old_panel() {
    for (id, texto) in [
        (p::VECTOR_BRUSH_SCALE_NUM, "3"),
        (p::VECTOR_BRUSH_OFFSET_NUM, "1"),
        (kid(0, K::WidthNum), "4"),
        (kid(1, K::AngleNum), "90"),
        (kid(1, K::OffsetNum), "4"),
        (p::VECTOR_VERT_X, "2"),
        (t::VECTOR_CONNECTOR_SPREAD, "0.2"),
    ] {
        let grava = |cartoes: bool| {
            let mut mesa = Mesa::com(cartoes, forma);
            let ev = mesa.host.type_into_number(id, texto);
            mesa.aplica(ev)
        };
        let novo = grava(true);
        assert!(!novo.is_empty(), "{id:?} mudo");
        assert_eq!(novo, grava(false), "{id:?}");
    }
}

/// ⭐ As perguntas de antes continuam a mandar. *Mutação: tirar cada `if` ⇒ vermelho.*
#[test]
fn what_cannot_be_used_is_not_offered() {
    let mut mesa = Mesa::com(true, forma);
    ph2d_panel_vector::set_current_vertex_count(1);
    ph2d_panel_vector::set_current_vertex_pos(None);
    mesa.assenta();
    assert!(
        mesa.rect(p::VECTOR_VERT_AVERAGE).is_none(),
        "Average de um nó"
    );
    assert!(mesa.rect(p::VECTOR_VERT_X).is_none(), "X sem mediana");
    assert!(mesa.rect(p::VECTOR_VERT_DELETE).is_some(), "Delete fica");
    ph2d_panel_vector::set_selected_vertex_type(None);
    mesa.assenta();
    assert!(
        mesa.rect(t::VECTOR_SECTION_VERTEX).is_none(),
        "Vertex sem nó"
    );

    let rota = |route: u8| ConnectorSnapshot {
        route,
        jetty: 0.25,
        spread: 0.5,
        corner: 0.125,
        curve: 0.75,
    };
    let mut mesa = Mesa::com(true, forma);
    ph2d_panel_vector::set_current_connector(Some(rota(0)));
    mesa.assenta();
    assert!(
        mesa.rect(t::VECTOR_CONNECTOR_JETTY).is_none(),
        "Jetty na reta"
    );
    assert!(
        mesa.rect(t::VECTOR_CONNECTOR_SPREAD).is_some(),
        "Spread fica"
    );
    ph2d_panel_vector::set_current_connector(Some(rota(2)));
    mesa.assenta();
    assert!(
        mesa.rect(t::VECTOR_CONNECTOR_CURVE).is_some(),
        "Curve na curva"
    );
    assert!(
        mesa.rect(t::VECTOR_CONNECTOR_CORNER).is_none(),
        "Corner na curva"
    );

    let mut mesa = Mesa::com(true, forma);
    let grade = TexturePatternRow {
        kind: 0,
        ..padrao()
    };
    ph2d_panel_vector::set_current_texture_pattern(1, Some(grade));
    mesa.assenta();
    assert!(mesa.rect(kid(1, K::OffsetNum)).is_none(), "Offset na grade");
    assert!(
        mesa.rect(kid(0, K::OffsetNum)).is_some(),
        "o outro sujeito fica"
    );
    let clamp = TexturePatternRow {
        mode: 2,
        ..padrao()
    };
    ph2d_panel_vector::set_current_texture_pattern(1, Some(clamp));
    mesa.assenta();
    assert!(mesa.rect(kid(1, K::ShiftXNum)).is_none(), "fase no Clamp");
    assert!(
        mesa.rect(kid(1, K::WidthNum)).is_some(),
        "o tamanho vive no Clamp"
    );
    assert!(
        mesa.rect(kid(1, K::GapNum)).is_some(),
        "o vão vive no Clamp"
    );
}

/// A marca de animação mora onde morava. *Mutação: tirar `.key` ou o canal da escolha ⇒ vermelho.*
#[test]
fn the_animation_marks_stay() {
    let marcados = [
        p::VECTOR_VERT_X,
        p::VECTOR_BRUSH_SCALE,
        p::VECTOR_BRUSH_OFFSET,
        kid(0, K::Width),
        kid(1, K::Angle),
        kid(1, K::Tile(0)),
        kid(1, K::Mode(0)),
    ];
    let mut mesa = Mesa::com(true, forma);
    set_key_marks(marcados.iter().map(|id| (*id, KeyMark::Chave)).collect());
    mesa.assenta();
    for id in marcados {
        assert!(mesa.rect(key_mark_id(id)).is_some(), "sem marca: {id:?}");
    }
    set_key_marks(BTreeMap::new());
}

/// O passo de cada campo. *Mutação: tirar uma linha do `prepare` ⇒ vermelho.*
#[test]
fn each_field_has_its_step() {
    let mesa = Mesa::com(true, forma);
    let spec = |id: NodeId| {
        mesa.host
            .store()
            .card_field(id)
            .unwrap_or_else(|| panic!("{id:?} sem marca"))
    };
    assert_eq!(spec(p::VECTOR_BRUSH_ROTATION_NUM).default, Some(0.0));
    assert_eq!(spec(p::VECTOR_BRUSH_OFFSET_NUM).default, Some(0.0));
    assert_eq!(spec(kid(0, K::GapNum)).default, Some(0.0));
    assert_eq!(
        spec(kid(1, K::OffsetNum)).step,
        1.0,
        "o denominador é inteiro"
    );
    assert!(spec(kid(1, K::WidthNum)).step < 1.0, "o tamanho é fino");
    assert!(spec(p::VECTOR_VERT_X).step < 1.0, "o nó é em mundo");
    assert!(spec(t::VECTOR_CONNECTOR_JETTY).step > 0.0, "o conector");
}

/// ⭐⭐ **Compacto como o Figma** (05 §4): irmãos curtos dividem a linha, e nada fica grudado.
/// *Mutação: separar um par ou tirar o vão ⇒ vermelho.*
#[test]
fn siblings_share_the_line_and_nothing_is_glued() {
    let mesa = Mesa::com(true, forma);
    let r = |id: NodeId| mesa.rect(id).unwrap_or_else(|| panic!("{id:?}"));
    let mut pares = vec![
        (p::VECTOR_VERT_X, p::VECTOR_VERT_Y),
        (p::VECTOR_VERT_SEL_SUBPATH, p::VECTOR_VERT_SEL_SAME),
        (p::VECTOR_VERT_AVERAGE, p::VECTOR_VERT_DELETE),
        (t::VECTOR_CONNECTOR_ROUTE, t::VECTOR_CONNECTOR_JETTY),
        (t::VECTOR_CONNECTOR_SPREAD, t::VECTOR_CONNECTOR_CORNER),
        (p::VECTOR_BRUSH_SCALE_NUM, p::VECTOR_BRUSH_SPACING_NUM),
        (p::VECTOR_BRUSH_ROTATION_NUM, p::VECTOR_BRUSH_OFFSET_NUM),
    ];
    for slot in 0..2 {
        for (a, b) in [
            (K::Source, K::PickShape),
            (K::OffsetNum, K::AngleNum),
            (K::WidthNum, K::HeightNum),
            (K::GapNum, K::GapYNum),
            (K::Lock, K::GapLink),
            (K::ShiftXNum, K::ShiftYNum),
        ] {
            pares.push((kid(slot, a), kid(slot, b)));
        }
    }
    for (a, b) in pares {
        let (ra, rb) = (r(a), r(b));
        assert!(
            (ra.y - rb.y).abs() < 1.0,
            "{a:?} e {b:?} em linhas diferentes"
        );
        assert!(ra.x + ra.w <= rb.x, "{a:?} por cima de {b:?}");
    }
    // A grelha: a 2.ª coluna começa no mesmo x em todo o par de números do cartão.
    let col = r(kid(0, K::HeightNum)).x;
    for k in [K::AngleNum, K::GapYNum, K::ShiftYNum, K::PickShape] {
        assert!((r(kid(0, k)).x - col).abs() < 1.0, "{k:?} fora da grelha");
    }
    // Nada grudado: entre duas linhas de botões, o vão de `metrics`.
    let (a, b) = (r(p::VECTOR_VERT_SEL_SUBPATH), r(p::VECTOR_VERT_AVERAGE));
    assert!(
        b.y - (a.y + a.h) >= m::FIELD_ROW_GAP - 0.5,
        "botões grudados: {a:?} {b:?}"
    );
    let (a, b) = (r(kid(0, K::Lock)), r(kid(0, K::ShiftXNum)));
    assert!(
        a.y + a.h <= b.y,
        "a caixa entra no campo de baixo: {a:?} {b:?}"
    );
}

/// O que sobra de um par fica com a linha inteira: sem Offset (na grade), o Angle ocupa-a.
#[test]
fn the_angle_alone_takes_the_line_when_there_is_no_offset() {
    let mut mesa = Mesa::com(true, forma);
    let grade = TexturePatternRow {
        kind: 0,
        ..padrao()
    };
    ph2d_panel_vector::set_current_texture_pattern(0, Some(grade));
    mesa.assenta();
    let angulo = mesa.rect(kid(0, K::AngleNum)).expect("Angle");
    let largura = mesa.rect(kid(0, K::WidthNum)).expect("Width");
    let altura = mesa.rect(kid(0, K::HeightNum)).expect("Height");
    assert!(
        (angulo.x + angulo.w - (altura.x + altura.w)).abs() < 1.0
            && (angulo.x - largura.x).abs() < 1.0,
        "Angle sem par: {angulo:?}"
    );
}

/// Rótulo com a unidade, nunca na caixa. *Mutação: tirar `com_unidade` ⇒ vermelho.*
#[test]
fn each_number_says_its_unit_in_the_label() {
    let mesa = Mesa::com(true, forma);
    let tem = |id: NodeId, nome: &str| {
        let dica = field_hint(mesa.host.store(), id).unwrap_or_else(|| panic!("{id:?} sem dica"));
        assert!(dica.starts_with(nome), "{id:?}: «{dica}» sem «{nome}»");
    };
    tem(kid(0, K::WidthNum), "Width (m)");
    tem(kid(0, K::GapYNum), "Gap Y (m)");
    tem(kid(0, K::ShiftXNum), "Shift X (%)");
    tem(kid(0, K::AngleNum), "Angle (°)");
    tem(kid(0, K::OffsetNum), "Offset (1/n)");
    tem(p::VECTOR_BRUSH_SCALE_NUM, "Size (×)");
    tem(p::VECTOR_BRUSH_SPACING_NUM, "Spacing (×)");
    tem(p::VECTOR_BRUSH_ROTATION_NUM, "Rotation (°)");
    tem(p::VECTOR_BRUSH_OFFSET_NUM, "Offset (m)");
}

/// A frase só fala onde tem sujeito: arte em falta, e a costura só no `Tile`. Prova-se pela altura do
/// cartão, medida no campo de baixo. *Mutação: tirar um `c.hint` ou o `p.mode == 0` ⇒ vermelho.*
#[test]
fn the_hints_speak_only_where_they_have_a_subject() {
    let medida = |linha: TexturePatternRow| {
        let mut mesa = Mesa::com(true, forma);
        ph2d_panel_vector::set_current_texture_pattern(0, Some(linha));
        mesa.assenta();
        let topo = mesa.rect(kid(0, K::Source)).expect("Source").y;
        let fundo = mesa.rect(kid(0, K::Mode(0))).expect("Mode").y;
        // O vão até ao cartão de baixo diz se a costura falou; o topo diz se a arte falou.
        (
            topo,
            mesa.rect(kid(1, K::Source)).expect("o outro").y - fundo,
        )
    };
    let sem_costura = TexturePatternRow {
        wrap_seam_visible: false,
        ..padrao()
    };
    let (topo, base) = medida(sem_costura);
    assert!(medida(padrao()).1 > base, "a costura não falou no Tile");
    let espelho = TexturePatternRow {
        mode: 1,
        ..padrao()
    };
    assert!(
        (medida(espelho).1 - base).abs() < 0.5,
        "a costura no Mirror"
    );
    for art in [
        ph2d_vec_pattern::PatternArt::NotChosen,
        ph2d_vec_pattern::PatternArt::Deleted,
    ] {
        let linha = TexturePatternRow {
            art,
            wrap_seam_visible: false,
            ..padrao()
        };
        assert!(
            medida(linha).0 > topo,
            "{art:?} sem a frase, por cima dos botões"
        );
    }
}

/// A pintura que `f` faz passa nas três regras do censo da fundação (05 §4), numa coluna de dock.
pub(crate) fn sem_acusacao(onde: &str, f: impl FnOnce()) {
    let ((), censo) = censo::grava(f);
    assert!(!censo.linhas.is_empty(), "{onde}: o censo não mediu nada");
    let mais_larga = censo.linhas.iter().map(|l| l.w).fold(0.0, f32::max);
    assert!(
        mais_larga < 400.0,
        "{onde}: a coluna não é a de um dock: {mais_larga}"
    );
    assert_eq!(censo.irmaos_separados(), Vec::<String>::new(), "{onde}");
    assert_eq!(censo.linhas_grudadas(), Vec::<String>::new(), "{onde}");
    assert_eq!(censo.textos_cortados(), Vec::<String>::new(), "{onde}");
}

/// ⭐⭐ **A lei da compactação sobre o estado que a varredura do registo não semeia**: os cartões só
/// existem com pincel, conector, nó e padrão.
#[test]
fn the_slice_passes_the_compaction_census() {
    for route in [0, 1, 2] {
        let mut mesa = Mesa::com(true, forma);
        ph2d_panel_vector::set_current_connector(Some(ConnectorSnapshot {
            route,
            jetty: 0.25,
            spread: 0.5,
            corner: 0.125,
            curve: 0.75,
        }));
        sem_acusacao(&format!("rota {route}"), || mesa.assenta());
    }
}
