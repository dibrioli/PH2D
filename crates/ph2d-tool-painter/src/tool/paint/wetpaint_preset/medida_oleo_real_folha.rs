//! A FOLHA de medição do Óleo Real (W0/W1): a cena arrastada, a folha PNG, o Óleo pelo ambiente e as alturas impressas
//! (partido de `medida_oleo_real.rs`, 29/10).

use super::*;

pub(super) const OW: u32 = 700;
pub(super) const OH: u32 = 460;

/// Uma pincelada de `a` a `z` com o passo de 3 px do `tracos.py`, e espera a água parar.
pub(super) fn arrasta(t: &mut PainterTool, a: [f32; 2], z: [f32; 2]) -> u32 {
    let n = ((z[0] - a[0]).hypot(z[1] - a[1]) / 3.0).ceil() as u32;
    t.on_canvas_pointer(ponto(a, PointerPhase::Down));
    for k in 1..=n {
        let f = k as f32 / n as f32;
        let pos = [a[0] + (z[0] - a[0]) * f, a[1] + (z[1] - a[1]) * f];
        t.on_canvas_pointer(ponto(pos, PointerPhase::Move));
        t.on_tick(8.0);
    }
    t.on_canvas_pointer(ponto(z, PointerPhase::Up));
    let mut ticks = 0;
    loop {
        t.on_tick(1000.0 / 60.0);
        ticks += 1;
        let s = t.paint.wetpaint.session.as_mut().expect("sessão");
        s.bring_home();
        if !s.engine.active_grid().has_fluid || ticks >= 60 * 60 {
            break;
        }
    }
    ticks
}

/// A folha em `dir`: `hoje.ppm`, `oleo.ppm`, os termos `t_*.pgm` e as entradas do oráculo.
pub(super) fn grava_folha(t: &mut PainterTool, dir: &std::path::Path) {
    grava_folha_com(t, dir, oleo_do_env());
}

