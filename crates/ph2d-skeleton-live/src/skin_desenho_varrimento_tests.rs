//! A promessa da cena `=4` (*«o CONTORNO faz um canto em V … sem um laço nem um olho desenhado por
//! dentro da curva»*) numa VARREDURA FINA da dobra, desde que a lei do ângulo é o meio-ângulo
//! (2026-10-06): o meio-ângulo leva o início da dobra-sobre-si-mesma da aresta de dentro de `~103°`
//! (círculo) para `~118°`, e a régua de oito dobras da [`super::numa_dobra_forte_o_desenho_nao_se_cruza`]
//! não via essa janela.

use super::{Forma, com_e_sem_contacto_em};

/// ⭐⭐⭐ **GATE — de `0°` a `175°`, grau a grau, nas três formas, o desenho COM a lei do contacto
/// nunca se cruza**; e o CONTROLO (sem ela) cruza em alguma dobra de cada forma, senão a varredura
/// não conteria o laço.
#[test]
fn de_zero_a_175_graus_o_desenho_nunca_se_cruza() {
    for forma in [Forma::C, Forma::Z, Forma::Uma] {
        let (mut cruzou_sem, mut cruzou_com) = (Vec::new(), Vec::new());
        for g in 0..=175u16 {
            let g = f32::from(g);
            let (sem, com) = com_e_sem_contacto_em(g, forma);
            if ph2d_vec_boolean::resolve_overlap(&sem).is_some() {
                cruzou_sem.push(g);
            }
            if ph2d_vec_boolean::resolve_overlap(&com).is_some() {
                cruzou_com.push(g);
            }
        }
        println!(
            "  {forma:?}: sem o contacto cruza em {} dobras ({:?}…) · com: {:?}",
            cruzou_sem.len(),
            cruzou_sem.first(),
            cruzou_com
        );
        assert!(
            cruzou_com.is_empty(),
            "{forma:?}: com a lei do contacto o desenho cruza-se a {cruzou_com:?}°"
        );
        assert!(
            !cruzou_sem.is_empty(),
            "{forma:?}: o CONTROLO — sem o contacto nenhuma dobra cruza, a varredura não contém o laço"
        );
    }
}
