//! **A metade da família `physics` que não precisa da shell** (W2/L2).
//!
//! A auditoria de velocidade de 2026-09-10 (§4-C2) mediu o mecanismo: o ADR-0075
//! põe cada feature numa drop-crate, e **a metade da feature que fala com a `App`
//! foi ficando na shell** — 22,8 k linhas de física entre elas. Resultado: a crate
//! que TODO módulo recompila é a maior do repo (493 k linhas, 34–45 s sozinha no
//! fim de cada gate), e uma cena de física que só sabe povoar um `World` estava
//! presa lá dentro por uma razão só: ela era um **método de `App`**.
//!
//! Esta crate é o outro lado. O que vive aqui depende de `ph2d-core`, `ph2d-ecs`,
//! `ph2d-physics-ecs` e `ph2d-render` — e **de mais nada**. ⛔ Em particular, não
//! depende da shell: o `shells/desktop` é um `bin`, logo não pode ser dependência
//! de ninguém, e é essa direcção de seta que torna a extracção possível.
//!
//! ## O que uma cena PEDE à shell é DADO, nunca uma chamada
//!
//! Uma cena de smoke faz duas coisas: povoa um `World` (que é desta crate) e pede
//! à shell que enquadre a câmera, abra um painel ou escolha um objecto (que é da
//! shell). A segunda metade viaja como **dados** num [`SceneSetup`] que o roteador
//! aplica depois de a cena correr — ADR-0075 ao pé da letra: *components +
//! events/resources, systems não se chamam*.
//!
//! ⚠️ **Porque não um trait `SceneHost` com métodos:** um trait obrigaria esta
//! crate a declarar a superfície da shell (câmera, painéis, gizmo) e a shell a
//! implementá-la — o acoplamento mudava de forma, não de tamanho. Um registo de
//! intenções é inspeccionável, testável sem janela, e **não cresce** quando a
//! shell muda por dentro. É também o que deixa um gate perguntar *«esta cena pede
//! o painel certo?»* sem GPU nenhuma.

/// O que uma cena pede à shell **depois** de povoar o mundo.
///
/// Tudo aqui é `Option`/`Vec` vazio por omissão: uma cena que não peça nada deixa
/// a shell exactamente como estava, e é isso que torna o registo seguro de passar
/// a **todas** as cenas, inclusive as 44 que só povoam o mundo.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct SceneSetup {
    /// Centro da câmera, em unidades de mundo.
    pub camera_center: Option<[f32; 2]>,
    /// Altura visível da câmera, em unidades de mundo (o «zoom» enquadrado).
    pub camera_height_world: Option<f32>,
    /// Painéis a abrir, pela chave que o `panel_visibility` da shell usa.
    ///
    /// ⚠️ A chave é `&'static str` **de propósito** — é o que a shell já usa, e
    /// mantê-la assim evita que esta crate tenha de conhecer o enum de painéis
    /// dela (que é exactamente a dependência que não pode existir).
    pub panels: Vec<&'static str>,
    /// A entidade que a cena quer deixar escolhida, em bits.
    ///
    /// ⚠️ Bits e não `Entity` porque o consumidor é o gizmo da shell, que guarda
    /// bits; e porque uma cena que escolhe alguém acabou de o criar, logo os bits
    /// são válidos no mesmo quadro.
    pub select: Option<u64>,
    /// Definições de mundo que a cena quer impor (gravidade, sub-passos, …).
    pub settings: Option<ph2d_physics_ecs::PhysicsSettings>,
}

/// O contexto que o roteador passa a uma cena: o mundo para povoar, as definições
/// **vigentes** para quem quer mudar só uma, e o registo do que pedir à shell.
///
/// ⚠️ **As definições entram por VALOR e saem pelo `want`.** Uma cena que quisesse
/// lê-las por referência mutável estaria a escrever no mundo da shell a meio da
/// construção — e o roteador deixaria de poder decidir a ordem em que as coisas
/// se aplicam. Ler o que está lá e declarar o que se quer são dois actos, e o
/// segundo é reversível.
pub struct SceneCtx<'w> {
    /// O mundo ECS que a cena povoa.
    pub world: &'w mut bevy_ecs::world::World,
    /// As definições de física vigentes no momento em que a cena corre.
    pub settings: ph2d_physics_ecs::PhysicsSettings,
    /// O que a cena pede à shell. O roteador aplica-o depois de a cena correr.
    pub want: SceneSetup,
}

impl<'w> SceneCtx<'w> {
    /// Um contexto sobre `world`, com as definições vigentes e nada pedido ainda.
    pub fn new(
        world: &'w mut bevy_ecs::world::World,
        settings: ph2d_physics_ecs::PhysicsSettings,
    ) -> Self {
        Self {
            world,
            settings,
            want: SceneSetup::default(),
        }
    }

    /// Acesso directo ao mundo — o que a esmagadora maioria das cenas usa.
    pub fn world(&mut self) -> &mut bevy_ecs::world::World {
        self.world
    }
}
