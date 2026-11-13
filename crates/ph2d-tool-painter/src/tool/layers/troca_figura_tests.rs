//! B18.3 — a troca da activa sem recompor: a figura que fica na tela depois da troca é, BIT a BIT, a de
//! uma recomposição inteira forçada, em todo estado (camadas escondidas, grupo, modos, opacidade, máscara,
//! filme da proteção, vista cinza, tinta viva do Wet, depois do undo). E nos estados normais o cache fica.

use super::super::*;
use super::cache::TROCA_RECOMPOE_SEMPRE;
use ph2d_editor_core::tool::{CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_painter_effects::adjustments::AdjustmentKind as K;

const LADO: u32 = 48;
const N: usize = (LADO as usize) * (LADO as usize);

fn ruido(semente: u32) -> Vec<u8> {
    let mut s = semente | 1;
    (0..N * 4)
        .map(|_| {
            s ^= s << 13;
            s ^= s >> 17;
            s ^= s << 5;
            (s >> 24) as u8
        })
        .collect()
}

fn relevo(semente: u32) -> Vec<f32> {
    ruido(semente)[..N]
        .iter()
        .map(|&b| f32::from(b) / 64.0)
        .collect()
}

fn cp(x: f32, y: f32, phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos: [x, y],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn traco(t: &mut PainterTool, y: f32) {
    t.set_brush_size_px(6.0);
    t.set_brush_color_srgb8([40, 90, 220]);
    t.on_canvas_pointer(cp(6.0, y, PointerPhase::Down));
    for k in 1..=9 {
        t.on_canvas_pointer(cp(6.0 + 4.0 * k as f32, y, PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(cp(42.0, y, PointerPhase::Up));
    t.on_tick(1000.0 / 60.0);
}

struct Doc {
    t: PainterTool,
    a: RtLayerId,
    b: RtLayerId,
    c: RtLayerId,
}

fn doc(wet: bool) -> Doc {
    let mut t = PainterTool::default();
    if wet {
        t.abre_a_sprite(vec![255u8; N * 4], LADO, LADO);
        t.set_wet_relogio_fixo(true);
    } else {
        t.set_source(vec![255u8; N * 4], LADO, LADO);
    }
    let a = t.add_raster_layer_with_pixels("A", ruido(7)).expect("A");
    let b = t.add_raster_layer_with_pixels("B", ruido(11)).expect("B");
    let c = t.add_raster_layer_with_pixels("C", ruido(13)).expect("C");
    let _ = t.take_preview_arc();
    Doc { t, a, b, c }
}

/// A figura que a tela mostra agora: o que o dreno devolve, ou o cache que ficou quando ele não tem nada.
fn na_tela(t: &mut PainterTool) -> Vec<u8> {
    match t.take_preview_arc() {
        Some((a, _, _)) => a.as_ref().clone(),
        None => t
            .composited
            .as_ref()
            .expect("a tela tem uma figura")
            .as_ref()
            .clone(),
    }
}

fn inteira(t: &mut PainterTool) -> Vec<u8> {
    t.pede_a_composicao_inteira();
    t.take_preview_arc()
        .expect("a recomposição inteira")
        .0
        .as_ref()
        .clone()
}

/// Os píxeis de cada camada (a activa na tela, as outras em `images`) e a activa.
fn planos(t: &PainterTool) -> (Option<RtLayerId>, Vec<(RtLayerId, Vec<u8>)>) {
    let px = t
        .layers
        .all_ids()
        .map(|id| {
            let p = if t.layers.active() == Some(id) {
                t.canvas_rgba.as_ref().clone()
            } else {
                t.images
                    .get(&id)
                    .map(|i| i.rgba8.as_ref().clone())
                    .unwrap_or_default()
            };
            (id, p)
        })
        .collect();
    (t.layers.active(), px)
}

fn confere_a_figura(t: &mut PainterTool, caso: &str) {
    let mostrada = na_tela(t);
    let certa = inteira(t);
    assert!(
        mostrada == certa,
        "{caso}: a figura depois da troca difere da recomposição inteira em {} bytes",
        mostrada.iter().zip(&certa).filter(|(x, y)| x != y).count()
    );
}

/// Troca para `alvo` e confere a figura contra a recomposição inteira; desfaz e refaz a troca, que devolvem
/// os píxeis de cada camada ao byte. Devolve se o cache ficou.
fn troca_e_confere(t: &mut PainterTool, alvo: RtLayerId, caso: &str) -> bool {
    let _ = na_tela(t); // o quadro anterior já drenado
    let antes = t.layers_revision;
    let planos_antes = planos(t);
    t.select_layer(alvo);
    assert_ne!(
        t.layers_revision, antes,
        "{caso}: o painel não soube da troca"
    );
    let ficou = t.composited_is_some() && !t.preview_dirty;
    confere_a_figura(t, caso);
    let planos_depois = planos(t);
    assert!(t.undo_last(), "{caso}: a troca desfaz-se");
    assert!(
        planos(t) == planos_antes,
        "{caso}: o undo da troca não volta ao byte"
    );
    confere_a_figura(t, caso);
    assert!(t.redo_last(), "{caso}: a troca refaz-se");
    assert!(
        planos(t) == planos_depois,
        "{caso}: o redo da troca não volta ao byte"
    );
    confere_a_figura(t, caso);
    ficou
}

#[test]
fn a_troca_sem_recompor_mostra_a_figura_da_recomposicao_inteira() {
    // (caso, preparo, o cache tem de ficar?)
    type Preparo = fn(&mut Doc);
    let casos: Vec<(&str, Preparo, bool)> = vec![
        ("normal", |_| {}, true),
        ("escondida", |d| d.t.set_layer_visible(d.b, false), true),
        (
            "modos e opacidade",
            |d| {
                d.t.set_layer_blend_mode(d.b, BlendMode::Multiply);
                d.t.set_layer_opacity(d.c, 0.4);
                d.t.set_layer_blend_mode(d.a, BlendMode::Screen);
            },
            true,
        ),
        (
            "grupo",
            |d| {
                d.t.select_layer(d.b);
                d.t.group_active().expect("o grupo");
            },
            true,
        ),
        (
            "ajuste",
            |d| {
                d.t.add_adjustment_layer(K::HueSaturationBrightness)
                    .expect("o ajuste");
                d.t.select_layer(d.a);
            },
            true,
        ),
        (
            "máscara",
            |d| {
                d.t.select_layer(d.b);
                d.t.add_mask_to_active().expect("a máscara");
                d.t.set_paint_tool_mode("brush");
                traco(&mut d.t, 20.0);
            },
            true,
        ),
        (
            "filme da proteção",
            |d| {
                d.t.select_layer(d.b);
                d.t.set_paint_tool_mode("mask");
                traco(&mut d.t, 24.0);
                assert!(d.t.mask_scratch_active(), "controlo: o filme vivo");
            },
            false,
        ),
        (
            "vista cinza",
            |d| {
                d.t.select_layer(d.b);
                let m = d.t.add_mask_to_active().expect("a máscara");
                d.t.toggle_mask_view_grayscale(m);
            },
            false,
        ),
        ("papel aceso", |d| d.t.set_substrate_depth(1.0), true),
        (
            "relevo de impasto",
            |d| {
                d.t.set_substrate_depth(1.0);
                d.t.heights.insert(d.b, Arc::new(relevo(3)));
                d.t.covers.insert(d.b, Arc::new(vec![180u8; N]));
                d.t.invalidate_composite();
            },
            true,
        ),
        (
            "depois do undo",
            |d| {
                d.t.select_layer(d.b);
                d.t.set_paint_tool_mode("brush");
                traco(&mut d.t, 30.0);
                assert!(d.t.undo_last());
                traco(&mut d.t, 12.0);
                assert!(d.t.undo_last());
                assert!(d.t.redo_last());
            },
            true,
        ),
    ];
    for (caso, prepara, deve_ficar) in casos {
        let mut d = doc(false);
        prepara(&mut d);
        let (a, b, c) = (d.a, d.b, d.c);
        let mut ficou = Vec::new();
        for alvo in [a, c, b, a, b, c] {
            if d.t.layers.active() == Some(alvo) {
                continue;
            }
            ficou.push(troca_e_confere(&mut d.t, alvo, caso));
        }
        if deve_ficar {
            assert!(
                ficou.iter().all(|&f| f),
                "{caso}: a troca recompôs ({ficou:?})"
            );
        } else {
            assert!(
                ficou.iter().any(|&f| !f),
                "{caso}: o estado que lê a activa não recompôs"
            );
        }
        // Undo/redo da troca também mostram a figura certa.
        assert!(d.t.undo_last(), "{caso}: a troca desfaz-se");
        let mostrada = na_tela(&mut d.t);
        assert!(mostrada == inteira(&mut d.t), "{caso}: o undo da troca");
    }
}

/// A tinta viva do Wet: adormecer escreve a última imagem na tela — a troca recompõe e mostra o que tem de.
#[test]
fn a_troca_com_a_tinta_viva_do_wet_recompoe_e_mostra_a_figura_certa() {
    let mut d = doc(true);
    d.t.select_layer(d.b);
    d.t.set_paint_tool_mode("wetpaint");
    traco(&mut d.t, 24.0);
    assert!(d.t.traco_ou_tinta_viva(), "controlo: a sessão viva");
    let (a, b, c) = (d.a, d.b, d.c);
    for alvo in [a, b, c, b] {
        let _ = troca_e_confere(&mut d.t, alvo, "wet");
        traco(&mut d.t, 10.0 + f32::from(alvo.0 as u8 % 5) * 6.0);
    }
    let _ = troca_e_confere(&mut d.t, a, "wet, depois do último traço");
}

/// N7 — a pista da PLACA drena com `composited = None`: sem sujo nem pista velha, a troca não pede drenagem
/// (o slot fica) e a figura é, bit a bit, a da recomposição inteira de antes. Com e sem o papel aceso; o
/// controlo (`TROCA_RECOMPOE_SEMPRE`) volta a pedir.
#[test]
fn a_troca_na_pista_da_placa_nao_pede_drenagem_e_a_figura_fica() {
    for (papel, antes) in [(false, false), (true, false), (true, true)] {
        let mut d = doc(false);
        if papel {
            d.t.set_substrate_depth(1.0);
            d.t.heights.insert(d.a, Arc::new(relevo(5)));
        }
        let (a, b, c) = (d.a, d.b, d.c);
        for alvo in [a, c, b, a] {
            if d.t.layers.active() == Some(alvo) {
                continue;
            }
            let certa = inteira(&mut d.t);
            d.t.pede_a_composicao_inteira();
            assert!(d.t.take_preview_dirty(), "a placa drena");
            assert!(d.t.composited.is_none() && !d.t.gpu_lane_stale);
            TROCA_RECOMPOE_SEMPRE.with(|x| x.set(antes));
            d.t.select_layer(alvo);
            TROCA_RECOMPOE_SEMPRE.with(|x| x.set(false));
            assert_eq!(
                d.t.preview_dirty, antes,
                "papel={papel} antes={antes}: a troca pediu (ou não) a drenagem da placa"
            );
            assert!(
                inteira(&mut d.t) == certa,
                "papel={papel}: a figura depois da troca não é a de antes"
            );
        }
    }
}
