//! **O READOUT de um joint — tudo que o desenho precisa, numa struct só (W-J1).**
//!
//! Um collider é invisível; um joint é MENOS que isso — é uma *relação*, sem
//! geometria nenhuma. Até esta wave o overlay recebia só o par de âncoras
//! ([`PhysicsBridge::joint_anchors`]), então os quatro tipos desenhavam a MESMA
//! figura e tudo que o artista autorou — o alcance de um limite, o comprimento
//! de repouso de uma mola, o quanto de corda sobra — era número cego no §12.
//!
//! ## A regra (plano 02, P2): quem DESENHA lê do MESMO `desc` que o SOLVER consome
//!
//! Os parâmetros aqui saem de `JointRef::rest` — o [`JointDesc`] que foi
//! entregue ao rapier — e as âncoras/poses saem do **solver vivo**. Nunca do
//! componente ECS: um joint cujos corpos não resolvem (nome renomeado) **não
//! está no `self.joints`** e portanto não produz view nenhuma, enquanto o
//! componente segue lá, autorado. Desenhar do componente pintaria um joint que
//! não existe — a única divergência possível entre as duas fontes, e é
//! exatamente a que um gate desta wave prova.
//!
//! É a mesma lei que o `scaled_shape` (W6) já impõe ao contorno do collider:
//! *uma segunda resposta desenha um mundo que o solver não simula, e ninguém lê
//! um número numa screenshot.*

use ph2d_ecs::Entity;

use crate::joint::{JointKind, LengthField};

use super::PhysicsBridge;

