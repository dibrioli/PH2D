//! ⏱️⭐⭐⭐⭐ **ONDE OS PASSOS DA MARCHA ACONTECEM** — a pergunta que a recusa da grade de longe deixou.
//!
//! A grade de longe (`device_probes_w9_longe.rs`) saltava o vazio com um limite provado e não
//! comprou nada sobre o recorte da caixa: *o custo mora perto da superfície*. Esta sonda mede isso
//! em vez de o supor. Ela refaz a marcha raio a raio, **com a lei do produto** (o recorte pela
//! `march_clip`, o passo `safe_march_step`, o orçamento `MAX_STEPS · shrink / passo`, o limiar
//! `Sharpness::for_frame`), e parte cada passo por DUAS perguntas:
//!
//! - **a que distância da superfície o raio estava**, em PIXELS de ecrã (`d < 1`, `< 4`, `< 16`,
//!   `≥ 16`) — um passo a menos de um pixel é o raio a rastejar;
//! - **o raio acabou por ACERTAR ou por FALHAR** — um raio que roça a silhueta e segue em frente
//!   paga os passos minúsculos sem desenhar nada.
//!
//! ⚠️ Mais as avaliações da NORMAL (quatro por acerto, o estêncil do dispositivo), porque essas
//! também são avaliações da árvore.
//!
//! ```text
//! bash scripts/ph2d-run.sh cargo test -p ph2d-app-field3d --lib --release -- --ignored --exact \
//!   preview::device_tests::sondas_w9::perto::diag_onde_os_passos_acontecem --nocapture
//! ```

/// Entrada e saída do raio na caixa `[lo, hi]` — o `slab` da CPU, reescrito aqui por ser privado.
fn laje(o: [f32; 3], d: [f32; 3], lo: [f32; 3], hi: [f32; 3]) -> Option<(f32, f32)> {
    let (mut a, mut b) = (f32::NEG_INFINITY, f32::INFINITY);
    for e in 0..3 {
        if d[e].abs() < 1e-30 {
            if o[e] < lo[e] || o[e] > hi[e] {
                return None;
            }
        } else {
            let t1 = (lo[e] - o[e]) / d[e];
            let t2 = (hi[e] - o[e]) / d[e];
            a = a.max(t1.min(t2));
            b = b.min(t1.max(t2));
        }
    }
    (a <= b && b > 0.0).then_some((a.max(0.0), b))
}

