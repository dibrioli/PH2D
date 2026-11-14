//! **Relevo do óleo grosso sobreposto: ondas e crateras** (queixa do dono 31/10: «camadas grossas que ao se
//! sobreporem provocam um assentamento feio da tinta… mesmo com baixa quantidade de água se comportam como líquidos e
//! provocam ondas e crateras»). Papel texturizado PADRÃO do app; camada 1 horizontal grossa, pausa, camada 2 a cruzar
//! (perpendicular, ou paralela meio sobreposta), pausa. Mede no campo de ALTURA do Óleo Real (o que a luz lê):
//! crateras (altura final < a da camada 1 − limiar) dentro e fora do contacto do pincel 2, ondas (RMS do passa-alto
//! ao longo do traço 2 ÷ o do traço 2 sozinho), deriva fora do contacto e o balanço de massa. Variantes no MESMO
//! processo (`PH2D_SONDA_VAR`): `prod`, `sem_ladeira` (rota 2), `sem_rewet` (Knob Rewet 0), `sem_level`
//! (Knob Leveling 0), `sem_bingham` (a lei antes da E7), `b<τ₀>_<φ_c>` (outras constantes de Bingham). Doc: `docs/Painter/Arquitetura/RELEVO_oleo_ondas_e_crateras.md`.
//! `PH2D_SONDA_OUT=<dir> bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --profile smoke --lib sonda_relevo_oleo -- --ignored --nocapture`
use super::sonda_oleo_grumos::{alturas, grava_png};
use super::*;
use crate::{PaintMedia, PainterTool};
use ph2d_brush_presets::Biblioteca;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PanelEvent, PointerPhase};
use ph2d_wet_paint::tuning::Knob;

const W: u32 = 360;
const H: u32 = 280;
const AZ: [u8; 3] = [40, 90, 200];
const AM: [u8; 3] = [230, 190, 30];
const ENTIDADE: u64 = 77;
/// O traço 1: horizontal em y = 130.
const Y1: f32 = 130.0;

#[derive(Clone, Copy, PartialEq)]
enum Cruz {
    Perp,
    Par,
}

fn preset(b: &Biblioteca, rel: &str) -> ph2d_brush_presets::Preset {
    let rel = format!("WetPainter/{rel}.ph2d-brush.png");
    b.entradas
        .iter()
        .find(|e| e.relativo == rel)
        .unwrap_or_else(|| panic!("preset {rel}"))
        .preset
        .clone()
}

thread_local! {
    /// A lei da tinta da célula (`PH2D_SONDA_LEI`, F4-2 E2), posta a cada traço e tique — o preset não a toca.
    static LEI: std::cell::Cell<ph2d_wet_paint::sim::LeiDaTinta> = const { std::cell::Cell::new(ph2d_wet_paint::sim::LeiDaTinta::Velha) };
}

fn var_aplica(t: &mut PainterTool, var: &str) {
    let Some(s) = t.paint.wetpaint.session.as_mut() else {
        return;
    };
    let e = &mut s.engine;
    e.sim.set_tinta(LEI.get());
    for var in var.split('+') {
        match var {
            "sem_rewet" => {
                e.tuning.set(Knob::Rewet, 0.0);
            }
            "sem_level" => {
                e.tuning.set(Knob::Leveling, 0.0);
            }
            "sem_pickup" => {
                e.tuning.set(Knob::Pickup, 0.0);
            }
            "sem_smear" => {
                e.tuning.set(Knob::Smear, 0.0);
            }
            "sem_lift" => {
                e.tuning.set(Knob::WetLift, 0.0);
            }
            "sem_drag" => {
                e.tuning.set(Knob::Drag, 0.0);
            }
            "sem_wetopen" => {
                e.tuning.set(Knob::WetOpen, 0.0);
            }
            "sem_dissolve" => {
                e.tuning.set(Knob::Dissolve, 0.0);
            }
            _ => {}
        }
    }
}

