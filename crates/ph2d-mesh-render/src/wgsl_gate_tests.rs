//! ⛔⛔ **O GATE QUE ESTA CRATE NÃO TINHA: o `mesh.wgsl` é PARSADO e VALIDADO
//! sem device.**
//!
//! O `cargo check` não olha para WGSL, e todos os gates desta crate que olham
//! precisam de adapter — logo são `#[ignore]` e **o CI nunca os corre**. Um
//! erro de shader (uma entrada de vértice a mais, um varying que não casa, um
//! tipo trocado) só aparecia no **primeiro quadro de quem abrisse a cena 3D**,
//! e nenhuma corrida de teste o via.
//!
//! ⚠️ Ele nasceu na wave da COR POR VÉRTICE (2026-09-19), que acrescenta uma
//! entrada de vértice, um varying e um produto no fragment — exactamente a
//! classe de mudança que este gate apanha e que nenhuma outra régua desta
//! crate apanhava. Espelha o `sprite.wgsl` (`ph2d-render`) e o
//! `impasto_light.wgsl`, que já viviam com a mesma forma.

/// **O shader compila** — parse + validação completa, na CPU.
#[test]
fn o_mesh_wgsl_parsa_e_valida_no_naga() {
    let src = include_str!("shaders/mesh.wgsl");
    let module = naga::front::wgsl::parse_str(src)
        .unwrap_or_else(|e| panic!("mesh.wgsl nao parsa: {}", e.emit_to_string(src)));
    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::empty(),
    );
    let r = validator.validate(&module);
    assert!(r.is_ok(), "mesh.wgsl nao valida: {:?}", r.err());
}

/// ⭐⭐ **AS ENTRADAS DE VÉRTICE DO SHADER SÃO AS QUE O PIPELINE AMARRA** — a
/// costura que um parse sozinho não vê.
///
/// ⚠️ **O `naga` valida o shader ISOLADO:** ele diz que `@location(8)` é uma
/// declaração legal e **não** sabe se alguém a alimenta. A outra metade é o
/// `pipeline_build`, e as duas só se encontram na criação do pipeline — que
/// precisa de device. ⇒ a régua lê os DOIS ficheiros como texto e compara os
/// conjuntos de `shader_location`.
///
/// ⛔ **Sem isto, acrescentar a entrada no shader e esquecer o buffer no
/// layout compila, valida e estoura no primeiro quadro** — que é precisamente
/// o modo de falha que esta crate tinha aberto.
#[test]
fn toda_entrada_de_vertice_do_shader_tem_buffer_no_pipeline() {
    let wgsl = include_str!("shaders/mesh.wgsl");
    let build = include_str!("pipeline_build.rs");

    // As localizações que o VERTEX pede. ⚠️ Só as do `vs_main`: o `vs_wire` é
    // um subconjunto por desenho (ele não lê a cor), e o `VsOut` também usa
    // `@location`, que é outro espaço de nomes.
    let corpo = wgsl
        .split("fn vs_main(")
        .nth(1)
        .expect("o `vs_main` tem de existir — ele é a entrada de vértice do barro")
        .split(") -> VsOut")
        .next()
        .expect("a assinatura do `vs_main` fecha em `) -> VsOut`");
    let pedidas = localizacoes(corpo, "@location(");

    // As que o pipeline DECLARA. O `f32_attr(n)` é o atalho da casa para um
    // escalar; um atributo escrito por extenso traz `shader_location: n`.
    // ⚠️ **As DUAS formas, e a lição é do próprio gate:** a 1.ª redacção lia só
    // o `f32_attr(` e acusou o pipeline de não dar a posição nem a normal, que
    // usam o irmão `vec3_attr(`. *Uma extracção que conhece metade das formas
    // acusa produto correcto* — e a terceira (`shader_location: n` escrito à
    // mão) fica aqui para o dia em que alguém não use nenhum dos dois atalhos.
    let dadas: std::collections::BTreeSet<u32> = localizacoes(build, "f32_attr(")
        .union(&localizacoes(build, "vec3_attr("))
        .copied()
        .collect::<std::collections::BTreeSet<u32>>()
        .union(&localizacoes(build, "shader_location: "))
        .copied()
        .collect();

    assert!(
        pedidas.len() >= 9,
        "piso de populacao: o `vs_main` pede {} entradas e a malha tem 9 canais \
         por vertice — a extraccao partiu-se",
        pedidas.len()
    );
    assert_eq!(
        pedidas, dadas,
        "as entradas de vertice do `mesh.wgsl` e os atributos do \
         `pipeline_build.rs` deixaram de coincidir: o shader pede {pedidas:?} e o \
         pipeline da' {dadas:?}"
    );
}

/// As localizações que seguem `agulha` no texto, uma vez cada.
fn localizacoes(texto: &str, agulha: &str) -> std::collections::BTreeSet<u32> {
    texto
        .split(agulha)
        .skip(1)
        .filter_map(|resto| {
            let n: String = resto.chars().take_while(char::is_ascii_digit).collect();
            n.parse().ok()
        })
        .collect()
}
