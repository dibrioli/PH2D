//! ⭐⭐⭐ **Toda secção que pinta uma linha de formulário RESERVA a coluna de animação.**
//!
//! Report do Enio (2026-09-03, com foto da secção Transform): *«várias não receberam pontos»*.
//!
//! # ⛔ Porque isto é um gate e não uma lista de tarefas minha
//!
//! O censo da pesquisa `07` §15.1 contou **pintores** — `paint_slider_with_chip` e
//! `paint_checkbox` — e por isso não viu esta família: linhas construídas **à mão dentro do
//! painel**, cada uma com a sua aritmética de larguras. Só o Inspector tem ~15 ficheiros assim.
//!
//! ⚠️ Uma promessa de *«faço as restantes a seguir»* não sobrevive a uma janela de contexto. Isto
//! sobrevive: a lista **só encolhe**, e uma secção NOVA que pinte um controlo de formulário sem
//! reservar a coluna **não compila verde**.
//!
//! # A metade que impede isto de virar licença
//!
//! ⛔ *Uma catraca sem censo de obsolescência não desce: vira licença* (`CLAUDE.md` §5.0). Por isso
//! há **dois** testes: um exige que ninguém entre na lista sem estar lá; o outro exige que quem já
//! reserva a coluna **saia** dela.

use std::fs;
use std::path::PathBuf;

/// Os pintores que denunciam «isto é uma linha de formulário».
const ROW_PAINTERS: &[&str] = &[
    "paint_number_input_with_buffer",
    "paint_segmented_adaptive",
    "paint_dropdown_chip",
    "paint_color_swatch",
    "paint_text_input_with_buffer",
];

/// As portas de BASE: as duas do `ph2d-editor-core` que de facto reservam a coluna.
///
/// ⚠️ A segunda COMPÕE com a primeira — a `widget::property_row_columns` (rótulo à esquerda,
/// controlo à direita) chama a `form_row_columns` por dentro e devolve o mesmo `dot`.
const BASE_DOORS: [&str; 2] = ["form_row_columns", "property_row_columns"];

/// ⭐⭐⭐ **As portas DERIVAM-SE do `rows.rs`, nunca se escrevem à mão — e isto é a TERCEIRA
/// correcção da mesma forma.**
///
/// ⛔⛔ Esta lista já foi de UM nome (`form_row_columns`), depois de dois (a linha de propriedade
/// nasceu em 2026-09-14 e o gate acusou quem a adoptou), e em 2026-09-15 acusou **três** secções
/// — `material_blend`, `sampling`, `slice_nine` — pelo mesmo motivo: elas passaram a chamar
/// `rows::property_label_row`, que chama `property_row_columns` por dentro. *Uma régua que
/// enumera portas à mão acusa, a cada wave, exactamente quem fez a coisa certa.*
///
/// ⇒ Uma porta de 2.ª ordem é **toda `fn` de `sections/rows.rs` cujo corpo chama uma de base**, e
/// quem a chama reserva a coluna por composição. ⛔ Sem `--write` e sem lista: o produto é a fonte.
///
/// ⚠️ **O modo de falha é ALTO, de propósito:** se o parse partir, a lista encolhe para as duas de
/// base e o gate passa a acusar as secções que usam `rows.rs` — vermelho em voz alta, nunca verde
/// a medir nada.
fn doors() -> Vec<String> {
    let mut out: Vec<String> = BASE_DOORS.iter().map(|d| (*d).to_string()).collect();
    let rows = fs::read_to_string(sections_dir().join("rows.rs")).expect("rows.rs legível");
    let fns: Vec<(String, String)> = rows
        .split("\npub(super) fn ")
        .skip(1)
        .filter_map(|bloco| {
            let (nome, corpo) = bloco.split_once('(')?;
            Some((nome.to_string(), corpo.to_string()))
        })
        .collect();
    // ⚠️ **Ponto FIXO, e não uma passagem só:** a `num_row` não chama porta de base nenhuma — ela
    // delega na `num_row_unit`, que chama. *Uma passagem só deixaria de fora quem está a DOIS
    // saltos*, e a secção que a usasse seria acusada de não reservar a coluna que ela reserva.
    loop {
        let antes = out.len();
        for (nome, corpo) in &fns {
            if !out.iter().any(|d| d == nome) && out.iter().any(|d| corpo.contains(d.as_str())) {
                out.push(nome.clone());
            }
        }
        if out.len() == antes {
            break;
        }
    }
    out
}

