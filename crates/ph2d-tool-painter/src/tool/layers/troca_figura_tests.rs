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

/// N7 — o Watercolor no *Textured Paper*: o papel de cor põe o documento na pista da CPU e a aguada deixa
/// vidro; a composição pelo vidro é função dos bytes e do vidro de cada camada, nunca da activa. A troca não
/// recompõe e a figura é, bit a bit, a da recomposição inteira (undo/redo ao byte); o controlo
/// (`TROCA_RECOMPOE_SEMPRE`) recompõe e dá a mesma figura.
#[test]
fn a_troca_com_a_aguada_no_papel_de_cor_nao_recompoe_e_a_figura_fica() {
    use crate::tool::paint::media::PaintMedia;
    for antes in [false, true] {
        let mut t = PainterTool::default();
        t.agenda_papel_texturizado(7);
        t.abre_a_sprite_nova(7, vec![255u8; N * 4], LADO, LADO);
        let a = t.add_raster_layer("A").expect("A");
        t.set_paint_tool_mode("brush");
        traco(&mut t, 12.0);
        let b = t.add_raster_layer("B").expect("B");
        t.set_paint_media(PaintMedia::Watercolor);
        crate::tool::paint::tests::aquarela_neutra(&mut t);
        traco(&mut t, 24.0);
        t.dry_session_now();
        let c = t.add_raster_layer("C").expect("C");
        traco(&mut t, 30.0);
        assert!(
            t.papel_pede_a_cpu() && t.papel_atravessa_vidro(),
            "controlo: papel de cor com vidro"
        );
        let _ = na_tela(&mut t);
        TROCA_RECOMPOE_SEMPRE.with(|x| x.set(antes));
        let mut ficou = Vec::new();
        for alvo in [a, b, c, a, b] {
            if t.layers.active() == Some(alvo) {
                continue;
            }
            ficou.push(troca_e_confere(&mut t, alvo, "aguada no papel de cor"));
        }
        TROCA_RECOMPOE_SEMPRE.with(|x| x.set(false));
        if antes {
            assert!(ficou.iter().all(|&f| !f), "controlo: recompõe ({ficou:?})");
        } else {
            assert!(ficou.iter().all(|&f| f), "a troca recompôs ({ficou:?})");
        }
    }
}

/// N7 — apagar, duplicar e agrupar mudam a figura só onde a camada tem tinta: a recomposição fica confinada à
/// janela dela (o cache fica fora), e a figura é, bit a bit, a da recomposição inteira — com modos e opacidade,
/// o papel aceso e a aguada sobre o papel de cor; undo/redo também. O controlo (`TROCA_RECOMPOE_SEMPRE`)
/// recompõe tudo e dá a mesma figura; o ajuste na pilha e o vidro (a aguada no papel de cor) não são provados e recompõem tudo.
#[test]
fn apagar_duplicar_e_agrupar_recompoem_so_a_janela_da_tinta() {
    use crate::tool::paint::media::PaintMedia;
    type Gesto = fn(&mut PainterTool, RtLayerId);
    let gestos: [(&str, Gesto); 3] = [
        ("apagar", |t, b| assert!(t.delete_layer(b))),
        ("duplicar", |t, b| {
            t.duplicate_layer(b).expect("a cópia");
        }),
        ("grupo", |t, b| {
            t.select_layer(b);
            t.group_active().expect("o grupo");
        }),
    ];
    // (caso, papel de cor + aguada, modos, ajuste, o cache fica?)
    let casos = [
        ("normal", false, false, false, true),
        ("modos e opacidade", false, true, false, true),
        ("aguada no papel de cor", true, true, false, false),
        ("ajuste", false, false, true, false),
    ];
    for (gesto, faz) in gestos {
        for (caso, aguada, modos, ajuste, deve_ficar) in casos {
            for antes in [false, true] {
                let mut t = PainterTool::default();
                if aguada {
                    t.agenda_papel_texturizado(7);
                    t.abre_a_sprite_nova(7, vec![255u8; N * 4], LADO, LADO);
                } else {
                    t.set_source(ruido(5), LADO, LADO);
                }
                let a = t.add_raster_layer("A").expect("A");
                t.set_paint_tool_mode("brush");
                traco(&mut t, 12.0);
                let b = t.add_raster_layer("B").expect("B");
                if aguada {
                    t.set_paint_media(PaintMedia::Watercolor);
                    crate::tool::paint::tests::aquarela_neutra(&mut t);
                }
                traco(&mut t, 20.0);
                t.dry_session_now();
                if modos {
                    t.set_layer_blend_mode(b, BlendMode::Multiply);
                    t.set_layer_opacity(b, 0.6);
                    t.set_layer_blend_mode(a, BlendMode::Screen);
                }
                if ajuste {
                    t.add_adjustment_layer(K::HueSaturationBrightness)
                        .expect("o ajuste");
                }
                t.select_layer(a);
                let _ = na_tela(&mut t);
                let rotulo = format!("{gesto} · {caso} · antes={antes}");
                TROCA_RECOMPOE_SEMPRE.with(|x| x.set(antes));
                faz(&mut t, b);
                TROCA_RECOMPOE_SEMPRE.with(|x| x.set(false));
                let ficou = t.composited_is_some();
                assert_eq!(
                    ficou,
                    deve_ficar && !antes,
                    "{rotulo}: o cache ficou (ou não)"
                );
                if ficou {
                    let r = t.dirty_rect.expect("a janela da tinta");
                    assert!(
                        r.w < LADO || r.h < LADO,
                        "{rotulo}: a janela é a tela inteira ({r:?})"
                    );
                }
                confere_a_figura(&mut t, &rotulo);
                assert!(t.undo_last(), "{rotulo}: desfaz");
                confere_a_figura(&mut t, &format!("{rotulo} · undo"));
                assert!(t.redo_last(), "{rotulo}: refaz");
                confere_a_figura(&mut t, &format!("{rotulo} · redo"));
            }
        }
    }
}

