//! GpuContext — owns wgpu Instance/Adapter/Device/Queue.
//!
//! Created once at app startup. Cloned/borrowed by the SurfaceContext
//! and (later) by render subsystems. Construction is the only place
//! we hit async wgpu APIs; we use `pollster::block_on` to keep the
//! rest of the core sync (per SKILL §10.1: "async morre na fronteira
//! da shell"; pollster is the approved sync runtime).

use std::sync::Arc;

#[derive(Debug)]
pub enum GpuError {
    NoAdapter(String),
    DeviceRequest(String),
}

impl std::fmt::Display for GpuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoAdapter(s) => write!(f, "no compatible GPU adapter found: {s}"),
            Self::DeviceRequest(s) => write!(f, "wgpu device request failed: {s}"),
        }
    }
}

impl std::error::Error for GpuError {}

/// Holder for the long-lived wgpu objects. Cheap to clone (`Arc`
/// internally on Device + Queue; Instance and Adapter are owned).
#[derive(Clone)]
pub struct GpuContext {
    pub instance: wgpu::Instance,
    pub adapter: Arc<wgpu::Adapter>,
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
}

impl GpuContext {
    /// Build a `GpuContext` compatible with the provided surface
    /// target. Must be called after the OS window exists (the surface
    /// target's window handle is needed during adapter selection so
    /// we pick an adapter that can render to it).
    pub fn new(
        instance: wgpu::Instance,
        compatible_surface: Option<&wgpu::Surface<'_>>,
    ) -> Result<Self, GpuError> {
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface,
            force_fallback_adapter: false,
        }))
        .map_err(|e| GpuError::NoAdapter(format!("{e:?}")))?;

        // Request the texture-compression families this adapter actually
        // advertises (KTX2 Fase 2 / W2.T4). wgpu only reports a feature in
        // `device.features()` if it was REQUESTED *and* granted — an
        // unrequested feature is invisible even on a capable adapter. So the
        // cooked-texture loader's `CompressionFeatureSet::best_tier()` (which
        // reads `device.features()`) would always fall to the uncompressed
        // RGBA8 `Constrained` tier unless we enable these here. Intersecting
        // the adapter's advertised features with the mask means we only ever
        // request what the adapter supports, so `request_device` cannot fail
        // on them (BC on desktop D3D12/Vulkan/Intel-Metal, ASTC/ETC2 on Apple
        // Silicon + mobile, none on a bare WebGPU adapter → RGBA8 floor).
        // Mirrors `CompressionFeatureSet::relevant_mask`.
        // TIMESTAMP_QUERY rides along when the adapter has it (it always does on
        // Metal/Vulkan/D3D12 desktops): the `pass_profiler` (PH2D_FLUID_PROFILE)
        // needs it to time GPU pass execution; granted-but-unused it costs nothing.
        let compression_features = adapter.features()
            & (wgpu::Features::TEXTURE_COMPRESSION_BC
                | wgpu::Features::TEXTURE_COMPRESSION_ASTC
                | wgpu::Features::TEXTURE_COMPRESSION_ETC2
                | wgpu::Features::TEXTURE_FORMAT_16BIT_NORM
                // ⭐⭐ PRIMITIVE_INDEX: e' ela que deixa um shader de FRAGMENTO
                // saber em que face esta', e e' disso que a TINTA FINA da
                // escultura vive (`ph2d-mesh-render/src/shaders/tinta.wgsl`).
                // Medida em 2026-09-20 como `true` nas TRES rotas desta maquina
                // (`ph2d-gpu --example o_que_a_placa_anuncia`).
                //
                // ⛔ Ela entra pela MESMA intersecao das outras: pedimos so' o
                // que o adaptador anuncia, logo o `request_device` nao pode
                // falhar por causa dela. Onde ela falta, a fonte do shader sai
                // SEM o bloco que a menciona (`ph2d_mesh_render::fonte`) e a
                // peca desenha com a cor por-vertice de sempre — a validacao de
                // um modulo WGSL e' tudo-ou-nada.
                | wgpu::Features::PRIMITIVE_INDEX
                | wgpu::Features::TIMESTAMP_QUERY);

        // Limits::default() (desktop tier) is required by Vello's
        // compute pipelines (M11 widget paint) — downlevel_defaults
        // caps storage texture bindings and workgroup sizes too low
        // for Vello to even compile its shaders. iPad/web shells will
        // need to revisit if/when they pick a downlevel adapter.
        //
        // **Large storage-buffer residency.** Some GPU-resident buffers (e.g. a
        // full-res 4K layer/effect working buffer) exceed the desktop-default
        // `max_storage_buffer_binding_size` (128 MiB). Raise the storage-buffer + total
        // buffer-size caps to the ADAPTER's advertised
        // max (always ≥ the default → a safe superset; `request_device` can't fail on it and
        // nothing that worked breaks — it only ALLOWS bigger buffers). The resident buffer then
        // allocates at full-res 4K where the hardware has the VRAM (Apple Silicon unified memory,
        // modern dGPUs). Smaller devices advertise less → the field stays low-res grid (canvas/4,
        // the production default) + the perf bench skips oversized configs (no silent cap).
        //
        // **Storage bindings per stage.** The default is 8 — the WebGPU
        // guaranteed minimum, which no desktop adapter is actually limited by.
        // A GPU-cook kernel binds one storage buffer per stream column it
        // touches, so a multi-input node reaches past 8 easily:
        // `motion.integrate` reads `P`/`vel`/`inv_mass` off `rest` and
        // `vel`/`sim_d`/`sim_t`/`accel` off last tick's state, and writes four
        // — 11 with every column present (ADR-0127 fatia 3). Raised to the
        // adapter's advertised max by the SAME argument as the sizes above: a
        // superset of the default, so `request_device` cannot fail on it and
        // nothing that worked breaks. A device that really does stop at 8
        // cannot run the integrator; the sequencer REFUSES such a kernel at cook
        // time (`GpuCookError::TooManyBindings`) and the caller falls back to the
        // CPU, rather than the pipeline blowing up at first dispatch.
        // **Vertex buffers per pipeline.** The default is 8 — again the WebGPU
        // guaranteed minimum, and again a floor no desktop adapter is limited
        // by: measured on this machine (2026-09-19, `vulkaninfo`), BOTH the
        // NVIDIA RTX 5060 Ti and the AMD RADV iGPU advertise
        // `maxVertexInputBindings = 32`, four times what the mesh pipeline
        // needs. The sculpt mesh feeds ONE buffer per per-vertex channel
        // (position, normal, mask, curvature, world curvature, thickness,
        // preview, AO — and, since the paint brushes, vertex COLOUR), which is
        // exactly 9 and lands one over the floor.
        //
        // Raised to the adapter's advertised max by the SAME argument as the
        // three below: a superset of the default, so `request_device` cannot
        // fail on it and nothing that worked breaks. ⚠️ Unlike the storage
        // bindings, there is no graceful per-kernel refusal here — a device
        // that really does stop at 8 cannot build the mesh pipeline at all. It
        // does not exist among the adapters this app ships to (desktop
        // Vulkan/Metal/D3D12 all advertise ≥ 16); a downlevel/WebGPU-floor
        // adapter would have to pack two scalar channels into one `vec2`
        // buffer, and the pair that costs nothing to pack is the curvature one
        // (both derived, both uploaded in the same call, full and partial).
        let adapter_limits = adapter.limits();
        let mut required_limits = wgpu::Limits::default();
        required_limits.max_vertex_buffers = required_limits
            .max_vertex_buffers
            .max(adapter_limits.max_vertex_buffers);
        required_limits.max_storage_buffer_binding_size = required_limits
            .max_storage_buffer_binding_size
            .max(adapter_limits.max_storage_buffer_binding_size);
        required_limits.max_buffer_size = required_limits
            .max_buffer_size
            .max(adapter_limits.max_buffer_size);
        required_limits.max_storage_buffers_per_shader_stage = required_limits
            .max_storage_buffers_per_shader_stage
            .max(adapter_limits.max_storage_buffers_per_shader_stage);
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("ph2d-gpu device"),
            required_features: compression_features,
            required_limits,
            experimental_features: wgpu::ExperimentalFeatures::default(),
            memory_hints: wgpu::MemoryHints::Performance,
            trace: wgpu::Trace::Off,
        }))
        .map_err(|e| GpuError::DeviceRequest(format!("{e:?}")))?;

        relata_erros_do_dispositivo(&device);

        Ok(Self {
            instance,
            adapter: Arc::new(adapter),
            device: Arc::new(device),
            queue: Arc::new(queue),
        })
    }

    /// Default Instance with PRIMARY backends (Metal/Vulkan/D3D12/WebGPU).
    pub fn default_instance() -> wgpu::Instance {
        wgpu::Instance::default()
    }
}

