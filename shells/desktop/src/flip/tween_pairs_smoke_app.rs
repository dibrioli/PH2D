//! **A metade que precisa da `App`** do `tween_pairs_smoke` (W2/L5, 2026-09-11).
//!
//! O resto do módulo vive em [`ph2d_app_flip::tween_pairs_smoke`] — as leis e a cena, que não
//! precisam da shell. Aqui fica só o que toca os agregados DELA: `AppGfx` (o `FlipDoc`
//! vivo, o registo de ferramentas), o `HeroScreen` (o barramento dos painéis) e o
//! relógio. É a lista que o substrato da `ph2d-app-host` tem de cobrir para isto
//! também sair (W2 Fase B).

#[allow(unused_imports)]
use ph2d_app_flip::tween_pairs_smoke::*;
use std::sync::atomic::Ordering;

impl crate::App {
    /// Roda no prólogo do frame (ao lado dos outros smokes). No-op sem a env.
    pub(crate) fn flip_tween_pairs_smoke(&mut self) {
        if !enabled() || self.gfx.is_none() {
            return;
        }
        if FRAME.fetch_add(1, Ordering::Relaxed) != 3 {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        let _ = gfx.tools.set_active(&ph2d_editor_core::ToolId::new("flip"));

        let oid = gfx.flip.push_object("Pairs Smoke");
        let obj = gfx.flip.object_mut(oid).expect("objeto recém-criado");
        obj.fps = 12.0;
        stage(obj);

        self.flip_state.strip.tween_count = 3;
        self.playhead.seek(0.0);
        self.playhead.pause();
        // **Pairs JÁ ABERTO** — o overlay aparece de cara, senão o artista teria de saber
        // ligar o toggle antes de ver qualquer coisa (um smoke que não mostra a feature na
        // largada não é ready-to-smoke).
        self.flip_state.strip.tween_correct = ph2d_app_flip::tween_correct::build(
            &self.gfx.as_ref().unwrap().flip,
            None,
            &self.playhead,
        );

        eprintln!(
            "\n[pairs-smoke] cena montada: um corpo compacto + uma FAISCA que salta de um \
             lado ao outro. Pairs ja esta ABERTO."
        );
        eprintln!(
            "\n\
             O QUE ESTA NA TELA\n\
             ==================\n\
             Duas POSES sobrepostas do mesmo desenho: a de PARTIDA em AZUL frio, a de\n\
             CHEGADA em LARANJA quente. Cada par de tracos (o que vira o que) esta ligado\n\
             por uma LINHA, pintada pela CONFIANCA da correspondencia:\n\
             \n\
                VERDE   : casou com confianca.\n\
                VERMELHO: casou, mas duvidoso -- o candidato a corrigir.\n\
                AMBAR   : voce corrigiu esse par a mao.\n\
             \n\
             E um traco SEM par ganha um ANEL MAGENTA: ele SOME (se estava na partida) ou\n\
             NASCE do nada (se estava na chegada) no meio do tween.\n\
             \n\
             NESTA CENA: no meio esta o CORPO (um tronco vertical + uma cabeca em losango --\n\
             parece uma chave, mas e' so o sujeito estavel do teste). Ele mal se move, entao\n\
             o tronco e a cabeca casam VERDE. De cada lado do corpo ha uma FAISCA pequena\n\
             (traco curto): ela pula de lado E muda de comprimento, o automatico DESISTE\n\
             dela, e as DUAS (a da esquerda e a da direita) ganham um ANEL MAGENTA, sem\n\
             linha ligando -- e' O QUE VOCE VAI PAREAR.\n\
             \n\
             O QUE FAZER\n\
             ===========\n\
             1) Clique na FAISCA da ESQUERDA (a azul). Ela fica BRANCA (marcada).\n\
             2) Clique na FAISCA da DIREITA (a laranja). Pronto: nasce uma linha AMBAR\n\
                ligando as duas -- voce forcou o par.\n\
             3) Aperte **Add**. Folheie 0 -> 2 -> 4 -> 6 -> 8.\n\
             \n\
             O QUE OLHAR\n\
             ===========\n\
             \n\
             SEM a correcao (so aperte Add sem parear a faisca):\n\
                a faisca fica PARADA na esquerda ate o quadro 8, onde PISCA para a direita\n\
                de uma vez. Ela nao viaja -- some e reaparece.\n\
             \n\
             COM a correcao (pareou a faisca, depois Add):\n\
                a faisca ATRAVESSA a tela quadro a quadro, da esquerda para a direita.\n\
                E' o par forcado dirigindo o movimento.\n\
             \n\
             OUTROS GESTOS\n\
             =============\n\
             - Clicar um traco JA marcado (a mesma faisca de novo) CORTA o par: ele volta a\n\
               ser orfao (o anel magenta reaparece).\n\
             - Clicar no vazio DESMARCA.\n\
             - Desligar **Pairs** joga a correcao fora (ela so vira desenho no Add).\n"
        );
    }
}
