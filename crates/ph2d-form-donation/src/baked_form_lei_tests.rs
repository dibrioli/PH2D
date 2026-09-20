//! ⭐⭐ **OS GATES DA FIAÇÃO DA LEI** — o que se pode afirmar sem um `Device`.
//!
//! ⚠️ **A acendida em si precisa de placa** (`upload_rgba` e `copy_texture_into_individual` são
//! `wgpu`), logo a comparação de PIXELS entre as duas leis é um gate `#[ignore]` que só corre com
//! adapter. O que mora aqui é a metade que é aritmética: *que a porta com a lei dita existe, que a
//! porta do produto a chama, e que a lei de fábrica continua a ser a de sempre.*

/// ⛔⛔ **A PORTA DO PRODUTO DELEGA** — e sem isto o `light()` podia ficar com uma terceira
/// redacção da escolha.
///
/// ⚠️ A régua é o TEXTO porque a função pede um `GpuContext`, um `SpriteRenderer` e um passe — ela
/// **não é alcançável de um teste** sem placa, e é exactamente a família de defeito que esta casa
/// já pagou quatro vezes: *um motor com a lei certa e a porta a não a ligar lê-se como um motor sem
/// a lei*.
#[test]
fn a_porta_do_produto_pergunta_ao_ambiente_e_delega() {
    let fonte = include_str!("baked_form.rs");

    // ⚠️⚠️ **A agulha é lida DENTRO do corpo da `light`, e em DOIS pedaços** — a 1.ª redacção
    // procurava `"acende_com(crate::lei_da_luz::do_ambiente()"` como uma corrida só, e o `cargo fmt`
    // partiu a chamada em sete linhas. *Uma agulha textual que o formatador pode partir mede o
    // formatador.* Ela falhou ALTO, que é a sorte da história; a cura é não depender do espaçamento.
    let i = fonte
        .find("pub fn light(")
        .expect("controlo: a porta do produto tem de existir com este nome");
    let corpo = &fonte[i..i + fonte[i..]
        .find("\npub fn acende_com(")
        .expect("controlo: a irmã vem a seguir")];
    for pedaco in ["acende_com(", "crate::lei_da_luz::do_ambiente()"] {
        assert!(
            corpo.contains(pedaco),
            "o corpo do `light` tem de conter `{pedaco}` — ele é quem pergunta ao ambiente"
        );
    }

    // E os DOIS braços têm de estar na porta que recebe a lei, não espalhados.
    for braco in [
        "Lei::Tinta => acende_pela_tinta",
        "Lei::Forma => acende_pela_forma",
    ] {
        assert!(fonte.contains(braco), "falta o braço `{braco}`");
    }

    // **O CONTROLO da régua**: uma agulha que *não* está lá tem de falhar, senão um `contains`
    // sobre um ficheiro que mudou de nome passaria por vácuo.
    assert!(
        !fonte.contains("Lei::Vidro =>"),
        "controlo: a régua tem de poder dizer NÃO"
    );
}

/// ⚠️ **A lei nova NÃO entra pela rota que o passe da tinta usa** — ela não toca no
/// `ImpastoLightPass`, e é isso que garante que ligar uma não pode mudar a outra.
#[test]
fn a_lei_nova_nao_toca_no_passe_da_tinta() {
    let fonte = include_str!("baked_form.rs");
    let i = fonte
        .find("fn acende_pela_forma(")
        .expect("controlo: a função tem de existir com este nome");
    let j = fonte[i..]
        .find("\nfn acende_pela_tinta(")
        .expect("controlo: a irmã tem de vir a seguir");
    let corpo = &fonte[i..i + j];
    assert!(
        !corpo.contains("ImpastoLightPass") && !corpo.contains("SpecLut"),
        "o caminho da forma não pode mencionar o passe da tinta"
    );
    // **O CONTROLO**: o caminho da TINTA menciona-o — senão a asserção acima seria sobre um
    // ficheiro onde aquele nome já não existe em lado nenhum.
    assert!(
        fonte[i + j..].contains("ImpastoLightPass"),
        "controlo: o caminho da tinta TEM de usar o passe da tinta"
    );
}
