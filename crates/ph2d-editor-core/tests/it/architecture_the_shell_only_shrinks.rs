//! ⭐⭐⭐ **A SHELL SÓ ENCOLHE** — a catraca da grandeza que de facto decide o tempo de build.
//!
//! # Por que este gate existe, com os números
//!
//! A auditoria de velocidade de 2026-09-10 mediu que o custo do laço de desenvolvimento **não**
//! está no `cargo check` (1,8–3,3 s) nem no linker: está em `shells/desktop` ser **UMA unidade de
//! compilação** de meio milhão de linhas, a **última** de toda build grande — sozinha, 34–45 s no
//! fim do portão de fecho, e 161 s de reconstrução em `--release` num núcleo só.
//!
//! A W2 (2026-09-11) tirou de lá **61 704 linhas** em seis linhas paralelas. ⛔ **Nada impedia que
//! elas voltassem.**
//!
//! # ⛔⛔ Por que os tectos que já existiam não respondem a esta pergunta
//!
//! Todo tecto de LOC deste repo é **por FICHEIRO** (`architecture_workspace_file_loc_cap` = 700,
//! `architecture_panel_loc_cap`, `architecture_widget_loc_cap`, `file_loc_caps` da shell). Eles
//! medem *legibilidade de um ficheiro*; nenhum mede **o tamanho da unidade que o compilador
//! constrói**. Uma shell de 465 mil linhas em 1 801 ficheiros de 258 linhas cada passa em todos
//! eles com folga — e continua a ser a unidade mais lenta da árvore.
//!
//! ⚠️ **É a mesma forma que o `CLAUDE.md` §5.0 já nomeia um nível abaixo** (*«um tecto por-ficheiro
//! é a única grandeza deste repo que SOMA entre linhas sem ninguém a contar»*), e aqui o que soma é
//! a **crate**: seis linhas podem acrescentar 200 linhas cada à shell, todas verdes de boa-fé, e o
//! total sobe 1 200 sem um único gate a acordar.
//!
//! # A regra que este gate torna executável
//!
//! ⭐ **Código de FAMÍLIA vive em `crates/ph2d-app-<família>`; a shell é COMPOSIÇÃO.** O molde, as
//! cinco portas do trait de host e as 15 armadilhas medidas estão em
//! [`HOWTO_partir_uma_familia_da_shell.md`](../../../../docs/IntegracaoMultiAgente/HOWTO_partir_uma_familia_da_shell.md).
//! Quando este gate reprovar, a cura é **mover para a crate da família**, nunca subir o número.
//!
//! # ⚠️ A catraca tem as DUAS metades, senão vira licença
//!
//! `CLAUDE.md` §5.0: *«uma catraca sem censo de obsolescência não desce: ela vira LICENÇA»*. Então
//! o gate reprova nos dois sentidos — cresceu acima do tecto **ou** encolheu tanto que o tecto
//! deixou de descrever a árvore e ninguém o apertou.
//!
//! ⛔ **A folga NÃO é permissão para crescer.** Ela existe porque a shell continua a ser a raiz de
//! composição: ligar uma família nova custa linhas legítimas no `main.rs`. O que a folga compra é
//! ~0,9 % — o suficiente para compor, longe do suficiente para uma família caber.
//!
//! Sem deps (só `std`), como os gates de arquitectura irmãos.

use std::fs;
use std::path::{Path, PathBuf};

