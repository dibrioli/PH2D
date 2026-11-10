//! B14.3 (missão 4K): a `SilMemo` reaproveita a caixa entre dabs — a silhueta guardada é a do oráculo AO BIT
//! (o `SilMemo` de antes, verbatim) e o `nova` deixa de escrever a caixa inteira (6 MB no Flat a 240).

use super::{INICIADAS, SilMemo};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

/// O `SilMemo` de antes de B14.3, verbatim (cada `nova` aloca e preenche a caixa).
mod oraculo {
    use std::sync::atomic::{AtomicU64, Ordering};
    const VAZIA: u64 = u64::MAX;
    pub(super) struct SilMemo {
        x0: i32,
        y0: i32,
        w: i32,
        h: i32,
        v: Vec<AtomicU64>,
    }
    impl SilMemo {
        pub(super) fn nova(x: f64, y: f64, meia: [f64; 2]) -> Self {
            let (ax, ay) = (meia[0].ceil() as i32 + 1, meia[1].ceil() as i32 + 1);
            let (cx, cy) = (x.round() as i32, y.round() as i32);
            let (w, h) = (2 * ax + 1, 2 * ay + 1);
            Self {
                x0: cx - ax,
                y0: cy - ay,
                w,
                h,
                v: (0..w * h).map(|_| AtomicU64::new(VAZIA)).collect(),
            }
        }
        pub(super) fn ou(&self, cx: i32, cy: i32, sil: impl Fn(i32, i32) -> f64) -> f64 {
            let (i, j) = (cx - self.x0, cy - self.y0);
            if i < 0 || j < 0 || i >= self.w || j >= self.h {
                return sil(cx, cy);
            }
            let c = &self.v[(j * self.w + i) as usize];
            match c.load(Ordering::Relaxed) {
                VAZIA => {
                    let v = sil(cx, cy);
                    c.store(v.to_bits(), Ordering::Relaxed);
                    v
                }
                b => f64::from_bits(b),
            }
        }
    }
}

/// Uma silhueta pura de dab (depende do dab `k`), com zeros, negativos-zero e subnormais à mistura.
fn silhueta(k: u32, cx: i32, cy: i32) -> f64 {
    let h = (cx.wrapping_mul(73_856_093)
        ^ cy.wrapping_mul(19_349_663)
        ^ (k as i32).wrapping_mul(83_492_791)) as u32;
    match h % 7 {
        0 => 0.0,
        1 => -0.0,
        2 => f64::MIN_POSITIVE / 3.0,
        _ => f64::from(h) / f64::from(u32::MAX),
    }
}

/// Os dabs de um traço: tamanhos que crescem, encolhem e se repetem, centros a andar.
fn dabs() -> Vec<(f64, f64, [f64; 2])> {
    (0..24)
        .map(|k| {
            let t = f64::from(k);
            let meia = [3.0 + (t * 1.7) % 19.0, 2.0 + (t * 2.3) % 11.0];
            (40.0 + t * 2.6, 30.0 - t * 1.1, meia)
        })
        .collect()
}

#[test]
fn a_memo_reaproveitada_da_a_silhueta_do_oraculo_ao_bit() {
    for (k, (x, y, meia)) in dabs().into_iter().enumerate() {
        let k = k as u32;
        let novo = SilMemo::nova(x, y, meia);
        let velho = oraculo::SilMemo::nova(x, y, meia);
        let (n_novo, n_velho) = (AtomicUsize::new(0), AtomicUsize::new(0));
        let (cx, cy) = (x.round() as i32, y.round() as i32);
        // Cada célula lida 3× (a 1.ª calcula), e uma margem fora da caixa (calcula sempre).
        for _ in 0..3 {
            for j in cy - meia[1] as i32 - 4..=cy + meia[1] as i32 + 4 {
                for i in cx - meia[0] as i32 - 4..=cx + meia[0] as i32 + 4 {
                    let a = novo.ou(i, j, |a, b| {
                        n_novo.fetch_add(1, Ordering::Relaxed);
                        silhueta(k, a, b)
                    });
                    let b = velho.ou(i, j, |a, b| {
                        n_velho.fetch_add(1, Ordering::Relaxed);
                        silhueta(k, a, b)
                    });
                    assert_eq!(a.to_bits(), b.to_bits(), "dab {k} célula ({i}, {j})");
                }
            }
        }
        assert_eq!(
            n_novo.into_inner(),
            n_velho.into_inner(),
            "dab {k}: as mesmas chamadas da silhueta"
        );
    }
}

