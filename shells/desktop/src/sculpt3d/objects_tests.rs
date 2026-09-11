//! **UMA PEÇA NASCE ESCULPÍVEL** — os gates da densidade de entrada (ADR-0150).
//!
//! Módulo irmão de teste do [`super`] (`#[path]`, `cfg(test)`). ⚠️ **Nada aqui pede GPU**, e é
//! deliberado: uma peça é feita por funções puras (`Primitive::mesh`), a régua é pura
//! (`span_per_edge`), a câmera é pura (`Camera3d::frame`) e o dab é puro (`SculptStroke`).
//! O defeito que estes gates fecham viveu **exatamente** por não existir um teste que juntasse
//! essas quatro coisas — cada uma tinha a sua suíte, e nenhuma delas se encontrava com as outras.
//!
//! ```text
//! cargo test -p ph2d-host-desktop --bins sculpt3d::objects::tests
//! ```

use ph2d_mesh_render::Camera3d;
use ph2d_sculpt3d::{Brush, Dab, SculptStroke, Symmetry};

use super::*;

/// As quatro, para nenhum gate afirmar sobre a esfera e calar sobre o resto.
const ALL: [Primitive; 4] = [
    Primitive::Sphere,
    Primitive::Cube,
    Primitive::Cylinder,
    Primitive::Torus,
];

/// A base NÃO-REFINADA de cada primitiva — a densidade de blocagem, antes da régua.
///
/// ⚠️ **Escrita aqui e não pedida ao produto de propósito:** o `Primitive::mesh` já refina, e um
/// gate que quisesse a base pedindo-a a ele estaria a comparar a cura com ela mesma.
fn base_of(p: Primitive) -> ph2d_mesh::Mesh {
    match p {
        Primitive::Sphere => ph2d_mesh::shapes::uv_sphere(SEGMENTS / 2, SEGMENTS, 1.0),
        Primitive::Cube => ph2d_mesh::shapes::cube(2.0 / 3.0_f32.sqrt()),
        Primitive::Cylinder => ph2d_mesh::shapes::cylinder(SEGMENTS, 0.7, 2.0),
        Primitive::Torus => ph2d_mesh::shapes::torus(SEGMENTS, SEGMENTS / 2, 0.7, 0.3),
    }
}

/// A tela em que o defeito é PIOR — o raio de mundo de um pincel de 50 px encolhe com a altura do
/// viewport, então 4K é o pior caso entre as resoluções que este app roda.
const VIEWPORT_4K: (u32, u32) = (3840, 2160);

/// Quantos vértices um dab do pincel PADRÃO move nesta malha, pelas portas do produto.
///
/// ⚠️ **A régua é a `Camera3d` de verdade**, enquadrada como o `Sculpt3dScene::new` a enquadra, e
/// a conversão é a `world_radius_for_screen_px` que o `armed_brush` chama. Um raio de mundo escrito
/// à mão aqui tornaria o gate cego justamente à metade que causou o defeito: *quanto mundo cabe em
/// 50 px*.
fn verts_moved_by_a_default_dab(mesh: &ph2d_mesh::Mesh) -> usize {
    let bounds = mesh.bounds();
    let aspect = VIEWPORT_4K.0 as f32 / VIEWPORT_4K.1 as f32;
    let mut cam = Camera3d::default();
    cam.frame(bounds, aspect);

    // O ponto mais próximo da câmera ao longo de +Z: é onde um clique no meio da tela pousa.
    let at = mesh
        .positions()
        .iter()
        .copied()
        .fold([0.0f32, 0.0, f32::NEG_INFINITY], |a, p| {
            if p[2] > a[2] { p } else { a }
        });
    let radius = cam.world_radius_for_screen_px(at, super::super::DEFAULT_RADIUS_PX, VIEWPORT_4K);
    assert!(
        radius > 0.0,
        "a camera nao devolveu raio para o ponto de acerto — a fixture nao contem o fenomeno"
    );

    let brush = Brush {
        radius,
        ..Brush::default()
    };
    let before: Vec<[f32; 3]> = mesh.positions().to_vec();
    let mut m = mesh.clone();
    let mut stroke = SculptStroke::default();
    stroke.begin(&m);
    stroke.dab(
        &mut m,
        &brush,
        &Dab::at(at, brush.radius, [0.0, 0.0, -1.0]),
        Symmetry::default(),
    );
    before
        .iter()
        .zip(m.positions())
        .filter(|(a, b)| a != b)
        .count()
}

