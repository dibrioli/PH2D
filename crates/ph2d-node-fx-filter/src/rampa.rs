//! **O degradê do Gradient Map, ASSADO** — o param de TEXTO [`crate::param::RAMP`] (uma `ColorRamp`)
//! vira a tabela que o passe lê. Saiu do `lib.rs` pelo tecto de LOC (ZERAR família B).

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use ph2d_fx_op::FxOp;
use ph2d_nodegraph::graph::{Graph, NodeId};

use crate::{Knob, kind_of, param, takes};

/// **O degradê do nó `no`, ASSADO** em [`FxOp::RAMP_LUT_TEXELS`] amostras uniformes de `[0,1]`
/// (RGBA linear reto, o alfa é a força) — `ColorRamp::eval` honra as interpolações, os espaços e as
/// paradas que o editor oferece; o passe só reconstrói entre vizinhas. `None` sem texto, com texto
/// que não parse (um documento futuro não espalha `NaN`), ou num tipo que não lê a rampa — e aí a
/// `op` desenha os seus stops, o preto→branco linear que é também o `ColorRamp::default` que o
/// editor mostra.
#[must_use]
pub fn bake_ramp(graph: &Graph, no: NodeId) -> Option<Arc<[[f32; 4]]>> {
    ramp_text(graph, no).and_then(assa)
}

/// O texto do degradê do nó, se o tipo dele lê a rampa.
pub(crate) fn ramp_text(graph: &Graph, no: NodeId) -> Option<&str> {
    if !takes(
        usize::from(kind_of(graph.node_param_overrides(no))),
        Knob::Ramp,
    ) {
        return None;
    }
    graph
        .node_text_param_overrides(no)
        .and_then(|m| m.get(param::RAMP))
        .map(String::as_str)
}

fn assa(texto: &str) -> Option<Arc<[[f32; 4]]>> {
    let ramp = ph2d_color::parse_gradient(texto)?;
    Some(ph2d_color::ramp_lut::bake(&ramp, FxOp::RAMP_LUT_TEXELS).into())
}

/// **Os degradês ASSADOS entre quadros, por texto** — assar são [`FxOp::RAMP_LUT_TEXELS`]
/// avaliações da `ColorRamp` e o texto só muda num gesto; a pilha lê-se a cada quadro. Guarda só
/// os textos da última leitura.
#[derive(Default)]
pub struct RampasAssadas(BTreeMap<String, Arc<[[f32; 4]]>>);

impl RampasAssadas {
    /// O degradê assado do `texto` — da memória, ou assado agora e guardado.
    pub(crate) fn pega(&mut self, texto: &str) -> Option<Arc<[[f32; 4]]>> {
        if let Some(a) = self.0.get(texto) {
            return Some(Arc::clone(a));
        }
        let a = assa(texto)?;
        self.0.insert(texto.to_string(), Arc::clone(&a));
        Some(a)
    }

    /// Esquece os textos que a última leitura não usou.
    pub(crate) fn guarda_so(&mut self, usados: &BTreeSet<&str>) {
        self.0.retain(|t, _| usados.contains(t.as_str()));
    }

    /// Quantos degradês estão guardados.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Nenhum guardado.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
