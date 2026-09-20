//! **OS PLANOS OPCIONAIS SOBREVIVEM AO COLAPSO** — irmão (`#[path]`) do
//! [`super::tests`], e o corte é o ASSUNTO: lá a TOPOLOGIA (fendas, `χ`, a
//! condição de elo, o ponto fixo do par refino/colapso), aqui os CANAIS
//! por-vértice que uma compactação carrega.
//!
//! ⚠️ **E ele é um ficheiro à parte porque o irmão estava a `685` de `700`** —
//! *um gate novo não cabe num ficheiro cheio, e a resposta é o CORTE e nunca
//! uma entrada nova na lista de folgas.*

use super::tests::{mean_edge, scratch, tri_sphere};
use super::*;

/// ⭐⭐⭐ **O COLAPSO CARREGA A COR E A MÁSCARA** — a metade simétrica do
/// `dyntopo_tests :: the_new_vertices_carry_colour_and_mask`, e que este ficheiro
/// **nunca teve**: até 2026-09-20 a palavra `colors` não aparecia uma vez aqui.
///
/// ⛔⛔ É essa ausência que fez a nota do roteador do módulo mandar procurar as
/// manchas pretas no colapso — *«ele não menciona `colors` nem `masks` em lado
/// nenhum, nem na implementação nem nos testes»*. A metade dos **testes** estava
/// certa; a da **implementação** não: a lei vive em
/// [`crate::Mesh::shrink_topology`], a porta que o colapso chama, e o doc dela
/// diz-se dona dela por escrito. *Uma ausência afirmada sobre o ficheiro do
/// operador é um palpite sobre a porta que ele usa.*
///
/// # As DUAS metades, e nenhuma basta sozinha
///
/// 1. **Campo CONSTANTE ⇒ tudo continua `C`, ao bit.** A média de dois iguais é
///    o próprio valor e uma permutação move-o sem o mudar, logo qualquer desvio
///    é um valor INVENTADO — e o valor nomeia a causa (preto = slot que ninguém
///    escreveu; [`crate::DEFAULT_COLOR`] = plano recriado em vez de permutado).
/// 2. **Campo de DUAS cores ⇒ nada sai do ENVELOPE.** A primeira é cega a uma
///    permutação — *trocar `C` por `C` é invisível* —, e esta vê um valor que
///    não podia sair de nenhuma média do que entrou.
#[test]
fn a_cor_e_a_mascara_sobrevivem_ao_colapso() {
    const C: [f32; 3] = [0.8, 0.35, 0.15];
    const M: f32 = 0.25;

    // ── 1. O campo constante. ──
    let mut m = tri_sphere(14, 20);
    let n = m.vert_count();
    m.put_masks(vec![M; n]);
    m.colors_mut().fill(C);
    let emin = 1.2 * mean_edge(&m);
    let r = collapse_in_sphere(
        &mut m,
        [0.0, 0.0, 1.0],
        0.9,
        emin,
        &mut Remap::default(),
        &mut scratch(),
    );
    assert!(
        matches!(r, Collapse::Done { .. }) && m.vert_count() < n,
        "o controlo: a fixtura tem de colapsar de verdade ({r:?})"
    );
    let cores = m.colors().expect("o colapso apagou o plano de cor");
    assert_eq!(cores.len(), m.vert_count(), "e ele mede a malha nova");
    for (v, c) in cores.iter().enumerate() {
        assert!(
            c.iter().zip(C).all(|(a, b)| a.to_bits() == b.to_bits()),
            "o vértice {v} de {} lê {c:?} numa peça inteiramente {C:?} — a média \
             de dois iguais é o valor, e uma permutação move-o sem o mudar. \
             Um [0,0,0] aqui é um slot que ninguém escreveu; um {:?} é o plano \
             recriado em vez de permutado",
            m.vert_count(),
            crate::DEFAULT_COLOR
        );
    }
    let masks = m.masks().expect("nem o plano de máscara");
    assert_eq!(masks.len(), m.vert_count());
    assert!(
        masks.iter().all(|v| v.to_bits() == M.to_bits()),
        "a máscara também não sobreviveu ao colapso"
    );

    // ── 2. O envelope, que é o que vê uma permutação. ──
    const QUENTE: [f32; 3] = [0.9, 0.1, 0.1];
    const FRIA: [f32; 3] = [0.1, 0.2, 0.9];
    let mut m = tri_sphere(14, 20);
    let xs: Vec<f32> = m.positions().iter().map(|p| p[0]).collect();
    for (c, x) in m.colors_mut().iter_mut().zip(&xs) {
        *c = if *x < 0.0 { FRIA } else { QUENTE };
    }
    let n = m.vert_count();
    let emin = 1.2 * mean_edge(&m);
    let r = collapse_in_sphere(
        &mut m,
        [0.0, 0.0, 1.0],
        0.9,
        emin,
        &mut Remap::default(),
        &mut scratch(),
    );
    assert!(
        matches!(r, Collapse::Done { .. }) && m.vert_count() < n,
        "o controlo da 2.ª metade ({r:?})"
    );
    let cores = m.colors().expect("o plano sobrevive");
    for (v, c) in cores.iter().enumerate() {
        for k in 0..3 {
            let (lo, hi) = (QUENTE[k].min(FRIA[k]), QUENTE[k].max(FRIA[k]));
            assert!(
                c[k] >= lo && c[k] <= hi && !c[k].is_nan(),
                "o vértice {v} lê {c:?}, e o canal {k} está fora do envelope \
                 [{lo}, {hi}] do que entrou — nem uma média nem uma mudança de \
                 casa saem do envelope do que entrou"
            );
        }
    }
}
