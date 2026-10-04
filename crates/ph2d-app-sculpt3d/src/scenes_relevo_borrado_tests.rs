//! A cena `=56` tem o FENÓMENO: o Gaussiano está por cima das camadas com relevo, nasce a raio `0`
//! (a peça nítida) e o raio amacia as riscas (a dobra da CPU, a referência).

use super::poe_o_desfoque;
use crate::pilha_da_peca::PilhaDaPeca;
use crate::scenes::relevo_camadas::{NIVEL, faixa, relevo_da_base};
use ph2d_tool_painter::{AdjustmentParams, GaussianBlurParams, LayerId, LayerKind};

fn bits(r: &[[f32; 2]]) -> Vec<[u32; 2]> {
    r.iter().map(|x| x.map(f32::to_bits)).collect()
}

/// O raio do Gaussiano do topo, em fracção do curso.
fn raio(p: &mut PilhaDaPeca, mesh: &ph2d_mesh::Mesh, frac: f32) -> LayerId {
    let topo = p.pilha().root()[0];
    let ph2d_tool_painter::SpatialUnits::Surface { size } =
        crate::vizinhanca_da_peca::unidades(mesh)
    else {
        panic!("unidades da peça")
    };
    let mut m = p.pilha().clone();
    m.adjustment_mut(topo).expect("ajuste").params =
        AdjustmentParams::GaussianBlur(GaussianBlurParams {
            radius: frac * ph2d_tool_painter::SURFACE_RADIUS_MAX * size,
        });
    p.troca_metadado(m).expect("o raio");
    topo
}

/// ⭐ **O Gaussiano da cena nasce a raio `0`: a peça das duas camadas fica ao bit** — e no fim do
/// curso as riscas (numa pilha só com elas) amaciam: medido `24 %` da variância (perto dos pólos as
/// riscas de `y` são largas na superfície e o raio não as alcança).
#[test]
fn o_desfoque_da_cena_nasce_nitido_e_amacia_as_riscas() {
    let mesh = crate::scenes::tinta_fina::peca();
    let faces = || mesh.faces().iter().map(ph2d_mesh::Face::verts);
    let tinta = ph2d_mesh_colors::Tinta::nova(mesh.vert_count(), faces(), NIVEL);
    let xs = crate::vizinhanca_da_peca::posicoes(&tinta, &mesh);
    let n = xs.len();
    let riscas: Vec<[f32; 2]> = xs.iter().map(|&x| relevo_da_base(x)).collect();
    let (cor, lomba): (Vec<[u8; 4]>, Vec<[f32; 2]>) = xs.iter().map(|&x| faixa(x)).unzip();

    let mut p = PilhaDaPeca::de_tinta(&tinta);
    let base = p.base().expect("base");
    assert!(p.pinta_camada(base, &vec![[200; 4]; n], Some(riscas.clone())));
    let cima = p.nova_camada("Layer 2").expect("cima");
    assert!(p.pinta_camada(cima, &cor, Some(lomba)));
    let nitido = p.relevo_composto().expect("relevo");
    poe_o_desfoque(&mut p, &mesh);
    let topo = p.pilha().root()[0];
    let Some(LayerKind::Adjustment(a)) = p.pilha().get(topo).map(|c| c.kind.clone()) else {
        panic!("o topo é o Gaussiano")
    };
    assert!(matches!(a.params, AdjustmentParams::GaussianBlur(g) if g.radius == 0.0));
    assert!(
        !p.relevo_atraves(),
        "a raio 0 o relevo não passa pelo desfoque"
    );
    assert_eq!(bits(&p.relevo_composto().expect("r")), bits(&nitido));

    // As riscas sozinhas, para a régua não ler a lomba borrada a espalhar-se.
    let mut q = PilhaDaPeca::de_tinta(&tinta);
    let base = q.base().expect("base");
    assert!(q.pinta_camada(base, &vec![[200; 4]; n], Some(riscas)));
    poe_o_desfoque(&mut q, &mesh);
    let antes = q.relevo_composto().expect("relevo");
    raio(&mut q, &mesh, 1.0);
    q.garante_vizinhanca(&tinta, &mesh);
    assert!(q.relevo_atraves());
    let macio = q.relevo_composto().expect("relevo");
    let var = |r: &[[f32; 2]]| {
        let m = r.iter().map(|x| f64::from(x[0])).sum::<f64>() / r.len() as f64;
        r.iter().map(|x| (f64::from(x[0]) - m).powi(2)).sum::<f64>()
    };
    assert!(
        var(&macio) < 0.35 * var(&antes),
        "as riscas não amaciaram: {} → {}",
        var(&antes),
        var(&macio)
    );
}
