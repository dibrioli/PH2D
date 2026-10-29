//! **Paper Press** (dono 28/10): a foto para o dono — Knife Stain e Knife Oily, Real Oil ligado, papel Cold de Relief
//! ALTO (1), com o Paper Press 0 · ½ · 1. Folha: `docs/Painter/Pinceis/pranchas/J6_paper_press.jpg`.

use super::*;
use ph2d_wet_paint::tuning::Knob;

/// A tela de dois traços horizontais com o pincel `rel` e o Paper Press `press`.
fn tela(rel: &str, press: f64) -> Vec<u8> {
    let b = biblioteca();
    let mut t = PainterTool::default();
    t.abre_a_sprite(vec![255u8; (OW * OH * 4) as usize], OW, OH);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_relogio_fixo(true);
    assert!(t.wet_preset_aplicar(&preset_rel(&b, rel)).is_empty());
    t.set_brush_paper_kind(1);
    t.set_substrate_depth(1.0);
    t.set_wet_oleo_real_ligado(true);
    t.paint.wetpaint.knobs.set(Knob::PaperPress, press);
    t.set_brush_size_px(70.0);
    t.set_brush_color_srgb8([30, 70, 210]);
    for y in [130.0f32, 320.0] {
        arrasta(&mut t, [60.0, y], [640.0, y + 12.0]);
    }
    {
        let s = t.paint.wetpaint.session.as_mut().expect("sessão");
        s.bring_home();
        let g = s.engine.active_grid();
        let v: Vec<f32> = g
            .material
            .iter()
            .map(|m| m.amassa_papel)
            .filter(|&a| a < 1.0)
            .collect();
        println!(
            "[paper press] {rel} {press}: {} células com amassa < 1 (de {})",
            v.len(),
            g.material.len()
        );
    }
    let mut px = t.run_full().0;
    t.apply_impasto_light(
        &mut px,
        crate::Region {
            x: 0,
            y: 0,
            w: OW,
            h: OH,
        },
    );
    px
}

/// `PH2D_OLEO_PNG=<pasta>`: `press_<pincel>_p<press>.ppm`.
#[test]
#[ignore = "medicao — corre-se à mão no perfil smoke"]
fn medida_paper_press_foto() {
    let d = std::env::var("PH2D_OLEO_PNG").expect("PH2D_OLEO_PNG=<pasta>");
    for (nome, rel) in [
        ("stain", "WetPainter/Oil/Knife/Knife Stain.ph2d-brush.png"),
        ("oily", "WetPainter/Oil/Knife/Knife Oily.ph2d-brush.png"),
    ] {
        for press in [0.0, 0.5, 1.0] {
            let px = tela(rel, press);
            let mut ppm = format!("P6\n{OW} {OH}\n255\n").into_bytes();
            for p in px.as_chunks::<4>().0 {
                ppm.extend_from_slice(&p[..3]);
            }
            let f = format!("{d}/press_{nome}_p{press}.ppm");
            std::fs::write(&f, ppm).expect("grava");
            println!("[paper press] {f}");
        }
    }
}

/// A tinta pousada de `t` com o Paper Press `p` em todas as células (a tela recomposta).
fn com_press(t: &mut PainterTool, p: f32) {
    let s = t.paint.wetpaint.session.as_mut().expect("sessão");
    s.bring_home();
    for l in &mut s.engine.layers {
        for m in ph2d_wet_paint::grid::material::garante(&mut l.grid) {
            m.amassa_papel = p;
        }
    }
    t.wet_recomposite_full();
}

/// A sonda da calibração de `Afinacao::papel_na_base`: com o Paper Press 0 o dente que o papel de Relief 1 põe no
/// miolo da tinta grossa (Paper Press 0 − 1) ÷ o que põe no papel nu (Relief 1 − Relief 0). Alvo: 1 (a tinta é uma
/// camada que segue o papel).
#[test]
#[ignore = "medicao — corre-se à mão no perfil smoke"]
fn medida_paper_press_calibra() {
    let (mut t, cob) = super::cena_papel_texturizado(true, 1.0, true);
    let (miolo, nu) = super::miolo_e_papel_nu(&cob);
    let (mut liso, _) = super::cena_papel_texturizado(true, 0.0, false);
    let (mut rugoso, _) = super::cena_papel_texturizado(true, 1.0, false);
    let nu_px = super::tela_final(&mut rugoso);
    let dn = super::dente_na_tela(&nu_px, &super::tela_final(&mut liso), &nu);
    com_press(&mut t, 1.0);
    let um = super::tela_final(&mut t);
    // A luz MULTIPLICA a cor: o mesmo relevo mexe menos níveis sob a tinta escura — o dente RELATIVO (÷ a média).
    let media = |px: &[u8], m: &[bool]| {
        let (s, n) = px
            .as_chunks::<4>()
            .0
            .iter()
            .zip(m)
            .filter(|(_, v)| **v)
            .fold((0.0, 0.0), |(s, n), (q, _)| {
                (
                    s + 0.2126 * f64::from(q[0])
                        + 0.7152 * f64::from(q[1])
                        + 0.0722 * f64::from(q[2]),
                    n + 1.0,
                )
            });
        s / n
    };
    let (lm, ln) = (media(&um, &miolo), media(&nu_px, &nu));
    for g in [0.5, 1.0, 1.5, 2.0, 2.5, 4.0] {
        let mut o = t.wet_oleo_real_valores();
        o.af.papel_na_base = g;
        t.set_wet_oleo_real(Some(o));
        com_press(&mut t, 0.0);
        let zero = super::tela_final(&mut t);
        let dm = super::dente_na_tela(&zero, &um, &miolo);
        println!(
            "[paper press] ganho {g}: miolo {dm:.3} ÷ nu {dn:.3} = {:.3} · relativo {:.3} (alvo 1)",
            dm / dn,
            (dm / lm) / (dn / ln)
        );
    }
}
