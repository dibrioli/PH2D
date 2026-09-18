//! ⭐⭐⭐ **NENHUMA PALAVRA DESTA CRATE É ESCRITA NO FONTE** — o HR-15 no DESIGN SYSTEM.
//!
//! ⛔⛔ **Esta crate nunca teve régua, e os dois painéis que a pintam fecham a ZERO.** O
//! `ph2d-panel-tokens` e o `ph2d-panel-widget-gallery` não têm um único literal — e os **nomes dos
//! oito temas** da barra do topo, os **quatro desenhos de slider** com as quatro linhas de ajuda e
//! as duas recusas de fórmula viviam aqui. *Um censo cuja crate não é DONA do texto que ela pinta
//! fica verde sobre texto cru* — a mesma forma do `component_catalog`, do `paint_brush` e do motor
//! da escultura, agora na folha de que 44 widgets dependem.
//!
//! ⚠️⚠️ **E a régua do repo não a alcança por construção:** o `--resumo` do
//! `ph2d-label-census` varre os `ph2d-panel-*`, as `ph2d-app-*`, a `ph2d-editor-core`, a
//! `ph2d-param-editors` e a shell. *Uma lista de crates a varrer é exactamente onde a próxima
//! fronteira se esconde* — quem achou esta foi o **idioma de teste**, na 2.ª fotografia do dono.
//!
//! ⚠️ Esta crate **não resolve** o texto (ela declara-se *design-data puro, zero runtime deps* e há
//! gate a afirmá-lo): ela guarda a CHAVE, e quem a resolve é o painel.

use ph2d_label_census::gate::{self, Excecao};

// ⚠️ **`design.` e não `tokens.`, e a razão está MEDIDA:** com o prefixo largo o censo colhia
// `tokens.close`, `tokens.panel`, `tokens.dtcg.export` e `tokens.json` — que são **ids de widget**
// e um nome de FICHEIRO, não chaves de i18n. *Um prefixo que casa com o espaço de nomes de outra
// coisa mede a outra coisa.*
const PREFIX: &str = "design.";
const TABLES: &[&str] = &["crates/ph2d-i18n/src/tokens.rs"];

/// ⭐ As excepções, **com o mecanismo** — nunca uma lista aberta.
const NOT_LANGUAGE: &[Excecao] = &[
    (
        "contrast.rs",
        "WCAG 2.2 AA 1.4.3",
        "o número de CLÁUSULA de uma norma (texto de contraste), não uma frase: ele é o mesmo \
         símbolo em toda língua, como um número de ADR — traduzi-lo tornaria a referência \
         inseguível",
    ),
    (
        "contrast.rs",
        "WCAG 2.2 AA 1.4.11",
        "o número de CLÁUSULA de uma norma (contraste de componentes não-textuais) — ver a irmã \
         acima: é uma referência normativa, não texto de interface",
    ),
    (
        "typography.rs",
        "Inter, -apple-system, BlinkMacSystemFont, 'SF Pro Text', system-ui, sans-serif",
        "uma PILHA DE FAMÍLIAS DE FONTE ao estilo CSS, lida pelo sistema de texto para escolher um \
         ficheiro no disco — os nomes são identificadores de fonte instalada, e traduzir um deles \
         faria a escolha falhar e cair no fallback",
    ),
    (
        "typography.rs",
        "'Inter Display', Inter, system-ui, sans-serif",
        "pilha de famílias de fonte — ver a irmã acima; o nome de uma fonte é o que o sistema \
         procura no disco, nunca o que o artista lê",
    ),
    (
        "typography.rs",
        "'JetBrains Mono', ui-monospace, 'SF Mono', Menlo, monospace",
        "pilha de famílias de fonte monoespaçada — ver as irmãs acima; identificador de ficheiro \
         instalado, não interface",
    ),
];

#[test]
fn cada_palavra_desta_crate_vem_da_tabela() {
    let (src, _) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let intrusos = gate::intrusos(&src, NOT_LANGUAGE);
    assert!(
        intrusos.is_empty(),
        "estes textos com cara de língua estão escritos no fonte do design system e nunca chegam \
         à tabela de strings (HR-15):\n  {}\n\nA cura é uma chave `{PREFIX}<família>.<nome>` em \
         `{TABLES:?}` e um `label_key()`/`display_name_key()` que a devolva — ⛔ **nunca um `tr` \
         aqui dentro**: esta crate declara-se sem dependência de runtime e há gate \
         (`the_leaf_stays_dep_free`). ⚠️ Se o texto NÃO é língua, a cura é uma linha em \
         `NOT_LANGUAGE` **com o mecanismo**.",
        intrusos.join("\n  ")
    );
}

/// ⭐ A metade justa — e o controlo de vacuidade: uma régua cega não acha as excepções.
#[test]
fn cada_excecao_nomeada_ainda_abriga_um_literal() {
    let (src, _) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let mortas = gate::excecoes_mortas(&src, NOT_LANGUAGE);
    assert!(
        mortas.is_empty(),
        "excepções sem mecanismo ou sem literal:\n  {}",
        mortas.join("\n  ")
    );
}

/// ⭐⭐ **Uma chave com erro de escrita pinta o identificador cru na tela** — o censo dos DOIS lados.
#[test]
fn cada_chave_desta_crate_existe_dos_dois_lados() {
    let (_, repo) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let c = gate::chaves(&repo, PREFIX, TABLES);
    // ⛔ Controlo de vacuidade: um caminho errado dá dois conjuntos vazios, que concordam sempre.
    assert!(
        c.declaradas >= 16 && c.usadas >= 16,
        "o censo achou {} declaradas e {} usadas — o piso é 16 e dois conjuntos vazios concordam \
         sempre",
        c.declaradas,
        c.usadas
    );
    assert!(
        c.sem_traducao.is_empty(),
        "usadas e NÃO declaradas em {TABLES:?} — o `tr` faz `leak_key` e pinta o identificador \
         cru:\n  {}",
        c.sem_traducao.join("\n  ")
    );
    assert!(
        c.orfas.is_empty(),
        "declaradas e sem quem as use — apague-as:\n  {:?}",
        c.orfas
    );
}
