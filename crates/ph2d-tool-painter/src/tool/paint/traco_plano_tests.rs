//! Os portões do PLANO DO TRAÇO (`super::traco_plano`): o tecto por traço e as orlas (L8), a cauda e o
//! fall-off (L5) — pela porta do ponteiro, no Digital, sobre a tela transparente (o alfa É a cobertura).

use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_painter_brush::{BrushSpec, TracoComposto};

const AZUL: [f32; 3] = [23.0 / 255.0, 101.0 / 255.0, 179.0 / 255.0];

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Um tool Digital `w×h` sobre `fundo` (RGBA por texel) com o pincel `b`.
fn tool(w: u32, h: u32, fundo: [u8; 4], b: BrushSpec) -> PainterTool {
    let mut t = PainterTool::default();
    let mut px = vec![0u8; (w * h * 4) as usize];
    for p in px.as_chunks_mut::<4>().0 {
        *p = fundo;
    }
    t.set_source(px, w, h);
    t.set_paint_media(crate::PaintMedia::Digital);
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    t
}

/// Um traço pelos pontos `pts` (um evento por ponto, um tique por evento); `levanta = false` deixa-o aberto.
fn traco(t: &mut PainterTool, pts: &[[f32; 2]], levanta: bool) {
    t.on_canvas_pointer(cp(pts[0], PointerPhase::Down));
    for p in &pts[1..] {
        t.on_canvas_pointer(cp(*p, PointerPhase::Move));
        t.paint_tick(1.0 / 60.0);
    }
    if levanta {
        t.on_canvas_pointer(cp(*pts.last().unwrap(), PointerPhase::Up));
    }
}

fn pincel(raio: f32) -> BrushSpec {
    BrushSpec {
        radius_px: raio,
        color: AZUL,
        ..Default::default()
    }
}

/// O alfa do texel `(x, y)`.
fn alfa(t: &PainterTool, w: u32, x: u32, y: u32) -> f32 {
    f32::from(t.canvas_rgba[((y * w + x) * 4 + 3) as usize]) / 255.0
}

/// Um laço que se CRUZA: uma volta e um quarto de círculo de raio 40 em torno de (100, 100).
fn laco() -> Vec<[f32; 2]> {
    (0..=150)
        .map(|k| {
            let a = k as f32 / 150.0 * 2.5 * std::f32::consts::PI;
            [100.0 + 40.0 * a.cos(), 100.0 + 40.0 * a.sin()]
        })
        .collect()
}

/// O `k` da régua do 06 §6: `ln T(sobreposição) ÷ ln T(recto)` — o cruzamento em (140, 100) (o início, que a
/// volta repassa) contra o braço de passagem única em (60, 100).
fn k_do_laco(b: BrushSpec) -> (f32, f32, f32) {
    let mut t = tool(200, 200, [0; 4], b);
    traco(&mut t, &laco(), true);
    let max = t.canvas_rgba.chunks(4).map(|p| p[3]).max().unwrap();
    let uma = alfa(&t, 200, 60, 100);
    let duas = alfa(&t, 200, 140, 100);
    let k = (1.0 - duas).max(1e-3).ln() / (1.0 - uma).max(1e-3).ln();
    (f32::from(max) / 255.0, uma, k)
}

/// O aerógrafo da régua: ACUMULA dentro do traço (o airbrush do Rebelle, `k` 1,8–4,5).
fn aerografo() -> BrushSpec {
    BrushSpec {
        accumulate: true,
        flow: 0.15,
        spacing: 0.25,
        ..pincel(12.0)
    }
}

/// ⭐ **L8 — o tecto do traço não escurece no próprio cruzamento** (06 §6: as aguadas e os glazes do Rebelle
/// tapam a opacidade dentro de UM traço, `k` 1,00–1,15). Com o tecto 0,25 (abaixo da passagem única, o glaze
/// UNIFORME) o laço que se cruza nunca passa dele e o `k` cai na faixa dos glazes; sem o tecto o mesmo pincel
/// acumula (`k` dos aerógrafos).
#[test]
fn o_tecto_do_traco_nao_escurece_no_proprio_cruzamento() {
    let (max0, uma0, k0) = k_do_laco(aerografo());
    assert!(
        max0 > uma0 + 0.12 && k0 > 1.6,
        "sem tecto o aerógrafo acumula: máx {max0} · k {k0}"
    );
    let b = BrushSpec {
        traco: TracoComposto {
            tecto: 0.25,
            ..TracoComposto::NEUTRO
        },
        ..aerografo()
    };
    let (max, uma, k) = k_do_laco(b);
    assert!(
        max <= 0.25 + 1.5 / 255.0,
        "o cruzamento passou do tecto: {max}"
    );
    assert!(max >= 0.24, "o tecto não chega a ser atingido: {max}");
    assert!(
        (uma - uma0.min(0.25)).abs() < 0.02,
        "a passagem única: {uma} × {uma0}"
    );
    assert!(
        (1.0..=1.15).contains(&k),
        "k {k} fora dos glazes do Rebelle (1,00–1,15)"
    );
}

