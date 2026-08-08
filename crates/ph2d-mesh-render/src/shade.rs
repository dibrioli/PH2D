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

/// **Quanto do AO assado entra por default: NADA.**
///
/// ⚠️ Não é timidez, é a única resposta que não muda a tela sozinha. O canal
/// nasce ausente e sobe como `1` em toda parte (ver `ao_of`), então com uma
/// força qualquer acima de zero a peça renderizaria igual **até o instante do
/// primeiro bake** — e aí
/// escureceria sem ninguém ter tocado num controle, que é a classe de mudança
/// que o artista não consegue reportar.
pub const DEFAULT_AO_STRENGTH: f32 = 0.0;

/// **Quanto do AO DE TELA entra por default: TUDO.**
///
/// ⚠️ **O oposto do [`DEFAULT_AO_STRENGTH`], e a assimetria é a wave inteira.**
/// O assado nasce em zero porque é um canal que **não existe** até alguém apertar
/// um botão, e ligá-lo por default faria a peça escurecer sozinha no instante do
/// primeiro bake. Este é medido **todo frame** a partir do que está na tela: ele
/// existe sempre que a forma existe, e nunca fica velho.
///
/// ⚠️ **E ele não muda nada até alguém MEDIR:** o barro lê a oclusão de um canal
/// que vale zero enquanto ninguém rodar o [`crate::MeshRenderer::render_ssao`] —
/// então este `1.0` é *"mostre o que foi medido"*, não *"escureça"*.
pub const DEFAULT_SSAO_STRENGTH: f32 = 1.0;

/// **OS MATERIAIS DO MATCAP**, na ordem em que o shader os numera.
///
/// ⚠️ **Os NÚMEROS ficam no WGSL e os NOMES aqui, e não há uma terceira cópia.**
/// Um material de matcap é um punhado de cores e expoentes que ninguém do lado
/// da CPU lê: o shader é o único consumidor, então duplicá-los aqui seria uma
/// segunda resposta a *"como a pérola é"* — a que fica velha na primeira
/// afinação. O que a CPU precisa saber é **quantos há** e **como se chamam**,
/// que é o que o painel pinta; a igualdade das duas contagens é gateada.
pub const MATCAPS: [&str; 6] = ["Clay", "Pearl", "Skin", "Jade", "Metal", "Wax"];

