//! ⭐ **OS GATES do caminho da EXTRACÇÃO** — o irmão de
//! [`sculpt3d_history_retopo_extract`].
//!
//! ⚠️ **Ele existe por causa do tecto de LOC do shell (HR-18)**, e o corte é o idioma da
//! casa: o `mod tests` inline vai para o irmão **do assunto**. *O ficheiro do produto
//! guarda o que o botão faz; este guarda o que se prova sobre ele.*
//!
//! ⚠️ **Sem `use super::*`, de propósito:** todo gate aqui chama o irmão pelo prefixo
//! (`super::extract_from`, `super::boundary_edges`, `super::worse`), então o glob era
//! morto — e um `unused_imports` é **erro** sob o `-D warnings` do `ship.sh`, não aviso.

/// ⭐⭐⭐ **GATE 11 — o caminho antigo continua byte-idêntico enquanto o
/// interruptor estiver desligado.**
///
/// ⚠️ **A decisão é pura de propósito.** O gesto em si precisa de GPU (a cena
/// segura buffers de device), então um gate sobre ele é `skip` gracioso na
/// máquina sem adapter — e *skip gracioso não é verde*. O que se pina aqui é a
/// **decisão**, que é a única coisa que a env acrescenta ao caminho de sempre.
#[test]
fn o_caminho_novo_e_o_de_omissao_e_so_o_zero_o_desliga() {
    for (value, want) in [
        // ⭐⭐ O caso por omissão VIROU em 2026-08-25 (ordem do dono do produto): é o
        // caminho NOVO que o Enio recebe sem configurar nada. *A lei «shipa
        // desligado» valeu enquanto ele não fechava a casca; ele fecha.*
        (None, true),
        // ⚠️ E o `"0"` é a ÚNICA palavra que desliga — quem quer o de sempre tem de
        // o pedir por este nome exacto.
        (Some("0"), false),
        (Some("1"), true),
        (Some("sim"), true),
        (Some(""), true),
    ] {
        assert_eq!(
            super::extract_from(value),
            want,
            "PH2D_RETOPO_EXTRACT={value:?} tinha de dar {want}"
        );
    }
}

/// ⭐⭐⭐ **A ORDEM DO CRITÉRIO: furos primeiro, e ela é a decisão de produto.**
///
/// ⛔⛔ Uma ordem que pusesse o enviesamento à frente escolheria *a peça mais bonita com
/// um buraco na ponta* — e «furos nas pontas» foi a queixa do artista **três vezes
/// seguidas**. ⚠️ *Nada no tipo impede trocar a ordem: são três números da mesma peça.*
#[test]
fn a_escolha_poe_os_furos_a_frente_do_enviesamento() {
    // Uma peça FECHADA e uma com bordo — o cubo de quads da casa, e um quad solto.
    let fechada = ph2d_mesh::shapes::cube(1.0);
    let furada = ph2d_mesh::Mesh::from_parts(
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ],
        vec![ph2d_mesh::Face::quad(0, 1, 2, 3)],
    )
    .expect("a fixtura e' construida aqui");
    assert_eq!(
        super::boundary_edges(&fechada),
        0,
        "⛔ a fixtura fechada tem de FECHAR, senao o gate compara duas peças furadas"
    );
    assert_eq!(
        super::boundary_edges(&furada),
        4,
        "⛔ a fixtura furada tem de CONTER o fenomeno"
    );

    // A furada e' PIOR mesmo com enviesamento perfeito contra uma fechada horrivel.
    assert!(
        super::worse(&furada, 0, 0.0, &fechada, 999, 89.0),
        "⛔ os FUROS tem de vir antes do enviesamento"
    );
    // Empatados nos furos, decide a contagem de faces >60.
    assert!(
        super::worse(&fechada, 10, 0.0, &fechada, 2, 89.0),
        "⛔ empatados nos furos, decide o >60"
    );
    // Empatados nos dois, decide a mediana.
    assert!(
        super::worse(&fechada, 3, 9.0, &fechada, 3, 8.0),
        "⛔ empatados nos dois, decide a mediana"
    );
    assert!(
        !super::worse(&fechada, 3, 8.0, &fechada, 3, 8.0),
        "⛔ iguais nao podem ser PIORES -- a comparacao tem de ser estrita"
    );
}

