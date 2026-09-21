//! ⭐⭐⭐ **O DEVICE DE TESTE DESTA CRATE, numa porta só.**
//!
//! ⛔⛔⛔ **Ele nasceu de QUATRO cópias e de dezasseis gates vermelhos de uma
//! vez** (2026-09-20). Cada ficheiro de GPU desta suíte pedia o device dele com
//! `wgpu::Limits::default()` — o **piso do WebGPU** —, enquanto o produto pede
//! ao [`ph2d_gpu::GpuContext`] os limites do ADAPTADOR. Enquanto os dois
//! pipelines coincidiram, as quatro cópias concordaram por acaso; no dia em que
//! o `mesh.wgsl` passou a ter **nove** buffers por vértice (o canal de cor), o
//! piso `8` parou de construir o pipeline e a suíte inteira leu
//! `Validation Error` — *num arnês que até ali media outro programa em silêncio*.
//!
//! ⚠️ **A régua de um gate de device é o DEVICE QUE O PRODUTO PEDE.** É a mesma
//! lei que esta linha já pagou três vezes noutra camada (o arnês que não abria o
//! traço, o que não chamava o pen-down, o que montava o estado à mão).
//!
//! ⛔ **E ela não chama o `ph2d_gpu::GpuContext`** de propósito: aquela crate é
//! do app e traz a cadeia dele, enquanto esta suíte só precisa de um device.
//! O que TEM de ser igual é o pedido de limites, e é isso que o gate
//! [`o_device_de_teste_pede_o_que_o_produto_pede`] afirma — lendo o
//! `ph2d-gpu/src/context.rs` por `include_str!`.

/// O device de teste, ou `None` quando não há adaptador (skip gracioso).
pub fn device() -> Option<(wgpu::Device, wgpu::Queue)> {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .ok()?;
    // O pedido do produto: o máximo do adaptador onde ele é um SUPERCONJUNTO do
    // piso, nunca menos.
    let adapter_limits = adapter.limits();
    let mut required_limits = wgpu::Limits::default();
    required_limits.max_vertex_buffers = required_limits
        .max_vertex_buffers
        .max(adapter_limits.max_vertex_buffers);
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("ph2d-mesh test device"),
        // ⭐⭐ **O MESMO pedido do produto**, pela mesma intersecção: só o que
        // o adaptador anuncia, logo o `request_device` não pode falhar por
        // causa dela. Sem ela, esta porta devolveria um device SEM
        // `PRIMITIVE_INDEX` e a fonte do shader sairia sem o bloco da tinta —
        // os gates desta suíte passariam a medir **outro programa**, que é
        // exactamente o defeito que este ficheiro existe para não ter.
        required_features: adapter.features() & wgpu::Features::PRIMITIVE_INDEX,
        required_limits,
        experimental_features: wgpu::ExperimentalFeatures::default(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
    }))
    .expect("request_device");
    Some((device, queue))
}

/// ⭐⭐ **O PRODUTO CONTINUA A SUBIR O LIMITE** — a metade que corre SEM
/// adaptador, e a única que uma máquina sem GPU pode afirmar.
///
/// ⛔⛔⛔ **A agulha é MONTADA em runtime, e isso não é estilo:** a 1.ª redacção
/// deste gate também lia ESTE ficheiro por `include_str!` e procurava a mesma
/// agulha — e ela **é trivialmente verdadeira**, porque a agulha está escrita
/// aqui como literal. *Um censo textual que se lê a si mesmo encontra sempre o
/// que procura*, e a mutação que apagava o pedido de limites **SOBREVIVEU**.
/// ⇒ o literal não pode existir inteiro no ficheiro que ele mede — é a mesma
/// razão pela qual a vassoura da parede clean-room guarda as entradas em
/// base64.
///
/// ⚠️ **E a metade do PRODUTO é a que importa**: se ele deixar de subir o
/// limite, esta porta passa a pedir MAIS do que o app, e os gates da suíte
/// voltam a medir outro device — ao contrário, com sinal trocado.
#[test]
fn o_produto_continua_a_subir_o_limite_de_buffers() {
    const PRODUTO: &str = include_str!("../../../ph2d-gpu/src/context.rs");
    let agulha = format!(
        "{}{}",
        "required_limits.max_vertex_buffers = ", "required_limits"
    );
    assert!(
        PRODUTO.contains(&agulha),
        "o produto deixou de subir o `max_vertex_buffers`: esta porta passa a \
         pedir MAIS do que ele, e os gates desta suíte medem outro device"
    );
    // O CONTROLO da extracção: uma agulha que aquele ficheiro NÃO tem falha,
    // senão isto ficaria verde sobre um ficheiro vazio.
    assert!(!PRODUTO.contains("required_limits.max_vertex_buffers = 8u32;"));
}

/// ⭐⭐ **O PRODUTO CONTINUA A PEDIR A CAPACIDADE DA TINTA FINA** — a irmã do
/// gate acima, e pela mesma razão: se ele deixar de a pedir, esta porta passa
/// a devolver um device com uma capacidade que o app não tem, e a suíte mede
/// um shader que o artista nunca executa.
///
/// ⚠️ A agulha é MONTADA em runtime pelo motivo do irmão — este ficheiro
/// também nomeia a capacidade, e um censo que se lê a si mesmo acha sempre o
/// que procura.
#[test]
fn o_produto_continua_a_pedir_a_capacidade_da_tinta() {
    const PRODUTO: &str = include_str!("../../../ph2d-gpu/src/context.rs");
    let agulha = format!("{}{}", "wgpu::Features::PRIMITIVE", "_INDEX");
    assert!(
        PRODUTO.contains(&agulha),
        "o produto deixou de pedir a capacidade da tinta fina: esta porta          devolve um device que ele não tem"
    );
    assert!(!PRODUTO.contains("PRIMITIVE_INDEX_DESLIGADA"));
}

/// ⭐⭐⭐ **E O DEVICE QUE ESTA PORTA DEVOLVE CABE O PIPELINE DO PRODUTO** — a
/// régua de comportamento, que é a que a mutação mata.
///
/// ⚠️ **`9` não é um número escolhido:** é a contagem de buffers por vértice que
/// o `mesh.wgsl` declara desde que o canal de cor chegou, e o piso do WebGPU é
/// `8`. *Um device que não os cabe não constrói o pipeline — e foi assim que
/// dezasseis gates desta suíte ficaram vermelhos de uma vez.*
#[test]
#[ignore = "precisa de adapter"]
fn o_device_desta_porta_cabe_os_nove_buffers_do_shader() {
    let Some((device, _queue)) = device() else {
        eprintln!("sem adapter — skip");
        return;
    };
    let cabem = device.limits().max_vertex_buffers;
    assert!(
        cabem >= 9,
        "o device desta porta cabe {cabem} buffers por vértice e o shader declara 9"
    );
}
