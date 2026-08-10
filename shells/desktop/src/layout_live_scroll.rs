//! **A ROLAGEM de uma moldura** — a terceira metade do passe de layout (o item 3 do estudo dos
//! contêineres).
//!
//! Módulo FILHO do [`super`], como o das âncoras, e pela mesma razão: o dono da tabela de poses
//! continua a ser UM ([`super::LayoutLive`]), e isto é só mais um braço dele. Um passe próprio
//! teria uma segunda opinião sobre onde um filho ficou.
//!
//! # O que a rolagem É, em duas frases
//!
//! O motor já **transborda** — medido (`ph2d-vec-layout::overflow_probe`): cinco filhos de 40 numa
//! moldura de 100 ficam em `y = 0, 40, 80, 120, 160`, e o excedente é 100. Eles não encolhem, e o
//! `clip` da moldura já recorta o que sai. ⇒ **rolar é deslocar os filhos por um número**, e o
//! trabalho todo é medir o excedente e herdá-lo pela árvore.
//!
//! # Três decisões, e o preço de cada uma
//!
//! **1. O excedente é DERIVADO, nunca autorado.** Uma moldura rola porque o conteúdo não cabe, e
//! não porque alguém marcou uma caixa. Um controlo ao lado seria um segundo lugar a discordar do
//! primeiro no instante em que o artista acrescentasse um filho.
//!
//! **2. O deslocamento é VISTA, e não documento.** A razão é a mesma que fez o modo de preview
//! existir ([`crate::render_loop::ui_preview`]): *o undo deste editor é por DIFF do mundo*, então
//! um deslocamento escrito no ECS faria **cada tique de roda virar um passo de undo**. O preço
//! honesto: ele não viaja no arquivo, e reabrir um projeto mostra o topo da lista.
//!
//! **3. O deslocamento entra no espaço do MOTOR**, antes do [`super::world_target`] — a troca de
//! eixo continua a acontecer num lugar só, que é o que impede a rolagem de descobrir sozinha que o
//! mundo é Y-up e errar o sinal.

use ph2d_ecs::Entity;
use ph2d_vec_layout::{Node, Solved};

/// **Uma moldura que a roda pode rolar**, com a caixa de mundo em que ela ficou — publicada pelo
/// passe, consumida pelo gesto.
pub(crate) struct ScrollTarget {
    pub(crate) frame: Entity,
    /// Profundidade na árvore do FLUXO — a régua do *mais interno ganha*.
    depth: usize,
    lo: [f64; 2],
    hi: [f64; 2],
}

impl super::LayoutLive {
    /// **Quanto o conteúdo desta moldura passa dela** — `[0, 0]` se cabe (ou se ela não flui).
    pub(crate) fn overflow_of(&self, frame: Entity) -> [f64; 2] {
        self.overflow
            .get(&frame.to_bits())
            .copied()
            .unwrap_or_default()
    }

    /// O deslocamento em vigor.
    pub(crate) fn scroll_of(&self, frame: Entity) -> [f64; 2] {
        self.scroll
            .get(&frame.to_bits())
            .copied()
            .unwrap_or_default()
    }

    /// **Rola `frame` por `d`**, clampado ao excedente que o último passe MEDIU. Devolve `true` se
    /// alguma coisa se mexeu — quem não tem excedente não rola, e o chamador usa isso para decidir
    /// se consome a roda ou a deixa passar para o zoom da câmera.
    ///
    /// ⚠️ O clamp mora aqui **e** no passe, e não é redundância: aqui ele impede o número de
    /// crescer sem limite enquanto a roda gira; lá ele responde ao conteúdo que ENCOLHEU depois
    /// (apagar cinco filhos deixaria a lista rolada para fora de si mesma, a mostrar vazio).
    pub(crate) fn scroll_by(&mut self, frame: Entity, d: [f64; 2]) -> bool {
        let max = self.overflow_of(frame);
        let cur = self.scroll_of(frame);
        let next = [
            (cur[0] + d[0]).clamp(0.0, max[0].max(0.0)),
            (cur[1] + d[1]).clamp(0.0, max[1].max(0.0)),
        ];
        if next == cur {
            return false;
        }
        self.scroll.insert(frame.to_bits(), next);
        true
    }

    /// **Que moldura ROLÁVEL está sob este ponto de mundo** — a mais INTERNA, ou `None`.
    ///
    /// ⚠️ Ela varre o que o PASSE PUBLICOU, e não re-deriva nada — a mesma lei do [`FlowSlots`],
    /// e aqui ela é obrigatória por duas razões que se somam: o handler da roda corre **fora** do
    /// frame (não tem a `LiveGeometry` na mão), e uma moldura que ABRAÇA mudou de tamanho neste
    /// frame — uma segunda medição poria o alvo da roda onde a moldura *estava*.
    ///
    /// [`FlowSlots`]: super::FlowSlots
    pub(crate) fn scrollable_frame_at(&self, p: [f64; 2]) -> Option<Entity> {
        self.scrollables
            .iter()
            .filter(|t| p[0] >= t.lo[0] && p[0] <= t.hi[0] && p[1] >= t.lo[1] && p[1] <= t.hi[1])
            // **A mais INTERNA ganha** — uma lista dentro de um card: a roda pertence à lista.
            .max_by_key(|t| t.depth)
            .map(|t| t.frame)
    }

