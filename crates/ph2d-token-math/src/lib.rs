#![forbid(unsafe_code)]
//! **A MATH dos tokens de design** — `{spacing.md} * 2` (plano UI/UX W4c.3).
//!
//! # O que esta crate é: um TRADUTOR e um PORTEIRO, nunca um parser
//!
//! O repo já tem **um** parser VEX-lite (`ph2d-expr-parse`, ADR-0144), extraído do
//! `motion.expression` justamente para que a timeline e o nó não tivessem dois. Um terceiro seria a
//! terceira resposta a *"o que é uma fórmula neste app?"*, e a terceira é a que diverge primeiro.
//! Aqui ele ganha o **terceiro consumidor**: traduzimos a nossa sintaxe de referência para a
//! linguagem dele, mandamos parsear, e depois decidimos o que é **admissível num token de design**
//! — que é a parte que é nossa.
//!
//! ⚠️ E ela é uma crate à parte por uma **aresta medida**: `ph2d-expr-parse` arrasta `ph2d-expr`,
//! que arrasta `ph2d-nodegraph`. A `ph2d-tokens` é a folha de que 44 widgets dependem — o
//! [`ph2d_tokens::num_expr::MathHost`] existe para que ela guarde a fórmula como texto sem nunca
//! ver o substrato de grafo.
//!
//! # ⚠️ As CHAVES vêm entre `{}`, e a razão é MEDIDA, não estética
//!
//! O lexer partilhado junta um `.` ao identificador **só quando o que vem a seguir é uma letra**
//! (é assim que `Sprite.x` lexa e `2.5` continua a ser um número). Quatro das 21 chaves têm um
//! DÍGITO depois do ponto — `spacing.2xl`, `spacing.3xl`, `spacing.4xl`, `radius.2xl` — e **não
//! lexam** como identificador nu: `spacing.2xl` pararia em `spacing` e o `.` seguinte daria
//! *"unexpected char"*.
//!
//! ⇒ ou a referência vem delimitada, ou quatro tokens do design system ficam **inexprimíveis**. E
//! ⚠️ **mexer na regra do lexer partilhado não é a saída**: ela está declarada em comentário
//! naquele arquivo e é observável por **dois consumidores que já shipam** (as expressões da
//! timeline e a `motion.expression`) — alargá-la mudaria o significado de `x.5` para os dois.
//!
//! A chave delimitada é também o que o plano-mãe escreveu desde o início (`{spacing.md} * 2`), e é
//! a forma que o **DTCG** fala — o que torna a W4c.5 uma tradução em vez de um segundo dialecto.
//!
//! # ⚠️ Um identificador desconhecido vale ZERO em silêncio — por isso a recusa é AQUI
//!
//! O contrato do `ph2d_expr::Bindings` diz, literalmente, *"unknown names return `0.0` (a missing
//! input is zero, not a panic)"*. Correcto para um nó de partículas, **venenoso** para um design
//! system: `{spacing.md} + gap` daria `12` calados, com o artista convencido de que `gap` existe.
//!
//! Então tudo o que a linguagem partilhada oferece e o nosso domínio não sustenta é recusado
//! **quando o artista escreve**:
//!
//! - um identificador que não seja uma referência nossa — o que **também** mata o `wiggle` sem um
//!   caso especial, porque ele é açúcar do parser para uma fórmula que lê o atributo `t`, e **um
//!   token que oscila com o relógio não é um token, é uma animação**;
//! - uma chave entre `{}` que o design system não tem.
//!
//! O laço e *"o resultado é um comprimento?"* são recusados uma camada acima, na porta de escrita
//! da [`ph2d_tokens::num_overrides`], que é quem conhece o modo e a tabela.

use ph2d_expr::{Bindings, Expr};
use ph2d_tokens::NumToken;
use ph2d_tokens::num_expr::{MathHost, install_math};

/// O prefixo dos identificadores que este módulo fabrica para a linguagem partilhada.
///
/// ⚠️ Ele não precisa de ser impronunciável, e é por isso que existe uma segunda camada: tudo o
/// que sobrar como identificador depois da tradução — inclusive alguém que digite `ref0` à mão — é
/// **recusado** por [`reject_unbound_names`]. A defesa é o conjunto fechado, não o nome exótico.
const REF: &str = "ref";

