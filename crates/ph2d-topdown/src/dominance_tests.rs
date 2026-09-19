//! ⭐⭐⭐ **A ÚLTIMA SETA MANDA** — os gates da dominância em 4 direcções.
//!
//! Ordem do dono, 2026-09-15, depois do smoke aprovado: *«No 4 dir, mesmo com duas setas
//! pressionadas, a última a ser pressionada sempre é dominante»*.
//!
//! # ⛔ O que havia antes, MEDIDO
//!
//! Com as duas setas em baixo a intenção crua é exactamente diagonal, e o encaixe do quantizador
//! resolvia-a por **arredondamento** — `roundf(ang / 90°)`, que parte o empate para longe do zero.
//! O resultado é **função do QUADRANTE**, não do dedo:
//!
//! | setas | ângulo | `ang/90°` | `roundf` | sai |
//! |---|---|---|---|---|
//! | `→` + `↑` | `+45°` | `+0,5` | `+1` | **cima** |
//! | `→` + `↓` | `−45°` | `−0,5` | `−1` | **baixo** |
//! | `←` + `↑` | `+135°` | `+1,5` | `+2` | **esquerda** |
//! | `←` + `↓` | `−135°` | `−1,5` | `−2` | **esquerda** |
//!
//! *Duas das quatro davam vertical e duas horizontal, e nada disso é uma lei — é o sinal de um
//! `round` a meio caminho.* ⇒ o que o dono pediu não é só um comportamento novo: é **substituir um
//! artefacto de arredondamento por uma lei**.
//!
//! # ⚠️ O ORÁCULO não tem isto, e a ausência foi MEDIDA
//!
//! §0.9 manda correr o alvo. A triagem do plano §1.1 já parava no Godot (MIT, o único instalado), e
//! a API dele foi despejada (`godot --headless --doctool`) e varrida: o `Input` tem `get_axis`,
//! `get_vector` e `get_joy_axis` — **nenhuma porta que carregue ORDEM de pressão**, e zero acertos
//! para *last pressed* / *priority*. Um utilizador do Godot escreveria esta lei à mão com o
//! `is_action_just_pressed`, que é exactamente a **transição** que a nossa memória observa.
//! ⇒ *a lei é NOSSA e declarada*, como a metade da isometria — ⛔ nunca apresentada como paridade.

use super::*;
use direction::{DirectionMode, Dominance};

const EPS: f32 = 1.0e-5;

fn perto(a: Vec2, b: Vec2, o_que: &str) {
    assert!(
        (a[0] - b[0]).abs() < EPS && (a[1] - b[1]).abs() < EPS,
        "{o_que}: ({:.5}, {:.5}) contra ({:.5}, {:.5})",
        a[0],
        a[1],
        b[0],
        b[1]
    );
}

/// Segura uma sequência de intenções cruas e devolve a última direcção quantizada.
fn sequencia(passos: &[Vec2], modo: DirectionMode) -> Vec2 {
    let mut mem = Dominance::default();
    let mut fim = [0.0, 0.0];
    for &p in passos {
        fim = direction::quantize(p, modo, mem.observe(p));
    }
    fim
}

/// ⭐⭐⭐ **A ORDEM DO DONO, nos quatro quadrantes.**
#[test]
fn em_quatro_direccoes_a_ultima_seta_manda() {
    // `→` primeiro, depois `↑` ⇒ manda o `↑`.
    perto(
        sequencia(&[[1.0, 0.0], [1.0, 1.0]], DirectionMode::FourWay),
        [0.0, 1.0],
        "direita e depois cima",
    );
    // E ao contrário: `↑` primeiro, depois `→` ⇒ manda o `→`.
    perto(
        sequencia(&[[0.0, 1.0], [1.0, 1.0]], DirectionMode::FourWay),
        [1.0, 0.0],
        "cima e depois direita",
    );
    // ⚠️ **Os quadrantes em que o arredondamento acertava por acaso também têm de obedecer** —
    // sem isto o gate passaria em metade do corpus com a lei apagada.
    perto(
        sequencia(&[[0.0, -1.0], [-1.0, -1.0]], DirectionMode::FourWay),
        [-1.0, 0.0],
        "baixo e depois esquerda",
    );
    perto(
        sequencia(&[[-1.0, 0.0], [-1.0, -1.0]], DirectionMode::FourWay),
        [0.0, -1.0],
        "esquerda e depois baixo",
    );
}

