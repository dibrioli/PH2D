//! Gates da alça de **raio de quina** — no SEAM, não na unidade.
//!
//! O motor já tem os gates dele (`ph2d_vec_scene::corner_live`), e eles provam que a
//! matemática está certa. Estes provam a outra metade, que é onde a feature morre de
//! verdade: **a alça está LIGADA?**
//!
//! Uma tool pode passar em todo unit test e estar morta no produto porque o input nunca
//! chega nela (a memória `feedback_tool_unit_green_integration_dead` — uma tool inteira
//! passou CI com o pill não-wirado). Então aqui não se chama `radius_for_setback` a seco:
//! **pressiona-se na alça, arrasta-se, e exige-se que a FORMA tenha arredondado** — pelo
//! mesmo caminho que o mouse do Enio percorre (`on_press_node` → `on_drag` → `on_release`).

use super::*;
use ph2d_vec_scene::corner_live::CORNER_HANDLE_PARK_PX;

fn nosnap(p: [f64; 2]) -> [f64; 2] {
    p
}

/// World-units por pixel. **Não é 1.0 de propósito**: 1.0 é o único valor que esconde um
/// erro de unidade (a memória `feedback_test_with_product_numbers_not_convenient_ones`).
const PTW: f64 = 0.01;

/// Um quadrado 10×10 selecionado, pronto para ter as quinas arredondadas.
fn square_scene() -> (VecScene, PenTool, VecPathId) {
    let mut scene = VecScene::new();
    let id = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    });
    let mut pen = PenTool::new();
    pen.select(Some(id));
    (scene, pen, id)
}

/// Onde a alça da quina `i` está desenhada, no mundo — pela MESMA função que o passe de
/// render chama. Se o hit-test procurasse noutro lugar, este teste ficaria vermelho.
fn handle_of(scene: &VecScene, pen: &PenTool, id: VecPathId, i: usize) -> [f64; 2] {
    let xf = ph2d_vec_scene::xform_of(&pen.xforms, id);
    corner_handle::view(scene, id, &xf, CORNER_HANDLE_PARK_PX * PTW)
        .into_iter()
        .find(|h| h.vert == i)
        .expect("a quina existe e tem alça")
        .pos
}

/// **O gate anti-item-morto: pressionar na alça e arrastar ARREDONDA a forma.**
///
/// Ele percorre o caminho inteiro — o hit-test acha a alça na posição em que ela foi
/// DESENHADA, o arrasto vira raio, o raio vira geometria. Qualquer elo solto (o `Part`
/// novo não chegando no `match`, o hit-test procurando noutro raio, o cozimento não sendo
/// consultado) deixa isto vermelho.
#[test]
fn pressing_the_handle_and_dragging_inward_actually_rounds_the_corner() {
    let (mut scene, mut pen, id) = square_scene();
    let h = handle_of(&scene, &pen, id, 0);

    assert_eq!(
        pen.on_press_node(&mut scene, h, PTW, false),
        PenClick::Grabbed,
        "a pressão SOBRE a alça tem de agarrá-la"
    );

    // Leva o CURSOR até `park + 2` ao longo da bissetriz da quina (0,0) — que num canto
    // reto de 90° aponta para (1,1)/√2. O recuo lido é `cursor − park` = 2, e numa quina
    // de 90° o raio É o recuo (tan 45° = 1).
    let park = CORNER_HANDLE_PARK_PX * PTW;
    let d = (park + 2.0) / 2.0_f64.sqrt();
    assert!(pen.on_drag(&mut scene, [d, d], &mut nosnap));
    pen.on_release();

    let path = scene.path_mut(id).unwrap();
    let r = path.verts[0].corner_radius;
    assert!(
        (r - 2.0).abs() < 1e-6,
        "recuo de 2 numa quina de 90° = raio 2 (tan 45° = 1); veio {r}"
    );
    // E a FORMA de fato arredondou: a quina virou dois vértices no cozido.
    let cooked = path.cooked();
    assert_eq!(
        cooked.verts.len(),
        5,
        "1 quina arredondada = 4 + 1 vértices"
    );
    assert!(
        cooked.verts.iter().all(|v| v.anchor != [0.0, 0.0]),
        "a quina afiada não pode ter sobrado na geometria cozida"
    );
    // ...mas a FONTE continua guardando a quina afiada. É o coração do "vivo".
    assert_eq!(path.verts.len(), 4, "o documento ainda tem 4 vértices");
    assert_eq!(path.verts[0].anchor, [0.0, 0.0], "e a quina afiada");
}

/// Arrastar a alça de volta até a âncora **afia a quina de novo** — o raio zera. É a mesma
/// afordância do waypoint de conector que some quando largado sobre a reta: o gesto que
/// cria é o gesto que desfaz, e o usuário não precisa procurar um botão.
#[test]
fn dragging_the_handle_back_to_the_anchor_sharpens_the_corner_again() {
    let (mut scene, mut pen, id) = square_scene();
    scene.path_mut(id).unwrap().verts[0].corner_radius = 3.0;

    let h = handle_of(&scene, &pen, id, 0);
    assert_eq!(
        pen.on_press_node(&mut scene, h, PTW, false),
        PenClick::Grabbed
    );
    // De volta para cima da âncora.
    assert!(pen.on_drag(&mut scene, [0.0, 0.0], &mut nosnap));
    pen.on_release();

    let path = scene.path_mut(id).unwrap();
    assert_eq!(
        path.verts[0].corner_radius, 0.0,
        "a quina voltou a ser afiada"
    );
    assert!(
        matches!(path.cooked(), std::borrow::Cow::Borrowed(_)),
        "sem raio, o cozimento volta a ser a identidade"
    );
}

