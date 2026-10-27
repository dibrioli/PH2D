//! **Medição W0(c) do Óleo Real** (doc `49_realshader/plano.md`): a escala de altura do Wet Paint nas pinceladas
//! equivalentes às sondas do Rebelle (`49_realshader/ferramentas/tracos.py`). Imprime, nunca barra:
//! `bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --profile smoke --lib medida_oleo_real_altura -- --ignored --nocapture`.
use crate::{PaintMedia, PainterTool};
use ph2d_brush_presets::Biblioteca;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_wet_paint::tuning::Knob;

const W: u32 = 1200;
const H: u32 = 800;
const AZ: [u8; 3] = [20, 150, 230];
const BR: [u8; 3] = [240, 240, 236];
const ES: [u8; 3] = [30, 25, 60];

fn ponto(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn biblioteca() -> Biblioteca {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    Biblioteca::carregar(&raiz, None)
}

/// Os knobs do relevo de cada pincel de fábrica do Wet Paint, depois de aplicado.
#[test]
#[ignore = "medicao — corre-se à mão no perfil smoke"]
fn medida_oleo_real_altura_knobs_de_fabrica() {
    let b = biblioteca();
    println!(
        "{:<44} {:>7} {:>7} {:>7} {:>7} {:>7} {:>8}",
        "preset", "Emboss", "EmbExp", "EmbRef", "BrHgt", "BrRem", "Pigment"
    );
    for e in b.entradas.iter().filter(|e| e.partes()[0] == "WetPainter") {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; 64 * 64 * 4], 64, 64);
        t.set_paint_media(PaintMedia::WetPaint);
        let _ = t.wet_preset_aplicar(&e.preset);
        let k = &t.paint.wetpaint.knobs;
        println!(
            "{:<44} {:>7.3} {:>7.3} {:>7.1} {:>7.3} {:>7.3} {:>8.1}",
            e.relativo,
            k.get(Knob::Emboss),
            k.get(Knob::EmbossSizeExp),
            k.get(Knob::EmbossSizeRef),
            k.get(Knob::BristleHeight),
            k.get(Knob::BristleRemove),
            k.get(Knob::PigmentPerDab),
        );
    }
}

struct Traco {
    rotulo: &'static str,
    preset: String,
    cor: [u8; 3],
    a: [f32; 2],
    b: [f32; 2],
    diametro: f32,
}

fn pct(v: &mut [f64], p: f64) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(f64::total_cmp);
    let i = ((v.len() - 1) as f64 * p).round() as usize;
    v[i]
}

/// As pinceladas do `tracos.py` com os NOSSOS pincéis de fábrica, numa tela 1200×800, como no Rebelle.
/// Presets por env (`PH2D_OLEO_FACA`, `PH2D_OLEO_FACA_TEX`, `PH2D_OLEO_OILY`, `PH2D_OLEO_THICK`: o `relativo`).
#[test]
#[ignore = "medicao — corre-se à mão no perfil smoke"]
fn medida_oleo_real_altura_tracos() {
    let mut t = pinta_folha();
    imprime_alturas(&mut t);
}

/// A folha do `tracos.py` sai em `$PH2D_OLEO_PNG/hoje.ppm` (o render do produto com o emboss do preset) e
/// `oleo.ppm` (o Óleo Real), sobre papel branco — para pôr ao lado de `capturas/A_base.jpg`.
#[test]
#[ignore = "medicao — corre-se à mão no perfil smoke"]
fn medida_oleo_real_folha_png() {
    let dir =
        std::path::PathBuf::from(std::env::var("PH2D_OLEO_PNG").expect("PH2D_OLEO_PNG=<pasta>"));
    let mut t = pinta_folha();
    grava_folha(&mut t, &dir);
}

/// **As entradas do oráculo Cycles** (W1c, `49_realshader/oraculo/`): a cena da captura do dono (vermelho
/// saturado empilhado com Dabber e Oil Bristle, e uma pilha clara) em `$PH2D_OLEO_PNG/` — a folha de
/// sempre mais a cor SEM relevo (`albedo.ppm`), a cobertura (`cobertura.pgm`) e as três alturas em f32 LE,
/// células (`h_forma.f32`, `h_fina.f32`, `h_sombra.f32`), descritas em `oraculo.txt`.
/// `PH2D_OLEO_VARIANTES="nome:campo=v,…;nome2:…"` pinta a cena UMA vez e grava cada variante em
/// `$PH2D_OLEO_PNG/<nome>/` (o `PH2D_OLEO_AF` de cada uma vem depois do `:`).
#[test]
#[ignore = "medicao — corre-se à mão no perfil smoke"]
fn medida_oleo_real_oraculo() {
    let dir =
        std::path::PathBuf::from(std::env::var("PH2D_OLEO_PNG").expect("PH2D_OLEO_PNG=<pasta>"));
    // `PH2D_OLEO_CENA=lisos`: os traços LISOS e baixos da bateria (régua do halo, K4).
    // `=smooth2`: o recto do Knife Smooth 2 (candidato W2) com `knifeLevel` 0 · 0,3 · 1 (as paredes, K6).
    let mut t = match std::env::var("PH2D_OLEO_CENA").as_deref() {
        Ok("lisos") => pinta_cena_lisa(),
        Ok("smooth2") => pinta_cena_smooth2(),
        _ => pinta_cena_do_dono(),
    };
    match std::env::var("PH2D_OLEO_VARIANTES") {
        Ok(vs) => {
            for v in vs.split(';').filter(|v| !v.is_empty()) {
                let (nome, af) = v.split_once(':').unwrap_or((v, ""));
                let d = dir.join(nome);
                std::fs::create_dir_all(&d).expect("pasta");
                grava_folha_com(&mut t, &d, oleo_de(af));
            }
        }
        Err(_) => grava_folha(&mut t, &dir),
    }
}

