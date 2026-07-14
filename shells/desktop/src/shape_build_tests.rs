//! Gates do Shape Builder — o gesto inteiro, sem gfx.
//!
//! ## A fixture é PARTE do gate, e era ela que estava errada
//!
//! A 1ª versão destes testes usou quadrados eixo-alinhados construídos à mão, na identidade.
//! Dezesseis gates verdes, e o Enio reprovou o smoke no primeiro clique. **Um gate só prova
//! o que a fixture dele contém**: mutar o código e ver vermelho dentro de um universo de
//! quadrados só prova coisas sobre quadrados.
//!
//! Aqui a fixture é a do produto: formas do **catálogo** (curvas), **centradas no local 0**,
//! com a pose num `Transform` (ADR-0111) — que é o que a Shape tool deixa na cena. A sessão
//! é aberta pelo `BuildSession::open` de verdade, e o resultado é aplicado pelo `commit` de
//! verdade: o que se mede é **a cena que o artista fica**, não o retorno de uma função.

use super::*;
use ph2d_vec_scene::{Paint, Rgba8, ShapeKind, VecVertex, Xform, contains_point, cook};

/// Uma forma do catálogo como a Shape tool a deixa: geometria LOCAL centrada no 0, pose num
/// `Xform`. `rgb` é o estilo DELA — é por ele que se prova que a sobra continua sendo ela.
fn live(kind: ShapeKind, half: [f64; 2], params: &[f64], rgb: u8) -> VecPath {
    let mut p = cook(kind, [-half[0], -half[1]], [half[0], half[1]], params);
    p.fill = Some(Paint::solid(Rgba8::new(rgb, rgb, rgb, 255)));
    p
}

fn at(x: f64, y: f64) -> Xform {
    Xform([1.0, 0.0, 0.0, 1.0, x, y])
}

/// A cena do smoke do Enio: **pentágono + estrela + retângulo arredondado**, sobrepostos,
/// cada um com a sua pose. Devolve (cena, xforms, ids em z fundo→topo).
///
/// Geometria (mundo): o pentágono ocupa x ∈ [-3,-0.5], a estrela x ∈ [0.5,3], e o retângulo
/// arredondado cobre os dois. Então: as três se sobrepõem duas a duas com o retângulo, e o
/// pentágono e a estrela **não se tocam** — o que dá faces com pertinência de 1 e de 2 bits,
/// e uma forma (a de baixo) que um gesto pode não tocar.
fn scene() -> (VecScene, VecXforms, Vec<VecPathId>) {
    let mut scene = VecScene::new();
    let mut xf = VecXforms::new();
    let rect = scene.push_path(live(
        ShapeKind::RoundRect,
        [3.5, 1.6],
        &[0.5, 0.0, 0.0, 0.0, 0.0],
        11,
    ));
    let pent = scene.push_path(live(ShapeKind::Polygon, [1.25, 1.25], &[5.0, 0.0], 22));
    let star = scene.push_path(live(ShapeKind::Star, [1.25, 1.25], &[5.0, 0.45, 0.0], 33));
    xf.insert(rect, at(0.0, 0.0));
    xf.insert(pent, at(-1.75, 0.0));
    xf.insert(star, at(1.75, 0.0));
    (scene, xf, vec![rect, pent, star])
}

fn session(scene: &VecScene, xf: &VecXforms, ids: &[VecPathId]) -> BuildSession {
    BuildSession::open(scene, xf, ids).expect("as três formas abrem uma sessão")
}

/// Solta o botão: o `resolve` + o `commit` do PRODUTO (o `build_up` chama exatamente estes
/// dois; o que ele acrescenta é só a seleção e o borrow do `gfx`).
fn commit(sc: &mut VecScene, s: &mut BuildSession) -> Vec<VecPathId> {
    let result = s.resolve();
    let sources = s.sources.clone();
    crate::shape_build::commit(sc, &sources, result)
}

/// Dentro do pentágono (e do retângulo), longe da estrela.
const IN_PENT: [f64; 2] = [-1.75, 0.0];
/// Dentro da estrela (e do retângulo), longe do pentágono.
const IN_STAR: [f64; 2] = [1.75, 0.0];
/// Só dentro do retângulo — o corredor entre as duas.
const ONLY_RECT: [f64; 2] = [0.0, 1.0];

/// O fill de um path, como número (a identidade do estilo na fixture).
fn tone(p: &VecPath) -> Option<u8> {
    match p.fill.as_ref()? {
        Paint::Solid(c) => Some(c.r),
        _ => None,
    }
}

