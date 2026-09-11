//! **A metade que precisa da `App`** do `tween_torsion_smoke` (W2/L5, 2026-09-11).
//!
//! O resto do módulo vive em [`ph2d_app_flip::tween_torsion_smoke`] — as leis e a cena, que não
//! precisam da shell. Aqui fica só o que toca os agregados DELA: `AppGfx` (o `FlipDoc`
//! vivo, o registo de ferramentas), o `HeroScreen` (o barramento dos painéis) e o
//! relógio. É a lista que o substrato da `ph2d-app-host` tem de cobrir para isto
//! também sair (W2 Fase B).

#[allow(unused_imports)]
use ph2d_app_flip::tween_torsion_smoke::*;
use std::sync::atomic::Ordering;

impl crate::App {
    /// Roda no prólogo do frame (ao lado dos outros smokes). No-op sem a env.
    pub(crate) fn flip_tween_torsion_smoke(&mut self) {
        if !enabled() || self.gfx.is_none() {
            return;
        }
        if FRAME.fetch_add(1, Ordering::Relaxed) != 3 {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("flip"));

        let oid = gfx.flip.push_object("Torsion Smoke");
        let obj = gfx.flip.object_mut(oid).expect("objeto recém-criado");
        obj.fps = 12.0;
        stage(obj);

        self.flip_state.strip.tween_count = 3;
        self.playhead.seek(0.0);
        self.playhead.pause();

        eprintln!(
            "\n[torsion-smoke] cena montada: uma ASA em 2 quadros (0 e 8). Tween ja esta em 3."
        );
        eprintln!(
            "\n\
             O QUE ESTA NA TELA\n\
             ==================\n\
             Uma ASA (um braço saindo do ombro para a DIREITA, quase reto, com uma corcovinha\n\
             de leve para cima). Ela tem so DOIS desenhos: o quadro 0 (esse) e o quadro 8,\n\
             onde a MESMA asa girou ~160 graus (agora aponta para a ESQUERDA-e-para-baixo) E\n\
             ganhou uma corcova bem MAIOR.\n\
             \n\
             O QUE FAZER\n\
             ===========\n\
             Aperte **Add** na barra da tira. Ele inventa os 3 quadros do meio (2, 4, 6).\n\
             Folheie 0 -> 2 -> 4 -> 6 -> 8 com as setas ^/v (ou clicando nas celulas).\n\
             \n\
             O QUE OLHAR (o quadro 4 -- o do meio)\n\
             =====================================\n\
             \n\
             A asa tem de girar meia-viagem (~80 graus) COM a corcova crescendo do lado\n\
             CERTO -- a corcova acompanha o corpo enquanto ele gira.\n\
             \n\
                CERTO  : uma asa meio-girada, com uma corcova de tamanho intermediario\n\
                         apontando para FORA da curva, na atitude do corpo a 80 graus.\n\
                ERRADO : a asa fica quase RETA no meio (a corcova ACHATA/some), ou a\n\
                         corcova aponta para o lado errado -- porque o resíduo (o crescimento\n\
                         da corcova) foi somado a 160 graus enquanto o corpo so girou 80, e\n\
                         os dois se cancelam.\n\
             \n\
             (Esse achatamento e' a torção do resíduo sob giro grande. A co-rotação gira o\n\
              resíduo JUNTO com o corpo, e a corcova do meio aparece inteira e no lugar.)\n"
        );
    }
}