/// Os interruptores do DEPÓSITO (Mecanismo 2): `sem_espalha` (o 1.º toque não baixa a espessura de baixo),
/// `sem_teto` (o tecto suave de 3000 desligado), `sem_piso` (a lei antes da cura: o espalhar podia cavar). Combinam-se com `+`.
fn var_globais(var: &str) {
    let tem = |n: &str| var.split('+').any(|v| v == n);
    ph2d_wet_paint::trail::espalha::sobrepoe(tem("sem_espalha").then_some(0.0));
    ph2d_wet_paint::trail::sonda::SEM_TETO
        .store(tem("sem_teto"), std::sync::atomic::Ordering::Relaxed);
    ph2d_wet_paint::trail::sonda::SEM_PISO
        .store(tem("sem_piso"), std::sync::atomic::Ordering::Relaxed);
}

fn tique(t: &mut PainterTool, var: &str) {
    var_aplica(t, var);
    t.on_tick(1000.0 / 60.0);
}

fn traco(t: &mut PainterTool, a: [f32; 2], b: [f32; 2], pressure: f32, var: &str) {
    let p = |pos, phase| CanvasPointer {
        pos,
        pressure,
        tilt: [0.0, 0.0],
        phase,
    };
    let n = ((b[0] - a[0]).hypot(b[1] - a[1]) / 3.0).ceil() as u32;
    t.on_canvas_pointer(p(a, PointerPhase::Down));
    for k in 1..=n {
        let s = k as f32 / n as f32;
        t.on_canvas_pointer(p(
            [a[0] + (b[0] - a[0]) * s, a[1] + (b[1] - a[1]) * s],
            PointerPhase::Move,
        ));
        tique(t, var);
    }
    t.on_canvas_pointer(p(b, PointerPhase::Up));
}

#[derive(Clone, Copy)]
struct Pedido<'a> {
    pincel: &'a str,
    agua: f64,
    pressao: f32,
    n: usize,
    pausa: u32,
    cruz: Cruz,
}

fn ferramenta() -> PainterTool {
    let mut t = PainterTool::default();
    t.agenda_papel_texturizado(ENTIDADE);
    t.bind_document(ENTIDADE, vec![255u8; (W * H * 4) as usize], W, H);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_relogio_fixo(true);
    t.set_semente_dos_tracos(0);
    t
}

fn camada(
    t: &mut PainterTool,
    p: &ph2d_brush_presets::Preset,
    q: Pedido,
    cor: [u8; 3],
    a: [f32; 2],
    z: [f32; 2],
    var: &str,
) {
    for k in 0..q.n {
        let erros = t.wet_preset_aplicar(p);
        assert!(erros.is_empty(), "{erros:?}");
        assert!(t.route_brush_wetpaint_event(&PanelEvent::SetValue(
            crate::ids::PAINTER_WETPAINT_WATER,
            q.agua
        )));
        t.sonda_raio(32.0);
        t.set_brush_color_srgb8(cor);
        var_aplica(t, var);
        if k % 2 == 0 {
            traco(t, a, z, q.pressao, var);
        } else {
            traco(t, z, a, q.pressao, var);
        }
        for _ in 0..10 {
            tique(t, var);
        }
    }
}

fn traco2(q: Pedido) -> ([f32; 2], [f32; 2]) {
    match q.cruz {
        Cruz::Perp => ([180.0, 30.0], [180.0, 250.0]),
        Cruz::Par => ([60.0, Y1 + 14.0], [300.0, Y1 + 14.0]),
    }
}

/// As células da linha central do traço 2 (para as ondas), em índices da grade fina.
fn linha2(q: Pedido, gw: usize, r: f32) -> Vec<usize> {
    let c = |p: f32| (p / r) as usize;
    match q.cruz {
        Cruz::Perp => (c(60.0)..c(220.0)).map(|y| c(180.0) + y * gw).collect(),
        Cruz::Par => (c(80.0)..c(280.0)).map(|x| x + c(Y1 + 14.0) * gw).collect(),
    }
}

