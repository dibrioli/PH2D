//! **AS LEIS QUE LEEM O ANEL** — os quatro verbos que perguntam aos vizinhos,
//! irmãos do [`super`], onde ficam as leis que só olham para o próprio vértice
//! (ou para o plano que ele ajustou).
//!
//! ⚠️ **A divisão é a classificação que o motor JÁ declara** —
//! [`Verb::uses_neighbours`] —, e não um corte por tamanho: quem responde `true`
//! ali paga a travessia do CSR por vértice, e é essa a propriedade que junta
//! estes quatro e separa os outros dezoito. Um verbo de anel novo entra aqui, e
//! a pergunta *"quem precisa da adjacência?"* segue tendo **uma** resposta.
//!
//! ⚠️ **E o `w` continua a chegar por ARGUMENTO, nunca a ser re-derivado:** ele
//! é o peso que o `compute_target` já resolveu (falloff × máscara × alpha), e
//! uma segunda derivação aqui seria a segunda resposta a *"quanto este vértice
//! recebe"* — divergindo no dia em que um canal novo entrasse naquele produto.

use super::*;
use aim::remove_along;

impl SculptStroke {
    /// `Smooth.js` — o único tool de geometria da referência **sem falloff** no
    /// kernel; aqui ele chega pelo `w`, que é o superconjunto (o artista escolhe
    /// a curva, e `Falloff::Constant` reproduz a referência). A forma é a dela:
    /// `pos·(1 − m) + média·m`.
    pub(super) fn target_smooth(
        &self,
        mesh: &Mesh,
        brush: &Brush,
        v: u32,
        live: [f32; 3],
        w: f32,
    ) -> [f32; 3] {
        let avg = self.neighbour_average(mesh, brush, v, live);
        let m = 1.0 - w;
        [
            live[0] * m + avg[0] * w,
            live[1] * m + avg[1] * w,
            live[2] * m + avg[2] * w,
        ]
    }

    /// NOSSO, e a referência não tem: o laplaciano com o sinal trocado. Reflete
    /// a média através do próprio vértice, com a mesma magnitude que o
    /// [`Self::target_smooth`] teria.
    pub(super) fn target_sharpen(
        &self,
        mesh: &Mesh,
        brush: &Brush,
        v: u32,
        live: [f32; 3],
        w: f32,
    ) -> [f32; 3] {
        let avg = self.neighbour_average(mesh, brush, v, live);
        [
            live[0] + (live[0] - avg[0]) * w,
            live[1] + (live[1] - avg[1]) * w,
            live[2] + (live[2] - avg[2]) * w,
        ]
    }

    /// **O RELAX** — a mesma média do [`Self::target_smooth`] com **uma** linha a
    /// mais, e essa linha é a ferramenta inteira: o que corre ao longo da normal
    /// é REMOVIDO, então o que sobra desliza pela superfície e a forma não se
    /// mexe (`translation_to_plane`, `sculpt_smooth.cc:458`).
    ///
    /// ⚠️ **A normal é a VIVA (`mesh.normals()`), ao contrário do
    /// [`Verb::Inflate`]** — e as duas escolhas estão certas porque a grandeza
    /// tem papéis opostos nos dois: lá ela é a direção em que se ANDA (congelar
    /// impede o empurrão de girar sob um traço parado), aqui ela é o plano em
    /// que se FICA (congelar prenderia o vértice ao plano tangente do pen-down,
    /// e ele sairia da superfície que os dabs anteriores moveram). Ver o doc de
    /// [`Verb::SlideRelax`].
    pub(super) fn target_slide_relax(
        &self,
        mesh: &Mesh,
        brush: &Brush,
        v: u32,
        live: [f32; 3],
        w: f32,
    ) -> [f32; 3] {
        match self.relax_normal(mesh, v, live) {
            Some(n) => {
                let avg = self.neighbour_average(mesh, brush, v, live);
                let d = remove_along([avg[0] - live[0], avg[1] - live[1], avg[2] - live[2]], n);
                [live[0] + d[0] * w, live[1] + d[1] * w, live[2] + d[2] * w]
            }
            None => live,
        }
    }

    /// **O HC** — a mesma média do [`Self::target_smooth`] e, logo a seguir, a
    /// DEVOLUÇÃO do que ela custou: `−w·[(1−β)·média(b) + β·b]`.
    ///
    /// ⚠️ **A primeira metade é o Smooth escrito por extenso, e é de propósito
    /// que ela não o CHAMA:** o `w` já entra nas duas metades, e fatorar a soma
    /// delas (`live + w·(avg − live − corr)`) mudaria a ordem das operações em
    /// `f32` — o verbo é um par de passes na referência, e as duas correções
    /// chegam ao vértice separadas.
    ///
    /// ⚠️ **O `b` NÃO é recomputado aqui**, e é o que faz o verbo caber num
    /// passe: `média(b)` precisa do `b` dos VIZINHOS, que o
    /// [`Self::fill_hc_disp`] já escreveu para a pegada inteira antes de
    /// qualquer escrita. Recomputá-lo por vértice daria o `b` do próprio e zero
    /// para todo vizinho — um HC sem a metade que o define.
    pub(super) fn target_surface_smooth(
        &self,
        mesh: &Mesh,
        v: u32,
        s: usize,
        live: [f32; 3],
        w: f32,
        brush: &Brush,
    ) -> [f32; 3] {
        let avg = self.neighbour_average(mesh, brush, v, live);
        let own = self.hc_disp(s);
        let avg_b = self.hc_disp_average(mesh, v, own);
        // ⚠️ **O piso é do MOTOR, não do slider** — ver [`crate::HC_VERTEX_MIN`]:
        // abaixo dele o operador amplifica, e um documento que traga o valor
        // errado tem de ser corrigido em vez de rebentar a malha.
        let beta = brush.hc_vertex.clamp(crate::HC_VERTEX_MIN, 1.0);
        let mut out = [0.0f32; 3];
        for k in 0..3 {
            let corr = (1.0 - beta) * avg_b[k] + beta * own[k];
            out[k] = live[k] + (avg[k] - live[k]) * w - corr * w;
        }
        out
    }
}
