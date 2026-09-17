//! Os gates do **TECTO** do `motion.wave` — o par `clamp`/tecto digitável e o alcance da lei.
//!
//! ⚠️ Irmão por `#[path]` e **FILHO** do `lib.rs`, como o `lib_tests.rs` ao lado.
//!
//! ⛔ **O corte é por RESPONSABILIDADE e a catraca de LOC pediu-o:** ao ganhar estes dois gates, o
//! `lib_tests.rs` passou de `700` para `735` linhas. A lei é *cortar, nunca subir o número* — e o
//! tecto é um assunto próprio, separado da lei do passo e dos produtores.

/// ⚠️ A fixtura mora no irmão `lib_tests.rs` (`pub(super)`), e é ele que a declara — duas cópias
/// de um construtor de `Params` divergiriam no dia em que um campo novo nascesse.
use super::tests::params;
use super::*;

/// ⭐⭐⭐ **A CAPACIDADE tem de ser alcançável, e o SLIDER tem de ser arrastável** — duas perguntas,
/// e até 2026-09-17 este nó respondia-as com **um número só** (`60` no clamp *e* no slider).
///
/// ⛔ Com os dois iguais, *a capacidade do motor acaba onde o dedo acaba*: não havia como pedir um
/// campo maior nem digitando. O `motion.soft_body` — a outra simulação de grelha 2D desta casa — já
/// shipa o par certo (clamp `512`, slider `64`) com um gate igual a este, e o doc 91 pôs a mesma
/// forma em 25 params. Este ficou de fora.
///
/// ⚠️ **O slider tem de ficar ESTRITAMENTE abaixo do clamp** e não igual: preso ao tecto, um track
/// de ~154 px moveria dezenas de linhas por pixel e o DEFAULT deixaria de ser alcançável com o dedo
/// — a lição que o doc 88 §11 mediu no irmão.
#[test]
fn o_teto_digitavel_alcanca_o_clamp_e_o_slider_fica_abaixo() {
    for param in ["rows", "cols"] {
        let hint = PARAM_HINTS
            .iter()
            .find(|h| h.param == param)
            .unwrap_or_else(|| panic!("{param} tem hint"));
        let hard = PARAM_HARD_MAX
            .iter()
            .find(|h| h.param == param)
            .unwrap_or_else(|| panic!("{param} tem teto digitavel"));
        #[expect(
            clippy::cast_precision_loss,
            reason = "um lado de grelha, sempre pequeno"
        )]
        let clamp = MAX_SIDE as f32;
        assert!(
            (hard.max - clamp).abs() < f32::EPSILON,
            "o teto DIGITAVEL de {param} ({}) tem de alcancar o clamp ({clamp}) e parar nele",
            hard.max
        );
        assert!(
            hint.max < hard.max,
            "o slider de {param} e' a FAIXA DE AUTORIA e nao o teto: soft {} devia ficar abaixo \
             do hard {}",
            hint.max,
            hard.max
        );
    }
}

/// ⭐⭐ **A LEI escala até ao tecto** — e ⛔ **só a lei**, que é o que uma mutação me ensinou.
///
/// ⚠️⚠️ Este gate monta os [`Params`] **à mão** e chama o [`simulate`]: ele nunca atravessa o
/// `clamp(2, MAX_SIDE)`, que vive no `eval`. Encolhi esse clamp para `60` numa prova de mutação e
/// **este gate ficou VERDE** — *um arnês que monta o estado à mão mede a lei e não a porta*, que é
/// a lição que esta casa já pagou quatro vezes noutros módulos.
///
/// ⇒ a metade que falta — *o artista consegue PEDIR um campo do tamanho do tecto?* — vive onde um
/// grafo se pode cozer de verdade:
/// [`ph2d_app_motion::motion_rig_relogio`](../../ph2d-app-motion/src/motion_rig_relogio.rs), no
/// gate `o_campo_chega_ao_tecto_pela_porta_do_produto`. As duas juntas é que afirmam a frase
/// inteira; nenhuma delas sozinha.
#[test]
fn a_lei_do_campo_escala_ate_ao_tecto() {
    #[expect(clippy::cast_sign_loss, reason = "MAX_SIDE e' positivo por construcao")]
    let lado = MAX_SIDE as usize;
    let p = params(lado, lado, 0.35, 0.02);
    let saida = simulate(None, &Stream::new(0), &[], 0.0, &p);
    assert_eq!(
        saida.count(),
        lado * lado,
        "o campo nao chegou ao tecto: pedidas {lado}x{lado}"
    );
}
