//! **QUANTOS PASSES ESTE PINCEL FAZ, E COM QUE PESO** — irmão do
//! [`super::brush`] e do [`super::brush_scale`], cortado por ASSUNTO.
//!
//! O `brush.rs` diz **o que um pincel É**; o `brush_scale.rs` diz **como cada
//! número do artista vira o número do kernel**. Aqui mora a terceira pergunta,
//! que nenhum dos dois respondia: **quantas vezes o laço do dab roda, e com que
//! fator**.
//!
//! ⚠️ **Ela nasceu com UM consumidor e a resposta dele é uma medição** — o
//! Smooth ENCOLHE, e a §0 mandou medir antes de construir
//! (`tests/measure_smooth_shrinkage.rs`, esfera unitária, `Falloff::Constant`,
//! força 1, pela porta do produto):
//!
//! | passadas | raio médio | encolhimento |
//! |---|---|---|
//! | 1 | 0,999092 | 0,09 % |
//! | 10 | 0,990924 | 0,91 % |
//! | 20 | 0,981938 | 1,81 % |
//! | 40 | 0,964246 | **3,58 %** |
//!
//! ⚠️ **O número não é o achado — a FORMA é.** Ele é **linear e não satura**: o
//! laplaciano umbrella contrai o volume a cada aplicação, para sempre. *Alisar
//! até ficar liso é alisar até sumir*, que é a objeção com que o Vollmer/Mencl/
//! Müller 1999 abre.
//!
//! # A cura, e por que é o Taubin e não o HC
//!
//! **Taubin 1995** (SIGGRAPH, *"A signal processing approach to fair surface
//! design"*) alterna um passo laplaciano de peso `λ > 0` com um de peso
//! `μ < 0`, `|μ| > λ`: o primeiro contrai, o segundo devolve. Medido pela porta
//! do PRODUTO, o mesmo gesto nos dois modos
//! (`tests/taubin_pair.rs::the_numbers_the_gates_assert`):
//!
//! | dabs | `S` | `L` |
//! |---|---|---|
//! | 1 | 0,0908 % | **−0,0008 %** |
//! | 10 | 0,9076 % | −0,0079 % |
//! | 20 | 1,8062 % | −0,0159 % |
//! | 40 | 3,5754 % | −0,0318 % |
//!
//! ⚠️ **A coluna `L` MOVEU quando o par passou a correr sobre o operador que o
//! paper nomeia** (o cotangente — ver [`RingOperator`]): ela era
//! `−0,0011 / −0,0104 / −0,0206 / −0,0409` sobre o laplaciano uniforme. O
//! resíduo encolheu **22 %** e o sinal não mudou. *O `S` é o contrato de
//! paridade e não se move; o `L` é o que a literatura declara, e mover para mais
//! perto do paper é o que ele existe para fazer.*
//!
//! ⚠️ **A INCLINAÇÃO cai ~87×, e o resíduo NÃO é limitado — eu escrevi
//! *"limitado"* primeiro e a tabela me desmentiu.** Deduzi o teto do SINAL ter
//! invertido (o `μ` sobre-corrige, então a esfera cresce em vez de encolher) e
//! isso não é um teto: as duas colunas são **lineares no número de dabs**, e o
//! que muda é a taxa (0,0894 %/dab contra 0,00102 %/dab). Alisar para sempre
//! ainda deriva — 87 vezes mais devagar, e para o outro lado.
//!
//! ⚠️ **E o par NÃO é afinado para anular o resíduo nesta esfera:** o `k_PB` é
//! o do paper, e um `μ` ajustado até a coluna zerar seria um número ajustado a
//! UMA fixture, que passa a mentir na malha seguinte.
//!
//! ⚠️ **O §3 do plano dava ao Smooth o HC, e a troca tem TRÊS razões de
//! código, não de gosto:**
//!
//! 1. **As duas metades do Taubin já são verbos deste motor.** O passo `λ` é o
//!    [`Verb::Smooth`] e o passo `μ` é o [`Verb::Sharpen`] — *"o laplaciano com
//!    o sinal trocado"*, palavra por palavra o que o `stroke_target.rs` já
//!    escreve. Não há kernel novo: há um FATOR COM SINAL.
//! 2. **O HC pede estado que este motor não tem.** Ele guarda um vetor `b` por
//!    vértice e faz uma **segunda passada sobre o anel** para tirar a média
//!    dele; o `compute_target` é função **pura por-vértice**, então o HC seria
//!    um buffer de pegada com ciclo de vida próprio — a classe do
//!    `render_inflate` do Painter, uma wave inteira.
//! 3. **O HC precisa do ORIGINAL, e isso briga com o artista.** O `o` dele é a
//!    pose do pen-down, então num traço longo ele PUXA de volta para ela — o
//!    oposto de *"passar de novo alisa mais"*, que é o que uma pincelada faz.
//!
//! ⇒ O HC fica **NOMEADO como a alternativa medida contra esta**, não
//! esquecido. Se ele voltar à mesa, volta contra `0,0159 %` em vinte dabs, não
//! contra `1,8062 %`.
//!
//! ⚠️ **E ele deixou de ser hipotético desde então:** o `SURFACE_SMOOTH` shipou
//! como VERBO próprio (o [`Verb::SurfaceSmooth`], que é o HC com o `b` num plano
//! da sessão), então a razão 2 acima — *"o HC pede estado que este motor não
//! tem"* — descreve o que era verdade quando o par foi escolhido, e o que mudou
//! foi o motor. As razões 1 e 3 seguem de pé, e são elas que mantêm o par no
//! `l-mode` do Smooth: um traço longo de HC PUXA de volta para a pose do
//! pen-down, e o Taubin não.
//!
//! # O que a lei de um passe NÃO muda
//!
//! ⚠️ **Todo pincel que não é o `L` do Smooth devolve EXATAMENTE um passe — ele
//! próprio, com fator `1.0`** — e `x * 1.0 == x` no IEEE-754 para todo finito.
//! É isso que torna o resto do motor **byte-idêntico por construção** em vez de
//! por promessa, e é o que o gate afirma.

