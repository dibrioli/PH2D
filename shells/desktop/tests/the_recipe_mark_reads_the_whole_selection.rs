//! **Arch-gate: o modo de edição de receita lê a selecção INTEIRA, e a cópia nova fica visível.**
//!
//! Os dois defeitos que este ficheiro guarda vieram da auditoria multiagêntica de 2026-08-27
//! (`docs/Components/22_auditoria_das_instancias_2026-08-27.md`), e os dois vivem na **fiação** —
//! não numa função que um teste de unidade alcance.
//!
//! * **§1.6** — `master_editing::mark` recebia `hero.gizmo.selection`, o **primário**. Shift ou
//!   Ctrl-clicar a linha de uma receita põe-na em `extra_selection` e **não** mexe no primário
//!   (`add_to_selection`), e o atalho `preserves_multi` do ramo `Replace` faz o mesmo. ⇒ a linha
//!   ficava realçada na Hierarquia e o canvas continuava vazio — *«cliquei nela e não aconteceu
//!   nada»*. O gate de unidade não o via porque a **assinatura** (`Option<u64>`) tornava o caso
//!   inexprimível: *uma função que só aceita um bits nunca pode medir o segundo*.
//! * **§1.4 / §1.2** — a row *Duplicate* tem três ramos no mesmo `if`. O de MODELAGEM selecciona a
//!   cópia; o VETORIAL desloca-a um degrau de tela; o genérico — sprites, grupos, instâncias e
//!   **receitas** — não fazia nem uma coisa nem outra. Duplicar deixava a cópia exactamente em
//!   cima da fonte, e duplicar uma RECEITA deixava-a invisível (uma cópia de mestre é outro
//!   mestre, e um mestre só desenha enquanto está seleccionado) com um toast de sucesso por cima.
//!
//! # Porque é arch-gate
//!
//! As duas asserções são sobre o que o **chamador** passa, e o chamador precisa de um `HeroScreen`
//! com janela e câmara. A lei de dentro tem gates de unidade próprios
//! (`render_loop::master_editing::tests` e `instantiate::tests`); o que nenhum deles alcança é
//! *«o produto de facto liga isto assim»*. [[feedback_a_rule_only_exists_if_it_is_on_the_path_of_who_executes_it]]

use std::fs;