/// **O GATE DO SMOKE REPROVADO.** Um clique numa face NÃO pode dissolver a arte.
///
/// A 1ª versão devolvia a sobra como `união(todas as fontes) − o levado`: um clique na
/// estrela fundia o pentágono e o retângulo num blob único, com um estilo só. O artista via
/// as formas que tinha acabado de desenhar SUMIREM (Enio, 2026-07-13) — e não havia gate
/// nenhum entre isso e a tela, porque nenhum deles olhava para a CENA depois do gesto.
#[test]
fn clicking_one_region_never_dissolves_the_shapes_it_did_not_touch() {
    let (mut sc, xf, ids) = scene();
    let mut s = session(&sc, &xf, &ids);
    let pent_before = sc.paths().iter().find(|p| p.id == ids[1]).cloned().unwrap();

    // Um clique na estrela: toca a estrela e o retângulo. O pentágono NÃO.
    s.dragging = true;
    s.touch(IN_STAR);
    let sel = commit(&mut sc, &mut s);

    // 1. O pentágono é o MESMO path — mesmo id, mesma geometria, mesmo estilo. Não foi
    //    assado, não foi fundido, não foi recolorido.
    let pent_after = sc
        .paths()
        .iter()
        .find(|p| p.id == ids[1])
        .expect("o pentágono intocado continua existindo, com o id dele");
    assert_eq!(pent_after.verts, pent_before.verts, "geometria intacta");
    assert_eq!(tone(pent_after), Some(22), "e o estilo é o dele");
    assert!(
        sel.contains(&ids[1]),
        "e continua selecionado (dá p/ seguir)"
    );

    // 2. Nada na cena tem a área do blob (o retângulo INTEIRO mais a estrela): o que sobrou
    //    do retângulo NÃO contém o miolo da estrela — foi de lá que a face saiu.
    let star_face = sc
        .paths()
        .iter()
        .filter(|p| p.id != ids[1])
        .filter(|p| contains_point(p, IN_STAR))
        .count();
    assert_eq!(star_face, 1, "o ponto da estrela pertence a UMA forma só");
}

/// A sobra de cada forma continua sendo **aquela forma**: o estilo é o dela.
///
/// **Onde a 1ª versão deste gate mentia:** o `apply_many` tira o estilo do ÚLTIMO argumento,
/// e na subtração `fonte − faces` o último argumento é uma FACE — que por sua vez herdou o
/// estilo de quem a produziu (o pentágono, aqui, que a face nem contém). Sem a restauração,
/// o que sobra do retângulo sai pintado de pentágono. O gate só morde se medir o path da
/// SOBRA — medir "algum path que contém o ponto" pegava a forma NOVA e ficava verde.
#[test]
fn what_is_left_of_a_shape_keeps_that_shapes_style() {
    let (mut sc, xf, ids) = scene();
    let mut s = session(&sc, &xf, &ids);
    s.dragging = true;
    s.touch(IN_STAR); // toca a estrela e o retângulo; o pentágono não
    commit(&mut sc, &mut s);

    // A ponta ESQUERDA do retângulo: fora do pentágono, fora da estrela, fora da face — só
    // a SOBRA do retângulo mora ali.
    let left_end = [-3.2, 0.0];
    let rest: Vec<&VecPath> = sc
        .paths()
        .iter()
        .filter(|p| contains_point(p, left_end))
        .collect();
    assert_eq!(rest.len(), 1, "a ponta do retângulo é de UM path só");
    assert_eq!(
        tone(rest[0]),
        Some(11),
        "e a sobra do retângulo tem a cor DELE (não a do pentágono, que a face carregava)"
    );
    // A forma NOVA (a face) herda o TOPO entre as tocadas: a estrela.
    let face: Vec<&VecPath> = sc
        .paths()
        .iter()
        .filter(|p| contains_point(p, IN_STAR))
        .collect();
    assert_eq!(face.len(), 1);
    assert_eq!(tone(face[0]), Some(33), "estilo do topo (Illustrator)");
    // E o pentágono nem foi tocado.
    assert_eq!(
        tone(sc.paths().iter().find(|p| p.id == ids[1]).unwrap()),
        Some(22)
    );
}

