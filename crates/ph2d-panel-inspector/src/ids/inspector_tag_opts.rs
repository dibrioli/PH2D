//! **As opções das quatro listas de tags** (Nav Agent, filtro do Ray Sensor, Signal Action, Physics) —
//! um id por tag da ÁRVORE, sem tecto (U5, auditoria GEF 07: a árvore não tem tecto, `9 344` tags
//! medidas na W1; o popover rola).
//!
//! O id da opção `n` é o molde `prefixo{n}` (o do `asset_cell_id`), memorizado à medida que a árvore
//! cresce; o painel regista-o no store quando a lista se pinta (`popovers_tags::publica`), e o despacho
//! acha a posição pela mesma tabela. ⛔ Ids PRÓPRIOS por lista: o despacho resolve a opção pela posição,
//! e com a mesma tabela dois braços casariam.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id_runtime;
use std::sync::Mutex;

/// A tabela crescente de uma lista de tags.
pub struct OpcoesDeTag {
    molde: fn(usize) -> NodeId,
    feitos: Mutex<Vec<NodeId>>,
}

impl OpcoesDeTag {
    const fn new(molde: fn(usize) -> NodeId) -> Self {
        Self {
            molde,
            feitos: Mutex::new(Vec::new()),
        }
    }

    /// O id da opção `n` (a `n`-ésima tag da árvore).
    pub fn id(&self, n: usize) -> NodeId {
        (self.molde)(n)
    }

    /// Os ids das primeiras `len` opções.
    pub fn ids(&self, len: usize) -> Vec<NodeId> {
        self.com(len, |t| t.to_vec())
    }

    /// A posição de `id` entre as primeiras `len` opções.
    pub fn posicao(&self, id: NodeId, len: usize) -> Option<usize> {
        self.com(len, |t| t.iter().position(|&o| o == id))
    }

    fn com<R>(&self, len: usize, f: impl FnOnce(&[NodeId]) -> R) -> R {
        let mut t = self.feitos.lock().unwrap_or_else(|e| e.into_inner());
        for n in t.len()..len {
            t.push((self.molde)(n));
        }
        f(&t[..len])
    }
}

fn nav(n: usize) -> NodeId {
    hash_node_id_runtime(&format!("insp_nav_tag_opt_{n}"))
}
fn ray(n: usize) -> NodeId {
    hash_node_id_runtime(&format!("insp_ray_see_tag_opt_{n}"))
}
fn action(n: usize) -> NodeId {
    hash_node_id_runtime(&format!("insp_action_tag_opt_{n}"))
}
fn phys(n: usize) -> NodeId {
    hash_node_id_runtime(&format!("insp_phys_tag_opt_{n}"))
}

/// A tag do modo `Tag` do Nav Agent.
pub static INSP_NAV_TAG_OPT: OpcoesDeTag = OpcoesDeTag::new(nav);
/// A tag do filtro do Ray Sensor.
pub static INSP_RAY_SEE_TAG_OPT: OpcoesDeTag = OpcoesDeTag::new(ray);
/// A tag alvo de uma Signal Action.
pub static INSP_ACTION_TAG_OPT: OpcoesDeTag = OpcoesDeTag::new(action);
/// O filtro por tag dos sinais da §11 Physics (o `(any)` à frente tem id próprio).
pub static INSP_PHYS_TAG_OPT: OpcoesDeTag = OpcoesDeTag::new(phys);
