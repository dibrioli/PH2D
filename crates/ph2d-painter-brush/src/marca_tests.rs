//! Os portões do jitter da MARCA (L4). A lei medida: `docs/Painter/Pinceis/06_LEIS_DAS_LACUNAS.md` §3.

use super::*;
use crate::{BrushSpec, Dab, Dynamics, Stroke, StrokePoint};

fn spec_base() -> BrushSpec {
    BrushSpec {
        radius_px: 20.0,
        spacing: 0.7,
        strength: 0.35,
        space_attenuation: false,
        stabilizer: 0.0,
        jitter: 0.15,
        jitter_scale: 0.3,
        jitter_rotate: 0.4,
        color_jitter_hue: 0.2,
        spray_count: 3,
        dab_flatten: 0.3,
        ..Default::default()
    }
}

fn sem_pressao() -> Dynamics {
    Dynamics {
        size_pressure: false,
        ..Default::default()
    }
}

/// Um recto horizontal de 880 px (o `recto_lento` da bateria), eventos de 4 px.
fn recto(spec: BrushSpec, seed: u64) -> Vec<Dab> {
    let mut s = Stroke::new(spec, sem_pressao(), seed);
    let (mut all, mut out) = (Vec::new(), Vec::new());
    let p = |x: f32| StrokePoint {
        pos: [60.0 + x, 300.0],
        pressure: 1.0,
    };
    s.begin(p(0.0), &mut out);
    all.extend_from_slice(&out);
    for i in 1..=220 {
        s.extend(p(i as f32 * 4.0), &mut out);
        all.extend_from_slice(&out);
    }
    s.finish(&mut out);
    all.extend_from_slice(&out);
    all
}

fn com(f: impl Fn(&mut DabJitter)) -> BrushSpec {
    let mut s = spec_base();
    f(&mut s.dab_jitter);
    s
}

fn media_sd(v: &[f32]) -> (f32, f32) {
    let n = v.len() as f32;
    let m = v.iter().sum::<f32>() / n;
    (m, (v.iter().map(|x| (x - m) * (x - m)).sum::<f32>() / n).sqrt())
}

/// ⭐ O neutro não sorteia nada: nenhum número sai do fluxo, a contagem e o rotor são os de sempre.
#[test]
fn o_neutro_nao_sorteia_nada() {
    let j = DabJitter::NEUTRO;
    let mut rng = 77u64;
    assert_eq!(sorteia(&mut rng, &j, 40.0, [1.0, 0.0]), None);
    assert_eq!(conta(&mut rng, &j, 8), 8);
    assert_eq!(rng, 77, "o neutro não avança o RNG");
    assert_eq!(rotor_do_traco(&j, 5), [1.0, 0.0]);
    assert_eq!(BrushSpec::default().dab_jitter, DabJitter::NEUTRO);
}