/// Passar o cursor SEM apertar realça, mas não pinta. (Sem isto, o realce de hover viraria
/// uma marcação — e o artista destruiria a arte só de mover o mouse por cima dela.)
#[test]
fn hovering_highlights_but_does_not_mark() {
    let (sc, xf, ids) = scene();
    let mut s = session(&sc, &xf, &ids);
    s.touch(IN_PENT);
    assert!(s.hover.is_some(), "a face sob o cursor é realçada");
    assert!(s.marked.is_empty(), "…e NÃO é pintada sem o botão");
    assert!(s.resolve().is_empty(), "e soltar não faz nada");
}

/// **O gesto central:** arrastar por cima de várias regiões e soltar dá UMA forma.
#[test]
fn dragging_across_regions_merges_them_into_one_shape() {
    let (mut sc, xf, ids) = scene();
    let mut s = session(&sc, &xf, &ids);
    s.dragging = true;
    for p in [IN_PENT, ONLY_RECT, IN_STAR] {
        s.touch(p);
    }
    assert_eq!(s.marked.len(), 3, "as três faces foram pintadas");
    let n_before = sc.paths().len();
    commit(&mut sc, &mut s);

    // As três faces se tocam (o corredor liga as duas pontas) ⇒ UMA forma nova, que contém
    // os três pontos. E ela herda o tom do TOPO (a estrela, 33).
    let merged: Vec<&VecPath> = sc
        .paths()
        .iter()
        .filter(|p| {
            [IN_PENT, ONLY_RECT, IN_STAR]
                .iter()
                .all(|q| contains_point(p, *q))
        })
        .collect();
    assert_eq!(merged.len(), 1, "uma forma só (as três faces se tocam)");
    assert_eq!(tone(merged[0]), Some(33), "estilo do topo (Illustrator)");
    assert!(
        sc.paths().len() < n_before + 3,
        "a cena não explodiu em cacos"
    );
}

/// **A LUA CRESCENTE** — o gesto que justifica a feature existir. Alt + clicar na
/// sobreposição: ela some, e sobram as duas fatias.
#[test]
fn alt_clicking_the_overlap_deletes_it_and_leaves_the_slices() {
    let (mut sc, xf, ids) = scene();
    let mut s = session(&sc, &xf, &ids);
    s.subtract = true;
    s.dragging = true;
    s.touch(IN_STAR); // estrela ∩ retângulo

    commit(&mut sc, &mut s);
    let hits = |p: [f64; 2]| sc.paths().iter().filter(|q| contains_point(q, p)).count();
    assert_eq!(hits(IN_STAR), 0, "a sobreposição SUMIU — é a lua crescente");
    assert!(hits(IN_PENT) > 0, "o pentágono ficou");
    assert!(hits(ONLY_RECT) > 0, "e o corredor do retângulo também");
    // E o pentágono continua sendo o pentágono.
    assert!(sc.paths().iter().any(|p| p.id == ids[1]), "intocado");
}

/// **Unir e subtrair são COMPLEMENTARES**, e é por isso que saem da mesma conta: o que uma
/// entrega, a outra joga fora. Se divergirem, existe uma sequência de gestos que perde ou
/// duplica geometria — e o artista descobre isso tarde.
#[test]
fn merge_and_subtract_are_exact_complements_of_the_same_gesture() {
    let paint = |subtract: bool| {
        let (mut sc, xf, ids) = scene();
        let mut s = session(&sc, &xf, &ids);
        s.subtract = subtract;
        s.dragging = true;
        s.touch(IN_STAR);
        commit(&mut sc, &mut s);
        sc
    };
    let merged = paint(false);
    let subtracted = paint(true);
    let hits =
        |sc: &VecScene, p: [f64; 2]| sc.paths().iter().filter(|q| contains_point(q, p)).count();

    // A sobra é IDÊNTICA nos dois; só a face pintada muda de lado.
    for p in [IN_PENT, ONLY_RECT] {
        assert_eq!(hits(&merged, p), hits(&subtracted, p), "a sobra é a mesma");
    }
    assert_eq!(hits(&merged, IN_STAR), 1, "unir entrega a face");
    assert_eq!(hits(&subtracted, IN_STAR), 0, "subtrair a joga fora");
}

