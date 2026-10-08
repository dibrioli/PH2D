//! Um binário de teste por crate (auditoria de velocidade de 2026-09-10, onda W1).
//!
//! Cada ficheiro em `tests/it/` é um MÓDULO deste binário, não um binário próprio: antes eram
//! 4 binários nesta crate, cada um a religar a closure inteira de dependências (matklad,
//! «Delete Cargo Integration Tests»: 3× menos compilação, 5× menos disco). Os nomes dos testes
//! ganham o prefixo do módulo (`ficheiro::fn`); filtros por `test(nome)` continuam a casar.
//! ⚠️ Teste novo = ficheiro novo AQUI + uma linha `mod` abaixo — nunca um `tests/*.rs` solto.
//! ⛔ Excepção: um ficheiro com `#[global_allocator]` tem de ser binário PRÓPRIO (dois alocadores
//! globais não cabem num binário, e um contador global veria as alocações dos vizinhos): fica em `tests/`.

mod a_luz_do_impasto_e_a_de_antes_da_poda;
mod gotas_pela_lei_do_rebelle;
mod measure_undo_capacity;
mod os_outros_meios_nao_mudam;
mod presets_de_fabrica;
mod sonda_gestos;
mod sonda_gotas;
mod sonda_largura;
mod sonda_tracos_por_meio;
