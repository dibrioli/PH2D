//! **A metade que precisa da `App`** do `self_overlap_smoke` (W2/L5, 2026-09-11).
//!
//! O resto do módulo vive em [`ph2d_app_flip::self_overlap_smoke`] — as leis e a cena, que não
//! precisam da shell. Aqui fica só o que toca os agregados DELA: `AppGfx` (o `FlipDoc`
//! vivo, o registo de ferramentas), o `HeroScreen` (o barramento dos painéis) e o
//! relógio. É a lista que o substrato da `ph2d-app-host` tem de cobrir para isto
//! também sair (W2 Fase B).

#[allow(unused_imports)]
use ph2d_app_flip::self_overlap_smoke::*;
use std::sync::atomic::Ordering;

impl crate::App {
    /// Roda no prólogo do frame (ao lado dos outros smokes). No-op sem a env.
    pub(crate) fn flip_self_overlap_smoke(&mut self) {
        if !enabled() || self.gfx.is_none() {
            return;
        }
        if FRAME.fetch_add(1, Ordering::Relaxed) != 3 {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        let tool_ok = gfx.tools.set_active(&ph2d_editor::ToolId::new("flip"));

        let oid = gfx.flip.push_object("Self Overlap Smoke");
        let obj = gfx.flip.object_mut(oid).expect("objeto recem-criado");
        let (x_off, x_on) = stage(obj);

        self.playhead.seek(0.0);
        self.playhead.pause();

        eprintln!(
            "\n[self-overlap-smoke] cena montada: 2 lacos a opacity 0.5 -- OFF em x={x_off}, \
             ON em x={x_on}. Ferramenta flip ativa: {}.",
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
             comecando com '[self-overlap-smoke] cena montada'? Se NAO,\n\
             PARE: o smoke nao rodou (arvore ou variavel de ambiente\n\
             errada).\n\
             ============================================================\n\
             \n\
             O que esta na tela: o MESMO laco (um traco que cruza a si\n\
             mesmo uma vez), a opacity 0.5, desenhado DUAS vezes:\n\
               - ESQUERDA : Self Overlap OFF -- o no do cruzamento fica\n\
                            IGUAL aos bracos (a uniao chapada de sempre).\n\
               - DIREITA  : Self Overlap ON  -- o no do cruzamento fica\n\
                            MAIS ESCURO (duas camadas de tinta compostas),\n\
                            e a passagem mais NOVA aparece por cima.\n\
             \n\
             ------------------------------------------------------------\n\
             Medido (sonda headless, cruzamento a opacity 0.5):\n\
                 braco (1 passagem)   ~128 de 255\n\
                 cruzamento OFF       ~128  (== braco, sem acumulo)\n\
                 cruzamento ON        ~191  (0.75 = 2 camadas 'over')\n\
             ------------------------------------------------------------\n\
             \n\
             O AJUSTE: no painel Flip (modo DRAW), abaixo da linha Tip,\n\
             ha o toggle 'Self Overlap'. Desenhe um rabisco que volte\n\
             sobre si mesmo com opacidade < 1: ligado, o cruzamento\n\
             escurece; desligado, fica a uniao chapada. Uma QUINA afiada\n\
             tambem escurece com ele ligado (bleed de marcador, por\n\
             design); a curva suave NAO. Se algo disso destoar, me diga.\n"
        );
    }
}
