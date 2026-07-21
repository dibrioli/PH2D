//! **A cena pronta para o smoke do COLORIZE** (`PH2D_FLIP_COLORIZE_SMOKE=1`).
//!
//! A arte é a que EXPÕE o valor do LazyBrush — o que só ele faz: uma moldura dividida por
//! uma linha **fora do centro** com um **VÃO no meio**, e DOIS rabiscos (um de cada lado).
//! Clicar **Apply** tem de:
//!
//! 1. **A cor segue a TINTA, não o meio.** O divisor está em x≈+1 (não no centro), então a
//!    cor da ESQUERDA é dona de mais área — a fronteira entre as cores cola na linha, não no
//!    ponto médio dos rabiscos.
//! 2. **O vão NÃO vaza.** O buraco no divisor está aberto, mas a cor não escorre por ele: o
//!    corte paga a largura do vão em branco em vez de contornar. É o *"um vão não precisa
//!    fechar"* — o balde comum precisaria fechar o vão com o Gap Closure.
//!
//! Dois rabiscos vêm pré-semeados **e VISÍVEIS** (o overlay ao vivo), então dá para conferir
//! o Apply de imediato — e depois rabiscar os seus, vendo a marca sair sob o cursor.

use ph2d_core::Vec2;
use ph2d_flip::{FlipStroke, Hold, KeyKind, Point, Rgba};
use ph2d_tool_flip::{FlipMode, FlipTool};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, Ordering};

static FRAME: AtomicU32 = AtomicU32::new(0);

fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("PH2D_FLIP_COLORIZE_SMOKE").is_some())
}

const INK: Rgba = Rgba::new(0.92, 0.92, 0.95, 1.0);

/// Um traço de mão (tremor determinístico) — numa reta perfeita o contorno colapsa e as
/// rotas empatam; a arte trêmula é o que a cena existe para exercer.
fn hand(pts: &[Vec2], seed: usize) -> FlipStroke {
    let mut s = FlipStroke::new();
    for (i, p) in pts.iter().enumerate() {
        let h = |k: usize| ((k as u64).wrapping_mul(2_654_435_761) % 1000) as f32 / 1000.0 - 0.5;
        s.push_point(Point {
            pos: Vec2::new(p.x + h(i + seed) * 0.05, p.y + h(i + seed + 91) * 0.05),
            width: 0.26,
            opacity: 1.0,
            color: INK,
        });
    }
    s.hardness = 0.35;
    s
}

/// Amostra um segmento reto em `n` pontos.
fn seg(a: Vec2, b: Vec2, n: usize) -> Vec<Vec2> {
    (0..n)
        .map(|i| {
            let t = i as f32 / (n - 1) as f32;
            Vec2::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t)
        })
        .collect()
}

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

        // ── Os DOIS rabiscos (MUNDO ≈ LOCAL num objeto fresco): vermelho à esquerda,
        //    azul à direita. Traços verticais, não pontos (um ponto degenera o corte). ──
        self.flip_colorize.push_scribble(
            [220, 70, 70, 255],
            seg(Vec2::new(-2.0, -1.5), Vec2::new(-2.0, 1.5), 8),
        );
        self.flip_colorize.push_scribble(
            [70, 120, 220, 255],
            seg(Vec2::new(2.6, -1.5), Vec2::new(2.6, 1.5), 8),
        );

        self.playhead.pause();
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
             ONION FILL (fatia C3) — a cena tem TRES chaves (0, 4, 8) com o divisor em\n\
             posicoes DIFERENTES:\n\
             6. Marque as tres chaves na tira (multi-selecao) e clique **Apply** UMA vez.\n\
             7. Passeie pelos quadros: as TRES tem de estar coloridas, e a fronteira de cada\n   \
                uma tem de colar no divisor DAQUELE quadro (nao no do quadro ativo) — cada\n   \
                quadro e um solve independente, porque a linha se move.\n\
             8. Arraste o **Trap**/**Bleed**: o ajuste ao vivo re-roda em TODOS os quadros\n   \
                que o gesto escreveu, nao so no ativo.\n\
             9. Um quadro em que a arte nao fecha falha em SILENCIO — o toast fala pelo\n   \
                quadro ATIVO, que e onde voce esta olhando.\n"
        );
    }
}
