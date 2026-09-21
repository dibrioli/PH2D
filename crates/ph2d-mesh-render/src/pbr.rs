//! ⭐⭐⭐ **O BARRO ACESO PELA LEI QUE ASSA** — o modo `Pbr` do visor.
//!
//! # Porque este módulo existe, com a frase que o pediu
//!
//! O cabeçalho do `mesh.wgsl` declara desde a W3, por escrito: *«O QUE NÃO É COMPARTILHADO É O
//! MATERIAL. A tinta tem rugosidade, metal e cera por-pixel com LUT baked; o barro tem uma cor e um
//! expoente. O material da malha é a wave do shader (docs/3D/05.1, W7).»* E a tabela do
//! [`crate::lighting`] escreve a mesma linha: *«rugosidade / metal / cera por-pixel · sim · **não —
//! W7***».
//!
//! ⇒ o barro vivo era um modelo de ARGILA — `CLAY`, `CLAY_EXPONENT` e `CLAY_SHINE` são três números
//! cravados no shader — enquanto o sprite assado passou a acender com o OpenPBR inteiro. *Duas leis
//! sobre o mesmo objecto respondem diferente por construção*, e o report do dono foi exactamente
//! isso: **«o que se vê no objecto 3D não é o que se vê na sprite cozida»**.
//!
//! # ⚠️ O que este modo partilha, e o que ele NÃO pode partilhar
//!
//! | | o sprite assado | o barro no visor |
//! |---|---|---|
//! | a **LEI** (`mx_direct` · `mx_indirect`) | `ph2d-material` | **a mesma fonte**, composta aqui |
//! | as **LÂMPADAS** | `ph2d_light::resolve` | **as mesmas**, já no uniform do rig |
//! | o **CÉU** | [`ph2d_light::env_ramp`] | **a mesma porta** |
//! | o **MATERIAL** | um `Surface` | **o mesmo**, empacotado por [`PbrRaw`] |
//! | a **VISTA** | `(0, 0, 1)` — o canvas é uma projecção 2D | **por pixel** — a câmera é PERSPECTIVA |
//!
//! ⛔⛔ **A última linha é uma divergência que NÃO se cura, e declará-la é o que torna as outras
//! quatro úteis:** o canvas projecta ortograficamente e o visor tem `fov_y = 45°`. A mesma lei com
//! vistas diferentes é uma imagem diferente — e isso está **certo**. O que o modo entrega é *o
//! material e a luz que você vai assar*, não um pixel igual.
//!
//! ⚠️ **E o LAÇO é nosso, a LEI não é.** O laço do sprite ([`ph2d_form_pbr::wgsl`]) assume a vista
//! constante e não serve aqui; este chama as mesmas `mx_*` com o `v` do pixel. *Um laço é onde as
//! amostras entram; a lei é o que responde* — e há gate a afirmar que nenhuma linha de óptica é
//! escrita neste ficheiro.

use bytemuck::{Pod, Zeroable};
use ph2d_material::Surface;
use ph2d_material::wgsl::{ENV_SLOT, EnvLobe, PACKED, SOURCE as LEI, pack};

/// ⭐⭐ **O CÉU DA CASA, em WGSL — as duas funções que a ranhura do ambiente pede.**
///
/// ⚠️ **Ele é o gémeo EXACTO do que a `ph2d-form-donation` monta para o passe do sprite**, e as duas
/// cópias existem porque a ranhura é preenchida DENTRO da fonte da lei, que vem antes de qualquer
/// `struct` do consumidor — ⛔ o `var<private>` não é estilo: aqui o `pbr` ainda não foi declarado.
/// *Quem as escreve é o `fs_main`, no topo.*
///
/// ⚠️ **`fma` EXPLÍCITO nos dois** — o gémeo em Rust usa `mul_add`, que tem UM arredondamento;
/// escrito solto, a igualdade ao bit passaria a depender de o compilador do WGSL contrair a
/// multiplicação-soma, e esta casa já mediu uma placa a contraí-la onde o fonte não o pedia.
///
/// ⚠️ **`1.5` é `1/(2/3)`**: o `ceu_inclinacao` é a inclinação da IRRADIÂNCIA, que já traz o `Â₁` do
/// lóbulo cosseno; a radiância quer a rampa crua. ⛔ O NOME da função que calcula o encolhimento não
/// se escreve neste texto — há gate a varrê-lo, e uma agulha num comentário lê-se como uma chamada.
const CEU_NA_RANHURA: &str = r#"
var<private> ceu_base: vec3<f32>;
var<private> ceu_inclinacao: vec3<f32>;

