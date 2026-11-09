//! **O gesto do preview** — o S, a pressão e a água até parar, e a redução ao painel; filho de
//! [`super`] (o preview), partido pelo tecto de LOC (movimento puro).

use super::*;

/// `(x0, x1, cy, amplitude)` das tintas por baixo e do S de quem só as mexe, na tela `lw × lh`: centrado, como a
/// miniatura (sem água a escorrer não há terço de baixo a guardar).
pub(super) fn trajeto_das_tintas(lw: u32, lh: u32) -> (f32, f32, f32, f32) {
    let (l, a) = (lw as f32, lh as f32);
    (0.08 * l, 0.92 * l, 0.5 * a, 0.22 * a)
}

/// `(x0, x1, cy, amplitude)` do S do gesto em `t`, na tela `lw × lh`.
pub(super) fn trajeto(t: &PainterTool, lw: u32, lh: u32) -> (f32, f32, f32, f32) {
    let d = (t.paint.brush.radius_px * 2.0).max(1.0);
    let (l, a) = (lw as f32, lh as f32);
    let margem = d.max(24.0).min(l * 0.25);
    // O S no terço de cima: por baixo cabem ~2,9 diâmetros de fios (medido na tela 1:1, Tilt no máximo,
    // água 1: Wet Round 0,7–0,8 d, Wet Wash 2,4–2,8 d — `medida_preview::sonda_a_tela_escorre`).
    (
        margem,
        l - margem,
        a * 0.22,
        (0.75 * d).clamp(16.0, 0.1 * a),
    )
}

/// A tela do preview exato para um pincel de raio `raio_px`: `10 ×` o diâmetro, 2:1.
pub(super) fn tela_do_preview(raio_px: f32) -> (u32, u32) {
    let d = (raio_px * 2.0).max(1.0);
    let lw = ((10.0 * d).clamp(LARGURA_MIN, LARGURA_MAX) as u32) & !1;
    (lw, lw / 2)
}

/// **O gesto do preview** sobre `t`, cuja tela (`lw × lh`) já está posta: o S com a pressão a subir e a
/// descer ([`super::pressao`]) e a água até parar. É o MESMO gesto que a régua da paridade corre sobre a tela de
/// um pincel armado à mão (`preview_paridade_tests`): o preview é a tela, nunca uma cópia dela.
pub(super) fn gesto_do_preview(
    t: &mut PainterTool,
    lw: u32,
    lh: u32,
    deve_parar: &dyn Fn() -> bool,
    emite: &mut dyn FnMut(&[u8], u32, u32),
) -> bool {
    let rota = trajeto(t, lw, lh);
    gesto_no_trajeto(t, rota, deve_parar, emite)
}

/// [`gesto_do_preview`] no trajeto `(x0, x1, cy, amplitude)` dado.
pub(super) fn gesto_no_trajeto(
    t: &mut PainterTool,
    (x0, x1, cy, amp): (f32, f32, f32, f32),
    deve_parar: &dyn Fn() -> bool,
    emite: &mut dyn FnMut(&[u8], u32, u32),
) -> bool {
    let n = ((x1 - x0) / PASSO_PX).ceil() as u32;
    let cada = n.div_ceil(QUADROS_DO_TRACO).max(1);
    let em = |f: f32| {
        [
            x0 + (x1 - x0) * f,
            cy + amp * (f * std::f32::consts::TAU).det_sin(),
        ]
    };
    let mut emite_se = |t: &mut PainterTool| {
        if let Some((px, w, h)) = quadro(t) {
            let r = reduz(&px, w, h, LARGURA_DO_PREVIEW, ALTURA_DO_PREVIEW);
            emite(&r, LARGURA_DO_PREVIEW, ALTURA_DO_PREVIEW);
        }
    };
    // U4 (auditoria W3b): o Óleo Real (o material do pincel) só no quadro FINAL, com a água assente — medido no gate
    // `cada_knob_do_material_muda_o_preview` (perfil smoke): o óleo em cada quadro custava 85 s por preview × 0,7 s.
    // O Real Oil é do traço (28/10): o traço pousa-o; o composite adia a óptica até ao quadro final.
    let oleo = t.paint.wetpaint.oleo_real.is_some().then_some(());
    t.paint.wetpaint.oleo_adiado = oleo.is_some();
    t.on_canvas_pointer(ponto(em(0.0), pressao(0.0), PointerPhase::Down));
    for k in 1..=n {
        if deve_parar() {
            return false;
        }
        let f = k as f32 / n as f32;
        t.on_canvas_pointer(ponto(em(f), pressao(f), PointerPhase::Move));
        t.on_tick(TICK_MS);
        if k % cada == 0 {
            emite_se(t);
        }
    }
    t.on_canvas_pointer(ponto(em(1.0), pressao(1.0), PointerPhase::Up));
    for k in 1..=AGUA_MAX_TICKS {
        if deve_parar() {
            return false;
        }
        t.on_tick(1000.0 / 60.0);
        let parou = t.paint.wetpaint.session.as_mut().is_none_or(|s| {
            s.bring_home();
            !s.engine.active_grid().has_fluid
        }) || k == AGUA_MAX_TICKS;
        if parou && oleo.is_some() {
            t.paint.wetpaint.oleo_adiado = false;
            t.wet_recomposite_full();
        }
        if parou || k % TICKS_POR_QUADRO_AGUA == 0 {
            emite_se(t);
        }
        if parou {
            break;
        }
    }
    true
}

/// Reduz `px` (`w×h`) a `ow×oh` pela MÉDIA da área de cada pixel de saída (o painel mostra a obra
/// inteira; o vizinho mais próximo perderia os fios finos).
pub(super) fn reduz(px: &[u8], w: u32, h: u32, ow: u32, oh: u32) -> Vec<u8> {
    let (w, h, ow, oh) = (w as usize, h as usize, ow as usize, oh as usize);
    let mut out = vec![0u8; ow * oh * 4];
    for oy in 0..oh {
        let (ya, yb) = (oy * h / oh, ((oy + 1) * h / oh).max(oy * h / oh + 1).min(h));
        for ox in 0..ow {
            let (xa, xb) = (ox * w / ow, ((ox + 1) * w / ow).max(ox * w / ow + 1).min(w));
            let mut s = [0u32; 4];
            for y in ya..yb {
                for x in xa..xb {
                    let i = (y * w + x) * 4;
                    for c in 0..4 {
                        s[c] += u32::from(px[i + c]);
                    }
                }
            }
            let n = ((yb - ya) * (xb - xa)) as u32;
            let o = (oy * ow + ox) * 4;
            for c in 0..4 {
                out[o + c] = ((s[c] + n / 2) / n) as u8;
            }
        }
    }
    out
}
