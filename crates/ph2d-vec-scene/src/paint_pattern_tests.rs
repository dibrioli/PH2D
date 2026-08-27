//! Os gates do **dado** do Texture Pattern (plano 33, W3).
//!
//! ⚠️ A fixtura contém os fenómenos que a §5.1 do plano declarou: escala não-uniforme, rotação,
//! e um vão que a colmeia tem de ignorar.

use super::{Paint, PatternFill, PatternSource, Rgba8, VecPath, VecPathId, VecScene, VecVertex};
use crate::paint_bind::BoundStyle;
use ph2d_asset_id::AssetId;
use ph2d_vec_pattern::{PatternMode, TileKind};

fn src() -> PatternSource {
    PatternSource::Image(AssetId::from_bytes(b"tijolo"))
}

fn fill() -> PatternFill {
    PatternFill::new(src(), [10.0, 20.0], Rgba8::new(200, 30, 30, 255))
}

/// Uma forma quadrada com o padrão de referência já vestido.
fn scene_with_pattern(f: PatternFill) -> (VecScene, VecPathId) {
    let mut scene = VecScene::default();
    let verts = [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]]
        .map(VecVertex::corner)
        .to_vec();
    let id = scene.push_path(VecPath {
        verts,
        closed: true,
        fill: Some(Paint::Pattern(Box::new(f))),
        ..VecPath::default()
    });
    (scene, id)
}

/// ⚠️ **O `Paint` NÃO ENGORDA** — e o número é medido, não estimado.
///
/// Ele mora dentro de todo `VecPath`, e todo `VecPath` entra em toda fotografia de undo. Medido em
/// 2026-08-27, ANTES desta wave: `size_of::<Paint>() == 56`. Um [`PatternFill`] em linha levá-lo-ia
/// a mais do dobro — pago por cada forma da cena a cada passo de undo, **inclusive pelas que não
/// têm padrão nenhum**.
///
/// ⭐ A desigualdade é a afirmação portátil (`Paint` mais pequeno que o `PatternFill` que ele
/// referencia ⇒ há indirecção); o `56` ao lado é o número que a medição deu.
#[test]
fn the_paint_enum_does_not_grow_when_pattern_lands() {
    let paint = std::mem::size_of::<Paint>();
    assert!(
        paint < std::mem::size_of::<PatternFill>(),
        "o PatternFill entrou em linha no Paint: {paint} bytes"
    );
    assert_eq!(
        paint, 56,
        "o Paint mudou de tamanho ({paint} != 56) - reconfira o custo por passo de undo antes de \
         actualizar este numero"
    );
}

/// ⭐⭐ **A colmeia DERIVA o passo vertical, e o `gap[1]` autorado é ignorado nela.**
///
/// Dois sítios a decidir o mesmo passo dariam um desenho num instante e um espaçamento noutro.
/// Aqui a lei vive na [`PatternFill::period`], e este gate mede que ela **manda** — mexer no vão
/// vertical de uma colmeia não pode mover coisa nenhuma.
#[test]
fn the_hex_period_is_derived_and_the_authored_y_gap_is_ignored() {
    let mut f = fill();
    f.kind = TileKind::Hex;
    f.gap = [0.0, 0.0];
    let base = f.period();
    f.gap = [0.0, 999.0];
    assert_eq!(base, f.period(), "a colmeia leu o vao vertical autorado");
    assert!(
        (base[1] - base[0] * ph2d_vec_pattern::HEX_ROW_RATIO).abs() < 1e-12,
        "o passo vertical da colmeia nao e' o derivado"
    );
    // Controlo: numa grade o vão vertical MANDA — senão este gate estaria a medir uma função morta.
    let mut g = fill();
    g.kind = TileKind::Grid;
    let a = g.period();
    g.gap = [0.0, 999.0];
    assert_ne!(a, g.period(), "numa grade o vao vertical tem de contar");
}

/// **Mundo -> pixels atravessa UMA porta**, e ela alimenta-se do período (não do vão autorado) —
/// é isso que faz a colmeia usar a mesma conta que todo o resto.
#[test]
fn the_pattern_law_crosses_one_door_from_world_to_pixels() {
    let mut f = fill();
    f.gap = [5.0, 10.0];
    // A arte mede 10x20 no mundo e tem 100x200 px ⇒ 10 px por unidade nos dois eixos.
    let law = f.law([100, 200]);
    assert_eq!(law.gap_px, [50, 100]);
    // E a colmeia entra pela MESMA porta, com o vão vertical que o período dela impõe.
    let mut h = fill();
    h.kind = TileKind::Hex;
    h.size = [10.0, 10.0];
    let hl = h.law([100, 100]);
    assert_eq!(hl.gap_px[0], 0, "sem vao horizontal autorado");
    assert!(
        hl.gap_px[1] < 0,
        "a colmeia aperta as linhas (sqrt(3)/2 < 1), deu {}",
        hl.gap_px[1]
    );
}

