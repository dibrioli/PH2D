//! **A queda da água na ladeira do óleo** (smoke do dono 30/10): um Wet Flat pintado POR CIMA da crista de um Dabber
//! assentado escorre ladeira abaixo — (a) devagar com pouca água, depressa muito diluído; (b) igual com o Real Oil
//! ligado no Wet Flat (o óleo de ANTES continua a ser relevo para a água de um traço novo).

use super::agua_sobre_oleo_tests::{com_oleo, ponteiro, preset, relato};
use super::*;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};

/// O tique da tela.
const TIQUE: f32 = 1000.0 / 60.0;

/// A água do pincel × `escala` com o MESMO pigmento (E10: o `Water` do ficheiro já não decide a receita): o veículo
/// `D·h` × `escala`, o pigmento `(1−D)·h` fica — Dilution e Volume juntos.
fn agua_vezes(t: &mut PainterTool, escala: f64) {
    use ph2d_wet_paint::tuning::Knob;
    let k = &mut t.paint.wetpaint.knobs;
    let (d, h) = (k.get(Knob::Dilution), k.get(Knob::Volume));
    let (pig, veiculo) = ((1.0 - d) * h, escala * d * h);
    k.set(Knob::Dilution, veiculo / (pig + veiculo).max(1e-12));
    k.set(Knob::Volume, pig + veiculo);
}

/// O `escala` do [`agua_vezes`] que leva o Wet Flat à Dilution `d` com o MESMO pigmento: `(1−d₀)/d₀ · d/(1−d)`.
fn escala_da_diluicao(d: f64) -> f64 {
    let mut t = PainterTool::default();
    t.preset_aplicar(&preset("Watercolor/Fluid/Wet Flat.ph2d-brush.png"))
        .expect("o meio");
    let d0 = t
        .paint
        .wetpaint
        .knobs
        .get(ph2d_wet_paint::tuning::Knob::Dilution);
    (1.0 - d0) / d0 * d / (1.0 - d)
}

fn massas(t: &mut PainterTool) -> Vec<f32> {
    t.wet_bring_home();
    let g = &t
        .paint
        .wetpaint
        .session
        .as_ref()
        .expect("a sessão")
        .engine
        .layers[0]
        .grid;
    (0..g.cells).map(|i| g.sett[i] + g.susp[i]).collect()
}

/// A massa nova FORA do óleo (células com ≤ 1000 de massa antes) — o que desceu a ladeira — nos tiques `marcas`
/// depois de soltar um Wet Flat (água `escala` × a do preset, Real Oil `oleo`) ao longo da crista do Dabber.
pub(super) fn queda(escala: f64, oleo: bool, marcas: &[usize]) -> Vec<f64> {
    let mut t = com_oleo();
    let antes = massas(&mut t);
    t.preset_aplicar(&preset("Watercolor/Fluid/Wet Flat.ph2d-brush.png"))
        .expect("o meio");
    agua_vezes(&mut t, escala);
    t.set_wet_oleo_real_ligado(oleo);
    t.set_brush_size_px(12.0);
    t.set_brush_color_srgb8([240, 130, 60]);
    t.on_canvas_pointer(ponteiro(60.0, 90.0, PointerPhase::Down));
    for k in 1..=30 {
        t.on_canvas_pointer(ponteiro(60.0 + 4.0 * k as f32, 90.0, PointerPhase::Move));
        t.on_tick(TIQUE);
    }
    t.on_canvas_pointer(ponteiro(180.0, 90.0, PointerPhase::Up));
    {
        t.wet_bring_home();
        let g = &t
            .paint
            .wetpaint
            .session
            .as_ref()
            .expect("a sessão")
            .engine
            .layers[0]
            .grid;
        let f: Vec<f32> = (0..g.cells)
            .filter(|&i| antes[i] > 1000.0 && g.film[i] > 0.0)
            .map(|i| g.film[i])
            .collect();
        let m = f.iter().sum::<f32>() / f.len().max(1) as f32;
        let mx = f.iter().copied().fold(0.0f32, f32::max);
        relato(format!(
            "  água ×{escala} óleo {oleo}: film no óleo ao soltar média {m:.3} máx {mx:.3} ({} células)",
            f.len()
        ));
    }
    let mut out = Vec::new();
    let mut feito = 0;
    for &m in marcas {
        for _ in feito..m {
            t.on_tick(TIQUE);
        }
        feito = m;
        let agora = massas(&mut t);
        let fora: f64 = antes
            .iter()
            .zip(&agora)
            .filter(|(a, _)| **a <= 1000.0)
            .map(|(a, b)| f64::from(b - a).max(0.0))
            .sum();
        out.push(fora);
    }
    out
}

const MARCAS: [usize; 5] = [0, 15, 30, 60, 240];

