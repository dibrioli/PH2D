//! **O id estável de um vértice** (W4.2, `VEC_SCENE` v24) — o endereço que uma regra de CAD, o Copy
//! Point e a medida animada guardam, e que sobrevive a inserir, apagar, inverter, soldar e partir.
//!
//! Mora DENTRO do [`VecVertex`] (a lei do `corner_radius`): as operações que movem vértices levam-no de
//! graça. Quem DÁ ids é [`VecPath::normaliza_ids`], chamado por [`crate::VecScene::push_path`] e pelo
//! gancho que corre antes de toda captura de undo — uma operação só tem de copiar o vértice (mantém o
//! id) ou construí-lo novo ([`VertId::NOVO`]). Desenho: `docs/BlenderConstraints/04_W4_cad.md` §3.1.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{VecPath, VecScene, VecVertex, VertexKind};

/// Id de um vértice dentro do seu path (os contornos de `subpaths` partilham o espaço). Monotónico
/// por path, nunca reusado (`VecPath::next_vert`).
#[derive(
    Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct VertId(pub u32);

impl VertId {
    /// «Ainda sem id» — o que todo construtor dá; o próximo `normaliza_ids` atribui.
    pub const NOVO: Self = Self(0);
}

impl VecVertex {
    /// Vértice de quina reto: handles coincidentes com a âncora (sem curvatura).
    pub fn corner(anchor: [f64; 2]) -> Self {
        Self::with_handles(anchor, anchor, anchor, VertexKind::Corner)
    }

    /// Vértice suave com handles absolutos explícitos.
    pub fn smooth(anchor: [f64; 2], in_handle: [f64; 2], out_handle: [f64; 2]) -> Self {
        Self::with_handles(anchor, in_handle, out_handle, VertexKind::Smooth)
    }

    /// Vértice com handles e natureza explícitos, sem raio de quina e sem id.
    pub fn with_handles(
        anchor: [f64; 2],
        in_handle: [f64; 2],
        out_handle: [f64; 2],
        kind: VertexKind,
    ) -> Self {
        Self {
            anchor,
            in_handle,
            out_handle,
            kind,
            corner_radius: 0.0,
            id: VertId::NOVO,
        }
    }
}

impl VecPath {
    /// Dá `next_vert` a cada [`VertId::NOVO`] e à 2.ª ocorrência de um id repetido, pela ordem
    /// `verts` → `subpaths`; um id vindo de fora acima do contador sobe-o. Idempotente; devolve se
    /// mudou algo.
    pub fn normaliza_ids(&mut self) -> bool {
        let todos = || {
            self.verts
                .iter()
                .chain(self.subpaths.iter().flat_map(|c| &c.verts))
        };
        let maior = todos().map(|v| v.id.0).max().unwrap_or(0);
        let mut mudou = false;
        if self.next_vert <= maior {
            self.next_vert = maior + 1;
            mudou = true;
        }
        let mut proximo = self.next_vert.max(1);
        let mut vistos = BTreeSet::new();
        let contornos =
            std::iter::once(&mut self.verts).chain(self.subpaths.iter_mut().map(|c| &mut c.verts));
        for verts in contornos {
            for v in verts {
                if v.id == VertId::NOVO || !vistos.insert(v.id) {
                    v.id = VertId(proximo);
                    vistos.insert(v.id);
                    proximo += 1;
                    mudou = true;
                }
            }
        }
        mudou |= self.next_vert != proximo;
        self.next_vert = proximo;
        mudou
    }

    /// Um vértice [`VertId::NOVO`] herda o id que o MESMO lugar (contorno e índice) tinha em `anterior`,
    /// se esse id não está em uso neste path; depois [`Self::normaliza_ids`]. Uma forma re-gerada com a
    /// mesma topologia (forma viva, envelope, conector) guarda os ids — e o undo não vê mudança —, e uma
    /// inserção ganha id novo, porque os vizinhos deslocados ainda usam os deles.
    pub fn normaliza_ids_desde(&mut self, anterior: &Self) -> bool {
        let mut usados: BTreeSet<VertId> = std::iter::once(&self.verts)
            .chain(self.subpaths.iter().map(|c| &c.verts))
            .flatten()
            .map(|v| v.id)
            .collect();
        let antes =
            std::iter::once(&anterior.verts).chain(anterior.subpaths.iter().map(|c| &c.verts));
        let agora =
            std::iter::once(&mut self.verts).chain(self.subpaths.iter_mut().map(|c| &mut c.verts));
        let mut mudou = false;
        for (agora, antes) in agora.zip(antes) {
            for (v, a) in agora.iter_mut().zip(antes) {
                if v.id == VertId::NOVO && a.id != VertId::NOVO && usados.insert(a.id) {
                    v.id = a.id;
                    mudou = true;
                }
            }
        }
        self.normaliza_ids() | mudou
    }

    /// O vértice de id `id`, no contorno primário ou num `subpath`.
    #[must_use]
    pub fn vert_by_id(&self, id: VertId) -> Option<&VecVertex> {
        if id == VertId::NOVO {
            return None;
        }
        self.verts
            .iter()
            .chain(self.subpaths.iter().flat_map(|c| &c.verts))
            .find(|v| v.id == id)
    }

    /// O irmão mutável de [`Self::vert_by_id`].
    pub fn vert_by_id_mut(&mut self, id: VertId) -> Option<&mut VecVertex> {
        if id == VertId::NOVO {
            return None;
        }
        self.verts
            .iter_mut()
            .chain(self.subpaths.iter_mut().flat_map(|c| &mut c.verts))
            .find(|v| v.id == id)
    }
}

impl VecScene {
    /// Dá ids a todo vértice ainda sem um, em todos os paths ([`VecPath::normaliza_ids`]).
    pub fn normaliza_ids(&mut self) -> bool {
        let mudou = self
            .paths
            .iter_mut()
            .fold(false, |m, p| p.normaliza_ids() | m);
        self.assenta_regras() | mudou
    }

    /// [`Self::normaliza_ids`] herdando pela posição os ids de `anterior` (a última fotografia do undo,
    /// [`VecPath::normaliza_ids_desde`]); um path que não estava lá só se normaliza. Tira as regras de CAD que
    /// citam um ponto que sumiu e dá id às novas ([`VecScene::assenta_regras`]).
    pub fn normaliza_ids_desde(&mut self, anterior: Option<&Self>) -> bool {
        let mudou = self.paths.iter_mut().fold(false, |m, p| {
            let mudou = match anterior.and_then(|a| a.path(p.id)) {
                Some(a) => p.normaliza_ids_desde(a),
                None => p.normaliza_ids(),
            };
            mudou | m
        });
        self.assenta_regras() | mudou
    }
}

#[cfg(test)]
#[path = "vert_id_tests.rs"]
mod tests;
