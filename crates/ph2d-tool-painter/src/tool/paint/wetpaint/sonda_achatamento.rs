//! **Sonda (ignorada, W1f): o ACHATAMENTO do escorrimento** — o S vermelho do smoke do dono (10/10): o
//! produto, a lei da gota da W1e (o que cortava em rectângulos) e a W1e inteira (a lei + a água no traço:
//! a captura do dono), no MESMO processo. As ~40 variantes da auditoria estão no 05 §4b. A cena é a do oráculo
//! (`fixtures/rebelle_gotas/s_*.png`, `ferramentas/gotas.py --forma s`): tela 1000×640, o S
//! `y = 180 − 85·sin θ` reamostrado a 3 px de arco, o Wet Round a 100 px, água 1, 30 s de água.
//!
//! Cada cena grava em `PH2D_SONDA_OUT`: `a_<variante>_<tilt>.ppm` (o ecrã — o composite incremental, o que
//! o dono vê) e `a_<variante>_<tilt>_full.ppm` (o MESMO motor recomposto inteiro). `PH2D_SONDA_SO` filtra as
//! variantes por nome (lista separada por vírgulas). A régua: `ferramentas/recto.py`.
use crate::tool::PainterTool;
use ph2d_brush_presets::{Biblioteca, Volumes};
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_wet_paint::sim::Sim;
use ph2d_wet_paint::tuning::Knob;

pub(super) const W: usize = 1000;
pub(super) const H: usize = 640;

/// O S do oráculo: `y = 180 − 85·sin θ`, `θ ∈ [−π/2, 2π + π/4]`, x 100 → 900, um ponto a cada 3 px de arco.
pub(super) fn o_s() -> Vec<[f32; 2]> {
    let (x0, x1, yc, amp) = (100.0f64, 900.0, 180.0, 85.0);
    let (t0, t1) = (
        -std::f64::consts::FRAC_PI_2,
        2.0 * std::f64::consts::PI + std::f64::consts::FRAC_PI_4,
    );
    let fino: Vec<(f64, f64)> = (0..=8000)
        .map(|k| {
            let u = f64::from(k) / 8000.0;
            (x0 + (x1 - x0) * u, yc - amp * (t0 + (t1 - t0) * u).sin())
        })
        .collect();
    let mut pts = vec![[fino[0].0 as f32, fino[0].1 as f32]];
    let mut acc = 0.0;
    for w in fino.windows(2) {
        acc += (w[1].0 - w[0].0).hypot(w[1].1 - w[0].1);
        if acc >= 3.0 {
            pts.push([w[1].0 as f32, w[1].1 as f32]);
            acc = 0.0;
        }
    }
    pts
}

