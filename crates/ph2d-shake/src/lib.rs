#![forbid(unsafe_code)]
//! ⭐⭐⭐ **O ABANÃO DA VISTA** — *isto explodiu, e a câmera tremeu.*
//!
//! # O que ela diz
//!
//! A vista tem um **trauma**: um número em `0..1` que sobe de repente quando algo acontece e
//! **decai sozinho**. O deslocamento é `amplitude × trauma^n × ruído(f·t)`, e o `n` é o que faz a
//! cauda **morrer** em vez de pairar — é o modelo de Squirrel Eiserloh (*Juicing Your Cameras With
//! Math*, GDC 2016), que o `bevy_trauma_shake` e o *Impulse* do Cinemachine implementam.
//!
//! ⚠️⚠️ **O deslocamento é um OFFSET da VISTA e nunca uma escrita no estado vivo da câmera.**
//! Quem a consome soma-o ao centro **depois** do `follow`, do amortecimento e dos limites — a
//! razão está escrita na `camera_2d::update` do shell desde a wave do #7, para o `offset`
//! autorado: *escrito no estado, ele realimenta a zona morta e a câmera afasta-se do alvo um pouco
//! mais a cada quadro.* Este é o segundo somando no mesmo sítio.
//!
//! # As três leis, cada uma uma porta
//!
//! | a pergunta | a porta |
//! |---|---|
//! | *quanto trauma tenho agora?* | [`acumula`] · [`decai`] |
//! | *quanto de um impulso chega DAQUI?* | [`atenuacao`] |
//! | *para onde a vista vai neste instante?* | [`deslocamento`] |
//!
//! # ⭐ O abanão é facto do TEMPO, nunca da taxa de quadros
//!
//! O ruído é de **VALOR** — amostrado numa grelha inteira e interpolado com `3t² − 2t³` —, logo
//! `deslocamento(…, t)` é uma função **contínua** de `t`. Dois relógios diferentes que passem pelo
//! mesmo instante lêem o mesmo sítio, e o caminho que a vista percorre é o mesmo a 60 e a 240 Hz.
//! ⛔ Um sorteio por QUADRO não tem esta propriedade: ali o abanão é função de quantas vezes
//! alguém olhou, que é a lei que o Painter pagou seis vezes (*o traço é facto do CAMINHO*).
//!
//! # ⚠️ O tecto do relógio, e de que recurso ele é
//!
//! O argumento do ruído é `frequencia × t` num `f32`, cuja mantissa tem **24 bits** ⇒ acima de
//! `2^24 ≈ 16,7 M` a grelha inteira deixa de ser representável e o ruído **congela**. À frequência
//! de fábrica (`20 Hz`) isso são **`9,7` dias de abanão contínuo** — e o `t` só corre enquanto há
//! trauma. *O limite é de precisão de representação, e está medido.*

/// **Os cinco números do abanão.** Config: ela viaja no componente que a guarda.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lei {
    /// Metros de deslocamento no pico (`trauma = 1`).
    pub amplitude: f32,
    /// Hz: quantas vezes por segundo a vista muda de direcção.
    pub frequencia: f32,
    /// Trauma por segundo que o decaimento leva.
    pub decaimento: f32,
    /// A potência a que o trauma é elevado — ver [`EXPOENTE_MIN`] e [`EXPOENTE_MAX`].
    pub expoente: u8,
    /// A semente do ruído. Duas câmeras com sementes diferentes tremem de maneiras diferentes.
    pub semente: u64,
}

/// O trauma satura aqui. ⚠️ Ele é uma FRACÇÃO do abanão máximo, não uma energia: dez explosões
/// juntas não podem abanar dez vezes mais do que a `amplitude` declarada.
pub const TRAUMA_MAX: f32 = 1.0;

/// ⛔ **`0` está fora, e a razão é que ele APAGA o trauma:** `trauma^0` é `1` para todo trauma > 0,
/// logo o abanão ficaria com a amplitude cheia até ao instante em que o decaimento chega a zero, e
/// depois pararia num degrau. *Um expoente que ignora o trauma não é uma curva, é um interruptor.*
pub const EXPOENTE_MIN: u8 = 1;

