//! **O fecho do painel: ajustes e camada Texture em linhas de cartão, sem captura** (`docs/CartoesUI/13_PAINTER_PLANO.md`).
//! Pelo gesto real, nos rects pintados, e o que chega ao barramento é o MESMO de `PH2D_CARDS=0`.

use super::h1_os_cartoes::{Mesa, esc, valor_enviado};
use super::h5_os_cartoes::sem_sobrepor;
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::panel::cards::censo;
use ph2d_host::PointerKind;
use ph2d_panel_painter_layers::state::set_current_layers;
use ph2d_panel_painter_layers::{FALLBACK_BRUSH, PainterLayersPanel};
use ph2d_tool_painter::ids as p;
use ph2d_tool_painter::ids::{PainterLayerWidget as W, painter_layer_widget_id};
use ph2d_tool_painter::{
    AdjustmentKind, AdjustmentParams, LayerId, LayerKind, LayerStack, TextureKind, TextureLayer,
};
use std::collections::BTreeSet;

const SLIDERS: [W; 8] = [
    W::AdjParam0,
    W::AdjParam1,
    W::AdjParam2,
    W::AdjParam3,
    W::AdjParam4,
    W::AdjParam5,
    W::AdjParam6,
    W::AdjParam7,
];
const CHIPS: [W; 8] = [
    W::AdjChip0,
    W::AdjChip1,
    W::AdjChip2,
    W::AdjChip3,
    W::AdjChip4,
    W::AdjChip5,
    W::AdjChip6,
    W::AdjChip7,
];

fn com_ajuste(k: AdjustmentKind) -> (LayerStack, LayerId) {
    let mut s = LayerStack::new();
    let _ = s.add_raster("Base", 8, 8).expect("nasce");
    let a = s.add_adjustment(k).expect("o ajuste nasce");
    (s, a)
}

/// Uma curva com um ponto no canal mestre e outro no verde: de fábrica, as curvas não têm pega nenhuma.
fn com_curva() -> (LayerStack, LayerId) {
    let (mut s, a) = com_ajuste(AdjustmentKind::Curves);
    if let Some(LayerKind::Adjustment(adj)) = s.get_mut(a).map(|l| &mut l.kind)
        && let AdjustmentParams::Curves(c) = &mut adj.params
    {
        c.points_rgb.points.push([0.5, 0.25]);
        c.points_g.points.push([0.5, 0.75]);
    }
    (s, a)
}

fn com_textura(kind: u8, ramp: bool) -> (LayerStack, LayerId) {
    let mut s = LayerStack::new();
    let _ = s.add_raster("Base", 8, 8).expect("nasce");
    let spec = TextureLayer {
        kind,
        ramp_enabled: ramp,
        ..TextureLayer::default()
    };
    let t = s.add_texture(spec).expect("a textura nasce");
    s.set_active(t);
    (s, t)
}

fn abre(cartoes: bool, s: LayerStack) -> Mesa {
    let mut m = Mesa::nova(cartoes, FALLBACK_BRUSH, true);
    set_current_layers(Some(s));
    m.assenta();
    m
}

fn so_a_base(cartoes: bool) -> BTreeSet<NodeId> {
    let mut s = LayerStack::new();
    let _ = s.add_raster("Base", 8, 8).expect("nasce");
    let m = abre(cartoes, s);
    let ids = m.ids();
    set_current_layers(None);
    ids
}

/// Os ids que a pilha `s` acrescenta à base.
fn os_dela(cartoes: bool, s: LayerStack) -> BTreeSet<NodeId> {
    let base = so_a_base(cartoes);
    let m = abre(cartoes, s);
    let ids: BTreeSet<NodeId> = m.ids().difference(&base).copied().collect();
    set_current_layers(None);
    ids
}

/// A barra com número mora no campo: o `Slider` continua no store, e é ele que avisa a ferramenta.
fn sliders_de(l: LayerId) -> BTreeSet<NodeId> {
    SLIDERS
        .iter()
        .map(|k| painter_layer_widget_id(l.0, *k))
        .collect()
}

fn chips_de(l: LayerId) -> Vec<(NodeId, NodeId)> {
    SLIDERS
        .iter()
        .zip(CHIPS)
        .map(|(s, c)| {
            (
                painter_layer_widget_id(l.0, *s),
                painter_layer_widget_id(l.0, c),
            )
        })
        .collect()
}

