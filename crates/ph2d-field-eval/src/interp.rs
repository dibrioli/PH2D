//! ⏱️⭐⭐⭐ **A FITA COMO BYTECODE, e o INTERPRETADOR dela em WGSL** — o instrumento que mede o
//! factor que decide a poda por região.
//!
//! A poda (`crate::poda`) corta a fita a `0,18`–`0,52` das instruções nas peças complexas, mas pede
//! **milhares** de fitas diferentes por quadro (o nó: `4 218`), e cada compilação custa `6`–`49 ms`
//! ⇒ ela só serve com um INTERPRETADOR no dispositivo. O ganho é
//! `razão da poda × (custo de uma instrução interpretada ÷ compilada)`, e é esse segundo factor que
//! este módulo existe para medir. ⛔ **Hoje é instrumento**: nada no produto o chama.
//!
//! ⭐ **Os casos do `switch` saem do MESMO emissor** ([`crate::wgsl`]): cada operação é escrita
//! pelas funções `unary`/`binary` com os operandos `r[a]`/`r[b]` — *uma segunda tabela de semântica
//! seria a segunda resposta a «o que faz um `Mod`?»*, e ela já divergiu uma vez no sinal do resto.
//!
//! # A codificação
//!
//! Uma palavra por instrução: `op | destino << 8 | a << 16 | b << 24`. Uma constante leva o valor na
//! palavra seguinte. Os registos são **reaproveitados** por vida (um valor liberta o registo na
//! última leitura), que é o que o compilador da placa faz à fita compilada — sem isso a comparação
//! mediria memória em vez de despacho.

use crate::point_tape::Instr;
use fidget::context::{BinaryOpcode, UnaryOpcode};

const OP_X: u32 = 1;
const OP_Y: u32 = 2;
const OP_Z: u32 = 3;
const OP_CONST: u32 = 4;
const OP_UNARIA: u32 = 8;
const OP_BINARIA: u32 = 40;

/// As unárias, pela ordem do código de operação — a POSIÇÃO é o código.
const UNARIAS: [UnaryOpcode; 17] = [
    UnaryOpcode::Neg,
    UnaryOpcode::Abs,
    UnaryOpcode::Recip,
    UnaryOpcode::Sqrt,
    UnaryOpcode::Square,
    UnaryOpcode::Floor,
    UnaryOpcode::Ceil,
    UnaryOpcode::Round,
    UnaryOpcode::Sin,
    UnaryOpcode::Cos,
    UnaryOpcode::Tan,
    UnaryOpcode::Asin,
    UnaryOpcode::Acos,
    UnaryOpcode::Atan,
    UnaryOpcode::Exp,
    UnaryOpcode::Ln,
    UnaryOpcode::Not,
];

/// As binárias, pela ordem do código de operação.
const BINARIAS: [BinaryOpcode; 11] = [
    BinaryOpcode::Add,
    BinaryOpcode::Sub,
    BinaryOpcode::Mul,
    BinaryOpcode::Div,
    BinaryOpcode::Atan,
    BinaryOpcode::Min,
    BinaryOpcode::Max,
    BinaryOpcode::Compare,
    BinaryOpcode::Mod,
    BinaryOpcode::And,
    BinaryOpcode::Or,
];

/// A fita em bytecode.
#[derive(Clone, Debug)]
pub struct Bytecode {
    /// As palavras, a terminar na raiz.
    pub palavras: Vec<u32>,
    /// Quantos registos o interpretador precisa.
    pub registos: usize,
    /// Em que registo acaba o valor.
    pub raiz: u32,
    /// Instruções que fazem trabalho (tudo menos as folhas) — o denominador por instrução.
    pub operacoes: usize,
}

#[allow(clippy::cast_possible_truncation)]
fn codigo_u(op: UnaryOpcode) -> u32 {
    OP_UNARIA
        + UNARIAS
            .iter()
            .position(|o| *o == op)
            .expect("unária por codificar") as u32
}

#[allow(clippy::cast_possible_truncation)]
fn codigo_b(op: BinaryOpcode) -> u32 {
    OP_BINARIA
        + BINARIAS
            .iter()
            .position(|o| *o == op)
            .expect("binária por codificar") as u32
}

