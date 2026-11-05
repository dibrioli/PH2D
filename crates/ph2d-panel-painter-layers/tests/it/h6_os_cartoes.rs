//! **Os cartões Paper, Grain e Randomize Color em linhas de cartão, fatia H6** (`docs/CartoesUI/13_PAINTER_PLANO.md`). Pelo gesto
//! real, nos rects pintados, e o que chega ao barramento é o MESMO do painel de antes. O universo sai do próprio painel de antes:
//! o que aparece entre os cabeçalhos destes cartões. Nenhum motor, preset ou pincel muda.

use super::h1_os_cartoes::{Mesa, esc, pincel, valor_enviado};
use super::h5_os_cartoes::{na_mesma_linha, sem_sobrepor};
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::panel::cards::{censo, grip_of};
use ph2d_editor_core::widget::key_mark::{KeyMark, add_key_mark, key_mark_target, set_key_marks};
use ph2d_host::PointerKind;
use ph2d_panel_painter_layers::FALLBACK_BRUSH;
use ph2d_tool_painter::ids as p;
use ph2d_tool_painter::{BrushSettings, Inercias, PaintMedia, TextureKind, TextureMapping};
use std::collections::BTreeSet;

const PAPEL: NodeId = p::PAINTER_WATERCOLOR_PAPER_SECTION;
const GRAO: NodeId = p::PAINTER_BRUSH_TEXTURE_SECTION;
const ALEATORIO: NodeId = p::PAINTER_BRUSH_RANDOMIZE_SECTION;
const COMPORTAMENTO: NodeId = p::PAINTER_BRUSH_GRAIN_BEHAVIOR_SECTION;
const RAMPA: NodeId = p::PAINTER_BRUSH_COLOR_RAMP_SECTION;
const ABERTOS: [NodeId; 3] = [PAPEL, GRAO, ALEATORIO];
/// Um padrão com parâmetros: o penúltimo da tabela, o mesmo da H5.
const PADRAO: u8 = TextureKind::COUNT - 2;

fn digital() -> BrushSettings {
    pincel(PaintMedia::Digital, "brush").0
}

fn com_grao() -> BrushSettings {
    let mut b = digital();
    b.texture_kind = PADRAO;
    b.inercias = Inercias::NENHUMA;
    b
}

fn com_papel(mut b: BrushSettings) -> BrushSettings {
    b.paper_kind = PADRAO;
    b
}

fn com_rampa() -> BrushSettings {
    let mut b = com_grao();
    b.texture_ramp_enabled = true;
    b
}

fn carimbo() -> BrushSettings {
    let mut b = com_grao();
    b.texture_mapping = TextureMapping::Stencil.to_u8();
    b
}

fn mesmo_papel() -> BrushSettings {
    let mut b = digital();
    b.grain_same_as_paper = true;
    b
}

fn aguada() -> BrushSettings {
    pincel(PaintMedia::Watercolor, "brush").0
}

fn aguada_do_papel() -> BrushSettings {
    let mut b = aguada();
    b.granulation_use_paper = true;
    b.watercolor_active = true;
    b
}

fn molhado() -> BrushSettings {
    pincel(PaintMedia::WetPaint, "brush").0
}

