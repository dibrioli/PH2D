//! **A cena pronta para o smoke da CORREÇÃO DE PARES** (`PH2D_FLIP_TWEEN_PAIRS_SMOKE=1`).
//!
//! O matcher automático é bom, mas a política dele é *na dúvida, orfanar o outlier* — um
//! traço solto que SALTA muito longe (enquanto o resto quase não anda) é recusado como
//! "provavelmente não é o mesmo traço". Às vezes ele ESTÁ certo; às vezes o artista sabe que
//! aquela faísca É a mesma, e ela deveria VIAJAR em vez de piscar. Esta cena arma exatamente
//! esse caso, e o Pairs já vem ABERTO para o overlay aparecer de cara.
//!
//! **O corpo** (tronco + cabeça) mal se move ⇒ pareia com confiança (linhas VERDES). **A
//! faísca** salta de um lado ao outro ⇒ o automático a ORFANA nos dois quadros (dois anéis
//! magenta). O artista clica a faísca de A, depois a de B ⇒ o par é forçado (linha ÂMBAR), e
//! o Add faz a faísca **atravessar** em vez de sumir e reaparecer.

use ph2d_core::Vec2;
use ph2d_flip::{FlipStroke, Hold, KeyKind, Point, Rgba};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, Ordering};

static FRAME: AtomicU32 = AtomicU32::new(0);

fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("PH2D_FLIP_TWEEN_PAIRS_SMOKE").is_some())
}

const INK: Rgba = Rgba::new(0.92, 0.92, 0.95, 1.0);
const SPARK: Rgba = Rgba::new(1.0, 0.85, 0.3, 1.0);

fn line(pts: &[Vec2], color: Rgba, closed: bool) -> FlipStroke {
    let mut s = FlipStroke::new();
    for p in pts {
        s.push_point(Point {
            pos: *p,
            width: 0.22,
            opacity: 1.0,
            color,
        });
    }
    s.closed = closed;
    s
}

fn seg(a: Vec2, b: Vec2, n: usize) -> Vec<Vec2> {
    (0..n)
        .map(|i| {
            let t = i as f32 / (n - 1) as f32;
            Vec2::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t)
        })
        .collect()
}

/// **Monta as duas chaves** — o corpo compacto (pareia certo) + a faísca que salta (o
/// automático orfana). Porta única: o gate encena pela MESMA função (senão a mensagem
/// impressa descreveria um desenho que ninguém mais produz).
pub(crate) fn stage(obj: &mut ph2d_flip::FlipObject) -> ph2d_flip::LayerId {
    let l = obj.add_layer("L");
    let torso = || {
        line(
            &seg(Vec2::new(0.0, 1.0), Vec2::new(0.0, -0.6), 8),
            INK,
            false,
        )
    };
    // Uma "cabeça" — um losango fechado no topo (fechado × fechado pareia; não é bloqueado).
    let head = || {
        line(
            &[
                Vec2::new(0.0, 1.5),
                Vec2::new(0.3, 1.2),
                Vec2::new(0.0, 0.9),
                Vec2::new(-0.3, 1.2),
            ],
            INK,
            true,
        )
    };
    // A faísca: um traço curto. Salta de x=-5 (chave 0) para x=+5 (chave 8), e é desenhada
    // um pouco mais longa na chegada. O salto (centróide) + a diferença de comprimento levam
    // o custo do par BEM acima do teto de recusa (0.38), então o automático a ORFANA — que é
    // exatamente o que o gate `the_scene_orphans_the_spark_until_paired` confirma.
    let spark = |x: f32, len: f32| {
        line(
            &seg(Vec2::new(x, 0.2), Vec2::new(x + len, 0.2), 4),
            SPARK,
            false,
        )
    };

    if let Some(d0) = obj.insert_frame(l, 0, Hold::Implicit, KeyKind::Keyframe) {
        let dr = obj.drawing_mut(d0).expect("desenho");
        dr.strokes.push(torso());
        dr.strokes.push(head());
        dr.strokes.push(spark(-5.0, 0.6)); // a faísca à ESQUERDA (curta)
    }
    if let Some(d8) = obj.insert_frame(l, 8, Hold::Implicit, KeyKind::Keyframe) {
        let dr = obj.drawing_mut(d8).expect("desenho");
        dr.strokes.push(torso());
        dr.strokes.push(head());
        dr.strokes.push(spark(4.4, 1.2)); // a faísca à DIREITA (mais longa)
    }
    l
}

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
        let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("flip"));

        let oid = gfx.flip.push_object("Pairs Smoke");
        let obj = gfx.flip.object_mut(oid).expect("objeto recém-criado");
        obj.fps = 12.0;
        stage(obj);

        self.flip_strip.tween_count = 3;
        self.playhead.seek(0.0);
        self.playhead.pause();
        // **Pairs JÁ ABERTO** — o overlay aparece de cara, senão o artista teria de saber
        // ligar o toggle antes de ver qualquer coisa (um smoke que não mostra a feature na
        // largada não é ready-to-smoke).
        self.flip_strip.tween_correct = crate::flip_tween_correct::build(
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
             NESTA CENA: o tronco e a cabeca mal se movem, entao casam VERDE. Mas a FAISCA\n\
             salta longe demais, e o automatico DESISTE dela -- os dois cantos ganham um\n\
             ANEL MAGENTA (uma faisca a esquerda na partida, uma a direita na chegada, sem\n\
             linha ligando as duas).\n\
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

#[cfg(test)]
#[path = "flip_tween_pairs_smoke_tests.rs"]
mod tests;