/// ⭐ **Cada campo, ligado SOZINHO, muda só o que é dele** — os sorteios de sempre (posição, escala, rotação,
/// cor) saem ao bit, porque os da marca sorteiam de um fluxo PRÓPRIO. É o «neutro ao bit por campo» dito ao
/// contrário: o que o campo não governa não se mexe.
#[test]
fn cada_campo_muda_so_o_que_e_dele() {
    let base = recto(spec_base(), 3);
    assert!(base.len() > 60, "{}", base.len());
    let iguais = |a: &Dab, b: &Dab| {
        (a.center, a.radius_px, a.color, a.coverage, a.rotation)
            == (b.center, b.radius_px, b.color, b.coverage, b.rotation)
    };
    // A opacidade: só a cobertura.
    let op = recto(com(|j| j.opacity = 0.3), 3);
    assert_eq!(op.len(), base.len());
    for (a, b) in base.iter().zip(&op) {
        assert_eq!((a.center, a.radius_px, a.color), (b.center, b.radius_px, b.color));
    }
    assert!(base.iter().zip(&op).any(|(a, b)| a.coverage != b.coverage));
    // A redondeza: só a pegada (o dab leva a forma; o raio fica a menos de um ulp do eixo maior).
    let rd = recto(com(|j| j.roundness = 0.6), 3);
    for (a, b) in base.iter().zip(&rd) {
        assert_eq!((a.center, a.color, a.coverage), (b.center, b.color, b.coverage));
        assert!((a.radius_px - b.radius_px).abs() <= 1e-4 * a.radius_px);
        assert!(b.tilt.is_some_and(|t| t.round <= 1.0 && t.round >= 0.4 && !t.espelho));
    }
    // Através: só o centro, e só na perpendicular (o traço é horizontal ⇒ o `x` fica).
    let at = recto(com(|j| j.across = 1.0), 3);
    for (a, b) in base.iter().zip(&at) {
        assert_eq!((a.radius_px, a.color, a.coverage), (b.radius_px, b.color, b.coverage));
        assert!((a.center[0] - b.center[0]).abs() < 1e-3, "{a:?} {b:?}");
    }
    assert!(base.iter().zip(&at).any(|(a, b)| a.center[1] != b.center[1]));
    // O ângulo do traço: só a rotação, e a MESMA em todo dab (o rotor é um só).
    let an = recto(com(|j| j.angle_per_stroke = 1.0), 3);
    for (a, b) in base.iter().zip(&an) {
        assert_eq!((a.center, a.radius_px, a.color, a.coverage), (b.center, b.radius_px, b.color, b.coverage));
    }
    let r = rotor_do_traco(&com(|j| j.angle_per_stroke = 1.0).dab_jitter, 3);
    assert_ne!(r, [1.0, 0.0]);
    assert_ne!(r, rotor_do_traco(&com(|j| j.angle_per_stroke = 1.0).dab_jitter, 4), "outro traço, outro ângulo");
    // A partícula: só o raio, × o factor.
    let pa = recto(com(|j| j.particle = 0.25), 3);
    for (a, b) in base.iter().zip(&pa) {
        assert_eq!((a.center, a.color, a.coverage), (b.center, b.color, b.coverage));
        assert!((b.radius_px - (a.radius_px * 0.25).max(0.5)).abs() < 1e-4);
    }
    // Os espelhos: só a pegada e a meia-volta do X.
    let fx = recto(com(|j| j.flip_x = 0.5), 3);
    for (a, b) in base.iter().zip(&fx) {
        assert_eq!((a.center, a.radius_px, a.color, a.coverage), (b.center, b.radius_px, b.color, b.coverage));
    }
    let virados = fx.iter().filter(|d| d.tilt.is_some_and(|t| t.espelho)).count();
    assert!(virados > fx.len() / 4 && virados < fx.len() * 3 / 4, "{virados}/{}", fx.len());
    // Nada ligado = a base inteira.
    let nada = recto(com(|_| {}), 3);
    assert!(base.iter().zip(&nada).all(|(a, b)| iguais(a, b)));
}

/// ⭐ **`o_jitter_de_opacidade_espalha_a_cobertura`** (06 §3): o Splats do Rebelle tem a opacidade por dab
/// com cv 0,08–0,20 em torno de 0,33–0,37 — a nossa a `0,35 ± 0,3` dá cv ≈ 0,17 e a MÉDIA fica (± 0,023).
#[test]
fn o_jitter_de_opacidade_espalha_a_cobertura() {
    let cob = |s: BrushSpec| recto(s, 11).iter().map(|d| d.coverage).collect::<Vec<_>>();
    let (m0, sd0) = media_sd(&cob(spec_base()));
    let (m1, sd1) = media_sd(&cob(com(|j| j.opacity = 0.3)));
    assert!(sd0 < 1e-6, "sem jitter a cobertura é uma só");
    assert!((m1 - m0).abs() <= 0.023, "a média fica: {m0} → {m1}");
    let cv = sd1 / m1;
    assert!((0.08..=0.20).contains(&cv), "cv {cv} (Rebelle 0,08–0,20)");
}

