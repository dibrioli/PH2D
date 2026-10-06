//! A13 (sonda) — a decisão «tapado» de um ponto do contorno em REPOUSO por três réguas: os
//! triângulos RECTOS do campo (a grossa), a malha fina do produto (`TOL_EM_LARGURAS`, F60) e uma
//! malha fina a `1 %` da largura (a quase exacta).

use super::super::{Arte, DIV_FIXO, Posada, TOL_FIXA};
use ph2d_skeleton::{Correccao, Skin};
use ph2d_vec_scene::VecPath;
use ph2d_vec_skin::pesos::{CampoDoDominio, IndiceDoCampo};

/// `(grossa, produto, quase exacta)` em cada ponto de repouso `pts` do contorno de `fonte`.
pub(crate) fn tapado_nas_tres(
    fonte: &VecPath,
    (campo, indice): (&CampoDoDominio, Option<&IndiceDoCampo>),
    (pele, correcoes): (&Skin, &[Correccao]),
    prof: &[f64],
    pts: &[[f64; 2]],
) -> Vec<(bool, bool, bool)> {
    let nova = || {
        let mut p =
            Posada::nova(campo, indice, pele, correcoes, true, prof).expect("a posada monta");
        p.avesso = false;
        p
    };
    let mut grossa = nova();
    grossa.arte = Arte::de(fonte);
    let produto = nova().com_a_arte(fonte);
    TOL_FIXA.with(|c| c.set(Some(0.01)));
    DIV_FIXO.with(|c| c.set(None));
    let exacta = nova().com_a_arte(fonte);
    TOL_FIXA.with(|c| c.set(None));
    pts.iter()
        .map(|&p| (grossa.tapado(p), produto.tapado(p), exacta.tapado(p)))
        .collect()
}

/// Quem tapa o ponto de repouso `p` (a malha fina do produto): o dono `(triângulo, virado, chave)`
/// e cada cobridor `(triângulo, virado, chave)`.
pub(crate) type Dono = (usize, bool, f64);

/// Ver [`Dono`].
pub(crate) fn quem_tapa(
    fonte: &VecPath,
    (campo, indice): (&CampoDoDominio, Option<&IndiceDoCampo>),
    (pele, correcoes): (&Skin, &[Correccao]),
    prof: &[f64],
    pts: &[[f64; 2]],
) -> Vec<Option<(Dono, Vec<Dono>)>> {
    let mut f = Posada::nova(campo, indice, pele, correcoes, true, prof).expect("posada");
    f.avesso = false;
    let f = f.com_a_arte(fonte);
    let fina = f.fina.as_ref().expect("fina");
    let m = &f.campo.malha;
    pts.iter()
        .map(|&p| {
            let (dono, q) = f.onde(p)?;
            let t = m.tris[dono];
            let minha = f.chave_tri[dono];
            let mut rascunho: Option<(Vec<f64>, Vec<f64>)> = None;
            let cob: Vec<Dono> = fina
                .grelha
                .balde(q)
                .iter()
                .map(|&k| k as usize)
                .filter(|&k| f.chave_tri[k] > minha && !m.tris[k].iter().any(|v| t.contains(v)))
                .filter(|&k| {
                    fina.cobre(k, q, |k, u, v| {
                        let (linha, w) = rascunho
                            .get_or_insert_with(|| (vec![0.0; f.campo.ossos()], f.pele.scratch()));
                        f.posa_no(k, u, v, linha, w)
                    })
                    .is_some_and(|r| f.arte.tem(r))
                })
                .map(|k| (k, f.virado[k], f.chave_tri[k]))
                .collect();
            Some(((dono, f.virado[dono], minha), cob))
        })
        .collect()
}