fn estados() -> Vec<(&'static str, BrushSettings)> {
    let mut aguada_com_grao = aguada();
    aguada_com_grao.texture_kind = PADRAO;
    aguada_com_grao.granulation_use_paper = false;
    let mut impasto_com_papel = com_papel(pincel(PaintMedia::Impasto, "brush").0);
    impasto_com_papel.texture_kind = PADRAO;
    let mut grao_sem_rake = com_grao();
    grao_sem_rake.watercolor_active = true;
    let mut imagem = com_grao();
    imagem.texture_kind = TextureKind::Image.to_u8();
    let mut rampa_na_borracha = pincel(PaintMedia::Digital, "eraser").0;
    rampa_na_borracha.texture_kind = PADRAO;
    rampa_na_borracha.texture_ramp_enabled = true;
    vec![
        ("digital", digital()),
        ("grão", com_grao()),
        ("papel", com_papel(digital())),
        ("rampa", com_rampa()),
        ("carimbo", carimbo()),
        ("o mesmo papel", mesmo_papel()),
        ("aguada", aguada()),
        ("aguada do papel", aguada_do_papel()),
        ("aguada com grão", aguada_com_grao),
        ("aguada com papel", com_papel(aguada())),
        ("molhado", molhado()),
        ("molhado com papel", com_papel(molhado())),
        ("impasto", pincel(PaintMedia::Impasto, "brush").0),
        ("impasto com papel", impasto_com_papel),
        ("grão sem rake", grao_sem_rake),
        ("imagem", imagem),
        ("rampa na borracha", rampa_na_borracha),
        ("borracha", pincel(PaintMedia::Digital, "eraser").0),
        ("smear", pincel(PaintMedia::Digital, "smear").0),
        ("blur", pincel(PaintMedia::Digital, "blur").0),
        ("inpaint", pincel(PaintMedia::Digital, "inpaint").0),
    ]
}

/// Nos cartões, só estes abertos; os dois títulos de dentro do Grain abrem nas duas mesas.
fn so(cartoes: bool, b: BrushSettings, abertos: &[NodeId]) -> Mesa {
    let mut m = Mesa::nova(cartoes, b, false);
    if cartoes {
        m.so_abertos(abertos);
    }
    for id in [COMPORTAMENTO, RAMPA] {
        if m.host.store().is_collapsed(id) {
            m.host.store_mut().toggle_collapsed(id);
        }
    }
    m.assenta();
    m
}

fn abre(cartoes: bool, b: BrushSettings) -> Mesa {
    so(cartoes, b, &ABERTOS)
}

/// Os ids que estes cartões trazem quando abrem, neste estado.
fn do_corpo(b: BrushSettings) -> BTreeSet<NodeId> {
    let abertos = abre(true, b).ids();
    let dobrados = so(true, b, &[]).ids();
    abertos.difference(&dobrados).copied().collect()
}

/// Os ids do painel de antes entre dois cabeçalhos, fora o cromo das secções.
fn entre(m: &Mesa, de: NodeId, ate: NodeId) -> BTreeSet<NodeId> {
    let (Some(de), Some(ate)) = (m.rect(grip_of(de)), m.rect(grip_of(ate))) else {
        return BTreeSet::new();
    };
    let barras = p::PAINTER_BRUSH_RANDOMIZE_SLIDERS;
    let cromo = [
        PAPEL,
        GRAO,
        ALEATORIO,
        grip_of(GRAO),
        p::PAINTER_WATERCOLOR_PAPER_RESET,
        p::PAINTER_BRUSH_TEXTURE_RESET,
        p::PAINTER_BRUSH_RANDOMIZE_RESET,
    ];
    m.ids()
        .into_iter()
        .filter(|id| !cromo.contains(id) && !barras.contains(id))
        .filter(|id| {
            m.rect(*id)
                .is_some_and(|r| r.y >= de.y + de.h && r.y < ate.y)
        })
        .collect()
}

fn do_painel_de_antes(b: BrushSettings) -> BTreeSet<NodeId> {
    let m = abre(false, b);
    let mut ids = entre(&m, ALEATORIO, p::PAINTER_SHAPE_SECTION);
    ids.extend(entre(&m, PAPEL, p::PAINTER_BRUSH_STROKE_SECTION));
    ids
}