/// **O tecto da crate inteira**, em linhas de `.rs` versionado.
///
/// ⚠️ **Recontado a cada integração que encolhe a shell** — é isso que impede a folga de virar
/// licença. Histórico: **465 105** / 1 801 ficheiros (11/09, fim da Fase A da W2) → **451 084** /
/// 1 757 (11/09, Fase B da `physics`) → **411 246** / 1 616 (12/09, Fase B da `sculpt3d`, da `vec`
/// e da `flip`, integradas na ordem do churn de costura; medido **depois** do `cargo fmt --all`,
/// porque reformatar 46 ficheiros move a contagem) → **398 037** / 1 561 (12/09, a
/// `line/shell-folhas`: as folhas partilhadas que prendiam TRÊS famílias saíram para sete crates
/// nomeadas por assunto) → **381 328** / 1 521 (12/09, a **2.ª volta** da Fase B da `flip`: 53
/// ficheiros, e com eles os CINCO que a `line/app-motion` tinha nomeado como bloqueadores dela) →
/// **373 937** / 1 498 (12/09, a 2.ª volta da `vec`: 16 ficheiros, e `"vec"` sai da catraca das
/// famílias sem roteador) → **262 540** / 1 040 (12/09, **FASE C da `motion`**: 421 ficheiros, a
/// maior família da wave, e com ela a catraca das famílias sem roteador MORREU — ver
/// `ph2d-app-registry-init`) → **238 732** / 966 (12/09, **Fase C da `vec`**: as 16 leis VIVAS, a
/// edição da estampa, o balde, o exportador de SVG — e o **Esqueleto saiu como família própria**,
/// `ph2d-app-skeleton`) → **225 394** / 917 (12/09, **Fase C da `physics`**: as juntas, o inspector
/// e o bake — 48 ficheiros, e o `impl App` era **assinatura**, zero portas novas). O tecto é o
/// medido mais a [`FOLGA_DE_COMPOSICAO`].
///
/// **FASE D** (a 4.ª rodada, 12/09) → **208 442** / 859 (a família das **INSTÂNCIAS** sai:
/// `crates/ph2d-app-components`, 58 ficheiros) → **196 612** / 821 (a `vec`: 38 f) →
/// **186 647** / 778 (o **PAINTER**, a última família grande sem crate própria:
/// `crates/ph2d-app-painter`, 43 f).
///
/// **A DIVISÃO DO QUADRO** (13/09, `line/render-loop` + `line/editor-core` integradas) → **192 990** /
/// 907. ⛔⛔ **A única vez que este tecto SOBE, e foi por ordem do dono:** o `run_render_frame` de
/// 13 685 linhas partiu-se em 125 fases chamadas pela MESMA ordem (`render_loop/fase_*.rs`), e cada
/// fase paga prelúdio, assinatura e o contexto que entra e sai (~24 linhas) — `src` `+3 719` e
/// `tests` `+1 894` líquidos, medidos pela linha. O dono, com o smoke aprovado: *«Autorizo subir o
/// limite para caber a separação.»* ⛔ **Não é precedente:** o que cresceu aqui é a FORMA do laço,
/// não código de família; a cura de um crescimento continua a ser MOVER para a crate da família.
///
/// **A PODA DO 3D** (05/10, `line/poda-3d`, ADR-0179): o modelador, a escultura e o Render3d saem do
/// PH2D, e com eles a metade-shell das duas famílias (as fases do quadro, o anfitrião da escultura,
/// os canais assados, ~30 gates) → **185 041** / 974 (de `196 867` / 1 021 no `b1a6f9b07`, `−11 826`
/// linhas), medido depois do `cargo fmt`. O tecto é o medido mais a folga.
///
/// ⭐⭐⭐ **De 526 809 para 186 647 em dois dias — a shell tem 35 % do tamanho que tinha**, e a
/// unidade de compilação que era o tecto do relógio deste repo deixou de o ser.
///
/// ⚠️ **Medido DEPOIS do `cargo fmt --all`, e a ordem não é cerimónia:** a formatação das duas
/// linhas que o integrador escreveu no censo das pontes moveu a contagem de `186 619` para `186 629`, e as notas que ele escreveu nos dois
/// gates que a mudança de casa partiu levaram-na a `186 647`. *Dez linhas não mudam nada aqui; mudaram quando foram 32, e a regra é a mesma.*
///
/// ⭐⭐⭐ **Em dois dias a shell caiu de 526 809 para 225 394 — `−301 415`, `−57,2 %`**, e de 2 023
/// ficheiros para 917. Mais de metade da unidade de compilação que era o tecto do relógio deste
/// repo deixou de existir ali.
///
/// ⭐ **E a conta paga-se no relógio, medida na própria integração:** o portão de testes impactados
/// desta rodada correu em **26–27 s**, contra **51 s** na rodada anterior sobre a mesma máquina e o
/// mesmo perfil. *A grandeza que este gate protege não é estética — é o tempo de todos os dias.*
///
/// ⚠️ **A `motion` sozinha vale mais do que as duas primeiras rodadas juntas** (`−111 397` contra
/// `−61 704` da Fase A) — e ela esteve BLOQUEADA duas voltas seguidas por `632` linhas de outra
/// família. *O tamanho de uma família não prevê a ordem em que ela pode sair; o FECHO dela prevê.*
///
/// ⚠️ **Esta recontagem NÃO precisou de `cargo fmt --all`** — a árvore combinada das duas linhas
/// chegou já formatada (`--check` limpo), e é por isso que este número é o mesmo antes e depois. A
/// ordem do §5 do `ESTADO_W2` continua a valer: *integrar → formatar → medir → escrever*; o que
/// muda quando o `fmt` tem trabalho para fazer é o **valor**, não o passo.
///
/// ⛔ **Nenhuma LINHA lhe toca** — é um número que soma entre linhas, logo CONTA-SE, nunca se
/// escolhe (`CLAUDE.md` §5.0): com cinco linhas a escrevê-lo o merge fica com um deles e nenhum
/// está certo, em silêncio. Quem o reconta é o integrador, sobre a árvore combinada.
const TETO_LOC: usize = 189_041;

