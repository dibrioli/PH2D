//! **A sessão de CINEMÁTICA DIRETA** (W-FK) — irmã do [`super::ik`], e o outro
//! lado da mesma moeda.
//!
//! | | IK (W-IK) | FK (esta) |
//! |---|---|---|
//! | o artista arrasta | a **PONTA** da cadeia | **um elo** qualquer |
//! | quem se move | todo mundo entre a raiz e a ponta | o elo e os **descendentes** dele |
//! | quem resolve | um solver (Levenberg amortecido) | **ninguém** — é geometria |
//! | o que se autora | uma pose alcançando um lugar | o **ângulo de UMA junta** |
//!
//! As duas são necessárias e nenhuma substitui a outra: a IK é para *pôr a mão
//! ali*, a FK é para *dobrar o cotovelo assim*. Todo pacote de animação carrega
//! as duas pelo mesmo motivo, e o artista alterna entre elas dentro do mesmo
//! plano.
//!
//! # Não há solver aqui, e é o desenho inteiro
//!
//! Girar um elo em torno da própria junta e levar os descendentes rigidamente é
//! um **movimento rígido**: ele preserva toda distância e todo ângulo dentro da
//! peça que se move, então nenhuma restrição interna é violada e não há nada a
//! resolver. A única coordenada que muda é a da junta que o gesto escolheu —
//! por exatamente o ângulo arrastado.
//!
//! Três consequências, todas boas:
//!
//! - **A sessão não guarda handles do rapier.** A IK precisa deles (a árvore de
//!   multibody vive na arena) e por isso teve de aprender a se re-montar quando
//!   um `Transform` escrito re-descreve os corpos. Aqui só há poses e um pivô,
//!   colhidos no press — o gesto é imune àquilo por construção.
//! - **É exato**, não iterativo: nenhum resíduo, nenhum `max_iters`.
//! - **Acumula e aplica UMA vez sobre a fonte congelada.** Cada Move re-deriva a
//!   pose a partir da pose de PRESS, nunca da anterior — a lei que esta linha e
//!   a do Painter pagaram várias vezes (um produto sobre a lista de eventos faz
//!   o resultado depender da taxa de polling do mouse).
//!
//! # A junta que o gesto move é a primeira ACIMA que tem grau de liberdade
//!
//! Pegar um elo soldado (Weld) ao pai não pode não fazer nada: um Weld é a
//! afirmação de que os dois corpos são UMA peça. Então a busca sobe pela árvore
//! enquanto a junta não oferece movimento ([`ph2d_physics::fk_dof`]), e o que se
//! move é a subárvore inteira a partir dali — a peça soldada viaja junta, que é
//! o que "soldado" quer dizer.
//!
//! # A hierarquia é a MESMA do IK, e isso não é reuso preguiçoso
//!
//! Quem é a raiz — logo, quem é pai de quem — sai do [`super::ik::IkPlan`]: um
//! corpo `Static`/`Kinematic` alcançável é a raiz, e sem nenhum a cadeia flutua e
//! a raiz é o corpo mais distante. Uma segunda política de raiz aqui seria uma
//! segunda resposta a *"para que lado desta cadeia é 'para cima'?"*, e as duas
//! discordariam no primeiro rig que tivesse âncora dos dois lados.
//!
//! ⚠️ **Corolário honesto:** numa cadeia SOLTA (sem âncora), a raiz é o corpo
//! mais distante do que você pegou, então o pego é sempre uma folha e a FK gira
//! só ele. É consistente (a cadeia não tem "para cima" nenhum, e a única
//! resposta era escolher uma) e é o que a IK já faz com a mesma cena.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use ph2d_ecs::{Entity, SimWorld};
use ph2d_physics::{FkDof, fk_dof, joint_coordinate_at};

use super::PhysicsBridge;
use super::anchors::JointSide;

/// Uma sessão de FK viva. Nasce no press, morre no release; nada aqui é estado
/// da cena — o que sobrevive ao gesto é o `Transform` que o chamador escreveu.
pub struct FkSession {
    /// O corpo que o artista pegou.
    pub tip: Entity,
    /// Todo corpo que este gesto move (o topo da peça rígida + descendentes) —
    /// o que o chamador precisa saber para abrir UM passo de undo.
    pub bodies: Vec<Entity>,
    /// A âncora da junta que o gesto move, em MUNDO. Fixa durante o gesto: o
    /// PAI não se move, então o pivô não pode andar.
    pivot: [f32; 2],
    dof: FkDof,
    /// Eixo do trilho em MUNDO, unitário. Só o [`FkDof::Slide`] o usa.
    axis: [f32; 2],
    /// A faixa autorada da junta, na coordenada dela (radianos numa dobradiça,
    /// metros num trilho) — ou `None` se a junta é livre.
    limits: Option<[f32; 2]>,
    /// A coordenada que a junta tinha no press. É contra ELA que o limite é
    /// medido: o gesto move um DELTA, e o limite fala da posição absoluta.
    coord0: f32,
    /// A referência do gesto: ângulo do cursor no press (dobradiça) ou projeção
    /// dele no eixo (trilho).
    grab: f32,
    /// O ângulo cru do cursor no último Move — para o delta acumular por VOLTAS
    /// em vez de saltar ao cruzar ±π.
    last: f32,
    /// O deslocamento que o CURSOR pede, acumulado e **sem clamp**.
    raw: f32,
    /// O que de fato foi aplicado: o `raw` trazido para a faixa autorada.
    applied: f32,
    /// A pose de MUNDO de cada corpo no press. A fonte congelada.
    start: Vec<(Entity, [f32; 2], f32)>,
}

