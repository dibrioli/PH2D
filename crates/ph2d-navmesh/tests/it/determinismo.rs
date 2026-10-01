//! ⭐ **A MESMA MALHA EM DOIS PROCESSOS** (plano 30 §2.6, a medição da W0). O `spade` usa um
//! `HashSet` dentro do carregamento em lote, e a semente de um `HashSet` do `std` muda POR PROCESSO
//! ⇒ correr duas vezes no mesmo processo não provava nada. Este gate relança o próprio binário de
//! teste como FILHO, que imprime a impressão digital da malha, e compara-a com a do pai.

use ph2d_navmesh::{Params, build};

use crate::cena::{Lcg, obstaculos, retangulo};

const FILHO: &str = "PH2D_NAVMESH_IMPRESSAO_FILHO";

/// A impressão digital: FNV-1a sobre os bits de cada vértice e os índices de cada polígono.
fn impressao() -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut mix = |x: u64| {
        for b in x.to_le_bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
    };
    for seed in [3u64, 7, 11] {
        let mut rng = Lcg(seed);
        let obs = obstaculos(&mut rng, 40, 30.0, 20.0);
        let b = build(
            &retangulo(30.0, 20.0),
            &obs,
            &Params {
                agent_radius: 0.4,
                ..Params::default()
            },
        )
        .expect("constrói");
        for v in b.mesh.verts() {
            mix(v[0].to_bits());
            mix(v[1].to_bits());
        }
        for p in b.mesh.polys() {
            mix(p.verts.len() as u64);
            for &v in &p.verts {
                mix(v as u64);
            }
        }
    }
    h
}

#[test]
fn a_malha_e_a_mesma_ao_bit_noutro_processo() {
    let aqui = impressao();
    if std::env::var_os(FILHO).is_some() {
        println!("IMPRESSAO={aqui:016x}");
        return;
    }
    let exe = std::env::current_exe().expect("o binário de teste");
    let saida = std::process::Command::new(exe)
        .args([
            "--exact",
            "determinismo::a_malha_e_a_mesma_ao_bit_noutro_processo",
            "--nocapture",
            "--test-threads=1",
        ])
        .env(FILHO, "1")
        .output()
        .expect("o filho corre");
    let texto = String::from_utf8_lossy(&saida.stdout);
    let linha = texto
        .lines()
        .find_map(|l| l.split("IMPRESSAO=").nth(1))
        .unwrap_or_else(|| {
            panic!(
                "o filho não imprimiu a impressão digital:\n{texto}\n{}",
                String::from_utf8_lossy(&saida.stderr)
            )
        });
    assert_eq!(
        linha.trim(),
        format!("{aqui:016x}"),
        "a malha mudou de um processo para o outro"
    );
}
