//! ⭐⭐⭐ **A MARCHA NO DISPOSITIVO** — o G-buffer do quadro, feito na GPU.
//!
//! # O que vai e o que fica
//!
//! Medido (`docs/Render3d/05` §34), o quadro assente a `1920×1080` reparte-se assim:
//!
//! | | CPU | |
//! |---|---:|---|
//! | traçado | `30,04 ms` | ⇒ **vai** |
//! | sombra directa | `61,73 ms` | ⇒ **vai** |
//! | **pintura** | **`10,87 ms`** | ⇒ **FICA** |
//!
//! ⭐⭐ **A pintura fica, e isso é a decisão importante.** Eu supunha que ela custava `~45 ms` e
//! teria escrito o material em WGSL por causa disso — uma **segunda** implementação do OpenPBR,
//! com tudo o que este repositório sabe sobre duas respostas para a mesma pergunta. Medida, ela
//! custa `10,87 ms` e cabe. *A lei do material continua a viver num sítio só.*
//!
//! # ⚠️ A câmera é a ÚNICA coisa escrita duas vezes, e é deliberado
//!
//! O [`ph2d_field_render::march`] avisa por escrito contra *«duas respostas para «que raio sai
//! daqui?»»*. Mandar os raios prontos seriam `50 MB` por quadro, logo o WGSL reconstrói o
//! `ray_at_plane`. ⇒ ela nasce **com o gate de paridade em cima**: qualquer divergência de câmera
//! move o ponto de acerto em unidades de mundo, e o gate mede exactamente isso.

use ph2d_field_eval::wgsl::TapeWgsl;

/// O que a marcha do dispositivo devolve, por pixel.
pub struct DeviceGbuffer {
    pub width: u32,
    pub height: u32,
    /// `t` do acerto, ou negativo quando o raio não acertou.
    pub t: Vec<f32>,
    /// A normal em espaço de **VISTA** — a mesma convenção do G-buffer da CPU.
    pub normal: Vec<[f32; 3]>,
    /// ⭐ Quanto da lâmpada chega a cada pixel — a [`ph2d_field_render::Shadows`] da CPU.
    pub shadow: Vec<f32>,
    /// ⭐ Quanto do céu chega a cada pixel — a oclusão.
    pub ambient: Vec<f32>,
    /// ⭐⭐⭐ **Os pixels de BORDA, re-amostrados no padrão 4-rook** — a `Gbuffer::edges` da CPU.
    ///
    /// ⚠️ Sem eles a silhueta sai serrilhada, e ligar o dispositivo ao produto seria trocar um
    /// defeito por outro. *É por isso que eles vêm na mesma passagem e não «depois».*
    pub edges: Vec<DeviceEdge>,
}

/// Um pixel de borda com as quatro amostras — o espelho da `ph2d_field_render::EdgePixel`.
#[derive(Clone, Copy, Debug)]
pub struct DeviceEdge {
    pub pixel: u32,
    pub hit: [bool; 4],
    pub normal: [[f32; 3]; 4],
}

impl DeviceGbuffer {
    #[must_use]
    pub fn hit(&self, i: usize) -> bool {
        self.t.get(i).is_some_and(|t| *t >= 0.0)
    }
}

