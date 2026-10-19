//! Um binário de teste por crate (auditoria de velocidade de 2026-09-10, onda W1).
//!
//! Cada ficheiro em `tests/it/` é um MÓDULO deste binário, não um binário próprio: antes eram
//! 4 binários nesta crate, cada um a religar a closure inteira de dependências (matklad,
//! «Delete Cargo Integration Tests»: 3× menos compilação, 5× menos disco). Os nomes dos testes
//! ganham o prefixo do módulo (`ficheiro::fn`); filtros por `test(nome)` continuam a casar.
//! ⚠️ Teste novo = ficheiro novo AQUI + uma linha `mod` abaixo — nunca um `tests/*.rs` solto.
//! ⛔ Excepção: um ficheiro com `#[global_allocator]` tem de ser binário PRÓPRIO (dois alocadores
//! globais não cabem num binário, e um contador global veria as alocações dos vizinhos): fica em `tests/`.

mod a_caneta_deitada;
mod a_lei_da_tinta;
mod a_luz_do_impasto_e_a_de_antes_da_poda;
mod as_formas;
mod macro_agua;
mod macro_colordrop;
mod macro_comum;
mod macro_em_cena;
mod macro_etapas;
mod macro_instantaneos;
mod macro_portas;
mod macro_traco;
mod macro_trilho;
mod macro_tutorial_fill;
mod measure_undo_capacity;
mod midias_de_fabrica;
mod o_pigmento_por_comprimento;
mod o_po_do_aerografo;
mod o_relevo_dos_fios;
mod os_fios;
mod os_outros_meios_nao_mudam;
mod presets_de_fabrica;
mod receitas_de_fabrica;
mod regua_escada;
mod regua_midia;
mod sonda_etapas;
mod sonda_gestos;
mod sonda_largura;
mod sonda_macro_custo;
mod sonda_midias;
mod sonda_spacing_continuo;
mod sonda_tilt_do_preset;
mod sonda_traco_rapido;
mod sonda_tracos_por_meio;