/// ⭐⭐⭐ **O DISPOSITIVO PASSA A DIZER QUANDO SE PARTE** — sem isto ele falha em SILÊNCIO.
///
/// ⛔⛔ **Report do dono, 2026-09-21: *«tela fica em branco ao maximizar»*, e a janela INTEIRA
/// (painéis, menus e a barra de baixo incluídos) fica vazia e **não recupera até fechar o app**.**
/// Essa assinatura — tudo desaparece de uma vez e para sempre — é a de um erro do DISPOSITIVO, e
/// até aqui o produto **não registava um único observador deles**: o `on_uncaptured_error` só
/// existia num teste de perfil (`ph2d-gpu/tests/it/pass_profiler_gpu.rs`). ⇒ o app entrava no
/// estado partido e **não tinha como o dizer**, nem ao dono nem ao terminal.
///
/// ⚠️ **Isto não é a cura do defeito — é o instrumento que permite achá-lo.** A máquina de
/// recuperação do `acquire` ([`crate::surface`]) trata `Lost`, `Outdated` e `Timeout` e é sólida;
/// o que não existia era voz para tudo o que acontece FORA dela (uma alocação recusada, um erro de
/// validação num submit, um device perdido). *Um app que fica em branco sem uma linha no terminal
/// obriga quem o investiga a adivinhar, e foi exactamente onde esta caça começou.*
///
/// ⚠️ **Ele não pode INUNDAR:** um dispositivo perdido devolve o mesmo erro em todo submit, ou
/// seja `60` linhas por segundo. As três primeiras saem inteiras — é a que interessa — e a partir
/// daí só as potências de dez, **com a contagem**, que é o que distingue *«aconteceu uma vez»* de
/// *«está a acontecer em todo quadro»*.
fn relata_erros_do_dispositivo(device: &wgpu::Device) {
    use std::sync::atomic::{AtomicU64, Ordering};
    static VISTOS: AtomicU64 = AtomicU64::new(0);
    device.on_uncaptured_error(std::sync::Arc::new(|e: wgpu::Error| {
        let n = VISTOS.fetch_add(1, Ordering::Relaxed) + 1;
        if n <= 3 || n.e_potencia_de_dez() {
            eprintln!(
                "[gpu-erro] #{n} — o dispositivo recusou trabalho. Se o ecra' ficou vazio e nao \
                 volta, e' isto:\n[gpu-erro] {e}"
            );
        }
    }));
}

