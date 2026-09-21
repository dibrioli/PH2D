//! ⭐⭐⭐ **A LEI DA FORMA NO DISPOSITIVO** — o mesmo laço da [`ph2d_form_pbr::imagem`], num compute.
//!
//! # ⛔ Porque ele é OBRIGATÓRIO, com o número
//!
//! O caminho de referência (CPU, **em paralelo**, 32 núcleos, `--release`, `load 3,3`) acende um
//! sprite de `1024²` em `11,1 ms` com **uma** lâmpada e `34,1 ms` com **quatro**, contra um
//! orçamento de quadro de `16,7 ms` ⇒ *ele atravessa o orçamento à SEGUNDA lâmpada, e o rig permite
//! quatro*. A re-acendida é o gesto contínuo que o [`super::relight_stale`] promete por escrito
//! (arrastar a lâmpada re-acende **todo** objecto assado, a cada quadro), logo este passe não é
//! aceleração: é a condição de a promessa ser verdade (`docs/Render3d/15` §7).
//!
//! ⚠️⚠️ **E foi a coluna PARALELA que tornou o veredito honesto** — com o número de um núcleo
//! (`111 ms`) a conclusão seria a mesma pela razão errada, que é o §0.0 ao contrário. A margem real
//! são **duas lâmpadas**, e é um número que outra pessoa pode mudar: *quem o mover reconfere isto.*
//!
//! # ⚠️ Porque ele mora AQUI e não na `ph2d-render`, que é o endereço que a §7 escreveu
//!
//! A §7 estimou *«~600 linhas, a medida do passe irmão»* e deu-lhe a morada do
//! [`ph2d_render::ImpastoLightPass`]. Medido, as duas partes daquela frase estão erradas e pela
//! mesma razão — **o irmão carrega coisas que esta acendida não tem**:
//!
//! | o irmão tem | esta acendida |
//! |---|---|
//! | região + janela de planos + `planes_seeded` | é sempre **a tela inteira**, uma vez |
//! | LUT especular a subir e a indexar | a óptica é o OpenPBR, sem tabela |
//! | planos persistentes entre quadros | os canais vêm do documento, prontos |
//!
//! ⇒ o que sobra é **a montagem e o despacho**, e ele cabe onde o consumidor está. E os três
//! argumentos que decidem:
//!
//! * **zero dependências novas** — esta crate já declara `ph2d-form-pbr` (a lei **e** o gémeo),
//!   `ph2d-view-transform`, `ph2d-light`, `ph2d-gpu` e `wgpu`; pô-lo na `ph2d-render` abriria a
//!   aresta `ph2d-render → ph2d-form-pbr` para servir **um** chamador, que vive aqui;
//! * o **único consumidor** é o [`super::acende_pela_forma`], a duas funções de distância;
//! * a `ph2d-render` é **território disputado** nesta rodada (o handoff da `line/Vector` nomeia os
//!   cinco ficheiros do passe de sprites), e um ficheiro novo lá é atrito de fusão a troco de nada.
//!
//! # ⭐ O que atravessa a costura: NENHUMA linha de óptica
//!
//! O corpo do shader é **composto** de três fontes que já existem, e este ficheiro acrescenta só o
//! ponto de entrada (ler três texels, chamar a lei, escrever um). A ordem é a que o cabeçalho do
//! [`ph2d_form_pbr::wgsl`] manda, e ela é load-bearing — o WGSL não tem referência para a frente:
//!
//! 1. [`ph2d_view_transform::wgsl::SOURCE`] — define `vt_to_display`;
//! 2. [`ph2d_form_pbr::wgsl::SOURCE_DA_LEI`] com a ranhura `{ENV}` preenchida — define `Mat`,
//!    `mx_at_base_color` e `mx_direct`;
//! 3. [`ph2d_form_pbr::wgsl::SOURCE`] com a ranhura `{MAX_LAMPADAS}` preenchida — define
//!    `forma_acende_texel`, que chama as três de cima;
//! 4. o [`ENTRADA`] deste ficheiro.
//!
//! ⭐⭐⭐ **A ranhura `{ENV}` leva o CÉU DA CASA, e ele é GERADO — nunca transcrito.**
//!
//! A [`ceu_em_wgsl`] escreve as três constantes lendo-as da [`ph2d_light`], com `format!` ⇒ *não
//! existe uma segunda cópia dos números*. O que é escrito duas vezes é a FÓRMULA (uma vez em Rust,
//! no `env_ambient`; uma vez aqui), e quem as prende é a paridade no PIXEL — que passou a ter algo
//! a dizer sobre ela no dia em que o ambiente deixou de ser zero.
//!
//! ⛔ **Só UMA das duas funções da ranhura faz alguma coisa.** A [`forma_acende_texel`] chama o
//! `env_irradiance` (o termo lambertiano de ambiente) e **não** chama o `env_radiance` (a espelhada
//! pré-filtrada, que só as closures INDIRECTAS do OpenPBR leem — a coluna **B3** do plano). O stub
//! fica porque a fonte da lei traz a ranhura e sem ela não parsa. Há gate a afirmar as duas metades.
//!
//! ⚠️ **A redacção anterior punha as DUAS a zero** com o argumento de que *«o rig é `KEY + 3 × FILL`
//! e as lâmpadas de preenchimento são o ambiente dele»* — e as três de preenchimento nascem
//! **apagadas**: com uma lâmpada acesa e ambiente nulo, `25,03 %` da peça saía PRETA ao bit.
//!
//! # ⚠️ A quantização é EXPLÍCITA, e a CURVA também
//!
//! A CPU escreve `linear_to_srgb_byte(c)`; um `textureStore` num `rgba8unorm` deixa o
//! arredondamento ao backend, e a metade exacta (`.5`) é onde os dois se separam. ⇒ o shader
//! arredonda ele próprio, que é a política que o passe irmão já declara por escrito.
//!
//! ⛔⛔ **E a CODIFICAÇÃO sRGB está no mesmo barco, pela MESMA razão.** A saída é `Rgba8Unorm`
//! (armazenamento **linear**) e vai copiada byte a byte para uma ranhura `Rgba8UnormSrgb`, que o
//! hardware **descodifica** ao amostrar — logo a curva tem de ser aplicada aqui, e o backend não a
//! pode aplicar por nós: um `rgba8unorm-srgb` como alvo de `texture_storage_2d` **não é suportado**.
//! ⇒ `linear_to_srgb` em WGSL, o gémeo do [`ph2d_color::srgb::linear_to_srgb_unit`] que a
//! [`ph2d_form_pbr::imagem`] chama, e a paridade no pixel prende os dois.
//!
//! ⚠️ **A ORDEM é load-bearing:** codificar e **depois** quantizar. Ao contrário, os degraus de 8
//! bits do linear ficam espalhados pela curva e aparecem como bandas no escuro.

