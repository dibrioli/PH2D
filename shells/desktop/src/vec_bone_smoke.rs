//! ⭐⭐⭐ **O DESENHO GANHA OSSOS** — `PH2D_VEC_BONE_SMOKE=1` (estudo 42, item 5).
//!
//! # O que a cena fecha
//!
//! Até 2026-09-06 um personagem deste editor só animava como **recorte de papel**: a timeline movia
//! a POSE de uma forma inteira, e mais nada. Dobrar um braço exigia desenhá-lo outra vez.
//!
//! # A cena, e o que cada peça prova
//!
//! | O quê | O que ela prova |
//! |---|---|
//! | **O BRAÇO** (barra com 3 ossos, já presa) | girar um osso DOBRA o desenho, e a base fica onde está |
//! | **O TENTÁCULO** (barra com 6 ossos, já presa) | a cadeia inteira: girar a RAIZ leva tudo, girar a PONTA leva só a ponta — a cinemática é a hierarquia |
//! | **A FOLHA SOLTA** (uma forma e um esqueleto **não ligados**) | o gesto do *Bind*: seleccionar a forma, carregar no botão, e ela passa a obedecer |
//!
//! ⚠️ **As duas primeiras são ligadas pela cena**, e é de propósito: sem uma peça que já obedece, o
//! primeiro gesto do artista seria montar um rig do zero para só então descobrir se ele funciona.
//!
//! ⚠️ Se a linha `[vec-bone-smoke]` não aparecer, PARE: a cena não montou.
//!
//! # ⚠️ Ela monta em DOIS tempos, e não é conforto
//!
//! Prender uma forma exige a **entidade** dela, e quem a cria é o `vec_entities::sync`, que corre
//! **depois** deste prólogo. Ligar no mesmo quadro em que se desenha encontraria o mapa vazio e
//! prenderia zero formas — em silêncio.

use ph2d_ecs::Entity;
use ph2d_vec_scene::ShapeKind;

use crate::build_smoke::shape;

/// Uma cadeia de `n` ossos de `a` a `b` (mundo), o 1.º sem pai. Devolve a RAIZ.
fn cadeia(sim: &mut ph2d_ecs::SimWorld, a: [f64; 2], b: [f64; 2], n: usize) -> Option<Entity> {
    #[expect(
        clippy::cast_precision_loss,
        reason = "n é a contagem de ossos da cena, sempre um punhado"
    )]
    let passo = [(b[0] - a[0]) / n as f64, (b[1] - a[1]) / n as f64];
    let mut pai: Option<Entity> = None;
    let mut raiz: Option<Entity> = None;
    for i in 0..n {
        #[expect(clippy::cast_precision_loss, reason = "idem")]
        let t = i as f64;
        let o = [a[0] + passo[0] * t, a[1] + passo[1] * t];
        let p = [o[0] + passo[0], o[1] + passo[1]];
        let bits = crate::bone_gesture::create(sim, pai, o, p)?;
        pai = Some(Entity::from_bits(bits));
        raiz = raiz.or(pai);
    }
    raiz
}

impl crate::App {
    /// No prólogo do frame. No-op sem a env.
    pub(crate) fn vec_bone_smoke(&mut self) {
        if std::env::var_os("PH2D_VEC_BONE_SMOKE").is_none() || self.gfx.is_none() {
            return;
        }
        match self.vec_bone_smoke_step {
            0 => self.bone_smoke_build(),
            // Um quadro de folga: é nele que o `vec_entities::sync` dá entidade às três formas.
            1 => self.vec_bone_smoke_step = 2,
            2 => self.bone_smoke_bind(),
            _ => {}
        }
    }

    /// O 1.º tempo: a arte e os três esqueletos.
    fn bone_smoke_build(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("vector"));
        // ⭐ O BRAÇO e o TENTÁCULO: barras deitadas, com a cadeia pelo MEIO delas.
        let braco = gfx.vec_scene.push_path(shape(
            ShapeKind::RoundRect,
            [-8.5, 2.0],
            [-1.5, 3.0],
            &[0.5],
            [230, 170, 90],
        ));
        let tentaculo = gfx.vec_scene.push_path(shape(
            ShapeKind::RoundRect,
            [-8.5, -0.5],
            [0.5, 0.3],
            &[0.4],
            [110, 190, 160],
        ));
        // A FOLHA SOLTA: a forma e o esqueleto existem, e **não se conhecem**.
        let folha = gfx.vec_scene.push_path(shape(
            ShapeKind::Ellipse,
            [2.5, -4.5],
            [8.5, -2.5],
            &[],
            [180, 140, 220],
        ));
        let a = cadeia(&mut gfx.sim, [-8.2, 2.5], [-1.8, 2.5], 3);
        let t = cadeia(&mut gfx.sim, [-8.2, -0.1], [0.2, -0.1], 6);
        let f = cadeia(&mut gfx.sim, [3.0, -3.5], [8.0, -3.5], 2);
        self.vec_bone_smoke_pend = Some([(braco, a), (tentaculo, t), (folha, f)]);
        self.vec_bone_smoke_step = 1;
    }

    /// O 2.º tempo: prende as DUAS primeiras. A folha fica solta de propósito.
    fn bone_smoke_bind(&mut self) {
        self.vec_bone_smoke_step = 3;
        let Some(pecas) = self.vec_bone_smoke_pend.take() else {
            return;
        };
        let gfx = self.gfx.as_mut().expect("gfx");
        let mut presas = 0;
        for (id, raiz) in pecas.iter().take(2) {
            presas += crate::skin_live::bind(
                &mut gfx.sim,
                &gfx.vec_scene,
                &self.vec_entities,
                &[*id],
                *raiz,
            );
        }
        eprintln!(
            "[vec-bone-smoke] {presas} forma(s) presa(s): o BRACO (3 ossos) e o TENTACULO (6). A \
             FOLHA roxa tem esqueleto e NAO esta' presa -- seleccione-a e carregue em `Bind to \
             Skeleton`. Para POSAR, fique na ferramenta Bone: arraste o CORPO de um osso para o \
             girar, ou a BOLINHA da junta para o deslocar."
        );
    }
}