/// ⭐ As mesmas condições, nas DUAS direcções, em cada estado. O universo sai do painel de antes.
#[test]
fn these_cards_show_the_same_controls_as_the_old_panel() {
    let mut vistos = BTreeSet::new();
    for (nome, b) in estados() {
        let (antes, agora) = (do_painel_de_antes(b), do_corpo(b));
        let so_antes: Vec<_> = antes.difference(&agora).collect();
        let so_agora: Vec<_> = agora.difference(&antes).collect();
        assert!(
            so_antes.is_empty() && so_agora.is_empty(),
            "{nome}: só antes {so_antes:?}, só agora {so_agora:?}"
        );
        vistos.extend(agora);
    }
    assert!(vistos.len() > 50, "a régua viu pouco: {}", vistos.len());
    for id in [
        p::PAINTER_WATERCOLOR_PAPER_COLOR_THUMB,
        p::PAINTER_WATERCOLOR_PAPER_KIND,
        p::PAINTER_SUBSTRATE_RELIEF,
        p::PAINTER_WATERCOLOR_PAPER_DEPTH,
        p::PAINTER_WETPAINT_SMOOTH_EDGES,
        p::PAINTER_WETPAINT_ABSORBENCY,
        p::PAINTER_WATERCOLOR_GRAN_SAME,
        p::PAINTER_WATERCOLOR_GRANULATION,
        p::PAINTER_BRUSH_GRAIN_SAME_AS_PAPER,
        p::PAINTER_BRUSH_TEXTURE_KIND,
        p::PAINTER_BRUSH_TEXTURE_MAPPING,
        p::PAINTER_BRUSH_TEXTURE_RAKE,
        p::PAINTER_BRUSH_STENCIL_ANGLE,
        p::PAINTER_BRUSH_GRAIN_DEPTH,
        p::PAINTER_BRUSH_TEXTURE_RAMP_ENABLE,
        p::PAINTER_BRUSH_TEXTURE_RAMP_SWATCH,
        p::PAINTER_MARCA_SECONDARY,
        p::PAINTER_BRUSH_COLOR_JITTER_PER_DAB,
    ] {
        assert!(vistos.contains(&id), "{id:?} nunca à vista");
    }
}

fn numeros(b: BrushSettings) -> Vec<NodeId> {
    let m = abre(true, b);
    do_corpo(b)
        .into_iter()
        .filter(|id| m.host.store().card_field(*id).is_some())
        .collect()
}

#[test]
fn typing_each_number_sends_the_same_action_as_the_old_panel() {
    let mut vistos = BTreeSet::new();
    for (nome, b) in estados() {
        let ids = numeros(b);
        vistos.extend(ids.iter().copied());
        let (mut velha, mut nova) = (abre(false, b), abre(true, b));
        for id in ids {
            for texto in ["0.04", "500", "-500"] {
                let ev = velha.host.type_into_number(id, texto);
                let a = velha.aplica(ev);
                let ev = nova.host.type_into_number(id, texto);
                let n = nova.aplica(ev);
                assert_eq!(a, n, "{nome}: {id:?} com {texto}");
            }
        }
    }
    assert!(vistos.len() >= 25, "viu {} números", vistos.len());
}

#[test]
fn every_number_shows_the_same_value_and_range_as_the_old_panel() {
    for (nome, mut b) in estados() {
        b.substrate_relief = 0.31;
        b.substrate_roughness = 0.77;
        b.paper_offset = [0.11, -0.13];
        b.paper_size = [1.2, 0.8];
        b.texture_offset = [0.21, -0.23];
        b.texture_size = [1.4, 0.6];
        b.color_jitter = [0.1, 0.2, 0.3];
        b.grain_depth = 0.42;
        let (velha, nova) = (abre(false, b), abre(true, b));
        for id in numeros(b) {
            let (s_v, s_n) = (velha.host.store(), nova.host.store());
            // O chip de uma barra só se escreve no gesto no painel de antes: a régua dele é a barra.
            if let Some(i) = p::PAINTER_BRUSH_RANDOMIZE_CHIPS
                .iter()
                .position(|c| *c == id)
            {
                let v = s_n.number_value(id).expect("semeado");
                assert!(
                    (v - f64::from(b.color_jitter[i])).abs() < 1e-3,
                    "{nome}: {id:?} {v}"
                );
            } else {
                assert_eq!(s_v.number_value(id), s_n.number_value(id), "{nome}: {id:?}");
            }
            // O chip do stop não tinha faixa; o campo do cartão arrasta, e tem: 0..último stop, 0..1.
            if ![
                p::PAINTER_BRUSH_TEXTURE_RAMP_STOP_INDEX,
                p::PAINTER_BRUSH_TEXTURE_RAMP_STOP_POS,
            ]
            .contains(&id)
            {
                assert_eq!(
                    s_v.number_range(id),
                    s_n.number_range(id),
                    "{nome}: faixa de {id:?}"
                );
            }
        }
    }
}

