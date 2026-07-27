//! **A MÃO, no nível da ENTIDADE** (W-Grab) — o artista aponta para um objeto,
//! não para um `RigidBodyHandle`.
//!
//! Fino de propósito: a física da mão mora inteira no wrapper
//! ([`ph2d_physics::PhysicsWorld::grab_body`], que explica o modelo e traz as
//! medições). O que este módulo acrescenta é a tradução entidade→handle e —
//! a parte que importa — o **cuidado com o determinismo**.
//!
//! # A mão é a primeira entrada NÃO-REPRODUZÍVEL deste módulo
//!
//! Todo o resto que perturba a sim é reproduzível: os params vivos são CONFIG
//! (reconciliados do ECS a cada dispatch), a pose de um corpo kinematic é
//! respondida pelo [`SceneAtTick`](super::SceneAtTick), o repouso vem do
//! documento. Um puxão de mão não está no documento e não estará: ele é um
//! gesto sobre uma corrida em andamento.
//!
//! Isso quebraria o invariante em que o scrub inteiro se apoia — *o mundo é
//! função do tick, dado o repouso e as curvas* — se o ring de checkpoints
//! guardasse estados afetados por ela. E o modo de falha não seria uma pose
//! errada qualquer: seria **a resposta depender do CACHE**, o mesmo defeito que
//! a auditoria do W4b nomeou (um scrub dentro da janela mostrando a trajetória
//! cutucada e um scrub fora dela replayando do repouso, para o MESMO tick).
//!
//! Duas regras, cada uma com gate:
//!
//! 1. **Pegar DESCARTA o ring, e nada é gravado enquanto a mão está lá** — o
//!    precedente é o [`hold`](super::PhysicsBridge::hold), que descarta pela
//!    mesma razão (*"todo checkpoint descreve uma corrida que acabou"*).
//! 2. **Um rewind SOLTA a mão** — um salto de relógio encerra o cutucão, e o
//!    Reset volta à cena AUTORADA. É também o que Unity e Godot fazem com
//!    edições de play mode.
//!
//! O corolário é o invariante de que o `checkpoint` do wrapper depende: **nenhum
//! checkpoint jamais contém a tralha da mão**.
//!
//! ⚠️ **Dynamic-only, e é FÍSICA:** um joint não move um corpo estático nem um
//! kinematic (massa infinita — o fato que a W-BakeJoint mediu), então a recusa é
//! do wrapper e o chamador fica livre para deixar o gesto de sempre acontecer.

use ph2d_ecs::Entity;

use super::PhysicsBridge;

impl PhysicsBridge {
    /// **Pegar `entity` pelo ponto de MUNDO `world_point`.** `false` quando não
    /// há o que segurar (entidade sem corpo, ou corpo não-dinâmico).
    ///
    /// Descarta o ring de checkpoints em caso de sucesso — ver as regras nos docs
    /// do módulo.
    pub fn grab(&mut self, entity: Entity, world_point: [f32; 2]) -> bool {
        let Some(b) = self.bodies.get(&entity) else {
            return false;
        };
        if !self.world.grab_body(b.handle, world_point) {
            return false;
        }
        // Regra 1. Feito DEPOIS do sucesso: uma recusa não é uma perturbação, e
        // derrubar o cache de um gesto que não aconteceu seria custo puro.
        self.ring.clear();
        true
    }

    /// **A mão andou** — no-op sem mão, para que o chamador possa chamar por
    /// frame sem perguntar.
    pub fn move_grab(&mut self, world_point: [f32; 2]) {
        self.world.move_grab(world_point);
    }

    /// **Soltar** (no-op sem mão — o caminho comum de todo release de botão).
    ///
    /// Não toca na velocidade do corpo: soltar em movimento é um ARREMESSO.
    pub fn release_grab(&mut self) {
        self.world.release_grab();
    }

    /// Há uma mão em voo?
    ///
    /// A ÚNICA fonte desse fato (o wrapper é o dono da tralha). O shell pergunta
    /// a ele em vez de guardar uma cópia — a lição do `last_painter_pushed_entity`
    /// do Painter, onde a segunda cópia de um fato mentiu.
    #[must_use]
    pub fn is_grabbing(&self) -> bool {
        self.world.grabbed_body().is_some()
    }

    /// A mão para DESENHAR: `(cursor, ponto de pega AGORA)`, em mundo.
    ///
    /// `None` sem mão — e também quando o corpo segurado deixou de existir (um
    /// delete no meio do gesto), caso em que não há o que desenhar e o release
    /// limpa a tralha normalmente.
    #[must_use]
    pub fn grab_marks(&self) -> Option<([f32; 2], [f32; 2])> {
        self.world.grab_marks()
    }
}