use ph2d_form_pbr::wgsl as gemeo;
use ph2d_form_pbr::{Ceu, Lampada, Surface, imagem::Planos};
use ph2d_gpu::GpuContext;
use ph2d_view_transform::Look;

/// Aresta do grupo de trabalho — espelha o `@workgroup_size(8, 8, 1)` da [`ENTRADA`].
const ARESTA: u32 = 8;

/// Quantas lâmpadas o uniform carrega.
///
/// ⚠️ **É o número do RIG, e não um espelho dele** — a lição que o passe irmão já pagou (ele tinha
/// um `4` escrito à mão com um comentário a dizer *«espelha o rig»* e um gate que o comparava
/// contra o literal `4`, ou seja um gate que não podia falhar pelo motivo que alegava).
pub const MAX_LAMPADAS: usize = ph2d_light::MAX_LIGHTS;

/// ⭐ **O PONTO DE ENTRADA — e a única coisa deste ficheiro que é WGSL.**
///
/// Ver o cabeçalho do módulo para a ordem da composição e para os dois stubs do ambiente.
const ENTRADA: &str = r#"
struct Globais {
    material: Mat,
    // ⚠️ `rgb` = RESERVA DECLARADA (era o ambiente, que hoje vive na ranhura do ambiente e é função da
    // normal); `a` = os stops de exposição do olhar. *Uma posição sem dono e sem régua é onde o
    // campo seguinte aterra por engano* — o `Globais::novo` deixa-a a ZERO, e há gate.
    olhar: vec4<f32>,
    // ⭐ O CÉU (`ph2d_form_pbr::Ceu`), em `rgb`: a base da rampa e a inclinação dela.
    //
    // ⚠️ Ele viaja como DADOS e não como constantes geradas: a rampa é derivada do RIG (ver o
    // `ceu_do_rig`), logo ela muda quando o artista mexe numa lâmpada — e uma constante no shader
    // pediria uma recompilação por gesto.
    ceu_base: vec4<f32>,
    ceu_inclinacao: vec4<f32>,
    // x = o código da vista (`ph2d_view_transform::wgsl::view_code`).
    vista: vec4<u32>,
    lampadas: Lampadas,
};

