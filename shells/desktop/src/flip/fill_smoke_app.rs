//! **A metade que precisa da `App`** do `fill_smoke` (W2/L5, 2026-09-11).
//!
//! O resto do módulo vive em [`ph2d_app_flip::fill_smoke`] — as leis e a cena, que não
//! precisam da shell. Aqui fica só o que toca os agregados DELA: `AppGfx` (o `FlipDoc`
//! vivo, o registo de ferramentas), o `HeroScreen` (o barramento dos painéis) e o
//! relógio. É a lista que o substrato da `ph2d-app-host` tem de cobrir para isto
//! também sair (W2 Fase B).

#[allow(unused_imports)]
use ph2d_app_flip::fill_smoke::*;
use ph2d_core::Vec2;
use ph2d_flip::{Hold, KeyKind};
use std::sync::atomic::Ordering;

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
