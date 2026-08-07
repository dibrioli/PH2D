//! **O SSS PRÉ-INTEGRADO** — a metade do [`05.1`](../../docs/3D/05-Shading/05.1-Shader-de-runtime.md)
//! §2a, e o terceiro dos três canais que o Enio nomeou (*"SSS, AO, Cavity"*).
//!
//! # O princípio, que é o do doc inteiro
//!
//! > **Qualidade de cinema em jogo não vem de integrar mais; vem de PRÉ-integrar.**
//!
//! A luz que entra na pele não volta no ponto em que entrou: ela espalha por
//! milímetros de tecido e sai por perto, e é isso que faz um rosto parecer carne
//! em vez de plástico. Calcular esse espalhamento em runtime custa um passe de
//! blur em tela; **tabelá-lo custa uma amostra de textura**.
//!
//! # O que a tabela É — portado, não inventado (DIRETIVA §1)
//!
//! **Penner & Borshukov, *Pre-Integrated Skin Shading*** (SIGGRAPH 2011 / GPU Pro
//! 2). A observação: numa superfície localmente esférica de raio `r`, o difuso
//! visto num ponto onde `N·L = cos θ` **não** é `max(cos θ, 0)` — é a média da
//! iluminação da vizinhança, pesada pelo perfil de difusão:
//!
//! ```text
//! D(θ, r) = ∫ clamp(cos(θ + x), 0, 1) · R(2r·sin(x/2)) dx  /  ∫ R(2r·sin(x/2)) dx
//! ```
//!
//! (`2r·sin(x/2)` é a **corda** entre dois pontos separados pelo ângulo `x` no
//! círculo de raio `r` — a distância que a luz de fato percorre.)
//!
//! O perfil `R` é a soma de gaussianas de **d'Eon & Luebke** (*Advanced
//! Techniques for Realistic Real-Time Skin Rendering*, GPU Gems 3) — seis
//! gaussianas por canal, e é a assimetria entre canais que produz o vermelho na
//! borda da sombra. As variâncias e pesos abaixo são os publicados.
//!
//! ⚠️ **`D` é PAR no sinal da curvatura.** A substituição `x → −x` leva o
//! numerador nele mesmo (o clamp inclusive) e `R` só depende do módulo da corda,
//! então `D(θ, r) = D(θ, |r|)`. A tabela é indexada por `|κ|`, e o sinal fica
//! onde ele tem uso: no Cavity.
//!
//! # O eixo, e por que ele não é o `κ` do Cavity
//!
//! A difusão tem **escala física**. O eixo da tabela é o produto adimensional
//!
//! ```text
//! t = scatter · |κ|  =  scatter / R
//! ```
//!
//! — *quantos comprimentos de espalhamento cabem num raio de curvatura*. Formar
//! esse produto exige um `κ` em `1/comprimento`, e é por isso que
//! [`ph2d_mesh::world_curvature_at`] existe ao lado do canal do Cavity, que é
//! **invariante de escala por construção** (medido:
//! `ph2d-mesh/tests/measure_curvature_units.rs`).
//!
//! ⚠️ Isto CORRIGE o *"um dado, dois usos"* do `05.1` §2a: é o mesmo **gather**,
//! não o mesmo **número**.
//!
//! # A normalização, para o knob ter significado
//!
//! As variâncias publicadas estão em **mm² de pele humana**. Aqui elas são
//! divididas pela variância média do canal VERMELHO, o mais largo — então o
//! perfil passa a ter raio RMS **1** naquele canal, e o `scatter` do artista
//! passa a significar *"a luz viaja esta distância de mundo dentro do material"*.
//! Sem isso o knob seria um número em milímetros para uma peça em unidades
//! arbitrárias, e o artista o descobriria por tentativa.

use bytemuck::{Pod, Zeroable};

/// O lado da tabela. `128² × 4 B = 64 KiB` — e ela é assada **uma vez**.
///
/// ⚠️ **Medido, não escolhido** (`tests/sss_tests.rs::the_table_is_fine_enough`):
/// contra a integral exata, uma consulta bilinear nesta resolução erra no
/// máximo **1/255**, que é o próprio degrau da quantização de 8 bits — ou seja,
/// dobrar o lado não muda um byte do que sobe.
pub const LUT_SIZE: u32 = 128;

/// Quantas amostras a integral usa por célula.
///
/// ⚠️ Também medido: de 512 para 2048 a maior célula move **menos de 1/255**.
const INTEGRAL_SAMPLES: usize = 512;

