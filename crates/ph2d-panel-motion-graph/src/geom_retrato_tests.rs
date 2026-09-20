//! Gates da LEI do lado do retrato — ver [`super`].

use super::*;
use crate::geom::{PREVIEW_FRAME_H, PREVIEW_GAP, extensao_de};

/// Um cartão com retrato, alto `ALTURA`, no `x`/`y` pedidos.
const ALTURA: f32 = 100.0;
/// Um passo de coluna: deixa um vão de `150` entre corpos, que é **menos** que a régua da banda.
const PASSO_DA_COLUNA: f32 = ALTURA + 150.0;
/// Um passo de BANDA: deixa `200` de vão, que é exactamente a régua — e ela é ESTRITA.
const PASSO_DA_BANDA: f32 = ALTURA + super::BAND_GAP;

fn caixa(x: f32, y: f32) -> CaixaDoCartao {
    CaixaDoCartao {
        x,
        y,
        altura: ALTURA,
        retrato: true,
    }
}

/// ⭐⭐⭐ **O CARTÃO DO TOPO DE UMA COLUNA PÕE O RETRATO EM CIMA, e mais nenhum** — o report do
/// dono de 2026-09-20 (*«neste caso o preview deveria ser colocado para cima»*), reduzido à sua
/// forma mínima: três cartões empilhados na mesma coluna.
///
/// ⚠️ **As outras duas linhas da tabela são o CONTROLO e valem metade do gate:** sem elas, uma
/// lei que respondesse *«em cima»* a toda gente passava a primeira asserção — e punha a moldura
/// do cartão do meio exactamente no corredor de que ela o queria tirar.
#[test]
fn o_cartao_do_topo_de_uma_coluna_poe_o_retrato_em_cima() {
    let pilha = [
        caixa(0.0, 0.0),
        caixa(0.0, PASSO_DA_COLUNA),
        caixa(0.0, 2.0 * PASSO_DA_COLUNA),
    ];
    assert_eq!(
        retratos_em_cima(&pilha),
        vec![true, false, false],
        "só o topo tem o corredor por baixo e o espaço aberto por cima"
    );
}

/// **Um cartão SOZINHO na coluna fica em baixo** — os dois lados são «fora», e a omissão é a que
/// não surpreende ninguém. FALSIFICADO por uma lei que responda à pergunta *«há alguém por
/// baixo?»* sem a metade *«e ninguém por cima?»*.
#[test]
fn um_cartao_sozinho_na_coluna_fica_em_baixo() {
    assert_eq!(retratos_em_cima(&[caixa(0.0, 0.0)]), vec![false]);
}

/// ⚠️ **«Na mesma coluna» é SOBREPOSIÇÃO EM `x`, e a régua é o corpo do cartão.** Dois cartões
/// afastados por mais do que uma largura não se vêem um ao outro, logo os dois ficam em baixo —
/// e um pixel de sobreposição já os põe na mesma coluna.
///
/// FALSIFICADO por perguntar só `c.x == o.x` (um passo de coluna de meia largura deixaria de
/// contar) ou por não perguntar `x` nenhum (toda a tela viraria uma coluna só).
#[test]
fn a_coluna_mede_se_pela_sobreposicao_em_x() {
    let longe = [caixa(0.0, 0.0), caixa(CARD_W, PASSO_DA_COLUNA)];
    assert_eq!(
        retratos_em_cima(&longe),
        vec![false, false],
        "encostados sem sobrepor não partilham coluna"
    );
    let tocam = [caixa(0.0, 0.0), caixa(CARD_W - 1.0, PASSO_DA_COLUNA)];
    assert_eq!(
        retratos_em_cima(&tocam),
        vec![true, false],
        "um pixel de sobreposição já é a mesma coluna"
    );
}

/// **Um cartão SEM retrato nunca é virado** — a pergunta só tem sentido para quem desenha uma
/// moldura, e responder `true` reservaria espaço para uma moldura que ninguém pinta.
#[test]
fn um_cartao_sem_retrato_nunca_e_virado() {
    let sem = CaixaDoCartao {
        retrato: false,
        ..caixa(0.0, 0.0)
    };
    assert_eq!(
        retratos_em_cima(&[sem, caixa(0.0, PASSO_DA_COLUNA)]),
        vec![false, false]
    );
}