/// ⚠️ **Desvanecer um padrão baixa a OPACIDADE dele, não uma cor.** Um padrão não tem cor para
/// escalar — se só a `fallback` descesse, a forma manteria o ladrilho a cheio e clarearia apenas o
/// instante em que ele ainda não resolveu, que é o contrário do que se vê.
#[test]
fn fading_a_pattern_dims_the_pattern_not_just_its_fallback() {
    let (scene, id) = scene_with_pattern(fill());
    let bound = BoundStyle {
        path: id,
        alpha: Some(128),
        ..BoundStyle::default()
    };
    let painted = scene.paths()[0].painted(Some(&bound));
    let Some(Paint::Pattern(p)) = &painted.fill else {
        panic!("o desvanecimento trocou a especie do preenchimento")
    };
    assert!(
        (p.alpha - 128.0 / 255.0).abs() < 1e-6,
        "a opacidade do padrao nao desceu: {}",
        p.alpha
    );
    // (255*128 + 127) / 255 = 128 — a conta arredondada do `fade`, não a minha estimativa.
    assert_eq!(p.fallback.a, 128, "a cor de recurso desce junto");
}

/// ⭐⭐ **O padrão CONSERVA A ORIENTAÇÃO quando a forma roda** — e é o único preenchimento desta
/// casa que o faz.
///
/// O gradiente radial não pode: um radial do peniko **é circular** e não tem onde guardar um
/// ângulo, e é por isso que o `transform_fill_geometry` lhe passa um `radius_scale` médio. O padrão
/// tem o campo, então a sonda do afim (as imagens dos dois eixos unitários) dá-lhe a resposta exacta.
#[test]
fn rotating_the_shape_rotates_the_pattern_with_it() {
    let (mut scene, id) = scene_with_pattern(fill());
    let quarter = std::f64::consts::FRAC_PI_2;
    assert!(scene.rotate_path_by(id, quarter, [0.0, 0.0]));
    let Some(Paint::Pattern(p)) = &scene.paths()[0].fill else {
        panic!("especie trocada")
    };
    assert!(
        (p.angle - quarter).abs() < 1e-9,
        "o padrao nao rodou com a forma: {}",
        p.angle
    );
    assert!(
        (p.size[0] - 10.0).abs() < 1e-9 && (p.size[1] - 20.0).abs() < 1e-9,
        "uma rotacao nao pode mudar o tamanho do ladrilho: {:?}",
        p.size
    );
}

/// **Escalar a forma escala o ladrilho, POR EIXO** — e a escala não-uniforme é o fenómeno que a
/// fixtura tinha de conter (plano 33 §5.1).
#[test]
fn scaling_the_shape_scales_the_tile_per_axis() {
    let (mut scene, id) = scene_with_pattern(fill());
    assert!(scene.scale_path(id, 3.0, 0.5, [0.0, 0.0]));
    let Some(Paint::Pattern(p)) = &scene.paths()[0].fill else {
        panic!("especie trocada")
    };
    assert!(
        (p.size[0] - 30.0).abs() < 1e-9 && (p.size[1] - 10.0).abs() < 1e-9,
        "o ladrilho nao seguiu a escala por eixo: {:?}",
        p.size
    );
    assert!(
        p.angle.abs() < 1e-9,
        "uma escala positiva nao pode rodar o padrao"
    );
}

/// **Mover a forma move o padrão com ela** — a lei que o `paint.rs` já escreveu para os gradientes,
/// e o oposto do defeito da origem-da-régua do Illustrator.
#[test]
fn moving_the_shape_moves_the_pattern_with_it() {
    let (mut scene, id) = scene_with_pattern(fill());
    assert!(scene.translate_path(id, 7.0, -3.0));
    let Some(Paint::Pattern(p)) = &scene.paths()[0].fill else {
        panic!("especie trocada")
    };
    assert_eq!(p.origin, [7.0, -3.0]);
    assert!(p.angle.abs() < 1e-9 && (p.size[0] - 10.0).abs() < 1e-9);
}

/// **A cor representativa de um padrão é a `fallback`** — uma resposta EXACTA, e não uma
/// aproximação: é literalmente a cor que ele pinta enquanto o ladrilho não resolve.
#[test]
fn the_swatch_colour_of_a_pattern_is_its_fallback() {
    let f = fill();
    let c = f.fallback;
    assert_eq!(Paint::Pattern(Box::new(f)).primary_color(), c);
}

/// **O padrão sobrevive ao save**, com a fonte e a lei intactas.
#[test]
fn a_pattern_round_trips_through_postcard() {
    let mut f = fill();
    f.kind = TileKind::BrickRow;
    f.offset_denom = 3;
    f.mode = PatternMode::Mirror;
    f.angle = 0.75;
    let (scene, _) = scene_with_pattern(f.clone());
    let bytes = scene.to_bytes().expect("serializa");
    let back = VecScene::from_bytes(&bytes).expect("desserializa");
    let Some(Paint::Pattern(p)) = &back.paths()[0].fill else {
        panic!("o padrao nao sobreviveu ao save")
    };
    assert_eq!(**p, f);
}