/// ⏳ **Dívida MEDIDA, e só encolhe.** Cada ficheiro aqui pinta pelo menos um controlo de
/// formulário e ainda não reserva a coluna — ou seja, as linhas dele aparecem ao artista **sem o
/// ponto**, ao lado de linhas que já o têm.
///
/// ⚠️ **Não acrescente entradas.** Uma secção nova nasce com a porta; é uma chamada.
/// ⭐⭐⭐ **VAZIA — a dívida fechou em 2026-09-03.** Todas as secções que pintam um controlo de
/// formulário reservam a coluna.
///
/// ⚠️ **A lista fica**, e não é cerimónia: ela é o que impede uma secção NOVA de nascer sem a
/// coluna. Um gate cuja lista de excepções desapareceu passa a ser um gate sem excepções — que é
/// exactamente o que se quer — mas só enquanto **alguém não a reintroduzir por conveniência**.
/// ⛔ Não acrescente entradas: uma secção nova nasce com a porta, é uma chamada.
const MISSING_OK: &[&str] = &[];

/// Tira as declarações de `use` — **todas as linhas delas**, até ao `;`.
///
/// ⚠️ **Três formas mordidas, uma a uma:** a 1.ª versão filtrava só a linha que começa por `use `
/// (e um `use` é multi-linha); a 2.ª esqueceu-se de que ele pode vir com **visibilidade**
/// (`pub use`, `pub(crate) use`, `pub(super) use`) — e era assim que o `mod.rs` o escrevia.
/// *Um gate que parseia o fonte tem de saber TODAS as formas do que parseia*, e a que falta é
/// sempre a que o ficheiro acusado usa.
fn strip_use_statements(src: &str) -> String {
    fn opens_a_use(line: &str) -> bool {
        let t = line.trim_start();
        let t = t.strip_prefix("pub").map_or(t, |rest| {
            rest.strip_prefix('(')
                .and_then(|r| r.split_once(')'))
                .map_or(rest, |(_, after)| after)
                .trim_start()
        });
        t.trim_start().starts_with("use ")
    }
    let mut out = Vec::new();
    let mut inside_use = false;
    for line in src.lines() {
        if !inside_use && opens_a_use(line) {
            inside_use = !line.contains(';');
            continue;
        }
        if inside_use {
            inside_use = !line.contains(';');
            continue;
        }
        out.push(line);
    }
    out.join("\n")
}

fn sections_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/sections")
}

