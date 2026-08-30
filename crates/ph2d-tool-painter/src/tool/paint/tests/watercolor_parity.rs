//! **A paridade INCREMENTAL × FULL do composite da aquarela.** A investigação de 2026-07: o
//! recomposite incremental tem de dar o mesmo byte que o recomposite inteiro, e quando não dá, qual
//! termo carrega o resíduo, onde ele diverge, e se a janela em si move os pixels.

use super::*;

/// DIFERENCIAL (diagnóstico da regressão do Spread, Enio 2026-07-06): o composite incremental
/// (dirty-rect por frame) deve ser equivalente a recompor o bbox cumulativo inteiro a cada frame
/// (o comportamento antigo). Pinta um traço vivo com Edge/Spread/Warp/Granulation realistas, então
/// força UMA recomposição full do cumulativo (sem commit) e compara byte a byte (tolerância ±1 por
/// arredondamento de prefix-sum do blur). Divergência ⇒ o dirty-rect deixa pixels stale.
#[test]
fn watercolor_incremental_composite_matches_full_recompose() {
    let size = 256u32;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    t.paint.brush = BrushSpec {
        radius_px: 14.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.24, 0.39, 0.63],
        space_attenuation: false,
        watercolor: true,
        edge_gain: 3.0,
        edge_spread: 12.0,
        granulation: 0.4,
        warp: 2.5,
        fill: 0.35,
        depth: 2.0,
        ..Default::default()
    };
    t.paint.brush_by_mode.fill(t.paint.brush);
    // Long diagonal live stroke, pen still down — TWO Moves per frame (a 120 Hz mouse at 60 fps), so
    // each composite covers the UNION of that frame's dabs. That is the harder case for the property
    // under test: a wider window has more room to leave a pixel stale, not less.
    assert!(t.on_canvas_pointer(cp([30.0, 30.0], PointerPhase::Down)));
    for i in 1..=40 {
        let p = 30.0 + i as f32 * 4.5;
        t.on_canvas_pointer(cp([p, 30.0 + i as f32 * 3.5], PointerPhase::Move));
        if i % 2 == 0 {
            frame(&mut t);
        }
    }
    let incremental: Vec<u8> = t.canvas_rgba.to_vec();
    // Force ONE full recompose of the whole cumulative bbox (what the old code did every frame).
    t.paint.wet_frame_dirty = t.paint.wet_cum_dirty;
    t.apply_watercolor(false);
    let full: Vec<u8> = t.canvas_rgba.to_vec();
    let mut worst = 0i32;
    let mut worst_i = 0usize;
    for (i, (a, b)) in incremental.iter().zip(full.iter()).enumerate() {
        let d = (i32::from(*a) - i32::from(*b)).abs();
        if d > worst {
            worst = d;
            worst_i = i;
        }
    }
    assert!(
        worst <= 1,
        "incremental deixou pixel stale: Δ{} no byte {} (px {},{})",
        worst,
        worst_i,
        (worst_i / 4) % size as usize,
        (worst_i / 4) / size as usize
    );
}

