//! **A LEI DA DEMÃO** — a saturação assintótica do `layer.cc`, e nada mais.
//!
//! Módulo próprio e minúsculo porque ela tem **dois** consumidores que não se
//! veem: o laço do dab (que a corre) e os gates (que a comparam contra a
//! recorrência do Blender escrita à mão). Uma segunda cópia num deles seria o
//! oráculo-espelho que esta casa varre a cada wave — um gate que chama a função
//! sob teste para computar o que espera é sempre verde.

/// **A CABEÇA que a referência dá ao incremento** — o `1.05` de
/// `offset_displacement_factors`.
///
/// ⚠️ **Ela é `> 1` de propósito, e é o que faz o centro do pincel chegar ao
/// teto num dab só:** com peso `1` e força `1` o primeiro passo vale
/// `0 + 1·1·(1,05 − 0) = 1,05`, que o clamp corta em `1`. Fosse exactamente
/// `1,0` o incremento seria `1 − d` e a demão **nunca** fecharia — ela
/// aproximar-se-ia do teto por metades, para sempre, e um platô que não fecha
/// não é um platô.
pub const COAT_HEAD: f32 = 1.05;

/// **Um passo da demão** — `d ← clamp(d + w·força·(1,05 − |d|), 0, teto)`.
///
/// `d` é a fração da demão já depositada neste vértice (o
/// `displacement_factor` da referência, que no nosso motor **é** o `accum`),
/// `w` o peso completo do dab e `teto` a máscara livre daquele vértice.
///
/// ⚠️ **O `teto` é a máscara e não `1`, e a referência aplica a máscara DUAS
/// vezes de propósito:** ela já está dentro do `w` (como taxa — um vértice meio
/// protegido recebe metade por dab) e volta aqui (como **altura** — ele para na
/// metade da demão). Sem a segunda metade um vértice mascarado chegaria à demão
/// INTEIRA, só mais devagar, e a máscara deixaria de proteger o que protege.
///
/// ⚠️ **`|d|` e não `d`:** o valor da referência é assinado (o `Ctrl` cava em
/// vez de encher) e o nosso é a MAGNITUDE, com o sinal a viajar no alvo — as
/// duas formas coincidem porque o sinal de um traço é constante do pen-down ao
/// pen-up. O `abs` fica porque é a lei escrita, e porque um `d` negativo que
/// chegasse aqui por outra via saturaria na direção certa.
#[inline]
#[must_use]
pub fn coat_step(d: f32, w: f32, strength: f32, cap: f32) -> f32 {
    (d + w * strength * (COAT_HEAD - d.abs())).clamp(0.0, cap)
}
