//! **A PORTA QUE ENCOLHE A TOPOLOGIA** — irmã da [`super::Mesh::splice_topology`]
//! e filha do [`crate::mesh`] pela mesma razão que ela: escreve os campos
//! PRIVADOS da malha, e privacidade em Rust alcança os descendentes.
//!
//! # Por que ela é uma porta e não um `rebuild`
//!
//! Colapsar uma aresta muda quatro faces e um punhado de anéis. Reconstruir a
//! malha para isso é o `O(malha)` que as waves anteriores tiraram do caminho
//! quente — medido **10,4 ms a 98k** para mudar duas faces.
//!
//! # As DUAS fases, e por que elas não se misturam
//!
//! Um colapso renumera faces **e** vértices, e as duas renumerações entrelaçadas
//! são a forma de errar: cada remendo teria de saber em que numeração o vizinho
//! já está, e o modo de falha é uma face citando um índice **válido** que é de
//! outro vértice — nada explode, a malha só fica errada. Aqui as fases são
//! sequenciais e cada uma tem um eixo só; entre elas a estrutura está íntegra.

use super::{Mesh, RegionScratch};
use crate::Remap;
use crate::face::Face;

/// **DOIS VÉRTICES VIRARAM UM** — o que um colapso de aresta pede à malha.
///
/// ⚠️ **O par é `keep`/`gone` e não *"os dois vértices"*, porque a assimetria é
/// real:** um deles fica com o índice, o outro some. Quem decide qual é a lei do
/// operador; o que ele HERDA é a lei da porta.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VertexMerge {
    /// O vértice que fica.
    pub keep: u32,
    /// O vértice que some — a cor e a máscara dele entram na média.
    pub gone: u32,
    /// Onde o sobrevivente pousa.
    pub at: [f32; 3],
}