/// Tudo o que a marcha precisa de saber e que **não** sai da fita — os mesmos números que a
/// [`ph2d_field_render::Scene`] carrega.
#[derive(Clone, Copy, Debug)]
pub struct MarchSetup {
    pub half_extent: f32,
    /// `min(w, h) * 0.5` — o `half` do [`ph2d_field_render::Screen`].
    pub half_px: f32,
    pub target: [f32; 3],
    pub right: [f32; 3],
    pub up: [f32; 3],
    /// O `toward_eye` da base. ⚠️ A direcção do raio na paralela é **`-fwd`**.
    pub fwd: [f32; 3],
    /// `ORTHO_START`, ou a distância ao olho quando há lente.
    pub ortho_start: f32,
    /// `0` = paralela. Caso contrário, a distância do olho ao alvo.
    pub eye_distance: f32,
    pub hit_eps: f32,
    pub normal_eps: f32,
    pub step: f32,
    pub budget: u32,
    pub t_max: f32,
    /// A lâmpada, no MUNDO. A sombra usa a mesma cerca da CPU: a distância à luz, cortada na saída
    /// da bola que contém a peça.
    pub lamp: [f32; 3],
    /// O raio da bola que contém a peça, e o centro dela.
    pub ball_center: [f32; 3],
    pub ball_radius: f32,
    /// ⭐ Quantos raios de oclusão por pixel — o [`ph2d_field_render::OCCLUSION_PASSES`].
    pub ao_rays: u32,
    /// O alcance da oclusão em unidades de mundo.
    pub ao_reach: f32,
    /// O cosseno abaixo do qual duas normais vizinhas são ARESTA — o `EDGE_COS` da CPU.
    /// ⭐⭐⭐ **A bandeira da W73 — *grosso a mexer, nítido ao assentar*.**
    ///
    /// `false` **salta o segundo despacho inteiro** (a borda re-amostrada) e devolve a lista de
    /// bordas vazia, que é exactamente o que o [`ph2d_field_render::trace_cancellable`] faz na CPU
    /// com o mesmo `antialias`. ⛔ Sem isto, mandar o quadro de MOVIMENTO ao dispositivo punha-o a
    /// pagar um passe que a lei do módulo manda não pagar — *dois motores, uma lei*.
    pub antialias: bool,
    pub edge_cos: f32,
}

const MOLDE: &str = r"
struct Setup {
    w: u32, h: u32, budget: u32, ao_rays: u32,
    half_extent: f32, half_px: f32, ortho_start: f32, eye_distance: f32,
    hit_eps: f32, normal_eps: f32, step: f32, t_max: f32,
    ball_radius: f32, ao_reach: f32, edge_cos: f32, _p1: f32,
    alvo: vec3<f32>, right: vec3<f32>, up: vec3<f32>, fwd: vec3<f32>,
    lamp: vec3<f32>, ball_center: vec3<f32>,
};
@group(0) @binding(0) var<uniform> s: Setup;
@group(0) @binding(1) var<storage, read> k: array<f32>;
@group(0) @binding(2) var<storage, read_write> centro: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read_write> luz: array<vec2<f32>>;
@group(0) @binding(4) var<storage, read_write> conta: atomic<u32>;
@group(0) @binding(5) var<storage, read_write> borda: array<vec4<f32>>;
{FIELD}

// ⚠️ **A MESMA lei de marcha da CPU**, e ela é uma função porque as quatro amostras do
// anti-serrilhado a repetem: uma segunda cópia seria a segunda resposta à mesma pergunta.
fn raio(px: f32, py: f32) -> vec2<f32> {
    let u = (px - f32(s.w) * 0.5) / s.half_px * s.half_extent;
    let v = -(py - f32(s.h) * 0.5) / s.half_px * s.half_extent;
    return vec2<f32>(u, v);
}
struct Raio { o: vec3<f32>, d: vec3<f32> };
fn ray_at_plane(uv: vec2<f32>) -> Raio {
    let on_plane = s.alvo + s.right * uv.x + s.up * uv.y;
    var r: Raio;
    if (s.eye_distance == 0.0) {
        r.o = on_plane + s.fwd * s.ortho_start;
        r.d = -s.fwd;
    } else {
        let eye = s.alvo + s.fwd * s.eye_distance;
        let d = on_plane - eye;
        let len = length(d);
        r.o = eye;
        if (len <= 0.0) { r.d = -s.fwd; } else { r.d = d / len; }
    }
    return r;
}
// Devolve `vec4(t, normal em VISTA)`, com `t < 0` quando não acerta.
fn marcha(r: Raio) -> vec4<f32> {
    var t = 0.0;
    var acertou = false;
    for (var n: u32 = 0u; n < s.budget; n = n + 1u) {
        let d = field(r.o + r.d * t);
        if (d < s.hit_eps) { acertou = true; break; }
        t = t + d * s.step;
        if (t >= s.t_max) { break; }
    }
    if (!acertou) { return vec4<f32>(-1.0, 0.0, 0.0, 0.0); }
    let p = r.o + r.d * t;
    let e = s.normal_eps;
    let o0 = vec3<f32>( 1.0, -1.0, -1.0);
    let o1 = vec3<f32>(-1.0, -1.0,  1.0);
    let o2 = vec3<f32>(-1.0,  1.0, -1.0);
    let o3 = vec3<f32>( 1.0,  1.0,  1.0);
    let world = o0 * field(p + o0 * e) + o1 * field(p + o1 * e)
              + o2 * field(p + o2 * e) + o3 * field(p + o3 * e);
    let len = length(world);
    if (len <= 0.0) { return vec4<f32>(-1.0, 0.0, 0.0, 0.0); }
    let nrm = world / len;
    return vec4<f32>(t, dot(nrm, s.right), dot(nrm, s.up), dot(nrm, s.fwd));
}