#[test]
fn scrubbing_each_number_moves_only_it_and_escape_gives_it_back() {
    for b in [com_papel(com_grao()), com_papel(molhado())] {
        // Os campos do stop têm prova própria (`h10`): a ferramenta recebe-os pelo editor da rampa.
        let mut ids = numeros(b);
        ids.retain(|i| {
            ![
                p::PAINTER_BRUSH_TEXTURE_RAMP_STOP_INDEX,
                p::PAINTER_BRUSH_TEXTURE_RAMP_STOP_POS,
            ]
            .contains(i)
        });
        assert!(ids.len() >= 15, "só {}", ids.len());
        for id in &ids {
            let le = |m: &Mesa, i: NodeId| m.host.store().number_value(i).expect("semeado");
            let mut mesa = abre(true, b);
            let antes = le(&mesa, *id);
            let outros: Vec<(NodeId, f64)> = ids
                .iter()
                .filter(|o| *o != id)
                .map(|o| (*o, le(&mesa, *o)))
                .collect();
            let mut ev = mesa.arrasta_a_pega(*id, 60.0);
            let mut no_gesto = le(&mesa, *id);
            if (no_gesto - antes).abs() < 1e-9 {
                mesa = abre(true, b);
                ev = mesa.arrasta_a_pega(*id, -60.0);
                no_gesto = le(&mesa, *id);
            }
            assert!((no_gesto - antes).abs() > 1e-9, "{id:?} não se moveu");
            let accoes = mesa.aplica(ev);
            let enviado = valor_enviado(&accoes, *id);
            assert!(
                enviado.is_some() || !accoes.is_empty(),
                "{id:?}: o painel não avisou a ferramenta"
            );
            for (o, v) in &outros {
                assert!(
                    (le(&mesa, *o) - v).abs() < 1e-9,
                    "{o:?} mexeu-se com {id:?}"
                );
            }
            mesa.quadro();
            assert!((le(&mesa, *id) - no_gesto).abs() < 1e-6, "a semente puxou");
            let _ = esc(&mut mesa);
            assert!((le(&mesa, *id) - antes).abs() < 1e-6, "{id:?}: o Esc");
        }
    }
}

#[test]
fn every_click_sends_the_same_action_as_the_old_panel() {
    let mut vistos = 0;
    let medem = [
        "digital",
        "grão",
        "rampa",
        "carimbo",
        "o mesmo papel",
        "aguada",
        "molhado com papel",
    ];
    for (nome, b) in estados().into_iter().filter(|(n, _)| medem.contains(n)) {
        let mesa = abre(true, b);
        let cliques: Vec<NodeId> = do_corpo(b)
            .into_iter()
            .filter(|id| {
                let s = mesa.host.store();
                s.card_field(*id).is_none()
                    && !matches!(
                        s.get(*id),
                        Some(
                            InteractiveState::CurvePoint { .. }
                                | InteractiveState::NumberInput { .. }
                        )
                    )
            })
            .collect();
        for id in cliques {
            let velho = abre(false, b).clica(id);
            let novo = abre(true, b).clica(id);
            assert_eq!(velho, novo, "{nome}: {id:?}");
            vistos += 1;
        }
    }
    assert!(vistos >= 25, "viu {vistos} cliques");
}