/// ⭐ **O espalhar através é ANISOTRÓPICO** (06 §3: o Splats espalha 0,61–0,92·D através, ~3× o ao longo).
/// O desvio de cada dab ao caminho, num recto horizontal com o disco pequeno (`jitter 0,15`) e o através a
/// `1,2·D`: sd através ÷ D em 0,6–0,9 e ≥ 2,5× o desvio ao longo que o disco dá.
#[test]
fn o_espalhar_atraves_e_anisotropico() {
    let mut s = com(|j| j.across = 1.2);
    s.spray_count = 1;
    let d = 2.0 * s.radius_px;
    let dabs = recto(s, 5);
    let mut base = s;
    base.dab_jitter = DabJitter::NEUTRO;
    let sem = recto(base, 5);
    let atraves: Vec<f32> = dabs.iter().map(|x| (x.center[1] - 300.0) / d).collect();
    // O ao longo é o do disco: o desvio de `x` à base sem disco não se mede aqui, mas o do disco ⊂ ±0,15·D.
    let ao_longo: Vec<f32> = sem.iter().map(|x| x.center[1] - 300.0).map(|v| v / d).collect();
    let (_, sd_at) = media_sd(&atraves);
    let (_, sd_disco) = media_sd(&ao_longo);
    assert!((0.6..=0.9).contains(&sd_at), "através sd {sd_at}·D (Rebelle 0,61–0,92)");
    assert!(sd_at >= 2.5 * sd_disco, "através {sd_at} × disco {sd_disco}");
}

/// A contagem sorteada fica entre 1 e `n`, e varia.
#[test]
fn a_contagem_sorteada_fica_entre_um_e_n() {
    let j = DabJitter {
        count: 1.0,
        ..DabJitter::NEUTRO
    };
    let mut rng = 9u64;
    let v: Vec<u32> = (0..400).map(|_| conta(&mut rng, &j, 8)).collect();
    assert!(v.iter().all(|&n| (1..=8).contains(&n)));
    assert!(v.contains(&1) && v.contains(&8) && v.contains(&4));
    let mut a = com(|j| j.count = 1.0);
    a.spray_count = 8;
    let mut b = a;
    b.dab_jitter.count = 0.0;
    assert!(recto(a, 2).len() < recto(b, 2).len() * 3 / 4);
}

/// ⭐ **O espelho vira a PONTA, não o disco** — a pegada espelhada lê o `v` trocado no referencial da ponta; o
/// falloff (simétrico) fica, as linhas da matriz que o device lê viram com ele, e sem espelho tudo é ao bit.
#[test]
fn o_espelho_vira_a_ponta() {
    let fp = crate::FootprintDeform::new(0.4, 30);
    let e = fp.espelhada(true);
    for p in [[0.3f32, 0.7], [-0.5, 0.2], [0.9, -0.1]] {
        let (a, b) = (fp.apply(p), e.apply(p));
        assert_eq!(a[0], b[0]);
        assert_eq!(a[1], -b[1]);
        assert_eq!(fp.falloff_t(p[0], p[1]), e.falloff_t(p[0], p[1]));
        let back = e.unapply_linear(e.apply(p));
        assert!((back[0] - p[0]).abs() < 1e-5 && (back[1] - p[1]).abs() < 1e-5);
    }
    let (r, re) = (fp.linear_rows(), e.linear_rows());
    assert_eq!((r[0], [-r[1][0], -r[1][1]]), (re[0], re[1]));
    assert_eq!(fp.espelhada(false), fp);
    assert!(!crate::FootprintDeform::identity().espelhada(true).is_identity());
}

