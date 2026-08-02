//! **Os helpers dos arch-gates de escultura** — a leitura da fonte.
//!
//! ⚠️ **Um subdiretório, e é a única forma que compartilha sem duplicar.** Todo
//! `.rs` direto em `tests/` vira um binário próprio, então dois gates irmãos que
//! precisem das MESMAS funções ou as copiam — e as cópias divergem — ou moram
//! aqui. É o `tests/common` da convenção do cargo, com nome que diz de quem é.
//!
//! ⚠️ **O `dead_code` é obrigatório e não é preguiça:** cada binário de teste
//! compila este módulo INTEIRO, então um helper que só o irmão usa parece morto
//! aqui. Sem o `allow`, o preço de compartilhar seria um warning por binário — e
//! o remédio errado (copiar o helper) é exatamente o que o módulo existe para
//! não fazer.

#![allow(dead_code)]

use std::fs;

/// A fonte **sem comentários**.
///
/// ⚠️ Não é higiene: um arch-gate que varre o arquivo cru afirma coisas sobre a
/// PROSA. Este mesmo gate nasceu vermelho porque o doc-comment do `undo_stroke`
/// explica *por que* ele não usa `refresh_region` — a explicação continha a
/// palavra que a asserção proibia. Um gate que dispara em documentação ensina a
/// não documentar.
pub fn source(name: &str) -> String {
    let raw = fs::read_to_string(format!("{}/src/{name}", env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|e| panic!("não consegui ler src/{name}: {e}"));
    raw.lines()
        .map(|l| match l.find("//") {
            Some(at) => &l[..at],
            None => l,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// O corpo de `fn <name>` até a chave que o fecha, contando profundidade.
pub fn function_body(src: &str, name: &str) -> String {
    let at = src
        .find(&format!("fn {name}"))
        .unwrap_or_else(|| panic!("não achei `fn {name}`"));
    let open = src[at..].find('{').expect("corpo") + at;
    let mut depth = 0i32;
    for (i, c) in src[open..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return src[open..open + i + 1].to_string();
                }
            }
            _ => {}
        }
    }
    panic!("`fn {name}` não fecha");
}

/// O bloco `{...}` que começa logo depois de `anchor`, balanceado.
///
/// ⚠️ Existe para afirmar **em que bloco** uma linha mora — que é uma pergunta
/// estrutural — em vez de *a quantos bytes* ela está de outra. A segunda forma é
/// um proxy que expira: a `line/Vector` teve dois arch-gates vermelhos por
/// medirem distância em bytes num arquivo que cresceu.
pub fn braced_block(src: &str, anchor: &str) -> String {
    let at = src
        .find(anchor)
        .unwrap_or_else(|| panic!("não achei `{anchor}`"));
    let open = src[at..].find('{').expect("bloco") + at;
    let mut depth = 0i32;
    for (i, c) in src[open..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return src[open..open + i + 1].to_string();
                }
            }
            _ => {}
        }
    }
    panic!("`{anchor}` não fecha");
}

/// O corpo de um braço de `match`: o bloco `{...}` se houver, senão o resto da
/// linha.
///
/// ⚠️ **Existe porque [`braced_block`] ATRAVESSA um braço de uma linha só.** Ele
/// procura a próxima `{` a partir da âncora, e num braço como
/// `Grip::Hold => scene.grab_at(x, y),` essa chave é a do braço **seguinte** —
/// então uma asserção de ausência (*"este braço não chama `walk`"*) sai lendo o
/// braço que chama, e um gate que passa por olhar o lugar errado é pior que
/// gate nenhum.
pub fn match_arm(src: &str, anchor: &str) -> String {
    let at = src
        .find(anchor)
        .unwrap_or_else(|| panic!("não achei o braço `{anchor}`"));
    let rest = &src[at + anchor.len()..];
    if rest.trim_start().starts_with('{') {
        braced_block(src, anchor)
    } else {
        rest.lines().next().unwrap_or_default().to_string()
    }
}

/// A fiação do módulo 3D no shell, **os dois arquivos como um**.
///
/// ⚠️ O corte entre *a cena* (`sculpt3d.rs`) e *o gesto* (`sculpt3d_input.rs`) é
/// de responsabilidade e já se moveu uma vez (o teto de LOC). Um gate que
/// nomeia o ARQUIVO de cada função vira vermelho no próximo split, sobre
/// produto correto — a `line/Vector` pagou isso duas vezes. As asserções aqui
/// são sobre o que a fiação FAZ, então elas leem o par.
pub fn sculpt_src() -> String {
    format!("{}\n{}", source("sculpt3d.rs"), source("sculpt3d_input.rs"))
}