// ⭐ **A MARCHA DE VISIBILIDADE** — a da sombra e a da oclusão são a mesma, e diferem só na cerca
// e na dureza. `INFINITY` para a oclusão (a pergunta é binária); `8` para a sombra (penumbra).
fn visivel(origem: vec3<f32>, dir: vec3<f32>, t_max: f32, dureza: f32) -> f32 {
    var vis = 1.0;
    var t = s.hit_eps * 4.0;
    for (var n: u32 = 0u; n < s.budget; n = n + 1u) {
        let d = field(origem + dir * t);
        if (d < s.hit_eps) { return 0.0; }
        vis = min(vis, dureza * d / t);
        t = t + d * s.step;
        if (t >= t_max) { break; }
    }
    return vis;
}

// ⭐⭐⭐ A direcção `k` do conjunto de cones — o `ph2d_field_render::cone_dir`, linha a linha.
// Reticulado de Fibonacci esférico em coordenadas de MUNDO: nem o pixel nem a câmera entram.
fn direccao_do_cone(k: u32, total: u32) -> vec3<f32> {
    let n = f32(max(total, 1u));
    let ki = f32(k);
    let z = 1.0 - (2.0 * ki + 1.0) / n;
    let r = sqrt(max(1.0 - z * z, 0.0));
    let phi = 6.283185307 * fract(ki * 0.618034);
    return vec3<f32>(r * cos(phi), r * sin(phi), z);
}

// A saída da bola, que é o DOMÍNIO da pergunta — ver `ph2d_field_render::shadow`.
fn cerca_da_bola(p: vec3<f32>, dir: vec3<f32>, ate: f32) -> f32 {
    let oc = p - s.ball_center;
    let b = dot(oc, dir);
    let c = dot(oc, oc) - s.ball_radius * s.ball_radius;
    let disc = b * b - c;
    if (disc <= 0.0) { return 0.0; }
    return clamp(-b + sqrt(disc), 0.0, ate);
}

@compute @workgroup_size(8, 8, 1)
fn centro_e_luz(@builtin(global_invocation_id) g: vec3<u32>) {
    if (g.x >= s.w || g.y >= s.h) { return; }
    let i = g.y * s.w + g.x;
    let r = ray_at_plane(raio(f32(g.x) + 0.5, f32(g.y) + 0.5));
    let c = marcha(r);
    centro[i] = c;
    if (c.x < 0.0) { luz[i] = vec2<f32>(1.0, 1.0); return; }

    let p = r.o + r.d * c.x;
    // A normal volta ao MUNDO — a base é ortonormal, logo a transposta é a inversa.
    let n = s.right * c.y + s.up * c.z + s.fwd * c.w;
    let erguido = p + n * (s.hit_eps * 4.0);

    // A SOMBRA: só quem VÊ a luz recebe raio.
    var sombra = 1.0;
    let d = s.lamp - p;
    let dist = length(d);
    if (dist > 1e-6) {
        let dir = d / dist;
        if (dot(n, dir) > 0.0) {
            sombra = visivel(erguido, dir, cerca_da_bola(erguido, dir, dist), 8.0);
        }
    }

    // ⭐⭐⭐ **A OCLUSÃO POR CONES** — `ao_rays` direcções FIXAS de mundo, pesadas pelo cosseno.
    //
    // A dureza de cada cone é `1/(n·d)`: é o cone que ROÇA o plano tangente, e é ele que faz um
    // corpo CONVEXO ler exactamente `1,0`. Ver `ph2d_field_render::cone_dir` para o porquê de o
    // conjunto ser de MUNDO e não de um referencial tangente.
    var ceu = 1.0;
    if (s.ao_rays > 0u) {
        var soma = 0.0;
        var peso = 0.0;
        for (var j: u32 = 0u; j < s.ao_rays; j = j + 1u) {
            let dd = direccao_do_cone(j, s.ao_rays);
            let c = dot(n, dd);
            if (c <= 0.0) { continue; }
            peso = peso + c;
            let ate = min(s.ao_reach, cerca_da_bola(erguido, dd, s.ao_reach));
            soma = soma + c * visivel(erguido, dd, ate, 1.0 / c);
        }
        if (peso > 0.0) { ceu = soma / peso; }
    }
    luz[i] = vec2<f32>(sombra, ceu);
}

