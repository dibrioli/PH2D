//! Os gates da lei do passo medido sobre a superfície — ver o módulo.

use super::{CaminhoNoMundo, MAX_DABS_POR_PASSO, passo_no_mundo};
use crate::{Verb, espacamento_do_verbo};

/// ⭐ **O passo é o espaçamento declarado, na régua certa** — e DECLARAR um
/// espaçamento não é o mesmo que MEDI-LO sobre a superfície.
///
/// ⚠️⚠️ **A afirmação é a UNIDADE, não o número de hoje:** o espaçamento é uma
/// percentagem do **DIÂMETRO** ⇒ `passo = 2·raio·pct/100`. A 1.ª redacção deste
/// gate fixava `0,04` à mão e reprovou no dia em que o dono mandou baixar o
/// espaçamento — *sobre produto correcto*, porque ela media a constante e não a
/// lei. O erro de factor `2` entre raio e diâmetro continua a ser apanhado, que
/// é o que este gate existe para dizer.
///
/// ⛔⛔ **E as TRÊS respostas são diferentes**, o que é a razão de a porta do
/// passo de mundo perguntar ao verbo: o **afiado** declara e mede no mundo · o
/// **plano** declara e **não** mede (ele corre a régua de ecrã da casa) · o
/// `Draw` não declara nada. *Uma porta que devolvesse um número ao plano estaria
/// a dar uma segunda resposta a «este verbo mede no mundo?», e o dia em que
/// alguém a lesse seria o dia em que o pincel de plano mudava de lei.*
#[test]
fn o_passo_e_o_espacamento_declarado_e_so_um_verbo_o_mede_no_mundo() {
    for (verb, raio) in [(Verb::DrawSharp, 0.4f32), (Verb::Plane, 0.5)] {
        let pct = espacamento_do_verbo(verb).expect("o verbo declara espaçamento");
        let b = crate::Brush {
            verb,
            ..crate::Brush::default()
        };
        let devido = 2.0 * raio * pct / 100.0;
        let ecra = crate::passo_do_traco(&b, raio);
        assert!(
            (ecra - devido).abs() < 1e-7,
            "{verb:?}: {ecra} contra {devido}"
        );
        match passo_no_mundo(&b, raio) {
            Some(p) => {
                assert!(verb.mede_o_passo_no_mundo(), "{verb:?} não mede no mundo");
                assert!((p - devido).abs() < 1e-7, "{verb:?}: {p} contra {devido}");
            }
            None => assert!(
                !verb.mede_o_passo_no_mundo(),
                "{verb:?} mede no mundo e a porta não lhe deu passo"
            ),
        }
    }
    // ⛔ Quem não declara não tem espaçamento — cai na régua mínima da casa.
    assert!(espacamento_do_verbo(Verb::Draw).is_none());
    let liso = crate::Brush {
        verb: Verb::Draw,
        ..crate::Brush::default()
    };
    assert!(passo_no_mundo(&liso, 0.4).is_none());

    // ⭐⭐ **E o pincel pode trazer o SEU espaçamento**, que é a porta pela qual a
    // bancada de paridade arrasta com o do ALVO enquanto o produto ship o nosso.
    let afiado = crate::Brush {
        verb: Verb::DrawSharp,
        espacamento_pct: Some(crate::ESPACAMENTO_DO_AFIADO_DO_ALVO_PCT),
        ..crate::Brush::default()
    };
    let p = passo_no_mundo(&afiado, 0.4).expect("o afiado mede no mundo");
    let devido = 2.0 * 0.4 * crate::ESPACAMENTO_DO_AFIADO_DO_ALVO_PCT / 100.0;
    assert!((p - devido).abs() < 1e-7, "{p} contra {devido}");
}

/// ⭐⭐⭐ **O RESÍDUO VIAJA — a mesma lei do carry do `walk`, e é ela que torna o
/// traço um facto do CAMINHO.**
///
/// O mesmo caminho entregue em candidatos GROSSOS e FINOS tem de fechar o mesmo
/// número de passos. ⛔ Sem o resíduo, o caminho fino perderia um pedaço por
/// candidato e o grosso não — que é a dependência da taxa de amostragem que o
/// eixo do alvo tem e esta lei existe para não ter.
#[test]
fn o_mesmo_caminho_fecha_os_mesmos_passos_seja_qual_for_a_amostragem() {
    let passo = 0.04f32;
    let comprimento = 1.0f32;
    let contar = |n: usize| {
        let mut c = CaminhoNoMundo::novo();
        let mut total = 0u32;
        for i in 0..=n {
            #[allow(clippy::cast_precision_loss)]
            let t = i as f32 / n as f32;
            total += c.avanca([t * comprimento, 0.0, 0.0], passo);
        }
        total
    };
    let (grosso, fino, finissimo) = (contar(10), contar(100), contar(1000));
    assert_eq!(
        (grosso, fino),
        (25, 25),
        "o caminho de 1,0 a passos de 0,04 fecha 25 passos, seja como for amostrado"
    );
    assert_eq!(finissimo, 25, "e o mais fino também");
}