use super::*;

/// **UM PASSE DE RELAXAÇÃO** — o fator, **com sinal**, que escala o peso de
/// cada vértice.
///
/// ⚠️ **Um tipo e não um `f32` solto:** o SINAL é a coisa inteira (um passe
/// negativo é o `μ` do Taubin, e ele lê como *"suavizar para o lado de fora"*),
/// e um `&'static [f32]` devolvido por `passes()` não diria de que grandeza os
/// números são. O molde é o da [`crate::GripLaw`]: uma tabela nomeada, não uma
/// tupla que o chamador decodifica.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pass {
    /// O fator que multiplica o peso (`falloff × intensidade × máscara`) deste
    /// vértice **neste passe**. `1.0` é o pincel de sempre.
    pub weight: f32,
}

/// **λ** — o passo que CONTRAI, o Smooth de sempre com um terço do peso.
///
/// ⚠️ **O par (λ, μ) não é ajustável pelo artista, e isso é deliberado:** ele é
/// a *banda de passagem* do filtro, não uma força — quem controla o quanto é o
/// slider de Strength, que multiplica os dois igualmente e preserva a razão.
/// Um segundo knob aqui seria um número que o artista não tem como julgar e que
/// quebra a propriedade no primeiro arrasto.
pub const TAUBIN_LAMBDA: f32 = 0.33;

/// **`k_PB`** — a frequência de passagem do filtro, o `0.1` do próprio paper
/// (Taubin 1995, §5: *"we have used k_PB = 0.1 in all our experiments"*).
pub const TAUBIN_PASS_BAND: f32 = 0.1;

