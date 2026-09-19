//! **O SHADER DO BRILHO PARSA E VALIDA** — sem placa nenhuma.
//!
//! ⚠️ **Ele é montado de TRÊS fontes em tempo de execução** — a lei da `ph2d-bloom`, a porta de
//! leitura desta crate e os pontos de entrada —, e um erro de sintaxe em qualquer uma delas só
//! aparece quando o `create_compute_pipeline` corre: *na primeira vez que o artista liga o Bloom*,
//! com a tela a não mudar e uma mensagem de naga num terminal que ele não está a ver.
//!
//! ⛔ Os gates que correriam o shader a sério são `#[ignore]` (precisam de adaptador), logo **o CI
//! nunca os corre** — a lei da casa sobre *skip gracioso não é verde*. Este corre em toda a máquina.

fn valida(rot: &str, src: &str) {
    let module = naga::front::wgsl::parse_str(src)
        .unwrap_or_else(|e| panic!("{rot} não parsa:\n{}", e.emit_to_string(src)));
    let mut v = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    );
    v.validate(&module)
        .unwrap_or_else(|e| panic!("{rot} não valida: {e:?}"));
}

#[test]
fn a_cadeia_e_a_composicao_parsam_e_validam() {
    let (cadeia, composicao) = ph2d_field_gpu::brilho::fontes_para_gate();
    valida("a cadeia do brilho", &cadeia);
    valida("a composição do brilho", &composicao);
}

/// ⭐⭐ **OS PONTOS DE ENTRADA CONTINUAM A EXISTIR, COM OS NOMES QUE O RUST PEDE.**
///
/// ⚠️ O lado Rust escreve-os como **string** e nada os liga em tempo de compilação — um `entry_point`
/// renomeado é um pipeline que falha no arranque do passe.
#[test]
fn os_quatro_pontos_de_entrada_existem() {
    let (cadeia, composicao) = ph2d_field_gpu::brilho::fontes_para_gate();
    for nome in ["desce", "sobe", "sobe_final"] {
        assert!(
            cadeia.contains(&format!("fn {nome}(")),
            "a cadeia perdeu o ponto de entrada `{nome}`"
        );
    }
    assert!(
        composicao.contains("fn compoe("),
        "a composição perdeu o ponto de entrada `compoe`"
    );
}