pub(super) fn grava_folha_com(
    t: &mut PainterTool,
    dir: &std::path::Path,
    o: ph2d_wet_paint::oleo_real::OleoReal,
) {
    let s = t.paint.wetpaint.session.as_mut().expect("sessão");
    s.bring_home();
    let (gw, gh) = s.grid;
    let params = s.engine.sim.gather_params(&s.engine.tuning);
    let camadas: Vec<ph2d_wet_paint::render::RenderLayer<'_>> = s
        .engine
        .layers
        .iter()
        .map(|l| ph2d_wet_paint::render::RenderLayer {
            grid: &l.grid,
            opacity: l.opacity,
            visible: l.visible,
        })
        .collect();
    let grades: Vec<(&ph2d_wet_paint::grid::Grid, bool)> =
        camadas.iter().map(|c| (c.grid, c.visible)).collect();
    let campo =
        ph2d_wet_paint::oleo_real::CampoAltura::de(&grades, &o, usize::from(s.ratio), 1, 1, gw, gh);
    // Os termos em cinza (0..1 → 0..255; `e` e o brilho a ½ de escala), só onde há tinta.
    let mut termos = [(); 4].map(|()| vec![0u8; gw * gh]);
    let ativa = s.engine.active_grid();
    for cy in 1..=gh {
        for cx in 1..=gw {
            if ativa.sett[cx + cy * ativa.s] + ativa.susp[cx + cy * ativa.s] <= 0.0 {
                continue;
            }
            let t = ph2d_wet_paint::oleo_real::termos(&o, &campo, cx, cy);
            let k = (cx - 1) + (cy - 1) * gw;
            let espec = (t.espec[0] + t.espec[1] + t.espec[2]) / 3.0;
            for (j, v) in [t.vis, t.luz, t.e(&o) * 0.5, espec * 0.5]
                .into_iter()
                .enumerate()
            {
                termos[j][k] = (v.clamp(0.0, 1.0) * 254.0) as u8 + 1;
            }
        }
    }
    // K4: os termos do PAPEL em toda a grade (`luz`, `vis` com a força da fenda, o ganho médio do céu), f32 LE —
    // a régua do halo (`oraculo/halo_regua.py`) ensaia as leis da sombra no papel sobre eles.
    let mut papel = Vec::with_capacity(gw * gh * 12);
    for cy in 1..=gh {
        for cx in 1..=gw {
            let t = ph2d_wet_paint::oleo_real::termos(&o, &campo, cx, cy);
            let g = t.dir.map_or(1.0, |d| (d[0] + d[1] + d[2]) / 3.0);
            for v in [t.luz, t.vis, g] {
                papel.extend_from_slice(&(v as f32).to_le_bytes());
            }
        }
    }
    std::fs::write(dir.join("t_papel.f32"), papel).expect("grava");
    for (nome, v) in ["t_vis", "t_luz", "t_e", "t_espec"].iter().zip(&termos) {
        let mut pgm = format!("P5\n{gw} {gh}\n255\n").into_bytes();
        pgm.extend_from_slice(v);
        std::fs::write(dir.join(format!("{nome}.pgm")), pgm).expect("grava");
    }
    // As entradas do oráculo: as três alturas (células) e a cobertura, sobre a grade inteira.
    let mut alturas = [(); 3].map(|()| Vec::with_capacity(gw * gh * 4));
    for cy in 1..=gh {
        for cx in 1..=gw {
            let (x, y) = (cx as isize, cy as isize);
            for (j, h) in [campo.at(x, y), campo.at_fina(x, y), campo.at_sombra(x, y)]
                .into_iter()
                .enumerate()
            {
                alturas[j].extend_from_slice(&(h as f32).to_le_bytes());
            }
        }
    }
    // K7: a marca do nivelamento da camada activa (f32 LE, células; zeros sem o campo).
    let mut marca = Vec::with_capacity(gw * gh * 4);
    for cy in 1..=gh {
        for cx in 1..=gw {
            let m = ph2d_wet_paint::grid::nivelado::em(ativa, cx + cy * ativa.s);
            marca.extend_from_slice(&m.to_le_bytes());
        }
    }
    std::fs::write(dir.join("nivelado.f32"), marca).expect("grava");
    for (nome, v) in ["h_forma", "h_fina", "h_sombra"].iter().zip(&alturas) {
        std::fs::write(dir.join(format!("{nome}.f32")), v).expect("grava");
    }
    std::fs::write(
        dir.join("oraculo.txt"),
        format!(
            "gw={gw}\ngh={gh}\nratio={}\nazimute_graus={}\naltitude_graus={}\nceu={}\npenumbra={}\nrugosidade={}\n# alturas f32 LE em células, linha a linha a partir do topo; x direita, y baixo\n# {o:?}\n",
            s.ratio,
            o.azimute.to_degrees(),
            o.altitude.to_degrees(),
            o.forca_ambiente,
            o.af.penumbra,
            o.material.rugosidade,
        ),
    )
    .expect("grava");
    for (nome, com_oleo, relevo) in [
        ("hoje", false, true),
        ("oleo", true, true),
        ("albedo", false, false),
    ] {
        let visual = ph2d_wet_paint::render::PigmentVisual {
            paper: false,
            relevo,
        };
        let mut px = vec![0u8; gw * gh * 4];
        for cy in 1..=gh {
            let linha = &mut px[(cy - 1) * gw * 4..cy * gw * 4];
            if com_oleo {
                ph2d_wet_paint::render::render_pigment_row_oleo(
                    Some(&params),
                    &camadas,
                    visual,
                    &o,
                    &campo,
                    cy,
                    1,
                    gw,
                    linha,
                );
            } else {
                ph2d_wet_paint::render::render_pigment_row_visual(
                    Some(&params),
                    &camadas,
                    visual,
                    cy,
                    1,
                    gw,
                    linha,
                );
            }
        }
        let mut ppm = format!("P6\n{gw} {gh}\n255\n").into_bytes();
        for (k, p) in px.as_chunks::<4>().0.iter().enumerate() {
            let a = u32::from(p[3]);
            // O papel recebe a sombra do Óleo Real como no composite do produto (o plano `sombra`).
            let papel = if com_oleo {
                let s =
                    ph2d_wet_paint::oleo_real::sombra_no_papel(&o, &campo, k % gw + 1, k / gw + 1);
                255 - u32::from(ph2d_wet_paint::oleo_real::sombra_em_u8(s))
            } else {
                255
            };
            for &v in &p[..3] {
                ppm.push(((u32::from(v) * a + papel * (255 - a)) / 255) as u8);
            }
        }
        let f = dir.join(format!("{nome}.ppm"));
        std::fs::write(&f, ppm).expect("grava");
        println!("[oleo] {}", f.display());
        if nome == "albedo" {
            let mut pgm = format!("P5\n{gw} {gh}\n255\n").into_bytes();
            pgm.extend(px.as_chunks::<4>().0.iter().map(|p| p[3]));
            std::fs::write(dir.join("cobertura.pgm"), pgm).expect("grava");
            grava_antes_da_e2(&o, &campo, &px, gw, dir);
        }
    }
}