#[test]
fn each_list_opens_under_its_box_and_its_option_sends_the_same() {
    let modo = p::PAINTER_PONTA_FIELDS[1];
    let controlo = p::PAINTER_PONTA_FIELDS[4];
    for (b, caixa, opcao) in [
        (
            digital(),
            p::PAINTER_WATERCOLOR_PAPER_KIND,
            p::painter_paper_kind_option_id(1),
        ),
        (
            digital(),
            p::PAINTER_BRUSH_TEXTURE_KIND,
            p::painter_brush_texture_kind_option_id(1),
        ),
        (
            com_grao(),
            p::PAINTER_BRUSH_TEXTURE_MAPPING,
            p::painter_brush_texture_mapping_option_id(1),
        ),
        (com_grao(), modo, p::painter_ponta_option_id(1, 1)),
        (com_grao(), controlo, p::painter_ponta_option_id(4, 1)),
        (
            com_rampa(),
            p::PAINTER_BRUSH_TEXTURE_RAMP_MODE,
            p::painter_brush_texture_ramp_mode_option_id(1),
        ),
    ] {
        let escolhe = |cartoes: bool| {
            let mut mesa = abre(cartoes, b);
            let chip = mesa.rect(caixa).expect("a caixa");
            let _ = mesa.clica(caixa);
            mesa.assenta();
            let r = mesa.rect(opcao).expect("a opção aberta");
            assert!(r.y >= chip.y, "{caixa:?} abriu acima: {chip:?} → {r:?}");
            mesa.clica(opcao)
        };
        let (velho, novo) = (escolhe(false), escolhe(true));
        assert!(!novo.is_empty(), "{caixa:?}: nada no barramento");
        assert_eq!(velho, novo, "{caixa:?}");
    }
}

#[test]
fn the_reset_of_each_section_lives_in_the_card_header() {
    for id in [
        p::PAINTER_WATERCOLOR_PAPER_RESET,
        p::PAINTER_BRUSH_TEXTURE_RESET,
        p::PAINTER_BRUSH_RANDOMIZE_RESET,
        p::PAINTER_BRUSH_COLOR_RAMP_RESET,
    ] {
        let b = com_rampa();
        let (velho, novo) = (abre(false, b).clica(id), abre(true, b).clica(id));
        assert!(!novo.is_empty(), "{id:?}: nada no barramento");
        assert_eq!(velho, novo, "{id:?}");
    }
}

/// Os títulos de dentro do Grain dobram só o que é deles.
#[test]
fn the_titles_inside_the_grain_card_fold_their_rows() {
    let b = com_rampa();
    let linha_do_grao = p::PAINTER_PONTA_FIELDS[1];
    let mut m = abre(true, b);
    assert!(m.rect(linha_do_grao).is_some());
    let _ = m.clica(COMPORTAMENTO);
    m.assenta();
    assert!(m.rect(linha_do_grao).is_none(), "Grain Behavior não dobrou");
    assert!(m.rect(p::PAINTER_BRUSH_TEXTURE_RAMP_ENABLE).is_some());
    let _ = m.clica(RAMPA);
    m.assenta();
    assert!(m.rect(p::PAINTER_BRUSH_TEXTURE_RAMP_ENABLE).is_none());
    assert!(m.rect(p::PAINTER_BRUSH_TEXTURE_RAMP_SWATCH).is_none());
}

/// O seletor partilhado escreve na amostra, e a escolha chega à ferramenta como no painel de antes.
#[test]
fn each_swatch_forwards_the_picked_colour_like_the_old_panel() {
    for id in [
        p::PAINTER_WATERCOLOR_PAPER_COLOR_THUMB,
        p::PAINTER_MARCA_SECONDARY,
    ] {
        let colhe = |cartoes: bool| {
            let mut m = abre(cartoes, digital());
            let _ = m.host.drained_actions();
            let store = m.host.store_mut();
            store.set_picker_target(Some(id));
            store.set_widget_color(id, [11, 22, 33, u8::MAX]);
            m.quadro();
            m.host
                .drained_actions()
                .iter()
                .map(|a| format!("{a:?}"))
                .filter(|a| a.contains("11,22,33"))
                .count()
        };
        assert!(colhe(true) >= 1, "{id:?}: a escolha não chegou");
        assert_eq!(colhe(false).min(1), colhe(true).min(1), "{id:?}");
    }
}

#[test]
fn each_control_keeps_its_animation_mark() {
    for (nome, b) in estados() {
        let ids = do_painel_de_antes(b);
        let conta = |cartoes: bool| {
            set_key_marks(std::collections::BTreeMap::new());
            for id in &ids {
                add_key_mark(*id, KeyMark::Chave);
            }
            let m = abre(cartoes, b);
            let alvos: BTreeSet<NodeId> = m
                .ids()
                .into_iter()
                .filter(|i| key_mark_target(*i).is_some() && ids.contains(i))
                .collect();
            set_key_marks(std::collections::BTreeMap::new());
            alvos.len()
        };
        assert!(conta(true) >= conta(false), "{nome}");
    }
}