/// ⚠️⚠️ **Este tecto NÃO nomeia um recurso, e dizê-lo é a única forma honesta de o escrever**
/// (§0.0): nada na máquina se parte com `n = 9` — o expoente é uma **faixa de PRODUTO**, e a fonte
/// dela são as referências, que param aqui (*«trauma ao quadrado ou ao cubo»*, Eiserloh; o
/// `bevy_trauma_shake` ship `2`).
///
/// ⭐ **O que ele compra está medido** (`mede_a_cauda_por_expoente`): a fracção do movimento que
/// cai no primeiro quarto da vida do abanão é `1 − (3/4)^(n+1)` ⇒ `44 %` · `58 %` · `68 %` em
/// `1` · `2` · `3`. Acima disto o abanão deixa de ser um **tremor** e passa a ser um **safanão**,
/// que é outra coisa — subir o tecto é decisão do dono, com esta tabela ao lado.
pub const EXPOENTE_MAX: u8 = 3;

// ⭐⭐ **A faixa é ERRO DE COMPILAÇÃO e não um gate**, e a forma foi cobrada por um lint:
// um `assert!` de teste sobre duas constantes é **dobrado pelo compilador** antes de correr, e o
// clippy di-lo em voz alta (`this assertion has a constant value`) — é a mesma cura que a ordem do
// dono sobre o espaçamento do pincel afiado recebeu em 16/09. ⛔ O `0` APAGA o trauma; um `MAX`
// abaixo do `MIN` faria a `potencia` coagir para um valor fora da faixa que o painel pinta.
const _: () = assert!(EXPOENTE_MIN >= 1, "o expoente `0` apaga o trauma");
const _: () = assert!(
    EXPOENTE_MAX >= EXPOENTE_MIN,
    "a faixa tem de ter ao menos um valor"
);

