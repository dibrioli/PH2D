//! ⭐⭐⭐ **NENHUMA PALAVRA QUE A ESCULTURA MOSTRA É ESCRITA NO FONTE** — o HR-15 na crate de família
//! do 3D.
//!
//! Medido em 2026-09-16: **20** textos que chegam ao ECRÃ — importar/exportar malhas e as **doze
//! recusas da retopologia**. Migrados para `ph2d-i18n/src/app_sculpt3d.rs`.
//!
//! ⛔⛔ **As doze recusas estavam em PORTUGUÊS**, e num app inglês — reescritas em inglês na
//! tabela, com a chave renomeada (um slug feito do texto velho é uma tradução que mente sobre si
//! mesma). ⚠️ Elas guardam a CURA dentro da frase (*FLATTEN the stack first*, *lower the Detail*),
//! e é isso que as separa de uma recusa muda.
//!
//! # ⚠️ Esta família FALA PELO TERMINAL, e isso é o idioma dela
//!
//! O grosso do que a escultura diz sai por `eprintln!` — o retrato da retopologia, o veredito de
//! cada tecla, as razões de um `Delete` recusado. Isso **não** é texto de interface: é a
//! instrumentação que o módulo usa desde que nasceu, e cada ficheiro está isento com o mecanismo.
//! ⛔ A fronteira é *quem lê*: o terminal é de quem bisseca; o ecrã é do artista.

use ph2d_label_census::gate::{self, Excecao, Isento};

const TABLE: &str = "crates/ph2d-i18n/src/app_sculpt3d.rs";

/// Um ficheiro isento inteiro, com o mecanismo — **todos são consola**.
const FORA: &[Isento] = &[
    (
        "export_assado.rs",
        "o NOME DO MATERIAL (`ph2d_<n>`) é um token DENTRO de um par de ficheiros — o `usemtl` do \
         `.obj` e o `newmtl` do `.mtl` têm de casar letra a letra, e quem os lê é outro programa. \
         ⛔ Traduzi-lo faria os dois ficheiros discordarem no dia em que alguém mudasse de língua, \
         e o destino abriria a peça SEM textura sem uma queixa — a mesma razão pela qual o aviso \
         da saída não nomeia a fileira `Paint Detail`: *um texto que atravessa a fronteira do \
         ficheiro não é texto de ecrã*",
    ),
    (
        "sonda_undo.rs",
        "a SONDA do undo (`PH2D_SCULPT3D_UNDO_PROBE`): as linhas `[probe-sculpt-undo]` e os nomes \
         dos passos do roteiro (`chip Plane`, `pista Height`, `campo Depth`) saem no TERMINAL de \
         quem corre a sonda — elas nomeiam o controlo que o passo aperta para quem lê o log, e \
         nenhuma chega ao ecrã do artista",
    ),
    (
        "history_retopo_global.rs",
        "a LINHA do relatório da retopologia (vértices, quads, aspecto, enviesamento) impressa no \
         terminal — a régua de quem mede o botão, com colunas que só fazem sentido lado a lado",
    ),
    (
        "panel.rs",
        "os vereditos de CONSOLA dos intents do painel (armou/desarmou, isolou, que nível ficou) — \
         o que o painel mostra são chips, e esses vêm do `panel_snapshot`",
    ),
    (
        "bake.rs",
        "as linhas `[sculpt3d]` do ASSAR no terminal (o que foi assado, em que sprite, e o que \
         fazer a seguir)",
    ),
    (
        "albedo.rs",
        "as DUAS razões de a matéria de um sprite não poder ser lida — elas são a cauda da MESMA \
         frase do `bake.rs` logo acima (`[sculpt3d] nao assou: {e}`), e vieram COM o código que as \
         escreve quando a lei da matéria ganhou o segundo leitor. ⚠️ **Uma isenção é propriedade \
         do CÓDIGO e não do sítio onde ele está** (`CLAUDE.md` §5.0): sem esta linha os dois \
         literais ficavam sem abrigo e liam-se como TEXTO NOVO, e a cura seria migrar metade de \
         uma frase cuja outra metade é consola",
    ),
    (
        "keys.rs",
        "o retrato de CONSOLA de cada tecla da escultura (o que ela fez, e a contagem depois)",
    ),
    (
        "keys_delete.rs",
        "as RAZÕES do `Delete` que não foi para a escultura, impressas com o guarda que a causou — \
         diagnóstico de quem caça um report",
    ),
    (
        "donation.rs",
        "os rótulos do interruptor A/B da doação (`BARRO`, `LUZ`, `DESLIGADA`), impressos no \
         terminal pela tecla que cicla o papel",
    ),
    (
        "keys_view.rs",
        "o veredito das quatro vistas (`QUATRO`/`UMA`), impresso no terminal pelo `Ctrl+Alt+Q`",
    ),
    (
        "doc.rs",
        "o `Display` do erro de um documento ilegível (bytes, versão) — a razão técnica que vai \
         para o log de quem carrega um ficheiro velho",
    ),
    (
        "scenes.rs",
        "o cabeçalho da FIXTURA que a cena `=9` imprime no terminal para quem a corre",
    ),
];

/// Nenhum literal-identificador sobra nesta crate — a lista existe para o dia em que sobrar.
const NOT_LANGUAGE: &[Excecao] = &[];

#[test]
fn every_word_this_family_shows_comes_from_the_string_table() {
    let (src, _) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let intrusos = gate::intrusos_fora_de(&src, NOT_LANGUAGE, FORA, gate::CENAS);
    assert!(
        intrusos.is_empty(),
        "texto com cara de língua escrito no fonte da escultura (HR-15):\n  {}\n\n\
         A cura é uma chave `app.sculpt3d.<ficheiro>.<frase>` em `{TABLE}` e um `tr(\"…\")` no sítio \
         (uma frase com peças do código: `tr_with`). ⚠️ Se o texto sai por `eprintln!` e não pelo \
         ecrã, a cura é uma linha em `FORA` **com o mecanismo** — e pense duas vezes: uma recusa \
         que só o terminal vê é o defeito que a caixa de saída (`Sculpt3dScene::fala`) curou.",
        intrusos.join("\n  ")
    );
}

/// ⭐ **A metade justa** — cada isenção ainda abriga o que nomeia.
#[test]
fn every_named_exemption_still_shelters_what_it_names() {
    let (src, _) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let mortas: Vec<String> = gate::excecoes_mortas(&src, NOT_LANGUAGE)
        .into_iter()
        .chain(gate::isentos_mortos(&src, FORA))
        .collect();
    assert!(
        mortas.is_empty(),
        "isenções mortas:\n  {}",
        mortas.join("\n  ")
    );
}

/// ⭐⭐ **As chaves existem dos dois lados.**
#[test]
fn every_key_of_this_family_exists_on_both_sides() {
    let (_, repo) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let c = gate::chaves(&repo, "app.sculpt3d.", &[TABLE]);
    assert!(
        c.declaradas >= 18 && c.usadas >= 18,
        "o censo achou {} declaradas e {} usadas — está a ler o sítio errado",
        c.declaradas,
        c.usadas
    );
    assert!(
        c.sem_traducao.is_empty(),
        "usadas e NÃO declaradas — o `tr` pinta o identificador cru:\n  {}",
        c.sem_traducao.join("\n  ")
    );
    assert!(
        c.orfas.is_empty(),
        "declaradas e ninguém as usa — apague-as:\n  {:?}",
        c.orfas
    );
}
