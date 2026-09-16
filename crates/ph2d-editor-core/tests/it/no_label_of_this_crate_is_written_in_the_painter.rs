//! ⭐⭐⭐ **O TEXTO DE INTERFACE DESTA CRATE, CONTADO PELA RÉGUA QUE VÊ TUDO** — o HR-15, com a dívida
//! escrita ao número, ficheiro a ficheiro, numa catraca que só desce.
//!
//! > `CLAUDE.md` §0.3: *«UI canônica: zero hex, zero `f32` literal de UI, **zero string
//! > hardcoded** — tudo via tokens / i18n (HR-15).»*
//!
//! # ⛔⛔ Este gate dizia «língua de produto: 0» — e a régua dele via uma fracção da crate
//!
//! Até 2026-09-13 este ficheiro trazia um leitor próprio (~400 linhas) que seguia o texto até um
//! PINTOR (`paint_*`/`draw_*` com parâmetro `&str`) e declarava não ver construtores, tabelas e
//! `format!`. Com ele a crate lia-se curada (`32` literais, todos da galeria-bancada). A régua
//! LEXICAL da `ph2d-label-census` — todo literal com cara de língua fora de teste e fora do que nunca
//! pinta — conta **641**: **546** fora da bancada, em **46** ficheiros, `164` só no
//! `screens/hero/menu_rows.rs`. *Declarar um ponto cego não o torna pequeno.*
//!
//! ⇒ o leitor saiu daqui (a régua é uma folha partilhada com os gates por crate dos painéis, em vez
//! de copiada para cada um), e a dívida passou a ser um NÚMERO por ficheiro, com as duas metades.
//!
//! # ⛔ Por que POR CRATE, e não do repo
//!
//! Uma catraca global com a dívida das outras crates dentro poria *a próxima linha que pinte um
//! rótulo vermelha por causa de um gate desta* — o contrário do que o `CLAUDE.md` §0.2 pede de um
//! toque foundational. Cada crate de painel tem o seu (`every_word_this_panel_shows_comes_from_the_
//! string_table`), a zero, sobre a mesma régua.
//!
//! # A cura de um vermelho
//!
//! - **Cresceu** — a palavra nova vai para a tabela (`crates/ph2d-i18n/src/chrome.rs`, ou o irmão do
//!   assunto) e o sítio chama `ph2d_i18n::tr` (uma frase com peças do código: `tr_with`). ⛔ Nunca
//!   subir o número.
//! - **Desceu** — escreva o número novo, ou apague a linha que chegou a zero. É a metade que impede a
//!   lista de virar licença (`CLAUDE.md` §5.0).
//! - **Não é língua** (o nome de um tipo nosso, dados de uma cena de amostra) — uma linha em
//!   `NOT_LANGUAGE` **com o mecanismo**, e o número sai da dívida.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ph2d_label_census::{keys, language_literals};

/// ⭐ As excepções, **com o mecanismo** — nunca uma lista aberta.
///
/// ⚠️ A entrada `widget/command_palette/header.rs` (o «X» do fecho e o «x» do visto) SAIU em
/// 2026-09-13: a régua lexical não conta uma letra solta como palavra, e o que ela conta naquele
/// ficheiro (`"{} items"`, `"Search"`) é língua de verdade — está na dívida.
const NOT_LANGUAGE: &[(&str, &str)] = &[(
    "widget/showcase",
    "A GALERIA DE WIDGETS e' uma BANCADA, nao uma superficie de produto -- o subtitulo dela di-lo: \
     «reference for peripheral agents». Os rotulos sao os NOMES DOS NOSSOS WIDGETS (`Rect2Editor`, \
     `BitmaskGrid32`, `VariantEditor (recursive, depth <=4)`) e o conteudo de amostra que os \
     demonstra. Traduzir o nome de um tipo nosso nao e' i18n, e' ruido -- e a bancada tem o mesmo \
     estatuto do `widget-lab`, que ate' a aparencia forca para o redesenho, «porque e' onde ele se \
     estuda».",
)];