/// SONDA — a velocidade da queda por diluição e com o Real Oil (tabela do relatório).
#[test]
#[ignore = "diagnóstico"]
fn diag_a_queda_na_ladeira() {
    relato(format!("tiques após soltar: {MARCAS:?}"));
    for oleo in [false, true] {
        for escala in [1.0, 0.5, 0.15] {
            let q = queda(escala, oleo, &MARCAS);
            let q: Vec<String> = q.iter().map(|v| format!("{v:.0}")).collect();
            relato(format!(
                "água ×{escala} Real Oil {oleo}: massa nova fora do óleo {}",
                q.join(" · ")
            ));
        }
    }
}

/// O que uma corrida J11 lê: a massa nova fora do óleo nas `marcas`, a distância média dessa massa ao eixo do óleo
/// (células) em cada marca, o `film` médio no óleo ao soltar e a imagem mostrada no fim.
pub(super) struct Leitura {
    pub(super) fora: Vec<f64>,
    pub(super) dist: Vec<f64>,
    film: f64,
    imagem: Vec<u8>,
}

fn grade(t: &mut PainterTool) -> (usize, Vec<f32>) {
    t.wet_bring_home();
    let g = &t
        .paint
        .wetpaint
        .session
        .as_ref()
        .expect("a sessão")
        .engine
        .layers[0]
        .grid;
    (g.s, (0..g.cells).map(|i| g.sett[i] + g.susp[i]).collect())
}

/// Um Wet Flat (água `escala` × a do preset, Real Oil ligado) ao longo da crista do Dabber assentado.
pub(super) fn queda_j11(escala: f64, marcas: &[usize]) -> Leitura {
    let mut t = com_oleo();
    let (s, antes) = grade(&mut t);
    // O eixo do óleo: a linha média das células com óleo.
    let (mut sy, mut n) = (0.0f64, 0.0f64);
    for (i, a) in antes.iter().enumerate() {
        if *a > 1000.0 {
            sy += (i / s) as f64;
            n += 1.0;
        }
    }
    let eixo = sy / n.max(1.0);
    t.preset_aplicar(&preset("Watercolor/Fluid/Wet Flat.ph2d-brush.png"))
        .expect("o meio");
    agua_vezes(&mut t, escala);
    t.set_wet_oleo_real_ligado(true);
    t.set_brush_size_px(12.0);
    t.set_brush_color_srgb8([240, 130, 60]);
    t.on_canvas_pointer(ponteiro(60.0, 90.0, PointerPhase::Down));
    for k in 1..=30 {
        t.on_canvas_pointer(ponteiro(60.0 + 4.0 * k as f32, 90.0, PointerPhase::Move));
        t.on_tick(TIQUE);
    }
    t.on_canvas_pointer(ponteiro(180.0, 90.0, PointerPhase::Up));
    let film = {
        t.wet_bring_home();
        let g = &t
            .paint
            .wetpaint
            .session
            .as_ref()
            .expect("a sessão")
            .engine
            .layers[0]
            .grid;
        let f: Vec<f32> = (0..g.cells)
            .filter(|&i| antes[i] > 1000.0 && g.film[i] > 0.0)
            .map(|i| g.film[i])
            .collect();
        f64::from(f.iter().sum::<f32>()) / f.len().max(1) as f64
    };
    let (mut fora, mut dist) = (Vec::new(), Vec::new());
    let mut feito = 0;
    for &m in marcas {
        for _ in feito..m {
            t.on_tick(TIQUE);
        }
        feito = m;
        let (_, agora) = grade(&mut t);
        let (mut mm, mut md) = (0.0f64, 0.0f64);
        for (i, (a, b)) in antes.iter().zip(&agora).enumerate() {
            if *a <= 1000.0 {
                let d = f64::from(b - a).max(0.0);
                mm += d;
                md += d * ((i / s) as f64 - eixo).abs();
            }
        }
        fora.push(mm);
        dist.push(md / mm.max(1e-9));
    }
    let imagem = super::agua_sobre_oleo_tests::mostrada(&mut t);
    Leitura {
        fora,
        dist,
        film,
        imagem,
    }
}

/// ⭐ **A queda cresce com a diluição** (dono 30/10, J11 — escolheu a V3 da prancha
/// `docs/Painter/Pinceis/pranchas/J11_queda_diluicao.png`): sobre a crista do óleo, o pincel quase seco não escorre
/// (≤ 10 % do que o cheio escorre) e a massa que DESCE depois de soltar cresce com a água: pouca < média < muita.
/// Régua J8 (`docs/Painter/Arquitetura/J8_vermelhos_antigos.md`): a queda = massa fora aos 240 tiques − ao soltar (a
/// de antes somava o que o pincel pousa fora do óleo — 584 → 4 426 com a E10) e a água × pela Dilution com o mesmo
/// pigmento ([`agua_vezes`]). Na J11 (847b472b3d): 234 · 68 977 · 119 325. Vermelho desde a E10g: 2 433 · 2 978 ·
/// 4 862 — a viscosidade do óleo ficava na célula seca e travava a água que chega. J9 (o meio anda com o líquido):
/// os pontos são Dilution 0,2 (quase seco) · 0,6 · a do preset (0,875) — o ×0,15 da J8 era Dilution 0,51, uma aguada
/// (curva medida: `docs/Painter/Arquitetura/J9_meio_no_liquido.md`).
#[test]
fn a_queda_cresce_com_a_diluicao() {
    let q: Vec<f64> = [escala_da_diluicao(0.2), escala_da_diluicao(0.6), 1.0]
        .iter()
        .map(|e| {
            let f = queda_j11(*e, &[0, 240]).fora;
            f[1] - f[0]
        })
        .collect();
    let (pouca, media, muita) = (q[0], q[1], q[2]);
    eprintln!("queda: pouca {pouca:.0} · média {media:.0} · muita {muita:.0}");
    assert!(
        pouca < media && media < muita,
        "pouca {pouca:.0} < média {media:.0} < muita {muita:.0}"
    );
    assert!(
        pouca <= 0.10 * muita,
        "o pincel quase seco ainda escorre: {pouca:.0} > 10 % de {muita:.0}"
    );
}

