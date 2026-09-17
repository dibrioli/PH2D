//! **Um gesto lê os fatos que o DESENHO derivou** — a metade desta lei que vive na CRATE.
//!
//! ⚠️⚠️ **Este ficheiro nasceu na integração de 2026-09-17, e a razão é o modo de falha que ele
//! evita.** O gate original vive em `shells/desktop/tests/it/the_gesture_reads_what_the_frame_drew.rs`
//! e tem três metades; quando o cacho do gizmo veio para cá, **duas** coisas aconteceram:
//!
//! 1. O `the_hit_test_composes_the_layout_pose` lia `src/vec_gizmo_view.rs` **por caminho em tempo
//!    de execução** e passou a estoirar com `No such file or directory`. ⛔ Ele é o *gémeo em
//!    runtime* que o `HOWTO_partir_uma_familia_da_shell` §2 nomeia: **o `cargo check` é cego a ele**
//!    (a árvore compilou verde), e só CORRER o teste o acusa — foi o único vermelho de `16 116` na
//!    árvore combinada. ⇒ aqui ele usa **`include_str!`**, que falha a COMPILAR se o ficheiro se
//!    mudar outra vez. *A espécie barata é a que falha alto.*
//! 2. A varredura `every_gesture_site_merges_the_derived_facts` varre a `src` da SHELL. Os sítios
//!    que apontam eram **sete**; dois vieram para cá, e a varredura de lá passou a ver **cinco** —
//!    ⛔ **sem uma linha vermelha**, porque o piso dela é `2`. *Um censo que passa a varrer MENOS
//!    lê-se exactamente como «já não há mais nada»* (a armadilha §2.7 do HOWTO, aqui na variante
//!    que encolhe em vez de zerar). ⇒ a mesma lei passa a ser afirmada dos DOIS lados, cada um com
//!    o seu piso, e a soma dos dois pisos é a população que existia antes.

/// As portas de pick — quem chama uma delas *APONTA* (recebe um ponto de MUNDO e pergunta que
/// forma está lá). Quem só quer saber *se pode mexer nisto* está a fazer uma pergunta da ÁRVORE, e
/// a porta bare é a resposta certa — é por isso que a isenção é por DERIVAÇÃO e não por lista.
const PICK_DOORS: [&str; 3] = [
    "pick_all_at_world(",
    "pick_in_world_rect(",
    "contains_world(",
];

/// **Todo ficheiro DESTA CRATE que APONTA usa a porta que funde as duas metades.**
///
/// Medido em 2026-09-17: os que apontam são `vec_gizmo_pick.rs` e `envelope_gesture.rs`, e os dois
/// estão limpos. ⚠️ O `cut_line.rs` e o `bucket.rs` chamam o `view_state` bare e **não** são
/// acusados — e isso é o desenho, não um furo: eles não chamam porta de pick nenhuma, logo a
/// pergunta deles é da árvore.
#[test]
fn every_gesture_site_merges_the_derived_facts() {
    let mut checked = 0;
    for entry in walk("src") {
        let Ok(src) = std::fs::read_to_string(&entry) else {
            continue;
        };
        if !PICK_DOORS.iter().any(|d| src.contains(d)) {
            continue; // este ficheiro nao aponta
        }
        if entry.contains("_tests.rs") {
            continue; // um gate monta o estado que ele quer julgar
        }
        checked += 1;
        assert_eq!(
            src.matches("ph2d_vec_entities::entities::view_state(")
                .count(),
            0,
            "{entry} APONTA (chama uma porta de pick) e monta o VecViewState do ZERO — os \
             intervalos das molduras e as poses do layout chegam VAZIOS, e o clique decide como \
             se nenhuma moldura existisse. Use `view_state_for_pick`."
        );
    }
    // ⚠️ O piso é a POPULAÇÃO medida no dia da mudança, não um número confortável: se ele deixar
    // de casar, o gate acusa em vez de emudecer.
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
        let Ok(rd) = std::fs::read_dir(&d) else {
            continue;
        };
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

/// **O hit-test compõe a pose**, e não só a pose autorada.
///
/// ⚠️ O gate de comportamento vive no `vec_gizmo_view_hit_tests` (ele mede o PONTO que pega). Este
/// afirma a outra metade — que a composição existe no produto —, porque um `contains_path` que
/// aceite o `VecViewState` e não o use compila, passa em tudo o que não o exercita, e devolve o
/// clique ao lugar de origem.
///
/// ⚠️⚠️ **`include_str!` e nunca `read_to_string`**: a versão anterior lia o caminho em runtime e
/// sobreviveu ao `git mv` como um teste vermelho em vez de um erro de compilação.
#[test]
fn the_hit_test_composes_the_layout_pose() {
    let src = include_str!("../../src/vec_gizmo_view.rs");
    assert!(
        src.contains("view_state.layout_pose(id)"),
        "o hit-test recebe o VecViewState e NAO pergunta a pose — o clique volta a procurar a \
         forma no lugar de onde ela saiu"
    );
}