/// O maior `t = scatter·|κ|` que a tabela representa. Acima dele a consulta
/// **satura** (o sampler clampa na borda), e isso é honesto: o controle para de
/// responder num regime que o material já não descreve.
///
/// ⚠️ **O meu primeiro palpite era 2, e a MEDIÇÃO o derrubou de duas maneiras
/// opostas** (`tests/measure_sss_curve.rs`):
///
/// **(a) A curva NÃO satura em 2** — ela tende a `1/π ≈ 0,318` (a média de
/// `clamp(cos)` sobre o círculo inteiro, que é o que espalhamento infinito
/// significa) e leva até `t ≈ 24` para chegar lá:
///
/// | `t` | 0 | 1 | 2 | 4 | 8 | 16 | 24 | ∞ |
/// |---|---|---|---|---|---|---|---|---|
/// | vaza no terminador | 0,000 | 0,070 | 0,106 | 0,156 | 0,231 | 0,304 | ~0,313 | 0,318 |
///
/// **(b) E perseguir a assíntota seria o teto errado**, porque `t = scatter/R` é
/// *quantos raios de curvatura a luz percorre*, e o material que este perfil
/// descreve vive muito abaixo disso:
///
/// | o que | `R` típico | `scatter` | `t` |
/// |---|---|---|---|
/// | bochecha | ~50 mm | ~2 mm | **0,04** |
/// | ponte do nariz | ~8 mm | ~2 mm | **0,25** |
/// | borda da orelha | ~3 mm | ~2 mm | **0,7** |
///
/// **4 é ~6× o caso extremo da pele real**, e a resolução da tabela o resolve com
/// folga (gate `the_table_is_fine_enough`). Um teto de 24 gastaria três quartos
/// das linhas num regime que nenhum material sólido ocupa.
pub const T_MAX: f32 = 4.0;

/// O perfil de difusão: `(variância_mm², [peso_r, peso_g, peso_b])`.
///
/// d'Eon & Luebke, GPU Gems 3 — o ajuste de seis gaussianas para pele. Os pesos
/// somam ~1 por canal, e é a cauda LARGA do vermelho (as duas últimas linhas)
/// que produz o sangramento avermelhado na borda da sombra.
const PROFILE: [(f64, [f64; 3]); 6] = [
    (0.0064, [0.233, 0.455, 0.649]),
    (0.0484, [0.100, 0.336, 0.344]),
    (0.187, [0.118, 0.198, 0.000]),
    (0.567, [0.113, 0.007, 0.007]),
    (1.99, [0.358, 0.004, 0.000]),
    (7.41, [0.078, 0.000, 0.000]),
];

/// A variância de referência — a média ponderada do canal VERMELHO, o mais
/// largo. Dividir por ela põe o raio RMS do vermelho em 1, que é o que dá
/// significado ao `scatter` do artista (ver o módulo).
///
/// ⚠️ **Derivada do próprio [`PROFILE`], nunca um literal:** um número copiado à
/// mão aqui ficaria velho no dia em que alguém trocasse o ajuste, e o sintoma
/// seria o knob mudar de escala sem ninguém ter mexido nele.
fn reference_variance() -> f64 {
    let (mut num, mut den) = (0.0, 0.0);
    for (v, w) in PROFILE {
        num += w[0] * v;
        den += w[0];
    }
    num / den
}

/// `R(d)` — o perfil radial, por canal, já normalizado (ver o módulo).
///
/// Cada gaussiana é a 2-D `1/(2πv)·exp(−d²/2v)`, que é a forma em que o perfil
/// de difusão é publicado: ele descreve energia por unidade de ÁREA em torno do
/// ponto de entrada.
fn profile(d: f64) -> [f64; 3] {
    let vref = reference_variance();
    let mut out = [0.0f64; 3];
    for (v, w) in PROFILE {
        let v = v / vref;
        let g = (-d * d / (2.0 * v)).exp() / (2.0 * core::f64::consts::PI * v);
        for c in 0..3 {
            out[c] += w[c] * g;
        }
    }
    out
}

