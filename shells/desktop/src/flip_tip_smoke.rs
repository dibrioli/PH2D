//! **A cena pronta para o smoke do *tip* pontilhado** (`PH2D_FLIP_TIP_SMOKE=1`).
//!
//! O *tip* (03 §8) é a PONTA do pincel ao longo do traço: a linha cheia de sempre, ou CONTAS
//! (redondas / quadradas) espaçadas por ARC-LENGTH — o espaçamento não depende da densidade de
//! input (dois traços da mesma forma pontilham igual). Esta cena desenha três traços de
//! referência (Line / Dots / Squares) para o artista VER a diferença, e o pincel abre no modo
//! Draw para ele testar o seletor **Tip** na seção Brush do painel.

use ph2d_core::Vec2;
use ph2d_flip::{FlipStroke, Hold, KeyKind, Point, Rgba, StrokeTip};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, Ordering};

static FRAME: AtomicU32 = AtomicU32::new(0);

fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("PH2D_FLIP_TIP_SMOKE").is_some())
}

const INK: Rgba = Rgba::new(0.92, 0.92, 0.95, 1.0);

/// Um traço horizontal reto na altura `y`, com a `tip` e o espaçamento dados. Largura e
/// espaçamento em MUNDO (a mesma unidade — as contas ficam proporcionais à linha).
fn line(y: f32, tip: StrokeTip, spacing: f32) -> FlipStroke {
    let mut s = FlipStroke::new();
    for i in 0..=12 {
        s.push_point(Point {
            pos: Vec2::new(-4.0 + i as f32 * 0.67, y),
            width: 0.22,
            opacity: 1.0,
            color: INK,
        });
    }
    s.tip = tip;
    s.dot_spacing = spacing;
    s
}

/// **Monta a chave** com os três traços de referência empilhados. Porta única: o gate encena
/// por AQUI (senão a mensagem descreveria um desenho que ninguém mais produz).
pub(crate) fn stage(obj: &mut ph2d_flip::FlipObject) -> ph2d_flip::LayerId {
    let l = obj.add_layer("L");
    if let Some(d) = obj.insert_frame(l, 0, Hold::Implicit, KeyKind::Keyframe) {
        let strokes = &mut obj.drawing_mut(d).expect("desenho").strokes;
        strokes.push(line(2.0, StrokeTip::Continuous, 0.5)); // linha cheia (o de cima)
        strokes.push(line(0.0, StrokeTip::Dots, 0.5)); // contas redondas
        strokes.push(line(-2.0, StrokeTip::Squares, 0.5)); // contas quadradas
    }
    l
}

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

        eprintln!("\n[tip-smoke] cena montada: 3 tracos de referencia (Line / Dots / Squares).");
        eprintln!(
            "\n\
             O QUE ESTA NA TELA\n\
             ==================\n\
             Tres tracos horizontais empilhados, todos da MESMA forma e largura:\n\
               em CIMA   : uma LINHA cheia (o traco de sempre).\n\
               no MEIO   : CONTAS REDONDAS (dots) espacadas ao longo do traco.\n\
               em BAIXO  : CONTAS QUADRADAS (squares).\n\
             As contas sao espacadas por COMPRIMENTO DE ARCO -- o mesmo espacamento em\n\
             qualquer densidade de pontos, e imune ao zoom (medem MUNDO, como o Size).\n\
             \n\
             O QUE FAZER (o seletor REAL)\n\
             ============================\n\
             No painel do Flip, clique o modo **Draw**. Na secao **Brush** aparece um seletor\n\
             **Tip** [Line | Dots | Squares] e (com contas) um slider **Spacing**.\n\
             \n\
               1. Clique **Dots** e DESENHE um traco -- ele sai pontilhado.\n\
               2. Troque para **Squares** e desenhe: as contas viram quadrados.\n\
               3. Arraste **Spacing**: as contas afastam/aproximam (o vao entre elas).\n\
               4. Volte para **Line**: o Spacing SOME (nao ha vao numa linha cheia) e o\n\
                  traco volta a ser a linha de sempre.\n\
             \n\
             O QUE OLHAR\n\
             ===========\n\
             As contas tem de ficar REDONDAS/QUADRADAS de verdade (nao 'linha tracejada'),\n\
             do tamanho da largura do traco, e o Spacing controla o VAO -- nao o tamanho.\n\
             Zoom in/out: as contas mantem o tamanho em DOCUMENTO (nao viram gigantes nem\n\
             somem), porque o Spacing mede MUNDO.\n"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_flip::{FlipDoc, FlipObjectId, LayerId};

    /// O traço `Dots` do meio tem o *tip* pontilhado (o que o gate GPU
    /// `dots_carve_gaps_that_a_continuous_line_does_not` renderiza), e o de cima é `Continuous`.
    /// Prova que a cena arma os DOIS extremos — senão a mensagem mandaria comparar traços iguais.
    #[test]
    fn the_tip_smoke_stages_a_dotted_and_a_continuous_stroke() {
        let mut doc = FlipDoc::default();
        let oid: FlipObjectId = doc.push_object("T");
        let obj = doc.object_mut(oid).expect("objeto");
        let l: LayerId = stage(obj);
        let d = obj
            .layer(l)
            .expect("camada")
            .drawing_at(0)
            .expect("desenho");
        let strokes = &obj.drawing(d).expect("arte").strokes;
        assert_eq!(
            strokes[0].tip,
            StrokeTip::Continuous,
            "o de cima e' a linha cheia"
        );
        assert_eq!(strokes[1].tip, StrokeTip::Dots, "o do meio e' pontilhado");
        assert_eq!(strokes[2].tip, StrokeTip::Squares, "o de baixo e' quadrado");
        assert!(strokes[1].dot_spacing > 0.0, "as contas tem espacamento");
    }
}
