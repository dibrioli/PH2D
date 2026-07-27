//! **POSAR ARRASTANDO A PONTA** (W-IK) — cinemática inversa sobre uma árvore de
//! joints, construída sob demanda e descartada.
//!
//! Autorar a pose de uma cadeia articulada era **cinemática DIRETA**: gire o
//! ombro, gire o cotovelo, gire o punho, e a mão cai onde cair. O artista sabe
//! onde a MÃO tem de estar; os ângulos são o que ele não quer digitar. A IK é a
//! seta ao contrário — o alvo é a pose da PONTA, e o solver responde quais
//! coordenadas de junta a produzem.
//!
//! # Por que "multibody", e por que ele é TRANSITÓRIO
//!
//! O módulo simula com **`ImpulseJoint`**: cada joint é uma *restrição* que o
//! solver negocia a cada passo. É o certo para simular — é o que faz uma
//! corrente balançar e um ragdoll cair —, mas não há jacobiano a inverter numa
//! restrição negociada.
//!
//! A IK do rapier opera sobre um **`Multibody`**: a mesma cadeia descrita em
//! *coordenadas reduzidas*, em que a pose de cada elo é uma FUNÇÃO dos ângulos
//! dos pais. É essa forma que permite montar o jacobiano (como a ponta se move
//! quando cada junta gira um pouco), amortecer, inverter e caminhar até o alvo.
//!
//! ⚠️ **Um joint passaria a existir em duas representações** — e duas
//! representações do mesmo fato é a falha de duas-portas que esta linha já pagou
//! várias vezes (a âncora que caminhava, o clobber do damping, os dois
//! contadores de componente). A saída é o ciclo de vida: o multibody **não é
//! estado da cena**. Ele é construído a partir dos joints AUTORADOS quando o
//! gesto começa, vive enquanto o gesto vive, e morre com ele. Nada no `step`
//! toca nele, o `MultibodyJointSet` do mundo continua **vazio**, e a simulação
//! segue impulse-based e **byte-idêntica** (o `physics_ecs_c9` não se mexe).
//!
//! É a lei do bake dita noutro eixo: *assar não é simular de novo, é ANOTAR*.
//! Aqui: **posar não é simular, é RESOLVER** — e o resultado é `Transform`
//! autorado, que é o que a cena guarda.
//!
//! # Quem pode ser elo, e isso é do rapier, não gosto nosso
//!
//! Duas leis vêm do `Multibody` e são **duras**:
//!
//! 1. **É uma ÁRVORE.** Cada corpo tem no máximo um pai
//!    (`MultibodyJointSet::do_insert` recusa a aresta que fecharia um ciclo). O
//!    construtor faz BFS a partir da raiz, então a árvore geradora sai por
//!    construção e a aresta de fecho é simplesmente ignorada.
//! 2. **Todo elo NÃO-raiz tem de ser `Dynamic`** — `Multibody::forward_kinematics`
//!    tem um `assert_eq!` sobre isso. A raiz pode ser de qualquer tipo (uma raiz
//!    estática vira uma raiz *fixa*, de zero graus de liberdade, que é
//!    exactamente o gancho de um pêndulo).
//!
//! Daí a política de raiz: **um corpo Static ou Kinematic alcançável a partir da
//! ponta é a raiz**; sem nenhum, a raiz é o corpo mais distante da ponta e a
//! cadeia flutua livre (raiz livre, 3 graus de liberdade — a IK pode transladar
//! o conjunto inteiro, que é o que faz sentido para um rig solto).
//!
//! ⚠️ **Só joints RÍGIDOS viram elo** — Pin (revolute), Weld (fixed) e Slider
//! (prismatic). Uma Spring e uma Rope são *soft*: a distância delas é um alvo,
//! não uma lei, e uma cadeia cuja pose depende de forças não tem coordenadas
//! generalizadas a resolver. Elas são **fronteiras**, exactamente como um corpo
//! Static — a travessia as alcança e não as atravessa.
//!
//! # O que o solver é
//!
//! Mínimos quadrados amortecidos sobre o jacobiano
//! (`Multibody::inverse_kinematics`), iterativo, com dois critérios de parada:
//! `max_iters` e um limiar de erro. O `damping` é o **fator de Levenberg**:
//! pequeno demais passa do alvo e não converge; grande demais converge devagar.
//! Os dois números são MEDIDOS (ver [`IkOptions`] e `world/tests.rs`), não
//! escolhidos.