/// ⭐ **L8 — sem tecto o carimbo é o de hoje**: o pincel neutro não arma o plano (nem nasce a sessão), e o plano
/// em `Mix` com o tecto inerte (1,0) dá a MESMA tinta que o carimbo de sempre (a lei do Accumulate desligado
/// telescopa para `pré·(1 − m) + cor·m`; a composição do plano é essa conta numa passagem) — ± 2/255.
#[test]
fn sem_tecto_o_carimbo_e_o_de_hoje() {
    let b = BrushSpec {
        strength: 0.6,
        ..pincel(10.0)
    };
    let pts: Vec<[f32; 2]> = (0..=40).map(|k| [20.0 + 4.0 * k as f32, 50.0]).collect();
    let mut hoje = tool(200, 100, [200, 120, 60, 255], b);
    traco(&mut hoje, &pts, false);
    assert!(
        hoje.paint.plano_do_traco.is_none(),
        "o neutro armou o plano"
    );
    hoje.on_canvas_pointer(cp(pts[40], PointerPhase::Up));
    let mut plano = tool(
        200,
        100,
        [200, 120, 60, 255],
        BrushSpec {
            traco: TracoComposto {
                tecto: 1.0,
                ..TracoComposto::NEUTRO
            },
            ..b
        },
    );
    traco(&mut plano, &pts, false);
    assert!(
        plano.paint.plano_do_traco.is_some(),
        "o tecto não armou o plano"
    );
    plano.on_canvas_pointer(cp(pts[40], PointerPhase::Up));
    let pior = hoje
        .canvas_rgba
        .iter()
        .zip(plano.canvas_rgba.iter())
        .map(|(a, b)| a.abs_diff(*b))
        .max()
        .unwrap();
    assert!(
        pior <= 2,
        "o plano em Mix desviou {pior}/255 do carimbo de hoje"
    );
}

/// O alfa ao longo da vertical `x` de um recto horizontal em `y = 50`, raio 18, na tela transparente.
fn perfil(traco_c: TracoComposto, fundo: [u8; 4]) -> (PainterTool, Vec<f32>) {
    let b = BrushSpec {
        traco: traco_c,
        hardness: 0.9,
        ..pincel(18.0)
    };
    let mut t = tool(220, 100, fundo, b);
    let pts: Vec<[f32; 2]> = (0..=40).map(|k| [20.0 + 4.5 * k as f32, 50.0]).collect();
    traco(&mut t, &pts, true);
    let p = (0..100).map(|y| alfa(&t, 220, 110, y)).collect();
    (t, p)
}

/// ⭐ **L8 — a orla molhada escurece a borda e clareia o miolo** (*Wet edges*; o aro da Watercolor): o miolo
/// cai para `1 − molhada` da opacidade do traço e a orla fica perto dela; sem orla o miolo é o máximo.
#[test]
fn a_orla_molhada_guarda_a_borda_e_clareia_o_miolo() {
    let (_, sem) = perfil(TracoComposto::NEUTRO, [0; 4]);
    let (_, com) = perfil(
        TracoComposto {
            molhada: 0.7,
            ..TracoComposto::NEUTRO
        },
        [0; 4],
    );
    let (miolo_sem, miolo) = (sem[50], com[50]);
    let orla = com.iter().copied().fold(0.0f32, f32::max);
    assert!(miolo_sem > 0.95, "o miolo sem orla é opaco: {miolo_sem}");
    assert!(
        (miolo - 0.3).abs() < 0.05,
        "o miolo molhado é 1 − 0,7 da opacidade: {miolo}"
    );
    assert!(
        orla > 0.8,
        "a orla guarda a opacidade do traço: {orla} (miolo {miolo})"
    );
}

