//! ⭐⭐⭐ **A FITA DO CAMPO, ESCRITA EM WGSL** — o que leva o modelador ao dispositivo.
//!
//! # ⭐ Porque são VINTE E OITO opcodes e não sessenta e duas primitivas
//!
//! O catálogo do modelador tem `62` primitivas e uma pilha de modificadores. Escrever WGSL para
//! cada uma seria meses de trabalho e um segundo sítio onde a forma vive — *duas respostas à
//! mesma pergunta, e a que o artista vê é a que envelhece*.
//!
//! ⭐ Mas o documento **já** é compilado numa **fita**: uma lista SSA de operações **aritméticas**
//! (`fidget`), onde uma rosca, uma superfórmula e um filete já são `min`, `max`, `sqrt` e
//! multiplicações. ⇒ o gerador percorre a fita e emite **uma linha por instrução**, e o catálogo
//! inteiro — incluindo o que ainda não foi escrito — sai de graça.
//!
//! # ⚠️ As CONSTANTES não são baked, e essa é a decisão que faz isto servir
//!
//! Arrastar um slider muda um **número**; a árvore fica igual. Se a constante fosse escrita no
//! shader, cada quadro de um arrasto recompilaria — medido, `6` a `49 ms` — e o gesto morria.
//! ⇒ elas saem num **vector** (`k[i]`), e a chave do pipeline é o **texto**, que não muda. *Um
//! arrasto de slider reescreve um buffer e não recompila nada.*
//!
//! # ⛔ A rota que a medição REJEITOU — e ⚠️ **a PREMISSA dela caiu em 2026-09-15**
//!
//! Interpretar a fita no dispositivo daria o catálogo de graça **e** zero compilação. A recusa
//! original dizia: *«não cabe — a pior cena real tem **`464`** valores vivos ao mesmo tempo, isto é
//! `1 856 B` por thread e `116 KB` por workgroup de 64, acima da memória partilhada de qualquer
//! GPU»*.
//!
//! ⚠️⚠️ **Esse `464` era uma propriedade da ORDEM da fita, não do grafo.** Com o
//! [`crate::tape_schedule`] a pior cena real mede **`89`** — `356 B` por thread e **`22 KB`** por
//! workgroup de 64, que **cabe**. ⇒ §0.0: *quem move o número que tornava algo inalcançável tem de
//! reconferir a nota.*
//!
//! ⛔ **A recusa FICA, e o motivo que sobra é outro:** gerando código, quem aloca registos é o
//! compilador da placa — um interpretador paga descodificação por amostra e perde a fusão de
//! operações que o `naga` faz. *O que mudou é que a rota deixou de ser impossível e passou a ser
//! uma medição por fazer* — e ela não se abre sem um número, que hoje não existe.
//!
//! Tabela e mecanismo: `docs/Render3d/05` §33 e §43.

use crate::point_tape::{Instr, PointTape};
use fidget::context::{BinaryOpcode, UnaryOpcode};

/// A fita escrita em WGSL, com as constantes **de fora**.
#[derive(Clone, Debug, PartialEq)]
pub struct TapeWgsl {
    /// O corpo de `fn field(p: vec3<f32>) -> f32`, já com `{`/`}`.
    ///
    /// ⭐ **É esta a chave do cache de pipelines** — ela não muda quando um número muda.
    pub source: String,
    /// Os valores de `k[i]`, na ordem em que o `source` os indexa.
    pub consts: Vec<f32>,
}

/// O nome que o gerador dá ao vector das constantes. Quem liga o buffer usa-o.
pub const CONSTS: &str = "k";

/// O prefixo das funções de ESCULTURA — `escultura_0`, `escultura_1`, … Quem as escreve é o
/// [`crate::device`]; a fita só as **chama**, e as duas metades têm de concordar no nome.
pub const ESCULTURA: &str = "escultura_";

impl PointTape {
    /// ⭐⭐⭐ **A fita em WGSL.** `None` quando o documento não tem fita (a mesma resposta que o
    /// [`crate::Field::at`] dá com `NaN`).
    #[must_use]
    pub fn to_wgsl(&self) -> Option<TapeWgsl> {
        self.to_wgsl_named("field", 0)
    }

