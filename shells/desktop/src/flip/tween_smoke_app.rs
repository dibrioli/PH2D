//! **A metade que precisa da `App`** do `tween_smoke` (W2/L5, 2026-09-11).
//!
//! O resto do módulo vive em [`ph2d_app_flip::tween_smoke`] — as leis e a cena, que não
//! precisam da shell. Aqui fica só o que toca os agregados DELA: `AppGfx` (o `FlipDoc`
//! vivo, o registo de ferramentas), o `HeroScreen` (o barramento dos painéis) e o
//! relógio. É a lista que o substrato da `ph2d-app-host` tem de cobrir para isto
//! também sair (W2 Fase B).

#[allow(unused_imports)]
use ph2d_app_flip::tween_smoke::*;
use std::sync::atomic::Ordering;

impl crate::App {
    /// Roda no prólogo do frame (ao lado dos outros smokes). No-op sem a env.
    pub(crate) fn flip_tween_smoke(&mut self) {
        if !enabled() || self.gfx.is_none() {
            return;
        }
        if FRAME.fetch_add(1, Ordering::Relaxed) != 3 {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("flip"));

        let oid = gfx.flip.push_object("Tween Smoke");
        let obj = gfx.flip.object_mut(oid).expect("objeto recém-criado");
        obj.fps = 12.0;
        stage(obj);

        self.flip_state.strip.tween_count = 3;
        self.playhead.seek(0.0);
        self.playhead.pause();

        // ⚠️ **A cena DIZ o que construiu** — sem isto, um Add que gera inbetweens tortos é
        // indistinguível de uma cena montada errada ([[feedback_ready_to_smoke_example]]).
        eprintln!(
            "\n[tween-smoke] cena montada: um bonequinho de palito em 2 quadros \
             (0 e 8), Tween ja esta em 3."
        );
        eprintln!(
            "\n\
             O QUE ESTA NA TELA\n\
             ==================\n\
             Um boneco de palito: um TRONCO em pe, um BRACO saindo do ombro para a\n\
             direita (como o ponteiro de um relogio marcando 3 horas), uma PERNA para a\n\
             esquerda, e um CHAPEUZINHO em cima.\n\
             \n\
             Ele tem so DOIS desenhos: o quadro 0 (esse) e o quadro 8. No quadro 8 o\n\
             braco girou para cima-e-para-a-esquerda (marcando ~10 horas), a perna trocou\n\
             de lado, e o chapeu sumiu.\n\
             \n\
             O QUE FAZER\n\
             ===========\n\
             Aperte **Add** na barra da tira. Ele inventa os 3 quadros do meio (2, 4, 6).\n\
             Depois use as setas ^/v (ou clique nas celulas da tira) para folhear\n\
             0 -> 2 -> 4 -> 6 -> 8, ida e volta. E o boneco se mexendo.\n\
             \n\
             O QUE OLHAR (3 coisas, todas no quadro 4 -- o do meio)\n\
             ======================================================\n\
             \n\
             1) O BRACO TEM DE FICAR DO MESMO TAMANHO.\n\
             \n\
                CERTO  : ele varre um arco, como o ponteiro do relogio indo das 3 para as\n\
                         10 horas. Sempre do mesmo comprimento.\n\
                ERRADO : ele ENCOLHE ate a METADE no quadro 4 e volta a crescer -- como\n\
                         uma antena de radio recolhendo e saindo de novo.\n\
             \n\
                (Esse encolhimento e' o que o Blender faz, e o que nos faziamos ate ontem:\n\
                 a ponta do braco corta o caminho em linha reta em vez de dar a volta.)\n\
             \n\
             2) O TRONCO NAO PODE SE MEXER, NEM UM POUCO.\n\
             \n\
                Ele foi desenhado IGUAL nos dois quadros, entao tem de ficar parado.\n\
             \n\
                Se ele escorregar para baixo ou para o lado, e' porque ele casou o\n\
                tronco com a PERNA -- eu desenhei o quadro 8 na ordem trocada (perna\n\
                primeiro) de proposito, que e' o que um animador faz sem pensar.\n\
             \n\
             3) O CHAPEU (marque **Fade** e aperte Add de novo).\n\
             \n\
                SEM Fade : o chapeu fica parado e inteiro ate o quadro 8, onde some de\n\
                           uma vez. E' o padrao.\n\
                COM Fade : ele vai ficando transparente quadro a quadro E acompanha o\n\
                           movimento -- em vez de ficar pregado no ar enquanto o resto do\n\
                           boneco se mexe embaixo dele.\n\
             \n\
             4) (opcional) O chip **Ease** muda o RITMO, nao o caminho: com 'Ease In' o\n\
                boneco comeca devagar e acelera; com 'Ease Out', o contrario. Aperte Add\n\
                de novo depois de trocar -- ele refaz, nao empilha.\n"
        );
    }
}
