//! ⭐ **SONDA A13 (2.ª ronda) — as leis de refino do bake contra o PADRÃO-OURO**, na fixtura dos
//! gates do desenho fiel ([`super::desenho_tests`]: a barra da cena, `8` nós, S e C). O bake é
//! chamado directamente (a gaveta do quadro guardaria o resultado da 1.ª lei).

use super::ondulacao_regua_tests::{DENSO, ideal_denso};
use super::ouro_reguas_tests::*;
use ph2d_vec_skin::curva::Refino;

/// Sem refino, e `k ∈ {2, 3}`.
fn leis() -> Vec<(String, Option<Refino>)> {
    let mut v = vec![("sem".to_string(), None)];
    for k in [2.0, 3.0] {
        v.push((format!("k{k}"), Some(Refino { k })));
    }
    v
}

/// O bake do PRODUTO sobre a fonte da fixtura (a contagem e a tolerância do `skin_desenho`).
fn assa(p: &BPalco, pele: &ph2d_skeleton::Skin, refino: Option<Refino>) -> ph2d_vec_scene::VecPath {
    let skin = p
        .sim
        .world()
        .get::<ph2d_skeleton_ecs::SkinBind>(p.alvo)
        .expect("pele")
        .clone();
    let g = crate::skinned_mesh::le(&skin.source).expect("fonte");
    let pesos = skin.pesos_do_quadro(if g.valida() { &g.pesos } else { &[] });
    let correcoes = skin.correcoes_resolvidas();
    let segs: usize = (0..g.path.contour_count())
        .filter_map(|c| g.path.contour(c))
        .map(|(v, f)| {
            if f {
                v.len()
            } else {
                v.len().saturating_sub(1)
            }
        })
        .sum();
    let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
    for v in g.path.verts_all() {
        for q in [v.anchor, v.in_handle, v.out_handle] {
            lo = [lo[0].min(q[0]), lo[1].min(q[1])];
            hi = [hi[0].max(q[0]), hi[1].max(q[1])];
        }
    }
    let campo = g.campo.as_ref();
    let indice = campo.and_then(|c| ph2d_vec_skin::pesos::IndiceDoCampo::novo(&c.malha));
    ph2d_vec_skin::curva::assa_a_pele(
        pele,
        &g.path,
        pesos,
        &correcoes,
        true,
        ph2d_vec_skin::curva::CampoIndexado {
            campo,
            indice: indice.as_ref(),
            suave: None,
        },
        ph2d_vec_skin::curva::Bake {
            amostras: crate::skin_desenho::amostras_por_segmento(segs),
            tolerancia: crate::skin_desenho::TOLERANCIA_DA_DIAGONAL
                * (hi[0] - lo[0]).hypot(hi[1] - lo[1]),
            refino,
        },
    )
}

/// ⭐ **SONDA — desenho → ouro (p90, máx: a régua dos gates) e ouro → desenho (máx: a tampa que
/// falta, A13), por lei e por dobra.**
#[test]
#[ignore = "sonda: imprime"]
fn diag_a13_o_refino_contra_o_ouro() {
    for (lado, s) in [("S", true), ("C", false)] {
        for graus in [90.0_f32, 140.0, 170.0] {
            let mut p = b_palco(false);
            p.reparte_com(1, false);
            p.lei_do_peso(false);
            if s {
                p.dobra_em_s(graus);
            } else {
                p.dobra(graus);
            }
            let pele = p.pele();
            let rest = b_amostra_com(&p.fonte, DENSO);
            let ouro = ideal_denso(&p, &pele, &rest, false);
            println!("  {lado} {graus:>4}°:");
            let mut v0: Option<Vec<[f64; 2]>> = None;
            for (nome, lei) in leis() {
                let d = assa(&p, &pele, lei);
                let dv = b_amostra_com(&d, DENSO);
                let (_, d90, dmax) = b_perfil(&dv, &ouro);
                let (_, _, omax) = b_perfil(&ouro, &dv);
                let igual = v0.as_ref().is_none_or(|a| a == &dv);
                v0.get_or_insert(dv);
                println!(
                    "    {nome:>10}: {:>3} nós · d→o p90 {d90:.5} máx {dmax:.5} · o→d máx {omax:.5}{}",
                    d.verts_all().count(),
                    if igual { " =V0" } else { "" }
                );
            }
        }
    }
}
