//! **A CORDA PELA PONTA** (W-LeadDrag) — o que acontece quando o corpo que o
//! artista pega é a **RAIZ** da árvore.
//!
//! A IK do [`super::ik`] responde *"ponha a PONTA ali"*: a raiz fica onde está e
//! o solver acha os ângulos do que vem atrás. Pegando a raiz não há nada atrás
//! dela, e a versão anterior recusava o gesto (`root == tip` devolvia `None`) —
//! o que deixava sem resposta justamente o gesto de **levar o rig**.
//!
//! A resposta não é um solver: é o **revezamento**. A mão leva o líder, cada elo
//! é puxado pelo que vem à frente, e o último chega por último — que é o que
//! *arrastar uma corda pela ponta* significa, e o que o Enio pediu por essas
//! palavras.
//!
//! # A lei, numa linha por elo
//!
//! O pai já se moveu, então a âncora dele em MUNDO já é conhecida. O filho é
//! rígido e tem de pôr a própria âncora exatamente ali; das infinitas atitudes
//! que satisfazem isso, a corda escolhe a que mantém a **JUNTA SEGUINTE**
//! apontada para onde ela ainda está — o elo é arrastado, não empurrado. Sai
//! daí, sem iteração e sem resíduo:
//!
//! ```text
//! a   = pose_do_pai ∘ âncora_local_do_pai      (onde o elo agora prende)
//! v   = junta_seguinte_ATUAL − a               (para onde o elo é puxado)
//! rotação_nova = ∠v̂ − ∠off ;  centro = a − R(rotação) · âncora_local_do_filho
//! ```
//!
//! com `off` = o vetor da âncora deste elo até a junta seguinte, no frame do
//! filho (constante, porque ele é rígido). No limite de passos pequenos isto é
//! a **tractriz** — a curva do que é puxado por uma corda —, e é por isso que a
//! forma converge com a amostragem em vez de divergir.
//!
//! ⚠️ **Encadear as JUNTAS, e não os centros, é o que separa uma corda de uma
//! gangorra** — e a diferença foi MEDIDA, não deduzida: ver
//! [`LeadLink::far_c`], onde os dois perfis estão lado a lado.
//!
//! # ⚠️ Este é o ÚNICO gesto desta linha que tem MEMÓRIA, e é de propósito
//!
//! O `advance` lê a pose VIVA do filho, não a de press. Todo outro gesto do
//! módulo re-deriva da fonte congelada, pela lei que esta linha e a do Painter
//! pagaram várias vezes (*um produto sobre a lista de eventos faz o resultado
//! depender da taxa de polling do mouse*).
//!
//! Uma corda **não pode** obedecer a essa lei: a forma dela é função do CAMINHO
//! que a mão fez, não da posição em que a mão parou — dar a volta num obstáculo
//! e voltar deixa a corda enrolada, e é isso que a torna uma corda. O que a lei
//! congelada existe para prevenir é **divergência** com o refino da amostragem,
//! e aqui o refino CONVERGE (a tractriz é o limite contínuo). O gate
//! `a_finer_drag_of_the_same_path_converges_instead_of_diverging` é essa frase
//! com número.
//!
//! # ⚠️ Só uma DOBRADIÇA dobra
//!
//! A árvore admite três tipos de elo ([`ph2d_physics::is_rigid_link`]): Pin,
//! Weld e Slider. A lei acima é ANGULAR, e só o Pin oferece o ângulo que ela
//! escolhe — um Weld é uma peça só (por definição não dobra) e um Slider
//! desliza ao longo de um EIXO, que é outra coordenada e pediria outra lei.
//!
//! Os dois seguem o pai **rigidamente**, mantendo a pose relativa do repouso.
//! Inventar um deslizamento aqui seria dar a um trilho um comportamento que
//! ninguém autorou; recusá-lo em silêncio, um elo que não acompanha. Seguir
//! rígido é o que os dois já significam quando ninguém está puxando.

use std::collections::BTreeMap;

use ph2d_ecs::Entity;
use ph2d_physics::{FkDof, fk_dof, is_rigid_link};

use super::PhysicsBridge;
use super::ik::IkPlan;

/// Pose de mundo de um corpo: posição e ângulo. A escala não entra — posar não
/// estica nada (a mesma decisão do `body_pose` da shell).
type Pose = ([f32; 2], f32);

