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
//! ⛔ Ele existe porque a alternativa é o `vulkaninfo` à mão, cujo resultado
//! fica num comentário: *um número medido fora da árvore envelhece sem ninguém
//! saber.*

fn main() {
    let inst = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle_from_env());
    let adapters = pollster::block_on(inst.enumerate_adapters(wgpu::Backends::all()));
    for a in adapters {
        let i = a.get_info();
        let f = a.features();
        println!("--- {} ({:?} / {:?})", i.name, i.device_type, i.backend);
        println!(
            "    {:<26} {}",
            "VERTEX_WRITABLE_STORAGE",
            f.contains(wgpu::Features::VERTEX_WRITABLE_STORAGE)
        );
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
