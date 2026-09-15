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
//! # ⛔ A rota que a medição REJEITOU
//!
//! Interpretar a fita no dispositivo daria o catálogo de graça **e** zero compilação. Não cabe: a
//! pior cena real tem **`464` valores vivos ao mesmo tempo** (`Field::tape_shape`), isto é
//! `1 856 B` por thread e `116 KB` por workgroup de 64 — acima da memória partilhada de qualquer
//! GPU. Gerando código, quem aloca registos é o compilador, que é quem sabe.
//!
//! Tabela e mecanismo: `docs/Render3d/05` §33.

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
        for (i, instr) in code.iter().enumerate() {
            let rhs = match instr {
                Instr::X => "p.x".to_string(),
                Instr::Y => "p.y".to_string(),
                Instr::Z => "p.z".to_string(),
                Instr::Const(c) => {
                    // ⚠️ **`f64` → `f32` acontece AQUI, e é a divergência declarada nº 1**: a fita
                    // da CPU é `f64` e o dispositivo é `f32`. O gate da paridade mede-a.
                    #[allow(clippy::cast_possible_truncation)]
                    let v = *c as f32;
                    consts.push(v);
                    format!("{CONSTS}[{}]", const_base + consts.len() - 1)
                }
                // ⭐⭐⭐ **A ESCULTURA: uma folha que não é uma expressão.** A fita traz o índice, e
                // quem escreve o corpo de `escultura_k` é o [`crate::device`] — aqui só se chama.
                Instr::Var(k) => format!("{ESCULTURA}{k}(p)"),
                Instr::Unary(op, a) => unary(*op, *a),
                Instr::Binary(op, a, b) => binary(*op, *a, *b),
            };
            s.push_str(&format!("  let v{i} = {rhs};\n"));
        }
        s.push_str(&format!("  return v{};\n}}\n", self.root()));
        Some(TapeWgsl { source: s, consts })
    }
}

fn unary(op: UnaryOpcode, a: u32) -> String {
    match op {
        UnaryOpcode::Neg => format!("-v{a}"),
        UnaryOpcode::Abs => format!("abs(v{a})"),
        UnaryOpcode::Recip => format!("1.0 / v{a}"),
        UnaryOpcode::Sqrt => format!("sqrt(v{a})"),
        UnaryOpcode::Square => format!("v{a} * v{a}"),
        UnaryOpcode::Floor => format!("floor(v{a})"),
        UnaryOpcode::Ceil => format!("ceil(v{a})"),
        UnaryOpcode::Round => format!("round(v{a})"),
        UnaryOpcode::Sin => format!("sin(v{a})"),
        UnaryOpcode::Cos => format!("cos(v{a})"),
        UnaryOpcode::Tan => format!("tan(v{a})"),
        UnaryOpcode::Asin => format!("asin(v{a})"),
        UnaryOpcode::Acos => format!("acos(v{a})"),
        UnaryOpcode::Atan => format!("atan(v{a})"),
        UnaryOpcode::Exp => format!("exp(v{a})"),
        UnaryOpcode::Ln => format!("log(v{a})"),
        // ⚠️ `(a == 0).into()` — `1.0` quando é zero, `0.0` caso contrário.
        UnaryOpcode::Not => format!("select(0.0, 1.0, v{a} == 0.0)"),
    }
}

fn binary(op: BinaryOpcode, a: u32, b: u32) -> String {
    match op {
        BinaryOpcode::Add => format!("v{a} + v{b}"),
        BinaryOpcode::Sub => format!("v{a} - v{b}"),
        BinaryOpcode::Mul => format!("v{a} * v{b}"),
        BinaryOpcode::Div => format!("v{a} / v{b}"),
        // ⚠️ **`atan2(a, b)`, e a ORDEM é a da `fidget`** (`a.atan2(b)`). Trocá-la espelha a peça
        // em torno da diagonal, e nenhuma régua de silhueta o diria de imediato.
        BinaryOpcode::Atan => format!("atan2(v{a}, v{b})"),
        BinaryOpcode::Min => format!("min(v{a}, v{b})"),
        BinaryOpcode::Max => format!("max(v{a}, v{b})"),
        // ⚠️ `partial_cmp` → `-1 / 0 / 1`, e **NaN** quando não são comparáveis.
        BinaryOpcode::Compare => format!(
            "select(select(select(0.0, 1.0, v{a} > v{b}), -1.0, v{a} < v{b}), \
             bitcast<f32>(0x7fc00000u), v{a} != v{a} || v{b} != v{b})"
        ),
        // ⚠️ **`rem_euclid`, não `%`**: o resto da WGSL leva o sinal do dividendo e o da `fidget`
        // é sempre não-negativo. *Uma repetição com o sinal trocado espelha metade da peça.*
        BinaryOpcode::Mod => format!("(v{a} - v{b} * floor(v{a} / v{b}))"),
        BinaryOpcode::And => format!("select(v{b}, v{a}, v{a} == 0.0)"),
        BinaryOpcode::Or => format!("select(v{b}, v{a}, v{a} != 0.0)"),
    }
}
