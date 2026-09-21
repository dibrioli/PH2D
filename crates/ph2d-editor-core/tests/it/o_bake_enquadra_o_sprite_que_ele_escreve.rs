//! ⭐⭐⭐⭐ **O QUE SE VÊ É O QUE SE ASSA — a metade que vive na SHELL.**
//!
//! Report do dono, 2026-09-21 (com foto): *«o Bake não é feito projetando o objeto 3d exatamente
//! como o posiciono sobre a sprite e tem perspectiva, posição e escala diferente do que eu
//! coloquei»* e *«depois do Bake e antes de apertar o D … o bake 3d recebe zoom e fica como na
//! imagem: um fundo deslocado do objeto 3d»*. **Um defeito, dois relatos.**
//!
//! # O mecanismo, medido
//!
//! A porta de assar rasterizava a forma no alvo INTEIRO, logo a peça ocupava, dentro dos texels do
//! sprite, a mesma fracção que ocupava **da altura do VIEWPORT** — e o sprite é um rectângulo
//! *dentro* dele. O erro de escala é exactamente `altura da vista ÷ altura do sprite no ecrã`, mais
//! o desvio entre os dois centros. A LEI da cura (o frustum fora-de-eixo) tem os gates dela na
//! `ph2d-mesh-render` (`view_region_tests`), com o CONTROLO que mede o defeito a `> 100 px`.
//!
//! # ⛔⛔ Por que este gate é de TEXTO
//!
//! A regra 2 da W2 parte a resposta em duas crates que não se conhecem: **a shell** sabe onde um
//! sprite aterra no ecrã (ela tem a câmera 2D, a janela e o afim partilhado) e **a família** sabe
//! onde a vista 3D está e como recortar o frustum. O elo entre as duas é um argumento do
//! `bake::drain`, e o sítio onde ele é montado (`fase_sculpt3d_bake`) pede um `HeroScreen`, um
//! `GpuContext` e um mundo — *não é alcançável de um teste*.
//!
//! ⚠️ **Um motor com a lei certa e a shell a não a ligar lê-se como um motor sem a lei** — a 5.ª
//! ocorrência nesta casa, e a razão de este ficheiro existir. As duas pontas têm gates de valor; o
//! meio tem este.
//!
//! # ⚠️ E por que ele vive nesta crate e não na SHELL
//!
//! **Pelo tecto de LOC dela**, que é a catraca `the_shell_only_shrinks` — ele conta `shells/desktop`
//! INTEIRO, `tests/` incluído, e esta wave levou-o `177` linhas acima. ⇒ a cura é CORTE, nunca
//! subir o número, e o corte mais honesto foi o próprio gate: o irmão que mede a shell
//! (`architecture_the_shell_only_shrinks`) já vive aqui e já a lê por caminho.
//!
//! ⛔ **O preço está registado e é conhecido:** *«um gate de família que lê a shell pelo caminho
//! escapa a quem move o código»*. Aqui ele erra para o lado BARULHENTO — o `fonte()` faz `panic!`
//! com o caminho dentro se o ficheiro mudar de sítio —, que é a única forma desta família que não
//! passa em silêncio.

use std::path::{Path, PathBuf};

/// A fase do quadro que assa a forma no sprite, **relativa à raiz da shell**.
const FASE: &str = "src/render_loop/fase_sculpt3d_bake.rs";

/// A raiz da shell, subindo da crate onde este gate vive — a MESMA travessia do
/// `architecture_the_shell_only_shrinks`, que é o irmão com o mesmo problema.
fn shell_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crates/<x>/ tem dois pais")
        .join("shells/desktop")
}

/// A porta partilhada que devolve onde a ARTE de um sprite aterra na janela.
///
/// ⚠️ Ela é da crate-folha [`ph2d_sprite_screen`] e não desta shell, de propósito: o afim
/// `imagem-px → ecrã-px` já tinha **quatro** consumidores antes desta wave, e uma quinta cópia da
/// aritmética divergiria no dia em que uma folha desdobrada ou uma rotação mudassem de lei.
const PORTA: &str = "ph2d_sprite_screen::rect_no_ecra(";

