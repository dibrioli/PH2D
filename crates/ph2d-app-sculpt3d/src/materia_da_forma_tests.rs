//! ⭐⭐⭐⭐ **A CORRENTE DA MATÉRIA — do gesto que assa até à rota B, elo a elo.**
//!
//! ⛔⛔ **Ela existe por um report do dono** (2026-09-21, foto com uma seta): *«o algoritmo que vc
//! criou tem esse fundo branco na sprite transparente. logo que roda o objeto o fundo aparece. OU
//! seja: parece que vc criou uma máscara»*. A causa não era uma máscara — era a silhueta que o
//! PRIMEIRO bake gravou no `base`, com a forma a ser re-rasterizada por quadro debaixo dela.
//!
//! A LEI está medida na [`ph2d_form_pbr::imagem`] (o alfa segue a cobertura, o albedo é o neutro) e
//! o PIXEL no [`crate::vivo_tests`], com adaptador. ⚠️ **O que nenhum dos dois vê é a FIAÇÃO**: o
//! facto nasce no gesto de assar, viaja no documento, e tem de chegar à porta da rota B atravessando
//! **quatro** sítios em duas crates. *Um motor com a lei certa e a shell a não a ligar lê-se como um
//! motor sem a lei* — a forma que esta casa já registou meia dúzia de vezes.
//!
//! ⚠️ **O gate é TEXTUAL de propósito, e a fronteira é honesta:** o primeiro elo vive numa função
//! que pede um `wgpu::Device` e uma malha, e o terceiro numa fase de quadro — nenhum dos dois é
//! alcançável de um teste sem placa, e um gate de placa é `#[ignore]`, logo o CI nunca o correria.
//! ⭐ Os `include_str!` são o que o torna honesto: se um destes ficheiros mudar de nome ou de sítio,
//! isto deixa de **COMPILAR** em vez de ficar verde a medir menos (a cegueira §2.7 do HOWTO, cujo
//! modo de falha é MUDO).

/// Os quatro elos, do gesto até à porta. ⚠️ **Cada entrada é `(ficheiro, agulha, quantos, porquê)`**,
/// e a agulha é a EXPRESSÃO que liga o elo — nunca o nome do campo solto, que apareceria também numa
/// declaração ou num doc-comment.
///
/// ⚠️⚠️ **A COLUNA DA CONTAGEM não é decoração, e foi o gate a reprovar que a pôs lá:** o último elo
/// casa **duas** vezes, porque o objecto assado tem DUAS rotas — a da placa (`acende_pela_forma`) e
/// a da régua em Rust (`planos_de`, que é o caminho de referência contra o qual a paridade mede).
/// *Um `>= 1` teria engolido isto e passaria também no dia em que uma das duas perdesse o elo.*
const CORRENTE: [(&str, &str, usize, &str); 4] = [
    (
        include_str!("bake.rs"),
        "materia_da_forma: vestidos > 0,",
        1,
        "o FACTO nasce no gesto que assa: se o vestir armou, o sprite não tinha arte",
    ),
    (
        include_str!("vivo_fase.rs"),
        "materia_da_forma: assado.materia_da_forma,",
        1,
        "a fase do quadro entrega-o à porta da rota B",
    ),
    (
        include_str!("../../ph2d-form-donation/src/baked_form/forma_viva.rs"),
        "materia_da_forma: alvo.materia_da_forma,",
        1,
        "a rota B (o catavento) passa-o à lei",
    ),
    (
        include_str!("../../ph2d-form-donation/src/baked_form.rs"),
        "materia_da_forma: bake.materia_da_forma,",
        2,
        "e o assado leva-o nas DUAS rotas dele: a da placa e a da régua em Rust — senão a \
         paridade media duas leis diferentes e chamava-lhe um defeito de shader",
    ),
];