fn estados() -> Vec<(String, LayerStack, LayerId)> {
    let mut v: Vec<(String, LayerStack, LayerId)> = AdjustmentKind::ALL
        .iter()
        .map(|k| {
            let (s, l) = com_ajuste(*k);
            (format!("{k:?}"), s, l)
        })
        .collect();
    let (s, l) = com_curva();
    v.push(("Curva com pontos".into(), s, l));
    let (s, l) = com_textura(0, false);
    v.push(("Textura sem padrão".into(), s, l));
    let (s, l) = com_textura(TextureKind::COUNT - 2, false);
    v.push(("Textura com padrão".into(), s, l));
    let (s, l) = com_textura(TextureKind::COUNT - 2, true);
    v.push(("Textura com rampa".into(), s, l));
    v
}

/// ⭐ **O inventário, estado a estado:** nenhum id do painel de antes morre, e nada nasce sem lá estar. Fora só as barras
/// que têm campo: agora o alvo é o campo, ligado à barra. *Mutação: tirar um ramo de `cards_ajustes::corpo` ⇒ vermelho.*
#[test]
fn no_id_of_an_adjustment_or_texture_layer_died() {
    let mut total = (0usize, 0usize);
    for (nome, s, l) in estados() {
        let (velho, novo) = (os_dela(false, s.clone()), os_dela(true, s));
        let sliders = sliders_de(l);
        let mortos: Vec<_> = velho
            .iter()
            .filter(|i| !novo.contains(i) && !sliders.contains(i))
            .collect();
        println!(
            "inventario {nome}: antes {} depois {}",
            velho.len(),
            novo.len()
        );
        assert!(mortos.is_empty(), "{nome}: morreram {mortos:?}");
        total = (total.0 + velho.len(), total.1 + novo.len());
    }
    println!("inventario total: antes {} depois {}", total.0, total.1);
    assert!(total.1 > 200, "a régua viu pouco: {}", total.1);
}

/// Cada botão, aba, opção e caixa manda o mesmo que no painel de antes. *Mutação: tirar `registra` ⇒ vermelho.*
#[test]
fn every_click_of_these_layers_sends_the_same_as_the_old_panel() {
    let mut vistos = 0;
    for (nome, s, l) in estados() {
        let sliders = sliders_de(l);
        let chips: BTreeSet<NodeId> = chips_de(l).iter().map(|(_, c)| *c).collect();
        for id in os_dela(true, s.clone()) {
            let m = abre(true, s.clone());
            let e_botao = matches!(
                m.host.store().get(id),
                Some(InteractiveState::Button { .. })
            );
            set_current_layers(None);
            if !e_botao || sliders.contains(&id) || chips.contains(&id) {
                continue;
            }
            let novo = abre(true, s.clone()).clica(id);
            set_current_layers(None);
            let velho = abre(false, s.clone()).clica(id);
            set_current_layers(None);
            assert_eq!(velho, novo, "{nome}: {id:?}");
            vistos += 1;
        }
    }
    println!("cliques comparados: {vistos}");
    assert!(vistos > 40, "viu {vistos}");
}

/// Arrastar a pega do campo chega à ferramenta pelo `Slider` de sempre, só o dele; o Esc devolve.
/// *Mutação: tirar `ligar_numero` de `cards_ajustes::prepare` ⇒ vermelho.*
#[test]
fn scrubbing_an_adjustment_field_writes_its_own_bar_and_esc_returns() {
    let mut vistos = 0;
    for k in AdjustmentKind::ALL {
        let (s, l) = com_ajuste(k);
        let mixer_etc = matches!(
            k,
            AdjustmentKind::ChannelMixer
                | AdjustmentKind::SelectiveColor
                | AdjustmentKind::GradientMap
        );
        for (slider, chip) in chips_de(l) {
            let m = abre(true, s.clone());
            let tem = m.rect(chip).is_some();
            set_current_layers(None);
            if !tem {
                continue;
            }
            let mut accoes = Vec::new();
            for dx in [40.0, -40.0] {
                let mut m = abre(true, s.clone());
                let ev = m.arrasta_a_pega(chip, dx);
                accoes = m.aplica(ev);
                if accoes.is_empty() {
                    set_current_layers(None);
                    continue;
                }
                let volta = esc(&mut m);
                set_current_layers(None);
                assert!(!volta.is_empty(), "{k:?}: o Esc não devolveu");
                break;
            }
            if mixer_etc {
                assert!(
                    accoes.iter().any(|a| a.contains("SelectOption")),
                    "{k:?}: {accoes:?}"
                );
            } else {
                assert!(
                    valor_enviado(&accoes, slider).is_some(),
                    "{k:?}: {accoes:?}"
                );
                let outros = sliders_de(l).into_iter().filter(|i| *i != slider);
                for o in outros {
                    assert!(valor_enviado(&accoes, o).is_none(), "{k:?}: escreveu {o:?}");
                }
            }
            vistos += 1;
        }
    }
    println!("campos arrastados: {vistos}");
    assert!(vistos > 60, "viu {vistos}");
}