// ⭐⭐⭐ **A SEGUNDA PASSAGEM: a borda re-amostrada.** Ela precisa dos VIZINHOS, logo não pode
// viver na primeira — e é por isso que são dois despachos e não um.
@compute @workgroup_size(8, 8, 1)
fn bordas(@builtin(global_invocation_id) g: vec3<u32>) {
    if (g.x >= s.w || g.y >= s.h) { return; }
    let i = g.y * s.w + g.x;
    let c = centro[i];
    // A MESMA regra da CPU: direita e baixo, e os DOIS pixels ficam marcados.
    var e = false;
    if (g.x + 1u < s.w) { e = e || difere(i, i + 1u); }
    if (g.y + 1u < s.h) { e = e || difere(i, i + s.w); }
    if (g.x > 0u) { e = e || difere(i - 1u, i); }
    if (g.y > 0u) { e = e || difere(i - s.w, i); }
    if (!e) { return; }

    let slot = atomicAdd(&conta, 1u);
    // ⚠️ Um lote cheio **descarta** em vez de escrever fora — a borda perde-se, o quadro não.
    if (slot * 5u + 4u >= arrayLength(&borda)) { return; }
    borda[slot * 5u] = vec4<f32>(bitcast<f32>(i), 0.0, 0.0, 0.0);
    // O padrão 4-rook (RGSS), o mesmo da CPU.
    let rook = array<vec2<f32>, 4>(
        vec2<f32>(0.125, 0.625), vec2<f32>(0.375, 0.125),
        vec2<f32>(0.625, 0.875), vec2<f32>(0.875, 0.375));
    for (var j = 0u; j < 4u; j = j + 1u) {
        let o = rook[j];
        borda[slot * 5u + 1u + j] = marcha(ray_at_plane(raio(f32(g.x) + o.x, f32(g.y) + o.y)));
    }
}

fn difere(a: u32, b: u32) -> bool {
    let ca = centro[a];
    let cb = centro[b];
    let ha = ca.x >= 0.0;
    let hb = cb.x >= 0.0;
    if (ha != hb) { return true; }
    if (!ha) { return false; }
    return dot(ca.yzw, cb.yzw) < s.edge_cos;
}
";

/// ⭐⭐⭐ **O TRAÇADOR: o dispositivo, o cache de pipelines e o layout, vivos entre quadros.**
///
/// ⛔⛔ **Ele existe porque a 1.ª sonda mediu a coisa errada.** A [`march`] abre o adaptador, pede
/// o dispositivo e **compila o shader** a cada chamada — e o relógio dela leu **`130 ms` a
/// `640×360`**, isto é, *mais lento que a CPU*. O que ela media era a abertura do dispositivo e a
/// compilação (`6`–`49 ms`, §33), não o quadro.
///
/// ⚠️ *Uma sonda cujo próprio doc diz «é a forma de sonda» ainda assim foi usada como relógio* — e
/// o número saiu convincente e ao contrário.
pub struct Tracer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    cache: crate::FieldPipelines,
}