fn env_irradiance(n: vec3<f32>) -> vec3<f32> {
    return fma(ceu_inclinacao, vec3<f32>(-n.y), ceu_base);
}

fn env_radiance(dir: vec3<f32>, alpha: f32, shrink: f32) -> vec3<f32> {
    let up = shrink * -dir.y;
    return fma(1.5 * ceu_inclinacao, vec3<f32>(up), ceu_base);
}
"#;

/// ⭐⭐⭐ **COMO O VISOR ACHA O TEXEL DA SPRITE** que corresponde a um fragmento seu.
///
/// O bake rasteriza a malha com a MESMA câmera, no tamanho da sprite. Logo o texel `(i,j)` dela é
/// o ponto da malha que aquela rasterização vê ali, e o visor pode reconstruir o `uv`:
///
/// - o `fov_y` é preservado nas duas ⇒ **o `y` normalizado é o MESMO**;
/// - o `x` normalizado escala pela razão dos aspectos, `(w/h da vista) / (w/h da fonte)`.
///
/// ⚠️ **Os inversos da ÁREA e não do alvo:** o `@builtin(position)` chega em coordenadas do ALVO e
/// o `set_viewport` move o rasterizador sem mover a aritmética do shader — a origem viaja no
/// `cam.viewport.zw` exactamente por isso.
#[derive(Clone, Copy, Debug)]
pub struct Projeccao {
    /// `(w/h da vista) / (w/h da fonte)`.
    pub razao_dos_aspectos: f32,
    /// `1 / largura` da área, em pixels.
    pub inv_w: f32,
    /// `1 / altura` da área, em pixels.
    pub inv_h: f32,
}

/// ⭐ **O material e o céu, como o `mesh.wgsl` os lê** — o quarto uniform do grupo 0.
///
/// ⚠️ **Um uniform NOVO e não campos no [`crate::shade::ShadeRaw`]:** aquele tem `32 B` exactos e o
/// doc do último campo dele diz que o derradeiro `_pad` foi gasto. E a frequência é a mesma (um
/// write por quadro), logo o corte é por ASSUNTO: *com que luz e quanto de cavidade* é a vista; *de
/// que matéria* é o objecto.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct PbrRaw {
    /// O material pronto, na arrumação do [`ph2d_material::wgsl::pack`].
    pub mat: [f32; PACKED],
    /// A irradiância na horizontal. `rgb`, com o `a` a padding.
    pub ceu_base: [f32; 4],
    /// Quanto ela sobe para o topo da TELA.
    pub ceu_inclinacao: [f32; 4],
    /// ⭐⭐⭐ **O OLHAR e a PROJECÇÃO DA FONTE DO ALBEDO** — `x` = os stops de exposição;
    /// `y` = a razão dos aspectos (`vista / fonte`); `z`, `w` = o inverso da largura e da altura da
    /// ÁREA em pixels.
    ///
    /// ⚠️⚠️ **`y`, `z` e `w` eram RESERVA DECLARADA e foram GASTOS em 2026-09-21**, pelo campo que
    /// a reserva existia para acolher sem surpresa. *Uma posição sem dono e sem régua é onde o
    /// próximo número entra calado* — e este entrou com nome, com doc e com gate.
    ///
    /// ⛔ **A reserva acabou:** o campo seguinte pede uma entrada NOVA, nunca um destes slots.
    pub olhar: [f32; 4],
    /// O código da [`ph2d_view_transform::ViewTransform`] em `x`; `y` = `1` quando há fonte de
    /// albedo posta. `zw` reserva declarada.
    pub vista: [u32; 4],
}