use rapier2d::dynamics::{
    FixedJointBuilder, InverseKinematicsOption, JointAxesMask, MultibodyJointHandle,
    MultibodyJointSet, PrismaticJointBuilder, RevoluteJointBuilder, RigidBodyHandle,
};
use rapier2d::na::{DVector, Isometry2, Point2, Vector2};

use super::joints::{JointDesc, JointKind};
use crate::PhysicsWorld;

/// Uma aresta da árvore de pose: *este joint pendura `child` em `parent`*.
///
/// A ordem importa e é do CHAMADOR: `parent` é quem já está na árvore. O
/// construtor recebe a lista já em ordem de BFS a partir da raiz, porque *quem é
/// a raiz* é uma pergunta sobre a CENA (existe um gancho estático?), não sobre a
/// cadeia — e a resposta mora na ponte do ECS, que é quem vê os tipos de corpo.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct IkLink {
    pub parent: RigidBodyHandle,
    pub child: RigidBodyHandle,
    pub joint: JointDesc,
}

/// A pose que a IK resolveu para um corpo: translação de mundo e rotação em
/// radianos — a mesma dupla que o `Transform` autorado guarda.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct IkPose {
    pub body: RigidBodyHandle,
    pub translation: [f32; 2],
    pub rotation: f32,
}

/// Os dois números do solver, mais a pergunta de produto.
///
/// ⚠️ **`damping` e `max_iters` são MEDIDOS** (`sweep_the_ik_damping` e
/// `sweep_the_ik_iterations` em `world/tests.rs`, sobre a cadeia de 3 elos que a
/// cena de smoke usa). O default do rapier é `damping: 1.0, max_iters: 10`; o
/// nosso confere ou diverge **pela tabela**, nunca por gosto.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct IkOptions {
    /// O fator de Levenberg. Ver a nota do módulo.
    pub damping: f32,
    /// Teto de iterações por chamada.
    pub max_iters: usize,
    /// **A ponta também obedece a um ÂNGULO?** Com `false` o solver resolve só
    /// a posição e deixa a atitude cair onde a cadeia a levar — que é o que se
    /// quer ao arrastar uma mão pelo canvas com o mouse (o mouse não tem
    /// rotação). Com `true` o alvo é a pose inteira.
    ///
    /// Não é um refinamento: são pedidos DIFERENTES, e pedir ângulo numa cadeia
    /// curta demais para atendê-lo faz o solver gastar o alcance dela girando a
    /// ponta em vez de chegar ao ponto.
    pub match_angle: bool,
}

impl Default for IkOptions {
    fn default() -> Self {
        Self {
            // ⚠️ **NÃO é o default do rapier (1.0), e a diferença é medida:**
            // numa cadeia de 3 elos, dez solves deixam a ponta a
            // **0,0787 m** do alvo com 1.0 e a **0,0004 m** com 0.1 —
            // duas ordens de grandeza, num alvo perfeitamente alcançável.
            // A cautela que justifica o 1.0 (amortecimento baixo passa do alvo
            // perto de uma singularidade) foi TESTADA no caso singular — a
            // cadeia esticada, puxada para 30 m — e 0,02..0,25 seguram o
            // alcance cheio exatamente como 1.0. Tabela: `sweep_the_ik_damping`.
            damping: 0.1,
            // Confirmado pela medição em vez de herdado: com `damping: 0.1`, o
            // erro de UM solve satura em 10 (1,9705 para 10, 16, 24 e 40).
            max_iters: 10,
            match_angle: false,
        }
    }
}

