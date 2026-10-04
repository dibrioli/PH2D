//! ⭐⭐⭐ **O DESENHISTA DE JOGO** — triângulos numa passada, luz de custo FIXO por quadro, nada
//! acumulado entre quadros (ordem do dono, 2026-10-02: *«quero o nível de Fortnite / Plants vs
//! Zombies, e a nossa engine roda em mobile»*).
//!
//! # O que ele é, e o que ele NÃO é
//!
//! | | este | o Render traçado do modelador |
//! |---|---|---|
//! | o que desenha | triângulos (a placa do celular foi feita para isto) | o campo, raio a raio |
//! | a luz do céu | a MESMA lei de material (`ph2d_material::wgsl`) com a oclusão ASSADA por vértice | a oclusão marchada a cada quadro |
//! | a sombra da caixa | mapa de sombra com penumbra física (PCSS) | marcha de sombra |
//! | quadros | cada um pronto na hora — o 1.º depois de um giro é o 10.º parado | acumula até assentar |
//! | o que pede à placa | `Features::empty()` + `Limits::downlevel_webgl2_defaults()` | compute e armazenamento |
//!
//! ⭐ **Uma porta por pergunta:** o material é o `ph2d_material::wgsl` (há paridade com a CPU lá), o
//! olhar é o `ph2d_view_transform::wgsl`, e o CÉU é de quem chama ([`Ambiente`]) — o modelador dá o
//! estúdio dele, um jogo dará o seu.

pub mod chao_tapa;
mod fonte;
mod gpu;
mod gpu_alvo;
mod gpu_brilho;
mod gpu_ceu_chao;
mod gpu_cobertura;
mod gpu_contacto;
mod gpu_ligacoes;
mod gpu_triplanar;

pub use fonte::fonte;
pub use gpu::Forward;
pub use gpu_cobertura::COBERTURA_LADO;
pub use gpu_triplanar::TEXTURA_V4;

/// Quantas lâmpadas pontuais por quadro. ⚠️ É o recurso do bloco uniforme do quadro (o WebGL2 só
/// garante `16 KiB` por bloco): `32` pares de `vec4` são `1 KiB`, folga larga para o resto.
pub const MAX_LUZES: usize = 32;

/// Largura da textura onde a tabela do céu viaja (o WebGL2 não tem armazenamento; `2048` é o lado
/// mínimo garantido, e `1024` deixa a tabela do estúdio em `26` linhas).
pub const TAB_W: u32 = 1024;

/// Amostras por pixel. ⚠️ `4×` é o que todo GLES3 garante, e é o anti-serrilhado de jogo de celular:
/// barato numa placa de ladrilhos, sem custo de banda na resolução.
pub const MSAA: u32 = 4;

/// O lado do mapa de sombra. ⚠️ `2048` é o `max_texture_dimension_2d` do WebGL2 — o teto é do
/// aparelho mais pequeno que a engine promete servir.
pub const SOMBRA_LADO: u32 = 2048;

/// Quantos `vec4` um material ocupa — os `48` floats do `ph2d_material::wgsl::pack`.
pub const MATERIAL_V4: u32 = (ph2d_material::wgsl::PACKED / 4) as u32;

/// ⭐ **O céu da cena, dado por quem chama.** O WGSL tem de declarar `struct Ceu` (até `16` floats,
/// em [`Ambiente::constantes`]) e as QUATRO funções das duas partes do céu:
///
/// ```wgsl
/// fn ceu_radiance_sem_caixa(dir: vec3<f32>, shrink: f32) -> vec3<f32>
/// fn ceu_radiance_da_caixa(dir: vec3<f32>, alpha: f32) -> vec3<f32>
/// fn ceu_irradiance_sem_caixa(n: vec3<f32>) -> vec3<f32>
/// fn ceu_irradiance_da_caixa(n: vec3<f32>) -> vec3<f32>
/// ```
///
/// A parte SEM caixa é tapada pela oclusão assada; a DA caixa (a luz forte, de cima) pela sombra.
/// A tabela chega pela função `tabela_ler(i: u32) -> f32`, que esta crate escreve.
#[derive(Clone, Copy, Debug)]
pub struct Ambiente<'a> {
    pub wgsl: &'a str,
    pub constantes: &'a [f32],
    pub tabela: &'a [f32],
    /// A distância mínima de uma lâmpada pontual (a lei do modelador, `POINT_LAMP_MIN_DISTANCE`).
    pub piso_luz: f32,
}

/// Uma malha pronta para subir: triângulos indexados, um material por vértice.
#[derive(Clone, Copy, Debug)]
pub struct Malha<'a> {
    pub posicoes: &'a [[f32; 3]],
    pub normais: &'a [[f32; 3]],
    /// A oclusão do céu assada no vértice: `1` = céu aberto.
    pub ao: &'a [f32],
    pub material: &'a [u32],
    pub indices: &'a [u32],
}

/// A câmara do quadro.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera {
    /// Mundo → recorte, coluna a coluna.
    pub view_proj: [[f32; 4]; 4],
    pub olho: [f32; 3],
    /// `false` = ortográfica: a vista é [`Camera::dir_vista`] em todo o pixel.
    pub perspectiva: bool,
    pub dir_vista: [f32; 3],
}