/// `antes_e2.ppm`: a MESMA óptica com o difuso do céu UNIFORME (a lei de antes da W2 E2: o ambiente só no
/// brilho; o papel sem o sector) — o `ilumina` sobre a cor sem relevo `albedo` (8 bits) com `dir = None`.
pub(super) fn grava_antes_da_e2(
    o: &ph2d_wet_paint::oleo_real::OleoReal,
    campo: &ph2d_wet_paint::oleo_real::CampoAltura,
    albedo: &[u8],
    gw: usize,
    dir: &std::path::Path,
) {
    use ph2d_wet_paint::colorops::transfer::{srgb255_of_linear, srgb255_to_linear};
    use ph2d_wet_paint::oleo_real::{Ambiente, OleoReal, difuso, rola, sombra_no_papel, termos};
    let classico = OleoReal {
        ambiente: Ambiente::Classico,
        ..*o
    };
    let gh = albedo.len() / 4 / gw;
    let mut ppm = format!("P6\n{gw} {gh}\n255\n").into_bytes();
    for (k, p) in albedo.as_chunks::<4>().0.iter().enumerate() {
        let (cx, cy) = (k % gw + 1, k / gw + 1);
        let a = u32::from(p[3]);
        let papel = 255
            - u32::from(ph2d_wet_paint::oleo_real::sombra_em_u8(sombra_no_papel(
                &classico, campo, cx, cy,
            )));
        let cor = if a > 0 {
            let mut t = termos(o, campo, cx, cy);
            t.dir = None;
            let cob = f64::from(p[3]) / 255.0;
            [0, 1, 2].map(|c| {
                let al = srgb255_to_linear(f64::from(p[c]));
                let v = rola(al * difuso(o, &t, al, c), t.espec[c] * cob).clamp(0.0, 1.0);
                srgb255_of_linear(v).round().clamp(0.0, 255.0) as u32
            })
        } else {
            [0; 3]
        };
        for c in cor {
            ppm.push(((c * a + papel * (255 - a)) / 255) as u8);
        }
    }
    std::fs::write(dir.join("antes_e2.ppm"), ppm).expect("grava");
}

/// O Óleo Real de fábrica com o que `PH2D_OLEO_AF="campo=valor,…"` pedir (knobs em graus para a luz).
pub(super) fn oleo_do_env() -> ph2d_wet_paint::oleo_real::OleoReal {
    oleo_de(&std::env::var("PH2D_OLEO_AF").unwrap_or_default())
}

