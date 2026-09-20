//! ⭐ **O QUE A PLACA DESTA MÁQUINA ANUNCIA** — as capacidades que uma wave
//! precisa de saber ANTES de desenhar o caminho.
//!
//!     bash scripts/ph2d-run.sh env PH2D_GPU=1 cargo run -p ph2d-gpu --example o_que_a_placa_anuncia
//!
//! ⚠️ **Ele lê o ADAPTADOR e não o device.** O `wgpu` só reporta uma feature em
//! `device.features()` se ela tiver sido **pedida** — logo perguntar ao device
//! responde *«o que já pedimos»* e nunca *«o que dava para pedir»*, que é a
//! pergunta de quem está a desenhar.
//!
//! ⛔ Ele existe porque a alternativa é o `vulkaninfo`, que esta casa já usou à
//! mão para medir os `maxVertexInputBindings` e cujo resultado ficou num
//! comentário: *um número medido fora da árvore envelhece sem ninguém saber.*
//!
//! Medido em 2026-09-20 nesta máquina: `PRIMITIVE_INDEX` é **true** nas três
//! rotas (RTX 5060 Ti/Vulkan, RADV iGPU, RTX/GL) — é ela que deixa um shader de
//! fragmento saber **em que face** está, que é o que a tinta por amostra pede.

fn main() {
    let inst = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle_from_env());
    let adapters = pollster::block_on(inst.enumerate_adapters(wgpu::Backends::all()));
    for a in adapters {
        let i = a.get_info();
        let f = a.features();
        println!("--- {} ({:?} / {:?})", i.name, i.device_type, i.backend);
        for (nome, bit) in [
            ("PRIMITIVE_INDEX", wgpu::Features::PRIMITIVE_INDEX),
            (
                "VERTEX_WRITABLE_STORAGE",
                wgpu::Features::VERTEX_WRITABLE_STORAGE,
            ),
        ] {
            println!("    {nome:<26} {}", f.contains(bit));
        }
        println!(
            "    max_storage_buffers_per_shader_stage = {}",
            a.limits().max_storage_buffers_per_shader_stage
        );
        println!(
            "    max_storage_buffer_binding_size = {} MB",
            a.limits().max_storage_buffer_binding_size / 1_000_000
        );
    }
}
