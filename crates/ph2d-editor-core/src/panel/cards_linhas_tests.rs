//! ⭐ **A lei da linha de tinta na coluna estreita** (`CardCtx::paint_line`): de um cartão muito
//! estreito a um largo, nenhuma caixa fica abaixo do que pede e o rótulo nunca quebra.

use super::*;
use crate::text_elide::elisao;

const COR: NodeId = NodeId(9001);
const CODIGO: NodeId = NodeId(9002);
const SOLTA: NodeId = NodeId(9003);
const OP: NodeId = NodeId(9004);
const OLHO: NodeId = NodeId(9005);
const TIRA: NodeId = NodeId(9006);
const ROTULO_COR: &str = "Color";
const ROTULO_OP: &str = "Opacity (%)";
/// Abaixo do corpo do cartão na coluna mínima (`220 px`) e acima da mais larga que o painel dá.
const LARGURAS: std::ops::RangeInclusive<u16> = 150..=460;

struct Linha {
    cor: f32,
    cor_y: f32,
    op: Rect,
    olho: Rect,
    pede_cor: f32,
    pede_op: f32,
    cabe: bool,
}

/// Pinta a linha a `w` e lê a geometria no índice de acerto, como um clique a veria.
fn linha(w: f32, texto: &str, ligado: bool) -> (Linha, Vec<elisao::Medido>) {
    let store = WidgetStore::default();
    let mut scene = VectorScene::new();
    let mut ts = TextSystem::without_system_fonts();
    let mut hit = HitIndex::default();
    let ink = CardInk::of_list(Theme::default());
    let seg = |id, icon| Seg {
        id,
        icon: Some(icon),
        text: "",
        on: false,
    };
    let botoes = [seg(OLHO, IconId::Eye), seg(TIRA, IconId::Minus)];
    let cor = Cell::new(Widget::Swatch {
        id: COR,
        rgba: Some([1, 2, 3, 255]),
        text: texto,
        edit: (!ligado).then_some(CODIGO),
        detach: ligado.then_some(SOLTA),
    })
    .label(ROTULO_COR);
    let op = Cell::new(Widget::Field {
        id: OP,
        glyph: Glyph::Icon(IconId::CardOpacity),
    })
    .label(ROTULO_OP);
    let nus = Cell::new(Widget::Bare(&botoes));
    let (medida, medidos) = elisao::medindo(|| {
        let mut c = CardCtx::flutuante(&mut scene, &mut ts, &store, &mut hit, ink, 0.0, w);
        let pede_cor = c.pede(&cor);
        let pede_op = c.pede(&op);
        let cabe = c.tudo_cabe(&[cor, op, nus]);
        let _ = c.paint_line(cor, op, nus, 0.0);
        (pede_cor, pede_op, cabe)
    });
    let rect = |id| hit.rect_for(id).expect("pintado");
    let amostra = rect(COR);
    let fim = rect(if ligado { SOLTA } else { CODIGO });
    (
        Linha {
            cor: fim.x + fim.w - amostra.x,
            cor_y: amostra.y,
            op: rect(OP),
            olho: rect(OLHO),
            pede_cor: medida.0,
            pede_op: medida.1,
            cabe: medida.2,
        },
        medidos,
    )
}

fn varre(texto: &str, ligado: bool) {
    let mut partida = 0;
    let mut inteira = 0;
    let mut antes: Option<bool> = None;
    for w in LARGURAS {
        let w = f32::from(w);
        let (l, medidos) = linha(w, texto, ligado);
        assert!(
            l.cor >= l.pede_cor.min(w) - 0.5,
            "«{texto}» a {w}: a cor mede {} e pede {}",
            l.cor,
            l.pede_cor
        );
        assert!(
            l.op.w >= l.pede_op - 0.5,
            "«{texto}» a {w}: a opacidade mede {} e pede {}",
            l.op.w,
            l.pede_op
        );
        assert!(
            medidos.iter().any(|m| m.texto == ROTULO_OP && m.coube()),
            "«{texto}» a {w}: o rótulo não saiu inteiro numa linha: {:?}",
            medidos.iter().map(|m| &m.pintado).collect::<Vec<_>>()
        );
        assert!((l.op.y - l.olho.y).abs() < 0.5, "o olho saiu da opacidade");
        if l.cabe {
            inteira += 1;
            assert!(
                (l.cor_y - l.op.y).abs() < 0.5,
                "a {w} tudo cabe, numa linha"
            );
        } else {
            partida += 1;
            assert!(l.cor_y < l.op.y, "a {w} não cabe, a cor vai por cima");
        }
        assert!(
            antes != Some(true) || l.cabe,
            "«{texto}»: a {w} voltou a partir depois de caber"
        );
        antes = Some(l.cabe);
    }
    assert!(
        partida > 0 && inteira > 0,
        "«{texto}»: a varredura não viu os dois regimes ({partida} partidas, {inteira} inteiras)"
    );
}

/// *Mutação: `paint_line` sempre numa linha ⇒ abaixo do limiar a cor fica na linha da opacidade.*
#[test]
fn a_paint_row_breaks_only_when_it_does_not_fit() {
    varre("4C8DF0", false);
}

/// O nome de um estilo ligado pede mais do que um código: o limiar sobe com ele, e a lei é a mesma.
#[test]
fn a_linked_style_name_keeps_the_same_law() {
    varre("Brand orange", true);
    varre("Very long style name", true);
    let limiar = |texto, ligado| {
        LARGURAS
            .map(f32::from)
            .find(|w| linha(*w, texto, ligado).0.cabe)
            .expect("cabe em alguma largura")
    };
    assert!(limiar("Very long style name", true) > limiar("4C8DF0", false));
}