/// ⭐⭐⭐ **O CAMINHO DA EXTRACÇÃO TEM ACABAMENTO — e ele pousa na ESCULTURA.**
///
/// ⛔⛔ **As duas metades são precisas, e a segunda defende o defeito que já custou o
/// produto inteiro.** Em 2026-08-21 a porta do shell passou ao `fill` a malha original
/// onde ele esperava a **indexada**, e os quatro números do relatório saíram
/// **bit-a-bit iguais** aos da corrida correta — o dano era só geométrico. Aqui a
/// direcção é a oposta e o erro seria o mesmo: alisar contra a `work` (a remalhada)
/// somaria os dois erros e apagaria o relevo que o F1 já arredondou.
///
/// ⚠️ **O gate LÊ O FONTE** pela mesma razão que o irmão dele abaixo: um alisamento que
/// desapareça, ou que troque de superfície, compila e passa a suíte inteira.
#[test]
fn a_extraccao_alisa_contra_a_escultura_e_nao_contra_a_remalhada() {
    let src = include_str!("sculpt3d_history_retopo_extract.rs");
    // ⚠️ **O token vem partido de propósito:** este gate lê o ficheiro em que ele
    // próprio vive, e um literal inteiro contar-se-ia a si mesmo. *Um gate que se conta
    // nunca mede o produto.*
    let call = concat!("ph2d_quadfill::", "finish_extracted(");
    let n = src.matches(call).count();
    assert_eq!(
        n, 1,
        "o caminho da extraccao chama o acabamento {n} vezes; tem de ser UMA -- ver o \
         doc do `ph2d_quadfill::fill` e o defeito de 2026-08-21"
    );
    let full = concat!(
        "ph2d_quadfill::",
        "finish_extracted(&mut out, &reference)"
    );
    assert!(
        src.contains(full),
        "⛔⛔ o acabamento tem de pousar na `reference` (a ESCULTURA) e nao na `work` \
         (a remalhada)"
    );
    // ⛔⛔ **E o alisamento CRU não pode voltar por uma segunda porta.** Em 2026-08-28 o
    // Laplaciano passou a ser a *ronda zero* de `finish_extracted`; uma chamada solta aqui
    // seria um segundo acabamento a correr por cima do primeiro, e as duas passariam neste
    // ficheiro sem se verem.
    assert_eq!(
        src.matches(concat!("ph2d_quadfill::", "smooth(")).count(),
        0,
        "⛔ o alisamento cru voltou a este caminho -- ele vive dentro de `finish_extracted`"
    );
}

/// ⭐⭐ **E A BIFURCAÇÃO É UMA SÓ** — o que faz o «byte-idêntico» ser
/// verificável em vez de prometido.
///
/// ⚠️ **O gate LÊ O FONTE**, e é de propósito: um segundo sítio a chamar
/// [`super::extract_requested`] compilaria, passaria a suíte, e partiria a
/// afirmação de que o caminho antigo está intocado. *Uma promessa sobre o
/// código não é uma propriedade do código até alguém a contar.*
#[test]
fn a_bifurcacao_para_o_caminho_novo_e_uma_so() {
    let src = include_str!("sculpt3d_history_retopo_global.rs");
    let n = src.matches("extract_requested()").count();
    assert_eq!(
        n, 1,
        "a cadeia global chama `extract_requested()` {n} vezes; tem de ser UMA, \
         na primeira linha da porta"
    );
    assert_eq!(
        src.matches("quad_remesh_extract(").count(),
        1,
        "e chama o caminho novo uma vez so'"
    );
}
