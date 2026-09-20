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

/// **O shader compila** — parse + validação completa, na CPU, sobre **o que o produto entrega ao
/// device**.
///
/// # ⛔⛔ A premissa que MORREU, e foi este gate que a matou
///
/// Ele lia `include_str!("shaders/mesh.wgsl")` — *o ficheiro sozinho* — e isso era verdade enquanto
/// o passe da malha tivesse toda a óptica dentro dele. Com o modo [`crate::shade::Lighting::Pbr`] a
/// lei passou a vir da [`ph2d_material`], e o ficheiro cru **deixou de parsar**: ele nomeia `Mat` e
/// `mx_direct`, que a composição traz.
///
/// ⭐ **E ele reprovou ANTES de eu ter ligado a composição ao `pipeline_build`** — que ainda dava o
/// `MESH_WGSL` cru ao `create_shader_module`. *Um gate que lê o ficheiro em vez da porta acusa o
/// dia em que o produto passa a montar; um que lê a porta acusa o dia em que a montagem quebra —
/// e só o segundo continua a valer depois.*
#[test]
fn o_mesh_wgsl_parsa_e_valida_no_naga() {
    let src = crate::pbr::fonte();
    let module = naga::front::wgsl::parse_str(&src)
        .unwrap_or_else(|e| panic!("a fonte da malha nao parsa: {}", e.emit_to_string(&src)));
    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::empty(),
    );
    let r = validator.validate(&module);
    assert!(r.is_ok(), "a fonte da malha nao valida: {:?}", r.err());
}

/// ⛔⛔ **E O QUE O `pipeline_build` ENTREGA AO DEVICE É A FONTE COMPOSTA** — a metade que o parse
/// não vê.
///
/// ⚠️ O gate acima pode ficar verde com o produto a dar o ficheiro CRU ao `create_shader_module`, e
/// foi exactamente esse o estado em que ele me apanhou. *Um gate sobre a fonte certa é cego a quem
/// compila a errada* — a mesma forma que esta casa já paga com os censos de fiação.
#[test]
fn o_produto_compila_a_fonte_composta_e_nao_o_ficheiro_cru() {
    let build = include_str!("pipeline_build.rs");
    assert!(
        build.contains("crate::pbr::fonte()"),
        "o `create_shader_module` da malha tem de receber a fonte COMPOSTA"
    );
    // ⭐ **O CONTROLO da régua:** ela tem de saber dizer NÃO — o nome cru não pode sobrar num
    // `ShaderSource`, que é como a recaída aconteceria.
    assert!(
        !build.contains("ShaderSource::Wgsl(MESH_WGSL"),
        "controlo: o ficheiro cru não pode voltar a ser a fonte do passe"
    );
}

/// ⭐⭐ **AS ENTRADAS DE VÉRTICE DO SHADER SÃO AS QUE O PIPELINE AMARRA** — a
/// costura que um parse sozinho não vê.
///
/// ⚠️ **O `naga` valida o shader ISOLADO:** ele diz que `@location(8)` é uma
/// declaração legal e **não** sabe se alguém a alimenta. A outra metade é o
/// [`crate::pipeline_vertex_layout`], e as duas só se encontram na criação do
/// pipeline — que precisa de device. ⇒ a régua lê os DOIS ficheiros como texto
/// e compara os conjuntos de `shader_location`.
///
/// ⚠️ **O endereço da segunda metade MUDOU em 2026-09-20** (o corte do tecto de
/// LOC do `pipeline_build.rs`), e este `include_str!` é o que torna a mudança
/// BARULHENTA: um caminho que já não existe **não compila**. *Uma régua que
/// procurasse a agulha por `grep` numa árvore teria lido zero e ficado verde.*
///
/// ⛔ **Sem isto, acrescentar a entrada no shader e esquecer o buffer no
/// layout compila, valida e estoura no primeiro quadro** — que é precisamente
/// o modo de falha que esta crate tinha aberto.
#[test]
fn toda_entrada_de_vertice_do_shader_tem_buffer_no_pipeline() {
    let wgsl = include_str!("shaders/mesh.wgsl");
    let build = include_str!("pipeline_vertex_layout.rs");

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

    // As que a DISPOSIÇÃO declara. O `f32_attr(n)` é o atalho da casa para um
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
         `pipeline_vertex_layout.rs` deixaram de coincidir: o shader pede {pedidas:?} e o \
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

/// ⭐⭐⭐⭐ **AS DUAS FONTES COMPILAM, e a de baixo compila SEM a capacidade** —
/// o gate que faz a degradação ser real em vez de uma promessa.
///
/// ⛔⛔ **A validação de um módulo WGSL é tudo-ou-nada:** uma fonte que
/// mencione `@builtin(primitive_index)` numa placa que não o anuncia **não
/// compila NADA**, e a peça deixaria de desenhar de todo — não só a tinta
/// fina. É por isso que o corte é na FONTE ([`crate::fonte::mesh_wgsl`]) e não
/// num `if` lá dentro.
///
/// ⭐ **E o corte é entre a LEI e a ENTRADA, de propósito:** o gémeo em WGSL
/// recebe o índice do triângulo como ARGUMENTO, logo ele valida sem a
/// capacidade e sem device — em toda máquina que corra `cargo test`. Quem
/// precisa da capacidade é só o `fs_main_tinta`.
#[test]
fn as_duas_fontes_do_shader_parsam_e_validam() {
    // ⚠️ A capacidade do `primitive_index` no naga, para a fonte de cima.
    let com_pi = naga::valid::Capabilities::PRIMITIVE_INDEX;
    for (nome, com_tinta, caps) in [
        (
            "sem tinta (a placa não anuncia)",
            false,
            naga::valid::Capabilities::empty(),
        ),
        ("com tinta", true, com_pi),
    ] {
        let src = crate::fonte::mesh_wgsl(com_tinta);
        let module = naga::front::wgsl::parse_str(&src)
            .unwrap_or_else(|e| panic!("{nome}: nao parsa: {}", e.emit_to_string(&src)));
        let r = naga::valid::Validator::new(naga::valid::ValidationFlags::all(), caps)
            .validate(&module);
        assert!(r.is_ok(), "{nome}: nao valida: {:?}", r.err());
    }

    // ⭐⭐ CONTROLO, e é ele que prova que o corte serve para alguma coisa: com
    //   a tinta dentro e SEM a capacidade, a validação TEM de recusar. Sem
    //   esta metade, o `if` da composição poderia não estar a fazer nada.
    let src = crate::fonte::mesh_wgsl(true);
    let module = naga::front::wgsl::parse_str(&src).expect("parsa");
    let r = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::empty(),
    )
    .validate(&module);
    assert!(
        r.is_err(),
        "a fonte COM tinta validou sem a capacidade do `primitive_index` — \
         então o corte da fonte não está a proteger nada"
    );

    // ⭐ CONTROLO: a fonte de baixo é a de cima MENOS o bloco, e não outra
    //   coisa — se alguém a escrever à mão, as duas divergem em silêncio.
    assert!(
        crate::fonte::mesh_wgsl(true).contains(crate::fonte::MESH_WGSL),
        "a fonte com tinta deixou de ser a de sempre MAIS o bloco"
    );
    assert!(
        crate::fonte::mesh_wgsl(true).starts_with("enable "),
        "a directiva `enable` tem de abrir a fonte — ela precede toda declaração"
    );
    assert!(
        !crate::fonte::mesh_wgsl(false).contains("primitive_index"),
        "a fonte sem tinta ainda menciona a capacidade que ela existe para evitar"
    );
}