/// A cena da captura do dono (W1c): o vermelho saturado empilhado com Dabber e Oil Bristle e uma pilha clara.
pub(super) const VERMELHO_DO_DONO: [u8; 3] = [215, 25, 30];

/// Pinta a cena da captura do dono numa tela `OW×OH` (a de [`medida_oleo_real_oraculo`] e do gate da fenda).
type Pilha = (&'static str, [u8; 3], f32, [[f32; 2]; 2]);

pub(super) fn pinta_cena_do_dono() -> PainterTool {
    let b = biblioteca();
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (OW * OH * 4) as usize], OW, OH);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_relogio_fixo(true);
    const VERMELHO: [u8; 3] = VERMELHO_DO_DONO;
    const CLARO: [u8; 3] = [238, 226, 196];
    let pilhas: [Pilha; 6] = [
        (
            "WetPainter/Oil/Dabber/Dabber.ph2d-brush.png",
            VERMELHO,
            60.0,
            [[60.0, 120.0], [460.0, 150.0]],
        ),
        (
            "WetPainter/Oil/Dabber/Dabber.ph2d-brush.png",
            VERMELHO,
            60.0,
            [[120.0, 60.0], [300.0, 260.0]],
        ),
        (
            "WetPainter/Oil/Bristle/Oil Bristle.ph2d-brush.png",
            VERMELHO,
            55.0,
            [[40.0, 330.0], [640.0, 290.0]],
        ),
        (
            "WetPainter/Oil/Bristle/Oil Bristle.ph2d-brush.png",
            VERMELHO,
            55.0,
            [[60.0, 400.0], [620.0, 380.0]],
        ),
        (
            "WetPainter/Oil/Dabber/Dabber.ph2d-brush.png",
            CLARO,
            55.0,
            [[500.0, 60.0], [640.0, 220.0]],
        ),
        (
            "WetPainter/Oil/Bristle/Oil Bristle.ph2d-brush.png",
            CLARO,
            50.0,
            [[520.0, 240.0], [660.0, 120.0]],
        ),
    ];
    for (preset, cor, raio, [a, z]) in pilhas {
        let p = &b
            .entradas
            .iter()
            .find(|e| e.relativo == preset)
            .unwrap_or_else(|| panic!("preset {preset} não existe"))
            .preset;
        let erros = t.wet_preset_aplicar(p);
        assert!(erros.is_empty(), "{erros:?}");
        t.set_brush_size_px(raio);
        t.set_brush_color_srgb8(cor);
        // Quatro passagens, a 2.ª e a 4.ª de volta e deslocadas: as cristas da captura do dono.
        for k in 0..4 {
            let d = 10.0 * (k % 2) as f32;
            let (a, z) = ([a[0], a[1] + d], [z[0], z[1] + d]);
            let ticks = if k % 2 == 0 {
                arrasta(&mut t, a, z)
            } else {
                arrasta(&mut t, z, a)
            };
            println!("[oraculo] {preset} passagem {} (água {ticks})", k + 1);
        }
    }
    t
}

/// Os traços LISOS da bateria (K4, o halo): uma passagem de cada faca e do Filbert a 40 px, azul sobre o
/// papel, horizontais e um vertical — o que a foto do Rebelle mostra sem sombra no papel.
pub(super) fn pinta_cena_lisa() -> PainterTool {
    let b = biblioteca();
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (OW * OH * 4) as usize], OW, OH);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_relogio_fixo(true);
    let tracos: [(&str, [[f32; 2]; 2]); 5] = [
        ("Knife/Knife Smooth 3", [[60.0, 70.0], [520.0, 70.0]]),
        ("Knife/Knife Oily", [[60.0, 160.0], [520.0, 160.0]]),
        ("Knife/Knife Dry", [[60.0, 250.0], [520.0, 250.0]]),
        ("Flat/Filbert", [[60.0, 340.0], [520.0, 340.0]]),
        ("Knife/Knife Smooth 3", [[620.0, 60.0], [620.0, 400.0]]),
    ];
    for (preset, [a, z]) in tracos {
        let rel = format!("WetPainter/Oil/{preset}.ph2d-brush.png");
        let p = &b
            .entradas
            .iter()
            .find(|e| e.relativo == rel)
            .unwrap_or_else(|| panic!("preset {rel} não existe"))
            .preset;
        let erros = t.wet_preset_aplicar(p);
        assert!(erros.is_empty(), "{erros:?}");
        t.set_brush_size_px(20.0);
        t.set_brush_color_srgb8(AZ);
        let ticks = arrasta(&mut t, a, z);
        println!("[oraculo] {preset} (água {ticks})");
    }
    t
}

/// O `recto_lento` da bateria (40 px, passo 3, o azul da bateria) com o Knife Smooth 2 candidato
/// (`Pinceis/W2_Knife_candidatos/`), a `knifeLevel` 0 · 0,3 · 1 em `y` 90 · 200 · 310 — o perfil transversal das
/// paredes × o Rebelle (K6).
pub(super) fn pinta_cena_smooth2() -> PainterTool {
    let f = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/Painter/Pinceis/W2_Knife_candidatos/Knife Smooth 2.ph2d-brush.png");
    let p = ph2d_brush_presets::ler(&std::fs::read(&f).expect("preset")).expect("preset");
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (OW * OH * 4) as usize], OW, OH);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_relogio_fixo(true);
    for (nivel, y) in [(0.0, 90.0), (0.3, 200.0), (1.0, 310.0)] {
        let erros = t.wet_preset_aplicar(&p);
        assert!(erros.is_empty(), "{erros:?}");
        t.paint.wetpaint.knobs.set(Knob::KnifeLevel, nivel);
        t.set_brush_size_px(20.0);
        t.set_brush_color_srgb8([23, 101, 179]);
        let ticks = arrasta(&mut t, [60.0, y], [560.0, y]);
        println!("[oraculo] Knife Smooth 2 knifeLevel {nivel} (água {ticks})");
    }
    t
}