/// Um recto vertical de 600 px a `passo` px por evento, um tique de 10 ms por evento (300 px/s a 3 px).
fn recto_a(passo: f32, d: Dynamics) -> Vec<Dab> {
    let s = BrushSpec {
        radius_px: 20.0,
        spacing: 0.1,
        space_attenuation: false,
        stabilizer: 0.0,
        ..Default::default()
    };
    let mut st = Stroke::new(s, d, 1);
    let (mut all, mut out) = (Vec::new(), Vec::new());
    let p = |y: f32| StrokePoint {
        pos: [500.0, 20.0 + y],
        pressure: 1.0,
    };
    st.begin(p(0.0), &mut out);
    all.extend_from_slice(&out);
    let n = (600.0 / passo) as usize;
    for i in 1..=n {
        st.extend(p(i as f32 * passo), &mut out);
        all.extend_from_slice(&out);
        st.tick(0.01, &mut out);
        all.extend_from_slice(&out);
    }
    all
}

/// A largura (o raio mediano) da metade final do recto — a rampa da velocidade já assentou.
fn raio_assente(passo: f32, d: Dynamics) -> f32 {
    let mut r: Vec<f32> = recto_a(passo, d)
        .iter()
        .filter(|x| x.center[1] > 320.0)
        .map(|x| x.radius_px)
        .collect();
    r.sort_by(f32::total_cmp);
    r[r.len() / 2]
}

/// ⭐ **`a_velocidade_estreita_como_o_fineliner`** (06 §10.4): o Fineliner do Krita a 3 · 6 · 12 · 25 · 50 px
/// por evento (300 → 5 000 px/s) dá a largura 1,00 · 0,95 · 0,95 · 0,90 · 0,85 da lenta; a nossa com
/// `speed_size 0,058` cai na curva ± 0,04 — a curva INTEIRA, não só o par. E sem a lei, 1,00 ± 0,001.
#[test]
fn a_velocidade_estreita_como_o_fineliner() {
    let fineliner = Dynamics {
        size_pressure: false,
        speed_size: 0.058,
        ..Default::default()
    };
    let neutro = Dynamics {
        size_pressure: false,
        ..Default::default()
    };
    let lento = raio_assente(3.0, fineliner);
    for (passo, krita) in [(6.0, 0.95), (12.0, 0.95), (25.0, 0.90), (50.0, 0.85)] {
        let r = raio_assente(passo, fineliner) / lento;
        assert!((r - krita).abs() <= 0.04, "{passo} px/evento: {r} (Krita {krita})");
        let n = raio_assente(passo, neutro) / raio_assente(3.0, neutro);
        assert!((n - 1.0).abs() <= 1e-3, "sem a lei a velocidade não conta: {n}");
    }
    // A cobertura pela mesma lei (o Thin_Pen do MyPaint: 0,57 a 5 000 px/s com `p ≈ 0,2`).
    let thin = Dynamics {
        size_pressure: false,
        speed_opacity: 0.2,
        ..Default::default()
    };
    let cob = |passo: f32| {
        let v: Vec<f32> = recto_a(passo, thin)
            .iter()
            .filter(|x| x.center[1] > 320.0)
            .map(|x| x.coverage)
            .collect();
        v[v.len() / 2]
    };
    let c = cob(50.0) / cob(3.0);
    assert!((c - 0.57).abs() <= 0.05, "Thin_Pen 0,57: {c}");
}

/// O neutro da lei: `p = 0` e a velocidade lenta dão `1` exacto, sem `powf`.
#[test]
fn a_lei_da_velocidade_e_neutra_no_zero() {
    assert_eq!(crate::dynamics::lei_da_velocidade(0.0, 9_000.0), 1.0);
    assert_eq!(crate::dynamics::lei_da_velocidade(0.4, 300.0), 1.0);
    assert!(crate::dynamics::lei_da_velocidade(0.45, 4_800.0) < 0.3);
    assert!(!Dynamics::default().speed_armed());
}

