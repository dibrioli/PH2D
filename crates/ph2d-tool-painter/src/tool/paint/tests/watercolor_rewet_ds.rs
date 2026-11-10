//! **B16.4 — o Rewet com a grelha reduzida (`ds > 1`, Spread > 12) não depende da janela.** Os campos
//! do Rewet nascem numa grelha `ds×` mais grossa sobre a janela de leitura; a folga dela (`pad`) é em
//! píxeis da tela e, com `ds > 1`, ficava até ~1,5 célula curta no lado direito/de baixo: o borrão da
//! célula da orla perdia suporte e o quadro dava outros bytes que a recomposição inteira.
//!
//! O arnês: Spread {12, 30, 48} × Rewet {0, 0,3} × dwell {não, sim}, tinta de ANTES da sessão (riscas
//! de cor: sem ela a presença é zero e os campos não mostram nada). Três réguas: os CAMPOS na janela
//! × na tela inteira (ao bit, com controlo: sem a folga, Spread 36/48 diferem até `0,067`); a janela
//! do composite (a união × retângulos pequenos, ao byte) e o incremental × a recomposição (≤ 1).

use super::*;
use crate::tool::paint::watercolor_field::REWET_SEM_FOLGA;

const LADO: u32 = 640;

fn riscas() -> Vec<u8> {
    let mut v = vec![255u8; (LADO * LADO * 4) as usize];
    for y in 0..LADO {
        for x in 0..LADO {
            let i = ((y * LADO + x) * 4) as usize;
            let k = (x / 7 + y / 11) % 3;
            let c = [[200u8, 60, 40], [40, 120, 200], [250, 230, 210]][k as usize];
            v[i..i + 3].copy_from_slice(&c);
        }
    }
    v
}

/// Pinta o traço vivo (uma cruz, dois Moves por quadro, dwell opcional a meio) e devolve a tela
/// incremental. A caneta fica em baixo.
fn traco_vivo(t: &mut PainterTool, spread: f32, rewet: f32, dwell: bool) -> Vec<u8> {
    t.set_source(riscas(), LADO, LADO);
    t.paint.brush = BrushSpec {
        radius_px: 14.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.24, 0.39, 0.63],
        space_attenuation: false,
        watercolor: true,
        edge_gain: 3.0,
        edge_spread: spread,
        granulation: 0.4,
        warp: 6.0,
        fill: 0.35,
        depth: 2.0,
        wet_rewet: rewet,
        ..Default::default()
    };
    t.paint.brush_by_mode.fill(t.paint.brush);
    // Uma cruz que atravessa a tela: a orla direita e a de baixo de qualquer janela pequena no
    // centro caem sobre tinta do traço (sem cobertura na orla o Rewet não pesa e a régua fica cega).
    let mut pontos: Vec<[f32; 2]> = (0..=65).map(|i| [60.0 + i as f32 * 8.0, 300.0]).collect();
    pontos.extend((0..=65).map(|i| [312.0, 60.0 + i as f32 * 8.0]));
    assert!(t.on_canvas_pointer(cp(pontos[0], PointerPhase::Down)));
    for (i, par) in pontos[1..].chunks(2).enumerate() {
        for &p in par {
            t.on_canvas_pointer(cp(p, PointerPhase::Move));
        }
        t.on_tick(16.0);
        if dwell && i == 16 {
            for _ in 0..10 {
                t.on_tick(200.0);
            }
        }
    }
    t.canvas_rgba.to_vec()
}

fn pior(a: &[u8], b: &[u8]) -> (i32, usize, usize) {
    let (mut w, mut wi, mut n) = (0i32, 0usize, 0usize);
    for (i, (x, y)) in a.iter().zip(b).enumerate() {
        let d = (i32::from(*x) - i32::from(*y)).abs();
        n += usize::from(d > 0);
        if d > w {
            (w, wi) = (d, i);
        }
    }
    (w, wi / 4, n)
}

fn casos() -> impl Iterator<Item = (f32, f32, bool)> {
    [12.0f32, 30.0, 48.0].into_iter().flat_map(|s| {
        [0.0f32, 0.3]
            .into_iter()
            .flat_map(move |r| [false, true].into_iter().map(move |d| (s, r, d)))
    })
}

