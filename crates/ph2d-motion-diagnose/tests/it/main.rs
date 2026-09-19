//! Um binário de teste por crate (auditoria de velocidade de 2026-09-10, onda W1).
//!
//! Cada ficheiro em `tests/it/` é um MÓDULO deste binário, não um binário próprio: antes eram
//! 2 binários nesta crate, cada um a religar a closure inteira de dependências (matklad,
//! «Delete Cargo Integration Tests»: 3× menos compilação, 5× menos disco). Os nomes dos testes
//! ganham o prefixo do módulo (`ficheiro::fn`); filtros por `test(nome)` continuam a casar.
//! ⚠️ Teste novo = ficheiro novo AQUI + uma linha `mod` abaixo — nunca um `tests/*.rs` solto.

mod conditional_production;
mod diagnose;

/// ⭐ **A SONDA da lei da aparência** (2026-09-19) — que TIPOS acrescentam aparência a uma
/// corrente que não a tinha, lidos pelo mesmo canal derivado que o resto do diagnosticador usa.
///
/// ⚠️ Ela existe porque a lei nova (*«uma fonte de posições cuja cadeia não chega a nada que
/// desenhe»*) precisa de saber se a pergunta é **derivável** — e esta casa já pagou por
/// construir sobre uma derivação que ninguém mediu.
///
/// `cargo test -p ph2d-motion-diagnose --test it -- --ignored --nocapture quem_da_aparencia`
#[test]
#[ignore = "sonda de medição, não é gate"]
fn quem_da_aparencia() {
    let mut reg = ph2d_node_registry::NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("regista");
    let ids: Vec<_> = reg.manifests().map(|m| (m.id, m.name)).collect();
    let exige_forma: Vec<&str> = ids
        .iter()
        .filter(|(id, _)| {
            reg.required_inputs(*id)
                .is_some_and(|ps| ps.contains(&"shape"))
        })
        .map(|(_, n)| *n)
        .collect();
    let fontes: Vec<&str> = ids
        .iter()
        .filter(|(id, _)| reg.is_object_source(*id) || reg.is_live_vector_source(*id))
        .map(|(_, n)| *n)
        .collect();
    let posicoes: Vec<&str> = ids
        .iter()
        .filter(|(id, _)| reg.so_posicoes(*id))
        .map(|(_, n)| *n)
        .collect();
    println!("exige `shape`: {} — {exige_forma:?}", exige_forma.len());
    for nome in ["field.shape", "motion.duplicator"] {
        let m = reg
            .manifests()
            .find(|m| m.name == nome)
            .expect("o no existe");
        let saidas: Vec<String> = m
            .outputs
            .iter()
            .map(|o| format!("{}:{:?}/{:?}", o.name, o.ty.domain, o.ty.dim))
            .collect();
        println!("  {nome} -> {saidas:?}");
    }
    println!("fontes de arte: {} — {fontes:?}", fontes.len());
    println!("so' posicoes: {} — {posicoes:?}", posicoes.len());
    for col in ["uv_rect", "geometry_id", "texture_id"] {
        let quem: Vec<&str> = ids
            .iter()
            .filter(|(id, _)| ph2d_motion_diagnose::produz_para_a_sonda(&reg, *id, col))
            .map(|(_, n)| *n)
            .collect();
        println!("{col}: {} de {} — {quem:?}", quem.len(), ids.len());
    }
}