@group(0) @binding(0) var base_tex: texture_2d<f32>;
@group(0) @binding(1) var form_tex: texture_2d<f32>;
@group(0) @binding(2) var occ_tex: texture_2d<f32>;
@group(0) @binding(3) var saida: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(4) var<uniform> g: Globais;

// ⭐⭐⭐ **IEC 61966-2-1, linear → sRGB, por canal** — o GEMEO do
// `ph2d_color::linear_to_srgb_unit`, que é quem a `ph2d_form_pbr::imagem` chama.
//
// ⚠️ **Ele NÃO pode vir do backend:** a saída é um `rgba8unorm` (armazenamento LINEAR) e o
// `textureStore` não codifica nada. Um `rgba8unorm-srgb` como alvo de armazenamento **não é
// suportado** pelo `wgpu`, logo a curva é nossa — como a quantização já era, e pela mesma razão.
//
// ⚠️ A forma é a que esta casa já escreve em `band_blit.wgsl` e `compositor.wgsl` (`step` + `mix`,
// sem ramo), e o `clamp` vem primeiro para o `pow` nunca ver um negativo.
fn srgb_to_linear(c: vec3<f32>) -> vec3<f32> {
    let lo = c / 12.92;
    let hi = pow((c + vec3<f32>(0.055)) / 1.055, vec3<f32>(2.4));
    let cutoff = step(vec3<f32>(0.04045), c);
    return mix(lo, hi, cutoff);
}

fn linear_to_srgb(c: vec3<f32>) -> vec3<f32> {
    let safe = clamp(c, vec3<f32>(0.0), vec3<f32>(1.0));
    let lo = safe * 12.92;
    let hi = 1.055 * pow(safe, vec3<f32>(1.0 / 2.4)) - vec3<f32>(0.055);
    let cutoff = step(vec3<f32>(0.0031308), safe);
    return mix(lo, hi, cutoff);
}

@compute @workgroup_size(8, 8, 1)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let dim = textureDimensions(saida);
    if (gid.x >= dim.x || gid.y >= dim.y) { return; }
    let p = vec2<i32>(i32(gid.x), i32(gid.y));

    // ⚠️ A textura é `rgba8unorm` e NUNCA `…Srgb` — logo o `textureLoad` entrega o **código**
    // normalizado e não a luz, e quem descodifica somos nós. ⛔ Pedir a ranhura como `…Srgb` para o
    // backend o fazer **não serve**: o `base` vai ao device pelo mesmo caminho do passe da TINTA,
    // que é RELATIVO e trata estes bytes como códigos de propósito. *Cada lei paga a convenção
    // dela na porta dela.* Ver o cabeçalho da [`ph2d_form_pbr::imagem`].
    let px = textureLoad(base_tex, p, 0);
    let albedo = srgb_to_linear(px.rgb);
    let f = textureLoad(form_tex, p, 0);
    let occ = textureLoad(occ_tex, p, 0).r;

    // ⚠️ **O céu entra pelos `var<private>` ANTES da chamada** — ver o `CEU_NA_RANHURA` para
    // porque ele não pode ler o `g` directamente.
    ceu_base = g.ceu_base.rgb;
    ceu_inclinacao = g.ceu_inclinacao.rgb;

    let c = forma_acende_texel(
        g.material,
        f.xyz,
        albedo,
        f.w,
        occ,
        g.lampadas,
        g.olhar.a,
        g.vista.x,
    );

    // ⚠️ **O ALFA atravessa intacto** — ele é a silhueta do sprite, e uma lei de luz que lhe
    // tocasse mudaria o RECORTE do objecto ao mover a lâmpada.
    // ⚠️ E a quantização é nossa, não do backend — ver o cabeçalho do módulo.
    // ⭐ A CURVA vem ANTES dela, e a ordem é load-bearing: quantizar o linear e codificar depois
    // dava os degraus do linear espalhados pela curva (bandas visíveis no escuro).
    let q = floor(linear_to_srgb(c) * 255.0 + 0.5) / 255.0;
    textureStore(saida, p, vec4<f32>(q, px.a));
}
"#;