/// ⭐⭐ **Um candidato que salta vários passos fecha vários** — e o tecto é
/// NOMEADO pelo recurso.
///
/// ⚠️ É o caso da silhueta: ali um píxel de ecrã cobre arco de sobra, e devolver
/// `1` deixaria o vinco raso exactamente onde o defeito está.
#[test]
fn um_salto_grande_fecha_varios_passos_ate_ao_tecto() {
    let passo = 0.01f32;
    let mut c = CaminhoNoMundo::novo();
    assert_eq!(c.avanca([0.0, 0.0, 0.0], passo), 0, "o primeiro é a âncora");
    assert_eq!(
        c.avanca([0.035, 0.0, 0.0], passo),
        3,
        "0,035 = 3 passos e resto"
    );
    // E o resto (0,005) viaja: mais 0,006 fecha o quarto.
    assert_eq!(c.avanca([0.041, 0.0, 0.0], passo), 1);
    // ⛔ O tecto: um salto de cem passos entrega o tecto, não cem dabs.
    let mut c = CaminhoNoMundo::novo();
    c.avanca([0.0, 0.0, 0.0], passo);
    assert_eq!(c.avanca([1.0, 0.0, 0.0], passo), MAX_DABS_POR_PASSO);
}

/// ⭐⭐⭐ **A FRONTEIRA: um salto de EXACTAMENTE um passo fecha UM dab** — e é
/// esta linha que paga a divergência D-1 do pincel afiado.
///
/// ⚠️ O [`crate::walk`] desta casa **recusa** esse salto (`<=`), o que a meio de
/// um traço dá a mesma lista de dabs e **na inversão** perde o dab adiado — a
/// sombra da ponta media `4,04e-2` contra o alvo. A fronteira desta lei é a do
/// alvo (`acumulado < passo` ⇒ zero, logo `== passo` ⇒ **um**), e a sombra cai
/// para `1,91e-2` (`oraculo_do_pincel_afiado_produto`, G-3b).
///
/// ⛔ **Sem este gate a fronteira é invisível:** medido, trocar o `<` por `<=`
/// deixa a suíte inteira verde, incluindo os gates de paridade do produto.
#[test]
fn um_salto_de_exactamente_um_passo_fecha_um_dab() {
    let passo = 0.25f32;
    let mut c = CaminhoNoMundo::novo();
    assert_eq!(c.avanca([0.0, 0.0, 0.0], passo), 0, "o primeiro é a âncora");
    assert_eq!(
        c.avanca([passo, 0.0, 0.0], passo),
        1,
        "exactamente um passo fecha UM — é a fronteira do alvo"
    );
    // E um cabelo abaixo não fecha nenhum: as duas metades, senão um `avanca`
    // que devolvesse `1` sempre passaria a primeira.
    let mut c = CaminhoNoMundo::novo();
    c.avanca([0.0, 0.0, 0.0], passo);
    assert_eq!(c.avanca([passo * 0.999, 0.0, 0.0], passo), 0);
}

/// ⛔ **A lei não existe com um passo não-positivo, e ela DIZ isso** devolvendo
/// zero — em vez de dividir e carimbar a peça inteira.
#[test]
fn um_passo_invalido_nao_carimba() {
    let mut c = CaminhoNoMundo::novo();
    c.avanca([0.0, 0.0, 0.0], 0.01);
    assert_eq!(c.avanca([1.0, 0.0, 0.0], 0.0), 0);
    assert_eq!(c.avanca([1.0, 0.0, 0.0], f32::NAN), 0);
    assert_eq!(c.avanca([f32::NAN, 0.0, 0.0], 0.01), 0);
}

