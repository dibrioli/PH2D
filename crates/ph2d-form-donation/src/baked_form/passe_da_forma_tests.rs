//! **Os gates do passe da forma — o que se afirma SEM PLACA.**
//!
//! ⚠️ A paridade no PIXEL é `#[ignore]` e vive na [`crate::prova_da_placa`], porque ela precisa de
//! adapter. O que mora aqui é o que **o CI de facto corre**: a fonte composta parsa e valida, o
//! uniform tem a forma que o WGSL lê, e as duas cercas de montagem.

use super::*;

/// ⭐⭐⭐ **A FONTE DO PRODUTO PARSA E VALIDA.**
///
/// ⚠️ Ele NÃO é o gémeo do gate da `ph2d-form-pbr`: aquele monta uma fonte *como um consumidor
/// montaria*, com um ambiente magro escrito no próprio teste. Este corre a [`fonte`] **do produto**
/// — o mesmo `String` que o [`PasseDaForma::new`] entrega ao `create_shader_module` —, e é a
/// diferença entre *«a lei encaixa»* e *«o que o app compila está bem formado»*.
///
/// ⛔ Sem ele, um erro no [`ENTRADA`] só apareceria no primeiro quadro de quem abrisse a cena: o
/// `cargo check` não olha para uma `&str`.
#[test]
fn a_fonte_do_produto_parsa_e_valida() {
    let src = fonte();
    let module = naga::front::wgsl::parse_str(&src)
        .unwrap_or_else(|e| panic!("a fonte do passe não parsa: {}", e.emit_to_string(&src)));
    let mut v = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::empty(),
    );
    let r = v.validate(&module);
    assert!(r.is_ok(), "a fonte do passe não valida: {:?}", r.err());
}

/// ⭐⭐ **O CONTROLO do gate acima** — ele reprova sobre uma fonte partida.
///
/// ⚠️ Sem ele, um dia em que o `parse_str` passasse a aceitar tudo o irmão ficaria **verde a
/// afirmar nada**. A agulha é montada em runtime pela razão de sempre: *um censo textual que se lê
/// a si mesmo encontra sempre o que procura.*
#[test]
fn o_validador_reprova_uma_fonte_partida() {
    let partida = format!("{}\n{}", fonte(), "fn x() -> f32 { return vec3(1.0); }");
    assert!(
        naga::front::wgsl::parse_str(&partida).is_err(),
        "controlo: o validador tinha de recusar um retorno de tipo errado"
    );
}

/// ⛔⛔ **AS DUAS RANHURAS SÃO PREENCHIDAS, e nenhuma chaveta sobra.**
///
/// ⚠️ **A segunda metade é a que carrega peso.** Uma marca por substituir deixaria a fonte com uma
/// chaveta que o WGSL não sabe ler, e o gate de cima diria *«não parsa»* sem dizer porquê — o
/// diagnóstico ficaria a dois saltos do defeito.
#[test]
fn as_duas_ranhuras_sao_preenchidas() {
    assert!(
        gemeo::SOURCE_DA_LEI.contains(gemeo::ENV_SLOT),
        "controlo: a fonte da lei ainda traz a ranhura do ambiente"
    );
    assert!(
        gemeo::SOURCE.contains(gemeo::CAP_SLOT),
        "controlo: o gémeo do laço ainda traz a ranhura do tecto"
    );
    let src = fonte();
    assert!(
        !src.contains(gemeo::ENV_SLOT) && !src.contains(gemeo::CAP_SLOT),
        "a fonte do produto não pode levar uma ranhura por preencher"
    );
}

/// ⛔⛔ **A LEI CHAMA UMA DAS DUAS METADES DA RANHURA — a IRRADIÂNCIA, nunca a ESPELHADA.**
///
/// # A premissa que morreu
///
/// Este gate chamava-se `a_lei_da_forma_nao_chama_o_ambiente` e afirmava `!SOURCE.contains("env_")`,
/// com a razão escrita ao lado: *«o rig desta casa é `KEY + 3 × FILL` e as lâmpadas de preenchimento
/// SÃO o ambiente dele»*. ⛔ **Medido, as três de preenchimento nascem APAGADAS** ⇒ na configuração
/// de fábrica havia uma lâmpada e nenhum ambiente, e `25,03 %` da peça saía PRETA ao bit. O laço
/// passou a chamar o `env_irradiance`, e o que este gate afirma inverteu-se.
///
/// # ⚠️ As três metades, e cada uma impede um defeito diferente
///
/// 1. o laço **CHAMA** a irradiância — sem isto o ambiente volta a ser inerte, em silêncio;
/// 2. o laço **NÃO CHAMA** a espelhada — ela é a indirecta do OpenPBR (coluna **B3**) e o stub dela
///    devolve zero, logo somá-la hoje seria somar nada com o custo de a chamar; *e no dia em que ela
///    deixar de ser um stub, chamá-la sem querer acrescenta uma segunda luz que ninguém autorou*;
/// 3. o stub da espelhada **continua a devolver zero** — é o que torna a 2.ª metade uma escolha e
///    não um acidente.
#[test]
fn a_lei_da_forma_chama_a_irradiancia_e_nao_a_espelhada() {
    assert!(
        gemeo::SOURCE.contains("env_irradiance(n)"),
        "o laço da forma TEM de ler o céu pela ranhura do ambiente"
    );
    assert!(
        !gemeo::SOURCE.contains("env_radiance"),
        "o laço da forma não pode chamar a espelhada pré-filtrada: ela é a indirecta do OpenPBR, \
         e hoje o stub dela devolve zero"
    );
    assert!(
        CEU_NA_RANHURA.contains("fn env_radiance(dir: vec3<f32>, alpha: f32, shrink: f32) -> vec3<f32> { return vec3<f32>(0.0); }"),
        "o stub da espelhada tem de devolver zero"
    );
}

