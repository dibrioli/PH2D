//! **A TABELA DECLARATIVA DOS MODOS** — o que cada referência DIZ sobre cada
//! verbo (curva, força, raio, `accumulate`, a curva do slider), e só isso.
//!
//! ⚠️ **O corte é o do próprio cabeçalho do [`super::ref_mode`], não o do
//! tamanho:** ele nomeia as duas metades de um modo — a **declarativa** (esta
//! tabela, `const`) e a **imperativa** (onde o modo escolhe uma FUNÇÃO
//! diferente: o [`super::ref_mode::KernelLaw`], o
//! [`super::ref_mode::LateralPull`], o campo elástico). Este arquivo é a
//! primeira; o irmão ficou com a segunda.
//!
//! ⚠️ **Cada `Option` é uma afirmação sobre a FONTE**, nunca um valor esquecido,
//! e é isso que impede o pior erro possível aqui: **inventar** um número e
//! shipá-lo com a autoridade de uma referência que não o declara.

use crate::brush::Verb;
use crate::falloff::Falloff;
use crate::ref_mode::{RefMode, StrengthCurve};

/// **O que uma referência DECLARA sobre um verbo.**
///
/// ⚠️ **A struct CRESCE wave a wave, e um campo só entra com o consumidor
/// dele.** Os campos da metade imperativa (`front_face`, `hardness`,
/// `normal_radius_factor`, `strength_curve`, `plane_side`, …) chegam junto com
/// os kernels que os leem — um campo que ninguém lê é estado morto, e este repo
/// varre isso a cada wave.
///
/// ⚠️ **Cada `Option` é uma afirmação sobre a FONTE**, nunca um valor esquecido:
/// `None` quer dizer *a referência é silenciosa aqui*, e quem arma deixa o
/// número do artista onde está.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VerbProfile {
    /// A curva do pincel — D1/E1, o maior número do estudo.
    ///
    /// ⚠️ **`None` é uma AFIRMAÇÃO, e ela ficou mais forte quando o `brush.cc`
    /// foi lido** (§7.0 do plano). As nove fórmulas do `BRUSH_CURVE_*` são
    /// legíveis e **já estão todas em [`Falloff`]** — mas elas são o que o
    /// artista pode ESCOLHER, não o que um pincel VESTE: o `curve_preset` de um
    /// `Brush` zero-inicializado é `BRUSH_CURVE_CUSTOM = 0`, e o
    /// `brush_init_data` semeia a *curvemapping* dele com `CURVE_PRESET_SMOOTH`
    /// — uma bézier editável, **nenhuma das nove**.
    ///
    /// ⇒ Declarar aqui *"o Blender usa a Smooth"* seria inventar um número e
    /// vesti-lo com o nome de outro produto. E a tabela por-TOOL (a força e o
    /// raio de fábrica do Clay Strips) não é lida de fonte nenhuma: desde o 4.3
    /// ela vive dentro de um `.blend` binário de assets.
    pub falloff: Option<Falloff>,
    /// A força de fábrica — D3/E2. `None` = a fonte não declara.
    pub strength: Option<f32>,
    /// O raio de fábrica como **fração do raio-base da fonte** — D4/E3.
    ///
    /// ⚠️ Fração e não pixels: o raio-base do SculptGL é `50` e o nosso
    /// `radius_px` também nasce em `50`, mas guardar `25` aqui congelaria a
    /// escolha do artista no dia em que ele mudasse o raio-base. Uma fração
    /// continua verdadeira em qualquer raio — é a mesma lei do
    /// `sss_scatter` (fração do maior lado) e do `Falloff` (distância
    /// normalizada).
    pub radius_factor: Option<f32>,
    /// O Accumulate de fábrica. `None` = a fonte não declara o campo.
    pub accumulate: Option<bool>,
    /// **Como o slider de força vira peso** — o E13.
    ///
    /// ⚠️ Não é `Option`: toda referência responde a esta pergunta, porque toda
    /// referência transforma o slider de ALGUMA maneira — e *"linearmente"* é
    /// uma resposta, não uma omissão.
    pub strength_curve: StrengthCurve,
}

