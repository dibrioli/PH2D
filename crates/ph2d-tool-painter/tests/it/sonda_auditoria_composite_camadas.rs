//! Sonda da AUDITORIA de 08/11 (pedido do dono): os 13 pincéis Wet blend do CSP × Composite Brush × Per-Layer Color
//! (Shape de várias camadas) × camadas do documento × undo/redo × Off. Não muda nada: só mede e imprime.
//! `cargo test --profile smoke -p ph2d-tool-painter --test it sonda_auditoria -- --ignored --nocapture`
//! Folha: `docs/Painter/handoffs/material_2026-11-08_auditoria_composite_camadas/`.

use crate::familias::{csp_secos, csp_tinta_grossa};
use crate::presets_de_fabrica::preset_do_meio;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_tool_painter::{ModoMisturaOpaca, PainterTool};

const W: usize = 400;
const H: usize = 240;
const Y: f32 = 120.0;
const BRUSH: u8 = 0;
const SMEAR: u8 = 1;
const BLUR: u8 = 2;

const PINCEIS: &[&str] = &[
    "Round Mixing",
    "Gouache",
    "Dry Gouache",
    "Thin Gouache",
    "Gouache Blender",
    "Oil Paint",
    "Paint And Apply",
    "Thick Oil Paint",
    "Crayon",
    "Pastel",
    "Chalk",
    "Finger Tip",
    "Pointillism",
];

fn arrasta(t: &mut PainterTool, a: [f32; 2], b: [f32; 2]) {
    let cp = |pos: [f32; 2], phase| CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(cp(a, PointerPhase::Down));
    let n = 60;
    for k in 1..=n {
        let u = k as f32 / n as f32;
        t.on_canvas_pointer(cp(
            [a[0] + (b[0] - a[0]) * u, a[1] + (b[1] - a[1]) * u],
            PointerPhase::Move,
        ));
        t.on_tick(8.0);
    }
    t.on_canvas_pointer(cp(b, PointerPhase::Up));
    t.on_tick(8.0);
}

#[derive(Clone, Copy, PartialEq)]
enum Fundo {
    /// Camada branca opaca, sem bloco.
    Branco,
    /// Camada transparente (o papel por baixo), sem bloco.
    Vazio,
    /// Branco + bloco vermelho na MESMA camada.
    Bloco,
    /// Branco + bloco vermelho na camada de baixo; o traço numa camada NOVA vazia por cima.
    BlocoOutraCamada,
}

fn aplica(t: &mut PainterTool, nome: &str) {
    let r = csp_tinta_grossa::MEIOS
        .iter()
        .chain(csp_secos::MEIOS.iter())
        .find(|r| r.nome == nome)
        .unwrap_or_else(|| panic!("receita {nome}"));
    let p = preset_do_meio(r);
    assert_eq!(t.preset_aplicar(&p), Ok(Vec::new()), "{nome}");
    t.sonda_raio(15.0);
}

/// A cena pronta, com o pincel `nome` aplicado e a cor azul; o traço ainda por dar.
fn cena(nome: &str, f: Fundo) -> PainterTool {
    let mut t = PainterTool::default();
    let fundo = if f == Fundo::Vazio { 0u8 } else { 255 };
    t.set_source(vec![fundo; W * H * 4], W as u32, H as u32);
    if matches!(f, Fundo::Bloco | Fundo::BlocoOutraCamada) {
        t.sonda_raio(20.0);
        t.set_brush_mistura_modo(ModoMisturaOpaca::Off);
        t.set_brush_color_srgb8([200, 0, 0]);
        for k in 0..9 {
            let y = Y - 40.0 + 10.0 * k as f32;
            arrasta(&mut t, [160.0, y], [260.0, y]);
        }
    }
    if f == Fundo::BlocoOutraCamada {
        assert!(t.add_raster_layer("traço").is_some(), "camada nova");
    }
    aplica(&mut t, nome);
    t.set_brush_color_srgb8([0, 40, 220]);
    t
}

fn traco(t: &mut PainterTool) {
    arrasta(t, [40.0, Y], [340.0, Y]);
}