#[test]
fn each_number_knows_its_factory_value() {
    let m = abre(true, com_papel(com_grao()));
    for (id, fabrica) in [
        (p::PAINTER_SUBSTRATE_RELIEF, FALLBACK_BRUSH.substrate_relief),
        (p::PAINTER_BRUSH_GRAIN_DEPTH, FALLBACK_BRUSH.grain_depth),
        (
            p::PAINTER_BRUSH_RANDOMIZE_CHIPS[0],
            FALLBACK_BRUSH.color_jitter[0],
        ),
    ] {
        let f = m.host.store().card_field(id).expect("marcado").default;
        assert!(
            (f.expect("fábrica") - f64::from(fabrica)).abs() < 1e-6,
            "{id:?}"
        );
    }
}

#[test]
fn no_control_of_these_cards_overlaps_another() {
    for (nome, b) in estados() {
        // O título de dentro é a linha inteira, e o Reset dele pinta-se por cima, como em todo cabeçalho.
        let mut ids = do_corpo(b);
        ids.remove(&RAMPA);
        sem_sobrepor(&abre(true, b), &ids, nome);
    }
}

#[test]
fn short_siblings_share_a_row() {
    let b = com_papel(com_grao());
    let m = abre(true, b);
    for (x, y) in [
        (
            p::PAINTER_WATERCOLOR_PAPER_COLOR_THUMB,
            p::PAINTER_WATERCOLOR_PAPER_KIND,
        ),
        (p::PAINTER_SUBSTRATE_RELIEF, p::PAINTER_SUBSTRATE_ROUGHNESS),
        (p::PAINTER_BRUSH_TEXTURE_ANGLE, p::PAINTER_BRUSH_GRAIN_DEPTH),
        (
            p::PAINTER_BRUSH_GRAIN_SAME_AS_PAPER,
            p::PAINTER_BRUSH_TEXTURE_KIND,
        ),
        (
            p::PAINTER_BRUSH_TEXTURE_MAPPING,
            p::PAINTER_BRUSH_TEXTURE_RAKE,
        ),
        (
            p::PAINTER_BRUSH_RANDOMIZE_CHIPS[0],
            p::PAINTER_BRUSH_RANDOMIZE_CHIPS[1],
        ),
        (
            p::PAINTER_BRUSH_RANDOMIZE_CHIPS[1],
            p::PAINTER_BRUSH_RANDOMIZE_CHIPS[2],
        ),
        (p::PAINTER_MARCA_FIELDS[8], p::PAINTER_MARCA_SECONDARY),
    ] {
        na_mesma_linha(&m, x, y);
    }
    let aguada = abre(true, aguada());
    na_mesma_linha(
        &aguada,
        p::PAINTER_WATERCOLOR_GRAN_SAME,
        p::PAINTER_WATERCOLOR_GRANULATION,
    );
    let mesmo = abre(true, mesmo_papel());
    na_mesma_linha(
        &mesmo,
        p::PAINTER_BRUSH_GRAIN_SAME_AS_PAPER,
        p::PAINTER_BRUSH_GRAIN_DEPTH,
    );
}

#[test]
fn no_text_is_cut_and_no_row_sticks_in_any_state() {
    for (nome, b) in estados() {
        let (_, c) = censo::grava(|| abre(true, b));
        let cortados = c.textos_cortados();
        assert!(cortados.is_empty(), "{nome}: {cortados:#?}");
        let estes = censo::Censo {
            linhas: c
                .linhas
                .iter()
                .filter(|l| ABERTOS.contains(&l.secao))
                .cloned()
                .collect(),
            textos: Vec::new(),
            pecas: Vec::new(),
            listas_soltas: Vec::new(),
        };
        let grudadas = estes.linhas_grudadas();
        assert!(grudadas.is_empty(), "{nome}: {grudadas:#?}");
    }
}

