//! ⭐⭐⭐ **NENHUMA PALAVRA DO INSPECTOR É ESCRITA NO FONTE** — o HR-15, com a régua que vê o painel
//! INTEIRO.
//!
//! > `CLAUDE.md` §0.3: *«UI canônica: zero hex, zero `f32` literal de UI, **zero string
//! > hardcoded** — tudo via tokens / i18n (HR-15).»*
//!
//! # Por que este painel
//!
//! Era o maior painel fora da tabela depois do Painter: **582** literais com cara de língua pela
//! régua lexical, que passaram a **626** quando a própria régua se corrigiu duas vezes a meio da
//! migração (as palavras GRITADAS dos títulos dos cards · as frases com uma BARRA, `"Speed (m/s)"`).
//! O gate antigo do HR-15 via **11** deles — só os `.placeholder("…")`.
//!
//! # ⭐ As tabelas `const` guardam `TextKey`, nunca `&str`
//!
//! Metade do vocabulário do painel mora em tabelas `const` (os tipos de junta, os cards da §14, os
//! modos de mistura), onde o `tr` não compila. Guardada como `&str`, uma chave e um texto são o mesmo
//! tipo, e um consumidor que se esquecesse de traduzir pintaria `panel.inspector.joint.pin` com a
//! suíte verde. Como `ph2d_i18n::TextKey`, o esquecimento é erro de compilação — e foi o compilador
//! que achou os treze rótulos `(m/s)` que a régua ainda não via.
//!
//! # A régua é a da `ph2d-label-census`, e este gate é POR CRATE
//!
//! Uma catraca global com a dívida das outras crates dentro poria a próxima linha vermelha por causa
//! de um gate desta (`CLAUDE.md` §0.2). Este fala só deste painel, a ZERO, com as excepções nomeadas
//! e o mecanismo de cada uma.

use std::path::{Path, PathBuf};

use ph2d_label_census::{keys, language_literals};

const PREFIX: &str = "panel.inspector.";
/// ⚠️ **Duas tabelas, um vocabulário**: a §14 Platform Player mora num irmão por SECÇÃO
/// (`inspector_player.rs`), cortado pelo tecto de 700 linhas da workspace.
const TABLES: &[&str] = &[
    "crates/ph2d-i18n/src/inspector.rs",
    "crates/ph2d-i18n/src/inspector_player.rs",
    // ⭐ **A TERCEIRA, pela mesma lei do tecto**: as secções de JOGO (TOP-20 #9..#18) somam 151
    //    chaves, e escrevê-las na `inspector.rs` levava-a de `559` a mais de `900` — acima do
    //    tecto de 700 da workspace. O corte é por ASSUNTO, que é o que esse tecto pede.
    "crates/ph2d-i18n/src/inspector_game.rs",
    // ⭐ **A QUARTA, pela mesma lei:** as secções HEALTH e DAMAGE (plano 28, W3) somam as
    //    chaves de vinte e seis campos mais os avisos, e a `inspector_game.rs` já está perto do
    //    tecto. O corte é por ASSUNTO outra vez.
    "crates/ph2d-i18n/src/inspector_vida.rs",
];

/// ⭐ As excepções, **com o mecanismo** — `(ficheiro relativo a src/, texto exacto, porquê)`.
const NOT_LANGUAGE: &[(&str, &str, &str)] = &[
    // ⭐⭐ As duas formas com que um valor de SCRIPT se lê numa linha de órfão. Elas são o
    // CONSTRUTOR que o artista escreveu no ficheiro `.luau` dele (`ph2d.vec2(1, 2)`), não língua
    // desta casa: traduzi-las mostrava-lhe uma forma que ele não pode escrever, e um órfão existe
    // exactamente para ele o reconhecer no ficheiro. É a MESMA isenção que o rótulo de uma opção
    // de enum já carrega, e que o cabeçalho da `opcoes_da_linha` declara por escrito.
    (
        "sections/script.rs",
        "vec2({}, {})",
        "o construtor que o artista escreveu no .luau dele; traduzi-lo mostra uma forma que ele \
         não pode escrever, e um órfão existe para ser reconhecido no ficheiro",
    ),
    (
        "sections/script.rs",
        "color({}, {}, {}, {})",
        "o construtor que o artista escreveu no .luau dele; traduzi-lo mostra uma forma que ele \
         não pode escrever, e um órfão existe para ser reconhecido no ficheiro",
    ),
];

fn src_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crates/<crate>/ tem dois pais")
        .to_path_buf()
}

