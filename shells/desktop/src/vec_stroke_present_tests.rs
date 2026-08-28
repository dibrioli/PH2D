//! Os gates da caixa **Stroke** (plano 34).
//!
//! ⚠️⚠️ **A fixtura tem de conter o FENÓMENO**: uma forma nascida de `..VecPath::default()` (sem
//! traço, que é como um importador ou uma cena de código a produz) **e** uma nascida como a
//! ferramenta a faz (com traço). Um gate só sobre a segunda passa com o buraco inteiro de pé — foi
//! **exactamente** assim que este defeito sobreviveu a uma wave inteira de gates verdes.

use super::*;
use ph2d_vec_scene::{Paint, Rgba8, StrokeSpec, VecPath, VecPathId, VecVertex};

/// Uma forma quadrada. `com_traco` decide se ela nasce vestida — os DOIS casos são a fixtura.
fn cena(com_traco: bool) -> (VecScene, PenTool, VecPathId) {
    let mut scene = VecScene::default();
    let id = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        fill: Some(Paint::solid(Rgba8::new(9, 9, 9, 255))),
        stroke: com_traco.then(|| StrokeSpec::new(Rgba8::new(1, 2, 3, 255), 0.5)),
        ..VecPath::default()
    });
    let mut pen = PenTool::default();
    pen.select_many(&[id]);
    (scene, pen, id)
}

fn traco(scene: &VecScene, id: VecPathId) -> Option<StrokeSpec> {
    scene.path(id).and_then(|p| p.stroke)
}

/// ⭐⭐ **O BURACO INTEIRO** (Enio, 2026-08-27): uma forma que nasceu sem traço não conseguia ganhar
/// um. Nenhum caminho do produto o fazia — o `restyle_selected_strokes` recusa por desenho.
#[test]
fn a_shape_can_gain_a_stroke_it_was_not_born_with() {
    let (mut scene, pen, id) = cena(false);
    let mut h = History::default();
    assert!(traco(&scene, id).is_none(), "a fixtura tem de comecar sem");
    assert!(toggle(&mut scene, &mut h, &pen, 1.0));
    assert!(
        traco(&scene, id).is_some(),
        "a forma continua sem traco - o buraco esta' de pe'"
    );
    assert_eq!(h.undo_len(), 1, "vestir e' UM passo de undo");
}

/// ⚠️⚠️ **O traço novo sai da ficha da FERRAMENTA, e a largura cruza o `px_to_world`.**
///
/// Um default escrito na porta seria uma segunda resposta a *"que traço uma coisa nova recebe?"*, e
/// a forma vestida pela caixa sairia diferente da desenhada pela ferramenta de forma.
#[test]
fn the_new_stroke_comes_from_the_tool_style_not_from_a_default() {
    let (mut scene, mut pen, id) = cena(false);
    let mut h = History::default();
    let mut estilo = pen.style();
    estilo.stroke = Rgba8::new(200, 30, 40, 255);
    estilo.stroke_w_px = 8.0;
    estilo.cap = ph2d_vec_scene::LineCap::Round;
    pen.set_style(estilo);

    assert!(toggle(&mut scene, &mut h, &pen, 0.25));
    let s = traco(&scene, id).expect("vestiu");
    assert_eq!(
        s.color,
        Rgba8::new(200, 30, 40, 255),
        "a COR nao veio da tool"
    );
    assert!(
        (s.width - 2.0).abs() < 1e-12,
        "8 px x 0,25 = 2 unidades de mundo, e deu {}",
        s.width
    );
    assert_eq!(
        s.cap,
        ph2d_vec_scene::LineCap::Round,
        "a ficha inteira tem de vir da tool, nao so' a cor"
    );
}

