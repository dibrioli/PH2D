//! **A cena pronta para o smoke do BALDE** (`PH2D_FLIP_FILL_SMOKE=1`).
//!
//! Cobre de uma vez os três fechamentos de 2026-07-18, e a arte é montada para que cada
//! um tenha um lugar onde é **contraditável** — não onde é conveniente:
//!
//! 1. **BUGS #22 — a cor termina no eixo.** A moldura é desenhada com pincel **MACIO**
//!    (`hardness = 0.35`), que é onde a franja vivia: com pincel duro o defeito é
//!    identicamente zero. Preencher e olhar a borda: a cor não pode passar da linha.
//! 2. **O DONUT.** A célula da esquerda tem uma **ilha** solta no meio. Preencher em
//!    volta dela: a ilha **não** pode ser pintada por cima.
//! 3. **A identidade (R3 revogada).** A forma da direita é fechada e sozinha — o balde
//!    põe a cor no PRÓPRIO traço dela. Preencher, ir pro **Sculpt**, selecionar SÓ a
//!    linha e esculpir: a cor tem de ir junto. (A seleção é o que separa as duas rotas;
//!    sem ela as duas empatam.)
//! 4. **O Gap Closure AO VIVO (doc 06 §8).** A caixa de baixo tem um vão DELIBERADO na
//!    parede de cima (0,2 unidade, fora do alcance da solda — ver o comentário da
//!    cena). Ctrl+roda sobe o Gap com o slider acompanhando; o helper VERDE aparece
//!    tapando a boca quando o alcance a atinge, e o clique preenche.
//!
//! Roteiro impresso no terminal quando a cena monta.

use ph2d_core::Vec2;
use ph2d_flip::{FlipStroke, Hold, KeyKind, Point, Rgba};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, Ordering};

static FRAME: AtomicU32 = AtomicU32::new(0);

fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("PH2D_FLIP_FILL_SMOKE").is_some())
}

const INK: Rgba = Rgba::new(0.92, 0.92, 0.95, 1.0);

/// Um traço de mão: os pontos com um tremor determinístico, como o artista desenha.
///
/// ⚠️ **O tremor não é enfeite.** Numa reta perfeita o contorno vetorizado já colapsa nos
/// cantos exatos e as duas rotas do balde dão o mesmo resultado — a arte lisa esconde
/// justamente o que esta cena existe para mostrar.
fn hand(pts: &[Vec2], seed: usize, closed: bool) -> FlipStroke {
    let mut s = FlipStroke::new();
    for (i, p) in pts.iter().enumerate() {
        let h = |k: usize| ((k as u64).wrapping_mul(2_654_435_761) % 1000) as f32 / 1000.0 - 0.5;
        s.push_point(Point {
            pos: Vec2::new(p.x + h(i + seed) * 0.06, p.y + h(i + seed + 91) * 0.06),
            width: 0.28,
            opacity: 1.0,
            color: INK,
        });
    }
    s.closed = closed;
    // **MACIO de propósito** — ver o item 1 do cabeçalho.
    s.hardness = 0.35;
    s
}

/// Amostra um segmento reto em `n` pontos (a mão não desenha com 2 vértices).
fn seg(a: Vec2, b: Vec2, n: usize) -> Vec<Vec2> {
    (0..n)
        .map(|i| {
            let t = i as f32 / (n - 1) as f32;
            Vec2::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t)
        })
        .collect()
}

fn ring(cx: f32, cy: f32, r: f32, n: usize) -> Vec<Vec2> {
    (0..n)
        .map(|i| {
            let t = i as f32 / n as f32 * std::f32::consts::TAU;
            Vec2::new(cx + r * libm::cosf(t), cy + r * libm::sinf(t))
        })
        .collect()
}