/// **μ** — o passo que DEVOLVE, e ele é **DERIVADO**, nunca escolhido.
///
/// A relação do paper é `1/λ + 1/μ = k_PB`; com λ = 0,33 e `k_PB` = 0,1 sai
/// **−0,341262**. Escrever um literal aqui seria a segunda resposta que diverge
/// no dia em que alguém mover o λ — e a propriedade que faz o par funcionar é
/// exatamente `|μ| > λ`, que a fórmula garante e um par digitado à mão não.
pub const TAUBIN_MU: f32 = 1.0 / (TAUBIN_PASS_BAND - 1.0 / TAUBIN_LAMBDA);

/// O pincel de sempre: **um** passe, ele próprio, com o peso intacto.
const SOLE: [Pass; 1] = [Pass { weight: 1.0 }];

/// O par λ|μ — **um dab é um PAR**, e é isso que a §7.6 do plano nomeia como a
/// única parte estrutural desta wave: um traço de `N` dabs seria `N` passos `λ`
/// sem nenhum `μ`, e aí o `L` encolheria igual ao `S`.
const TAUBIN: [Pass; 2] = [
    Pass {
        weight: TAUBIN_LAMBDA,
    },
    Pass { weight: TAUBIN_MU },
];

/// **SOBRE QUE OPERADOR o alvo do anel é construído** — a segunda metade
/// imperativa do `l-mode` do Smooth.
///
/// ⚠️ **O §4 do plano 21 já dizia onde este operador vive, e eu li a linha ao
/// contrário na primeira vez.** A tabela credita Meyer/Desbrun/Schröder/Barr
/// 2003 com *"laplaciano por **cotangentes**, normal de curvatura média"* para
/// **"Inflate · o operador dos dois acima"** — e *os dois acima* são o Taubin
/// 1995 e o fairing implícito de Desbrun 1999. Ou seja: o cotangente é o
/// **operador sobre o qual o par λ|μ deveria correr**, e o nosso corria sobre o
/// uniforme.
///
/// # ⛔ A outra metade da mesma célula foi RECUSADA por medição
///
/// A célula do **Inflate** pede a *normal de curvatura média* como direção, e
/// isso está medido em `tests/measure_curvature_normal.rs` — **não ship**:
///
/// | fixture | côncavos | eixo médio | eixo p95 |
/// |---|---|---|---|
/// | `sculpt_sphere` (a malha DEFAULT) | 0,0 % | **0,003°** | 0,020° |
/// | `uv_sphere` 24×32 | 0,0 % | 0,213° | 0,933° |
/// | depois de 4 traços | 3,3 % | **0,709°** | 2,030° |
/// | `uv_sphere_shuffled` | 12,6 % | 26,3° | **87,9°** |
///
/// Duas leituras, e as duas recusam. **(1)** Na malha que o artista de facto tem
/// o eixo diverge **três milésimos de grau** — um chip que não move um pixel.
/// **(2)** Onde ele diverge, diverge **por não ser uma normal**: `p95 = 87,9°` é
/// quase TANGENTE, porque numa malha ruidosa o vetor de curvatura segue a ruga e
/// não a superfície. E a coluna *côncavos* nomeia o resto: `K = 2·κ_H·n` carrega
/// a curvatura **com sinal**, então numa cova ele aponta para DENTRO —
/// caminhar por ele não é inflar, é **afiar**, que é outro verbo e já tem
/// l-mode próprio (o μ do Taubin). O mecanismo inteiro mora no módulo
/// `ph2d_mesh::cotangent`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum RingOperator {
    /// A média **não-ponderada** do anel — o `Smooth.js` da referência, e o que
    /// o app sempre rodou. Todo peso vale 1, então ela nunca degenera.
    #[default]
    Uniform,
    /// A média ponderada por **cotangentes** — o Laplace-Beltrami discreto.
    ///
    /// ⚠️ **O que ele compra está MEDIDO** (`ph2d-mesh`,
    /// `tests/measure_cotangent_smoothing.rs`, `uv_sphere` 24×32, força cheia —
    /// a *deriva tangencial* é quanto o vértice escorregou AO LONGO da
    /// superfície, que é o que "a parametrização se distorce" significa em
    /// número):
    ///
    /// | passes | operador | raio médio | deriva tangencial |
    /// |---|---|---|---|
    /// | 1 | uniforme | 0,990715 | 0,003164 |
    /// | 1 | **cotangente** | **0,992060** | **0,000014** |
    /// | 4 | uniforme | 0,963384 | 0,012277 |
    /// | 4 | **cotangente** | **0,968796** | **0,000322** |
    /// | 16 | uniforme | 0,864052 | 0,043883 |
    /// | 16 | **cotangente** | **0,882856** | **0,004974** |
    ///
    /// **226× · 38× · 8,8× menos deriva**, e encolhe menos nos três. Numa malha
    /// irregular (`uv_sphere_shuffled`, 4 passes) a deriva cai **0,047561 →
    /// 0,007172**.
    ///
    /// ⚠️ **A fixture serve por ACIDENTE da própria construção, e é isso que a
    /// torna honesta:** numa esfera de anéis quadrados os dois operadores
    /// coincidem por SIMETRIA e a sonda mediria zero; na `uv_sphere` o passo em
    /// longitude encolhe com `sin θ` e o em latitude não, então a anisotropia
    /// aparece sem ninguém a fabricar.
    Cotangent,
}