/// Quanto a shell pode crescer acima do medido antes de o gate reprovar — a margem da raiz de
/// composição, **não** espaço para um módulo.
const FOLGA_DE_COMPOSICAO: usize = 4_000;

/// ⭐ **O censo de obsolescência:** quando a shell cai mais do que isto abaixo do tecto, o número
/// acima deixou de a descrever e tem de ser reescrito com a medição do dia.
///
/// ⚠️ **É maior que a folga de propósito.** Um limiar apertado dispararia a cada linha removida —
/// e durante a Fase B a shell encolhe todos os dias. O que este censo existe para apanhar é *uma
/// wave inteira que aterrou e deixou a catraca para trás*, não uma limpeza de dez linhas.
const CENSO_OBSOLETO_ACIMA_DE: usize = 20_000;

fn shell_dir() -> PathBuf {
    // CARGO_MANIFEST_DIR = crates/ph2d-editor-core; dois pais = a raiz da workspace.
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crates/<x>/ tem dois pais")
        .join("shells/desktop")
}

/// Todas as linhas de `.rs` sob `dir`, recursivamente.
///
/// ⚠️ Salta `target/` por defesa: numa workspace ele não nasce aqui, mas um `CARGO_TARGET_DIR`
/// apontado para dentro da crate faria este gate medir código gerado e reprovar sobre nada.
fn conta(dir: &Path, linhas: &mut usize, ficheiros: &mut usize) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            if p.file_name().is_some_and(|n| n == "target") {
                continue;
            }
            conta(&p, linhas, ficheiros);
        } else if p.extension().is_some_and(|x| x == "rs") {
            let Ok(s) = fs::read_to_string(&p) else {
                continue;
            };
            *linhas += s.lines().count();
            *ficheiros += 1;
        }
    }
}