/// ⭐⭐⭐ **O CÉU DA CASA, em WGSL — as duas funções que a ranhura `{ENV}` pede.**
///
/// ⚠️ **As três constantes são GERADAS da [`ph2d_light`] e nunca transcritas**, que é o que impede a
/// segunda cópia de um número. O `{:?}` de um `f32` é a representação mais curta que faz round-trip,
/// e é um literal válido de WGSL.
///
/// ⚠️ **O `fma` é EXPLÍCITO e isso é para a paridade**: a [`ph2d_light::env_ambient`] escreve
/// `ENV_SLOPE[i].mul_add(up, ENV_BASE[i])`, que é uma multiplicação-soma com **um** arredondamento.
/// Escrito como `base - slope * n.y`, o WGSL fica livre de contrair ou não — e esta casa já mediu
/// uma placa a contrair `a*b + c` num `fma` onde o fonte não o pedia. *Pedir o `fma` nos dois lados
/// é a única forma de a igualdade não depender do compilador.*
///
/// ⭐⭐⭐ **O `env_radiance` DEIXOU DE SER ZERO em 2026-09-20 (a coluna B3)** — e a redacção anterior
/// desta linha dizia *«ele fica a ZERO de propósito: nada nesta lei o chama»*. Chama.
const CEU_NA_RANHURA: &str = r#"
// **O CEU** — a rampa linear na altura da TELA (`ph2d_form_pbr::Ceu`).
//
// ⛔⛔ Os dois valores chegam por `var<private>` e NAO por uma leitura do `g`, e a razao e' a ORDEM
// da composicao: esta ranhura e' preenchida DENTRO da fonte da lei, que vem antes do `ENTRADA` —
// logo o `struct Globais` ainda nao existe aqui. Quem os escreve e' o `cs_main`, no topo.
var<private> ceu_base: vec3<f32>;
var<private> ceu_inclinacao: vec3<f32>;

// ⚠️ O ceu e' o topo da TELA, e neste referencial (CANVAS) o topo e' `-y`.
// ⚠️ **`fma` EXPLICITO** — o gemeo em Rust usa `mul_add`, que tem UM arredondamento; escrito solto,
// a igualdade ao bit passaria a depender de o compilador do WGSL contrair a multiplicacao-soma.
fn env_irradiance(n: vec3<f32>) -> vec3<f32> {
    return fma(ceu_inclinacao, vec3<f32>(-n.y), ceu_base);
}

// ⭐⭐⭐ **A ESPELHADA PRE'-FILTRADA** — o gemeo da `ph2d_form_pbr::Ceu::radiancia`.
//
// ⚠️ **O `alpha` NAO e' lido e o `shrink` e'**: o encolhimento do lobulo e' constante por MATERIAL e
// viaja PRONTO dentro do `Mat` empacotado. Correr aqui o logaritmo que o produz seria por a mesma
// conta a dar o mesmo numero um milhao de vezes. ⛔ O NOME da funcao que o calcula nao se escreve
// neste texto: ha' gate a varre'-lo, e uma agulha num comentario le-se igual a uma chamada.
//
// ⚠️ **`1.5` e' `1/(2/3)`**: o `ceu_inclinacao` e' a inclinacao da IRRADIANCIA, que ja' traz o `A1`
// do lobulo cosseno la' dentro; a radiancia quer a rampa crua. ⚠️ E a ASSOCIACAO e' a do gemeo em
// Rust — o `1.5 * inc` primeiro, e so' depois o `fma`.
fn env_radiance(dir: vec3<f32>, alpha: f32, shrink: f32) -> vec3<f32> {
    let up = shrink * -dir.y;
    return fma(1.5 * ceu_inclinacao, vec3<f32>(up), ceu_base);
}
"#;