fn rms_passa_alto(h: &[f64], idx: &[usize]) -> f64 {
    let v: Vec<f64> = idx.iter().map(|i| h[*i]).collect();
    let mut s = 0.0;
    let mut n = 0;
    for i in 4..v.len().saturating_sub(4) {
        let m = v[i - 4..=i + 4].iter().sum::<f64>() / 9.0;
        s += (v[i] - m).powi(2);
        n += 1;
    }
    (s / n.max(1) as f64).sqrt()
}

#[derive(Default, Debug)]
struct Relevo {
    /// Crateras (células) dentro / fora do contacto do pincel 2, e a mais funda (÷ mediana da camada 1).
    crat_in: usize,
    crat_out: usize,
    fundo_in: f64,
    fundo_out: f64,
    /// RMS do passa-alto ao longo do traço 2 ÷ o do traço 2 sozinho.
    ondas: f64,
    /// |Δh| fora do contacto (média e máx) ÷ mediana da camada 1.
    deriva_med: f64,
    deriva_max: f64,
    /// (Σm depois − Σm antes − Σm do traço sozinho) ÷ Σm do traço sozinho.
    massa: f64,
    /// Mediana da altura da camada 1 (unidades do campo).
    med1: f64,
    /// O mesmo balanço de [`Relevo::massa`] sobre o VOLUME (Σ altura, o que a luz lê; a massa tem o tecto da cor).
    vol: f64,
}

fn falsa_cor(dir: &std::path::Path, nome: &str, h: &[f64], gw: usize, gh: usize, teto: f64) {
    use crate::tool::paint::tests::look_watercolor_arc::{Canvas, write_png};
    let px = h
        .iter()
        .map(|v| {
            let s = (v / teto).clamp(0.0, 1.0);
            // Rampa preto→azul→verde→amarelo→branco.
            let f = |a: f64, b: f64| ((s - a) / (b - a)).clamp(0.0, 1.0);
            [
                (255.0 * f(0.45, 0.75)) as u8,
                (255.0 * f(0.25, 0.6)) as u8,
                (255.0 * (f(0.0, 0.25) - f(0.4, 0.6) + f(0.8, 1.0)).clamp(0.0, 1.0)) as u8,
            ]
        })
        .collect();
    write_png(&dir.join(nome), &Canvas { w: gw, h: gh, px }).expect("png");
}

