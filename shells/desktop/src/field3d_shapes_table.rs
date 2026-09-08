//! ⭐⭐ **A LISTA DO CATÁLOGO DE FORMAS** (W136) — uma linha por porta da paleta.
//!
//! ⚠️ **Só a lista.** O que uma [`super::Family`] é, o que uma [`super::Shape`] carrega e como
//! uma forma nasce vivem no ficheiro irmão — este é o **dado**, e é ele que cresce a cada wave.
//!
//! ⚠️ **A ORDEM não é a identidade de nada** — a chave é (ver o doc da [`super::Shape::key`]).
//! ⛔ E as quatro constantes `SHAPES.len() − N` que existiram aqui morreram na W100: acrescentar
//! no fim fazia o botão *Extrude* abrir o diálogo de escultura, **sem erro nenhum**.

use super::{Family, Make, Shape};
use super::{
    a_banner, a_bent_arrow, a_bezier, a_bolt, a_box, a_box_frame, a_brace, a_capsule, a_check,
    a_chevron, a_circle_segment, a_circle_wave, a_cloud, a_cone, a_cratered_sphere, a_cross,
    a_cut_sphere, a_cylinder, a_delay, a_display, a_document, a_double_arrow, a_drop, a_gear,
    a_gyroid, a_heart, a_helix, a_hollow_dome, a_knurl, a_lens, a_link, a_moon, a_parabola,
    a_parallelogram, a_pie, a_polygon, a_prism, a_pyramid, a_rhombus, a_ring_arc, a_round_cone,
    a_rounded_cylinder, a_shield, a_solid_angle, a_speech_oval, a_speech_rect, a_sphere, a_spiral,
    a_star, a_superformula, a_superquadric, a_tag, a_thought, a_thread, a_torus, a_torus_arc,
    a_torus_knot, a_trapezoid, a_triangle, a_truncated_cone, a_truncated_pyramid, a_tube, a_vesica,
    a_washer, a_wedge, an_arrow, an_ellipsoid, an_octahedron, an_off_page,
};