/// O Óleo Real de fábrica com os `campo=valor,…` de `af` (a língua do `PH2D_OLEO_AF`).
pub(super) fn oleo_de(af: &str) -> ph2d_wet_paint::oleo_real::OleoReal {
    let mut o = ph2d_wet_paint::oleo_real::OleoReal::default();
    for par in af.split(',').filter(|p| !p.is_empty()) {
        let (k, v) = par
            .split_once('=')
            .unwrap_or_else(|| panic!("{par}: campo=valor"));
        let v: f64 = v.parse().unwrap_or_else(|_| panic!("{par}: número"));
        let af = &mut o.af;
        match k {
            "profundidade" => o.profundidade = v,
            "azimute" => o.azimute = v.to_radians(),
            "altitude" => o.altitude = v.to_radians(),
            "sombra" => o.sombra = v,
            "brilho" => o.brilho = v,
            "px_por_massa" => af.px_por_massa = v,
            "raio_suave" => af.raio_suave = v as usize,
            "raio_sombra" => af.raio_sombra = v as usize,
            "raio_detalhe" => af.raio_detalhe = v as usize,
            "grao" => af.grao = v,
            "fios" => af.fios = v,
            "ambiente" => {
                o.ambiente = ph2d_wet_paint::oleo_real::Ambiente::de_indice(v as u8)
                    .unwrap_or_else(|| panic!("ambiente {v}: 0..=3"));
            }
            "fenda" => o.fenda = v,
            "alcance_fenda" => o.alcance_fenda = v,
            // `ceu` = o nome de antes (o `oraculo.txt` e o `cycles.py` leem-no assim).
            "ceu" | "forca_ambiente" => o.forca_ambiente = v,
            // O material GLOBAL do cartão (`mat_<campo>` de `MaterialCartao::CAMPOS`, valores absolutos da W3).
            m if m.starts_with("mat_") => {
                let i = ph2d_wet_paint::oleo_real::MaterialCartao::CAMPOS
                    .iter()
                    .position(|c| m[4..] == **c)
                    .unwrap_or_else(|| panic!("{m}: campo do material desconhecido"));
                o.material.poe(i, v);
            }
            "penumbra" => af.penumbra = v,
            // K10: o rebote da tinta vizinha no papel (`0` = a sombra de antes).
            "rebote_papel" => af.rebote_papel = v,
            // A Roughness ABSOLUTA do cartão (antes da W3 era `Afinacao::rugosidade`; o `cycles.py` usa o nome).
            "rugosidade" => o.material.rugosidade = v,
            "brilho_max" => af.brilho_max = v,
            "antisserrilhado" => af.antisserrilhado = v,
            "antisserrilhado_ambiente" => af.antisserrilhado_ambiente = v,
            _ => panic!("{k}: campo desconhecido"),
        }
    }
    println!("[oleo] {o:?}");
    o
}

pub(super) fn pinta_folha() -> PainterTool {
    let b = biblioteca();
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (W * H * 4) as usize], W, H);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_relogio_fixo(true);
    pinta_folha_com(&mut t, &b);
    t
}