/// ⭐⭐ **E ela SOBREVIVE enquanto as duas estiverem em baixo** — a dominância não pisca de tique
/// para tique.
#[test]
fn e_ela_nao_pisca_enquanto_as_duas_ficarem_em_baixo() {
    let mut mem = Dominance::default();
    let _ = direction::quantize([1.0, 0.0], DirectionMode::FourWay, mem.observe([1.0, 0.0]));
    for tique in 0..30 {
        let d = direction::quantize([1.0, 1.0], DirectionMode::FourWay, mem.observe([1.0, 1.0]));
        perto(
            d,
            [0.0, 1.0],
            &format!("tique {tique} com as duas em baixo"),
        );
    }
}

/// ⭐⭐ **Largar a dominante devolve o comando à que ficou** — e voltar a carregar rouba-o de novo.
#[test]
fn largar_a_dominante_devolve_o_comando_a_que_ficou() {
    let mut mem = Dominance::default();
    let modo = DirectionMode::FourWay;
    let _ = direction::quantize([1.0, 0.0], modo, mem.observe([1.0, 0.0]));
    perto(
        direction::quantize([1.0, 1.0], modo, mem.observe([1.0, 1.0])),
        [0.0, 1.0],
        "o `↑` chegou depois",
    );
    perto(
        direction::quantize([1.0, 0.0], modo, mem.observe([1.0, 0.0])),
        [1.0, 0.0],
        "largou-se o `↑` e sobra o `→`",
    );
    perto(
        direction::quantize([1.0, 1.0], modo, mem.observe([1.0, 1.0])),
        [0.0, 1.0],
        "carregou-se o `↑` outra vez",
    );
}

/// ⛔⛔ **A lei é do FOUR-WAY e de mais nenhum modo.**
///
/// ⚠️ Em 8 direcções a diagonal **é** a resposta certa — apagá-la ali seria tirar metade dos rumos
/// ao modo cuja razão de existir são eles.
#[test]
fn os_outros_modos_nao_mudam_uma_virgula() {
    for modo in [
        DirectionMode::Free,
        DirectionMode::EightWay,
        DirectionMode::AxisX,
        DirectionMode::AxisY,
    ] {
        let mut mem = Dominance::default();
        let _ = mem.observe([1.0, 0.0]);
        let com = direction::quantize([1.0, 1.0], modo, mem.observe([1.0, 1.0]));
        let sem = direction::quantize([1.0, 1.0], modo, direction::DominantAxis::None);
        assert_eq!(com, sem, "o modo {modo:?} mudou por causa da dominancia");
    }
}

