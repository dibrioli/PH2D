//! Os gates do gémeo em WGSL que NÃO precisam de placa.
//!
//! ⚠️ **A paridade a sério vive onde há dispositivo** (`ph2d-field-gpu`), e é lá que ela corre
//! contra a [`crate::halo`]. Aqui ficam as duas coisas que um texto pode afirmar e cuja ausência
//! seria **muda**: o contrato existir, e as funções que o chamador despacha estarem declaradas.

use super::{CADEIA, CORTE, fonte};

/// A lei inteira, com uma porta de brincar — é sobre ela que os censos correm.
fn lei() -> String {
    fonte("fn bl_le(i: u32) -> vec3<f32> { return vec3<f32>(f32(i)); }")
}

/// ⭐⭐ **A PORTA DO CONTRATO É CHAMADA** — sem isto a lei não lê dado nenhum e ninguém repara.
///
/// ⚠️ O `bl_le` é declarado por **quem usa** (é a mesma forma do `ceu_radiance` que o pintor do
/// campo exige). Se a lei deixar de o chamar, o shader do chamador compila na mesma — com uma
/// função por usar e um halo de zeros. *Este gate é o que torna esse silêncio audível.*
#[test]
fn a_lei_le_os_dados_pela_porta_do_chamador() {
    assert!(
        lei().contains("bl_le("),
        "a lei deixou de chamar a porta `bl_le` do chamador"
    );
    // ⭐ A lei não a DECLARA — quem a declara é o chamador, e a [`fonte`] é que a costura no meio.
    assert!(
        !CORTE.contains("fn bl_le(") && !CADEIA.contains("fn bl_le("),
        "a lei DECLAROU a porta: ela é de quem chama, senão o chamador não pode ligar buffer nenhum"
    );
    // ⚠️⚠️ **A ORDEM é a lei**: o `bl_corte` tem de vir ANTES da porta (o chamador chama-o de lá
    // dentro) e a `bl_amostra_uv` DEPOIS (ela chama a porta). Uma const única não conseguia as duas.
    assert!(
        CORTE.contains("fn bl_corte(") && !CORTE.contains("bl_le("),
        "o CORTE tem de estar antes da porta e não pode lê-la"
    );
    assert!(
        CADEIA.contains("bl_le(") && !CADEIA.contains("fn bl_corte("),
        "a CADEIA vem depois da porta e é ela quem a lê"
    );
    // ⛔ Um `textureSample` aqui seria a segunda amostragem da mesma lei — ver a nota do módulo.
    assert!(
        !lei().contains("textureSample"),
        "a lei passou a amostrar por sampler: a paridade com a bilinear da CPU deixa de fechar"
    );
}

/// ⭐⭐ **CADA FUNÇÃO DA CADEIA É DECLARADA UMA VEZ** — e a lista é a dos quatro passos do
/// [`crate::halo`] mais a amostragem.
///
/// ⚠️ Uma a menos e o shader do chamador **não compila** (falha alto, e isso é bom); uma a MAIS e
/// o WGSL escolhe uma delas em silêncio.
#[test]
fn as_cinco_funcoes_existem_e_uma_vez_cada() {
    for nome in [
        "fn bl_amostra_uv(",
        "fn bl_corte(",
        "fn bl_desce13(",
        "fn bl_sobe_tenda(",
        "fn bl_cor(",
    ] {
        assert_eq!(
            lei().matches(nome).count(),
            1,
            "a lei declara {nome} {} vezes",
            lei().matches(nome).count()
        );
    }
}

/// ⭐⭐⭐ **OS PESOS SÃO OS DA CPU** — lidos do FONTE dela, não escritos à mão aqui.
///
/// ⛔⛔ Sem isto, os dois lados divergem no dia em que alguém afinar um peso de um lado só — e o
/// modo de falha é **mudo**: a imagem muda um pouco e nenhuma suíte reprova (a paridade a sério é
/// `#[ignore]`, logo o CI nunca a corre). *Um gate que corre em toda máquina vale mais do que um
/// que corre numa.*
///
/// ⚠️ A régua é de PRESENÇA e não de posição: o `cargo fmt` reescreve a expressão da CPU quando ela
/// cresce, e um censo por linha casaria zero sobre produto certo.
#[test]
fn os_pesos_da_cadeia_sao_os_do_fonte_da_cpu() {
    let cpu = include_str!("lib.rs");
    // Os quatro pesos do 13-tap, os três da tenda e os três da luminância.
    for peso in [
        "0.125", "0.031_25", "0.0625", "4.0", "2.0", "16.0", "0.2126", "0.7152", "0.0722",
    ] {
        assert!(cpu.contains(peso), "o fonte da CPU perdeu o peso {peso}");
        // ⚠️ O WGSL não escreve `0.031_25` — o separador de dígitos é de Rust.
        let no_wgsl = peso.replace('_', "");
        assert!(
            lei().contains(&no_wgsl),
            "o gémeo em WGSL não tem o peso {no_wgsl} que a CPU usa"
        );
    }
}
