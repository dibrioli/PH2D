//! **AS OPÇÕES DE SOMBREAMENTO DO BARRO** — o que o artista escolhe sobre como
//! a forma é lida, separado de com que luz ela é acesa.
//!
//! Hoje há uma: a **CAVIDADE** (`docs/3D/05.1` §4), que o doc nomeia como *o
//! canal que mais melhora a leitura de uma escultura por unidade de custo*. As
//! próximas (SSS, AO, IBL) entram aqui pelo mesmo caminho, e é por isso que ele
//! é um struct e não um `f32` solto.
//!
//! # Por que UNIFORM e não `const` de permutação
//!
//! O `docs/3D/05.1` fecha pedindo *"um WGSL único, com `const` de permutação
//! resolvidos na compilação do pipeline … não `if` em runtime"*. Isso vale para
//! **capacidades** (`SSS on/off`, `IBL on/off`) — coisas que mudam o corpo do
//! shader e cujo custo é pago por quem não as usa. Uma **quantidade** que o
//! artista arrasta é o oposto: recompilar um pipeline por posição de slider é
//! uma trava de meio segundo por passo do gesto.
//!
//! ⚠️ E o zero **não** precisa de permutação para ser barato: o termo é uma
//! multiplicação por `1.0`, não um passe.

use bytemuck::{Pod, Zeroable};

/// **QUANTO a cavidade escurece a fresta e clareia a crista.**
///
/// Ela vale para os DOIS sinais, e isso é uma decisão sobre o modelo e não uma
/// economia: a curvatura é *um* número com sinal, e escurecer o côncavo e
/// clarear o convexo são as duas metades da mesma multiplicação
/// (`1 − amount × k`). O `docs/3D/05.1` §4 fala em dois sliders — *Cavity* e
/// *Edge Wear* — e eles são a UI de um MATERIAL (sujeira na fresta × tinta gasta
/// na quina são histórias físicas diferentes, com quantidades diferentes). Para
/// **ler forma**, que é o que esta wave entrega, um número simétrico é o modelo
/// honesto; inventar o segundo agora seria um knob que nenhum gesto alcança.
///
/// ⚠️ **Se o smoke disser que os dois lados querem quantidades diferentes, é ELE
/// que parte este número em dois** — o oráculo disso é o olho, não a aritmética.
pub const DEFAULT_CAVITY: f32 = 0.0;

/// **O GANHO** que leva a curvatura crua à faixa que o olho usa.
///
/// **MEDIDO** (`measure_curvature`, esferas trianguladas com sete traços de
/// Draw — o que uma mão faz nos primeiros segundos):
///
/// | malha | `k` mediano | `\|k\|` p99 | `\|k\|` máximo |
/// |---|---|---|---|
/// | esfera CRUA 48×72 | −0,0372 | 0,045 | 0,045 |
/// | esculpida 48×72 | −0,0372 | **0,305** | 0,685 |
/// | esculpida 96×144 | −0,0189 | **0,140** | 0,704 |
///
/// ⚠️ **A tabela traz um fato que eu não tinha previsto, e ele é a razão de o
/// ganho poder ser fixo:** a curvatura de FUNDO (a da própria esfera) cai pela
/// metade quando a tesselação dobra — exatamente o `−h/(2R)` —, mas a de um
/// VINCO fica onde está (0,685 e 0,704). Um vinco é um vinco em qualquer
/// densidade. Então o que o canal desenha — o CONTRASTE entre fresta e fundo —
/// não é função de quantos triângulos a peça tem, e um ganho constante serve as
/// duas.
///
/// **4,0 satura em `\|k\| ≥ 0,25`**, que fica entre os dois p99 medidos: o 1%
/// mais vincado clampa e todo o resto responde proporcionalmente. O fundo liso
/// escurece 7 a 15% em `cavity = 1` — uniforme, portanto invisível como
/// artefato, que é o que sobra depois de o contraste ter sido gasto no que
/// interessa.
pub const CAVITY_GAIN: f32 = 4.0;

/// As opções, como o fragment shader as lê.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct ShadeRaw {
    /// Quanto da cavidade entra. `0` = o barro liso da W3, **ao byte**.
    pub cavity: f32,
    /// ⚠️ Declarado e não implícito: um `f32` solto num uniform alinha em 16 B de
    /// qualquer jeito, e os três `f32` que sobram são exatamente onde SSS, AO e
    /// IBL vão pousar sem mexer no layout.
    pub _pad: [f32; 3],
}

impl Default for ShadeRaw {
    fn default() -> Self {
        Self {
            cavity: DEFAULT_CAVITY,
            _pad: [0.0; 3],
        }
    }
}

impl ShadeRaw {
    /// Bytes do uniform. Constante, para o buffer nascer com o tamanho certo.
    pub const SIZE: usize = std::mem::size_of::<Self>();

    /// Empacota a quantidade que o artista escolheu, **clampada na porta**.
    ///
    /// ⚠️ O clamp mora aqui e não no chamador porque o device não tem opinião: um
    /// `cavity` de 3 faria `1 − 3k` ficar negativo numa fresta funda e o barro
    /// sairia com a cor invertida. Clampar no shader seria a segunda cópia da
    /// mesma regra, e ela divergiria no dia em que o painel chegasse.
    #[must_use]
    pub fn pack(cavity: f32) -> Self {
        Self {
            cavity: cavity.clamp(0.0, 1.0),
            _pad: [0.0; 3],
        }
    }
}