/// O Gamma do Levels só se mostra: a barra nua é o alvo, com o valor no rótulo.
#[test]
fn the_levels_gamma_is_a_bare_bar_that_drags() {
    let (s, l) = com_ajuste(AdjustmentKind::Levels);
    let (gama, chip) = chips_de(l)[1];
    let mut m = abre(true, s);
    assert!(m.rect(chip).is_none(), "o Gamma não tem número a escrever");
    let r = m.rect(gama).expect("a barra");
    let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let mut ev = m.ponteiro(PointerKind::Down, x, y);
    ev.extend(m.ponteiro(PointerKind::Move, x + 10.0, y));
    ev.extend(m.ponteiro(PointerKind::Up, x + 10.0, y));
    let accoes = m.aplica(ev);
    set_current_layers(None);
    assert!(valor_enviado(&accoes, gama).is_some(), "{accoes:?}");
}

/// As abas mudam o que as barras editam: a aba é vista do painel, e o evento da barra leva-a.
/// *Mutação: tirar o `Choice` da saída ⇒ vermelho.*
#[test]
fn the_mixer_tab_chooses_what_the_bars_edit() {
    let (s, l) = com_ajuste(AdjustmentKind::ChannelMixer);
    let mut m = abre(true, s);
    let _ = m.clica(p::painter_mixer_tab_id(l.0, 2));
    m.assenta();
    let (_, chip) = chips_de(l)[0];
    let ev = m.arrasta_a_pega(chip, 40.0);
    let accoes = m.aplica(ev);
    set_current_layers(None);
    assert!(
        accoes
            .iter()
            .any(|a| a.contains(&format!("\"{}:2:0:", l.0))),
        "{accoes:?}"
    );
}

/// A aba do canal troca as pegas da curva.
#[test]
fn the_curve_tab_changes_the_handles() {
    let (s, l) = com_curva();
    let mut m = abre(true, s);
    let g = p::painter_curve_point_id(l.0, 2, 0);
    assert!(m.rect(g).is_none(), "o canal G só com a aba dele");
    let _ = m.clica(p::painter_curve_tab_id(l.0, 2));
    m.assenta();
    let tem = m.rect(g).is_some();
    let _ = m.clica(p::painter_curve_tab_id(l.0, 0));
    set_current_layers(None);
    assert!(tem, "a aba não trocou as pegas");
}

fn gesto_na_pega(cartoes: bool, s: LayerStack, id: NodeId) -> Vec<String> {
    let mut m = abre(cartoes, s);
    let r = m.rect(id).expect("a pega");
    let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let Some(InteractiveState::CurvePoint { canvas, .. }) = m.host.store().get(id).cloned() else {
        panic!("{id:?} sem CurvePoint: o `Tela::depois` não correu");
    };
    assert!(
        x >= canvas.x - 1.0
            && x <= canvas.x + canvas.w + 1.0
            && y >= canvas.y - 1.0
            && y <= canvas.y + canvas.h + 1.0,
        "a pega {r:?} fora da tela {canvas:?}"
    );
    // Um arrasto a meia tela: o ponto de destino é o mesmo nas duas caras, medido na tela do próprio gesto.
    let (dx, dy) = (canvas.w * 0.25, canvas.h * 0.25);
    let mut ev = m.ponteiro(PointerKind::Down, x, y);
    ev.extend(m.ponteiro(PointerKind::Move, x + dx, y + dy));
    ev.extend(m.ponteiro(PointerKind::Up, x + dx, y + dy));
    let a = m.aplica(ev);
    set_current_layers(None);
    a
}

/// A pega da curva e a do degradê: mesmo número de acções e a mesma espécie nas duas caras.
/// *Mutação: pintar a tela sem `depois` ⇒ vermelho.*
#[test]
fn the_curve_and_gradient_handles_reach_the_tool_as_in_the_old_panel() {
    for k in [AdjustmentKind::Curves, AdjustmentKind::GradientMap] {
        let (s, l) = if k == AdjustmentKind::Curves {
            com_curva()
        } else {
            com_ajuste(k)
        };
        let id = if k == AdjustmentKind::Curves {
            p::painter_curve_point_id(l.0, 0, 0)
        } else {
            p::painter_gradient_stop_id(l.0, 0)
        };
        let velho = gesto_na_pega(false, s.clone(), id);
        let novo = gesto_na_pega(true, s, id);
        assert!(!novo.is_empty(), "{k:?}: o arrasto não fez nada");
        assert_eq!(velho.len(), novo.len(), "{velho:?} × {novo:?}");
    }
}

