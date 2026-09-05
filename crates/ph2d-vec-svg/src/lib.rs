#![forbid(unsafe_code)]
//! ⭐⭐⭐ **UM FICHEIRO SVG ⇄ O DOCUMENTO VECTORIAL** (estudo 42, item 3).
//!
//! Até 2026-09-05 o app **exportava** uma curva e não sabia **ler** nenhuma: o `ph2d-imageio-svg`
//! validava o ficheiro e devolvia `VectorDoc::default()` — *"intentionally empty"* — então nenhum
//! acervo de artista entrava. Esta crate é o outro sentido, e passa a ser a dona dos DOIS.
//!
//! # ⛔⛔ A LEI DOS EIXOS, e por que ela mora aqui
//!
//! > **O SVG mede o Y para BAIXO; o mundo do PH2D mede-o para CIMA.**
//!
//! Não é uma convenção escolhida aqui: é o que a câmara do app faz
//! (`ph2d_render::Camera2d::world_to_screen_affine` é `translate ∘ scale(k, **−k**) ∘ translate`, e
//! o doc dela guarda o report que a fixou — *"mouse e grid descem enquanto sprites sobem"*). O
//! assador de tiles do Motion escreve a mesma lei por outras palavras: *"sem o `-BAKE_DPI` a
//! estrela assada aponta para BAIXO"*.
//!
//! ⚠️⚠️ **E foi por não haver porta que o exportador nasceu espelhado.** O `vec_svg_export` de
//! 02/09 escrevia as coordenadas de mundo **cruas** dentro de um `<svg>`, com o cabeçalho a
//! afirmar *"em coordenadas de MUNDO (Y para baixo, como o SVG)"* — as duas metades da frase
//! contradizem-se, e o ficheiro saía **verticalmente espelhado**. Ninguém o viu porque o consumidor
//! era uma LLM a ler números, e nenhum gate media orientação.
//!
//! ⇒ [`svg_to_world`] e [`world_to_svg`] são a porta, uma é a inversa da outra, e há gate a
//! prová-lo. *Uma lei escrita em dois sítios ainda não é uma lei — só uma PORTA é.*
//!
//! # A escala: **um px é um px**
//!
//! Um `.svg` de 512 unidades entra com o MESMO tamanho de mundo que um `.png` de 512 px, porque
//! passa pelo mesmo divisor (`pixels_per_meter` do projecto — *"sprite world size = source pixels /
//! pixels_per_meter"*). ⛔ Sem ele um ícone de 1024 nasceria com 1024 unidades de largura, cem
//! vezes fora do ecrã, e o artista concluiria que o import se partiu.
//!
//! # O que atravessa, e o que é NOMEADO
//!
//! Geometria (com os `<g transform>` já resolvidos), preenchimento sólido e gradiente linear /
//! radial, regra de preenchimento, traço (largura, ponta, junção, tracejado), **opacidade de
//! objecto** e **modo de mistura** — os dois últimos existem no documento desde a v19 do schema, e
//! são exactamente o que o `opacity` e o `mix-blend-mode` do SVG dizem.
//!
//! O que o ficheiro carrega e o documento não exprime **sai numa nota que nomeia a camada**
//! ([`Drawing::notes`]) — a lei do importador `.ase`: *um importador que ignora em silêncio é pior
//! do que um que recusa*.

use ph2d_vec_scene::{VecPath, Xform};

mod import;
mod paint;

pub use import::{Options, import, parse};

#[cfg(test)]
mod tests;

/// **O tecto de bytes de um `.svg`.**
///
/// Um SVG é texto, então o tecto defende o parser de expansão hostil (billion-laughs, entidades
/// externas) na FRONTEIRA DE LEITURA, antes de o usvg tocar nos bytes. 16 MiB cobre qualquer
/// acervo de ícones plausível.
///
/// ⚠️ **É o mesmo número que o `ph2d_imageio::MAX_ARCHIVE_TEXT_BYTES`, e há gate a prová-lo** — ele
/// vive no `ph2d-imageio-svg`, que é a única crate que vê os dois. *Duas constantes para a mesma
/// lei divergem no primeiro dia em que alguém mexe numa delas.*
pub const MAX_SVG_BYTES: u64 = 16 * 1024 * 1024;

/// O que correu mal a ler um ficheiro.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Passou do [`MAX_SVG_BYTES`] — recusado ANTES do parse.
    TooLarge(u64),
    /// O usvg não o aceitou (XML malformado, sem `<svg>`, gzip sem a feature…).
    Parse(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooLarge(n) => write!(f, "SVG source {n} bytes > {MAX_SVG_BYTES} (DoS defence)"),
            Self::Parse(m) => write!(f, "SVG parse: {m}"),
        }
    }
}

impl std::error::Error for Error {}

/// ⭐⭐⭐ **SVG → MUNDO.** A porta única da lei dos eixos, no sentido do import.
///
/// `pixels_per_meter` é o mesmo número que dimensiona uma sprite importada (**um px é um px**); o
/// sinal do Y é a lei, e não um parâmetro.
///
/// ⚠️ Componha-a com o `abs_transform` do nó — nunca a aplique a coordenadas que ainda não passaram
/// por ele (no usvg 0.48 o `Path::data()` está em espaço LOCAL, apesar do doc dele dizer
/// *"absolute coordinates"*, que ali quer dizer *comandos* absolutos e não *espaço* absoluto).
#[must_use]
pub fn svg_to_world(pixels_per_meter: f64) -> Xform {
    let k = if pixels_per_meter.is_finite() && pixels_per_meter.abs() > f64::EPSILON {
        1.0 / pixels_per_meter
    } else {
        1.0
    };
    Xform([k, 0.0, 0.0, -k, 0.0, 0.0])
}