impl Tracer {
    /// `None` sem adaptador.
    #[must_use]
    pub fn new() -> Option<Self> {
        let instance = wgpu::Instance::default();
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        }))
        .ok()?;
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("marcha do campo"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            experimental_features: wgpu::ExperimentalFeatures::default(),
            memory_hints: wgpu::MemoryHints::Performance,
            trace: wgpu::Trace::Off,
        }))
        .ok()?;
        Some(Self {
            device,
            queue,
            cache: crate::FieldPipelines::new(),
        })
    }

    /// Quantos pipelines já foram compilados — o número que um gate de *«um arrasto não
    /// recompila»* observa.
    #[must_use]
    pub fn compiled(&self) -> usize {
        self.cache.compiled()
    }

    /// ⭐ **Um quadro.** O shader compila-se na primeira estrutura e fica.
    pub fn frame(
        &mut self,
        fita: &TapeWgsl,
        setup: MarchSetup,
        width: u32,
        height: u32,
    ) -> DeviceGbuffer {
        marcha_com(
            &self.device,
            &self.queue,
            &mut self.cache,
            fita,
            setup,
            width,
            height,
        )
    }
    /// O dispositivo e a fila — para quem precisa de despachar **outro** passe sobre o MESMO
    /// dispositivo (o arnês de paridade do material, e o passe de sombreamento).
    ///
    /// ⚠️ **Abrir um segundo dispositivo custaria mais do que o trabalho** (medido: `130 ms` para
    /// abrir e compilar, contra `13` da CPU inteira) — e um `Device` não fala com buffers de outro.
    #[must_use]
    pub fn parts(&self) -> (&wgpu::Device, &wgpu::Queue) {
        (&self.device, &self.queue)
    }

    /// ⭐ **Quanto custa TRAZER `bytes` de volta** — a fase que o `submit` do quadro esconde.
    ///
    /// ⚠️ **É diagnóstico, e não o caminho do produto:** o quadro copia e lê no MESMO `submit` que
    /// despacha o compute, logo um relógio à volta dele mede os dois juntos. Esta porta mede só a
    /// travessia, sobre o mesmo volume de bytes, e é assim que a sonda `frame_budget` separa as
    /// fases sem tocar no que shipa.
    #[must_use]
    pub fn mede_leitura(&self, bytes: u64, corridas: usize) -> f64 {
        let bytes = bytes.max(16);
        let origem = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("origem"),
            size: bytes,
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let mut melhor = f64::INFINITY;
        for _ in 0..corridas {
            let t = std::time::Instant::now();
            let destino = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("leitura"),
                size: bytes,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let mut enc = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            enc.copy_buffer_to_buffer(&origem, 0, &destino, 0, bytes);
            self.queue.submit([enc.finish()]);
            destino.slice(..).map_async(wgpu::MapMode::Read, |_| {});
            self.device.poll(wgpu::PollType::wait_indefinitely()).ok();
            let dados = destino.slice(..).get_mapped_range();
            std::hint::black_box(dados[0]);
            drop(dados);
            melhor = melhor.min(t.elapsed().as_secs_f64() * 1e3);
        }
        melhor
    }
}

/// A forma de **SONDA**: abre o dispositivo, corre um quadro e fecha.
///
/// ⛔ **Não a use para medir tempo** — ver [`Tracer`].
#[must_use]
pub fn march(fita: &TapeWgsl, setup: MarchSetup, width: u32, height: u32) -> Option<DeviceGbuffer> {
    Some(Tracer::new()?.frame(fita, setup, width, height))
}