/// ⭐ **L8 — a orla queimada escurece a borda sobre a tinta** (*Burnt edges*, Linear Burn): sobre um cinza
/// opaco, a borda do traço fica mais escura com a orla queimada; o miolo não muda.
#[test]
fn a_orla_queimada_escurece_a_borda() {
    let cinza = [150, 150, 150, 255];
    let (sem, _) = perfil(TracoComposto::NEUTRO, cinza);
    let (com, _) = perfil(
        TracoComposto {
            queimada: 1.0,
            ..TracoComposto::NEUTRO
        },
        cinza,
    );
    let luma = |t: &PainterTool, y: u32| {
        let i = ((y * 220 + 110) * 4) as usize;
        u32::from(t.canvas_rgba[i])
            + u32::from(t.canvas_rgba[i + 1])
            + u32::from(t.canvas_rgba[i + 2])
    };
    let mais_escura = (30..70)
        .filter(|&y| luma(&com, y) + 6 < luma(&sem, y))
        .count();
    assert!(mais_escura >= 2, "nenhuma linha da borda escureceu");
    assert_eq!(luma(&com, 50), luma(&sem, 50), "o miolo mudou");
}

/// A altura (texels com alfa ≥ 0,5) da coluna `x`, na tela 400×100 de um recto horizontal em `y = 50`.
fn altura(t: &PainterTool, x: u32) -> usize {
    (0..100).filter(|&y| alfa(t, 400, x, y) >= 0.5).count()
}

fn recto_afilado(end: f32, levanta: bool) -> PainterTool {
    recto_afilado_com(end, levanta, false)
}

/// O recto afilado com ou sem a CÁPSULA (`continuous_spacing`: cada dab interpola de volta ao estado do anterior).
fn recto_afilado_com(end: f32, levanta: bool, capsula: bool) -> PainterTool {
    let mut b = pincel(10.0);
    b.hardness = 1.0;
    b.continuous_spacing = capsula;
    b.taper.start = 3.0;
    b.taper.end = end;
    let mut t = tool(400, 100, [0; 4], b);
    let pts: Vec<[f32; 2]> = (0..=80).map(|k| [40.0 + 4.0 * k as f32, 50.0]).collect();
    traco(&mut t, &pts, levanta);
    t
}

/// ⭐ **L5 — a cauda afila como a cabeça** (a lei do Procreate: o fim re-pinta-se ao levantar). Com
/// `start = end = 3·D` e as duas pontas agudas, a largura a `k` px do fim é a largura a `k` px do início (± 2
/// px); antes de levantar a cauda ainda é larga (a prévia não é o documento); e o traço inteiro, cauda
/// incluída, desfaz-se num passo.
#[test]
fn a_cauda_afila_como_a_cabeca() {
    let aberto = recto_afilado(3.0, false);
    assert!(
        altura(&aberto, 355) >= 18,
        "antes de levantar a cauda é a do corpo"
    );
    let t = recto_afilado(3.0, true);
    let (x0, x1) = (40u32, 360u32);
    for k in (6..60).step_by(6) {
        let (h, c) = (altura(&t, x0 + k), altura(&t, x1 - k));
        assert!(h.abs_diff(c) <= 2, "a {k} px: cabeça {h} × cauda {c}");
    }
    assert!(
        altura(&t, x1 - 6) < altura(&t, 200) / 2,
        "a cauda não afilou"
    );
    let sem = recto_afilado(0.0, true);
    assert!(altura(&sem, x1 - 6) >= 18, "sem cauda o fim é o de hoje");
    let mut t = t;
    assert!(t.undo_last(), "o traço não deixou passo de desfazer");
    assert!(
        t.canvas_rgba.iter().all(|&b| b == 0),
        "o desfazer deixou tinta: a cauda é outro passo"
    );
}