fn src(name: &str) -> String {
    fs::read_to_string(format!("{}/src/{name}", env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|e| panic!("{name}: {e}"))
}

/// O passe que acende a receita vê os EXTRAS, e não só o primário.
#[test]
fn the_recipe_mark_is_fed_the_extra_selection_too() {
    let s = src("render_loop/mod.rs");
    let Some(at) = s.find("master_editing::mark(") else {
        panic!("o passe que acende a receita mudou de nome — reancore este gate");
    };
    // A janela acaba na própria chamada que ela julga (o `);` do argumento), nunca numa linha
    // vizinha: um gate ancorado no que ele não julga é um proxy que expira.
    let block = &s[at..];
    let end = block
        .find("sim_extract::run(")
        .expect("o `mark` deixou de correr imediatamente antes do extract — a ORDEM era a lei");
    let block = &block[..end];
    assert!(
        block.contains("extra_selection"),
        "o modo de edicao de receita voltou a ler so' a selecao primaria. Shift-clicar a linha de \
         uma receita realca-a na Hierarquia e deixa o canvas vazio, que e' o report «cliquei nela \
         e nao aconteceu nada».\n{block}"
    );
}

/// A row *Duplicate* devolve uma cópia que o artista VÊ: deslocada e seleccionada.
#[test]
fn the_duplicate_row_offsets_the_copy_and_puts_the_gizmo_on_it() {
    let s = src("render_loop/hierarchy_duplicate.rs");
    let Some(at) = s.find("crate::instantiate::duplicate_subtree(") else {
        panic!("a row Duplicate deixou de chamar a copia profunda — reancore este gate");
    };
    // Do início da chamada até ao toast que a fecha: é exactamente o ramo genérico.
    let block = &s[at..];
    let end = block
        .find("Duplicated entity")
        .expect("o ramo generico do Duplicate deixou de responder ao artista");
    let block = &block[..end];
    // ⚠️ O degrau é calculado ANTES do `if` (os dois ramos partilham-no), então a janela do ramo
    // genérico vê o `dx`/`dy` e não a derivação — ela é afirmada logo abaixo, sobre o ficheiro.
    assert!(
        block.contains("dx as f32") && block.contains("dy as f32"),
        "o ramo generico do Duplicate deixou de PASSAR o degrau a' copia. Sem ele ela aterra \
         exactamente em cima da fonte: o toast diz «Duplicated entity» e a tela nao muda.\n{block}"
    );
    assert!(
        s.contains("screen_offset_world") && s.contains("PASTE_OFFSET_PX"),
        "o degrau deixou de vir da CAMARA. Um numero de MUNDO fixo e' invisivel com o zoom \
         afastado e atira a copia para fora do ecra com o zoom perto — ja' aconteceu uma vez."
    );
    assert!(
        block.contains("replace_selection"),
        "o ramo generico do Duplicate deixou de seleccionar a copia. Alem de o artista ter de a \
         cacar na lista, uma copia de RECEITA so' desenha enquanto esta' seleccionada — sem isto \
         ela nasce invisivel com um toast de sucesso.\n{block}"
    );
}

/// ⚠️ **O texto de cada cena de smoke nomeia o gesto que ACENDE a receita** — auditoria §1.7.
///
/// A parte de código está gateada em `master_editing::tests::the_smoke_scene_shows_its_recipe_
/// only_after_the_row_is_clicked` (a receita não tem um pixel no quadro 0). O que nenhum gate de
/// unidade alcança são as **strings**: elas descreviam coordenadas vazias — *«receita lá em cima»*,
/// *«à ESQUERDA, longe das cópias»* — e o artista lia isso, olhava para o canvas e reportava «o
/// mestre ficou invisível». *Um instrumento que descreve um estado que ele próprio não produz é
/// pior que instrumento nenhum.*
#[test]
fn each_smoke_scene_tells_the_artist_to_click_the_recipe_row() {
    let s = src("instance_smoke.rs");
    for scene in ["[instance smoke 1] PASSO 1", "[instance smoke 2] PASSO 1"] {
        let at = s
            .find(scene)
            .unwrap_or_else(|| panic!("a cena perdeu o PASSO 1: {scene:?}"));
        let step = &s[at..s[at..].find("println!").map_or(s.len(), |n| at + n)];
        assert!(
            step.contains("clique na linha"),
            "o PASSO 1 de {scene:?} deixou de mandar clicar na linha da receita. Sem esse gesto a \
             receita nao tem UM pixel no canvas, e o smoke entrega o report que existe para \
             evitar.\n{step}"
        );
    }
}

/// ⭐⭐⭐ **A FOTOGRAFIA reconcilia o documento primeiro** — report do Enio, 2026-08-27: *«as peças
/// apagadas voltaram sem pais e na posição (0,0) do mundo»*.
///
/// A reconciliação `path ⟺ entidade` corre CEDO no quadro e o *Delete* corre TARDE, então o quadro
/// em que uma forma vetorial é apagada termina com a entidade morta e o `VecPath` dela vivo. O
/// undo fotografava esse instante; ao repô-lo, a reconciliação seguinte **cunhava** uma entidade
/// para o path órfão — `Transform::default()`, sem `ChildOf`. Um objeto **sem pai na origem**.
///
/// # Porque é arch-gate
///
/// O gate de unidade (`undo_vec_ghost_tests`) mede a PROPRIEDADE com a reconciliação escrita nele.
/// O que ele não pode ver é se o **produto** a chama — e era exactamente isso que faltava: as
/// funções todas estavam certas, e ninguém as punha por esta ordem. *Um gate sobre uma lei não
/// prova que alguém a invoca.*
#[test]
fn the_photograph_reconciles_the_document_first() {
    // ⚠️⚠️ **O PAR, não um ficheiro.** A `App` que opera a fila mudou-se para o irmão
    // `undo_app.rs` na integração de 2026-09-04 (tecto de LOC estourado pela SOMA de duas
    // linhas), e todo gate que lia só `undo.rs` ficou a afirmar sobre o ficheiro errado — em
    // silêncio no dia seguinte, se a lei ainda lá estivesse. ⇒ *um gate que PARSEIA o fonte lê
    // a família inteira, nunca um nome de ficheiro.*
    let s = src("undo_app.rs");
    let Some(at) = s.find("fn capture_project(") else {
        panic!("a porta unica da fotografia mudou de nome — reancore este gate");
    };
    let block = &s[at..];
    let end = block
        .find("ProjectState::capture(")
        .expect("o `capture_project` deixou de tirar a fotografia");
    let block = &block[..end];
    assert!(
        block.contains("vec_entities::sync"),
        "a fotografia do undo (e do SAVE — e' a mesma porta) deixou de reconciliar o documento \
         com o mundo antes de fotografar. Um quadro que apaga uma forma vetorial termina \
         inconsistente, e repor esse instante cunha um objeto sem pai na origem.\n{block}"
    );
}
