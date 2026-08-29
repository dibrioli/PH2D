//! **A gramática** — texto de uma linha → as regras que a derivação aplica.
//!
//! # A forma, e por que é de UMA LINHA
//!
//! ⛔ **Um `\n` num text param CORROMPE o `.ph2dproj`.** O formato do grafo é
//! linha-orientado (`writeln!(out, "x {} {} {}", …)` a escrever, `splitn(4, ' ')` sobre a
//! LINHA a ler), e uma segunda linha cai no `ParseError::BadLine` — o documento inteiro
//! deixa de abrir. O `set_label` recusa quebras de linha de propósito; o `set_text_param`
//! **não**. ⇒ As regras separam-se por **`;`**, e isso não é gosto de sintaxe: é o formato.
//!
//! ```text
//! F -> F[+F]F[-F]F ;  X -> F[+X]F[-X]+X
//! ```
//!
//! # A gramática de uma regra
//!
//! ```text
//! regra := [ ctxE '<' ] pred [ '>' ctxD ] [ ':' condição ] '->' [ '(' peso ')' ] sucessor
//! pred  := SÍMBOLO [ '(' nome, nome, … ')' ]
//! ```
//!
//! - **Paramétrica** (ABOP §1.10): `A(x) -> F(x) [ +A(x*0.7) ] -A(x*0.7)`. Os nomes entre
//!   parênteses do predecessor são **formais**; os do sucessor são **expressões** sobre eles.
//! - **Estocástica** (ABOP §1.7): o mesmo predecessor várias vezes, cada um com um peso —
//!   `F -> (0.4) F[+F]F ; F -> (0.6) FF`. Sem peso declarado, o peso é `1`.
//! - **Sensível a contexto** (ABOP §1.8, 2L): `A < B > C -> D`. O `<`/`>` só existem à
//!   ESQUERDA do `:`, e é por isso que `x > 0.1` numa condição não é ambíguo — a condição é
//!   tudo o que vem depois do primeiro `:` de topo.
//! - **Condicional**: `A(x) : x > 0.1 -> ...`. Sem condição, a regra aplica-se sempre.
//!
//! ⚠️ **O peso vem DEPOIS da seta, entre parênteses**, e a posição é o que o torna
//! inequívoco: um sucessor é uma sequência de módulos, e um módulo nunca COMEÇA por `(` —
//! os parênteses só existem a seguir a uma letra, como lista de argumentos dela.
//!
//! # ⚠️ Uma regra malformada é DESCARTADA, e a alternativa é pior
//!
//! Uma regra que não faz nada deixa o símbolo por reescrever — o desenho fica mais simples
//! do que o artista queria, e ele vê imediatamente qual ramo não cresceu. Recusar a
//! gramática INTEIRA por um erro de digitação apagaria a planta enquanto se escreve a
//! segunda regra, que é o estado normal de quem está a autorar.

use ph2d_expr::Expr;

/// **Quantos argumentos um módulo carrega** — e é um teto de MEMÓRIA, declarado.
///
/// Um módulo derivado é `{ símbolo, geração, nargs, [f32; MAX_ARGS] }` = 24 bytes com este
/// valor, e a cadeia derivada tem centenas de milhares deles ([`crate::MAX_MODULES`]). Subir
/// para 8 argumentos põe o módulo em 40 bytes e **encolhe a cadeia alcançável na mesma
/// memória para 60 %** — é essa a troca, e não «4 parece que chega».
///
/// Os exemplos paramétricos do ABOP (cap. 1.10 e todo o cap. 5) usam **1 a 3**.
pub(crate) const MAX_ARGS: usize = 4;

/// Um módulo da cadeia derivada: um símbolo com os seus argumentos já AVALIADOS.
///
/// `Copy` e sem `Vec` de propósito — é a célula de que a cadeia é feita, e um `Vec` por
/// módulo poria uma alocação por letra numa cadeia de 100 000.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Module {
    pub sym: u8,
    /// A geração em que este módulo NASCEU. É o que torna as gerações fraccionárias
    /// possíveis: só os mais novos crescem enquanto o número sobe.
    pub born: u16,
    pub nargs: u8,
    pub args: [f32; MAX_ARGS],
}

impl Module {
    pub(crate) fn bare(sym: u8, born: u16) -> Self {
        Self {
            sym,
            born,
            nargs: 0,
            args: [0.0; MAX_ARGS],
        }
    }
    pub(crate) fn arg(&self, i: usize) -> Option<f32> {
        (i < self.nargs as usize).then(|| self.args[i])
    }
}

