//! ⏱️⭐⭐⭐ **A FITA COMO BYTECODE, e o INTERPRETADOR dela em WGSL** — o instrumento que mede o
//! factor que decide a poda por região.
//!
//! A poda (`crate::poda`) corta a fita a `0,18`–`0,52` das instruções nas peças complexas, mas pede
//! **milhares** de fitas diferentes por quadro (o nó: `4 218`), e cada compilação custa `6`–`49 ms`
//! ⇒ ela só serve com um INTERPRETADOR no dispositivo. O ganho é
//! `razão da poda × (custo de uma instrução interpretada ÷ compilada)`, e é esse segundo factor que
//! este módulo existe para medir. ⚠️ **Para a poda continua instrumento** (recusada: o
//! interpretador come o ganho dez vezes, `docs/Render3d/03` §W9) — mas desde 2026-10-01 **o produto
//! chama-o noutro sítio**: a lei do dono ([`crate::owners::wgsl`]) interpreta UMA folha por ponto
//! pintado, onde o custo é ruído e o que se compra é o texto do pintor deixar de mudar com a peça
//! ([`em_floats`] + [`interpretador_em_k_wgsl`]).
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

/// Os casos do `switch` das operações — UMA lista para os dois interpretadores, e escrita pelo
/// MESMO emissor da fita compilada (`crate::wgsl::unary`/`binary`). *Duas tabelas de semântica
/// seriam duas respostas a «o que faz um `Mod`?»*.
fn casos_das_operacoes() -> String {
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
    casos
}

/// ⭐ O interpretador: `fn field(p) -> f32` que percorre o armazém `codigo`.
///
/// ⚠️ `registos` entra no TEXTO (o tamanho do vector privado), logo fitas com registos diferentes
/// são shaders diferentes — um produto arredondaria para uma potência de dois.
#[must_use]
pub fn interpretador_wgsl(registos: usize) -> String {
    let casos = casos_das_operacoes();
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

/// ⭐⭐⭐ **A FITA ESCRITA EM FLOATS** — o formato que viaja no armazém de constantes `k`.
///
/// ⚠️⚠️ **Floats e não bits**, e a razão é a placa: um `u32` arbitrário guardado num `array<f32>`
/// e relido por `bitcast` passa por um registo de vírgula flutuante, e um padrão que seja um
/// SUBNORMAL pode ser esvaziado a zero (a casa já o mediu num gate de estilo). Aqui cada palavra é
/// um INTEIRO pequeno escrito como valor — `op | destino << 8` e `a | b << 8`, os dois `< 2¹⁶`,
/// exactos em `f32` —, e uma constante vai como o próprio valor.
///
/// `palavras` é a fita do [`codifica`] **sem** a palavra final da raiz.
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn em_floats(palavras: &[u32]) -> Vec<f32> {
    let mut out = Vec::with_capacity(palavras.len() * 2);
    let mut i = 0;
    while i < palavras.len() {
        let w = palavras[i];
        i += 1;
        let op = w & 0xff;
        let d = (w >> 8) & 0xff;
        let a = (w >> 16) & 0xff;
        let b = w >> 24;
        out.push((op | d << 8) as f32);
        out.push((a | b << 8) as f32);
        if op == OP_CONST {
            out.push(f32::from_bits(palavras.get(i).copied().unwrap_or(0)));
            i += 1;
        }
    }
    out
}

/// ⭐⭐⭐ **O INTERPRETADOR QUE LÊ DO `k`** — `fn {nome}(p, ini, fim, raiz) -> f32`.
///
/// O irmão do [`interpretador_wgsl`] para o formato do [`em_floats`]: a fita vive entre `ini` e
/// `fim` (índices absolutos no `k`) e o valor acaba no registo `raiz`. ⭐ **O texto não depende da
/// fita** — só de `registos`, que é um TECTO escolhido pelo chamador —, e é isso que o torna útil:
/// um shader que o leve compila UMA vez, por mais folhas e formas que a peça ganhe.
#[must_use]
pub fn interpretador_em_k_wgsl(nome: &str, registos: usize) -> String {
    let casos = casos_das_operacoes();
    let k = crate::wgsl::CONSTS;
    format!(
        "fn {nome}(p: vec3<f32>, ini: u32, fim: u32, raiz: u32) -> f32 {{
  var r: array<f32, {registos}>;
  var pc = ini;
  loop {{
    if (pc >= fim) {{ break; }}
    let w0 = u32({k}[pc]);
    let w1 = u32({k}[pc + 1u]);
    pc = pc + 2u;
    let op = w0 & 0xffu;
    let d = w0 >> 8u;
    let a = w1 & 0xffu;
    let b = w1 >> 8u;
    switch op {{
      case {OP_X}u: {{ r[d] = p.x; }}
      case {OP_Y}u: {{ r[d] = p.y; }}
      case {OP_Z}u: {{ r[d] = p.z; }}
      case {OP_CONST}u: {{ r[d] = {k}[pc]; pc = pc + 1u; }}
{casos}      default: {{ }}
    }}
  }}
  return r[raiz];
}}
"
    )
}

/// ⏱️⭐⭐⭐⭐ **O TECTO DE REGISTOS da marcha interpretada** — o tamanho do vector privado do corpo
/// que o [`corpo_da_fita_interpretada`] escreve. ⚠️ Ele entra no TEXTO, logo é UM número para toda
/// peça (um tecto por peça seria um shader por peça, que é o defeito que isto existe para curar).
/// Uma fita que pede mais cai na fita compilada, que serve sempre.
pub const REGISTOS_DA_MARCHA: usize = 128;

/// ⭐⭐⭐⭐ **O `fn field(p)` que INTERPRETA a peça a partir do `k`** (a forma do
/// [`crate::wgsl::TapeWgsl::source`], com a assinatura) — o *ubershader* da
/// marcha: o texto não depende da peça, só do [`REGISTOS_DA_MARCHA`].
///
/// O formato no `k`, a começar no índice `0` (a fita é sempre a PRIMEIRA a entrar no vector das
/// constantes — o pintor e a escultura apendam a seguir): `k[0]` = quantos floats de fita,
/// `k[1]` = o registo da raiz, `k[2..]` = o [`em_floats`].
#[must_use]
pub fn corpo_da_fita_interpretada(registos: usize) -> String {
    let casos = casos_das_operacoes();
    let k = crate::wgsl::CONSTS;
    format!(
        "fn field(p: vec3<f32>) -> f32 {{
  var r: array<f32, {registos}>;
  var pc = 2u;
  let fim = 2u + u32({k}[0]);
  loop {{
    if (pc >= fim) {{ break; }}
    let w0 = u32({k}[pc]);
    let w1 = u32({k}[pc + 1u]);
    pc = pc + 2u;
    let op = w0 & 0xffu;
    let d = w0 >> 8u;
    let a = w1 & 0xffu;
    let b = w1 >> 8u;
    switch op {{
      case {OP_X}u: {{ r[d] = p.x; }}
      case {OP_Y}u: {{ r[d] = p.y; }}
      case {OP_Z}u: {{ r[d] = p.z; }}
      case {OP_CONST}u: {{ r[d] = {k}[pc]; pc = pc + 1u; }}
{casos}      default: {{ }}
    }}
  }}
  return r[u32({k}[1])];
}}
"
    )
}
