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
use std::sync::atomic::AtomicU32;

pub static FRAME: AtomicU32 = AtomicU32::new(0);

pub fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("PH2D_FLIP_TWEEN_PAIRS_SMOKE").is_some())
}

pub const INK: Rgba = Rgba::new(0.92, 0.92, 0.95, 1.0);
pub const SPARK: Rgba = Rgba::new(1.0, 0.85, 0.3, 1.0);

pub fn line(pts: &[Vec2], color: Rgba, closed: bool) -> FlipStroke {
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

pub fn seg(a: Vec2, b: Vec2, n: usize) -> Vec<Vec2> {
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
pub fn stage(obj: &mut ph2d_flip::FlipObject) -> ph2d_flip::LayerId {
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
    // A faísca: um traço curto, um de cada lado do corpo, DENTRO do quadro. A câmera padrão
    // mostra ~±3 unidades — pôr a faísca em ±5 a deixava FORA da tela, e a demonstração
    // inteira com ela. O órfão vem da diferença de FORMA (o salto de lado + a de comprimento),
    // não da distância pura: `custo ≈ centróide + |Δcomprimento|` passa BEM do teto de recusa
    // (0.38), então o automático a ORFANA — o que o gate
    // `the_scene_orphans_the_spark_until_paired` confirma.
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
        dr.strokes.push(spark(-2.0, 0.5)); // a faísca à ESQUERDA (curta), on-screen
    }
    if let Some(d8) = obj.insert_frame(l, 8, Hold::Implicit, KeyKind::Keyframe) {
        let dr = obj.drawing_mut(d8).expect("desenho");
        dr.strokes.push(torso());
        dr.strokes.push(head());
        dr.strokes.push(spark(1.1, 1.6)); // a faísca à DIREITA (mais longa), on-screen
    }
    l
}

#[cfg(test)]
#[path = "tween_pairs_smoke_tests.rs"]
mod tests;