impl FkSession {
    /// Quantos corpos este gesto move.
    #[must_use]
    pub fn len(&self) -> usize {
        self.bodies.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bodies.is_empty()
    }

    /// O deslocamento que o gesto de fato aplicou, na coordenada da junta —
    /// radianos numa dobradiça, metros num trilho. Já dentro da faixa autorada.
    #[must_use]
    pub fn delta(&self) -> f32 {
        self.applied
    }
}

impl PhysicsBridge {
    /// **Há um gesto de FK em voo?**
    #[must_use]
    pub fn is_posing_fk(&self) -> bool {
        self.fk.is_some()
    }

    /// A sessão viva, para um gate ler o deslocamento que ela aplicou.
    #[must_use]
    pub fn fk_session(&self) -> Option<&FkSession> {
        self.fk.as_ref()
    }

    /// Todo corpo que a FK viva move.
    #[must_use]
    pub fn fk_bodies(&self) -> &[Entity] {
        self.fk.as_ref().map_or(&[], |s| &s.bodies)
    }

    /// **Abre o gesto.** `true` se há uma junta com grau de liberdade acima de
    /// `tip`.
    ///
    /// Toda a autoridade sobre *quando* isto pode acontecer é do chamador (a
    /// shell vê o relógio e a ferramenta); aqui a recusa é só estrutural.
    pub fn fk_begin(&mut self, sim: &SimWorld, tip: Entity, cursor: [f32; 2]) -> bool {
        self.fk = self.build_fk_session(sim, tip, cursor);
        self.fk.is_some()
    }

    fn build_fk_session(&self, sim: &SimWorld, tip: Entity, cursor: [f32; 2]) -> Option<FkSession> {
        let plan = self.ik_plan(tip)?;
        // A árvore, nas duas direções: subir para achar a junta que move, descer
        // para colher quem vem junto.
        let mut parent_of: BTreeMap<Entity, (Entity, Entity)> = BTreeMap::new();
        let mut children: BTreeMap<Entity, Vec<Entity>> = BTreeMap::new();
        for &(p, c, j) in &plan.edges {
            parent_of.insert(c, (p, j));
            children.entry(p).or_default().push(c);
        }

        // Sobe enquanto a junta não oferece movimento — um Weld é uma peça só.
        let mut node = tip;
        let (parent, joint, dof) = loop {
            let &(p, j) = parent_of.get(&node)?;
            let kind = self.joints.get(&j)?.rest.kind;
            if let Some(d) = fk_dof(kind) {
                break (p, j, d);
            }
            node = p;
        };

        // A subárvore de `node`: ele e tudo pendurado nele.
        let mut bodies = Vec::new();
        let mut seen = BTreeSet::from([node]);
        let mut q = VecDeque::from([node]);
        while let Some(e) = q.pop_front() {
            bodies.push(e);
            for &c in children.get(&e).map_or(&[][..], |v| &v[..]) {
                if seen.insert(c) {
                    q.push_back(c);
                }
            }
        }

        let jr = self.joints.get(&joint)?;
        // ⚠️ A ORIENTAÇÃO: o `JointDesc` guarda as âncoras na ordem do joint
        // AUTORADO, e a coordenada é medida na ordem (pai, filho). Alcançar o
        // joint pelo lado B troca as duas — a mesma correção que a árvore de IK
        // faz, pela mesma porta.
        let from_a = jr.entities == (parent, node);
        let desc = if from_a {
            jr.rest
        } else {
            super::ik::swap_anchors(jr.rest)
        };
        let side = if from_a { JointSide::A } else { JointSide::B };
        let pivot = self.joint_anchor_world(sim, joint, side)?;

        let pb = self.bodies.get(&parent)?;
        let parent_pose = [pb.rest.x, pb.rest.y, pb.rest.rotation];
        // O eixo do trilho é local ao corpo A do joint (já reorientado acima):
        // levá-lo ao mundo é a rotação do PAI, e só ela.
        let (psin, pcos) = libm::sincosf(parent_pose[2]);
        let axis = {
            let [ax, ay] = desc.axis_a;
            let n = (ax * ax + ay * ay).sqrt();
            let (ux, uy) = if n > 0.0 {
                (ax / n, ay / n)
            } else {
                (1.0, 0.0)
            };
            [ux * pcos - uy * psin, ux * psin + uy * pcos]
        };

        let mut start = Vec::with_capacity(bodies.len());
        for &e in &bodies {
            let b = self.bodies.get(&e)?;
            start.push((e, [b.rest.x, b.rest.y], b.rest.rotation));
        }
        let nb = self.bodies.get(&node)?;
        let coord0 =
            joint_coordinate_at(&desc, parent_pose, [nb.rest.x, nb.rest.y, nb.rest.rotation])?;

        let grab = reference(dof, pivot, axis, cursor);
        Some(FkSession {
            tip,
            bodies,
            pivot,
            dof,
            axis,
            limits: desc.limits,
            coord0,
            grab,
            last: grab,
            raw: 0.0,
            applied: 0.0,
            start,
        })
    }