/// Um predecessor (ou um contexto): o símbolo mais os NOMES formais dos seus argumentos.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Pred {
    pub sym: u8,
    pub formals: Vec<String>,
}

/// Um módulo do SUCESSOR: o símbolo mais uma expressão por argumento.
#[derive(Clone, Debug)]
pub(crate) struct SuccModule {
    pub sym: u8,
    pub args: Vec<Expr>,
}

/// Uma produção completa.
#[derive(Clone, Debug)]
pub(crate) struct Rule {
    pub left: Option<Pred>,
    pub pred: Pred,
    pub right: Option<Pred>,
    pub cond: Option<Expr>,
    /// Peso da escolha estocástica entre as regras que casam. `1` por omissão.
    pub weight: f32,
    pub succ: Vec<SuccModule>,
}

/// Um símbolo de módulo: qualquer caractere visível que não seja estrutura da sintaxe.
///
/// ⚠️ O `;` fica de fora porque é o separador de regras (ver o cabeçalho: as regras vivem
/// numa linha só, e isso é o formato do documento, não uma escolha de estilo).
fn is_symbol(c: char) -> bool {
    !c.is_whitespace() && !matches!(c, '<' | '>' | ':' | ';' | '(' | ')' | ',')
}

/// Devolve o índice do primeiro `->` de TOPO (fora de parênteses).
fn find_arrow(s: &str) -> Option<usize> {
    let b = s.as_bytes();
    let mut depth = 0i32;
    let mut i = 0;
    while i + 1 < b.len() {
        match b[i] {
            b'(' => depth += 1,
            b')' => depth -= 1,
            b'-' if depth == 0 && b[i + 1] == b'>' => return Some(i),
            _ => {}
        }
        i += 1;
    }
    None
}

/// O índice do primeiro `:` de topo, se houver.
fn find_colon(s: &str) -> Option<usize> {
    let mut depth = 0i32;
    for (i, c) in s.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            ':' if depth == 0 => return Some(i),
            _ => {}
        }
    }
    None
}

/// Divide `s` por vírgulas de TOPO (as de dentro de parênteses aninhados ficam).
fn split_args(s: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let (mut depth, mut start) = (0i32, 0usize);
    for (i, c) in s.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                out.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    if !s[start..].trim().is_empty() || !out.is_empty() {
        out.push(&s[start..]);
    }
    out
}

/// `A` ou `A(x, y)` → o símbolo e os nomes formais. `None` se não houver símbolo.
fn parse_pred(s: &str) -> Option<Pred> {
    let s = s.trim();
    let mut chars = s.char_indices();
    let (_, first) = chars.next()?;
    if !is_symbol(first) || !first.is_ascii() {
        return None;
    }
    let rest = &s[first.len_utf8()..];
    let formals = match rest.trim().strip_prefix('(') {
        None => Vec::new(),
        Some(inner) => {
            let body = inner.strip_suffix(')').unwrap_or(inner);
            split_args(body)
                .into_iter()
                .map(|f| f.trim().to_string())
                .filter(|f| !f.is_empty())
                .collect()
        }
    };
    Some(Pred {
        sym: first as u8,
        formals,
    })
}

/// Uma sequência de módulos com argumentos-EXPRESSÃO: o lado direito de uma regra, e
/// também o axioma.
pub(crate) fn parse_succ(s: &str) -> Vec<SuccModule> {
    let b = s.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        let c = b[i] as char;
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        if !is_symbol(c) || !c.is_ascii() {
            i += 1;
            continue;
        }
        i += 1;
        let mut args = Vec::new();
        // Uma lista de argumentos, se a houver, colada ao símbolo.
        if i < b.len() && b[i] == b'(' {
            let (mut depth, start) = (0i32, i + 1);
            let mut j = i;
            while j < b.len() {
                match b[j] {
                    b'(' => depth += 1,
                    b')' => {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    _ => {}
                }
                j += 1;
            }
            let end = j.min(b.len());
            for a in split_args(&s[start..end]) {
                if a.trim().is_empty() {
                    continue;
                }
                // ⚠️ Um argumento que não compila vira `0` em vez de descartar o módulo:
                // um comprimento zero é visível (o ramo não cresce) e recuperável; um
                // módulo em falta reordena a cadeia inteira a jusante.
                args.push(ph2d_expr_parse::parse(a).unwrap_or(Expr::Const(0.0)));
            }
            args.truncate(MAX_ARGS);
            i = end + 1;
        }
        out.push(SuccModule { sym: c as u8, args });
    }
    out
}