/// **COMO O BARRO É MOSTRADO** — as opções de vista, num tipo só.
///
/// ⚠️ Um struct e não três argumentos soltos no [`crate::MeshRenderer::render`]:
/// ele já carrega nove, e cada opção de vista nova o empurraria mais. E o corte
/// é honesto — *quanto de cavidade*, *com que luz* e *com ou sem a malha* são as
/// três respostas à mesma pergunta, e elas viajam juntas do painel ao device.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shade {
    /// Quanto a curvatura escurece a fresta e clareia a crista.
    pub cavity: f32,
    /// **Com que luz** — `None` é o RIG DO ARTISTA (a luz do documento, a mesma
    /// que acende a tinta ao lado); `Some(i)` é o matcap `i` de [`MATCAPS`], a
    /// luz do OLHO.
    ///
    /// ⚠️ `Option` e não um índice com `0` reservado: *"nenhum matcap"* não é um
    /// matcap, e um sentinela obrigaria todo leitor a saber disso. A conversão
    /// para o sentinela do uniform acontece **uma vez**, no [`ShadeRaw::pack`].
    pub matcap: Option<u8>,
    /// **Quanto do AO assado entra.** `0` = o barro sem oclusão, e é o default:
    /// um canal que nem foi assado não pode escurecer nada.
    ///
    /// ⚠️ **Ele NÃO é irmão da `cavity`, e a diferença decide o default.** A
    /// cavidade é derivada e existe em toda malha, então ligá-la por padrão é
    /// mostrar um dado que já está lá. O AO só existe depois de um BAKE
    /// explícito, e um default > 0 faria a peça não-assada renderizar igual (o
    /// canal é 1 em toda parte) até o instante do primeiro bake — quando ela
    /// escureceria sozinha, sem ninguém ter mexido num controle.
    pub ao: f32,
    /// **Quanto do AO de TELA entra.** `1` = todo ele, e é o default.
    ///
    /// ⚠️ Ele e o [`Self::ao`] respondem a mesma pergunta em ALCANCES
    /// diferentes: o assado mede metros de campo SDF e enxerga o corpo inteiro
    /// em qualquer direção; este mede um raio em torno do pixel e só vê o que
    /// está na tela. O barro os compõe pelo MENOS-OCLUÍDO (ver `mesh.wgsl`), e
    /// não pelo produto — dois canais que descrevem a mesma sombra multiplicados
    /// a escureceriam em dobro exatamente onde os dois acertam.
    pub ssao: f32,
    /// **O ESPALHAMENTO SUB-SUPERFICIAL** (`crate::sss`) — quanto, e até onde.
    ///
    /// ⚠️ Um struct aninhado e não dois `f32` soltos: `strength` e `scatter` são
    /// as duas metades de UMA resposta (*como este material conduz luz por
    /// dentro*), o [`crate::sss::SssParams`] é quem sabe semeá-las pelo tamanho
    /// da peça, e ele é a porta única de empacotamento.
    pub sss: crate::sss::SssParams,
    /// A malha desenhada por cima da forma.
    ///
    /// ⚠️ Ele viaja aqui e **não entra no [`ShadeRaw`]**: é um segundo PASSE, não
    /// um termo do sombreamento. Uma flag no uniform o faria parecer uma opção do
    /// fragment, e o dia em que alguém a lesse lá dentro o wireframe passaria a
    /// pintar as FACES.
    pub wireframe: bool,
}

impl Default for Shade {
    fn default() -> Self {
        Self {
            cavity: DEFAULT_CAVITY,
            ao: DEFAULT_AO_STRENGTH,
            ssao: DEFAULT_SSAO_STRENGTH,
            sss: crate::sss::SssParams::default(),
            matcap: None,
            wireframe: false,
        }
    }
}

/// As opções, como o fragment shader as lê.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct ShadeRaw {
    /// Quanto da cavidade entra. `0` = o barro liso da W3, **ao byte**.
    pub cavity: f32,
    /// **Qual matcap** — `0` é o rig do artista, `n` é o material `n − 1`.
    ///
    /// ⚠️ O sentinela existe SÓ aqui, na fronteira do device: um uniform não tem
    /// `Option`, e a alternativa (um segundo `u32` dizendo *"tem matcap?"*) seria
    /// dois campos que precisam concordar.
    pub matcap: u32,
    /// Quanto do AO assado entra. `0` = byte-idêntico ao barro sem o canal.
    ///
    /// ⚠️ **Este é um dos dois `f32` que o `_pad` reservava** dizendo *"é aqui
    /// que SSS e AO vão pousar sem mexer no layout"*. A promessa foi cobrada e o
    /// layout não mudou — sobra um, e ele continua nomeando o SSS.
    pub ao: f32,
    /// Quanto do AO de TELA entra.
    ///
    /// ⚠️ **Este era o slot que o `_pad` guardava para o SSS.** A promessa foi
    /// cobrada por outro inquilino, e quando o SSS de fato chegou o `size_of`
    /// cresceu para 32 por alinhamento — exatamente o que este comentário
    /// anunciava, e o gate `the_uniform_grew_the_way_it_said_it_would` é quem
    /// cobra que o WGSL tenha crescido junto.
    pub ssao: f32,
    /// **Quanto do espalhamento sub-superficial entra**, `0..1`. `0` = o barro de
    /// sempre, **ao byte** — a tabela nem é consultada.
    pub sss_strength: f32,
    /// O `scatter` já dividido pelo teto da tabela ([`crate::sss::T_MAX`]) — a
    /// coordenada `v` sai de `|κ| ×` isto. A divisão mora no
    /// [`crate::sss::SssRaw::pack`], e só lá.
    pub sss_scale: f32,
    /// **O coeficiente da TRANSMITÂNCIA**: `1 / scatter`, para o shader escrever
    /// `exp(-espessura × isto × k_canal)`.
    ///
    /// ⚠️ **Ele ocupa um dos dois `_pad` que o comentário acima reservava** — e a
    /// promessa era exatamente esta: *"é aqui que o SSS pousa sem mexer no
    /// layout"*. O `size_of` fica em 32 B.
    ///
    /// ⚠️ **`1/scatter` e não `scatter`**, e a divisão mora numa porta só (o
    /// [`crate::sss::SssRaw::pack`]): o device multiplicando é mais barato que o
    /// device dividindo por frame, e — o que decide — `scatter = 0` vira um
    /// número **grande e finito** aqui, que é a leitura certa (*a luz não anda
    /// nada dentro deste material* ⇒ opaco), em vez de um `inf` que o
    /// interpolador transformaria em `NaN`.
    pub trans_scale: f32,
    /// Padding explícito até 32 B. ⚠️ Ele existe para o `size_of` do Rust e o
    /// tamanho que o WGSL calcula concordarem **sem depender do que o compilador
    /// resolve fazer** — um uniform em que os dois discordam lê campos deslocados,
    /// e o sintoma é um sombreamento levemente errado que ninguém consegue nomear.
    pub _pad: [f32; 1],
}

