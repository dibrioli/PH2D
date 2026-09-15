//! Os gates das leis que não são o deslize: a ORDEM, a quantização, a isometria,
//! as rampas e a rotação.

use super::*;
use direction::DirectionMode;
use rotation::RotationMode;
use viewpoint::Viewpoint;

const EPS: f32 = 1.0e-5;

fn perto(a: Vec2, b: Vec2, o_que: &str) {
    assert!(
        (a[0] - b[0]).abs() < EPS && (a[1] - b[1]).abs() < EPS,
        "{o_que}: {a:?} != {b:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// A QUANTIZAÇÃO

#[test]
fn a_diagonal_nao_e_mais_rapida_que_o_eixo() {
    // Duas teclas dão `(1,1)`, de comprimento √2 — o defeito de principiante mais
    // antigo do género. O comprimento é CORTADO a 1, nunca esticado.
    let d = direction::quantize(
        [1.0, 1.0],
        DirectionMode::EightWay,
        direction::DominantAxis::None,
    );
    assert!(
        (len(d) - 1.0).abs() < EPS,
        "a diagonal anda {:.4} e o eixo anda 1,0 — 41 % mais depressa",
        len(d)
    );
    let eixo = direction::quantize(
        [1.0, 0.0],
        DirectionMode::EightWay,
        direction::DominantAxis::None,
    );
    assert!((len(eixo) - 1.0).abs() < EPS);
}

#[test]
fn um_manipulo_a_meio_curso_anda_a_meia_velocidade() {
    // ⚠️ Só a DIRECÇÃO encaixa; o comprimento sobrevive. Sem isto, um analógico
    // vira um interruptor.
    let d = direction::quantize(
        [0.5, 0.0],
        DirectionMode::EightWay,
        direction::DominantAxis::None,
    );
    assert!((len(d) - 0.5).abs() < EPS, "comprimento {:.4}", len(d));
}

#[test]
fn quatro_direccoes_encaixa_no_rumo_mais_perto() {
    // ⚠️ As entradas são UNITÁRIAS de propósito: o comprimento sobrevive ao
    // encaixe, então `[0,92 · 0,38]` (comprimento 0,995) sairia `[0,995 · 0]` e
    // a 1.ª redacção deste gate esperava `[1 · 0]` — o teste é que estava errado.
    let vinte_e_tres = 23.0_f32.to_radians();
    perto(
        direction::quantize(
            [vinte_e_tres.cos(), vinte_e_tres.sin()],
            DirectionMode::FourWay,
            direction::DominantAxis::None,
        ),
        [1.0, 0.0],
        "23° encaixa em 0°",
    );
    let sessenta_e_sete = 67.0_f32.to_radians();
    perto(
        direction::quantize(
            [sessenta_e_sete.cos(), sessenta_e_sete.sin()],
            DirectionMode::FourWay,
            direction::DominantAxis::None,
        ),
        [0.0, 1.0],
        "67° encaixa em 90°",
    );
}

#[test]
fn um_eixo_apaga_a_outra_componente_e_nao_encaixa_no_rumo_mais_perto() {
    // ⚠️ A distinção é medida: uma intenção a 80° daria `→` num ENCAIXE de 180°
    // e `↑` num APAGAMENTO de `x`. Os dois modos de eixo apagam.
    let d = direction::quantize(
        [0.17, 0.98],
        DirectionMode::AxisX,
        direction::DominantAxis::None,
    );
    perto(d, [0.17, 0.0], "AxisX apaga o y");
    let d = direction::quantize(
        [0.98, 0.17],
        DirectionMode::AxisY,
        direction::DominantAxis::None,
    );
    perto(d, [0.0, 0.17], "AxisY apaga o x");
}

#[test]
fn sem_intencao_nao_ha_direccao() {
    for m in DirectionMode::ALL {
        perto(
            direction::quantize([0.0, 0.0], m, direction::DominantAxis::None),
            [0.0, 0.0],
            m.label(),
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// O VIEWPOINT

#[test]
fn a_vista_de_cima_e_identidade_ao_bit() {
    // ⛔ Não é «perto»: é o MESMO `f32`. Uma identidade que passasse por uma
    // rotação de 0° e uma divisão por 1,0 deixaria de ser reproduzível, e toda
    // cena sem isometria paga isso para sempre.
    for v in [[0.3_f32, -0.7], [1.0, 0.0], [-0.123_456, 0.987_654]] {
        let saida = viewpoint::reproject(v, Viewpoint::TopDown, 26.565);
        assert_eq!(saida, v, "a identidade tem de ser bit-a-bit");
    }
}

#[test]
fn a_isometria_manda_as_duas_teclas_para_as_arestas_do_losango() {
    // `→` vai para nordeste e `↑` para noroeste — as duas arestas de um losango
    // 2:1, cuja elevação é arctan(0,5) = 26,565°.
    let direita = viewpoint::reproject([1.0, 0.0], Viewpoint::Isometric2to1, 0.0);
    let cima = viewpoint::reproject([0.0, 1.0], Viewpoint::Isometric2to1, 0.0);
    assert!(
        direita[0] > 0.0 && direita[1] > 0.0,
        "→ vai para nordeste: {direita:?}"
    );
    assert!(
        cima[0] < 0.0 && cima[1] > 0.0,
        "↑ vai para noroeste: {cima:?}"
    );
    // A inclinação é a do tabuleiro: 2 de largura por 1 de altura.
    let inclinacao = direita[1] / direita[0];
    assert!(
        (inclinacao - 0.5).abs() < 1.0e-3,
        "a aresta do losango 2:1 tem inclinacao 0,5 e tem {inclinacao:.4}"
    );
}

#[test]
fn a_isometria_preserva_o_comprimento_da_intencao() {
    for v in [Viewpoint::Isometric2to1, Viewpoint::Isometric30] {
        let d = viewpoint::reproject([0.5, 0.0], v, 0.0);
        assert!((len(d) - 0.5).abs() < EPS, "{}: {:.4}", v.label(), len(d));
    }
}

#[test]
fn uma_elevacao_impossivel_cai_na_identidade_em_vez_de_inventar_um_losango() {
    for ang in [0.0_f32, 90.0, -10.0, f32::NAN] {
        let d = viewpoint::reproject([0.3, -0.7], Viewpoint::Custom, ang);
        assert_eq!(d, [0.3, -0.7], "elevacao {ang} tinha de cair na identidade");
    }
}

#[test]
fn so_o_modo_custom_le_o_angulo() {
    // ⭐ É esta pergunta que esconde a linha do ângulo no painel.
    assert!(Viewpoint::Custom.reads_angle());
    for v in [
        Viewpoint::TopDown,
        Viewpoint::Isometric2to1,
        Viewpoint::Isometric30,
    ] {
        assert!(!v.reads_angle(), "{} nao le o angulo", v.label());
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// A ORDEM (a lei que o cabeçalho da crate desenha)

#[test]
fn a_quantizacao_corre_no_espaco_da_intencao_e_nao_no_do_ecra() {
    // ⚠️⚠️ O gate que defende a ordem. Com «4 direcções» num tabuleiro 2:1, as
    // saídas TÊM de ser as quatro arestas do losango — se o encaixe corresse
    // depois da reprojecção, elas seriam os eixos do ECRÃ.
    let law = TopDownLaw {
        direction: DirectionMode::FourWay,
        viewpoint: Viewpoint::Isometric2to1,
        ..TopDownLaw::default()
    };
    // Uma intenção quase-direita: encaixa em `→`, que no tabuleiro é nordeste.
    let d = world_direction([0.95, 0.31], &law, &mut direction::Dominance::default());
    assert!(
        d[0] > 0.0 && d[1] > 0.0,
        "com a ordem certa isto e' nordeste; com a ordem trocada seria (1,0). Deu {d:?}"
    );
    assert!(
        (d[1] / d[0] - 0.5).abs() < 1.0e-3,
        "e a inclinacao tem de ser a do tabuleiro: {:.4}",
        d[1] / d[0]
    );
    // ⛔ O controlo: NENHUMA das quatro saídas pode ser um eixo do ecrã.
    for intent in [[1.0_f32, 0.0], [0.0, 1.0], [-1.0, 0.0], [0.0, -1.0]] {
        let d = world_direction(intent, &law, &mut direction::Dominance::default());
        assert!(
            d[0].abs() > 1.0e-3 && d[1].abs() > 1.0e-3,
            "a saida {d:?} e' um eixo do ECRA — a quantizacao correu depois da reprojeccao"
        );
    }
}

#[test]
fn o_neutro_atravessa_a_porta_sem_tocar_no_vector() {
    let law = TopDownLaw {
        direction: DirectionMode::Free,
        viewpoint: Viewpoint::TopDown,
        ..TopDownLaw::default()
    };
    let v = [0.31_f32, -0.42];
    assert_eq!(
        world_direction(v, &law, &mut direction::Dominance::default()),
        v,
        "Free + TopDown = identidade"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AS RAMPAS

#[test]
fn zero_de_rampa_e_instantaneo_e_nao_parado() {
    // A convenção do `step_height` desta casa. Sem ela, os defaults entregam um
    // componente que não anda.
    let v = intent::advance([0.0, 0.0], [1.0, 0.0], 4.0, 0.0, 0.0, 1.0 / 60.0);
    perto(v, [4.0, 0.0], "arranque instantaneo");
    let parado = intent::advance([4.0, 0.0], [0.0, 0.0], 4.0, 0.0, 0.0, 1.0 / 60.0);
    perto(parado, [0.0, 0.0], "travagem instantanea");
}

#[test]
fn a_rampa_leva_o_tempo_que_o_numero_diz() {
    // 4 m/s com 8 m/s² ⇒ meio segundo ⇒ 30 tiques.
    let mut v = [0.0_f32, 0.0];
    for _ in 0..30 {
        v = intent::advance(v, [1.0, 0.0], 4.0, 8.0, 8.0, 1.0 / 60.0);
    }
    assert!(
        (len(v) - 4.0).abs() < 0.02,
        "depois de 30 tiques: {:.4}",
        len(v)
    );
    let mut v = [0.0_f32, 0.0];
    for _ in 0..15 {
        v = intent::advance(v, [1.0, 0.0], 4.0, 8.0, 8.0, 1.0 / 60.0);
    }
    assert!((len(v) - 2.0).abs() < 0.02, "a meio: {:.4}", len(v));
}

#[test]
fn a_rampa_governa_tambem_a_viragem() {
    // ⚠️ Uma lei que acelerasse só ao longo do eixo do movimento deixaria a
    // viragem instantânea — o carro que muda de rumo sem desacelerar.
    let v = intent::advance([4.0, 0.0], [-1.0, 0.0], 4.0, 8.0, 8.0, 1.0 / 60.0);
    assert!(
        v[0] > 3.8,
        "num tique a 8 m/s² a velocidade mal se move, e deu {v:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// A ROTAÇÃO

#[test]
fn a_rotacao_le_a_direccao_de_mundo() {
    // Num tabuleiro isométrico a seta `→` manda para nordeste, e é para nordeste
    // que ele olha — nunca para 0°.
    let law = TopDownLaw {
        viewpoint: Viewpoint::Isometric2to1,
        rotation: RotationMode::ToMovement,
        rotation_speed_deg: 0.0,
        ..TopDownLaw::default()
    };
    let d = world_direction([1.0, 0.0], &law, &mut direction::Dominance::default());
    let ang = rotation::rotate_toward(0.0, d, law.rotation, law.rotation_speed_deg, 1.0 / 60.0);
    assert!(
        ang > 0.1,
        "o angulo tinha de apontar para nordeste (~26,6°) e deu {:.2}°",
        ang.to_degrees()
    );
}

#[test]
fn parado_ele_fica_onde_estava() {
    let ang = rotation::rotate_toward(1.234, [0.0, 0.0], RotationMode::ToMovement, 0.0, 1.0 / 60.0);
    assert_eq!(ang, 1.234, "sem intencao nao ha direccao");
}

#[test]
fn ele_vira_pelo_lado_curto() {
    // De 175° para −175° são 10°, não 350°.
    let de = 175.0_f32.to_radians();
    let alvo = [
        (-175.0_f32).to_radians().cos(),
        (-175.0_f32).to_radians().sin(),
    ];
    let ang = rotation::rotate_toward(de, alvo, RotationMode::ToMovement, 600.0, 1.0 / 60.0);
    let andou = (ang - de).to_degrees();
    assert!(
        andou > 0.0 && andou < 11.0,
        "tinha de virar ~10° para a frente e virou {andou:.2}°"
    );
}

#[test]
fn o_encaixe_da_rotacao_e_do_alvo_e_nao_do_caminho() {
    // ⚠️ `22,4°` está do lado de CÁ do meio de um encaixe de 45 (22,5) — a 1.ª
    // redacção deste gate esperava 45 e o arredondamento dá 0. São dois alvos.
    let vinte_e_dois = 22.4_f32.to_radians();
    let alvo = [vinte_e_dois.cos(), vinte_e_dois.sin()];
    let ang = rotation::rotate_toward(0.0, alvo, RotationMode::Snap90, 0.0, 1.0 / 60.0);
    assert!(
        ang.abs() < EPS,
        "22,4° encaixa em 0° e deu {:.3}°",
        ang.to_degrees()
    );
    let trinta = 30.0_f32.to_radians();
    let alvo = [trinta.cos(), trinta.sin()];
    let ang = rotation::rotate_toward(0.0, alvo, RotationMode::Snap45, 0.0, 1.0 / 60.0);
    assert!(
        (ang.to_degrees() - 45.0).abs() < 0.01,
        "30° encaixa em 45° e deu {:.3}°",
        ang.to_degrees()
    );
}

#[test]
fn so_quem_roda_le_a_velocidade_de_viragem() {
    assert!(!RotationMode::None.reads_speed());
    for m in [
        RotationMode::ToMovement,
        RotationMode::Snap90,
        RotationMode::Snap45,
    ] {
        assert!(m.reads_speed(), "{}", m.label());
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// O FIO

#[test]
fn o_fio_de_cada_enum_e_uma_ida_e_volta_exacta() {
    for m in DirectionMode::ALL {
        assert_eq!(
            direction::from_wire(direction::to_wire(m)),
            m,
            "{}",
            m.label()
        );
    }
    for v in Viewpoint::ALL {
        assert_eq!(
            viewpoint::from_wire(viewpoint::to_wire(v)),
            v,
            "{}",
            v.label()
        );
    }
    for r in RotationMode::ALL {
        assert_eq!(
            rotation::from_wire(rotation::to_wire(r)),
            r,
            "{}",
            r.label()
        );
    }
}

#[test]
fn os_numeros_do_fio_sao_literais_e_nao_a_ordem_da_declaracao() {
    // ⛔ Este gate existe para reprovar no dia em que alguém reordenar as
    // variantes: o postcard é posicional, e uma reordenação trocaria o modo de
    // toda cena já gravada, em silêncio. Os números aqui são escritos à mão de
    // propósito — é a tabela do FORMATO, não um espelho do código.
    assert_eq!(direction::to_wire(DirectionMode::Free), 0);
    assert_eq!(direction::to_wire(DirectionMode::EightWay), 1);
    assert_eq!(direction::to_wire(DirectionMode::FourWay), 2);
    assert_eq!(direction::to_wire(DirectionMode::AxisX), 3);
    assert_eq!(direction::to_wire(DirectionMode::AxisY), 4);
    assert_eq!(viewpoint::to_wire(Viewpoint::TopDown), 0);
    assert_eq!(viewpoint::to_wire(Viewpoint::Isometric2to1), 1);
    assert_eq!(viewpoint::to_wire(Viewpoint::Isometric30), 2);
    assert_eq!(viewpoint::to_wire(Viewpoint::Custom), 3);
    assert_eq!(rotation::to_wire(RotationMode::None), 0);
    assert_eq!(rotation::to_wire(RotationMode::ToMovement), 1);
    assert_eq!(rotation::to_wire(RotationMode::Snap90), 2);
    assert_eq!(rotation::to_wire(RotationMode::Snap45), 3);
}

#[test]
fn um_byte_desconhecido_cai_no_default_e_nao_em_panico() {
    assert_eq!(direction::from_wire(200), DirectionMode::default());
    assert_eq!(viewpoint::from_wire(200), Viewpoint::default());
    assert_eq!(rotation::from_wire(200), RotationMode::default());
}
