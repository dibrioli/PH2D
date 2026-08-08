//! **AS PORTAS QUE MUDAM A TOPOLOGIA** — filho do [`crate::mesh`], não irmão.
//!
//! ⚠️ **E o parentesco é load-bearing:** as duas portas escrevem os campos
//! PRIVADOS da malha (posições, faces, os dois anéis, o octree), e privacidade
//! em Rust alcança os descendentes do módulo que declara. Um módulo irmão
//! precisaria abrir os campos ao crate inteiro — e o dia em que um kernel de
//! pincel escrever `mesh.faces` direto é o dia em que o octree passa a apontar
//! para onde a malha estava, sem erro nenhum.
//!
//! O corte é por RESPONSABILIDADE: o pai diz *o que uma malha é* (os buffers, o
//! que se pergunta a ela, como ela se reconstrói e como uma REGIÃO se refresca);
//! aqui fica *como a topologia dela é EDITADA*.

use super::{Mesh, RegionScratch};
use crate::face::Face;

/// Os vértices que uma mudança de topologia acrescenta.
///
/// ⚠️ **A POSIÇÃO vem do chamador e a COR/MÁSCARA não**, e a assimetria é
/// deliberada: onde o ponto médio pousa é a lei do operador (o corte o desloca ao
/// longo da normal para não achatar a curva), mas *o que ele herda dos pais* é a
/// mesma resposta para qualquer operação que parta uma aresta.
#[derive(Clone, Copy, Debug)]
pub struct VertexAppend<'a> {
    /// Onde cada vértice novo fica.
    pub positions: &'a [[f32; 3]],
    /// De que par cada um nasceu — o meio da aresta `a—b`.
    pub parents: &'a [(u32, u32)],
}

impl Mesh {
    /// **REESCREVE A LIGAÇÃO DE FACES EXISTENTES** — sem criar, apagar nem mover
    /// vértice nenhum.
    ///
    /// É a edição que uma troca de diagonal faz (`dyntopo_flip`), e ela existe
    /// para que essa troca **deixe de custar uma malha nova**. O caminho antigo
    /// era `Mesh::from_parts` + [`Self::rebuild`]: validar toda face, refazer os
    /// dois anéis, o octree e todas as normais — `O(malha)` medido em **10,4 ms
    /// a 98k**, para mudar duas faces.
    ///
    /// ⚠️ **A ORDEM dos três passos é o contrato**, e trocá-la não falha: dá a
    /// resposta errada em silêncio. O anel de faces é remendado contra as faces
    /// ANTIGAS (é delas que o índice sai); as faces são escritas; e só então o
    /// anel de vértices é re-derivado contra as NOVAS. Um passe de região que
    /// rodasse antes leria a adjacência velha e refrescaria a normal de quem
    /// deixou de ser vizinho.
    ///
    /// ⚠️ **O octree fica com a partição de antes, e isso é correto** — a face
    /// mudou de forma, não de dono. É a mesma situação que o
    /// [`crate::Octree::refit`] documenta para a geometria que se move, e é o
    /// `face_leaf` que a torna exata.
    pub fn relink_faces(&mut self, changes: &[(u32, Face)], scratch: &mut RegionScratch) {
        if changes.is_empty() {
            scratch.forget();
            return;
        }
        // ⚠️ Só o que MUDA é copiado. Guardar a lista inteira de faces para ter
        // o "antes" seria `O(malha)` dentro da porta que existe para não ser.
        let edits: Vec<(u32, Face, Face)> = changes
            .iter()
            .map(|&(i, new)| (i, self.faces[i as usize], new))
            .collect();
        for &(i, _, new) in &edits {
            self.faces[i as usize] = new;
        }
        let affected = self.adjacency.relink(&edits, &self.faces);
        self.refresh_region(&affected, scratch);
    }

