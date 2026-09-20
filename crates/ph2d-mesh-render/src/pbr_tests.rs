//! Os gates do modo `Pbr`. ⚠️ Os que precisam de placa vivem noutro sítio; estes são aritmética.

/// ⭐⭐⭐ **A FONTE COMPOSTA PARSA E VALIDA** — sem device.
///
/// ⛔ Sem ele, um erro na composição (um nome que colide entre a lei e o `mesh.wgsl`, uma ranhura
/// por preencher) só apareceria no **primeiro quadro** de quem abrisse a cena 3D: o `cargo check`
/// não olha para uma `&str`, e todo gate desta crate que olha para um shader com device é
/// `#[ignore]`, logo o CI nunca o corre.
#[test]
fn a_fonte_composta_do_barro_parsa_e_valida() {
    let src = super::fonte();
    let module = naga::front::wgsl::parse_str(&src).unwrap_or_else(|e| {
        panic!(
            "a fonte do passe da malha não parsa: {}",
            e.emit_to_string(&src)
        )
    });
    let mut v = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::empty(),
    );
    let r = v.validate(&module);
    assert!(
        r.is_ok(),
        "a fonte do passe da malha não valida: {:?}",
        r.err()
    );
}

/// ⛔⛔ **NENHUMA RANHURA SOBRA POR PREENCHER.**
///
/// ⚠️ *Uma marca escrita num comentário lê-se exactamente igual a uma ranhura por preencher* — esta
/// casa pagou-o três vezes —, e é por isso que o gate varre a fonte COMPOSTA e não a montagem.
#[test]
fn a_ranhura_do_ambiente_e_preenchida() {
    assert!(
        !super::fonte().contains(ph2d_material::wgsl::ENV_SLOT),
        "a fonte do produto não pode levar uma ranhura por preencher"
    );
    // ⭐ **O CONTROLO da régua:** a marca EXISTE na fonte da lei, senão este `contains` passaria
    // por vácuo sobre um dia em que a ranhura mudasse de nome.
    assert!(
        ph2d_material::wgsl::SOURCE.contains(ph2d_material::wgsl::ENV_SLOT),
        "controlo: a fonte da lei tem de trazer a ranhura"
    );
}

/// ⭐⭐ **O BARRO NÃO ESCREVE ÓPTICA** — a razão de este modo existir.
///
/// O `mesh.wgsl` tem um modelo de ARGILA escrito à mão (`CLAY_EXPONENT`, `CLAY_SHINE`), e ele fica
/// — é a lei certa para o modo `Rig`. ⛔ O que este gate proíbe é o modo novo ganhar uma **segunda**
/// redacção da óptica: ele tem de chamar as `mx_*` da crate da lei.
///
/// ⚠️ **A agulha é a CHAMADA e não o nome do ficheiro**, e as duas metades existem porque as curas
/// são opostas: sem a primeira o modo não usa a lei; sem a segunda ele usa-a **e** escreve outra.
#[test]
fn o_modo_pbr_chama_a_lei_e_nao_escreve_uma_segunda() {
    let mesh = crate::fonte::MESH_WGSL;
    for agulha in ["mx_direct(", "mx_indirect(", "mx_at_base_color("] {
        assert!(
            mesh.contains(agulha),
            "o modo PBR tem de chamar `{agulha}` — a lei é a da `ph2d-material`"
        );
    }
    // ⛔ E a segunda metade: o `mesh.wgsl` não pode declarar uma closure própria.
    assert!(
        !mesh.contains("fn mx_"),
        "o passe da malha não pode DECLARAR uma função da lei — ele chama a da crate que a tem"
    );
}

/// ⭐ **O CÉU DO BARRO É O MESMO QUE O DO SPRITE** — a mesma porta, e a mesma fórmula em WGSL.
///
/// ⚠️ Os NÚMEROS não estão aqui: eles são derivados do RIG e viajam no uniform, logo não existe uma
/// segunda cópia deles. O que é escrito duas vezes é a FÓRMULA, e o que este gate afirma é que as
/// duas redacções pedem a **mesma** operação — com o `fma` explícito nos dois lados.
#[test]
fn o_ceu_do_barro_pede_a_mesma_conta_que_o_do_sprite() {
    for agulha in [
        "fma(ceu_inclinacao, vec3<f32>(-n.y), ceu_base)",
        "let up = shrink * -dir.y;",
        "fma(1.5 * ceu_inclinacao, vec3<f32>(up), ceu_base)",
    ] {
        assert!(
            super::CEU_NA_RANHURA.contains(agulha),
            "a ranhura do céu tem de pedir `{agulha}`"
        );
    }
    // ⛔ **E ela não pode recalcular o encolhimento por pixel** — ele viaja pronto no material.
    assert!(
        !super::CEU_NA_RANHURA.contains("lobe_shrink"),
        "o gémeo não pode recalcular o encolhimento do lóbulo por pixel"
    );
    // ⭐ **O CONTROLO da régua:** ela tem de saber dizer NÃO.
    assert!(
        !super::CEU_NA_RANHURA.contains("fn env_transmission("),
        "controlo: a régua tem de poder dizer NÃO a uma metade que não existe"
    );
}

/// ⚠️ **O uniform tem a forma que o WGSL lê** — e a ordem dos cinco campos é a ordem do `struct Pbr`.
///
/// ⛔ Um `vec3` num uniform alinha a `16 B`, logo os dois do céu viajam como `vec4` com o `a` a
/// padding: *declarar o preenchimento é mais honesto do que deixá-lo implícito*, que é a mesma
/// decisão que o [`crate::lighting::LampRaw`] já tomou.
///
/// ⭐ **E ele cresceu em 2026-09-21** (o OLHAR: os stops mais o código da vista) — o gate reprovou
/// com `left: 256, right: 224`, que é exactamente ele a funcionar: *um uniform que cresce sem a
/// régua crescer é onde o WGSL passa a ler um campo pelo offset de outro, em silêncio*.
#[test]
fn o_uniform_do_material_tem_a_forma_que_o_shader_le() {
    use ph2d_material::wgsl::PACKED;
    assert_eq!(
        super::PbrRaw::SIZE,
        (PACKED + 4 + 4 + 4 + 4) * 4,
        "o uniform do material tem de ser o `Mat`, os dois `vec4` do céu e os dois do olhar"
    );
    // E o shader declara-o na mesma ordem.
    let mesh = crate::fonte::MESH_WGSL;
    assert!(
        mesh.contains("struct Pbr {")
            && mesh.contains("ceu_base: vec4<f32>")
            // ⭐ O OLHAR é a metade que o report do dono pediu — sem ele o visor mostra a radiância
            // CRUA e a sprite mostra a exposta.
            && mesh.contains("olhar: vec4<f32>")
            && mesh.contains("vista: vec4<u32>"),
        "o `mesh.wgsl` tem de declarar o uniform do material"
    );
}
