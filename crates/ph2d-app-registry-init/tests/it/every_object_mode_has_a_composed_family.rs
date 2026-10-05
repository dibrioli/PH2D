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

/// ⚠️ Os caminhos são da SHELL — este gate mora no registo das famílias (saiu da shell pela catraca
/// `the_shell_only_shrinks`, na rodada de 04/10), e o `cwd` do `cargo test` é o desta crate.
fn shell(rel: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("a raiz da workspace")
        .join("shells/desktop")
        .join(rel)
}

/// `(o construtor no fonte, a variável que entra na lista)` de cada família.
///
/// ⚠️ **2026-10-05: `5` -> `3`, delta -2** — o 3D sai do PH2D (ADR-0179): as famílias do Sculpt e
/// do Model deixam o quadro.
const FAMILIAS: [(&str, &str); 3] = [
    ("ph2d_app_painter::paint_mode::Family", "&mut paint"),
    ("ph2d_app_flip::flip_mode::Family::new", "&mut flip"),
    ("ph2d_app_vec::vector_mode::Family::new", "&mut vector"),
];

/// ⭐⭐ GATE — cada família é construída E entra na lista do quadro.
///
/// *Mutação que sangra:* tirar `&mut flip,` da lista (o Draw Mode morre com tudo verde).
#[test]
fn every_mode_family_is_in_the_frame_list() {
    let src = std::fs::read_to_string(shell(FASE)).expect("a fase do modo existe");
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
/// declarar o Edit, ou a do vetor a declarar o par do Flip, ou a não declarar o dela.
#[test]
fn the_composed_families_declare_every_creation_mode() {
    let mut sim = ph2d_ecs::SimWorld::new();
    let mut state = ph2d_app_flip::state::FlipState::default();
    let doc = ph2d_flip::FlipDoc::new();
    let paint = ph2d_app_painter::paint_mode::Family;
    let flip = ph2d_app_flip::flip_mode::Family::new(&mut state, &doc);
    let mut vec_state = ph2d_app_vec::state::VecState::default();
    let vector = ph2d_app_vec::vector_mode::Family::new(&mut vec_state, &mut sim);
    let pairs: Vec<_> = [paint.modes(), flip.modes(), vector.modes()].concat();
    // ⭐ A tabela D6 (spec/06 §3.4) INTEIRA: o Edit do vetor some sem nenhum modo morrer (o Flip
    // também o declara) — só os PARES o vêem. ⚠️ 2026-10-05: `7` -> `4` pares, delta -3 — o 3D
    // sai do PH2D (ADR-0179): (Sculpt3D, Sculpt), (Sculpt3D, Paint) e (Model3D, Edit).
    use ph2d_component_desc::ObjectKind as K;
    let d6 = [
        (K::Image, ObjectMode::Paint),
        (K::Flip, ObjectMode::Draw),
        (K::Flip, ObjectMode::Edit),
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
    let src = std::fs::read_to_string(shell("src/render_loop/fase_vector_view_and_drives.rs"))
        .expect("a fase da vista vetorial existe");
    assert!(
        src.contains("vec_view.editing = self.vec.edit.editing(&self.vec.entities)"),
        "a vista do quadro não recebe as formas do Edit"
    );
    // ⛔ E a vista do CLIQUE (`view_derived`, lida pelo pick do Select, do laço e do realce) tem de
    // a receber: sem esta linha o Select em Edit lia Object e subia da forma ao objecto inteiro —
    // report do dono, 04/10: «não consigo selecionar as formas vetoriais dentro do objeto».
    let recook = std::fs::read_to_string(shell("src/render_loop/fase_vector_layout_recook.rs"))
        .expect("a fase do recook existe");
    assert!(
        recook.contains("view_derived.editing.clone_from(&vec_view.editing)"),
        "a vista do clique não recebe o Edit do quadro"
    );
}

/// ⭐⭐ GATE (spec/06 F3 ▸ Vector, 2.ª volta) — **o *Add ▸ Vector Drawing* da shell só ARMA a
/// ferramenta e pede Object** (nada nasce: a 1.ª forma pede o Edit). Sem o pedido de Object, um Add
/// feito em Paint deixaria o Painter a disputar a mão com o vetor.
///
/// *Mutação que sangra:* apagar o `push` do pedido de Object, ou voltar a passar o `sim` ao `add`.
#[test]
fn the_vector_add_only_arms_the_tool_and_asks_for_object() {
    let src = std::fs::read_to_string(shell("src/render_loop/fase_object_add.rs"))
        .expect("a fase do Add existe");
    let arm = src
        .split("ph2d_app_vec::object_add::add(entry, &mut self.vec)")
        .nth(1)
        .expect("a entrada do vetor não arma a ferramenta");
    let braço = arm.split("} else").next().unwrap_or_default();
    assert!(
        braço.contains("ModeRequest::Enter(ObjectMode::Object)"),
        "o Add do vetor não pede Object"
    );
}

/// ⭐ GATE — **a cópia de uma forma na Hierarquia fica no pai da original** (escolha do dono, 04/10).
///
/// *Mutação que sangra:* apagar a chamada.
#[test]
fn the_hierarchy_duplicate_keeps_the_copy_beside_its_source() {
    let src = std::fs::read_to_string(shell("src/render_loop/hierarchy_duplicate.rs"))
        .expect("o ficheiro existe");
    assert!(src.contains("duplicate::place_beside("));
}
