//! ⭐⭐⭐ **Toda FILA de edições do Inspector chega à porta que a APLICA** — o censo da cadeia da
//! shell (plano 30, W4).
//!
//! Uma edição de painel atravessa a shell em três elos: o barramento empurra-a para uma fila do
//! [`DrainOut`](../../src/render_loop/fase_bus_drain_out.rs) · o `fase_hero_commits` tira a fila
//! (`take(&mut pd.X_edits)`) para as intenções do quadro · a fase de commits passa a fatia à porta
//! da família que a escreve no mundo.
//!
//! ⛔⛔ **Dois destes elos são TIPOS e o compilador guarda-os; os outros dois NÃO:** um
//! `X_edits: Vec::new()` no lugar do `take` compila e deixa a fila a crescer para sempre, e um
//! parâmetro da `aplicar` que nenhuma chamada lê compila (com `_` à frente, nem avisa). Medido na
//! W4: apagar a linha `nav_inspector::apply_all(sim, nav)` deixava a suíte inteira VERDE — os
//! gates da ponte entram pela porta da família directamente, ABAIXO da costura da shell.
//!
//! ⇒ **as duas metades são DERIVADAS do fonte**, nunca de uma lista escrita à mão: uma fila nova
//! que não chegue ao `take`, ou um parâmetro novo que não chegue a uma chamada, reprovam aqui sem
//! ninguém se lembrar de os acrescentar. ⚠️ Com **piso de população**, porque um padrão que deixe
//! de casar (um renome do `pub(in crate::render_loop)`, a assinatura partida pelo `fmt`) lê `0` de
//! `0` e fica verde a medir nada.

use std::path::Path;

/// As filas, lidas do fonte do `DrainOut`.
const DRAIN_OUT: &str = include_str!("../../src/render_loop/fase_bus_drain_out.rs");
/// A fase que aplica as famílias do TOP-20 em diante.
const TOP20: &str = include_str!("../../src/render_loop/fase_inspector_commits_top20.rs");

/// ⭐ Tira os comentários de linha — a prosa que EXPLICA um elo não pode satisfazer a agulha que
/// o procura (a forma que este repo já pagou em mais de um censo).
fn sem_prosa(src: &str) -> String {
    src.lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// As filas `X_edits` que o `DrainOut` declara.
fn filas() -> Vec<String> {
    let src = sem_prosa(DRAIN_OUT);
    let agulha = "pub(in crate::render_loop) ";
    let mut out = Vec::new();
    for linha in src.lines() {
        let Some(resto) = linha.trim_start().strip_prefix(agulha) else {
            continue;
        };
        let Some((nome, _)) = resto.split_once(':') else {
            continue;
        };
        if nome.ends_with("_edits") && nome.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            out.push(nome.to_string());
        }
    }
    out
}

/// O texto de todos os `.rs` do `render_loop` (sem prosa) — quem tira uma fila pode morar numa
/// fase-filha, e o censo não pode adivinhar qual.
fn render_loop_inteiro() -> String {
    fn varre(dir: &Path, out: &mut String, n: &mut usize) {
        let Ok(entradas) = std::fs::read_dir(dir) else {
            return;
        };
        for e in entradas.flatten() {
            let p = e.path();
            if p.is_dir() {
                varre(&p, out, n);
            } else if p.extension().is_some_and(|x| x == "rs")
                && let Ok(s) = std::fs::read_to_string(&p)
            {
                out.push_str(&sem_prosa(&s));
                out.push('\n');
                *n += 1;
            }
        }
    }
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/render_loop");
    let mut out = String::new();
    let mut n = 0usize;
    varre(&raiz, &mut out, &mut n);
    assert!(
        n >= 100,
        "o render_loop leu {n} ficheiros — a varredura partiu e mediria o nada"
    );
    out
}

/// ⭐⭐ **Toda fila do `DrainOut` é TIRADA** — senão as edições dela acumulam-se e nunca chegam.
///
/// **Mutação que deve sangrar:** `nav_edits: Vec::new()` no lugar do `take` do `fase_hero_commits`.
#[test]
fn toda_fila_do_drain_out_e_tirada_por_um_take() {
    let filas = filas();
    // ⚠️ `40 → 39` em 2026-10-05: a fila do catavento (`mesh3d_edits`) saiu com o 3D (ADR-0179).
    assert!(
        filas.len() >= 39,
        "o DrainOut declara {} filas `_edits` — medido 39 desde a poda do 3D; um padrão que deixou de casar \
         lê-se igual a «todas drenadas»",
        filas.len()
    );
    let tudo = render_loop_inteiro();
    let orfas: Vec<&String> = filas
        .iter()
        .filter(|f| !tudo.contains(&format!("take(&mut pd.{f})")))
        .collect();
    assert!(
        orfas.is_empty(),
        "estas filas do DrainOut nunca são tiradas — as edições delas ficam presas no quadro: \
         {orfas:?}"
    );
}

/// ⭐⭐⭐ **Todo parâmetro de fatia da `aplicar` do TOP-20 chega a uma chamada** — senão a secção
/// pinta, o clique empurra, a fila drena, e o mundo nunca muda.
///
/// **Mutação que deve sangrar:** apagar `| ph2d_app_components::nav_inspector::apply_all(sim, nav)`.
#[test]
fn todo_parametro_da_aplicacao_chega_a_uma_porta() {
    let src = sem_prosa(TOP20);
    let i = src
        .find("pub(super) fn aplicar(")
        .expect("a `aplicar` do TOP-20 existe");
    let j = i + src[i..].find(") -> bool {").expect("a assinatura fecha");
    // ⚠️ `"\n}"` e não `"\n}\n"`: o `sem_prosa` não deixa a quebra final, e a `aplicar` pode ser
    // o último item do ficheiro.
    let fim = j + src[j..].find("\n}").expect("o corpo fecha");
    let assinatura = &src[i..j];
    let corpo = &src[j..fim];
    let params: Vec<&str> = assinatura
        .lines()
        .filter_map(|l| {
            let (nome, tipo) = l.trim().split_once(':')?;
            (tipo.trim_start().starts_with("&[") && !nome.is_empty()).then_some(nome)
        })
        .collect();
    // ⚠️ `19 → 18` em 2026-10-05: o parâmetro do catavento (`mesh3d`) saiu com o 3D (ADR-0179).
    assert!(
        params.len() >= 18,
        "a `aplicar` tem {} parâmetros de fatia — medido 18 desde a poda do 3D",
        params.len()
    );
    assert!(
        params.contains(&"nav"),
        "a fila da NAVEGAÇÃO entra na `aplicar`"
    );
    let mudos: Vec<&&str> = params
        .iter()
        .filter(|p| !corpo.contains(&format!("(sim, {p})")))
        .collect();
    assert!(
        mudos.is_empty(),
        "estes parâmetros da `aplicar` não chegam a porta nenhuma — o painel edita e o mundo não \
         muda: {mudos:?}"
    );
}