/// Δ máximo e bytes diferentes: o mesmo estado recomposto pela união inteira × por retângulos
/// pequenos (fases `0..8` em x e y, para a orla cair em cada posição da célula `ds`).
fn pela_janela(spread: f32, rewet: f32, dwell: bool, sem_folga: bool) -> (i32, usize) {
    REWET_SEM_FOLGA.set(sem_folga);
    let mut t = PainterTool::default();
    traco_vivo(&mut t, spread, rewet, dwell);
    t.paint.wet_frame_dirty = t.paint.wet_cum_dirty;
    t.apply_watercolor(false);
    let inteira = t.canvas_rgba.to_vec();
    let (mut w, mut n) = (0i32, 0usize);
    for f in 0..8u32 {
        t.paint.wet_frame_dirty = Some(Region {
            x: 296 + f,
            y: 288 + f,
            w: 24,
            h: 20,
        });
        t.apply_watercolor(false);
        let (d, _, k) = pior(&t.canvas_rgba, &inteira);
        (w, n) = (w.max(d), n.max(k));
    }
    REWET_SEM_FOLGA.set(false);
    (w, n)
}

/// Δ máximo, píxel e bytes: o traço vivo (quadro a quadro) × a recomposição inteira do mesmo estado.
fn pelo_traco(spread: f32, rewet: f32, dwell: bool, sem_folga: bool) -> (i32, usize, usize) {
    REWET_SEM_FOLGA.set(sem_folga);
    let mut t = PainterTool::default();
    let inc = traco_vivo(&mut t, spread, rewet, dwell);
    t.paint.wet_frame_dirty = t.paint.wet_cum_dirty;
    t.apply_watercolor(false);
    REWET_SEM_FOLGA.set(false);
    pior(&inc, &t.canvas_rgba)
}

/// A JANELA não move píxeis (ao byte). Medido: o de antes também dá `0` nesta cena (o ponto
/// deslocado raramente cai na orla curta); a régua que VÊ a folga é a directa,
/// [`os_campos_do_rewet_na_janela_sao_os_da_tela_inteira`].
#[test]
fn o_rewet_de_spread_largo_nao_depende_da_janela() {
    let (mut falhas, mut antes) = (Vec::new(), Vec::new());
    for (spread, rewet, dwell) in casos() {
        let (w, n) = pela_janela(spread, rewet, dwell, false);
        let (wa, na) = pela_janela(spread, rewet, dwell, true);
        eprintln!("Spread {spread} Rewet {rewet} dwell {dwell}: antes Δ{wa}/{na} B → Δ{w}/{n} B");
        if w > 0 {
            falhas.push(format!(
                "Spread {spread} Rewet {rewet} dwell {dwell}: Δ{w} em {n} B"
            ));
        }
        if wa > 0 {
            antes.push((spread, rewet));
        }
    }
    assert!(
        falhas.is_empty(),
        "a janela move píxeis:\n{}",
        falhas.join("\n")
    );
    // Com ds = 1 a folga não existe: o de antes tem de ser o mesmo (Basic ao byte).
    assert!(antes.iter().all(|&(s, _)| s > 12.0), "ds = 1 mudou");
}

/// O traço vivo (quadro a quadro) dá o byte da recomposição inteira do mesmo estado (≤ 1, a
/// tolerância do gate irmão de Spread 12).
#[test]
fn o_rewet_de_spread_largo_incremental_e_a_recomposicao_inteira() {
    let mut falhas = Vec::new();
    for (spread, rewet, dwell) in casos() {
        let (w, px, n) = pelo_traco(spread, rewet, dwell, false);
        let (wa, _, na) = pelo_traco(spread, rewet, dwell, true);
        eprintln!("Spread {spread} Rewet {rewet} dwell {dwell}: antes Δ{wa}/{na} B → Δ{w}/{n} B");
        if w > 1 {
            falhas.push(format!(
                "Spread {spread} Rewet {rewet} dwell {dwell}: Δ{w} em {n} B (px {},{})",
                px % LADO as usize,
                px / LADO as usize
            ));
        }
    }
    assert!(
        falhas.is_empty(),
        "incremental ≠ inteira:\n{}",
        falhas.join("\n")
    );
}

