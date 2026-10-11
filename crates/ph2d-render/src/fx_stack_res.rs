//! **Os RECURSOS do passe da pilha de FX** — o layout do uniform e as texturas de trabalho.
//!
//! Irmão de [`super::fx_stack`] pelo teto de LOC, e o corte é por responsabilidade: aquele arquivo
//! é *o FOLD* (que passes correm, em que ordem, com que bind groups), este é *o que o passe ALOCA
//! e como ele fala com o device*.
//!
//! ⚠️ O [`Globals`] mora aqui e o gate de paridade (`the_wgsl_globals_members_match_the_rust_struct`)
//! lê ESTE arquivo: um uniform é lido por OFFSET, então a ordem dos membros dos dois lados é a
//! coisa mais frágil do módulo — trocar dois `u32` não falha ligação nenhuma, só passa a desenhar
//! outra coisa.

use ph2d_gpu::GpuContext;

use crate::fx_stack::{UNIFORM_STRIDE, kernel_half};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct Globals {
    pub(crate) dims: [u32; 2],
    pub(crate) half: u32,
    pub(crate) kind: u32,
    pub(crate) tint: [f32; 4],
    pub(crate) inv_two_sigma2: f32,
    pub(crate) opacity: f32,
    pub(crate) off_x: f32,
    pub(crate) off_y: f32,
    /// O passo do salto do JFA (só os passes de campo de distância o leem).
    pub(crate) jump: i32,
    /// A largura da banda (modo Contour) / do contorno, em pixels.
    pub(crate) band: f32,
    /// Quantos segmentos de SILHUETA o chamador entregou. `0` = semeia pela cobertura (o caminho
    /// antigo, e o único disponível quando não há geometria — os gates de raster puro).
    pub(crate) n_segs: u32,
    /// A LEI DE MISTURA deste degrau (o código de `BlendMode`; `0` = Normal, o neutro).
    pub(crate) blend: u32,
    /// O tamanho das ondulações do ruído, em pixels (só a turbulência o lê).
    pub(crate) noise_scale: f32,
    /// Quantas oitavas o ruído soma.
    pub(crate) octaves: u32,
    /// Qual realização do ruído.
    pub(crate) seed: u32,
    /// O MODO do degrau — o índice em `FxKindSpec::modes`.
    ///
    /// ⚠️ **O `mode` chega ao DEVICE pela primeira vez aqui.** Os modos anteriores (Proximity vs
    /// Contour) escolhem o PLANO, então nunca precisaram de um ramo no shader; os do ruído
    /// escolhem a lei da soma de oitavas, que só existe lá dentro.
    pub(crate) mode: u32,
    /// A ORIGEM da grade de ruído, em texels do scratch — a margem que a pilha reservou. É ela que
    /// prega o padrão na FORMA em vez de na textura.
    pub(crate) org: [f32; 2],
    /// Quanto a silhueta engorda, em pixels — **com sinal** (só a morfologia o lê).
    pub(crate) grow_px: f32,
    /// ⚠️ **O padding é EXPLÍCITO de propósito.** O `min_binding_size` do layout é
    /// `size_of::<Globals>()`, e o WGSL arredonda o tamanho de um struct de uniform ao alinhamento
    /// dele (16, por causa do `vec4`); sem estes campos o Rust reportaria um tamanho MENOR que o do
    /// WGSL e o wgpu recusaria o bind group. Há gate a pinar o número (`fx_stack_tests`).
    ///
    /// ⚠️ Esta nota morava, obsoleta, num doc-comment órfão sobre um `use` no `fx_stack_shader` —
    /// ela ainda dizia "64 bytes" com o struct em 112.
    /// `1` = a origem do ruído é POR TEXEL, lida do `t1` do passe de deformação (as células).
    pub(crate) org_map: u32,
    /// A MATIZ em VOLTAS, a saturação e o brilho em `-1..1` — só o Color Adjust os lê.
    ///
    /// ⚠️ **Voltas, e não o par (sen, cos) precomputado no host.** Precomputar seria mais barato
    /// por um `sin`/`cos` por op, e escreveria um kernel DIFERENTE do que o compositor de camadas
    /// já roda — que é exactamente o que esta wave existe para não fazer. O `adjust_hsb` é UM.
    pub(crate) hue: f32,
    pub(crate) sat: f32,
    pub(crate) bright: f32,
    pub(crate) _pad2: [f32; 1],
    /// A SEGUNDA cor RETA do degrau — a ponta CLARA da rampa do Duotone. Só ele a lê.
    ///
    /// ⚠️ **Um `vec4` inteiro, alinhado**, e é por isso que ele vem no FIM: o WGSL alinha `vec4` a
    /// 16 bytes, e a fileira `hue`/`sat`/`bright`/`_pad2` fecha exactamente uma. Encaixado no meio,
    /// ele empurraria padding implícito que o Rust não escreve e o `min_binding_size` recusaria.
    pub(crate) tint_b: [f32; 4],
    /// **Onde, na textura de FONTE, começa a célula desta forma.** Só o `cs_ingest` o lê — e é
    /// isso que faz dele a porta única do ATLAS de raster (ver o comentário lá).
    ///
    /// `[0, 0]` = a fonte É a forma, o mundo de antes do atlas, byte-idêntico.
    pub(crate) src_org: [i32; 2],
    /// Quantos stops da rampa valem. `0` = a rampa preto→branco, a identidade em luma.
    ///
    /// ⚠️ **Ele mora AQUI, na fileira do `src_org`, e o lugar é OBRIGATÓRIO** — ela tinha 8 bytes
    /// livres, e a alternativa (um `vec3<u32>` de padding *depois* dos arrays) tem **alinhamento 16
    /// no WGSL**, o que empurra padding implícito que o Rust não escreve. **Medido pelo device:**
    /// WGSL **336** contra Rust **320**, e o `min_binding_size` recusou o bind group com os dois
    /// números no erro — exactamente a classe de falha que o doc do `_pad` promete pegar, e que
    /// derrubou os 82 gates de GPU de uma vez.
    pub(crate) stop_count: u32,
    /// **A fonte chega PRÉ-MULTIPLICADA?** (`1` = sim; só o ingest o lê). O Vector entrega sRGB
    /// reto e escreve `0`; a camada do Motion (doc 129 §3.3) é o acumulador do desenhista,
    /// pré-multiplicado. Fecha a fileira de 16 bytes que o `src_org` abriu.
    pub(crate) src_premul: u32,
    /// **Os STOPS da rampa**, RGBA — o `rgb` é sRGB reto em `[0,1]` (o shader o lineariza pela
    /// MESMA porta das duas pontas do Duotone) e o **alfa é a FORÇA** daquele stop.
    ///
    /// ⚠️ **Vem no FIM, com o `stop_pos`,** porque `array<vec4<f32>, N>` exige offset múltiplo de
    /// 16: encaixado antes de um escalar, ele empurraria padding que os dois lados contariam
    /// diferente.
    pub(crate) stops: [[f32; 4]; 8],
    /// As POSIÇÕES dos oito stops, empacotadas em dois `vec4`.
    ///
    /// ⚠️ **Empacotadas de propósito:** o WGSL dá stride **16** a um `array<f32>` no address space
    /// de uniform, então oito floats soltos custariam 128 bytes para carregar 32.
    pub(crate) stop_pos: [[f32; 4]; 2],
    /// Os passes da PIRÂMIDE do borrão (`fx_stack_piramide`): as dimensões de entrada/de baixo, o
    /// fator, e as marcas «1.º nível» / «já é a fonte». Zeros = o caminho de sempre.
    pub(crate) aux: [i32; 4],
    /// A rampa ASSADA deste degrau no buffer `ramp_lut` (binding 5): `[base, comprimento, 0, 0]`.
    /// Comprimento `0` = os `stops` (o Vector, sempre).
    pub(crate) ramp: [u32; 4],
}

