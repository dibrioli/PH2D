//! A fenda e o papel: a cena do dono sem pontos pretos, o papel nos vãos, o papel texturizado e a borda (partido de
//! `medida_oleo_real.rs`, 29/10).

use super::*;

/// A cor final (sRGB, sobre o papel com a sombra do Óleo Real, como em [`grava_folha`]) e a cor SEM relevo
/// de cada célula, com a cobertura: `(final, albedo, alfa)`.
pub(super) fn render_da_cena(
    t: &mut PainterTool,
    o: &ph2d_wet_paint::oleo_real::OleoReal,
) -> Vec<([u8; 3], [u8; 3], u8)> {
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
        ph2d_wet_paint::oleo_real::CampoAltura::de(&grades, o, usize::from(s.ratio), 1, 1, gw, gh);
    let (mut oleo, mut albedo) = (vec![0u8; gw * gh * 4], vec![0u8; gw * gh * 4]);
    for cy in 1..=gh {
        let faixa = (cy - 1) * gw * 4..cy * gw * 4;
        ph2d_wet_paint::render::render_pigment_row_oleo(
            Some(&params),
            &camadas,
            ph2d_wet_paint::render::PigmentVisual {
                paper: false,
                relevo: true,
            },
            o,
            &campo,
            cy,
            1,
            gw,
            &mut oleo[faixa.clone()],
        );
        ph2d_wet_paint::render::render_pigment_row_visual(
            Some(&params),
            &camadas,
            ph2d_wet_paint::render::PigmentVisual {
                paper: false,
                relevo: false,
            },
            cy,
            1,
            gw,
            &mut albedo[faixa],
        );
    }
    let (oleo, albedo) = (oleo.as_chunks::<4>().0, albedo.as_chunks::<4>().0);
    (0..gw * gh)
        .map(|k| {
            let (p, a) = (oleo[k], u32::from(oleo[k][3]));
            let sombra =
                ph2d_wet_paint::oleo_real::sombra_no_papel(o, &campo, k % gw + 1, k / gw + 1);
            let papel = 255 - u32::from(ph2d_wet_paint::oleo_real::sombra_em_u8(sombra));
            let fim = [0, 1, 2].map(|c| ((u32::from(p[c]) * a + papel * (255 - a)) / 255) as u8);
            (
                fim,
                [albedo[k][0], albedo[k][1], albedo[k][2]],
                albedo[k][3],
            )
        })
        .collect()
}

