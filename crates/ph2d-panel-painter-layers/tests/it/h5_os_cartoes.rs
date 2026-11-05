//! **Os cartões Shape e Shape Color em linhas de cartão, fatia H5** (`docs/CartoesUI/13_PAINTER_PLANO.md`), e as dívidas das
//! fatias H3 e H4. Pelo gesto real, nos rects pintados, e o que chega ao barramento é o MESMO do painel de antes. O universo
//! é derivado do próprio painel de antes: o que aparece quando estes cartões abrem. Nenhum motor, preset ou pincel muda.

use super::h1_os_cartoes::{Mesa, esc, pincel, valor_enviado};
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::panel::cards::{censo, grip_of};
use ph2d_editor_core::widget::key_mark::{KeyMark, add_key_mark, key_mark_target, set_key_marks};
use ph2d_editor_core::zones::Rect;
use ph2d_host::PointerKind;
use ph2d_panel_painter_layers::{FALLBACK_BRUSH, PainterLayersPanel};
use ph2d_tool_painter::ids as p;
use ph2d_tool_painter::{BrushSettings, Falloff, Inercias, PaintMedia, TextureKind, WetTool};
use std::collections::BTreeSet;

const FORMA: NodeId = p::PAINTER_SHAPE_SECTION;
const TOM: NodeId = p::PAINTER_SHAPE_RAMP_SECTION;

fn digital() -> BrushSettings {
    pincel(PaintMedia::Digital, "brush").0
}

fn molhado() -> BrushSettings {
    let mut b = pincel(PaintMedia::WetPaint, "brush").0;
    b.dab_follows_stroke = true;
    b.pontas_rectas = true;
    b
}

fn com_textura() -> BrushSettings {
    let mut b = digital();
    b.shape_kind = TextureKind::COUNT - 2;
    b.inercias = Inercias::NENHUMA;
    b.shape_relief = 0.4;
    b.impasto_applies = true;
    b
}

fn com_imagem() -> BrushSettings {
    let mut b = digital();
    b.shape_kind = TextureKind::Image.to_u8();
    b.shape_has_image = true;
    b.shape_hose_count = 3;
    b.shape_layer_count = 1;
    b.shape_has_alpha_choice = true;
    b.document_layer_count = 2;
    b
}

fn por_camada() -> BrushSettings {
    let mut b = com_imagem();
    b.shape_layer_count = 2;
    b.shape_per_layer_color = true;
    b.shape_layer_color_on[1] = true;
    b
}

pub(super) fn com_rampa() -> BrushSettings {
    let mut b = rampa_inerte();
    b.shape_kind = TextureKind::COUNT - 2;
    b.inercias = Inercias::NENHUMA;
    b
}

/// Sem textura na forma, a Texture Color fica esmaecida e sem clique: a lei que a ferramenta publica.
fn rampa_inerte() -> BrushSettings {
    let mut b = digital();
    b.shape_color_ramp_enabled = true;
    b.shape_texture_color = true;
    b.shape_falloff_color = true;
    b
}

fn queda_por_cima() -> BrushSettings {
    let mut b = com_rampa();
    b.shape_falloff_first = true;
    b
}

fn com_dual() -> BrushSettings {
    let mut b = digital();
    b.ponta[14] = 1.0;
    b.ponta[15] = f64::from(TextureKind::Image.to_u8()) as _;
    b
}

fn custom() -> BrushSettings {
    let mut b = digital();
    b.falloff = Falloff::Custom.to_u8();
    b
}