/// Um elo, já resolvido para o que o arrasto precisa saber dele.
struct LeadLink {
    parent: Entity,
    child: Entity,
    /// A âncora no PAI, no frame local dele.
    local_p: [f32; 2],
    /// A âncora deste elo no FILHO, no frame local dele.
    local_c: [f32; 2],
    /// **O ponto do filho que a puxada mantém apontado**, no frame local dele:
    /// a âncora do filho para o PRÓPRIO filho, ou o centro se ele é folha.
    ///
    /// ⚠️ **Este campo é a diferença entre uma corda e uma gangorra, e foi
    /// MEDIDO.** A primeira versão apontava o CENTRO do corpo, e um bastão que
    /// pivota em torno da âncora levantada joga a outra ponta para baixo com
    /// ALAVANCA: o perfil saía `L1 0,100 · L2 0,010 · L3 0,028 · L4 0,043` —
    /// o fim da corda andando **quatro vezes** o começo dela, que é o oposto
    /// exato do que arrastar uma corda faz.
    ///
    /// Encadeando as JUNTAS a alavanca não existe: cada junta é puxada em
    /// direção à anterior, e é essa a formulação clássica do
    /// *follow-the-leader*. Num corpo-FOLHA não há junta seguinte e o centro é
    /// a resposta honesta — não há nada além dele para apontar.
    far_c: [f32; 2],
    /// `None` = este elo NÃO dobra: o filho segue o pai rigidamente, com a pose
    /// relativa guardada aqui (`posição`, `ângulo`, no frame do pai).
    rigid: Option<Pose>,
}

/// Uma corda viva: quem é o líder, onde cada corpo está AGORA, e os elos na
/// ordem em que a puxada os alcança.
pub(super) struct LeadDrag {
    lead: Entity,
    /// A pose de cada corpo neste instante. **É a memória da corda** — ver o
    /// cabeçalho para por que ela existe aqui e em nenhum outro gesto.
    live: BTreeMap<Entity, Pose>,
    /// Ordem de BFS a partir do líder: todo pai antes de todo filho, que é o
    /// que faz uma única passada bastar.
    links: Vec<LeadLink>,
}

fn rot(a: f32) -> (f32, f32) {
    libm::sincosf(a)
}

/// `pai ∘ local` — um ponto local levado ao mundo.
fn to_world(pose: Pose, local: [f32; 2]) -> [f32; 2] {
    let (s, c) = rot(pose.1);
    [
        pose.0[0] + local[0] * c - local[1] * s,
        pose.0[1] + local[0] * s + local[1] * c,
    ]
}

/// A pose do filho **relativa** à do pai, para um elo que não dobra.
fn relative(parent: Pose, child: Pose) -> Pose {
    let (s, c) = rot(-parent.1);
    let (dx, dy) = (child.0[0] - parent.0[0], child.0[1] - parent.0[1]);
    ([dx * c - dy * s, dx * s + dy * c], child.1 - parent.1)
}

/// O inverso de [`relative`]: o filho recolocado atrás do pai que se moveu.
fn compose(parent: Pose, rel: Pose) -> Pose {
    (to_world(parent, rel.0), parent.1 + rel.1)
}

impl PhysicsBridge {
    /// **Monta a corda** a partir de um plano cuja raiz é o próprio corpo pego.
    ///
    /// `None` quando algum corpo ou joint do plano não está construído — a mesma
    /// recusa estrutural do irmão que monta a árvore de multibody.
    pub(super) fn build_lead_drag(&self, plan: &IkPlan) -> Option<LeadDrag> {
        let mut live = BTreeMap::new();
        let pose_of = |e: Entity| -> Option<Pose> {
            let b = self.bodies.get(&e)?;
            Some(([b.rest.x, b.rest.y], b.rest.rotation))
        };
        live.insert(plan.root, pose_of(plan.root)?);

        // Primeira passada: a âncora de SAÍDA de cada corpo — a que prende o
        // primeiro filho dele. É ela que a puxada mantém apontada (ver
        // [`LeadLink::far_c`]); quem não tem fica sem, e o centro responde.
        let mut out_anchor: BTreeMap<Entity, [f32; 2]> = BTreeMap::new();
        for &(parent, child, je) in &plan.edges {
            let Some(jr) = self.joints.get(&je) else {
                continue;
            };
            let local = if jr.entities == (parent, Some(child)) {
                jr.rest.anchor_a
            } else {
                jr.rest.anchor_b
            };
            out_anchor.entry(parent).or_insert(local);
        }

        let mut links = Vec::with_capacity(plan.edges.len());
        for &(parent, child, je) in &plan.edges {
            let jr = self.joints.get(&je)?;
            let (pp, cp) = (pose_of(parent)?, pose_of(child)?);
            live.insert(child, cp);

            // ⚠️ A ORIENTAÇÃO: o `JointDesc` guarda as âncoras na ordem do joint
            // AUTORADO, e a puxada precisa delas na ordem (pai, filho). Chegar
            // pelo lado B troca as duas — a mesma correção que a árvore de IK e
            // a subida da FK fazem, pelo mesmo motivo.
            let from_a = jr.entities == (parent, Some(child));
            let (local_p, local_c) = if from_a {
                (jr.rest.anchor_a, jr.rest.anchor_b)
            } else {
                (jr.rest.anchor_b, jr.rest.anchor_a)
            };

            let bends =
                is_rigid_link(jr.rest.kind) && matches!(fk_dof(jr.rest.kind), Some(FkDof::Hinge));
            links.push(LeadLink {
                parent,
                child,
                local_p,
                local_c,
                far_c: out_anchor.get(&child).copied().unwrap_or([0.0, 0.0]),
                rigid: (!bends).then(|| relative(pp, cp)),
            });
        }
        Some(LeadDrag {
            lead: plan.root,
            live,
            links,
        })
    }
}