/// **A alça não existe onde não há quina.** Um vértice suave tem handles colineares — não é
/// canto, e arredondá-lo não quer dizer nada. O predicado é o MESMO do cozimento, então a
/// alça nunca promete um arredondamento que a geometria vai recusar.
#[test]
fn a_smooth_vertex_has_no_radius_handle_at_all() {
    let mut scene = VecScene::new();
    let id = scene.push_path(VecPath {
        verts: vec![
            VecVertex::corner([0.0, -10.0]),
            VecVertex::corner([10.0, 0.0]),
            // Suave: handles colineares ⇒ não é quina.
            VecVertex::smooth([0.0, 10.0], [4.0, 10.0], [-4.0, 10.0]),
            VecVertex::corner([-10.0, 0.0]),
        ],
        closed: true,
        ..VecPath::default()
    });
    let mut pen = PenTool::new();
    pen.select(Some(id));

    let xf = ph2d_vec_scene::xform_of(&pen.xforms, id);
    let handles = corner_handle::view(&scene, id, &xf, CORNER_HANDLE_PARK_PX * PTW);
    assert_eq!(handles.len(), 3, "3 quinas, e o vértice suave não é uma");
    assert!(
        handles.iter().all(|h| h.vert != 2),
        "o vértice suave não pode ter alça"
    );
}

/// A alça de uma quina AFIADA fica **estacionada longe da âncora** — se fosse desenhada em
/// cima dela, sumiria dentro do ponto e brigaria com o hit-test do vértice. E o arrasto é
/// relativo: agarrar a alça estacionada e mover um tico faz o raio crescer **de zero**, não
/// saltar para o valor do estacionamento.
#[test]
fn the_handle_of_a_sharp_corner_parks_away_from_the_anchor_and_the_drag_starts_from_zero() {
    let (mut scene, mut pen, id) = square_scene();
    let h = handle_of(&scene, &pen, id, 0);
    let park_world = CORNER_HANDLE_PARK_PX * PTW;
    let d_from_anchor = h[0].hypot(h[1]);
    assert!(
        (d_from_anchor - park_world).abs() < 1e-9,
        "a alça afiada estaciona a {park_world} da âncora, não em cima dela (veio {d_from_anchor})"
    );

    assert_eq!(
        pen.on_press_node(&mut scene, h, PTW, false),
        PenClick::Grabbed
    );
    // Um tico para dentro a partir da posição estacionada: 0.001 world-units na bissetriz.
    let step = 0.001 / 2.0_f64.sqrt();
    assert!(pen.on_drag(&mut scene, [h[0] + step, h[1] + step], &mut nosnap));
    let r = scene.path_mut(id).unwrap().verts[0].corner_radius;
    assert!(
        r > 0.0 && r < 0.01,
        "o raio tem de crescer DE ZERO (veio {r}) — se o arrasto fosse absoluto, ele \
         saltaria para ~{park_world}, o valor do estacionamento"
    );
}

/// A alça é do path SELECIONADO. Sem seleção não há alça — e sem esta guarda o hit-test
/// varreria a cena inteira toda pressão.
#[test]
fn no_selection_means_no_radius_handle_to_grab() {
    let (mut scene, mut pen, id) = square_scene();
    let h = handle_of(&scene, &pen, id, 0);
    pen.select(None);
    // A pressão cai no vazio (a alça fica dentro da forma, mas longe do traço): sem
    // seleção ela não é agarrável, e a quina fica afiada.
    pen.on_press_node(&mut scene, h, PTW, false);
    pen.on_drag(&mut scene, [3.0, 3.0], &mut nosnap);
    pen.on_release();
    assert_eq!(
        scene.path_mut(id).unwrap().verts[0].corner_radius,
        0.0,
        "sem seleção, nada de alça de raio"
    );
}

/// **A alça não escorrega do dedo.** Durante o arrasto, a bolinha tem de ficar exatamente
/// onde o cursor está — foi assim que este gate nasceu vermelho.
///
/// O bug era sutil e só o dedo o via: o estacionamento estava implementado como PISO
/// (`max(setback, park)`) no desenho, enquanto o arrasto lia `setback = projeção − park`.
/// Agarrar a alça estacionada e levar o cursor a 2,0 deixava a bolinha em 1,86 — ela FUGIA
/// enquanto o usuário arrastava. É a mesma família do bug que o `connector.rs` documenta
/// (desenhar num lugar e capturar noutro), na versão que se acumula ao longo do gesto.
#[test]
fn the_handle_stays_under_the_cursor_all_the_way_through_the_drag() {
    let (mut scene, mut pen, id) = square_scene();
    let park = CORNER_HANDLE_PARK_PX * PTW;
    let h = handle_of(&scene, &pen, id, 0);
    assert_eq!(
        pen.on_press_node(&mut scene, h, PTW, false),
        PenClick::Grabbed
    );

    // Vários pontos ao longo do gesto — o erro do piso CRESCIA com a distância, então um
    // ponto só perto do início poderia passar.
    for cursor_dist in [park, park + 0.5, park + 2.0, park + 4.0] {
        let c = cursor_dist / 2.0_f64.sqrt();
        assert!(pen.on_drag(&mut scene, [c, c], &mut nosnap));

        let drawn = handle_of(&scene, &pen, id, 0);
        let drift = (drawn[0] - c).hypot(drawn[1] - c);
        assert!(
            drift < 1e-9,
            "com o cursor a {cursor_dist:.3} da quina, a bolinha foi desenhada a \
             {drift:.6} DELE — ela escorregou do dedo"
        );
    }
    pen.on_release();
}
