//! ⭐⭐⭐ **ARRUMAR O DOCUMENTO — a porta única, e a única que sabe medir um cartão.**
//!
//! A arrumação em camadas vive no `ph2d-nodegraph` e a versão consciente de grupos no
//! `ph2d-motion-doc`; desde 2026-09-20 nenhuma das duas escolhe o ESPAÇAMENTO sozinha, porque
//! ele depende de quanto cada cartão desenha — e isso é o nome dele (texto medido) e a contagem
//! de fileiras dele (registo + params do cartão). Esta é a porta em que essas duas pontas se
//! encontram.
//!
//! ⚠️ **Ela existe porque há DOIS chamadores** — a tecla de arrumar do painel do grafo
//! (`GraphIntent::ArrangeLayout`) e as cenas de smoke que se arrumam ao nascer
//! ([`crate::smoke_layout`]) — e *duas chamadas à mesma lei divergem no dia em que uma delas
//! ganhar um argumento*. O da direita já tinha divergido antes: ele arrumava sem medir.
//!
//! ⛔ **E o ramo sem painel não é um esquecimento:** a `ph2d-app-motion` compila sem a feature
//! `panel-motion-graph`, e ali não há medidor de texto nenhum — o que se pode fazer é o cartão
//! histórico, que é exactamente a arrumação que este repo teve até hoje.

use crate::motion_state::MotionState;

/// Arruma a raiz e o interior de cada grupo, com cada cartão medido.
pub(crate) fn arrumar(motion: &mut MotionState) {
    #[cfg(feature = "panel-motion-graph")]
    {
        crate::motion_bridge::medida::arrumar(motion);
    }
    #[cfg(not(feature = "panel-motion-graph"))]
    {
        ph2d_motion_doc::layout::arrange(
            &mut motion.doc,
            &ph2d_motion_doc::layout::CartaoDeFabrica,
        );
    }
}