fn estados() -> Vec<(&'static str, BrushSettings)> {
    let mut aguada_manual = pincel(PaintMedia::Watercolor, "brush").0;
    aguada_manual.watercolor_shape_auto = false;
    let mut aguada_auto = aguada_manual;
    aguada_auto.watercolor_shape_auto = true;
    let mut rampa_sem_camadas = com_rampa();
    rampa_sem_camadas.shape_texture_color = false;
    rampa_sem_camadas.shape_falloff_color = false;
    let mut sem_bw = com_rampa();
    sem_bw.eraser = true;
    let mut so_queda = queda_por_cima();
    so_queda.shape_texture_color = false;
    let mut sem_pontas = molhado();
    sem_pontas.dab_follows_stroke = false;
    sem_pontas.pontas_rectas = false;
    let mut uma_imagem = com_imagem();
    uma_imagem.shape_hose_count = 1;
    uma_imagem.shape_has_alpha_choice = false;
    let mut camadas_sem_cor = por_camada();
    camadas_sem_cor.shape_per_layer_color = false;
    let mut borracha_com_camada = por_camada();
    borracha_com_camada.eraser = true;
    let mut borracha_na_agua = pincel(PaintMedia::Watercolor, "eraser").0;
    borracha_na_agua.media = PaintMedia::Watercolor.to_u8();
    borracha_na_agua.watercolor = true;
    let mut ferramenta_da_agua = molhado();
    ferramenta_da_agua.wet_tool = WetTool::Blow;
    let mut mistura_da_agua = molhado();
    mistura_da_agua.wet_tool = WetTool::Blend;
    let mut esculpe = pincel(PaintMedia::Impasto, "brush").0;
    esculpe.is_sculpt = true;
    // O Smear não lê a cor por camada: a caixa dela não se oferece, mesmo com camadas na ponta.
    let mut smear_com_camadas = pincel(PaintMedia::Digital, "smear").0;
    smear_com_camadas.shape_kind = TextureKind::Image.to_u8();
    smear_com_camadas.shape_has_image = true;
    smear_com_camadas.shape_layer_count = 2;
    vec![
        ("smear com camadas", smear_com_camadas),
        ("digital", digital()),
        ("molhado", molhado()),
        ("impasto", pincel(PaintMedia::Impasto, "brush").0),
        ("aguada automática", aguada_auto),
        ("aguada à mão", aguada_manual),
        ("textura", com_textura()),
        ("imagem", com_imagem()),
        ("por camada", por_camada()),
        ("dual", com_dual()),
        ("custom", custom()),
        ("rampa", com_rampa()),
        ("rampa inerte", rampa_inerte()),
        ("queda por cima", queda_por_cima()),
        ("rampa sem camadas", rampa_sem_camadas),
        ("rampa na borracha", sem_bw),
        (
            "borracha no molhado",
            pincel(PaintMedia::WetPaint, "eraser").0,
        ),
        (
            "borracha na aguada",
            pincel(PaintMedia::Watercolor, "eraser").0,
        ),
        ("smear", pincel(PaintMedia::Digital, "smear").0),
        ("inpaint", pincel(PaintMedia::Digital, "inpaint").0),
        ("só a queda", so_queda),
        ("molhado sem pontas", sem_pontas),
        ("uma imagem", uma_imagem),
        ("camadas sem cor", camadas_sem_cor),
        ("borracha com camadas", borracha_com_camada),
        ("borracha com a aguada armada", borracha_na_agua),
        ("Blow da água", ferramenta_da_agua),
        ("Blend da água", mistura_da_agua),
        ("esculpir", esculpe),
        ("blur", pincel(PaintMedia::Digital, "blur").0),
    ]
}

/// Nos cartões, só estes abertos. O painel de antes fica inteiro: no mock a dobra dele não assenta.
fn so(cartoes: bool, b: BrushSettings, abertos: &[NodeId]) -> Mesa {
    let mut m = Mesa::nova(cartoes, b, false);
    if cartoes {
        m.so_abertos(abertos);
    }
    m
}

/// Os ids que estes dois cartões trazem quando abrem, neste estado.
fn do_corpo(b: BrushSettings) -> BTreeSet<NodeId> {
    let abertos = so(true, b, &[FORMA, TOM]).ids();
    let dobrados = so(true, b, &[]).ids();
    abertos.difference(&dobrados).copied().collect()
}