/// ⭐⭐ **O CÉU DO SHADER LÊ A MESMA RAMPA QUE O GÉMEO EM RUST.**
///
/// ⛔ **Os números NÃO estão aqui, e essa é a decisão:** eles são derivados do RIG
/// ([`super::super::ceu_do_rig`]) e viajam no uniform, logo não existe uma segunda cópia deles. O
/// que é escrito duas vezes é a FÓRMULA — e o que este gate afirma é que as duas redacções dela
/// pedem a **mesma** operação.
///
/// ⚠️ **O `fma` é EXPLÍCITO nos dois lados**, e é para a paridade: o [`ph2d_form_pbr::Ceu`] usa
/// `mul_add`, que tem UM arredondamento. Escrito solto (`base + inclinação * up`), a igualdade ao
/// bit passaria a depender de o compilador do WGSL contrair a multiplicação-soma — e esta casa já
/// mediu uma placa a contraí-la onde o fonte não o pedia.
///
/// ⚠️ **E o sinal do `y` está nas DUAS redacções**: o céu é o topo da TELA e o referencial é CANVAS.
/// *Um sinal trocado aqui acende a peça por baixo, e a paridade não o veria — os dois motores
/// estariam de acordo na mesma peça errada.*
#[test]
fn o_ceu_do_shader_le_a_mesma_rampa_que_o_gemeo_em_rust() {
    assert!(
        CEU_NA_RANHURA.contains("fma(ceu_inclinacao, vec3<f32>(-n.y), ceu_base)"),
        "a rampa em WGSL tem de pedir o `fma` sobre `-n.y`, como o `mul_add` do gémeo"
    );
    assert!(
        CEU_NA_RANHURA
            .contains("fn env_radiance(dir: vec3<f32>, alpha: f32, shrink: f32) -> vec3<f32> { return vec3<f32>(0.0); }"),
        "o stub da espelhada tem de devolver ZERO — nada nesta lei o chama"
    );
    // **O CONTROLO da régua**: uma agulha que não está lá tem de falhar, senão um `contains` sobre
    // um ficheiro reescrito passaria por vácuo.
    assert!(
        !CEU_NA_RANHURA.contains("fma(ceu_inclinacao, vec3<f32>(n.y)"),
        "controlo: a régua tem de poder dizer NÃO ao sinal trocado"
    );
    // E o gémeo em Rust faz a MESMA conta, medida: a rampa sobe para o topo da TELA (`-y`).
    let c = ph2d_form_pbr::Ceu {
        base: [1.0; 3],
        inclinacao: [2.0; 3],
    };
    assert_eq!(c.irradiancia([0.0, -1.0, 0.0])[0], 3.0, "topo da tela");
    assert_eq!(c.irradiancia([0.0, 1.0, 0.0])[0], -1.0, "fundo da tela");
    assert_eq!(
        c.irradiancia([0.0, 0.0, 1.0])[0],
        1.0,
        "o horizonte é a base"
    );
}

/// ⭐ **O TECTO DE LÂMPADAS É O DO RIG**, e ele chega ao shader.
///
/// ⚠️ As duas metades: o número é lido do dono (`ph2d_light`) e não escrito aqui, **e** ele é o que
/// a fonte composta declara — um tecto certo que não chegasse ao WGSL faria a segunda lâmpada ser
/// lida de fora do array.
#[test]
fn o_tecto_de_lampadas_e_o_do_rig_e_chega_ao_shader() {
    assert_eq!(MAX_LAMPADAS, ph2d_light::MAX_LIGHTS);
    let agulha = format!("array<LampadaGpu, {MAX_LAMPADAS}u>");
    assert!(
        fonte().contains(&agulha),
        "a fonte composta tem de declarar `{agulha}`"
    );
}