pub(crate) const SHAPES: &[Shape] = &[
    Shape {
        key: "panel.model3d.add.box",
        family: Family::Blocks,
        make: Make::Formula(a_box),
    },
    Shape {
        key: "panel.model3d.add.sphere",
        family: Family::Round,
        make: Make::Formula(a_sphere),
    },
    Shape {
        key: "panel.model3d.add.cylinder",
        family: Family::Round,
        make: Make::Formula(a_cylinder),
    },
    Shape {
        key: "panel.model3d.add.cone",
        family: Family::Round,
        make: Make::Formula(a_cone),
    },
    Shape {
        key: "panel.model3d.add.cone_truncated",
        family: Family::Round,
        make: Make::Formula(a_truncated_cone),
    },
    Shape {
        key: "panel.model3d.add.capsule",
        family: Family::Round,
        make: Make::Formula(a_capsule),
    },
    Shape {
        key: "panel.model3d.add.prism",
        family: Family::Blocks,
        make: Make::Formula(a_prism),
    },
    Shape {
        key: "panel.model3d.add.pyramid",
        family: Family::Blocks,
        make: Make::Formula(a_pyramid),
    },
    Shape {
        key: "panel.model3d.add.pyramid_truncated",
        family: Family::Blocks,
        make: Make::Formula(a_truncated_pyramid),
    },
    Shape {
        key: "panel.model3d.add.wedge",
        family: Family::Blocks,
        make: Make::Formula(a_wedge),
    },
    Shape {
        key: "panel.model3d.add.box_frame",
        family: Family::Blocks,
        make: Make::Formula(a_box_frame),
    },
    Shape {
        key: "panel.model3d.add.ellipsoid",
        family: Family::Round,
        make: Make::Formula(an_ellipsoid),
    },
    Shape {
        key: "panel.model3d.add.octahedron",
        family: Family::Blocks,
        make: Make::Formula(an_octahedron),
    },
    Shape {
        key: "panel.model3d.add.round_cone",
        family: Family::Round,
        make: Make::Formula(a_round_cone),
    },
    Shape {
        key: "panel.model3d.add.cut_sphere",
        family: Family::Round,
        make: Make::Formula(a_cut_sphere),
    },
    Shape {
        key: "panel.model3d.add.hollow_dome",
        family: Family::Round,
        make: Make::Formula(a_hollow_dome),
    },
    Shape {
        key: "panel.model3d.add.solid_angle",
        family: Family::Round,
        make: Make::Formula(a_solid_angle),
    },
    Shape {
        key: "panel.model3d.add.link",
        family: Family::Rings,
        make: Make::Formula(a_link),
    },
    Shape {
        key: "panel.model3d.add.gear",
        family: Family::Plates,
        make: Make::Formula(a_gear),
    },
    Shape {
        key: "panel.model3d.add.cross",
        family: Family::Plates,
        make: Make::Formula(a_cross),
    },
    Shape {
        key: "panel.model3d.add.heart",
        family: Family::Plates,
        make: Make::Formula(a_heart),
    },
    Shape {
        key: "panel.model3d.add.moon",
        family: Family::Plates,
        make: Make::Formula(a_moon),
    },
    Shape {
        key: "panel.model3d.add.drop",
        family: Family::Plates,
        make: Make::Formula(a_drop),
    },
    Shape {
        key: "panel.model3d.add.pie",
        family: Family::Plates,
        make: Make::Formula(a_pie),
    },
    Shape {
        key: "panel.model3d.add.trapezoid",
        family: Family::Plates,
        make: Make::Formula(a_trapezoid),
    },
    Shape {
        key: "panel.model3d.add.vesica",
        family: Family::Plates,
        make: Make::Formula(a_vesica),
    },
    // ⭐⭐ **A PRIMEIRA CHAPA** (W103) — a família nasceu vazia na W100 à espera dela.
    Shape {
        key: "panel.model3d.add.star",
        family: Family::Plates,
        make: Make::Formula(a_star),
    },
    // ─────────────────────────── W119 ───────────────────────────
    // ⭐ **Nove portas para seis formas** — ver [`make`]: a seta dupla e as três do anel são a mesma
    // primitiva com outros números, e é a PORTA que o artista procura.
    Shape {
        key: "panel.model3d.add.arrow",
        family: Family::Signs,
        make: Make::Formula(an_arrow),
    },
    Shape {
        key: "panel.model3d.add.double_arrow",
        family: Family::Signs,
        make: Make::Formula(a_double_arrow),
    },
    Shape {
        key: "panel.model3d.add.bent_arrow",
        family: Family::Signs,
        make: Make::Formula(a_bent_arrow),
    },
    Shape {
        key: "panel.model3d.add.chevron",
        family: Family::Signs,
        make: Make::Formula(a_chevron),
    },
    Shape {
        key: "panel.model3d.add.rhombus",
        family: Family::Plates,
        make: Make::Formula(a_rhombus),
    },
    Shape {
        key: "panel.model3d.add.circle_segment",
        family: Family::Plates,
        make: Make::Formula(a_circle_segment),
    },
    Shape {
        key: "panel.model3d.add.tube",
        family: Family::Rings,
        make: Make::Formula(a_tube),
    },
    Shape {
        key: "panel.model3d.add.washer",
        family: Family::Rings,
        make: Make::Formula(a_washer),
    },
    Shape {
        key: "panel.model3d.add.ring_arc",
        family: Family::Rings,
        make: Make::Formula(a_ring_arc),
    },
    // ─────────────────────────── W120 ───────────────────────────
    // ⭐ **Dez portas para nove formas** — a nuvem e o balão de pensamento são a mesma primitiva
    // com a fieira ligada ou não, e é a PORTA que o artista procura.
    Shape {
        key: "panel.model3d.add.speech_rect",
        family: Family::Signs,
        make: Make::Formula(a_speech_rect),
    },
    Shape {
        key: "panel.model3d.add.speech_oval",
        family: Family::Signs,
        make: Make::Formula(a_speech_oval),
    },
    Shape {
        key: "panel.model3d.add.thought",
        family: Family::Signs,
        make: Make::Formula(a_thought),
    },
    Shape {
        key: "panel.model3d.add.cloud",
        family: Family::Signs,
        make: Make::Formula(a_cloud),
    },
    Shape {
        key: "panel.model3d.add.bolt",
        family: Family::Signs,
        make: Make::Formula(a_bolt),
    },
    Shape {
        key: "panel.model3d.add.shield",
        family: Family::Signs,
        make: Make::Formula(a_shield),
    },
    Shape {
        key: "panel.model3d.add.tag",
        family: Family::Signs,
        make: Make::Formula(a_tag),
    },
    Shape {
        key: "panel.model3d.add.check",
        family: Family::Signs,
        make: Make::Formula(a_check),
    },
    Shape {
        key: "panel.model3d.add.banner",
        family: Family::Signs,
        make: Make::Formula(a_banner),
    },
    Shape {
        key: "panel.model3d.add.brace",
        family: Family::Signs,
        make: Make::Formula(a_brace),
    },
    // ─────────────────────────── W122 — o fluxograma ───────────────────────────
    // ⚠️ **Na `Plates`, e não numa família nova** — ver [`make_signs`] para a razão medida.
    Shape {
        key: "panel.model3d.add.parallelogram",
        family: Family::Plates,
        make: Make::Formula(a_parallelogram),
    },
    Shape {
        key: "panel.model3d.add.delay",
        family: Family::Plates,
        make: Make::Formula(a_delay),
    },
    Shape {
        key: "panel.model3d.add.display",
        family: Family::Plates,
        make: Make::Formula(a_display),
    },
    Shape {
        key: "panel.model3d.add.off_page",
        family: Family::Plates,
        make: Make::Formula(an_off_page),
    },
    // ─────────────────────────── W123 ───────────────────────────
    // ⭐⭐ **As duas que o plano dava por «tem de ser desenhada»** — e a recusa respondia a outra
    // pergunta: o que não é fechado é a distância EXACTA, e a marcha só pede um minorante.
    Shape {
        key: "panel.model3d.add.spiral",
        family: Family::Plates,
        make: Make::Formula(a_spiral),
    },
    Shape {
        key: "panel.model3d.add.document",
        family: Family::Plates,
        make: Make::Formula(a_document),
    },
    // ─────────────────────────── W124 ───────────────────────────
    // ⭐⭐ **A MOLA vai para `Rings`** — ela tem furo no meio por construção, que é o que a família
    // diz que é —, e a **REDE** para `Blocks`, porque a caixa É a peça.
    Shape {
        key: "panel.model3d.add.helix",
        family: Family::Rings,
        make: Make::Formula(a_helix),
    },
    Shape {
        key: "panel.model3d.add.gyroid",
        family: Family::Blocks,
        make: Make::Formula(a_gyroid),
    },
    // ─────────────────────────── W125 ───────────────────────────
    Shape {
        key: "panel.model3d.add.rounded_cylinder",
        family: Family::Round,
        make: Make::Formula(a_rounded_cylinder),
    },
    // ─────────────────────────── W127 ───────────────────────────
    // ⭐ **Vai para `Blocks`** — no ponto em que nasce ela é um bloco de cantos moles, e é como
    // bloco que o artista a procura. (De `1` a `2` ela é redonda; a família inteira não cabe numa
    // gaveta, e a gaveta é onde ela NASCE.)
    Shape {
        key: "panel.model3d.add.superquadric",
        family: Family::Blocks,
        make: Make::Formula(a_superquadric),
    },
    // ─────────────────────────── W128 ───────────────────────────
    // ⭐ **Vai para `Round`** — ela nasce estrela do mar, e é entre as redondas que o artista a
    // procura.
    Shape {
        key: "panel.model3d.add.superformula",
        family: Family::Round,
        make: Make::Formula(a_superformula),
    },
    // ─────────────────────────── W131 ───────────────────────────
    Shape {
        key: "panel.model3d.add.triangle",
        family: Family::Plates,
        make: Make::Formula(a_triangle),
    },
    // ─────────────────────────── W132 ───────────────────────────
    // ⭐ **Vai para `Plates`, ao lado do triângulo** — é a mesma pergunta («que contorno tem esta
    // chapa?») com a contagem aberta.
    // ─────────────────────────── W134 ───────────────────────────
    // ⭐ **Vai para `Rings`** — a família do que tem furo no meio por construção, que é onde um
    // artista procura um nó depois de olhar para o toro.
    Shape {
        key: "panel.model3d.add.torus_knot",
        family: Family::Rings,
        make: Make::Formula(a_torus_knot),
    },
    // ⭐⭐ **W135: as DUAS vão para `Round`** — são um cilindro antes de serem outra coisa, e é aí que
    // um artista procura um parafuso. ⚠️ **Uma primitiva, duas portas** (a lei do tubo/anilha).
    Shape {
        key: "panel.model3d.add.thread",
        family: Family::Round,
        make: Make::Formula(a_thread),
    },
    Shape {
        key: "panel.model3d.add.knurl",
        family: Family::Round,
        make: Make::Formula(a_knurl),
    },
    // ⭐⭐⭐ **W138: as DUAS COMPOSTAS** — as únicas entradas do catálogo que devolvem uma ÁRVORE em
    // vez de uma primitiva. ⚠️ **As duas vão para `Round`**: uma bola com uma cratera e uma lente
    // são redondas antes de serem outra coisa, e é aí que um artista procura.
    Shape {
        key: "panel.model3d.add.cratered_sphere",
        family: Family::Round,
        make: Make::Composed(a_cratered_sphere),
    },
    Shape {
        key: "panel.model3d.add.lens",
        family: Family::Round,
        make: Make::Composed(a_lens),
    },
    // ─────────────────────────── W136 ───────────────────────────
    // ⭐⭐ **A BEZIER e a PARÁBOLA são a MESMA primitiva** — a segunda é a primeira com os três
    // pontos no sítio que a torna `y = k·x²`, medido a `5,5e-17`. Duas portas, uma fórmula.
    Shape {
        key: "panel.model3d.add.bezier",
        family: Family::Plates,
        make: Make::Formula(a_bezier),
    },
    Shape {
        key: "panel.model3d.add.parabola",
        family: Family::Plates,
        make: Make::Formula(a_parabola),
    },
    // ⭐ **A ONDA vai para `Rings`** — ela tem furo no meio por construção.
    Shape {
        key: "panel.model3d.add.circle_wave",
        family: Family::Rings,
        make: Make::Formula(a_circle_wave),
    },
    Shape {
        key: "panel.model3d.add.polygon",
        family: Family::Plates,
        make: Make::Formula(a_polygon),
    },
    Shape {
        key: "panel.model3d.add.torus",
        family: Family::Rings,
        make: Make::Formula(a_torus),
    },
    Shape {
        key: "panel.model3d.add.torus_arc",
        family: Family::Rings,
        make: Make::Formula(a_torus_arc),
    },
    // ⭐⭐ **AS FORMAS DE PERFIL** (W53) — o desenho do editor vetorial vira peça.
    Shape {
        key: "panel.model3d.add.extrude",
        family: Family::Drawn,
        make: Make::Extrude,
    },
    Shape {
        key: "panel.model3d.add.revolve",
        family: Family::Drawn,
        make: Make::Revolve,
    },
    Shape {
        key: "panel.model3d.add.sculpt",
        family: Family::Imported,
        make: Make::Sculpt,
    },
    Shape {
        key: "panel.model3d.add.sculpt_scene",
        family: Family::Imported,
        make: Make::SculptScene,
    },
];
