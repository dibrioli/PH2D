//! ⭐⭐⭐⭐ **O VEREDITO DO GESTO DE ASSAR** — a metade que uma FOTO do dono encomendou (21/09) e
//! que uma **mutação sobrevivente** obrigou a sair para onde pudesse ser afirmada.
//!
//! ⛔⛔ **O defeito que o dono fotografou:** `✓ [sculpt3d] nao assou: this sprite is fully tra…` —
//! o visto **verde** de sucesso a anunciar que o gesto não tinha corrido. A causa era de tipo: a
//! porta devolvia uma `String`, e a fase não tinha como saber o que tinha acontecido.
//!
//! ⛔⛔⛔ **E a cura ficou sem régua durante uma hora:** com o [`super::Veredito`] no sítio, trocar
//! o `Err(…) => Recusado` por `Assado` passava os **dois** gates de GPU do gesto e a suíte inteira.
//! ⚠️ *Os dois só percorrem o caminho que ASSA — um gate que só corre o caminho feliz não afirma
//! nada sobre a cara de uma falha*, e a cara de uma falha era exactamente o report.

use super::bake::Veredito;

/// **UM ERRO É UMA RECUSA, E UM `Ok` É UM BAKE** — a tradução, sem device nenhum.
///
/// ⚠️ **As três metades, e cada uma mata uma mutação diferente:** a cara do erro · a cara do
/// sucesso (um gate que só exigisse a primeira deixava passar um app que anuncia todo bake como
/// falha) · e a frase do erro **carregar a razão dentro**, senão a recusa volta a ser muda.
#[test]
fn um_erro_do_gesto_e_uma_recusa_e_um_ok_e_um_bake() {
    let recusa = Veredito::do_gesto(Err("a cena nao tem malha".into()), "");
    assert!(
        !recusa.assou(),
        "um Err do gesto TEM de se declarar recusa: e' isso que escolhe a cara do aviso"
    );
    assert!(
        recusa.frase().contains("a cena nao tem malha"),
        "a recusa perdeu a RAZAO pelo caminho: {}",
        recusa.frase()
    );

    let ok = Veredito::do_gesto(Ok((64, 64, 0)), "");
    assert!(ok.assou(), "um Ok do gesto assou");
    assert!(
        ok.frase().contains("64x64"),
        "o aviso do bake diz o tamanho: {}",
        ok.frase()
    );
}

/// ⭐⭐ **UM SPRITE QUE VESTIU A PEÇA DIZ-LO** — a frase é OUTRA, e não a mesma com um número.
///
/// ⚠️ **Porquê duas frases:** quando o sprite estava vazio o que aconteceu não foi *«assei a tua
/// arte»*, foi *«ele não tinha arte nenhuma, por isso vestiu a peça»* — e o artista tem de o saber
/// para não pintar por cima sem perceber porquê (ver [`super::albedo::veste_a_forma`]).
///
/// **Mutação que deve sangrar:** os dois braços `Ok` colapsados num só.
#[test]
fn um_sprite_que_vestiu_a_peca_diz_lo() {
    let normal = Veredito::do_gesto(Ok((32, 32, 0)), "");
    let vestido = Veredito::do_gesto(Ok((32, 32, 4096)), "");
    assert!(normal.assou() && vestido.assou());
    assert_ne!(
        normal.frase(),
        vestido.frase(),
        "vestir a peca tem de ser dito: as duas frases sao a mesma"
    );
    assert!(
        vestido.frase().contains("4096"),
        "a frase do vestido diz QUANTOS texels vestiram a peca: {}",
        vestido.frase()
    );
    // ⭐ O CONTROLO: a frase normal NÃO fala de vestir — sem isto, duas frases iguais com o
    // número colado passariam na asserção de cima.
    assert!(
        !normal.frase().contains("4096"),
        "a frase do bake normal nao fala de texels vestidos: {}",
        normal.frase()
    );

    // ⛔⛔ **A metade que uma MUTAÇÃO SOBREVIVENTE obrigou:** trocar a chave do braço normal pela
    // do vestido deixava as duas frases **diferentes** (uma com `4096`, outra com o marcador
    // `{vestidos}` **por resolver**) e passava em tudo acima. ⚠️ *Um marcador que chega ao ecrã
    // com as chavetas dentro é sempre um defeito*, e é ele que denuncia a chave errada.
    for v in [&normal, &vestido] {
        assert!(
            !v.frase().contains('{'),
            "a frase chegou ao artista com um marcador por resolver (a chave nao casa com os \
             argumentos): {}",
            v.frase()
        );
    }
}

/// ⭐⭐⭐⭐ **O PADRÃO VEM DO QUE O ARTISTA VÊ, NÃO DO QUE O SPRITE GUARDA** — e desde 21/09 isto é
/// uma LEI MEDIDA e já não um censo de texto.
///
/// ⚠️ **Ela era afirmada lendo o escrutínio de um `match` no corpo da fase da shell**
/// (`the_brush_pattern_reads_the_live_layers_before_the_stored_image`), e a catraca da shell
/// trouxe a lei para esta crate. ⭐ *Um gate que percorre a porta é melhor do que um que lê o texto
/// de quem a chama* — e este CONTA as leituras, que é a única forma de provar a ordem.
///
/// O report que a originou: Enio, 2026-08-09 — *«veja a textura ao lado e veja a textura no
/// preview»*. Um sprite cuja aparência nasce das CAMADAS do Painter continua a apontar para a
/// imagem de origem, e ler a origem devolve outra textura.
///
/// **Mutações que devem sangrar:** a ordem invertida · o `or_else` trocado por `or(`.
#[test]
fn o_padrao_pergunta_as_camadas_vivas_antes_da_imagem_guardada() {
    use std::cell::Cell;
    let img = || ph2d_sculpt3d::AlphaImage::from_rgba(2, 2, &[128; 16]);

    // ⭐ **Com camadas vivas, a imagem guardada NEM É LIDA** — é isso que o `or_else` compra, e uma
    // leitura de sprite é um `readback` à placa.
    let (c, s) = (Cell::new(0), Cell::new(0));
    let achada = super::alpha_pedido::escolhe_a_fonte(
        &mut || {
            c.set(c.get() + 1);
            img()
        },
        &mut || {
            s.set(s.get() + 1);
            img()
        },
    );
    assert!(achada.is_some(), "as camadas vivas dao um padrao");
    assert_eq!((c.get(), s.get()), (1, 0), "a ordem inverteu-se");

    // ⭐ E sem camadas, a imagem guardada É o padrão — senão um sprite sem documento vivo deixava
    // de servir.
    let (c2, s2) = (Cell::new(0), Cell::new(0));
    let achada = super::alpha_pedido::escolhe_a_fonte(
        &mut || {
            c2.set(c2.get() + 1);
            None
        },
        &mut || {
            s2.set(s2.get() + 1);
            img()
        },
    );
    assert!(achada.is_some(), "a imagem guardada da' um padrao");
    assert_eq!(
        (c2.get(), s2.get()),
        (1, 1),
        "as duas fontes sao perguntadas quando a 1a nao responde"
    );

    // ⭐ E o CONTROLO: sem nenhuma das duas, não há padrão — senão o gate acima passaria com uma
    // lei que devolvesse sempre `Some`.
    assert!(
        super::alpha_pedido::escolhe_a_fonte(&mut || None, &mut || None).is_none(),
        "sem fonte nenhuma nao ha' padrao"
    );
}
