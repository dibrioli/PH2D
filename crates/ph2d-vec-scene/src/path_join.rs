//! **A JUNÇÃO** — o inverso do [`crate::path_cut`] (plano 25 §7, a W4): fechar um caminho aberto e
//! soldar dois caminhos abertos num só. É o `Ctrl+J` do Illustrator.
//!
//! Módulo irmão de `path_ops` pelo teto de LOC, e o corte é o mesmo do `path_cut`: `path_ops` move
//! e transforma um caminho INTEIRO; aqui a topologia dele muda.
//!
//! # Nada de geometria nova
//!
//! A receita de *"que ponta encosta em que ponta, e para que lado cada caminho tem de correr"* já
//! existia dentro do [`VecScene::weld_new_shape`], que a usa para soldar uma forma recém-desenhada
//! nas vizinhas. A diferença é só de GATILHO: lá o par é escolhido por TOLERÂNCIA (as pontas têm
//! de estar perto), aqui o artista **escolheu os dois objetos**, então o par mais próximo vence
//! sempre e a tolerância decide apenas *se a emenda funde os dois vértices num só ou se nasce um
//! segmento entre eles* — que é exatamente a distinção que o Illustrator faz.
//!
//! # Quem sobrevive
//!
//! O de mais BAIXO na pilha de z, com o id e o estilo dele — a mesma regra do
//! [`VecScene::make_compound`]. Duas respostas a *"de quem é a aparência do resultado?"* dariam,
//! no mesmo gesto, cores diferentes conforme a ordem de clique.

use crate::{VecPathId, VecScene, VecXforms};

/// Distância ao quadrado entre dois pontos.
fn d2(a: [f64; 2], b: [f64; 2]) -> f64 {
    (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)
}

impl VecScene {
    /// **Fecha** o contorno primário do path aberto `id`, soldando as pontas se elas coincidem.
    ///
    /// Se as duas pontas já estão a ≤ `tol` (coordenadas locais) elas viram **um vértice só** — o
    /// último herda a tangente de SAÍDA do primeiro e a duplicata some; senão nasce o segmento de
    /// fecho normal. `false` se não houve fecho.
    ///
    /// ⚠️ **É a porta ÚNICA de "fechar".** O botão `Close Path` do painel passa por aqui: antes ele
    /// só virava o flag, então fechar um laço cujas pontas o artista tinha acabado de encostar
    /// deixava **dois vértices sobrepostos** no mesmo lugar — invisível no desenho e presente em
    /// todo Delete, Average e Simplify seguinte. A metade que decide o flag continua sendo o
    /// [`Self::set_path_closed`] (a regra dos ≥ 2 vértices mora lá, e só lá).
    ///
    /// A fusão é a MESMA que o [`Self::weld_new_shape`] faz ao detectar um laço; um `pop()` cego
    /// apagaria a ponta da última curva e a deformaria.
    pub fn close_path(&mut self, id: VecPathId, tol: f64) -> bool {
        let Some(p) = self.paths.iter_mut().find(|p| p.id == id) else {
            return false;
        };
        // ⚠️ Só solda com ≥ 3: com 2 vértices coincidentes não sobra região nenhuma depois da
        // fusão, e o path viraria um ponto fechado.
        if !p.closed && p.verts.len() >= 3 {
            let first = p.verts[0];
            let last = *p.verts.last().expect("len >= 3");
            if d2(first.anchor, last.anchor) <= tol * tol {
                if let Some(l) = p.verts.last_mut() {
                    l.out_handle = first.out_handle;
                }
                p.verts.remove(0);
            }
        }
        self.set_path_closed(id, true)
    }

    /// **Solda** os paths ABERTOS `a` e `b` num só, pelo par de pontas mais PRÓXIMO (em MUNDO).
    /// Devolve o id do sobrevivente (o de mais baixo na pilha de z), ou `None` se algum id sumiu,
    /// algum está fechado ou vazio, ou são o mesmo.
    ///
    /// Pontas a ≤ `tol` fundem-se num vértice só; mais longe que isso, o segmento entre elas
    /// aparece — o artista pediu para ligar, e ligar duas pontas separadas é uma linha.
    pub fn join_paths(
        &mut self,
        a: VecPathId,
        b: VecPathId,
        xforms: &VecXforms,
        tol: f64,
    ) -> Option<VecPathId> {
        if a == b {
            return None;
        }
        let (za, zb) = (
            self.paths.iter().position(|p| p.id == a)?,
            self.paths.iter().position(|p| p.id == b)?,
        );
        // O de baixo é a base (a regra do `make_compound`).
        let (dst, src) = if za <= zb { (a, b) } else { (b, a) };
        for id in [dst, src] {
            let p = self.paths.iter().find(|p| p.id == id)?;
            if p.closed || p.verts.is_empty() {
                return None;
            }
        }

        let (df, dl) = (
            self.endpoint_world(xforms, dst, true)?,
            self.endpoint_world(xforms, dst, false)?,
        );
        let (sf, sl) = (
            self.endpoint_world(xforms, src, true)?,
            self.endpoint_world(xforms, src, false)?,
        );
        // (a ponta do dst é a PRIMEIRA?, a ponta do src é a PRIMEIRA?, distância²)
        let mut best = (true, true, f64::INFINITY);
        for (d_first, dp) in [(true, df), (false, dl)] {
            for (s_first, sp) in [(true, sf), (false, sl)] {
                let dd = d2(dp, sp);
                if dd < best.2 {
                    best = (d_first, s_first, dd);
                }
            }
        }
        let (d_first, s_first, dd) = best;

        // O FIM do dst tem de encontrar o COMEÇO do src. Se a ponta escolhida do dst é a
        // primeira, o dst corre ao contrário; o src é invertido pelo próprio `merge_path_into`.
        if d_first {
            self.reverse_path(dst);
        }
        let weld = dd <= tol * tol;
        self.merge_path_into(dst, src, xforms, !s_first, weld)
            .then_some(dst)
    }
}

#[cfg(test)]
#[path = "path_join_tests.rs"]
mod tests;