/// ⭐⭐⭐ **MUNDO → SVG.** A inversa exacta da [`svg_to_world`], e a porta do exportador.
///
/// ⚠️ O exportador chama-a com [`EXPORT_PIXELS_PER_UNIT`] porque escreve **unidades de mundo** no
/// ficheiro — a escolha dele, documentada; o que ele NÃO pode escolher é o sinal do Y.
#[must_use]
pub fn world_to_svg(pixels_per_meter: f64) -> Xform {
    let k = if pixels_per_meter.is_finite() && pixels_per_meter.abs() > f64::EPSILON {
        pixels_per_meter
    } else {
        1.0
    };
    Xform([k, 0.0, 0.0, -k, 0.0, 0.0])
}

/// ⭐⭐⭐ **COMO UM MODO DE MISTURA SE ESCREVE EM SVG** — a porta única do nome, nos dois sentidos.
///
/// O `mix-blend-mode` do CSS tem exactamente os 16 modos do W3C, e são os mesmos 16 que o
/// [`crate::import`] traduz de volta. ⛔ **`None` para os que o CSS não tem** (o `Add`, o `Behind`,
/// o `Clear` e os três do Photoshop): escrever um nome que o leitor não conhece fá-lo cair em
/// `normal` **em silêncio**, e o ficheiro passaria a mentir sobre o que o desenho faz.
///
/// ⚠️ *Um vocabulário com dois donos escrito duas vezes diverge no primeiro modo novo* — e este já
/// tem dois: o importador (que lê) e o exportador da shell (que escreve).
#[must_use]
pub fn css_blend_name(m: ph2d_blend_mode::BlendMode) -> Option<&'static str> {
    use ph2d_blend_mode::BlendMode as B;
    Some(match m {
        B::Normal => return None, // o neutro não se escreve
        B::Multiply => "multiply",
        B::Screen => "screen",
        B::Overlay => "overlay",
        B::Darken => "darken",
        B::Lighten => "lighten",
        B::ColorDodge => "color-dodge",
        B::ColorBurn => "color-burn",
        B::HardLight => "hard-light",
        B::SoftLight => "soft-light",
        B::Difference => "difference",
        B::Exclusion => "exclusion",
        B::Hue => "hue",
        B::Saturation => "saturation",
        B::Color => "color",
        B::Luminosity => "luminosity",
        // ⛔ Sem equivalente em CSS. Quem exporta NOMEIA a perda; escrever `normal` calado seria o
        // ficheiro a afirmar uma composição que o documento não faz.
        B::Add | B::Behind | B::Clear | B::LinearBurn | B::VividLight | B::LinearLight => {
            return None;
        }
    })
}

/// **O exportador escreve UNIDADES DE MUNDO**: uma unidade do documento vira uma unidade do
/// ficheiro. É decisão dele (o consumidor lê números e compara-os com a régua do editor), e é o
/// único grau de liberdade que ele tem sobre a lei — o Y desce em qualquer escala.
pub const EXPORT_PIXELS_PER_UNIT: f64 = 1.0;

/// Um GRUPO do ficheiro, para a Hierarquia do editor o reproduzir.
///
/// ⚠️ **Um grupo não é um tipo de nó neste app** — é *"uma entidade comum com filhos"*
/// (`vec_entities`), então o importador não precisa de inventar nada: entrega a árvore, e a shell
/// põe cada forma debaixo do pai que este índice nomeia.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SvgGroup {
    /// O `id` do `<g>`, ou `Group` quando o ficheiro não lho deu.
    pub name: String,
    /// Índice do grupo pai em [`Drawing::groups`]; `None` = raiz.
    pub parent: Option<usize>,
}

/// Uma FORMA importada: a geometria já em mundo, mais onde ela vive na árvore.
#[derive(Debug, Clone, PartialEq)]
pub struct Shape {
    /// Já em coordenadas de MUNDO — a lei dos eixos e a escala já foram assadas
    /// ([`ph2d_vec_scene::bake_xform`], que leva geometria, gradiente, raio de quina **e** largura
    /// de traço pela mesma porta).
    pub path: VecPath,
    /// O `id` do elemento, ou um nome derivado. A shell passa-o pela porta do nome ÚNICO.
    pub name: String,
    /// Índice do grupo em [`Drawing::groups`]; `None` = filho da raiz.
    pub group: Option<usize>,
}

/// O DESENHO inteiro, pronto a entrar no documento.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Drawing {
    pub shapes: Vec<Shape>,
    pub groups: Vec<SvgGroup>,
    /// ⛔ **O que o ficheiro carrega e o documento não exprime**, uma linha por espécie, com a
    /// contagem. Vazio = nada se perdeu.
    pub notes: Vec<String>,
    /// O tamanho do `viewBox` já em unidades de mundo — é o que a shell usa para colocar o desenho
    /// e para o próximo import não cair por cima deste.
    pub size: [f64; 2],
}