/// N7 onda 3 — a troca com a tinta viva do Wet: a sessão adormece (e a do alvo acorda) e a figura muda só na janela
/// da tinta delas; o cache fica fora dela e a figura é, bit a bit, a da recomposição inteira (undo/redo também), com
/// e sem o papel aceso. O controlo (`TROCA_RECOMPOE_SEMPRE`) recompõe tudo e dá a mesma figura.
#[test]
fn a_troca_com_a_tinta_viva_do_wet_marca_so_a_janela_da_tinta() {
    for (papel, antes) in [(false, false), (true, false), (false, true)] {
        // Tinta só numa faixa de cada camada: a janela não é a tela inteira.
        let faixa = |semente: u32, y0: usize| -> Vec<u8> {
            let mut px = ruido(semente);
            let l = LADO as usize * 4;
            for (y, linha) in px.chunks_mut(l).enumerate() {
                if !(y0..y0 + 6).contains(&y) {
                    linha.fill(0);
                }
            }
            px
        };
        let mut t = PainterTool::default();
        t.abre_a_sprite(vec![255u8; N * 4], LADO, LADO);
        t.set_wet_relogio_fixo(true);
        let a = t.add_raster_layer_with_pixels("A", faixa(7, 2)).expect("A");
        let b = t
            .add_raster_layer_with_pixels("B", faixa(11, 30))
            .expect("B");
        let c = t
            .add_raster_layer_with_pixels("C", faixa(13, 40))
            .expect("C");
        let _ = t.take_preview_arc();
        let mut d = Doc { t, a, b, c };
        if papel {
            d.t.set_substrate_depth(1.0);
        }
        d.t.select_layer(d.b);
        d.t.set_paint_tool_mode("wetpaint");
        traco(&mut d.t, 24.0);
        assert!(d.t.tinta_viva(), "controlo: a sessão viva");
        let (a, b, c) = (d.a, d.b, d.c);
        for (k, alvo) in [a, b, c, b, a].into_iter().enumerate() {
            let caso = format!("papel={papel} antes={antes} passo {k}");
            let _ = na_tela(&mut d.t);
            let viva = d.t.tinta_viva();
            TROCA_RECOMPOE_SEMPRE.with(|x| x.set(antes));
            d.t.select_layer(alvo);
            TROCA_RECOMPOE_SEMPRE.with(|x| x.set(false));
            let ficou = d.t.composited_is_some();
            assert_eq!(ficou, !antes, "{caso}: o cache ficou (ou não); viva={viva}");
            if ficou && d.t.preview_dirty {
                let r = d.t.dirty_rect.expect("a janela da tinta");
                assert!(
                    r.w < LADO || r.h < LADO,
                    "{caso}: a janela é a tela inteira ({r:?})"
                );
            }
            confere_a_figura(&mut d.t, &caso);
            assert!(d.t.undo_last(), "{caso}: desfaz");
            confere_a_figura(&mut d.t, &format!("{caso} · undo"));
            assert!(d.t.redo_last(), "{caso}: refaz");
            confere_a_figura(&mut d.t, &format!("{caso} · redo"));
            traco(&mut d.t, 10.0 + 6.0 * k as f32);
        }
    }
}

