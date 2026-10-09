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
/// Disco opaco de 100 px ÷ 25 px (RC, relógio da placa, RTX 5060 Ti): com a saída antecipada
/// 1,34; sem ela 3,41 (medido no mesmo processo, 08/10, `load ~12–20`).
const RAZAO_INTERIOR: f64 = 2.0;

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

/// Um disco OPACO no centro do quadro, de raio `raio` (metros).
fn opaco(raio: f32) -> ph2d_light2d_ecs::CenaDeLuz {
    ph2d_light2d_ecs::CenaDeLuz {
        emissores: vec![crate::comum::disco(
            [2.56, 2.56],
            raio,
            [1.0; 3],
            f32::INFINITY,
        )],
        oclusores: vec![],
        ceu: [0.0; 3],
        ..Default::default()
    }
}

/// O custo do interior de um opaco — mesmo relógio, mesma faixa isolada. Um intervalo que COMEÇA
/// dentro de algo opaco devolve `(0, 0)` (exato: o raio que passa ali já foi absorvido); sem isso a
/// marcha anda o interior texel a texel e o custo segue a ÁREA do objeto (medido: um disco de
/// 100 px custava 6,4 ms contra 2,0 ms com a saída). A régua: o disco de 100 px contra o de 25 px,
/// mesmo centro.
#[test]
fn g4_o_interior_de_um_opaco_nao_custa() {
    let Some(gpu) = gpu() else { return };
    let (_, lado, texel) = DEGRAUS[0];
    let mut casos: Vec<Caso> = [("raio 25 px", 0.25), ("raio 100 px", 1.0)]
        .into_iter()
        .map(|(nome, r)| Caso {
            nome: nome.into(),
            solver: Solver::novo(&gpu, Config::padrao(grade(lado, texel))).expect("solver"),
            cena: opaco(r),
        })
        .collect();
    let carga = loadavg();
    let Some(r) = rodadas(&gpu, &mut casos, RODADAS) else {
        eprintln!("G4 interior: a placa não tem TIMESTAMP_QUERY — saltado");
        return;
    };
    let razao = r[1].rc[0] / r[0].rc[0];
    eprintln!(
        "G4 interior · perfil {} · loadavg {carga} · RC {:.4} ms (25 px) · {:.4} ms (100 px) · razão {razao:.2} (limite {RAZAO_INTERIOR})",
        perfil(),
        r[0].rc[0],
        r[1].rc[0]
    );
    assert!(
        razao <= RAZAO_INTERIOR,
        "o interior do opaco voltou a custar: razão {razao:.2}"
    );
}
