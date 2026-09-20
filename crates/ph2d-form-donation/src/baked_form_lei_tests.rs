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

/// ⛔⛔⛔ **A OCLUSÃO DE FORMA É INERTE NESTA LEI — e quem o descobriu foi uma mutação SOBREVIVENTE.**
///
/// A mutação que apaga a leitura da textura de oclusão no [`super::passe_da_forma`] (`occ = 1.0`)
/// **sobreviveu** à paridade no pixel, com `pior = 0` bytes. ⚠️ **E não é a fixtura que não contém o
/// fenómeno: é a LEI.** A [`ph2d_form_pbr::acende_texel`] aplica a oclusão a **um** termo —
/// `albedo × ambiente × oclusão` — e o ambiente desta lei é **zero por desenho** (o rig desta casa é
/// `KEY + 3 × FILL`, ver o [`super::AMBIENTE_DA_FORMA`]) ⇒ *o canal é multiplicado por zero antes de
/// chegar a um pixel.*
///
/// ⏳ **ABERTO, e é do dono:** o objecto assado GUARDA a cavidade × os dois AOs (`form_occ`), a lei
/// da TINTA lê-a, e esta **não**. Parte do *«a sombra mais funda»* que o smoke das duas leis
/// mostrou pode ser isto — e a cura não é inventar aqui um termo que nenhuma referência declara
/// (`docs/Render3d/15` §7).
///
/// ⚠️ **As DUAS metades:** a de cima afirma a inércia (o que a mutação mede) e a de baixo é o
/// CONTROLO que impede o gate de ser vácuo — com um ambiente **não** nulo o canal move pixels. ⇒ no
/// dia em que esta lei ganhar ambiente, a metade de cima reprova e a premissa morre à vista no diff.
#[test]
fn a_oclusao_de_forma_e_inerte_enquanto_o_ambiente_for_zero() {
    let lado = 16u32;
    let n = (lado * lado) as usize;
    let base = vec![200u8; n * 4];
    // Uma forma virada ao ecrã, com cobertura cheia — o regime em que a lei de facto acende.
    let mut form = vec![0.0f32; n * 4];
    for t in form.as_chunks_mut::<4>().0 {
        *t = [0.0, 0.0, 1.0, 1.0];
    }
    let cheia = vec![1.0f32; n];
    // ⚠️ Uma oclusão que VARIA — uma constante diferente de `1` seria indistinguível de um albedo
    // mais escuro, e o que se mede é se o CANAL chega.
    let variada: Vec<f32> = (0..n).map(|i| (i % 8) as f32 / 8.0).collect();

    let material = crate::lei_da_luz::material_da_forma();
    let rig = ph2d_light::LightRig::default();
    let lampadas = super::lampadas_do_rig(&rig).expect("o rig de fábrica tem lâmpada acesa");
    let acende = |occ: &[f32], ambiente| {
        ph2d_form_pbr::imagem::acende_imagem(
            &material,
            &ph2d_form_pbr::imagem::Planos {
                size: (lado, lado),
                base: &base,
                form: &form,
                form_occ: occ,
            },
            &lampadas,
            ambiente,
            crate::lei_da_luz::OLHAR_DA_FORMA,
        )
        .expect("acende")
    };

    assert_eq!(
        acende(&cheia, super::AMBIENTE_DA_FORMA),
        acende(&variada, super::AMBIENTE_DA_FORMA),
        "com o ambiente a ZERO a oclusão de forma não pode mover um único byte"
    );
    // **O CONTROLO**: com ambiente, ela move — senão este gate ficaria verde sobre um canal que
    // ninguém liga a nada.
    assert_ne!(
        acende(&cheia, [0.5; 3]),
        acende(&variada, [0.5; 3]),
        "controlo: com ambiente a oclusão TEM de mover pixels, senão a inércia acima é vácuo"
    );
}
