//! ⭐⭐⭐ **NENHUMA PALAVRA QUE O VETOR DIZ É ESCRITA NO FONTE** — o HR-15 na crate de família do
//! vectorial.
//!
//! Medido em 2026-09-16: **8** textos — os **selos da booleana viva** na linha da Hierarquia
//! (`BSE`/`RCP`/`UNI`/`SUB`/`INT`/`EXC`) e as duas recusas de importar um `.svg`. Migrados para
//! `ph2d-i18n/src/app_vec.rs`.
//!
//! ⚠️⚠️ **O selo defendia-se por escrito** (*«não passa por i18n, de propósito … a tabela de TOM
//! casa contra a própria string»*) e a nota estava meia certa: o `badge_tone` da Hierarquia **casa
//! mesmo contra o texto pintado**, mas isso não é razão para o literal — é um defeito de desenho
//! que o gate `the_badge_tone_still_matches_what_the_table_paints` passa a nomear.

use ph2d_label_census::gate::{self, Excecao, Isento};

const TABLE: &str = "crates/ph2d-i18n/src/app_vec.rs";

/// Um ficheiro isento inteiro, com o mecanismo.
const FORA: &[Isento] = &[
    (
        "svg_export.rs",
        "o FORMATO do ficheiro que se exporta (atributos SVG, `<linearGradient>`, o cabeçalho) — \
         texto que outro programa lê, e que muda de significado se alguém o traduzir",
    ),
    (
        "texture_pattern_edit.rs",
        "o retrato `[pattern]` impresso no terminal (que tinta, que traço, que padrão a forma tem) \
         — diagnóstico de quem segue um report sobre o padrão",
    ),
    (
        "fx_dump.rs",
        "o despejo `PH2D_FX_DUMP` da pilha de efeitos, escrito num ficheiro de texto para \
         bissecção — nunca pintado",
    ),
    (
        "bucket_repro.rs",
        "os pedaços de SVG que a reprodução do balde procura (`data-ph2d-id=`, `fill=`) — sintaxe \
         de um formato, não vocabulário",
    ),
    (
        "shape_build.rs",
        "os rótulos do log da construção (`ARTE` / `lasca (descartada)`, a área de cada peça) — \
         terminal, e só com o log ligado",
    ),
    (
        "overlay_diag.rs",
        "o desenho de DIAGNÓSTICO sobre o canvas (`⚠ NÃO-FINITO`), que só existe com a env ligada \
         — a instrumentação de quem caça uma alça disparada",
    ),
    // ⚠️ **Esta isenção VEIO DA SHELL na integração de 2026-09-17**, com o ficheiro: ela vivia no
    // `FORA` do censo de `shells/desktop` e o `git mv` deixou-a lá **morta** (o censo de
    // obsolescência de lá acusou-a na mesma corrida em que este acusou os cinco literais).
    // *Uma isenção é uma propriedade do CÓDIGO, não do sítio onde o código está, logo ela viaja.*
    (
        "envelope_gesture.rs",
        "os cinco argumentos de `overlay_diag::refused` — o MESMO diagnóstico da linha acima, visto \
         do lado de quem o chama: ele só existe com a env ligada, e quem o lê caça uma alça que \
         recusou mexer-se",
    ),
];

/// Um literal isento, com o mecanismo — todos NOMES de objecto.
const NOT_LANGUAGE: &[Excecao] = &[
    (
        "blend_live.rs",
        "Blend {id}",
        "o nome por omissão de um objecto de mistura NOVO — entra no `Name`, que é identidade \
         durável (`stable_name_id`)",
    ),
    (
        "connector_live.rs",
        "Connector {id}",
        "o nome por omissão de um conector novo — identidade durável",
    ),
    (
        "morph_live.rs",
        "Morph {id}",
        "o nome por omissão de um conjunto de morph novo — identidade durável",
    ),
    (
        "frame_labels.rs",
        "Frame",
        "o nome por omissão de uma moldura nova — identidade durável",
    ),
    (
        "bool_gesture.rs",
        "Boolean",
        "o nome do GRUPO que o gesto booleano cria — identidade durável",
    ),
    (
        "svg_import.rs",
        "Drawing",
        "o nome por omissão de um desenho importado sem nome no ficheiro — identidade durável",
    ),
];

#[test]
fn every_word_this_family_shows_comes_from_the_string_table() {
    let (src, _) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let intrusos = gate::intrusos_fora_de(&src, NOT_LANGUAGE, FORA, gate::CENAS);
    assert!(
        intrusos.is_empty(),
        "texto com cara de língua escrito no fonte do vectorial (HR-15):\n  {}\n\n\
         A cura é uma chave `app.vec.<ficheiro>.<frase>` em `{TABLE}` e um `tr(\"…\")` no sítio. \
         ⚠️ Se é um NOME de objecto ou a sintaxe de um formato, a cura é uma linha em \
         `NOT_LANGUAGE` ou `FORA` **com o mecanismo**.",
        intrusos.join("\n  ")
    );
}

/// ⭐ **A metade justa**, com o piso de população das cenas.
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
    let em_cena = gate::literais_de_cena(&src, gate::CENAS);
    assert!(
        em_cena >= 5,
        "as cenas de smoke abrigam {em_cena} textos — em 2026-09-16 eram 7. Ou elas saíram desta \
         crate, ou a régua por NOME deixou de casar e este gate mede o nada"
    );
}

/// ⭐⭐ **As chaves existem dos dois lados.**
#[test]
fn every_key_of_this_family_exists_on_both_sides() {
    let (_, repo) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let c = gate::chaves(&repo, "app.vec.", &[TABLE]);
    assert!(
        c.declaradas >= 6 && c.usadas >= 6,
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
