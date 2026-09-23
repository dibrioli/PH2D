//! ⭐⭐ **A FONTE VECTORIAL CONDICIONAL** — um nó cuja saída só carrega um `geometry_id` em
//! ALGUNS dos modos dele.
//!
//! O [`NodeRegistry::register_live_vector_source`] responde por TIPO: o `source.shape` e o
//! `source.text` emitem sempre uma forma viva. O `source.lsystem` não: no modo `Lines` ele emite
//! posições, e no modo `Branches` emite a fita que a shell construiu — um `geometry_id`, que o
//! cozimento na placa **não sabe desenhar** (sai como quadrados de atlas em branco).
//!
//! ⛔⛔ **Por que não registar o L-System como fonte vectorial por tipo:** a bandeira de tipo tem
//! OUTROS leitores (a lei da aparência e a das fontes de posições), e para eles um L-System em
//! `Lines` é uma fonte de POSIÇÕES. Marcá-lo pelo tipo mentiria a esses dois para curar o
//! terceiro — *uma resposta de tipo a uma pergunta de instância*.
//!
//! ⭐ **O predicado é do NÓ** (a crate que implementa o modo é quem sabe quando ele desenha uma
//! forma) e recebe os params já RESOLVIDOS (conduzido → override → default) — a escada é de quem
//! pergunta, porque só ele sabe o que conduz o quê neste quadro.
//!
//! Achado pela varredura das cenas de várias saídas contra a CPU (doc 119 §7): a `=108` tem cinco
//! L-Systems em `Branches`, corria na CPU só porque tinha várias saídas, e com a cerca do
//! multi-sink levantada a placa desenhava cinco quadrados onde a CPU desenha cinco plantas.

use crate::NodeRegistry;
use ph2d_nodegraph::node::NodeTypeId;

/// «Com ESTES params, a saída deste nó carrega um `geometry_id` vivo?» — o getter devolve o
/// valor RESOLVIDO de um param declarado do nó.
pub type LiveVectorWhen = fn(&dyn Fn(&str) -> f32) -> bool;

impl NodeRegistry {
    /// Regista `id` como fonte vectorial **em alguns modos** — ver o módulo. Aditivo; a última
    /// escrita ganha.
    pub fn register_live_vector_source_when(&mut self, id: NodeTypeId, when: LiveVectorWhen) {
        self.live_vector_when.insert(id, when);
    }

    /// **A saída deste nó, com estes params, carrega um `geometry_id` vivo?** — a pergunta de
    /// INSTÂNCIA: verdade se o tipo é sempre vectorial ([`Self::is_live_vector_source`]) ou se
    /// o predicado condicional dele diz que sim.
    #[must_use]
    pub fn emits_live_vector(&self, id: NodeTypeId, param: &dyn Fn(&str) -> f32) -> bool {
        self.is_live_vector_source(id) || self.live_vector_when.get(&id).is_some_and(|f| f(param))
    }

    /// `id` declara um predicado condicional? — para quem pergunta decidir se vale a pena
    /// resolver os params (o `emits_live_vector` pede-os).
    #[must_use]
    pub fn has_live_vector_condition(&self, id: NodeTypeId) -> bool {
        self.live_vector_when.contains_key(&id)
    }
}
