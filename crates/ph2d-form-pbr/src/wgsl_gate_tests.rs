//! ⛔⛔ **O GÉMEO EM WGSL É COMPILADO, e a razão de isto existir é medida.**
//!
//! Uma `&str` não é compilada por nada: o `cargo check` não olha para ela, e **todos** os gates
//! deste repo que olham para um shader precisam de adapter e são `#[ignore]` ⇒ um erro de sintaxe
//! ou de tipo no gémeo só apareceria **no primeiro quadro de quem abrisse a cena**. É a mesma
//! lacuna que a `ph2d-sculpt3d` pagou ao ganhar o 9.º buffer de vértice.
//!
//! ⚠️ **A composição AQUI é a do produto, e ela NÃO é uma concatenação** — foi este gate que o
//! descobriu: o [`ph2d_material::wgsl::SOURCE`] traz um `{ENV}` por preencher, e o doc do módulo
//! vizinho dizia, por escrito, que bastava juntar as duas fontes. *Um exemplo de montagem que
//! ninguém corre é uma promessa.*

use super::*;

/// As duas funções que o [`ph2d_material::wgsl::ENV_SLOT`] exige, na forma mais magra possível.
///
/// ⚠️ **Elas são do CONSUMIDOR e não desta crate**, e é por isso que vivem no gate: quem monta o
/// passe é quem tem um ambiente. O que se afirma aqui é que o nosso laço **encaixa** na fonte da
/// lei — não que este ambiente seja o do produto.
const ENV_MAGRO: &str = r#"
fn env_radiance(dir: vec3<f32>, alpha: f32, shrink: f32) -> vec3<f32> { return vec3<f32>(0.0); }
fn env_irradiance(n: vec3<f32>) -> vec3<f32> { return vec3<f32>(0.0); }
"#;

/// O tecto que o consumidor escreve. Ver o doc do [`super::CAP_SLOT`] para porque ele não é um
/// número desta crate. Aqui é o do rig de hoje — o gate mede a MONTAGEM, não este valor.
const CAP_DO_RIG: &str = "4u";

/// Monta a fonte como um consumidor a montaria. Ver o cabeçalho do módulo.
fn fonte_composta() -> String {
    format!(
        "{}\n{}\n{}",
        ph2d_view_transform::wgsl::SOURCE,
        ph2d_material::wgsl::SOURCE.replace(ph2d_material::wgsl::ENV_SLOT, ENV_MAGRO),
        SOURCE.replace(CAP_SLOT, CAP_DO_RIG),
    )
}

/// ⭐⭐ **As DUAS marcas são load-bearing** — e sem este gate elas podiam ser apagadas em silêncio.
///
/// ⚠️ O irmão que parsa NÃO cobre isto: quem trocasse o `{MAX_LAMPADAS}` por um `4u` escrito à mão
/// deixaria a fonte a parsar na mesma, com o tecto do rig **cravado** numa crate que não depende
/// dele — que é exactamente a segunda cópia do número que a marca existe para impedir.
#[test]
fn as_duas_marcas_da_montagem_existem() {
    assert!(
        SOURCE.contains(CAP_SLOT),
        "o nosso SOURCE tem de declarar o tecto por MARCA ({CAP_SLOT}), nunca por literal"
    );
    assert!(
        ph2d_material::wgsl::SOURCE.contains(ph2d_material::wgsl::ENV_SLOT),
        "controlo: a fonte da lei ainda traz o {} por preencher",
        ph2d_material::wgsl::ENV_SLOT
    );
    // **O CONTROLO da substituição**: depois dela, nenhuma marca sobra — senão a fonte composta
    // carregaria uma chaveta que o WGSL não sabe ler, e o gate irmão diria «não parsa» sem dizer
    // porquê.
    let composta = fonte_composta();
    assert!(!composta.contains(CAP_SLOT));
    assert!(!composta.contains(ph2d_material::wgsl::ENV_SLOT));
}

/// ⭐⭐⭐ **A fonte composta PARSA e VALIDA.**
#[test]
fn o_gemeo_em_wgsl_parsa_e_valida() {
    let fonte = fonte_composta();
    let module = naga::front::wgsl::parse_str(&fonte)
        .unwrap_or_else(|e| panic!("o gémeo não parsa: {}", e.emit_to_string(&fonte)));
    let mut v = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::empty(),
    );
    let r = v.validate(&module);
    assert!(r.is_ok(), "o gémeo não valida: {:?}", r.err());
}

/// ⭐⭐ **O CONTROLO do gate acima** — ele reprova sobre uma fonte partida.
///
/// ⚠️ Sem ele, um dia em que a `naga` deixasse de ser chamada (ou em que o `parse_str` passasse a
/// aceitar tudo) o irmão ficaria **verde a afirmar nada**. A agulha é montada em runtime pela mesma
/// razão da vassoura da parede: *um censo textual que se lê a si mesmo encontra sempre o que
/// procura.*
#[test]
fn o_validador_reprova_uma_fonte_partida() {
    let partida = format!(
        "{}\n{}",
        fonte_composta(),
        "fn x() -> f32 { return vec3(1.0); }"
    );
    assert!(
        naga::front::wgsl::parse_str(&partida).is_err(),
        "controlo: o validador tinha de recusar um retorno de tipo errado"
    );
}

/// ⛔ **O nosso laço CHAMA a lei, e não a reescreve.**
///
/// ⚠️ A régua é a `naga` e não um `contains`: ela resolve o nome contra a fonte COMPOSTA, logo
/// prova que o `mx_direct` que o laço chama é **o do `ph2d-material`** — um `grep` passaria verde
/// sobre uma segunda redacção local com o mesmo nome. E a metade que a torna honesta é a segunda: o
/// nosso módulo **não pode declarar** uma função cujo nome comece por `mx_`.
#[test]
fn o_laco_chama_a_optica_de_la_e_nao_declara_optica_nenhuma() {
    let fonte = fonte_composta();
    let module = naga::front::wgsl::parse_str(&fonte).expect("parse");

    let laco = module
        .functions
        .iter()
        .find(|(_, f)| f.name.as_deref() == Some("forma_acende_texel"))
        // **CONTROLO POSITIVO**: sem isto, renomear o laço deixaria o gate verde por vácuo.
        .expect("a fonte composta declara o `forma_acende_texel`")
        .1;

    let chama_a_lei = laco.expressions.iter().any(|(_, e)| {
        matches!(e, naga::Expression::CallResult(h)
            if module.functions[*h].name.as_deref() == Some("mx_direct"))
    });
    assert!(
        chama_a_lei,
        "o laço tem de chamar o `mx_direct` da `ph2d-material`"
    );

    // A segunda metade: esta crate não escreve óptica. Um nome `mx_*` declarado no NOSSO `SOURCE`
    // seria exactamente a divergência que a `ph2d-material` existe para impedir.
    let nosso: Vec<&str> = SOURCE
        .lines()
        .filter_map(|l| l.trim().strip_prefix("fn "))
        .filter_map(|l| l.split('(').next())
        .collect();
    assert!(
        !nosso.is_empty(),
        "controlo: a extracção tem de achar alguma fn no nosso SOURCE"
    );
    for n in &nosso {
        assert!(
            !n.starts_with(&format!("{}_", "mx")),
            "esta crate declarou `{n}` — a óptica é da `ph2d-material`, aqui só mora o laço"
        );
    }
}