/// As listas da camada Texture abrem por baixo da caixa, e a opção manda o mesmo.
/// *Mutação: tirar `cards_grao::listas_da_camada` ⇒ vermelho.*
#[test]
fn the_texture_layer_lists_open_under_their_box_and_send_the_same() {
    for id in [
        p::PAINTER_BRUSH_TEXTURE_KIND,
        p::PAINTER_BRUSH_TEXTURE_RAMP_MODE,
        p::PAINTER_BRUSH_TEXTURE_RAMP_ALPHA_MODE,
    ] {
        let manda = |cartoes: bool| {
            let (s, _) = com_textura(TextureKind::COUNT - 2, true);
            let mut m = abre(cartoes, s);
            let caixa = m.rect(id).expect("a caixa");
            let antes = m.ids();
            let _ = m.clica(id);
            m.assenta();
            let novos: Vec<NodeId> = m.ids().difference(&antes).copied().collect();
            assert!(!novos.is_empty(), "{id:?}: a lista não abriu");
            let o = m.rect(novos[0]).expect("a opção");
            if cartoes {
                assert!(o.y >= caixa.y - 1.0, "{o:?} × {caixa:?}");
            }
            let a = m.clica(novos[0]);
            set_current_layers(None);
            a
        };
        assert_eq!(manda(false), manda(true), "{id:?}");
    }
}

/// Os números da camada Texture escrevem nos ids fixos de sempre.
#[test]
fn the_texture_layer_numbers_scrub_as_in_the_brush() {
    for id in [
        p::PAINTER_BRUSH_TEXTURE_OFFSET_X,
        p::PAINTER_BRUSH_TEXTURE_SIZE_Y,
        p::PAINTER_BRUSH_TEXTURE_RAMP_STOP_POS,
    ] {
        let (s, _) = com_textura(TextureKind::COUNT - 2, true);
        let mut m = abre(true, s);
        let ev = m.arrasta_a_pega(id, 40.0);
        let accoes = m.aplica(ev);
        set_current_layers(None);
        assert!(!accoes.is_empty(), "{id:?}: o scrub não chegou");
    }
}

/// O que a camada cobre inteira não aparece: Mapping, Rake, ângulo, Depth; e sem padrão, só a lista.
#[test]
fn the_texture_layer_shows_only_what_it_reads() {
    let (s, _) = com_textura(TextureKind::COUNT - 2, true);
    let m = abre(true, s);
    let achado = [
        p::PAINTER_BRUSH_TEXTURE_MAPPING,
        p::PAINTER_BRUSH_TEXTURE_RAKE,
        p::PAINTER_BRUSH_TEXTURE_ANGLE,
        p::PAINTER_BRUSH_GRAIN_DEPTH,
    ]
    .map(|i| m.rect(i).is_some());
    let oferta = m.rect(p::PAINTER_BRUSH_TEXTURE_RAMP_ALPHA_MODE).is_some();
    set_current_layers(None);
    assert_eq!(achado, [false; 4]);
    assert!(oferta, "a camada tem a lista Alpha");
    let (s, _) = com_textura(0, false);
    let m = abre(true, s);
    let (kind, offset) = (
        m.rect(p::PAINTER_BRUSH_TEXTURE_KIND).is_some(),
        m.rect(p::PAINTER_BRUSH_TEXTURE_OFFSET_X).is_some(),
    );
    set_current_layers(None);
    assert_eq!((kind, offset), (true, false));
}

/// Cada ícone sem texto tem dica, e cada letra do Selective Color diz a cor.
/// *Mutação: tirar `set_tooltip` de `registra` ⇒ vermelho.*
#[test]
fn the_icon_only_buttons_and_the_color_letters_have_tips() {
    for (k, ids) in [
        (AdjustmentKind::Curves, Vec::new()),
        (AdjustmentKind::GradientMap, Vec::new()),
        (AdjustmentKind::SelectiveColor, (0..9).collect::<Vec<u8>>()),
    ] {
        let (s, l) = com_ajuste(k);
        let m = abre(true, s);
        let alvos: Vec<NodeId> = match k {
            AdjustmentKind::Curves => vec![
                p::painter_curve_add_id(l.0),
                p::painter_curve_remove_id(l.0),
            ],
            AdjustmentKind::GradientMap => vec![
                p::painter_gradient_add_id(l.0),
                p::painter_gradient_remove_id(l.0),
            ],
            _ => ids
                .iter()
                .map(|i| p::painter_selcolor_bucket_id(l.0, *i))
                .collect(),
        };
        for id in alvos {
            assert!(m.host.store().tooltip_for(id).is_some(), "{k:?}: {id:?}");
        }
        set_current_layers(None);
    }
}