/// ⭐ **A fenda forte nunca volta aos pontos pretos** (W2 E1) — a régua de `oraculo/compara.py` §1 sobre a
/// cena do dono: no vermelho (cobertura > 0,9, albedo linear R > 0,4 e G < 0,05) nenhum pixel tem
/// `max(R,G,B) < 70` com a força no TECTO da faixa (`FENDA_MAX`); e a força escurece mesmo (a luminância
/// média do vermelho desce de 1 para o tecto). Mutação vista a sangrar: o tecto subido a 6 (`max` 60, a
/// medição em `oraculo/README.md` §E1). A lei da força mede-se no CLÁSSICO (céu uniforme): desde a W2 E2 o
/// Estúdio tem o lado de costas para a janela ESCURO de verdade — o Cycles com o mesmo estúdio põe 0,30 %
/// do vermelho abaixo de 70 (o nosso 0,37 % na força 1, 0,02 % no tecto); ali o gate barra a FRACÇÃO
/// (`≤ 0,5 %`), não o pixel (`oraculo/README.md` §E2).
#[test]
fn a_fenda_forte_nao_volta_aos_pontos_pretos_na_cena_do_dono() {
    use ph2d_wet_paint::colorops::transfer::srgb255_to_linear as lin;
    use ph2d_wet_paint::oleo_real::{Ambiente, FENDA_MAX, OleoReal};
    let mut t = pinta_cena_do_dono();
    let lum =
        |c: [u8; 3]| 0.2126 * f64::from(c[0]) + 0.7152 * f64::from(c[1]) + 0.0722 * f64::from(c[2]);
    let mut medias = Vec::new();
    for fenda in [1.0, FENDA_MAX] {
        let o = OleoReal {
            fenda,
            ambiente: Ambiente::Classico,
            ..OleoReal::default()
        };
        let px = render_da_cena(&mut t, &o);
        let vermelho: Vec<[u8; 3]> = px
            .iter()
            .filter(|(_, alb, a)| {
                f64::from(*a) / 255.0 > 0.9
                    && lin(f64::from(alb[0])) > 0.4
                    && lin(f64::from(alb[1])) < 0.05
            })
            .map(|(fim, _, _)| *fim)
            .collect();
        assert!(
            vermelho.len() > 100_000,
            "a cena do dono mudou: {} células vermelhas",
            vermelho.len()
        );
        let mais_escuro = vermelho
            .iter()
            .map(|c| c[0].max(c[1]).max(c[2]))
            .min()
            .expect("vermelho");
        assert!(
            mais_escuro >= 70,
            "força {fenda}: um ponto preto, max(R,G,B) = {mais_escuro}"
        );
        medias.push(vermelho.iter().map(|&c| lum(c)).sum::<f64>() / vermelho.len() as f64);
    }
    assert!(
        medias[1] < medias[0] - 1.0,
        "a força não escurece: {medias:?}"
    );
    for fenda in [1.0, FENDA_MAX] {
        let o = OleoReal {
            fenda,
            ambiente: Ambiente::Estudio,
            ..OleoReal::default()
        };
        let px = render_da_cena(&mut t, &o);
        let (mut n, mut pretos) = (0usize, 0usize);
        for (fim, alb, a) in &px {
            if f64::from(*a) / 255.0 > 0.9
                && lin(f64::from(alb[0])) > 0.4
                && lin(f64::from(alb[1])) < 0.05
            {
                n += 1;
                pretos += usize::from(fim[0].max(fim[1]).max(fim[2]) < 70);
            }
        }
        let f = 100.0 * pretos as f64 / n as f64;
        assert!(
            f <= 0.5,
            "Estúdio, força {fenda}: {f:.2} % do vermelho abaixo de 70"
        );
    }
}

/// ⭐ **K10: o papel nos VÃOS entre lascas nunca fica preto** — a régua da E1 (`max(R,G,B) < 70`) estendida ao
/// PAPEL, que lhe escapava: no Dabber 3 a 200 px o Estúdio punha 21,75 % dos vãos abaixo de 70 (o Cycles da mesma
/// malha 0,30 %, o Rebelle 0 %; `oraculo/README.md` §K10). Vão = célula sem tinta (alfa ≤ 25) com tinta a ≤ 6 células
/// para os QUATRO lados, no recto de 200 (y 230..350): 820 vãos, 30,24 % abaixo de 70 sem o rebote da tinta vizinha
/// (`rebote_papel` 0, visto vermelho), 5,12 % com ele; tecto 8 %. O Clássico: 0 % antes e depois.
/// §BUG-ressalto (27/10): o relevo saiu do papel sem pigmento — a SAIA da forma desfocada inclinava o papel do
/// vão e dava-lhe o céu — e o papel é plano também sob a orla parcial. O Cycles da MESMA malha põe 20,49 % dos vãos
/// abaixo de 70 (0,85 % na malha com saia), p10 52; nós 35,73 %, p10 **57** (a malha do Cycles faz da orla parcial
/// uma cunha de tinta inclinada; para nós ela é papel plano com ≤ 10 % de tinta). O tecto passa a 38 % e o p10 a
/// ≥ 50 (o «nunca preto»: o mais escuro dos vãos fica acima do do Cycles). O Rebelle (0 %) não projecta sombra
/// nenhuma — aberto para o dono (`oraculo/README.md` §BUG-ressalto).
#[test]
fn o_papel_nos_vaos_entre_lascas_nao_fica_preto() {
    use ph2d_wet_paint::oleo_real::{Ambiente, OleoReal};
    let mut t = pinta_cena_dabber3();
    let (gw, gh) = {
        let s = t.paint.wetpaint.session.as_ref().expect("sessão");
        s.grid
    };
    // §BUG-bordas: 38 → 42 % (35,73 → 39,63): o contorno sub-célula tira à orla da lasca a parede em ESCADA que
    // devolvia luz ao vão (o rebote K10: sem ele os vãos ficam MAIS claros que antes, 62,8 → 53,9 %).
    for (ambiente, tecto) in [(Ambiente::Estudio, 42.0), (Ambiente::Classico, 0.0)] {
        let o = OleoReal {
            ambiente,
            ..OleoReal::default()
        };
        let px = render_da_cena(&mut t, &o);
        let tinta = |x: usize, y: usize| px[x + y * gw].2 > 25;
        let (mut n, mut pretos, mut escuros) = (0usize, 0usize, Vec::new());
        for y in 230..350.min(gh - 7) {
            for x in 7..gw - 7 {
                let fechado = !tinta(x, y)
                    && (1..=6).any(|d| tinta(x - d, y))
                    && (1..=6).any(|d| tinta(x + d, y))
                    && (1..=6).any(|d| tinta(x, y - d))
                    && (1..=6).any(|d| tinta(x, y + d));
                if fechado {
                    let c = px[x + y * gw].0;
                    n += 1;
                    pretos += usize::from(c[0].max(c[1]).max(c[2]) < 70);
                    escuros.push(c[0].max(c[1]).max(c[2]));
                }
            }
        }
        assert!(n > 500, "a cena mudou: {n} vãos");
        let f = 100.0 * pretos as f64 / n as f64;
        escuros.sort_unstable();
        let p10 = escuros[n / 10];
        println!("[vãos] {ambiente:?}: {n} vãos, {f:.2} % abaixo de 70, p10 {p10}");
        assert!(
            p10 >= 50,
            "{ambiente:?}: o p10 dos vãos é {p10} (Cycles 52)"
        );
        assert!(
            f <= tecto,
            "{ambiente:?}: {f:.2} % dos vãos abaixo de 70 (tecto {tecto} %)"
        );
    }
}