/// **Todas as regras de um text param** — separadas por `;`, malformadas descartadas.
pub(crate) fn parse_rules(src: &str) -> Vec<Rule> {
    let mut out = Vec::new();
    for raw in src.split(';') {
        if raw.trim().is_empty() {
            continue;
        }
        let Some(arrow) = find_arrow(raw) else {
            continue;
        };
        let (head, tail) = (&raw[..arrow], &raw[arrow + 2..]);

        // ── cabeça: [ctxE <] pred [> ctxD] [: cond] ──
        let (ctx_part, cond_src) = match find_colon(head) {
            Some(i) => (&head[..i], Some(&head[i + 1..])),
            None => (head, None),
        };
        let (left_src, rest) = match ctx_part.split_once('<') {
            Some((l, r)) => (Some(l), r),
            None => (None, ctx_part),
        };
        let (pred_src, right_src) = match rest.split_once('>') {
            Some((p, r)) => (p, Some(r)),
            None => (rest, None),
        };
        let Some(pred) = parse_pred(pred_src) else {
            continue;
        };
        // ⚠️⚠️ **FALHA FECHADA, como o predecessor logo acima** — auditoria de 2026-08-29.
        //
        // A 1.ª redacção era `cond_src.and_then(|c| parse(c).ok())`, e um `None` lê-se a
        // jusante (`derive.rs`) como *«esta regra não TEM condição»*. Ou seja: **qualquer erro
        // de digitação numa condição removia o travão**, em silêncio, e a regra passava a
        // disparar sempre. Medido, com uma gramática e um caractere de diferença, a 14
        // gerações:
        //
        // | condição | módulos |
        // |---|---|
        // | (sem condição) | 16 384 |
        // | `n < 6` — compila | **32** |
        // | `n <= 6` — NÃO compila | 16 384 ← **512×**, e byte-a-byte «sem condição» |
        // | vazia · truncada | 16 384 |
        //
        // *Três sub-campos da mesma regra tinham três políticas de erro diferentes.* Agora têm
        // uma: o que não se entende **descarta a regra**, que é o que já acontecia com o
        // predecessor — a planta muda de forma em vez de perder um travão em silêncio.
        let cond = match cond_src {
            None => None,
            Some(c) => match ph2d_expr_parse::parse(c) {
                Ok(e) => Some(e),
                Err(_) => continue,
            },
        };

        // ── cauda: [(peso)] sucessor ──
        let t = tail.trim_start();
        let (weight, succ_src) = match t.strip_prefix('(') {
            Some(after) => match after.split_once(')') {
                Some((w, rest)) => match w.trim().parse::<f32>() {
                    Ok(v) if v.is_finite() && v > 0.0 => (v, rest),
                    // ⚠️⚠️ **E aqui a falha ABERTA custava DUAS vezes.** O `_ => (1.0, t)` dava
                    // ao peso ilegível o neutro `1,0` — o **maior** de três pesos típicos, logo
                    // a regra mal escrita passava a ser a mais provável — **e** devolvia a
                    // cauda com os parênteses lá dentro, que o `parse_succ` interpreta como
                    // símbolos. Medido, com uma regra só, 4 gerações:
                    //
                    // | cauda | módulos | caixa |
                    // |---|---|---|
                    // | `F[+F]F` (controlo) | 82 | 2,59 × 8,00 |
                    // | `(0.001) F[+F]F` legal | 82 | 2,59 × 8,00 |
                    // | `(40%) F[+F]F` | **1** | **0,00 × 0,00** ← a planta INTEIRA apagada |
                    // | `(-0.5) F[+F]F` | 82 | 5,15 × 3,59 ← outra planta |
                    //
                    // O `%` é o **corte** e o `-`/`+` viram a tartaruga: o texto do peso vira
                    // desenho. ⇒ mesma política do predecessor e da condição.
                    _ => continue,
                },
                None => continue,
            },
            None => (1.0, t),
        };
        let succ = parse_succ(succ_src);
        out.push(Rule {
            left: left_src.and_then(parse_pred),
            pred,
            right: right_src.and_then(parse_pred),
            cond,
            weight,
            succ,
        });
    }
    out
}

#[cfg(test)]
#[path = "grammar_tests.rs"]
mod tests;
