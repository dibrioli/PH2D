//! Gates do gesto do corte.

use super::*;

const VISTA: [f32; 3] = [0.0, 0.0, -1.0];
const FORA: [f32; 3] = [0.0, 0.0, 0.0];

/// **A caixa guarda os CANTOS, não o caminho.**
#[test]
fn a_caixa_guarda_so_os_cantos() {
    let mut g = Gesto::comeca(Forma::Caixa, [10.0, 20.0], None);
    for k in 1..40 {
        let t = k as f32;
        g.move_para([10.0 + t * 2.0, 20.0 + t]);
    }
    let anel = g.anel();
    assert_eq!(anel.len(), 4, "a caixa tem quatro cantos");
    assert_eq!(anel[0], [10.0, 20.0]);
    assert_eq!(anel[2], [88.0, 59.0], "o canto oposto é o ÚLTIMO ponto");
}

/// **O laço guarda o caminho, com PASSO MÍNIMO.**
///
/// ⚠️ O passo nomeia o recurso: a tampa é triangulada em `O(n²)` e o prisma tem
/// `2n` vértices, logo um arrasto lento guardado ponto a ponto pagaria o
/// quadrado de centenas por uma forma que o artista não distingue.
#[test]
fn o_laco_guarda_o_caminho_com_passo_minimo() {
    let mut g = Gesto::comeca(Forma::Laco, [0.0, 0.0], None);
    // 100 eventos a UM pixel de distância: o passo mínimo tem de os filtrar.
    for k in 1..=100 {
        g.move_para([k as f32, 0.0]);
    }
    let anel = g.anel();
    assert!(
        anel.len() < 30 && anel.len() > 20,
        "100 eventos a 1 px com passo de 4 px tinham de dar ~25 pontos, e deram {}",
        anel.len()
    );
    // ⭐ E o que ficou guardado respeita mesmo o passo.
    for par in anel.windows(2) {
        let d = (par[1][0] - par[0][0]).hypot(par[1][1] - par[0][1]);
        assert!(d >= 4.0 - 1e-6, "dois pontos a {d} px, abaixo do passo");
    }
}

/// **Um gesto sem área devolve anel VAZIO** — quem chama recusa em voz alta.
#[test]
fn um_gesto_sem_area_devolve_anel_vazio() {
    let mut caixa = Gesto::comeca(Forma::Caixa, [5.0, 5.0], None);
    caixa.move_para([5.4, 40.0]);
    assert!(
        caixa.anel().is_empty(),
        "uma caixa de meio pixel de largura"
    );

    let mut laco = Gesto::comeca(Forma::Laco, [5.0, 5.0], None);
    laco.move_para([6.0, 5.0]);
    assert!(
        laco.anel().is_empty(),
        "um laço com dois pontos não fecha nada"
    );
}

/// ⭐⭐⭐ **SEM SUPERFÍCIE A ORIENTAÇÃO É COAGIDA — e as DUAS metades são
/// obrigatórias** (espec §3).
///
/// ⚠️ *Só a metade de cima mostraria um knob MORTO; só a de baixo mostraria um
/// knob VIVO.* Juntas elas dizem a lei: **o knob é vivo, e é anulado quando não
/// há superfície de onde tirar uma normal.**
///
/// ⭐ E a terceira asserção é a divergência que esta casa escolhe: o alvo faz
/// esta correcção **em silêncio**, e nós **devolvemos o facto** para quem chama
/// o poder dizer. *Um knob que muda de valor sem avisar é a espécie de controlo
/// que mente.*
#[test]
fn sem_superficie_a_orientacao_e_coagida_e_o_facto_e_devolvido() {
    // (a) SEM acerto: a superfície é anulada, e o facto volta.
    let sem = Gesto::comeca(Forma::Caixa, [0.0, 0.0], None);
    assert_eq!(
        sem.orientacao(Orientacao::Superficie),
        (Orientacao::Vista, true),
        "sem superfície não há normal — e o `true` é o que permite DIZÊ-LO"
    );
    assert_eq!(
        sem.orientacao(Orientacao::Vista),
        (Orientacao::Vista, false),
        "pedir a vista sem acerto não é coacção nenhuma"
    );

    // (b) COM acerto: o knob está VIVO e sobrevive.
    let com = Gesto::comeca(
        Forma::Caixa,
        [0.0, 0.0],
        Some(([1.0, 0.0, 0.0], [1.0, 0.0, 0.0])),
    );
    assert_eq!(
        com.orientacao(Orientacao::Superficie),
        (Orientacao::Superficie, false),
        "com superfície o knob autorado tem de sobreviver — senão ele é morto"
    );
}

