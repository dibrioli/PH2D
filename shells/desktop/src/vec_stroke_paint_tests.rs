//! Os gates da fileira **Type** da secção *Stroke* (plano 35, wave D).
//!
//! ⚠️⚠️ **A fixtura tem de conter os TRÊS estados**: uma forma **sem traço** (que não tem tinta de
//! traço nenhuma e por isso não pinta a fileira), uma com traço **sólido**, e uma com traço já
//! **padrão**. Um gate só sobre as duas primeiras passa com metade da wave por construir — é a lei
//! que o §4 do plano escreveu, e a mesma que o buraco do plano 34 explorou.

use super::*;
use ph2d_vec_scene::{Paint, PatternFill, Rgba8, StrokeSpec, VecPath, VecPathId, VecVertex};

/// A fonte de arte de um padrão de teste. ⚠️ Um `AssetId` que **não resolve** é o caso honesto aqui:
/// estes gates falam do MODELO, e resolver pixels exigiria uma `AssetDb` que nada nesta lei lê.
fn arte() -> PatternSource {
    PatternSource::Image(ph2d_asset::AssetId::from_bytes(&[7u8; 32]))
}

/// `None` = sem traço · `Some(false)` = traço sólido · `Some(true)` = traço com padrão.
fn cena(traco: Option<bool>) -> (VecScene, ph2d_vec_edit::PenTool, VecPathId) {
    let mut scene = VecScene::default();
    let stroke = traco.map(|padrao| {
        let mut s = StrokeSpec::new(Rgba8::new(11, 22, 33, 255), 0.5);
        if padrao {
            s.paint = StrokePaint::Pattern(Box::new(PatternFill::new(
                arte(),
                [2.0, 2.0],
                Rgba8::new(11, 22, 33, 255),
            )));
        }
        s
    });
    let id = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        fill: Some(Paint::solid(Rgba8::new(9, 9, 9, 255))),
        stroke,
        ..VecPath::default()
    });
    let mut pen = ph2d_vec_edit::PenTool::default();
    pen.select_many(&[id]);
    (scene, pen, id)
}

fn tinta(scene: &VecScene, id: VecPathId) -> Option<StrokePaint> {
    scene
        .path(id)
        .and_then(|p| p.stroke.as_ref())
        .map(|s| s.paint.clone())
}

/// ⭐⭐ **O BURACO INTEIRO da wave D: um traço sólido consegue passar a padrão.**
#[test]
fn a_solid_stroke_can_become_a_patterned_one() {
    let (mut scene, pen, id) = cena(Some(false));
    let mut h = History::default();
    assert!(
        matches!(tinta(&scene, id), Some(StrokePaint::Solid(_))),
        "a fixtura tem de comecar solida"
    );
    assert!(set_kind(
        &mut scene,
        &mut h,
        &pen,
        StrokePaintKind::Pattern,
        Some((arte(), [3.0, 4.0], [1.0, 2.0])),
    ));
    let Some(StrokePaint::Pattern(p)) = tinta(&scene, id) else {
        panic!("o traco continua solido - o buraco esta' de pe'");
    };
    assert_eq!(p.size, [3.0, 4.0], "o tamanho pedido nao entrou");
    assert_eq!(
        p.origin,
        [1.0, 2.0],
        "⛔ o canto e' o da FORMA, nunca a origem do mundo - e' o report do `Clamp` em branco"
    );
    assert_eq!(h.undo_len(), 1, "trocar a tinta e' UM passo de undo");
}

/// ⭐ **Voltar a `Solid` devolve a COR DE RECURSO do padrão, e não uma cor arbitrária.**
///
/// ⚠️ É ela que a linha já pintava enquanto o ladrilho não resolvia ⇒ ir e voltar **não pisca**.
#[test]
fn going_back_to_solid_keeps_the_colour_the_line_was_already_showing() {
    let (mut scene, pen, id) = cena(Some(true));
    let mut h = History::default();
    assert!(set_kind(
        &mut scene,
        &mut h,
        &pen,
        StrokePaintKind::Solid,
        None
    ));
    assert_eq!(
        tinta(&scene, id),
        Some(StrokePaint::Solid(Rgba8::new(11, 22, 33, 255))),
        "a cor nao e' a `fallback` do padrao - a troca pisca para uma cor arbitraria"
    );
}

