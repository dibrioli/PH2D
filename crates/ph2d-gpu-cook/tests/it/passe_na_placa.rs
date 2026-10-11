//! ⭐⭐⭐ **O PASSE DE SEPARAÇÃO NA PLACA × a CPU, por passo** (doc 130 §6).
//!
//! O oráculo é `ph2d_contact::passe::separa_com_relatorio`; a placa corre a MESMA corrente a
//! `1, 2, 4, 8` varreduras e lê de volta `P`, `rot` e as varreduras. A tolerância é relativa ao
//! `alcance_max` da fixtura e está MEDIDA (ver [`TOL`]).

use ph2d_contact::passe::separa_com_relatorio;
use ph2d_gpu_cook::GpuCook;
use ph2d_nodegraph::attr::{
    COLLIDER_BOX_COLUMN, COLLIDER_COLUMN, COLLIDER_OFFSET_COLUMN, Column, INV_INERTIA_COLUMN, Stream,
};

use crate::placa::try_headless_gpu;

/// O desvio máximo admitido, em fracção do `alcance_max` (posição) e em graus (`rot`). MEDIDO em
/// 11/10 (RTX, as sete fixturas a `1, 2, 4, 8`): o pior foi `2,84e-6` (posição, a nuvem mista) e
/// `1,07e-4°` (`rot`) — ULPs do `sqrt`/FMA, a ordem de soma é a mesma. A folga é ~4× e ~9×.
const TOL: f32 = 1e-5;
const TOL_GRAUS: f32 = 1e-3;
const PASSOS: [u32; 4] = [1, 2, 4, 8];

fn corrente(p: Vec<[f32; 2]>) -> Stream {
    let mut s = Stream::new(p.len());
    s.set("P", Column::Vec2(p));
    s
}

fn esc(s: &mut Stream, nome: &str, v: Vec<f32>) {
    s.set(nome, Column::Scalar(v));
}

fn par(s: &mut Stream, nome: &str, v: Vec<[f32; 2]>) {
    s.set(nome, Column::Vec2(v));
}

/// Um gerador fixo (LCG), para a nuvem de peças mistas.
fn lcg(semente: &mut u32) -> f32 {
    *semente = semente.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    (*semente >> 8) as f32 / 16_777_216.0
}

