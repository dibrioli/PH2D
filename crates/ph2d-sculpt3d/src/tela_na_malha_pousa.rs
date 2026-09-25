//! ⭐⭐⭐ **O DEPÓSITO da tela na peça** — a lei por amostra e o percurso da
//! retícula de [`super::TelaNaMalha`]. Filho de `tela_na_malha.rs`, que é a
//! VISTA e a OCLUSÃO; o cabeçalho de lá explica as duas misturas.

use ph2d_mesh::Mesh;

use super::{Mistura, Rectangulo, Tela, TelaNaMalha, combina};
use crate::SculptStroke;
use crate::preenche::keep_da_amostra;

/// A mistura de uma amostra — ver o cabeçalho.
fn pousa(base: [f32; 3], mistura: Mistura, k: f32) -> [f32; 3] {
    match mistura {
        Mistura::Sobre { pm, a } => {
            let fica = 1.0 - a * k;
            [
                base[0] * fica + pm[0] * k,
                base[1] * fica + pm[1] * k,
                base[2] * fica + pm[2] * k,
            ]
        }
        Mistura::Diferenca(d) => [
            (base[0] + d[0] * k).clamp(0.0, 1.0),
            (base[1] + d[1] * k).clamp(0.0, 1.0),
            (base[2] + d[2] * k).clamp(0.0, 1.0),
        ],
    }
}

/// O lado, em amostras, de um bloco da retícula de um quad que se testa
/// contra a caixa do traço de uma vez ([`SculptStroke::pousa_a_tela`]).
const BLOCO: u32 = 16;

/// O ponto de ecrã `s` cai na caixa do traço? — UMA regra para os dois
/// destinos (vértice e amostra), senão os dois limites divergem em silêncio.
fn na_caixa(caixa: [f32; 4], s: [f32; 2]) -> bool {
    s[0] >= caixa[0] && s[0] <= caixa[2] && s[1] >= caixa[1] && s[1] <= caixa[3]
}

/// ⭐ **Uma amostra do plano recebe a tela** — o corpo que os dois percursos da
/// retícula (triângulo e quad) partilham. Devolve se ela MUDOU.
#[allow(clippy::too_many_arguments)]
fn pousa_amostra(
    sessao: &mut TelaNaMalha,
    fina: &mut crate::tinta_fina::TintaDoTraco,
    mesh: &Mesh,
    tela: &Tela<'_>,
    caixa: [f32; 4],
    fi: u32,
    cantos: &[u32],
    idx: u32,
    w: &[f32],
    m: &[f32],
) -> bool {
    if sessao.repetida(idx) {
        return false;
    }
    let p = combina(mesh.positions(), cantos, w);
    sessao.projetadas += 1;
    let Some(s) = sessao.vista.ecra(p) else {
        return false;
    };
    if !na_caixa(caixa, s) {
        return false;
    }
    let (mistura, vazia) = sessao.leitura(tela, s);
    if vazia && !fina.tocou(idx) {
        return false;
    }
    if !sessao.ve_se_no_pixel(mesh, fi, s, idx, p) {
        return false;
    }
    let k = keep_da_amostra(w, m);
    fina.repinta(idx, |base| pousa(base, mistura, k))
}