/// SONDA J11 — a queda pela diluição na lei do produto (tabela do relatório; grava `J11_<escala>.ppm` em
/// `PH2D_SONDA_DIR`).
#[test]
#[ignore = "diagnóstico J11: corre-se à mão com PH2D_SONDA_DIR"]
fn diag_a_queda_pela_diluicao() {
    const M: [usize; 4] = [15, 60, 120, 240];
    relato(format!("J11 marcas {M:?}"));
    for escala in [0.15, 0.5, 1.0] {
        let l = queda_j11(escala, &M);
        let f: Vec<String> = l.fora.iter().map(|x| format!("{x:.0}")).collect();
        let d: Vec<String> = l.dist.iter().map(|x| format!("{x:.1}")).collect();
        relato(format!(
            "água ×{escala} film {:.3} | massa fora {} | dist {}",
            l.film,
            f.join(" · "),
            d.join(" · ")
        ));
        super::agua_sobre_oleo_tests::grava(&format!("J11_{escala}"), &l.imagem);
    }
}

/// SONDA J8 — o que trava a queda do Wet Flat cheio na crista (a queda = massa fora aos 240 tiques − ao soltar), por
/// interruptor de medição, em série (são estáticos globais).
#[test]
#[ignore = "sonda J8"]
fn diag_o_que_trava_a_queda() {
    use ph2d_wet_paint::solver::regua::Regua;
    use ph2d_wet_paint::solver::{LADEIRA_ROTA, PAPEL_BINGHAM, PAPEL_CORPO};
    let corre = |nome: &str| {
        for e in [0.15, 1.0] {
            let f = queda_j11(e, &[0, 240]).fora;
            relato(format!(
                "{nome} água ×{e}: queda {:.0} (ao soltar {:.0})",
                f[1] - f[0],
                f[0]
            ));
        }
    };
    for e in [0.03, 0.06, 0.1, 0.15, 0.3, 0.5, 1.0] {
        let mut t = PainterTool::default();
        t.preset_aplicar(&preset("Watercolor/Fluid/Wet Flat.ph2d-brush.png"))
            .expect("o meio");
        agua_vezes(&mut t, e);
        let k = &t.paint.wetpaint.knobs;
        use ph2d_wet_paint::tuning::Knob;
        let f = queda_j11(e, &[0, 240]).fora;
        relato(format!(
            "curva água ×{e}: Dilution {:.3} Volume {:.3} queda {:.0}",
            k.get(Knob::Dilution),
            k.get(Knob::Volume),
            f[1] - f[0]
        ));
    }
    corre("produto");
    // B13.11: cada `static` de medição numa régua — o valor de antes volta no fim, também se o teste rebentar.
    let parado = Regua::poe(&ph2d_wet_paint::solver::MEIO_PARADO, true);
    corre("o meio parado na célula (antes da J9)");
    let bingham = Regua::poe(&PAPEL_BINGHAM, (-1.0f32).to_bits());
    corre("meio parado e sem Bingham");
    parado.muda(false);
    corre("meio anda e sem Bingham");
    let endurece = Regua::poe(&ph2d_wet_paint::meio::PAPEL_ENDURECE, 1.0f32.to_bits());
    let eta = Regua::poe(&ph2d_wet_paint::meio::PAPEL_ETA, 1.0f32.to_bits());
    corre("meio anda, o meio sem efeito (ETA 1, sem endurecer, sem Bingham)");
    parado.muda(true);
    corre("meio parado, o meio sem efeito (ETA 1, sem endurecer, sem Bingham)");
    parado.muda(false);
    bingham.muda(0);
    endurece.muda(0);
    corre("sem a viscosidade do óleo (ETA 1)");
    endurece.muda(1.0f32.to_bits());
    corre("ETA 1 e sem endurecer");
    eta.muda(0);
    endurece.muda(0);
    bingham.muda((-1.0f32).to_bits());
    corre("sem Bingham");
    bingham.muda(0);
    let corpo = Regua::poe(&PAPEL_CORPO, (1e-9f32).to_bits());
    corre("sem Body");
    corpo.muda(0);
    let ladeira = Regua::poe(&LADEIRA_ROTA, 2);
    corre("sem ladeira");
    ladeira.muda(1);
    corre("ladeira de 30/10");
}