fn ponto(p: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos: p,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// O que uma variante mexe: os knobs, a lei da gota (`drip_head`; o produto não a usa), a faixa viva, a
/// água no traço.
#[derive(Clone, Default)]
pub(super) struct Variante {
    pub knobs: Vec<(Knob, f64)>,
    pub gota: Option<f64>,
    pub sem_faixa: bool,
    /// `Some(true)` = a água corre com o traço aberto (a W1e; o produto desligou-a na W1f).
    pub agua_no_traco: Option<bool>,
}

impl Variante {
    pub(super) fn ajusta(&self, sim: &mut Sim) {
        if let Some(h) = self.gota {
            sim.drip_head = h;
        }
        if let Some(a) = self.agua_no_traco {
            sim.sim_while_painting = a;
        }
    }
}

/// A cena: `tilt` = `None` (direita) ou `(raio, anel)`; devolve (o ecrã, a recomposição inteira) e chama
/// `quadro(meios_segundos, tool)` a cada meio segundo de água.
pub(super) fn cena(
    v: &Variante,
    tilt: Option<(u8, u8)>,
    segundos: u32,
    quadro: &mut dyn FnMut(u32, &mut PainterTool),
) -> (Vec<u8>, Vec<u8>) {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let b = Biblioteca::carregar(&raiz, None);
    let p = &b
        .entradas
        .iter()
        .find(|e| e.preset.parametros.name == "Wet Round")
        .expect("o pincel de fábrica")
        .preset;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    assert!(t.wet_preset_aplicar(p).is_empty());
    t.wet_preset_aplicar_volumes(&Volumes {
        size_px: Some(100.0),
        water: Some(1.0),
        ..Volumes::default()
    });
    for (k, val) in &v.knobs {
        t.paint.wetpaint.knobs.set(*k, *val);
    }
    t.paint.wetpaint.tilt_on = tilt.is_some();
    let (raio, anel) = tilt.unwrap_or((3, 4));
    t.paint.wetpaint.tilt_spoke = raio;
    t.paint.wetpaint.tilt_ring = anel;
    t.set_brush_color_srgb8([200, 25, 25]);
    assert!(t.ensure_wet_session());
    {
        let s = t.paint.wetpaint.session.as_mut().expect("sessão");
        v.ajusta(&mut s.engine.sim);
        if v.sem_faixa {
            s.engine.active_grid_mut().spans_enabled = false;
        }
    }
    let pts = o_s();
    t.on_canvas_pointer(ponto(pts[0], PointerPhase::Down));
    for &q in &pts[1..] {
        t.on_canvas_pointer(ponto(q, PointerPhase::Move));
        t.on_tick(8.0);
    }
    t.on_canvas_pointer(ponto(*pts.last().expect("o S"), PointerPhase::Up));
    for k in 1..=segundos * 60 {
        t.on_tick(1000.0 / 60.0);
        if k.is_multiple_of(30) {
            quadro(k / 30, &mut t);
        }
    }
    let tela = t.canvas_rgba.as_ref().clone();
    {
        let s = t.paint.wetpaint.session.as_mut().expect("sessão");
        s.bring_home();
        s.engine.mark_dirty_full();
    }
    t.wetpaint_composite();
    (tela, t.run_full().0)
}

pub(super) fn grava(caminho: &str, px: &[u8]) {
    let mut f = format!("P6\n{W} {H}\n255\n").into_bytes();
    for p in px.as_chunks::<4>().0 {
        f.extend_from_slice(&p[..3]);
    }
    std::fs::write(caminho, f).expect("grava");
}

#[test]
#[ignore = "sonda: as hipóteses do achatamento (prompt 12); corre-se à mão"]
fn sonda_achatamento() {
    let out = std::env::var("PH2D_SONDA_OUT").unwrap_or_else(|_| "/tmp".into());
    let so = std::env::var("PH2D_SONDA_SO").ok();
    let variantes: Vec<(&str, Variante)> = vec![
        ("produto", Variante::default()),
        (
            "lei",
            Variante {
                gota: Some(ph2d_wet_paint::solver::ALTURA_DA_GOTA),
                ..Variante::default()
            },
        ),
        (
            "vivo",
            Variante {
                gota: Some(ph2d_wet_paint::solver::ALTURA_DA_GOTA),
                agua_no_traco: Some(true),
                ..Variante::default()
            },
        ),
    ];
    let tilts: [(&str, Option<(u8, u8)>); 4] = [
        ("direita", None),
        ("meio", Some((3, 4))),
        ("max", Some((3, 8))),
        ("diag", Some((2, 8))),
    ];
    for (nome, v) in &variantes {
        if so
            .as_ref()
            .is_some_and(|s| !s.split(',').any(|x| x == *nome))
        {
            continue;
        }
        for (tn, tilt) in tilts {
            let filme = *nome == "lei" && tn == "meio";
            let marcas: &[u32] = if filme { &[1, 3, 8, 15] } else { &[] };
            let mut quadro = |meio_s: u32, t: &mut PainterTool| {
                if meio_s.is_multiple_of(2) && marcas.contains(&(meio_s / 2)) {
                    grava(
                        &format!("{out}/a_{nome}_{tn}_t{:02}.ppm", meio_s / 2),
                        &t.canvas_rgba,
                    );
                }
            };
            let (tela, full) = cena(v, tilt, 30, &mut quadro);
            let difere = tela
                .as_chunks::<4>()
                .0
                .iter()
                .zip(full.as_chunks::<4>().0)
                .filter(|(a, b)| a != b)
                .count();
            println!("[achatamento] {nome} {tn}: ecrã ≠ recomposição em {difere} px");
            grava(&format!("{out}/a_{nome}_{tn}.ppm"), &tela);
            grava(&format!("{out}/a_{nome}_{tn}_full.ppm"), &full);
        }
        for (tn, anel) in [("off", None), ("meio", Some(4)), ("max", Some(8))] {
            let px = super::sonda_gota::bancada_com(
                0.005,
                v.gota.unwrap_or(0.0),
                &v.knobs,
                anel,
                [1.0, 0.7, 0.4, 0.2],
                &|sim| v.ajusta(sim),
            );
            let mut f = format!("P6\n{} {}\n255\n", 1000, 640).into_bytes();
            for p in px.as_chunks::<4>().0 {
                f.extend_from_slice(&p[..3]);
            }
            std::fs::write(format!("{out}/b_{nome}_{tn}.ppm"), f).expect("grava");
        }
    }
}

/// **Sonda (W1f): a PONTA de um fio a crescer**, célula a célula — por coluna, a linha mais baixa com
/// água, a altura dela, e o fluxo que desce na célula seca logo abaixo. `PH2D_SONDA_X=a:b` as colunas.
#[test]
#[ignore = "sonda: corre-se à mão"]
fn sonda_ponta() {
    let (xa, xb) = std::env::var("PH2D_SONDA_X")
        .ok()
        .and_then(|v| {
            let (a, b) = v.split_once(':')?;
            Some((a.parse().ok()?, b.parse().ok()?))
        })
        .unwrap_or((640usize, 672usize));
    for (nome, v) in [
        ("produto", Variante::default()),
        (
            "lei",
            Variante {
                gota: Some(ph2d_wet_paint::solver::ALTURA_DA_GOTA),
                ..Variante::default()
            },
        ),
    ] {
        let mut quadro = |meio_s: u32, t: &mut PainterTool| {
            if meio_s < 16 {
                return;
            }
            let s = t.paint.wetpaint.session.as_mut().expect("sessão");
            s.bring_home();
            let g = s.engine.active_grid();
            let mut ponta = String::new();
            let mut filme = String::new();
            let mut fluxo = String::new();
            for x in xa..xb {
                let y = (200..400)
                    .rev()
                    .find(|&y| g.film[x + y * g.s] > 1e-3)
                    .unwrap_or(0);
                ponta.push_str(&format!("{:4}", y));
                filme.push_str(&format!("{:4.0}", g.film[x + y * g.s]));
                let i = x + (y + 1) * g.s;
                fluxo.push_str(&format!("{:4.0}", 100.0 * g.flow_y[i]));
            }
            println!(
                "[ponta {nome} t{:4.1}] linha {ponta}",
                f64::from(meio_s) / 2.0
            );
            println!(
                "[ponta {nome} t{:4.1}] filme {filme}",
                f64::from(meio_s) / 2.0
            );
            println!(
                "[ponta {nome} t{:4.1}] fy%   {fluxo}",
                f64::from(meio_s) / 2.0
            );
        };
        let _ = cena(&v, Some((3, 4)), 30, &mut quadro);
    }
}