/// Uma mudança de píxeis ainda por drenar na activa (a simulação viva do Wet marca a sua a ~40 imagens/s): escreve
/// uma faixa e marca-a. Devolve a região.
fn sujo_pendente(t: &mut PainterTool, k: u8) -> crate::compositor::Region {
    let r = crate::compositor::Region {
        x: 4,
        y: 8 + u32::from(k % 4) * 6,
        w: 30,
        h: 4,
    };
    let l = LADO as usize * 4;
    let px = Arc::make_mut(&mut t.canvas_rgba);
    for y in r.y..r.y + r.h {
        let i = y as usize * l + r.x as usize * 4;
        for p in px[i..i + r.w as usize * 4].chunks_mut(4) {
            p.copy_from_slice(&[200, 30 + k, 60, 255]);
        }
    }
    t.mark_dirty(r);
    r
}

/// N7 onda 4 — a troca com um sujo CONFINADO pendente (no app a simulação viva do Wet deixa-o quase sempre no
/// clique): o cache fica, o sujo continua o da mudança (nunca a tela inteira), e a figura mostrada é, bit a bit, a
/// da recomposição inteira (undo/redo também) — pista CPU com e sem a tinta viva, e pista da placa (a reivindicação
/// fica confinada). O controlo (`TROCA_RECOMPOE_SEMPRE`) recompõe tudo e dá a mesma figura.
#[test]
fn a_troca_com_um_sujo_pendente_nao_recompoe_e_a_figura_fica() {
    for (wet, antes) in [(false, false), (true, false), (false, true), (true, true)] {
        let mut d = doc(wet);
        if wet {
            d.t.select_layer(d.b);
            d.t.set_paint_tool_mode("wetpaint");
            traco(&mut d.t, 24.0);
            assert!(d.t.tinta_viva(), "controlo: a sessão viva");
        }
        let (a, b, c) = (d.a, d.b, d.c);
        for (k, alvo) in [a, c, b, a].into_iter().enumerate() {
            if d.t.layers.active() == Some(alvo) {
                continue;
            }
            let caso = format!("wet={wet} antes={antes} passo {k}");
            let _ = na_tela(&mut d.t);
            let r = sujo_pendente(&mut d.t, k as u8);
            TROCA_RECOMPOE_SEMPRE.with(|x| x.set(antes));
            d.t.select_layer(alvo);
            TROCA_RECOMPOE_SEMPRE.with(|x| x.set(false));
            assert_eq!(
                d.t.composited_is_some(),
                !antes,
                "{caso}: o cache ficou (ou não)"
            );
            if !antes {
                let s = d.t.dirty_rect.expect("o sujo pendente");
                // Com a tinta viva a janela da que adormece soma-se (o ruído da fixture cobre a tela inteira).
                assert!(
                    wet || s.w < LADO || s.h < LADO,
                    "{caso}: o sujo virou a tela inteira ({s:?})"
                );
                assert!(
                    s.x <= r.x && s.y <= r.y && s.x + s.w >= r.x + r.w && s.y + s.h >= r.y + r.h,
                    "{caso}: o sujo pendente perdeu-se ({s:?} ⊉ {r:?})"
                );
            }
            confere_a_figura(&mut d.t, &caso);
            assert!(d.t.undo_last(), "{caso}: desfaz");
            confere_a_figura(&mut d.t, &format!("{caso} · undo"));
            assert!(d.t.redo_last(), "{caso}: refaz");
            confere_a_figura(&mut d.t, &format!("{caso} · redo"));
        }
    }
    // A pista da placa: o slot é a figura fora do sujo; a troca não a declara velha e a reivindicação fica a do sujo.
    for antes in [false, true] {
        let mut d = doc(false);
        let (a, c) = (d.a, d.c);
        for (k, alvo) in [a, c].into_iter().enumerate() {
            d.t.pede_a_composicao_inteira();
            assert!(d.t.take_preview_dirty(), "a placa drena");
            let r = sujo_pendente(&mut d.t, k as u8);
            TROCA_RECOMPOE_SEMPRE.with(|x| x.set(antes));
            d.t.select_layer(alvo);
            TROCA_RECOMPOE_SEMPRE.with(|x| x.set(false));
            assert_eq!(
                d.t.gpu_lane_stale, antes,
                "placa antes={antes}: a pista ficou velha (ou não)"
            );
            assert!(d.t.take_preview_dirty(), "placa: o sujo pendente drena");
            let claim = d.t.preview_gpu_region();
            if antes {
                assert_eq!(claim, None, "controlo: a placa refaz inteira");
            } else {
                assert_eq!(
                    claim,
                    Some((r.x, r.y, r.w, r.h)),
                    "placa: a reivindicação é a do sujo"
                );
            }
        }
    }
}

