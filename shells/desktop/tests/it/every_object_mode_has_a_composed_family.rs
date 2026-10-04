//! ⭐⭐ **Todo modo de criação tem uma família COMPOSTA no quadro** (spec/06 F2/F3).
//!
//! O quadro do modo (`ph2d_editor_core::screens::hero::mode_drive`) só conhece as famílias que a
//! shell lhe passa em `render_loop/fase_object_mode.rs`. Uma família que sai dessa lista compila e
//! passa em todo gate da crate dela — e o modo fica morto: o seletor não o oferece, o `Tab` não o
//! acha, o menu Add pede-o em vão. Nenhum teste de unidade vê a lista (ela precisa da `App`), por
//! isso este lê-a, e pergunta às famílias VERDADEIRAS o que declaram.

use ph2d_editor_core::object_mode::ObjectMode;
use ph2d_editor_core::screens::hero::mode_drive::ModeFamily;

const FASE: &str = "src/render_loop/fase_object_mode.rs";

/// `(o construtor no fonte, a variável que entra na lista)` de cada família.
const FAMILIAS: [(&str, &str); 5] = [
    ("ph2d_app_painter::paint_mode::Family", "&mut paint"),
    ("ph2d_app_sculpt3d::sculpt_mode::Family::new", "&mut sculpt"),
    ("ph2d_app_flip::flip_mode::Family::new", "&mut flip"),
    ("ph2d_app_field3d::model_mode::Family::new", "&mut model"),
    ("ph2d_app_vec::vector_mode::Family::new", "&mut vector"),
];

/// ⭐⭐ GATE — cada família é construída E entra na lista do quadro.
///
/// *Mutação que sangra:* tirar `&mut flip,` da lista (o Draw Mode morre com tudo verde).
#[test]
fn every_mode_family_is_in_the_frame_list() {
    let src = std::fs::read_to_string(FASE).expect("a fase do modo existe");
    let lista = src
        .split("let families")
        .nth(1)
        .and_then(|r| r.split("];").next())
        .expect("controlo: a lista `families` mudou de forma e este gate leria o vazio");
    for (construtor, entrada) in FAMILIAS {
        assert!(
            src.contains(construtor),
            "a família {construtor} não é construída"
        );
        assert!(
            lista.contains(entrada),
            "a família {construtor} é construída mas NÃO entra na lista do quadro ({entrada})"
        );
    }
}

/// ⭐⭐ GATE — as famílias da tabela declaram, juntas, TODO modo de criação do vocabulário, e
/// nenhum par (tipo, modo) duas vezes (o quadro abriria só a primeira).
///
/// *Mutação que sangra:* um modo novo em `ObjectMode` sem família, ou a do Flip a deixar de
/// declarar o Edit, ou a do Model a declarar o par de outra, ou a do vetor a não declarar o dela.
#[test]
fn the_composed_families_declare_every_creation_mode() {
    let mut sim = ph2d_ecs::SimWorld::new();
    let mut state = ph2d_app_flip::state::FlipState::default();
    let doc = ph2d_flip::FlipDoc::new();
    let paint = ph2d_app_painter::paint_mode::Family;
    let sculpt = ph2d_app_sculpt3d::sculpt_mode::Family::new(&mut sim, None);
    let flip = ph2d_app_flip::flip_mode::Family::new(&mut state, &doc);
    let model = ph2d_app_field3d::model_mode::Family::new(&mut sim, false);
    let mut vec_state = ph2d_app_vec::state::VecState::default();
    let vector = ph2d_app_vec::vector_mode::Family::new(&mut vec_state);
    let pairs: Vec<_> = [
        paint.modes(),
        sculpt.modes(),
        flip.modes(),
        model.modes(),
        vector.modes(),
    ]
    .concat();
    // ⭐ A tabela D6 (spec/06 §3.4) INTEIRA: o Edit do vetor some sem nenhum modo morrer (o Flip e
    // o Model também o declaram) — só os PARES o vêem.
    use ph2d_component_desc::ObjectKind as K;
    let d6 = [
        (K::Image, ObjectMode::Paint),
        (K::Sculpt3D, ObjectMode::Sculpt),
        (K::Sculpt3D, ObjectMode::Paint),
        (K::Flip, ObjectMode::Draw),
        (K::Flip, ObjectMode::Edit),
        (K::Model3D, ObjectMode::Edit),
        (K::Vector, ObjectMode::Edit),
    ];
    for p in d6 {
        assert!(
            pairs.contains(&p),
            "{p:?} (D6) não é declarado por família composta nenhuma"
        );
    }
    for (i, p) in pairs.iter().enumerate() {
        assert!(!pairs[..i].contains(p), "{p:?} declarado por duas famílias");
    }
    let declared: Vec<ObjectMode> = pairs.iter().map(|(_, m)| *m).collect();
    for m in ObjectMode::ALL {
        if m == ObjectMode::Object {
            continue;
        }
        assert!(
            declared.contains(&m),
            "{m:?} está no vocabulário e nenhuma família composta o declara — um modo morto"
        );
    }
}

/// ⭐⭐ GATE (spec/06 F3 ▸ Vector) — **as formas do Edit chegam à vista do quadro**: é a vista que
/// as ferramentas lêem (`is_pickable`) e que desenha os nós (`in_edit`). Sem esta linha o Edit do
/// vetor abre, o seletor diz Edit — e toda forma continua a agarrar-se e a mostrar nós.
///
/// *Mutação que sangra:* apagar a linha da fase.
#[test]
fn the_vector_edit_reaches_the_frame_view() {
    let src = std::fs::read_to_string("src/render_loop/fase_vector_view_and_drives.rs")
        .expect("a fase da vista vetorial existe");
    assert!(
        src.contains("vec_view.editing.clone_from(&self.vec.edit.paths)"),
        "a vista do quadro não recebe as formas do Edit"
    );
}
