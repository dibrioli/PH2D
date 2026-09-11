//! **A metade que precisa da `App`** do `colorize_smoke` (W2/L5, 2026-09-11).
//!
//! O resto do módulo vive em [`ph2d_app_flip::colorize_smoke`] — as leis e a cena, que não
//! precisam da shell. Aqui fica só o que toca os agregados DELA: `AppGfx` (o `FlipDoc`
//! vivo, o registo de ferramentas), o `HeroScreen` (o barramento dos painéis) e o
//! relógio. É a lista que o substrato da `ph2d-app-host` tem de cobrir para isto
//! também sair (W2 Fase B).

#[allow(unused_imports)]
use ph2d_app_flip::colorize_smoke::*;
use ph2d_core::Vec2;
use ph2d_flip::{Hold, KeyKind};
use ph2d_tool_flip::{FlipMode, FlipTool};
use std::sync::atomic::Ordering;

impl crate::App {
    /// Roda no prólogo do frame (ao lado dos outros smokes). No-op sem a env.
    pub(crate) fn flip_colorize_smoke(&mut self) {
        if !enabled() || self.gfx.is_none() {
            return;
        }
        if FRAME.fetch_add(1, Ordering::Relaxed) != 3 {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("flip"));
        // Entra no modo Colorize já montado.
        if let Some(tool) = gfx
            .tools
            .tool_by_id_mut(&ph2d_editor::ToolId::new("flip"))
            .and_then(|t| t.as_any_mut().downcast_mut::<FlipTool>())
        {
            tool.set_mode(FlipMode::Colorize);
        }

        let oid = gfx.flip.push_object("Colorize Smoke");
        let obj = gfx.flip.object_mut(oid).expect("objeto recém-criado");
        obj.fps = 12.0;
        let l = obj.add_layer("L");
        let Some(d) = obj.insert_frame(l, 0, Hold::Implicit, KeyKind::Keyframe) else {
            return;
        };
        let dr = obj.drawing_mut(d).expect("desenho");

        // ── A MOLDURA: uma caixa [-4,4] × [-2.5,2.5], quatro traços de mão. ──
        for (a, b, s) in [
            (Vec2::new(-4.0, -2.5), Vec2::new(4.0, -2.5), 0),
            (Vec2::new(4.0, -2.5), Vec2::new(4.0, 2.5), 7),
            (Vec2::new(4.0, 2.5), Vec2::new(-4.0, 2.5), 13),
            (Vec2::new(-4.0, 2.5), Vec2::new(-4.0, -2.5), 29),
        ] {
            dr.strokes.push(hand(&seg(a, b, 24), s));
        }
        // ── O DIVISOR OFF-CENTER em x=+1, com um VÃO no meio (y ∈ [-0.6, 0.6]). ──
        dr.strokes.push(hand(
            &seg(Vec2::new(1.0, -2.5), Vec2::new(1.0, -0.6), 41),
            41,
        ));
        dr.strokes
            .push(hand(&seg(Vec2::new(1.0, 0.6), Vec2::new(1.0, 2.5), 53), 53));

        // ── A FATIA C3 (onion fill): mais DUAS chaves com a MESMA arte DESLOCADA. É o que
        //    um desenho animado faz — a linha se move —, e é o que obriga cada quadro a
        //    resolver por conta própria. Marque as três na tira e um Apply colore as três.
        let obj = gfx.flip.object_mut(oid).expect("objeto");
        for (frame, dx) in [(4i32, 0.6f32), (8, 1.2)] {
            let Some(d2) = obj.insert_frame(l, frame, Hold::Implicit, KeyKind::Keyframe) else {
                continue;
            };
            let dr2 = obj.drawing_mut(d2).expect("desenho");
            for (a, b, s) in [
                (Vec2::new(-4.0, -2.5), Vec2::new(4.0, -2.5), 0),
                (Vec2::new(4.0, -2.5), Vec2::new(4.0, 2.5), 7),
                (Vec2::new(4.0, 2.5), Vec2::new(-4.0, 2.5), 13),
                (Vec2::new(-4.0, 2.5), Vec2::new(-4.0, -2.5), 29),
            ] {
                dr2.strokes.push(hand(&seg(a, b, 24), s));
            }
            // O divisor ANDA: é a pose daquele quadro.
            dr2.strokes.push(hand(
                &seg(Vec2::new(1.0 + dx, -2.5), Vec2::new(1.0 + dx, -0.6), 41),
                41,
            ));
            dr2.strokes.push(hand(
                &seg(Vec2::new(1.0 + dx, 0.6), Vec2::new(1.0 + dx, 2.5), 53),
                53,
            ));
        }

        // ⚠️ **A cena ARMA a seleção das três chaves.** Sem isto a C3 ficava atrás de um
        //    gesto que a cena nem dizia onde fica (Shift/Ctrl+clique célula a célula na
        //    tira), e um Apply "normal" coloria só o quadro ativo — indistinguível da
        //    feature quebrada. Ready-to-smoke (plano §8): a fatia nasce clicável.
        //    Para ver o comportamento de UM quadro, clique numa célula da tira SEM
        //    modificador (isso zera a seleção).
        let keys: Vec<i32> = obj
            .layer(l)
            .map(|lay| lay.cells().iter().map(|(k, _, _)| *k).collect())
            .unwrap_or_default();

        // ── Os DOIS rabiscos (MUNDO ≈ LOCAL num objeto fresco): vermelho à esquerda,
        //    azul à direita. Traços verticais, não pontos (um ponto degenera o corte). ──
        self.flip_state.colorize.push_scribble(
            [220, 70, 70, 255],
            seg(Vec2::new(-2.0, -1.5), Vec2::new(-2.0, 1.5), 8),
        );
        self.flip_state.colorize.push_scribble(
            [70, 120, 220, 255],
            seg(Vec2::new(2.6, -1.5), Vec2::new(2.6, 1.5), 8),
        );

        self.flip_state.strip.selection.clone_from(&keys);
        self.playhead.pause();
        // ⚠️ **A cena DIZ o que construiu.** "Não sei se o arquivo de teste está certo" é
        // uma pergunta que o smoke tem de responder sozinho — sem isto, um Apply que colore
        // um quadro só é indistinguível de uma cena com um quadro só.
        eprintln!(
            "\n[colorize-smoke] cena montada: {} chave(s) em {keys:?}, {} marcada(s) na \
             tira. O divisor anda 0.0 / 0.6 / 1.2 entre elas.",
            keys.len(),
            self.flip_state.strip.selection.len()
        );
        eprintln!(
            "\n[colorize-smoke] Modo Colorize. Dois rabiscos ja estao na tela (vermelho esq,\n\
             azul dir) — voce os VE porque o overlay desenha o que esta acumulado.\n\
             \n\
             COMO USAR (o fluxo real):\n\
             1. Cor na swatch **Color** + espessura no **Size** (o MESMO Size do pincel).\n\
             2. **Rabisque DENTRO** de uma regiao — a marca sai sob o cursor, na espessura\n   \
                do Size. O que voce PINTA e o que SEMEIA (um toque grosso ja pega).\n\
             3. Troque a cor, rabisque noutra regiao. Repita.\n\
             4. **Apply** — cada regiao vira preenchimento. **Clear** joga os rabiscos fora.\n\
             5. **Ctrl+Z** com rabisco pendente remove o ULTIMO rabisco (Ctrl+Shift+Z\n   \
                devolve); depois do Apply, Ctrl+Z desfaz o Apply inteiro (um passo).\n\
             \n\
             CONFIRA no Apply:\n\
             a) A FRONTEIRA entre as cores cola na LINHA em x=+1 (fora do centro) — a cor da\n   \
                esquerda e dona de mais area. Se caisse no meio dos rabiscos (~x=0.3), o\n   \
                corte estaria ignorando a tinta.\n\
             b) O VAO no meio do divisor deixa a cor entrar (a LENTE) — e' honesto (vao\n   \
                aberto = passagem). AJUSTES no painel — **AGORA EM TEMPO REAL depois do\n   \
                Apply** (arraste o slider e o corte re-roda ao vivo, sem clicar Apply de novo):\n   \
                · **Bleed**: quanto a cor respeita o vao. **0 = SELA** (as cores param na\n     \
                  linha do divisor, sem lente, mesmo com o buraco aberto); subindo, o vao\n     \
                  abre e a cor entra mais fundo; **1 = flui livre**. O 0 alimenta a bola de\n     \
                  selagem (o pedagio sozinho satura em ~+0.94 e NUNCA fecha o vao largo).\n   \
                · **Trap**: fecha o vao de vez (bola que nao passa por vao < 2r; ate 50 px).\n   \
                (Ctrl+Z desfaz o ajuste; de novo, o Apply. Editar o desenho encerra o ajuste.)\n\
             \n\
             ONION FILL (fatia C3) — a cena ja' vem com TRES chaves (0, 4, 8), o divisor em\n\
             posicoes diferentes, e **as tres JA MARCADAS na tira**. Entao o Apply do passo 4\n\
             ja' e' o multiframe: um clique colore os tres quadros.\n\
             6. Passeie pelos quadros (setas / a tira): as TRES tem de estar coloridas, e a\n   \
                fronteira de cada uma tem de colar no divisor DAQUELE quadro (nao no do\n   \
                ativo) — cada quadro e um solve independente, porque a linha se move.\n\
             7. Arraste o **Trap**/**Bleed**: o ajuste ao vivo re-roda em TODOS os quadros\n   \
                que o gesto escreveu, nao so no ativo.\n\
             8. Para ver UM quadro so': clique numa celula da tira SEM modificador (zera a\n   \
                selecao) e clique Apply de novo. Shift/Ctrl+clique volta a marcar.\n\
             9. Um quadro em que a arte nao fecha falha em SILENCIO — o toast fala pelo\n   \
                quadro ATIVO, que e onde voce esta olhando.\n\
             \n\
             O FANTASMA: o divisor AZULADO a direita do branco e' o onion skin do quadro\n\
             seguinte (a pose para onde a linha anda). Nao e' arte deste quadro — e' o que\n\
             torna o gesto do onion fill possivel: voce rabisca por cima das poses empilhadas.\n"
        );
    }
}