impl LeadDrag {
    /// **Leva o líder ao cursor e deixa a corda ser puxada.** Devolve a pose de
    /// todo corpo da árvore, líder incluso.
    ///
    /// ⚠️ O líder **translada e não gira**: o cursor dá um lugar, não uma
    /// atitude, e girar o corpo pego por um ângulo que a mão não pediu é a
    /// mesma classe de invenção que o §11 recusa nos knobs.
    pub(super) fn advance(&mut self, cursor: [f32; 2]) -> Vec<(Entity, [f32; 2], f32)> {
        if let Some(p) = self.live.get_mut(&self.lead) {
            p.0 = cursor;
        }

        for link in &self.links {
            let Some(&parent) = self.live.get(&link.parent) else {
                continue;
            };
            let next = match link.rigid {
                Some(rel) => compose(parent, rel),
                None => {
                    let Some(&child) = self.live.get(&link.child) else {
                        continue;
                    };
                    bend(parent, child, link)
                }
            };
            self.live.insert(link.child, next);
        }

        self.live.iter().map(|(&e, &(t, r))| (e, t, r)).collect()
    }

    /// Todo corpo que esta corda move — o que o chamador precisa para abrir UM
    /// passo de undo cobrindo a cadeia inteira.
    pub(super) fn bodies(&self) -> Vec<Entity> {
        self.live.keys().copied().collect()
    }
}

/// O elo que DOBRA: o filho põe a própria âncora onde o pai a levou e gira para
/// continuar apontando o ponto que ele já apontava.
fn bend(parent: Pose, child: Pose, link: &LeadLink) -> Pose {
    let a = to_world(parent, link.local_p);
    // Da âncora deste elo até o ponto apontado, no frame do FILHO — constante,
    // porque ele é rígido.
    let off = [
        link.far_c[0] - link.local_c[0],
        link.far_c[1] - link.local_c[1],
    ];
    let len = off[0].hypot(off[1]);
    // ⚠️ As duas âncoras no MESMO ponto do filho: não há braço, logo não há
    // direção que a puxada possa escolher. O filho vai para a âncora com a
    // atitude que tinha — recusar seria um elo que não acompanha, e inventar um
    // ângulo seria girar um corpo por um motivo que ninguém autorou.
    if len < 1e-6 {
        return (place(a, child.1, link.local_c), child.1);
    }
    let phi = libm::atan2f(off[1], off[0]);
    let far_old = to_world(child, link.far_c);
    let v = [far_old[0] - a[0], far_old[1] - a[1]];
    // Degenerado só se o ponto apontado já caiu EXATAMENTE sobre a âncora nova;
    // aí a direção anterior é a única informação honesta que resta.
    let psi = if v[0].hypot(v[1]) < 1e-6 {
        child.1 + phi
    } else {
        libm::atan2f(v[1], v[0])
    };
    let rotation = psi - phi;
    (place(a, rotation, link.local_c), rotation)
}

/// Onde fica o CENTRO de um corpo cuja âncora `local` tem de pousar em `a` com
/// a rotação dada. O centro é derivado da restrição, nunca estimado — é o que
/// mantém a âncora exata seja qual for a direção escolhida.
fn place(a: [f32; 2], rotation: f32, local: [f32; 2]) -> [f32; 2] {
    let (s, c) = rot(rotation);
    [
        a[0] - (local[0] * c - local[1] * s),
        a[1] - (local[0] * s + local[1] * c),
    ]
}