/// ⭐⭐ **O corpo do shader, composto** — e uma função pública porque o gate o quer **sem placa**.
///
/// ⚠️ Um WGSL que ninguém compila é prosa, e todo gate desta casa que olha para um shader precisa de
/// adapter e é `#[ignore]`. Com a montagem numa porta, a `naga` pode parsá-la e validá-la como
/// aritmética — que é o que apanhou os dois defeitos que o gémeo tinha antes de haver um pixel.
#[must_use]
pub fn fonte() -> String {
    format!(
        "{}\n{}\n{}\n{}",
        ph2d_view_transform::wgsl::SOURCE,
        gemeo::SOURCE_DA_LEI.replace(gemeo::ENV_SLOT, CEU_NA_RANHURA),
        gemeo::SOURCE.replace(gemeo::CAP_SLOT, &format!("{MAX_LAMPADAS}u")),
        ENTRADA,
    )
}

/// O uniform, com a disposição do `struct Globais` do [`ENTRADA`].
///
/// ⚠️ **Escrito como `f32` crus e não como uma struct com `repr(C)`**, porque a parte grande dele é
/// o material — que chega já empacotado da porta que a `ph2d-material` declara, e re-declarar os
/// doze `vec4` aqui seria a segunda redacção de um layout. O gate `o_uniform_tem_a_forma_que_o_wgsl_le`
/// prende os quatro offsets.
/// ⭐⭐⭐ **A LEI DA LUZ de uma acendida, nos quatro valores de que o [`Globais::novo`] é função.**
///
/// ⛔⛔ **Ela NÃO é açúcar para calar um lint.** Os quatro viajam SEMPRE juntos: as **duas** portas
/// do passe ([`PasseDaForma::acende`] e [`PasseDaForma::acende_residente`]) recebem-nos só para os
/// entregar ao MESMO `Globais::novo`, e nenhuma delas lê um sem os outros. *Um grupo que já é a
/// lista de argumentos de uma função é um TIPO que faltava* — e sem ele a 2.ª porta escrevia a
/// mesma quádrupla por extenso, que é a segunda ortografia da mesma lei.
///
/// ⚠️ **É `Copy` de propósito**: ela é um empréstimo de quatro coisas que o chamador já tem, e não
/// um dono — passá-la não muda a vida de nada.
#[derive(Clone, Copy)]
pub struct LuzDaCena<'a> {
    /// O material do sprite (a [`ph2d_form_pbr`] empacota-o).
    pub material: &'a Surface,
    /// As lâmpadas do rig — ⚠️ mais do que [`MAX_LAMPADAS`] é **recusado**, não truncado.
    pub lampadas: &'a [Lampada],
    /// O céu, nas duas metades que o `env_ambient` lê.
    pub ceu: Ceu,
    /// O olhar (exposição + vista), que o `vt_to_display` pede.
    pub olhar: Look,
}

struct Globais {
    dados: Vec<f32>,
}

impl Globais {
    /// `Mat` + `olhar` + `ceu_base` + `ceu_inclinacao` + `vista` + `Lampadas { n, _pad×3, l[MAX] }`.
    const FLOATS: usize = gemeo::PACKED + 4 + 4 + 4 + 4 + 4 + MAX_LAMPADAS * gemeo::LAMPADA_FLOATS;