/// ⏱️⭐⭐⭐⭐ **Sonda: onde os passos da marcha acontecem, e de quem são.**
#[test]
#[ignore = "sonda de CPU, cara"]
fn diag_onde_os_passos_acontecem() {
    use ph2d_field_render::{MAX_STEPS, Orbit, Screen, Sharpness, T_MAX};
    const W: u32 = 1920;
    const H: u32 = 1080;
    const FAIXAS: [f32; 3] = [1.0, 4.0, 16.0];
    let cam = Orbit::default();
    let screen = Screen::new(W, H, cam.half_extent);
    let sharp = Sharpness::for_frame(cam.half_extent, W.min(H) as usize);
    let pixel = 2.0 * cam.half_extent / W.min(H) as f32;
    println!(
        "\n  {W}×{H} · pixel {pixel:.2e} · acerto < {:.2e}\n  cena · raios na caixa · acertos · \
         passos/pixel · % dos passos em FALHAS · passos por faixa de distância (px) <1 | <4 | <16 \
         | ≥16 · % na NORMAL · esgotados",
        sharp.hit
    );
    for cena in [5u32, 28, 1, 11, 26, 27, 29, 30] {
        let doc = crate::smoke::scene(cena);
        let reg = crate::smoke::sampled_registry();
        let Some(bola) = ph2d_field_eval::bounds::bounding_ball(&doc, &reg) else {
            continue;
        };
        let (lo, hi) = ph2d_field_eval::bounds_clip::march_clip(bola);
        let passo = ph2d_field_eval::safe_march_step(&doc);
        let shrink = ph2d_field_eval::field_shrink(&doc, &reg);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let budget =
            ((MAX_STEPS as f32) * shrink.max(1.0) / passo.clamp(f32::EPSILON, 1.0)).ceil() as usize;

        // Os raios que entram na caixa — os outros não custam passo nenhum.
        let (mut o, mut d, mut t, mut fim) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        let mut pix = Vec::new();
        for y in 0..H {
            for x in 0..W {
                #[allow(clippy::cast_precision_loss)]
                let (u, v) = screen.plane_at(x as f32 + 0.5, y as f32 + 0.5);
                let (ro, rd) = cam.ray_at_plane(u, v);
                if let Some((a, b)) = laje(ro, rd, lo, hi) {
                    o.push(ro);
                    d.push(rd);
                    t.push(a);
                    fim.push(b.min(T_MAX));
                    pix.push((y * W + x) as usize);
                }
            }
        }
        let n = o.len();
        // Por raio: passos em cada faixa (a última é `≥ 16 px`), e o destino.
        let mut faixa = vec![[0u32; 4]; n];
        let mut acertou = vec![false; n];
        let mut vivos: Vec<usize> = (0..n).filter(|&i| t[i] < fim[i]).collect();
        let mut h = ph2d_field_eval::hybrid::Hybrid::new(&doc, &reg);
        let (mut xs, mut ys, mut zs) = (Vec::new(), Vec::new(), Vec::new());
        for _ in 0..budget {
            if vivos.is_empty() {
                break;
            }
            xs.clear();
            ys.clear();
            zs.clear();
            for &i in &vivos {
                xs.push(o[i][0] + d[i][0] * t[i]);
                ys.push(o[i][1] + d[i][1] * t[i]);
                zs.push(o[i][2] + d[i][2] * t[i]);
            }
            let f = h.eval(&xs, &ys, &zs).expect("a árvore").to_vec();
            let mut prox = Vec::with_capacity(vivos.len());
            for (j, &i) in vivos.iter().enumerate() {
                let em_px = f[j] / pixel;
                let k = FAIXAS.iter().position(|b| em_px < *b).unwrap_or(3);
                faixa[i][k] += 1;
                if f[j] < sharp.hit {
                    acertou[i] = true;
                    continue;
                }
                t[i] += f[j] * passo;
                if t[i] < fim[i] {
                    prox.push(i);
                }
            }
            vivos = prox;
        }
        let esgotados = vivos.len();
        let mut passos = vec![0u32; (W * H) as usize];
        let mut falhou = vec![false; (W * H) as usize];
        for i in 0..n {
            passos[pix[i]] = faixa[i].iter().sum::<u32>() + if acertou[i] { 4 } else { 0 };
            falhou[pix[i]] = !acertou[i];
        }
        let dv = divergencia(&passos, &falhou, W as usize, H as usize);
        let acertos = acertou.iter().filter(|a| **a).count();
        let mut por_faixa = [0u64; 4];
        let (mut em_falha, mut total) = (0u64, 0u64);
        for i in 0..n {
            let s: u32 = faixa[i].iter().sum();
            total += u64::from(s);
            if !acertou[i] {
                em_falha += u64::from(s);
            }
            for k in 0..4 {
                por_faixa[k] += u64::from(faixa[i][k]);
            }
        }
        // ⚠️ A normal do dispositivo são QUATRO avaliações por acerto (o estêncil tetraédrico).
        let normal = 4 * acertos as u64;
        #[allow(clippy::cast_precision_loss)]
        let pct = |a: u64, b: u64| 100.0 * a as f64 / b.max(1) as f64;
        #[allow(clippy::cast_precision_loss)]
        let por_pixel = total as f64 / f64::from(W * H);
        println!(
            "  {cena:4} · {n:>9} · {acertos:>8} · {por_pixel:>6.1} · {:>5.1} % · {:>5.1} | {:>5.1} | \
             {:>5.1} | {:>5.1} % · {:>4.1} % · {esgotados}\n         por raio p50 {} · p99 {} · max {} · \
             EFICIÊNCIA dos blocos de 32 {:.1} % · custo dos blocos cujo lento FALHOU {:.1} %",
            pct(em_falha, total),
            pct(por_faixa[0], total),
            pct(por_faixa[1], total),
            pct(por_faixa[2], total),
            pct(por_faixa[3], total),
            pct(normal, total + normal),
            dv.p50,
            dv.p99,
            dv.max,
            100.0 * dv.eficiencia,
            100.0 * dv.culpa_das_falhas,
        );
    }
    println!();
}

/// ⭐⭐⭐ **A DIVERGÊNCIA: a placa paga o raio MAIS LENTO de cada bloco de 32.**
///
/// O shader corre em grupos de `8×8`, e a placa desta máquina avança `32` pixels juntos (`8×4`). ⇒ o
/// custo de um bloco é `32 × max(passos)`, e a eficiência é o trabalho útil sobre esse custo.
/// `passos` aqui conta a árvore toda: a marcha mais as quatro da normal num acerto.
pub(super) struct Divergencia {
    pub eficiencia: f64,
    pub p50: u32,
    pub p99: u32,
    pub max: u32,
    /// Fracção do custo dos blocos cujo raio mais lento FALHOU.
    pub culpa_das_falhas: f64,
}