/// ⚠️⚠️ **Desistir do diálogo da arte NÃO muda nada** — o artista fechou a janela, e apagar-lhe a
/// cor do traço por isso seria o pior dos dois mundos. É a lei do `apply_vec_set_fill_kind`.
#[test]
fn giving_up_on_the_art_dialog_leaves_the_stroke_alone() {
    let (mut scene, pen, id) = cena(Some(false));
    let mut h = History::default();
    let antes = tinta(&scene, id);
    assert!(
        !set_kind(&mut scene, &mut h, &pen, StrokePaintKind::Pattern, None),
        "sem arte, `set_kind` tem de recusar"
    );
    assert_eq!(
        tinta(&scene, id),
        antes,
        "a tinta mudou apesar da desistencia"
    );
    assert_eq!(h.undo_len(), 0, "uma desistencia nao e' um passo de undo");
}

/// ⭐ **Pedir a tinta que já lá está é um no-op** — e o que isso compra é a LEI inteira do padrão a
/// sobreviver a trocar de chip e voltar (a arte, o reticulado, a colocação).
///
/// ⚠️ **O controle é a metade que importa**: com uma arte NOVA na mão, um `set_kind(Pattern)` numa
/// forma que já tem padrão tem de a **ignorar**. Sem esta metade, o gate passaria com uma
/// implementação que reconstrói o padrão do zero a cada clique.
#[test]
fn asking_for_the_kind_it_already_has_preserves_the_whole_law() {
    let (mut scene, pen, id) = cena(Some(true));
    let mut h = History::default();
    let antes = tinta(&scene, id);
    assert!(!set_kind(
        &mut scene,
        &mut h,
        &pen,
        StrokePaintKind::Pattern,
        Some((
            PatternSource::Image(ph2d_asset::AssetId::from_bytes(&[9u8; 32])),
            [99.0, 99.0],
            [50.0, 50.0],
        )),
    ));
    assert_eq!(
        tinta(&scene, id),
        antes,
        "a lei do padrao foi reconstruida - trocar de chip e voltar perde a arte"
    );
    assert_eq!(h.undo_len(), 0);
}

/// ⚠️⚠️ **Sem traço não há tinta de traço** — a fileira NÃO é pintada, e é isso que a distingue da
/// caixa *Stroke*, que tem uma resposta (`Some(false)`) para a mesma forma.
#[test]
fn a_shape_without_a_stroke_has_no_paint_kind_to_show() {
    let (scene, pen, _) = cena(None);
    assert_eq!(
        selected_stroke_paint_kind(&scene, &pen),
        None,
        "uma forma SEM traco oferece um tipo de tinta - a fileira pinta sobre o nada"
    );
    // CONTROLO: as duas outras metades da fixtura respondem, senão este gate ficaria verde num
    // produto que nunca responde nada.
    let (s1, p1, _) = cena(Some(false));
    assert_eq!(
        selected_stroke_paint_kind(&s1, &p1),
        Some(StrokePaintKind::Solid)
    );
    let (s2, p2, _) = cena(Some(true));
    assert_eq!(
        selected_stroke_paint_kind(&s2, &p2),
        Some(StrokePaintKind::Pattern)
    );
}

