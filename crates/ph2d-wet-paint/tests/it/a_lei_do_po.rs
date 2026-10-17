//! **O PÓ do pincel** (prompt 43, doc `43_medicao_o_po_fino.md`): com `Dust > 0` a tinta de cada dab cai em
//! GOTAS de 2 células — a gota cai com a probabilidade `p = 1 − 0,95·dust` e leva a dose `÷ p`.

use ph2d_wet_paint::painter::Engine;
use ph2d_wet_paint::trail::{Dab, Trail, TrailMode};
use ph2d_wet_paint::tuning::Knob;

fn dab(x: f64) -> Dab {
    Dab {
        x,
        y: 100.0,
        r: 30.0,
        hardness: 9.0,
        intensity: 0.4,
        water_amount: 0.0,
        dry_gate: 0.0,
        reach: 1.0,
        window_reach: 1.0,
    }
}

/// O pigmento da janela do rasto depois de UM dab em `x`, com o pó `po` e a semente do hospedeiro `semente`.
fn pousa(po: Option<f64>, semente: u32, x: f64) -> Vec<f32> {
    let mut e = Engine::new(200, 200);
    e.tuning.set(Knob::Drag, 0.0);
    // A média é a de sempre ANTES do portão do papel (a gota concentrada vence o dente que o contínuo não
    // vence): o papel neutro e a ponta lisa medem a lei sozinha.
    e.tuning.set(Knob::PaperGate, 0.0);
    if let Some(po) = po {
        e.tuning.set(Knob::Dust, po);
    }
    e.sim.semente_do_po = semente;
    let p = e.sim.gather_params(&e.tuning);
    let tex = vec![1.0; e.bristle_texture_for_measure().len()];
    let mut t = Trail::default();
    t.start_stroke(100.0, 100.0, [23.0, 101.0, 179.0], TrailMode::Paint);
    t.accumulate_paint(e.active_grid_mut(), &p, &tex, &dab(x), false);
    t.planes_for_measure().0.to_vec()
}

/// O lado da janela (quadrada) — para ler vizinhos.
fn lado(v: &[f32]) -> usize {
    (v.len() as f64).sqrt() as usize
}

/// Pó 0 é o carimbo contínuo AO BIT, seja qual for a semente; e todo o miolo recebe tinta.
#[test]
fn po_zero_e_o_carimbo_de_sempre() {
    let antes = pousa(None, 0, 100.0);
    assert_eq!(pousa(Some(0.0), 12345, 100.0), antes);
    assert!(antes.iter().filter(|&&v| v > 0.0).count() > 2000);
}

/// ⭐ **A massa é a de sempre**: a média de 24 sementes com pó 0,8 é a massa do carimbo contínuo (± 3 %) — a
/// gota leva a dose das que faltam.
#[test]
fn o_po_conserva_a_massa() {
    let soma = |v: &[f32]| v.iter().map(|&x| f64::from(x)).sum::<f64>();
    let continuo = soma(&pousa(Some(0.0), 0, 100.0));
    let po = (1..=24)
        .map(|s| soma(&pousa(Some(0.8), s, 100.0)))
        .sum::<f64>()
        / 24.0;
    assert!(
        (po / continuo - 1.0).abs() < 0.03,
        "pó {po:.1} contra o contínuo {continuo:.1}"
    );
}

/// ⭐ **As gotas são de 2 células, e caem na fracção `p`** (o grão do Rebelle: 1,28 px a 20, 40 e 80 px): a
/// correlação da máscara com a vizinha a 1 célula é ½ (meio par cai na mesma gota), a 2 células é 0.
#[test]
fn as_gotas_sao_de_duas_celulas() {
    let (mut cai, mut tudo) = (0.0, 0.0);
    let (mut c1, mut c2) = (Vec::new(), Vec::new());
    for s in 1..=8 {
        let v = pousa(Some(0.8), s, 100.0);
        let n = lado(&v);
        let m = |x: usize, y: usize| f64::from(u8::from(v[y * n + x] > 0.0));
        let base = pousa(Some(0.0), 0, 100.0);
        let dentro = |x: usize, y: usize| base[y * n + x] > 0.0;
        for y in 0..n {
            for x in 0..n - 2 {
                if dentro(x, y) && dentro(x + 1, y) && dentro(x + 2, y) {
                    tudo += 1.0;
                    cai += m(x, y);
                    c1.push((m(x, y), m(x + 1, y)));
                    c2.push((m(x, y), m(x + 2, y)));
                }
            }
        }
    }
    let corr = |pares: &[(f64, f64)]| {
        let k = pares.len() as f64;
        let (ma, mb) = pares
            .iter()
            .fold((0.0, 0.0), |(a, b), &(x, y)| (a + x / k, b + y / k));
        let cov = pares.iter().map(|&(x, y)| (x - ma) * (y - mb)).sum::<f64>() / k;
        let va = pares.iter().map(|&(x, _)| (x - ma).powi(2)).sum::<f64>() / k;
        let vb = pares.iter().map(|&(_, y)| (y - mb).powi(2)).sum::<f64>() / k;
        cov / (va * vb).sqrt()
    };
    let p = cai / tudo;
    assert!((p - 0.24).abs() < 0.03, "caem {p:.3} das gotas (p = 0,24)");
    let (r1, r2) = (corr(&c1), corr(&c2));
    assert!((0.4..0.6).contains(&r1), "a 1 célula: {r1:.3}");
    assert!(r2.abs() < 0.1, "a 2 células: {r2:.3}");
}

/// A semente do hospedeiro escolhe as gotas — a mesma semente, as mesmas gotas (ao bit); outra, outras.
#[test]
fn a_semente_escolhe_as_gotas() {
    let a = pousa(Some(0.8), 1, 100.0);
    assert_eq!(pousa(Some(0.8), 1, 100.0), a);
    let b = pousa(Some(0.8), 2, 100.0);
    let difere = a
        .iter()
        .zip(&b)
        .filter(|(x, y)| (**x > 0.0) != (**y > 0.0))
        .count();
    assert!(difere > 400, "{difere} células mudaram com a semente");
}

/// Sem hospedeiro (semente 0), o CENTRO do dab muda as gotas: dois dabs a ¼ de célula um do outro caem nas
/// MESMAS células (a gota é da célula de tela), e só a semente — que leva o centro — as distingue.
#[test]
fn o_centro_do_dab_muda_as_gotas() {
    let a = pousa(Some(0.8), 0, 100.0);
    let b = pousa(Some(0.8), 0, 100.25);
    let difere = a
        .iter()
        .zip(&b)
        .filter(|(x, y)| (**x > 0.0) != (**y > 0.0))
        .count();
    assert!(difere > 400, "{difere} células mudaram com o centro");
}
