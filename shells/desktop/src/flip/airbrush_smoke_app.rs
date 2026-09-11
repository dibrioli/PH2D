//! **A metade que precisa da `App`** do `airbrush_smoke` (W2/L5, 2026-09-11).
//!
//! O resto do módulo vive em [`ph2d_app_flip::airbrush_smoke`] — as leis e a cena, que não
//! precisam da shell. Aqui fica só o que toca os agregados DELA: `AppGfx` (o `FlipDoc`
//! vivo, o registo de ferramentas), o `HeroScreen` (o barramento dos painéis) e o
//! relógio. É a lista que o substrato da `ph2d-app-host` tem de cobrir para isto
//! também sair (W2 Fase B).

#[allow(unused_imports)]
use ph2d_app_flip::airbrush_smoke::*;
use std::sync::atomic::Ordering;

impl crate::App {
    /// Roda no prólogo do frame (ao lado dos outros smokes). No-op sem a env.
    pub(crate) fn flip_airbrush_smoke(&mut self) {
        if !enabled() || self.gfx.is_none() {
            return;
        }
        if FRAME.fetch_add(1, Ordering::Relaxed) != 3 {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        let tool_ok = gfx.tools.set_active(&ph2d_editor::ToolId::new("flip"));

        let oid = gfx.flip.push_object("Airbrush Smoke");
        let obj = gfx.flip.object_mut(oid).expect("objeto recem-criado");
        let (x_std, x_air) = stage(obj);

        self.playhead.seek(0.0);
        self.playhead.pause();

        eprintln!(
            "\n[airbrush-smoke] cena montada: 2 tracos grossos a hardness 0.5 -- padrao em x={x_std}, \
             airbrush em x={x_air}. Ferramenta flip ativa: {}.",
            if tool_ok {
                "sim"
            } else {
                "NAO (PARE: sem ela o traco nao e dirigido pela tool Flip)"
            }
        );
        eprintln!(
            "\n\
             ============================================================\n\
             ANTES DE TUDO: este terminal imprimiu, logo acima, a linha\n\
             comecando com '[airbrush-smoke] cena montada'? Se NAO, PARE:\n\
             o smoke nao rodou (arvore ou variavel de ambiente errada).\n\
             ============================================================\n\
             \n\
             O que esta na tela: o MESMO traco grosso, a MESMA hardness\n\
             (0.5), desenhado DUAS vezes:\n\
               - ESQUERDA : Airbrush OFF -- o pincel padrao (a lei do\n\
                            Painter): nucleo CHEIO ate a hardness, depois\n\
                            cai ate zero na borda.\n\
               - DIREITA  : Airbrush ON  -- um DOMO largo: a tinta cobre\n\
                            quase toda a largura antes de rolar suave a\n\
                            zero na borda (borda SEMPRE macia).\n\
             \n\
             ------------------------------------------------------------\n\
             Medido (sonda headless, banda raio 10 a hardness 0.5):\n\
                 eixo (centro)     255 padrao  vs  252 airbrush\n\
                 aro  (dn ~0.8)     55 padrao  vs  231 airbrush\n\
                 aro  (dn ~0.9)      7 padrao  vs  192 airbrush\n\
             ------------------------------------------------------------\n\
             \n\
             O AJUSTE: no painel Flip (modo DRAW), abaixo do toggle Self\n\
             Overlap, ha o toggle 'Airbrush'. Ligado, o slider Hardness\n\
             vira a DENSIDADE da nevoa (0 = tenue, 1 = domo quase solido\n\
             de borda macia). Casa com o Self Overlap: a acumulacao de\n\
             airbrush e o build-up fisico da tinta. Se algo destoar, diga.\n"
        );
    }
}
