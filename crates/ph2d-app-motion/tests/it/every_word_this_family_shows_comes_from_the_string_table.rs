//! ⭐⭐⭐ **NENHUMA PALAVRA QUE O MOTION DIZ É ESCRITA NO FONTE** — o HR-15 na crate de família dos
//! nós.
//!
//! Medido em 2026-09-16: **47** textos fora das cenas — as recusas de LIGAR dois pinos, os
//! **conselhos** que o grafo dá a um nó que não faz nada (*«This node has no points to work on —
//! wire a source (Grid / Emitter) into it»*), os intents do caminho desenhado, os rótulos do cartão
//! dobrado e o leitor do canvas. Migrados para `ph2d-i18n/src/app_motion.rs`.
//!
//! ⚠️⚠️ **O que é uma CENA nesta crate não se chama `smoke`:** as cenas de demonstração da
//! conferência chamam-se `motion_state_conferencia_demos_*`, e são **93 ficheiros** de conteúdo de
//! cena (nomes de nós, legendas de canvas, instruções de consola para quem as corre). ⛔ Uma régua
//! que só conhecesse `smoke` teria convertido **471** literais que são o DADO de uma demonstração —
//! é por isso que os marcadores de cena se passam ao gate em vez de estarem embutidos nele.

use ph2d_label_census::gate::{self, Excecao, Isento};

const TABLE: &str = "crates/ph2d-i18n/src/app_motion.rs";

/// ⭐ **O que é uma CENA nesta crate** — o piso de população abaixo é o controlo desta enumeração.
const CENAS: &[&str] = &["smoke", "probe", "demos", "motion_state_"];

/// Um ficheiro isento inteiro, com o mecanismo.
const FORA: &[Isento] = &[
    (
        "motion_bridge_gpu.rs",
        "as RAZÕES da rota do cook (`device: o plano inteiro`, `CPU: o documento nao tem saida`), impressas \
         atrás de `PH2D_MOTION_ROUTE_LOG` — diagnóstico de quem mede onde o grafo corre",
    ),
    (
        // ⚠️⚠️ **Esta entrada é a IRMÃ da de cima e nasceu de uma MUDANÇA DE ENDEREÇO**, não de
        // texto novo: as duas frases (`RECUSA_COLISOR` · `RECUSA_COLISOR_EXTERNO`) viviam no
        // ficheiro acima e saíram para o irmão quando o tecto de LOC obrigou ao corte (doc 115
        // W4). *Uma isenção de censo é propriedade do CÓDIGO e viaja com ele* — e a prova de que
        // é mudança de endereço e não texto novo é o `isentos_mortos` deste mesmo gate NÃO ter
        // acusado a entrada de cima, porque as outras razões da rota ficaram lá.
        "motion_bridge_gpu_colisor.rs",
        "as duas RAZÕES de recusa do colisor, irmãs das de cima e pela mesma porta `say_route` — \
         impressas atrás de `PH2D_MOTION_ROUTE_LOG`, nunca no ecrã",
    ),
    (
        // A TERCEIRA irmã, pelo mesmo motivo (doc 119 §7): a razão da recusa da forma viva
        // CONDICIONAL (o L-System em `Branches`) saiu para um ficheiro próprio pelo tecto de LOC
        // do de cima, e é impressa pela mesma porta.
        "motion_bridge_gpu_forma.rs",
        "a RAZÃO da recusa da forma viva condicional, irmã das de cima e pela mesma porta \
         `say_route` — impressa atrás de `PH2D_MOTION_ROUTE_LOG`, nunca no ecrã",
    ),
    (
        "motion_glow_layer.rs",
        "o `[glow-diag]` atrás de `PH2D_GLOW_DIAG` — a lista de que é feito o bright-pass, para \
         quem caça um halo que não aparece",
    ),
    (
        "warp_overlay.rs",
        "o `vec_overlay_diag::diag` do warp — uma linha de terminal que só sai com a env ligada",
    ),
    (
        "motion_flip_bake_offscreen.rs",
        "o RÓTULO de depuração do wgpu do readback (o que aparece no RenderDoc, nunca no ecrã)",
    ),
    (
        "motion_text_gen.rs",
        "a CHAVE de cache de um glifo (`glyph:<fonte>:<tamanho>:<ch>`) — um identificador montado \
         com `format!`, não uma frase",
    ),
    (
        "motion_table_gen.rs",
        "os nomes das COLUNAS que a tabela gerada publica (`Index`, `Count`) e o nome derivado do \
         recurso — um nó a jusante lê a coluna PELO NOME, logo traduzi-la partiria o grafo",
    ),
    (
        "motion_lsystem_rows.rs",
        "os mesmos nomes de COLUNA (`Index`, `Count`) na tabela que o L-System publica — lidos por \
         nome a jusante",
    ),
    (
        "motion_lsystem_leaves.rs",
        "as CHAVES de dedup do `on_rising_edge` (`<id> inert tropism`) — o que decide se um aviso \
         já foi dado, nunca o aviso",
    ),
];

/// Um literal isento, com o mecanismo.
const NOT_LANGUAGE: &[Excecao] = &[(
    "motion_bridge_fold.rs",
    "Root",
    "o nome do nível de topo na trilha de grupos — o mesmo `Root` que o documento guarda, e a \
     trilha compara-o contra o nome gravado",
)];

#[test]
fn every_word_this_family_shows_comes_from_the_string_table() {
    let (src, _) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let intrusos = gate::intrusos_fora_de(&src, NOT_LANGUAGE, FORA, CENAS);
    assert!(
        intrusos.is_empty(),
        "texto com cara de língua escrito no fonte do Motion (HR-15):\n  {}\n\n\
         A cura é uma chave `app.motion.<ficheiro>.<frase>` em `{TABLE}` e um `tr(\"…\")` no sítio \
         (uma frase com peças do código: `tr_with`). ⚠️ Se o texto é o DADO de uma cena, ele mora \
         num ficheiro cujo nome o declara (`*_demos_*`, `*smoke*`) — e se é um nome de COLUNA, \
         lembre-se de que um nó a jusante o lê pelo nome.",
        intrusos.join("\n  ")
    );
}

/// ⭐ **A metade justa**, com o piso de população das cenas — a régua delas é o NOME do ficheiro.
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
    let em_cena = gate::literais_de_cena(&src, CENAS);
    assert!(
        em_cena >= 350,
        "as cenas abrigam {em_cena} textos — em 2026-09-16 eram 471 em 93 ficheiros. Ou elas saíram \
         desta crate, ou a régua por NOME deixou de casar e este gate passou a medir o nada"
    );
}

/// ⭐⭐ **As chaves existem dos dois lados.**
#[test]
fn every_key_of_this_family_exists_on_both_sides() {
    let (_, repo) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let c = gate::chaves(&repo, "app.motion.", &[TABLE]);
    assert!(
        c.declaradas >= 40 && c.usadas >= 40,
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
