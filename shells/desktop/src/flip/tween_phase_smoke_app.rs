//! **A metade que precisa da `App`** do `tween_phase_smoke` (W2/L5, 2026-09-11).
//!
//! O resto do módulo vive em [`ph2d_app_flip::tween_phase_smoke`] — as leis e a cena, que não
//! precisam da shell. Aqui fica só o que toca os agregados DELA: `AppGfx` (o `FlipDoc`
//! vivo, o registo de ferramentas), o `HeroScreen` (o barramento dos painéis) e o
//! relógio. É a lista que o substrato da `ph2d-app-host` tem de cobrir para isto
//! também sair (W2 Fase B).

#[allow(unused_imports)]
use ph2d_app_flip::tween_phase_smoke::*;
use std::sync::atomic::Ordering;

impl crate::App {
    /// Roda no prólogo do frame (ao lado dos outros smokes). No-op sem a env.
    pub(crate) fn flip_tween_phase_smoke(&mut self) {
        if !enabled() || self.gfx.is_none() {
            return;
        }
        if FRAME.fetch_add(1, Ordering::Relaxed) != 3 {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        let _ = gfx.tools.set_active(&ph2d_editor_core::ToolId::new("flip"));

        let oid = gfx.flip.push_object("Phase Smoke");
        let obj = gfx.flip.object_mut(oid).expect("objeto recém-criado");
        obj.fps = 12.0;
        stage(obj);

        self.flip_state.strip.tween_count = 3;
        self.playhead.seek(0.0);
        self.playhead.pause();

        eprintln!(
            "\n[phase-smoke] cena montada: o MESMO blob em 2 quadros (0 e 8), desenhado a \
             partir de pontos de partida diferentes. Tween ja esta em 3."
        );
        eprintln!(
            "\n\
             O QUE ESTA NA TELA\n\
             ==================\n\
             Um BLOB (uma gota/virgula com um narizinho apontando para a DIREITA), a\n\
             esquerda do centro. Ele tem so DOIS desenhos: o quadro 0 (esse) e o quadro 8,\n\
             onde o MESMO blob esta a direita do centro.\n\
             \n\
             A pegadinha: nos dois quadros o blob e' identico em FORMA, mas foi 'desenhado'\n\
             a partir de pontos de partida diferentes -- no quadro 0 o traço fecha no\n\
             nariz; no quadro 8, nas costas. Num traço FECHADO o ponto de partida e'\n\
             arbitrario (e' so onde a linha fecha), e o tween pareia ponto-por-ponto a\n\
             partir dali.\n\
             \n\
             O QUE FAZER\n\
             ===========\n\
             Aperte **Add** na barra da tira. Ele inventa os 3 quadros do meio (2, 4, 6).\n\
             Folheie 0 -> 2 -> 4 -> 6 -> 8 com as setas ^/v (ou clicando nas celulas).\n\
             \n\
             O QUE OLHAR (o quadro 4 -- o do meio)\n\
             =====================================\n\
             \n\
             O BLOB TEM DE DESLIZAR EM LINHA RETA, da esquerda para a direita, sempre em\n\
             pe e do mesmo tamanho.\n\
             \n\
                CERTO  : uma gota inteira que atravessa RETO pelo centro, o narizinho\n\
                         sempre apontando para a direita.\n\
                ERRADO : ela MERGULHA para baixo e faz um LAÇO (uma cambalhota), voltando\n\
                         a subir e chegando de cabeca para baixo -- porque o nariz de um\n\
                         quadro foi pareado com as COSTAS do outro, e a espiral leu isso\n\
                         como uma virada de 180 graus.\n\
             \n\
             (Esse laço e' o que o pareamento por indice faz com traço fechado quando a\n\
              costura muda de lugar. O alinhamento de fase gira o pareamento ate as duas\n\
              formas coincidirem, e ai o giro fantasma some.)\n"
        );
    }
}
