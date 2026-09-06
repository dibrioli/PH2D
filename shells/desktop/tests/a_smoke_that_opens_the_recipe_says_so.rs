//! ⛔⛔⛔ **UMA CENA QUE ABRE A RECEITA DIZ QUE A ABRIU — E VICE-VERSA** (ADR-0164, 2026-09-06).
//!
//! # O defeito que este censo existe para não repetir
//!
//! Uma receita **não é uma linha da cena**: desde 2026-08-30 a Hierarquia retira da lista tudo o
//! que o `off_canvas::is_unedited_recipe` acusa, e o `MasterRoot` também é `MasterPiece` — logo a
//! receita **inteira** sai. A marca que a traz de volta é derivada da **selecção**.
//!
//! ⇒ duas cenas de smoke desta linha ficaram a mandar o dono agir sobre linhas que **não existem**:
//! a `=2` desde 30/08 (o texto dela foi escrito a 27/08, quando era verdade) e a `=7` no dia em que
//! nasceu. *É a lei do §5.0 à letra: quando um comportamento muda, a cena que o demonstra é o
//! último sítio a ser lembrado e o primeiro que o dono lê.*
//!
//! # ⚠️ Por que o censo é de DOIS SENTIDOS
//!
//! Abrir sem dizer é um robô a mais na tela que ninguém explicou — lê-se como defeito. Dizer sem
//! abrir é o passo impossível de volta. *Nenhuma das duas metades sozinha descreve o par.*
//!
//! ⛔ Ele descasca comentários antes de varrer: um censo textual que não separa prosa de código
//! mente nos dois sentidos, e esta linha já o pagou três vezes.

use std::path::Path;

/// As cenas desta família, e o ficheiro de cada uma.
const SCENES: &[&str] = &[
    "instance_smoke.rs",
    "instance_removed_smoke.rs",
    "instance_added_smoke.rs",
    "instance_move_smoke.rs",
];

/// A frase que o dono lê quando a cena abriu a receita por ele.
const SAYS_OPEN: &str = "ja' esta' ABERT";
/// A chamada que de facto a abre — a marca `MasterEditing` é derivada da selecção.
const OPENS: &str = "replace_selection(Some(master_bits))";

fn source(rel: &str) -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}

fn code_only(body: &str) -> String {
    body.lines()
        .map(|l| match l.find("//") {
            Some(i) => &l[..i],
            None => l,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// ⭐⭐⭐ **Quem abre diz, e quem diz abre.**
///
/// **Mutação que deve sangrar:** tirar o `replace_selection` de qualquer uma das duas cenas que o
/// têm, ou tirar-lhes a frase.
#[test]
fn a_scene_that_opens_the_recipe_is_exactly_a_scene_that_says_so() {
    for rel in SCENES {
        let body = source(rel);
        let code = code_only(&body);
        let opens = code.contains(OPENS);
        // ⚠️ A frase vive DENTRO de um `println!`, logo no código — não em prosa.
        let says = code.contains(SAYS_OPEN);
        assert_eq!(
            opens, says,
            "{rel}: abre a receita = {opens}, diz que a abriu = {says}. Abrir sem dizer poe um \
             objecto a mais na tela que ninguem explicou; dizer sem abrir manda o dono procurar \
             uma linha que nao esta' na lista."
        );
    }
}

/// ⛔ **E pelo menos UMA cena tem de exercer cada lado** — senão o censo acima é satisfeito por
/// quatro cenas que não fazem nem dizem nada, e mede zero.
///
/// *Um censo cuja população pode ser vazia é um censo que passa por não ter sujeito.*
#[test]
fn the_census_has_a_scene_on_each_side() {
    let opens = SCENES
        .iter()
        .filter(|rel| code_only(&source(rel)).contains(OPENS))
        .count();
    assert!(
        opens > 0 && opens < SCENES.len(),
        "{opens} de {} cenas abrem a receita — o censo perdeu um dos dois lados e passa a valer \
         nada",
        SCENES.len()
    );
}
