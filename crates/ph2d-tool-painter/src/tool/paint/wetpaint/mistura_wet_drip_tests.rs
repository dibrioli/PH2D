//! **Uma mistura nunca é mais clara que as tintas** (smoke do dono 10/10: o azul-claro do Wet Drip com
//! «Pigment mixing (K-M)» + «Glaze layering (K-M)»; prompt 16, Fase E). A cena das capturas
//! `fixtures/smoke_dono_mistura_wet_drip/`: tela branca; duas passadas de VERMELHO `[220, 60, 60]` com o Wet
//! Round (y 250 · 255, raio 30), que fica MOLHADO (30 tiques) ou SECO (Dry canvas); 5 passadas de AZUL ESCURO
//! `[20, 50, 110]` com o Wet Drip de fábrica (serpentinas y 175 · 200 · 225, raio 20 — a de baixo morde o
//! vermelho); a água escorre 600 tiques e o Dry canvas.
//!
//! As réguas, por pixel (= célula, razão 1), partilhadas com a [`super::sonda_mistura_wet_drip`]:
//! **mais clara à mesma massa** (uma tinta sozinha com a massa `m` mostra `255 − α(m)·(255 − tinta)` por canal;
//! nenhuma mistura subtractiva da mesma massa passa da tinta mais clara, em luz linear); **o índice do cone**
//! (`D = −ln T` por canal projectada no cone `{α·D(vermelho) + β·D(azul)}`, resíduo ÷ `|D|`); **o pontilhado**
//! (o miolo 5×5 da tinta mais CLARO que a mediana 3×3) e a régua `N32` do Rebelle.
//!
//! ⚠️ O CONTROLO de cada portão é a lei de antes do «Glaze layering» (`Lei::Camada`): tem de reprovar.
//! Números e recusas: `docs/Painter/NewFeaturesWetPaint/05_plano_da_linha_e_o_antes.md` §4b «Mistura II».
use super::mistura_tests as mt;
use crate::tool::PainterTool;
use ph2d_brush_presets::Biblioteca;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{
    CanvasPaintTool, CanvasPointer, PanelEvent, PointerPhase, RasterEditTool,
};
use ph2d_wet_paint::assenta::Lei;
use ph2d_wet_paint::opacity::alpha_of_mass;
use std::sync::{Arc, Mutex, OnceLock};

pub(super) const W: usize = 600;
pub(super) const H: usize = 400;
pub(super) const VERMELHO: [u8; 3] = [220, 60, 60];
pub(super) const AZUL: [u8; 3] = [20, 50, 110];
const PASSADAS_AZUIS: usize = 5;
const ESCORRE: usize = 600;

/// O excesso (luz linear, por canal) sobre a tinta mais clara à mesma massa: o produto SEM caixas mede
/// `+0,148` (o azul seco que TAPA o vermelho seco, a lei do Rebelle); a lei de antes `+0,646`. Com a caixa das
/// tintas no ecrã (Bug #50, 12/10): `+0,1516` (K–M, seco) — o controlo continua a `+0,646`.
const TOL_MAIS_CLARA: f64 = 0.16;
/// O índice do cone (o máximo da tela): sem caixas `0,08`, com o «Pigment mixing» `0,14`; a lei de antes `0,56`.
const TECTO_CONE: f64 = 0.20;
/// «Mais claro que a mediana 3×3» conta acima de `N` níveis: o P99,9 de cada tinta SOZINHA sem caixas (25,0).
const N_PONTILHADO: f64 = 25.0;
/// Quantos pixels do miolo passam de [`N_PONTILHADO`]: o produto mede `38`–`96` (sem caixas `90`); a lei de
/// antes `2 498`–`3 012`.
const TECTO_PONTILHADO: usize = 150;
/// A barra do Rebelle: `≤ 0,13 %` do interior da tinta mais claro que a mediana 3×3 por `> 32`.
const REBELLE_N32: f64 = 0.13;

