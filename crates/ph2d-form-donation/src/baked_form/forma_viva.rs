//! ⭐⭐⭐ **A FORMA VIVA — a rota B do catavento, do lado da LUZ.**
//!
//! A [`super::light`] acende um objecto **ASSADO**: a forma dele são `Vec<f32>` que viajam no
//! documento e sobem à placa a cada acendida. Esta acende um objecto **VIVO**: a forma dele são
//! duas vistas de textura que **já estão na placa**, tipicamente escritas por uma rasterização
//! feita no mesmo quadro.
//!
//! ## ⛔⛔ Porque ela existe: a medição da §5.0
//!
//! *Tabelas em `docs/Render3d/17_a_rota_b_o_catavento.md`.*
//!
//! - A porta que rasteriza e **lê de volta** custa **`31×`** a rasterização a `512²` — o readback é
//!   o preço inteiro. Chamá-la por quadro dá `4` objectos; com a costura residente a corrente
//!   inteira cabe **`26`** vezes num quadro.
//! - E a obra só se justifica **FORA DO PLANO DO ECRÃ**: a rotação NO plano a rota A já a dá
//!   **exactamente** (`0,00°` de desacordo de normais contra `28,58°` de um plano fixo), porque
//!   rodar a imagem e rodar as normais são duas operações 2D sobre o plano já assado. *Uma rota B
//!   construída para a rotação no plano seria trabalho a duplicar o que o assado já entrega.*
//!
//! ## ⛔ A FRONTEIRA, declarada: esta rota é da LEI DA FORMA e não da lei da TINTA
//!
//! A [`super::acende_com`] despacha pela [`crate::lei_da_luz::Lei`] do objecto, e o braço da TINTA é
//! o [`ph2d_render::ImpastoLightPass`], que **recebe fatias da CPU**. Ele não tem por onde consumir
//! uma vista de textura, logo a rota B **não existe** naquela lei — e isso é uma propriedade do
//! passe antigo, não uma escolha desta porta. ⚠️ *Um despacho que aceitasse as duas e caísse
//! silenciosamente na de sempre entregaria um catavento que não gira, sem dizer porquê.*

use ph2d_gpu::GpuContext;
use ph2d_light::LightRig;
use ph2d_render::SpriteRenderer;

use super::{PassesDaLuz, ceu_do_rig, lampadas_do_rig, passe_da_forma};

/// **O que a luz precisa de saber sobre o SPRITE** — o que nesta rota não vem da malha.
///
/// ⚠️ **O `base` continua a ser carregado, e isso é o desenho:** ele são os pixels que o artista
/// desenhou — a ARTE —, e não um subproduto da malha. O que a rota B tira do caminho é a FORMA.
#[derive(Clone, Copy)]
pub struct AlvoVivo<'a> {
    /// O rectângulo do sprite, em pixels.
    pub size: (u32, u32),
    /// O albedo, `w × h × 4` bytes.
    pub base: &'a [u8],
    /// A ranhura individual do sprite, para onde a saída é copiada.
    pub texture_id: u32,
    /// ⭐⭐⭐⭐ **A MATÉRIA DESTE OBJECTO É A PRÓPRIA FORMA** — ver
    /// [`crate::baked_form::BakedForm::materia_da_forma`].
    ///
    /// ⚠️ **É aqui que ele importa a sério.** Na irmã ASSADA o `base` e a forma foram escritos no
    /// mesmo gesto, logo a silhueta congelada está certa; nesta rota a forma é **re-rasterizada
    /// por quadro** e o `base` não — sem esta linha a peça roda por baixo do recorte que o
    /// primeiro bake lhe deu, que é o report da «máscara» (2026-09-21, com foto).
    pub materia_da_forma: bool,
}

/// ⭐⭐⭐ **Acende um objecto VIVO e copia o resultado para o slot do sprite.**
///
/// A irmã assada é a [`super::light`]; a diferença é de onde vem a forma, e **mais nada** — as duas
/// entram no mesmo [`passe_da_forma::PasseDaForma`], com a mesma [`passe_da_forma::LuzDaCena`].
///
/// ⚠️ **As vistas podem ter OUTRO formato** do que as que a rota assada cria (`Rgba32Float` /
/// `R32Float`): o layout declara `Float { filterable: false }` e o shader só faz `textureLoad`,
/// logo o `Rgba16Float` / `R16Float` que a rasterização produz liga-se ali **sem uma linha de
/// shader mudar** — medido a **um byte** no pior pixel sobre `0,26 %` dos componentes.
///
/// # Errors
///
/// Se o `base` não medir `w × h × 4`, se o rig trouxer mais lâmpadas do que o uniform carrega, ou
/// se a cópia para o slot falhar.
pub fn acende_vivo(
    gpu: &GpuContext,
    renderer: &mut SpriteRenderer,
    passes: &mut PassesDaLuz,
    rig: &LightRig,
    alvo: AlvoVivo<'_>,
    forma: (&wgpu::TextureView, &wgpu::TextureView),
) -> Result<(), String> {
    let (w, h) = alvo.size;
    let lampadas = lampadas_do_rig(rig)?;
    let passe = passes
        .forma
        .get_or_insert_with(|| passe_da_forma::PasseDaForma::new(gpu));
    let out = passe.acende_residente(
        gpu,
        passe_da_forma::LuzDaCena {
            material: &crate::lei_da_luz::material_da_forma(),
            lampadas: &lampadas,
            ceu: ceu_do_rig(&lampadas),
            olhar: crate::lei_da_luz::OLHAR_DA_FORMA,
            materia_da_forma: alvo.materia_da_forma,
        },
        alvo.size,
        alvo.base,
        forma,
    )?;
    renderer
        .copy_texture_into_individual(alvo.texture_id, out, w, h)
        .map_err(|e| format!("nao consegui copiar para o slot do sprite: {e}"))
}