impl IkOptions {
    /// A faixa que a UI oferece, com a medição ao lado
    /// (`sweep_the_ik_damping`, cadeia de 3 elos, erro após 10 solves):
    ///
    /// | damping | erro | alcance no alvo inalcançável |
    /// |---|---|---|
    /// | 0.02 | 0,0002 | cheio |
    /// | **0.10** | **0,0004** | cheio |
    /// | 0.25 | 0,0009 | cheio |
    /// | 0.50 | 0,0066 | cheio |
    /// | 1.00 | 0,0787 | cheio |
    /// | 2.00 | 0,2734 | cheio |
    /// | 4.00 | 0,3535 | cheio |
    ///
    /// O topo é **1.0** e não 4.0: acima disso a ponta simplesmente não chega
    /// (27 cm e 35 cm de resíduo numa cadeia de 3 m), e uma posição de slider
    /// em que a ferramenta visivelmente não faz o que promete é faixa morta.
    /// O piso é 0.05 — a varredura desce a 0.02 sem instabilidade, então 0.05
    /// tem margem em vez de sentar no extremo medido.
    pub const MIN_DAMPING: f32 = 0.05;
    pub const MAX_DAMPING: f32 = 1.0;
    /// ⚠️ **`max_iters` NÃO tem slider, e é decisão da MEDIÇÃO.** Com o
    /// `damping` default, tudo de 10 para cima dá o MESMO erro (1,9705 num
    /// solve; 0,0004 em vinte), e abaixo de 10 a diferença é o que o *warm
    /// start* recupera no frame seguinte. Um knob que a medição mostra inerte é
    /// um knob morto — ele fica no tipo porque a varredura o varre, e fora da
    /// UI porque não há o que escolher.
    pub const MIN_ITERS: usize = 1;
    pub const MAX_ITERS: usize = 40;

    /// A única porta de saneamento. Um `damping` zero ou não-finito faz o
    /// pseudo-inverso explodir perto de uma configuração singular (a cadeia
    /// esticada), e o preço não é um número feio: é `NaN` na pose, que vai parar
    /// no `Transform` autorado e daí no arquivo.
    #[must_use]
    pub fn clamped(self) -> Self {
        Self {
            damping: if self.damping.is_finite() {
                self.damping.clamp(Self::MIN_DAMPING, Self::MAX_DAMPING)
            } else {
                Self::default().damping
            },
            max_iters: self.max_iters.clamp(Self::MIN_ITERS, Self::MAX_ITERS),
            match_angle: self.match_angle,
        }
    }
}

/// A árvore de pose viva — o multibody transitório e o elo que o gesto arrasta.
///
/// Vive **num gesto**. Guarda as coordenadas generalizadas entre chamadas, o que
/// dá ao solver um *warm start*: cada Move continua de onde o anterior parou, em
/// vez de re-semear da cena e perder o caminho que a mão já percorreu.
pub struct IkChain {
    joints: MultibodyJointSet,
    /// O handle do elo da PONTA. `MultibodyJointHandle` é o handle do corpo
    /// FILHO da aresta (`do_insert` devolve `MultibodyJointHandle(body2.0)`),
    /// então a ponta tem de ser um elo não-raiz — e é por isso que
    /// [`PhysicsWorld::ik_chain`] recusa uma ponta que seja a própria raiz.
    tip: MultibodyJointHandle,
    /// Rascunho das coordenadas, reusado entre Moves: o solve roda por
    /// movimento de mouse e alocar um `DVector` por evento é trabalho que não
    /// precisa existir.
    displacements: DVector<f32>,
    /// As juntas que têm limite, com o grau de liberdade que cada uma ocupa.
    /// Vazio quando nenhuma tem — e aí a projeção do §limites é um `for` sobre
    /// nada, byte-idêntico ao mundo sem limites.
    limits: Vec<LimitedDof>,
    /// O teto de passo desta cadeia, em metros — derivado do comprimento dos
    /// elos DELA (ver [`PhysicsWorld::IK_STEP_LINK_FACTOR`]).
    max_step: f32,
    /// **O ALCANCE**: a soma dos vãos, ou seja a distância máxima da raiz à
    /// ponta. Ver o clamp de alcance em `ik_solve_stepped`.
    reach: f32,
}

/// Uma dobradiça com limite, e ONDE ela mora no vetor de coordenadas.
///
/// O `dof` é a soma-prefixo dos `ndofs()` na ordem dos elos — o
/// `assembly_id` do rapier, que é `pub(crate)`. Há gate provando que a soma
/// bate com o comprimento total (`ndofs`), porque uma soma-prefixo que
/// discorda escreveria a correção no grau de liberdade do VIZINHO.
#[derive(Copy, Clone, Debug)]
struct LimitedDof {
    link: usize,
    dof: usize,
    min: f32,
    max: f32,
}