pub(super) fn tracos() -> Vec<Traco> {
    let env = |k: &str, d: &str| std::env::var(k).unwrap_or_else(|_| d.to_owned());
    let faca = env(
        "PH2D_OLEO_FACA",
        "WetPainter/Oil/Dabber/Dabber.ph2d-brush.png",
    );
    let faca_tex = env("PH2D_OLEO_FACA_TEX", &faca);
    let oily = env(
        "PH2D_OLEO_OILY",
        "WetPainter/Oil/Flat/Filbert.ph2d-brush.png",
    );
    let thick = env(
        "PH2D_OLEO_THICK",
        "WetPainter/Oil/Bristle/Oil Bristle.ph2d-brush.png",
    );
    vec![
        Traco {
            rotulo: "faca H azul",
            preset: faca.clone(),
            cor: AZ,
            a: [80.0, 110.0],
            b: [560.0, 110.0],
            diametro: 120.0,
        },
        Traco {
            rotulo: "faca V azul",
            preset: faca.clone(),
            cor: AZ,
            a: [700.0, 60.0],
            b: [700.0, 420.0],
            diametro: 120.0,
        },
        Traco {
            rotulo: "oily azul",
            preset: oily.clone(),
            cor: AZ,
            a: [850.0, 100.0],
            b: [1150.0, 100.0],
            diametro: 100.0,
        },
        Traco {
            rotulo: "thick azul",
            preset: thick.clone(),
            cor: AZ,
            a: [850.0, 260.0],
            b: [1150.0, 260.0],
            diametro: 100.0,
        },
        Traco {
            rotulo: "faca H branco",
            preset: faca.clone(),
            cor: BR,
            a: [80.0, 300.0],
            b: [560.0, 300.0],
            diametro: 120.0,
        },
        Traco {
            rotulo: "faca H escuro",
            preset: faca.clone(),
            cor: ES,
            a: [80.0, 480.0],
            b: [560.0, 480.0],
            diametro: 120.0,
        },
        Traco {
            rotulo: "faca-tex azul",
            preset: faca_tex.clone(),
            cor: AZ,
            a: [700.0, 560.0],
            b: [1150.0, 560.0],
            diametro: 120.0,
        },
        Traco {
            rotulo: "oily escuro",
            preset: oily.clone(),
            cor: ES,
            a: [850.0, 720.0],
            b: [1150.0, 720.0],
            diametro: 100.0,
        },
    ]
}

pub(super) fn pinta_folha_com(t: &mut PainterTool, b: &Biblioteca) {
    for tr in &tracos() {
        let p = &preset_rel(b, &tr.preset);
        let erros = t.wet_preset_aplicar(p);
        assert!(erros.is_empty(), "{erros:?}");
        t.set_brush_size_px(tr.diametro / 2.0);
        t.set_brush_color_srgb8(tr.cor);
        // `PH2D_OLEO_FACA_H3=1`: as facas com a lâmina H3 (W0b), o fio ATRAVÉS da marcha de cada traço.
        if tr.rotulo.starts_with("faca")
            && std::env::var("PH2D_OLEO_FACA_H3").is_ok_and(|v| v == "1")
        {
            lamina(t);
            t.paint.wetpaint.knobs.set(Knob::Leveling, 0.0);
            let vertical = (tr.b[1] - tr.a[1]).abs() > (tr.b[0] - tr.a[0]).abs();
            t.set_brush_dab_angle(if vertical { 0.0 } else { 90.0 });
        }
        let ticks = arrasta(t, tr.a, tr.b);
        println!(
            "[oleo] {} ({}): água parou após {} ticks",
            tr.rotulo, tr.preset, ticks
        );
    }
}