/// Soltar sem ter pintado nada é um NO-OP. Sem esta guarda, um clique perdido no vazio
/// destruiria as formas de origem.
#[test]
fn releasing_without_painting_anything_is_a_no_op() {
    let (mut sc, xf, ids) = scene();
    let before: Vec<VecPathId> = sc.paths().iter().map(|p| p.id).collect();
    let mut s = session(&sc, &xf, &ids);
    s.dragging = true;
    s.touch([50.0, 50.0]); // fora de tudo
    assert!(s.marked.is_empty());
    assert!(s.resolve().is_empty(), "nada pintado, nada feito");
    commit(&mut sc, &mut s);
    let after: Vec<VecPathId> = sc.paths().iter().map(|p| p.id).collect();
    assert_eq!(before, after, "a cena não foi tocada");
}

/// A mesma face pintada duas vezes (o cursor volta por cima dela) conta UMA vez — senão a
/// união receberia a mesma geometria repetida.
#[test]
fn painting_the_same_face_twice_marks_it_once() {
    let (sc, xf, ids) = scene();
    let mut s = session(&sc, &xf, &ids);
    s.dragging = true;
    s.touch(IN_STAR);
    s.touch([1.8, 0.05]); // ainda na estrela
    s.touch(IN_STAR);
    assert_eq!(s.marked.len(), 1, "uma face, uma marca");
}

/// **Os índices do arranjo e os ids TÊM de andar juntos.** Uma forma ABERTA na seleção não
/// entra no arranjo — e se ela ficasse na lista de ids, o índice `i` da fonte apontaria
/// para o id errado e o `commit` consumiria a forma errada. (Latente na 1ª versão: os ids
/// eram copiados sem filtro, e nenhum gate misturava aberta com fechada.)
#[test]
fn an_open_path_in_the_selection_does_not_shift_the_ids() {
    let (mut sc, xf, mut ids) = scene();
    let open = sc.push_path(VecPath {
        verts: vec![
            VecVertex::corner([-9.0, -9.0]),
            VecVertex::corner([-8.0, -8.0]),
        ],
        closed: false,
        ..VecPath::default()
    });
    // A aberta entra na seleção NO MEIO (o pior caso p/ o alinhamento).
    ids.insert(1, open);
    let s = session(&sc, &xf, &ids);
    assert_eq!(s.sources.len(), s.arr.len(), "ids e arranjo alinhados");
    assert!(!s.sources.contains(&open), "a aberta não é uma fonte");
    for (i, id) in s.sources.iter().enumerate() {
        let world = sc.path_world_curve_bbox(&xf, *id).unwrap();
        let src = &s.arr.sources()[i];
        let cx = src.verts.iter().map(|v| v.anchor[0]).sum::<f64>() / src.verts.len() as f64;
        assert!(
            cx >= world.0[0] && cx <= world.1[0],
            "a fonte[{i}] é mesmo o path {id}"
        );
    }
}

/// Uma sessão precisa de 2+ formas FECHADAS. Com menos, não há região para pintar — e abrir
/// a sessão mesmo assim faria o modo capturar o canvas e não fazer nada (o pior estado de UI
/// que existe: parece quebrado).
#[test]
fn a_session_needs_at_least_two_closed_shapes() {
    let (sc, xf, ids) = scene();
    assert!(BuildSession::open(&sc, &xf, &ids[..1]).is_none(), "uma só");
    assert!(BuildSession::open(&sc, &xf, &[]).is_none(), "nenhuma");
    assert!(BuildSession::open(&sc, &xf, &ids[..2]).is_some());
}

/// **O arranjo é assado em MUNDO — então a POSE faz parte da identidade dele.** Se a forma
/// se move (ou volta de um undo) e o arranjo não é refeito, o véu descreve a forma onde ela
/// *estava*. É a chave que o `build_session_upkeep` compara a cada frame.
#[test]
fn the_source_key_changes_when_the_pose_or_the_geometry_changes() {
    let (mut sc, mut xf, ids) = scene();
    let k0 = crate::shape_build::source_key(&sc, &xf, &ids);

    // A pose muda (o gizmo moveu a forma).
    xf.insert(ids[2], at(2.25, 0.4));
    let k1 = crate::shape_build::source_key(&sc, &xf, &ids);
    assert_ne!(k0, k1, "mover a forma reabre o arranjo");

    // A geometria muda (um undo trouxe outra forma no mesmo id).
    let p = sc.paths_mut().iter_mut().find(|p| p.id == ids[0]).unwrap();
    p.verts[0].anchor[0] += 0.5;
    let k2 = crate::shape_build::source_key(&sc, &xf, &ids);
    assert_ne!(k1, k2, "mexer na geometria reabre o arranjo");

    // E a seleção mesma.
    assert_ne!(k2, crate::shape_build::source_key(&sc, &xf, &ids[..2]));
}