impl Globals {
    /// **Os globals de UM degrau.**
    ///
    /// ⚠️ Mora aqui, e não no fold, porque o arquivo que é DONO do uniform é quem tem de saber
    /// preenchê-lo: os dois lados do layout (este struct e o WGSL) já vivem juntos por um gate de
    /// paridade, e a terceira metade — *com que números cada campo é escrito* — divergia num arquivo
    /// separado. O fold decide QUAIS passes correm; este decide o que cada um LÊ.
    ///
    /// O `n_segs` chega RESOLVIDO do plano de propósito: *"contra o quê este degrau mede?"* tem uma
    /// resposta, e ela é o plano — derivá-la aqui outra vez seria a segunda porta.
    pub(crate) fn for_op(op: &crate::FxOpGpu, dims: [u32; 2], org: [f32; 2], n_segs: u32) -> Self {
        let sigma = op.sigma_px.max(1e-4);
        Self {
            dims,
            half: kernel_half(op.sigma_px),
            kind: u32::from(op.kind),
            tint: op.tint,
            inv_two_sigma2: 1.0 / (2.0 * sigma * sigma),
            opacity: op.opacity,
            off_x: op.offset_px[0],
            off_y: op.offset_px[1],
            jump: 0,
            band: op.sigma_px.max(0.0),
            n_segs,
            blend: u32::from(op.blend),
            noise_scale: op.noise_scale_px.max(1e-3),
            octaves: u32::from(op.detail.max(1)),
            seed: u32::from(op.seed),
            mode: u32::from(op.mode),
            org,
            grow_px: op.grow_px,
            org_map: 0,
            hue: op.hue,
            sat: op.sat,
            bright: op.bright,
            _pad2: [0.0],
            tint_b: op.tint_b,
            // Nenhum op lê a fonte — o ingest já a trouxe para o espaço de trabalho.
            src_org: [0, 0],
            stop_count: op.stop_count,
            src_premul: 0,
            stops: op.stops,
            stop_pos: op.stop_pos,
            aux: [0; 4],
            ramp: [0; 4],
        }
    }
}

