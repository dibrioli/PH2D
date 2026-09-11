//! **A metade que precisa da `App`** do `tip_smoke` (W2/L5, 2026-09-11).
//!
//! O resto do módulo vive em [`ph2d_app_flip::tip_smoke`] — as leis e a cena, que não
//! precisam da shell. Aqui fica só o que toca os agregados DELA: `AppGfx` (o `FlipDoc`
//! vivo, o registo de ferramentas), o `HeroScreen` (o barramento dos painéis) e o
//! relógio. É a lista que o substrato da `ph2d-app-host` tem de cobrir para isto
//! também sair (W2 Fase B).

#[allow(unused_imports)]
use ph2d_app_flip::tip_smoke::*;
use std::sync::atomic::Ordering;

impl crate::App {
    /// Roda no prólogo do frame (ao lado dos outros smokes). No-op sem a env.
    pub(crate) fn flip_tip_smoke(&mut self) {
        if !enabled() || self.gfx.is_none() {
            return;
        }
        if FRAME.fetch_add(1, Ordering::Relaxed) != 3 {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("flip"));

        let oid = gfx.flip.push_object("Tip Smoke");
        let obj = gfx.flip.object_mut(oid).expect("objeto recém-criado");
        obj.fps = 12.0;
        stage(obj);

        // A ferramenta Flip já está ativa (o painel aparece). O MODO (Draw) e o *tip* o
        // artista escolhe pelo painel REAL — nada é pré-armado por baixo (a doutrina: o smoke
        // que arma o estado por baixo pula justamente a costura que devia provar).
        self.playhead.seek(0.0);
        self.playhead.pause();

        eprintln!(
            "\n[tip-smoke] cena montada: 3 tracos finos de referencia (Line / Dots / Squares) + 1 GROSSO pontilhado."
        );
        eprintln!(
            "\n\
             O QUE ESTA NA TELA\n\
             ==================\n\
             Quatro tracos horizontais empilhados:\n\
               em CIMA   : uma LINHA cheia (o traco de sempre).\n\
               2o        : CONTAS REDONDAS (dots) fina.\n\
               3o        : CONTAS QUADRADAS (squares) fina.\n\
               em BAIXO  : CONTAS REDONDAS num traco GROSSO -- o report do Enio.\n\
             As quatro usam o MESMO espacamento (2.0). O espacamento e RELATIVO A ESPESSURA\n\
             (um multiplo do diametro do traco), entao o traco grosso mostra o padrao IGUAL\n\
             ao fino -- antes, com espacamento absoluto, o grosso fundia num borrao.\n\
             \n\
             O QUE FAZER (o seletor REAL)\n\
             ============================\n\
             No painel do Flip, clique o modo **Draw**. Na secao **Brush** aparece um seletor\n\
             **Tip** [Line | Dots | Squares] e (com contas) um slider **Spacing**.\n\
             \n\
               1. Clique **Dots**, suba o **Size** para um pincel GROSSO e DESENHE -- as\n\
                  contas aparecem, na mesma razao de um pincel fino (o bug do report).\n\
               2. Troque para **Squares**: as contas viram quadrados.\n\
               3. Arraste **Spacing** (1.0 = encostadas .. 6.0 = bem esparsas).\n\
               4. Volte para **Line**: o Spacing SOME e o traco volta a ser a linha de sempre.\n\
             \n\
             O QUE OLHAR\n\
             ===========\n\
             O padrao tem de aparecer em QUALQUER espessura -- contas REDONDAS/QUADRADAS de\n\
             verdade (nao 'linha tracejada' nem um borrao solido), do tamanho da largura, e o\n\
             Spacing controla o VAO como multiplo do diametro. Zoom in/out: contas mantem o\n\
             tamanho em DOCUMENTO (a espessura e o Size ja medem mundo).\n"
        );
    }
}