fn fonte(rel: &str) -> String {
    let path: PathBuf = shell_dir().join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path:?}: {e}"))
}

/// O corpo sem comentários de linha — este ficheiro **cita** os dois nomes na prosa acima, e um
/// comentário não liga fio nenhum. (A mesma cerca do `a_verb_that_costs_precision_says_so`, e pelo
/// mesmo motivo medido: uma régua textual lê o doc-comment que EXPLICA a cura e acusa-o de ser o
/// defeito.)
fn corpo(src: &str) -> String {
    src.lines()
        .map(|l| l.split("//").next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n")
}

/// ⭐⭐⭐ **A shell MEDE onde o sprite está e ENTREGA isso a quem assa.**
///
/// As duas metades, porque cada uma sozinha mente: medir sem entregar é um número calculado e
/// deitado fora (o `drain` cairia no `None`, que é a vista inteira — *o defeito do report, com o
/// cálculo feito*), e entregar sem medir não compila.
#[test]
fn a_shell_mede_onde_o_sprite_esta_e_entrega_isso_a_quem_assa() {
    let src = fonte(FASE);
    let body = corpo(&src);
    assert!(
        body.contains(PORTA),
        "{FASE} deixou de perguntar `{PORTA}` — sem isso o bake volta a escrever o alvo INTEIRO e \
         o assado sai deslocado e com a escala errada (report do dono, 2026-09-21)"
    );
    assert!(
        body.contains("rect_do_sprite"),
        "{FASE} mede o rectangulo e nao o nomeia — ele tem de chegar ao `bake::drain`"
    );
    // ⚠️ **O `drain` tem de o RECEBER**, e a agulha é a chamada com o argumento dentro: procurar os
    // dois nomes soltos deixaria passar um `rect_do_sprite` calculado e um `drain` que continua a
    // assar a vista inteira — o defeito exacto que este gate existe para impedir.
    let chamada = body
        .split_once("bake::drain(")
        .map(|(_, resto)| resto)
        .unwrap_or_else(|| {
            panic!("{FASE} ja' nao chama `bake::drain(` — quem assa mudou de sitio")
        });
    let ate_ao_fecho = chamada.split(") {").next().unwrap_or(chamada);
    assert!(
        ate_ao_fecho.contains("rect_do_sprite"),
        "{FASE}: o rectangulo do sprite NAO e' passado ao `bake::drain` — ele e' medido e deitado \
         fora, e o assado volta a sair deslocado"
    );
}

/// ⭐⭐ **O facto é colhido ANTES do gesto, e não a meio dele.**
///
/// ⚠️ Duas razões, e as duas são load-bearing: o `bake::drain` leva o mundo por `&mut`, logo ler o
/// `Transform` lá dentro nem compila; e *um facto colhido antes é também um facto que o gesto não
/// pode mudar debaixo de si próprio*. A vizinha `holds_sixteen_bit` já obedece à mesma lei, pelo
/// mesmo motivo escrito no ficheiro.
#[test]
fn o_rectangulo_e_medido_antes_de_o_bake_correr() {
    let body = corpo(&fonte(FASE));
    let (antes, _) = body
        .split_once("bake::drain(")
        .unwrap_or_else(|| panic!("{FASE} ja' nao chama `bake::drain(`"));
    assert!(
        antes.contains(PORTA),
        "{FASE}: o rectangulo do sprite e' medido DEPOIS de o bake comecar — o mundo ja' esta' \
         emprestado, e o facto deixa de ser o do momento do gesto"
    );
}

/// ⛔ **E o CONTROLO da varredura:** se o ficheiro deixar de existir ou ficar vazio, os dois gates
/// acima passam a medir o vazio. *Um censo sem piso de população é `true` trivialmente.*
#[test]
fn a_fase_que_este_gate_mede_ainda_existe() {
    let src = fonte(FASE);
    assert!(
        src.len() > 2_000,
        "{FASE} tem {} bytes — a fase mudou de sitio e estes gates medem o vazio",
        src.len()
    );
    assert!(
        corpo(&src).contains("fn fase_sculpt3d_bake"),
        "{FASE} ja' nao declara a fase — os gates acima perderam o sujeito"
    );
}