/// Os ids do painel de antes entre o cabeçalho do Shape e o do Paper, fora o cromo das secções.
fn do_painel_de_antes(b: BrushSettings) -> BTreeSet<NodeId> {
    let m = so(false, b, &[]);
    let (Some(de), Some(ate)) = (
        m.rect(grip_of(FORMA)),
        m.rect(grip_of(p::PAINTER_WATERCOLOR_PAPER_SECTION)),
    ) else {
        return BTreeSet::new();
    };
    let cromo = [
        FORMA,
        TOM,
        grip_of(TOM),
        p::PAINTER_SHAPE_RESET,
        p::PAINTER_SHAPE_RAMP_RESET,
    ];
    m.ids()
        .into_iter()
        .filter(|id| !cromo.contains(id))
        .filter(|id| {
            m.rect(*id)
                .is_some_and(|r| r.y >= de.y + de.h && r.y < ate.y)
        })
        .collect()
}

/// ⭐ As mesmas condições, nas DUAS direcções, em cada estado. O universo sai do painel de antes.
#[test]
fn shape_and_shape_color_show_the_same_controls_as_the_old_panel() {
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
    assert!(vistos.len() > 70, "a régua viu pouco: {}", vistos.len());
    for id in [
        p::PAINTER_BRUSH_FALLOFF,
        p::PAINTER_TAPER_TIP_START,
        p::PAINTER_SHAPE_KIND,
        p::PAINTER_SHAPE_HOSE_ADD,
        p::PAINTER_DUAL_IMAGE,
        p::PAINTER_SHAPE_COLOR_SWAP,
        p::PAINTER_SHAPE_COLOR_BLEND,
        p::PAINTER_BRUSH_PONTAS_ALEM,
    ] {
        assert!(vistos.contains(&id), "{id:?} nunca à vista");
    }
}

/// Os números nativos deste estado, os que o cartão marca.
fn numeros(b: BrushSettings) -> Vec<NodeId> {
    let m = so(true, b, &[FORMA, TOM]);
    do_corpo(b)
        .into_iter()
        .filter(|id| m.host.store().card_field(*id).is_some())
        .collect()
}