impl Brush {
    /// **O OPERADOR DE ANEL DESTE PINCEL** — irmão do [`Self::passes`], com a
    /// MESMA chave.
    ///
    /// ⚠️ **A chave tem de ser a mesma, e é o único jeito de o chip não
    /// mentir:** o `L` do Smooth **é** o Taubin, e o Taubin só é o que o paper
    /// descreve se correr sobre o operador que o paper nomeia. Chaveados
    /// diferente, o par λ|μ rodaria sobre um operador e o chip anunciaria outro.
    ///
    /// ⚠️ **E é escrito, não DERIVADO de `passes().len() > 1`** — pela mesma
    /// razão que o `RefMode::declares` recusa a derivada: *fazer dois passes* e
    /// *usar o operador geométrico* são perguntas diferentes que hoje têm a
    /// mesma resposta. O gate `the_taubin_pair_runs_on_the_cotangent_operator`
    /// pina a coincidência em vez de a deixar como disciplina.
    #[must_use]
    pub fn ring_operator(&self) -> RingOperator {
        match (self.verb, self.mode) {
            (Verb::Smooth, crate::RefMode::L) => RingOperator::Cotangent,
            _ => RingOperator::Uniform,
        }
    }

    /// **OS PASSES DESTE PINCEL** — a porta única de *quantas vezes o laço do
    /// dab roda*.
    ///
    /// ⚠️ **A pergunta é do par (verbo, modo), e não de um dos dois:** o
    /// Taubin é o que a LITERATURA declara sobre o SMOOTH, e nem *"o modo `L`"*
    /// nem *"o verbo Smooth"* sozinhos respondem. É a mesma forma do
    /// [`Verb::profile`], e é ela que mantém o `L` sendo oferecido exatamente
    /// onde ele tem conteúdo — o gate
    /// `the_literature_mode_is_offered_exactly_where_it_declares_a_law` afirma
    /// a bi-implicação em vez de a deixar como disciplina.
    #[must_use]
    pub fn passes(&self) -> &'static [Pass] {
        match (self.verb, self.mode) {
            // ⚠️ **Só o Smooth, e o Sharpen NÃO entra.** O par λ|μ do paper
            // descreve um filtro passa-baixa que não encolhe; um *sharpen* é a
            // banda de passagem no outro lado, e o Taubin não fala dele. Dar-lhe
            // o par seria pôr o nome de um paper numa lei que ele não declara —
            // exatamente o chip mentiroso que o `RefMode::declares` existe para
            // impedir.
            (Verb::Smooth, crate::RefMode::L) => &TAUBIN,
            _ => &SOLE,
        }
    }
}