/// `n` é uma potência de dez — a escada de silêncio do [`relata_erros_do_dispositivo`].
///
/// ⚠️ Escrita por extenso porque a `u64` não a tem, e **sem `f64::log10`**: a partir de `10^15` o
/// `f64` deixa de representar cada inteiro, e uma escada de log que erre num extremo cala
/// exactamente o caso que ela existe para contar.
trait PotenciaDeDez {
    fn e_potencia_de_dez(self) -> bool;
}

impl PotenciaDeDez for u64 {
    fn e_potencia_de_dez(self) -> bool {
        let mut p: u64 = 1;
        loop {
            if p == self {
                return true;
            }
            match p.checked_mul(10) {
                Some(maior) if maior <= self => p = maior,
                _ => return false,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression guard (W2.T4): every texture-compression family the adapter
    /// advertises MUST be granted on the created device, or the cooked-texture
    /// loader's `best_tier()` silently falls to the RGBA8 floor on capable
    /// hardware. `required_features: Features::empty()` (the pre-W2.T4 default)
    /// fails this. #[ignore] — needs a real adapter (no GPU on CI).
    #[test]
    #[ignore = "requires a GPU adapter (no GPU on CI); run with --ignored on a dev machine"]
    fn device_enables_every_adapter_advertised_compression_feature() {
        let mask = wgpu::Features::TEXTURE_COMPRESSION_BC
            | wgpu::Features::TEXTURE_COMPRESSION_ASTC
            | wgpu::Features::TEXTURE_COMPRESSION_ETC2
            | wgpu::Features::TEXTURE_FORMAT_16BIT_NORM;
        let Ok(gpu) = GpuContext::new(GpuContext::default_instance(), None) else {
            return; // no adapter on this machine — nothing to assert.
        };
        let adapter_caps = gpu.adapter.features() & mask;
        let device_caps = gpu.device.features() & mask;
        assert_eq!(
            device_caps, adapter_caps,
            "device must grant every adapter-advertised compression family \
             (adapter={adapter_caps:?}, device={device_caps:?}) — else best_tier() \
             silently picks RGBA8 on capable hardware"
        );
        // On any real desktop / Apple-Silicon adapter at least ONE compression
        // family is present; a bare WebGPU adapter legitimately has none.
        eprintln!("compression features granted on this device: {device_caps:?}");
    }
}

#[cfg(test)]
mod voz_do_dispositivo_tests {
    use super::PotenciaDeDez;

    /// ⭐ A escada de silêncio do [`super::relata_erros_do_dispositivo`], afirmada nos dois lados.
    ///
    /// ⚠️ **As duas metades são precisas.** Sem a positiva a escada pode nunca disparar e o
    /// relatório cala-se para sempre depois da 3.ª linha; sem a negativa ela pode disparar SEMPRE,
    /// e um dispositivo perdido enche o terminal a `60` linhas por segundo — o que esconde
    /// exactamente a 1.ª linha, que é a que diz a causa.
    #[test]
    fn a_escada_do_relatorio_e_so_as_potencias_de_dez() {
        for p in [1u64, 10, 100, 1_000, 10_000, 1_000_000_000_000_000_000] {
            assert!(p.e_potencia_de_dez(), "{p} E' uma potencia de dez");
        }
        for n in [0u64, 2, 9, 11, 99, 101, 999, 1_001, u64::MAX] {
            assert!(!n.e_potencia_de_dez(), "{n} NAO e' uma potencia de dez");
        }
        // ⚠️ O caso que uma escada por `f64::log10` erraria: acima de `2^53` o `f64` deixa de
        // representar cada inteiro, e `10^19` não cabe num `u64` (o `checked_mul` tem de parar).
        assert!(!(u64::MAX - 1).e_potencia_de_dez());
        assert!(10_000_000_000_000_000_000u64.e_potencia_de_dez());
    }

    /// ⛔⛔ **O produto REGISTA o observador de erros do dispositivo.**
    ///
    /// Sem isto o app entra no estado que o dono reportou em 2026-09-21 — a janela inteira vazia,
    /// sem recuperar — e **não diz uma palavra**. O gate não pode criar um device (precisaria de
    /// adaptador, logo seria `#[ignore]` e o CI nunca o correria), então ele afirma a FIAÇÃO: quem
    /// constrói o [`super::GpuContext`] chama o relator.
    ///
    /// ⚠️ **A agulha é montada em runtime**, nunca escrita como literal: *um censo textual que se
    /// lê a si mesmo encontra sempre o que procura* (a armadilha que a `line/sculpt3d` registou ao
    /// escrever um gate trivialmente verdadeiro). Aqui o texto lido é o do PRODUTO e o gate vive
    /// noutro módulo, mas a regra vale na mesma — o `concat!` garante-o para quem os juntar.
    #[test]
    fn quem_cria_o_dispositivo_liga_a_voz_dele() {
        // ⛔ Sem os comentários (auditoria do fecho, 2026-09-24): o doc do relator cita o
        // `on_uncaptured_error` pelo nome, logo contra o ficheiro inteiro a 2.ª metade ficava
        // verde com o registo APAGADO.
        let fonte: String = include_str!("context.rs")
            .lines()
            .filter(|l| !l.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n");
        let chamada = concat!("relata_erros_do_", "dispositivo(&device);");
        assert!(
            fonte.contains(chamada),
            "o construtor do GpuContext tem de chamar o relator logo apos o request_device -- \
             sem ele um erro de dispositivo deixa a janela vazia EM SILENCIO"
        );
        let registo = concat!("on_uncaptured", "_error");
        assert!(
            fonte.contains(registo),
            "o relator tem de registar o observador do wgpu"
        );
    }
}
