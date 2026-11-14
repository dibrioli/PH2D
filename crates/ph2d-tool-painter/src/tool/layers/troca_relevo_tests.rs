//! n.º 9 item 3 — a troca da activa com a tinta viva do Wet muda a figura e NÃO o relevo: a pista GPU recebe a
//! parte do sujo que pode mudá-lo (`preview_gpu_relevo`), e fora dela os planos da dobra são os mesmos ao bit.

use super::super::*;
use ph2d_editor_core::tool::{CanvasPointer, PointerPhase, RasterEditTool};

const LADO: u32 = 64;
const N: usize = (LADO as usize) * (LADO as usize);

fn cp(x: f32, y: f32, phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos: [x, y],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn traco(t: &mut PainterTool, y: f32) {
    t.set_brush_size_px(8.0);
    t.set_brush_color_srgb8([40, 90, 220]);
    t.on_canvas_pointer(cp(8.0, y, PointerPhase::Down));
    for k in 1..=10 {
        t.on_canvas_pointer(cp(8.0 + 4.5 * k as f32, y, PointerPhase::Move));
        ph2d_editor_core::Tool::on_tick(t, 1000.0 / 60.0);
    }
    t.on_canvas_pointer(cp(54.0, y, PointerPhase::Up));
    ph2d_editor_core::Tool::on_tick(t, 1000.0 / 60.0);
}

/// Os cinco planos da dobra da tela inteira, texel a texel (`relevo, cobertura, mat0, mat1, papel`).
fn planos(t: &PainterTool) -> Vec<Vec<u8>> {
    let p = t.impasto_gpu_planes_in((0, 0, LADO, LADO)).expect("planos");
    let por_texel = |b: &[u8]| -> Vec<u8> { b.to_vec() };
    vec![
        por_texel(bytemuck::cast_slice(&p.relief)),
        por_texel(&p.cover),
        por_texel(&p.mat0),
        por_texel(&p.mat1),
        por_texel(bytemuck::cast_slice(&p.papel)),
    ]
}

/// Os texels que diferem entre `a` e `b`, em qualquer plano.
fn diferem(a: &[Vec<u8>], b: &[Vec<u8>]) -> Vec<(u32, u32)> {
    let mut out = Vec::new();
    for i in 0..N {
        let muda = a.iter().zip(b).any(|(x, y)| {
            let bpp = x.len() / N;
            x[i * bpp..(i + 1) * bpp] != y[i * bpp..(i + 1) * bpp]
        });
        if muda {
            out.push(((i % LADO as usize) as u32, (i / LADO as usize) as u32));
        }
    }
    out
}

/// **A troca com a tinta viva diz à placa onde o relevo pode ter mudado, e fora disso ele não mudou.** Em cada
/// troca (e com um traço por drenar antes ou depois dela) os planos da dobra antes e depois diferem só dentro da
/// parte do relevo; sem traço ela é vazia (`Some(None)`). Controlo: a cura disparou nas trocas sem traço.
#[test]
fn a_troca_com_a_tinta_viva_muda_o_relevo_so_onde_a_placa_o_redobra() {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; N * 4], LADO, LADO);
    t.set_wet_relogio_fixo(true);
    let a = t.add_raster_layer("A").expect("A");
    let b = t.add_raster_layer("B").expect("B");
    t.set_substrate_depth(1.0);
    t.set_paint_tool_mode("wetpaint");
    t.select_layer(a);
    traco(&mut t, 20.0);
    t.select_layer(b);
    traco(&mut t, 40.0);
    let mut vazias = 0;
    // (alvo, traço antes da troca, traço depois)
    for (k, (alvo, antes, depois)) in [
        (a, None, None),
        (b, None, None),
        (a, Some(30.0), None),
        (b, None, Some(50.0)),
        (a, None, None),
    ]
    .into_iter()
    .enumerate()
    {
        let _ = t.take_preview_dirty();
        let velhos = planos(&t);
        if let Some(y) = antes {
            traco(&mut t, y);
        }
        assert!(t.tinta_viva(), "passo {k}: controlo, a sessão viva");
        t.select_layer(alvo);
        if let Some(y) = depois {
            traco(&mut t, y);
        }
        if antes.is_none() && depois.is_none() {
            // n.º 10 F5: a troca pura não muda píxel nem relevo — nada a drenar (`troca_pura_tests`).
            assert!(!t.take_preview_dirty(), "passo {k}: a troca pura não marca nada");
            assert_eq!(planos(&t), velhos, "passo {k}: o relevo ficou");
            vazias += 1;
            continue;
        }
        assert!(t.take_preview_dirty(), "passo {k}: há o que drenar");
        let regiao = t.preview_gpu_region().expect("a troca confina");
        let relevo = t.preview_gpu_relevo();
        let novos = planos(&t);
        let fora: Vec<_> = diferem(&velhos, &novos)
            .into_iter()
            .filter(|&(x, y)| match relevo {
                None => {
                    !(x >= regiao.0
                        && y >= regiao.1
                        && x < regiao.0 + regiao.2
                        && y < regiao.1 + regiao.3)
                }
                Some(None) => true,
                Some(Some((rx, ry, rw, rh))) => !(x >= rx && y >= ry && x < rx + rw && y < ry + rh),
            })
            .collect();
        assert!(
            fora.is_empty(),
            "passo {k}: o relevo mudou fora da parte declarada {relevo:?} em {} texels (1.º {:?})",
            fora.len(),
            fora.first()
        );
        assert!(
            !matches!(relevo, Some(None)),
            "passo {k}: o traço redobra o relevo dele ({relevo:?})"
        );
    }
    assert_eq!(vazias, 3, "controlo: as trocas sem traço");
}