/// A régua DIRECTA da folga: os campos do Rewet construídos sobre uma janela de leitura pequena,
/// amostrados em todo o ponto que o composite pode pedir (`[rx0 + reach + 2, rx1 − reach − 2)`, de
/// ¼ em ¼ px, onde o `pad` deixa o ponto deslocado), dão o `f32` dos campos da tela inteira — por
/// `ds` de 1 a 4, com dwell e água. CONTROLO: sem a folga de células algum `ds > 1` difere.
#[test]
fn os_campos_do_rewet_na_janela_sao_os_da_tela_inteira() {
    use crate::tool::paint::watercolor_field::{RewetFields, build_rewet_fields};
    let (fw, fh) = (LADO as usize, LADO as usize);
    let base = riscas();
    let ground = vec![255u8; fw * fh * 4];
    let byte = |i: usize, k: usize| ((i * k) % 251) as u8;
    let soak: Vec<u8> = (0..fw * fh).map(|i| byte(i, 7)).collect();
    let water: Vec<u8> = (0..fw * fh).map(|i| byte(i, 13)).collect();
    fn campos(f: &RewetFields) -> Vec<&[f32]> {
        let mut v: Vec<&[f32]> = vec![&f.pres, &f.soak_raw, &f.soak_halo, &f.water_halo];
        v.extend(f.near.iter().map(|c| &c[..]));
        v.extend(f.far.iter().flatten().map(|c| &c[..]));
        v
    }
    let mede = |sem_folga: bool| -> Vec<String> {
        REWET_SEM_FOLGA.set(sem_folga);
        let mut erros = Vec::new();
        for spread in [12usize, 24, 36, 48] {
            let reach = 2 * spread;
            let b = |r| {
                build_rewet_fields(
                    &base,
                    &ground,
                    &soak,
                    true,
                    &water,
                    true,
                    (fw, fh),
                    r,
                    spread,
                )
            };
            let toda = b((0, 0, fw, fh));
            for f in 0..4usize {
                let (rx0, ry0) = (150 + f, 160 + 2 * f);
                let (rx1, ry1) = (rx0 + 2 * reach + 4 + 40 + f, ry0 + 2 * reach + 4 + 36);
                let jan = b((rx0, ry0, rx1, ry1));
                let (mut pior, mut n) = (0.0f32, 0usize);
                let lado = |a: usize, z: usize| {
                    let (lo, hi) = (a + reach + 2, z - reach - 2);
                    (0..(hi - lo) * 4).map(move |k| lo as f32 + k as f32 * 0.25)
                };
                for gy in lado(ry0, ry1).step_by(7) {
                    for gx in lado(rx0, rx1) {
                        for (cj, ct) in campos(&jan).into_iter().zip(campos(&toda)) {
                            let a = jan.samp(cj, rx0, ry0, gx - rx0 as f32, gy - ry0 as f32);
                            let z = toda.samp(ct, 0, 0, gx, gy);
                            if a.to_bits() != z.to_bits() {
                                pior = pior.max((a - z).abs());
                                n += 1;
                            }
                        }
                    }
                }
                if n > 0 {
                    erros.push(format!(
                        "Spread {spread} fase {f}: {n} amostras, pior {pior:e}"
                    ));
                }
            }
        }
        REWET_SEM_FOLGA.set(false);
        erros
    };
    let antes = mede(true);
    let depois = mede(false);
    eprintln!("antes: {antes:#?}\ndepois: {depois:#?}");
    assert!(
        depois.is_empty(),
        "a janela muda os campos:\n{}",
        depois.join("\n")
    );
    assert!(
        antes.iter().all(|e| !e.starts_with("Spread 12 ")),
        "CONTROLO: ds = 1 não leva folga, o de antes é o mesmo"
    );
    assert!(
        !antes.is_empty(),
        "CONTROLO: sem a folga, ds > 1 devia diferir (a régua é cega)"
    );
}

