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
    let anel = g.anel(0.0);
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
    let anel = g.anel(0.0);
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
        caixa.anel(0.0).is_empty(),
        "uma caixa de meio pixel de largura"
    );

    let mut laco = Gesto::comeca(Forma::Laco, [5.0, 5.0], None);
    laco.move_para([6.0, 5.0]);
    assert!(
        laco.anel(0.0).is_empty(),
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
    assert!(g.previa(0.0).is_none(), "um clique parado não desenha nada");
    g.move_para([60.0, 40.0]);
    let previa = g.previa(0.0).expect("agora há caixa");
    assert_eq!(previa, g.anel(0.0), "a prévia É o anel que a lei recebe");
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
    use ph2d_sculpt3d::{TrimForma, Verb};
    // ⚠️ **O nome da FERRAMENTA e o nome da FORMA são duas perguntas**, e por
    // isso vivem em duas portas: o dono pediu *«o botão nos tools para box
    // trim»* e *«nos parâmetros botões box e circle e laço»* — a ferramenta é
    // uma, as formas são três.
    assert_eq!(Verb::BoxTrim.label(), "Box Trim");
    assert_eq!(TrimForma::Caixa.label(), "Box");
    assert_eq!(TrimForma::Circulo.label(), "Circle");
    assert_eq!(TrimForma::Laco.label(), "Lasso");

    let teclado = include_str!("keys.rs");
    for porta in ["Verb::BoxTrim.label()", "trim_forma.label()"] {
        assert!(
            teclado.contains(porta),
            "o teclado deixou de ler o rótulo pela porta `{porta}`"
        );
    }
    for escrito_a_mao in ["\"Box Trim\"", "\"caixa\"", "\"laco\"", "\"Lasso\""] {
        assert!(
            !teclado.contains(escrito_a_mao),
            "o nome voltou a ser escrito à mão no teclado: {escrito_a_mao}"
        );
    }
}

/// ⭐⭐ **O CÍRCULO é um círculo, e a contagem de lados sai do ECRÃ.**
///
/// ⚠️ **As três metades são três defeitos diferentes:** um raio errado corta no
/// sítio errado; um centro errado corta a peça errada; e uma contagem escolhida
/// à mão entrega um polígono que o artista reconhece como polígono (poucos
/// lados) ou uma tampa que a triangulação não aguenta (muitos).
#[test]
fn o_circulo_e_um_circulo_e_os_lados_saem_do_ecra() {
    use ph2d_sculpt3d::TrimForma;
    for raio in [20.0f32, 120.0, 600.0] {
        let mut g = Gesto::comeca(TrimForma::Circulo, [100.0, 100.0], None);
        g.move_para([100.0 + raio, 100.0]);
        let anel = g.anel(0.0);
        assert!(anel.len() >= 12, "raio {raio}: só {} lados", anel.len());
        // Todo ponto está no círculo, e o centro é o pen-down.
        for p in &anel {
            let r = (p[0] - 100.0).hypot(p[1] - 100.0);
            assert!(
                (r - raio).abs() < 1e-2,
                "raio {raio}: um ponto ficou a {r} do centro"
            );
        }
        // ⭐ **A FLECHA é a régua da contagem** — ela tem de caber no meio
        // pixel que a lei promete, e um polígono com metade dos lados **não**
        // cabe (senão a contagem estava inflada).
        let n = anel.len() as f32;
        let flecha = raio * (1.0 - (std::f32::consts::PI / n).cos());
        assert!(
            flecha <= 0.5,
            "raio {raio}: a flecha mede {flecha} px com {n} lados"
        );
        let metade = raio * (1.0 - (std::f32::consts::PI / (n * 0.5)).cos());
        assert!(
            metade > 0.5 || anel.len() == 12,
            "raio {raio}: metade dos lados ainda caberia ({metade} px) — a \
             contagem está inflada"
        );
    }
    // Um arrasto de menos de um pixel não delimita área.
    let mut g = Gesto::comeca(TrimForma::Circulo, [0.0, 0.0], None);
    g.move_para([0.5, 0.0]);
    assert!(g.anel(0.0).is_empty());
}

/// ⛔⛔ **A SUAVIZAÇÃO CHEGA AO ANEL, e SÓ no laço** — a pergunta que o §5.0 do
/// `CLAUDE.md` diz que nenhum instrumento deste repo faz: *o valor chega a um
/// consumidor?*
///
/// ⚠️ **As três metades:** ela move o anel do laço · ela **não** toca a caixa
/// nem o círculo (um knob que agisse ali seria uma lei inventada) · e `0` é o
/// traço cru **ao bit**.
#[test]
fn a_suavizacao_chega_ao_anel_e_so_no_laco() {
    use ph2d_sculpt3d::TrimForma;
    // Um laço com tremor: um quadrado grosseiro com os pontos a saltar.
    let mut laco = Gesto::comeca(TrimForma::Laco, [0.0, 0.0], None);
    for i in 1u8..40 {
        let i = f32::from(i);
        let t = i * 12.0;
        let r = 3.0 * ((i * 2.399_9).sin());
        laco.move_para([t.cos() * 100.0 + r, t.sin() * 100.0 - r]);
    }
    let cru = laco.anel(0.0);
    let suave = laco.anel(1.0);
    assert_eq!(cru.len(), suave.len(), "a suavização mudou a contagem");
    let movido = cru
        .iter()
        .zip(&suave)
        .map(|(a, b)| (a[0] - b[0]).hypot(a[1] - b[1]))
        .fold(0.0f32, f32::max);
    assert!(
        movido > 0.5,
        "a pista não chega ao anel: o ponto que mais andou moveu {movido} px"
    );
    // `0` é o traço CRU, ao bit.
    let bits = |v: &[[f32; 2]]| {
        v.iter()
            .map(|p| [p[0].to_bits(), p[1].to_bits()])
            .collect::<Vec<_>>()
    };
    let mut nu = Gesto::comeca(TrimForma::Laco, [0.0, 0.0], None);
    for i in 1u8..40 {
        let t = f32::from(i) * 12.0;
        nu.move_para([t.cos() * 100.0, t.sin() * 100.0]);
    }
    assert_eq!(bits(&nu.anel(0.0)), bits(&nu.pontos_para_o_gate()));

    // ⛔ **E ela NÃO toca as formas de dois pontos.**
    for forma in [TrimForma::Caixa, TrimForma::Circulo] {
        let mut g = Gesto::comeca(forma, [0.0, 0.0], None);
        g.move_para([80.0, 60.0]);
        assert_eq!(
            bits(&g.anel(0.0)),
            bits(&g.anel(1.0)),
            "a suavização mexeu no {} — ele guarda DOIS pontos e não tem traço",
            forma.label()
        );
    }
}