/// **A resposta pré-integrada**, exata (a integral, não a tabela).
///
/// `n_dot_l` em `[-1, 1]` e `t = scatter·|κ|` em `[0, ∞)`. Com `t = 0` a
/// superfície é plana e a resposta é o difuso de sempre — **e o caso é tratado
/// por ramo, não por limite numérico**: com `r = ∞` o perfil colapsa num delta e
/// a razão vira `0/0`.
///
/// É esta função que a tabela aproxima, e é contra ela que o gate mede o erro —
/// nunca contra uma segunda tabela.
#[must_use]
pub fn integrate(n_dot_l: f32, t: f32) -> [f32; 3] {
    let cos_theta = f64::from(n_dot_l.clamp(-1.0, 1.0));
    // ⚠️ **`abs`, e o guard é `== 0`.** A primeira versão escrevia `t <= 0.0` e
    // devolvia o lambert — o que faz `plano` e `côncavo` significarem a mesma
    // coisa. O gate `the_response_does_not_know_the_sign_of_the_curvature`
    // nasceu vermelho nisto, e o defeito teria chegado ao produto: o `κ` que o
    // shader passa **tem sinal**, e uma narina teria sido tratada como uma
    // superfície plana.
    let t = t.abs();
    if t == 0.0 {
        let d = cos_theta.max(0.0) as f32;
        return [d, d, d];
    }
    let theta = cos_theta.acos();
    let r = 1.0 / f64::from(t);
    let (mut num, mut den) = ([0.0f64; 3], [0.0f64; 3]);
    let n = INTEGRAL_SAMPLES;
    for i in 0..n {
        // Ponto médio de cada fatia — a regra do retângulo centrado, que numa
        // função lisa e periódica converge tão rápido quanto Simpson.
        let x =
            -core::f64::consts::PI + (i as f64 + 0.5) * (2.0 * core::f64::consts::PI / n as f64);
        let lit = (theta + x).cos().clamp(0.0, 1.0);
        let chord = 2.0 * r * (x * 0.5).sin();
        let w = profile(chord.abs());
        for c in 0..3 {
            num[c] += lit * w[c];
            den[c] += w[c];
        }
    }
    let mut out = [0.0f32; 3];
    for c in 0..3 {
        out[c] = if den[c] > 0.0 {
            (num[c] / den[c]) as f32
        } else {
            cos_theta.max(0.0) as f32
        };
    }
    out
}

/// A tabela como o device a lê: `RGBA8`, linha por linha, `x = N·L`, `y = t`.
///
/// ⚠️ **`Rgba8Unorm` e não meio-float**, e o motivo é honesto: o formato de 16
/// bits exigiria uma conversão `f32 → f16` que este repo não tem sem uma
/// dependência nova, e o gate mede que 8 bits **bastam** (o erro da tabela é
/// menor que o próprio degrau da quantização). É também o formato que o artigo
/// original shipa.
#[must_use]
pub fn bake_lut() -> Vec<u8> {
    let n = LUT_SIZE as usize;
    let mut out = vec![0u8; n * n * 4];
    for y in 0..n {
        let t = lut_t(y);
        for x in 0..n {
            let d = integrate(lut_n_dot_l(x), t);
            let o = (y * n + x) * 4;
            for c in 0..3 {
                out[o + c] = (d[c].clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
            }
            out[o + 3] = 255;
        }
    }
    out
}

/// O `N·L` que a coluna `x` representa — **centro de texel**, e é isso que faz a
/// consulta bilinear do shader cair sobre os valores assados em vez de meio
/// texel ao lado ([[feedback_derived_coordinate_seed_must_match_sample]]).
#[must_use]
pub fn lut_n_dot_l(x: usize) -> f32 {
    let u = (x as f32 + 0.5) / LUT_SIZE as f32;
    u * 2.0 - 1.0
}

/// O `t = scatter·|κ|` que a linha `y` representa — centro de texel, pela mesma
/// razão da irmã.
#[must_use]
pub fn lut_t(y: usize) -> f32 {
    ((y as f32 + 0.5) / LUT_SIZE as f32) * T_MAX
}

/// **COMO o artista pede o espalhamento.**
///
/// ⚠️ O corte é o mesmo do [`crate::ssao`]: *quanto* aparece aqui, e o *quão
/// escuro* de cada canal mora no [`crate::Shade`] junto da cavidade e do AO,
/// porque é lá que ficam as opções de sombreamento que se arrastam.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SssParams {
    /// **Quanto do difuso passa a ser a resposta espalhada**, `0..1`.
    ///
    /// `0` é o barro de sempre, **ao byte** — a tabela nem é consultada.
    pub strength: f32,
    /// **Até onde a luz viaja dentro do material**, em unidades de MUNDO.
    ///
    /// É o outro fator do eixo da tabela (`t = scatter · |κ|`): ele decide se
    /// uma dobra de raio `R` conta como *quase plana* ou como *fina o bastante
    /// para a luz atravessar*.
    pub scatter: f32,
}