    /// **Publica uma moldura como alvo da roda** — chamado pelo passe, no laço que coloca.
    ///
    /// Duas condições, e cada uma tira um caso que seria um defeito:
    ///
    /// 1. **ela RECORTA** — sem recorte o conteúdo excedente está à vista, e rolar moveria formas
    ///    visíveis sob o cursor sem que nada as escondesse;
    /// 2. **ela TRANSBORDA** — é o que torna a roda um gesto raro e deliberado em vez de um roubo
    ///    do zoom da câmera, que é o que o artista faz o tempo todo.
    pub(super) fn offer_scroll_target(
        &mut self,
        frame: Entity,
        clips: bool,
        depth: usize,
        target: ([f64; 2], [f64; 2]),
    ) {
        if !clips || self.overflow_of(frame) == [0.0, 0.0] {
            return;
        }
        self.scrollables.push(ScrollTarget {
            frame,
            depth,
            lo: target.0,
            hi: target.1,
        });
    }

    /// **Quanto o conteúdo de cada nó que FLUI passa dele** — a primeira das duas passadas.
    ///
    /// O `solved` é relativo à RAIZ, então o alcance de um filho dentro do pai é a diferença entre
    /// as duas origens. ⚠️ E o **recuo do lado final entra**: um card com 8 de recuo em baixo cujo
    /// último item encosta no fundo tem 8 de conteúdo que o excedente precisa de conhecer, senão a
    /// rolagem para 8 unidades antes do fim e a lista parece cortada.
    pub(super) fn measure_overflow(
        &mut self,
        nodes: &[Node],
        solved: &[Solved],
        ents: &[Option<Entity>],
    ) {
        let mut over = vec![[0.0_f64; 2]; nodes.len()];
        for (i, n) in nodes.iter().enumerate() {
            let Some(p) = n.parent else { continue };
            for a in 0..2 {
                // O lado final do recuo do PAI: direita para x, base para y (a ordem do CSS).
                let pad = nodes[p]
                    .frame
                    .map_or(0.0, |f| f.pad[if a == 0 { 1 } else { 2 }]);
                let far = (solved[i][a] + solved[i][a + 2]) - solved[p][a] + pad;
                over[p][a] = over[p][a].max(far - solved[p][a + 2]);
            }
        }
        for (i, o) in over.into_iter().enumerate() {
            // ⚠️ Só quem FLUI tem conteúdo a transbordar, e o negativo (sobra) **não é guardado**:
            // quem cabe não rola, e um número negativo ali seria um alcance ao contrário.
            //
            // ⚠️ **Este `max` e o acumulador que começa em zero são a MESMA defesa, medida:** com o
            // acumulador em `0.0`, um `max` contra um negativo já devolve zero, e mutar só um dos
            // dois **não é observável por gate nenhum**. A mutação que morde ataca os dois (o
            // acumulador em `NEG_INFINITY` *e* este `max`), e aí sangram três — é o precedente das
            // defesas em camada: uma camada sozinha não tem gate porque não tem efeito.

            if nodes[i].frame.is_none() {
                continue;
            }
            let Some(e) = ents[i] else { continue };
            let v = [o[0].max(0.0), o[1].max(0.0)];
            if v != [0.0, 0.0] {
                self.overflow.insert(e.to_bits(), v);
            }
        }
    }

    /// **O deslocamento que cada nó HERDA** — a segunda passada, `O(nós)` porque a fatia vem com o
    /// pai antes dos filhos (é o que o `solve` exige).
    ///
    /// ⚠️ Um nó é deslocado pelo scroll dos ANCESTRAIS, nunca pelo próprio: rolar uma lista move o
    /// que está dentro dela, e não a lista. É por isso que a moldura fica onde está — e é o que
    /// distingue *rolar* de *arrastar a moldura*.
    pub(super) fn scroll_offsets(&self, nodes: &[Node], ents: &[Option<Entity>]) -> Vec<[f64; 2]> {
        let mut off = vec![[0.0_f64; 2]; nodes.len()];
        for i in 0..nodes.len() {
            let Some(p) = nodes[i].parent else { continue };
            let s = ents[p].map_or([0.0; 2], |e| {
                let max = self.overflow_of(e);
                let cur = self.scroll_of(e);
                [cur[0].clamp(0.0, max[0]), cur[1].clamp(0.0, max[1])]
            });
            off[i] = [off[p][0] + s[0], off[p][1] + s[1]];
        }
        off
    }
}

#[cfg(test)]
#[path = "layout_live_scroll_tests.rs"]
mod tests;