/// O arrasto de um punho da rampa, capturada: o mesmo ponto da tela, a mesma acção.
#[test]
fn dragging_a_ramp_handle_sends_the_same_action_as_the_old_panel() {
    let b = {
        let mut b = com_rampa();
        b.texture_ramp_stop_count = 2;
        b.texture_ramp_stops[0] = [0.1, 0.0, 0.0, 0.0, 1.0, 0.0];
        b.texture_ramp_stops[1] = [0.9, 1.0, 1.0, 1.0, 1.0, 1.0];
        b
    };
    let punho = p::painter_brush_texture_ramp_handle_id(1);
    let outro = p::painter_brush_texture_ramp_handle_id(0);
    let arrasta = |cartoes: bool| {
        let mut m = abre(cartoes, b);
        let r = m.rect(punho).expect("o punho");
        let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
        // A barra tem outra largura na coluna do cartão: o gesto é a mesma fracção dela.
        let barra = (r.x - m.rect(outro).expect("o outro punho").x) / 0.8;
        let dx = barra * 0.2;
        let mut ev = m.ponteiro(PointerKind::Down, x, y);
        ev.extend(m.ponteiro(PointerKind::Move, x - dx * 0.5, y));
        ev.extend(m.ponteiro(PointerKind::Move, x - dx, y));
        ev.extend(m.ponteiro(PointerKind::Up, x - dx, y));
        let accoes = m.aplica(ev);
        let accao = accoes.last().expect("o punho mandou").clone();
        let (cabeca, cauda) = accao.split_once("\"1:").expect("o stop 1");
        let pos: f32 = cauda.trim_end_matches("\"))").parse().expect("a posição");
        (cabeca.to_owned(), pos)
    };
    let ((id_v, pos_v), (id_n, pos_n)) = (arrasta(false), arrasta(true));
    assert_eq!(id_v, id_n);
    assert!(
        (pos_v - 0.7).abs() < 0.02 && (pos_n - 0.7).abs() < 0.02,
        "{pos_v} {pos_n}"
    );
}

/// A prévia de cada padrão tem o seu lugar, debaixo da lista dele: no mock ela só se vê no espaço que ocupa.
#[test]
fn the_previews_have_their_place() {
    let b = com_papel(com_grao());
    let m = abre(true, b);
    for (lista, seguinte) in [
        (
            p::PAINTER_SUBSTRATE_ROUGHNESS,
            p::PAINTER_WATERCOLOR_PAPER_OFFSET_X,
        ),
        (
            p::PAINTER_BRUSH_TEXTURE_KIND,
            p::PAINTER_BRUSH_TEXTURE_MAPPING,
        ),
    ] {
        let (a, c) = (
            m.rect(lista).expect("a lista"),
            m.rect(seguinte).expect("o seguinte"),
        );
        assert!(
            c.y - (a.y + a.h) > 56.0,
            "{lista:?}: sem prévia, {a:?} {c:?}"
        );
    }
}

/// Sem Randomize na ferramenta que não pinta cor; o cartão Paper mantém-se.
#[test]
fn a_colourless_tool_has_no_randomize_card() {
    let m = abre(true, pincel(PaintMedia::Digital, "smear").0);
    assert!(m.rect(p::PAINTER_BRUSH_RANDOMIZE_CHIPS[0]).is_none());
    assert!(m.rect(grip_of(ALEATORIO)).is_none());
    assert!(m.rect(p::PAINTER_SUBSTRATE_RELIEF).is_some());
}

#[test]
fn the_inert_grain_colors_keep_their_hint() {
    let b = com_rampa();
    let m = abre(true, b);
    let esperada = abre(false, b)
        .host
        .store()
        .tooltip_for(p::PAINTER_BRUSH_TEXTURE_RAMP_ENABLE)
        .map(str::to_owned);
    assert_eq!(
        m.host
            .store()
            .tooltip_for(p::PAINTER_BRUSH_TEXTURE_RAMP_ENABLE)
            .map(str::to_owned),
        esperada
    );
}