impl VerbProfile {
    /// O perfil que **não afirma nada** — a base sobre a qual as entradas da
    /// tabela são escritas, para uma linha nova nascer explícita no que declara.
    const SILENT: Self = Self {
        falloff: None,
        strength: None,
        radius_factor: None,
        accumulate: None,
        strength_curve: StrengthCurve::Linear,
    };
}

/// O raio de fábrica do SculptGL, em pixels — `SculptBase`/`Brush.js:11`. É o
/// denominador de todo [`VerbProfile::radius_factor`] da coluna `S`.
const S_BASE_RADIUS_PX: f32 = 50.0;

/// **A coluna `S`** — lida das fontes do SculptGL, arquivo e linha ao lado.
///
/// ⚠️ **Todo número aqui foi LIDO, nenhum foi afinado.** As tools de geometria
/// do original compartilham a quártica `3t⁴ − 4t³ + 1` (a nossa
/// [`Falloff::Plateau`], que sai da porta única [`crate::ref_kernels::falloff`]
/// em `f64`), e o que difere entre elas é força, raio e `accumulate`.
const fn profile_s(verb: Verb) -> Option<VerbProfile> {
    // Um `radius_factor` é `_radius / 50`.
    let p = match verb {
        // `Brush.js:11-16` — `_radius 50 · _intensity 0.5 · _clay true ·
        // _accumulate true`. A tool `Brush` do original é a nossa **Draw E
        // Clay** (o `_clay` é um checkbox dela, ligado de fábrica).
        //
        // ⚠️ **A força do Draw é a ÚNICA que já batia com a nossa** (0,5). As
        // divergências do D3 são as outras: 0,75 em quatro tools e 0,30 no
        // Inflate.
        Verb::Draw | Verb::Clay => VerbProfile {
            falloff: Some(Falloff::Plateau),
            strength: Some(0.5),
            radius_factor: Some(1.0),
            accumulate: Some(true),
            ..VerbProfile::SILENT
        },
        // `Inflate.js:9-11` — o mais fraco do catálogo, e por um motivo: inflar
        // é a operação que estoura mais rápido.
        Verb::Inflate => VerbProfile {
            falloff: Some(Falloff::Plateau),
            strength: Some(0.3),
            radius_factor: Some(1.0),
            accumulate: Some(false),
            ..VerbProfile::SILENT
        },
        // `Smooth.js:10-13` — e ⚠️ o `_tangent = false` dele é o
        // `smoothTangent`, **código vivo que nenhuma UI do original alcança**;
        // é o E7 do estudo e a wave W4 daqui.
        Verb::Smooth => VerbProfile {
            falloff: Some(Falloff::Plateau),
            strength: Some(0.75),
            radius_factor: Some(1.0),
            accumulate: Some(false),
            ..VerbProfile::SILENT
        },
        // `Flatten.js:9-11` — ⚠️ e a família do plano é mais forte que um
        // default: o `Flatten.js` **não declara `_accumulate`** e o kernel dele
        // pergunta `this._accumulate === false`, que em `undefined` é FALSO ⇒
        // ele lê o vivo **sempre**, sem checkbox. Nós temos o interruptor, então
        // o honesto é nascer armado.
        Verb::Flatten | Verb::Fill | Verb::Scrape => VerbProfile {
            falloff: Some(Falloff::Plateau),
            strength: Some(0.75),
            radius_factor: Some(1.0),
            accumulate: Some(true),
            ..VerbProfile::SILENT
        },
        // `Pinch.js:9-11`.
        Verb::Pinch | Verb::Magnify => VerbProfile {
            falloff: Some(Falloff::Plateau),
            strength: Some(0.75),
            radius_factor: Some(1.0),
            accumulate: Some(false),
            ..VerbProfile::SILENT
        },
        // `Crease.js:9-11` — ⚠️ **raio 25, metade do resto**: um vinco é fino
        // por definição, e é a divergência D4 mais visível do catálogo.
        Verb::Crease => VerbProfile {
            falloff: Some(Falloff::Plateau),
            strength: Some(0.75),
            radius_factor: Some(25.0 / S_BASE_RADIUS_PX),
            accumulate: Some(false),
            ..VerbProfile::SILENT
        },
        // `Masking.js:13-16` — força CHEIA, e o nosso default já concorda.
        Verb::Mask => VerbProfile {
            falloff: Some(Falloff::Plateau),
            strength: Some(1.0),
            radius_factor: Some(1.0),
            accumulate: Some(false),
            ..VerbProfile::SILENT
        },
        // `Move.js:10-11` — ⚠️ **raio 150, TRÊS vezes o resto**: puxar é um
        // gesto de região, e um Move com raio de pincel de detalhe é o que faz
        // um artista concluir que a ferramenta não funciona.
        Verb::Move => VerbProfile {
            falloff: Some(Falloff::Plateau),
            strength: Some(1.0),
            radius_factor: Some(150.0 / S_BASE_RADIUS_PX),
            ..VerbProfile::SILENT
        },
        // `Drag.js:10` — mesmo raio do Move e ⚠️ **sem `_intensity` declarada**:
        // o `None` é a fonte sendo silenciosa, não um número esquecido.
        Verb::SnakeHook => VerbProfile {
            falloff: Some(Falloff::Plateau),
            radius_factor: Some(150.0 / S_BASE_RADIUS_PX),
            ..VerbProfile::SILENT
        },
        // `Twist.js:10` — raio 75, e também sem força declarada.
        Verb::Twist => VerbProfile {
            falloff: Some(Falloff::Plateau),
            radius_factor: Some(75.0 / S_BASE_RADIUS_PX),
            ..VerbProfile::SILENT
        },
        // `LocalScale.js:8` — raio-base, sem força.
        Verb::LocalScale => VerbProfile {
            falloff: Some(Falloff::Plateau),
            radius_factor: Some(1.0),
            ..VerbProfile::SILENT
        },
        // ⚠️ **O SculptGL NÃO TEM Sharpen**, e este `None` é essa frase.
        // (O Blender tem o equivalente, `Enhance Details` — doc 20 §9 item 12.)
        //
        // ⚠️ **Nem Clay Strips** — a faixa é do Blender (`clay_strips.cc`), e o
        // `None` aqui é a MESMA frase. Quem arma um verbo que a fonte não
        // declara cai no [`VerbProfile::SILENT`], que é o nosso default; o chip
        // `S` segue oferecido porque a LEI DE KERNEL dele (lateral direta,
        // plano de um lado, front-face ignorado) é universal naquele motor —
        // é a distinção que o [`RefMode::declares`] documenta.
        // ⚠️ **Nem o Blob** — o `crease.cc` é do Blender, e este `None` é a
        // mesma frase que os dois vizinhos carregam.
        // ⚠️ **Nem o Clay Thumb** — `clay_thumb.cc`, a mesma frase pela terceira
        // vez.
        // ⚠️ **Nem o Multiplane Scrape** — `multiplane_scrape.cc`, a quarta.
        // ⚠️ **Nem o Slide Relax** — `relax.cc`, a quinta. E aqui o `None` custa
        // mais que nos outros: o SculptGL não tem verbo NENHUM que redistribua a
        // malha sem mexer na forma, então não há sequer um parente de quem herdar
        // força ou curva — os quatro defaults deste verbo são NOSSOS, e o
        // `unwrap_or` do [`Verb::default_strength`] é literalmente *"a referência
        // não respondeu"*.
        Verb::Sharpen
        | Verb::ClayStrips
        | Verb::Blob
        | Verb::ClayThumb
        | Verb::MultiplaneScrape
        | Verb::SlideRelax
        // ⚠️ **Nem o Surface Smooth** — `surface_smooth.cc`, a sexta. E aqui o
        // `None` cobra o mesmo que no vizinho: o SculptGL tem o laplaciano cru e
        // nada que devolva o volume, então força e curva de fábrica são NOSSAS.
        | Verb::SurfaceSmooth
        | Verb::Layer => return None,
    };
    Some(p)
}

