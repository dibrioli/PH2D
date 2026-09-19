//! ⭐⭐⭐⭐ **A GRADE POR BANDA — a régua que diz ONDE, e não só QUANTO.**
//!
//! ⚠️ Ela saiu da [`super::medida_do_pente`] por **TECTO DE LOC** (`752` contra
//! `700`), e o corte é por responsabilidade: ali fica *«que fracção da faixa
//! segue a grade»*, aqui fica *«e em que PARTE da faixa»*.

use ph2d_mesh::Mesh;

use super::{comprimento, direccao_do_troco, sub};

/// ⭐⭐⭐⭐ **A GRADE POR BANDA DE DISTÂNCIA AO PERCURSO — a régua que diz ONDE.**
///
/// ⚠️⚠️ **A [`grade_da_faixa`] agrega a faixa INTEIRA**, e quando o dono diz
/// *«várias áreas ainda não muito boas»* ela não sabe dizer QUAIS. É a mesma
/// forma que este módulo já pagou quatro vezes — o `edge_max` global cego ao
/// quad fino, o `χ` cego à almofada, as três réguas da ponta a deitarem fora o
/// índice antes de devolver. *Uma régua que conta QUANTOS nunca vê QUAIS.*
///
/// Reparte as arestas da faixa por **distância ao percurso, em raios do
/// pincel**, e devolve `(alinhadas, total)` por banda — `bandas` fatias iguais
/// de `0` a `1` raio, com a última a apanhar tudo o que sobra.
///
/// ⚠️ **A distância é ao PERCURSO e não ao centro do último dab**: o que
/// interessa é quantas vezes aquele pedaço de malha foi tocado pelo traço, e
/// isso é função da distância à linha que a mão andou.
#[must_use]
pub fn grade_por_banda(
    malha: &Mesh,
    percurso: &[[f32; 3]],
    raio: f32,
    bandas: usize,
) -> Vec<(usize, usize)> {
    let bandas = bandas.max(1);
    let mut out = vec![(0usize, 0usize); bandas];
    if percurso.is_empty() || raio <= 0.0 {
        return out;
    }
    let pos = malha.positions();
    let mut vistas = std::collections::BTreeSet::new();
    for f in malha.faces() {
        let vs = f.verts();
        for k in 0..vs.len() {
            let (a, b) = (vs[k], vs[(k + 1) % vs.len()]);
            if !vistas.insert((a.min(b), a.max(b))) {
                continue;
            }
            let (pa, pb) = (pos[a as usize], pos[b as usize]);
            let meio = [
                (pa[0] + pb[0]) * 0.5,
                (pa[1] + pb[1]) * 0.5,
                (pa[2] + pb[2]) * 0.5,
            ];
            let Some(direccao) = direccao_do_troco(percurso, meio, raio) else {
                continue;
            };
            let mut d2 = f32::INFINITY;
            for q in percurso {
                let v = sub(meio, *q);
                d2 = d2.min(v[0].mul_add(v[0], v[1].mul_add(v[1], v[2] * v[2])));
            }
            let banda = (((d2.sqrt() / raio) * bandas as f32) as usize).min(bandas - 1);

            let aresta = sub(pb, pa);
            let (la, ld) = (comprimento(aresta), comprimento(direccao));
            if la <= 0.0 || ld <= 0.0 {
                continue;
            }
            let c = (f64::from(aresta[0]) * f64::from(direccao[0])
                + f64::from(aresta[1]) * f64::from(direccao[1])
                + f64::from(aresta[2]) * f64::from(direccao[2]))
                / (la * ld);
            let angulo = c.clamp(-1.0, 1.0).acos().to_degrees() % 90.0;
            let desvio = angulo.min(90.0 - angulo);
            out[banda].1 += 1;
            if desvio < 15.0 {
                out[banda].0 += 1;
            }
        }
    }
    out
}