fn cena(b: &Biblioteca, q: Pedido, var: &str, out: Option<(&std::path::Path, &str)>) -> Relevo {
    var_globais(var);
    let rota = if var == "sem_ladeira" { 2 } else { 0 };
    let _rota =
        ph2d_wet_paint::solver::regua::Regua::poe(&ph2d_wet_paint::solver::LADEIRA_ROTA, rota);
    // `sem_bingham` = a lei de antes da E7 (sem tensão de cedência); `b<τ₀>_<φ_c>` = outras constantes; o resto = o produto.
    let (tau, phic): (f32, f32) = if var.starts_with("sem_bingham") {
        (-1.0, 0.0)
    } else if let Some(v) = var.strip_prefix('b') {
        let mut it = v.split('_').map(|x| x.parse().unwrap_or(0.0));
        (it.next().unwrap_or(0.0), it.next().unwrap_or(0.0))
    } else {
        (0.0, 0.0)
    };
    let _tau = ph2d_wet_paint::solver::regua::Regua::poe(
        &ph2d_wet_paint::solver::PAPEL_BINGHAM,
        tau.to_bits(),
    );
    let _phic = ph2d_wet_paint::solver::regua::Regua::poe(
        &ph2d_wet_paint::solver::BINGHAM_PHI_C,
        phic.to_bits(),
    );
    let _sett = ph2d_wet_paint::solver::regua::Regua::poe(
        &ph2d_wet_paint::solver::BINGHAM_COM_SETT,
        u8::from(var.ends_with("sett")),
    );
    let p = preset(b, q.pincel);
    let (a2, z2) = traco2(q);
    // O traço 2 sozinho: a pegada (contacto) e a referência das ondas e da massa.
    let mut c = ferramenta();
    camada(&mut c, &p, q, AM, a2, z2, var);
    for _ in 0..q.pausa {
        tique(&mut c, var);
    }
    let (hc, mc, gw, gh, r) = alturas(&mut c);
    let mmax = mc.iter().copied().fold(0.0, f64::max);
    let mut toque = vec![false; gw * gh];
    for y in 0..gh {
        for x in 0..gw {
            if mc[x + y * gw] > 0.01 * mmax {
                for dy in -2isize..=2 {
                    for dx in -2isize..=2 {
                        let (xx, yy) = (x as isize + dx, y as isize + dy);
                        if xx >= 0 && yy >= 0 && (xx as usize) < gw && (yy as usize) < gh {
                            toque[xx as usize + yy as usize * gw] = true;
                        }
                    }
                }
            }
        }
    }
    // A cena: camada 1, pausa, camada 2, pausa.
    let mut t = ferramenta();
    camada(&mut t, &p, q, AZ, [50.0, Y1], [310.0, Y1], var);
    for _ in 0..q.pausa {
        tique(&mut t, var);
    }
    let (h1, m1, ..) = alturas(&mut t);
    if let Some((d, rot)) = out {
        t.wet_recomposite_full();
        grava_png(
            &d.join(format!("{rot}_1luz.png")),
            &t.canvas_rgba.to_vec(),
            W as usize,
            H as usize,
        );
    }
    camada(&mut t, &p, q, AM, a2, z2, var);
    for _ in 0..q.pausa {
        tique(&mut t, var);
    }
    let (h2, m2, ..) = alturas(&mut t);
    drop((_rota, _tau, _phic, _sett));
    var_globais("prod");
    let m1max = m1.iter().copied().fold(0.0, f64::max);
    let banda: Vec<usize> = (0..gw * gh).filter(|i| m1[*i] > 0.05 * m1max).collect();
    let mut hs: Vec<f64> = banda.iter().map(|i| h1[*i]).collect();
    hs.sort_by(f64::total_cmp);
    let med1 = hs.get(hs.len() / 2).copied().unwrap_or(0.0).max(1e-9);
    let lim = 0.15 * med1;
    let mut rl = Relevo {
        med1,
        ..Default::default()
    };
    let (mut ds, mut nd) = (0.0, 0);
    for &i in &banda {
        let cav = h1[i] - h2[i];
        if toque[i] {
            if cav > lim {
                rl.crat_in += 1;
            }
            rl.fundo_in = rl.fundo_in.max(cav / med1);
        } else {
            if cav > lim {
                rl.crat_out += 1;
            }
            rl.fundo_out = rl.fundo_out.max(cav / med1);
            let d = (h2[i] - h1[i]).abs() / med1;
            ds += d;
            nd += 1;
            rl.deriva_max = rl.deriva_max.max(d);
        }
    }
    rl.deriva_med = ds / f64::from(nd.max(1));
    let l2 = linha2(q, gw, r);
    rl.ondas = rms_passa_alto(&h2, &l2) / rms_passa_alto(&hc, &l2).max(1e-9);
    let soma = |v: &[f64]| v.iter().sum::<f64>();
    let mcs = soma(&mc).max(1e-9);
    rl.massa = (soma(&m2) - soma(&m1) - mcs) / mcs;
    let hcs = soma(&hc).max(1e-9);
    rl.vol = (soma(&h2) - soma(&h1) - hcs) / hcs;
    if let Some((d, rot)) = out {
        t.wet_recomposite_full();
        grava_png(
            &d.join(format!("{rot}_2luz.png")),
            &t.canvas_rgba.to_vec(),
            W as usize,
            H as usize,
        );
        let teto = 2.5 * med1;
        falsa_cor(d, &format!("{rot}_3altura1.png"), &h1, gw, gh, teto);
        falsa_cor(d, &format!("{rot}_4altura2.png"), &h2, gw, gh, teto);
        // A cratera: onde a altura desceu (vermelho = mais fundo), o contacto a cinzento.
        use crate::tool::paint::tests::look_watercolor_arc::{Canvas, write_png};
        let px = (0..gw * gh)
            .map(|i| {
                let cav = ((h1[i] - h2[i]) / med1).clamp(0.0, 1.0);
                let g = if toque[i] { 70 } else { 20 };
                [g.max((255.0 * cav) as u8), g, g]
            })
            .collect();
        write_png(
            &d.join(format!("{rot}_5cratera.png")),
            &Canvas { w: gw, h: gh, px },
        )
        .expect("png");
    }
    rl
}

