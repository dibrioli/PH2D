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