/// ⭐ **L5 — a cauda com a CÁPSULA espelha a cabeça** (J3, `docs/Painter/Pinceis/exames/canetas.md` 2.ª onda): cada
/// dab da cápsula interpola de volta ao estado do dab ANTERIOR (`Dab::ante`), e a cauda re-afilava o dab sem
/// re-afilar esse estado — o sub-dab de trás voltava ao raio CHEIO e o fim saía uma bolha de ~½ largura
/// (`pranchas/canetas/cauda_L5_bolha_no_fim.png`). A largura a `k` px do fim é a de `k` px do início (± 2 px) e
/// nenhuma coluna da cauda passa da cabeça à mesma distância por mais de 2 px.
#[test]
fn a_cauda_com_a_capsula_afila_como_a_cabeca() {
    let t = recto_afilado_com(3.0, true, true);
    let (x0, x1) = (40u32, 360u32);
    let mut falhas = Vec::new();
    for k in (2..60).step_by(2) {
        let (h, c) = (altura(&t, x0 + k), altura(&t, x1 - k));
        if h.abs_diff(c) > 2 {
            falhas.push(format!("a {k} px: cabeça {h} × cauda {c}"));
        }
    }
    assert!(
        falhas.is_empty(),
        "a cauda com a cápsula não espelha a cabeça:\n{}",
        falhas.join("\n")
    );
}

/// O ArtRage Oil **ThinAndDry** (`fixtures/artrage_bateria/OilBrush/ThinAndDry_tamanhos_A.png`, recto de 880
/// px a 40 px): a cobertura por bloco de 40 px ÷ a dos blocos 1–2 (a régua da curva do 06 §5) — a carga acaba.
const THIN_AND_DRY: [f32; 22] = [
    0.79, 1.00, 1.00, 1.00, 1.00, 1.00, 1.00, 0.93, 0.83, 0.69, 0.62, 0.57, 0.53, 0.25, 0.07, 0.02,
    0.01, 0.01, 0.01, 0.00, 0.00, 0.00,
];

/// ⭐ **L5 — o fall-off esmaece ao longo** contra o oráculo que o TEM. ⚠️ O Charcoal Wide do Rebelle (a «0,63»
/// do 06 §5) NÃO tem fall-off: medido por bloco de 40 px (a mesma régua) a curva dele é ruído (0,56–1,18,
/// sem tendência) e o 0,63 era o bloco da TAMPA do fim; o Rebelle não esgota (`paint_fadeout_length` = 1 nos
/// 281). O ArtRage ThinAndDry esgota: patamar até ~7·D e nada aos ~15·D. Com `fall_off_start 6` e
/// `fall_off 10` (a mesma área de carga), a curva do recto: patamar ≥ 0,97, o fim ≤ 0,05, a carga total ± 10 % e
/// cada bloco ± 0,2 (a forma do smoothstep contra a do óleo, que cai em dois degraus: o ombro 0,57–0,53 e o
/// seco aos 540 px).
#[test]
fn o_fall_off_esmaece_ao_longo() {
    let mut b = pincel(20.0);
    b.hardness = 1.0;
    b.taper.fall_off_start = 6.0;
    b.taper.fall_off = 10.0;
    let mut t = tool(1000, 80, [0; 4], b);
    let pts: Vec<[f32; 2]> = (0..=220).map(|k| [60.0 + 4.0 * k as f32, 40.0]).collect();
    traco(&mut t, &pts, true);
    let blocos: Vec<f32> = (0..22)
        .map(|i| {
            (0..40)
                .map(|j| alfa(&t, 1000, 60 + 40 * i + j, 40))
                .sum::<f32>()
                / 40.0
        })
        .collect();
    let c0 = (blocos[1] + blocos[2]) / 2.0;
    let curva: Vec<f32> = blocos.iter().map(|v| v / c0).collect();
    assert!(
        curva[1..7].iter().all(|&v| v >= 0.97),
        "o patamar caiu: {curva:?}"
    );
    assert!(
        curva[16..].iter().all(|&v| v <= 0.05),
        "a carga não acabou: {curva:?}"
    );
    let (nosso, oraculo): (f32, f32) = (curva[1..].iter().sum(), THIN_AND_DRY[1..].iter().sum());
    assert!(
        (nosso / oraculo - 1.0).abs() <= 0.10,
        "carga {nosso} × ArtRage {oraculo}: {curva:?}"
    );
    for (i, (a, o)) in curva.iter().zip(THIN_AND_DRY).enumerate().skip(1) {
        assert!((a - o).abs() <= 0.2, "bloco {i}: {a} × ArtRage {o}");
    }
}