/// ⭐⭐⭐ **O painel não escreve palavras no fonte.**
#[test]
fn every_word_this_panel_shows_comes_from_the_string_table() {
    let intrusos: Vec<String> = language_literals(&src_root())
        .into_iter()
        .filter(|l| {
            !NOT_LANGUAGE
                .iter()
                .any(|(f, t, _)| l.rel == *f && l.text == *t)
        })
        .map(|l| format!("{}:{} · {:?}", l.rel, l.line, l.text))
        .collect();
    assert!(
        intrusos.is_empty(),
        "estes textos com cara de língua estão escritos no fonte do Inspector e nunca chegam à \
         tabela de strings (HR-15):\n  {}\n\nA cura é uma chave `{PREFIX}<secção>.<nome>` em \
         `{TABLES:?}` e um `tr(\"…\")` no sítio (uma frase com peças do código: `tr_with`; dentro de \
         uma tabela `const`: `TextKey::new(\"…\")`). ⚠️ Se o texto NÃO é língua, a cura é uma linha \
         em `NOT_LANGUAGE` **com o mecanismo** — nunca sem ele.",
        intrusos.join("\n  ")
    );
}

/// ⭐ **A METADE JUSTA: cada excepção ainda descreve alguma coisa?** — e é ela o controlo de
/// vacuidade: uma régua partida devolve zero literais e lê-se como aprovada, mas não acha as
/// excepções.
#[test]
fn every_named_exception_still_shelters_a_real_literal() {
    let hits = language_literals(&src_root());
    for (file, text, why) in NOT_LANGUAGE {
        assert!(
            why.len() > 40,
            "a excepção `{file}` · {text:?} não diz o mecanismo — uma lista sem mecanismo é uma \
             licença"
        );
        assert!(
            hits.iter().any(|l| l.rel == *file && l.text == *text),
            "a excepção `{file}` · {text:?} já não abriga literal nenhum (ou a régua ficou cega) — \
             apague a linha, senão ela fica aberta para o próximo texto que caia ali"
        );
    }
}

/// ⭐⭐⭐ **Uma chave com erro de escrita pinta o identificador cru** (e vaza, por quadro); uma
/// declarada e não usada é uma órfã. Os dois lados.
#[test]
fn every_inspector_key_exists_on_both_sides() {
    let repo = repo_root();
    let used = keys::keys_used(&repo, PREFIX, TABLES);
    let declared = keys::keys_declared(&repo, TABLES, PREFIX);
    // ⛔ Controlo de vacuidade: um caminho errado dá dois conjuntos vazios, que concordam. O piso
    //    é o vocabulário medido na migração de 2026-09-13 menos folga para o painel encolher.
    assert!(
        declared.len() >= 450 && used.len() >= 450,
        "o censo achou {} declaradas e {} usadas — está a ler o sítio errado",
        declared.len(),
        used.len()
    );
    let sem_traducao: Vec<String> = used
        .iter()
        .filter(|(k, _)| !declared.contains(*k))
        .map(|(k, f)| format!("{k}  (usada em {f})"))
        .collect();
    assert!(
        sem_traducao.is_empty(),
        "estas chaves são usadas e NÃO existem em `{TABLES:?}` — o `tr` pinta o identificador cru:\n  \
         {}",
        sem_traducao.join("\n  ")
    );
    let orfas: Vec<&String> = declared.iter().filter(|k| !used.contains_key(*k)).collect();
    assert!(
        orfas.is_empty(),
        "estas chaves estão na tabela e ninguém as usa — apague-as:\n  {orfas:?}"
    );
}

/// ⭐ **Uma chave mora em UMA tabela, e a §14 mora na dela.** Com o vocabulário partido por secção, a
/// mesma chave nas duas metades é um braço MORTO na segunda (a cadeia do `tr` pára na primeira), e o
/// censo de dois lados não o vê — um conjunto não conta repetições.
#[test]
fn every_inspector_key_lives_in_exactly_one_table() {
    let repo = repo_root();
    // ⚠️⚠️ **A partição é por NOME, nunca por índice** (2026-09-16): a metade *player* era
    //    `TABLES[1..]`, e no dia em que a terceira tabela entrou ela passou a conter as 151 chaves
    //    das secções de jogo — o gate acusou-as todas de estarem «na secção errada» sem uma linha
    //    de produto ter mudado. *Uma fatia de índice é uma lista escrita à mão com outra sintaxe.*
    let e_do_player = |t: &&&str| t.ends_with("inspector_player.rs");
    let player_t: Vec<&str> = TABLES.iter().filter(e_do_player).copied().collect();
    let geral_t: Vec<&str> = TABLES.iter().filter(|t| !e_do_player(t)).copied().collect();
    assert_eq!(player_t.len(), 1, "a §14 mora numa tabela só");
    let geral = keys::keys_declared(&repo, &geral_t, PREFIX);
    let player = keys::keys_declared(&repo, &player_t, PREFIX);
    assert!(
        geral.len() >= 250 && player.len() >= 150,
        "o censo achou {} e {} chaves — uma das metades está a ser lida no sítio errado",
        geral.len(),
        player.len()
    );
    let nas_duas: Vec<&String> = geral.intersection(&player).collect();
    assert!(
        nas_duas.is_empty(),
        "estas chaves estão nas DUAS tabelas do Inspector: {nas_duas:?}"
    );
    // ⚠️ **O prefixo da §14 é DERIVADO, nunca escrito** — um literal `panel.inspector.player.` neste
    //    ficheiro é lido pelo censo como uma chave EM USO, e o gate acusa-se a si próprio (aconteceu
    //    na 1.ª corrida desta metade).
    let seccao = format!("{PREFIX}player.");
    let fora: Vec<&String> = geral
        .iter()
        .filter(|k| k.starts_with(&seccao))
        .chain(player.iter().filter(|k| !k.starts_with(&seccao)))
        .collect();
    assert!(
        fora.is_empty(),
        "estas chaves estão na tabela da secção ERRADA (a §14 mora em `{}`): {fora:?}",
        player_t[0]
    );
}

