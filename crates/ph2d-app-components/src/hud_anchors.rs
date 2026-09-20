//! ⭐⭐⭐ **QUEM um canvas de HUD ancora, e contra que rectângulo** (TOP-20 #20) — o item que o
//! handoff do #20 §7 deixou aberto.
//!
//! # ⛔⛔ O que a sonda do §5.0 corrigiu na redacção do item
//!
//! Ele escreve *«as quatro âncoras do `VecAnchors` não estão LIGADAS ao canvas»*. Medido, elas
//! **estão ligadas e são INERTES**: o [`ph2d_ecs::VecAnchors::delta_local`] pergunta *«a moldura
//! mudou de tamanho?»* e a caixa de um canvas é **a mesma em toda janela** — o que muda é a ESCALA
//! da raiz ⇒ o delta saía `0,0` por subtracção de iguais. *A régua media uma grandeza que não se
//! mexe.*
//!
//! A grandeza que de facto muda é a **caixa EFECTIVA** (`ph2d_hud::effective_box`): a de referência
//! crescida pela banda do letterbox, em unidades locais do canvas.
//!
//! # ⚠️ Porque a SELECÇÃO mora aqui e o desenho fica na shell
//!
//! *«quem ancora quem, e contra que rectângulo»* é uma pergunta sobre COMPONENTES, e responde-se
//! com o mundo e o mapa — sem cena, sem `LiveGeometry` e sem device, logo com gates baratos. O que
//! fica do lado de lá é só **publicar a pose**, que é a maquinaria que o passe das molduras já tem.

use ph2d_ecs::{Entity, SimWorld, VecAnchors};
use ph2d_vec_entities::entities::VecEntityMap;

use crate::hud_bridge::{View, anchor_frame_of};

/// **Um filho ancorado por um canvas**, com a moldura que ele deve ler.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Ancorado {
    /// O filho.
    pub kid: Entity,
    /// A caixa EFECTIVA do canvas, em unidades locais dele.
    pub now: [f64; 4],
    /// A escala que leva essas unidades ao mundo — a que **conduziu** a raiz neste quadro.
    pub scale: [f64; 2],
}

/// **Todos os filhos que um canvas de HUD ancora neste quadro.**
///
/// ⚠️ A varredura é pelo lado do **FILHO**, e é a mesma forma do `anchoring_frame` das molduras:
/// quem tem regra é o filho, e o pai é uma pergunta sobre ele. ⛔ Uma consulta por `UiCanvas`
/// pediria o mundo MUTÁVEL, que o passe de layout não tem.
///
/// ⚠️ **Sem vista, a lista é VAZIA** — a mesma lei que a raiz do canvas já obedece: sem câmera de
/// jogo nada é conduzido, e ancorar contra uma vista inventada poria o placar num canto que ninguém
/// escolheu.
///
/// ⚠️ A moldura mede-se **uma vez por canvas**: os filhos de um canvas vêm juntos na ordem do mapa,
/// e repetir a conta por filho seria pagá-la N vezes pelo mesmo rectângulo.
#[must_use]
pub fn ancorados(sim: &SimWorld, map: &VecEntityMap, vista: Option<View>) -> Vec<Ancorado> {
    let Some(vista) = vista else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let mut medido: Option<(Entity, [f64; 4], [f64; 2])> = None;
    for &bits in map.values() {
        let kid = Entity::from_bits(bits);
        if sim.world().get::<VecAnchors>(kid).is_none() {
            continue;
        }
        let Some((root, cfg)) = canvas_de(sim, kid) else {
            continue;
        };
        if medido.map(|(r, _, _)| r) != Some(root) {
            let Some((now, scale)) = anchor_frame_of(cfg, vista) else {
                continue; // caixa impossível — o painel é que a acusa
            };
            medido = Some((root, now, scale));
        }
        if let Some((_, now, scale)) = medido {
            out.push(Ancorado { kid, now, scale });
        }
    }
    out
}

/// **O CANVAS que ancora este filho** — `None` se o pai não é um canvas de HUD.
///
/// ⚠️⚠️ **Ela devolve o PAR, e isso nasceu de uma mutação SOBREVIVENTE:** a 1.ª redacção devolvia
/// só a entidade e o chamador voltava a perguntar `get::<UiCanvas>(root)` — logo a cerca estava
/// escrita **duas vezes**, e apagar esta era **neutralizado** pela outra. *Uma mutação neutralizada
/// por uma segunda guarda lê-se como sobrevivência num relatório e não é* — e a cura não é um gate
/// a mais, é a **segunda resposta a menos**.
///
/// ⭐ É esta porta que torna as duas populações — filho de moldura, filho de canvas — **disjuntas
/// por construção**: um filho tem UM pai, e ele ou tem `VecFrame` ou tem `UiCanvas`.
#[must_use]
fn canvas_de(sim: &SimWorld, kid: Entity) -> Option<(Entity, ph2d_ecs::UiCanvas)> {
    let w = sim.world();
    let parent = w.get::<ph2d_ecs::ChildOf>(kid)?.parent();
    let cfg = *w.get::<ph2d_ecs::UiCanvas>(parent)?;
    Some((parent, cfg))
}

#[cfg(test)]
#[path = "hud_anchors_tests.rs"]
mod tests;