impl Globals {
    /// Os globals do INGEST e do RESOLVE: só as dimensões — nenhum lê tint, sigma ou banda.
    pub(crate) fn bordas(dims: [u32; 2]) -> Self {
        Self {
            dims,
            half: 1,
            kind: 0,
            tint: [0.0; 4],
            inv_two_sigma2: 1.0,
            opacity: 1.0,
            off_x: 0.0,
            off_y: 0.0,
            jump: 0,
            band: 1.0,
            n_segs: 0,
            blend: 0,
            noise_scale: 1.0,
            octaves: 1,
            seed: 0,
            mode: 0,
            org: [0.0, 0.0],
            grow_px: 0.0,
            org_map: 0,
            hue: 0.0,
            sat: 0.0,
            bright: 0.0,
            _pad2: [0.0],
            tint_b: [0.0; 4],
            src_org: [0, 0],
            stop_count: 0,
            src_premul: 0,
            stops: [[0.0; 4]; 8],
            stop_pos: [[0.0; 4]; 2],
            aux: [0; 4],
            ramp: [0; 4],
        }
    }
}

/// Os segmentos da silhueta, num buffer grow-only. ⚠️ Ele NUNCA encolhe a zero: um `storage` de
/// tamanho 0 é binding inválido, e o layout exige a entrada mesmo nos passes que não a leem. Sem
/// geometria o shader olha `n_segs == 0` e cai no caminho da cobertura — o buffer fica intocado.
pub(crate) fn ensure_segments(
    gpu: &GpuContext,
    segs: &mut wgpu::Buffer,
    cap: &mut u64,
    geom: &[[f32; 4]],
) {
    if geom.is_empty() {
        return;
    }
    let need = std::mem::size_of_val(geom) as u64;
    if need > *cap {
        *cap = need.next_power_of_two();
        *segs = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-render fx_stack segments"),
            size: *cap,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
    }
    gpu.queue.write_buffer(segs, 0, bytemuck::cast_slice(geom));
}