    fn novo(luz: LuzDaCena) -> Result<Self, String> {
        let LuzDaCena {
            material,
            lampadas,
            ceu,
            olhar,
        } = luz;
        if lampadas.len() > MAX_LAMPADAS {
            return Err(format!(
                "o passe da forma carrega {MAX_LAMPADAS} lampadas e o rig trouxe {}",
                lampadas.len()
            ));
        }
        let mut d = vec![0.0f32; Self::FLOATS];
        // ⭐⭐ **O encolhimento do lóbulo viaja PRONTO** (a coluna B3): os dois `shrink` são lidos
        // pelo `env_radiance`, e a porta que os compõe é a da crate que os nomeia. ⛔ A redacção
        // anterior passava `EnvLobe::IGNORED` *«porque esta lei não chama o `env_radiance`»* — hoje
        // chama, e um `IGNORED` deixaria a espelhada colada ao equador em todo material.
        d[..gemeo::PACKED].copy_from_slice(&gemeo::pack(material, gemeo::EnvLobe::of(material)));
        let i = gemeo::PACKED;
        // ⚠️ `d[i..i+3]` fica a ZERO: é a RESERVA declarada do `olhar` — ver o `struct Globais`.
        d[i + 3] = olhar.exposure_stops;
        // ⭐ O CÉU, no `rgb` de dois `vec4` — o `a` de cada um é reserva declarada, pelo mesmo motivo.
        d[i + 4..i + 7].copy_from_slice(&ceu.base);
        d[i + 8..i + 11].copy_from_slice(&ceu.inclinacao);
        // ⚠️ O código da vista viaja como `u32` e é escrito aqui pelos bits: o `f32` do vector é só
        // o transporte, e o WGSL lê o slot como `vec4<u32>`.
        d[i + 12] = f32::from_bits(ph2d_view_transform::wgsl::view_code(olhar.view));
        // O `n` das lâmpadas, no primeiro slot do `Lampadas` — os três a seguir são o padding que o
        // alinhamento de 16 bytes do array exige (ver [`gemeo::LAMPADA_FLOATS`]).
        let j = i + 16;
        d[j] = f32::from_bits(u32::try_from(lampadas.len()).unwrap_or(0));
        let empacotadas = gemeo::pack_lampadas(lampadas);
        d[j + 4..j + 4 + empacotadas.len()].copy_from_slice(&empacotadas);
        Ok(Self { dados: d })
    }

    fn bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.dados)
    }
}

/// As cinco texturas de um tamanho, **reconstruídas juntas**.
///
/// ⚠️ Uma checagem de dimensão em vez de cinco que podiam discordar — a lei que o passe irmão
/// declara por escrito. E a ranhura é **uma só**: o gesto que a paga é arrastar a lâmpada, onde o
/// mesmo sprite é re-aceso quadro após quadro.
struct Alvo {
    largura: u32,
    altura: u32,
    base: wgpu::TextureView,
    base_tex: wgpu::Texture,
    form: wgpu::TextureView,
    form_tex: wgpu::Texture,
    occ: wgpu::TextureView,
    occ_tex: wgpu::Texture,
    saida: wgpu::TextureView,
    saida_tex: wgpu::Texture,
}

/// ⭐⭐⭐ **O passe.** Um por sessão, ao lado do [`ph2d_render::ImpastoLightPass`] que acende a outra lei.
pub struct PasseDaForma {
    pipeline: wgpu::ComputePipeline,
    bgl: wgpu::BindGroupLayout,
    uniforme: wgpu::Buffer,
    alvo: Option<Alvo>,
}