/// §BUG-papel (smoke do dono 27/10): tinta GROSSA sobre papel TEXTURIZADO — o grão do papel aparece na superfície
/// da tinta «como se ela fosse transparente». A tela final (o composite + a luz do relevo, `apply_impasto_light`)
/// com o Óleo Real ligado e desligado, o papel Cold com Relief 0 e 0,6. Imprime o desvio-padrão da
/// luminância (o grão) no miolo da tinta grossa e no papel nu; `PH2D_OLEO_PNG` grava as telas.
pub(super) fn tela_papel_texturizado(oleo: bool, relief: f32) -> (Vec<u8>, Vec<bool>, Vec<bool>) {
    let (mut t, cob) = cena_papel_texturizado(oleo, relief, true);
    let px = tela_final(&mut t);
    let (miolo, nu) = miolo_e_papel_nu(&cob);
    (px, miolo, nu)
}

/// A cena de [`tela_papel_texturizado`] pintada (`pinta = false`: só o papel) e a máscara da tinta grossa (massa
/// > 3000) em píxeis da tela.
pub(super) fn cena_papel_texturizado(
    oleo: bool,
    relief: f32,
    pinta: bool,
) -> (PainterTool, Vec<bool>) {
    let b = biblioteca();
    let mut t = PainterTool::default();
    // O documento do produto: a tela branca nasce PAPEL com a camada vazia (`abre_a_sprite`).
    t.abre_a_sprite(vec![255u8; (OW * OH * 4) as usize], OW, OH);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_relogio_fixo(true);
    let p = &preset_rel(&b, "WetPainter/Oil/Dabber/Dabber.ph2d-brush.png");
    assert!(t.wet_preset_aplicar(p).is_empty());
    t.set_brush_paper_kind(1);
    t.set_substrate_depth(relief);
    t.set_wet_oleo_real_ligado(oleo);
    t.set_brush_size_px(40.0);
    t.set_brush_color_srgb8([30, 70, 210]);
    for k in 0..if pinta { 3 } else { 0 } {
        let y = 200.0 + 6.0 * k as f32;
        arrasta(&mut t, [120.0, y], [580.0, y]);
    }
    if !pinta {
        return (t, vec![false; (OW * OH) as usize]);
    }
    let cob = {
        let s = t.paint.wetpaint.session.as_mut().expect("sessão");
        s.bring_home();
        let g = s.engine.active_grid();
        let (gw, gh, r) = (s.grid.0, s.grid.1, usize::from(s.ratio));
        let mut miolo = vec![false; (OW * OH) as usize];
        for y in 0..OH as usize {
            for x in 0..OW as usize {
                let (cx, cy) = ((x / r + 1).min(gw), (y / r + 1).min(gh));
                miolo[x + y * OW as usize] = g.sett[cx + cy * g.s] + g.susp[cx + cy * g.s] > 3000.0;
            }
        }
        miolo
    };
    (t, cob)
}