/// ⛔⛔ **O UNIFORM TEM A FORMA QUE O WGSL LÊ** — os quatro blocos, nesta ordem.
///
/// ⚠️ **Nada no tipo prende as duas metades**: o lado do Rust escreve `f32` crus num `Vec` e o lado
/// do WGSL lê uma `struct`. Um bloco a mais ou a menos no meio desloca **tudo o que vem depois**, e
/// o sintoma seria a peça acender com o material de outra coisa — plausível, e sem nada a dizer
/// porquê.
///
/// A régua é a ARITMÉTICA do layout (que o Rust escreve) contra a ORDEM DOS CAMPOS (que o WGSL
/// declara), e as duas têm de concordar.
#[test]
fn o_uniform_tem_a_forma_que_o_wgsl_le() {
    // A ordem declarada no `struct Globais` do ENTRADA — lida do próprio texto, nunca de memória.
    let campos = [
        "material: Mat",
        "olhar",
        "ceu_base",
        "ceu_inclinacao",
        "vista",
        "lampadas",
    ];
    let mut cursor = 0usize;
    for c in campos {
        let at = ENTRADA[cursor..].find(c).unwrap_or_else(|| {
            panic!("o `struct Globais` tem de declarar `{c}` depois do anterior")
        });
        cursor += at + c.len();
    }
    // E a aritmética do lado do Rust: `Mat` (12 vec4) + dois vec4 + o `Lampadas`.
    assert_eq!(gemeo::PACKED, 48, "o `Mat` são doze vec4");
    assert_eq!(
        Globais::FLOATS,
        48 + 4 + 4 + 4 + 4 + 4 + MAX_LAMPADAS * gemeo::LAMPADA_FLOATS,
        "o uniform mede o material, o olhar, o céu, a vista e as lâmpadas — e mais nada"
    );
    // ⚠️ O `n` das lâmpadas vive a `16` bytes do princípio do `Lampadas`, e não colado ao array:
    // um `vec3<f32>` num array alinha a 16, logo os três slots a seguir ao `n` são PADDING. Sem
    // eles a primeira lâmpada seria lida do meio do cabeçalho.
    assert_eq!(gemeo::LAMPADA_FLOATS, 8);
}

/// ⛔ **UM RIG MAIOR QUE O UNIFORM RECUSA EM VOZ ALTA.**
///
/// ⚠️ O caminho calado seria o caro: escrever `n = 6` num array de `4` faria o laço do shader ler
/// dois blocos de lixo, e o sintoma é a peça acender com duas lâmpadas que ninguém acendeu.
#[test]
fn um_rig_maior_que_o_uniform_recusa() {
    let l = Lampada {
        para_a_luz: [0.0, 0.0, 1.0],
        radiancia: [1.0; 3],
    };
    let muitas = vec![l; MAX_LAMPADAS + 1];
    let s = ph2d_form_pbr::OpenPbr::default().prepare();
    let Err(e) = Globais::novo(&s, &muitas, Ceu::PRETO, Look::default()) else {
        panic!("um rig maior que o uniform tem de recusar");
    };
    assert!(
        e.contains(&format!("{}", MAX_LAMPADAS + 1)),
        "a queixa diz o número que chegou: {e}"
    );
    // CONTROLO: o rig de hoje cabe.
    assert!(Globais::novo(&s, &muitas[..MAX_LAMPADAS], Ceu::PRETO, Look::default()).is_ok());
}

/// ⭐⭐ **O `n` e a VISTA viajam pelos BITS, e voltam.**
///
/// ⚠️ Eles são `u32` num buffer de `f32`: um `as f32` faria `4` virar `4.0`, cujos bits são
/// `0x40800000` — e o shader leria **1 082 130 432** lâmpadas. A conversão é por `from_bits`, e
/// este gate é a ida-e-volta que o prova.
#[test]
fn a_contagem_e_a_vista_viajam_pelos_bits() {
    let s = ph2d_form_pbr::OpenPbr::default().prepare();
    let l = Lampada {
        para_a_luz: [0.0, 0.0, 1.0],
        radiancia: [1.0; 3],
    };
    let olhar = Look {
        exposure_stops: 3.0,
        view: ph2d_view_transform::ViewTransform::Neutral,
    };
    let g = Globais::novo(&s, &[l, l], Ceu::PRETO, olhar).expect("duas lâmpadas cabem");
    let i = gemeo::PACKED;
    assert!(
        (g.dados[i + 3] - 3.0).abs() < 1e-6,
        "os stops são um f32 normal"
    );
    assert_eq!(
        g.dados[i + 12].to_bits(),
        ph2d_view_transform::wgsl::view_code(ph2d_view_transform::ViewTransform::Neutral),
        "o código da vista viaja pelos bits"
    );
    assert_eq!(g.dados[i + 16].to_bits(), 2, "a contagem viaja pelos bits");
}