#[derive(Clone, Copy, PartialEq, Debug)]
pub(super) enum Tintas {
    Ambas,
    SoVermelho,
    SoAzul,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub(super) struct Var {
    pub(super) km: bool,
    pub(super) glaze: bool,
    pub(super) seco: bool,
    /// O Heavy Edge (`1` na cena das capturas).
    pub(super) borda: f64,
    /// O CONTROLO: a lei de antes do «Glaze layering» ([`Lei::Camada`]) no ecrã e nas transferências.
    pub(super) lei_de_antes: bool,
}

impl Var {
    pub(super) const DUAS: Var = Var {
        km: true,
        glaze: true,
        seco: false,
        borda: 1.0,
        lei_de_antes: false,
    };
}

/// As telas `[passadas, +120 tiques, escorrimento, seca]` e a massa de pigmento por pixel nas duas últimas.
pub(super) struct Cena {
    pub(super) telas: [Vec<u8>; 4],
    pub(super) massa: [Vec<f32>; 2],
}

pub(super) const MOMENTOS: [&str; 4] = ["passadas", "t120", "escorre", "seco"];

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn linha(a: [f32; 2], b: [f32; 2]) -> Vec<[f32; 2]> {
    let n = ((b[0] - a[0]).abs().max((b[1] - a[1]).abs()) / 3.0).max(1.0) as usize;
    (0..=n)
        .map(|k| {
            let f = k as f32 / n as f32;
            [a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f]
        })
        .collect()
}

pub(super) fn biblioteca() -> Biblioteca {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    Biblioteca::carregar(&raiz, None)
}

fn preset(b: &Biblioteca, nome: &str) -> ph2d_brush_presets::Preset {
    b.entradas
        .iter()
        .find(|e| e.preset.parametros.name == nome)
        .expect("o pincel de fábrica")
        .preset
        .clone()
}

fn ticks(t: &mut PainterTool, n: usize) {
    for _ in 0..n {
        t.on_tick(1000.0 / 60.0);
    }
}

fn traco(t: &mut PainterTool, pts: &[[f32; 2]]) {
    t.on_canvas_pointer(cp(pts[0], PointerPhase::Down));
    for &p in &pts[1..] {
        t.on_canvas_pointer(cp(p, PointerPhase::Move));
        t.on_tick(8.0);
    }
    t.on_canvas_pointer(cp(pts[pts.len() - 1], PointerPhase::Up));
}

fn seca(t: &mut PainterTool) {
    ticks(t, 60);
    assert!(
        t.route_brush_wetpaint_event(&PanelEvent::Click(crate::ids::PAINTER_WETPAINT_DRYCANVAS))
    );
    ticks(t, 30);
}

fn sim(t: &mut PainterTool) -> &mut ph2d_wet_paint::sim::Sim {
    &mut t
        .paint
        .wetpaint
        .session
        .as_mut()
        .expect("sessão")
        .engine
        .sim
}

/// A lei do controlo (a do produto fica a do motor: o reconcile nunca escolhe a `Camada`).
fn aplica_lei(t: &mut PainterTool, v: Var) {
    if v.lei_de_antes {
        t.wet_bring_home();
        sim(t).lei = Lei::Camada;
    }
}

fn massa(t: &mut PainterTool) -> Vec<f32> {
    t.wet_bring_home();
    let g = &t
        .paint
        .wetpaint
        .session
        .as_ref()
        .expect("sessão")
        .engine
        .layers[0]
        .grid;
    (0..W * H)
        .map(|i| {
            let c = (i % W + 1) + (i / W + 1) * g.s;
            g.susp[c] + g.sett[c]
        })
        .collect()
}

pub(super) fn cena(b: &Biblioteca, v: Var, tintas: Tintas) -> Cena {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_tool_mode("wetpaint");
    // A física mede-se no carimbo do motor de sempre: a W2 só re-rotula o número do painel.
    t.set_wet_largura_crua(1.0);
    t.set_wet_relogio_fixo(true);
    assert!(t.wet_preset_aplicar(&preset(b, "Wet Round")).is_empty());
    t.set_wet_largura_crua(1.0);
    t.paint.wetpaint.tilt_on = false;
    t.set_brush_size_px(30.0);
    t.set_wet_borda_que_pesa(v.borda);
    t.paint.wetpaint.km_mixing = v.km;
    t.paint.wetpaint.km_glaze = v.glaze;
    assert!(t.ensure_wet_session());
    aplica_lei(&mut t, v);
    ticks(&mut t, 1);
    if tintas != Tintas::SoAzul {
        // Duas passadas de vermelho (ida e volta), o traço carregado das capturas.
        t.set_brush_color_srgb8(VERMELHO);
        traco(&mut t, &linha([80.0, 250.0], [520.0, 250.0]));
        traco(&mut t, &linha([520.0, 255.0], [80.0, 255.0]));
    }
    if v.seco {
        seca(&mut t);
    } else {
        ticks(&mut t, 30);
    }
    if tintas != Tintas::SoVermelho {
        assert!(t.wet_preset_aplicar(&preset(b, "Wet Drip")).is_empty());
        t.set_wet_largura_crua(1.0);
        assert!(t.paint.wetpaint.tilt_on && t.paint.wetpaint.tilt_ring == 8);
        aplica_lei(&mut t, v);
        t.set_brush_size_px(20.0);
        t.set_brush_color_srgb8(AZUL);
        // «Várias passadas» sobre a MESMA faixa: cada uma uma serpentina de 3 linhas, com 4 tiques entre elas.
        for _ in 0..PASSADAS_AZUIS {
            let mut pts: Vec<[f32; 2]> = Vec::new();
            for (k, y) in [175.0f32, 200.0, 225.0].into_iter().enumerate() {
                let (a, z) = if k % 2 == 0 {
                    ([180.0, y], [420.0, y])
                } else {
                    ([420.0, y], [180.0, y])
                };
                if let Some(&u) = pts.last() {
                    pts.extend(linha(u, a).into_iter().skip(1));
                }
                pts.extend(linha(a, z).into_iter().skip(usize::from(!pts.is_empty())));
            }
            traco(&mut t, &pts);
            ticks(&mut t, 4);
        }
    }
    let t0 = t.canvas_rgba.to_vec();
    ticks(&mut t, 120);
    let t1 = t.canvas_rgba.to_vec();
    ticks(&mut t, ESCORRE - 120);
    let t2 = t.canvas_rgba.to_vec();
    let m2 = massa(&mut t);
    seca(&mut t);
    let m3 = massa(&mut t);
    let lei = sim(&mut t).lei;
    assert_eq!(lei == Lei::Camada, v.lei_de_antes, "a lei da cena: {lei:?}");
    Cena {
        telas: [t0, t1, t2, t.canvas_rgba.to_vec()],
        massa: [m2, m3],
    }
}

/// [`cena`] com as duas tintas, uma vez por processo (os portões partilham-nas sob `cargo test`).
pub(super) fn cena_partilhada(b: &Biblioteca, v: Var) -> Arc<Cena> {
    type Feitas = Mutex<Vec<(Var, Arc<Cena>)>>;
    static FEITAS: OnceLock<Feitas> = OnceLock::new();
    let feitas = FEITAS.get_or_init(Mutex::default);
    if let Some((_, c)) = feitas.lock().expect("cache").iter().find(|(k, _)| *k == v) {
        return Arc::clone(c);
    }
    let c = Arc::new(cena(b, v, Tintas::Ambas));
    feitas.lock().expect("cache").push((v, Arc::clone(&c)));
    c
}

// ── As réguas ─────────────────────────────────────────────────────────────────────────────────────

pub(super) fn lin(c: f64) -> f64 {
    let c = c / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn dens(c: [f64; 3]) -> [f64; 3] {
    c.map(|v| -(lin(v).max(1e-4)).ln())
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// O resíduo relativo de `d` no cone `{α·a + β·b, α, β ≥ 0}` (NNLS de dois vectores, fechado).
fn indice(d: [f64; 3], a: [f64; 3], b: [f64; 3]) -> f64 {
    let n = dot(d, d).sqrt();
    if n < 0.05 {
        return 0.0;
    }
    let res = |x: f64, y: f64| {
        let r = [0, 1, 2].map(|c| d[c] - x * a[c] - y * b[c]);
        dot(r, r).sqrt()
    };
    let (aa, ab, bb, ad, bd) = (dot(a, a), dot(a, b), dot(b, b), dot(a, d), dot(b, d));
    let det = aa * bb - ab * ab;
    let mut melhor = res(0.0, 0.0);
    if det.abs() > 1e-12 {
        let (x, y) = ((ad * bb - bd * ab) / det, (bd * aa - ad * ab) / det);
        if x >= 0.0 && y >= 0.0 {
            melhor = melhor.min(res(x, y));
        }
    }
    melhor = melhor.min(res((ad / aa).max(0.0), 0.0));
    melhor = melhor.min(res(0.0, (bd / bb).max(0.0)));
    melhor / n
}

pub(super) fn px(t: &[u8], i: usize) -> [f64; 3] {
    [t[i * 4], t[i * 4 + 1], t[i * 4 + 2]].map(f64::from)
}

pub(super) fn pintado(t: &[u8], i: usize) -> bool {
    px(t, i).iter().any(|&v| v < 250.0)
}

/// O índice do cone de cada pixel pintado (`-1` fora).
pub(super) fn indices(t: &[u8]) -> Vec<f64> {
    let (a, b) = (dens(VERMELHO.map(f64::from)), dens(AZUL.map(f64::from)));
    (0..W * H)
        .map(|i| {
            if pintado(t, i) {
                indice(dens(px(t, i)), a, b)
            } else {
                -1.0
            }
        })
        .collect()
}

/// O PONTILHADO: quanto o pixel é mais CLARO (máximo por canal, níveis sRGB) que a mediana 3×3, só no
/// MIOLO da tinta (os 5×5 pintados — a borda de um fio contra o branco não conta); `-1` fora.
pub(super) fn desvios(t: &[u8]) -> Vec<f64> {
    let mut out = vec![-1.0; W * H];
    for y in 2..H - 2 {
        for x in 2..W - 2 {
            if !(0..25).all(|k| pintado(t, (y + k / 5 - 2) * W + x + k % 5 - 2)) {
                continue;
            }
            let mut d: f64 = f64::MIN;
            for c in 0..3 {
                let mut v: Vec<u8> = (0..9)
                    .map(|k| t[((y + k / 3 - 1) * W + x + k % 3 - 1) * 4 + c])
                    .collect();
                v.sort_unstable();
                d = d.max(f64::from(t[(y * W + x) * 4 + c]) - f64::from(v[4]));
            }
            out[y * W + x] = d.max(0.0);
        }
    }
    out
}

/// **Mais clara que a tinta mais clara À MESMA MASSA**, em luz linear: o excesso (máximo por canal) de cada
/// pixel pintado com massa `≥ 1` (`-1` fora).
pub(super) fn excessos(t: &[u8], massa: &[f32]) -> Vec<f64> {
    let teto = [0, 1, 2].map(|c| f64::from(VERMELHO[c].max(AZUL[c])));
    (0..W * H)
        .map(|i| {
            if !pintado(t, i) || massa[i] < 1.0 {
                return -1.0;
            }
            let a = alpha_of_mass(f64::from(massa[i]));
            let c = px(t, i);
            (0..3)
                .map(|k| lin(c[k]) - lin(255.0 - a * (255.0 - teto[k])))
                .fold(f64::MIN, f64::max)
        })
        .collect()
}

/// A régua `N32` do Rebelle (`ferramentas/mistura_wet_drip_regua.py` (c)): % das células do INTERIOR da tinta
/// (`> 25` níveis abaixo do branco, erodido 2 px) mais CLARAS que a mediana 3×3 por `> 32` em todos os canais.
pub(super) fn claras_n32(t: &[u8]) -> f64 {
    let tinta = |i: usize| 255.0 - px(t, i).iter().copied().fold(255.0, f64::min) > 25.0;
    let mut m: Vec<bool> = (0..W * H).map(tinta).collect();
    for _ in 0..2 {
        let a = m.clone();
        for y in 0..H {
            for x in 0..W {
                let viz = |dx: i64, dy: i64| {
                    let (u, v) = (x as i64 + dx, y as i64 + dy);
                    (0..W as i64).contains(&u)
                        && (0..H as i64).contains(&v)
                        && a[v as usize * W + u as usize]
                };
                m[y * W + x] = a[y * W + x] && viz(1, 0) && viz(-1, 0) && viz(0, 1) && viz(0, -1);
            }
        }
    }
    let (mut n, mut k) = (0usize, 0usize);
    for y in 0..H {
        for x in 0..W {
            if !m[y * W + x] {
                continue;
            }
            n += 1;
            let mut clar = f64::MAX;
            for c in 0..3 {
                let mut v: Vec<u8> = (0..9)
                    .map(|q| {
                        let xx = (x as i64 + q % 3 - 1).clamp(0, W as i64 - 1) as usize;
                        let yy = (y as i64 + q / 3 - 1).clamp(0, H as i64 - 1) as usize;
                        t[(yy * W + xx) * 4 + c]
                    })
                    .collect();
                v.sort_unstable();
                clar = clar.min(f64::from(t[(y * W + x) * 4 + c]) - f64::from(v[4]));
            }
            k += usize::from(clar > 32.0);
        }
    }
    100.0 * k as f64 / n.max(1) as f64
}

fn maximo(v: &[f64]) -> f64 {
    v.iter().copied().fold(0.0, f64::max)
}

/// `(excesso máximo, índice do cone máximo)` nas telas do escorrimento e seca.
fn mais_claro_e_cone(c: &Cena) -> (f64, f64) {
    let mut out = (0.0f64, 0.0f64);
    for k in [2usize, 3] {
        out.0 = out.0.max(maximo(&excessos(&c.telas[k], &c.massa[k - 2])));
        out.1 = out.1.max(maximo(&indices(&c.telas[k])));
    }
    out
}

/// `(pixels do miolo acima de N, % N32)` — o pior das duas telas.
fn soltas(c: &Cena) -> (usize, f64) {
    let mut out = (0usize, 0.0f64);
    for k in [2usize, 3] {
        let n = desvios(&c.telas[k])
            .iter()
            .filter(|&&d| d > N_PONTILHADO)
            .count();
        out = (out.0.max(n), out.1.max(claras_n32(&c.telas[k])));
    }
    out
}

/// A cena do Wet Drip com as duas caixas, e cada uma sozinha, molhado e seco: nenhuma célula mais clara que a
/// tinta mais clara à mesma massa (acima do que o produto SEM caixas mostra) e o cone dentro do tecto.
#[test]
fn uma_mistura_nunca_e_mais_clara_que_as_tintas() {
    let b = biblioteca();
    let ctl = Var {
        lei_de_antes: true,
        ..Var::DUAS
    };
    // Uma variante por thread (independentes).
    let ((exc, cone), falhas) = std::thread::scope(|s| {
        let b = &b;
        let controlo = s.spawn(move || mais_claro_e_cone(&cena_partilhada(b, ctl)));
        let vars: Vec<_> = [(true, true), (true, false), (false, true)]
            .into_iter()
            .flat_map(|(km, glaze)| [false, true].map(|seco| (km, glaze, seco)))
            .map(|(km, glaze, seco)| {
                s.spawn(move || {
                    let v = Var {
                        km,
                        glaze,
                        seco,
                        ..Var::DUAS
                    };
                    let (exc, cone) = mais_claro_e_cone(&cena_partilhada(b, v));
                    println!("km {km} glaze {glaze} seco {seco}: mais clara {exc:+.4} · cone {cone:.3}");
                    (exc > TOL_MAIS_CLARA || cone > TECTO_CONE).then(|| {
                        format!(
                            "km {km} glaze {glaze} seco {seco}: mais clara {exc:+.4} (tol {TOL_MAIS_CLARA}) · cone {cone:.3} (tecto {TECTO_CONE})"
                        )
                    })
                })
            })
            .collect();
        let falhas: Vec<String> = vars
            .into_iter()
            .filter_map(|h| h.join().expect("a variante entrou em pânico"))
            .collect();
        (
            controlo.join().expect("o controlo entrou em pânico"),
            falhas,
        )
    });
    println!("CONTROLO (a lei de antes): mais clara {exc:+.4} · cone {cone:.3}");
    assert!(
        exc > TOL_MAIS_CLARA && cone > TECTO_CONE,
        "CONTROLO: a lei de antes passou (mais clara {exc:+.4}, cone {cone:.3}) — a régua não discrimina"
    );
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}

/// Sem pontinhos claros soltos dentro da tinta (as duas caixas, molhado e seco): o miolo mais claro que a
/// mediana 3×3 por `> N` dentro do tecto, e a barra `N32` do Rebelle.
#[test]
fn sem_celulas_soltas() {
    let b = biblioteca();
    let mut falhas = Vec::new();
    let ctl = Var {
        lei_de_antes: true,
        ..Var::DUAS
    };
    // As três cenas são independentes: uma por thread (67 s sob a suíte, 11/10).
    let [molhado, seco_, controlo] = std::thread::scope(|s| {
        [
            Var::DUAS,
            Var {
                seco: true,
                ..Var::DUAS
            },
            ctl,
        ]
        .map(|v| {
            let b = &b;
            s.spawn(move || soltas(&cena_partilhada(b, v)))
        })
        .map(|h| h.join().expect("a cena entrou em pânico"))
    });
    for (seco, (n, n32)) in [(false, molhado), (true, seco_)] {
        println!("seco {seco}: pontilhado {n} · N32 {n32:.3} %");
        if n > TECTO_PONTILHADO || n32 > REBELLE_N32 {
            falhas.push(format!(
                "seco {seco}: pontilhado {n} (tecto {TECTO_PONTILHADO}) · N32 {n32:.3} % (Rebelle ≤ {REBELLE_N32})"
            ));
        }
    }
    let (n, n32) = controlo;
    println!("CONTROLO (a lei de antes): pontilhado {n} · N32 {n32:.3} %");
    assert!(
        n > TECTO_PONTILHADO,
        "CONTROLO: a lei de antes passou (pontilhado {n}) — a régua não discrimina"
    );
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}

/// **«Glaze layering»: a camada de baixo vê-se através da de cima** (a cena W1e, azul sobre vermelho SECO):
/// com a caixa o cruzamento fica mais perto da multiplicação (γ maior) que sem ela, nunca além dela (γ ≤ 1), e
/// nenhum canal do cruzamento fica mais claro que as duas tintas sozinhas (luz linear). Sem a caixa o γ é o
/// controlo: uma caixa que não chega à tela reprova.
#[test]
fn o_glaze_layering_deixa_ver_a_camada_de_baixo() {
    let le = |glaze: bool| {
        mt::CAIXAS.set((false, glaze));
        let (z, a, q) = mt::cena_toda(true, true, None, false, false, false);
        mt::CAIXAS.set((false, false));
        (mt::le(&a, &z), mt::le(&q, &z), q)
    };
    let (sem_d, sem, _) = le(false);
    let (com_d, com, tela) = le(true);
    let (cruz, verm, azul) = (
        mt::media(&tela, 295, 306, 145, 156),
        mt::media(&tela, 195, 206, 145, 156),
        mt::media(&tela, 295, 306, 60, 100),
    );
    let claro = (0..3)
        .map(|c| lin(cruz[c]) - lin(verm[c]).max(lin(azul[c])))
        .fold(f64::MIN, f64::max);
    println!(
        "γ seco sem a caixa {:.2}·{:.2} · com {:.2}·{:.2} · cruzamento acima das tintas {claro:+.4}",
        sem_d.gama, sem.gama, com_d.gama, com.gama
    );
    for (rot, s, c) in [("durante", &sem_d, &com_d), ("depois", &sem, &com)] {
        assert!(
            c.gama > s.gama + 0.15 && c.gama <= 1.0,
            "{rot}: γ com a caixa {:.2} contra {:.2} sem ela",
            c.gama,
            s.gama
        );
    }
    assert!(
        claro <= 0.0,
        "o cruzamento ficou mais claro que as tintas: {claro:+.4}"
    );
}