pub(super) fn divergencia(passos: &[u32], falhou: &[bool], w: usize, h: usize) -> Divergencia {
    let mut v: Vec<u32> = passos.iter().copied().filter(|p| *p > 0).collect();
    v.sort_unstable();
    let q = |f: f64| {
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss
        )]
        v.get(((v.len().max(1) - 1) as f64 * f) as usize)
            .copied()
            .unwrap_or(0)
    };
    let (mut util, mut custo, mut das_falhas) = (0u64, 0u64, 0u64);
    for by in (0..h).step_by(4) {
        for bx in (0..w).step_by(8) {
            let (mut m, mut quem_falhou) = (0u32, false);
            for y in by..(by + 4).min(h) {
                for x in bx..(bx + 8).min(w) {
                    let i = y * w + x;
                    util += u64::from(passos[i]);
                    if passos[i] > m {
                        m = passos[i];
                        quem_falhou = falhou[i];
                    }
                }
            }
            custo += 32 * u64::from(m);
            if quem_falhou {
                das_falhas += 32 * u64::from(m);
            }
        }
    }
    #[allow(clippy::cast_precision_loss)]
    Divergencia {
        eficiencia: util as f64 / custo.max(1) as f64,
        p50: q(0.5),
        p99: q(0.99),
        max: v.last().copied().unwrap_or(0),
        culpa_das_falhas: das_falhas as f64 / custo.max(1) as f64,
    }
}

/// Um pedaço: `(quadrado x, quadrado y, fatia)`.
type Pedaco = (usize, usize, usize);
/// A caixa `(lo, hi)` do que se avaliou num pedaço.
type Caixa = ([f32; 3], [f32; 3]);

