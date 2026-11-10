//! B48.2: o tique do slider do balde trabalha na REGIÃO do flood, não na tela — e dá o mesmo bit que o
//! refill antigo (guardado aqui verbatim como oráculo).

use super::flood_fill;
use super::nucleo;
use crate::tool::PainterTool;
use crate::tool::paint::PaintMode;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::tool::{CanvasPointer, PanelEvent, PointerPhase, RasterEditTool, Tool};
use ph2d_painter_brush::Falloff;
use std::sync::Arc;
use std::time::{Duration, Instant};

impl PainterTool {
    /// O `refill_from_snapshot` de antes do B48.2, VERBATIM (só os caminhos `super::` reescritos).
    fn refill_antigo(&mut self) {
        let (Some(seed), size) = (self.paint.fill_seed, self.source_size) else {
            return;
        };
        let (w, h) = (size.0 as usize, size.1 as usize);
        if w == 0 || h == 0 {
            return;
        }
        let sx = (seed[0].floor() as i32).clamp(0, w as i32 - 1) as usize;
        let sy = (seed[1].floor() as i32).clamp(0, h as i32 - 1) as usize;
        let tol = (self.paint.fill_threshold.clamp(0.0, 1.0) * 255.0).round() as u8;
        let color = [
            (self.paint.brush.color[0].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            (self.paint.brush.color[1].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            (self.paint.brush.color[2].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
        ];
        let clip = (self.paint.selection_active && self.paint.selection_mask.len() == w * h)
            .then(|| nucleo::componente_da_selecao(&self.paint.selection_mask, sx, sy, w, h));
        let keep_mask = self.fill_selection_keep();
        let mut fill_mask = self.paint.brush.deposits_height().then(|| vec![0u8; w * h]);
        let buf = crate::tool::paint::plane_fork::fork_canvas(
            &mut self.canvas_rgba,
            &self.undo.write_state,
            self.source_size.0,
            None,
        );
        if buf.len() != w * h * 4 {
            return;
        }
        if self.paint.fill_snapshot.len() == buf.len() {
            buf.copy_from_slice(&self.paint.fill_snapshot);
        }
        let filled = flood_fill(buf, w, h, (sx, sy), color, tol, fill_mask.as_deref_mut());
        if let Some(comp) = &clip {
            let mask = &keep_mask;
            let snap = &self.paint.fill_snapshot;
            let n = (w * h).min(snap.len() / 4);
            for i in 0..n {
                let keep = if comp[i] >= 128 {
                    f32::from(mask[i]) / 255.0
                } else {
                    0.0
                };
                if keep >= 1.0 {
                    continue;
                }
                let b = i * 4;
                for c in 0..4 {
                    let painted = f32::from(buf[b + c]);
                    let orig = f32::from(snap[b + c]);
                    buf[b + c] = (painted * keep + orig * (1.0 - keep))
                        .round()
                        .clamp(0.0, 255.0) as u8;
                }
            }
        }
        let mut body = None;
        if let Some(fm) = fill_mask {
            body = self.paint.relief.stroke_relief_bbox;
            self.reset_stroke_height();
            let keep = &keep_mask;
            let cov: Vec<u8> = (0..w * h)
                .map(|i| {
                    let inside = clip.as_ref().is_none_or(|c| c[i] >= 128);
                    if !inside || fm[i] == 0 {
                        return 0;
                    }
                    let k = clip
                        .as_ref()
                        .map_or(255, |_| keep.get(i).copied().unwrap_or(255));
                    (u32::from(fm[i]) * u32::from(k) / 255) as u8
                })
                .collect();
            self.arm_fill_body(&cov);
            body = match (body, self.paint.relief.stroke_relief_bbox) {
                (Some(gone), Some(armed)) => Some(super::super::union_region(gone, armed)),
                (a, b) => a.or(b),
            };
        }
        let colour = match (self.paint.fill_last_rect, filled) {
            (Some(prev), Some(new)) => Some(super::super::union_region(prev, new)),
            (Some(prev), None) => Some(prev),
            (None, new) => new,
        };
        self.paint.fill_last_rect = filled;
        let dirty = match (colour, body) {
            (Some(c), Some(b)) => super::super::union_region(c, b),
            (Some(c), None) => c,
            (None, Some(b)) => b,
            (None, None) => return,
        };
        self.mark_dirty(dirty);
    }
}

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// `w×h` branco, disco vermelho de raio `r` em `c`, uma faixa de ruído (os limiares mudam a região) e,
/// se `sel`, a seleção: disco com Feather de raio `1,3·r` + uma segunda ilha disjunta.
fn tela(w: u32, h: u32, c: [f32; 2], r: f32, impasto: bool, sel: bool) -> PainterTool {
    let (wu, hu) = (w as usize, h as usize);
    let mut src = vec![255u8; wu * hu * 4];
    let mut s = 0x9E37_79B9u32;
    for y in 0..hu {
        for x in 0..wu {
            let d = ((x as f32 + 0.5 - c[0]).powi(2) + (y as f32 + 0.5 - c[1]).powi(2)).sqrt();
            let i = (y * wu + x) * 4;
            if d <= r {
                src[i..i + 4].copy_from_slice(&[220, 20, 20, 255]);
            } else if (y * 7 / hu.max(1)) == 3 {
                s ^= s << 13;
                s ^= s >> 17;
                s ^= s << 5;
                let v = 150 + (s % 100) as u8;
                src[i..i + 4].copy_from_slice(&[v, v, 255 - v / 2, 255]);
            }
        }
    }
    let mut t = PainterTool::default();
    t.set_source(src, w, h);
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "fill".to_string(),
    ));
    assert!(matches!(t.paint.paint_mode, PaintMode::Fill));
    t.paint.brush.color = [0.1, 0.7, 0.2];
    t.paint.brush.radius_px = 12.0;
    t.paint.brush.hardness = 0.0;
    if impasto {
        t.set_brush_impasto(true);
        t.set_brush_impasto_depth(1.0);
        t.set_brush_falloff(Falloff::Smooth as u8);
    }
    if sel {
        let mut m = vec![0u8; wu * hu];
        let rs = r * 1.3;
        for y in 0..hu {
            for x in 0..wu {
                let d = ((x as f32 + 0.5 - c[0]).powi(2) + (y as f32 + 0.5 - c[1]).powi(2)).sqrt();
                let feather = ((rs - d) / 6.0).clamp(0.0, 1.0);
                m[y * wu + x] = (feather * 255.0).round() as u8;
                if x < wu / 8 && y < hu / 8 {
                    m[y * wu + x] = 255; // a ilha disjunta: o balde não a pinta
                }
            }
        }
        t.paint.selection_mask = Arc::new(m);
        t.paint.selection_active = true;
    }
    t
}

fn solta(t: &mut PainterTool, pos: [f32; 2], antigo: bool) {
    assert!(t.fill_pointer(cp(pos, PointerPhase::Down)));
    if antigo {
        t.paint.fill_seed = Some(pos);
        t.refill_antigo();
    } else {
        assert!(t.fill_pointer(cp(pos, PointerPhase::Up)));
    }
}

fn tique(t: &mut PainterTool, thr: f32, antigo: bool) {
    if antigo {
        t.paint.fill_threshold = thr.clamp(0.0, 1.0);
        t.refill_antigo();
    } else {
        t.set_fill_threshold(thr);
    }
}

fn bits(v: &[f32]) -> Vec<u32> {
    v.iter().map(|x| x.to_bits()).collect()
}

fn mesmo_estado(a: &PainterTool, b: &PainterTool, ctx: &str) {
    assert!(a.canvas_rgba[..] == b.canvas_rgba[..], "{ctx}: pixels");
    let (ra, rb) = (&a.paint.relief, &b.paint.relief);
    assert_eq!(
        bits(&ra.stroke_paint),
        bits(&rb.stroke_paint),
        "{ctx}: paint"
    );
    assert_eq!(
        bits(&ra.stroke_height),
        bits(&rb.stroke_height),
        "{ctx}: height"
    );
    assert_eq!(
        bits(&ra.stroke_radius),
        bits(&rb.stroke_radius),
        "{ctx}: radius"
    );
    assert_eq!(ra.stroke_grain, rb.stroke_grain, "{ctx}: grain");
    assert_eq!(ra.stroke_film, rb.stroke_film, "{ctx}: film");
    assert_eq!(ra.stroke_relief_bbox, rb.stroke_relief_bbox, "{ctx}: bbox");
    assert_eq!(
        a.paint.fill_last_rect, b.paint.fill_last_rect,
        "{ctx}: last"
    );
    assert_eq!(a.marks, b.marks, "{ctx}: marks");
    assert_eq!(a.dirty_rect, b.dirty_rect, "{ctx}: dirty");
}

/// **Paridade ao bit**: digital/impasto × sem/com seleção, 6 limiares (sobe e encolhe), o Done — pixels,
/// os 5 planos, a bbox, as marcas de sujo e o relevo commitado.
#[test]
fn o_tique_do_balde_e_o_antigo_ao_bit() {
    let (w, h, c, r) = (97u32, 83u32, [48.5, 40.0], 22.0);
    let mut circulou = 0;
    for (impasto, sel) in [(false, false), (false, true), (true, false), (true, true)] {
        for semente in [c, [5.0, 5.0], [90.0, 41.0]] {
            let mut a = tela(w, h, c, r, impasto, sel);
            let mut b = tela(w, h, c, r, impasto, sel);
            solta(&mut a, semente, false);
            solta(&mut b, semente, true);
            let ctx = format!("impasto={impasto} sel={sel} semente={semente:?}");
            mesmo_estado(&a, &b, &format!("{ctx} drop"));
            for thr in [0.0, 0.08, 0.3, 0.6, 1.0, 0.15] {
                let antes = a.paint.relief.stroke_paint.as_ptr();
                let armado = !a.paint.relief.stroke_paint.is_empty();
                tique(&mut a, thr, false);
                tique(&mut b, thr, true);
                mesmo_estado(&a, &b, &format!("{ctx} thr={thr}"));
                if armado && !a.paint.relief.stroke_paint.is_empty() {
                    assert_eq!(
                        antes,
                        a.paint.relief.stroke_paint.as_ptr(),
                        "{ctx}: os planos não circularam"
                    );
                    circulou += 1;
                }
            }
            a.fill_commit();
            b.fill_commit();
            mesmo_estado(&a, &b, &format!("{ctx} done"));
            let la = a.layers.active().unwrap();
            assert_eq!(
                a.heights.get(&la).map(|v| bits(v)),
                b.heights.get(&la).map(|v| bits(v)),
                "{ctx}: relevo commitado"
            );
        }
    }
    assert!(
        circulou >= 10,
        "os planos do corpo quase nunca circularam: {circulou}"
    );
}

/// O tique do slider, por variante, à tela `s²` com um disco de 300 px e a seleção (Impasto): o MÍNIMO
/// de 7 tiques intercalados, cada variante na SUA ferramenta (o antigo troca os planos a cada tique, o
/// que desligaria a circulação do novo).
fn tempos(s: u32) -> (Duration, Duration) {
    let c = [s as f32 * 0.5, s as f32 * 0.5];
    let mut novo_t = tela(s, s, c, 150.0, true, true);
    let mut antigo_t = tela(s, s, c, 150.0, true, true);
    solta(&mut novo_t, c, false);
    solta(&mut antigo_t, c, true);
    let (mut novo, mut antigo) = (Duration::MAX, Duration::MAX);
    for k in 0..7 {
        let thr = if k % 2 == 0 { 0.1 } else { 0.2 };
        let t0 = Instant::now();
        tique(&mut novo_t, thr, false);
        novo = novo.min(t0.elapsed());
        let t0 = Instant::now();
        tique(&mut antigo_t, thr, true);
        antigo = antigo.min(t0.elapsed());
    }
    (novo, antigo)
}

/// **O tique custa pela REGIÃO, não pela tela** (B48.2): `t(4096)/t(2048) < 1,5` com o mesmo disco de
/// 300 px. O antigo, no MESMO processo, mostra o fenómeno (cresce ≥ 1,5× com a tela), e o novo a 4K é
/// ≥ 10× o antigo.
#[test]
fn o_tique_do_balde_custa_pela_regiao_nao_pela_tela() {
    let (n2, a2) = tempos(2048);
    let (n4, a4) = tempos(4096);
    let rn = n4.as_secs_f64() / n2.as_secs_f64();
    let ra = a4.as_secs_f64() / a2.as_secs_f64();
    eprintln!(
        "B48.2 tique: novo 2048 {n2:?} 4096 {n4:?} (×{rn:.2}) · antigo 2048 {a2:?} 4096 {a4:?} (×{ra:.2})"
    );
    assert!(ra >= 1.5, "a régua não vê o fenómeno no antigo: ×{ra:.2}");
    assert!(rn < 1.5, "o tique ainda cresce com a tela: ×{rn:.2}");
    assert!(
        n4 * 10 <= a4,
        "o tique a 4K não é 10× o antigo: {n4:?} × {a4:?}"
    );
}
