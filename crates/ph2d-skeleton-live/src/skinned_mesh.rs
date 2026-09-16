//! ⭐⭐⭐ **A MALHA DE UMA IMAGEM PRESA, COM OS PESOS DENTRO** — o que o `SkinBind::source` guarda
//! desde que a pele de imagem passou a usar o padrão-ouro.
//!
//! # Porque os pesos passaram a ser GUARDADOS
//!
//! ⛔ **A lei deste repo era o contrário, e está escrita:** *«os pesos não se guardam, derivam-se —
//! uma tabela por ordem de varredura é o vector paralelo que o `corner_radius` proíbe»*. Ela foi
//! escrita para a lei **euclidiana** (`bump` sobre a distância ao eixo), que é uma **função de um
//! ponto**: derivar custava `0,146 %` de um quadro e guardar não comprava nada.
//!
//! ⭐ **Os [*Bounded Biharmonic Weights*](ph2d_skin_weights) não são função de um ponto.** Eles são
//! a solução de um problema variacional **sobre a arte inteira** (minimizar `∫‖Δw‖²` com caixas e
//! Dirichlet nos ossos), e resolvê-lo custa dezenas de milissegundos. ⇒ *quem quer o padrão-ouro
//! resolve uma vez, ao prender, e guarda* — que é o que Spine, Blender e Maya fazem.
//!
//! # ⭐⭐⭐ E eles NÃO são um vector paralelo: são o MESMO objecto
//!
//! A objecção da lei antiga era real — *duas listas que uma edição pode dessincronizar*. A cura é
//! não ter duas listas: a malha e os pesos viajam **numa struct só**, o comprimento de uma é
//! função do da outra, e a porta [`SkinnedMesh::valida`] recusa o par que não fecha. ⛔ Não há
//! como gravar uma e não a outra, porque não há duas coisas para gravar.
//!
//! ⚠️ **O número de ossos DERIVA-SE** ([`SkinnedMesh::ossos`]) — guardá-lo seria a terceira
//! grandeza a poder discordar das outras duas, exactamente o defeito que a
//! [`ph2d_poly2d::Mesh2d`] já evita ao derivar a UV do tamanho.
//!
//! # ⚠️ A tabela é por TENDÃO, nunca por pose resolvida
//!
//! Um osso que dobra dá `N` poses à pele, e um osso apagado dá **zero**. A coluna `j` desta tabela
//! é o osso **autorado** — o `j`-ésimo [`ph2d_skeleton_ecs::Tendon`] da mesma [`SkinBind`] —, e é o
//! [`ph2d_skeleton::SkinBone::tendon`] que reencontra as poses dele no quadro. *Guardar por pose
//! resolvida faria a arte saltar no dia em que alguém apagasse um osso.*
//!
//! [`SkinBind`]: ph2d_skeleton_ecs::SkinBind

use ph2d_poly2d::Mesh2d;

/// ⭐⭐⭐ **A malha de repouso de uma imagem mais a fracção de cada vértice que pertence a cada osso.**
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SkinnedMesh {
    /// A malha em pixels da imagem, no repouso.
    pub mesh: Mesh2d,
    /// **Achatado**: `pesos[v * ossos + j]` é a fracção do vértice `v` que pertence ao tendão `j`.
    ///
    /// ⚠️ **Achatado e não `Vec<Vec<f64>>`**, e o motivo é o quadro: o refinamento do `Smooth`
    /// interpola isto por vértice inventado, e um `Vec` por vértice seria uma alocação por vértice
    /// **por quadro**. A fatia contígua atravessa a [`ph2d_poly2d::refine_posed_attrs`] sem alocar
    /// nada além do buffer de saída.
    ///
    /// ⭐ **Vazio é legal e significa *«esta malha não traz pesos»***: a jusante ela cai na lei
    /// derivada, que é o caminho da 1.ª mídia. ⛔ Não é um erro silencioso — é o único estado em
    /// que uma malha gravada antes desta wave pode chegar aqui, e ele é nomeado.
    pub pesos: Vec<f64>,
}

impl SkinnedMesh {
    /// A malha sem pesos — a leitura de *«resolve pela lei derivada»*.
    #[must_use]
    pub fn sem_pesos(mesh: Mesh2d) -> Self {
        Self {
            mesh,
            pesos: Vec::new(),
        }
    }

    /// ⭐ **Quantos ossos a tabela cobre** — `0` quando não há tabela.
    ///
    /// ⚠️ **DERIVADO**, e é isso que impede uma terceira grandeza de discordar das outras duas.
    #[must_use]
    pub fn ossos(&self) -> usize {
        self.pesos
            .len()
            .checked_div(self.mesh.rest.len())
            .unwrap_or(0)
    }

    /// ⛔ **O par fecha?** — `pesos` tem de ser um múltiplo exacto do número de vértices.
    ///
    /// ⚠️ Um resto diferente de zero quer dizer que a malha e a tabela descrevem coisas
    /// diferentes, e a resposta certa é **recusar**, nunca ler a tabela deslocada: uma tabela
    /// deslocada por um vértice dá pesos plausíveis e arte errada.
    #[must_use]
    pub fn valida(&self) -> bool {
        let n = self.mesh.rest.len();
        n > 0 && self.pesos.len().is_multiple_of(n)
    }

    /// Os pesos do vértice `v`. Vazio quando não há tabela — ⛔ nunca uma fatia de outro vértice.
    #[must_use]
    pub fn pesos_de(&self, v: usize) -> &[f64] {
        let m = self.ossos();
        if m == 0 {
            return &[];
        }
        self.pesos.get(v * m..(v + 1) * m).unwrap_or(&[])
    }
}

#[cfg(test)]
#[path = "skinned_mesh_tests.rs"]
mod tests;