/// **UM DAB MOVE UMA PEGADA, e não um espeto** — o report *"a escultura não funciona"* numa
/// asserção.
///
/// ⚠️ **O oráculo é *quantos vértices se MOVERAM*, e essa escolha é a wave inteira.** O
/// `sculpt_at` do produto devolve `true` quando o RAIO ACERTA, e as faces da peça de blocagem
/// medem `0,262` de mundo, então o raio sempre acertava: o gesto virava `Drag::Sculpt`, o
/// fallback de órbita nunca disparava, e o artista arrastava sem que o modelo deformasse **nem
/// girasse**. Nenhum erro, nenhum log. Um gate que perguntasse *"o clique foi aceito?"* teria
/// ficado verde sobre isso — foi o que os gates existentes fizeram.
///
/// ⚠️ **O piso é a cena do SMOKE que o artista já aprovou** (`uv_sphere(96,144)`: **15** vértices
/// no dab a 4K), e não um número escolhido — ele é MEDIDO na mesma corrida, ao lado do que julga,
/// então nenhum dos dois pode envelhecer sozinho. Toda peça nasce nessa densidade ou acima dela
/// (medido: **21 · 37 · 67 · 15** para esfera · cubo · cilindro · toro).
#[test]
fn a_default_dab_on_a_newborn_piece_moves_a_real_footprint() {
    // O piso: o que a cena do smoke entrega na MESMA medição.
    let smoke = verts_moved_by_a_default_dab(&ph2d_mesh::shapes::uv_sphere(96, 144, 1.0));
    assert!(
        smoke >= 8,
        "a cena do smoke deixou de mover uma pegada ({smoke} vertices) — o PISO deste gate saiu \
         de baixo dele, e o resto do que ele afirma nao significa mais nada"
    );

    for p in ALL {
        let moved = verts_moved_by_a_default_dab(&p.mesh());
        assert!(
            moved >= smoke,
            "a peca `{}` nasce inesculpivel: um dab do pincel padrao move {moved} vertices a 4K, \
             contra os {smoke} da cena do smoke. Foi assim que a peca de blocagem chegou ao \
             produto — com UM vertice movido, o do centro, em TODA resolucao",
            p.label()
        );
    }
}

/// **A peça de blocagem CRUA move um espeto** — o controle positivo do gate acima.
///
/// ⚠️ Sem ele o irmão poderia estar verde por vácuo (uma malha absurdamente densa passaria, e uma
/// régua quebrada também). Este mede a MESMA coisa na base não-refinada e exige que ela **falhe**:
/// é a prova de que a asserção do irmão é sobre o refinamento e não sobre o acaso.
#[test]
fn the_unrefined_blocking_base_is_the_defect_this_wave_closes() {
    for p in ALL {
        let moved = verts_moved_by_a_default_dab(&base_of(p));
        assert!(
            moved <= 1,
            "a base de blocagem de `{}` move {moved} vertices a 4K — se ela deixou de ser o \
             defeito, o refinamento perdeu a razao de existir e este par de gates tem de ser \
             relido",
            p.label()
        );
    }
}

/// **TODA peça carrega padrão** — o report *"as configurações da textura não foram obedecidas"*
/// pelo outro lado.
///
/// ⚠️ **O `0.50` que o artista viu no `Alpha Scale` não foi escolhido por ninguém:** é o
/// [`ph2d_sculpt3d::MAX_ALPHA_SCALE`], o valor em que a `recommended_scale` pousa no regime que o
/// doc-comment dela nomeia — *"nem o teto basta: a malha não carrega padrão nenhum"*. Enquanto a
/// peça nascer ali, o padrão sai grosso **e nenhum controle conserta**, porque não existe escala
/// que sirva.
///
/// ⚠️ **Isto é CONSEQUÊNCIA da régua, não a régua** — a primeira versão desta wave usou o teto do
/// padrão como regra de parada e deixou o cubo com UM vértice na pegada. Uma malha que carrega um
/// traço carrega um padrão; a recíproca é falsa, e o gate irmão é quem mede a que importa.
#[test]
fn every_primitive_is_born_able_to_carry_a_pattern() {
    for p in ALL {
        let s = ph2d_sculpt3d::recommended_scale(&p.mesh());
        assert!(
            s < ph2d_sculpt3d::MAX_ALPHA_SCALE,
            "a peca `{}` nasce no regime em que a malha nao carrega padrao nenhum \
             (rec_scale {s:.4} == teto {:.4})",
            p.label(),
            ph2d_sculpt3d::MAX_ALPHA_SCALE
        );
    }
    // Controle positivo: a base CRUA está no teto. Sem isto o laço acima poderia estar verde
    // porque a régua parou de distinguir os dois regimes.
    let base = ph2d_mesh::shapes::uv_sphere(SEGMENTS / 2, SEGMENTS, 1.0);
    assert!(
        ph2d_sculpt3d::recommended_scale(&base) >= ph2d_sculpt3d::MAX_ALPHA_SCALE,
        "a base crua saiu do regime degenerado sozinha — a regua mudou, e a regra de parada do \
         `refine_until_sculptable` tem de ser relida contra ela"
    );
}

/// **O refinamento PARA** — e para pelo próprio critério, não pelo backstop.
///
/// ⚠️ O backstop existe para um laço nunca fugir; se ele passasse a ser quem termina o trabalho,
/// a peça sairia com a densidade que o TETO deu em vez da que a régua pediu, e o gate acima
/// continuaria verde por acidente.
#[test]
fn the_refinement_stops_by_its_own_rule_never_by_the_backstop() {
    for p in ALL {
        let mut levels = 0;
        let mut m = base_of(p);
        while span_per_edge(&m) < MIN_SPAN_PER_EDGE {
            m = ph2d_mesh::subdivide(&m);
            levels += 1;
            assert!(
                levels <= MAX_REFINE_LEVELS,
                "a peca `{}` nao satisfaz o criterio em {MAX_REFINE_LEVELS} niveis",
                p.label()
            );
        }
        assert!(
            levels < MAX_REFINE_LEVELS,
            "a peca `{}` so' para no backstop ({levels} niveis): quem terminou o trabalho foi o \
             teto, nao a regra",
            p.label()
        );
    }
}
