//! **O QUE O TRAÇO ESCREVE** — os dois aplicadores e a aritmética que eles
//! partilham.
//!
//! ⚠️ **Filho (`#[path]`) de [`super`], e não um irmão:** eles leem os planos
//! congelados (`base_pos`, `base_mask`, `accum`, `slot`) e a lista `moved`, e um
//! módulo irmão os obrigaria a virar `pub(crate)` — a visibilidade viraria
//! função do TAMANHO do arquivo, que é o oposto do que o teto de LOC existe para
//! fazer. É o mesmo corte que o [`super::target`] já usa.
//!
//! O corte é o que a própria prosa do pai já separava: lá mora **a LEI** (o que
//! um dab decide, de onde ele mede, o que ele acumula); aqui mora **a
//! ESCRITA** — as duas superfícies que um traço toca, e o fato de elas usarem
//! aritméticas DIFERENTES por razões que estão nos doc-comments de cada uma.
//!
//! ⚠️ O corte cobra **três** aberturas, e elas são o mínimo: um filho VÊ o
//! privado do pai, mas o pai não vê o do filho, então os dois aplicadores e o
//! [`toward`] viram `pub(super)` — estritamente mais estreito que o
//! `pub(crate)` que um módulo irmão pediria. É a mesma conta que o
//! [`super::target`] já paga.

use super::*;

impl SculptStroke {
    /// ⚠️ **`coat` é a DEMÃO, e ali o alvo já é a posição FINAL.**
    ///
    /// Os outros quinze verbos compõem o alvo SEM peso e é o `accum` que os
    /// atenua (`toward(base, alvo, accum)`). A demão não cabe nisso porque o
    /// `accum` dela **não é um peso** — é o `displacement_factors` do
    /// `layer.cc`, a fração da camada já depositada, e a referência o consome
    /// DENTRO do alvo (`orig + n·altura·disp`) e depois anda até lá por
    /// `factors`, a partir do VIVO. Rodar o `toward` por cima disso aplicaria o
    /// `disp` uma segunda vez.
    ///
    /// ⚠️ **Um `bool` e não `accum = 1` carimbado**, que é como os quatro grips
    /// de gesto resolvem o mesmo problema: aqui o `accum` **tem de continuar
    /// sendo o `disp`**, porque é o que o dab seguinte lê para saber quanto da
    /// demão ainda cabe. As duas leituras — *onde este vértice vai parar* e
    /// *quanto já foi depositado* — deixaram de ser o mesmo número.
    pub(super) fn apply_positions(&self, mesh: &mut Mesh, coat: bool) {
        let out = mesh.positions_mut();
        for &v in &self.moved {
            let vi = v as usize;
            let s = self.slot[vi] as usize;
            let (b, t, a) = (self.base_pos[s], self.target[s], self.accum[s]);
            out[vi] = if coat {
                t
            } else {
                [
                    toward(b[0], t[0], a),
                    toward(b[1], t[1], a),
                    toward(b[2], t[2], a),
                ]
            };
        }
    }

