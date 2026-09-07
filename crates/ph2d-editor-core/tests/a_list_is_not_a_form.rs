//! ⭐⭐⭐ **Uma LISTA não é um formulário — e o vão entre as suas linhas é OUTRO número.**
//!
//! Um formulário empilha controlos independentes (`label | control`), e o vão diz *estes são dois
//! assuntos*. Uma lista empilha os itens **de uma coisa só** — as camadas, os objectos da cena, as
//! variações de um som — e ali o vão diz o contrário: *isto é um corpo*. É a lei do grupo de
//! botões (wave 10) virada na vertical.
//!
//! ⛔ **Censo de 2026-09-06, antes desta wave: cinco listas, QUATRO respostas.**
//!
//! | superfície | altura | vão escrito |
//! |---|---|---|
//! | hierarquia | `HIER_ROW_H` = **32** (token só dela) | `Spacing::Xxs` = 2 |
//! | variações do áudio | `VAR_ROW_H` = 22 à mão | `Spacing::Xs` = 4 |
//! | inspector da grade | `ROW_H` = 22 à mão | `ROW_GAP` = 2 (const local) |
//! | âncoras do Inspector | `ROW_H` = 22 à mão | **0** (implícito) |
//! | animações do Inspector | `ROW_H` = 22 à mão | **0** (implícito) |
//!
//! ⚠️⚠️ **A lição está nas duas últimas: a resposta certa já estava escrita DUAS vezes e não era
//! alcançável.** Elas avançam `cur_y += ROW_H` — sem vão nenhum, que é praticamente o que o modelo
//! manda — e como isso é a **ausência** de um termo, nenhuma varredura por operador o vê, nenhum
//! doc o afirma e ninguém o podia copiar. *Uma lei escrita em dois sítios ainda não é uma lei; só
//! uma PORTA é.*
//!
//! ⭐ **E há uma SEGUNDA lei de lista aqui** (wave 18, no mesmo dia): quando as linhas encostam, o
//! que devolve *onde acaba uma e começa a outra* deixa de ser o vão e passa a ser o TOM — a
//! alternância par/ímpar do *Outliner* do Blender, que o dono apontou. Ver
//! [`every_list_stripes_its_rows_or_says_why_not`].
//!
//! **O número é derivado** (Godot Modern, MIT, `theme_modern.cpp:650`), e a derivação vive na
//! porta [`ph2d_tokens::list_row_gap_px`]. ⚠️ **E ela corrigiu uma afirmação minha:** a wave 8
//! registou `Tree.v_separation = pow(base·0.175,3) = 0` truncando `0,7³`; o `EDSCALE_RND`
//! **arredonda primeiro**, e o cubo é de `1`. *Uma derivação copiada sem se avaliar a expressão
//! inteira é um número escolhido com cara de lei.*

use std::fs;
use std::path::{Path, PathBuf};

/// `(caminho relativo à raiz do repo, constante de altura da linha da lista)`.
///
/// ⚠️ **É uma lista DECLARADA de propósito** — «isto é uma lista» é um facto de produto que
/// nenhum parser lê do fonte: uma pilha de linhas homogéneas e uma pilha de controlos diferentes
/// escrevem-se com o mesmo `y += h + g`. O que o gate impede é a superfície declarada **responder
/// à pergunta sozinha**, e a metade de baixo impede a lista de envelhecer.
const LIST_SURFACES: &[(&str, &str, &str)] = &[
    ("crates/ph2d-panel-hierarchy/src/paint.rs", "HIER_ROW_H", ""),
    (
        "crates/ph2d-panel-inspector/src/sections/anchors.rs",
        "ROW_H",
        "",
    ),
    (
        "crates/ph2d-panel-inspector/src/sections/anim_rows.rs",
        "ROW_H",
        "",
    ),
    (
        "crates/ph2d-panel-audio-editor/src/paint_variation.rs",
        "VAR_ROW_H",
        "TODA linha ja' enche o proprio fundo (`Bg3`, ou `Accent` na seleccionada): uma listra por \
         baixo de um fundo opaco nao se ve^, e trocar o fundo de sempre por uma alternancia e' \
         mudar o look daquela lista, nao aplicar-lhe a lei",
    ),
    (
        "crates/ph2d-editor-core/src/grid_snap/inspect.rs",
        "ROW_H",
        "e' um BLOCO DE LEITURA rotulo/valor de 5 linhas fixas, nao uma lista de itens \
         escolhiveis: alternar tons ali desenha uma tabela onde nao ha' uma",
    ),
];

/// Os degraus da escada que alguém poderia somar à altura de uma linha de lista.
const RUNGS: &[&str] = &["Xxs", "Xs", "Sm", "Md", "Lg"];

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

fn read(rel: &str) -> Option<String> {
    fs::read_to_string(Path::new(&repo_root()).join(rel)).ok()
}

