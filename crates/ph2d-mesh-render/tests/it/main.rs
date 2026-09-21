//! Um binário de teste por crate (auditoria de velocidade de 2026-09-10, onda W1).
//!
//! Cada ficheiro em `tests/it/` é um MÓDULO deste binário, não um binário próprio: antes eram
//! 10 binários nesta crate, cada um a religar a closure inteira de dependências (matklad,
//! «Delete Cargo Integration Tests»: 3× menos compilação, 5× menos disco). Os nomes dos testes
//! ganham o prefixo do módulo (`ficheiro::fn`); filtros por `test(nome)` continuam a casar.
//! ⚠️ Teste novo = ficheiro novo AQUI + uma linha `mod` abaixo — nunca um `tests/*.rs` solto.

/// **O DEVICE DE TESTE, numa porta só** — ver [`device_de_teste`]. ⛔ Ele
/// existe porque quatro cópias pediam o PISO do WebGPU enquanto o produto pede
/// os limites do adaptador, e dezasseis gates ficaram vermelhos de uma vez.
mod device_de_teste;

mod camera_uniform_layout;
mod gpu_render;
mod gpu_viewport;
mod measure_bake_framing;
mod measure_matcap_decode;
mod measure_screen_radius;
mod measure_sss_curve;
mod measure_the_view_ruler_at_two_anchors;
mod mede_o_que_a_composicao_ja_da_ao_catavento;
mod probe_pick_round_trip;
mod probe_wire_continuity;
mod tinta_no_device;
mod tinta_paridade;