/// **A porta é de ida E volta** — tirar e voltar a pôr.
///
/// ⚠️ E o CONTROLO: tirar tem de deixar `None` de facto. Uma porta que só soubesse vestir passaria
/// a primeira metade deste gate.
#[test]
fn unchecking_removes_the_stroke_and_rechecking_brings_one_back() {
    let (mut scene, pen, id) = cena(true);
    let mut h = History::default();
    assert!(toggle(&mut scene, &mut h, &pen, 1.0));
    assert!(traco(&scene, id).is_none(), "tirar nao tirou");
    assert!(toggle(&mut scene, &mut h, &pen, 1.0));
    assert!(traco(&scene, id).is_some(), "voltar a por nao pos");
    assert_eq!(h.undo_len(), 2, "dois gestos, dois passos");
}

/// ⚠️ **A resposta é `None` sem uma selecção de UMA forma** — e é esse `None` que impede a caixa de
/// ser pintada. Uma caixa que descreve um objecto que não está lá é pior que caixa nenhuma.
#[test]
fn there_is_no_answer_without_exactly_one_shape_selected() {
    let (scene, pen, _) = cena(true);
    assert_eq!(selected_stroke_present(&scene, &pen), Some(true));

    let (scene2, _, _) = cena(false);
    let vazio = PenTool::default();
    assert_eq!(
        selected_stroke_present(&scene2, &vazio),
        None,
        "sem seleccao nao ha' resposta"
    );

    // Selecção MÚLTIPLA: as duas formas podem discordar, então não há uma caixa a mostrar.
    let mut scene3 = VecScene::default();
    let a = scene3.push_path(VecPath {
        verts: [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    });
    let b = scene3.push_path(VecPath {
        verts: [[2.0, 0.0], [3.0, 0.0], [3.0, 1.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        stroke: Some(StrokeSpec::new(Rgba8::new(1, 2, 3, 255), 0.5)),
        ..VecPath::default()
    });
    let mut muitos = PenTool::default();
    muitos.select_many(&[a, b]);
    assert_eq!(selected_stroke_present(&scene3, &muitos), None);
    // E a porta também não age — senão o clique escreveria numa das duas, à sorte.
    let mut h = History::default();
    assert!(!toggle(&mut scene3, &mut h, &muitos, 1.0));
    assert_eq!(h.undo_len(), 0);
}

/// ⭐⭐ **A forma que a CAIXA veste é indistinguível da que a FERRAMENTA desenha** — o gate da porta
/// única, e a razão de este módulo não ter um default próprio.
///
/// ⚠️ Ele compara com o resultado da MESMA função que a `ph2d-vec-edit` chama ao criar uma forma
/// (`PenStyle::stroke_spec`, com a largura em px convertida pelo mesmo factor). Um campo esquecido
/// — a junta, o tracejado, a ponta — daria uma forma vestida pela caixa que se comporta diferente
/// da desenhada, e o artista descobriria isso ao esticar a linha.
///
/// ⛔ **Nenhum piso de largura aqui, de propósito.** Um `px_to_world` degenerado daria um traço de
/// largura zero — mas daria o MESMO à ferramenta de forma, e a lei que este módulo promete é
/// *"igual ao que a ferramenta faz"*, não *"sempre visível"*. Um piso inventado seria um cap sem
/// medição a divergir das duas portas.
#[test]
fn the_box_dresses_a_shape_exactly_like_the_tool_draws_one() {
    let (mut scene, mut pen, id) = cena(false);
    let mut h = History::default();
    let mut estilo = pen.style();
    estilo.stroke_w_px = 5.0;
    estilo.join = ph2d_vec_scene::LineJoin::Bevel;
    estilo.dash = Some((3.0, 2.0));
    estilo.marker_end = ph2d_vec_scene::Marker::Triangle;
    pen.set_style(estilo);

    let px_to_world = 0.4;
    assert!(toggle(&mut scene, &mut h, &pen, px_to_world));
    assert_eq!(
        traco(&scene, id).expect("vestiu"),
        estilo.stroke_spec(estilo.stroke_w_px * px_to_world),
        "a caixa e a ferramenta deixaram de dar o MESMO traco"
    );
}
