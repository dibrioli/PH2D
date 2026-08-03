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
//! # Cada elo usa o grau de liberdade que ELE tem (W-RailRope)
//!
//! A árvore admite três tipos de elo ([`ph2d_physics::is_rigid_link`]): Pin,
//! Weld e Slider, e a lei acima é enunciada em ÂNGULO porque foi escrita para o
//! primeiro. ⚠️ **Mas o princípio não é angular — é *use a sua única liberdade
//! para atrapalhar o menos possível***, e essa frase tem uma resposta em CADA
//! coordenada:
//!
//! | elo | grau de liberdade | o que a puxada escolhe |
//! |---|---|---|
//! | **Pin** | ângulo em torno da âncora | o ângulo que mantém o ponto apontado |
//! | **Slider** | distância ao longo do eixo | o **deslize** que mantém o ponto apontado, **clampado ao curso** |
//! | **Weld** | nenhum | nada: a peça viaja inteira, que é o que *soldado* significa |
//!
//! ⚠️ **O trilho LAGA, e é isso que o torna uma corda em vez de um bloco:** o
//! filho segue o pai rigidamente e depois **desliza para trás** ao longo do
//! eixo, exatamente o quanto o eixo permite recuperar do movimento que ele
//! sofreu. Puxe o líder pela direção do trilho e o carrinho arrasta atrás; puxe
//! na perpendicular e ele vai junto, porque um trilho não tem liberdade nessa
//! direção.
//!
//! ⚠️ **E o CURSO é load-bearing.** Sem o clamp em [`JointDesc::limits`] o
//! carrinho deslizaria para fora do trilho, e o desenho seria de um rail com
//! percurso infinito — o solver o traria de volta com um estalo no Play
//! seguinte. Com ele, chegar ao fim do curso e passar a viajar junto é o que um
//! trilho de verdade faz.
//!
//! Um **Weld** segue rígido, e continua: ele não tem uma segunda coordenada onde
//! a mesma pergunta pudesse ser feita.
//!
//! [`JointDesc::limits`]: ph2d_physics::JointDesc::limits

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
    /// **O que este elo deixa a puxada escolher** (W-RailRope).
    motion: LeadMotion,
}

/// A liberdade de um elo, resolvida no build.
enum LeadMotion {
    /// **Dobradiça:** o filho põe a âncora onde o pai a levou e gira.
    Bend,
    /// **Trilho:** o filho segue rígido e depois DESLIZA pelo eixo.
    ///
    /// Carrega a pose relativa de repouso (o rígido), o eixo no frame do PAI, e
    /// o curso — o deslize acumulado nunca sai dele.
    Slide {
        rel: Pose,
        /// O eixo do trilho no frame do PAI, já normalizado.
        ///
        /// ⚠️ **Do pai e não do filho**, porque é o pai que se move primeiro e é
        /// contra o frame dele que o rígido é composto; usar o do filho daria a
        /// mesma direção só enquanto os dois estivessem alinhados, que é
        /// precisamente o caso em que nenhum gate distingue os dois.
        axis_p: [f32; 2],
        /// `[min, max]` do curso, metros. `None` = trilho sem batentes.
        limits: Option<[f32; 2]>,
        /// Onde o carrinho está no curso AGORA. **É memória, como o resto desta
        /// corda** — e pelo mesmo motivo: o deslize é função do CAMINHO que a mão
        /// fez, não da posição em que ela parou.
        s: f32,
    },
    /// **Solda:** nenhuma liberdade; a peça viaja inteira.
    Rigid(Pose),
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

