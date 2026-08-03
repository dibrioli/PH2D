//! **Um gesto lê os fatos que o DESENHO derivou** — arch-gate sobre a costura que nenhum teste
//! de unidade alcança.
//!
//! # Por que este arquivo existe
//!
//! Os intervalos das molduras (`VecClipSpan`) e as poses do auto layout são resultado do passe de
//! LAYOUT, que roda no desenho. O hit-test monta o `VecViewState` dele **do zero** a cada evento
//! de ponteiro, e a porta que ele usava (`vec_entities::view_state`) só sabe o que a ÁRVORE diz —
//! escondido e travado. As duas listas chegavam VAZIAS, e o gesto decidia como se nenhuma moldura
//! existisse e nenhuma forma tivesse sido colocada.
//!
//! ⚠️ **Isto já aconteceu, e passou despercebido por um commit inteiro.** A demoção da moldura no
//! pick (para o clique pegar o filho) lê `view_state.clips`; os gates dela montam o `clips` À MÃO
//! e ficaram verdes, enquanto no produto a lista era vazia e a cura era **inerte**. Um gate de
//! unidade é cego à fiação da shell — este é o par dele.

use std::fs;

fn read(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// **Todo arquivo que APONTA usa a porta que funde as duas metades.**
///
/// ⚠️ **A isenção é por DERIVAÇÃO, não por lista:** *aponta* quem chama uma das portas de pick
/// (elas recebem um ponto de MUNDO e perguntam que forma está lá). Quem só quer saber *se pode
/// mexer nisto* — escondido, travado — está a fazer uma pergunta da ÁRVORE, e a porta bare é a
/// resposta certa. Uma lista de nomes exemptos apodrece no dia em que um arquivo novo aponta;
/// esta regra cobre-o sozinha.
const PICK_DOORS: [&str; 3] = [
    "pick_all_at_world(",
    "pick_in_world_rect(",
    "contains_world(",
];

#[test]
fn every_gesture_site_merges_the_derived_facts() {
    let mut checked = 0;
    for entry in walk("src") {
        let src = read(&entry);
        if !PICK_DOORS.iter().any(|d| src.contains(d)) {
            continue; // este arquivo nao aponta
        }
        if entry.contains("_tests.rs") {
            continue; // um gate monta o estado que ele quer julgar
        }
        checked += 1;
        assert_eq!(
            src.matches("vec_entities::view_state(").count(),
            0,
            "{entry} APONTA (chama uma porta de pick) e monta o VecViewState do ZERO — os \
             intervalos das molduras e as poses do layout chegam VAZIOS, e o clique decide como \
             se nenhuma moldura existisse. Use `view_state_for_pick`."
        );
    }
    assert!(
        checked >= 2,
        "a varredura nao achou os sitios que apontam ({checked}) — o gate esta' a medir nada"
    );
}

/// Todo `.rs` sob `dir`, recursivamente.
fn walk(dir: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_string()];
    while let Some(d) = stack.pop() {
        let Ok(rd) = fs::read_dir(&d) else { continue };
        for e in rd.flatten() {
            let p = e.path();
            let name = p.to_string_lossy().to_string();
            if p.is_dir() {
                stack.push(name);
            } else if name.ends_with(".rs") {
                out.push(name);
            }
        }
    }
    out
}

/// **E o desenho PUBLICA o que derivou.** Sem esta metade a porta de fusão funde com nada, e o
/// gate acima passaria sobre um produto igualmente quebrado.
#[test]
fn the_draw_pass_publishes_the_facts_it_derived() {
    let src = read("src/render_loop/mod.rs");
    for (what, needle) in [
        ("os intervalos das molduras", "self.vec_view_derived.clips"),
        ("as poses do auto layout", "self.vec_view_derived.poses"),
    ] {
        assert!(
            src.contains(needle),
            "o passe de desenho nao publica {what} — quem aponta nunca os vera'"
        );
    }
    // ⚠️ **A ORDEM é load-bearing**: publicar ANTES do passe de layout publicaria a tabela do
    // frame anterior, e a forma recém-colocada ficaria um frame inteira sem pose.
    let recook = src
        .find(".layout_live\n                .recook(")
        .or_else(|| src.find("layout_live\n                .recook("))
        .expect("o passe de layout");
    let publish = src
        .find("self.vec_view_derived.poses")
        .expect("a publicacao das poses");
    assert!(
        publish > recook,
        "as poses sao publicadas ANTES do passe que as produz — a tabela seria a do frame anterior"
    );
}

/// **O hit-test compõe a pose**, e não só a pose autorada.
///
/// ⚠️ O gate de comportamento vive no `vec_gizmo_view_hit_tests` (ele mede o PONTO que pega). Este
/// afirma a outra metade — que a composição existe no produto —, porque um `contains_path` que
/// aceite o `VecViewState` e não o use compila, passa em tudo o que não o exercita, e devolve o
/// clique ao lugar de origem.
#[test]
fn the_hit_test_composes_the_layout_pose() {
    let src = read("src/vec_gizmo_view.rs");
    assert!(
        src.contains("view_state.layout_pose(id)"),
        "o hit-test recebe o VecViewState e NAO pergunta a pose — o clique volta a procurar a \
         forma no lugar de onde ela saiu"
    );
}