fn fixturas() -> Vec<(&'static str, Stream)> {
    let mut v = Vec::new();
    // Discos em fila, sobrepostos.
    let mut s = corrente((0..6).map(|i| [i as f32 * 0.6, 0.05 * (i % 2) as f32]).collect());
    esc(&mut s, COLLIDER_COLUMN, vec![0.5; 6]);
    v.push(("discos", s));
    // Caixas alinhadas empilhadas: o TRECHO de dois pontos.
    let mut s = corrente((0..5).map(|i| [0.1 * i as f32, i as f32 * 0.8]).collect());
    par(&mut s, COLLIDER_BOX_COLUMN, vec![[0.5, 0.5]; 5]);
    v.push(("caixas_empilhadas", s));
    // Caixas rodadas, com desvio e tamanho — a forma rodada e o centro deslocado.
    let mut s = corrente(vec![[0.0, 0.0], [0.7, 0.2], [1.3, -0.1], [0.4, 0.9]]);
    par(&mut s, COLLIDER_BOX_COLUMN, vec![[0.5, 0.3]; 4]);
    par(&mut s, COLLIDER_OFFSET_COLUMN, vec![[0.1, 0.0], [0.0, 0.0], [-0.05, 0.1], [0.0, 0.2]]);
    par(&mut s, "size", vec![[1.0, 1.0], [1.2, 0.8], [0.9, 0.9], [-1.0, 1.0]]);
    esc(&mut s, "rot", vec![0.0, 30.0, 90.0, 210.0]);
    v.push(("caixas_rodadas", s));
    // Coincidentes: o desempate por paridade do par.
    let mut s = corrente(vec![[0.0, 0.0], [0.0, 0.0], [2.0, 2.0], [2.0, 2.0], [2.0, 2.0]]);
    esc(&mut s, COLLIDER_COLUMN, vec![0.5, 0.5, 0.0, 0.0, 0.0]);
    par(&mut s, COLLIDER_BOX_COLUMN, vec![[0.0, 0.0], [0.0, 0.0], [0.4, 0.4], [0.4, 0.4], [0.4, 0.4]]);
    v.push(("coincidentes", s));
    // Obstáculo (`inv_mass = 0`), inércia por coluna e `falloff`.
    let mut s = corrente((0..6).map(|i| [i as f32 * 0.5, 0.1 * i as f32]).collect());
    esc(&mut s, COLLIDER_COLUMN, vec![0.4, 0.4, 0.4, 0.0, 0.4, 0.4]);
    par(&mut s, COLLIDER_BOX_COLUMN, vec![[0.0, 0.0], [0.0, 0.0], [0.0, 0.0], [0.3, 0.2], [0.0, 0.0], [0.0, 0.0]]);
    esc(&mut s, "inv_mass", vec![1.0, 0.0, 2.0, 1.0, f32::NAN, 0.5]);
    esc(&mut s, INV_INERTIA_COLUMN, vec![f32::NAN, 0.0, 3.0, f32::NAN, 1.0, f32::NAN]);
    esc(&mut s, "falloff", vec![1.0, 1.0, 0.3, 0.0, 1.5, 0.7]);
    v.push(("obstaculo_inercia_falloff", s));
    // O filtro do repouso: o mesmo corpo junto no REPOUSO não colide.
    let mut s = corrente((0..6).map(|i| [i as f32 * 0.5, 0.0]).collect());
    esc(&mut s, COLLIDER_COLUMN, vec![0.4; 6]);
    par(&mut s, "rest", (0..6).map(|i| [i as f32 * 0.5, 0.0]).collect());
    esc(&mut s, "body", vec![1.0, 1.0, 1.0, 2.0, 2.0, 0.0]);
    v.push(("ligados", s));
    // A nuvem mista: 300 peças (a CPU parte em blocos a partir de 128), discos e caixas.
    let mut semente = 7u32;
    let n = 300;
    let p: Vec<[f32; 2]> = (0..n).map(|_| [lcg(&mut semente) * 8.0, lcg(&mut semente) * 8.0]).collect();
    let mut s = corrente(p);
    let caixa: Vec<[f32; 2]> = (0..n)
        .map(|i| if i % 3 == 0 { [0.2 + 0.1 * lcg(&mut semente), 0.15] } else { [0.0, 0.0] })
        .collect();
    par(&mut s, COLLIDER_BOX_COLUMN, caixa);
    esc(&mut s, COLLIDER_COLUMN, (0..n).map(|_| 0.15 + 0.1 * lcg(&mut semente)).collect());
    esc(&mut s, "rot", (0..n).map(|_| 360.0 * lcg(&mut semente)).collect());
    v.push(("nuvem_mista", s));
    v
}

fn coluna_p(s: &Stream) -> Vec<[f32; 2]> {
    match s.get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => panic!("sem P"),
    }
}

fn coluna_rot(s: &Stream) -> Vec<f32> {
    match s.get("rot") {
        Some(Column::Scalar(v)) => v.clone(),
        _ => vec![0.0; s.count()],
    }
}

fn alcance_max(s: &Stream) -> f32 {
    ph2d_contact::colisores(s)
        .expect("a fixtura declara")
        .iter()
        .flatten()
        .map(ph2d_contact::Colisor::alcance)
        .fold(0.0, f32::max)
}

/// O maior desvio entre os dois lados, por fixtura e por passo — impresso, para que a tolerância
/// seja uma MEDIÇÃO e não um palpite.
#[test]
fn o_passe_na_placa_segue_a_cpu_por_passo() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("sem placa: o gate não corre aqui");
        return;
    };
    let mut cook = GpuCook::default();
    for (nome, s) in fixturas() {
        let alc = alcance_max(&s);
        for v in PASSOS {
            let (cpu, rel) = separa_com_relatorio(&s, v as usize);
            let cpu = cpu.unwrap_or_else(|| s.clone());
            let (gp, gr, gv) = cook
                .separa_e_le(&gpu, &s, v, ph2d_contact::REPOUSO_VISIVEL)
                .unwrap_or_else(|| panic!("{nome}: a placa recusou um passe que a CPU corre"));
            let (cp, cr) = (coluna_p(&cpu), coluna_rot(&cpu));
            let mut pior = 0.0_f32;
            let mut pior_rot = 0.0_f32;
            for i in 0..s.count() {
                let (a, b) = (cp[i], gp[i]);
                if a[0].is_nan() {
                    assert!(b[0].is_nan(), "{nome}[{i}] v={v}: NaN na CPU, {b:?} na placa");
                    continue;
                }
                pior = pior.max((a[0] - b[0]).abs().max((a[1] - b[1]).abs()));
                pior_rot = pior_rot.max((cr[i] - gr[i]).abs());
            }
            eprintln!(
                "{nome:<28} v={v}: |ΔP|/alc = {:.2e} · |Δrot| = {pior_rot:.2e}° · varreduras CPU {} placa {gv}",
                pior / alc,
                rel.varreduras,
            );
            assert!(pior <= TOL * alc, "{nome} v={v}: |ΔP| = {pior} > {TOL}·{alc}");
            assert!(pior_rot <= TOL_GRAUS, "{nome} v={v}: |Δrot| = {pior_rot}°");
            // Medido: iguais em todas as fixturas (a paragem cai do mesmo lado).
            assert_eq!(rel.varreduras, gv as usize, "{nome} v={v}: varreduras CPU × placa");
        }
    }
}

