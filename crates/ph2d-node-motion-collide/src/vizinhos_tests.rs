//! A varredura por vizinhos dá **os mesmos bits** que todos-os-pares — sobre cenas com tudo o que
//! muda a soma: raios variados, alfinetes, `falloff` parcial e nulo, corpos com repouso, pontos
//! coincidentes. Cada gate prova também que a GRELHA correu (senão compararia a referência consigo).

use super::super::{Estrutura, push_apart_com};
use super::{grelha_serve, lado_da_celula};
use ph2d_contact::GrelhaDeDiscos;

/// Um gerador determinístico (LCG) — sem `rand`, a cena é a mesma em toda corrida.
struct Lcg(u64);

impl Lcg {
    fn f(&mut self) -> f32 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        #[expect(clippy::cast_precision_loss, reason = "24 bits cabem num f32")]
        let v = (self.0 >> 40) as f32 / (1u64 << 24) as f32;
        v
    }
}

struct Cena {
    p: Vec<[f32; 2]>,
    w: Vec<f32>,
    radii: Vec<f32>,
    fall: Vec<f32>,
    estrutura: Option<Estrutura>,
}

/// `n` discos numa caixa de lado `caixa`, com tudo o que entra na soma.
fn cena(n: usize, caixa: f32, semente: u64, com_corpos: bool) -> Cena {
    let mut r = Lcg(semente);
    let mut p: Vec<[f32; 2]> = (0..n).map(|_| [r.f() * caixa, r.f() * caixa]).collect();
    // Coincidentes: o canto que depende do índice do par.
    for k in (0..n).step_by(37).skip(1) {
        p[k] = p[k - 1];
    }
    let radii = (0..n).map(|_| 0.2 + r.f()).collect();
    let w = (0..n)
        .map(|k| if k % 11 == 0 { 0.0 } else { 0.5 + r.f() })
        .collect();
    let fall = (0..n)
        .map(|k| match k % 9 {
            0 => 0.0,
            1 => 0.5,
            _ => 1.0,
        })
        .collect();
    let estrutura = com_corpos.then(|| Estrutura {
        rest: p.iter().map(|q| [q[0] * 0.9, q[1] * 0.9]).collect(),
        #[expect(clippy::cast_precision_loss, reason = "ids pequenos")]
        body: (0..n).map(|k| (k % 5) as f32).collect(),
    });
    Cena {
        p,
        w,
        radii,
        fall,
        estrutura,
    }
}

fn corre(c: &Cena, iteracoes: usize, grelha: bool) -> Vec<[f32; 2]> {
    push_apart_com(
        &c.p,
        &c.w,
        &c.radii,
        &c.fall,
        c.estrutura.as_ref(),
        iteracoes,
        1.0,
        grelha,
    )
}

fn bits(v: &[[f32; 2]]) -> Vec<[u32; 2]> {
    v.iter().map(|q| [q[0].to_bits(), q[1].to_bits()]).collect()
}

/// A grelha SERVE para a cena (o controlo de não-vacuidade).
fn grelha_usada(c: &Cena) -> bool {
    let r_max = c.radii.iter().fold(0.0_f32, |a, r| a.max(*r));
    grelha_serve(&mut GrelhaDeDiscos::default(), &c.p, lado_da_celula(r_max))
}

#[test]
fn a_grelha_da_os_mesmos_bits_que_todos_os_pares() {
    for (semente, corpos) in [(1, false), (2, true), (3, true)] {
        let c = cena(800, 40.0, semente, corpos);
        assert!(
            grelha_usada(&c),
            "semente {semente}: a grelha não serviu — o gate mediria nada"
        );
        let (a, b) = (corre(&c, 8, true), corre(&c, 8, false));
        assert_ne!(
            bits(&a),
            bits(&c.p),
            "semente {semente}: nada se moveu — a cena não colide"
        );
        assert_eq!(bits(&a), bits(&b), "semente {semente}: a grelha mudou bits");
    }
}

/// A cena apertada (muitos contactos por disco, a média por contagem a pesar) também.
#[test]
fn a_grelha_da_os_mesmos_bits_numa_nuvem_apertada() {
    let c = cena(6000, 50.0, 7, true);
    assert!(grelha_usada(&c));
    assert_eq!(bits(&corre(&c, 12, true)), bits(&corre(&c, 12, false)));
}

