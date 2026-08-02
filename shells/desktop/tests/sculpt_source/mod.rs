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
        .unwrap_or_else(|| panic!("não achei `{anchor}`"))
        // ⚠️ **Depois do FIM da âncora, não do começo dela.** Um braço de `match`
        // sobre struct traz chaves no próprio padrão
        // (`StrokeUndo::Descended { from, stamped } =>`), e procurar a partir do
        // início devolvia esse `{ from, stamped }` como se fosse o corpo — um
        // bloco que existe, fecha, e não contém nada do que a asserção procura.
        + anchor.len();
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

/// Os argumentos de uma chamada `name(...)`, balanceando **parênteses**.
///
/// ⚠️ Existe porque [`braced_block`] procura a próxima CHAVE, e uma chamada não
/// tem nenhuma: apontá-lo a uma devolve o bloco seguinte, e a asserção passa a
/// medir outro lugar. Foi exatamente o que a primeira versão do gate do `shift`
/// fez — ela leu o corpo do `if` e disse que o modificador não chegava.
pub fn call_args(src: &str, name: &str) -> String {
    let at = src
        .find(name)
        .unwrap_or_else(|| panic!("não achei a chamada `{name}`"));
    let open = src[at..].find('(').expect("a chamada abre") + at;
    let mut depth = 0i32;
    for (i, c) in src[open..].char_indices() {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return src[open..open + i + 1].to_string();
                }
            }
            _ => {}
        }
    }
    panic!("a chamada `{name}` não fecha");
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

/// A fiação do módulo 3D no shell, **o CLUSTER inteiro**.
///
/// ⚠️ O corte entre *a cena* (`sculpt3d.rs`), *o gesto* (`_input.rs`), *a
/// doação* (`_donation.rs`) e *a história* (`_history.rs`) é de responsabilidade
/// e já se moveu DUAS vezes (o teto de LOC). Um gate que nomeia o ARQUIVO de
/// cada função vira vermelho no próximo split, **sobre produto correto** — a
/// `line/Vector` pagou isso duas vezes e esta linha pagou uma. As asserções aqui
/// são sobre o que a fiação FAZ, então ela lê **todo `sculpt3d*.rs`**: o quinto
/// arquivo nasce coberto, que é como o quarto nasceu descoberto.
pub fn sculpt_src() -> String {
    let dir = format!("{}/src", env!("CARGO_MANIFEST_DIR"));
    let mut names: Vec<String> = fs::read_dir(&dir)
        .expect("src/")
        .filter_map(|e| e.ok().map(|e| e.file_name().to_string_lossy().into_owned()))
        .filter(|n| n.starts_with("sculpt3d") && n.ends_with(".rs"))
        .collect();
    names.sort();
    assert!(
        names.len() >= 4,
        "o cluster tem pelo menos quatro arquivos, e achei {names:?}"
    );
    let joined = names
        .iter()
        .map(|n| source(n))
        .collect::<Vec<_>>()
        .join("\n");
    elide_active_object(&joined)
}

/// A fonte com a **porta do objeto ativo ELIDIDA**.
///
/// ⚠️ Desde a W8.1 a cena é uma LISTA, e toda pilha é alcançada por uma de três
/// grafias da MESMA porta: `self.obj().stack`, `self.obj_mut().stack` e —
/// onde o borrow checker exige campos disjuntos — `self.objects[self.active].
/// stack` (ver `sculpt3d_space.rs`). O que estes gates afirmam é **qual verbo da
/// pilha é chamado**, nunca por qual das três grafias; sem esta normalização
/// eles ficariam vermelhos a cada rearranjo de empréstimo, **sobre produto
/// correto** — que é o proxy que expira, pela oitava vez nesta linha.
///
/// ⚠️ A elisão é do ENDEREÇO, não do verbo: `.mesh_mut()`, `.add_level()` e
/// companhia atravessam intactos, e é sobre eles que as asserções falam.
fn elide_active_object(src: &str) -> String {
    src.replace("self.objects[self.active].stack", "self.stack")
        .replace("self.obj_mut().stack", "self.stack")
        .replace("self.obj().stack", "self.stack")
}

/// O corpo do braço de `match` que **CONTÉM** este padrão — mesmo quando ele é
/// um de vários no mesmo braço (`A | B | C => { … }`).
///
/// ⚠️ Existe porque [`braced_block`] ancora no TEXTO do padrão, e um braço
/// agrupado não tem `=>` logo depois de cada um deles: o gate da reversão
/// quebrou exatamente assim no dia em que um segundo caso passou a partilhar o
/// mesmo corpo — vermelho sobre produto correto, o sétimo proxy a expirar nesta
/// linha. A pergunta que o gate quer fazer é *o que o braço DESTE caso faz*, e
/// isso não muda quando ele ganha companhia.
pub fn arm_with(src: &str, pattern: &str) -> String {
    let at = src
        .find(pattern)
        .unwrap_or_else(|| panic!("não achei o padrão `{pattern}`"));
    let rest = &src[at..];
    let arrow = rest
        .find("=>")
        .unwrap_or_else(|| panic!("`{pattern}` não está num braço de match"));
    match_arm(rest, &rest[..arrow + 2])
}