            // ⚠️ **`is_rigid_link` PRIMEIRO**: um Spring/Rope nem é elo — a pose
            // dele é resultado de forças —, e chegar aqui perguntando o DOF dele
            // daria a um par que só a física decide um comportamento de pose.
            let dof = is_rigid_link(jr.rest.kind)
                .then(|| fk_dof(jr.rest.kind))
                .flatten();
            let motion = match dof {
                Some(FkDof::Hinge) => LeadMotion::Bend,
                Some(FkDof::Slide) => LeadMotion::Slide {
                    rel: relative(pp, cp),
                    // O eixo vem do lado que É o pai neste percurso — o desc o
                    // guarda nos DOIS frames justamente para não ter de
                    // re-derivá-lo contra a pose em que os corpos derivaram.
                    axis_p: unit(if from_a {
                        jr.rest.axis_a
                    } else {
                        jr.rest.axis_b
                    }),
                    limits: jr.rest.limits,
                    // O carrinho começa onde está: zero deslize SOBRE o rígido
                    // de repouso, que é o que `rel` já descreve.
                    s: 0.0,
                },
                // Weld (e todo elo sem grau de liberdade): a peça viaja inteira.
                None => LeadMotion::Rigid(relative(pp, cp)),
            };
            links.push(LeadLink {
                parent,
                child,
                local_p,
                local_c,
                far_c: out_anchor.get(&child).copied().unwrap_or([0.0, 0.0]),
                motion,
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

        for link in &mut self.links {
            let Some(&parent) = self.live.get(&link.parent) else {
                continue;
            };
            let Some(&child) = self.live.get(&link.child) else {
                continue;
            };
            let next = match &mut link.motion {
                LeadMotion::Rigid(rel) => compose(parent, *rel),
                LeadMotion::Bend => bend(parent, child, link.local_p, link.local_c, link.far_c),
                LeadMotion::Slide {
                    rel,
                    axis_p,
                    limits,
                    s,
                } => slide(parent, child, link.far_c, *rel, *axis_p, *limits, s),
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
fn bend(parent: Pose, child: Pose, local_p: [f32; 2], local_c: [f32; 2], far_c: [f32; 2]) -> Pose {
    let a = to_world(parent, local_p);
    // Da âncora deste elo até o ponto apontado, no frame do FILHO — constante,
    // porque ele é rígido.
    let off = [far_c[0] - local_c[0], far_c[1] - local_c[1]];
    let len = off[0].hypot(off[1]);
    // ⚠️ As duas âncoras no MESMO ponto do filho: não há braço, logo não há
    // direção que a puxada possa escolher. O filho vai para a âncora com a
    // atitude que tinha — recusar seria um elo que não acompanha, e inventar um
    // ângulo seria girar um corpo por um motivo que ninguém autorou.
    if len < 1e-6 {
        return (place(a, child.1, local_c), child.1);
    }
    let phi = libm::atan2f(off[1], off[0]);
    let far_old = to_world(child, far_c);
    let v = [far_old[0] - a[0], far_old[1] - a[1]];
    // Degenerado só se o ponto apontado já caiu EXATAMENTE sobre a âncora nova;
    // aí a direção anterior é a única informação honesta que resta.
    let psi = if v[0].hypot(v[1]) < 1e-6 {
        child.1 + phi
    } else {
        libm::atan2f(v[1], v[0])
    };
    let rotation = psi - phi;
    (place(a, rotation, local_c), rotation)
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

/// Um vetor normalizado; `+X` para um eixo degenerado — a MESMA queda que o
/// `spawn_joint` faz, para o desenho da puxada e a restrição do solver não
/// escolherem direções diferentes num eixo que ninguém autorou.
fn unit(v: [f32; 2]) -> [f32; 2] {
    let n = v[0].hypot(v[1]);
    if n.is_finite() && n > 1e-6 {
        [v[0] / n, v[1] / n]
    } else {
        [1.0, 0.0]
    }
}

/// **O elo que DESLIZA** (W-RailRope): o filho segue o pai rigidamente e depois
/// escorrega pelo eixo, exatamente o quanto o eixo permite recuperar do
/// movimento que ele acabou de sofrer.
///
/// # É o MESMO princípio do [`bend`], na outra coordenada
///
/// A dobradiça escolhe o **ângulo** que mantém o ponto apontado onde estava;
/// o trilho escolhe o **deslize**. Em ambos a pergunta é *"use a sua única
/// liberdade para atrapalhar o menos possível"*, e é por isso que isto não é um
/// comportamento inventado: é a lei que já shipava, dita na coordenada que um
/// Slider de fato tem.
///
/// # O que ele faz, e o que ele NÃO pode fazer
///
/// 1. `rigid` = onde o filho estaria seguindo o pai sem liberdade nenhuma.
/// 2. `want` = o quanto o ponto apontado se afastou de onde estava.
/// 3. **Só a componente ao longo do EIXO é recuperável** — na perpendicular um
///    trilho não tem liberdade, e o carrinho vai junto. É exatamente isso que
///    faz puxar *pela* direção do trilho arrastar o carrinho atrás e puxar *na
///    perpendicular* levá-lo inteiro.
/// 4. O deslize acumula em `s` e é **CLAMPADO ao curso**. ⚠️ Sem o clamp o
///    carrinho sairia do trilho e o desenho seria de um rail com percurso
///    infinito, que o solver desfaz com um estalo no Play seguinte.
///
/// ⚠️ **O `s` acumula em vez de ser re-derivado**, como toda esta corda: o
/// deslize é função do CAMINHO que a mão fez. Ir até o fim do curso, voltar, e
/// ir de novo tem de deixar o carrinho onde a soma dos movimentos o pôs — não
/// onde a posição final do cursor sugere.
fn slide(
    parent: Pose,
    child: Pose,
    far_c: [f32; 2],
    rel: Pose,
    axis_p: [f32; 2],
    limits: Option<[f32; 2]>,
    s: &mut f32,
) -> Pose {
    // O eixo em MUNDO: o trilho gira com o pai.
    let (sp, cp) = rot(parent.1);
    let ax = [
        axis_p[0] * cp - axis_p[1] * sp,
        axis_p[0] * sp + axis_p[1] * cp,
    ];
    // Onde o filho ficaria sem liberdade nenhuma, JÁ com o deslize acumulado.
    let base = compose(parent, rel);
    let rigid = ([base.0[0] + ax[0] * *s, base.0[1] + ax[1] * *s], base.1);
    // O ponto apontado: onde ele estava, e onde o rígido o levaria.
    let was = to_world(child, far_c);
    let now = to_world(rigid, far_c);
    let want = [was[0] - now[0], was[1] - now[1]];
    // Só o que o eixo alcança.
    let along = want[0] * ax[0] + want[1] * ax[1];
    let mut next = *s + along;
    if let Some([lo, hi]) = limits {
        // ⚠️ `min`/`max` na ordem que sobrevive a um par INVERTIDO: um autor
        // pode digitar `[0.5, -0.5]`, e um `clamp` panica nesse caso. A mesma
        // cautela que o §12 já toma ao ler os limites de uma dobradiça.
        next = next.max(lo.min(hi)).min(lo.max(hi));
    }
    *s = next;
    ([base.0[0] + ax[0] * next, base.0[1] + ax[1] * next], base.1)
}
