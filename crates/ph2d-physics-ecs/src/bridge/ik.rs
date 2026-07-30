//! **A sessão de POSE** (W-IK) — a ponte entre o gesto do artista e a árvore
//! transitória de cinemática inversa do wrapper.
//!
//! O núcleo (`ph2d_physics::world::ik`) sabe resolver uma árvore que alguém lhe
//! entrega. Este módulo responde as duas perguntas que só a CENA pode responder:
//! **quem é a raiz** e **quais arestas existem** — e depois guarda a árvore
//! enquanto o gesto dura.
//!
//! # A raiz é o que a cena já diz que é fixo
//!
//! Uma árvore de multibody tem uma raiz, e a raiz é o que **não se move** ao
//! posar. A cena já carrega essa informação: um corpo `Static` é uma parede e um
//! `Kinematic` segue uma curva — nenhum dos dois pode ser reposicionado por um
//! solver de pose. Então:
//!
//! 1. A partir da PONTA, caminhe pelo grafo de joints RÍGIDOS, conduzindo só
//!    por corpos `Dynamic`.
//! 2. Se a travessia encostar num corpo não-dinâmico, **ele é a raiz** — é o
//!    gancho do pêndulo, a pelve presa, a parede. O mais RASO vence (o primeiro
//!    alcançado por BFS a partir da ponta), porque é ele que a cadeia de fato
//!    pendura; empate desfeito pelos bits da entidade, para o plano ser
//!    determinístico.
//! 3. Sem nenhum, a cadeia flutua: a raiz é o corpo dinâmico **mais distante**
//!    da ponta, e o rapier lhe dá uma raiz LIVRE (3 graus de liberdade) — a IK
//!    pode transladar o conjunto inteiro, que é o que faz sentido para um rig
//!    solto.
//!
//! É a mesma família de leis do [`crate::jointed_group`] (o bake) e do
//! [`crate::jointed_rig`] (o arrasto): *quem conduz* muda com a pergunta, e a
//! pergunta aqui é **quem pode dobrar**.
//!
//! # Lê o estado RECONCILIADO, não o ECS autorado
//!
//! Ao contrário do `jointed_group`, esta travessia lê `self.joints` /
//! `self.bodies` — o que a ponte de fato construiu. É deliberado e é o oposto do
//! trade que o bake fez:
//!
//! - o bake precisa de um joint feito **neste frame** (o artista clica Join e
//!   depois Bake), então lê o ECS;
//! - a pose reusa o **`JointDesc` que o solver usa** (âncoras já semeadas, já
//!   `clamped()`, já com a política de tipo aplicada). Uma segunda derivação
//!   aqui seria uma segunda resposta a *"onde este joint prende?"*, e as duas
//!   divergiriam no dia em que a política de âncora mudasse.
//!
//! O preço, nomeado: um joint criado no MESMO frame do gesto não entra na
//! árvore. Um gesto começa num pointer-down depois de muitos dispatches, então
//! isso não é alcançável pela mão — mas é uma premissa, e premissa não escrita é
//! a que apodrece.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use ph2d_ecs::Entity;
use ph2d_physics::{IkChain, IkLink, IkOptions, RigidBodyHandle, is_rigid_link};

use crate::components::BodyKind;

use super::PhysicsBridge;

/// Uma sessão de pose viva: a árvore transitória e o que o gesto está fazendo.
///
/// Nasce no press, morre no release. Nada aqui é estado da cena — o resultado
/// de cada solve vai para o `Transform` AUTORADO, que é o que a cena guarda.
pub struct IkSession {
    chain: IkChain,
    /// O corpo que o gesto arrasta.
    pub tip: Entity,
    /// Todo corpo da árvore, raiz inclusa — o que o chamador precisa saber para
    /// tirar um snapshot de undo ANTES de a primeira pose ser escrita.
    pub bodies: Vec<Entity>,
    /// `handle → entidade`, para traduzir a resposta do wrapper de volta.
    by_handle: BTreeMap<u32, Entity>,
    /// **Os handles com que a árvore foi CONSTRUÍDA**, para saber quando ela
    /// deixou de descrever o mundo — ver [`PhysicsBridge::ik_move`].
    built_with: Vec<(Entity, RigidBodyHandle)>,
}