impl Default for ShadeRaw {
    fn default() -> Self {
        Self::pack(Shade::default())
    }
}

impl ShadeRaw {
    /// Bytes do uniform. Constante, para o buffer nascer com o tamanho certo.
    pub const SIZE: usize = std::mem::size_of::<Self>();

    /// Empacota o que o artista escolheu, **clampado na porta**.
    ///
    /// ⚠️ O clamp mora aqui e não no chamador porque o device não tem opinião: um
    /// `cavity` de 3 faria `1 − 3k` ficar negativo numa fresta funda e o barro
    /// sairia com a cor invertida. Clampar no shader seria a segunda cópia da
    /// mesma regra, e ela divergiria no dia em que o painel chegasse.
    ///
    /// ⚠️ **E o índice do matcap é clampado pelo MESMO motivo:** um `Some(9)` numa
    /// tabela de seis cairia no `default` do `switch` do shader — que é um
    /// material legítimo — e o artista veria a cera vermelha ao pedir algo que
    /// não existe, sem nada dizendo que o pedido era inválido.
    #[must_use]
    pub fn pack(shade: Shade) -> Self {
        let n = u8::try_from(MATCAPS.len()).unwrap_or(u8::MAX);
        Self {
            cavity: shade.cavity.clamp(0.0, 1.0),
            matcap: shade.matcap.map_or(0, |i| u32::from(i.min(n - 1)) + 1),
            // Clampado pela mesma razão da cavidade: o device não tem opinião, e
            // um `ao` de 3 faria o `mix` extrapolar para além do canal.
            ao: shade.ao.clamp(0.0, 1.0),
            ssao: shade.ssao.clamp(0.0, 1.0),
            // ⚠️ Pelo `SssRaw`, nunca à mão: é ele que sabe que o `scatter` viaja
            // dividido pelo teto da tabela, e uma segunda cópia dessa divisão
            // divergiria no dia em que o teto mudasse.
            sss_strength: crate::sss::SssRaw::pack(shade.sss).params[0],
            sss_scale: crate::sss::SssRaw::pack(shade.sss).params[1],
            trans_scale: crate::sss::SssRaw::pack(shade.sss).params[2],
            _pad: [0.0; 1],
        }
    }
}

#[cfg(test)]
#[path = "shade_tests.rs"]
mod tests;