/// ⭐ **O pen-up esquece o caminho** — e sem isto o traço seguinte herdaria o
/// resíduo do anterior, que é estado de TRAÇO a atravessar a caneta levantada.
#[test]
fn o_pen_up_esquece_o_caminho() {
    let passo = 0.01f32;
    let mut c = CaminhoNoMundo::novo();
    c.avanca([0.0, 0.0, 0.0], passo);
    assert_eq!(c.avanca([0.009, 0.0, 0.0], passo), 0, "ainda não fecha");
    c.esquece();
    // Depois do pen-up, o primeiro ponto é âncora outra vez — e o resíduo morreu.
    assert_eq!(c.avanca([0.009, 0.0, 0.0], passo), 0);
    assert_eq!(
        c.avanca([0.018, 0.0, 0.0], passo),
        0,
        "o resíduo não sobreviveu"
    );
}

/// ⭐⭐⭐ **O REGIME PARA QUE A GRANULARIDADE ADAPTATIVA EXISTE** — uma superfície
/// cujo arco DISPARA por píxel de ecrã, percorrida **um evento de cada vez**,
/// que é o que acontece junto à silhueta com a mão a andar.
///
/// ⚠️⚠️ **Este gate existe porque o corpus da silhueta NÃO consegue exercitar
/// aquele mecanismo, e isso foi medido:** o traço do report sai do regime
/// quase-tangente ao fim de meia dúzia de dabs, e as mutações que apagam a
/// previsão do candidato seguinte sobreviviam ao gate da cura, ao da invariância
/// à taxa de eventos e à suíte inteira. *Uma linha que nenhuma mutação mata não
/// é lei — é comentário com sintaxe de código*, e a saída honesta é construir a
/// medição ou apagar o mecanismo.
///
/// ⛔⛔ **E ele tem de chamar a lei POR EVENTO, como o app faz.** Uma corrida só
/// sobre o caminho inteiro deixa a previsão adaptar-se dentro dela e não afirma
/// nada sobre o `previsto` ATRAVESSAR os eventos — que é metade do mecanismo, e
/// a metade que já tinha mordido uma vez (`20,8 %` de divergência entre `1` e
/// `4` px por evento, antes de o `previsto` sair para a struct).
///
/// Aqui um píxel de ecrã perto do fim vale dezenas de passos. Com a previsão, os
/// candidatos encolhem e **nenhum dab se perde**; sem ela, cada candidato pede
/// mais do que o [`MAX_DABS_POR_PASSO`] e o tecto **trunca em silêncio** — que é
/// exactamente o vinco a saltar que o dono fotografou.
#[test]
fn a_granularidade_adaptativa_nao_perde_dabs_numa_superficie_que_dispara() {
    /// A superfície: `f(t) = L·(t/T)⁸`, monótona, com arco total `L`. ⇒ o número
    /// de dabs que a lei DEVE entregar é `floor(L / passo)`, sem modelo nenhum.
    /// No último píxel ela anda `8·L/T ≈ 3,1`, que são `31` passos — quase
    /// quatro vezes o tecto.
    struct Rampa {
        carimbos: usize,
    }
    const T: f32 = 512.0;
    const L: f32 = 200.0;
    impl crate::CarimboDoCaminho for Rampa {
        fn congelado(&mut self, t: f32) -> Option<[f32; 3]> {
            let u = t / T;
            let u4 = u * u * u * u;
            Some([L * u4 * u4, 0.0, 0.0])
        }
        fn carimba(&mut self, _t: f32) -> bool {
            self.carimbos += 1;
            true
        }
    }
    let passo = 0.1f32;
    let mut c = CaminhoNoMundo::novo();
    let mut r = Rampa { carimbos: 0 };
    // ⭐ **Um evento de rato por píxel** — a mesma granularidade que o app entrega.
    for i in 0..(T as u32) {
        c.percorre(
            f32::from(u16::try_from(i).unwrap_or(u16::MAX)),
            (i + 1) as f32,
            passo,
            &mut r,
        );
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let devidos = (L / passo).floor() as usize;
    // ⚠️ A folga de UM é a âncora: o primeiro candidato não fecha passo nenhum.
    assert!(
        r.carimbos + 1 >= devidos,
        "a lei perdeu dabs: {} entregues contra {devidos} devidos — o tecto truncou",
        r.carimbos
    );
    assert!(
        r.carimbos <= devidos,
        "a lei carimbou a MAIS: {} contra {devidos}",
        r.carimbos
    );
}