impl IkSession {
    /// Quantos corpos a pose move. Um por elo, raiz inclusa.
    #[must_use]
    pub fn len(&self) -> usize {
        self.bodies.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bodies.is_empty()
    }
}

/// O plano da árvore: a raiz e as arestas em ordem de BFS (todo pai antes do
/// filho, que é o que `ik_chain` exige).
///
/// Tipo próprio, e público, porque *quem é a raiz* é a decisão inteira desta
/// wave e um gate tem de poder afirmá-la sem montar um multibody.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IkPlan {
    pub root: Entity,
    /// `(pai, filho, joint)` — a entidade-joint vai junto porque é ela que
    /// carrega o `JointDesc`, e porque um gate que só visse corpos não
    /// distinguiria duas arestas paralelas.
    pub edges: Vec<(Entity, Entity, Entity)>,
}

impl PhysicsBridge {
    /// **Há uma sessão de pose em voo?** A shell pergunta isto antes de tratar
    /// um Move como qualquer outra coisa.
    #[must_use]
    pub fn is_posing(&self) -> bool {
        self.ik.is_some()
    }

    /// A ponta que o gesto vivo arrasta, se houver gesto.
    #[must_use]
    pub fn posing_tip(&self) -> Option<Entity> {
        self.ik.as_ref().map(|s| s.tip)
    }

    /// Todo corpo que a pose viva move — o que o chamador precisa para abrir UM
    /// passo de undo cobrindo a cadeia inteira.
    #[must_use]
    pub fn posing_bodies(&self) -> &[Entity] {
        self.ik.as_ref().map_or(&[], |s| &s.bodies)
    }