/// **Instala a capacidade.** Uma linha, no boot.
///
/// ⚠️ Sem ela o app compila, corre, e o painel simplesmente **não oferece** o botão de fórmula —
/// que é o modo de falha certo (o padrão do `set_ml_available`: sem a capacidade o controlo não
/// existe, em vez de existir e não fazer nada). Um arch-gate na shell prova que ela é chamada.
pub fn install() {
    install_math(MathHost {
        deps: |src| translate(src).map(|(_, refs)| refs),
        eval: |src, value_of| {
            let (rewritten, refs) = translate(src)?;
            let ir = ph2d_expr_parse::parse(&rewritten).map_err(|e| e.to_string())?;
            reject_unbound_names(&ir)?;
            Ok(ph2d_expr::eval(
                &ir,
                &Refs {
                    refs: &refs,
                    value_of,
                },
            ))
        },
    });
}

/// `{spacing.md} * 2` → `(ref0 * 2, [spacing.md])`, já **admitido**.
///
/// ⚠️ Ela parseia também no caminho de `deps` — e não é desperdício, é a propriedade: *uma fórmula
/// que a porta admitiu não pode falhar depois*. Se `deps` só varresse as chaves, uma fórmula com
/// sintaxe partida passaria a validação de ciclo e explodiria na leitura, onde o único recurso é
/// cair na fábrica em silêncio.
fn translate(src: &str) -> Result<(String, Vec<NumToken>), String> {
    let (rewritten, refs) = translate_references(src)?;
    let ir = ph2d_expr_parse::parse(&rewritten)
        .map_err(|e| format!("that formula doesn't parse: {e}"))?;
    reject_unbound_names(&ir)?;
    Ok((rewritten, refs))
}

/// A tradução da sintaxe de referência. A mesma chave duas vezes dá **o mesmo** `ref`: a lista é o
/// conjunto de dependências, e repeti-la faria a lei do ciclo percorrer o mesmo ramo por nada.
fn translate_references(src: &str) -> Result<(String, Vec<NumToken>), String> {
    let mut out = String::with_capacity(src.len());
    let mut refs: Vec<NumToken> = Vec::new();
    let mut rest = src;

    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        let close = after
            .find('}')
            .ok_or_else(|| "a `{` is missing its `}`".to_string())?;
        let key = after[..close].trim();
        let token = NumToken::from_key(key)
            .ok_or_else(|| format!("`{key}` is not a token in this design system"))?;
        let idx = refs.iter().position(|t| *t == token).unwrap_or_else(|| {
            refs.push(token);
            refs.len() - 1
        });
        out.push_str(REF);
        out.push_str(&idx.to_string());
        rest = &after[close + 1..];
    }
    out.push_str(rest);
    Ok((out, refs))
}

/// Todo identificador que sobreviveu à tradução tem de ser um `ref<N>` **nosso**.
///
/// ⚠️ O `match` é EXAUSTIVO de propósito — um `_ => Ok(())` admitiria em silêncio um variant que a
/// linguagem partilhada ganhasse depois e que carregasse um nome; o compilador é quem tem de
/// perguntar. (E `Select` não tem nome próprio: ele só carrega três sub-expressões.)
fn reject_unbound_names(e: &Expr) -> Result<(), String> {
    let bad = |n: &str| {
        Err(format!(
            "`{n}` is not a token reference — write `{{spacing.md}}` instead"
        ))
    };
    match e {
        Expr::Const(_) => Ok(()),
        Expr::Attr(name) => {
            if name
                .strip_prefix(REF)
                .is_some_and(|n| n.parse::<usize>().is_ok())
            {
                Ok(())
            } else {
                bad(name)
            }
        }
        Expr::Param(name) => bad(name),
        Expr::Unary(_, inner) => reject_unbound_names(inner),
        Expr::Binary(_, l, r) => {
            reject_unbound_names(l)?;
            reject_unbound_names(r)
        }
        Expr::Call(_, args) => args.iter().try_for_each(reject_unbound_names),
        Expr::Select { cond, a, b } => {
            reject_unbound_names(cond)?;
            reject_unbound_names(a)?;
            reject_unbound_names(b)
        }
    }
}

/// As bindings da avaliação: `ref<N>` → o token na posição `N`, perguntado ao chamador.
struct Refs<'a> {
    refs: &'a [NumToken],
    value_of: &'a dyn Fn(NumToken) -> f32,
}

impl Bindings for Refs<'_> {
    fn attr(&self, name: &str) -> f32 {
        // O `reject_unbound_names` já garantiu que todo identificador é um `ref<N>` nosso.
        name.strip_prefix(REF)
            .and_then(|n| n.parse::<usize>().ok())
            .and_then(|i| self.refs.get(i))
            .map_or(0.0, |t| (self.value_of)(*t))
    }
    fn param(&self, _name: &str) -> f32 {
        0.0
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