/// Escrever em cada número manda o mesmo número pelo mesmo evento, com a faixa de antes.
#[test]
fn typing_each_number_sends_the_same_action_as_the_old_panel() {
    let mut vistos = BTreeSet::new();
    for (nome, b) in estados() {
        let ids = numeros(b);
        vistos.extend(ids.iter().copied());
        let (mut velha, mut nova) = (so(false, b, &[FORMA, TOM]), so(true, b, &[FORMA, TOM]));
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
    assert!(vistos.len() >= 20, "viu {} números", vistos.len());
}

/// Cada número semeia o valor e a faixa do painel de antes.
#[test]
fn every_number_shows_the_same_value_and_range_as_the_old_panel() {
    for (b, id) in [
        (digital(), p::PAINTER_BRUSH_HARDNESS),
        (com_textura(), p::PAINTER_SHAPE_ANGLE),
        (com_rampa(), p::PAINTER_SHAPE_TEXTURE_OPACITY),
        (com_rampa(), p::PAINTER_SHAPE_FALLOFF_OPACITY),
    ] {
        assert!(
            numeros(b).contains(&id),
            "{id:?} sem marcar: a régua não o veria"
        );
    }
    for (nome, mut b) in estados() {
        b.hardness = 0.37;
        b.taper.tip_start = 0.21;
        b.taper.fall_off = 1.5;
        b.dab_quina = 0.6;
        b.shape_offset = [0.11, -0.13];
        b.shape_size = [1.2, 0.8];
        b.shape_relief = b.shape_relief.max(0.3);
        let (velha, nova) = (so(false, b, &[FORMA, TOM]), so(true, b, &[FORMA, TOM]));
        for id in numeros(b) {
            let (s_v, s_n) = (velha.host.store(), nova.host.store());
            assert_eq!(s_v.number_value(id), s_n.number_value(id), "{nome}: {id:?}");
            // O chip do stop não tinha faixa; o campo do cartão arrasta, e tem: 0..último stop, 0..1.
            if !STOPS.contains(&id) {
                assert_eq!(
                    s_v.number_range(id),
                    s_n.number_range(id),
                    "{nome}: faixa de {id:?}"
                );
            }
        }
    }
}

const STOPS: [NodeId; 4] = [
    p::PAINTER_SHAPE_RAMP_STOP_INDEX,
    p::PAINTER_SHAPE_RAMP_STOP_POS,
    p::PAINTER_SHAPE_FALLOFF_RAMP_STOP_INDEX,
    p::PAINTER_SHAPE_FALLOFF_RAMP_STOP_POS,
];

/// O scrub de cada número move só o seu campo, um quadro a meio não o puxa de volta, e o Esc devolve.
#[test]
fn scrubbing_each_number_moves_only_it_and_escape_gives_it_back() {
    for b in [molhado(), com_textura(), com_rampa()] {
        // Os campos do stop têm prova própria (`h10`): a ferramenta recebe-os pelo editor da rampa.
        let mut ids = numeros(b);
        ids.retain(|i| !STOPS.contains(i));
        assert!(ids.len() >= 6, "só {}", ids.len());
        for id in &ids {
            let le = |m: &Mesa, i: NodeId| m.host.store().number_value(i).expect("semeado");
            let mut mesa = so(true, b, &[FORMA, TOM]);
            let antes = le(&mesa, *id);
            let outros: Vec<(NodeId, f64)> = ids
                .iter()
                .filter(|o| *o != id)
                .map(|o| (*o, le(&mesa, *o)))
                .collect();
            let mut ev = mesa.arrasta_a_pega(*id, 60.0);
            let mut no_gesto = le(&mesa, *id);
            if (no_gesto - antes).abs() < 1e-9 {
                mesa = so(true, b, &[FORMA, TOM]);
                ev = mesa.arrasta_a_pega(*id, -60.0);
                no_gesto = le(&mesa, *id);
            }
            assert!((no_gesto - antes).abs() > 1e-9, "{id:?} não se moveu");
            let accoes = mesa.aplica(ev);
            let enviado = valor_enviado(&accoes, *id).expect("o painel avisa a ferramenta");
            assert!((enviado - no_gesto).abs() < 1e-3, "{id:?} {enviado}");
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

/// Cada caixa, botão e lista: o mesmo clique, a mesma acção.
#[test]
fn every_click_sends_the_same_action_as_the_old_panel() {
    let mut vistos = 0;
    // Só os estados que trazem controlos diferentes: cada clique são duas mesas.
    let medem = [
        "digital",
        "molhado",
        "aguada à mão",
        "textura",
        "imagem",
        "por camada",
        "dual",
        "custom",
        "rampa",
        "queda por cima",
    ];
    for (nome, b) in estados().into_iter().filter(|(n, _)| medem.contains(n)) {
        let mesa = so(true, b, &[FORMA, TOM]);
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
            let velho = so(false, b, &[FORMA, TOM]).clica(id);
            let novo = so(true, b, &[FORMA, TOM]).clica(id);
            assert_eq!(velho, novo, "{nome}: {id:?}");
            vistos += 1;
        }
    }
    assert!(vistos >= 40, "viu {vistos} cliques");
}

/// Cada lista abre debaixo da caixa, e a opção envia o mesmo.
#[test]
fn each_list_opens_under_its_box_and_its_option_sends_the_same() {
    let dual = p::PAINTER_PONTA_FIELDS[14];
    for (b, caixa, opcao) in [
        (
            digital(),
            p::PAINTER_BRUSH_FALLOFF,
            p::painter_brush_falloff_option_id(1),
        ),
        (
            digital(),
            p::PAINTER_SHAPE_KIND,
            p::painter_shape_kind_option_id(1),
        ),
        (
            digital(),
            p::PAINTER_SHAPE_FOLLOW,
            p::painter_shape_follow_option_id(1),
        ),
        (digital(), dual, p::painter_ponta_option_id(14, 1)),
        (
            com_rampa(),
            p::PAINTER_SHAPE_COLOR_BLEND,
            p::painter_shape_color_blend_option_id(1),
        ),
    ] {
        let escolhe = |cartoes: bool| {
            let mut mesa = so(cartoes, b, &[FORMA, TOM]);
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

/// O arrasto de cada gizmo capturado (curva, Taper, Flatten, punhos da rampa): a tela do gesto é a que está pintada, por
/// isso o mesmo ponto da tela dá a mesma acção do painel de antes.
#[test]
fn each_gizmo_drag_sends_the_same_action_as_the_old_panel() {
    let mut vistos = BTreeSet::new();
    for b in [custom(), com_rampa(), queda_por_cima()] {
        let punhos: Vec<NodeId> = {
            let m = so(true, b, &[FORMA, TOM]);
            do_corpo(b)
                .into_iter()
                .filter(|id| {
                    matches!(
                        m.host.store().get(*id),
                        Some(InteractiveState::CurvePoint { .. })
                    )
                })
                .collect()
        };
        for id in punhos {
            let arrasta = |cartoes: bool| {
                let mut mesa = so(cartoes, b, &[FORMA, TOM]);
                let tela = match mesa.host.store().get(id) {
                    Some(InteractiveState::CurvePoint { canvas, .. }) => *canvas,
                    _ => panic!("{id:?} sem tela"),
                };
                let r = mesa.rect(id).expect("o punho");
                let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
                assert!(
                    cx >= tela.x - 12.0 && cx <= tela.x + tela.w + 12.0,
                    "{id:?}: punho {cx} fora da tela {tela:?}"
                );
                assert!(
                    cy >= tela.y - 12.0 && cy <= tela.y + tela.h + 12.0,
                    "{id:?}: punho {cy} fora da tela {tela:?}"
                );
                let mut ev = mesa.ponteiro(PointerKind::Down, cx, cy);
                let (ax, ay) = (tela.x + tela.w * 0.25, tela.y + tela.h * 0.25);
                ev.extend(mesa.ponteiro(PointerKind::Move, ax, ay));
                ev.extend(mesa.ponteiro(PointerKind::Up, ax, ay));
                let accoes = mesa.aplica(ev);
                mesa.quadro();
                (accoes, mesa.host.drained_actions().len())
            };
            let (velho, novo) = (arrasta(false), arrasta(true));
            assert_eq!(velho, novo, "{id:?}");
            vistos.insert(id);
        }
    }
    assert!(vistos.len() >= 7, "viu {} punhos", vistos.len());
}

/// O Reset de cada cartão mora no cabeçalho, com o mesmo id e o mesmo efeito.
#[test]
fn the_reset_of_each_section_lives_in_the_card_header() {
    for id in [p::PAINTER_SHAPE_RESET, p::PAINTER_SHAPE_RAMP_RESET] {
        let b = com_rampa();
        let (velho, novo) = (
            so(false, b, &[FORMA, TOM]).clica(id),
            so(true, b, &[FORMA, TOM]).clica(id),
        );
        assert!(!novo.is_empty(), "{id:?}: nada no barramento");
        assert_eq!(velho, novo, "{id:?}");
    }
}

/// Os números levam a marca de animação de antes; as listas e as caixas, que não a tinham, continuam sem ela.
#[test]
fn each_control_keeps_its_animation_mark() {
    for (nome, b) in estados() {
        let ids = do_painel_de_antes(b);
        let conta = |cartoes: bool| {
            set_key_marks(std::collections::BTreeMap::new());
            for id in &ids {
                add_key_mark(*id, KeyMark::Chave);
            }
            let m = so(cartoes, b, &[FORMA, TOM]);
            let alvos = m
                .ids()
                .into_iter()
                .filter(|i| key_mark_target(*i).is_some())
                .count();
            set_key_marks(std::collections::BTreeMap::new());
            alvos
        };
        // Os cartões dobrados dos outros não pintam marca, e o painel de antes pinta só a destes ids.
        assert_eq!(conta(false), conta(true), "{nome}");
    }
}

/// O duplo clique do campo volta à fábrica do motor.
#[test]
fn each_number_knows_its_factory_value() {
    let m = so(true, digital(), &[FORMA, TOM]);
    for (id, fabrica) in [
        (p::PAINTER_BRUSH_HARDNESS, FALLBACK_BRUSH.hardness),
        (p::PAINTER_TAPER_FALL_OFF, FALLBACK_BRUSH.taper.fall_off),
    ] {
        let f = m.host.store().card_field(id).expect("marcado").default;
        assert!(
            (f.expect("fábrica") - f64::from(fabrica)).abs() < 1e-6,
            "{id:?}"
        );
    }
}

pub(super) fn sem_sobrepor(m: &Mesa, ids: &BTreeSet<NodeId>, nome: &str) {
    let rects: Vec<(NodeId, Rect)> = ids
        .iter()
        .filter(|id| {
            !matches!(
                m.host.store().get(**id),
                Some(InteractiveState::CurvePoint { .. })
            )
        })
        .filter_map(|id| m.rect(*id).map(|r| (*id, r)))
        .collect();
    for (i, (a, ra)) in rects.iter().enumerate() {
        for (b, rb) in &rects[i + 1..] {
            let longe = ra.x + ra.w <= rb.x + 0.5
                || rb.x + rb.w <= ra.x + 0.5
                || ra.y + ra.h <= rb.y + 0.5
                || rb.y + rb.h <= ra.y + 0.5;
            assert!(longe, "{nome}: {a:?} por cima de {b:?}: {ra:?} {rb:?}");
        }
    }
}

/// Nenhuma peça capturada ou linha fica por cima de outra, em nenhum estado.
#[test]
fn no_control_of_these_cards_overlaps_another() {
    for (nome, b) in estados() {
        let m = so(true, b, &[FORMA, TOM]);
        sem_sobrepor(&m, &do_corpo(b), nome);
    }
}

pub(super) fn na_mesma_linha(m: &Mesa, a: NodeId, b: NodeId) {
    let (ra, rb) = (m.rect(a).expect("a"), m.rect(b).expect("b"));
    assert!(
        (ra.y - rb.y).abs() < 0.5 && ra.x + ra.w <= rb.x,
        "{a:?} e {b:?}: {ra:?} {rb:?}"
    );
}

/// Irmãos curtos dividem a linha (05 §4, lei 1).
#[test]
fn short_siblings_share_a_row() {
    let m = so(true, com_textura(), &[FORMA, TOM]);
    na_mesma_linha(&m, p::PAINTER_BRUSH_FALLOFF, p::PAINTER_BRUSH_HARDNESS);
    na_mesma_linha(&m, p::PAINTER_TAPER_TIP_START, p::PAINTER_TAPER_OPACITY);
    na_mesma_linha(
        &m,
        p::PAINTER_TAPER_FALL_OFF,
        p::PAINTER_TAPER_FALL_OFF_START,
    );
    na_mesma_linha(&m, p::PAINTER_SHAPE_OFFSET_X, p::PAINTER_SHAPE_OFFSET_Y);
    na_mesma_linha(&m, p::PAINTER_SHAPE_SIZE_X, p::PAINTER_SHAPE_SIZE_Y);
    let m = so(true, com_rampa(), &[FORMA, TOM]);
    let rampa = ph2d_tool_painter::ids::PAINTER_SHAPE_RAMP_ENABLE;
    na_mesma_linha(&m, rampa, p::PAINTER_SHAPE_RAMP_BW);
    na_mesma_linha(
        &m,
        p::PAINTER_SHAPE_FALLOFF_COLOR,
        p::PAINTER_SHAPE_COLOR_SWAP,
    );
    na_mesma_linha(
        &m,
        p::PAINTER_SHAPE_TEXTURE_OPACITY,
        p::PAINTER_SHAPE_COLOR_BLEND,
    );
    let m = so(true, molhado(), &[FORMA, TOM]);
    na_mesma_linha(&m, p::PAINTER_SHAPE_FOLLOW, p::PAINTER_BRUSH_DAB_QUINA);
    na_mesma_linha(
        &m,
        p::PAINTER_BRUSH_DAB_ROTATION_LAG,
        p::PAINTER_BRUSH_PONTAS_ESBATE,
    );
    na_mesma_linha(
        &m,
        p::PAINTER_BRUSH_PONTAS_RECTAS,
        p::PAINTER_BRUSH_PONTAS_ALEM,
    );
}

/// Em cada estado, nenhum texto cortado e nenhuma linha grudada nestes dois cartões.
#[test]
fn no_text_is_cut_in_any_state() {
    for (nome, b) in estados() {
        let (_, c) = censo::grava(|| so(true, b, &[FORMA, TOM]));
        let cortados = c.textos_cortados();
        assert!(cortados.is_empty(), "{nome}: {cortados:#?}");
        // Só as linhas destes cartões: os corpos ainda capturados dos outros têm o passo de antes.
        let estes = censo::Censo {
            linhas: c
                .linhas
                .iter()
                .filter(|l| l.secao == FORMA || l.secao == TOM)
                .cloned()
                .collect(),
            textos: Vec::new(),
            pecas: Vec::new(),
        };
        let grudadas = estes.linhas_grudadas();
        assert!(grudadas.is_empty(), "{nome}: {grudadas:#?}");
    }
}

/// A prévia da silhueta tem o seu lugar, entre a lista da textura e a moldura.
#[test]
fn the_preview_has_its_place() {
    let m = so(true, com_textura(), &[FORMA, TOM]);
    let lista = m.rect(p::PAINTER_SHAPE_KIND).expect("a lista");
    let moldura = m.rect(p::PAINTER_SHAPE_OFFSET_X).expect("o Offset");
    let dual = m.rect(p::PAINTER_PONTA_FIELDS[14]).expect("o Dual brush");
    // Sem nada entre elas, a fonte e o Dual brush dividem a linha.
    assert!((dual.y - lista.y).abs() < 0.5, "{lista:?} {dual:?}");
    assert!(moldura.y >= dual.y + dual.h + 56.0, "{dual:?} {moldura:?}");
}

/// Os punhos de um cartão solto continuam nele: a tela vem de lá, nunca da coluna da lista.
#[test]
fn the_gizmos_follow_a_loose_card() {
    let b = digital();
    let id = p::painter_taper_handle_id(0);
    let mut mesa = so(true, b, &[FORMA]);
    let antes = mesa.rect(id).expect("na lista");
    let g = mesa.rect(grip_of(FORMA)).expect("a pega");
    let (gx, gy) = (g.x + g.w * 0.5, g.y + g.h * 0.5);
    let _ = mesa.ponteiro(PointerKind::Down, gx, gy);
    let (dx, dy) = (-500.0, 40.0);
    for k in 1..=8 {
        let f = k as f32 / 8.0;
        let _ = mesa.ponteiro(PointerKind::Move, gx + dx * f, gy + dy * f);
        mesa.quadro();
    }
    let _ = mesa.ponteiro(PointerKind::Up, gx + dx, gy + dy);
    mesa.assenta();
    assert!(
        mesa.host
            .store()
            .is_loose_in(FORMA, ph2d_editor_core::ids::PAINTER_LAYERS_PANEL),
        "o cartão não se soltou"
    );
    let depois = mesa.rect(id).expect("no solto");
    assert!((depois.x - antes.x).abs() > 100.0, "{antes:?} {depois:?}");
    let tela = match mesa.host.store().get(id) {
        Some(InteractiveState::CurvePoint { canvas, .. }) => *canvas,
        _ => panic!("sem tela"),
    };
    let cx = depois.x + depois.w * 0.5;
    assert!(
        cx >= tela.x - 12.0 && cx <= tela.x + tela.w + 12.0,
        "a tela ficou na coluna: {cx} {tela:?}"
    );
}

// ── As dívidas de H3 e H4 ─────────────────────────────────────────────────────────────────────────────

/// `a_presente_sem_aguada`: a Borracha apaga como no Digital, e o cartão da aguada nem o cabeçalho deixa.
#[test]
fn the_eraser_in_watercolor_leaves_no_watercolor_card() {
    let grip = grip_of(p::PAINTER_WATERCOLOR_SECTION);
    let pincel_a = Mesa::nova(true, pincel(PaintMedia::Watercolor, "brush").0, false);
    assert!(pincel_a.rect(grip).is_some(), "a régua não vê o cartão");
    let mut b = pincel(PaintMedia::Watercolor, "eraser").0;
    b.media = PaintMedia::Watercolor.to_u8();
    let borracha = Mesa::nova(true, b, false);
    assert!(borracha.rect(grip).is_none(), "um cartão vazio à vista");
}

/// `i_ferramenta_sem_ligar`: o verbo aceso é o que está na mão. Cada um dos dez pinta outra sequência.
#[test]
fn the_lit_tool_is_the_one_in_hand() {
    let mut vistos = BTreeSet::new();
    let janela = Rect::new(0.0, 0.0, 1600.0, 6000.0);
    for i in 0..p::PAINTER_IMPASTO_TOOL_IDS.len() {
        let mut b = pincel(PaintMedia::Impasto, "brush").0;
        b.impasto_tool = i as u8;
        let mut m = Mesa::nova(true, b, false);
        m.so_abertos(&[p::PAINTER_IMPASTO_SECTION]);
        let dados = m
            .host
            .paint_draw_data::<PainterLayersPanel>(&mut m.st, janela);
        assert!(vistos.insert(dados), "o verbo {i} pinta como outro");
    }
}

/// `h3_borracha_com_knob`: com a Borracha, os knobs da ferramenta da água não se oferecem, nem com o Wet escolhido.
#[test]
fn the_eraser_shows_no_knob_of_the_water_tool_left_chosen() {
    let mut b = pincel(PaintMedia::WetPaint, "eraser").0;
    b.wet_tool = WetTool::Wet;
    let knobs = b.wet_knobs.knobs_da_ferramenta(WetTool::Wet);
    assert!(!knobs.is_empty(), "a régua precisa de um knob");
    let m = Mesa::nova(true, b, false);
    for (id, ..) in knobs {
        assert!(m.rect(id).is_none(), "{id:?} à vista com a Borracha");
    }
}

/// A sobra da H3: Real Oil, Pigment mixing e Live Water dividem linhas, e ligar o Real Oil não os troca de lugar.
#[test]
fn the_wet_checks_share_rows_and_real_oil_moves_none_of_them() {
    let wet = p::PAINTER_WETPAINT_SECTION;
    let mut b = pincel(PaintMedia::WetPaint, "brush").0;
    b.wet_oleo_real = false;
    let desligado = so(true, b, &[wet]);
    na_mesma_linha(
        &desligado,
        p::PAINTER_WETPAINT_OLEO_REAL,
        p::PAINTER_WETPAINT_PIGMENT_MIXING,
    );
    na_mesma_linha(
        &desligado,
        p::PAINTER_WETPAINT_LIVE_WATER,
        p::PAINTER_WETPAINT_TUNING,
    );
    na_mesma_linha(
        &desligado,
        p::PAINTER_WETPAINT_SHOWWET,
        p::PAINTER_WETPAINT_PAPER_VISUAL,
    );
    b.wet_oleo_real = true;
    let ligado = so(true, b, &[wet]);
    for id in [
        p::PAINTER_WETPAINT_OLEO_REAL,
        p::PAINTER_WETPAINT_PIGMENT_MIXING,
        p::PAINTER_WETPAINT_LIVE_WATER,
        p::PAINTER_WETPAINT_TUNING,
    ] {
        assert_eq!(desligado.rect(id), ligado.rect(id), "{id:?} mudou de lugar");
    }
}

/// A sobra da H4: a lista TOOL não deixa um verbo sozinho na última fila.
#[test]
fn the_tool_list_leaves_no_orphan() {
    let m = so(
        true,
        pincel(PaintMedia::Impasto, "brush").0,
        &[p::PAINTER_IMPASTO_SECTION],
    );
    let mut filas = std::collections::BTreeMap::<i32, usize>::new();
    for id in p::PAINTER_IMPASTO_TOOL_IDS {
        *filas
            .entry(m.rect(id).expect("o verbo").y as i32)
            .or_default() += 1;
    }
    let (menor, maior) = (
        filas.values().min().copied().unwrap_or(0),
        filas.values().max().copied().unwrap_or(0),
    );
    assert!(menor >= 2 && maior - menor <= 1, "filas: {filas:?}");
}