    /// ⭐⭐⭐ **A MESMA fita com OUTRO nome e outra origem no vector das constantes.**
    ///
    /// # ⛔⛔ Porque ela existe, e porque a alternativa textual é um defeito à espera
    ///
    /// A lei do **dono** ([`crate::owners`]) precisa de **uma função por folha** no mesmo módulo, e
    /// cada folha traz as constantes dela. A forma óbvia — gerar `N` vezes com [`Self::to_wgsl`] e
    /// depois reescrever o texto (`fn field` → `fn folha_3`, `k[7]` → `k[62]`) — falha de duas
    /// maneiras **mudas**: o nome `field` aparece dentro de qualquer comentário ou identificador que
    /// o contenha, e a renumeração por expressão regular sobre `k[i]` não sabe distinguir a
    /// constante `k[7]` da constante `k[70]` sem reescrever da direita para a esquerda.
    ///
    /// ⇒ o nome e a origem entram **onde o texto é escrito**, que é o único sítio que sabe o que
    /// cada coisa é. *Uma reescrita de texto é uma segunda análise do que o gerador já sabia.*
    ///
    /// ⚠️ `const_base` é o índice em que as constantes desta fita começam dentro do vector
    /// partilhado — quem concatena os vectores é quem escolhe os bases, e os dois têm de concordar.
    #[must_use]
    pub fn to_wgsl_named(&self, name: &str, const_base: usize) -> Option<TapeWgsl> {
        let code = self.code()?;
        let mut consts: Vec<f32> = Vec::new();
        let mut s = String::with_capacity(code.len() * 24);
        s.push_str(&format!("fn {name}(p: vec3<f32>) -> f32 {{\n"));
        // ⭐⭐⭐ **Como cada slot se LÊ.** Uma folha lê-se onde é usada (`p.x`, `k[7]`); tudo o mais
        // ganha um `let` e lê-se pelo nome dele — ver [`Instr::ocupa_registo`].
        //
        // ⛔ **`escultura_k(p)` NÃO é uma folha para este efeito**, embora o seja para a fita:
        // reescrevê-la em cada uso repetiria uma consulta de oito amostras a uma grade.
        let mut como: Vec<String> = Vec::with_capacity(code.len());
        for (i, instr) in code.iter().enumerate() {
            let rhs = match instr {
                Instr::X => {
                    como.push("p.x".to_string());
                    continue;
                }
                Instr::Y => {
                    como.push("p.y".to_string());
                    continue;
                }
                Instr::Z => {
                    como.push("p.z".to_string());
                    continue;
                }
                Instr::Const(c) => {
                    // ⚠️ **`f64` → `f32` acontece AQUI, e é a divergência declarada nº 1**: a fita
                    // da CPU é `f64` e o dispositivo é `f32`. O gate da paridade mede-a.
                    #[allow(clippy::cast_possible_truncation)]
                    let v = *c as f32;
                    consts.push(v);
                    como.push(format!("{CONSTS}[{}]", const_base + consts.len() - 1));
                    continue;
                }
                // ⭐⭐⭐ **A ESCULTURA: uma folha que não é uma expressão.** A fita traz o índice, e
                // quem escreve o corpo de `escultura_k` é o [`crate::device`] — aqui só se chama.
                Instr::Var(k) => format!("{ESCULTURA}{k}(p)"),
                Instr::Unary(op, a) => unary(*op, &como[*a as usize]),
                Instr::Binary(op, a, b) => binary(*op, &como[*a as usize], &como[*b as usize]),
            };
            s.push_str(&format!("  let v{i} = {rhs};\n"));
            como.push(format!("v{i}"));
        }
        s.push_str(&format!("  return {};\n}}\n", como[self.root() as usize]));
        Some(TapeWgsl { source: s, consts })
    }
}

/// ⚠️ **`a` já vem ESCRITO** (`v12`, `p.x` ou `k[7]`) e é sempre ATÓMICO — uma expressão composta
/// ganha sempre um `let`, logo nenhuma destas formas precisa de parênteses à volta do operando.
fn unary(op: UnaryOpcode, a: &str) -> String {
    match op {
        UnaryOpcode::Neg => format!("-{a}"),
        UnaryOpcode::Abs => format!("abs({a})"),
        UnaryOpcode::Recip => format!("1.0 / {a}"),
        UnaryOpcode::Sqrt => format!("sqrt({a})"),
        UnaryOpcode::Square => format!("{a} * {a}"),
        UnaryOpcode::Floor => format!("floor({a})"),
        UnaryOpcode::Ceil => format!("ceil({a})"),
        UnaryOpcode::Round => format!("round({a})"),
        UnaryOpcode::Sin => format!("sin({a})"),
        UnaryOpcode::Cos => format!("cos({a})"),
        UnaryOpcode::Tan => format!("tan({a})"),
        UnaryOpcode::Asin => format!("asin({a})"),
        UnaryOpcode::Acos => format!("acos({a})"),
        UnaryOpcode::Atan => format!("atan({a})"),
        UnaryOpcode::Exp => format!("exp({a})"),
        UnaryOpcode::Ln => format!("log({a})"),
        // ⚠️ `(a == 0).into()` — `1.0` quando é zero, `0.0` caso contrário.
        UnaryOpcode::Not => format!("select(0.0, 1.0, {a} == 0.0)"),
    }
}

/// ⚠️ Ver [`unary`]: os dois operandos vêm escritos e são atómicos.
fn binary(op: BinaryOpcode, a: &str, b: &str) -> String {
    match op {
        BinaryOpcode::Add => format!("{a} + {b}"),
        BinaryOpcode::Sub => format!("{a} - {b}"),
        BinaryOpcode::Mul => format!("{a} * {b}"),
        BinaryOpcode::Div => format!("{a} / {b}"),
        // ⚠️ **`atan2(a, b)`, e a ORDEM é a da `fidget`** (`a.atan2(b)`). Trocá-la espelha a peça
        // em torno da diagonal, e nenhuma régua de silhueta o diria de imediato.
        BinaryOpcode::Atan => format!("atan2({a}, {b})"),
        BinaryOpcode::Min => format!("min({a}, {b})"),
        BinaryOpcode::Max => format!("max({a}, {b})"),
        // ⚠️ `partial_cmp` → `-1 / 0 / 1`, e **NaN** quando não são comparáveis.
        BinaryOpcode::Compare => format!(
            "select(select(select(0.0, 1.0, {a} > {b}), -1.0, {a} < {b}), \
             bitcast<f32>(0x7fc00000u), {a} != {a} || {b} != {b})"
        ),
        // ⚠️ **`rem_euclid`, não `%`**: o resto da WGSL leva o sinal do dividendo e o da `fidget`
        // é sempre não-negativo. *Uma repetição com o sinal trocado espelha metade da peça.*
        BinaryOpcode::Mod => format!("({a} - {b} * floor({a} / {b}))"),
        BinaryOpcode::And => format!("select({b}, {a}, {a} == 0.0)"),
        BinaryOpcode::Or => format!("select({b}, {a}, {a} != 0.0)"),
    }
}