/// N7 onda 5 — apagar, duplicar e agrupar com a tinta viva do Wet: a que adormece (de outra camada) ou acorda (na
/// nova activa) marca só a janela da tinta dela; o cache fica e a figura é, bit a bit, a da recomposição inteira
/// (undo/redo também). A placa recebe `(antes, depois)`. O controlo (`TROCA_RECOMPOE_SEMPRE`) recompõe tudo.
#[test]
fn apagar_duplicar_e_agrupar_com_a_tinta_viva_do_wet_marcam_so_a_janela() {
    type Gesto = fn(&mut PainterTool, RtLayerId, RtLayerId);
    let gestos: [(&str, Gesto); 4] = [
        ("apagar a viva", |t, b, _| assert!(t.delete_layer(b))),
        ("duplicar a viva", |t, b, _| {
            t.duplicate_layer(b).expect("a cópia");
        }),
        ("duplicar outra", |t, _, c| {
            t.duplicate_layer(c).expect("a cópia");
        }),
        ("grupo", |t, _, _| {
            t.group_active().expect("o grupo");
        }),
    ];
    let faixa = |semente: u32, y0: usize| -> Vec<u8> {
        let mut px = ruido(semente);
        for (y, linha) in px.chunks_mut(LADO as usize * 4).enumerate() {
            if !(y0..y0 + 6).contains(&y) {
                linha.fill(0);
            }
        }
        px
    };
    for (gesto, faz) in gestos {
        for antes in [false, true] {
            let mut t = PainterTool::default();
            t.abre_a_sprite(vec![255u8; N * 4], LADO, LADO);
            t.set_wet_relogio_fixo(true);
            t.add_raster_layer_with_pixels("A", faixa(7, 2)).expect("A");
            let b = t
                .add_raster_layer_with_pixels("B", faixa(11, 30))
                .expect("B");
            let c = t
                .add_raster_layer_with_pixels("C", faixa(13, 40))
                .expect("C");
            t.select_layer(b);
            t.set_paint_tool_mode("wetpaint");
            traco(&mut t, 24.0);
            assert!(t.tinta_viva(), "controlo: a sessão viva");
            let _ = na_tela(&mut t);
            let caso = format!("{gesto} · antes={antes}");
            TROCA_RECOMPOE_SEMPRE.with(|x| x.set(antes));
            faz(&mut t, b, c);
            TROCA_RECOMPOE_SEMPRE.with(|x| x.set(false));
            let ficou = t.composited_is_some();
            assert_eq!(ficou, !antes, "{caso}: o cache ficou (ou não)");
            assert_eq!(
                t.take_estrutura_confinada().is_some(),
                !antes,
                "{caso}: a placa recebe a lista (ou não)"
            );
            if ficou && t.preview_dirty {
                let r = t.dirty_rect.expect("a janela da tinta");
                assert!(
                    r.w < LADO || r.h < LADO,
                    "{caso}: a janela é a tela inteira ({r:?})"
                );
            }
            confere_a_figura(&mut t, &caso);
            assert!(t.undo_last(), "{caso}: desfaz");
            confere_a_figura(&mut t, &format!("{caso} · undo"));
            assert!(t.redo_last(), "{caso}: refaz");
            confere_a_figura(&mut t, &format!("{caso} · redo"));
        }
    }
}