#[test]
fn the_shell_only_shrinks() {
    let (mut loc, mut ficheiros) = (0usize, 0usize);
    conta(&shell_dir(), &mut loc, &mut ficheiros);

    // ⛔ Controlo positivo: uma varredura que não achasse nada reprovaria «cresceu zero» e leria-se
    //    como aprovada — a armadilha do censo que mede zero. Sem isto o gate é vácuo se o caminho
    //    da shell mudar de nome.
    assert!(
        ficheiros > 500,
        "controlo positivo: a varredura achou {ficheiros} ficheiros `.rs` em `shells/desktop` — \
         o caminho mudou e este gate passou a medir o nada"
    );

    assert!(
        loc <= TETO_LOC,
        "a SHELL CRESCEU: {loc} linhas contra o tecto de {TETO_LOC} ({ficheiros} ficheiros).\n\
         Ela é UMA unidade de compilação e a última de toda build grande — 34-45 s sozinha no \
         portão de fecho (auditoria de 2026-09-10).\n\
         ⇒ a cura é MOVER para `crates/ph2d-app-<família>`, nunca subir este número. O molde está \
         em `docs/IntegracaoMultiAgente/HOWTO_partir_uma_familia_da_shell.md`.\n\
         ⛔ Subir o tecto sem mover código é desfazer, uma wave de cada vez, as 61 704 linhas que \
         a W2 tirou de lá."
    );

    assert!(
        loc + CENSO_OBSOLETO_ACIMA_DE >= TETO_LOC,
        "a catraca está OBSOLETA: a shell tem {loc} linhas e o tecto ainda diz {TETO_LOC} — uma \
         folga de {} linhas.\n\
         Uma catraca que não desce vira LICENÇA (CLAUDE.md §5.0): reescreva `TETO_LOC` como \
         {} (o medido de hoje + a folga de composição de {FOLGA_DE_COMPOSICAO}).",
        TETO_LOC - loc,
        loc + FOLGA_DE_COMPOSICAO
    );
}

/// ⭐ **Toda `ph2d-app-*` também só encolhe** (item M da comissão de velocidade, 29/10,
/// `docs/DevOps/COMISSAO_VELOCIDADE_2026-10-29.md`). Tirar a família da shell só mudou a unidade
/// lenta de sítio: `ph2d-app-motion` tem 193 644 linhas — mais que a shell inteira. O tecto é por
/// CRATE porque a crate é a unidade que o compilador constrói (o mesmo argumento do topo).
///
/// Cada linha: o tecto = o MEDIDO pela mesma [`conta`] (o diretório da crate inteiro: `src`,
/// `tests`, `benches`) + [`folga_da_crate`]. Medido 2026-10-29 no `a0fcfc1f19`, árvore limpa nas
/// `crates/ph2d-app-*`; o medido está no comentário de cada linha.
///
/// ⛔ É um número que SOMA entre linhas: quem o reconta é o integrador, sobre a árvore combinada.
/// `ph2d-app-host` fica de fora: é o substrato (`Folha` em
/// `architecture_no_dependency_climbs_a_layer.rs`), não uma família.
const TETOS_DAS_CRATES_APP: &[(&str, usize)] = &[
    ("ph2d-app-audio", 10_547),       // 10_341
    ("ph2d-app-board", 15_600),       // 15_295
    ("ph2d-app-code", 26_628),        // 26_106
    ("ph2d-app-components", 80_869),  // 79_284
    ("ph2d-app-constraints", 10_526), // 10_320
    ("ph2d-app-flip", 28_960),        // 28_393
    ("ph2d-app-game", 2_869), // 1_993 (12/11: as fotos e a mão do smoke saíram para `ph2d-game-smoke-fotos`); +776 (integ 13/11: `posse_do_jogo`, o registo e o `game_mouse` vieram da `ph2d-app-components`, que estava 29 acima do tecto)
    ("ph2d-app-input-map", 449), // 349 (InputMap R2a, 10/11: + `listen.rs`, a escuta do comando que saiu da shell)
    ("ph2d-app-joint-editor", 4_749), // 4_649 (recontado 13/11 após a W3f, `conta()` + `folga_da_crate`)
    ("ph2d-app-kanban", 32_175),      // 31_545
    ("ph2d-app-light2d", 25_523),     // 25_023
    ("ph2d-app-macros", 14_630),      // 14_344
    ("ph2d-app-motion", 197_516),     // 193_644
    ("ph2d-app-painter", 24_359), // 23_882 (integ 13/11: −724 do `CanvasMap` para a `ph2d-painter-canvas-map`; o tecto desce); 23_950
    ("ph2d-app-physics", 37_798), // 37_778 (GEF02 onda 8: a cena 48 saiu para `ph2d-physics-smoke-scenes`, 287 linhas com o tecto)
    ("ph2d-app-pixel", 11_278),   // 11_278 (P12a: as teclas pela tabela `ph2d-pixel-keys`)
    ("ph2d-app-planner", 5_824),  // 5_710
    ("ph2d-app-registry-init", 799), // 699
    ("ph2d-app-sfxlab", 2_692),   // 2_592
    ("ph2d-app-skeleton", 11_263), // 11_043
    ("ph2d-app-sprite-anim", 5_209), // 5_107 (GEF 26: o documento do editor de animações saiu para `ph2d-anim-editor`)
    ("ph2d-app-text", 423), // 323 (GEF 21: Add ▸ Text, crate nova porque a ph2d-app-vec está no tecto)
    ("ph2d-app-vec", 93_637), // 91_801
];