#[allow(clippy::too_many_lines, clippy::too_many_arguments)] // seis bindings e dois despachos
fn marcha_com(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    cache: &mut crate::FieldPipelines,
    fita: &TapeWgsl,
    setup: MarchSetup,
    width: u32,
    height: u32,
) -> DeviceGbuffer {
    // ⚠️ **O layout é EXPLÍCITO** — ver [`crate::FieldPipelines::entry_with_layout`]: as duas
    // entradas usam bindings diferentes, e um layout por entrada recusa o grupo de seis.
    let uniforme = |b: u32| wgpu::BindGroupLayoutEntry {
        binding: b,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    };
    let armazem = |b: u32, so_leitura: bool| wgpu::BindGroupLayoutEntry {
        binding: b,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage {
                read_only: so_leitura,
            },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    };
    let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("campo"),
        entries: &[
            uniforme(0),
            armazem(1, true),
            armazem(2, false),
            armazem(3, false),
            armazem(4, false),
            armazem(5, false),
        ],
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("campo"),
        bind_group_layouts: &[Some(&bgl)],
        immediate_size: 0,
    });

    let p_centro = cache
        .entry_with_layout(device, MOLDE, fita, "centro_e_luz", Some(&layout))
        .clone();
    // ⚠️ **Compilar é o caro** — o pipeline da borda só nasce quando ela vai de facto correr.
    let p_bordas = setup.antialias.then(|| {
        cache
            .entry_with_layout(device, MOLDE, fita, "bordas", Some(&layout))
            .clone()
    });

    // O uniforme, campo a campo — a mesma ordem da `struct Setup`. ⚠️ Um `vec3` alinha a 16 B.
    let mut u: Vec<u8> = Vec::with_capacity(160);
    for v in [width, height, setup.budget, setup.ao_rays] {
        u.extend_from_slice(&v.to_le_bytes());
    }
    for f in [
        setup.half_extent,
        setup.half_px,
        setup.ortho_start,
        setup.eye_distance,
        setup.hit_eps,
        setup.normal_eps,
        setup.step,
        setup.t_max,
        setup.ball_radius,
        setup.ao_reach,
        setup.edge_cos,
        0.0,
    ] {
        u.extend_from_slice(&f.to_le_bytes());
    }
    for v in [
        setup.target,
        setup.right,
        setup.up,
        setup.fwd,
        setup.lamp,
        setup.ball_center,
    ] {
        for f in v {
            u.extend_from_slice(&f.to_le_bytes());
        }
        u.extend_from_slice(&0f32.to_le_bytes()); // o padding do `vec3`
    }

    use wgpu::util::DeviceExt;
    let ub = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("setup"),
        contents: &u,
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let consts = if fita.consts.is_empty() {
        vec![0.0f32]
    } else {
        fita.consts.clone()
    };
    let mut kb_bytes = Vec::with_capacity(consts.len() * 4);
    for c in &consts {
        kb_bytes.extend_from_slice(&c.to_le_bytes());
    }
    let kb = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("k"),
        contents: &kb_bytes,
        usage: wgpu::BufferUsages::STORAGE,
    });
    let n = u64::from(width) * u64::from(height);
    let storage = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC;
    let cria = |nome: &str, bytes: u64| {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(nome),
            size: bytes.max(16),
            usage: storage,
            mapped_at_creation: false,
        })
    };
    let b_centro = cria("centro", n * 16);
    let b_luz = cria("luz", n * 8);
    // ⛔⛔ **O TECTO da lista de bordas era `6 %` e ESTOUROU** — o gate da paridade apanhou-o: na
    // ROSCA a GPU devolveu exactamente `1 296` bordas, que **é** o tecto, contra `1 745` da CPU, e
    // a sobreposição das listas caiu para `72,6 %`.
    //
    // ⚠️ **O `0,5`–`1,2 %` que eu citei é da SILHUETA** (`docs/3DModeling/05`), e a borda deste
    // passe é silhueta **mais VINCO**: uma peça de ranhuras finas é quase toda vinco. Medido, a
    // rosca dá `8,4 %`. ⇒ `25 %`, que é três vezes o pior medido — e o custo é `20 B` por pixel
    // (`41 MB` a `1920×1080`), que a leitura já paga em `~2 ms`.
    //
    // ⚠️ *Um tecto derivado da grandeza ERRADA lê-se como generoso.*
    let max_bordas = (n / 4).max(1024);
    let b_borda = cria("bordas", max_bordas * 5 * 16);
    let b_conta = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("conta"),
        // ⚠️ Quatro palavras e não uma: a cópia de leitura alinha a `16 B`, e um buffer de `4`
        // seria lido fora dos limites.
        contents: &[0u8; 16],
        usage: storage,
    });

    let bind = |_p: &wgpu::ComputePipeline| {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: ub.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: kb.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: b_centro.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: b_luz.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: b_conta.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: b_borda.as_entire_binding(),
                },
            ],
        })
    };
    let bg_centro = bind(&p_centro);
    let bg_bordas = p_bordas.as_ref().map(&bind);

    let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    // ⚠️ **DOIS despachos, e a ordem é a lei**: a borda pergunta pelos VIZINHOS, logo o centro tem
    // de estar escrito para toda a imagem antes de ela correr.
    let despachos: Vec<(&wgpu::ComputePipeline, &wgpu::BindGroup)> =
        match (p_bordas.as_ref(), bg_bordas.as_ref()) {
            (Some(p), Some(bg)) => vec![(&p_centro, &bg_centro), (p, bg)],
            _ => vec![(&p_centro, &bg_centro)],
        };
    for (p, bg) in despachos {
        let mut cp = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });
        cp.set_pipeline(p);
        cp.set_bind_group(0, bg, &[]);
        cp.dispatch_workgroups(width.div_ceil(8), height.div_ceil(8), 1);
    }

    let ler = |enc: &mut wgpu::CommandEncoder, b: &wgpu::Buffer, bytes: u64| {
        let r = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("leitura"),
            size: bytes.max(16),
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        enc.copy_buffer_to_buffer(b, 0, &r, 0, bytes.max(16));
        r
    };
    let r_centro = ler(&mut enc, &b_centro, n * 16);
    let r_luz = ler(&mut enc, &b_luz, n * 8);
    let r_conta = ler(&mut enc, &b_conta, 16);
    queue.submit([enc.finish()]);

    for b in [&r_centro, &r_luz, &r_conta] {
        b.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    }
    device.poll(wgpu::PollType::wait_indefinitely()).ok();

    let d_centro = r_centro.slice(..).get_mapped_range();
    let d_luz = r_luz.slice(..).get_mapped_range();
    let d_conta = r_conta.slice(..).get_mapped_range();
    let quantas = u32::from_le_bytes([d_conta[0], d_conta[1], d_conta[2], d_conta[3]]) as u64;

    // ⛔⛔ **A LISTA DE BORDAS LÊ-SE PELO QUE FOI ESCRITO, e não pelo tecto** — e é a diferença
    // entre `35 ms` e o que a máquina de facto faz. O tecto é `25 %` dos pixels (`41 MB` a
    // `1920×1080`) e a ocupação real é `1`–`8 %`: copiar o tecto inteiro a cada quadro era
    // **quase metade** dos `90 MB` de leitura. ⇒ um segundo `submit`, que custa um ida-e-volta e
    // poupa dezenas de megabytes. *Um buffer dimensionado para o pior caso não se lê no pior caso.*
    // ⚠️ **Sem anti-serrilhado não há segunda travessia nenhuma** — nem o `submit`, nem o
    // `poll`, que é um ida-e-volta completo ao dispositivo por quadro.
    let usadas = if setup.antialias {
        quantas.min(max_bordas)
    } else {
        0
    };
    let r_borda = {
        let mut enc2 =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let r = ler(&mut enc2, &b_borda, usadas * 5 * 16);
        if usadas > 0 {
            queue.submit([enc2.finish()]);
            r.slice(..).map_async(wgpu::MapMode::Read, |_| {});
            device.poll(wgpu::PollType::wait_indefinitely()).ok();
        }
        r
    };
    let d_borda = if usadas > 0 {
        Some(r_borda.slice(..).get_mapped_range())
    } else {
        None
    };

    let f4 = |q: &[u8; 16], o: usize| f32::from_le_bytes([q[o], q[o + 1], q[o + 2], q[o + 3]]);
    let mut t = Vec::with_capacity(d_centro.len() / 16);
    let mut normal = Vec::with_capacity(t.capacity());
    for q in d_centro.as_chunks::<16>().0 {
        t.push(f4(q, 0));
        normal.push([f4(q, 4), f4(q, 8), f4(q, 12)]);
    }
    let mut shadow = Vec::with_capacity(t.len());
    let mut ambient = Vec::with_capacity(t.len());
    for q in d_luz.as_chunks::<8>().0 {
        shadow.push(f32::from_le_bytes([q[0], q[1], q[2], q[3]]));
        ambient.push(f32::from_le_bytes([q[4], q[5], q[6], q[7]]));
    }
    let vazio: [u8; 0] = [];
    let quads = d_borda.as_deref().unwrap_or(&vazio).as_chunks::<16>().0;
    #[allow(clippy::cast_possible_truncation)]
    let usadas = usadas as usize;
    let mut edges = Vec::with_capacity(usadas);
    for slot in 0..usadas.min(quads.len() / 5) {
        let cabeca = &quads[slot * 5];
        let pixel = u32::from_le_bytes([cabeca[0], cabeca[1], cabeca[2], cabeca[3]]);
        let mut hit = [false; 4];
        let mut nrm = [[0.0f32; 3]; 4];
        for j in 0..4 {
            let q = &quads[slot * 5 + 1 + j];
            hit[j] = f4(q, 0) >= 0.0;
            nrm[j] = [f4(q, 4), f4(q, 8), f4(q, 12)];
        }
        edges.push(DeviceEdge {
            pixel,
            hit,
            normal: nrm,
        });
    }
    // ⚠️ **Ordenada por pixel**, como a `Gbuffer::edges` da CPU promete — a ordem da lista aqui é
    // a de chegada dos workgroups, que é arbitrária.
    edges.sort_by_key(|e| e.pixel);

    drop(d_centro);
    drop(d_luz);
    drop(d_conta);
    drop(d_borda);

    DeviceGbuffer {
        width,
        height,
        t,
        normal,
        shadow,
        ambient,
        edges,
    }
}

