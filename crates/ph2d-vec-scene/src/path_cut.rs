//! **O CORTE** — abrir um contorno num vértice (plano 25 §7, a W4).
//!
//! Este módulo tem UMA função de produto, e ela é o primitivo de que **as quatro** ferramentas de
//! corte são feitas: a tesoura, a faca, a borracha de caminho e o "break path" do modo Node. A
//! diferença entre elas não é geométrica — é só *de onde vem o vértice*:
//!
//! | ferramenta | de onde vem o vértice |
//! |---|---|
//! | Tesoura | um clique: o vértice sob o cursor, senão [`crate::split_segment`] no ponto mais próximo |
//! | Faca | cada cruzamento de uma lâmina reta com a curva |
//! | Borracha de caminho | as duas pontas de um arrasto AO LONGO da curva (dois cortes + um delete) |
//!
//! ⚠️ **Nenhuma delas tem aritmética de arco própria.** A borracha, em particular, não reimplementa
//! o `fx_trim`: dois cortes deixam o trecho do meio como um contorno inteiro, e apagá-lo é
//! [`VecPath::remove_contour`] ou [`VecScene::remove_path`] — que já existem. Uma segunda resposta
//! a *"onde este caminho termina?"* divergiria da primeira no dia em que uma quina viva ou um
//! efeito entrasse no meio.
//!
//! # As três respostas, e por que a última é uma pergunta de FILL RULE
//!
//! - **Contorno FECHADO** → vira ABERTO, re-enraizado no vértice do corte, que passa a aparecer
//!   nas DUAS pontas. Um objeto, um id, um contorno. (É o que o Illustrator faz com a tesoura.)
//! - **Contorno ABERTO, vértice INTERIOR** → parte em dois, e o vértice do corte fica nos dois
//!   lados (as duas metades encostam onde a tesoura passou).
//! - **Ponta de contorno aberto** → `None`. Não há o que cortar ali, e o Illustrator também recusa.
//!
//! A segunda metade vira um **path novo** (arrastável para longe) num path de contorno único, e um
//! **contorno irmão** num compound. A pergunta é feita UMA vez, aqui: separar um contorno de um
//! compound em dois OBJETOS mudaria o que a [`crate::FillRule`] significa — o buraco deixaria de
//! ser buraco no clique que era para ser um corte.
//!
//! # O que o corte NÃO faz
//!
//! Não toca `selected_verts` de ninguém: os índices planos a jusante do corte andam, e quem tem
//! seleção é quem sabe o que ela significa. O chamador limpa.

use crate::{Contour, VecPath, VecPathId, VecScene};

/// O que um corte produziu — o suficiente para o chamador achar as duas metades sem re-derivar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cut {
    /// O contorno estava FECHADO e apenas ABRIU (um objeto, nada nasceu).
    pub opened: bool,
    /// O path NOVO que recebeu a segunda metade (path de contorno único partido em dois).
    pub new_path: Option<VecPathId>,
    /// O índice do contorno NOVO que recebeu a segunda metade (um contorno de compound partido).
    pub new_contour: Option<usize>,
}

impl VecScene {
    /// **Corta o path `id` no vértice de índice PLANO `vert`** — o primitivo de todas as
    /// ferramentas de corte (ver o doc do módulo). `None` quando não há corte a fazer: id
    /// inexistente, índice fora do alcance, contorno degenerado (< 2 vértices) ou o vértice é uma
    /// PONTA de contorno aberto.
    ///
    /// ⚠️ **Os handles do vértice cortado são preservados nas duas cópias**, e isso é deliberado:
    /// um contorno aberto só consome o `out_handle` do primeiro e o `in_handle` do último, então
    /// os handles "mortos" ficam guardados e **re-fechar devolve a curva original ao bit**. Zerá-los
    /// pareceria higiene e destruiria a tangente que o artista autorou.
    pub fn cut_path_at_vertex(&mut self, id: VecPathId, vert: usize) -> Option<Cut> {
        let path = self.paths.iter().find(|p| p.id == id)?;
        let (c, local) = path.locate_vert(vert)?;
        let (verts, closed) = path.contour(c)?;
        if verts.len() < 2 {
            return None;
        }

        if closed {
            let path = self.paths.iter_mut().find(|p| p.id == id)?;
            let (verts, closed) = path.contour_mut(c)?;
            verts.rotate_left(local);
            let seam = verts[0];
            verts.push(seam);
            *closed = false;
            return Some(Cut {
                opened: true,
                new_path: None,
                new_contour: None,
            });
        }

        // Aberto: só um vértice INTERIOR parte o contorno.
        if local == 0 || local + 1 >= verts.len() {
            return None;
        }
        let compound = path.is_compound();
        let z = self.paths.iter().position(|p| p.id == id)?;

        let tail: Vec<crate::VecVertex> = {
            let path = &mut self.paths[z];
            let (verts, _) = path.contour_mut(c)?;
            let tail = verts[local..].to_vec();
            verts.truncate(local + 1);
            tail
        };

        if compound {
            let path = &mut self.paths[z];
            path.subpaths.push(Contour {
                verts: tail,
                closed: false,
            });
            return Some(Cut {
                opened: false,
                new_path: None,
                new_contour: Some(path.contour_count() - 1),
            });
        }

        // Contorno único: a segunda metade vira um OBJETO, logo acima da fonte na pilha de z (as
        // duas metades ficam vizinhas, e nenhuma salta para a frente da cena).
        let src = &self.paths[z];
        let half = VecPath {
            verts: tail,
            closed: false,
            fill: src.fill.clone(),
            stroke: src.stroke,
            fill_rule: src.fill_rule,
            // A pilha de efeitos é APARÊNCIA, e as duas metades continuam a mesma aparência.
            effects: src.effects.clone(),
            ..VecPath::default()
        };
        let new_path = self.insert_path(z + 1, half);
        Some(Cut {
            opened: false,
            new_path: Some(new_path),
            new_contour: None,
        })
    }
}

#[cfg(test)]
#[path = "path_cut_tests.rs"]
mod tests;