/// ⭐⭐ **UM CARTÃO DE OUTRA BANDA NÃO CONTA** — a régua é [`super::BAND_GAP`] e é ESTRITA, que é
/// exactamente o que separa *«o cartão seguinte da minha corrente»* de *«o primeiro cartão do
/// pedaço de grafo a seguir»*. Sem ela, só o cartão do topo do ECRÃ subia o retrato e os outros
/// painéis liam o cartão da banda anterior como se fosse do corredor deles.
///
/// FALSIFICADO por tirar a régua (o de baixo deixa de ler o de cima e sobe o retrato para dentro
/// do vão da banda) ou por a pôr não-estrita.
#[test]
fn um_cartao_de_outra_banda_nao_conta() {
    let bandas = [caixa(0.0, 0.0), caixa(0.0, PASSO_DA_BANDA)];
    assert_eq!(
        retratos_em_cima(&bandas),
        vec![false, false],
        "a `{}` de vao os dois estao sozinhos na banda deles",
        super::BAND_GAP
    );
    // O CONTROLO: um pixel a menos de vao e eles voltam a ser vizinhos de coluna.
    let colados = [caixa(0.0, 0.0), caixa(0.0, PASSO_DA_BANDA - 1.0)];
    assert_eq!(retratos_em_cima(&colados), vec![true, false]);
}

/// **Um vizinho que se SOBREPÕE na vertical não conta para nenhum lado** — ele não está nem por
/// cima nem por baixo, e contá-lo como um dos dois faria a resposta depender de qual dos testes
/// corre primeiro. (Depois de uma arrumação isto não acontece — a lei separa os cartões de uma
/// coluna por `GAP_Y` —, mas uma disposição feita à mão chega aqui.)
#[test]
fn um_vizinho_que_se_sobrepoe_na_vertical_nao_conta() {
    let sobrepostos = [caixa(0.0, 0.0), caixa(0.0, ALTURA * 0.5)];
    assert_eq!(retratos_em_cima(&sobrepostos), vec![false, false]);
}

/// ⭐⭐ **A CAIXA E A EXTENSÃO CONCORDAM SOBRE A ALTURA DO CORPO** — a caixa é o corpo nu e a
/// extensão é o corpo mais a moldura, do lado em que ela fica. Esta é a costura entre a lei e a
/// reserva de espaço: se as duas divergirem, a arrumação reserva um lado e a moldura sai pelo
/// outro.
///
/// FALSIFICADO por `caixa_de` esquecer o `.max(CAPSULA_H)` que a extensão faz, ou por a extensão
/// deixar de tirar a moldura do lado pedido.
#[test]
fn a_caixa_e_a_extensao_concordam_sobre_o_corpo() {
    let (fileiras, params, readout) = (3.0, 2.0, true);
    let c = caixa_de(0.0, 0.0, fileiras, params, readout, true);
    let em_baixo = extensao_de("Shape", fileiras, params, readout, true, false);
    let em_cima = extensao_de("Shape", fileiras, params, readout, true, true);
    assert!(
        (em_baixo.top - 0.0).abs() < 1e-3,
        "em baixo nada sai por cima"
    );
    assert!(
        (em_baixo.bottom - c.altura - PREVIEW_GAP - PREVIEW_FRAME_H).abs() < 1e-3,
        "em baixo a extensão é o corpo mais a moldura"
    );
    assert!(
        (em_cima.bottom - c.altura).abs() < 1e-3,
        "em cima o corpo acaba onde a caixa acaba"
    );
    assert!(
        (em_cima.top + PREVIEW_GAP + PREVIEW_FRAME_H).abs() < 1e-3,
        "em cima a moldura sai por cima, e por exactamente o mesmo tanto"
    );
}

/// ⭐⭐ **A LEI CHEGA AO PINTOR E AO GESTO** — gate de TEXTO, e a razão de ele ser de texto está
/// medida: o retrato de um cartão é desenhado numa `vello::Scene` e não há régua nesta casa que
/// leia um rectângulo de lá. *Uma lei certa com o pintor a não a ler lê-se como uma lei ausente*
/// — e este repo já pagou essa forma quatro vezes noutras crates.
///
/// ⚠️ **As DUAS metades são precisas e as curas são diferentes:** a pintura escolhe onde a
/// moldura SAI, e a virada do cabeçalho parte do que está NA TELA (sem a segunda, o primeiro
/// clique sobre um cartão que a lei já pôs em cima é um no-op visual).
///
/// ⚠️ A lei é calculada **uma vez por quadro e sobre a tela INTEIRA** — um cartão fora do ecrã
/// continua a ser o vizinho de coluna que decide o lado de quem está dentro.
#[test]
fn a_lei_chega_ao_pintor_e_ao_gesto() {
    let paint = include_str!("paint_cards.rs");
    assert!(
        paint.contains("geom::retratos_em_cima_por_id(&p.snap.nodes)"),
        "o pintor calcula a lei uma vez por quadro, sobre a tela inteira"
    );
    assert!(
        paint.contains("p.state.preview_position(n.id, &retratos_em_cima)"),
        "e entrega a CADA cartao o lado que ela escolheu"
    );
    let interact = include_str!("interact.rs");
    assert!(
        interact.contains("toggle_preview_position(")
            && interact.contains("retratos_em_cima_por_id(&snap.nodes)"),
        "a virada do cabecalho parte do que esta NA TELA"
    );
}