/// **B16.5 — a grelha guardada é a refeita.** Uma sessão de janelas que andam, com o soak e a água
/// a mudar dentro do sujo de cada quadro: cada amostra que o composite pode pedir dá o `f32` dos
/// campos refeitos na tela inteira (ao bit), por `ds` de 1 a 4. CONTROLO: esquecer o sujo de um
/// quadro (`changed = None`) dá campos velhos — a régua vê o que a cache tem de refazer.
#[test]
fn o_campo_do_rewet_guardado_e_o_refeito() {
    use crate::tool::paint::watercolor_field::{
        RewetEntradas, RewetFields, build_rewet_fields, build_rewet_fields_cached,
    };
    let (fw, fh) = (LADO as usize, LADO as usize);
    let base = riscas();
    let ground = vec![255u8; fw * fh * 4];
    fn campos(f: &RewetFields) -> Vec<&[f32]> {
        let mut v: Vec<&[f32]> = vec![&f.pres, &f.soak_raw, &f.soak_halo, &f.water_halo];
        v.extend(f.near.iter().map(|c| &c[..]));
        v.extend(f.far.iter().flatten().map(|c| &c[..]));
        v
    }
    let sessao = |esquece: bool| -> Vec<String> {
        let mut erros = Vec::new();
        for spread in [6usize, 12, 24, 36, 48] {
            let reach = 2 * spread;
            let mut soak = vec![0u8; fw * fh];
            let mut water = vec![0u8; fw * fh];
            let mut cache = None;
            for q in 0..6usize {
                // O dab do quadro: um quadrado que anda na diagonal, escrito no soak e na água.
                let (dx, dy) = (200 + q * 23, 190 + q * 17);
                for y in dy..dy + 18 {
                    for x in dx..dx + 18 {
                        soak[y * fw + x] = soak[y * fw + x].saturating_add(40 + q as u8);
                        water[y * fw + x] = water[y * fw + x].saturating_add(55);
                    }
                }
                let d = Region {
                    x: dx as u32,
                    y: dy as u32,
                    w: 18,
                    h: 18,
                };
                let pad = reach + 6;
                let (rx0, ry0) = (dx.saturating_sub(2 * pad), dy.saturating_sub(2 * pad));
                let (rx1, ry1) = ((dx + 18 + 2 * pad).min(fw), (dy + 18 + 2 * pad).min(fh));
                let e = RewetEntradas {
                    base: &base,
                    ground: &ground,
                    wet_soak: &soak,
                    soaked: true,
                    stroke_water: &water,
                    watered: true,
                    fw,
                    fh,
                };
                let changed = (!(esquece && q == 3)).then_some(d);
                let (jan, c) = build_rewet_fields_cached(
                    e,
                    (rx0, ry0, rx1, ry1),
                    spread,
                    changed,
                    cache.take(),
                );
                cache = Some(c);
                let toda = build_rewet_fields(
                    &base,
                    &ground,
                    &soak,
                    true,
                    &water,
                    true,
                    (fw, fh),
                    (0, 0, fw, fh),
                    spread,
                );
                let (mut pior, mut n) = (0.0f32, 0usize);
                let lado = |a: usize, z: usize| {
                    let (lo, hi) = (a + pad, z.saturating_sub(pad).max(a + pad));
                    (0..(hi - lo) * 2).map(move |k| lo as f32 + k as f32 * 0.5)
                };
                for gy in lado(ry0, ry1).step_by(3) {
                    for gx in lado(rx0, rx1).step_by(2) {
                        for (cj, ct) in campos(&jan).into_iter().zip(campos(&toda)) {
                            let a = jan.samp(cj, rx0, ry0, gx - rx0 as f32, gy - ry0 as f32);
                            let z = toda.samp(ct, 0, 0, gx, gy);
                            if a.to_bits() != z.to_bits() {
                                pior = pior.max((a - z).abs());
                                n += 1;
                            }
                        }
                    }
                }
                if n > 0 {
                    erros.push(format!(
                        "Spread {spread} quadro {q}: {n} amostras, pior {pior:e}"
                    ));
                }
            }
        }
        erros
    };
    let certo = sessao(false);
    let velho = sessao(true);
    eprintln!("guardado × refeito: {certo:#?}\nCONTROLO (sujo esquecido): {velho:#?}");
    assert!(
        certo.is_empty(),
        "a grelha guardada envelheceu:\n{}",
        certo.join("\n")
    );
    assert!(
        !velho.is_empty(),
        "CONTROLO: esquecer o sujo devia dar campos velhos (régua cega)"
    );
}

/// **B16.5 — o composite com a cache dá o byte do composite sem ela** (o `build_rewet_fields` de
/// sempre): o traço vivo inteiro, Spread {6, 12, 30, 48} × Rewet {0; 0,3} × dwell. Spread ≤ 12 é a
/// grelha exacta (`ds = 1`), a do Watercolor Basic.
#[test]
fn o_rewet_guardado_da_o_byte_do_refeito() {
    use crate::tool::paint::watercolor_field::REWET_SEM_CACHE;
    let mut falhas = Vec::new();
    for (spread, rewet, dwell) in casos().chain([(6.0, 0.0, false), (6.0, 0.3, true)]) {
        let com = traco_vivo(&mut PainterTool::default(), spread, rewet, dwell);
        REWET_SEM_CACHE.set(true);
        let sem = traco_vivo(&mut PainterTool::default(), spread, rewet, dwell);
        REWET_SEM_CACHE.set(false);
        let (w, _, n) = pior(&com, &sem);
        eprintln!("Spread {spread} Rewet {rewet} dwell {dwell}: Δ{w} em {n} B");
        if n > 0 {
            falhas.push(format!(
                "Spread {spread} Rewet {rewet} dwell {dwell}: Δ{w} em {n} B"
            ));
        }
    }
    assert!(
        falhas.is_empty(),
        "a cache muda bytes:\n{}",
        falhas.join("\n")
    );
}