/// O que um joint VIVO é, na hora de desenhá-lo.
///
/// Ângulos em radianos, comprimentos e pontos em **metros de mundo** — a
/// fronteira de graus (a UI) é o painel, como em todo o resto do módulo.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct JointView {
    /// A entidade que AUTORA este joint (a que a Hierarquia lista).
    pub entity: Entity,
    /// Qual restrição — decide o vocabulário inteiro do desenho.
    pub kind: JointKind,
    /// As duas âncoras, mundo, **vivas do solver**.
    ///
    /// Coincidem num Pin/Weld em repouso (é o que um pino É) e ficam separadas
    /// numa mola/corda por construção. A separação num tipo que compartilha
    /// ponto é DEFORMAÇÃO — o único fato do modelo que já vinha de graça no
    /// desenho antigo, e agora tem cor própria.
    pub anchor_a: [f32; 2],
    /// Idem, no corpo B.
    pub anchor_b: [f32; 2],
    /// Onde o corpo A ESTÁ. A linha de posse corre âncora→centro: é ela que
    /// responde *"de quem é esta ponta?"* sem abrir painel (o buraco que o
    /// godot-proposals #5778 nomeia nos `node_a`/`node_b` deles).
    pub centre_a: [f32; 2],
    /// Idem, corpo B.
    pub centre_b: [f32; 2],
    /// QUEM é o corpo B. O overlay precisa dele para desenhar o **fantasma**
    /// (W-J3): a silhueta de B na pose que o limite sendo arrastado permitiria.
    /// A view já carrega onde B está e como está virado; esta é a mesma
    /// pergunta — *de quem é essa pose?* — e sem ela o desenhista teria de
    /// re-resolver o nome, que é a segunda resposta que esta linha evita.
    pub body_b: Entity,
    /// Rotação viva do corpo A (rad). O arco de limite é desenhado no frame
    /// DELE — o limite do rapier é sobre o ângulo RELATIVO `θb − θa`.
    pub angle_a: f32,
    /// Rotação viva do corpo B (rad): onde a agulha do arco aponta AGORA.
    pub angle_b: f32,
    /// [`JointKind::Pin`]: a faixa angular autorada, radianos, **relativa**
    /// (`θb − θa`), ou `None` para uma dobradiça livre.
    pub limits: Option<[f32; 2]>,
    /// [`JointKind::Pin`]: a velocidade-alvo do motor, ou `None` se passivo.
    /// Só o **sinal** é desenhado (para que lado gira) — a magnitude é a row.
    pub motor_speed: Option<f32>,
    /// O comprimento que este tipo NOMEIA: repouso na mola, máximo na corda.
    ///
    /// ⚠️ Um campo só para os dois **porque é a mesma pergunta ao desenho** —
    /// *que raio tem o anel?* — e o `kind` ao lado já diz o que ele significa.
    /// Dois campos seriam duas respostas para uma geometria, e o desenhista
    /// teria de escolher entre elas por um `match` que o `kind` já fez.
    pub length: Option<f32>,
    /// **Pulley:** as duas roldanas em MUNDO, ou `None` para todo outro tipo.
    ///
    /// Sem elas o desenho não teria como mostrar uma polia: a corda dela não vai
    /// de âncora a âncora, ela SOBE até uma roldana, atravessa, e desce até a
    /// outra ponta. Um span reto A→B descreveria uma corda que não existe.
    pub wheels: Option<([f32; 2], [f32; 2])>,
    /// **Slider:** a direção do trilho em MUNDO (unitária), ou `None` para todo
    /// outro tipo — que não tem eixo, então não oferece um (o padrão do
    /// `length` acima).
    pub axis: Option<[f32; 2]>,
    /// **Este joint AINDA SEGURA?** `true` depois que ele se rompeu (W-J7).
    ///
    /// ESTADO, não transição — o irmão do `JointBreakEvent`, e os dois existem
    /// porque respondem perguntas diferentes: *ele está segurando?* (todo frame,
    /// o desenho) e *ele acabou de romper, com que carga?* (uma vez, o toast e o
    /// estouro). Um joint rompido continua na cena, com os parâmetros que o
    /// artista autorou — o que parou foi a restrição.
    pub broken: bool,
    /// **O que este joint está segurando AGORA**, newtons e newton-metros — o
    /// pico deste tick (`PhysicsWorld::joint_load`).
    ///
    /// Zero num joint rompido: ele não segura mais nada, e é por isso que o
    /// número que interessa depois de um rompimento é o [`Self::peak`].
    pub load: ph2d_physics::JointLoad,
    /// **O mais forte que ele foi puxado desde que o relógio recomeçou.**
    ///
    /// O número que se DIGITA. O pico de um tick é exato e transiente — um
    /// tranco acaba antes de o artista conseguir lê-lo —, então sem esta marca
    /// d'água ajustar um teto é busca binária sem sinal de retorno
    /// ([[feedback_ergonomics_verdict_is_a_design_bug]]). Num rig parado ele é
    /// igual ao `load`; num rompimento ele **congela na carga que cruzou**.
    pub peak: ph2d_physics::JointLoad,
    /// O teto de força autorado, newtons — `f32::INFINITY` quando o joint não é
    /// quebrável (o `∞ = off` do P7, já resolvido pela ponte).
    pub break_force: f32,
    /// O teto de torque, newton-metros. `∞` fora do Pin, onde ele não pode
    /// disparar (ver [`ph2d_physics::JointLoad`]).
    pub break_torque: f32,
    /// **Este joint está EM VIGOR?** (W-J8.) `false` quando o artista o desligou
    /// — o objeto continua inteiro, com tudo que ele autorou; o que parou foi a
    /// restrição.
    ///
    /// ⚠️ **Irmão do [`Self::broken`], e os dois NÃO são a mesma pergunta** —
    /// embora escrevam a MESMA flag do rapier. Este é AUTORADO (vem do `desc`,
    /// então um Reset o traz desligado); aquele é RUNTIME (o solver o pôs, e um
    /// Reset o traz segurando). Colapsá-los pintaria um joint desligado de
    /// vermelho, com o estouro de ruptura, dizendo que ele *partiu* quando o
    /// artista apenas o desarmou.
    pub active: bool,
}

/// De volta ao vocabulário AUTORADO.
///
/// O par direto vive em [`super::joints::joint_desc`]; os dois `match` são
/// exaustivos, então um 5º tipo de joint **não compila** até que ambos o
/// conheçam — o oposto de uma tabela que se pode esquecer de atualizar.
fn authored_kind(k: ph2d_physics::JointKind) -> JointKind {
    match k {
        ph2d_physics::JointKind::Pin => JointKind::Pin,
        ph2d_physics::JointKind::Spring => JointKind::Spring,
        ph2d_physics::JointKind::Rope => JointKind::Rope,
        ph2d_physics::JointKind::Weld => JointKind::Weld,
        ph2d_physics::JointKind::Slider => JointKind::Slider,
        ph2d_physics::JointKind::Rod => JointKind::Rod,
        ph2d_physics::JointKind::Wheel => JointKind::Wheel,
    }
}