/// ⭐ **splitmix64** — o gerador determinista da casa, o mesmo que a fábrica (TOP-20 #11) usa para
/// sortear um nascimento. ⚠️ Ele vive AQUI porque um gerador com **um** consumidor e sem porta é a
/// segunda cópia à espera de ser escrita — e a segunda cópia chegou nesta wave.
///
/// Avança o estado e devolve o número. ⛔ Ele nunca devolve a mesma sequência para duas sementes
/// diferentes, o que é o que faz duas câmeras tremerem de maneiras distintas.
pub fn baralha(estado: &mut u64) -> u64 {
    *estado = estado.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *estado;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Um valor em `-1..1` amarrado ao par `(semente, n)` — o mesmo par dá sempre o mesmo valor.
fn no_no(semente: u64, n: i64) -> f32 {
    // A rotação é o que impede `semente ^ n` de colidir com `semente' ^ n'` para sementes vizinhas.
    let mut estado = semente ^ (n as u64).rotate_left(32);
    let bits = baralha(&mut estado) >> 40; // 24 bits: o maior inteiro exacto num `f32`.
    // `/ 2^23 - 1` ⇒ `-1..1` com o zero representável.
    (bits as f32 / f32::from(1u16 << 12) / f32::from(1u16 << 11)) - 1.0
}

/// **Ruído de VALOR em `-1..1`, contínuo.** É ele que torna o abanão facto do TEMPO — ver o
/// cabeçalho do módulo.
///
/// ⚠️ **`3t² − 2t³` e não uma interpolação linear:** com a linear a velocidade salta em cada nó da
/// grelha, e uma vista que muda de velocidade de repente lê-se como um **corte**, que é metade do
/// que a folha da pesquisa proíbe (*«nunca brusco nem flutuante»*).
#[must_use]
pub fn ruido(semente: u64, t: f32) -> f32 {
    let base = t.floor();
    let f = t - base;
    // ⚠️ `as i64` satura em Rust desde a 1.45, então um `t` absurdo não dá comportamento
    // indefinido — ele fica preso na ponta da grelha, que é o congelamento descrito no cabeçalho.
    let n = base as i64;
    let a = no_no(semente, n);
    let b = no_no(semente, n.saturating_add(1));
    let u = f * f * (3.0 - 2.0 * f);
    a + (b - a) * u
}

/// A semente do eixo `y` é a do eixo `x` virada — ⛔ **nunca `semente + 1`**, que num splitmix
/// vizinho de um passo daria dois fluxos com a mesma cara.
const EIXO_Y: u64 = 0x5DEE_CE66_D18B_1A5F;

/// **O trauma sobe.** Satura em [`TRAUMA_MAX`] — ver o doc dela.
#[must_use]
pub fn acumula(trauma: f32, impulso: f32) -> f32 {
    (trauma + impulso.max(0.0)).min(TRAUMA_MAX) // CLAMP-OK: a saturação É a lei (ver `TRAUMA_MAX`)
}

/// **O trauma desce, linearmente no tempo.** ⚠️ Linear e não exponencial de propósito: um
/// decaimento exponencial **nunca chega a zero**, e uma vista que treme infinitamente pouco é uma
/// vista que nunca assenta — e que paga o ruído em todo quadro para sempre.
#[must_use]
pub fn decai(trauma: f32, decaimento: f32, dt: f32) -> f32 {
    (trauma - decaimento.max(0.0) * dt.max(0.0)).max(0.0)
}

/// **Quanto de um impulso emitido a `distancia` chega à vista:** `1` dentro de `dentro`, `0` a
/// partir de `fora`, e `3t² − 2t³` entre os dois.
///
/// ⚠️ **`fora <= dentro` é um CORTE DURO em `dentro`, e não um erro** — as duas primeiras cláusulas
/// já o exprimem, porque não existe `d` entre `fora` e `dentro` quando `fora` é o menor. *Uma
/// degenerescência que a lei responde sozinha não precisa de um braço a lembrá-la;* há gate.
#[must_use]
pub fn atenuacao(distancia: f32, dentro: f32, fora: f32) -> f32 {
    if distancia <= dentro {
        return 1.0;
    }
    if distancia >= fora {
        return 0.0;
    }
    let u = (distancia - dentro) / (fora - dentro);
    1.0 - u * u * (3.0 - 2.0 * u)
}

/// `trauma^n` por multiplicação repetida.
///
/// ⛔ **Nunca `f32::powi`:** ele baixa para `llvm.powi`, cuja associação de multiplicações não é
/// pinada entre plataformas — e esta lei entra numa vista que o `physics_ecs_c9` fotografa na
/// matriz de três sistemas operativos. *Um laço de `n` multiplicações é exacto em toda máquina.*
fn potencia(trauma: f32, expoente: u8) -> f32 {
    let n = expoente.clamp(EXPOENTE_MIN, EXPOENTE_MAX); // CLAMP-OK: a faixa é a do knob, com o porquê em cada const
    let mut r = trauma;
    for _ in 1..n {
        r *= trauma;
    }
    r
}

/// ⭐⭐⭐ **O deslocamento da vista, em metros.** Esta é a lei.
///
/// `t` é o relógio **do abanão** (segundos com trauma acumulados), e não o relógio do mundo: ele
/// só anda enquanto há o que abanar, o que mantém o argumento do ruído longe do tecto do `f32`
/// (cabeçalho) e faz duas explosões seguidas começarem em **fases diferentes**.
///
/// ⚠️ **Com `trauma = 0` a resposta é `[0, 0]` AO BIT** — `potencia(0, n)` é `0` para todo `n ≥ 1`
/// —, logo a vista de uma câmera sem trauma é byte-idêntica à de antes desta wave.
#[must_use]
pub fn deslocamento(lei: &Lei, trauma: f32, t: f32) -> [f32; 2] {
    let k = lei.amplitude * potencia(trauma.clamp(0.0, TRAUMA_MAX), lei.expoente); // CLAMP-OK: o domínio do trauma
    let fase = lei.frequencia * t;
    [
        k * ruido(lei.semente, fase),
        k * ruido(lei.semente ^ EIXO_Y, fase),
    ]
}

pub mod perfil;
pub use perfil::{Numeros, Perfil};

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