impl crate::App {
    /// Roda no prólogo do frame (ao lado dos outros smokes). No-op sem a env.
    pub(crate) fn flip_fill_smoke(&mut self) {
        if !enabled() || self.gfx.is_none() {
            return;
        }
        if FRAME.fetch_add(1, Ordering::Relaxed) != 3 {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("flip"));
        let oid = gfx.flip.push_object("Fill Smoke");
        let obj = gfx.flip.object_mut(oid).expect("objeto recém-criado");
        obj.fps = 12.0;
        let l = obj.add_layer("L");
        let Some(d) = obj.insert_frame(l, 0, Hold::Implicit, KeyKind::Keyframe) else {
            return;
        };
        let dr = obj.drawing_mut(d).expect("desenho");

        // ── ESQUERDA: uma célula fechada por QUATRO traços, com uma ILHA dentro. ──
        //
        // Quatro traços, e não um retângulo, porque é assim que o balde recebe arte de
        // verdade — e é a topologia em que a rota do contorno é exercida (uma forma
        // fechada sozinha vai por outro caminho).
        dr.strokes.push(hand(
            &seg(Vec2::new(-4.6, -2.2), Vec2::new(-0.4, -2.2), 20),
            0,
            false,
        ));
        dr.strokes.push(hand(
            &seg(Vec2::new(-4.6, 2.2), Vec2::new(-0.4, 2.2), 20),
            7,
            false,
        ));
        dr.strokes.push(hand(
            &seg(Vec2::new(-4.4, -2.4), Vec2::new(-4.4, 2.4), 20),
            13,
            false,
        ));
        dr.strokes.push(hand(
            &seg(Vec2::new(-0.6, -2.4), Vec2::new(-0.6, 2.4), 20),
            29,
            false,
        ));
        // A ilha: o buraco do donut.
        dr.strokes.push(hand(&ring(-2.5, 0.0, 0.75, 14), 41, true));

        // ── DIREITA: uma forma fechada SOZINHA (a rota do traço próprio). ──
        dr.strokes.push(hand(&ring(2.6, 0.0, 1.7, 22), 53, true));

        // ── EMBAIXO: a caixa com um VÃO DELIBERADO (doc 06 §8 — o Gap Closure ao vivo).
        //
        // Os números são um TRADE medido, não estética: o vão tem de caber entre dois
        // tetos que apertam em direções opostas. (a) A solda das juntas fecha sozinha
        // até `meia-largura + meia-largura` — com a tinta padrão do smoke (0,28) isso é
        // 0,28 de alcance, então a caixa usa tinta FINA (0,12 ⇒ solda ≤ 0,12) para o
        // vão de 0,2 ficar FORA dela (senão o balde preenche com Gap 0 e o teste morre).
        // (b) O Gap é em unidades de MUNDO (Enio 2026-07-25), com teto 1,0 doc: o vão de
        // 0,2 fecha com o slider ≥ 0,20 — e agora ZOOM-INVARIANTE (aproximar a câmera não
        // faz o vão "sair de alcance": era o *"de perto some"*).
        let thin = |pts: &[Vec2], seed: usize| {
            let mut s = hand(pts, seed, false);
            s.widths_mut().fill(0.12);
            s
        };
        dr.strokes.push(thin(
            &seg(Vec2::new(-1.4, -5.6), Vec2::new(-0.1, -5.6), 12),
            61,
        ));
        dr.strokes.push(thin(
            &seg(Vec2::new(0.1, -5.6), Vec2::new(1.4, -5.6), 12),
            67,
        ));
        dr.strokes.push(thin(
            &seg(Vec2::new(-1.4, -5.6), Vec2::new(-1.4, -7.6), 14),
            71,
        ));
        dr.strokes.push(thin(
            &seg(Vec2::new(1.4, -5.6), Vec2::new(1.4, -7.6), 14),
            79,
        ));
        dr.strokes.push(thin(
            &seg(Vec2::new(-1.4, -7.6), Vec2::new(1.4, -7.6), 16),
            83,
        ));

        self.playhead.pause();
        eprintln!(
            "\n[fill-smoke] Pincel MACIO (hardness 0.35) — e ali que a franja vivia.\n\
             \n\
             1) BUGS #22 — clique DENTRO da moldura da esquerda (fora da ilha) e olhe a\n   \
                borda: a cor tem de PARAR na linha, sem faixa palida por fora.\n\
             2) DONUT — a ilha no meio NAO pode ficar pintada por cima.\n\
             3) IDENTIDADE — preencha a forma da DIREITA, va pro Sculpt, selecione SO a\n   \
                linha dela e esculpa: a cor tem de ir junto. (Sem selecao as duas rotas\n   \
                empatam; e a selecao que as separa.)\n\
             4) GAP AO VIVO — a caixa de BAIXO tem um vao deliberado (0,2 doc) na parede\n   \
                de cima. Com Gap 0 o clique dentro VAZA (toast). Segure Ctrl e role a RODA\n   \
                sobre o canvas: o Gap sobe/desce 0,05 doc por tique (o slider acompanha), e\n   \
                em >= 0,20 um segmento VERDE aparece tapando a boca — clique dentro:\n   \
                preenche. ZOOM-INVARIANTE: o Gap agora mede MUNDO, entao APROXIME bem a\n   \
                camera e o helper NAO some (era o 'de perto some'). A roda SEM Ctrl e' zoom.\n\
             \n\
             Grow e Trap em ZERO para ver a rota nova — armados, o balde cai na rota velha\n\
             de proposito (ela sabe deslocar; a nova poe a fronteira no eixo).\n"
        );
    }
}