/// No Black & White, a Tint fica entre os pesos e a tinta que ela governa, como no painel de antes. Os números saem
/// redondos: `0.65 * 100.0` apareceria `65.000`. *Mutação: tirar o `mirror_value` ⇒ vermelho.*
#[test]
fn black_and_white_keeps_the_tint_between_its_weights_and_shows_round_numbers() {
    let (s, l) = com_ajuste(AdjustmentKind::BlackAndWhite);
    let m = abre(true, s);
    let tinta = m
        .rect(painter_layer_widget_id(l.0, W::AdjToggle0))
        .expect("a caixa");
    let (_, ultimo_peso) = chips_de(l)[5];
    let peso = m.rect(ultimo_peso).expect("o peso");
    let valores: Vec<f64> = chips_de(l)
        .iter()
        .take(6)
        .map(|(_, c)| m.host.store().number_value(*c).expect("o número"))
        .collect();
    set_current_layers(None);
    assert!(tinta.y >= peso.y + peso.h, "{tinta:?} × {peso:?}");
    for v in valores {
        assert_eq!(v, v.round(), "não é um inteiro: {v}");
    }
}

/// Só a camada Texture ATIVA abre o editor: os ids são fixos, e há uma instância.
/// *Mutação: tirar a condição `stack.active() == Some(id)` ⇒ vermelho.*
#[test]
fn only_the_active_texture_layer_opens_the_editor() {
    let (mut s, t) = com_textura(TextureKind::COUNT - 2, false);
    let base = s.all_ids().find(|i| *i != t).expect("a base");
    s.set_active(base);
    let m = abre(true, s);
    let tem = m.rect(p::PAINTER_BRUSH_TEXTURE_KIND).is_some();
    set_current_layers(None);
    assert!(!tem, "o editor abriu com outra camada ativa");
}

/// O que a camada acrescenta ao corpo, sem as linhas de camada, que têm dentro as células delas.
fn os_dela_em(m: &Mesa) -> BTreeSet<NodeId> {
    let janela = m.janela;
    m.ids()
        .into_iter()
        .filter(|i| {
            m.rect(*i).is_some_and(|r| r.y + r.h <= janela.h)
                && !matches!(
                    m.host.store().panel_row_family(*i),
                    Some(ph2d_editor_core::interaction::PanelRowFamily::PainterLayer)
                )
                && !so_a_base_ids().contains(i)
        })
        .collect()
}

fn so_a_base_ids() -> &'static BTreeSet<NodeId> {
    static BASE: std::sync::OnceLock<BTreeSet<NodeId>> = std::sync::OnceLock::new();
    BASE.get_or_init(|| {
        let mut s = LayerStack::new();
        let _ = s.add_raster("Base", 8, 8).expect("nasce");
        let m = {
            let mut m = Mesa::nova(true, FALLBACK_BRUSH, true);
            set_current_layers(Some(s));
            m.assenta();
            m
        };
        m.ids()
    })
}

/// A lei da compactação em todos os estados: nada cortado, nada grudado, irmãos curtos na mesma linha, nada por cima.
#[test]
fn nothing_is_cut_stuck_apart_or_overlapping_in_any_of_these_layers() {
    for (nome, s, l) in estados() {
        let _ = so_a_base_ids();
        let mut m = abre(true, s);
        let (_, c) = censo::grava(|| {
            let (h, st, j) = (&mut m.host, &mut m.st, m.janela);
            h.paint::<PainterLayersPanel>(st, j)
        });
        assert!(
            c.textos_cortados().is_empty(),
            "{nome}: {:#?}",
            c.textos_cortados()
        );
        assert!(
            c.linhas_grudadas().is_empty(),
            "{nome}: {:#?}",
            c.linhas_grudadas()
        );
        assert!(
            c.irmaos_separados().is_empty(),
            "{nome}: {:#?}",
            c.irmaos_separados()
        );
        m.quadro();
        // Só os controlos desta camada: a linha dela agarra-se inteira, e o cromo não é desta fatia.
        let ids: BTreeSet<NodeId> = os_dela_em(&m)
            .into_iter()
            .filter(|i| *i != painter_layer_widget_id(l.0, W::Row))
            .collect();
        sem_sobrepor(&m, &ids, &nome);
        set_current_layers(None);
    }
}