/// ⭐ As crates PURAS para onde a família saiu da `ph2d-app-*` (CodeEditor fecho, 08/11): a isenção
/// viaja com o código (CLAUDE.md §5.0) — o que sai da catraca da `ph2d-app-code` entra nesta, com
/// a mesma régua. Medido 2026-11-08 na `line/ScriptEditor` (o medido no comentário).
const TETOS_DAS_CRATES_DA_FAMILIA: &[(&str, usize)] = &[
    ("ph2d-code-assist", 4_188), // 4_088 (10/11: as dobras fechadas, os problemas do Play e o `--!strict` saíram da `ph2d-app-code`)
    ("ph2d-code-bancada", 1_028), // 928 (Onda B, a Bancada; os nomes da cena saíram da `ph2d-app-code`)
    ("ph2d-code-debugcard", 663), // 563 (plano 14: o modelo do quadro saiu da `ph2d-code-assist`; a pintura foi para o `ph2d-panel-code`)
    ("ph2d-code-scene-arena", 223), // 123 (10/11: a cena 15, crate nova porque a `ph2d-code-scenes` e a `ph2d-code-smoke` estão no tecto)
    ("ph2d-code-scenes", 666),      // 566
    ("ph2d-code-smoke", 614),       // 514
    ("ph2d-code-viewmodel", 304),   // 204
    ("ph2d-game-smoke-fotos", 1_077), // 977 (12/11: as fotos e a mão saíram da `ph2d-app-game`, com o miolo de `janela_do_jogo_smoke.rs` da shell)
    ("ph2d-joint-assembly", 1_814), // 1_714 (corte 13/11: a montagem de juntas e os gestos de criar junta saíram da `ph2d-app-physics`)
    ("ph2d-painter-canvas-map", 824), // 724 (integ 13/11: o `CanvasMap` e o gate dele saíram da `ph2d-app-painter`, que estava 177 acima do tecto)
    ("ph2d-physics-smoke-jump", 1_258), // 1_158 (13/11: as cenas 110 e 111 e o gate do pulo na parede saíram da `ph2d-physics-smoke-scenes`, que estava 204 acima do tecto)
    ("ph2d-physics-smoke-scenes", 18_422), // 18_061 (13/11: −1 151 do pulo para a `ph2d-physics-smoke-jump`; o tecto desce); 18_859 (integ 12/11: +149 da cena 49, `joint_break`, que saiu da `ph2d-app-physics` para curar o tecto dela); 18_732 (GEF02 onda 8: +287 da cena 48 com o tecto, +122 da cena 122)
];