impl PasseDaForma {
    /// Compila o pipeline. Barato — nenhuma textura até à primeira [`Self::acende`].
    #[must_use]
    pub fn new(gpu: &GpuContext) -> Self {
        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("ph2d-form-donation passe_da_forma"),
                source: wgpu::ShaderSource::Wgsl(fonte().into()),
            });
        let lido = |binding: u32| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Texture {
                // ⚠️ `filterable: false` e não uma escolha: os canais da forma são `Rgba32Float`,
                // que o núcleo do WebGPU não deixa filtrar — e o shader só faz `textureLoad`.
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let bgl = gpu
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("ph2d-form-donation passe_da_forma bgl"),
                entries: &[
                    lido(0), // base, por acender
                    lido(1), // a forma doada
                    lido(2), // a oclusão de forma
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            format: wgpu::TextureFormat::Rgba8Unorm,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });
        let layout = gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("ph2d-form-donation passe_da_forma layout"),
                bind_group_layouts: &[Some(&bgl)],
                immediate_size: 0,
            });
        let pipeline = gpu
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("ph2d-form-donation passe_da_forma pipeline"),
                layout: Some(&layout),
                module: &shader,
                entry_point: Some("cs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            });
        let uniforme = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-form-donation passe_da_forma globais"),
            size: (Globais::FLOATS * 4) as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            pipeline,
            bgl,
            uniforme,
            alvo: None,
        }
    }

    /// ⭐⭐⭐ **Acende o sprite inteiro e devolve a textura acesa.**
    ///
    /// Ela sai com `COPY_SRC` porque o destino dela é o slot do sprite — ⛔ a usage em falta era um
    /// **panic** do wgpu e não um erro devolvido, e nada sem placa a via (ver o doc da
    /// [`super::planes::upload_rgba_copiavel`], que pagou a mesma lição).
    ///
    /// # Errors
    ///
    /// Se algum plano não medir o que o `size` pede (a MESMA [`Planos::confere`] que o caminho de
    /// referência corre — *um segundo predicado de «este pedido está bem formado» continuaria a
    /// passar depois de o primeiro ficar torto*), ou se o rig trouxer mais lâmpadas do que o
    /// uniform carrega.
    pub fn acende(
        &mut self,
        gpu: &GpuContext,
        luz: LuzDaCena,
        planos: &Planos,
    ) -> Result<&wgpu::Texture, String> {
        planos.confere()?;
        let globais = Globais::novo(luz)?;
        let (w, h) = planos.size;
        self.garante_alvo(gpu, w, h);
        {
            let alvo = self.alvo.as_ref().expect("acabou de ser garantido");
            escreve(gpu, &alvo.base_tex, planos.base, w * 4, (w, h));
            escreve(
                gpu,
                &alvo.form_tex,
                bytemuck::cast_slice(planos.form),
                w * 16,
                (w, h),
            );
            escreve(
                gpu,
                &alvo.occ_tex,
                bytemuck::cast_slice(planos.form_occ),
                w * 4,
                (w, h),
            );
        }
        self.despacha(gpu, &globais, (w, h), None)
    }

    /// ⭐⭐⭐ **O MESMO PASSE, com a FORMA JÁ NA PLACA** — a costura da **rota B** (o catavento).
    ///
    /// A [`Self::acende`] recebe os dois planos da forma como fatias da CPU e **carrega-os** a cada
    /// acendida, porque quem a chama é o assado: ele corre **uma vez, num botão**, e o `Vec<f32>` é
    /// o que viaja no documento. Esta recebe-os como **vistas de textura que já vivem na placa** —
    /// tipicamente a saída de uma rasterização feita no mesmo quadro — e **nada desce nem sobe**.
    ///
    /// # ⛔⛔ Porque ela existe: a medição da §5.0 do catavento
    ///
    /// A porta que rasteriza e traz os planos de volta ([`ph2d_mesh_render::MeshRenderer::form_plane`])
    /// custa **`31×`** a rasterização sozinha a `512²` — o readback é o preço inteiro. Chamá-la por
    /// quadro dá `4` objectos; com esta costura a corrente cabe `26` vezes num quadro.
    /// Tabelas: `docs/Render3d/17_a_rota_b_o_catavento.md`.
    ///
    /// # ⭐ E ela não custou uma linha de shader
    ///
    /// ⚠️ **As vistas podem ter OUTRO formato do que as texturas da [`Self::acende`]** (que são
    /// `Rgba32Float`/`R32Float`): o layout declara `Float { filterable: false }` e o shader só faz
    /// `textureLoad`, logo o `Rgba16Float`/`R16Float` que a rasterização produz liga-se ali sem
    /// nada mudar — e o `f16` sobe a `f32` **sem perda** (10 bits de mantissa contra 23).
    ///
    /// ⛔ **O `base` continua a ser carregado**, e isso é o desenho: ele são os pixels que o artista
    /// desenhou — a arte —, e não um subproduto da malha.
    ///
    /// # Errors
    ///
    /// Se o `base` não medir `w × h × 4`, ou se o rig trouxer mais lâmpadas do que o uniform carrega.
    pub fn acende_residente(
        &mut self,
        gpu: &GpuContext,
        luz: LuzDaCena,
        size: (u32, u32),
        base: &[u8],
        forma: (&wgpu::TextureView, &wgpu::TextureView),
    ) -> Result<&wgpu::Texture, String> {
        let (w, h) = size;
        let esperado = w as usize * h as usize * 4;
        if base.len() != esperado {
            return Err(format!(
                "base mede {} e o tamanho pede {esperado}",
                base.len()
            ));
        }
        let globais = Globais::novo(luz)?;
        self.garante_alvo(gpu, w, h);
        {
            let alvo = self.alvo.as_ref().expect("acabou de ser garantido");
            escreve(gpu, &alvo.base_tex, base, w * 4, (w, h));
        }
        self.despacha(gpu, &globais, (w, h), Some(forma))
    }

    /// **O DESPACHO — a lei, uma vez só.**
    ///
    /// ⚠️ Ela existe porque as duas entradas acima diferem **apenas em de onde vem a forma**, e
    /// *uma lei escrita em dois sítios ainda não é uma lei*: com duas cópias, a próxima linha que
    /// mexesse no bind group ou no despacho teria de ser escrita nas duas, e a que alguém
    /// esquecesse divergia em silêncio.
    fn despacha(
        &self,
        gpu: &GpuContext,
        globais: &Globais,
        (w, h): (u32, u32),
        residente: Option<(&wgpu::TextureView, &wgpu::TextureView)>,
    ) -> Result<&wgpu::Texture, String> {
        let alvo = self.alvo.as_ref().expect("acabou de ser garantido");
        // ⛔ **O `unwrap_or` é a única diferença entre as duas rotas** — e é aqui, e não em dois
        // corpos paralelos, porque é isso que torna a rota B *a mesma lei com outra fonte*.
        let (forma, oclusao) = residente.unwrap_or((&alvo.form, &alvo.occ));
        gpu.queue.write_buffer(&self.uniforme, 0, globais.bytes());

        let bg = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-form-donation passe_da_forma bg"),
            layout: &self.bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&alvo.base),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(forma),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(oclusao),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&alvo.saida),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: self.uniforme.as_entire_binding(),
                },
            ],
        });
        let mut enc = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ph2d-form-donation passe_da_forma enc"),
            });
        {
            let mut cp = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("ph2d-form-donation passe_da_forma"),
                timestamp_writes: None,
            });
            cp.set_pipeline(&self.pipeline);
            cp.set_bind_group(0, &bg, &[]);
            cp.dispatch_workgroups(w.div_ceil(ARESTA), h.div_ceil(ARESTA), 1);
        }
        gpu.queue.submit(std::iter::once(enc.finish()));
        Ok(&alvo.saida_tex)
    }

    fn garante_alvo(&mut self, gpu: &GpuContext, w: u32, h: u32) {
        if self
            .alvo
            .as_ref()
            .is_some_and(|a| a.largura == w && a.altura == h)
        {
            return;
        }
        use wgpu::TextureFormat as F;
        use wgpu::TextureUsages as U;
        let base_tex = textura(gpu, "base", w, h, F::Rgba8Unorm, U::TEXTURE_BINDING);
        // ⚠️ Ponto flutuante, e não um formato normalizado: as componentes de uma normal vivem em
        // `[-1, 1]` — a mesma razão que o passe irmão já escreve.
        let form_tex = textura(gpu, "form", w, h, F::Rgba32Float, U::TEXTURE_BINDING);
        let occ_tex = textura(gpu, "form_occ", w, h, F::R32Float, U::TEXTURE_BINDING);
        let saida_tex = textura(
            gpu,
            "saida",
            w,
            h,
            F::Rgba8Unorm,
            U::STORAGE_BINDING | U::COPY_SRC,
        );
        let v = |t: &wgpu::Texture| t.create_view(&wgpu::TextureViewDescriptor::default());
        self.alvo = Some(Alvo {
            largura: w,
            altura: h,
            base: v(&base_tex),
            form: v(&form_tex),
            occ: v(&occ_tex),
            saida: v(&saida_tex),
            base_tex,
            form_tex,
            occ_tex,
            saida_tex,
        });
    }
}

fn textura(
    gpu: &GpuContext,
    nome: &str,
    w: u32,
    h: u32,
    formato: wgpu::TextureFormat,
    usos: wgpu::TextureUsages,
) -> wgpu::Texture {
    gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some(&format!("ph2d-form-donation passe_da_forma {nome}")),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: formato,
        usage: usos | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    })
}

fn escreve(gpu: &GpuContext, tex: &wgpu::Texture, dados: &[u8], linha: u32, (w, h): (u32, u32)) {
    gpu.queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: tex,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        dados,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(linha),
            rows_per_image: Some(h),
        },
        wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
    );
}

#[cfg(test)]
#[path = "passe_da_forma_tests.rs"]
mod tests;
