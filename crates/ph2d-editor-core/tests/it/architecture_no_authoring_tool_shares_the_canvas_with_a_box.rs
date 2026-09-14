//! **Arch-gate: a caixa de objecto não divide o canvas com uma ferramenta que AUTORA** (ADR-0112).
//!
//! # A lei, e porque ela é uma CONDIÇÃO e não um `if` por família
//!
//! As alças do gizmo registam hit-rects, e os ramos de canvas do despacho exigem `on_canvas`
//! (= *nenhum painel e nenhum widget sob o cursor*). Logo uma caixa publicada sobre um canvas de
//! autoria **mata o gesto**, seja qual for a família do objecto que a publica.
//!
//! Isto foi descoberto duas vezes, e a segunda sem report: em 2026-09-13 a caixa de uma SPRITE
//! matou o posar de ossos (*«selecionar o osso não é mais possível»*), e em 2026-09-14 mediu-se que
//! a ferramenta **Flip** tinha o mesmo buraco — o gate dela (`flip_gizmo_on`) suprimia só a caixa do
//! objecto Flip, então uma sprite de referência seleccionada continuava a publicar a sua e a
//! apagava o desenho por cima dela. ⇒ hoje é **uma** condição (*nenhuma ferramenta autora*) numa
//! **uma** porta (`snapshots::publish_gizmo::build_view`), e os três `if` por família morreram.
//!
//! ⛔ **A saída NÃO é isentar o gizmo no hit-test** (o que o Painter faz, que tem porta própria com
//! excepção declarada): ali a caixa continua PINTADA e deixa de pegar — a alça morta que este repo
//! nomeia. O que tem de não existir é a CAIXA.
//!
//! # Porque textual, e porque AQUI
//!
//! A condição vive numa fase do `render_frame`, que exige janela, GPU e superfície: **nenhum teste
//! de unidade a alcança** (a porta que a consome tem gate próprio, `snapshots_object_gizmo_tests`).
//! É a forma do irmão `the_motion_path_is_offered_only_on_the_keys_tab`. Ele mora nesta crate, e
//! não em `shells/desktop/tests/it/`, pela razão MEDIDA que o `the_onion_speaks_the_clip_clock`
//! (vizinho) escreve: a folga da shell contra o `the_shell_only_shrinks` está em dezenas de linhas.

use std::path::{Path, PathBuf};

/// As ferramentas cujo ramo de canvas exige `on_canvas` — e que por isso **têm** de aparecer na
/// condição. ⚠️ Os ids são os que o `ToolId::new(..)` do shell escreve.
const AUTORAS: &[&str] = &["\"vector\"", "\"flip\""];

/// Os ramos do `on_mouse_input` que recebem o `on_canvas`. ⚠️ **É um CENSO, não uma lista de
/// conveniência:** um quarto ramo a receber aquele booleano é uma ferramenta nova a partilhar o
/// canvas, e ela tem de decidir se entra na condição acima. O gate acorda; a resposta é humana.
const RAMOS: &[&str] = &[
    "ramo_flip_premidos(",
    "ramo_select_modais(",
    "ramo_ferramenta_vetorial(",
];

fn shell_src() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crates/<x>/ tem dois pais")
        .join("shells/desktop/src")
}

/// ⚠️ A PROSA sai antes de a lei ser aplicada — um censo textual que não separa prosa de código
/// mente nos dois sentidos (medido no gate irmão, 2026-09-14).
fn sem_comentarios(src: &str) -> String {
    src.lines()
        .map(|l| l.split_once("//").map_or(l, |(antes, _)| antes))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn the_object_box_condition_names_every_authoring_tool() {
    let f = shell_src().join("render_loop/fase_snapshots_publish.rs");
    let src = sem_comentarios(&std::fs::read_to_string(&f).expect("a fase do publish"));
    assert!(
        src.contains("snapshots::publish("),
        "controlo positivo: {} deixou de chamar o `snapshots::publish` — este gate passou a medir o \
         nada",
        f.display()
    );
    for tool in AUTORAS {
        assert!(
            src.contains(&format!("ToolId::new({tool})")),
            "{} não nomeia a ferramenta {tool} na condição da caixa de objecto.\n\
             As alças do gizmo registam hit-rects e o ramo de canvas dela exige `on_canvas`: uma \
             caixa publicada ali mata o gesto de autoria, seja qual for a família do objecto \
             seleccionado (ADR-0112; medido em 13/09 nos ossos e em 14/09 no Flip).",
            f.display()
        );
    }
}

#[test]
fn a_fourth_branch_on_the_canvas_wakes_this_law() {
    let f = shell_src().join("input_dispatch.rs");
    let src = sem_comentarios(&std::fs::read_to_string(&f).expect("o despacho"));
    let recebem: Vec<&str> = RAMOS.iter().filter(|r| src.contains(*r)).copied().collect();
    assert_eq!(
        recebem.len(),
        RAMOS.len(),
        "um dos ramos que partilham o canvas mudou de nome: achei {recebem:?}"
    );
    // O censo: quantas chamadas passam o `on_canvas`. Um quarto ramo é uma ferramenta nova a
    // partilhar o canvas — ela entra na condição da caixa, ou declara por escrito porque não.
    let passam = src.matches("on_canvas)").count() + src.matches("on_canvas,").count();
    assert_eq!(
        passam,
        RAMOS.len(),
        "o `on_canvas` chega a {passam} ramos e a lei conhece {}.\n\
         Um ramo NOVO que o receba é uma ferramenta que passa a partilhar o canvas com a caixa de \
         objecto: ou o id dela entra na condição de `fase_snapshots_publish` (ver o gate irmão), ou \
         fica escrito aqui porque não precisa.",
        RAMOS.len()
    );
}