fn lista<T: std::str::FromStr>(var: &str, omissao: &str) -> Vec<T> {
    std::env::var(var)
        .unwrap_or_else(|_| omissao.into())
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect()
}

#[test]
#[ignore = "sonda — corre-se à mão no perfil smoke"]
fn sonda_relevo_oleo() {
    let b = Biblioteca::carregar(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes"),
        None,
    );
    let dir = std::env::var("PH2D_SONDA_OUT")
        .ok()
        .map(std::path::PathBuf::from);
    if let Some(d) = &dir {
        std::fs::create_dir_all(d).expect("pasta");
    }
    let pinceis: Vec<String> = lista(
        "PH2D_SONDA_PINCEIS",
        "Oil/Flat/Thick Impasto,Oil/Knife/Knife Smooth 3,Oil/Flat/Filbert,Oil/Flat/Flat Bristle",
    );
    let vars: Vec<String> = lista("PH2D_SONDA_VAR", "prod");
    let leis: Vec<ph2d_wet_paint::sim::LeiDaTinta> = lista("PH2D_SONDA_LEI", "velha");
    let aguas: Vec<f64> = lista("PH2D_SONDA_AGUA", "0,0.2,0.5");
    let pressoes: Vec<f32> = lista("PH2D_SONDA_PRESSAO", "1");
    let ns: Vec<usize> = lista("PH2D_SONDA_N", "1");
    let pausas: Vec<u32> = lista("PH2D_SONDA_PAUSA", "120");
    let cruzes: Vec<String> = lista("PH2D_SONDA_CRUZ", "perp,par");
    let fotos = std::env::var("PH2D_SONDA_FOTOS").is_ok();
    println!(
        "[relevo] pincel\tcruz\tagua\tpressao\tn\tpausa\tvar\tcrat_in\tcrat_out\tfundo_in\tfundo_out\tondas\tderiva_med\tderiva_max\tmassa\tmed1\tvol"
    );
    for pincel in &pinceis {
        for cruz in &cruzes {
            let cz = if cruz == "perp" {
                Cruz::Perp
            } else {
                Cruz::Par
            };
            for &agua in &aguas {
                for &pressao in &pressoes {
                    for &n in &ns {
                        for &pausa in &pausas {
                            // As variantes INTERCALADAS dentro da mesma célula.
                            for (var, lei) in
                                vars.iter().flat_map(|v| leis.iter().map(move |l| (v, *l)))
                            {
                                LEI.set(lei);
                                let q = Pedido {
                                    pincel,
                                    agua,
                                    pressao,
                                    n,
                                    pausa,
                                    cruz: cz,
                                };
                                let rot = format!(
                                    "{}_{cruz}_a{agua}_p{pressao}_n{n}_t{pausa}_{var}",
                                    pincel
                                        .rsplit('/')
                                        .next()
                                        .unwrap_or("")
                                        .replace(' ', "_")
                                        .to_lowercase()
                                );
                                let o = if fotos {
                                    dir.as_deref().map(|d| (d, rot.as_str()))
                                } else {
                                    None
                                };
                                let r = cena(&b, q, var, o);
                                println!(
                                    "[relevo] {pincel}\t{cruz}\t{agua}\t{pressao}\t{n}\t{pausa}\t{var}@{}\t{}\t{}\t{:.2}\t{:.2}\t{:.2}\t{:.3}\t{:.2}\t{:+.2}\t{:.3}\t{:+.2}",
                                    lei.nome(),
                                    r.crat_in,
                                    r.crat_out,
                                    r.fundo_in,
                                    r.fundo_out,
                                    r.ondas,
                                    r.deriva_med,
                                    r.deriva_max,
                                    r.massa,
                                    r.med1,
                                    r.vol
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}