/// **B16.5 — a grelha guardada só aloca os ladrilhos que a janela visitou** (planos esparsos): uma
/// janela de 200² num canto de 2048², dwell + água (os 12 planos), aloca ≤ 1/20 da grelha densa.
#[test]
fn o_rewet_guardado_so_aloca_os_ladrilhos_visitados() {
    use crate::tool::paint::watercolor_field::{RewetEntradas, build_rewet_fields_cached};
    let n = 2048usize;
    let (base, ground) = (vec![200u8; n * n * 4], vec![255u8; n * n * 4]);
    let (soak, water) = (vec![30u8; n * n], vec![50u8; n * n]);
    let e = RewetEntradas {
        base: &base,
        ground: &ground,
        wet_soak: &soak,
        soaked: true,
        stroke_water: &water,
        watered: true,
        fw: n,
        fh: n,
    };
    for spread in [12usize, 48] {
        let (_, c) = build_rewet_fields_cached(e, (0, 0, 200, 200), spread, None, None);
        let (vivos, densos) = c.bytes();
        eprintln!("Spread {spread}: {vivos} B alocados × {densos} B densos");
        assert!(
            vivos * 20 <= densos,
            "Spread {spread}: {vivos} B de {densos} B (grelha densa?)"
        );
    }
}

/// **B16.5 — a sonda: o quadro da aguada com Rewet 0,4 a 4096², com a cache × sem ela** (o MESMO
/// processo, os dois tools intercalados quadro a quadro, ABBA; a aguada é CPU, nenhuma placa).
#[test]
#[ignore = "sonda: --profile smoke -- --ignored"]
fn mede_o_rewet_guardado_a_4096() {
    use crate::tool::paint::media::PaintMedia;
    use crate::tool::paint::watercolor_field::REWET_SEM_CACHE;
    use std::time::Instant;
    const N: u32 = 4096;
    let novo = || {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (N * N * 4) as usize], N, N);
        t.paint.brush = BrushSpec {
            radius_px: 100.0,
            hardness: 0.5,
            falloff: Falloff::Smooth,
            strength: 1.0,
            color: [0.8, 0.2, 0.1],
            space_attenuation: false,
            ..Default::default()
        };
        t.paint.brush_by_mode.fill(t.paint.brush);
        t.set_paint_media(PaintMedia::Watercolor);
        t.set_brush_wet_rewet(0.4);
        t
    };
    let (mut a, mut b) = (novo(), novo());
    let br = &a.paint.brush;
    eprintln!(
        "pincel: rewet {} spread {} r {}",
        br.wet_rewet, br.edge_spread, br.radius_px
    );
    let y = (N / 2) as f32;
    for t in [&mut a, &mut b] {
        assert!(t.on_canvas_pointer(cp([300.0, y], PointerPhase::Down)));
    }
    let quadro = |t: &mut PainterTool, sem: bool, x: f32| {
        REWET_SEM_CACHE.set(sem);
        let t0 = Instant::now();
        t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
        t.on_canvas_pointer(cp([x + 20.0, y], PointerPhase::Move));
        t.on_tick(16.0);
        let ms = t0.elapsed().as_secs_f64() * 1e3;
        REWET_SEM_CACHE.set(false);
        ms
    };
    let (mut com, mut sem) = (Vec::new(), Vec::new());
    for k in 0..48usize {
        let x = 320.0 + 40.0 * k as f32;
        let (ca, sb) = if k % 2 == 0 {
            (quadro(&mut a, false, x), quadro(&mut b, true, x))
        } else {
            let sb = quadro(&mut b, true, x);
            (quadro(&mut a, false, x), sb)
        };
        if k >= 6 {
            com.push(ca);
            sem.push(sb);
        }
    }
    let resumo = |v: &mut Vec<f64>| {
        v.sort_by(f64::total_cmp);
        (v[0], v[v.len() / 2], v.iter().sum::<f64>())
    };
    let (c, s) = (resumo(&mut com), resumo(&mut sem));
    let mem = a.paint.wet_rewet_cache.as_ref().map(|c| c.bytes());
    eprintln!(
        "quadro 4096² Rewet 0,4 (42 quadros): sem cache mín {:.2} med {:.2} soma {:.0} ms × com cache \
         mín {:.2} med {:.2} soma {:.0} ms · cache (alocado, denso) = {mem:?} B",
        s.0, s.1, s.2, c.0, c.1, c.2
    );
}