/// **O plano: a normal vem da superfície, ou é a vista INVERTIDA.**
///
/// ⚠️ E o par emparelhado outra vez: **sem** acerto as duas orientações dão o
/// **mesmo** eixo; **com** acerto elas dão eixos **diferentes**.
#[test]
fn o_plano_usa_a_normal_da_superficie_ou_a_vista_invertida() {
    let com = Gesto::comeca(
        Forma::Caixa,
        [0.0, 0.0],
        Some(([2.0, 3.0, 4.0], [1.0, 0.0, 0.0])),
    );
    let (sup, _) = com.plano(Orientacao::Superficie, VISTA, FORA);
    let (vis, _) = com.plano(Orientacao::Vista, VISTA, FORA);
    assert_eq!(
        sup.origem,
        [2.0, 3.0, 4.0],
        "a origem é onde o gesto começou"
    );
    assert_eq!(sup.normal, [1.0, 0.0, 0.0], "a normal da superfície");
    assert_eq!(vis.normal, [0.0, 0.0, 1.0], "a vista INVERTIDA");
    assert_ne!(
        sup.normal, vis.normal,
        "com acerto as duas orientações TÊM de diferir — senão o knob é morto"
    );

    let sem = Gesto::comeca(Forma::Caixa, [0.0, 0.0], None);
    let (a, coagida) = sem.plano(Orientacao::Superficie, VISTA, FORA);
    let (b, _) = sem.plano(Orientacao::Vista, VISTA, FORA);
    assert!(coagida, "e o facto da coacção volta");
    assert_eq!(
        a.normal, b.normal,
        "sem acerto as duas TÊM de coincidir — é a linha de cima da tabela da espec"
    );
    assert_eq!(a.origem, FORA, "sem acerto a origem é a que o chamador dá");
}

/// ⭐ **A PRÉ-VISUALIZAÇÃO só existe quando há forma.**
///
/// ⚠️ **As duas metades:** antes de haver área ela é `None` (senão a moldura
/// pintaria um risco degenerado a cada clique), e depois ela é exactamente o
/// anel que a lei vai receber — *um indicador que mostra outra coisa que não a
/// ferramenta é pior que nenhum.*
#[test]
fn a_previa_so_existe_quando_ha_forma_e_e_o_mesmo_anel() {
    let mut g = Gesto::comeca(Forma::Caixa, [10.0, 10.0], None);
    assert!(g.previa().is_none(), "um clique parado não desenha nada");
    g.move_para([60.0, 40.0]);
    let previa = g.previa().expect("agora há caixa");
    assert_eq!(previa, g.anel(), "a prévia É o anel que a lei recebe");
    assert_eq!(previa.len(), 4);
}

/// **A ferramenta chama-se BOX TRIM, e o nome vive num sítio só.**
///
/// Ordem do dono (2026-09-15): *«Coloque como Box Trim»*. ⚠️ A metade de baixo
/// é a que impede a recaída: o teclado tem de LER o rótulo, nunca escrever o
/// nome à mão — *duas superfícies sobre o mesmo valor divergem no dia em que
/// uma delas mudar*.
#[test]
fn a_ferramenta_chama_se_box_trim() {
    use super::Forma;
    assert_eq!(Forma::Caixa.label(), "Box Trim");
    assert_eq!(Forma::Laco.label(), "Lasso Trim");

    let teclado = include_str!("keys.rs");
    assert!(
        teclado.contains("f.label()"),
        "o teclado deixou de ler o rótulo da forma"
    );
    for escrito_a_mao in ["\"caixa\"", "\"laco\""] {
        assert!(
            !teclado.contains(escrito_a_mao),
            "o nome da forma voltou a ser escrito à mão no teclado: {escrito_a_mao}"
        );
    }
}
