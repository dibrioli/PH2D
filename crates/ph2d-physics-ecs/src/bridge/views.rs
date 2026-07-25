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

use crate::joint::JointKind;

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
    }
}

impl PhysicsBridge {
    /// Todo joint VIVO, com o que o desenho precisa (W-J1).
    ///
    /// Irmão mais rico do [`PhysicsBridge::joint_anchors`], que segue existindo
    /// para quem só quer o par de pontos. Um joint que o reconcile não
    /// conseguiu construir não aparece aqui — ver a nota do módulo.
    pub fn joint_views(&self) -> impl Iterator<Item = JointView> + '_ {
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
                length: match kind {
                    JointKind::Spring => Some(j.rest.rest_length),
                    JointKind::Rope => Some(j.rest.max_length),
                    JointKind::Pin | JointKind::Weld => None,
                },
            })
        })
    }
}