/// **As rampas ASSADAS da corrida, uma atrás da outra**, num buffer grow-only (o mesmo regime dos
/// segmentos: nunca encolhe a zero). Devolve, por degrau, o `[base, comprimento, 0, 0]` do
/// [`Globals::ramp`]; sem rampa nenhuma o buffer fica intocado.
pub(crate) fn ensure_ramps(
    gpu: &GpuContext,
    buf: &mut wgpu::Buffer,
    cap: &mut u64,
    ops: &[crate::FxOpGpu],
) -> Vec<[u32; 4]> {
    let mut dados: Vec<[f32; 4]> = Vec::new();
    let onde = ops
        .iter()
        .map(|op| match op.ramp_lut.as_deref() {
            Some(lut) if lut.len() >= 2 => {
                let base = u32::try_from(dados.len()).unwrap_or(u32::MAX);
                dados.extend_from_slice(lut);
                [base, u32::try_from(lut.len()).unwrap_or(u32::MAX), 0, 0]
            }
            _ => [0; 4],
        })
        .collect();
    if dados.is_empty() {
        return onde;
    }
    let need = std::mem::size_of_val(dados.as_slice()) as u64;
    if need > *cap {
        *cap = need.next_power_of_two();
        *buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-render fx_stack ramps"),
            size: *cap,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
    }
    gpu.queue.write_buffer(buf, 0, bytemuck::cast_slice(&dados));
    onde
}

/// As texturas de trabalho, **grow-only**: uma cena com formas de tamanhos diferentes não paga uma
/// realocação por forma por frame. O shader escreve só `dims`, então uma textura maior é inofensiva.
pub(crate) fn ensure_work(gpu: &GpuContext, work: &mut Option<[Tex; 4]>, w: u32, h: u32) {
    if matches!(work, Some(t) if t[0].w >= w && t[0].h >= h) {
        return;
    }
    let (nw, nh) = match work {
        Some(t) => (t[0].w.max(w), t[0].h.max(h)),
        None => (w, h),
    };
    let f = wgpu::TextureFormat::Rgba16Float;
    *work = Some([
        make_tex(gpu, nw, nh, f),
        make_tex(gpu, nw, nh, f),
        make_tex(gpu, nw, nh, f),
        make_tex(gpu, nw, nh, f),
    ]);
}

/// O buffer dos globals de todos os passes, grow-only.
pub(crate) fn ensure_uniform(gpu: &GpuContext, buf: &mut wgpu::Buffer, cap: &mut u64, need: u64) {
    if need <= *cap {
        return;
    }
    *buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ph2d-render fx_stack globals"),
        size: need,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    *cap = need;
}

pub(crate) struct Tex {
    pub(crate) texture: wgpu::Texture,
    pub(crate) view: wgpu::TextureView,
    pub(crate) w: u32,
    pub(crate) h: u32,
}

pub(crate) fn make_tex(gpu: &GpuContext, w: u32, h: u32, format: wgpu::TextureFormat) -> Tex {
    let (w, h) = (w.max(1), h.max(1));
    let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ph2d-render fx_stack tex"),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::STORAGE_BINDING
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    Tex {
        texture,
        view,
        w,
        h,
    }
}

/// Cria uma textura de SAÍDA para o FX (o chamador a mantém viva por-forma — o Vello a copia no
/// render, DEPOIS do recook). `Rgba8Unorm` com os usos que o `register_texture` exige.
#[must_use]
pub fn make_output_texture(gpu: &GpuContext, w: u32, h: u32) -> wgpu::Texture {
    make_tex(gpu, w, h, wgpu::TextureFormat::Rgba8Unorm).texture
}

/// Escreve os globals do passe `slot` no blob, no offset dinâmico dele.
///
/// A pilha escreve os globals de TODOS os passes de uma vez e indexa por offset — um
/// `write_buffer` por passe antes de um único `submit` deixaria o ÚLTIMO a valer para todos.
pub(crate) fn write_at(blob: &mut [u8], slot: usize, g: &Globals) {
    let off = slot * UNIFORM_STRIDE as usize;
    let bytes = bytemuck::bytes_of(g);
    blob[off..off + bytes.len()].copy_from_slice(bytes);
}

pub(crate) fn dispatch(
    encoder: &mut wgpu::CommandEncoder,
    pipeline: &wgpu::ComputePipeline,
    bg: &wgpu::BindGroup,
    slot: u32,
    gx: u32,
    gy: u32,
) {
    let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
        label: Some("ph2d-render fx_stack pass"),
        timestamp_writes: None,
    });
    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, bg, &[slot * UNIFORM_STRIDE as u32]);
    pass.dispatch_workgroups(gx, gy, 1);
}