/// Os ficheiros que pintam um controlo de formulário, e se já reservam a coluna.
fn census() -> Vec<(String, bool)> {
    let portas = doors();
    let mut out = Vec::new();
    for entry in fs::read_dir(sections_dir()).expect("sections/ existe") {
        let path = entry.expect("entrada legível").path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let src = fs::read_to_string(&path).expect("ficheiro legível");
        // ⛔ **As declarações de `use` não pintam nada.** O `mod.rs` só RE-EXPORTA os pintores e
        // entrou na dívida por falso positivo — uma entrada que a metade da obsolescência **nunca
        // conseguiria limpar**, porque o ficheiro nunca vai chamar a porta. *Um censo que conta a
        // palavra conta também quem só a nomeia.*
        //
        // ⚠️ **E um `use` é MULTI-LINHA.** A 1.ª versão filtrava só a linha que começa por `use `,
        // e o `mod.rs` continuou acusado — o nome que o denunciava estava na **terceira** linha do
        // bloco. *Um gate que parseia o fonte tem de saber todas as formas do que parseia.*
        let painting = strip_use_statements(&src);
        if !ROW_PAINTERS.iter().any(|p| painting.contains(p)) {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .expect("nome utf-8")
            .to_string();
        out.push((name, portas.iter().any(|d| painting.contains(d.as_str()))));
    }
    out.sort();
    out
}

/// **Ninguém pinta uma linha sem reservar a coluna, a não ser a dívida declarada.**
///
/// **Mutação que deve sangrar:** uma secção nova que chame `paint_number_input_with_buffer` sem a
/// porta — as linhas dela nascem sem ponto, ao lado de linhas que o têm, e é exactamente o que o
/// dono reportou.
#[test]
fn every_form_row_reserves_the_animation_column() {
    let missing: Vec<String> = census()
        .into_iter()
        .filter(|(name, has_door)| !has_door && !MISSING_OK.contains(&name.as_str()))
        .map(|(name, _)| name)
        .collect();
    assert!(
        missing.is_empty(),
        "estas seccoes pintam um controlo de formulario e NAO reservam a coluna de animacao \
         (as linhas delas aparecem sem o ponto, ao lado de linhas que o tem): {missing:?}\n\
         cura: `let (control_w, dot) = widget::form_row_columns(x, w, row_y, row_h);` — desenhe \
         dentro de `control_w` e chame `paint_decorator_dot(scene, theme, dot)`."
    );
}

/// ⛔ **A metade que impede a lista de virar licença: quem já reserva a coluna SAI dela.**
#[test]
fn the_debt_list_has_no_stale_entries() {
    let done: Vec<String> = census()
        .into_iter()
        .filter(|(name, has_door)| *has_door && MISSING_OK.contains(&name.as_str()))
        .map(|(name, _)| name)
        .collect();
    assert!(
        done.is_empty(),
        "estas entradas ja' nao descrevem nada — os ficheiros JA' reservam a coluna: {done:?}\n\
         cura: apague-as do MISSING_OK. Uma catraca sem censo de obsolescencia nao desce, vira \
         licenca."
    );
    // E a lista não pode descrever ficheiros que já não existem.
    let names: Vec<String> = census().into_iter().map(|(n, _)| n).collect();
    let ghosts: Vec<&&str> = MISSING_OK
        .iter()
        .filter(|e| !names.contains(&(**e).to_string()))
        .collect();
    assert!(
        ghosts.is_empty(),
        "entradas sobre ficheiros que ja' nao pintam linha nenhuma (ou nao existem): {ghosts:?}"
    );
}

/// ⭐⭐ **O CONTROLO da derivação** — sem ele, um `doors()` que devolvesse só as duas de base
/// deixaria o gate a acusar quem faz a coisa certa, e um que devolvesse *tudo* deixaria o gate a
/// aprovar quem não reserva coluna nenhuma.
///
/// ⚠️ **Piso MEDIDO, não escolhido:** em 2026-09-15 o `rows.rs` tem **cinco** portas de 2.ª ordem
/// (`seg_row` · `num_row_unit` · `num_row` — esta a dois saltos — · `property_label_row` ·
/// `fields_row`). O piso fica em `4` para tolerar um corte honesto e ainda apanhar um parse morto.
#[test]
fn the_door_census_derives_the_second_order_doors() {
    let portas = doors();
    let derivadas: Vec<&String> = portas
        .iter()
        .filter(|d| !BASE_DOORS.contains(&d.as_str()))
        .collect();
    assert!(
        derivadas.len() >= 4,
        "a derivacao leu {} porta(s) de 2.a ordem no rows.rs — o parse dela morreu: {derivadas:?}",
        derivadas.len()
    );
    for esperada in ["num_row", "fields_row", "property_label_row"] {
        assert!(
            portas.iter().any(|d| d == esperada),
            "a derivacao nao achou a porta `{esperada}`: {portas:?}"
        );
    }
}