impl Verb {
    /// **O que a referência `mode` declara sobre este verbo** — ou `None` se ela
    /// não tem resposta para ele.
    ///
    /// ⚠️ É a porta única da metade declarativa: o painel pergunta para saber
    /// **que chips oferecer** e **que valor pintar**, o arming pergunta para
    /// saber **o que escrever**, e o gate pergunta para conferir contra a fonte.
    /// Uma segunda tabela em qualquer um dos três derivaria em silêncio.
    #[must_use]
    pub const fn profile(self, mode: RefMode) -> Option<VerbProfile> {
        match mode {
            RefMode::S => profile_s(self),
            RefMode::B => profile_b(self),
            // ⚠️ Chega nas waves W4/W5/W7, um paper por vez. Enquanto for `None`
            // **nenhum chip é oferecido**, que é a lei anti-chip-morto valendo
            // por construção em vez de por disciplina.
            RefMode::L => None,
        }
    }
}

/// **A coluna `B`** — o que o Blender DECLARA, e só isso.
///
/// ⛔ **Ela não traz DEFAULTS, e a ausência é MEDIDA — o arquivo foi trazido e
/// respondeu que a resposta não está nele** (§7.0 do plano). O
/// `BKE_brush_sculpt_reset`, onde a força, o raio e a curva de fábrica de cada
/// tool viviam, **não existe mais em C** (`git grep` sobre a árvore: zero):
/// desde o Blender 4.3 os pincéis são ASSETS, num `.blend` binário. O que
/// sobrou, o `brush_defaults()` (`brush.cc:597`), copia de um `Brush def = {}`
/// — **um** conjunto para todas as tools, não uma tabela por ferramenta.
///
/// ⇒ *"a força de fábrica do Clay Strips"* não é lida de fonte nenhuma. Isto
/// **não é uma lacuna do nosso clone**: é onde o Blender passou a guardar a
/// resposta, e trazer mais arquivos não muda.
///
/// ✅ **O que ela traz é LIDO literalmente** (`sculpt.cc:2337-2339`): o slider é
/// a RAIZ do peso, com o comentário do próprio Blender ao lado — *"square it to
/// make lower values more sensitive"*. Vale para TODA tool: ele mora no
/// `brush_strength`, que é o funil de todas elas — e é por isso que este `match`
/// não tem braços por verbo.
///
/// ✅ **E a CURVA de fábrica também é lida — não da fonte, do Blender A CORRER.**
/// A leitura estática dizia que o pincel de fábrica veste uma *curvemapping*
/// custom (*"nenhuma das nove"*); o oráculo executável
/// (`docs/3D/ferramentas/blender_sculpt_oracle.py`, Blender 5.2 com GUI —
/// em `--background` o `region.data` é nulo e o sculpt segfaulta) reporta
/// `curve_distance_falloff_preset = SMOOTH` e **deposita a analítica**: a
/// `r/R = 0,258` o vértice move `0,417503` de um pico de `0,5` ⇒ razão
/// **0,835**, contra **0,8348** de `3u² − 2u³` e **0,94** do spline de quatro
/// pontos que a leitura estática previa. ⇒ [`Falloff::Smooth`], que desde
/// 2026-08-16 **é** a smoothstep.
///
/// ⚠️ **O `accumulate` fica `None` aqui, e a assimetria é MEDIDA, não esquecida:**
/// a curva o oráculo respondeu, os defaults por-tool ele não pode responder —
/// eles moram no `.blend` binário (§7.0), e um valor inventado aqui seria pior
/// que a ausência.
const fn profile_b(_verb: Verb) -> Option<VerbProfile> {
    Some(VerbProfile {
        strength_curve: StrengthCurve::Squared,
        falloff: Some(Falloff::Smooth),
        ..VerbProfile::SILENT
    })
}