/// ⚠️ **Selecção múltipla não tem resposta** — a mesma lei do `resize_box` e da caixa irmã.
#[test]
fn a_multiple_selection_has_no_answer() {
    let (mut scene, mut pen, id) = cena(Some(false));
    let outro = scene.push_path(VecPath {
        verts: [[20.0, 0.0], [30.0, 0.0], [30.0, 10.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        stroke: Some(StrokeSpec::new(Rgba8::new(1, 1, 1, 255), 0.5)),
        ..VecPath::default()
    });
    pen.select_many(&[id, outro]);
    assert_eq!(selected_stroke_paint_kind(&scene, &pen), None);
    // E nada se escreve com duas formas em mãos.
    let mut h = History::default();
    assert!(!set_kind(
        &mut scene,
        &mut h,
        &pen,
        StrokePaintKind::Pattern,
        Some((arte(), [1.0, 1.0], [0.0, 0.0])),
    ));
}

/// **Os dois chips são alcançáveis pelo `NodeId`, e mais nenhum id os reclama.**
#[test]
fn the_two_chips_map_to_the_two_kinds_and_nothing_else_does() {
    assert_eq!(
        kind_for_id(ph2d_editor::ids::VECTOR_STROKE_KIND_SOLID),
        Some(StrokePaintKind::Solid)
    );
    assert_eq!(
        kind_for_id(ph2d_editor::ids::VECTOR_STROKE_KIND_PATTERN),
        Some(StrokePaintKind::Pattern)
    );
    // ⛔ O chip do PREENCHIMENTO não pode cair aqui: seria a fileira do traço a consumir o clique
    // do vizinho, e o preenchimento deixaria de mudar sem uma mensagem sequer.
    assert_eq!(
        kind_for_id(ph2d_editor::ids::VECTOR_FILL_KIND_PATTERN),
        None
    );
    assert_eq!(kind_for_id(ph2d_editor::ids::VECTOR_STROKE_PRESENT), None);
}

/// ⚠️ **A OPACIDADE atravessa a troca de tinta, nos DOIS sentidos.**
///
/// Um traço a 50% que vira padrão nasceria opaco (o `alpha` do construtor é `1,0`) e **saltaria** no
/// clique; e a primeira mexida no painel puxá-lo-ia de volta, porque é no `StrokeStyle::onto` que a
/// opacidade do traço mora. *Uma opacidade, uma casa — inclusive no nascimento.*
#[test]
fn the_opacity_survives_the_paint_switch_in_both_directions() {
    let (mut scene, pen, id) = cena(Some(false));
    // Meia opacidade no traço sólido de partida.
    if let Some(s) = scene.path_mut(id).and_then(|p| p.stroke.as_mut()) {
        s.paint = StrokePaint::Solid(Rgba8::new(11, 22, 33, 128));
    }
    let mut h = History::default();
    assert!(set_kind(
        &mut scene,
        &mut h,
        &pen,
        StrokePaintKind::Pattern,
        Some((arte(), [1.0, 1.0], [0.0, 0.0])),
    ));
    let Some(StrokePaint::Pattern(p)) = tinta(&scene, id) else {
        panic!("o traco tem de ser padrao");
    };
    assert!(
        (p.alpha - 128.0 / 255.0).abs() < 1e-6,
        "o padrao nasceu OPACO sobre um traco a meia opacidade - ele salta no clique (alpha={})",
        p.alpha
    );
    // E a volta devolve a mesma alfa, pela `fallback`.
    assert!(set_kind(
        &mut scene,
        &mut h,
        &pen,
        StrokePaintKind::Solid,
        None
    ));
    assert_eq!(
        tinta(&scene, id),
        Some(StrokePaint::Solid(Rgba8::new(11, 22, 33, 128))),
        "a opacidade nao voltou do padrao para o solido"
    );
}

// ── ⭐⭐⭐ A W4 do plano 36: a criação e os knobs ──────────────────────────────────

/// Uma cena com a ARTE e uma forma cujo traço é um pincel (com ou sem arte apontada).
fn cena_pincel(com_arte: bool) -> (VecScene, ph2d_vec_edit::PenTool, VecPathId, VecPathId) {
    let mut scene = VecScene::default();
    let arte = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    });
    let mut s = StrokeSpec::new(Rgba8::new(11, 22, 33, 255), 0.5);
    s.paint = StrokePaint::Brush(Box::new(ph2d_vec_scene::BrushStroke {
        art: com_arte.then_some(arte),
        ..ph2d_vec_scene::BrushStroke::default()
    }));
    let id = scene.push_path(VecPath {
        verts: [[5.0, 0.0], [9.0, 0.0], [9.0, 4.0], [5.0, 4.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        stroke: Some(s),
        ..VecPath::default()
    });
    let mut pen = ph2d_vec_edit::PenTool::default();
    pen.select_many(&[id]);
    (scene, pen, id, arte)
}

fn pincel_de(scene: &VecScene, id: VecPathId) -> ph2d_vec_scene::BrushStroke {
    scene
        .path(id)
        .and_then(|p| p.stroke.as_ref())
        .and_then(ph2d_vec_scene::StrokeSpec::brush)
        .expect("o traco e' um pincel")
        .clone()
}

/// ⭐⭐ **O GESTO DE DUAS MÃOS põe a arte**, e resolve por ID — não pela selecção.
///
/// ⚠️ O alvo é capturado no *arm*; o clique seguinte cai noutra forma, que passa a ser a
/// selecionada. Ler a selecção aqui apontaria o pincel para a forma errada — o *"escolhendo a si
/// mesmo"* que o picker do padrão já documenta.
#[test]
fn the_two_handed_gesture_sets_the_brush_art_by_id() {
    let (mut scene, _pen, id, arte) = cena_pincel(false);
    let mut h = History::default();
    assert!(set_art(&mut scene, &mut h, id, arte));
    assert_eq!(pincel_de(&scene, id).art, Some(arte));
    assert_eq!(h.undo_len(), 1, "por a arte e' UM passo de undo");
    // ⚠️ O MESMO valor não grava passo — re-armar e escolher a mesma forma encheria a pilha.
    assert!(!set_art(&mut scene, &mut h, id, arte));
    assert_eq!(h.undo_len(), 1);
}

/// ⛔⛔ **UMA FORMA NÃO PODE SER O PRÓPRIO PINCEL** — a recusa é a primeira linha da porta.
///
/// ⚠️ Há uma **segunda** metade, PURA, no `brush_live` — duas porque as duas portas existem: esta
/// autora, aquela resolve, e o documento pode chegar ao estado mau por outro caminho (um save).
#[test]
fn a_shape_can_never_author_itself_as_its_own_brush() {
    let (mut scene, _pen, id, _) = cena_pincel(false);
    let mut h = History::default();
    assert!(!set_art(&mut scene, &mut h, id, id));
    assert_eq!(h.undo_len(), 0);
    assert_ne!(pincel_de(&scene, id).art, Some(id));
}

/// ⭐ **CADA KNOB DA SECÇÃO ESCREVE O SEU CAMPO — e só o seu.**
///
/// ⚠️ **A metade que importa é a segunda:** os outros quatro campos ficam INTACTOS. Sem ela, o gate
/// ficaria verde sobre uma porta que reconstrói o pincel do zero a cada mexida — e o artista veria
/// os outros knobs saltarem para o default.
#[test]
fn every_brush_knob_writes_its_own_field_and_only_its_own() {
    for (nome, cmd, ler) in [
        (
            "scale",
            BrushCmd::Scale(3.0),
            (|b: &ph2d_vec_scene::BrushStroke| b.scale) as fn(&_) -> f64,
        ),
        ("spacing", BrushCmd::Spacing(2.5), |b| b.spacing),
        ("offset", BrushCmd::Offset(-1.25), |b| b.offset),
        ("rotation", BrushCmd::Rotation(90.0), |b| b.rotation_deg),
    ] {
        let (mut scene, pen, id, _) = cena_pincel(true);
        // Um estado de partida DISTINTO do default em todos os campos, senão «intacto» não mede.
        let arte_id = pincel_de(&scene, id).art;
        if let Some(s) = scene.path_mut(id).and_then(|p| p.stroke.as_mut()) {
            s.paint = StrokePaint::Brush(Box::new(ph2d_vec_scene::BrushStroke {
                art: arte_id,
                spacing: 1.5,
                scale: 1.75,
                offset: 0.5,
                rotation_deg: 45.0,
                flip: true,
                ..ph2d_vec_scene::BrushStroke::default()
            }));
        }
        let antes = pincel_de(&scene, id);
        let mut h = History::default();
        assert!(apply(&mut scene, &mut h, &pen, cmd), "{nome} nao escreveu");
        let depois = pincel_de(&scene, id);
        assert!(
            (ler(&depois) - ler(&antes)).abs() > 1e-9,
            "{nome} nao mudou o proprio campo"
        );
        // Os outros ficam onde estavam.
        let mut esperado = antes.clone();
        match cmd {
            BrushCmd::Scale(v) => esperado.scale = v,
            BrushCmd::Spacing(v) => esperado.spacing = v,
            BrushCmd::Offset(v) => esperado.offset = v,
            BrushCmd::Rotation(v) => esperado.rotation_deg = v,
            BrushCmd::Flip => esperado.flip = !esperado.flip,
        }
        assert_eq!(depois, esperado, "{nome} mexeu num campo que nao e' o dele");
        assert_eq!(h.undo_len(), 1, "{nome}: UM passo de undo");
        // ⚠️ E o MESMO valor não grava passo — o defeito que fazia todo quadro virar undo.
        assert!(!apply(&mut scene, &mut h, &pen, cmd));
        assert_eq!(h.undo_len(), 1);
    }
}

/// **Os controlos da secção são alcançáveis pelo `NodeId`, e mais nenhum id os reclama.**
#[test]
fn the_brush_controls_map_to_their_commands_and_nothing_else_does() {
    use ph2d_editor::ids as i;
    assert_eq!(cmd_for_id(i::VECTOR_BRUSH_FLIP), Some(BrushCmd::Flip));
    assert_eq!(cmd_for_id(i::VECTOR_STROKE_KIND_BRUSH), None);
    assert_eq!(
        slider_cmd_for_id(i::VECTOR_BRUSH_SCALE, 2.0),
        Some(BrushCmd::Scale(2.0))
    );
    assert_eq!(
        slider_cmd_for_id(i::VECTOR_BRUSH_OFFSET, -1.0),
        Some(BrushCmd::Offset(-1.0))
    );
    // ⛔ Um slider do PADRÃO não pode cair aqui: seria a secção do pincel a consumir o clique do
    // vizinho, e o padrão deixaria de responder sem uma mensagem sequer.
    assert_eq!(slider_cmd_for_id(i::VECTOR_DASH, 1.0), None);
}