    /// **A TOPOLOGIA MUDOU** — vértices nasceram, faces trocaram de cantos,
    /// faces nasceram. Sem reconstruir coisa alguma.
    ///
    /// É a porta que um CORTE usa, e ela existe pelo número: o caminho antigo era
    /// montar posições e faces novas e chamar [`Self::from_parts`], ou seja
    /// validar tudo, refazer os dois anéis, TODAS as normais e o octree inteiro —
    /// **10,9 ms a 98k para mudar 4% da malha**, e o custo não respondia ao
    /// tamanho do pincel.
    ///
    /// ⚠️ **Nada é renumerado, e é isso que torna a edição possível.** Os
    /// vértices novos entram no fim; uma face que mudou de forma **fica no
    /// próprio slot** e as filhas dela são apendadas. Um índice de face que o
    /// chamador guardou entre passes continua apontando para a mesma face, e o
    /// `face_leaf` do octree e os anéis de quem não foi tocado seguem válidos.
    ///
    /// ⚠️ **A ORDEM dos cinco passos é o contrato** (a mesma lei do
    /// [`Self::relink_faces`], com dois passos a mais): vértices → faces →
    /// anéis → octree → normais da região. O octree tem de absorver as faces
    /// novas ANTES do `refresh_region`, senão o `refit` que ele roda lá dentro
    /// procura a folha de uma face que ele não conhece.
    ///
    /// `added` é `(face, face de onde ela saiu)` — a mãe é o que diz ao octree em
    /// que folha a filha nasce.
    pub fn splice_topology(
        &mut self,
        verts: &VertexAppend<'_>,
        edits: &[(u32, Face)],
        added: &[(Face, u32)],
        scratch: &mut RegionScratch,
    ) {
        if verts.positions.is_empty() && edits.is_empty() && added.is_empty() {
            scratch.forget();
            return;
        }
        debug_assert_eq!(
            verts.positions.len(),
            verts.parents.len(),
            "todo vértice novo declara de que aresta nasceu"
        );
        // 1 — os vértices. ⚠️ Cor e máscara são interpoladas AQUI e não pelo
        // chamador: *o que um ponto médio herda* é uma lei só, e o dia em que
        // entrar um quinto plano por-vértice quem esquecer dele é esta função,
        // não cada operação de topologia.
        for (&p, &(a, b)) in verts.positions.iter().zip(verts.parents) {
            self.positions.push(p);
            self.normals.push([0.0, 1.0, 0.0]);
            // ⚠️ **A curvatura entra como PLACEHOLDER, e é o mesmo tratamento da
            // normal ao lado — porque as duas são DERIVADAS.** Interpolar a dos
            // pais seria inventar um número que o `refresh_region` do passo 5 vai
            // sobrescrever de qualquer jeito (o vértice novo está em `affected`
            // por construção), com a diferença de que a versão inventada estaria
            // *errada* no meio-tempo: o ponto médio de uma aresta tem a curvatura
            // do LUGAR onde ele caiu, não a média de quem o gerou.
            self.curvatures.push(0.0);
            // ⚠️ A irmã de MUNDO (`curv_world`) pelo MESMO motivo e no MESMO
            // lugar — as duas são derivadas do anel, e o anel de um vértice
            // recém-nascido só existe depois do passo 5.
            self.curv_world.push(0.0);
            if let Some(c) = self.colors.as_mut() {
                let (ca, cb) = (c[a as usize], c[b as usize]);
                c.push([
                    (ca[0] + cb[0]) * 0.5,
                    (ca[1] + cb[1]) * 0.5,
                    (ca[2] + cb[2]) * 0.5,
                ]);
            }
            if let Some(m) = self.masks.as_mut() {
                let v = (m[a as usize] + m[b as usize]) * 0.5;
                m.push(v);
            }
            // ⚠️ **O AO herda a MÉDIA dos pais, ao contrário da curvatura logo
            // acima** — e a diferença é a classificação do canal, não gosto. A
            // curvatura entra como placeholder porque o `refresh_region` a
            // reescreve; o AO **ninguém reescreve** (ele só volta com um bake),
            // então um placeholder ficaria na malha como um buraco preto no meio
            // de uma superfície assada. A média dos pais é a melhor estimativa
            // disponível — e o `baked_stale` abaixo é o que impede essa estimativa
            // de se apresentar como medição.
            if let Some(a_o) = self.ao.as_mut() {
                let v = (a_o[a as usize] + a_o[b as usize]) * 0.5;
                a_o.push(v);
            }
            // ⚠️ A ESPESSURA pelo mesmo argumento do AO, uma linha acima: ela é
            // MEDIDA e ninguém a reescreve sem um bake, então um placeholder
            // ficaria como uma janela translúcida no meio de matéria opaca.
            if let Some(th) = self.thickness.as_mut() {
                let v = (th[a as usize] + th[b as usize]) * 0.5;
                th.push(v);
            }
        }
        // A topologia mudou: o que estava assado descreve outra malha.
        self.baked_stale = true;
        // 2 — as faces. Só o que MUDA é copiado para ter o "antes".
        let changed: Vec<(u32, Face, Face)> = edits
            .iter()
            .map(|&(i, new)| (i, self.faces[i as usize], new))
            .collect();
        for &(i, _, new) in &changed {
            self.faces[i as usize] = new;
        }
        let first_new = u32::try_from(self.faces.len()).unwrap_or(u32::MAX);
        let mut added_idx: Vec<u32> = Vec::with_capacity(added.len());
        let mut births: Vec<(u32, u32)> = Vec::with_capacity(added.len());
        for (k, &(f, parent)) in added.iter().enumerate() {
            let idx = first_new + u32::try_from(k).unwrap_or(0);
            self.faces.push(f);
            added_idx.push(idx);
            births.push((idx, parent));
        }
        self.face_normals.resize(self.faces.len(), [0.0, 1.0, 0.0]);
        // 3 — os anéis.
        let affected =
            self.adjacency
                .splice(&changed, &added_idx, &self.faces, self.positions.len());
        // 4 — o índice espacial.
        self.octree
            .insert_alongside(&self.positions, &self.faces, &births);
        // 5 — normais e caixas, limitadas pela região.
        self.refresh_region(&affected, scratch);
    }
}