/// ⛔ **A DÍVIDA, contada pela régua lexical em 2026-09-13** — `(ficheiro relativo a src/, literais)`.
///
/// ⚠️ **Ela SÓ DESCE.** O número é o que a régua mede hoje; um ficheiro que ganhe um literal reprova
/// no `every_label_this_crate_paints_comes_from_the_string_table`, e um que perca reprova no
/// `the_debt_only_describes_what_is_still_there` até alguém escrever o número novo. ⚠️ E ela ainda
/// não foi TRIADA: parte do que conta pode não ser língua (os nomes da cena de amostra do
/// `fixture.rs`, por exemplo) — a triagem move essas para `NOT_LANGUAGE` com o mecanismo, nunca
/// esconde o número.
///
/// ⛔ **Os números SUBIRAM uma vez, e foi a RÉGUA — não o código** (2026-09-13, na migração do
/// Inspector). A 1.ª redacção do `is_language` descartava toda palavra só em MAIÚSCULAS, e os títulos
/// dos cards do Inspector (`"LEG"`, `"WALK"`) apanharam-na. Aqui ela escondia **46**: o trilho
/// esquerdo (`"BRUSH"`, `"PICK"`… `left_rail.rs` +26), os módulos do topo (`fixture.rs` +13), o
/// `"EDIT"` do HUD (`bottom_hud.rs` +1), os três modelos do seletor de cor (`segmented.rs` +3 — `RGB`,
/// `HSV`, `OKLCH`, que a triagem deve mover para `NOT_LANGUAGE`: são o mesmo símbolo em toda língua)
/// e os três modos do `paint.rs` (entrada nova). ⚠️ E a MESMA jornada corrigiu a régua uma segunda
/// vez — uma BARRA numa frase não é um caminho —, o que descobriu mais **3**: o `grid_snap/inspect.rs`
/// (`"Line / Neighbors"`, `"{} cells / {}"`) e o `bottom_hud.rs` (`"{}\u{2192}{} ev/stamp"`).
/// *Uma correcção da medição, nunca licença para crescer*: a partir daqui os números voltam a só
/// descer.
///
/// ✅ **2026-09-16 (`line/UIUX`): os MENUS e a BARRA DE FERRAMENTAS saíram — 311 literais em nove
/// ficheiros** (`menu_rows` 164 · `ids/menus_timeline` 50 · `left_rail` 71+3 · barra de menus,
/// paleta, radial, fila horizontal, cabeçalho da paleta). As tabelas passaram a guardar `TextKey`
/// (`ids::MenuRow`, `left_rail::RailTool`) e o texto sai de `ph2d-i18n/src/chrome_menus.rs` e
/// `chrome_rail.rs`. ⚠️ O `left_rail` tinha SUBIDO de 71 para 74 nesse dia, e foi a RÉGUA outra
/// vez: `"C&F"` passou a contar como língua (`B&W` da pilha de ajustes).
const DIVIDA: &[(&str, usize)] = &[
    ("floating_panel.rs", 13),
    ("grid_snap/inspect.rs", 21),
    ("grid_snap/state.rs", 9),
    ("interaction/dispatch/hierarchy.rs", 3),
    ("interaction/drag_payload.rs", 2),
    ("interaction/state/blender_ops.rs", 3),
    ("interaction/state/chrome_ops.rs", 1),
    ("interaction/state/store_core.rs", 1),
    ("paint.rs", 3),
    ("panel/registry.rs", 1),
    ("screens/hero.rs", 1),
    ("screens/hero/asset_drag_ghost.rs", 1),
    ("screens/hero/bottom_hud.rs", 12),
    ("screens/hero/canvas.rs", 2),
    ("screens/hero/chrome/fill_modal.rs", 3),
    ("screens/hero/color_picker_demo.rs", 1),
    ("screens/hero/context_menu_dialogs.rs", 9),
    ("screens/hero/fixture.rs", 26),
    ("screens/hero/inspector_model.rs", 1),
    ("screens/hero/inspector_model_anchor.rs", 3),
    ("screens/hero/inspector_model_instance.rs", 15),
    ("screens/hero/pre_populate.rs", 17),
    ("screens/hero/pre_populate_blender.rs", 2),
    ("screens/hero/prefab_bar.rs", 4),
    ("screens/hero/topbar/chip_name.rs", 35),
    ("screens/hero/topbar/cluster_painter.rs", 6),
    ("screens/hero/topbar/tooltips.rs", 28),
    ("screens/task_layout.rs", 6),
    ("widget/blender_color_picker/harmony.rs", 8),
    ("widget/blender_color_picker/paint.rs", 24),
    ("widget/blender_color_picker/palette.rs", 3),
    ("widget/blender_color_picker/segmented.rs", 4),
    ("widget/blender_color_picker/state.rs", 1),
    ("widget/color_picker.rs", 11),
    ("widget/combobox.rs", 1),
    ("widget/dropdown/mod.rs", 1),
    ("widget/key_value_list.rs", 2),
    ("widget/variant_editor.rs", 9),
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

/// `ficheiro -> ["linha: texto", …]`, fora das excepções.
fn per_file() -> BTreeMap<String, Vec<String>> {
    let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for l in language_literals(&src_root()) {
        if NOT_LANGUAGE.iter().any(|(p, _)| l.rel.starts_with(p)) {
            continue;
        }
        out.entry(l.rel)
            .or_default()
            .push(format!("{}: {:?}", l.line, l.text));
    }
    out
}

/// ⭐⭐⭐ **Nenhum ficheiro desta crate escreve mais língua no fonte do que a dívida dele.**
#[test]
fn every_label_this_crate_paints_comes_from_the_string_table() {
    let mut intrusos = Vec::new();
    for (rel, hits) in &per_file() {
        let allowed = DIVIDA.iter().find(|(f, _)| f == rel).map_or(0, |(_, n)| *n);
        if hits.len() > allowed {
            intrusos.push(format!(
                "{rel} — {} literais, a dívida tolerada é {allowed}:\n      {}",
                hits.len(),
                hits.join("\n      ")
            ));
        }
    }
    assert!(
        intrusos.is_empty(),
        "texto com cara de língua escrito no fonte da ph2d-editor-core, acima da dívida (HR-15):\n  \
         {}\n\nA cura é uma chave na tabela (`crates/ph2d-i18n/src/chrome.rs` ou o irmão do assunto) \
         e um `ph2d_i18n::tr(\"…\")` no sítio. ⛔ Nunca subir o número da DIVIDA. ⚠️ Se o texto NÃO \
         é língua, a cura é uma linha em `NOT_LANGUAGE` **com o mecanismo**.",
        intrusos.join("\n  ")
    );
}

/// ⭐ **A METADE JUSTA das excepções** — e o controlo de vacuidade: uma régua partida devolve zero
/// literais e lê-se como aprovada; a bancada, que tem ~95, não.
#[test]
fn every_named_exception_still_shelters_a_real_literal() {
    let all = language_literals(&src_root());
    for (path, why) in NOT_LANGUAGE {
        assert!(
            why.len() > 40,
            "a excepção `{path}` não diz o mecanismo — uma lista sem mecanismo é uma licença"
        );
        let n = all.iter().filter(|l| l.rel.starts_with(path)).count();
        assert!(
            n >= 20,
            "a excepção `{path}` abriga {n} literais: ou a bancada saiu (apague a linha) ou a régua \
             ficou cega (ela tinha ~95 em 2026-09-13)"
        );
    }
}

/// ⛔⛔ **A METADE QUE IMPEDE A LICENÇA: a dívida desceu — escreva o número novo.**
///
/// Uma catraca sem censo de obsolescência não desce, vira licença (`CLAUDE.md` §5.0): se um ficheiro
/// passou de `13` para `9` e a linha continua a dizer `13`, os `4` que alguém curou voltam a caber
/// ali sem que nenhum gate acorde.
#[test]
fn the_debt_only_describes_what_is_still_there() {
    let found = per_file();
    let obsoletas: Vec<String> = DIVIDA
        .iter()
        .filter_map(|(rel, n)| {
            let now = found.get(*rel).map_or(0, Vec::len);
            (now < *n).then(|| {
                if now == 0 {
                    format!("{rel}: a dívida diz {n}, a régua conta 0 — apague a linha")
                } else {
                    format!("{rel}: a dívida diz {n}, a régua conta {now} — escreva {now}")
                }
            })
        })
        .collect();
    assert!(
        obsoletas.is_empty(),
        "a dívida DESCEU e a catraca não desceu com ela:\n  {}",
        obsoletas.join("\n  ")
    );
    let mut names: Vec<&str> = DIVIDA.iter().map(|(f, _)| *f).collect();
    let listed = names.len();
    names.sort_unstable();
    names.dedup();
    assert_eq!(
        names.len(),
        listed,
        "um ficheiro aparece duas vezes na DIVIDA — uma das contagens fica escondida"
    );
    assert!(
        DIVIDA.iter().all(|(_, n)| *n > 0),
        "uma entrada a 0 vale o mesmo que a ausência — apague-a"
    );
}

/// ⭐⭐⭐ **UMA CHAVE COM ERRO DE ESCRITA PINTA O IDENTIFICADOR CRU NA TELA** — e vaza a string, por
/// quadro (`leak_key`). E o censo é dos DOIS lados: uma chave **usada e não declarada** pinta o
/// identificador; uma **declarada e não usada** é uma órfã.
#[test]
fn every_chrome_key_exists_on_both_sides() {
    const PREFIX: &str = "chrome.";
    // ⚠️ **Três tabelas, um prefixo** (2026-09-16): os menus e a barra de ferramentas moram em
    // irmãs do `chrome.rs` (isolamento entre linhas, `CLAUDE.md` §0.2), e as chaves continuam
    // `chrome.*`. Com uma tabela só aqui, as 244 chaves novas liam-se «sem tradução».
    const TABLES: &[&str] = &[
        "crates/ph2d-i18n/src/chrome.rs",
        "crates/ph2d-i18n/src/chrome_menus.rs",
        "crates/ph2d-i18n/src/chrome_rail.rs",
    ];
    let repo = repo_root();
    let used = keys::keys_used(&repo, PREFIX, TABLES);
    let declared = keys::keys_declared(&repo, TABLES, PREFIX);
    // ⛔ Controlo de vacuidade: um caminho errado dá dois conjuntos vazios, que concordam.
    assert!(
        declared.len() >= 10 && used.len() >= 10,
        "o censo achou {} declaradas e {} usadas — está a ler o sítio errado, e dois conjuntos \
         vazios concordam sempre",
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
        "estas chaves são usadas e NÃO existem em {TABLES:?} — o `tr` faz `leak_key` e pinta o \
         identificador cru na tela, um vazamento por quadro:\n  {}",
        sem_traducao.join("\n  ")
    );
    let orfas: Vec<&String> = declared.iter().filter(|k| !used.contains_key(*k)).collect();
    assert!(
        orfas.is_empty(),
        "estas chaves estão na tabela e ninguém as usa — apague-as: uma string órfã é onde alguém \
         escreve, um dia, uma frase sobre um controlo que já não existe:\n  {orfas:?}"
    );
}