    /// A MESMA lei, no canal da máscara: [`toward`] entre o `base` e o alvo, onde
    /// o alvo é `1` (mascarar) ou `0` (limpar). Um verbo, uma aritmética.
    /// **O CANAL — `clamp(base ± Σ, 0, 1)`, e não um lerp para um alvo.**
    ///
    /// ⚠️ **A troca não é cosmética, e o par de leis diverge onde o artista
    /// trabalha.** `toward(base, goal, a)` escala o incremento pela folga que
    /// resta (`base + (goal − base)·a`), então mascarar por cima de meia
    /// máscara pinta METADE do que a mesma esfregada pinta em tela limpa —
    /// assintótico, nunca chega, e (com o envelope que vinha antes) nem sequer
    /// crescia. A referência SOMA (`Masking.js:70`), o que faz de cada
    /// esfregada a mesma esfregada, e é a lei que torna o canal construível.
    ///
    /// ⚠️ **A POLARIDADE é invertida entre os dois mundos e a conversão é
    /// EXATA**, não uma aproximação: o nosso `1` é *protegido* e o `free` da
    /// referência é *livre*, com `free = 1 − mask`. Sob essa troca
    /// `clamp(free − f)` **é** `clamp(mask + f)` — as duas leis são a mesma
    /// escrita nos dois sentidos, e é isso que permite comparar o produto com o
    /// porte sem um fator de correção no meio.
    pub(super) fn apply_mask(&self, mesh: &mut Mesh, brush: &Brush) {
        // O `Ctrl` do canal: sem ele o gesto PROTEGE (o `_negative = true` de
        // fábrica da `Masking`, lido na nossa polaridade), com ele limpa.
        let sign = if brush.invert { -1.0 } else { 1.0 };
        let out = mesh.masks_mut();
        for &v in &self.moved {
            let vi = v as usize;
            let s = self.slot[vi] as usize;
            out[vi] = (self.base_mask[s] + sign * self.accum[s]).clamp(0.0, 1.0);
        }
    }
}

/// **O APLICADOR** — de `b` para `t`, andando a fração `a`.
///
/// ⚠️ **Ele é ancorado no ALVO, e não no `base`.** A forma óbvia — `b + (t−b)·a`,
/// que esteve aqui até 2026-08-11 — e esta são a mesma coisa em aritmética
/// exata, e **nenhuma das três formas possíveis é exata nas três pontas que
/// importam**. Medido em 400 mil pares, contando divergências:
///
/// | forma | `a = 1` → `t` | `a = 0` → `b` | `t = b` → parado |
/// |---|---|---|---|
/// | `b + (t−b)·a` | **139 522** | 0 | 0 |
/// | `b·(1−a) + t·a` | 0 | 0 | **53 315** |
/// | `t − (t−b)·(1−a)` | 0 | **139 697** | 0 |
///
/// A escolha não é de gosto: é de **quais promessas o produto faz**.
///
/// - **`a = 1` devolve `t`** — o [`Grip::Hook`] e o [`Grip::Turn`] põem o peso
///   DENTRO do alvo e carimbam `accum = 1`, então para eles o alvo **é** a
///   posição final. Sem esta exatidão a paridade bit-a-bit com o kernel da
///   referência morre **no aplicador**, e nenhum gate de verbo saberia dizer:
///   todos medem deslocamento com tolerância. Um gate do produto pegou um Twist
///   escrevendo `1,3164502e-8` onde o alvo dizia `1,3164501e-8`.
/// - **`t = b` não move nada** — o `Fill` e o `Scrape` devolvem o próprio `base`
///   para o lado errado do plano, e o `Move` sem gesto devolve `base` para toda
///   a pegada. É uma promessa que o artista vê: *este verbo não toca aquele
///   lado*. A segunda forma a quebra, e o gate `a_dab_with_no_gesture_moves_
///   nothing` ficou VERMELHO nela.
/// - **`a = 0` devolve `b`** — é a que sobra, e é a que o produto **não usa**:
///   `apply_positions` percorre `moved`, e um vértice só entra ali com `w > 0`.
///
/// ⚠️ **E a medição de projeto que dizia que a forma antiga bastava era do
/// REGIME ERRADO.** Eu havia medido `b + (t−b)·1 == t` em 9 M pares gerados como
/// `t = fl(b + d)`, onde a subtração é uma transformação livre de erro e a
/// identidade é **garantida por construção**. O alvo do produto não nasce assim:
/// ele é uma expressão inteira (uma rotação de Rodrigues, uma projeção em plano)
/// arredondada **uma vez** ao `f32`, e contra o `base` ele é um float
/// independente. *Uma fixture que fabrica o valor pela mesma aritmética que vai
/// testar não contém o fenômeno.*
#[inline]
pub(super) fn toward(b: f32, t: f32, a: f32) -> f32 {
    t - (t - b) * (1.0 - a)
}