#[test]
fn a_memo_le_em_paralelo_o_mesmo_bit() {
    let memo = SilMemo::nova(100.0, 80.0, [40.0, 25.0]);
    let soma = AtomicU64::new(0);
    std::thread::scope(|s| {
        for _ in 0..4 {
            s.spawn(|| {
                let mut acc = 0u64;
                for j in 50..110 {
                    for i in 55..145 {
                        let v = memo.ou(i, j, |a, b| silhueta(7, a, b));
                        assert_eq!(v.to_bits(), silhueta(7, i, j).to_bits());
                        acc = acc.wrapping_add(v.to_bits());
                    }
                }
                soma.fetch_add(acc, Ordering::Relaxed);
            });
        }
    });
    assert_ne!(soma.into_inner(), 0);
}

/// ⭐ O `nova` a cada sub-dab escrevia a caixa inteira (~(1 617 × 487) × 8 B ≈ 6 MB no Flat a 240): da 2.ª vez,
/// com a caixa do mesmo tamanho, nenhuma célula é escrita à frente.
#[test]
fn a_memo_nao_inicializa_a_caixa_a_cada_dab() {
    let meia = [807.0, 242.0];
    drop(SilMemo::nova(1000.0, 600.0, meia));
    INICIADAS.with(|c| c.set(0));
    for k in 0..4 {
        let m = SilMemo::nova(1000.0 + f64::from(k) * 9.0, 600.0, meia);
        assert_eq!(
            m.ou(1000, 600, |a, b| silhueta(k, a, b)).to_bits(),
            silhueta(k, 1000, 600).to_bits()
        );
    }
    assert_eq!(
        INICIADAS.with(std::cell::Cell::get),
        0,
        "células escritas à frente em 4 dabs do mesmo tamanho"
    );
}

/// Sonda (smoke): `nova` + leitura de ¼ da caixa do Flat a 240, o oráculo × o reaproveitado, intercalados, o mínimo.
/// `bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --profile smoke --lib sonda_a_memo_do_flat_240 -- --ignored --nocapture`
#[test]
#[ignore = "sonda de custo — corre-se à mão no perfil smoke"]
fn sonda_a_memo_do_flat_240() {
    let meia = [807.0, 242.0];
    let le = |ou: &dyn Fn(i32, i32) -> f64| {
        let mut acc = 0u64;
        for j in (600 - 242..600 + 242).step_by(2) {
            for i in (1000 - 807..1000 + 807).step_by(2) {
                acc = acc.wrapping_add(ou(i, j).to_bits());
            }
        }
        acc
    };
    let mut melhor = [f64::INFINITY; 2];
    for ronda in 0..20 {
        for v in [ronda % 2, 1 - ronda % 2] {
            let t0 = std::time::Instant::now();
            let acc = if v == 0 {
                let m = oraculo::SilMemo::nova(1000.0, 600.0, meia);
                le(&|i, j| m.ou(i, j, |a, b| silhueta(1, a, b)))
            } else {
                let m = SilMemo::nova(1000.0, 600.0, meia);
                le(&|i, j| m.ou(i, j, |a, b| silhueta(1, a, b)))
            };
            melhor[v] = melhor[v].min(t0.elapsed().as_secs_f64() * 1e3);
            std::hint::black_box(acc);
        }
    }
    println!(
        "[sonda] SilMemo Flat 240 (¼ da caixa lida): antes {:.3} ms · depois {:.3} ms (mínimo de 20)",
        melhor[0], melhor[1]
    );
}