/// ⛔⛔ **O TECTO DA SUAVIZAÇÃO É DERIVADO DA RESOLUÇÃO DO GESTO — e este é o
/// único sítio onde as duas constantes se encontram.**
///
/// A lei ([`ph2d_trim::suaviza`]) vive numa crate que **não sabe** com que passo
/// o laço guarda pontos; o gesto ([`super::PASSO_MINIMO_PX`]) vive aqui e não
/// sabe quantas passagens a lei gasta. ⇒ *a derivação do tecto atravessa a
/// fronteira, e uma derivação sem gate é uma nota que envelhece.*
///
/// ⚠️ **A afirmação tem DUAS metades:** no tecto o canto tem de sobreviver
/// (deslocar-se menos que o passo com que o traço foi registado — uma diferença
/// que o anel não consegue representar não é feição perdida), e o tecto tem de
/// estar **abaixo** do ponto onde isso deixa de valer, senão ele foi escolhido e
/// não derivado.
#[test]
fn o_tecto_da_suavizacao_e_derivado_da_resolucao_do_gesto() {
    // Um quadrado LIMPO, com o espaçamento do próprio gesto: os cantos são o
    // que esta lei pode destruir, e sem tremor a régua mede só isso.
    let lado = 100.0f32;
    let por_lado = (2.0 * lado / super::PASSO_MINIMO_PX) as usize;
    let cantos = [[-lado, -lado], [lado, -lado], [lado, lado], [-lado, lado]];
    let mut anel = Vec::new();
    for c in 0..4 {
        let (a, b) = (cantos[c], cantos[(c + 1) % 4]);
        for i in 0..por_lado {
            let t = i as f32 / por_lado as f32;
            anel.push([a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]);
        }
    }
    let perdido = |grau: f32| {
        let s = ph2d_trim::suaviza::suaviza(&anel, grau);
        (0..4)
            .map(|c| {
                let i = c * por_lado;
                (s[i][0] - anel[i][0]).hypot(s[i][1] - anel[i][1])
            })
            .fold(0.0f32, f32::max)
    };
    let no_tecto = perdido(1.0);
    assert!(
        no_tecto < super::PASSO_MINIMO_PX,
        "no tecto o canto desloca-se {no_tecto:.2} px, mais do que o passo com \
         que o laço guarda pontos ({}) — a lei passou a apagar feição que o \
         traço ainda representava",
        super::PASSO_MINIMO_PX
    );
    // ⭐⭐ **E a metade que impede um tecto escolhido por baixo:** a TRIPLA
    // aplicação tem de PASSAR a resolução. Sem ela, um `PARES_MAX = 1` passaria
    // a primeira metade com folga e a pista do artista não faria nada.
    //
    // ⚠️⚠️ **É `4 ×`, e as duas redacções anteriores erraram nisto:** com o
    // dobro o canto mede `3,59 px` e ainda cabe, e a `3 ×` ele pousa
    // **exactamente em cima** da régua (`4,00 px`) — uma comparação de `f32` no
    // fio da navalha, que é o que um gate não pode ser. *O cruzamento está
    // MEDIDO nos `192` pares* (tabela no doc do
    // `ph2d_trim::suaviza::PARES_MAX`), e a folga **é** a conservação que o
    // tecto declara — a medição corre no
    // espaçamento MÍNIMO, e um arrasto rápido guarda pontos mais afastados, onde
    // as mesmas passagens alcançam mais longe em pixels.
    let mut triplo = anel.clone();
    for _ in 0..4 {
        triplo = ph2d_trim::suaviza::suaviza(&triplo, 1.0).into_owned();
    }
    let com_triplo = (0..4)
        .map(|c| {
            let i = c * por_lado;
            (triplo[i][0] - anel[i][0]).hypot(triplo[i][1] - anel[i][1])
        })
        .fold(0.0f32, f32::max);
    assert!(
        com_triplo >= super::PASSO_MINIMO_PX,
        "com QUATRO vezes as passagens o canto ainda se desloca só {com_triplo:.2} \
         px — o tecto está escolhido muito abaixo do que a régua permite, e a \
         pista do artista tem menos alcance do que podia"
    );
}
