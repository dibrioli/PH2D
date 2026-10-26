//! ⏱ **A cena 6 a correr** (W4.5, `docs/BlenderConstraints/04_W4_cad.md` §5 — os números antes da mensagem): o pior
//! passo do arrasto de um canto pela caneta (60 passos num círculo de 0,4, ≈ 4 px a 100 px por unidade) e o pior
//! quadro da medida animada (resolver a partir do autorado), intercalados no MESMO processo, 7 rodadas com a ordem
//! rodada, resumo = o MÍNIMO · a mediana. Cada passo e cada quadro cumprem as regras. Sonda, não gate:
//! `bash scripts/ph2d-run.sh cargo test --profile smoke -p ph2d-app-constraints -- --ignored --nocapture a_regua_a_correr`.

use std::time::Instant;

use ph2d_app_constraints::smoke_regua::{self, MEDIDA};
use ph2d_ecs::canal_cad::{MEDIDA as CAMPO, chave};
use ph2d_ecs::{SimWorld, VecPathRef};
use ph2d_vec_constraints::{Folha, violacao};
use ph2d_vec_edit::PenTool;
use ph2d_vec_edit::snap::SnapResult;
use ph2d_vec_entities::medida::{self, ConducaoCad};
use ph2d_vec_scene::{VecScene, Xform};

const RODADAS: usize = 7;
const TOL: f64 = 1e-6;

fn ms(t: Instant) -> f64 {
    t.elapsed().as_secs_f64() * 1e3
}

fn viola(s: &VecScene, id: u64) -> f64 {
    violacao(
        &Folha::new([(s.path(id).unwrap(), Xform::IDENTITY)]),
        s.regras(),
    )
}

/// O canto de cima à direita do último dente, arrastado num círculo.
fn arrasto(base: &VecScene, id: u64) -> f64 {
    let mut s = base.clone();
    let mut pen = PenTool::new();
    let n = s.path(id).unwrap().verts.len();
    let em = s.path(id).unwrap().verts[n - 2].anchor;
    pen.on_press_node(&mut s, em, 1e-3, false);
    let mut pior: f64 = 0.0;
    for k in 1..=60 {
        let a = f64::from(k) / 60.0 * std::f64::consts::TAU;
        let alvo = [em[0] + 0.4 * (a.cos() - 1.0), em[1] + 0.4 * a.sin()];
        let t = Instant::now();
        assert!(pen.on_drag_encaixe(&mut s, alvo, &mut |_| SnapResult::default()));
        pior = pior.max(ms(t));
        assert!(viola(&s, id) <= TOL, "passo {k}: {:e}", viola(&s, id));
    }
    pen.on_release();
    pior
}

/// Os 49 quadros da medida animada pela curva (a interpolação das chaves feita à mão: só o solver se mede).
fn medida_animada(base: &VecScene, id: u64, regra: u32) -> f64 {
    let mut s = base.clone();
    let mut sim = SimWorld::new();
    let e = sim.world_mut().spawn(VecPathRef(id)).id().to_bits();
    let mut cond = ConducaoCad::default();
    let mut pior: f64 = 0.0;
    for q in 0..=48 {
        let (a, b) = if q <= 24 {
            (MEDIDA[0], MEDIDA[1])
        } else {
            (MEDIDA[1], MEDIDA[2])
        };
        #[expect(clippy::cast_precision_loss, reason = "quadros")]
        let u = (q - a.0) as f32 / (b.0 - a.0) as f32;
        let v = a.1 + (b.1 - a.1) * u;
        let t = Instant::now();
        medida::aplicar(
            &mut sim,
            &mut s,
            vec![(e, chave(CAMPO, regra), v)],
            &mut cond,
        );
        pior = pior.max(ms(t));
        cond.settle();
        assert!(viola(&s, id) <= TOL, "quadro {q}");
    }
    pior
}

fn resumo(v: &mut [f64]) -> String {
    v.sort_by(f64::total_cmp);
    format!("{:.3} · {:.3}", v[0], v[v.len() / 2])
}

#[test]
#[ignore = "sonda de velocidade: corre em --profile smoke, à mão"]
fn a_regua_a_correr() {
    let carga = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    let mut base = VecScene::new();
    let r = smoke_regua::semear(&mut base);
    let n = base.path(r.path).unwrap().verts.len();
    arrasto(&base, r.path);
    medida_animada(&base, r.path, r.medida.0);
    let (mut a, mut m) = (Vec::new(), Vec::new());
    for rodada in 0..RODADAS {
        if rodada % 2 == 0 {
            a.push(arrasto(&base, r.path));
            m.push(medida_animada(&base, r.path, r.medida.0));
        } else {
            m.push(medida_animada(&base, r.path, r.medida.0));
            a.push(arrasto(&base, r.path));
        }
    }
    println!("loadavg: {}", carga.trim());
    println!(
        "| cena 6 ({n} cantos, {} regras) | pior passo do arrasto (ms) | pior quadro da medida (ms) |",
        base.regras().len()
    );
    println!("|---|---|---|");
    println!(
        "| mínimo · mediana de {RODADAS} | {} | {} |",
        resumo(&mut a),
        resumo(&mut m)
    );
}