/// Um recto de 600 px no Wet Paint sobre papel branco (relógio fixo), com `f` a mexer no tool; a tela.
fn recto_wet(f: impl Fn(&mut PainterTool)) -> Vec<u8> {
    use ph2d_editor_core::Tool;
    let (w, h) = (700usize, 80usize);
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; w * h * 4], w as u32, h as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    t.set_brush_size_px(16.0);
    t.set_brush_color_srgb8([30, 60, 160]);
    f(&mut t);
    let p = |x: f32, phase| cp([x, 40.0], phase);
    t.on_canvas_pointer(p(40.0, PointerPhase::Down));
    for k in 1..=150 {
        t.on_canvas_pointer(p(40.0 + 4.0 * k as f32, PointerPhase::Move));
        t.on_tick(8.0);
    }
    t.on_canvas_pointer(p(640.0, PointerPhase::Up));
    for _ in 0..30 {
        t.on_tick(1000.0 / 60.0);
    }
    t.canvas_rgba.to_vec()
}

/// A escuridão média (`1 − R/255`) da linha `y = 40` por bloco de 50 px, de x = 50 a 600.
fn blocos_wet(px: &[u8]) -> Vec<f32> {
    (0..11)
        .map(|b| {
            (0..50)
                .map(|j| 1.0 - f32::from(px[(40 * 700 + 50 + 50 * b + j) * 4]) / 255.0)
                .sum::<f32>()
                / 50.0
        })
        .collect()
}

/// ⭐ **L5 — o `tipClean` do Wet NÃO é um fall-off** (medido antes de construir, 03 §6): ele é a LIMPEZA da cor
/// da ponta de volta à do traço (`ph2d-wet-paint` `trail/transfer.rs`, «Tip self-cleaning»), não uma carga que
/// acaba — no papel limpo não há o que limpar e o recto de 600 px é o MESMO com ele a zero e no máximo (0,05).
/// O fall-off do Wet é a carga do pigmento (o que cada dab pousa, `dab_route`): esse sim esmaece ao longo.
#[test]
fn o_tip_clean_nao_e_um_fall_off_e_o_fall_off_chega_ao_wet() {
    use ph2d_wet_paint::tuning::Knob;
    let sem = blocos_wet(&recto_wet(|t| {
        t.paint.wetpaint.knobs.set(Knob::TipClean, 0.0)
    }));
    let limpo = blocos_wet(&recto_wet(|t| {
        t.paint.wetpaint.knobs.set(Knob::TipClean, 0.05)
    }));
    for (i, (a, b)) in sem.iter().zip(&limpo).enumerate() {
        assert!(
            (a - b).abs() < 0.01,
            "bloco {i}: tipClean mudou a carga {a} → {b}"
        );
    }
    assert!(
        sem[10] > 0.8 * sem[1],
        "sem fall-off o recto do Wet não esmaece: {sem:?}"
    );
    let caindo = blocos_wet(&recto_wet(|t| {
        t.paint.brush.taper.fall_off_start = 4.0;
        t.paint.brush.taper.fall_off = 12.0;
    }));
    assert!(
        caindo[1] > 0.8 * sem[1],
        "o patamar do Wet caiu: {caindo:?}"
    );
    // O pincel que esgota ainda ARRASTA a tinta molhada que já pousou (a lane leva-a): o recto esmaece aos poucos
    // depois da carga, não se corta — medido 20/10: 0,36 → 0,25 no fim (sem fall-off fica 0,35).
    assert!(
        caindo[10] < 0.8 * caindo[1],
        "o fall-off não chegou ao Wet: {caindo:?}"
    );
    assert!(
        caindo[10] < sem[10] - 0.05,
        "o fall-off não tirou carga: {caindo:?} × {sem:?}"
    );
    assert!(
        caindo.windows(2).skip(4).all(|w| w[1] <= w[0] + 0.01),
        "não esmaece ao longo: {caindo:?}"
    );
}