/// **O espalhamento nasce como uma FRAÇÃO da peça, não como um número absoluto**
/// — a lição que o `bake_ao` e o [`crate::ssao`] já pagaram (uma miniatura e um
/// colosso pedem números diferentes, e o artista descobre isso reclamando de que
/// o controle não faz nada).
///
/// ⚠️ **O meu palpite era 0,02, e ele fazia do canal um knob MORTO.** A sonda
/// `measure_which_scatter_makes_the_channel_visible` mediu o que cada fração
/// produz numa esfera de raio 1 com sete traços (`|κ|` mediano 1,00, p99 8,30):
///
/// | fração | `t` mediano | `t` p99 | vaza no terminador |
/// |---|---|---|---|
/// | **0,02** (o palpite) | 0,041 | 0,34 | **0,0066** — 1,7 níveis de 255, invisível |
/// | 0,05 | 0,103 | 0,85 | 0,0090 |
/// | 0,10 | 0,206 | 1,71 | 0,0173 |
/// | **0,25** | **~0,52** | ~4,3 (satura) | **~0,042** — 11 níveis, à vista |
/// | 0,40 | 0,823 | 6,84 | 0,0615 |
///
/// ⚠️ **E a leitura que corrige o raciocínio, não só o número:** `t = scatter·|κ|`
/// mede a curvatura das **FEATURES**, não a da peça. Um vinco tem `|κ|` oito
/// vezes o do corpo (p99 8,30 contra mediana 1,00), então a fração que põe a
/// superfície típica em `t ≈ 0,5` põe os vincos **no teto da tabela** — que é
/// exatamente o certo: uma aresta fina é o que a luz de fato atravessa.
///
/// ⚠️ Isto é o DEFAULT do alcance, e não do efeito: a força nasce em **zero**
/// (ver [`SssParams::default`]), então nada muda até o artista ligar. O que esta
/// fração garante é que, ao ligar, ele **veja**.
pub const SCATTER_FRACTION: f32 = 0.25;

impl SssParams {
    /// Os parâmetros semeados pelo tamanho da peça.
    #[must_use]
    pub fn for_bounds(bounds: ph2d_mesh::Aabb) -> Self {
        let e = [
            bounds.max[0] - bounds.min[0],
            bounds.max[1] - bounds.min[1],
            bounds.max[2] - bounds.min[2],
        ];
        let longest = e[0].max(e[1]).max(e[2]).max(1e-4);
        Self {
            scatter: longest * SCATTER_FRACTION,
            ..Self::default()
        }
    }
}

impl Default for SssParams {
    fn default() -> Self {
        Self {
            // ⚠️ **Zero, e a assimetria com o AO de tela é deliberada.** Aquele é
            // uma MEDIÇÃO da forma, e mostrá-la é o default honesto. Este é um
            // MATERIAL: barro não é pele, e ligar o espalhamento por padrão
            // mudaria a aparência de toda escultura já feita.
            strength: 0.0,
            // O piso para quem não semear pela peça — o produto sempre chama o
            // [`Self::for_bounds`], e o gate `the_scatter_is_a_fraction_of_the_piece`
            // é quem garante que a semente chegue.
            scatter: SCATTER_FRACTION,
        }
    }
}

/// Os parâmetros como o shader os lê. Ver [`SssParams`].
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct SssRaw {
    /// `x` = força · `y` = o `scatter` já dividido por [`T_MAX`], que é o que
    /// leva `|κ|` direto à coordenada `v` da textura · `zw` = reservado.
    pub params: [f32; 4],
}

impl SssRaw {
    /// Bytes do uniform.
    pub const SIZE: usize = std::mem::size_of::<Self>();

    /// Empacota, **clampado na porta** — o device não tem opinião.
    ///
    /// ⚠️ A divisão por [`T_MAX`] acontece **aqui e só aqui**: o shader
    /// multiplica `|κ|` por este número e usa o resultado como `v`. Fazer a
    /// conversão no WGSL seria a segunda cópia do teto da tabela, e ela
    /// divergiria no dia em que o teto mudasse — com o sintoma sendo um
    /// espalhamento levemente errado que ninguém consegue nomear.
    #[must_use]
    pub fn pack(p: SssParams) -> Self {
        Self {
            params: [
                p.strength.clamp(0.0, 1.0),
                p.scatter.max(0.0) / T_MAX,
                0.0,
                0.0,
            ],
        }
    }
}

#[cfg(test)]
#[path = "sss_tests.rs"]
mod tests;