/// Uma polia VIVA, com o que o desenho precisa dela.
///
/// Ela não está no `ImpulseJointSet` (o rapier não tem polia), então
/// [`PhysicsBridge::joint_views`] não a encontraria pela rota dos outros tipos —
/// e uma polia invisível é uma polia que o artista não pode autorar. Este é o
/// registro que o reconcile deixa para o desenho, e é também de onde a tabela
/// que vai ao solver é derivada: **uma lista, duas leituras**.
#[derive(Copy, Clone, Debug)]
pub(super) struct PulleyRecord {
    /// A entidade que AUTORA a polia (a que a Hierarquia lista).
    pub entity: Entity,
    /// As entidades dos dois corpos — `body_b` alimenta o fantasma, como no
    /// resto das views.
    pub entities: (Entity, Entity),
    /// A corda como o solver a recebeu.
    pub desc: ph2d_physics::world::pulley::PulleyDesc,
}

impl PhysicsBridge {
    /// Todo joint VIVO, com o que o desenho precisa (W-J1).
    ///
    /// Irmão mais rico do [`PhysicsBridge::joint_anchors`], que segue existindo
    /// para quem só quer o par de pontos. Um joint que o reconcile não
    /// conseguiu construir não aparece aqui — ver a nota do módulo.
    pub fn joint_views(&self) -> impl Iterator<Item = JointView> + '_ {
        self.joint_views_of_joints().chain(self.pulley_views())
    }

    /// As views das POLIAS, que não vivem no `ImpulseJointSet`.
    ///
    /// Encadeada na `joint_views` em vez de exposta à parte porque para quem
    /// DESENHA um vínculo é um vínculo — dois iteradores no chamador seria a
    /// segunda lista que nasce esquecida quando o desenho ganha um passo.
    fn pulley_views(&self) -> impl Iterator<Item = JointView> + '_ {
        self.pulley_records.iter().filter_map(|r| {
            let pose_a = self.world.body_pose(r.desc.body_a)?;
            let pose_b = self.world.body_pose(r.desc.body_b)?;
            // Pela MESMA porta que a mão usa para desenhar onde ela pegou: o
            // ponto de amarração é body-local e tem de girar com o corpo.
            // Pela MESMA porta que a mão usa para desenhar onde ela pegou.
            let pa = [
                pose_a.translation.x,
                pose_a.translation.y,
                pose_a.rotation.angle(),
            ];
            let pb = [
                pose_b.translation.x,
                pose_b.translation.y,
                pose_b.rotation.angle(),
            ];
            use ph2d_physics::PhysicsWorld;
            Some(JointView {
                entity: r.entity,
                kind: JointKind::Pulley,
                anchor_a: PhysicsWorld::world_from_local_at_pose(pa, r.desc.local_a),
                anchor_b: PhysicsWorld::world_from_local_at_pose(pb, r.desc.local_b),
                centre_a: [pose_a.translation.x, pose_a.translation.y],
                centre_b: [pose_b.translation.x, pose_b.translation.y],
                body_b: r.entities.1,
                angle_a: pose_a.rotation.angle(),
                angle_b: pose_b.rotation.angle(),
                limits: None,
                motor_speed: None,
                length: Some(r.desc.total_length),
                wheels: Some((r.desc.wheel_a, r.desc.wheel_b)),
                axis: None,
                // Uma polia não parte (`JointKind::can_break`): nada mede a
                // reação dela, então não há ruptura a desenhar.
                broken: false,
                // Uma polia inativa não é sequer instalada, então toda view
                // que existe descreve uma corda que está segurando.
                active: true,
                // Nada mede a carga de algo que não está no `ImpulseJointSet`;
                // zero aqui é *não há leitura*, e a §12 não pinta a row porque
                // `can_break` já a recusa.
                load: ph2d_physics::JointLoad {
                    force: 0.0,
                    torque: 0.0,
                },
                peak: ph2d_physics::JointLoad {
                    force: 0.0,
                    torque: 0.0,
                },
                break_force: 0.0,
                break_torque: 0.0,
            })
        })
    }

    fn joint_views_of_joints(&self) -> impl Iterator<Item = JointView> + '_ {
        self.joints.iter().filter_map(|(&entity, j)| {
            let (anchor_a, anchor_b) = self.world.joint_anchors(j.handle)?;
            let pose_a = self.world.body_pose(j.bodies.0)?;
            let pose_b = self.world.body_pose(j.bodies.1)?;
            let kind = authored_kind(j.rest.kind);
            Some(JointView {
                entity,
                kind,
                anchor_a,
                anchor_b,
                centre_a: [pose_a.translation.x, pose_a.translation.y],
                centre_b: [pose_b.translation.x, pose_b.translation.y],
                body_b: j.entities.1,
                angle_a: pose_a.rotation.angle(),
                angle_b: pose_b.rotation.angle(),
                // ⚠️ **Sem re-filtrar por tipo, e é o ponto:** o `rest` É o
                // desc que o solver recebeu, e `joint_desc` já recusou ali todo
                // parâmetro que o tipo ignora (`is_hinge()`). Perguntar de novo
                // aqui seria uma SEGUNDA resposta a *"que parâmetros este tipo
                // usa?"* — e o dia em que as duas discordassem, o desenho
                // mostraria um limite que o solver não impõe, que é
                // precisamente o defeito que esta wave existe para tornar
                // impossível. A 1ª versão re-filtrava; a mutação que apagou o
                // filtro **não sangrou**, e foi assim que a duplicata apareceu.
                limits: j.rest.limits,
                motor_speed: j.rest.motor.map(|m| m.speed),
                // Só uma polia tem roldanas — o padrão do `axis`/`length`.
                wheels: None,
                // Pela porta única `length_field`: o desenho, o gesto de criar
                // e a escrita do anel têm de concordar sobre QUAL campo carrega
                // o comprimento deste tipo, e três respostas independentes é
                // como duas delas passam a discordar em silêncio.
                length: match kind.length_field() {
                    Some(LengthField::Rest) => Some(j.rest.rest_length),
                    Some(LengthField::Max) => Some(j.rest.max_length),
                    None => None,
                },
                // O eixo do trilho em MUNDO, resolvido aqui e não pelo
                // desenhista: `rest.axis_a` está no frame de A, e girar aquilo
                // pela pose viva de A é uma conversão que só pode existir uma
                // vez — duas respostas desenhariam um trilho que o solver não
                // usa (a mesma razão pela qual `limits` sai do `rest`).
                // Perguntado ao SOLVER, nunca derivado do componente: quem
                // rompeu foi o mundo, e o componente segue autorado como estava
                // (é isso que faz um Reset trazer o joint de volta).
                //
                // ⚠️ **`&& j.rest.enabled` (W-J8), e sem isso o botão Active
                // pintaria RUPTURA.** Um joint desligado pelo artista e um
                // rompido pelo solver carregam a MESMA flag do rapier; o que os
                // separa é o `desc`, que só o autorado move. Perguntar apenas ao
                // solver faria desarmar um joint desenhá-lo vermelho, com o
                // estouro — a cena dizendo *partiu* onde o artista disse
                // *desliga*.
                broken: !self.world.joint_is_enabled(j.handle).unwrap_or(true) && j.rest.enabled,
                active: j.rest.enabled,
                // ⚠️ Os dois tetos saem do `rest` — o `JointDesc` que o SOLVER
                // recebeu — e não do componente, pela MESMA lei que `limits`
                // segue duas linhas acima: o desenho tem de mostrar o número que
                // está em vigor, não o que está digitado num painel que a ponte
                // ainda não reconciliou.
                load: self
                    .world
                    .joint_load(j.handle)
                    .unwrap_or(ph2d_physics::JointLoad::ZERO),
                peak: self.joint_peak(entity),
                break_force: j.rest.break_force,
                break_torque: j.rest.break_torque,
                axis: (kind == JointKind::Slider).then(|| {
                    let (s, c) = (pose_a.rotation.im, pose_a.rotation.re);
                    let [x, y] = j.rest.axis_a;
                    [c * x - s * y, s * x + c * y]
                }),
            })
        })
    }
}