fn media(p: &[u8], x: usize, y: usize) -> [f32; 4] {
    let mut s = [0.0f32; 4];
    let mut n = 0.0;
    for yy in y - 2..=y + 2 {
        for xx in x - 4..=x + 4 {
            let i = (yy * W + xx) * 4;
            for c in 0..4 {
                s[c] += f32::from(p[i + c]);
            }
            n += 1.0;
        }
    }
    s.map(|v| (v / n).round())
}

fn hash(p: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in p {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x100_0000_01b3);
    }
    h
}

fn diff(a: &[u8], b: &[u8]) -> (f64, u8) {
    let mut s = 0u64;
    let mut m = 0u8;
    for (x, y) in a.iter().zip(b) {
        let d = x.abs_diff(*y);
        s += u64::from(d);
        m = m.max(d);
    }
    (s as f64 / a.len() as f64, m)
}

/// x = 100 branco antes do bloco · 210 dentro do bloco · 300 depois do bloco (o que se arrastou).
fn resumo(p: &[u8]) -> String {
    format!(
        "branco{:?} bloco{:?} cauda{:?} #{:016x}",
        media(p, 100, Y as usize),
        media(p, 210, Y as usize),
        media(p, 300, Y as usize),
        hash(p)
    )
}

fn monta_composite(t: &mut PainterTool, pilha: &[(u8, f32)]) {
    if !t.composite_enabled() {
        t.toggle_composite();
    }
    for (i, &(op, s)) in pilha.iter().enumerate() {
        t.acrescenta_camada(op);
        t.set_composite_layer_strength(i, s);
    }
    assert_eq!(t.composite_len(), pilha.len());
}

fn pinta(nome: &str, f: Fundo, prepara: impl FnOnce(&mut PainterTool)) -> Vec<u8> {
    let mut t = cena(nome, f);
    prepara(&mut t);
    traco(&mut t);
    t.run_full().0
}