/// ⭐ **E a separação EXISTE** — controlo de que o gate acima não compara duas cópias da entrada:
/// a fixtura dos discos sai da placa com as peças afastadas.
#[test]
fn a_placa_de_facto_afasta_os_discos() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let (_, s) = fixturas().remove(0);
    let (gp, _, gv) = GpuCook::default()
        .separa_e_le(&gpu, &s, 8, ph2d_contact::REPOUSO_VISIVEL)
        .expect("corre");
    let antes = coluna_p(&s);
    let mexeu = antes.iter().zip(&gp).filter(|(a, b)| (a[0] - b[0]).abs() > 1e-3).count();
    assert!(mexeu >= 4 && gv >= 1, "{mexeu} discos mexeram em {gv} varreduras");
}

/// Sem coluna de colisor ⇒ nenhum passe — a mesma resposta (`None`) que a CPU dá.
#[test]
fn sem_colisor_a_placa_nao_corre_o_passe() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let s = corrente(vec![[0.0, 0.0], [0.1, 0.0]]);
    assert!(separa_com_relatorio(&s, 8).0.is_none());
    assert!(GpuCook::default().separa_e_le(&gpu, &s, 8, 1e-5).is_none());
}

/// Os nove módulos do passe validam no `naga` sem placa (a CI não tem uma).
#[test]
fn os_modulos_do_passe_validam_sem_placa() {
    for (nome, src) in ph2d_gpu_cook::passe_na_placa::fontes_wgsl() {
        let m = naga::front::wgsl::parse_str(&src).unwrap_or_else(|e| panic!("{nome}: {e}"));
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::empty(),
        )
        .validate(&m)
        .unwrap_or_else(|e| panic!("{nome}: {e:?}"));
    }
}

/// Sonda (não afirma nada): o custo do passe a `8` varreduras, CPU × placa, mínimo de 5 corridas
/// intercaladas. ⚠️ A placa inclui o envio e a leitura de volta de `P`/`rot` (o produto não lê).
/// `cargo test --release -p ph2d-gpu-cook --test it measure_o_passe -- --ignored --nocapture`
#[test]
#[ignore = "sonda: mede, não julga"]
fn measure_o_passe_cpu_contra_placa() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let mut cook = GpuCook::default();
    for n in [1_000usize, 10_000, 32_761] {
        let lado = (n as f32).sqrt() * 0.42;
        let mut semente = 11u32;
        let p: Vec<[f32; 2]> = (0..n).map(|_| [lcg(&mut semente) * lado, lcg(&mut semente) * lado]).collect();
        let mut s = corrente(p);
        let caixa: Vec<[f32; 2]> = (0..n).map(|i| if i % 2 == 0 { [0.25, 0.2] } else { [0.0, 0.0] }).collect();
        par(&mut s, COLLIDER_BOX_COLUMN, caixa);
        esc(&mut s, COLLIDER_COLUMN, vec![0.25; n]);
        let (mut cpu, mut placa) = (f64::MAX, f64::MAX);
        for _ in 0..5 {
            let t = std::time::Instant::now();
            let r = separa_com_relatorio(&s, 8);
            cpu = cpu.min(t.elapsed().as_secs_f64());
            std::hint::black_box(r);
            let t = std::time::Instant::now();
            let g = cook.separa_e_le(&gpu, &s, 8, ph2d_contact::REPOUSO_VISIVEL);
            placa = placa.min(t.elapsed().as_secs_f64());
            std::hint::black_box(g);
        }
        eprintln!("n = {n:>6}: CPU {:>8.2} ms · placa {:>7.2} ms · {:.1}×", cpu * 1e3, placa * 1e3, cpu / placa);
    }
}