    /// **Monta o plano da árvore** a partir de `tip`. Puro sobre o estado
    /// reconciliado (ver as notas do módulo), então um gate o afirma sem
    /// resolver nada.
    ///
    /// `None` quando não há o que posar: a ponta não é um corpo, não é
    /// dinâmica, ou não tem joint rígido nenhum.
    #[must_use]
    pub fn ik_plan(&self, tip: Entity) -> Option<IkPlan> {
        if self.bodies.get(&tip)?.kind != BodyKind::Dynamic {
            return None;
        }
        // Adjacência sobre joints RÍGIDOS. Uma mola e uma corda não são elos
        // (a pose delas é resultado de forças), então nem entram no grafo —
        // `is_rigid_link` é a porta única, no wrapper, ao lado da função que
        // constrói o joint reduzido.
        let mut adj: BTreeMap<Entity, Vec<(Entity, Entity)>> = BTreeMap::new();
        for (&je, jr) in &self.joints {
            if !is_rigid_link(jr.rest.kind) {
                continue;
            }
            // ⚠️ Um joint preso ao MUNDO (W-JointWorld) não contribui ARESTA —
            // é a mesma lei do rig walk (`joint_group.rs`): a adjacência é entre
            // dois CORPOS, e o cenário não é um deles. O corpo preso continua na
            // árvore pelos outros joints dele; o que ele não faz é a pose andar
            // para dentro da parede.
            //
            // ⚠️ E isto NÃO é o mesmo que dizer que um pino de mundo é inútil
            // para posar: ele é a raiz natural de uma cadeia, e a `IkPlan` já
            // tem política de raiz própria. Ensiná-la a preferir um corpo
            // ancorado é wave própria, nomeada no plano 02 §9.3.
            let (a, Some(b)) = jr.entities else {
                continue;
            };
            adj.entry(a).or_default().push((b, je));
            adj.entry(b).or_default().push((a, je));
        }

        // BFS a partir da ponta, conduzindo só por Dynamic. O primeiro
        // não-dinâmico alcançado é a raiz; sem nenhum, é o dinâmico mais
        // profundo. Um único percurso responde as duas coisas.
        let mut depth: BTreeMap<Entity, u32> = BTreeMap::new();
        depth.insert(tip, 0);
        let mut queue = VecDeque::from([tip]);
        let mut anchor: Option<(u32, Entity)> = None;
        let mut deepest = (0u32, tip);
        while let Some(e) = queue.pop_front() {
            let d = depth[&e];
            if d > deepest.0 {
                deepest = (d, e);
            }
            let Some(ns) = adj.get(&e) else { continue };
            for &(n, _) in ns {
                if depth.contains_key(&n) {
                    continue;
                }
                match self.bodies.get(&n).map(|b| b.kind) {
                    Some(BodyKind::Dynamic) => {
                        depth.insert(n, d + 1);
                        queue.push_back(n);
                    }
                    // Uma parede é alcançada e não é atravessada — a mesma
                    // fronteira do `jointed_group`. Aqui, porém, ela vira a
                    // RAIZ em vez de ser descartada: é justamente o que não se
                    // move.
                    Some(_) => {
                        let cand = (d + 1, n);
                        if anchor.is_none_or(|a| cand < a) {
                            anchor = Some(cand);
                        }
                    }
                    None => {}
                }
            }
        }
        let root = anchor.map_or(deepest.1, |(_, e)| e);
        if root == tip {
            // Uma cadeia de um corpo só: nada a dobrar. Arrastar isto é uma
            // TRANSLAÇÃO, e essa já é a W-JG.
            return None;
        }

        // Segundo BFS, agora a partir da RAIZ, produzindo as arestas na ordem
        // pai→filho. Só os corpos que o primeiro percurso alcançou participam
        // (mais a raiz), então um segundo braço do rig que não leva à ponta não
        // entra — a árvore descreve o que o gesto move.
        let mut in_tree: BTreeSet<Entity> = depth.keys().copied().collect();
        in_tree.insert(root);
        let mut seen = BTreeSet::from([root]);
        let mut q2 = VecDeque::from([root]);
        let mut edges = Vec::new();
        while let Some(e) = q2.pop_front() {
            let Some(ns) = adj.get(&e) else { continue };
            for &(n, je) in ns {
                if !in_tree.contains(&n) || !seen.insert(n) {
                    continue;
                }
                edges.push((e, n, je));
                q2.push_back(n);
            }
        }
        if edges.is_empty() {
            return None;
        }
        Some(IkPlan { root, edges })
    }

    /// **Abre a sessão de pose.** `true` se a cadeia foi montada.
    ///
    /// Toda a autoridade sobre *quando* isto pode acontecer é do chamador (a
    /// shell vê o relógio e a ferramenta); aqui a única recusa é estrutural.
    pub fn ik_begin(&mut self, tip: Entity) -> bool {
        self.ik = self.build_ik_session(tip);
        self.ik.is_some()
    }

    /// Monta a sessão. Separado do [`Self::ik_begin`] porque o `ik_move` também
    /// a monta — ver a nota de RE-CONSTRUÇÃO lá.
    fn build_ik_session(&self, tip: Entity) -> Option<IkSession> {
        let plan = self.ik_plan(tip)?;
        let mut links = Vec::with_capacity(plan.edges.len());
        let mut by_handle = BTreeMap::new();
        let mut bodies = vec![plan.root];
        let root_h = self.bodies.get(&plan.root).map(|b| b.handle)?;
        let mut built_with = vec![(plan.root, root_h)];
        by_handle.insert(root_h.0.into_raw_parts().0, plan.root);
        for &(parent, child, je) in &plan.edges {
            let (Some(ph), Some(ch)) = (
                self.bodies.get(&parent).map(|b| b.handle),
                self.bodies.get(&child).map(|b| b.handle),
            ) else {
                return None;
            };
            let jr = self.joints.get(&je)?;
            let desc = jr.rest;
            // ⚠️ A ORIENTAÇÃO importa: o `JointDesc` guarda as âncoras na ordem
            // (corpo A, corpo B) do joint AUTORADO, e a árvore precisa delas na
            // ordem (pai, filho). Quando o BFS chega pelo lado B, as duas trocam
            // — e trocá-las é o bug que pendura o elo pela ponta errada.
            let desc = if jr.entities == (parent, Some(child)) {
                desc
            } else {
                swap_anchors(desc)
            };
            links.push(IkLink {
                parent: ph,
                child: ch,
                joint: desc,
            });
            by_handle.insert(ch.0.into_raw_parts().0, child);
            bodies.push(child);
            built_with.push((child, ch));
        }
        let tip_h = self.bodies.get(&tip).map(|b| b.handle)?;
        let chain = self.world.ik_chain(root_h, &links, tip_h)?;
        Some(IkSession {
            chain,
            tip,
            bodies,
            by_handle,
            built_with,
        })
    }

