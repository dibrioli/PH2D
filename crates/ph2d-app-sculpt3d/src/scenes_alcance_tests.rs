//! Os gates da cena `=39` — ⚠️ **eles medem que a FIXTURA contém o fenómeno**,
//! que é a coisa que a jornada de 10/09 pagou duas vezes: uma fixtura sem o
//! fenómeno lê-se exactamente como *«não há defeito»*.

use super::{FOLGA, duas_pecas_vizinhas};

/// ⭐⭐⭐ **A cena tem DOIS componentes, e é isso que faz a pergunta ser binária.**
///
/// A máscara de alcance corta quem a superfície não liga; se a fixtura fosse um
/// componente só, o corte passaria a depender do tecto e a resposta deixaria de
/// ser *sim ou não*. ⛔ Um gate sobre a FOLGA não bastaria: duas peças a tocar-se
/// pela ponta de um vértice teriam folga zero e um componente só.
#[test]
fn a_cena_tem_duas_pecas_soltas_e_nao_uma() {
    let m = duas_pecas_vizinhas();
    let n = m.vert_count();
    let viz = &m.adjacency().vert_verts;
    let mut visto = vec![false; n];
    let mut componentes = 0usize;
    let mut pilha = Vec::new();
    for s in 0..n {
        if visto[s] {
            continue;
        }
        componentes += 1;
        visto[s] = true;
        pilha.push(s);
        while let Some(u) = pilha.pop() {
            for &v in viz.neighbours(u) {
                if !visto[v as usize] {
                    visto[v as usize] = true;
                    pilha.push(v as usize);
                }
            }
        }
    }
    assert_eq!(
        componentes, 2,
        "a cena =39 tem {componentes} componente(s) -- ela existe para o caso em que a \
         SUPERFICIE nao liga as duas partes, e com um componente so' o smoke mede outra coisa"
    );
}

/// ⚠️ **E as duas têm de estar PERTO, senão a bola do pincel não alcança a
/// vizinha e o defeito não aparece.**
///
/// ⭐ A barra é medida e a vizinha é a prova: a `0,05` a máscara corta `47,1 %`
/// do carimbo a raio `0,35`; a `0,15` ela corta `6,2 %` — *três vezes a folga e
/// o defeito deixa de se ver*.
#[test]
fn as_duas_pecas_estao_perto_o_bastante_para_o_pincel_alcancar_as_duas() {
    let m = duas_pecas_vizinhas();
    let pos = m.positions();
    let mut vao = f32::INFINITY;
    for p in pos.iter().filter(|p| p[0] < 0.0) {
        for q in pos.iter().filter(|q| q[0] > 0.0) {
            let d = [q[0] - p[0], q[1] - p[1], q[2] - p[2]];
            vao = vao.min(d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])));
        }
    }
    let vao = vao.sqrt();
    // ⚠️ **A barra é o RAIO do pincel que o roteiro manda usar**, e não um número
    // escolhido: se o vão passar dele, a esfera de consulta não alcança a peça
    // vizinha e o passo (4) fica sem sujeito.
    assert!(
        vao <= 0.20,
        "o vao entre as duas pecas mede {vao:.4} -- acima de 0,20 a bola do pincel do \
         roteiro nao alcanca a vizinha, e a cena deixa de conter o defeito (FOLGA = {FOLGA})"
    );
    assert!(
        vao > 0.0,
        "as duas pecas tocam-se: com vao zero elas seriam um componente so'"
    );
}