/// ⭐ **Revisão 1 — o plano guarda a mistura K–M do pincel**: com Pigment, o azul sobre o amarelo dá o verde do
/// carimbo de hoje; o plano (o tecto inerte 1,0) compõe com a MESMA mistura — ± 3/255 em todo texel a mais de um
/// texel da borda do disco (medido: 3 no ombro, o K–M em vários passos × num passo; 1 no miolo). Vermelho antes da cura: o plano compunha sem pigmento (o miolo ficava AZUL, ~180/255).
/// ⚠️ Medido: na franja de um texel (cobertura < 1/255) o K–M é íngreme — `t = 0,02` de azul tira 14/255 ao
/// vermelho do amarelo — e o alfa de 8 bits do plano arredonda-a a zero: até 14/255 em ~50 valores da franja.
#[test]
fn o_plano_mistura_o_pigmento_como_o_carimbo() {
    let b = BrushSpec {
        pigment: true,
        pigment_mix: 1.0,
        strength: 0.7,
        ..pincel(10.0)
    };
    let pts: Vec<[f32; 2]> = (0..=40).map(|k| [20.0 + 4.0 * k as f32, 50.0]).collect();
    let amarelo = [235, 200, 30, 255];
    let mut hoje = tool(200, 100, amarelo, b);
    traco(&mut hoje, &pts, true);
    let tecto = TracoComposto {
        tecto: 1.0,
        ..TracoComposto::NEUTRO
    };
    let mut plano = tool(200, 100, amarelo, BrushSpec { traco: tecto, ..b });
    traco(&mut plano, &pts, true);
    let i = ((50 * 200 + 100) * 4) as usize;
    assert!(
        hoje.canvas_rgba[i + 1] > hoje.canvas_rgba[i + 2],
        "fixtura: o K–M de hoje dá verde"
    );
    // A distância de cada texel ao segmento do traço (x 20..180, y 50).
    let dentro = |p: usize| {
        let (x, y) = ((p % 200) as f32 + 0.5, (p / 200) as f32 + 0.5);
        let dx = (x - x.clamp(20.0, 180.0)).abs();
        (dx * dx + (y - 50.0) * (y - 50.0)).sqrt() <= 10.0 - 1.5
    };
    let (mut miolo, mut franja, mut n_franja) = (0u8, 0u8, 0usize);
    for (k, (a, b)) in hoje
        .canvas_rgba
        .iter()
        .zip(plano.canvas_rgba.iter())
        .enumerate()
    {
        let d = a.abs_diff(*b);
        if dentro(k / 4) {
            miolo = miolo.max(d);
        } else {
            franja = franja.max(d);
            n_franja += usize::from(d > 2);
        }
    }
    assert!(
        miolo <= 3,
        "o plano desviou {miolo}/255 da mistura K–M do carimbo"
    );
    assert!(
        franja <= 16 && n_franja <= 80,
        "a franja: {franja}/255 em {n_franja} valores"
    );
}

/// ⭐ **Revisão 6 — o traço curto e o toque** (a lei do Procreate: cabeça + cauda ≤ comprimento, as duas janelas
/// escalam). O toque com cabeça e cauda armadas fica o PONTO cheio (não encolhe a 1 px ao levantar); o traço de
/// 40 px com janelas de 60 + 60 px afila simétrico e chega à largura cheia no meio.
#[test]
fn o_toque_e_o_traco_curto_escalam_as_janelas() {
    let afilado = |end: f32| {
        let mut b = pincel(10.0);
        b.hardness = 1.0;
        b.taper.start = 3.0;
        b.taper.end = end;
        b
    };
    let cheio = {
        let mut t = tool(
            400,
            100,
            [0; 4],
            BrushSpec {
                hardness: 1.0,
                ..pincel(10.0)
            },
        );
        traco(&mut t, &[[200.0, 50.0], [200.0, 50.0]], true);
        altura(&t, 200)
    };
    let mut toque = tool(400, 100, [0; 4], afilado(3.0));
    traco(&mut toque, &[[200.0, 50.0], [200.0, 50.0]], true);
    assert!(
        altura(&toque, 200) + 2 >= cheio,
        "o toque encolheu: {} × {cheio}",
        altura(&toque, 200)
    );
    let mut curto = tool(400, 100, [0; 4], afilado(3.0));
    let pts: Vec<[f32; 2]> = (0..=10).map(|k| [180.0 + 4.0 * k as f32, 50.0]).collect();
    traco(&mut curto, &pts, true);
    let max = (180..=220).map(|x| altura(&curto, x)).max().unwrap();
    assert!(
        max + 2 >= cheio,
        "o traço curto não chega à largura cheia: {max} × {cheio}"
    );
    // As pontas pintadas (o fim do traço é o último dab, não o ponteiro).
    let tinta: Vec<u32> = (0..400).filter(|&x| altura(&curto, x) > 0).collect();
    let (x0, x1) = (tinta[0], *tinta.last().unwrap());
    for k in (3..(x1 - x0) / 2).step_by(2) {
        let (h, c) = (altura(&curto, x0 + k), altura(&curto, x1 - k));
        assert!(
            h.abs_diff(c) <= 2,
            "a {k} px de {x0}..{x1}: cabeça {h} × cauda {c}"
        );
    }
}

