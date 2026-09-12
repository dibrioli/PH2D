//! **A shell PREENCHE o contexto das cenas de Motion** — a metade que fica quando a família sai.
//!
//! ⚠️ **Este ficheiro NÃO é da família**, e o nome diz-lho: ele está em `src/`, não em
//! `src/motion/`. O que sai no corte são os CORPOS das cenas; o que decide *de onde vêm os
//! dez campos* é composição, e composição é da shell (HOWTO §4).
//!
//! ⭐ **Ele é a razão de as catorze cenas deixarem de nomear a `App`.** Antes, cada uma abria
//! com `app.gfx.is_none()` e destravava o `gfx` sozinha — catorze cópias da mesma pergunta. Hoje
//! a pergunta é feita **uma vez**, aqui, onde a resposta existe; uma cena que corre já tem tudo.
//!
//! ⚠️⚠️ **O que faz isto compilar é o EMPRÉSTIMO DISJUNTO DE CAMPOS:** `self.gfx` e
//! `self.vec_entities` são campos diferentes da mesma struct, logo podem ser emprestados
//! mutavelmente ao mesmo tempo. ⛔ Um método `fn gfx_mut(&mut self)` no meio disto quebraria a
//! propriedade — ele empresta a `App` INTEIRA, e os cinco campos seguintes deixariam de estar
//! disponíveis. *É a mesma razão pela qual o `AppHost` não devolve handles.*

use ph2d_app_motion::motion_scene_ctx::MotionSceneCtx;

impl crate::App {
    /// Corre `f` com o contexto das cenas de Motion montado. **Sem `gfx` é no-op** — devolve
    /// `None` e a cena nem chega a ser chamada.
    pub(crate) fn with_motion_scene<R>(
        &mut self,
        f: impl FnOnce(&mut MotionSceneCtx<'_>) -> R,
    ) -> Option<R> {
        let gfx = self.gfx.as_mut()?;
        let mut cx = MotionSceneCtx {
            motion: &mut gfx.motion,
            tools: &mut gfx.tools,
            sim: &mut gfx.sim,
            vec_scene: &mut gfx.vec_scene,
            flip: &mut gfx.flip,
            hero: gfx.hero_screen.as_mut(),
            vec_entities: &mut self.vec_entities,
            motion_shell: &mut self.motion_shell,
            flip_state: &self.flip_state,
            playhead: &mut self.playhead,
            timeline: &mut self.timeline,
        };
        Some(f(&mut cx))
    }
}