impl SculptStroke {
    /// ⭐⭐⭐ **POUSA A TELA NA PEÇA** dentro do rectângulo `r` — no plano de
    /// tinta fina quando o traço o tem emprestado, senão na cor por vértice.
    ///
    /// Devolve os VÉRTICES cuja cor mudou (para o upload incremental; vazio no
    /// caminho do plano, que sobe pelas amostras sujas do próprio empréstimo) e
    /// quantas amostras mudaram.
    ///
    /// ⚠️ O traço tem de ter sido aberto (`begin`) sobre esta malha.
    pub fn pousa_a_tela(
        &mut self,
        mesh: &mut Mesh,
        sessao: &mut TelaNaMalha,
        tela: &Tela<'_>,
        r: Rectangulo,
    ) -> (Vec<u32>, usize) {
        // ⚠️ **Um píxel de folga à volta**: a amostragem é bilinear, logo uma
        // amostra até um píxel fora do rectângulo mudado lê um píxel de dentro.
        let caixa = [
            r[0] as f32 - 1.0,
            r[1] as f32 - 1.0,
            r[0].saturating_add(r[2]) as f32 + 1.0,
            r[1].saturating_add(r[3]) as f32 + 1.0,
        ];
        sessao.proxima_epoca();
        let faces = sessao.faces_em(caixa);
        let mut mudaram = 0usize;
        let mut vertices = Vec::new();
        for fi in faces {
            let face = mesh.faces()[fi as usize];
            let cantos = face.verts();
            let n = cantos.len();
            let mut m = [0.0f32; 4];
            for (mk, &v) in m.iter_mut().zip(cantos) {
                *mk = mesh
                    .masks()
                    .map_or(ph2d_mesh::DEFAULT_MASK, |k| k[v as usize]);
            }
            if let Some(fina) = self.tinta_fina.as_mut() {
                let lado = fina.tinta().lado_da_face(fi as usize);
                let ld = lado as f32;
                if n == 3 {
                    let mut pedidas: Vec<(u32, [f32; 4])> = Vec::new();
                    fina.tinta()
                        .para_cada_amostra_tri(fi as usize, cantos, |idx, (i, j, k)| {
                            pedidas.push((idx, [i as f32 / ld, j as f32 / ld, k as f32 / ld, 0.0]));
                        });
                    for (idx, w4) in pedidas {
                        if pousa_amostra(
                            sessao,
                            fina,
                            mesh,
                            tela,
                            caixa,
                            fi,
                            cantos,
                            idx,
                            &w4[..3],
                            &m[..3],
                        ) {
                            mudaram += 1;
                        }
                    }
                } else {
                    // ⭐ **Blocos da retícula que a caixa não alcança nem se
                    // projectam.** Um retalho bilinear fica dentro do fecho
                    // convexo dos quatro cantos dele, e a projecção perspectiva
                    // preserva esse fecho para pontos à frente da câmera ⇒ a
                    // caixa dos quatro cantos projectados LIMITA o bloco inteiro.
                    // A `256x` uma face tem `65 536` amostras e um traço toca
                    // poucas: projectá-las todas era o custo que sobrava.
                    let pos = mesh.positions();
                    let mut b0 = 0u32;
                    loop {
                        let j1 = (b0 + BLOCO).min(lado);
                        let mut a0 = 0u32;
                        loop {
                            let i1 = (a0 + BLOCO).min(lado);
                            let alcanca = [(a0, b0), (i1, b0), (i1, j1), (a0, j1)]
                                .iter()
                                .try_fold(
                                    [
                                        f32::INFINITY,
                                        f32::INFINITY,
                                        f32::NEG_INFINITY,
                                        f32::NEG_INFINITY,
                                    ],
                                    |b, &(i, j)| {
                                        let w = crate::tinta_fina::bilinear(
                                            i as f32 / ld,
                                            j as f32 / ld,
                                        );
                                        let q = sessao.vista.ecra(combina(pos, cantos, &w))?;
                                        Some([
                                            b[0].min(q[0]),
                                            b[1].min(q[1]),
                                            b[2].max(q[0]),
                                            b[3].max(q[1]),
                                        ])
                                    },
                                )
                                .is_none_or(|b| {
                                    b[2] >= caixa[0]
                                        && b[0] <= caixa[2]
                                        && b[3] >= caixa[1]
                                        && b[1] <= caixa[3]
                                });
                            if alcanca {
                                for j in b0..=j1 {
                                    for i in a0..=i1 {
                                        // As bordas partilhadas entre blocos
                                        // repetem-se; a marca da pousada salta-as.
                                        let idx =
                                            fina.tinta().indice_quad(fi as usize, cantos, i, j);
                                        let w4 = crate::tinta_fina::bilinear(
                                            i as f32 / ld,
                                            j as f32 / ld,
                                        );
                                        if pousa_amostra(
                                            sessao, fina, mesh, tela, caixa, fi, cantos, idx, &w4,
                                            &m,
                                        ) {
                                            mudaram += 1;
                                        }
                                    }
                                }
                            }
                            if i1 >= lado {
                                break;
                            }
                            a0 = i1;
                        }
                        if j1 >= lado {
                            break;
                        }
                        b0 = j1;
                    }
                }
            } else {
                for (c, &v) in cantos.iter().enumerate() {
                    if sessao.repetida(v) {
                        continue;
                    }
                    let Some(s) = sessao.ecra[v as usize] else {
                        continue;
                    };
                    if !na_caixa(caixa, s) {
                        continue;
                    }
                    let (mistura, vazia) = sessao.leitura(tela, s);
                    if vazia && !self.tocou_vertice(v) {
                        continue;
                    }
                    let p = mesh.positions()[v as usize];
                    if !sessao.ve_se(mesh, v, p) {
                        continue;
                    }
                    let k = keep_da_amostra(&[1.0], &m[c..=c]);
                    if self.repinta_vertice(mesh, v, |base| pousa(base, mistura, k)) {
                        mudaram += 1;
                        vertices.push(v);
                    }
                }
            }
        }
        (vertices, mudaram)
    }
}