impl DeviceGbuffer {
    /// ⭐⭐⭐ **O G-buffer do dispositivo no vocabulário da CPU** — para a pintura correr onde já
    /// corre, sem saber que a marcha mudou de sítio.
    ///
    /// ⚠️ **O `point` RECONSTRÓI-SE do `t`**, e é por isso que ele não atravessa o barramento: são
    /// mais `12 B` por pixel (`25 MB` a `1920×1080`) para uma conta que a CPU faz em microssegundos.
    /// *O que se lê de volta é o que não se pode derivar.*
    #[must_use]
    pub fn to_cpu(
        &self,
        cam: &ph2d_field_render::Orbit,
        screen: ph2d_field_render::Screen,
    ) -> (ph2d_field_render::Gbuffer, ph2d_field_render::Shadows) {
        let n = self.t.len();
        let mut hit = Vec::with_capacity(n);
        let mut point = Vec::with_capacity(n);
        let w = self.width as usize;
        // ⭐⭐⭐ **O PONTO reconstrói-se SEM normalizar a direcção** — ver
        // [`ph2d_field_render::Rays::point_at`] para a álgebra e a tabela. Medido a `1920×1080`
        // neste laço: **`37,87 → 5,75 ms`**, que era a maior fatia do quadro inteiro (`87,58`).
        let raios = cam.rays();
        for y in 0..self.height as usize {
            #[allow(clippy::cast_precision_loss)]
            let py = y as f32 + 0.5;
            for x in 0..w {
                let t = self.t[y * w + x];
                hit.push(t >= 0.0);
                #[allow(clippy::cast_precision_loss)]
                let (u, v) = screen.plane_at(x as f32 + 0.5, py);
                point.push(raios.point_at(u, v, t));
            }
        }
        let edges = self
            .edges
            .iter()
            .map(|e| ph2d_field_render::EdgePixel {
                pixel: e.pixel,
                hit: e.hit,
                normal: e.normal,
            })
            .collect();
        let g = ph2d_field_render::Gbuffer {
            width: self.width,
            height: self.height,
            hit,
            normal: self.normal.clone(),
            point,
            edges,
        };
        let mut sh = ph2d_field_render::Shadows::default();
        sh.set_lamp(0, self.shadow.clone());
        // ⚠️ **A suavização é aplicada AQUI**, como o refinamento da CPU a aplica no publicar — ela
        // faz parte do que a oclusão entrega, e não do que ela calcula.
        sh.set_ambient(ph2d_field_render::blur_occlusion(&g, &self.ambient));
        (g, sh)
    }
}