pub(super) fn imprime_alturas(t: &mut PainterTool) {
    let s = t.paint.wetpaint.session.as_mut().expect("sessão");
    s.bring_home();
    let ratio = s.ratio;
    let (gw, gh) = s.grid;
    let g = s.engine.active_grid();
    let st = g.s;
    let tem_altura = !g.altura.is_empty();
    println!(
        "[oleo] razão (px de tela por célula) = {ratio} · grade {gw}×{gh} · altura preenchida: {tem_altura} \
         (não-zero: {})",
        g.altura.iter().filter(|&&h| h != 0.0).count()
    );
    let m = |i: usize| f64::from(g.sett[i]) + f64::from(g.susp[i]);
    let h = |i: usize| {
        if tem_altura {
            f64::from(g.altura[i])
        } else {
            0.0
        }
    };
    let emb_cru = |i: usize| -> (f64, f64, f64) {
        let k = f64::from(g.relevo[i]);
        let e_m = ((m(i + 1) - m(i - 1)) * 0.5 + (m(i + st) - m(i - st)) * 1.0) * 0.008 * k;
        let e_h = ((h(i + 1) - h(i - 1)) * 0.5 + (h(i + st) - h(i - st)) * 1.0)
            * ph2d_wet_paint::bristles_relevo::LUZ
            * k;
        (e_m + e_h, e_m, e_h)
    };
    let r = f32::from(ratio);
    let cel = |x: f32, y: f32| -> usize {
        let (cx, cy) = ((x / r) as usize + 1, (y / r) as usize + 1);
        cy * st + cx
    };
    println!(
        "{:<14} {:>6} | {:>8} {:>8} {:>8} {:>8} | {:>6} {:>6} {:>6} {:>6} | {:>6} {:>6} {:>6} {:>6} | {:>6} {:>6} | {:>6}",
        "traço",
        "células",
        "m p50",
        "m p90",
        "m p99",
        "m max",
        "h p50",
        "h p90",
        "h p99",
        "h max",
        "|e|p50",
        "|e|p90",
        "|e|p99",
        "|e|max",
        "|em|99",
        "|eh|99",
        "clamp%"
    );
    for tr in &tracos() {
        let rad = tr.diametro / 2.0 + 10.0;
        let (x0, x1) = (tr.a[0].min(tr.b[0]) - rad, tr.a[0].max(tr.b[0]) + rad);
        let (y0, y1) = (tr.a[1].min(tr.b[1]) - rad, tr.a[1].max(tr.b[1]) + rad);
        let (mut vm, mut vh, mut ve, mut vem, mut veh) = (vec![], vec![], vec![], vec![], vec![]);
        let mut morde = 0usize;
        let mut relevos = vec![];
        let mut y = y0.max(0.0);
        while y < y1.min(H as f32) {
            let mut x = x0.max(0.0);
            while x < x1.min(W as f32) {
                let i = cel(x, y);
                if m(i) > 0.0 {
                    vm.push(m(i));
                    vh.push(h(i));
                    let (e, em, eh) = emb_cru(i);
                    if e.abs() > 40.0 {
                        morde += 1;
                    }
                    ve.push(e.abs());
                    vem.push(em.abs());
                    veh.push(eh.abs());
                    relevos.push(f64::from(g.relevo[i]));
                }
                x += r;
            }
            y += r;
        }
        let n = vm.len();
        println!(
            "{:<14} {:>6} | {:>8.1} {:>8.1} {:>8.1} {:>8.1} | {:>6.3} {:>6.3} {:>6.3} {:>6.3} | {:>6.2} {:>6.2} {:>6.2} {:>6.2} | {:>6.2} {:>6.2} | {:>6.2}  relevo p50 {:.3}",
            tr.rotulo,
            n,
            pct(&mut vm, 0.5),
            pct(&mut vm, 0.9),
            pct(&mut vm, 0.99),
            pct(&mut vm, 1.0),
            pct(&mut vh, 0.5),
            pct(&mut vh, 0.9),
            pct(&mut vh, 0.99),
            pct(&mut vh, 1.0),
            pct(&mut ve, 0.5),
            pct(&mut ve, 0.9),
            pct(&mut ve, 0.99),
            pct(&mut ve, 1.0),
            pct(&mut vem, 0.99),
            pct(&mut veh, 0.99),
            100.0 * morde as f64 / n.max(1) as f64,
            pct(&mut relevos, 0.5),
        );
    }
    // O perfil da borda DIREITA da faca vertical, a meio (y = 240): 40 amostras de 2 px centradas na borda.
    let yc = 240.0f32;
    let borda = (700..W as i32)
        .map(|x| x as f32)
        .take_while(|&x| m(cel(x, yc)) > 10.0)
        .last()
        .unwrap_or(700.0);
    println!("[oleo] perfil faca V em y={yc}: borda direita (última célula com m>10) em x={borda}");
    println!("{:>6} {:>9} {:>8} {:>8}", "x", "m", "altura", "emb");
    for k in -20..20 {
        let x = borda + 2.0 * k as f32;
        let i = cel(x, yc);
        println!("{:>6} {:>9.1} {:>8.3} {:>8.2}", x, m(i), h(i), emb_cru(i).0);
    }
}
