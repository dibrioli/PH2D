//! ⭐⭐⭐ **O FILTRO MEDE O ARRASTO, NÃO CONTA EVENTOS** — a lei que este repo já
//! pagou seis vezes no Painter (*o traço é facto do **CAMINHO**, nunca de quão
//! fino o motor amostrou o caminho*), aplicada ao *Expand*.
//!
//! # O defeito, medido antes da cura
//!
//! No filtro, `S` é a distância acumulada ao ponto de pressão e **um passo é um
//! movimento do rato** ([espec](../../../docs/3D/cleanroom/SPEC_cloth_brush.md)
//! §7). O *Expand* soma `τ += 0,01 · f` **por passo** ⇒ dois artistas que
//! arrastem exactamente o mesmo tanto recebem `τ` proporcionais ao **polling do
//! rato deles**. Medido no produto, o MESMO arrasto (`s` de `0` a `1`) sobre uma
//! esfera, volume normalizado ao repouso:
//!
//! | amostras | 8 | 15 | 30 | 60 | 120 | 240 |
//! |---|---:|---:|---:|---:|---:|---:|
//! | como estava | `1,97` | `2,93` | `3,50` | `5,49` | `19,51` | **`55,84`** |
//!
//! ⚠️ **Os outros quatro tipos não acumulam nada** — eles escrevem uma força ou
//! uma âncora, e o equilíbrio contra a rede é fixado pela MAGNITUDE de `S`. Só o
//! *Expand* soma, e por isso só ele é multiplicado pela amostragem.
//!
//! # A régua é o `τ`, e não a malha — e a razão é do instrumento
//!
//! ⛔⛔ **Uma barra sobre a malha seria uma barra sobre OUTRA coisa:** cada passo
//! corre também as varreduras de relaxação e a integração, logo `240` passos
//! relaxam `30×` mais que `8` e a peça chega mais perto do repouso novo **mesmo
//! com o `τ` idêntico**. *O que a cura afirma é sobre o desvio de repouso; medir a
//! malha misturaria a afirmação com o número de relaxações.*
//!
//! ⚠️ E a soma é uma **soma de Riemann à direita** de `∫₀¹ s ds`, então ela
//! converge com a amostragem em vez de ser exacta: a soma vale `(n+1)/2n` contra
//! o integral `1/2`, logo o desvio é **`1/n`** — `12,5 %` a `8` amostras e
//! `0,4 %` a `240`. ⭐ **A barra é essa conta, com `5 %` de margem** — ⛔ não um
//! número escolhido, e a primeira redacção escreveu `1/(2n)` e reprovou sobre
//! produto correcto (`12,0 %` medido contra `6,25 %` afirmado). *Quem escreve uma
//! barra derivada tem de derivar a conta certa.*

use ph2d_cloth::V3;
use ph2d_cloth::verlet_gesto::{Accionamento, Area, Modo, Passo, Pincel, PincelTecido};

/// Uma grelha `n × n` no plano `z = 0`, com passo `h`.
fn grelha(n: usize, h: f64) -> (Vec<V3>, Vec<Vec<u32>>) {
    let meio = (n - 1) as f64 * h * 0.5;
    let mut pos = Vec::with_capacity(n * n);
    for j in 0..n {
        for i in 0..n {
            pos.push([i as f64 * h - meio, j as f64 * h - meio, 0.0]);
        }
    }
    let mut faces = Vec::new();
    for j in 0..n - 1 {
        for i in 0..n - 1 {
            let a = (j * n + i) as u32;
            faces.push(vec![a, a + 1, a + 1 + n as u32, a + n as u32]);
        }
    }
    (pos, faces)
}

/// **A soma de `τ` no fim de um arrasto de `s = 0` a `s = 1` em `n` passos.**
fn tau_do_arrasto(n: usize) -> f64 {
    let (base, faces) = grelha(16, 0.1);
    let mut aneis: Vec<Vec<u32>> = vec![Vec::new(); base.len()];
    for f in &faces {
        for (k, &v) in f.iter().enumerate() {
            aneis[v as usize].push(f[(k + 1) % f.len()]);
        }
    }
    let pincel = Pincel {
        modo: Modo::Expandir,
        area: Area::Global,
        accionamento: Accionamento::Filtro { s: 0.0 },
        ..Pincel::default()
    };
    let mut t = PincelTecido::pen_down(pincel, &base, [0.0, 0.0, 0.0], Vec::new());
    let normais = vec![[0.0, 0.0, 1.0]; base.len()];
    let anel = |v: u32| aneis[v as usize].clone();
    let mut pos = base.clone();
    for k in 0..=n {
        t.pincel.accionamento = Accionamento::Filtro {
            s: k as f64 / n as f64,
        };
        let p = Passo {
            cursor: [0.0, 0.0, 0.0],
            delta: [0.0; 3],
            delta_3d: [0.0; 3],
            parado: false,
            vista: [0.0, 0.0, 1.0],
            normais: &normais,
            pressao: 1.0,
        };
        t.passo(&pos, &anel, &p);
        pos.clone_from(&t.sim.x);
    }
    t.sim.tau.iter().sum()
}

/// ⭐⭐⭐ **O MESMO ARRASTO DÁ O MESMO `τ`, seja qual for o rato.**
#[test]
fn o_expand_mede_o_arrasto_e_nao_conta_eventos() {
    let amostras = [8usize, 15, 30, 60, 120, 240];
    let taus: Vec<f64> = amostras.iter().map(|&n| tau_do_arrasto(n)).collect();
    for (n, t) in amostras.iter().zip(&taus) {
        println!("n={n:>4}: tau {t:.6}");
    }
    // ⭐ A referência é a amostragem mais FINA, que é a que está mais perto do
    // integral; a barra é a soma de Riemann da mais grosseira, `1/(2·8)`.
    let fino = taus[taus.len() - 1];
    for (n, t) in amostras.iter().zip(&taus) {
        let erro = (t / fino - 1.0).abs();
        let barra = 1.05 / *n as f64;
        assert!(
            erro < barra,
            "com {n} amostras o tau desviou {:.1}% do arrasto ({t:.6} contra {fino:.6}); \
             a discretizacao explica {:.1}%",
            100.0 * erro,
            100.0 * barra
        );
    }
}