impl Mesh {
    /// **A TOPOLOGIA ENCOLHEU** — faces trocaram de cantos, faces sumiram,
    /// vértices sumiram, sobreviventes se mudaram de lugar.
    ///
    /// Devolve o [`Remap`], que é **o contrato desta porta**: quem guardava um
    /// índice de vértice ou de face entre chamadas tem de o aplicar, na ordem em
    /// que ele veio. O refino não precisa de nada disso porque só APENDA; um
    /// colapso apaga, e apagar sem deixar buraco é trocar com o último.
    ///
    /// ⚠️ **`dead_faces` e `dead_verts` chegam CRESCENTES e sem repetição**, na
    /// numeração de entrada. Ordenar aqui esconderia de onde o conjunto veio, e
    /// um repetido faria a compactação remover um vivo — em `debug` o
    /// [`Remap::plan`] o denuncia.
    ///
    /// ⚠️ **`merges` é aplicado ANTES de qualquer renumeração**, então os índices
    /// dele são os de entrada.
    pub fn shrink_topology(
        &mut self,
        edits: &[(u32, Face)],
        dead_faces: &[u32],
        dead_verts: &[u32],
        merges: &[VertexMerge],
        scratch: &mut RegionScratch,
    ) -> Remap {
        let remap = Remap::plan(
            dead_faces,
            self.faces.len(),
            dead_verts,
            self.positions.len(),
        );
        if dead_faces.is_empty() && dead_verts.is_empty() && edits.is_empty() && merges.is_empty() {
            scratch.forget();
            return remap;
        }
        let old_verts = self.positions.len();
        // ⚠️ **A média de cor e máscara mora AQUI e não no chamador**, pela mesma
        // lei do ponto médio na porta irmã: *o que um vértice herda de quem ele
        // absorveu* é uma resposta só, e o dia em que entrar um quinto plano
        // por-vértice quem esquecer dele é esta função.
        for m in merges {
            let (keep, gone) = (m.keep as usize, m.gone as usize);
            self.positions[keep] = m.at;
            if let Some(c) = self.colors.as_mut() {
                let (ck, cg) = (c[keep], c[gone]);
                c[keep] = [
                    (ck[0] + cg[0]) * 0.5,
                    (ck[1] + cg[1]) * 0.5,
                    (ck[2] + cg[2]) * 0.5,
                ];
            }
            if let Some(mk) = self.masks.as_mut() {
                mk[keep] = (mk[keep] + mk[gone]) * 0.5;
            }
            if let Some(a) = self.ao.as_mut() {
                a[keep] = (a[keep] + a[gone]) * 0.5;
            }
        }

        // ── Fase 1: as FACES. A numeração de vértice não se move aqui. ──
        //
        // Só o que muda é copiado para ter o "antes" — guardar a lista inteira
        // seria o `O(malha)` entrando pela porta dos fundos.
        let changed: Vec<(u32, Face, Face)> = edits
            .iter()
            .map(|&(i, new)| (i, self.faces[i as usize], new))
            .collect();
        for &(i, _, new) in &changed {
            self.faces[i as usize] = new;
        }
        // A face morta some do vetor, então ela é lida AGORA — depois não há de
        // onde saber que cantos ela tinha, e são eles que dizem de que anéis
        // tirá-la.
        let dead_pairs: Vec<(u32, Face)> = dead_faces
            .iter()
            .map(|&f| (f, self.faces[f as usize]))
            .collect();
        for &(from, to) in &remap.face_moves {
            self.faces[to as usize] = self.faces[from as usize];
            self.face_normals[to as usize] = self.face_normals[from as usize];
        }
        self.faces.truncate(remap.faces);
        self.face_normals.truncate(remap.faces);
        let touched = self
            .adjacency
            .shrink_faces(&changed, &dead_pairs, &self.faces);
        self.octree.shrink_faces(dead_faces, &remap);

        // ── Fase 2: os VÉRTICES. As faces já estão na numeração final. ──
        //
        // ⚠️ A marca é um vetor de bytes do tamanho da malha, e é o mesmo preço
        // que o refino já paga no `pending` do grafo de arestas. Ela é carregada
        // pelas trocas em vez de coletada no fim porque um destino pode mudar de
        // casa outra vez — ver [`crate::Adjacency::shrink_verts`].
        let mut mark = vec![false; old_verts];
        for &v in &touched {
            mark[v as usize] = true;
        }
        for &(from, to) in &remap.vert_moves {
            self.positions[to as usize] = self.positions[from as usize];
            self.normals[to as usize] = self.normals[from as usize];
            // **A curvatura viaja com as duas de cima**, e ⚠️ **ela é
            // REDUNDANTE hoje — a medição me corrigiu, não o contrário.** Eu
            // escrevi aqui que o vértice do fim do vetor *"quase nunca está em
            // `affected`, então ninguém vai recomputá-lo"*. É falso: o
            // `Adjacency::shrink_verts` faz `mark[to] = true`
            // **incondicionalmente** para todo destino, logo todo vértice que
            // muda de casa entra em `affected` e o `refresh_region` do fim o
            // recomputa. A mutação que apaga esta linha **sobrevive**.
            //
            // ⚠️ E o CONTROLE fecha o argumento: a mesma mutação na linha das
            // NORMAIS, uma acima, que shipa desde a W9.3 com um oráculo de bit ao
            // lado, **também sobrevive** — as 243 do crate ficam verdes. As duas
            // são defesa em camada, não a camada que segura.
            //
            // Elas ficam porque a lista de planos por-vértice tem de ser
            // UNIFORME: o `mark[to]` parece redundante para quem lê o laço de
            // faces logo acima dele, e é exatamente o tipo de linha que uma
            // otimização futura apaga — momento em que os dois planos ficariam
            // velhos juntos, e os dois gates de bit sangrariam juntos. É a camada
            // que existe para o dia em que a de baixo sair.
            self.curvatures[to as usize] = self.curvatures[from as usize];
            if let Some(c) = self.colors.as_mut() {
                c[to as usize] = c[from as usize];
            }
            if let Some(m) = self.masks.as_mut() {
                m[to as usize] = m[from as usize];
            }
            if let Some(a) = self.ao.as_mut() {
                a[to as usize] = a[from as usize];
            }
        }
        self.positions.truncate(remap.verts);
        self.normals.truncate(remap.verts);
        self.curvatures.truncate(remap.verts);
        if let Some(c) = self.colors.as_mut() {
            c.truncate(remap.verts);
        }
        if let Some(m) = self.masks.as_mut() {
            m.truncate(remap.verts);
        }
        if let Some(a) = self.ao.as_mut() {
            a.truncate(remap.verts);
        }
        // A topologia mudou: o que estava assado descreve outra malha.
        self.ao_stale = true;
        let affected = self
            .adjacency
            .shrink_verts(&mut self.faces, &remap, &mut mark);

        self.refresh_region(&affected, scratch);
        remap
    }
}