/// A folga de uma crate: **2 %** do medido, piso de 100 linhas. É a razão da shell
/// (`FOLGA_DE_COMPOSICAO` 4 000 sobre 185 041 medidas = 2,2 %): o bastante para compor, longe do
/// bastante para uma família caber. O piso existe porque 2 % de 699 linhas são 13 — uma
/// formatação moveria o gate.
fn folga_da_crate(medido: usize) -> usize {
    (medido / 50).max(100)
}

/// O censo de obsolescência de uma crate: **5× a folga** (10 % do tecto, piso 500) — a mesma razão
/// da shell (`CENSO_OBSOLETO_ACIMA_DE` 20 000 = 5 × 4 000). Apanha uma onda que partiu a crate e
/// deixou o tecto para trás, não uma limpeza de dez linhas.
fn censo_da_crate(teto: usize) -> usize {
    (teto / 10).max(500)
}

#[test]
fn the_shell_only_shrinks_and_every_app_crate_too() {
    let crates_dir = shell_dir()
        .parent()
        .and_then(Path::parent)
        .expect("shells/desktop tem dois pais")
        .join("crates");
    let mut no_disco: Vec<String> = fs::read_dir(&crates_dir)
        .expect("crates/ legível")
        .flatten()
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| n.starts_with("ph2d-app-") && n != "ph2d-app-host")
        .collect();
    no_disco.sort();

    // ⛔ Controlo positivo: a varredura das crates tem de achar a população inteira.
    assert!(
        no_disco.len() >= 10,
        "controlo positivo: só {} crates `ph2d-app-*` em {} — o caminho mudou e o gate mede o nada",
        no_disco.len(),
        crates_dir.display()
    );

    let mut erros = Vec::new();
    for nome in &no_disco {
        if !TETOS_DAS_CRATES_APP.iter().any(|(n, _)| n == nome) {
            erros.push(format!(
                "`{nome}` existe e não tem linha em `TETOS_DAS_CRATES_APP` — crate nova entra com o \
                 seu número MEDIDO por `conta()` + `folga_da_crate`"
            ));
        }
    }
    for &(nome, teto) in TETOS_DAS_CRATES_APP
        .iter()
        .chain(TETOS_DAS_CRATES_DA_FAMILIA)
    {
        if !crates_dir.join(nome).is_dir() {
            erros.push(format!(
                "`{nome}` tem linha em `TETOS_DAS_CRATES_APP` e a crate já não existe — apague a \
                 linha (e conte a crate para onde o código foi)"
            ));
            continue;
        }
        let (mut loc, mut ficheiros) = (0usize, 0usize);
        conta(&crates_dir.join(nome), &mut loc, &mut ficheiros);
        if ficheiros == 0 {
            erros.push(format!(
                "controlo positivo: `{nome}` varreu ZERO ficheiros `.rs` — o gate mediria o nada"
            ));
            continue;
        }
        if loc > teto {
            erros.push(format!(
                "`{nome}` CRESCEU: {loc} linhas contra o tecto de {teto} ({ficheiros} ficheiros). \
                 A cura é MOVER/cortar (para uma crate da família, ou apagar o morto), nunca subir \
                 o número (CLAUDE.md §2)."
            ));
        } else if loc + censo_da_crate(teto) < teto {
            erros.push(format!(
                "a catraca de `{nome}` está OBSOLETA: {loc} linhas e o tecto ainda diz {teto}. \
                 Uma catraca que não desce vira LICENÇA (CLAUDE.md §5.0): reescreva-o como {} \
                 (o medido + `folga_da_crate`).",
                loc + folga_da_crate(loc)
            ));
        }
    }
    assert!(
        erros.is_empty(),
        "{} crate(s) `ph2d-app-*` fora da catraca:\n  {}",
        erros.len(),
        erros.join("\n  ")
    );
}
