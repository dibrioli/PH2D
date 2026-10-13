//! ⭐ O modo `=18` — **A SOMBRA EM CADA SPRITE** (doc 132 §6, o P6): 2 048 cópias do `Object` (uma
//! imagem da cena) a cair num campo, com `Drop Shadow` em `Each Shape`. O grafo é o do
//! [`crate::motion_state::cada_forma_demo::monta_sprites`] (a chuva da `=138`); aqui só o objecto
//! que ele carimba — a sprite mora na HIERARQUIA, que uma cena de grafo não povoa.

/// Monta a cena: o objecto, o grafo e o roteiro.
pub(super) fn run(cx: &mut crate::motion_scene_ctx::MotionSceneCtx<'_>) {
    super::art::spawn_sprite(cx.sim);
    let m = &mut *cx.motion;
    let Some(cena) =
        crate::motion_state::cada_forma_demo::monta_sprites(&mut m.doc, &m.registry, super::OBJECT)
    else {
        eprintln!("[motion.obj smoke =18] a cena nao montou");
        return;
    };
    m.sinks.push(cena.saida);
    let _ = cx
        .tools
        .set_active(&ph2d_editor_core::ToolId::new("motion"));
    crate::motion_state::cada_forma_demo::announce_sprites();
}