#[test]
#[ignore = "sonda da auditoria"]
fn sonda_auditoria_composite_e_camadas() {
    let so = std::env::var("PH2D_AUD_SO").ok();
    for &nome in PINCEIS {
        if so
            .as_deref()
            .is_some_and(|s| !s.split(',').any(|n| n == nome))
        {
            continue;
        }
        let ref_bloco = pinta(nome, Fundo::Bloco, |_| {});
        let ref_bloco_2 = pinta(nome, Fundo::Bloco, |_| {});
        let off = pinta(nome, Fundo::Bloco, |t| {
            t.set_brush_mistura_modo(ModoMisturaOpaca::Off)
        });
        // Off com números da mistura diferentes: tem de dar o MESMO byte (Off não lê os números).
        let off_b = pinta(nome, Fundo::Bloco, |t| {
            t.set_brush_mistura_modo(ModoMisturaOpaca::Off);
            t.set_brush_mistura_numero(0, 0.9);
            t.set_brush_mistura_numero(1, 0.1);
            t.set_brush_mistura_numero(2, 0.0);
        });
        let c1 = pinta(nome, Fundo::Bloco, |t| monta_composite(t, &[(BRUSH, 1.0)]));
        let c_bb = pinta(nome, Fundo::Bloco, |t| {
            monta_composite(t, &[(BRUSH, 1.0), (BRUSH, 1.0)]);
            t.set_composite_layer_color(0, [1.0, 0.85, 0.0]);
        });
        let c_bb_off = pinta(nome, Fundo::Bloco, |t| {
            t.set_brush_mistura_modo(ModoMisturaOpaca::Off);
            monta_composite(t, &[(BRUSH, 1.0), (BRUSH, 1.0)]);
            t.set_composite_layer_color(0, [1.0, 0.85, 0.0]);
        });
        let c_sb = pinta(nome, Fundo::Bloco, |t| {
            monta_composite(t, &[(SMEAR, 1.0), (BRUSH, 1.0)])
        });
        let c_bs = pinta(nome, Fundo::Bloco, |t| {
            monta_composite(t, &[(BRUSH, 1.0), (SMEAR, 1.0)])
        });
        let c_rec = pinta(nome, Fundo::Bloco, |t| {
            // A receita do dono (memória 08/11): Blur 1 · Smear 0,25 · Brush 0,10 sobre o pincel com Wet blend.
            monta_composite(t, &[(BLUR, 1.0), (SMEAR, 0.25), (BRUSH, 0.10)])
        });
        let outra = pinta(nome, Fundo::BlocoOutraCamada, |_| {});
        let outra_off = pinta(nome, Fundo::BlocoOutraCamada, |t| {
            t.set_brush_mistura_modo(ModoMisturaOpaca::Off)
        });
        let outra_c = pinta(nome, Fundo::BlocoOutraCamada, |t| {
            monta_composite(t, &[(SMEAR, 1.0), (BRUSH, 1.0)])
        });
        let branco = pinta(nome, Fundo::Branco, |_| {});
        let vazio = pinta(nome, Fundo::Vazio, |_| {});
        let vazio_c = pinta(nome, Fundo::Vazio, |t| monta_composite(t, &[(BRUSH, 1.0)]));
        // Undo / redo do traço misturado, na tela directa e no Composite.
        let so_bloco = {
            let mut t = cena(nome, Fundo::Bloco);
            t.run_full().0
        };
        let (undo_ok, redo_ok) = {
            let mut t = cena(nome, Fundo::Bloco);
            traco(&mut t);
            assert!(t.undo_last(), "undo");
            let mut u = cena(nome, Fundo::Bloco);
            traco(&mut u);
            assert!(u.undo_last());
            assert!(u.redo_last(), "redo");
            (t.run_full().0 == so_bloco, u.run_full().0 == ref_bloco)
        };
        let (undo_c_ok, redo_c_ok) = {
            let mut t = cena(nome, Fundo::Bloco);
            monta_composite(&mut t, &[(SMEAR, 1.0), (BRUSH, 1.0)]);
            traco(&mut t);
            assert!(t.undo_last());
            let mut u = cena(nome, Fundo::Bloco);
            monta_composite(&mut u, &[(SMEAR, 1.0), (BRUSH, 1.0)]);
            traco(&mut u);
            assert!(u.undo_last());
            assert!(u.redo_last());
            (t.run_full().0 == so_bloco, u.run_full().0 == c_sb)
        };
        // Undo entre camadas: traço na camada nova sobre o bloco, undo → a pilha só com o bloco.
        let undo_outra_ok = {
            let base = {
                let mut t = cena(nome, Fundo::BlocoOutraCamada);
                t.run_full().0
            };
            let mut t = cena(nome, Fundo::BlocoOutraCamada);
            traco(&mut t);
            assert!(t.undo_last());
            t.run_full().0 == base
        };
        let d = |a: &[u8], b: &[u8]| {
            let (m, x) = diff(a, b);
            format!("Δmédio {m:.3} máx {x}")
        };
        eprintln!("\n[aud] ===== {nome}");
        eprintln!("[aud] wet        {}", resumo(&ref_bloco));
        eprintln!("[aud] off        {}", resumo(&off));
        eprintln!(
            "[aud] determinismo {} · off ignora números {} · wet≠off {}",
            ref_bloco == ref_bloco_2,
            off == off_b,
            d(&ref_bloco, &off)
        );
        eprintln!(
            "[aud] C[B]       {} · vs wet {}",
            resumo(&c1),
            d(&c1, &ref_bloco)
        );
        eprintln!("[aud] C[Bam,B]   {}", resumo(&c_bb));
        eprintln!(
            "[aud] C[Bam,B]off {} · wet vs off {}",
            resumo(&c_bb_off),
            d(&c_bb, &c_bb_off)
        );
        eprintln!("[aud] C[S,B]     {}", resumo(&c_sb));
        eprintln!(
            "[aud] C[B,S]     {} · ordem {}",
            resumo(&c_bs),
            d(&c_sb, &c_bs)
        );
        eprintln!("[aud] C[receita] {}", resumo(&c_rec));
        eprintln!("[aud] outra      {}", resumo(&outra));
        eprintln!(
            "[aud] outra off  {} · wet vs off {}",
            resumo(&outra_off),
            d(&outra, &outra_off)
        );
        eprintln!("[aud] outra C[S,B] {}", resumo(&outra_c));
        eprintln!("[aud] branco     {}", resumo(&branco));
        eprintln!("[aud] vazio      {}", resumo(&vazio));
        eprintln!(
            "[aud] vazio C[B] {} · vs vazio {}",
            resumo(&vazio_c),
            d(&vazio_c, &vazio)
        );
        eprintln!(
            "[aud] undo {undo_ok} redo {redo_ok} · undo C {undo_c_ok} redo C {redo_c_ok} · undo outra camada {undo_outra_ok}"
        );
    }
}