    /// **Resolve para o alvo e devolve a pose de cada corpo da árvore.**
    ///
    /// Vazio sem sessão. Quem ESCREVE o `Transform` é o chamador — a física não
    /// autora a cena, e é isso que mantém o passo de undo e a hierarquia com um
    /// dono só.
    pub fn ik_move(
        &mut self,
        target: [f32; 2],
        angle: f32,
        opts: IkOptions,
    ) -> Vec<(Entity, [f32; 2], f32)> {
        // ⚠️ **A árvore guarda HANDLES, e o gesto os invalida sozinho.**
        //
        // Cada Move escreve o `Transform` dos corpos — é o que posar É —, e o
        // dispatch do frame seguinte vê a pose AUTORADA mudada em repouso, o que
        // é exatamente a condição em que a ponte **re-descreve** o corpo. Um
        // corpo re-descrito ganha handle NOVO, e a árvore fica apontando para
        // uma arena que andou. Medido: `No element at index`, dentro do rapier,
        // no segundo frame do arrasto — um PÂNICO no meio de um gesto, que é o
        // pior lugar possível para um.
        //
        // A cura é re-montar em vez de tentar impedir o re-describe: ele é
        // legítimo (a pose de repouso mudou de fato), e re-montar custa
        // **0,005–0,015 ms** — medido, um terço do custo de um solve. E não é
        // uma perda de *warm start*: o estado que importa é a POSE, e ela está
        // no mundo, que é justamente de onde a árvore nova se semeia.
        let stale = self.ik.as_ref().is_some_and(|s| {
            s.built_with
                .iter()
                .any(|(e, h)| self.bodies.get(e).map(|b| b.handle) != Some(*h))
        });
        if stale {
            let tip = self.ik.as_ref().map(|s| s.tip);
            self.ik = tip.and_then(|t| self.build_ik_session(t));
        }
        let Some(s) = self.ik.as_mut() else {
            return Vec::new();
        };
        let poses = self.world.ik_solve(&mut s.chain, target, angle, opts);
        poses
            .into_iter()
            .filter_map(|p| {
                s.by_handle
                    .get(&p.body.0.into_raw_parts().0)
                    .map(|&e| (e, p.translation, p.rotation))
            })
            .collect()
    }

    /// Encerra o gesto. Idempotente — soltar sem ter pegado é o caso comum de
    /// quase todo release do app.
    pub fn ik_end(&mut self) {
        self.ik = None;
    }
}

/// O mesmo joint visto do outro lado: as âncoras (e os eixos) trocam de corpo.
///
/// Só isso — limite, motor, comprimento e rigidez são simétricos por natureza,
/// e o único par lateralizado num `JointDesc` são as duas âncoras e os dois
/// eixos locais.
pub(super) fn swap_anchors(mut d: ph2d_physics::JointDesc) -> ph2d_physics::JointDesc {
    std::mem::swap(&mut d.anchor_a, &mut d.anchor_b);
    std::mem::swap(&mut d.axis_a, &mut d.axis_b);
    d
}