/// ⭐ **Nenhuma lista escolhe o próprio vão — ela chama a porta.**
///
/// **Mutação que deve sangrar:** repor `y += VAR_ROW_H + Spacing::Xs.px();` no
/// `paint_variation.rs` — é exactamente o estado em que esta wave encontrou o app.
#[test]
fn no_list_surface_writes_its_own_row_gap() {
    let mut offenders = Vec::new();
    for (rel, row_h, _) in LIST_SURFACES {
        let Some(src) = read(rel) else {
            continue; // a metade de baixo acusa o caminho morto
        };
        if !src.contains("list_row_gap_px") {
            offenders.push(format!(
                "{rel}: e' uma LISTA e nunca chama `ph2d_tokens::list_row_gap_px()`"
            ));
        }
        for (n, line) in src.lines().enumerate() {
            let t = line.trim_start();
            if t.starts_with("//") {
                continue;
            }
            // ⚠️ **Só o AVANÇO conta (`+=`), e a distinção é estrutural, não estética:**
            // `y += altura + rung` anda para a linha seguinte — a pergunta desta wave; um
            // `return y + altura + rung` **sai** da lista, que é *quanto ar fica DEPOIS dela* — a
            // pergunta aberta nº 4 do handoff (o fim de um grupo, `Md` em 4 sítios e `Lg` em 1
            // contra o `base·2` = 8 do Godot). ⭐ **A 1.ª redacção deste censo não as separava, e
            // por isso apanhou um sítio a sério**: o `paint_variation.rs` fechava a MESMA lista
            // com `Sm` no braço vazio e `Xs` no cheio. *Uma régua larga demais ainda acusa
            // verdades — só não é sobre elas que ela fala.*
            if !t.contains("+=") {
                continue;
            }
            let needle = format!("{row_h} + Spacing::");
            if let Some(at) = line.find(&needle) {
                let tail = &line[at + needle.len()..];
                if RUNGS.iter().any(|r| tail.starts_with(r)) {
                    offenders.push(format!("{rel}:{}: {}", n + 1, line.trim()));
                }
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "{} sitio(s) respondem sozinhos «quanto avanca uma linha de lista» em vez de chamar \
         `ph2d_tokens::list_row_gap_px()`. Cada um e' a segunda resposta a uma pergunta que ja \
         tem uma:\n  {}",
        offenders.len(),
        offenders.join("\n  ")
    );
}

/// ⛔ **A metade que impede a lista de virar licença** — um alvo que já não existe, ou que mudou
/// de nome, sai daqui em vez de ficar a descrever nada. (A catraca de 30/08: *uma lista tolerada
/// sem censo de obsolescência não desce — ela vira licença.*)
#[test]
fn the_declared_list_surfaces_still_exist() {
    let mut stale = Vec::new();
    for (rel, row_h, _) in LIST_SURFACES {
        match read(rel) {
            None => stale.push(format!("{rel}: o ficheiro nao existe")),
            Some(src) => {
                if !src.contains(&format!("const {row_h}")) {
                    stale.push(format!("{rel}: nao declara `const {row_h}`"));
                }
            }
        }
    }
    assert!(
        stale.is_empty(),
        "{} entrada(s) de LIST_SURFACES ja nao descrevem nada:\n  {}",
        stale.len(),
        stale.join("\n  ")
    );
}

/// ⭐⭐ **O vão de uma lista é MENOR que o de um formulário, e não é zero.**
///
/// ⚠️ **Deliberadamente NÃO re-escrevo a expressão da porta aqui** — comparar `list_row_gap_px()`
/// com `(Spacing::Xs.px()*0.175).round().powi(3)` seria calcular a mesma conta e compará-la
/// consigo própria, que é o gate vácuo que esta jornada já pagou três vezes. O que se afirma são
/// as duas propriedades que o número TEM DE ter, e as duas são derivadas:
///
/// - **maior que zero** — dois itens seleccionados em seguida têm de continuar a ler-se como dois;
/// - **menor que o vão de formulário** — é a frase inteira desta wave.
///
/// (Hoje avalia em **1 px**, o mesmo fio de que uma peça de grupo é feita.)
#[test]
fn a_list_breathes_less_than_a_form_and_still_breathes() {
    let list = ph2d_tokens::list_row_gap_px();
    let form = ph2d_tokens::row_gap_px();
    assert!(
        list > 0.0,
        "o vao de lista e' {list} — a zero duas linhas seleccionadas fundem-se numa"
    );
    assert!(
        list < form,
        "o vao de lista ({list}) nao e' menor que o de formulario ({form}) — entao a lista nao \
         ficou mais compacta que o formulario, que e' a wave inteira"
    );
}

/// ⭐⭐⭐ **A LISTRA: toda lista alterna o tom das linhas, ou diz porque não.**
///
/// Enio, 2026-09-06, com a foto do *Outliner* do Blender: *«linhas pares e ímpares têm tonalidade
/// discretamente diferente».* É o companheiro da lei de cima: quando as linhas encostam, o que
/// devolve *onde acaba uma e começa a outra* deixa de ser o vão e passa a ser o tom.
///
/// ⚠️ **As duas isenções são de PRODUTO e estão medidas**, não são «ainda não fizemos»: uma lista
/// cujas linhas já têm fundo opaco não tem onde pôr uma listra, e um bloco de leitura de cinco
/// linhas fixas não é uma lista de itens. Cada uma escreve o mecanismo ao lado.
///
/// **Mutação que deve sangrar:** apagar a chamada ao `paint_row_stripe` de qualquer uma das três.
#[test]
fn every_list_stripes_its_rows_or_says_why_not() {
    let mut offenders = Vec::new();
    for (rel, _, why_not) in LIST_SURFACES {
        let Some(src) = read(rel) else { continue };
        let stripes = src.contains("paint_row_stripe");
        if stripes && !why_not.is_empty() {
            offenders.push(format!(
                "{rel}: chama `paint_row_stripe` E declara um motivo para nao o fazer — o motivo \
                 ja' nao descreve nada:\n      «{why_not}»"
            ));
        }
        if !stripes && why_not.is_empty() {
            offenders.push(format!(
                "{rel}: e' uma LISTA, nao alterna o tom das linhas e nao diz porque nao"
            ));
        }
    }
    assert!(
        offenders.is_empty(),
        "{} superficie(s) de lista fora da lei da listra:\n  {}",
        offenders.len(),
        offenders.join("\n  ")
    );
}