/// Per-Layer Color (Shape de 2 camadas: metade de cima amarela, de baixo verde) com e sem Wet blend, no branco.
#[test]
#[ignore = "sonda da auditoria"]
fn sonda_auditoria_per_layer_color() {
    const S: u32 = 64;
    let metade = |cima: bool| -> (Vec<u8>, u32, u32) {
        let v = (0..S * S)
            .map(|i| {
                let y = i / S;
                if (y < S / 2) == cima { 255 } else { 0 }
            })
            .collect();
        (v, S, S)
    };
    for nome in ["Round Mixing", "Oil Paint", "Finger Tip"] {
        let corre = |modo: Option<ModoMisturaOpaca>, por_camada: bool, f: Fundo| {
            let mut t = cena(nome, f);
            t.sonda_raio(30.0);
            t.set_brush_shape_layers(vec![metade(false), metade(true)]);
            if por_camada {
                t.toggle_brush_shape_per_layer_color();
                t.set_brush_shape_layer_color(0, [0.1, 0.8, 0.1]);
                t.set_brush_shape_layer_color(1, [1.0, 0.85, 0.0]);
            }
            if let Some(m) = modo {
                t.set_brush_mistura_modo(m);
            }
            traco(&mut t);
            t.run_full().0
        };
        for f in [Fundo::Branco, Fundo::Bloco] {
            let fz = if f == Fundo::Branco {
                "branco"
            } else {
                "bloco"
            };
            let wet_pl = corre(None, true, f);
            let wet_sem = corre(None, false, f);
            let off_pl = corre(Some(ModoMisturaOpaca::Off), true, f);
            let lado = |p: &[u8]| {
                format!(
                    "cima{:?} baixo{:?} cauda-cima{:?}",
                    media(p, 100, Y as usize - 12),
                    media(p, 100, Y as usize + 12),
                    media(p, 300, Y as usize - 12)
                )
            };
            let (m, x) = diff(&wet_pl, &wet_sem);
            eprintln!("\n[pl] ===== {nome} · {fz}");
            eprintln!("[pl] off+PerLayer {}", lado(&off_pl));
            eprintln!("[pl] wet+PerLayer {}", lado(&wet_pl));
            eprintln!("[pl] wet sem PL   {}", lado(&wet_sem));
            eprintln!("[pl] wet: PerLayer liga×desliga Δmédio {m:.3} máx {x}");
        }
    }
}

/// O Amount no limiar `AMOUNT_QUE_COBRE` (0,25) numa camada vazia, e o Multiply do Wet Wash Mix com a mistura.
#[test]
#[ignore = "sonda da auditoria"]
fn sonda_auditoria_limiar_e_multiply() {
    for a in [0.0f32, 0.1, 0.2, 0.24, 0.25, 0.26, 0.5, 1.0] {
        let mut t = cena("Round Mixing", Fundo::Vazio);
        t.set_brush_mistura_numero(0, a);
        traco(&mut t);
        let p = t.run_full().0;
        let mut b = cena("Round Mixing", Fundo::Bloco);
        b.set_brush_mistura_numero(0, a);
        traco(&mut b);
        let q = b.run_full().0;
        eprintln!(
            "[lim] amount {a:.2}: vazio x100 {:?} · bloco x210 {:?} cauda x300 {:?}",
            media(&p, 100, Y as usize),
            media(&q, 210, Y as usize),
            media(&q, 300, Y as usize)
        );
    }
    // O Multiply com a mistura ligada e desligada.
    for f in [Fundo::Bloco, Fundo::Vazio] {
        let mult = pinta("Wet Wash Mix", f, |_| {});
        let normal = pinta("Wet Wash Mix", f, |t| t.set_brush_blend(0));
        let mult_off = pinta("Wet Wash Mix", f, |t| {
            t.set_brush_mistura_modo(ModoMisturaOpaca::Off)
        });
        let normal_off = pinta("Wet Wash Mix", f, |t| {
            t.set_brush_mistura_modo(ModoMisturaOpaca::Off);
            t.set_brush_blend(0);
        });
        let (m, x) = diff(&mult, &normal);
        let (m2, x2) = diff(&mult_off, &normal_off);
        eprintln!(
            "[mul] {}: wet Multiply×Normal Δmédio {m:.3} máx {x} · off Multiply×Normal Δmédio {m2:.3} máx {x2}",
            if f == Fundo::Bloco { "bloco" } else { "vazio" }
        );
        eprintln!("[mul]   wet mult {}", resumo(&mult));
        eprintln!("[mul]   wet norm {}", resumo(&normal));
        eprintln!("[mul]   off mult {}", resumo(&mult_off));
    }
}