/// **Paridade incremental com ÁGUA (Enio 2026-07-09, "retângulo no preview com Charge 1 +
/// Dilution > 0"):** o composite vivo por dirty-rect tem que bater com a recomposição full
/// TAMBÉM com o canal d'água ativo — o anel lê o halo numa coordenada SERRILHADA (±JAG_PX), e a
/// janela viva não padava esse deslocamento: perto da borda da janela o blur do halo perdia
/// suporte e os valores mudavam a cada frame (retângulos que somem no pen-up).
#[test]
fn watercolor_incremental_composite_matches_full_with_water() {
    let size = 256u32;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    t.paint.brush = BrushSpec {
        radius_px: 14.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.24, 0.39, 0.63],
        space_attenuation: false,
        watercolor: true,
        edge_gain: 3.0,
        edge_spread: 12.0,
        granulation: 0.4,
        warp: 2.5,
        fill: 0.35,
        depth: 2.0,
        wet_rewet: 0.3,
        wet_dilution: 0.6, // água carregada — o caso do smoke (Charge 1 default)
        ..Default::default()
    };
    t.paint.brush_by_mode.fill(t.paint.brush);
    // Wash A assado (SEM água) — o anel/lift d'água só liga sobre PIGMENTO (bp_ring > 0); numa
    // tela virgem o halo nem é lido e a paridade passa vazia.
    t.paint.brush.wet_dilution = 0.0;
    t.paint.brush_by_mode.fill(t.paint.brush);
    assert!(t.on_canvas_pointer(cp([30.0, 60.0], PointerPhase::Down)));
    for i in 1..=40 {
        t.on_canvas_pointer(cp([30.0 + i as f32 * 4.5, 60.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([210.0, 60.0], PointerPhase::Up));
    // Traço B com ÁGUA, VIVO, cruzando o wash em diagonal — paridade no estado ao vivo.
    t.paint.brush.wet_dilution = 0.6;
    t.paint.brush_by_mode.fill(t.paint.brush);
    assert!(t.on_canvas_pointer(cp([30.0, 30.0], PointerPhase::Down)));
    for i in 1..=40 {
        let p = 30.0 + i as f32 * 4.5;
        t.on_canvas_pointer(cp([p, 30.0 + i as f32 * 3.5], PointerPhase::Move));
        if i % 2 == 0 {
            frame(&mut t); // dois Moves por quadro: a janela viva é a UNIÃO deles
        }
    }
    let incremental: Vec<u8> = t.canvas_rgba.to_vec();
    t.paint.wet_frame_dirty = t.paint.wet_cum_dirty;
    t.apply_watercolor(false);
    let full: Vec<u8> = t.canvas_rgba.to_vec();
    let mut worst = 0i32;
    let mut worst_i = 0usize;
    for (i, (a, b)) in incremental.iter().zip(full.iter()).enumerate() {
        let d = (i32::from(*a) - i32::from(*b)).abs();
        if d > worst {
            worst = d;
            worst_i = i;
        }
    }
    assert!(
        worst <= 1,
        "incremental com água deixou pixel stale: Δ{} no byte {} (px {},{})",
        worst,
        worst_i,
        (worst_i / 4) % size as usize,
        (worst_i / 4) / size as usize
    );
}

/// Paridade incremental×full nos **params do APP** (investigação 2026-07-09, doc 12 take 7): todo
/// repro anterior do harness rodou `Falloff::Constant`/hardness 1/warp 0/gran 0/sem papel/radius
/// 12-14/**sem `on_tick`** — o app real roda o preset Watercolor (feather auto-shape, warp 6,
/// gran 0.3, PaperCold, spacing 0.05), radius 60-100 e o heartbeat por frame (soak/secagem ativos).
/// Este cenário replica o gesto do smoke do Enio (wash assado + traço diagonal VIVO cruzando) na
/// escala do app, com `on_tick(16)` intercalado por Move. O retângulo do preview (sintoma B) é a
/// classe "janela incremental ≠ full": se este teste FALHAR, a reprodução do gap harness×app está
/// fechada na árvore.
fn watercolor_app_params_incremental_vs_full(
    wet_charge: f32,
    edge_spread: f32,
    probe_at: Option<u32>,
) -> (Vec<u8>, Vec<u8>, u32) {
    watercolor_app_params_incremental_vs_full_ablated(wet_charge, edge_spread, probe_at, |_| {})
}

/// A MESMA cena, com uma torneira: `tweak` mexe no `BrushSpec` dos DOIS tracos antes do gesto, para
/// a ablacao por ENTRADA (knob a knob) atribuir o residuo a um termo em vez de a uma teoria.
fn watercolor_app_params_incremental_vs_full_ablated(
    wet_charge: f32,
    edge_spread: f32,
    probe_at: Option<u32>,
    tweak: impl Fn(&mut BrushSpec),
) -> (Vec<u8>, Vec<u8>, u32) {
    use ph2d_painter_brush::{TextureKind, TextureMapping, TextureSettings};
    let size = 512u32;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    // Preset "Watercolor Basic" (watercolor_settings::apply_brush_preset idx 1) + água do smoke.
    // Falloff/hardness ficam no default do app (Smooth/0 + watercolor_shape_auto = feather).
    // Gap restante conhecido: pressão fixa 1.0 (o desktop manda pressão real; dynamics.size_pressure
    // encolhe o primeiro dab) — o dump [wet-diag] do app fecha esse resíduo.
    t.paint.brush = BrushSpec {
        radius_px: 80.0,
        color: [0.24, 0.39, 0.63],
        spacing: 0.05,
        watercolor: true,
        fill: 0.12,
        depth: 1.2,
        edge_gain: 3.0,
        edge_spread,
        warp: 6.0,
        granulation: 0.30,
        pigment: false,
        paper: TextureSettings {
            kind: TextureKind::PaperCold,
            mapping: TextureMapping::Tiled,
            ..TextureSettings::default()
        },
        wet_rewet: 0.3,
        wet_dilution: 0.0, // wash A sem água (liga no traço B)
        wet_charge,
        ..Default::default()
    };
    tweak(&mut t.paint.brush);
    t.paint.brush_by_mode.fill(t.paint.brush);
    // Wash A assado (SEM água), horizontal — com o heartbeat do app entre os Moves.
    assert!(t.on_canvas_pointer(cp([60.0, 250.0], PointerPhase::Down)));
    for i in 1..=40 {
        t.on_canvas_pointer(cp([60.0 + i as f32 * 10.0, 250.0], PointerPhase::Move));
        t.on_tick(16.0);
    }
    t.on_canvas_pointer(cp([460.0, 250.0], PointerPhase::Up));
    t.on_tick(16.0);
    // Traço B com ÁGUA, VIVO, cruzando o wash em diagonal (cruza em ~(250,250)).
    t.paint.brush.wet_dilution = 0.6;
    tweak(&mut t.paint.brush);
    t.paint.brush_by_mode.fill(t.paint.brush);
    assert!(t.on_canvas_pointer(cp([100.0, 60.0], PointerPhase::Down)));
    for i in 1..=40 {
        t.on_canvas_pointer(cp(
            [100.0 + i as f32 * 7.5, 60.0 + i as f32 * 9.5],
            PointerPhase::Move,
        ));
        // O app entrega ~2 eventos de ponteiro por frame (120 Hz pointer / 60 Hz frame): um segundo
        // Move no MESMO batch antes do tick, como o shell faz.
        t.on_canvas_pointer(cp(
            [
                100.0 + (i as f32 + 0.5) * 7.5,
                60.0 + (i as f32 + 0.5) * 9.5,
            ],
            PointerPhase::Move,
        ));
        t.on_tick(16.0);
        // Dwell do gesto real: logo após cruzar o wash o artista PARA a caneta (~2 s) — o soak
        // jorra sob a nib parada e liga o branch global `soaked` dos fields; só a janela do frame
        // é recomposta, o resto da união fica com o field pré-soak (o retângulo).
        if i == 22 {
            for _ in 0..10 {
                t.on_tick(200.0);
            }
        }
        // Probe MID-STROKE: o retângulo do smoke é transiente (frames posteriores repintam por
        // cima); a comparação de estado-final não o captura — esta sim.
        if probe_at == Some(i) {
            let incremental: Vec<u8> = t.canvas_rgba.to_vec();
            t.paint.wet_frame_dirty = t.paint.wet_cum_dirty;
            t.apply_watercolor(false);
            let full: Vec<u8> = t.canvas_rgba.to_vec();
            return (incremental, full, size);
        }
    }
    let incremental: Vec<u8> = t.canvas_rgba.to_vec();
    t.paint.wet_frame_dirty = t.paint.wet_cum_dirty;
    t.apply_watercolor(false);
    let full: Vec<u8> = t.canvas_rgba.to_vec();
    (incremental, full, size)
}

fn worst_byte_delta(incremental: &[u8], full: &[u8]) -> (i32, usize) {
    let mut worst = 0i32;
    let mut worst_i = 0usize;
    for (i, (a, b)) in incremental.iter().zip(full.iter()).enumerate() {
        let d = (i32::from(*a) - i32::from(*b)).abs();
        if d > worst {
            worst = d;
            worst_i = i;
        }
    }
    (worst, worst_i)
}

/// Diag espacial do gap (rode com `--ignored --nocapture`): o diff incremental×full forma um
/// RETÂNGULO coerente (o artefato do smoke) ou speckle disperso (ruído de arredondamento)?
#[test]
#[ignore = "diag exploratório — imprime o mapa espacial do diff incremental×full nos params do app"]
fn watercolor_app_params_diff_spatial_map() {
    for (label, charge, spread, probe) in [
        ("diluted(chg=1,spr=7)", 1.0f32, 7.0f32, None),
        ("diluted(chg=1,spr=30)", 1.0, 30.0, None),
        ("mixer(chg=0.7,spr=30)", 0.7, 30.0, None),
        ("MID diluted(chg=1,spr=7)@23", 1.0, 7.0, Some(23)),
        ("MID diluted(chg=1,spr=30)@23", 1.0, 30.0, Some(23)),
        ("MID mixer(chg=0.7,spr=30)@23", 0.7, 30.0, Some(23)),
    ] {
        let (inc, full, size) = watercolor_app_params_incremental_vs_full(charge, spread, probe);
        let s = size as usize;
        let (worst, _) = worst_byte_delta(&inc, &full);
        let mut count = 0usize;
        let (mut x0, mut y0, mut x1, mut y1) = (usize::MAX, usize::MAX, 0usize, 0usize);
        // Mapa 32×32 (célula = 16px): nº de pixels com Δ≥1 por célula.
        let mut grid = vec![0u32; 32 * 32];
        for i in 0..s * s {
            let d = (0..4)
                .map(|c| (i32::from(inc[i * 4 + c]) - i32::from(full[i * 4 + c])).abs())
                .max()
                .unwrap_or(0);
            if d >= 1 {
                count += 1;
                let (x, y) = (i % s, i / s);
                x0 = x0.min(x);
                y0 = y0.min(y);
                x1 = x1.max(x);
                y1 = y1.max(y);
                grid[(y / 16).min(31) * 32 + (x / 16).min(31)] += 1;
            }
        }
        eprintln!(
            "[spatial-diag {label}] worst=Δ{worst} pixels_diff={count} bbox=({x0},{y0})..({x1},{y1})"
        );
        for gy in 0..32 {
            let row: String = (0..32)
                .map(|gx| match grid[gy * 32 + gx] {
                    0 => '.',
                    1..=9 => 'o',
                    10..=99 => 'O',
                    _ => '#',
                })
                .collect();
            eprintln!("[spatial-diag {label}] {row}");
        }
    }
}

/// ⚠️ **RED conhecido — e o diagnóstico do take 7 foi SUPERSEDIDO pela medição de 2026-08-09.**
///
/// O take 7 deixou o resíduo como *"speckle disperso; ou depende dos params exatos do Enio, ou não
/// está no canvas CPU"*. Está no canvas CPU, e as sondas irmãs deste arquivo o caracterizam:
///
/// - **Não é ruído numérico.** `measure_whether_the_window_itself_moves_the_pixels`: MESMO estado,
///   união contra um retângulo de 64×40 dentro dela ⇒ **Δ0 em 2560 px**. O composite é função pura
///   do estado sobre a região dele.
/// - **Não é a soma-prefixo do `box_blur`.** Ela É dependente da janela (`measure_box_blur_window_
///   invariance`: até **1,98e-4** num sinal fracionário), mas torná-la exata em `f64` deixa os dois
///   gates em Δ2 — hipótese construída, medida e REFUTADA.
/// - **Não é o `settled`.** Duas ablações (`owner != cur_o` sozinho · todo dono settled) deixam 139
///   dos 152 px de pé.
/// - **É RAIO DE INVALIDAÇÃO.** Varrendo o `pad` do [`super::watercolor_render::window`]:
///   `+0 → 152 px · +64 → 38 · +128 → 1 · +2·raio → 0`. O resíduo escala com o pincel (**12 px a
///   r=20 · 152 a r=80 · 361 a r=160**), vive no ARO, e o termo de borda o amplifica 17×
///   (`edge_gain = 0` ⇒ 9 px) — `measure_which_term_carries_the_incremental_residue`.
///
/// ⛔ **O `pad += 2·raio` NÃO é a cura**, e isso é medição, não gosto: a janela é `dirty ⊕ 4·pad` por
/// eixo, então num pincel de 80 px ela vira o CANVAS INTEIRO todo quadro — exatamente o custo que o
/// caminho incremental existe para evitar (e que o `measure_the_area_a_watercolor_frame_walks`
/// vigia). Falta a grandeza NOMEADA de alcance `2·raio`; até ela aparecer, o gate fica `#[ignore]`.
#[test]
#[ignore = "RED conhecido: Δ2 de raio de invalidação no wash incremental (sub-visível; a tolerância \
do gate é ≤1). Vira gate regular quando o residual for corrigido — diagnóstico medido no doc-comment."]
fn watercolor_app_params_incremental_matches_full_diluted() {
    // Sintoma B do smoke (Charge 1 + Dilution > 0): retângulo no preview que some no mouse-up.
    let (inc, full, size) = watercolor_app_params_incremental_vs_full(1.0, 7.0, None);
    let (worst, worst_i) = worst_byte_delta(&inc, &full);
    assert!(
        worst <= 1,
        "params do APP: incremental deixou pixel stale (retângulo do preview): Δ{} no byte {} (px {},{})",
        worst,
        worst_i,
        (worst_i / 4) % size as usize,
        (worst_i / 4) / size as usize
    );
}

/// Irmão do gate acima com o mixer ligado — **o mesmo mecanismo medido**, e o diagnóstico completo
/// (com os números de cada ablação) vive no doc-comment de
/// [`watercolor_app_params_incremental_matches_full_diluted`].
#[test]
#[ignore = "RED conhecido: Δ2 de raio de invalidação no wash incremental, mixer ligado (sub-visível). \
Vira gate regular quando o residual for corrigido — diagnóstico medido no gate irmao."]
fn watercolor_app_params_incremental_matches_full_mixer_on() {
    // Sintoma A do smoke (Charge < 1, mixer ligado): borda dura na junção entre traços.
    let (inc, full, size) = watercolor_app_params_incremental_vs_full(0.7, 7.0, None);
    let (worst, worst_i) = worst_byte_delta(&inc, &full);
    assert!(
        worst <= 1,
        "params do APP c/ mixer: incremental deixou pixel stale na travessia: Δ{} no byte {} (px {},{})",
        worst,
        worst_i,
        (worst_i / 4) % size as usize,
        (worst_i / 4) / size as usize
    );
}

/// Diag (2026-08-09) — **o composite depende da JANELA?** (o teste que separa as duas hipóteses)
///
/// MESMO estado, duas janelas: renderiza a união inteira, depois re-renderiza só um retângulo
/// pequeno DENTRO dela. Se um byte se mexer, o composite não é função apenas do estado — e aí o
/// resíduo dos dois `#[ignore]` não é *staleness* (estado que mudou sem invalidar), é a janela.
#[test]
#[ignore = "diag exploratório: o composite do wash depende da janela em que roda?"]
fn measure_whether_the_window_itself_moves_the_pixels() {
    use ph2d_painter_brush::{TextureKind, TextureMapping, TextureSettings};
    let size = 512u32;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    t.paint.brush = BrushSpec {
        radius_px: 80.0,
        color: [0.24, 0.39, 0.63],
        spacing: 0.05,
        watercolor: true,
        fill: 0.12,
        depth: 1.2,
        edge_gain: 3.0,
        edge_spread: 7.0,
        warp: 6.0,
        granulation: 0.30,
        paper: TextureSettings {
            kind: TextureKind::PaperCold,
            mapping: TextureMapping::Tiled,
            ..TextureSettings::default()
        },
        wet_rewet: 0.3,
        wet_charge: 1.0,
        ..Default::default()
    };
    t.paint.brush_by_mode.fill(t.paint.brush);
    assert!(t.on_canvas_pointer(cp([60.0, 250.0], PointerPhase::Down)));
    for i in 1..=40 {
        t.on_canvas_pointer(cp([60.0 + i as f32 * 10.0, 250.0], PointerPhase::Move));
        t.on_tick(16.0);
    }
    t.on_canvas_pointer(cp([460.0, 250.0], PointerPhase::Up));
    t.on_tick(16.0);

    // (a0) o que o BAKE deixou (pen-up = `apply_watercolor(true)`).
    let baked: Vec<u8> = t.canvas_rgba.to_vec();
    // (a) a união inteira, viva.
    t.paint.wet_frame_dirty = t.paint.wet_cum_dirty;
    t.apply_watercolor(false);
    let full: Vec<u8> = t.canvas_rgba.to_vec();
    let (bw, bn) = {
        let (mut w, mut n) = (0i32, 0usize);
        for (a, b) in baked
            .as_chunks::<4>()
            .0
            .iter()
            .zip(full.as_chunks::<4>().0.iter())
        {
            let d = (0..4)
                .map(|c| (i32::from(a[c]) - i32::from(b[c])).abs())
                .max()
                .unwrap_or(0);
            w = w.max(d);
            n += usize::from(d > 0);
        }
        (w, n)
    };
    eprintln!("[bake x vivo] um traco so, MESMO estado: pior Δ{bw} em {bn} px");
    // (b) o MESMO estado, num retângulo pequeno bem no meio do wash.
    let (rx, ry, rw, rh) = (200u32, 220u32, 64u32, 40u32);
    t.paint.wet_frame_dirty = Some(Region {
        x: rx,
        y: ry,
        w: rw,
        h: rh,
    });
    t.apply_watercolor(false);
    let part: Vec<u8> = t.canvas_rgba.to_vec();

    let (mut worst, mut n) = (0i32, 0usize);
    for y in ry..ry + rh {
        for x in rx..rx + rw {
            let i = ((y * size + x) * 4) as usize;
            let d = (0..4)
                .map(|c| (i32::from(full[i + c]) - i32::from(part[i + c])).abs())
                .max()
                .unwrap_or(0);
            worst = worst.max(d);
            n += usize::from(d > 0);
        }
    }
    eprintln!(
        "[janela] mesmo estado, uniao x retangulo {rw}x{rh}: pior Δ{worst} em {n} de {} px",
        rw * rh
    );
}

/// Diag (2026-08-09) — **QUAL termo carrega o resíduo Δ2?** Ablação por ENTRADA, um knob de cada vez
/// (a receita do doc 28 §5.11: knobs do painel, nunca instrumentação — uma sonda com laço próprio
/// fica cega à porta). Cada linha desliga UM termo do preset do app e mede o pior Δ do
/// incremental×full; o termo cuja ablação leva o Δ a ≤ 1 é o portador.
#[test]
#[ignore = "diag exploratório: ablação por entrada do resíduo Δ2 do wash incremental"]
fn measure_which_term_carries_the_incremental_residue() {
    type Ab = (&'static str, fn(&mut BrushSpec));
    let ablations: &[Ab] = &[
        ("baseline           ", |_| {}),
        ("warp = 0           ", |b| b.warp = 0.0),
        ("granulation = 0    ", |b| b.granulation = 0.0),
        ("paper = None       ", |b| {
            b.paper.kind = ph2d_painter_brush::TextureKind::None;
        }),
        ("edge_gain = 0      ", |b| b.edge_gain = 0.0),
        ("wet_rewet = 0      ", |b| b.wet_rewet = 0.0),
        ("wet_dilution = 0   ", |b| b.wet_dilution = 0.0),
        ("smooth_edges = off ", |b| b.smooth_edges = false),
        ("rewet = 0 + dil = 0", |b| {
            b.wet_rewet = 0.0;
            b.wet_dilution = 0.0;
        }),
        ("raio 20            ", |b| b.radius_px = 20.0),
        ("raio 160           ", |b| b.radius_px = 160.0),
    ];
    for (label, tweak) in ablations {
        let (inc, full, _) =
            watercolor_app_params_incremental_vs_full_ablated(1.0, 7.0, None, tweak);
        let (worst, _) = worst_byte_delta(&inc, &full);
        let n = inc
            .as_chunks::<4>()
            .0
            .iter()
            .zip(full.as_chunks::<4>().0.iter())
            .filter(|(a, b)| a.iter().zip(b.iter()).any(|(x, y)| x != y))
            .count();
        eprintln!("[ablacao] {label} pior Δ{worst} · {n} px");
    }
}

/// Diag (2026-08-09) — **em QUE quadro o incremental começa a divergir, e onde?**
///
/// A ablação *"toda composição viva renderiza a UNIÃO"* leva os dois gates `#[ignore]` acima a Δ0,
/// então o resíduo é STALENESS de janela (o composite É função pura do estado sobre a região dele) —
/// não ruído numérico. Falta o *quando*: este probe varre o quadro do `probe_at` e imprime o primeiro
/// em que o pior Δ passa de 1, com a contagem de pixels divergentes ao lado.
#[test]
#[ignore = "diag exploratório: em que quadro o incremental do wash começa a divergir do full"]
fn measure_when_the_incremental_diverges() {
    for (label, charge, spread) in [("diluted", 1.0f32, 7.0f32), ("mixer", 0.7, 7.0)] {
        let mut first = None;
        for i in 1..=40u32 {
            let (inc, full, size) =
                watercolor_app_params_incremental_vs_full(charge, spread, Some(i));
            let (worst, _) = worst_byte_delta(&inc, &full);
            let s = size as usize;
            let (mut n, mut x0, mut y0, mut x1, mut y1) = (0usize, usize::MAX, usize::MAX, 0, 0);
            for (p, (a, b)) in inc
                .as_chunks::<4>()
                .0
                .iter()
                .zip(full.as_chunks::<4>().0.iter())
                .enumerate()
            {
                if a.iter().zip(b.iter()).any(|(x, y)| x != y) {
                    n += 1;
                    let (x, y) = (p % s, p / s);
                    x0 = x0.min(x);
                    y0 = y0.min(y);
                    x1 = x1.max(x);
                    y1 = y1.max(y);
                }
            }
            if worst > 1 && first.is_none() {
                first = Some(i);
            }
            if i <= 4 || i % 10 == 0 {
                eprintln!(
                    "[diverge {label}] quadro {i:2}: pior Δ{worst} · {n} px · bbox ({x0},{y0})..({x1},{y1})"
                );
            }
        }
        eprintln!("[diverge {label}] PRIMEIRO quadro com Δ>1: {first:?}");
    }
}

/// Diag (2026-08-09) — **o `box_blur` é invariante à JANELA?**
///
/// O composite incremental blura sobre a janela do QUADRO; o recompose "full" blura sobre a janela da
/// UNIÃO. O `pad` garante suporte cheio nas duas, então a MATEMÁTICA concorda — mas o kernel é uma
/// soma-prefixo (`pref[hi+1] - pref[lo]`) acumulada em `f32` desde o `x = 0` da JANELA: duas janelas
/// de origens diferentes chegam ao mesmo pixel com prefixos diferentes, e a subtração devolve o
/// mesmo número MATEMÁTICO com arredondamento diferente. Este probe mede isso no PRIMITIVO, sem a
/// cadeia óptica no meio — é o candidato do resíduo Δ2 dos dois `#[ignore]` acima.
#[test]
#[ignore = "diag exploratório: mede a invariância-à-janela do box_blur (candidato do resíduo Δ2)"]
fn measure_box_blur_window_invariance() {
    let (w, h, r) = (512usize, 64usize, 7usize);
    // Sinal determinístico na ESCALA dos campos de cor do rewet (0..255, presence-premultiplied).
    // ⚠️ FRACIONÁRIO de propósito: o campo real é `cor · smoothstep(...)`, e uma fixture de
    // INTEIROS não contém o fenômeno — a soma de 512 inteiros ≤ 255 cabe exata em `f32` (< 2²⁴),
    // então o prefixo não erra um bit e a sonda mede zero sobre um kernel que pode estar derivando.
    let big: Vec<f32> = (0..w * h)
        .map(|i| {
            let k = (i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
            f32::from((k >> 33) as u16) / 257.0
        })
        .collect();
    let blur_big = super::watercolor_field::box_blur(&big, w, h, r);
    for off in [0usize, 1, 64, 129, 256] {
        let sw = 128usize;
        if off + sw > w {
            continue;
        }
        let sub: Vec<f32> = (0..h)
            .flat_map(|y| big[y * w + off..y * w + off + sw].to_vec())
            .collect();
        let blur_sub = super::watercolor_field::box_blur(&sub, sw, h, r);
        // Só os pixels com suporte CHEIO nas duas janelas (o que o `pad` do composite garante).
        // ⚠️ `seen` é o CONTROLE POSITIVO: a 1ª versão deste probe percorria `r..h - r.min(h/2)`,
        // que com `h = 8, r = 7` é `7..4` — faixa VAZIA. Ele reportou `0.000000000` sem comparar um
        // único pixel, que é exatamente a forma de um verde por vácuo.
        let (mut worst, mut seen) = (0.0f32, 0usize);
        for y in r..h - r {
            for x in r..sw - r {
                let d = (blur_big[y * w + off + x] - blur_sub[y * sw + x]).abs();
                worst = worst.max(d);
                seen += 1;
            }
        }
        assert!(seen > 0, "faixa vazia: o probe nao comparou nada");
        eprintln!("[blur-window off={off:3}] pior |big - sub| = {worst:.9} ({seen} px)");
    }
}