/// ⭐⭐⭐ **AS LETRAS SOLTAS — o que a régua lexical não pode ver, e por isso precisa de gate.**
///
/// ⛔⛔ **Este gate nasce de uma medição (2026-09-18):** o censo desta crate estava VERDE e o
/// painel pintava `S` · `R` · `M` · `F` · `-` na grelha do 9-slice e `X` · `Y` · `W` · `H` nas
/// células da região. A cegueira é **por construção**: o [`ph2d_label_census::is_language`] exige
/// duas letras SEGUIDAS, senão acusaria todo identificador (`"x"`, `"n"`, `"b"` são nomes de
/// param neste repo às centenas). *Uma letra sozinha não tem forma que a distinga de um
/// identificador — quem a distingue é o TIPO.*
///
/// ⇒ o pintor recebe [`ph2d_i18n::TextKey`] e não `&str`, logo escrever `"W"` lá **não compila**;
/// e o que este gate acrescenta é a outra metade, que o tipo não pode dar: **a chave existe na
/// tabela**. ⚠️ A população é DERIVADA — as do 9-slice das próprias `const` que o painel pinta, as
/// da região do FONTE que as escreve —, nunca de uma segunda lista escrita à mão.
#[test]
fn cada_letra_solta_deste_painel_vem_da_tabela() {
    use ph2d_i18n::TextKey;

    // 1. As do 9-SLICE, lidas das const do produto.
    let do_produto: Vec<TextKey> = ph2d_panel_inspector::REGION_LETTERS
        .into_iter()
        .chain(ph2d_panel_inspector::CORNER_LETTERS)
        .chain([ph2d_panel_inspector::MIXED_LETTER])
        .collect();

    // 2. As da REGIÃO, lidas do ficheiro que as pinta — elas nascem inline, ao lado do rect de
    //    cada célula, e juntá-las numa const só para o gate poria a ORDEM da grelha num sítio
    //    onde ninguém a lê.
    // ⚠️⚠️ **O ficheiro é o IRMÃO da REGIÃO, e não o da proveniência** (2026-09-22): o bloco da
    //    amostragem saiu do `render_source.rs` pelo tecto de LOC, e este gate reprovou em voz alta
    //    a ler `0` células — *a espécie BARULHENTA de gate partido por mover código* (`CLAUDE.md`
    //    §5.0), que é a sorte da história: a irmã MUDA teria ficado verde a medir nada.
    const PINTOR: &str = include_str!("../../src/sections/render_source_regiao.rs");
    let mut do_fonte: Vec<String> = Vec::new();
    for pedaco in PINTOR.split("TextKey::new(\"").skip(1) {
        if let Some(k) = pedaco.split('"').next() {
            do_fonte.push(k.to_string());
        }
    }
    assert_eq!(
        do_fonte.len(),
        4,
        "a região tem QUATRO células (X · Y · W · H) e o pintor declara {}: {do_fonte:?}",
        do_fonte.len()
    );

    // ⛔ Piso de população: sem ele um `REGION_LETTERS` vazio deixaria a varredura a medir nada.
    assert_eq!(
        do_produto.len(),
        7,
        "as letras do 9-slice são 4 de região + 2 de canto + a de mista"
    );

    let cruas: Vec<String> = do_produto
        .iter()
        .map(|k| k.key().to_string())
        .chain(do_fonte)
        .filter(|k| ph2d_i18n::tr(k) == *k)
        .collect();
    assert!(
        cruas.is_empty(),
        "estas chaves de LETRA não existem na tabela — o painel vai pintar o identificador no \
         lugar da letra:\n  {}",
        cruas.join("\n  ")
    );
}