/// ⭐⭐⭐ **O veredito, elo a elo.**
#[test]
fn a_materia_da_forma_atravessa_do_gesto_que_assa_ate_a_rota_b() {
    for (fonte, agulha, quantos, porque) in CORRENTE {
        let n = fonte.matches(agulha).count();
        assert_eq!(
            n, quantos,
            "o elo `{agulha}` casou {n} vezes e devia casar {quantos} — {porque}. Sem ele o dono \
             vê a peça a rodar por baixo do recorte que o primeiro bake lhe deu."
        );
    }
}

/// ⛔⛔ **E o CONTROLO da própria extracção:** uma agulha que NÃO está lá tem de ler zero.
///
/// ⚠️ Sem esta metade, um `matches` partido (um `include_str!` a apontar para o ficheiro errado que
/// por acaso contivesse as quatro linhas, uma busca que devolvesse sempre `1`) deixaria o gate
/// acima verde a afirmar **nada**. *Um censo que não acha nada acusa tudo; um que acha sempre não
/// acusa nada, e é este o lado que passa despercebido.*
#[test]
fn o_censo_da_corrente_sabe_dizer_que_nao() {
    // ⚠️ Montada em pedaços: escrita como um literal, ela seria ela própria um elo da corrente
    // dentro deste ficheiro — e este ficheiro não está no censo, mas o hábito é o que protege o dia
    // em que alguém o acrescentar.
    let ausente = concat!("materia_da_forma: ", "isto_nao_existe,");
    for (fonte, agulha, _, _) in CORRENTE {
        assert_eq!(fonte.matches(ausente).count(), 0, "controlo sobre {agulha}");
    }
    assert_eq!(CORRENTE.len(), 4, "o piso de população da corrente");
}

/// ⭐⭐⭐ **A CENA DO CATAVENTO PEDE UMA TELA TRANSPARENTE, e a do bake pede branco.**
///
/// ⛔⛔ **Sem esta metade a wave inteira é invisível ao dono:** o `albedo::veste_a_forma` só arma
/// quando **nenhum** texel do sprite tem alfa, logo sobre uma tela BRANCA a matéria nunca é a forma
/// e a `=53` mostraria a peça dentro de um cartão — *a cena não conteria o fenómeno que ela existe
/// para julgar*, que é a espécie que o `CLAUDE.md` §5.0 chama de pior que uma cena ausente.
///
/// ⚠️ **E os dois números são atados ao SIGNIFICADO deles**, não afirmados soltos: a escada vive no
/// `ph2d_image_import::spawn_blank_canvas`, e um gate que dissesse `== 0` continuaria verde no dia
/// em que alguém renumerasse as três escolhas do diálogo de imagem nova.
#[test]
fn a_cena_do_catavento_pede_a_tela_em_que_a_peca_sai_recortada() {
    assert_eq!(
        crate::donation::fundo_da_tela(true),
        0,
        "a =53 tem de pedir uma tela TRANSPARENTE — sobre branco o vestir nunca arma"
    );
    assert_eq!(
        crate::donation::fundo_da_tela(false),
        2,
        "controlo: a =11 continua a julgar a LUZ sobre branco, que é o neutro multiplicativo"
    );

    // ⭐ **A escada, lida de quem a implementa.** As agulhas são montadas — ver o irmão acima.
    let escada = include_str!("../../ph2d-image-import/src/lib.rs");
    for (valor, agulha, o_que) in [
        (0u8, concat!("_ => [0, 0, 0, ", "0],"), "transparente"),
        (2u8, concat!("2 => [255, 255, 255, ", "255],"), "branco"),
    ] {
        assert!(
            escada.contains(agulha),
            "o `{valor}` da escada do `spawn_blank_canvas` já não é {o_que} — os dois números \
             acima passaram a dizer outra coisa"
        );
    }
    // ⛔ E o elo: a porta da cena tem de ser quem o `canvas_wanted` consulta.
    let fonte = include_str!("donation.rs");
    assert_eq!(
        fonte
            .matches(concat!(
                "bg: fundo_da_tela(",
                "super::scenes::catavento_scene()),"
            ))
            .count(),
        1,
        "o `canvas_wanted` tem de perguntar à LEI — um literal ali e a escolha da cena evapora"
    );
}