fn traco_n(t: &mut PainterTool, n: usize) {
    let cp = |pos: [f32; 2], phase| CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(cp([40.0, Y], PointerPhase::Down));
    for k in 1..=n {
        let u = k as f32 / n as f32;
        t.on_canvas_pointer(cp([40.0 + 300.0 * u, Y], PointerPhase::Move));
        t.on_tick(8.0);
    }
    t.on_canvas_pointer(cp([340.0, Y], PointerPhase::Up));
    t.on_tick(8.0);
}

/// Lote 2: a Strength da camada com a mistura, o estado partilhado entre camadas Brush, a receita do dono Off × Wet,
/// a Erase dentro do Composite, e a invariância ao fatiamento dos eventos.
#[test]
#[ignore = "sonda da auditoria"]
fn sonda_auditoria_lote2() {
    const ERASE: u8 = 3;
    for nome in ["Round Mixing", "Gouache", "Oil Paint", "Finger Tip"] {
        let p = |off: bool, pilha: &'static [(u8, f32)], n: usize| {
            let mut t = cena(nome, Fundo::Bloco);
            if off {
                t.set_brush_mistura_modo(ModoMisturaOpaca::Off);
            }
            if !pilha.is_empty() {
                monta_composite(&mut t, pilha);
            }
            traco_n(&mut t, n);
            t.run_full().0
        };
        let d = |a: &[u8], b: &[u8]| {
            let (m, x) = diff(a, b);
            format!("Δmédio {m:.3} máx {x}")
        };
        eprintln!("\n[l2] ===== {nome}");
        for off in [false, true] {
            let m = if off { "off" } else { "wet" };
            let s = p(off, &[], 60);
            let s120 = p(off, &[], 120);
            let bb = p(off, &[(BRUSH, 1.0), (BRUSH, 1.0)], 60);
            let bb120 = p(off, &[(BRUSH, 1.0), (BRUSH, 1.0)], 120);
            let bb01 = p(off, &[(BRUSH, 0.1), (BRUSH, 0.1)], 60);
            let sb = p(off, &[(SMEAR, 1.0), (BRUSH, 1.0)], 60);
            let rec = p(off, &[(BLUR, 1.0), (SMEAR, 0.25), (BRUSH, 0.10)], 60);
            let rec10 = p(off, &[(BLUR, 1.0), (SMEAR, 0.25), (BRUSH, 1.0)], 60);
            let eb = p(off, &[(ERASE, 1.0), (BRUSH, 1.0)], 60);
            let be = p(off, &[(BRUSH, 1.0), (ERASE, 1.0)], 60);
            eprintln!("[l2] {m} só          {}", resumo(&s));
            eprintln!("[l2] {m} 60×120 ev   {}", d(&s, &s120));
            eprintln!(
                "[l2] {m} C[B,B]      {} · vs só {}",
                resumo(&bb),
                d(&bb, &s)
            );
            eprintln!("[l2] {m} C[B,B] 60×120 {}", d(&bb, &bb120));
            eprintln!(
                "[l2] {m} C[B.1,B.1]  {} · vs C[B,B] {}",
                resumo(&bb01),
                d(&bb01, &bb)
            );
            eprintln!("[l2] {m} C[S,B]      {}", resumo(&sb));
            eprintln!("[l2] {m} receita     {}", resumo(&rec));
            eprintln!(
                "[l2] {m} receita B1  {} · Strength .1×1 {}",
                resumo(&rec10),
                d(&rec, &rec10)
            );
            eprintln!("[l2] {m} C[E,B]      {}", resumo(&eb));
            eprintln!("[l2] {m} C[B,E]      {}", resumo(&be));
        }
    }
}