/// ⏱️⭐⭐⭐⭐ **Sonda: quanto a PODA POR REGIÃO compraria** (Keeter 2020) — a alavanca que o
/// `docs/Render3d/05` §43 nomeia e diz que **ninguém mediu**.
///
/// A marcha é a mesma da irmã (a lei do produto); cada avaliação cai num pedaço
/// `(quadrado de T×T píxeis, fatia de profundidade)`. Por pedaço, a caixa exacta do que foi
/// avaliado (o segmento de cada raio do quadrado dentro da fatia) vai à
/// [`ph2d_field_eval::poda::Podador`], e as instruções que sobram pesam-se pelas avaliações que o
/// pedaço fez. ⚠️ Uma caixa que o intervalo prova VAZIA conta `0` instruções — os raios
/// atravessam-na sem avaliar — e sai numa coluna à parte, para não se confundir com poda.
#[test]
#[ignore = "sonda de CPU, cara"]
fn diag_quanto_a_poda_por_regiao_compraria() {
    use ph2d_field_render::{MAX_STEPS, Orbit, Screen, Sharpness, T_MAX};
    const W: usize = 1920;
    const H: usize = 1080;
    const CONFIGS: [(usize, usize); 6] = [(64, 1), (32, 1), (16, 1), (32, 4), (32, 16), (16, 16)];
    let cam = Orbit::default();
    #[allow(clippy::cast_possible_truncation)]
    let screen = Screen::new(W as u32, H as u32, cam.half_extent);
    let sharp = Sharpness::for_frame(cam.half_extent, W.min(H));
    println!(
        "\n  {W}×{H} · razão = instruções médias por avaliação ÷ peça inteira\n  cena · \
         instruções da peça · [quadrado px × fatias: razão · % das avaliações em caixa VAZIA · \
         pedaços · ms da poda]"
    );
    for cena in [5u32, 28, 1, 11, 26, 27, 29, 30] {
        let doc = crate::smoke::scene(cena);
        let reg = crate::smoke::sampled_registry();
        let Some(bola) = ph2d_field_eval::bounds::bounding_ball(&doc, &reg) else {
            continue;
        };
        let (lo, hi) = ph2d_field_eval::bounds_clip::march_clip(bola);
        let passo = ph2d_field_eval::safe_march_step(&doc);
        let shrink = ph2d_field_eval::field_shrink(&doc, &reg);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let budget =
            ((MAX_STEPS as f32) * shrink.max(1.0) / passo.clamp(f32::EPSILON, 1.0)).ceil() as usize;
        let (mut o, mut d, mut t, mut ent, mut sai, mut pix) = (
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );
        for y in 0..H {
            for x in 0..W {
                #[allow(clippy::cast_precision_loss)]
                let (u, v) = screen.plane_at(x as f32 + 0.5, y as f32 + 0.5);
                let (ro, rd) = cam.ray_at_plane(u, v);
                if let Some((a, b)) = laje(ro, rd, lo, hi) {
                    o.push(ro);
                    d.push(rd);
                    t.push(a);
                    ent.push(a);
                    sai.push(b.min(T_MAX));
                    pix.push((x, y));
                }
            }
        }
        let n = o.len();
        let t_min = ent.iter().copied().fold(f32::INFINITY, f32::min);
        let t_max = sai.iter().copied().fold(0.0f32, f32::max);
        let fatia_de = |tt: f32, s: usize| {
            #[allow(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                clippy::cast_precision_loss
            )]
            (((tt - t_min) / (t_max - t_min).max(1e-9) * s as f32) as usize).min(s - 1)
        };
        // Avaliações por pedaço, uma tabela por configuração.
        let mut conta: Vec<std::collections::BTreeMap<Pedaco, u64>> =
            vec![std::collections::BTreeMap::new(); CONFIGS.len()];
        let mut vivos: Vec<usize> = (0..n).filter(|&i| t[i] < sai[i]).collect();
        let mut h = ph2d_field_eval::hybrid::Hybrid::new(&doc, &reg);
        let (mut xs, mut ys, mut zs) = (Vec::new(), Vec::new(), Vec::new());
        let mut total = 0u64;
        for _ in 0..budget {
            if vivos.is_empty() {
                break;
            }
            xs.clear();
            ys.clear();
            zs.clear();
            for &i in &vivos {
                xs.push(o[i][0] + d[i][0] * t[i]);
                ys.push(o[i][1] + d[i][1] * t[i]);
                zs.push(o[i][2] + d[i][2] * t[i]);
            }
            let f = h.eval(&xs, &ys, &zs).expect("a árvore").to_vec();
            let mut prox = Vec::with_capacity(vivos.len());
            for (j, &i) in vivos.iter().enumerate() {
                total += 1;
                let (px, py) = pix[i];
                for (c, &(lado, fatias)) in CONFIGS.iter().enumerate() {
                    *conta[c]
                        .entry((px / lado, py / lado, fatia_de(t[i], fatias)))
                        .or_default() += 1;
                }
                if f[j] < sharp.hit {
                    continue;
                }
                t[i] += f[j] * passo;
                if t[i] < sai[i] {
                    prox.push(i);
                }
            }
            vivos = prox;
        }
        let arvore = ph2d_field_eval::compile(&doc);
        let mut podador = ph2d_field_eval::poda::Podador::new(&arvore);
        let inteira = podador.tamanho();
        let mut linha = format!("  {cena:4} · {inteira:>5}");
        for (c, &(lado, fatias)) in CONFIGS.iter().enumerate() {
            // A caixa exacta de cada pedaço: os segmentos dos raios do quadrado dentro da fatia.
            let mut caixas: std::collections::BTreeMap<Pedaco, Caixa> =
                std::collections::BTreeMap::new();
            for i in 0..n {
                let (px, py) = pix[i];
                for s in 0..fatias {
                    #[allow(clippy::cast_precision_loss)]
                    let (fa, fb) = (
                        t_min + (t_max - t_min) * s as f32 / fatias as f32,
                        t_min + (t_max - t_min) * (s + 1) as f32 / fatias as f32,
                    );
                    let (a, b) = (fa.max(ent[i]), fb.min(sai[i]));
                    if a > b {
                        continue;
                    }
                    let chave = (px / lado, py / lado, s);
                    if !conta[c].contains_key(&chave) {
                        continue;
                    }
                    let e = caixas
                        .entry(chave)
                        .or_insert(([f32::INFINITY; 3], [f32::NEG_INFINITY; 3]));
                    for tt in [a, b] {
                        for k in 0..3 {
                            let p = o[i][k] + d[i][k] * tt;
                            e.0[k] = e.0[k].min(p);
                            e.1[k] = e.1[k].max(p);
                        }
                    }
                }
            }
            let t0 = std::time::Instant::now();
            let (mut pesado, mut em_vazia) = (0u128, 0u64);
            for (chave, &cnt) in &conta[c] {
                let Some(&(clo, chi)) = caixas.get(chave) else {
                    pesado += u128::from(cnt) * inteira as u128;
                    continue;
                };
                match podador.na_caixa(clo, chi) {
                    ph2d_field_eval::poda::Poda::Vazia { .. } => em_vazia += cnt,
                    ph2d_field_eval::poda::Poda::Fita { instrucoes } => {
                        pesado += u128::from(cnt) * instrucoes as u128;
                    }
                }
            }
            let ms = t0.elapsed().as_secs_f64() * 1e3;
            #[allow(clippy::cast_precision_loss)]
            let razao = pesado as f64 / (total.max(1) as f64 * inteira as f64);
            #[allow(clippy::cast_precision_loss)]
            let vazia = 100.0 * em_vazia as f64 / total.max(1) as f64;
            linha += &format!(
                " · [{lado}×{fatias}: {razao:.3} · {vazia:.1} % · {} · {ms:.0}]",
                conta[c].len()
            );
        }
        println!("{linha}");
    }
    println!();
}
