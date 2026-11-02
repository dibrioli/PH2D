//! ⭐ **A cena 22** (`PH2D_OBJECT_MODE_SMOKE=22`, dono 15/10): O ARRANGE E O ALINHAR NA ④. No layout
//! *Vetor*, três formas espalhadas e desalinhadas, o *Select* escolhido pelo MESMO clique que o
//! botão dele na coluna ① dá, e as três selecionadas — a foto mostra a ④ com as DUAS linhas: o
//! Arrange inteiro na de cima, o Alinhar e o Distribuir na de baixo; e o painel sem as secções.
//!
//! ⭐ **A cena 52** (dono 02/11, o Draw FICA ao escolher outras formas): a 22 com uma IMAGEM em branco
//! ao lado (o `Ctrl+N`, antes das formas) e SÓ o rectângulo seleccionado no fim — o dono escolhe as
//! outras formas (canvas, Hierarchy, laço) e o Draw fica; escolhe a imagem e volta a Object.
//!
//! ⭐ **A cena 53** (F50, dono 02/11: Boolean e Convert na ④): o rectângulo e a elipse SOBREPOSTOS e
//! os dois escolhidos, a estrela ao lado — a ④ mostra as quatro booleanas, o Flatten, o Outline
//! Stroke e o «⋯» das raras.

use crate::state::VecState;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_vec_scene::{ShapeKind, VecPathId};

/// As três formas: o tipo, o centro (m) e a meia-largura (m) — alturas e tamanhos diferentes, para
/// que cada botão do Alinhar mexa em todas.
const FORMAS: [(ShapeKind, [f64; 2], f64); 3] = [
    (ShapeKind::Rectangle, [-1.6, 0.5], 0.5),
    (ShapeKind::Ellipse, [0.0, -0.4], 0.35),
    (ShapeKind::Star, [1.5, 0.2], 0.6),
];

/// As da cena 53: o rectângulo e a elipse sobrepostos (as booleanas MOSTRAM algo), a estrela à parte.
const FORMAS_53: [(ShapeKind, [f64; 2], f64); 3] = [
    (ShapeKind::Rectangle, [-0.6, 0.0], 0.6),
    (ShapeKind::Ellipse, [0.2, 0.45], 0.55),
    (ShapeKind::Star, [1.9, 0.0], 0.55),
];

/// O primeiro estágio (uma forma por estágio, o clique no Select, a seleção das três).
pub(crate) const FIRST: u8 = 220;
const SELECT: u8 = FIRST + FORMAS.len() as u8;
pub(crate) const LAST: u8 = SELECT + 1;

/// Um passo da cena 22 (`h` = o nº da família: 8 = a 22, 38 = a 52, 39 = a 53); devolve o próximo
/// estágio, se mudou. Cada forma espera a anterior virar
/// objecto; a troca de ferramenta reduz a seleção, por isso as três são selecionadas DEPOIS dela.
pub fn step(
    stage: u8,
    vec: &mut VecState,
    scene: &mut ph2d_vec_scene::VecScene,
    sim: &mut ph2d_ecs::SimWorld,
    hero: &mut HeroScreen,
    h: u8,
) -> Option<u8> {
    let (cena_52, cena_53) = (h == 38, h == 39);
    let formas = if cena_53 { FORMAS_53 } else { FORMAS };
    let feitas = vec.entities.len();
    match stage {
        FIRST if cena_52 && imagem(sim).is_none() => {
            if !PEDIU.swap(true, std::sync::atomic::Ordering::Relaxed) {
                hero.store.set_new_image_size(IMAGEM_PX);
                hero.store.set_new_image_bg(2);
                hero.store.request_new_image();
            }
            None
        }
        // À esquerda das formas: o clique nela não cai em nenhuma forma.
        FIRST if cena_52 && !MOVEU.swap(true, std::sync::atomic::Ordering::Relaxed) => {
            let e = imagem(sim)?;
            sim.world_mut()
                .get_mut::<ph2d_ecs::Transform>(e)?
                .translation
                .x = IMAGEM_X;
            None
        }
        s if (FIRST..SELECT).contains(&s) && feitas == usize::from(s - FIRST) => {
            let (kind, at, half) = formas[usize::from(s - FIRST)];
            crate::vector_mode::draw_shape(vec, scene, kind, at, half);
            Some(s + 1)
        }
        SELECT if feitas == FORMAS.len() => {
            hero.apply_event(WidgetEvent::Click(
                ph2d_tool_vector::ids::VECTOR_MODE_SELECT,
            ));
            Some(LAST)
        }
        LAST if hero.store.tool_bars().is_some_and(|b| {
            b.column
                .iter()
                .flatten()
                .any(|x| x.on && x.id == ph2d_tool_vector::ids::VECTOR_MODE_SELECT)
        }) =>
        {
            let all: Vec<VecPathId> = vec.entities.keys().copied().collect();
            let quantas = match (cena_52, cena_53) {
                (true, _) => 1,
                (_, true) => 2,
                _ => all.len(),
            };
            vec.pen.select_many(&all[..quantas]);
            Some(9)
        }
        _ => None,
    }
}

/// A imagem da cena 52 (px do documento): pequena, ao lado das formas.
const IMAGEM_PX: u32 = 256;
/// O centro dela (m): a borda direita fica à esquerda do rectângulo (`-1.6 ± 0.5`).
const IMAGEM_X: f32 = -4.0;
static PEDIU: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static MOVEU: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn imagem(sim: &mut ph2d_ecs::SimWorld) -> Option<ph2d_ecs::Entity> {
    let w = sim.world_mut();
    let mut q = w.query::<(ph2d_ecs::Entity, &ph2d_render::Sprite)>();
    q.iter(w).next().map(|(e, _)| e)
}