impl IkChain {
    /// Quantos corpos a árvore tem. Um por elo, raiz inclusa.
    #[must_use]
    pub fn len(&self) -> usize {
        self.joints
            .get(self.tip)
            .map_or(0, |(mb, _)| mb.num_links())
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl PhysicsWorld {
    /// **Quanto o alvo pode andar num único solve, em COMPRIMENTOS DE ELO.**
    ///
    /// Ver a nota no `ik_solve_stepped`: é o `clampMag` do DLS. ⚠️ **O número é
    /// adimensional de propósito, e a primeira versão dele — 0,25 m absolutos —
    /// estava errada**: a instabilidade vem do passo ser grande *em relação ao
    /// jacobiano*, que escala com o comprimento do elo. Um teto em metros
    /// protege uma cadeia de 1 m e deixa uma de 0,2 m colapsar exatamente do
    /// mesmo jeito, com o gate da cadeia de 1 m verde. Medido em
    /// `sweep_the_ik_step_factor`: a degradação começa em **2 comprimentos**,
    /// então 1 é a metade disso com a convergência idêntica (4 solves).
    pub const IK_STEP_LINK_FACTOR: f32 = 1.0;

    /// O piso do teto de passo, em metros — a cadeia degenerada (todos os elos
    /// ancorados no mesmo ponto) tem vão zero, e um teto zero congelaria a
    /// ferramenta em vez de a proteger.
    pub const MIN_IK_STEP_M: f32 = 1.0e-3;

    /// Monta a árvore de pose. `links` vem em ordem de BFS a partir de `root`
    /// (todo `parent` já inserido), e `tip` é o corpo que o gesto arrasta.
    ///
    /// Devolve `None` quando a árvore não serve: sem elos, ponta ausente, ou
    /// ponta que é a própria raiz (uma raiz não tem junta a resolver — arrastar
    /// a raiz é uma translação, e essa já é a W-JG).
    ///
    /// ⚠️ Recusa **antes** de construir qualquer coisa um elo não-raiz que não
    /// seja `Dynamic`: o `assert_eq!` do rapier é um PÂNICO, e um pânico dentro
    /// de um arrasto derruba o app com a arte por salvar.
    #[must_use]
    pub fn ik_chain(
        &self,
        root: RigidBodyHandle,
        links: &[IkLink],
        tip: RigidBodyHandle,
    ) -> Option<IkChain> {
        if links.is_empty() || tip == root {
            return None;
        }
        self.bodies.get(root)?;
        let mut joints = MultibodyJointSet::new();
        for l in links {
            // A lei 2 do módulo, conferida em vez de suposta.
            let child = self.bodies.get(l.child)?;
            if !child.is_dynamic() {
                return None;
            }
            self.bodies.get(l.parent)?;
            // `insert` devolve `None` para a aresta que fecharia um ciclo, e
            // ignorá-la É a árvore geradora — não um erro a reportar.
            let _ = joints.insert(l.parent, l.child, multibody_joint(&l.joint), false);
        }
        let tip = MultibodyJointHandle(tip.0);
        let (mb, _) = joints.get_mut(tip)?;
        // Semeia as coordenadas da pose AUTORADA (a raiz inclusive: é a única
        // chamada em que ler a pose do corpo-raiz é o certo — daqui pra frente a
        // raiz pertence ao solver, e re-lê-la desfaria o que ele resolveu).
        mb.forward_kinematics(&self.bodies, true);
        let ndofs = mb.ndofs();
        // Onde cada dobradiça LIMITADA mora no vetor de coordenadas. A soma
        // prefixa `ndofs()` na ordem dos elos, que é como o rapier atribui os
        // `assembly_id` — o campo é `pub(crate)`, então derivamos, com gate.
        let mut limits = Vec::new();
        let mut dof = 0usize;
        for l in mb.links() {
            let n = l.joint().ndofs();
            if n == 1
                && let Some(src) = links.iter().find(|k| k.child == l.rigid_body_handle())
                && let Some([min, max]) = src.joint.limits
                && limit_is_a_coordinate(src.joint.kind)
            {
                limits.push(LimitedDof {
                    link: l.link_id(),
                    dof,
                    min: min.min(max),
                    max: min.max(max),
                });
            }
            dof += n;
        }
        debug_assert_eq!(dof, ndofs, "the prefix sum of ndofs must be the dof count");
        // **O comprimento típico de um elo**, medido na pose AUTORADA: a média
        // da distância entre a origem de cada elo e a do pai. É a régua do teto
        // de passo (ver `IK_STEP_LINK_FACTOR`) e é medida uma vez por gesto,
        // porque é propriedade da CADEIA, não do movimento.
        let mut sum = 0.0f32;
        let mut n = 0u32;
        for l in mb.links() {
            let Some(p) = l.parent_id().and_then(|i| mb.link(i)) else {
                continue;
            };
            let a = l.local_to_world().translation;
            let b = p.local_to_world().translation;
            sum += ((a.x - b.x).powi(2) + (a.y - b.y).powi(2)).sqrt();
            n += 1;
        }
        let span = if n > 0 { sum / n as f32 } else { 0.0 };
        let max_step = (span * Self::IK_STEP_LINK_FACTOR).max(Self::MIN_IK_STEP_M);
        // A SOMA dos mesmos vãos é o alcance — a distância máxima da raiz à
        // ponta, atingida com a cadeia esticada.
        let reach = sum;
        Some(IkChain {
            joints,
            tip,
            displacements: DVector::zeros(ndofs),
            limits,
            max_step,
            reach,
        })
    }

    /// Resolve para que a ponta chegue a `target` e devolve a pose de **todos**
    /// os elos — a cadeia inteira se move, que é a coisa inteira.
    ///
    /// `target_angle` só é lido com `opts.match_angle`; sem ele o solver recebe
    /// a atitude ATUAL da ponta como alvo angular e o eixo angular sai da
    /// máscara, então o resíduo angular já nasce zero e não disputa o jacobiano
    /// com a posição.
    ///
    /// ⚠️ **`&self`, não `&mut self`** — e isto é o desenho, não uma economia:
    /// a IK do rapier lê o `RigidBodySet` (precisa das propriedades de massa e
    /// da pose da raiz) e escreve só no multibody. O mundo simulado **não é
    /// tocado**; quem recebe a pose é o `Transform` autorado, pela ponte.
    pub fn ik_solve(
        &self,
        chain: &mut IkChain,
        target: [f32; 2],
        target_angle: f32,
        opts: IkOptions,
    ) -> Vec<IkPose> {
        let step = chain.max_step;
        self.ik_solve_stepped(chain, target, target_angle, opts, step)
    }

    /// A porta INTERNA: o teto de passo entra como argumento.
    ///
    /// Existe para que a varredura que escolheu [`Self::MAX_IK_STEP_M`] possa
    /// varrê-lo **no caminho do produto** em vez de reimplementar o clamp por
    /// fora — uma sonda que re-implementa o laço fica cega à porta, e uma que o
    /// aplica DUAS vezes (por fora e por dentro) mede outra coisa. É o mesmo
    /// molde do `spawn_joint_tuned`.
    pub(super) fn ik_solve_stepped(
        &self,
        chain: &mut IkChain,
        target: [f32; 2],
        target_angle: f32,
        opts: IkOptions,
        max_step: f32,
    ) -> Vec<IkPose> {
        let opts = opts.clamped();
        let chain_reach = chain.reach;
        let Some((mb, tip_link)) = chain.joints.get_mut(chain.tip) else {
            return Vec::new();
        };
        let ndofs = mb.ndofs();
        if chain.displacements.len() != ndofs {
            chain.displacements = DVector::zeros(ndofs);
        } else {
            chain.displacements.fill(0.0);
        }
        let angle = if opts.match_angle {
            target_angle
        } else {
            // A atitude que a ponta JÁ tem: resíduo angular zero na primeira
            // iteração, e o eixo está fora da máscara de qualquer forma.
            mb.link(tip_link)
                .map_or(0.0, |l| l.local_to_world().rotation.angle())
        };
        // ⚠️ **O ALVO É LIMITADO POR PASSO, e sem isto a cadeia COLAPSA.**
        // O passo dos mínimos quadrados amortecidos é proporcional ao erro
        // (`Jᵀ(JJᵀ+λ²I)⁻¹Δ`), então um alvo a 30 m de uma cadeia de 3 m produz
        // um passo enorme, as juntas giram várias voltas e a configuração que
        // sobra é arbitrária — MEDIDO: arrastar para fora do alcance deixava a
        // ponta a **0,245 m** do gancho, isto é, a cadeia *enrolada sobre si
        // mesma*, quando o certo é ESTICADA na direção do alvo. É o `clampMag`
        // do Buss (2004), a cautela padrão de todo DLS.
        //
        // Um gesto real nunca é limitado por isto (a mão não anda
        // [`Self::MAX_IK_STEP_M`] entre dois frames); quem bate no teto é
        // exatamente o alvo inalcançável, e aí o teto o transforma numa
        // caminhada — cada solve avança um passo e a cadeia acaba esticada.
        // ⚠️ **PRIMEIRO o alvo é trazido para dentro do ALCANCE, e sem isto a
        // cadeia esticada PARA DE GIRAR.** Medido: com um alvo a 36 m, a cadeia
        // de 3 m estica (raio 2,484 de 2,5) e **empaca a 0,098 rad enquanto o
        // alvo está a 0,588** — 28° fora, para sempre, por mais solves que se
        // rode. O gate irmão só afirmava o RAIO e ficava verde sobre isso.
        //
        // O mecanismo: no alcance máximo o jacobiano é singular na direção
        // RADIAL, e um resíduo quase todo radial é justamente o que os mínimos
        // quadrados amortecidos não conseguem atender — sobra quase nada para a
        // componente tangencial, que é a única realizável. Trazer o alvo para a
        // casca do alcance torna o resíduo TANGENCIAL, e aí o solver gira.
        // É o *clamping the target* do Buss (2004) §5, o irmão do `clampMag`.
        let root = mb.link(0).map_or([0.0f32, 0.0], |l| {
            let t = l.local_to_world().translation;
            [t.x, t.y]
        });
        let (rx, ry) = (target[0] - root[0], target[1] - root[1]);
        let far = (rx * rx + ry * ry).sqrt();
        let target = if far > chain_reach && far > 0.0 {
            let k = chain_reach / far;
            [root[0] + rx * k, root[1] + ry * k]
        } else {
            target
        };
        let tip_now = mb.link(tip_link).map_or([target[0], target[1]], |l| {
            let t = l.local_to_world().translation;
            [t.x, t.y]
        });
        let (dx, dy) = (target[0] - tip_now[0], target[1] - tip_now[1]);
        let dist = (dx * dx + dy * dy).sqrt();
        let goal = if dist > max_step && dist > 0.0 {
            let k = max_step / dist;
            [tip_now[0] + dx * k, tip_now[1] + dy * k]
        } else {
            target
        };
        let pose = Isometry2::new(Vector2::new(goal[0], goal[1]), angle);
        let options = InverseKinematicsOption {
            damping: opts.damping,
            max_iters: opts.max_iters,
            constrained_axes: if opts.match_angle {
                JointAxesMask::all()
            } else {
                JointAxesMask::LIN_X | JointAxesMask::LIN_Y
            },
            ..Default::default()
        };
        mb.inverse_kinematics(
            &self.bodies,
            tip_link,
            &options,
            &pose,
            // Toda junta da árvore é livre. *Quais* joints participam já foi
            // decidido na construção (uma Spring nem virou elo), então um
            // segundo filtro aqui seria a mesma pergunta com outra resposta.
            |_| true,
            &mut chain.displacements,
        );
        mb.apply_displacements(chain.displacements.as_slice());
        // Recomputa as poses dos elos a partir das coordenadas novas. `false`:
        // a raiz agora é do solver (ver `ik_chain`).
        mb.forward_kinematics(&self.bodies, false);
        // **Os limites, projetados DEPOIS** — ver [`project_limits`].
        if !chain.limits.is_empty() {
            project_limits(mb, &chain.limits, &mut chain.displacements);
            mb.forward_kinematics(&self.bodies, false);
        }
        mb.links()
            .map(|l| IkPose {
                body: l.rigid_body_handle(),
                translation: [
                    l.local_to_world().translation.x,
                    l.local_to_world().translation.y,
                ],
                rotation: l.local_to_world().rotation.angle(),
            })
            .collect()
    }
}

/// **O limite deste tipo de joint É a coordenada da junta?**
///
/// Para o Pin sim: `local_frame1`/`local_frame2` que [`multibody_joint`] monta
/// carregam **só translação**, então o ângulo de `local_to_parent` É a
/// coordenada, e o limite (radianos) mede exactamente esse número.
///
/// ⚠️ Para o **Slider NÃO**, e isso é honesto em vez de silencioso: o
/// `local_frame1` dele carrega a ROTAÇÃO que leva `+X` ao eixo do trilho, então
/// a pose relativa não entrega a distância percorrida sem desfazer aquele
/// frame. Um Slider limitado **posa sem limite** e o Play o traz de volta ao
/// curso — nomeado no ADR e gateado, para ninguém "descobrir" isso num smoke.
fn limit_is_a_coordinate(kind: JointKind) -> bool {
    matches!(kind, JointKind::Pin)
}

/// **A projeção dos limites, depois do solve.**
///
/// ⚠️ **O `inverse_kinematics` do rapier IGNORA limites** — `apply_displacement`
/// é `integrate(1.0, disp)`, aritmética pura sobre a coordenada, sem clamp
/// (conferido no source, não suposto). MEDIDO: uma dobradiça limitada a
/// `[0, 0.3]` rad, puxada para baixo, dobrava até **−90°**. Uma pose que o
/// solver do Play desfaz no primeiro tick não é uma pose: é uma promessa que o
/// produto quebra assim que o artista aperta Play.
///
/// A cura é uma projeção: leia o ângulo que a junta de fato ficou, clampe, e
/// aplique a DIFERENÇA como mais um deslocamento. Para uma dobradiça (um grau
/// de liberdade, e o ângulo é linear na coordenada) **uma passada é exata** —
/// não é iteração, é uma correção fechada.
///
/// O preço, que é o certo: a ponta deixa de alcançar o alvo quando um limite
/// está no caminho. Um cotovelo que não dobra para trás é um cotovelo que não
/// dobra para trás.
fn project_limits(
    mb: &mut rapier2d::dynamics::Multibody,
    limits: &[LimitedDof],
    scratch: &mut DVector<f32>,
) {
    scratch.fill(0.0);
    let mut any = false;
    for l in limits {
        let Some(link) = mb.link(l.link) else {
            continue;
        };
        let theta = link.local_to_parent().rotation.angle();
        let clamped = theta.clamp(l.min, l.max);
        let delta = clamped - theta;
        if delta != 0.0 && l.dof < scratch.len() {
            scratch[l.dof] = delta;
            any = true;
        }
    }
    if any {
        mb.apply_displacements(scratch.as_slice());
    }
}

/// O joint de coordenadas reduzidas equivalente a um joint autorado.
///
/// ⚠️ **Só os rígidos chegam aqui** — o construtor da árvore não gera aresta
/// para Spring nem Rope (ver a nota do módulo). Os braços existem para o `match`
/// ser exaustivo sem um `_ =>` que engoliria um tipo novo em silêncio: um Weld é
/// a resposta conservadora (trava tudo), que é visivelmente errado num smoke em
/// vez de sutilmente errado.
fn multibody_joint(desc: &JointDesc) -> rapier2d::dynamics::GenericJoint {
    let a = Point2::new(desc.anchor_a[0], desc.anchor_a[1]);
    let b = Point2::new(desc.anchor_b[0], desc.anchor_b[1]);
    match desc.kind {
        JointKind::Pin => {
            let mut builder = RevoluteJointBuilder::new()
                .local_anchor1(a)
                .local_anchor2(b);
            // ⚠️ Os limites do joint VALEM na pose: um cotovelo que não dobra
            // para trás na simulação não pode dobrar para trás ao ser posado, ou
            // a pose que o artista autora é uma que o Play desfaz no 1º tick.
            if let Some([min, max]) = desc.limits {
                builder = builder.limits([min, max]);
            }
            builder.into()
        }
        JointKind::Slider => {
            let mut builder = PrismaticJointBuilder::new(super::joints::unit_or_x(desc.axis_a))
                .local_axis1(super::joints::unit_or_x(desc.axis_a))
                .local_axis2(super::joints::unit_or_x(desc.axis_b))
                .local_anchor1(a)
                .local_anchor2(b);
            if let Some([min, max]) = desc.limits {
                builder = builder.limits([min, max]);
            }
            builder.into()
        }
        JointKind::Weld | JointKind::Spring | JointKind::Rope => FixedJointBuilder::new()
            .local_anchor1(a)
            .local_anchor2(b)
            .into(),
    }
}

/// **Este tipo de joint vira elo de uma árvore de pose?**
///
/// A porta única: o construtor da árvore (na ponte do ECS) pergunta a ELA quais
/// arestas existem, e enumerar os tipos lá seria a lista que nasce incompleta no
/// dia em que um tipo novo chegar.
#[must_use]
pub fn is_rigid_link(kind: JointKind) -> bool {
    matches!(kind, JointKind::Pin | JointKind::Weld | JointKind::Slider)
}

#[cfg(test)]
#[path = "ik_tests.rs"]
mod tests;