    /// **Aplica o gesto e devolve a pose de cada corpo que ele move.**
    ///
    /// Vazio sem sessão. Quem ESCREVE o `Transform` é o chamador — a física não
    /// autora a cena, que é o que mantém o passo de undo e a hierarquia com um
    /// dono só (a mesma divisão do `ik_move`).
    pub fn fk_move(&mut self, cursor: [f32; 2]) -> Vec<(Entity, [f32; 2], f32)> {
        let Some(s) = self.fk.as_mut() else {
            return Vec::new();
        };
        let now = reference(s.dof, s.pivot, s.axis, cursor);
        match s.dof {
            // ⚠️ O ângulo é acumulado por INCREMENTOS: um arrasto que passa de
            // ±π daria um salto de uma volta inteira se o delta fosse medido
            // contra o press. Assim uma dobradiça livre gira quantas voltas o
            // artista quiser, e é o que uma dobradiça faz.
            FkDof::Hinge => {
                let mut step = now - s.last;
                let tau = std::f32::consts::TAU;
                while step > std::f32::consts::PI {
                    step -= tau;
                }
                while step <= -std::f32::consts::PI {
                    step += tau;
                }
                s.last = now;
                s.raw += step;
            }
            // Uma distância não dá voltas: o delta é absoluto contra o press, o
            // que também o torna imune a qualquer sequência de Moves.
            FkDof::Slide => s.raw = now - s.grab,
        }
        // ⚠️ **O clamp mora no MAPEAMENTO, não no acumulador — e a diferença é
        // um gate.** Isto é manipulação DIRETA: o ângulo do cursor em torno do
        // pivô *é* o ângulo da junta, então a lei é `junta = clamp(cursor)`,
        // como um slider clampa a posição e não o passo do mouse. Clampar o
        // acumulador jogaria o excedente fora, e voltar de fora da faixa moveria
        // a junta em relação à PAREDE em vez de em relação ao cursor — o elo
        // sairia de sincronia com a mão e não voltaria mais
        // (`coming_back_from_a_limit_is_immediate` é o gate, e ele nasceu
        // vermelho contra a primeira versão desta função).
        s.applied = match s.limits {
            Some([lo, hi]) => (s.coord0 + s.raw).clamp(lo, hi) - s.coord0,
            None => s.raw,
        };

        let d = s.applied;
        match s.dof {
            FkDof::Hinge => {
                let (sin, cos) = libm::sincosf(d);
                s.start
                    .iter()
                    .map(|&(e, [x, y], r)| {
                        let (dx, dy) = (x - s.pivot[0], y - s.pivot[1]);
                        (
                            e,
                            [
                                s.pivot[0] + dx * cos - dy * sin,
                                s.pivot[1] + dx * sin + dy * cos,
                            ],
                            r + d,
                        )
                    })
                    .collect()
            }
            FkDof::Slide => s
                .start
                .iter()
                .map(|&(e, [x, y], r)| (e, [x + s.axis[0] * d, y + s.axis[1] * d], r))
                .collect(),
        }
    }

    /// Encerra o gesto. Idempotente — soltar sem ter pegado é o caso comum de
    /// quase todo release do app.
    pub fn fk_end(&mut self) {
        self.fk = None;
    }
}

/// O escalar que o cursor vale para este grau de liberdade: um ÂNGULO em torno
/// do pivô, ou a PROJEÇÃO no eixo do trilho.
///
/// Uma função e não duas linhas em cada sítio: o press e o Move têm de medir a
/// MESMA coisa, e é dessa igualdade que sai o gesto ser *grab-relative* (pegar o
/// elo em qualquer ponto não o teleporta).
fn reference(dof: FkDof, pivot: [f32; 2], axis: [f32; 2], cursor: [f32; 2]) -> f32 {
    let (dx, dy) = (cursor[0] - pivot[0], cursor[1] - pivot[1]);
    match dof {
        // ⚠️ `libm`, não o `atan2` da `std`: a lei 6 desta linha é que todo
        // transcendental que possa alcançar um número autorado passa pela mesma
        // implementação em todo OS. Uma pose é autorada.
        FkDof::Hinge => libm::atan2f(dy, dx),
        FkDof::Slide => dx * axis[0] + dy * axis[1],
    }
}