/// ⭐ **Revisão 5 — a cauda re-carimba com o MESMO sorteio**: com o grão em «Random» (um desvio sorteado por dab),
/// a faixa logo DENTRO da janela da cauda — tinta dos dabs de antes dela e dos primeiros da cauda, ainda à largura
/// cheia — é a do mesmo traço sem cauda (± 8/255). Vermelho antes da cura: o re-carimbo sorteava de novo (costura).
#[test]
fn a_cauda_nao_costura_o_grao_aleatorio() {
    let grao = |end: f32| {
        let mut b = pincel(10.0);
        b.texture = ph2d_painter_brush::TextureSettings {
            kind: ph2d_painter_brush::TextureKind::Noise,
            mapping: ph2d_painter_brush::TextureMapping::Random,
            ..b.texture
        };
        b.grain_depth = 1.0;
        b.taper.end = end;
        b.traco.tecto = 1.0; // o plano armado nos dois: só a cauda muda
        b
    };
    let pts: Vec<[f32; 2]> = (0..=80).map(|k| [40.0 + 4.0 * k as f32, 50.0]).collect();
    let mut sem = tool(400, 100, [255, 255, 255, 255], grao(0.0));
    traco(&mut sem, &pts, true);
    let mut com = tool(400, 100, [255, 255, 255, 255], grao(3.0));
    traco(&mut com, &pts, true);
    // A janela da cauda começa em x = 360 − 60 = 300; a faixa limpa começa um raio antes.
    let pior = (290..298)
        .flat_map(|x| (35..65).map(move |y| (x, y)))
        .map(|(x, y)| {
            let i = ((y * 400 + x) * 4) as usize;
            (0..4)
                .map(|k| sem.canvas_rgba[i + k].abs_diff(com.canvas_rgba[i + k]))
                .max()
                .unwrap()
        })
        .max()
        .unwrap();
    assert!(
        pior <= 8,
        "a cauda costurou o grão aleatório: {pior}/255 na faixa de antes dela"
    );
}

/// ⭐ **Revisão 10 — trocar de documento a meio do traço abandona o plano** (a lei do cancelar): ele segurava duas
/// telas do documento velho.
#[test]
fn o_reset_abandona_o_plano_do_traco() {
    let b = BrushSpec {
        traco: TracoComposto {
            tecto: 0.5,
            ..TracoComposto::NEUTRO
        },
        ..pincel(10.0)
    };
    let mut t = tool(200, 100, [0; 4], b);
    traco(&mut t, &[[20.0, 50.0], [60.0, 50.0], [100.0, 50.0]], false);
    assert!(t.paint.plano_do_traco.is_some(), "fixtura: o plano armou");
    t.reset_transient_edit_state();
    assert!(
        t.paint.plano_do_traco.is_none(),
        "o plano sobreviveu ao reset"
    );
}

/// ⭐ **Revisão 7 — trocas de plano ANINHADAS** (o gate de proteção e o plano do traço por dentro): o journal sabe
/// que está fora da tela até à ÚLTIMA saída. Vermelho antes da cura: o XOR dava «na tela» depois de duas entradas.
#[test]
fn as_trocas_de_plano_aninhadas_ficam_fora_da_tela() {
    use std::sync::Arc;
    let w = crate::undo::window::WriteState::default();
    let (mut tela, mut livre, mut plano) = (
        Arc::new(vec![1u8]),
        Arc::new(vec![2u8]),
        Arc::new(vec![3u8]),
    );
    super::plane_fork::swap_canvas_plane(&mut tela, &mut livre, &w);
    assert!(w.on_foreign_plane());
    super::plane_fork::swap_canvas_plane(&mut tela, &mut plano, &w);
    assert!(w.on_foreign_plane(), "duas entradas: ainda fora da tela");
    super::plane_fork::swap_canvas_plane(&mut tela, &mut plano, &w);
    assert!(w.on_foreign_plane(), "uma saída: ainda no plano livre");
    super::plane_fork::swap_canvas_plane(&mut tela, &mut livre, &w);
    assert!(!w.on_foreign_plane(), "as duas saídas: de volta à tela");
    assert_eq!(*tela, vec![1u8]);
}