impl Default for PbrRaw {
    fn default() -> Self {
        Self {
            mat: [0.0; PACKED],
            ceu_base: [0.0; 4],
            ceu_inclinacao: [0.0; 4],
            olhar: [0.0; 4],
            vista: [0; 4],
        }
    }
}

impl PbrRaw {
    /// Bytes do uniform, para o buffer nascer com o tamanho certo.
    pub const SIZE: usize = core::mem::size_of::<Self>();

    /// Empacota o material e o céu que o rig produz.
    ///
    /// ⚠️ **O encolhimento do lóbulo viaja PRONTO** ([`EnvLobe::of`]): ele é constante por MATERIAL,
    /// e correr no dispositivo o logaritmo que o produz seria a mesma conta a dar o mesmo número um
    /// milhão de vezes.
    ///
    /// ⭐ **O céu entra pela porta da [`ph2d_light::env_ramp`]**, que é a MESMA que o sprite assado
    /// atravessa — é ela que faz as duas leis dobrarem o ambiente do mesmo jeito.
    #[must_use]
    pub fn pack(
        surface: &Surface,
        plano: [f32; 3],
        olhar: ph2d_view_transform::Look,
        albedo: Option<Projeccao>,
    ) -> Self {
        let (base, inclinacao) = ph2d_light::env_ramp(plano);
        let (k, inv_w, inv_h, tem) = albedo.map_or((0.0, 0.0, 0.0, 0), |p| {
            (p.razao_dos_aspectos, p.inv_w, p.inv_h, 1)
        });
        Self {
            mat: pack(surface, EnvLobe::of(surface)),
            ceu_base: [base[0], base[1], base[2], 0.0],
            ceu_inclinacao: [inclinacao[0], inclinacao[1], inclinacao[2], 0.0],
            olhar: [olhar.exposure_stops, k, inv_w, inv_h],
            vista: [ph2d_view_transform::wgsl::view_code(olhar.view), tem, 0, 0],
        }
    }
}

/// ⭐⭐⭐ **A FONTE DO PASSE DA MALHA, composta** — a lei do OpenPBR mais o `mesh.wgsl`.
///
/// ⚠️ **Uma função e não uma `const`**, porque a fonte da lei traz a ranhura do ambiente por
/// preencher e a concatenação crua **não parsa** — foi assim que a `ph2d-form-donation` a montou, e
/// é o mesmo idioma.
///
/// ⚠️ **A LEI vem PRIMEIRO:** o `mesh.wgsl` chama as `mx_*` e um WGSL não tem declaração antecipada
/// de função.
///
/// ⭐⭐ **E o OLHAR entra como TERCEIRA peça, pela mesma razão que a lei entra:** ele é o último acto
/// da lei que assa ([`ph2d_form_pbr::acende_texel`] recebe-o como argumento), e deixá-lo de fora
/// entregava a radiância CRUA onde o sprite entrega a exposta — medido, `0,128` contra `0,539`.
/// ⛔ Ele é a fonte da [`ph2d_view_transform`] tal e qual, e **não** uma segunda redacção da curva:
/// duas cópias de um tonemap divergem no primeiro dia em que alguém afinar uma.
#[must_use]
pub fn fonte() -> String {
    format!(
        "{}\n{}\n{}",
        LEI.replace(ENV_SLOT, CEU_NA_RANHURA),
        ph2d_view_transform::wgsl::SOURCE,
        crate::fonte::MESH_WGSL
    )
}

#[cfg(test)]
#[path = "pbr_tests.rs"]
mod pbr_tests;
