//! **Uma cena de smoke roda no DEFAULT que shipa** (2026-08-08).
//!
//! # ⚠️ O defeito que este gate existe para não repetir
//!
//! Os GATES do player cinemático correm com o `spring_damping` **abaixo do
//! teto**, e por um motivo escrito: no default os dois defeitos que aquele modo
//! promete zerar já são zero nos DOIS modos, então um gate no default passaria
//! no controle e estaria a medir a coisa errada.
//!
//! A cena 101 copiou esse valor. O artista abriu o smoke e viu **um pula-pula**
//! (report do Enio, 2026-08-08: *"o ciano está com uma mola extremamente
//! exagerada"*) — e estava certo: o quique medido foi de **5913 mm**.
//!
//! ⚠️ **A lição não é *"0,25 é um valor mau"*** — sozinho ele dá `0,0 mm`. O
//! quique exigiu TRÊS condições ao mesmo tempo (rampa + berço fora do repouso +
//! amortecimento baixo), e a cena tinha as três. A lição é a fronteira: **um
//! número escolhido para uma FIXTURE não atravessa para a mão do artista** — lá
//! ele deixa de ser *"o valor que faz o fenômeno aparecer"* e passa a ser *"o que
//! este produto é"*.
//!
//! ⚠️ **Arch-gate porque o defeito é de PRODUTO, não de comportamento:** cada
//! número que a cena escreve é legítimo isolado, e nenhum teste de unidade tem
//! opinião sobre qual deles o artista devia ver primeiro.

use std::fs;

/// A cena que o report nomeou.
const SCENE: &str = "src/physics_smoke_kinematic.rs";

/// O CÓDIGO da cena — sem os comentários.
///
/// ⚠️ **A primeira versão deste scanner reprovou a própria prosa.** O aviso do
/// módulo da cena *explica* o defeito e por isso escreve `spring_damping`; um
/// scanner que lê o arquivo inteiro não distingue *"a cena faz isto"* de *"a
/// cena conta que já fez isto"* — e o preço de deixar assim seria a próxima
/// pessoa a apagar a explicação para o gate ficar verde.
fn scene_source() -> String {
    fs::read_to_string(format!("{}/{SCENE}", env!("CARGO_MANIFEST_DIR")))
        .expect("a cena 101 tem de existir")
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// O arquivo inteiro, comentários incluídos — para o que se afirma da MENSAGEM.
fn scene_text() -> String {
    fs::read_to_string(format!("{}/{SCENE}", env!("CARGO_MANIFEST_DIR")))
        .expect("a cena 101 tem de existir")
}

/// **O CONTROLE:** o scanner acha o arquivo e ele é a cena que se pensa.
///
/// ⚠️ Sem isto um caminho errado deixa a afirmação abaixo verde por vácuo — uma
/// busca negativa sobre um arquivo vazio encontra zero de tudo.
#[test]
fn the_scanner_reads_the_scene_it_claims_to_scan() {
    let src = scene_source();
    assert!(
        src.contains("PlatformPlayer {"),
        "a cena tem de autorar um player"
    );
    assert!(
        src.contains("PlayerMode::Kinematic"),
        "e o modo que ela existe para comparar"
    );
    // ⚠️ E o strip de comentários não pode ter comido o arquivo — sem isto a
    // busca negativa abaixo fica verde por vácuo.
    assert!(
        src.lines().count() > 40,
        "o strip de comentarios deixou {} linhas: o scanner esvaziou o arquivo",
        src.lines().count()
    );
}

/// **A cena não afina a perna para baixo do que o produto shipa.**
#[test]
fn the_kinematic_scene_does_not_detune_the_spring() {
    let src = scene_source();
    assert!(
        !src.contains("spring_damping"),
        "a cena 101 nao pode escrever `spring_damping`: o artista tem de ver o \
         default. Se um dia ela precisar do valor detunado para demonstrar \
         alguma coisa, o caminho e' PEDIR ao artista que o baixe pelo painel \
         (o passo 4 da propria cena), nao nascer com ele."
    );
}

/// **E ela avisa que no default os dois modos são quietos.**
///
/// ⚠️ Isto não é decoração: a versão anterior desta cena **vendia** a deriva de
/// rampa como a diferença visível, e a deriva no default é `0,0000 m` nos dois
/// modos. Uma cena que promete o que o produto não faz gasta o smoke do Enio a
/// procurar um defeito que não existe.
#[test]
fn the_kinematic_scene_says_what_the_default_actually_does() {
    let src = scene_text();
    assert!(
        src.contains("0,0000 m nos DOIS"),
        "a mensagem tem de dizer que a deriva de rampa e' zero nos dois modos"
    );
}
