//! **A metade que precisa da `App`** do `multiplane_smoke` (W2/L5, 2026-09-11).
//!
//! O resto do módulo vive em [`ph2d_app_flip::multiplane_smoke`] — as leis e a cena, que não
//! precisam da shell. Aqui fica só o que toca os agregados DELA: `AppGfx` (o `FlipDoc`
//! vivo, o registo de ferramentas), o `HeroScreen` (o barramento dos painéis) e o
//! relógio. É a lista que o substrato da `ph2d-app-host` tem de cobrir para isto
//! também sair (W2 Fase B).

#[allow(unused_imports)]
use ph2d_app_flip::multiplane_smoke::*;
use std::sync::atomic::Ordering;

impl crate::App {
    /// Roda no prólogo do frame (ao lado dos outros smokes). No-op sem a env.
    pub(crate) fn flip_multiplane_smoke(&mut self) {
        if !enabled() || self.gfx.is_none() {
            return;
        }
        if FRAME.fetch_add(1, Ordering::Relaxed) != 3 {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        let tool_ok = gfx.tools.set_active(&ph2d_editor::ToolId::new("flip"));

        let oid = gfx.flip.push_object("Multiplane Smoke");
        let obj = gfx.flip.object_mut(oid).expect("objeto recém-criado");
        let planes = stage(obj);

        self.playhead.seek(0.0);
        self.playhead.pause();

        eprintln!(
            "\n[multiplane-smoke] cena montada: 3 planos {planes:?} (nome, depth). \
             Ferramenta flip ativa: {}.",
            if tool_ok {
                "sim"
            } else {
                "NAO (PARE: sem ela a paralaxe nao e dirigida pela tool Flip)"
            }
        );
        eprintln!(
            "\n\
             ============================================================\n\
             ANTES DE TUDO: este terminal imprimiu, logo acima, a linha\n\
             comecando com '[multiplane-smoke] cena montada'? Se NAO,\n\
             PARE: o smoke nao rodou (arvore ou variavel de ambiente\n\
             errada).\n\
             ============================================================\n\
             \n\
             O que esta na tela: uma PAISAGEM em tres planos.\n\
               - FUNDO  : uma serra AZUL-PALIDA atravessando o alto.\n\
               - MEIO   : uma ARVORE verde no centro.\n\
               - FRENTE : uma CERCA laranja embaixo (4 postes).\n\
             Parado, os tres estao alinhados (a camera esta sobre a\n\
             origem, e ai todos os planos coincidem -- e' o certo).\n\
             \n\
             ------------------------------------------------------------\n\
             O TESTE: de PAN na camera (arraste o fundo para o lado, ou\n\
             use o gesto de pan do app) para a ESQUERDA e para a DIREITA.\n\
             ------------------------------------------------------------\n\
             A CERCA (frente) deve correr RAPIDO com a camera; a ARVORE\n\
             (meio) na METADE da velocidade; a SERRA (fundo) quase nao se\n\
             move. Ao panhar 3 unidades de mundo isso e', medido:\n\
                 Cerca  216 px  |  Arvore  108 px  |  Ceu  32 px\n\
             ou seja o deslocamento e' EXATAMENTE depth x pan.\n\
             Se os tres se movem JUNTOS (mesma velocidade), a paralaxe\n\
             esta quebrada -- me diga.\n\
             \n\
             ------------------------------------------------------------\n\
             O AJUSTE: no painel Flip, no bloco de cada camada, ha um\n\
             segundo slider abaixo da Opacity -- e' o DEPTH. Arraste o\n\
             Depth do 'Ceu' para 100% e panhe: agora a serra corre junto\n\
             com a cerca (virou flat). Volte para ~15% e ela volta a\n\
             ficar para tras. Cada camada tem o seu.\n"
        );
    }
}