/// ⭐ **O espalhar ATRAVÉS também é de cada sub-dab da cápsula** (Spacing Continuous, doc 51 §4): cada sub-dab tira
/// o SEU desvio através, como o traçador faria se pousasse um dab ali — senão todos herdavam o do dab da frente e a
/// cápsula era uma faixa lisa deslocada (revisão do bloco C). Num recto horizontal, o `y` dos sub-dabs de UMA
/// cápsula varia (sd ≥ 0,2·D); sem o desvio através, controlo, não varia.
#[test]
fn o_espalhar_atraves_e_de_cada_sub_dab_da_capsula() {
    let sd_dentro = |across: f32| {
        let mut s = com(|j| j.across = across);
        s.spray_count = 1;
        s.jitter = 0.0;
        s.continuous_spacing = true;
        s.spacing = 1.5;
        let d = 2.0 * s.radius_px;
        let mut sds = Vec::new();
        for dab in recto(s, 7) {
            let subs = crate::capsula::sub_dabs(&dab, crate::capsula::sub_passo_px(dab.radius_px));
            if subs.len() >= 4 {
                let y: Vec<f32> = subs.iter().map(|x| x.center[1] / d).collect();
                sds.push(media_sd(&y).1);
            }
        }
        assert!(sds.len() > 5, "o recto tem de pousar cápsulas: {}", sds.len());
        sds.iter().sum::<f32>() / sds.len() as f32
    };
    assert!(sd_dentro(0.0) < 1e-4, "controlo: sem através a cápsula é uma faixa");
    let v = sd_dentro(1.0);
    assert!(v >= 0.2, "dentro de uma cápsula o através não varia: sd {v}·D");
}

/// ⭐ **O espalhar através também é dos dabs que preenchem o vão do ARREMESSO** (`LineKind::Speed`, `speed.rs`): cada
/// preenchimento sorteia o seu desvio através, como um dab do caminho — antes saíam todos SOBRE o caminho (só o
/// disco), uma fileira alinhada no meio da nuvem. Recto horizontal rápido: nenhum dab, além do primeiro (sem
/// direcção ainda), fica no eixo do traço.
#[test]
fn o_espalhar_atraves_e_dos_preenchimentos_do_arremesso() {
    let mut s = com(|j| j.across = 1.0);
    s.spray_count = 1;
    s.jitter = 0.0;
    s.line_kind = crate::line_kind::LineKind::Speed;
    let mut st = Stroke::new(s, sem_pressao(), 4);
    let (mut all, mut out) = (Vec::new(), Vec::new());
    let p = |x: f32| StrokePoint {
        pos: [60.0 + x, 300.0],
        pressure: 1.0,
    };
    st.begin(p(0.0), &mut out);
    all.extend_from_slice(&out);
    for i in 1..=40 {
        st.extend(p(i as f32 * 20.0), &mut out);
        all.extend_from_slice(&out);
        st.tick(1.0 / 60.0, &mut out);
        all.extend_from_slice(&out);
    }
    assert!(all.len() > 40, "o arremesso preenche: {}", all.len());
    let no_eixo = all.iter().filter(|d| (d.center[1] - 300.0).abs() < 1e-3).count();
    assert!(no_eixo <= 2, "{no_eixo} de {} dabs sem o desvio através", all.len());
}

