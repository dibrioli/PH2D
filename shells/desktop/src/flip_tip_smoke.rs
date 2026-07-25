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

/// Um traço horizontal reto na altura `y`, com a `tip`, a `width` (mundo) e o `spacing`
/// dados. O `spacing` é um MÚLTIPLO do diâmetro (relativo à espessura), então a MESMA razão
/// pontilha igual num traço fino e num grosso — é o que este smoke prova.
fn line(y: f32, tip: StrokeTip, spacing: f32, width: f32) -> FlipStroke {
    let mut s = FlipStroke::new();
    for i in 0..=12 {
        s.push_point(Point {
            pos: Vec2::new(-4.0 + i as f32 * 0.67, y),
            width,
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
        // Espaçamento 2.0 = centros a 2 diâmetros (vão de 1 diâmetro), a MESMA razão nos
        // três finos E no grosso. Os índices 0/1/2 são os extremos que o gate afirma.
        strokes.push(line(2.5, StrokeTip::Continuous, 2.0, 0.22)); // linha cheia (fina)
        strokes.push(line(1.0, StrokeTip::Dots, 2.0, 0.22)); // contas redondas (fina)
        strokes.push(line(-0.5, StrokeTip::Squares, 2.0, 0.22)); // contas quadradas (fina)
        // O traço GROSSO (o report do Enio): com o espaçamento absoluto antigo o padrão
        // fundia num borrão; relativo à espessura, as contas aparecem na MESMA razão.
        strokes.push(line(-2.5, StrokeTip::Dots, 2.0, 0.6)); // contas redondas (GROSSA)
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

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_flip::{FlipDoc, FlipObjectId, LayerId};

    /// A cena arma os extremos do *tip* (linha cheia + as duas contas) E — o ponto DESTE fix —
    /// um traço GROSSO pontilhado: o report do Enio (2026-07-25) é que traços grossos não
    /// mostravam o padrão, então a cena que o demonstra TEM de conter um. Sem o 4º traço, a
    /// mensagem mandaria olhar um efeito que a cena não encena.
    #[test]
    fn the_tip_smoke_stages_a_dotted_a_continuous_and_a_thick_stroke() {
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
        assert_eq!(strokes[1].tip, StrokeTip::Dots, "o 2o e' pontilhado");
        assert_eq!(strokes[2].tip, StrokeTip::Squares, "o 3o e' quadrado");
        assert!(strokes[1].dot_spacing > 0.0, "as contas tem espacamento");
        // O 4o e' pontilhado E mais GROSSO que os finos (o caso do report), com a MESMA
        // razao de espacamento — a prova de que a pitch escala com a espessura.
        assert_eq!(
            strokes[3].tip,
            StrokeTip::Dots,
            "o de baixo e' o grosso pontilhado"
        );
        assert!(
            strokes[3].widths()[0] > strokes[1].widths()[0] * 2.0,
            "o 4o traco e' bem mais grosso que os finos"
        );
        assert_eq!(
            strokes[3].dot_spacing, strokes[1].dot_spacing,
            "grosso e fino usam a MESMA razao de espacamento"
        );
    }
}