/// Uma lâmpada pontual — a radiância a distância `1` (`ph2d_field_render::PointLamp`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Luz {
    pub posicao: [f32; 3],
    pub radiancia_a_um: [f32; 3],
}

/// ⭐⭐ **O CÉU FOTOGRÁFICO no quadro** ([`ph2d_sky`]) — substitui as DUAS partes do céu de quem chama:
/// a SEM caixa (a que a oclusão tapa) é o panorama sem o sol, e a DA caixa (a que a sombra tapa) é o
/// SOL dele ([`ph2d_sky::Sol`]), com o peso [`Foto::caixa`] — a `1` é o panorama inteiro. O atlas e a
/// tabela do sol sobem uma vez por céu ([`Forward::sobe_ceu`]); girar, mudar a força ou o fundo não
/// sobe nem compila nada.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Foto {
    /// `(cos θ, sin θ)` do giro do céu em torno de `+y`.
    pub giro: [f32; 2],
    /// O fator linear da radiância do céu.
    pub forca: f32,
    /// O peso do SOL do céu (`1` = o panorama como foi fotografado; `0` = o céu sem o disco).
    pub caixa: f32,
    /// O céu ATRÁS da peça: `None` = transparente (o fundo é de quem chama); `Some(α)` = o céu
    /// filtrado pelo lóbulo de `α` (`0` nítido).
    pub fundo: Option<f32>,
}

/// Um objeto no quadro: a malha subida e a matriz de modelo (coluna a coluna).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Instancia {
    pub malha: u64,
    pub modelo: [[f32; 4]; 4],
}

/// ⭐ **O quadro inteiro** — tudo o que muda de um quadro para o outro, e nada mais.
#[derive(Clone, Copy, Debug)]
pub struct Cena<'a> {
    pub objetos: &'a [Instancia],
    /// Os materiais empacotados (`ph2d_material::wgsl::pack`); o índice é o do vértice.
    pub materiais: &'a [[f32; ph2d_material::wgsl::PACKED]],
    pub camera: Camera,
    pub luzes: &'a [Luz],
    /// A altura do chão que só recebe; `None` = sem chão.
    pub chao: Option<f32>,
    /// A tangente do raio angular da caixa de luz (de cima, `+y`); `None` = sem sombra.
    pub caixa_tan: Option<f32>,
    pub exposicao: f32,
    /// O código do `ph2d_view_transform::wgsl::view_code`.
    pub vista: u32,
    pub tamanho: (u32, u32),
    /// ⭐ **O brilho** — o MESMO tipo que o Render traçado e o Motion autoram (`ph2d_bloom`). Sai
    /// em cena-linear, é somado depois do olhar e com a mesma regra sobre a peça e o fundo
    /// (`ph2d_field_render::soma_halo`). Onde a placa não o tem ([`Forward::tem_brilho`]) é ignorado.
    pub brilho: ph2d_bloom::Bloom,
    /// ⭐ **O estilo** — a MESMA camada do Render traçado (`ph2d_style`): saturação do indirecto
    /// antes das lâmpadas, e tinta de curvatura, contorno e zonas entre a física e o olhar. A
    /// curvatura que ele lê é a de [`Forward::sobe_curvatura`], medida ao passo do estilo.
    pub estilo: ph2d_style::Style,
    /// O raio da bola que envolve a PEÇA — torna a curvatura adimensional (`H · raio`).
    pub raio_da_peca: f32,
    /// ⭐ **O céu fotográfico** — `None` = o céu de quem chama ([`Ambiente`]). Sem céu subido
    /// ([`Forward::tem_ceu`]) é ignorado.
    pub foto: Option<Foto>,
    /// ⭐ **A textura de cada material** (o índice é o de [`Cena::materiais`]); `None` ou fora da
    /// lista = sem textura. A camada é a de [`Forward::sobe_textura`].
    pub texturas: &'a [Option<TexturaMaterial>],
}

/// ⭐⭐ **A textura triplanar de um material** ([`ph2d_triplanar`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TexturaMaterial {
    /// A camada subida por [`Forward::sobe_textura`].
    pub camada: u32,
    pub triplanar: ph2d_triplanar::Triplanar,
    pub tem_normal: bool,
    pub tem_rugosidade: bool,
    /// Mundo → espaço da FOLHA (linhas de uma afim `3 × 4`): a textura anda com a forma.
    pub mundo_para_folha: [[f32; 4]; 3],
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_chao;
#[cfg(test)]
mod tests_chao_tapa;
#[cfg(test)]
mod tests_chao_tapa_baixo;
#[cfg(test)]
mod tests_contacto;
#[cfg(test)]
mod tests_custo_chao;
#[cfg(test)]
mod tests_custo_contacto;
#[cfg(test)]
mod tests_custo_sondas;
#[cfg(test)]
mod tests_custo_textura;
#[cfg(test)]
mod tests_passe_chao_tapa;
#[cfg(test)]
mod tests_reflexo;
#[cfg(test)]
mod tests_sol;
#[cfg(test)]
mod tests_sonda_cpu;
#[cfg(test)]
mod tests_sondas;
#[cfg(test)]
mod tests_textura;