/// ⭐⭐⭐ **O CARDEAL QUE A DOMINÂNCIA DEVOLVE É O MESMO, AO BIT, QUE A SETA SOZINHA DEVOLVE.**
///
/// ⚠️ ⛔ Esta é a que não se adivinha: `libm::cosf(π/2)` é `−4,4e-8`, **não** zero, e o encaixe é
/// feito por `cos`/`sin` de um ângulo. Uma cura que devolvesse um `[0.0, 1.0]` exacto criaria
/// **duas aritméticas** para o mesmo rumo — a mesma direcção com dois valores conforme a seta
/// estivesse sozinha ou acompanhada. ⇒ a dominância reescreve o **ÍNDICE do encaixe** e cai no
/// mesmo `cos`/`sin` de sempre.
#[test]
fn o_cardeal_da_dominancia_e_o_mesmo_que_a_seta_sozinha_da() {
    let modo = DirectionMode::FourWay;
    let casos: [(Vec2, Vec2, Vec2); 4] = [
        ([1.0, 0.0], [1.0, 1.0], [1.0, 0.0]),
        ([0.0, 1.0], [1.0, 1.0], [0.0, 1.0]),
        ([-1.0, 0.0], [-1.0, 1.0], [-1.0, 0.0]),
        ([0.0, -1.0], [1.0, -1.0], [0.0, -1.0]),
    ];
    for (primeira, ambas, sozinha) in casos {
        // A que chega POR ÚLTIMO é a `primeira` deste caso: carrega-se a outra antes.
        let outra = [ambas[0] - primeira[0], ambas[1] - primeira[1]];
        let mut mem = Dominance::default();
        let _ = mem.observe(outra);
        let com_duas = direction::quantize(ambas, modo, mem.observe(ambas));
        let so_uma = direction::quantize(sozinha, modo, direction::DominantAxis::None);
        assert_eq!(
            com_duas, so_uma,
            "a {primeira:?} com a {outra:?} em baixo tem de dar EXACTAMENTE o que ela da' sozinha"
        );
    }
}

/// ⛔⛔ **Um manípulo ANALÓGICO não é governado pela ordem — é governado pelo que está mais longe.**
///
/// ⚠️ Esta é a cerca que faz a lei ser correcta fora do teclado. A dominância só decide quando a
/// intenção está **exactamente** na diagonal; com componentes diferentes ganha a maior, que é o que
/// o encaixe já fazia. *Sem isto, rodar um manípulo faria o corpo andar para o lado errado a meio
/// da volta* — o eixo que cruzasse o limiar por último mandaria mesmo apontando o outro.
#[test]
fn mas_um_manipulo_fora_da_diagonal_obedece_a_componente_maior() {
    let modo = DirectionMode::FourWay;
    let mut mem = Dominance::default();
    // O `x` acorda primeiro, o `y` depois — mas o manípulo aponta claramente para a direita.
    let _ = mem.observe([0.90, 0.00]);
    let d = direction::quantize([0.90, 0.20], modo, mem.observe([0.90, 0.20]));
    perto(d, [0.921_954, 0.0], "a componente maior manda, nao a ordem");
}

/// ⚠️ **Duas setas no MESMO tique** — não há «última», e a resposta é **declarada**: manda o
/// horizontal.
///
/// ⛔ Ela é a convenção dos 4-direcções clássicos, e o que importa é que seja **determinística e
/// igual nos quatro quadrantes** — que é exactamente o que o arredondamento de antes não era.
#[test]
fn duas_setas_no_mesmo_tique_caem_no_horizontal_declarado() {
    let modo = DirectionMode::FourWay;
    for ambas in [[1.0, 1.0], [1.0, -1.0], [-1.0, 1.0], [-1.0, -1.0]] {
        let mut mem = Dominance::default();
        let d = direction::quantize(ambas, modo, mem.observe(ambas));
        perto(
            d,
            [ambas[0], 0.0],
            &format!("as duas de uma vez em {ambas:?}"),
        );
    }
}

/// ⚠️ **Soltar tudo esquece** — senão a primeira seta do gesto SEGUINTE herdaria um comando velho.
#[test]
fn soltar_tudo_esquece_quem_mandava() {
    let mut mem = Dominance::default();
    let _ = mem.observe([1.0, 0.0]);
    let _ = mem.observe([1.0, 1.0]);
    assert_eq!(mem.observe([0.0, 0.0]), direction::DominantAxis::None);
    // E o gesto seguinte, com as duas de uma vez, cai no declarado — não no `y` de antes.
    perto(
        direction::quantize([1.0, 1.0], DirectionMode::FourWay, mem.observe([1.0, 1.0])),
        [1.0, 0.0],
        "o gesto novo nao herda o comando do anterior",
    );
}
