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
//!
//! ⛔⛔⛔ **E ESSA FRONTEIRA FOI MEDIDA ERRADA UMA VEZ, com foto** (report do dono, 21/09): o
//! `bake.rs` estava isento como *«as linhas `[sculpt3d]` do ASSAR no TERMINAL»* — e a
//! `fase_sculpt3d_bake` faz `eprintln!(…)` **E** `toasts.push(…)` com a **MESMA** `String`.
//! *Ele é as duas coisas, e a isenção era metade da verdade.* O que o dono viu foi
//! `[sculpt3d] nao assou: this sprite is fully tra…` — um prefixo português colado a uma frase
//! inglesa, num aviso de ecrã.
//!
//! ⚠️ **E o `albedo.rs` saiu com ele, pela razão que a própria isenção dele escrevia:** ela
//! dizia-se *«a cauda da MESMA frase do `bake.rs`»* ⇒ **uma isenção que herda a premissa de outra
//! herda o erro dela**, e as duas caíram na mesma corrida.
//!
//! ⇒ as duas entradas SAÍRAM desta lista, e as frases vivem na tabela
//! (`app.sculpt3d.bake.*` e `app.sculpt3d.albedo.*`). O prefixo `[sculpt3d]` ficou onde ele é
//! verdade: no `eprintln!` da fase, e **fora** do que o artista lê.

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
        "alpha_pedido.rs",
        "o **nome por omissão do objecto** que virou padrão (`Sprite`), quando o sprite não tem \
         `Name` — é a excepção que o cabeçalho desta tabela já declara por escrito: *um nome que \
         entra num `Name` é identidade durável (`stable_name_id` fecha um hash sobre ele), e \
         traduzi-lo é decisão do dono*. ⚠️ **A isenção VIAJOU com o código em 21/09:** ela vivia na \
         lista da shell, e quando a lei do padrão se mudou para esta crate as duas metades \
         acusaram na mesma corrida — a órfã lá, o literal sem abrigo aqui. *Cada uma sozinha \
         mente: uma lê-se como «alguém apagou isto» e a outra como «alguém escreveu texto novo».*",
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
const NOT_LANGUAGE: &[Excecao] = &[
    (
        "vivo.rs",
        "ph2d-app-sculpt3d forma viva normal",
        "o RÓTULO DE DEPURAÇÃO de uma textura de wgpu (`TextureDescriptor::label`) — ele existe \
         para aparecer num depurador gráfico (RenderDoc, o validador) ao lado das outras texturas \
         desta árvore, e não há caminho por onde chegue a um pixel do ecrã",
    ),
    (
        "vivo.rs",
        "ph2d-app-sculpt3d forma viva oclusao",
        "o irmão do de cima, o segundo plano do G-buffer vivo — mesmo mecanismo",
    ),
];

// ⚠️ **A isenção é por FRASE e não por FICHEIRO, e a escolha é deliberada.** O idioma vizinho
// (`FORA`) isenta um ficheiro inteiro, e é o que a `donation.rs` usa — mas ali o que sai são as
// LINHAS de um relatório, uma família aberta. Aqui são **duas cordas nomeadas**, e um ficheiro
// isento seria um cheque em branco para o texto de interface que o `vivo.rs` ainda pode ganhar.
// ⭐ E a `every_named_exemption_still_shelters_what_it_names` conta as duas: no dia em que um
// rótulo mudar de texto, a isenção dele fica **órfã** e o gate diz.

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
