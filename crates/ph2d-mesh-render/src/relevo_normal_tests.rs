//! ⭐⭐⭐ **Os gates do HORIZONTE** — a lei pura, sem placa. A paridade da placa
//! vive em `tests/it/relevo_horizonte.rs`.

use super::{HORIZONTE_T, horizonte, inclina, normaliza};

/// O `canvas_normal` do `mesh.wgsl`, na metade que importa aqui: uma normal
/// vista por trás é virada INTEIRA.
fn canvas(n: [f32; 3]) -> [f32; 3] {
    if n[2] < 0.0 { [-n[0], -n[1], -n[2]] } else { n }
}

/// A lei de ANTES do horizonte — o CONTROLO de cada gate.
fn sem_horizonte(n_in: [f32; 3], gv: [f32; 3]) -> [f32; 3] {
    let n = normaliza(n_in);
    let d = n[0] * gv[0] + n[1] * gv[1] + n[2] * gv[2];
    normaliza([
        n[0] - (gv[0] - n[0] * d),
        n[1] - (gv[1] - n[1] * d),
        n[2] - (gv[2] - n[2] * d),
    ])
}

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// A base de uma peça vista quase de lado: inclinada `alfa` para `+x`.
fn base(z: f32) -> [f32; 3] {
    [(1.0 - z * z).sqrt(), 0.0, z]
}

/// Um gerador sem dependências, determinista: `(base, gradiente)`, com o
/// gradiente escalado por um factor em `[-0,1; 1,1]` preso a `0..1` — uma parte
/// das amostras sem declive nenhum.
fn amostras(n: usize) -> impl Iterator<Item = ([f32; 3], [f32; 3])> {
    let mut s = 0x9e37_79b9_7f4a_7c15_u64;
    let mut r = move || {
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        (s >> 40) as f32 / (1u64 << 24) as f32
    };
    (0..n).map(move |_| {
        let n = [r() * 2.0 - 1.0, r() * 2.0 - 1.0, r() * 2.0 - 1.0];
        let g = [(r() - 0.5) * 16.0, (r() - 0.5) * 16.0, (r() - 0.5) * 16.0];
        let c = (r() * 1.2 - 0.1).clamp(0.0, 1.0);
        (n, g.map(|x| x * c))
    })
}

/// ⭐⭐⭐ **GATE — A LUZ NÃO SALTA NA PONTA DO TRAÇO VISTA DE LADO** (report do
/// dono de 01/10, 2.ª volta, a seta na foto). A encosta da ponta cresce para
/// fora sobre uma base quase de lado (`z = 0,15`), em passos finos; a régua é
/// o maior passo da normal QUE A LUZ LÊ (depois do `canvas_normal`) entre dois
/// vizinhos. ⚠️ O CONTROLO vem primeiro: a lei de antes TEM de saltar ali,
/// senão a fixtura não contém o fenómeno.
#[test]
fn a_luz_nao_salta_na_ponta_vista_de_lado() {
    for z in [0.05f32, 0.15, 0.3] {
        let n = base(z);
        // A encosta aponta para FORA da peça (para `+x`, o lado do contorno):
        // o gradiente da altura aponta para dentro.
        let passo = |lei: &dyn Fn(f32) -> [f32; 3]| {
            let mut pior = 0.0f32;
            let mut antes = canvas(lei(0.0));
            for i in 1..=4000 {
                let agora = canvas(lei(i as f32 * 1e-3));
                pior = pior.max(dist(antes, agora));
                antes = agora;
            }
            pior
        };
        let velha = passo(&|a| sem_horizonte(n, [-a, 0.0, 0.0]));
        let nova = passo(&|a| inclina(n, [-a, 0.0, 0.0]));
        if z < HORIZONTE_T + 0.2 {
            assert!(
                velha > 0.5,
                "CONTROLO z={z}: a lei de antes não salta ({velha}) — a fixtura não tem o fenómeno"
            );
        }
        assert!(
            nova < 0.01,
            "z={z}: a luz salta {nova} entre dois passos de 1e-3 da encosta (antes: {velha})"
        );
    }
}

/// ⭐⭐⭐ **GATE — A NORMAL INCLINADA NUNCA PASSA O HORIZONTE**, para o lado de
/// trás da base (que é o que o `canvas_normal` viraria). O CONTROLO: a lei de
/// antes passa-o num pedaço real das amostras.
#[test]
fn a_normal_inclinada_nunca_passa_o_horizonte() {
    let (mut cruzou_antes, mut total) = (0usize, 0usize);
    for (n_in, g) in amostras(200_000) {
        let n = normaliza(n_in);
        if n[2] == 0.0 || !n[2].is_finite() {
            continue;
        }
        total += 1;
        if sem_horizonte(n_in, g)[2] * n[2].signum() < 0.0 {
            cruzou_antes += 1;
        }
        let r = inclina(n_in, g);
        assert!(
            r[2] * n[2].signum() > 0.0,
            "a normal passou o horizonte: base {n:?} gradiente {g:?} ⇒ {r:?}"
        );
        let l = (r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt();
        assert!((l - 1.0).abs() < 1e-5, "a normal saiu com comprimento {l}");
    }
    assert!(
        cruzou_antes * 50 > total,
        "CONTROLO: só {cruzou_antes} de {total} cruzavam com a lei de antes"
    );
}

/// ⭐⭐ **GATE — Onde não há declive, a base sai AO BIT**, e acima do limiar a
/// normal inclinada não é tocada: a compressão só existe perto do horizonte.
#[test]
fn sem_declive_ou_acima_do_limiar_nada_muda() {
    let mut acima = 0usize;
    for (n_in, g) in amostras(50_000) {
        let n = normaliza(n_in);
        if !n[2].is_finite() {
            continue;
        }
        let r0 = inclina(n_in, [0.0; 3]);
        assert_eq!(
            r0.map(f32::to_bits),
            n.map(f32::to_bits),
            "sem declive mexeu na base"
        );
        if g == [0.0; 3] {
            continue;
        }
        let nb = sem_horizonte(n_in, g);
        if nb[2] * n[2].signum() >= n[2].abs().min(HORIZONTE_T) + 1e-4 {
            acima += 1;
            let r = inclina(n_in, g);
            assert!(
                dist(r, nb) < 1e-6,
                "acima do limiar a normal foi tocada: {nb:?} ⇒ {r:?}"
            );
        }
    }
    assert!(
        acima > 10_000,
        "a fixtura: {acima} amostras acima do limiar"
    );
}

/// ⭐ **GATE — a compressão TOCA o limiar sem vinco**: valor e declive iguais
/// dos dois lados de `z = t` (é o que impede uma linha nova na luz).
#[test]
fn a_compressao_toca_o_limiar_sem_vinco() {
    let n = base(0.15);
    let t = 0.15f32;
    let z_de = |z: f32| horizonte(n, [(1.0 - z * z).sqrt(), 0.0, z])[2];
    let h = 1e-3;
    let acima = (z_de(t + 2.0 * h) - z_de(t + h)) / h;
    let abaixo = (z_de(t - h) - z_de(t - 2.0 * h)) / h;
    assert!((z_de(t) - t).abs() < 1e-6);
    assert!(
        (acima - abaixo).abs() < 0.05,
        "o declive salta no limiar: {abaixo} contra {acima}"
    );
}