const OW: u32 = 700;
const OH: u32 = 460;

/// Uma pincelada de `a` a `z` com o passo de 3 px do `tracos.py`, e espera a água parar.
fn arrasta(t: &mut PainterTool, a: [f32; 2], z: [f32; 2]) -> u32 {
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
fn grava_folha(t: &mut PainterTool, dir: &std::path::Path) {
    grava_folha_com(t, dir, oleo_do_env());
}

fn grava_folha_com(
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
fn grava_antes_da_e2(
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
fn oleo_do_env() -> ph2d_wet_paint::oleo_real::OleoReal {
    oleo_de(&std::env::var("PH2D_OLEO_AF").unwrap_or_default())
}

/// O Óleo Real de fábrica com os `campo=valor,…` de `af` (a língua do `PH2D_OLEO_AF`).
fn oleo_de(af: &str) -> ph2d_wet_paint::oleo_real::OleoReal {
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

fn pinta_folha() -> PainterTool {
    let b = biblioteca();
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (W * H * 4) as usize], W, H);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_relogio_fixo(true);
    pinta_folha_com(&mut t, &b);
    t
}

fn tracos() -> Vec<Traco> {
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

fn pinta_folha_com(t: &mut PainterTool, b: &Biblioteca) {
    for tr in &tracos() {
        let p = &b
            .entradas
            .iter()
            .find(|e| e.relativo == tr.preset)
            .unwrap_or_else(|| panic!("preset {} não existe", tr.preset))
            .preset;
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

fn imprime_alturas(t: &mut PainterTool) {
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

// ── W0b: a espátula exprime-se com os knobs de hoje? ────────────────────────────────────────────────
// Um traço horizontal (tamanho 120, 450 px, azul, pressão 1, passo 3 px, relógio fixo, espera a água) por
// candidata, num tool novo. Mede a faixa: largura (m>10), rampa da borda 90→10 % do patamar, patamar p50,
// ruído célula-a-célula (m − média 3×3) no miolo 60 %, e orla (máx a ≤5 px da borda ÷ patamar).

const FW: u32 = 700;
const FH: u32 = 320;
const FY: f32 = 160.0;
const FX0: f32 = 100.0;
const FX1: f32 = 550.0;

type Ajuste = fn(&mut PainterTool);

fn lisa_dura(t: &mut PainterTool) {
    t.paint.brush.jitter = 0.0;
    t.set_brush_hardness(1.0);
    t.set_brush_falloff(ph2d_painter_brush::Falloff::Constant as u8);
    t.set_brush_shape_kind(ph2d_painter_brush::TextureKind::None.to_u8());
}

fn quadrada(t: &mut PainterTool) {
    lisa_dura(t);
    t.set_brush_dab_quina(1.0);
}

fn quadrada_sem_dente(t: &mut PainterTool) {
    quadrada(t);
    t.paint.wetpaint.knobs.set(Knob::PaperGate, 0.0);
}

/// A lâmina: quadrada, lisa, dura, sem dente, achatada ATRAVÉS do traço (o fio da faca perpendicular à marcha).
fn lamina(t: &mut PainterTool) {
    quadrada_sem_dente(t);
    t.set_brush_dab_flatten(0.8);
    t.set_brush_dab_angle(90.0);
}

/// A lâmina com a curva Custom de queda: miolo a 0,6, pico 1 a 0,85 do raio, 0 na borda (cantos Vector) — a orla
/// pela composição, sem lei nova.
fn lamina_com_orla(t: &mut PainterTool) {
    lamina(t);
    t.set_brush_hardness(0.0);
    t.set_brush_falloff(ph2d_painter_brush::Falloff::Custom as u8);
    t.set_brush_falloff_point(0, 0.0, 0.6);
    t.set_brush_falloff_point(1, 1.0, 0.0);
    let pico = t.add_brush_falloff_point_at(0.85, 1.0).expect("ponto");
    for id in [0, 1, pico] {
        t.set_brush_falloff_point_handle(id, 1);
    }
}

fn candidatas() -> Vec<(&'static str, &'static str, Ajuste)> {
    const DAB: &str = "WetPainter/Oil/Dabber/Dabber.ph2d-brush.png";
    const FIL: &str = "WetPainter/Oil/Flat/Filbert.ph2d-brush.png";
    const MRK: &str = "WetPainter/Marker/Marker/Marker Bullet.ph2d-brush.png";
    vec![
        ("A Dabber fabrica", DAB, |_| {}),
        ("B +jitter0", DAB, |t| t.paint.brush.jitter = 0.0),
        ("C +dura1 Constant", DAB, |t| {
            t.paint.brush.jitter = 0.0;
            t.set_brush_hardness(1.0);
            t.set_brush_falloff(ph2d_painter_brush::Falloff::Constant as u8);
        }),
        ("D C+sem cerdas", DAB, lisa_dura),
        ("E D+quina1", DAB, quadrada),
        ("F E+gate0", DAB, quadrada_sem_dente),
        ("G F+flat.8 ang0", DAB, |t| {
            quadrada_sem_dente(t);
            t.set_brush_dab_flatten(0.8);
            t.set_brush_dab_angle(0.0);
        }),
        ("H F+flat.8 ang90", DAB, |t| {
            quadrada_sem_dente(t);
            t.set_brush_dab_flatten(0.8);
            t.set_brush_dab_angle(90.0);
        }),
        ("I F+lev0 cap0", DAB, |t| {
            quadrada_sem_dente(t);
            t.paint.wetpaint.knobs.set(Knob::Leveling, 0.0);
            t.paint.wetpaint.knobs.set(Knob::Capillary, 0.0);
        }),
        ("J F+lev2", DAB, |t| {
            quadrada_sem_dente(t);
            t.paint.wetpaint.knobs.set(Knob::Leveling, 2.0);
            t.paint.wetpaint.knobs.set(Knob::LevelingOwnStroke, 1.0);
        }),
        ("K F+agua0", DAB, |t| {
            quadrada_sem_dente(t);
            t.paint.wetpaint.knobs.water = 0.0;
        }),
        ("L F+pig500", DAB, |t| {
            quadrada_sem_dente(t);
            t.paint.wetpaint.knobs.set(Knob::PigmentPerDab, 500.0);
        }),
        ("M F+drag0 pick0", DAB, |t| {
            quadrada_sem_dente(t);
            t.paint.wetpaint.knobs.set(Knob::Drag, 0.0);
            t.paint.wetpaint.knobs.set(Knob::Pickup, 0.0);
        }),
        ("H2 H+pig500", DAB, |t| {
            lamina(t);
            t.paint.wetpaint.knobs.set(Knob::PigmentPerDab, 500.0);
        }),
        ("H3 H+lev0", DAB, |t| {
            lamina(t);
            t.paint.wetpaint.knobs.set(Knob::Leveling, 0.0);
        }),
        ("H4 H+edgeDark50", DAB, |t| {
            lamina(t);
            t.paint.wetpaint.knobs.set(Knob::EdgeDarkening, 50.0);
        }),
        ("H5 H+edgeDark200", DAB, |t| {
            lamina(t);
            t.paint.wetpaint.knobs.set(Knob::EdgeDarkening, 200.0);
        }),
        ("H6 H+heavyEdge1", DAB, |t| {
            lamina(t);
            t.set_wet_borda_que_pesa(1.0);
        }),
        ("H7 H+flat.95", DAB, |t| {
            lamina(t);
            t.set_brush_dab_flatten(0.95);
        }),
        ("R H sem quina", DAB, |t| {
            lamina(t);
            t.set_brush_dab_quina(0.0);
        }),
        ("S R+lev0", DAB, |t| {
            lamina(t);
            t.set_brush_dab_quina(0.0);
            t.paint.wetpaint.knobs.set(Knob::Leveling, 0.0);
        }),
        ("T D+gate0 lev0", DAB, |t| {
            lisa_dura(t);
            t.paint.wetpaint.knobs.set(Knob::PaperGate, 0.0);
            t.paint.wetpaint.knobs.set(Knob::Leveling, 0.0);
        }),
        ("U lamina curva-orla", DAB, lamina_com_orla),
        ("V U+lev0", DAB, |t| {
            lamina_com_orla(t);
            t.paint.wetpaint.knobs.set(Knob::Leveling, 0.0);
        }),
        ("N Filbert fabrica", FIL, |_| {}),
        ("O Filbert+F", FIL, quadrada_sem_dente),
        ("P Marker fabrica", MRK, |_| {}),
        ("Q Marker+F", MRK, quadrada_sem_dente),
    ]
}

fn mediana(mut v: Vec<f64>) -> f64 {
    pct(&mut v, 0.5)
}

/// Onde, andando de `de` para fora (passo `dir` = ±1 em linhas), a coluna `m` cruza `alvo`
/// pela primeira vez — em linhas fracionárias (interpolação linear), `None` se nunca.
fn cruza(m: &[f64], de: usize, dir: i64, alvo: f64) -> Option<f64> {
    let mut i = de as i64;
    loop {
        let j = i + dir;
        if j < 0 || j as usize >= m.len() {
            return None;
        }
        let (a, b) = (m[i as usize], m[j as usize]);
        if a >= alvo && b < alvo {
            let f = (a - alvo) / (a - b);
            return Some(i as f64 + dir as f64 * f);
        }
        i = j;
    }
}

#[test]
#[ignore = "medicao — corre-se à mão no perfil smoke"]
fn medida_oleo_real_faca_candidatas() {
    let b = biblioteca();
    println!(
        "{:<20} {:>5} {:>7} {:>7} {:>7} {:>8} {:>8} {:>7} {:>6} {:>6}  perfil topo (m/P, passos de 1 célula da borda 10% p/ dentro)",
        "candidata",
        "razão",
        "larg",
        "rampT",
        "rampB",
        "P p50",
        "ruído",
        "ruído%",
        "orlaT",
        "orlaB"
    );
    for (nome, preset, ajuste) in candidatas() {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (FW * FH * 4) as usize], FW, FH);
        t.set_paint_media(PaintMedia::WetPaint);
        t.set_wet_relogio_fixo(true);
        let p = &b
            .entradas
            .iter()
            .find(|e| e.relativo == preset)
            .unwrap_or_else(|| panic!("preset {preset} não existe"))
            .preset;
        let erros = t.wet_preset_aplicar(p);
        assert!(erros.is_empty(), "{erros:?}");
        t.set_brush_size_px(60.0);
        t.set_brush_color_srgb8(AZ);
        ajuste(&mut t);
        let n = ((FX1 - FX0) / 3.0).ceil() as u32;
        t.on_canvas_pointer(ponto([FX0, FY], PointerPhase::Down));
        for k in 1..=n {
            let x = FX0 + (FX1 - FX0) * k as f32 / n as f32;
            t.on_canvas_pointer(ponto([x, FY], PointerPhase::Move));
            t.on_tick(8.0);
        }
        t.on_canvas_pointer(ponto([FX1, FY], PointerPhase::Up));
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
        let s = t.paint.wetpaint.session.as_mut().expect("sessão");
        s.bring_home();
        let r = f64::from(s.ratio);
        let (gw, gh) = s.grid;
        let g = s.engine.active_grid();
        let st = g.s;
        let m = |cx: usize, cy: usize| {
            f64::from(g.sett[cx + cy * st]) + f64::from(g.susp[cx + cy * st])
        };
        // Colunas do meio do traço (px 200..450), em células.
        let c0 = (200.0 / r) as usize + 1;
        let c1 = ((450.0 / r) as usize + 1).min(gw);
        let yc = (f64::from(FY) / r) as usize + 1;
        let mut larg = vec![];
        let mut miolo = vec![];
        let mut cols: Vec<Vec<f64>> = vec![];
        for cx in c0..c1 {
            let col: Vec<f64> = (1..=gh).map(|cy| m(cx, cy)).collect();
            larg.push(col.iter().filter(|&&v| v > 10.0).count() as f64 * r);
            cols.push(col);
        }
        // O miolo 60 % da faixa (pela largura mediana), centrado no eixo do traço.
        let meia = mediana(larg.clone()) / r * 0.5;
        let (lo, hi) = (
            (yc as f64 - 0.6 * meia).ceil() as usize,
            (yc as f64 + 0.6 * meia).floor() as usize,
        );
        let mut ruido = vec![];
        for cx in c0..c1 {
            for cy in lo.max(2)..=hi.min(gh - 1) {
                miolo.push(m(cx, cy));
                let mut soma = 0.0;
                for dy in [-1i64, 0, 1] {
                    for dx in [-1i64, 0, 1] {
                        soma += m((cx as i64 + dx) as usize, (cy as i64 + dy) as usize);
                    }
                }
                ruido.push(m(cx, cy) - soma / 9.0);
            }
        }
        let plat = mediana(miolo).max(1e-9);
        // A ondulação: o desvio das médias por COLUNA (ao longo do traço) e por LINHA (através), ÷ patamar.
        let dp = |v: &[f64]| {
            let n = v.len().max(1) as f64;
            let mu = v.iter().sum::<f64>() / n;
            (v.iter().map(|x| (x - mu) * (x - mu)).sum::<f64>() / n).sqrt()
        };
        let por_col: Vec<f64> = (c0..c1)
            .map(|cx| {
                (lo.max(2)..=hi.min(gh - 1))
                    .map(|cy| m(cx, cy))
                    .sum::<f64>()
                    / (hi + 1 - lo) as f64
            })
            .collect();
        let por_lin: Vec<f64> = (lo.max(2)..=hi.min(gh - 1))
            .map(|cy| (c0..c1).map(|cx| m(cx, cy)).sum::<f64>() / (c1 - c0) as f64)
            .collect();
        let (ond_x, ond_y) = (100.0 * dp(&por_col) / plat, 100.0 * dp(&por_lin) / plat);
        let nr = ruido.len().max(1) as f64;
        let mu = ruido.iter().sum::<f64>() / nr;
        let sd = (ruido.iter().map(|v| (v - mu) * (v - mu)).sum::<f64>() / nr).sqrt();
        // Rampa e orla por coluna (índice da coluna = cy − 1).
        let (mut rt, mut rb, mut ot, mut ob) = (vec![], vec![], vec![], vec![]);
        let i0 = yc - 1;
        let orla_cel = (5.0 / r).ceil() as i64;
        let mut perfil_topo: Vec<Vec<f64>> = vec![];
        for col in &cols {
            for (dir, rampa, orla) in [(-1i64, &mut rt, &mut ot), (1i64, &mut rb, &mut ob)] {
                // A borda: andar do eixo para fora até cair abaixo de 10 %; o 90 % procura-se a partir daí PARA DENTRO
                // (a última vez que estava acima de 90 % antes da borda), para a orla não confundir.
                let Some(e10) = cruza(col, i0, dir, 0.1 * plat) else {
                    continue;
                };
                let k10 = e10.round() as i64;
                let mut e90 = None;
                let mut i = k10;
                while (i - i0 as i64) * dir > 0 {
                    let v = col[i as usize];
                    if v >= 0.9 * plat {
                        let prox = col[(i + dir) as usize];
                        e90 = Some(
                            i as f64
                                + dir as f64
                                    * ((v - 0.9 * plat) / (v - prox).max(1e-9)).clamp(0.0, 1.0),
                        );
                        break;
                    }
                    i -= dir;
                }
                if let Some(e90) = e90 {
                    rampa.push((e10 - e90).abs() * r);
                }
                let mx = (0..=orla_cel)
                    .map(|d| col[(k10 - dir * d).clamp(0, col.len() as i64 - 1) as usize])
                    .fold(0.0f64, f64::max);
                orla.push(mx / plat);
                if dir == -1 {
                    perfil_topo.push(
                        (0..12)
                            .map(|d| col[(k10 + d).clamp(0, col.len() as i64 - 1) as usize] / plat)
                            .collect(),
                    );
                }
            }
        }
        let perfil: Vec<String> = (0..12)
            .map(|d| format!("{:.2}", mediana(perfil_topo.iter().map(|p| p[d]).collect())))
            .collect();
        println!(
            "{:<20} {:>5} {:>7.1} {:>7.1} {:>7.1} {:>8.1} {:>8.1} {:>7.1} {:>6.2} {:>6.2}  {}  (ondX {:.1}% ondY {:.1}%, n rampa {}/{}, água {} ticks)",
            nome,
            r,
            mediana(larg),
            mediana(rt.clone()),
            mediana(rb.clone()),
            plat,
            sd,
            100.0 * sd / plat,
            mediana(ot),
            mediana(ob),
            perfil.join(" "),
            ond_x,
            ond_y,
            rt.len(),
            cols.len(),
            ticks
        );
    }
}

// ── Acúmulo: o relevo do Óleo Real sobe com as passagens? ───────────────────────────────────────────
// O MESMO traço horizontal (tamanho 100, azul, pressão 1, passo 3 px, relógio fixo, espera a água) 1·2·4·8
// vezes, empilhado (`dy = 0`) ou desfasado 15 px por passagem (`dy = 15`). Mede na faixa central do traço
// (x 200..500, ±15 px do eixo médio): massa sett+susp por camada e total, a altura da FORMA (`CampoAltura::at`)
// e a fracção de células com tinta com `luz < 0,9` (sombra própria), ao Óleo Real de fábrica.

const AW: u32 = 700;
const AH: u32 = 420;
const AY: f32 = 130.0;
const AX0: f32 = 100.0;
const AX1: f32 = 600.0;

fn um_traco(t: &mut PainterTool, y: f32) -> u32 {
    let n = ((AX1 - AX0) / 3.0).ceil() as u32;
    t.on_canvas_pointer(ponto([AX0, y], PointerPhase::Down));
    for k in 1..=n {
        let x = AX0 + (AX1 - AX0) * k as f32 / n as f32;
        t.on_canvas_pointer(ponto([x, y], PointerPhase::Move));
        t.on_tick(8.0);
        if std::env::var("PH2D_OLEO_ACUMULO_TRACO").is_ok()
            && k % 6 == 0
            && (300.0..=420.0).contains(&x)
        {
            let s = t.paint.wetpaint.session.as_mut().expect("sessão");
            s.bring_home();
            let r = f32::from(s.ratio);
            let g = s.engine.active_grid();
            let i = ((350.0 / r) as usize + 1) + ((y / r) as usize + 1) * g.s;
            println!(
                "[traço] x={x:>5.0} célula(350,{y}) sett {:>7.0} susp {:>7.0} film {:>6.2}",
                g.sett[i], g.susp[i], g.film[i]
            );
        }
    }
    t.on_canvas_pointer(ponto([AX1, y], PointerPhase::Up));
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

fn imprime_acumulo(t: &mut PainterTool, rot: &str, passes: usize, yc: f32) {
    // As variantes da altura da SOMBRA no MESMO processo, sobre a MESMA tinta (W1b passo 2).
    for raio in [3usize, 1, 0] {
        let mut o = ph2d_wet_paint::oleo_real::OleoReal::default();
        o.af.raio_sombra = raio;
        imprime_acumulo_com(t, &format!("{rot} rs{raio}"), passes, yc, &o);
    }
}

fn imprime_acumulo_com(
    t: &mut PainterTool,
    rot: &str,
    passes: usize,
    yc: f32,
    o: &ph2d_wet_paint::oleo_real::OleoReal,
) {
    let o = *o;
    let s = t.paint.wetpaint.session.as_mut().expect("sessão");
    s.bring_home();
    let r = f32::from(s.ratio);
    let (gw, gh) = s.grid;
    let grades: Vec<(&ph2d_wet_paint::grid::Grid, bool)> = s
        .engine
        .layers
        .iter()
        .map(|l| (&l.grid, l.visible))
        .collect();
    let nl = grades.len();
    let campo =
        ph2d_wet_paint::oleo_real::CampoAltura::de(&grades, &o, usize::from(s.ratio), 1, 1, gw, gh);
    let st = grades[0].0.s;
    let (c0, c1) = ((200.0 / r) as usize + 1, ((500.0 / r) as usize + 1).min(gw));
    let (l0, l1) = (
        ((yc - 15.0) / r) as usize + 1,
        (((yc + 15.0) / r) as usize + 1).min(gh),
    );
    let mut por_camada: Vec<Vec<f64>> = vec![vec![]; nl];
    let (mut tot, mut alt, mut susp, mut horiz) = (vec![], vec![], vec![], vec![]);
    let (mut com_tinta, mut sombra) = (0usize, 0usize);
    for cy in l0..=l1 {
        for cx in c0..=c1 {
            let i = cx + cy * st;
            let mut m = 0.0;
            let mut su = 0.0;
            for (k, (g, vis)) in grades.iter().enumerate() {
                let v = f64::from(g.sett[i]) + f64::from(g.susp[i]);
                por_camada[k].push(v);
                if *vis {
                    m += v;
                    su += f64::from(g.susp[i]);
                }
            }
            tot.push(m);
            susp.push(su);
            alt.push(campo.at_sombra(cx as isize, cy as isize));
            let (x, y) = (cx as isize, cy as isize);
            // A subida máxima (células de altura por célula) nos 24 passos para a luz (topo-esquerda).
            let mut sobe = f64::NEG_INFINITY;
            for k in 1..=24isize {
                let d = (k as f64 * core::f64::consts::FRAC_1_SQRT_2).round() as isize;
                sobe = sobe.max((campo.at_sombra(x - d, y - d) - campo.at_sombra(x, y)) / k as f64);
            }
            horiz.push(sobe);
            if m > 0.0 {
                com_tinta += 1;
                if ph2d_wet_paint::oleo_real::termos(&o, &campo, cx, cy).luz < 0.9 {
                    sombra += 1;
                }
            }
        }
    }
    // A sombra própria em TODA a área do traço (não só a faixa central): onde cai?
    let (mut tinta_all, mut sombra_all, mut sombra_borda) = (0usize, 0usize, 0usize);
    let (b0, b1) = (
        ((yc - 80.0).max(0.0) / r) as usize + 1,
        (((yc + 130.0) / r) as usize + 1).min(gh),
    );
    for cy in b0..=b1 {
        for cx in c0..=c1 {
            let i = cx + cy * st;
            let m: f64 = grades
                .iter()
                .filter(|g| g.1)
                .map(|(g, _)| f64::from(g.sett[i]) + f64::from(g.susp[i]))
                .sum();
            if m > 0.0 {
                tinta_all += 1;
                if ph2d_wet_paint::oleo_real::termos(&o, &campo, cx, cy).luz < 0.9 {
                    sombra_all += 1;
                    // borda = algum vizinho a ≤ 3 células sem tinta
                    let perto_papel = (-3i64..=3).any(|d| {
                        let j = (cx as i64 + (cy as i64 + d) * st as i64) as usize;
                        grades
                            .iter()
                            .filter(|g| g.1)
                            .map(|(g, _)| f64::from(g.sett[j]) + f64::from(g.susp[j]))
                            .sum::<f64>()
                            <= 0.0
                    });
                    if perto_papel {
                        sombra_borda += 1;
                    }
                }
            }
        }
    }
    let camadas: Vec<String> = por_camada
        .iter_mut()
        .map(|v| format!("{:.0}/{:.0}/{:.0}", pct(v, 0.5), pct(v, 0.9), pct(v, 1.0)))
        .collect();
    println!(
        "{rot:<26} {passes:>2} r{r} | {:>7.0} {:>7.0} {:>7.0} | susp p50 {:>6.0} | {:>6.3} {:>6.3} {:>6.3} | horiz p50 {:>6.3} p99 {:>6.3} | sombra {:>5.1}% ({com_tinta}) | área {:>5.1}% borda {:>5.1}% de {tinta_all} | camadas {nl} [{}]",
        pct(&mut tot, 0.5),
        pct(&mut tot, 0.9),
        pct(&mut tot, 1.0),
        pct(&mut susp, 0.5),
        pct(&mut alt, 0.5),
        pct(&mut alt, 0.9),
        pct(&mut alt, 1.0),
        pct(&mut horiz, 0.5),
        pct(&mut horiz, 0.99),
        100.0 * sombra as f64 / com_tinta.max(1) as f64,
        100.0 * sombra_all as f64 / tinta_all.max(1) as f64,
        100.0 * sombra_borda as f64 / sombra_all.max(1) as f64,
        camadas.join(" · "),
    );
}

#[test]
#[ignore = "medicao — corre-se à mão no perfil smoke"]
fn medida_oleo_real_acumulo() {
    let b = biblioteca();
    println!(
        "{:<26} {:>2} | {:>7} {:>7} {:>7} | {:>13} | {:>6} {:>6} {:>6} | sombra própria (luz<0,9) na faixa · área · borda",
        "preset/variante", "n", "m p50", "m p90", "m max", "", "h p50", "h p90", "h max"
    );
    for preset in [
        "WetPainter/Oil/Dabber/Dabber.ph2d-brush.png",
        "WetPainter/Oil/Bristle/Oil Bristle.ph2d-brush.png",
    ] {
        let p = &b
            .entradas
            .iter()
            .find(|e| e.relativo == preset)
            .expect("preset")
            .preset;
        for dy in [0.0f32, 15.0] {
            let mut t = PainterTool::default();
            t.set_source(vec![255u8; (AW * AH * 4) as usize], AW, AH);
            t.set_paint_media(PaintMedia::WetPaint);
            t.set_wet_relogio_fixo(true);
            let erros = t.wet_preset_aplicar(p);
            assert!(erros.is_empty(), "{erros:?}");
            t.set_brush_size_px(50.0);
            t.set_brush_color_srgb8(AZ);
            let nome = preset
                .rsplit('/')
                .next()
                .unwrap()
                .trim_end_matches(".ph2d-brush.png");
            let rot = format!("{nome} dy{dy}");
            for k in 0..8usize {
                let ticks = um_traco(&mut t, AY + dy * k as f32);
                let n = k + 1;
                if [1, 2, 4, 8].contains(&n) {
                    let yc = AY + dy * (k as f32) * 0.5;
                    imprime_acumulo(&mut t, &format!("{rot} (água {ticks})"), n, yc);
                }
            }
        }
    }
}

/// O custo por célula da óptica na CPU (o caminho da pintura ao vivo até haver passe de placa): os
/// ambientes no MESMO processo, intercalados, o MÍNIMO de 7 rodadas (`docs/DevOps/MEDIR_VELOCIDADE.md`).
#[test]
#[ignore = "medicao — corre-se à mão no perfil smoke"]
fn medida_oleo_real_custo() {
    use ph2d_wet_paint::oleo_real::{Ambiente, CampoAltura, OleoReal, termos_com};
    let mut t = pinta_folha();
    let s = t.paint.wetpaint.session.as_mut().expect("sessão");
    s.bring_home();
    let (gw, gh) = s.grid;
    let grades: Vec<(&ph2d_wet_paint::grid::Grid, bool)> = s
        .engine
        .layers
        .iter()
        .map(|l| (&l.grid, l.visible))
        .collect();
    let campo = CampoAltura::de(
        &grades,
        &OleoReal::default(),
        usize::from(s.ratio),
        1,
        1,
        gw,
        gh,
    );
    // W2 E3-B: os quatro ambientes com o material NEUTRO e o Clássico e o Estúdio com um material CHEIO
    // (todos os lóbulos a meio), intercalados.
    let cheio = ph2d_wet_paint::material::MaterialCelula {
        metal: 0.5,
        verniz: 0.5,
        veludo: 0.5,
        subsuperficie: 0.5,
        ..ph2d_wet_paint::material::MaterialCelula::NEUTRO
    };
    let neutro = ph2d_wet_paint::material::MaterialCelula::NEUTRO;
    let variantes: Vec<(String, Ambiente, ph2d_wet_paint::material::MaterialCelula)> =
        Ambiente::TODOS
            .iter()
            .map(|&a| (format!("{a:?}"), a, neutro))
            .chain(
                [Ambiente::Classico, Ambiente::Estudio].map(|a| (format!("{a:?}+cheio"), a, cheio)),
            )
            .collect();
    let nv = variantes.len();
    let mut melhor = vec![f64::INFINITY; nv];
    let mut soma = 0.0;
    for rodada in 0..7 {
        for k in 0..nv {
            let i = (k + rodada) % nv;
            let (_, a, m) = &variantes[i];
            let o = OleoReal {
                ambiente: *a,
                ..OleoReal::default()
            };
            let t0 = std::time::Instant::now();
            for cy in (1..=gh).step_by(4) {
                for cx in (1..=gw).step_by(4) {
                    soma += termos_com(&o, &campo, cx, cy, m, [0.6, 0.1, 0.1]).vis;
                }
            }
            let ns = t0.elapsed().as_nanos() as f64 / ((gw / 4) * (gh / 4)) as f64;
            melhor[i] = melhor[i].min(ns);
        }
    }
    println!(
        "[custo] ns/célula (mínimo de 7): {:?} · {soma:.0}",
        variantes
            .iter()
            .map(|v| &v.0)
            .zip(melhor)
            .collect::<Vec<_>>()
    );
    // W3 M1: o CAMPO com a Subsurface (a irradiância da região alargada + as duas passagens), na tela inteira e
    // numa janela de pincelada (64×64 + o alcance), com o raio no padrão, a 16 e sem ela — intercalados.
    let (mx, my) = (gw / 2, gh / 2);
    let janelas = [("tela", (1, 1, gw, gh)), ("pincelada", (mx - 32, my - 32, mx + 32, my + 32))];
    let raios = [("sem", 0.0, 4.0), ("r4", 1.0, 4.0), ("r16", 1.0, 16.0)];
    let mut melhor = [[f64::INFINITY; 3]; 2];
    for rodada in 0..5 {
        for (j, &(_, (x0, y0, x1, y1))) in janelas.iter().enumerate() {
            for k in 0..3 {
                let i = (k + rodada) % 3;
                let mut o = OleoReal::default();
                o.material.subsuperficie = raios[i].1;
                o.material.raio_subsuperficie = raios[i].2;
                let a = ph2d_wet_paint::oleo_real::alcance(&o);
                let (jx0, jy0) = (x0.saturating_sub(a).max(1), y0.saturating_sub(a).max(1));
                let (jx1, jy1) = ((x1 + a).min(gw), (y1 + a).min(gh));
                let t0 = std::time::Instant::now();
                let c = CampoAltura::de(&grades, &o, usize::from(s.ratio), jx0, jy0, jx1, jy1);
                soma += c.subsuperficie_em(mx, my)[0];
                let ns = t0.elapsed().as_nanos() as f64 / ((x1 + 1 - x0) * (y1 + 1 - y0)) as f64;
                melhor[j][i] = melhor[j][i].min(ns);
            }
        }
    }
    for (j, (nome, _)) in janelas.iter().enumerate() {
        println!(
            "[custo] campo {nome}, ns por célula PEDIDA (mínimo de 5): {:?} · {soma:.0}",
            raios.iter().map(|r| r.0).zip(melhor[j]).collect::<Vec<_>>()
        );
    }
    // K4: o RECOMPOR de uma pincelada (o campo + o sombreado e a sombra no papel de cada célula pedida) sem
    // Subsurface, com ela e a luz do céu GUARDADA, e com ela contada de novo (`esquece_luz_do_ceu`) — intercalados.
    let modos = ["sem", "r4 guardada", "r4 contada"];
    let mut melhor = [f64::INFINITY; 3];
    let (x0, y0, x1, y1) = (mx - 32, my - 32, mx + 32, my + 32);
    for rodada in 0..5 {
        for k in 0..3 {
            let i = (k + rodada) % 3;
            let mut o = OleoReal::default();
            if i > 0 {
                o.material.subsuperficie = 1.0;
            }
            let t0 = std::time::Instant::now();
            let mut c = CampoAltura::de(&grades, &o, usize::from(s.ratio), x0, y0, x1, y1);
            if i == 2 {
                c.esquece_luz_do_ceu();
            }
            for cy in y0..=y1 {
                for cx in x0..=x1 {
                    let m = c.material_em(cx, cy);
                    soma += termos_com(&o, &c, cx, cy, &m, [0.6, 0.1, 0.1]).vis;
                    soma += ph2d_wet_paint::oleo_real::sombra_no_papel(&o, &c, cx, cy);
                }
            }
            let ns = t0.elapsed().as_nanos() as f64 / ((x1 + 1 - x0) * (y1 + 1 - y0)) as f64;
            melhor[i] = melhor[i].min(ns);
        }
    }
    println!(
        "[custo] recompor da pincelada 65², ns por célula (mínimo de 5): {:?} · {soma:.0}",
        modos.iter().zip(melhor).collect::<Vec<_>>()
    );
    println!("[custo] loadavg {}", std::fs::read_to_string("/proc/loadavg").unwrap_or_default());
}

/// A cor final (sRGB, sobre o papel com a sombra do Óleo Real, como em [`grava_folha`]) e a cor SEM relevo
/// de cada célula, com a cobertura: `(final, albedo, alfa)`.
fn render_da_cena(
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