/// A decisão de custo: na grelha de lado 1 do tecto, raio `0,6` usa a grelha e raio `12` (todo
/// disco vê ~20 % dos outros) volta a todos-os-pares — onde a sonda a mediu `3,5×` mais lenta.
#[test]
fn o_raio_grande_volta_a_todos_os_pares() {
    let p: Vec<[f32; 2]> = (0..128 * 128)
        .map(|k| {
            #[expect(clippy::cast_precision_loss, reason = "128 cabe")]
            let (x, y) = ((k % 128) as f32, (k / 128) as f32);
            [x, y]
        })
        .collect();
    let serve = |raio: f32| grelha_serve(&mut GrelhaDeDiscos::default(), &p, lado_da_celula(raio));
    assert!(serve(0.6), "raio 0,6: a grelha tem de correr");
    assert!(
        serve(2.0),
        "raio 2: a grelha tem de correr (8× mais rápida)"
    );
    assert!(
        !serve(12.0),
        "raio 12: a grelha é mais lenta e tem de ser recusada"
    );
}

/// SONDA (não afirma nada): o relógio das duas passadas a `16 384` discos numa grelha de lado 1
/// (a fixtura do tecto), por raio. `cargo test -p ph2d-node-motion-collide sonda_custo -- --ignored
/// --nocapture`.
#[test]
#[ignore = "sonda: medicao"]
fn sonda_custo_da_grelha() {
    // (lado da grelha de pontos, espaçamento, raio, passadas) — a última é a cena `=8` no pico.
    for (lado, gap, raio, passadas) in [
        (128_usize, 1.0_f32, 0.3_f32, 12_usize),
        (128, 1.0, 0.6, 12),
        (128, 1.0, 2.0, 12),
        (128, 1.0, 12.0, 12),
        (181, 0.25, 0.3 * 1.35, 8),
    ] {
        let p: Vec<[f32; 2]> = (0..lado * lado)
            .map(|k| {
                #[expect(clippy::cast_precision_loss, reason = "181 cabe")]
                let (x, y) = ((k % lado) as f32 * gap, (k / lado) as f32 * gap);
                [x, y]
            })
            .collect();
        let n = p.len();
        let c = Cena {
            p,
            w: vec![1.0; n],
            radii: vec![raio; n],
            fall: vec![1.0; n],
            estrutura: None,
        };
        let mut melhor = [f64::MAX; 2];
        for _ in 0..2 {
            for (i, g) in [true, false].into_iter().enumerate() {
                let t = std::time::Instant::now();
                std::hint::black_box(corre(&c, passadas, g));
                melhor[i] = melhor[i].min(t.elapsed().as_secs_f64());
            }
        }
        eprintln!(
            "n {n} gap {gap} raio {raio} × {passadas}: grelha {:.3} s · todos-os-pares {:.3} s · \
             serve={}",
            melhor[0],
            melhor[1],
            grelha_usada(&c)
        );
    }
}

/// Um NaN passa o guarda do `par` e toca toda a gente: a grelha recusa-se, e os bits são os de
/// todos-os-pares.
#[test]
fn coordenada_nao_finita_recusa_a_grelha() {
    let mut c = cena(1000, 60.0, 5, false);
    assert!(grelha_usada(&c));
    c.p[3] = [f32::NAN, 1.0];
    assert!(!grelha_usada(&c), "um NaN tem de recusar a grelha");
    assert_eq!(bits(&corre(&c, 4, true)), bits(&corre(&c, 4, false)));
}

/// Longe da origem, onde `floor(x / lado)` arredonda grosso, a grelha CORRE e os bits continuam os
/// de todos-os-pares — a nuvem inteira deslocada até o espaçamento das posições ser da ordem do raio.
#[test]
fn longe_da_origem_a_grelha_corre_com_os_mesmos_bits() {
    for longe in [3.0e5_f32, 4.0e6] {
        let mut c = cena(1000, 60.0, 9, false);
        for q in &mut c.p {
            (q[0], q[1]) = (q[0] + longe, q[1] - longe);
        }
        assert!(
            grelha_usada(&c),
            "a {longe}: a grelha não serviu — o gate mediria nada"
        );
        let (a, b) = (corre(&c, 6, true), corre(&c, 6, false));
        assert_ne!(bits(&a), bits(&c.p), "a {longe}: nada se moveu");
        assert_eq!(bits(&a), bits(&b), "a {longe}: a grelha mudou bits");
    }
}
