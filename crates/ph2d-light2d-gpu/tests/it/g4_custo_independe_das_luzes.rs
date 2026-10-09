//! **G4 — gate de RELÓGIO** (faixa isolada do nextest; `docs/DevOps/FLAKES_DE_CARGA.md`). O custo
//! do solver (C1 + C2, relógio da placa) com 1 · 10 · 100 · 1 000 · 10 000 LUZES na MESMA geometria
//! (`cenas::enxame_de_luzes`: 10 000 formas, as que não emitem são oclusores pretos) varia ≤ 5 % —
//! o kill-criterion K3 do plano: acima disso algo no caminho depende do NÚMERO de luzes e a W2 não
//! fecha. A captura (C0) é medida e impressa ao lado.
//!
//! ⚠️ A dependência da GEOMETRIA (a marcha por campo de distância anda mais onde há mais bordas) é
//! outro eixo — a S3 do relatório — e a sonda `medida_luz2d` mede-a com o `enxame` de área emissora
//! fixa. Ela não entra neste gate; está no handoff da W2 com os números.

use ph2d_light2d_gpu::{Config, Solver};

use crate::cenas::{DEGRAUS, enxame_de_luzes};
use crate::comum::{gpu, grade};
use crate::medicao::{Caso, loadavg, perfil, rodadas};

/// K3 (plano §7; relatório, critérios de abandono).
const K3: f64 = 0.05;
const CONTAGENS: [u32; 5] = [1, 10, 100, 1_000, 10_000];
const RODADAS: usize = 15;

#[test]
fn g4_o_custo_do_rc_nao_depende_do_numero_de_luzes() {
    let Some(gpu) = gpu() else { return };
    let (_, lado, texel) = DEGRAUS[0];
    let mut casos: Vec<Caso> = CONTAGENS
        .iter()
        .map(|&n| Caso {
            nome: format!("{n} luzes"),
            solver: Solver::novo(&gpu, Config::padrao(grade(lado, texel))).expect("solver"),
            cena: enxame_de_luzes(n),
        })
        .collect();
    for c in &casos {
        let fora = ph2d_light2d_gpu::o_que_fica_fora(&c.cena, &c.solver.config().grade);
        assert!(
            fora.fora.is_empty(),
            "{}: o enxame entra inteiro na C0 ({} fora)",
            c.nome,
            fora.fora.len()
        );
    }
    let carga = loadavg();
    let Some(r) = rodadas(&gpu, &mut casos, RODADAS) else {
        eprintln!("G4: a placa não tem TIMESTAMP_QUERY — saltado");
        return;
    };
    eprintln!("G4 · perfil {} · loadavg {carga} → {}", perfil(), loadavg());
    eprintln!("G4 | luzes | C0 ms (mín · med) | RC = C1 + C2 ms (mín · med)");
    for x in &r {
        eprintln!(
            "G4 | {} | {:.4} · {:.4} | {:.4} · {:.4}",
            x.nome, x.c0[0], x.c0[1], x.rc[0], x.rc[1]
        );
    }
    let rc: Vec<f64> = r.iter().map(|x| x.rc[0]).collect();
    let (min, max) = rc
        .iter()
        .fold((f64::INFINITY, 0.0_f64), |(a, b), &v| (a.min(v), b.max(v)));
    let variacao = (max - min) / min;
    eprintln!(
        "G4 · K3: variação do RC entre 1 e 10 000 luzes = {:.2} % (limite {:.0} %)",
        100.0 * variacao,
        100.0 * K3
    );
    assert!(
        variacao <= K3,
        "K3: o RC varia {:.2} % com o número de luzes",
        100.0 * variacao
    );
}