/// ⭐ A fita em bytecode — `None` sem fita, com escultura, ou com mais de `255` registos vivos.
pub(crate) fn codifica(code: &[Instr], raiz: u32) -> Option<Bytecode> {
    let n = code.len();
    // A última leitura de cada valor (a raiz vive até ao fim).
    let mut ultima = vec![0usize; n];
    for (i, ins) in code.iter().enumerate() {
        match *ins {
            Instr::Unary(_, a) => ultima[a as usize] = i,
            Instr::Binary(_, a, b) => {
                ultima[a as usize] = i;
                ultima[b as usize] = i;
            }
            _ => {}
        }
    }
    ultima[raiz as usize] = n;
    let mut reg = vec![0u32; n];
    let mut livres: Vec<u32> = Vec::new();
    let mut proximo = 0u32;
    let mut palavras = Vec::with_capacity(n * 2);
    let mut operacoes = 0usize;
    for (i, ins) in code.iter().enumerate() {
        // Os operandos lidos pela última vez libertam o registo ANTES do destino ser escolhido: o
        // interpretador lê `a` e `b` antes de escrever `d`, logo `d` pode reaproveitá-los.
        let (op, a, b) = match *ins {
            Instr::X => (OP_X, 0, 0),
            Instr::Y => (OP_Y, 0, 0),
            Instr::Z => (OP_Z, 0, 0),
            Instr::Const(_) => (OP_CONST, 0, 0),
            Instr::Var(_) => return None,
            Instr::Unary(o, a) => (codigo_u(o), reg[a as usize], 0),
            Instr::Binary(o, a, b) => (codigo_b(o), reg[a as usize], reg[b as usize]),
        };
        match *ins {
            Instr::Unary(_, x) if ultima[x as usize] == i => livres.push(reg[x as usize]),
            Instr::Binary(_, x, y) => {
                if ultima[x as usize] == i {
                    livres.push(reg[x as usize]);
                }
                if ultima[y as usize] == i && y != x {
                    livres.push(reg[y as usize]);
                }
            }
            _ => {}
        }
        livres.sort_unstable_by(|p, q| q.cmp(p));
        let d = livres.pop().unwrap_or_else(|| {
            proximo += 1;
            proximo - 1
        });
        if d > 255 {
            return None;
        }
        reg[i] = d;
        palavras.push(op | d << 8 | a << 16 | b << 24);
        if let Instr::Const(c) = *ins {
            #[allow(clippy::cast_possible_truncation)]
            palavras.push((c as f32).to_bits());
        }
        if op >= OP_UNARIA {
            operacoes += 1;
        }
    }
    Some(Bytecode {
        palavras,
        registos: proximo.max(1) as usize,
        raiz: reg[raiz as usize],
        operacoes,
    })
}

/// ⭐ O interpretador: `fn field(p) -> f32` que percorre o armazém `codigo`.
///
/// ⚠️ `registos` entra no TEXTO (o tamanho do vector privado), logo fitas com registos diferentes
/// são shaders diferentes — um produto arredondaria para uma potência de dois.
#[must_use]
pub fn interpretador_wgsl(registos: usize) -> String {
    let mut casos = String::new();
    for (i, op) in UNARIAS.iter().enumerate() {
        casos += &format!(
            "      case {}u: {{ r[d] = {}; }}\n",
            OP_UNARIA as usize + i,
            crate::wgsl::unary(*op, "r[a]")
        );
    }
    for (i, op) in BINARIAS.iter().enumerate() {
        casos += &format!(
            "      case {}u: {{ r[d] = {}; }}\n",
            OP_BINARIA as usize + i,
            crate::wgsl::binary(*op, "r[a]", "r[b]")
        );
    }
    format!(
        "fn field(p: vec3<f32>) -> f32 {{
  var r: array<f32, {registos}>;
  var pc = 0u;
  let n = arrayLength(&codigo) - 1u;
  loop {{
    if (pc >= n) {{ break; }}
    let w = codigo[pc];
    pc = pc + 1u;
    let op = w & 0xffu;
    let d = (w >> 8u) & 0xffu;
    let a = (w >> 16u) & 0xffu;
    let b = w >> 24u;
    switch op {{
      case {OP_X}u: {{ r[d] = p.x; }}
      case {OP_Y}u: {{ r[d] = p.y; }}
      case {OP_Z}u: {{ r[d] = p.z; }}
      case {OP_CONST}u: {{ r[d] = bitcast<f32>(codigo[pc]); pc = pc + 1u; }}
{casos}      default: {{ }}
    }}
  }}
  return r[codigo[n]];
}}
"
    )
}