/// A TELA FINAL do produto: o composite e a luz do relevo (`apply_impasto_light`).
pub(super) fn tela_final(t: &mut PainterTool) -> Vec<u8> {
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

/// As amostras (1 em 9 píxeis) do miolo da tinta grossa `cob` (a ≥ 12 px de papel) e do papel nu (a ≥ 40 px).
pub(super) fn miolo_e_papel_nu(cob: &[bool]) -> (Vec<bool>, Vec<bool>) {
    // O miolo: tinta grossa (massa > 3000, onde o grão do pigmento já é 0) a ≥ 12 px de papel; o papel nu: a ≥ 40 px.
    let n = (OW * OH) as usize;
    let perto = |m: &[bool], x: usize, y: usize, r: usize, v: bool| {
        (y.saturating_sub(r)..(y + r + 1).min(OH as usize)).all(|yy| {
            (x.saturating_sub(r)..(x + r + 1).min(OW as usize))
                .all(|xx| m[xx + yy * OW as usize] == v)
        })
    };
    let mut miolo = vec![false; n];
    let mut nu = vec![false; n];
    for y in 0..OH as usize {
        for x in 0..OW as usize {
            miolo[x + y * OW as usize] = (y % 3 == 0 && x % 3 == 0) && perto(cob, x, y, 12, true);
            nu[x + y * OW as usize] = (y % 3 == 0 && x % 3 == 0) && perto(cob, x, y, 40, false);
        }
    }
    (miolo, nu)
}

/// O DENTE que o Relief acrescenta: o desvio-padrão da diferença de luminância entre a tela com relevo `px` e a
/// mesma tela sem ele `liso`, nas células `m` (o que a tinta já tinha de textura própria cancela-se).
pub(super) fn dente_na_tela(px: &[u8], liso: &[u8], m: &[bool]) -> f64 {
    let l = |p: &[u8; 4]| {
        0.2126 * f64::from(p[0]) + 0.7152 * f64::from(p[1]) + 0.0722 * f64::from(p[2])
    };
    let d: Vec<f64> = px
        .as_chunks::<4>()
        .0
        .iter()
        .zip(liso.as_chunks::<4>().0)
        .zip(m)
        .filter(|(_, v)| **v)
        .map(|((a, b), _)| l(a) - l(b))
        .collect();
    let mu = d.iter().sum::<f64>() / d.len() as f64;
    (d.iter().map(|v| (v - mu).powi(2)).sum::<f64>() / d.len() as f64).sqrt()
}

/// ⭐ §BUG-papel: **o dente do papel não atravessa a tinta grossa** — o que o Relief 0,6 acrescenta ao miolo de
/// uma pilha do Dabber de fábrica (massa > 3000) é ≤ 10 % do que acrescenta ao papel nu (antes 67 %, e 49 % sem o
/// Óleo Real: o dente SOMAVA à tinta em toda a parte), com o Óleo Real ligado; e o papel nu mostra-o inteiro.
#[test]
fn o_dente_do_papel_nao_atravessa_a_tinta_grossa() {
    let (liso, miolo, nu) = tela_papel_texturizado(true, 0.0);
    let (px, _, _) = tela_papel_texturizado(true, 0.6);
    let (a, b) = (
        dente_na_tela(&px, &liso, &miolo),
        dente_na_tela(&px, &liso, &nu),
    );
    assert!(b > 4.0, "o papel nu devia mostrar o dente: {b}");
    assert!(
        a <= 0.1 * b,
        "o dente atravessa a tinta grossa: {a} × papel nu {b}"
    );
}

#[test]
#[ignore = "medicao — corre-se à mão no perfil smoke"]
fn medida_oleo_real_papel_texturizado() {
    for oleo in [false, true] {
        let (liso, miolo, nu) = tela_papel_texturizado(oleo, 0.0);
        let (px, _, _) = tela_papel_texturizado(oleo, 0.6);
        let (a, b) = (
            dente_na_tela(&px, &liso, &miolo),
            dente_na_tela(&px, &liso, &nu),
        );
        println!(
            "[papel] óleo {oleo}: o dente do papel (Relief 0,6 − 0) na tinta grossa {a:.2} · no papel nu {b:.2} · razão {:.2}",
            a / b
        );
        if let Ok(d) = std::env::var("PH2D_OLEO_PNG") {
            for (nome, im) in [("r0", &liso), ("r06", &px)] {
                let mut ppm = format!("P6\n{OW} {OH}\n255\n").into_bytes();
                for p in im.as_chunks::<4>().0 {
                    ppm.extend_from_slice(&p[..3]);
                }
                std::fs::write(format!("{d}/papel_o{oleo}_{nome}.ppm"), ppm).expect("grava");
            }
        }
    }
}

/// §BUG-bordas (smoke do dono 27/10): a TELA FINAL (composite + a luz do relevo) de um recto INCLINADO (~5°) e de
/// um S com o pincel `rel`, Óleo Real ligado, papel branco (`texturizado` = Cold, Relief 0,6), Smooth Edges `suave`.
pub(super) fn tela_da_borda(rel: &str, texturizado: bool, suave: f32) -> Vec<u8> {
    let b = biblioteca();
    let mut t = PainterTool::default();
    t.abre_a_sprite(vec![255u8; (OW * OH * 4) as usize], OW, OH);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_relogio_fixo(true);
    assert!(t.wet_preset_aplicar(&preset_rel(&b, rel)).is_empty());
    if texturizado {
        t.set_brush_paper_kind(1);
        t.set_substrate_depth(0.6);
    }
    t.set_wet_oleo_real_ligado(true);
    t.set_wet_smooth_edges(suave);
    t.set_brush_size_px(40.0);
    t.set_brush_color_srgb8([30, 70, 210]);
    arrasta(&mut t, [60.0, 100.0], [640.0, 150.0]);
    // O S: uma senóide de amplitude 50 px e período 330 px, de x 60 a 640 (como o S da bateria).
    let s = |x: f32| {
        [
            x,
            320.0 - 50.0 * (std::f32::consts::TAU * (x - 60.0) / 330.0).sin(),
        ]
    };
    t.on_canvas_pointer(ponto(s(60.0), PointerPhase::Down));
    for k in 1..=290 {
        t.on_canvas_pointer(ponto(s(60.0 + 2.0 * k as f32), PointerPhase::Move));
        t.on_tick(8.0);
    }
    arrasta(&mut t, s(640.0), s(640.0));
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

/// `PH2D_OLEO_PNG=<pasta>`: as telas da borda (`borda_<pincel>_<papel>_s<suave>.ppm`) — régua `oraculo/bordas_regua.py`.
#[test]
#[ignore = "medicao — corre-se à mão no perfil smoke"]
fn medida_oleo_real_bordas() {
    let d = std::env::var("PH2D_OLEO_PNG").expect("PH2D_OLEO_PNG=<pasta>");
    for (nome, rel) in [
        (
            "knifeoily",
            "WetPainter/Oil/Knife/Knife Oily.ph2d-brush.png",
        ),
        ("dabber", "WetPainter/Oil/Dabber/Dabber.ph2d-brush.png"),
    ] {
        for tex in [false, true] {
            for suave in [0.0f32, 1.0] {
                let t0 = std::time::Instant::now();
                let px = tela_da_borda(rel, tex, suave);
                let mut ppm = format!("P6\n{OW} {OH}\n255\n").into_bytes();
                for p in px.as_chunks::<4>().0 {
                    ppm.extend_from_slice(&p[..3]);
                }
                let f = format!(
                    "{d}/borda_{nome}_{}_s{suave}.ppm",
                    if tex { "tex" } else { "branco" }
                );
                std::fs::write(&f, ppm).expect("grava");
                println!("[borda] {f} ({:.1} s)", t0.elapsed().as_secs_f64());
            }
        }
    }
}