/// ⭐ **A partícula em PÍXEIS** (J4 `texturas`, `docs/Painter/Pinceis/exames/texturas.md`): com `particle_px` cada
/// marca do spray tem o MESMO diâmetro na tela num pincel de 40 e de 200 px (a gotícula do Krita), e o
/// espalhamento continua a crescer com o pincel; num pincel MENOR que ela a gota fica do seu tamanho (o cordão do
/// Krita a 5 px); desligada = a partícula relativa.
#[test]
fn a_particula_em_px_e_a_mesma_em_todo_pincel() {
    let mede = |raio: f32, px: f32| {
        let mut s = com(|j| j.particle_px = px);
        s.radius_px = raio;
        s.jitter_scale = 0.0;
        let d = recto(s, 5);
        let r: Vec<f32> = d.iter().map(|d| d.radius_px).collect();
        let y: Vec<f32> = d.iter().map(|d| d.center[1]).collect();
        let (lo, hi) = (r.iter().copied().fold(f32::MAX, f32::min), r.iter().copied().fold(0.0, f32::max));
        (lo, hi, media_sd(&y).1)
    };
    let (a_lo, a_hi, a_sd) = mede(20.0, 6.0);
    let (b_lo, b_hi, b_sd) = mede(100.0, 6.0);
    assert!((a_lo - 3.0).abs() < 1e-4 && (a_hi - 3.0).abs() < 1e-4, "a 40 px: raio {a_lo}..{a_hi}");
    assert!((b_lo - 3.0).abs() < 1e-4 && (b_hi - 3.0).abs() < 1e-4, "a 200 px: raio {b_lo}..{b_hi}");
    assert!(b_sd > 3.0 * a_sd, "o espalhamento é do pincel: sd {a_sd} × {b_sd}");
    // Num pincel de 4 px a gota de 6 continua de 6; e desligada é a relativa de sempre.
    let (c_lo, c_hi, _) = mede(2.0, 6.0);
    assert!((c_lo - 3.0).abs() < 1e-4 && (c_hi - 3.0).abs() < 1e-4, "a 4 px: raio {c_lo}..{c_hi}");
    let (d_lo, _, _) = mede(100.0, 0.0);
    assert!((d_lo - 100.0).abs() < 1e-4, "desligada: {d_lo}");
}

/// ⭐ **O tecto da partícula em px** (J4 `texturas`, o Spatter/Star do Rebelle: nenhuma estrela passa de 56 px a 200
/// px, o espalhamento continua o do pincel): com `particle_max_px` nenhuma marca do spray passa de metade dele de
/// raio, as POSIÇÕES são as de sem o tecto (o espalhamento leu o pincel), e as menores que ele ficam ao bit.
#[test]
fn o_tecto_da_particula_corta_so_as_maiores() {
    let mut s = spec_base();
    s.radius_px = 100.0;
    let sem = recto(s, 7);
    s.dab_jitter.particle_max_px = 200.0;
    let com_tecto = recto(s, 7);
    assert_eq!(sem.len(), com_tecto.len());
    let mut cortadas = 0;
    for (a, b) in sem.iter().zip(&com_tecto) {
        assert_eq!(a.center, b.center, "o espalhamento é do pincel");
        assert!(b.radius_px <= 100.0 + 1e-4, "raio {} acima do tecto", b.radius_px);
        if a.radius_px <= 100.0 {
            assert_eq!(a.radius_px, b.radius_px);
        } else {
            cortadas += 1;
        }
    }
    assert!(cortadas > 0 && cortadas < sem.len(), "{cortadas} de {} cortadas", sem.len());
}

/// ⭐ **O VÉU** (J4 `texturas`, o Texture Spray do Krita: a névoa clara por baixo das gotas): a fracção `veu` das marcas
/// sai maior e pálida (`veu_escala`, `veu_opacidade`), sorteada do fluxo PRÓPRIO — as outras marcas ficam AO BIT.
#[test]
fn o_veu_e_uma_segunda_populacao_maior_e_palida() {
    let base = spec_base();
    let sem = recto(base, 3);
    let mut s = base;
    s.dab_jitter.veu = 0.3;
    s.dab_jitter.veu_escala = 3.0;
    s.dab_jitter.veu_opacidade = 0.2;
    let com_veu = recto(s, 3);
    assert_eq!(sem.len(), com_veu.len());
    let mut veu = 0;
    for (a, b) in sem.iter().zip(&com_veu) {
        assert_eq!(a.center, b.center);
        if b.radius_px == a.radius_px {
            assert_eq!(a.coverage, b.coverage);
        } else {
            veu += 1;
            assert!((b.radius_px - 3.0 * a.radius_px).abs() < 1e-3, "{} × {}", b.radius_px, a.radius_px);
            assert!((b.coverage - 0.2 * a.coverage).abs() < 1e-5);
        }
    }
    let f = veu as f32 / sem.len() as f32;
    assert!((0.2..=0.4).contains(&f), "véu {veu} de {} ({f})", sem.len());
}
