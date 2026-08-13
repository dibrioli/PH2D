//! Os gates do substrato — *ele é neutro quando vazio, ele assenta, e ele herda a velocidade*.

use super::*;

/// ⚠️ `hash_node_id` exige `&'static str` (o id nasce de uma constante, por desenho). Uma fixture
/// que precisa de N ids distintos constrói-os pelo número — o substrato não olha para o valor.
fn id(n: u64) -> NodeId {
    NodeId(n + 1)
}

/// Anda `secs` segundos em quadros de 60 Hz.
fn run(m: &mut UiMotion, secs: f64) {
    let dt = 1.0 / 60.0;
    let mut t = 0.0;
    while t < secs {
        m.advance(dt);
        t += dt;
    }
}

/// ⭐ **A NEUTRALIDADE — é ela que torna esta wave segura de landar sozinha.**
///
/// Com o substrato a devolver sempre o alvo, o pintor desenha exactamente o que desenhava antes
/// desta wave existir. *Mutação: `animate` a devolver `from` na primeira vista ⇒ sangra.*
#[test]
fn a_first_sight_arrives_at_the_target_it_does_not_animate_from_nowhere() {
    let mut m = UiMotion::default();
    // Um widget que acaba de aparecer não tem de onde vir; animá-lo do zero seria inventar uma
    // história que não aconteceu.
    assert!((m.animate(id(2), 1.0, Role::Fade) - 1.0).abs() < 1e-6);
    assert_eq!(m.in_flight(), 0, "nada em voo na primeira vista");
}

/// Um app que ninguém tocou não integra nada.
#[test]
fn an_untouched_app_has_an_empty_map() {
    let m = UiMotion::default();
    assert_eq!(m.remembered(), 0);
    assert_eq!(m.in_flight(), 0);
}

/// ⭐ **O QUE A INTERRUPÇÃO COMPRA.** Reverter a meio parte do valor VIVO — não salta para a ponta.
///
/// *Mutação: `t.from = t.to` (partir do alvo antigo em vez do valor vivo) ⇒ o valor SALTA e o gate
/// mede o salto.*
#[test]
fn an_interrupted_target_starts_from_the_live_value_never_from_the_authored_one() {
    let mut m = UiMotion::default();
    m.set_character(UiCharacter::Expressive);
    let a = id(1);
    m.animate(a, 0.0, Role::Fade);
    m.animate(a, 1.0, Role::Fade);
    run(&mut m, 0.08);
    let meio = m.animate(a, 1.0, Role::Fade);
    assert!(
        meio > 0.02 && meio < 0.98,
        "a fixture tem de apanhar a mola A MEIO, e apanhou {meio}"
    );
    // Reverte.
    let depois = m.animate(a, 0.0, Role::Fade);
    assert!(
        (depois - meio).abs() < 1e-5,
        "reverter parte de onde a cena ESTÁ ({meio}), e partiu de {depois}"
    );
}

/// A herança de velocidade: revertendo, o valor **continua a subir por um instante** antes de
/// virar — que é o que uma mola compra sobre uma curva.
#[test]
fn a_reversal_carries_the_velocity_it_had() {
    let mut m = UiMotion::default();
    m.set_character(UiCharacter::Expressive);
    let a = id(1);
    m.animate(a, 0.0, Role::Fade);
    m.animate(a, 1.0, Role::Fade);
    run(&mut m, 0.08);
    let no_pico = m.animate(a, 0.0, Role::Fade);
    m.advance(1.0 / 60.0);
    let logo_a_seguir = m.animate(a, 0.0, Role::Fade);
    assert!(
        logo_a_seguir > no_pico,
        "a mola tinha velocidade POSITIVA e o alvo novo é 0; ela tem de passar do ponto de \
         reversão antes de virar — {no_pico} -> {logo_a_seguir}"
    );
}

/// ⭐ **O CONTRATO DO DISCRETO É ESTRUTURAL, não uma promessa:** `ζ = 1` não tem termo oscilatório.
///
/// *Mutação: `DISCRETE.damping = 0.7` ⇒ ultrapassa e o gate mede quanto.*
#[test]
fn the_discrete_character_never_overshoots() {
    let mut m = UiMotion::default();
    m.set_character(UiCharacter::Discrete);
    let a = id(3);
    m.animate(a, 0.0, Role::Travel);
    m.animate(a, 1.0, Role::Travel);
    let mut pico: f32 = 0.0;
    for _ in 0..240 {
        m.advance(1.0 / 60.0);
        pico = pico.max(m.animate(a, 1.0, Role::Travel));
    }
    assert!(pico <= 1.0 + 1e-4, "Discreto ultrapassou até {pico}");
}

/// E o contraste, sem o qual o gate acima é **vácuo** (uma mola que nem chega também não ultrapassa).
#[test]
fn the_expressive_character_does_overshoot_and_that_is_the_difference() {
    let mut m = UiMotion::default();
    m.set_character(UiCharacter::Expressive);
    let a = id(3);
    m.animate(a, 0.0, Role::Travel);
    m.animate(a, 1.0, Role::Travel);
    let mut pico: f32 = 0.0;
    for _ in 0..240 {
        m.advance(1.0 / 60.0);
        pico = pico.max(m.animate(a, 1.0, Role::Travel));
    }
    assert!(
        pico > 1.01,
        "Expressivo devia ultrapassar, e o pico foi {pico}"
    );
}

/// Ela assenta — e assentar **larga o voo**, senão o app integra para sempre.
#[test]
fn a_settled_track_costs_no_integration() {
    let mut m = UiMotion::default();
    let a = id(3);
    m.animate(a, 0.0, Role::Fade);
    m.animate(a, 1.0, Role::Fade);
    run(&mut m, 3.0);
    assert_eq!(m.in_flight(), 0, "assentou e continua em voo");
    assert!(
        (m.animate(a, 1.0, Role::Fade) - 1.0).abs() < 1e-4,
        "não chegou ao alvo EXACTO"
    );
}

/// ⭐ **A cerca do número.** Um valor que alguém LÊ nunca balança — e nem sequer ocupa memória.
#[test]
fn a_number_never_animates_and_never_remembers() {
    let mut m = UiMotion::default();
    m.set_character(UiCharacter::Expressive);
    let a = id(4);
    assert!((m.animate(a, 0.0, Role::Number) - 0.0).abs() < 1e-6);
    assert!(
        (m.animate(a, 42.0, Role::Number) - 42.0).abs() < 1e-6,
        "um número tem de ser o número, no quadro em que muda"
    );
    assert_eq!(m.remembered(), 0, "um número não ocupa memória");
}

/// ⭐ **OS DOIS EIXOS SÃO INDEPENDENTES** — as quatro combinações são alcançáveis, e o reduced
/// mata PERCURSO nos dois caracteres sem matar o fade.
///
/// *Mutação: colapsar em um seletor de três posições ⇒ `Expressivo + reduced` some.*
#[test]
fn the_taste_and_the_guarantee_are_two_axes() {
    for ch in [UiCharacter::Discrete, UiCharacter::Expressive] {
        let mut m = UiMotion::default();
        m.set_character(ch);
        m.set_reduced_motion(true);
        let a = id(5);
        m.animate(a, 0.0, Role::Travel);
        assert!(
            (m.animate(a, 1.0, Role::Travel) - 1.0).abs() < 1e-6,
            "reduced tem de matar o PERCURSO em {ch:?}"
        );
        assert!(
            m.law(Role::Fade).is_some(),
            "reduced NÃO mata o fade em {ch:?}"
        );
        assert!(!m.decorates(), "reduced mata a decoração em {ch:?}");
    }
    // E sem reduced, só o Expressivo decora.
    let mut m = UiMotion::default();
    m.set_character(UiCharacter::Expressive);
    assert!(m.decorates());
    m.set_character(UiCharacter::Discrete);
    assert!(
        !m.decorates(),
        "a decoração é AUSENTE em Discreto, não atenuada"
    );
}

/// ⭐ **A PODA é o que torna verdadeira a alegação de custo.**
///
/// *Mutação: tirar o `retain` ⇒ o mapa cresce com todo id transiente e o `O(...)` vira falso em
/// silêncio.*
#[test]
fn ids_that_stop_being_painted_are_pruned() {
    let mut m = UiMotion::default();
    for i in 0..50u64 {
        m.animate(id(100 + i), 1.0, Role::Fade);
    }
    assert_eq!(m.remembered(), 50);
    for _ in 0..((PRUNE_AFTER_S * 60.0) as usize + 2) {
        m.advance(1.0 / 60.0);
    }
    assert_eq!(
        m.remembered(),
        0,
        "ninguém os pintou mais; têm de ser podados"
    );
}

/// Um widget que pisca fora da tela por UM quadro não perde a memória — senão ele re-animaria do
/// zero ao voltar, e o artista veria um flash sem causa.
#[test]
fn a_single_missed_frame_does_not_forget() {
    let mut m = UiMotion::default();
    let a = id(3);
    m.animate(a, 1.0, Role::Fade);
    m.advance(1.0 / 60.0);
    assert_eq!(m.remembered(), 1);
}

/// ⭐ **O TEMPO É DE PAREDE.** A mesma animação, dirigida a 30 e a 120 fps, tem de estar no mesmo
/// sítio ao fim do mesmo número de SEGUNDOS.
///
/// *Mutação: `advance` a ignorar o `dt` e a andar um passo fixo ⇒ as duas leituras divergem.*
#[test]
fn the_motion_is_a_fact_of_the_wall_clock_not_of_the_frame_rate() {
    let mut leituras = Vec::new();
    for fps in [30.0_f64, 120.0] {
        let mut m = UiMotion::default();
        m.set_character(UiCharacter::Expressive);
        let a = id(3);
        m.animate(a, 0.0, Role::Travel);
        m.animate(a, 1.0, Role::Travel);
        let dt = 1.0 / fps;
        // ⚠️ **0,1 s cabe INTEIRO nas duas taxas** (3 quadros a 30, 12 a 120). A primeira versão
        // pedia 0,12 s, que arredonda para 4 e 14 quadros = 0,1333 s contra 0,1167 s — a fixture
        // fazia DUAS PERGUNTAS DIFERENTES e chamava à diferença um defeito do código.
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let n = (0.1 * fps).round() as usize;
        for _ in 0..n {
            m.advance(dt);
            // ⚠️ Um pintor chama `animate` TODO quadro; uma fixture que só avança o relógio não
            // contém o fenómeno que mede.
            m.animate(a, 1.0, Role::Travel);
        }
        leituras.push(m.animate(a, 1.0, Role::Travel));
    }
    let (a, b) = (leituras[0], leituras[1]);
    assert!(
        (a - b).abs() < 5e-3,
        "0,12 s são 0,12 s a qualquer taxa: 30 fps deu {a}, 120 fps deu {b}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// F2 — a mistura chega ao widget
// ─────────────────────────────────────────────────────────────────────────────

use ph2d_tokens::Color as TokenColor;

const A: TokenColor = TokenColor {
    r: 0,
    g: 0,
    b: 0,
    a: 255,
};
const B: TokenColor = TokenColor {
    r: 100,
    g: 200,
    b: 40,
    a: 255,
};

/// ⭐ **A NEUTRALIDADE do campo.** `t = 1` devolve o lado quente EXACTO — logo toda construção que
/// não define `hover_t` pinta o que pintava antes desta wave.
///
/// *Mutação: `blend_token_color` a devolver o `rest` em `t = 1` ⇒ todo widget do app muda de cor.*
#[test]
fn the_neutral_t_paints_exactly_what_the_app_painted_before() {
    assert_eq!(blend_token_color(Some(A), Some(B), 1.0), Some(B));
    assert_eq!(blend_token_color(Some(A), Some(B), 0.0), Some(A));
}

/// A meio é o meio — e o `t` é clampado **na porta**, não em cada chamador.
#[test]
fn the_blend_is_a_blend_and_the_door_clamps() {
    let m = blend_token_color(Some(A), Some(B), 0.5).unwrap();
    assert_eq!((m.r, m.g, m.b), (50, 100, 20));
    assert_eq!(
        blend_token_color(Some(A), Some(B), 9.0),
        Some(B),
        "clampa em cima"
    );
    assert_eq!(
        blend_token_color(Some(A), Some(B), -9.0),
        Some(A),
        "clampa em baixo"
    );
}

/// ⚠️ **Um lado AUSENTE é transparente, não "a outra cor".** Um botão `Default` em repouso não tem
/// fundo nenhum, e o hover dele tem de EMERGIR do nada — pintar a cor cheia com alfa cheia faria
/// o fundo aparecer de repente no primeiro pixel de movimento.
///
/// *Mutação: `(None, Some(b)) => Some(b)` ⇒ o fundo pisca em vez de emergir.*
#[test]
fn an_absent_side_fades_through_transparency() {
    let half = blend_token_color(None, Some(B), 0.5).unwrap();
    assert_eq!(
        (half.r, half.g, half.b),
        (B.r, B.g, B.b),
        "a tinta é a mesma"
    );
    assert_eq!(half.a, 128, "o que anda é o ALFA");
    assert_eq!(blend_token_color(None, Some(B), 0.0).unwrap().a, 0);
    assert_eq!(blend_token_color(None, None, 0.5), None);
}

/// ⭐ **O botão MISTURA no eixo do hover, e a SAÍDA funciona** — que é a metade que se perde quando
/// quem escolhe a cor é o estado em vez do escalar.
///
/// *Mutação: `bg_color` a ignorar o `hover_t` ⇒ entrada e saída voltam a ser um degrau.*
#[test]
fn a_button_leaving_the_hover_fades_out_even_though_its_state_is_already_normal() {
    use crate::widget::{Button, ButtonKind, ButtonState};
    let theme = ph2d_tokens::Theme::Forge;
    let hot = Button::new(NodeId(9), "x")
        .kind(ButtonKind::Accent)
        .state(ButtonState::Hovered)
        .bg_color(theme);
    let rest = Button::new(NodeId(9), "x")
        .kind(ButtonKind::Accent)
        .state(ButtonState::Normal)
        .bg_color(theme);
    assert_ne!(rest, hot, "a fixture precisa de dois tons distintos");
    // O rato SAIU: o estado já é Normal, e é o `t` que ainda segura a cor a meio caminho.
    let a_sair = Button::new(NodeId(9), "x")
        .kind(ButtonKind::Accent)
        .state(ButtonState::Normal)
        .hover_t(0.5)
        .bg_color(theme);
    assert_ne!(
        a_sair, rest,
        "com t = 0,5 a saída NÃO pode já estar em repouso"
    );
    assert_ne!(a_sair, hot);
}

/// ⭐ **O REDUCED MOTION nasce no MESMO commit que a animação.** Com ele, o alvo é atingido no
/// quadro em que muda — e o widget pinta o estado duro, sem meio-caminho nenhum.
#[test]
fn reduced_motion_makes_a_hover_arrive_in_the_frame_it_changes() {
    let mut m = UiMotion::default();
    m.set_character(UiCharacter::Expressive);
    m.set_reduced_motion(true);
    let a = id(7);
    m.animate(a, 0.0, Role::Fade);
    // ⚠️ `Fade` SOBREVIVE ao reduced (o gatilho vestibular é percurso, não tinta) — então o que se
    // afirma aqui é o `Travel`, que é o que ele mata.
    assert!(m.law(Role::Fade).is_some());
    m.animate(a, 0.0, Role::Travel);
    assert!((m.animate(a, 1.0, Role::Travel) - 1.0).abs() < 1e-6);
}

// ─────────────────────────────────────────────────────────────────────────────
// A CASCATA (F5) — o horário de alvos que faz N cartões lerem-se como UM gesto.
// ─────────────────────────────────────────────────────────────────────────────

/// **Quantos QUADROS separam dois cartões vizinhos, e quanto dura a entrada inteira.**
///
/// ⚠️ **A régua é o QUADRO, e é essa a lição da wave.** A primeira versão desta sonda media só
/// segundos, e um total honesto escondia o defeito que o Enio viu: com `ε = 0,020` os vizinhos
/// partem a **um** quadro de distância — distinguível por um cronómetro, simultâneo para um olho.
#[test]
#[ignore = "sonda: imprime a tabela que escolhe o CASCADE_STAGGER_SECS"]
fn probe_cascade_frames() {
    // Em que QUADRO cada cartão parte, e em que quadro o último assenta.
    fn run(eps: f64, n: usize) -> (Vec<usize>, usize) {
        let dt = 1.0 / 60.0;
        let mut starts = vec![usize::MAX; n];
        let mut secs = 0.0;
        let mut m = UiMotion::default();
        for f in 0..240 {
            for (i, start) in starts.iter_mut().enumerate() {
                #[allow(clippy::cast_precision_loss)]
                let due = i as f64 * eps;
                let t = m.animate(id(i as u64), f32::from(u8::from(secs > due)), Role::Travel);
                if t > 0.0 && *start == usize::MAX {
                    *start = f;
                }
            }
            secs += dt;
            m.advance(dt);
            if (0..n).all(|i| m.get(id(i as u64)).is_some_and(|v| (v - 1.0).abs() < 0.01)) {
                return (starts, f);
            }
        }
        (starts, 240)
    }
    println!("\n eps  | n | quadros entre vizinhos | entrada inteira");
    println!("------|---|------------------------|----------------");
    for eps in [0.020, 0.035, 0.050, 0.065, 0.080] {
        for n in [3usize, 7] {
            let (st, end) = run(eps, n);
            let gap = st[1].saturating_sub(st[0]);
            #[allow(clippy::cast_precision_loss)]
            let secs = end as f64 / 60.0;
            println!(" {eps:.3}| {n} |          {gap:2} quadros       |   {secs:.2} s");
        }
    }
}

/// ⭐ **A CASCATA é VISIVELMENTE sequencial — e a régua é o QUADRO, não o segundo.**
///
/// ⚠️ **Este gate SUBSTITUI um que foi retirado por ter o critério errado.** O anterior chamava-se
/// `the_cascade_still_reads_as_one_gesture` e exigia `(n−1)·ε < assentamento`; era invenção minha,
/// e o smoke mostrou que uma cascata pode durar mais que o assentamento e continuar a ler-se bem.
/// O que ele **não** media era a única coisa que o olho reprovou: a separação entre vizinhos.
///
/// Três quadros a 60 Hz (50 ms) é o que separa *uma sequência* de *um bloco*.
/// *Mutação: `0,020`, o valor que o Enio reprovou ⇒ 1 quadro ⇒ sangra.*
#[test]
fn the_cascade_is_visibly_sequential_not_simultaneous() {
    const MIN_FRAMES: usize = 3;
    let dt = 1.0 / 60.0;
    let mut m = UiMotion::default();
    let mut secs = 0.0;
    let (mut a, mut b) = (usize::MAX, usize::MAX);
    for f in 0..240 {
        for i in 0..2u64 {
            #[allow(clippy::cast_precision_loss)]
            let due = i as f64 * CASCADE_STAGGER_SECS;
            let t = m.animate(id(i), f32::from(u8::from(secs > due)), Role::Travel);
            if t > 0.0 {
                if i == 0 && a == usize::MAX {
                    a = f;
                } else if i == 1 && b == usize::MAX {
                    b = f;
                }
            }
        }
        secs += dt;
        m.advance(dt);
    }
    assert!(
        b.saturating_sub(a) >= MIN_FRAMES,
        "vizinhos partem a {} quadro(s) — o olho le isso como SIMULTANEO",
        b.saturating_sub(a)
    );
}

/// ⚠️ **E a entrada acaba a tempo — o outro lado, agora medido no que importa.** O artista abriu a
/// paleta para a USAR; um cartão que chega depois de ele já estar a ler chegou tarde.
/// *Mutação: `0,080` ⇒ 0,65 s na paleta grande ⇒ sangra.*
#[test]
fn the_whole_entrance_lands_inside_its_budget() {
    /// A maior paleta REAL: uma categoria por token `NodeCat*` (a global tem 3).
    const N_MAX: u64 = 7;
    /// ⚠️ Do lado do PRODUTO: 0,48 s é o que `ε = 0,050` entrega, e a barra deixa uma casa de folga
    /// sem admitir o 0,065 (0,57 s), que compra os MESMOS três quadros por mais espera.
    const BUDGET_S: f64 = 0.55;
    let dt = 1.0 / 60.0;
    let mut m = UiMotion::default();
    let mut secs = 0.0;
    for f in 0..240 {
        for i in 0..N_MAX {
            #[allow(clippy::cast_precision_loss)]
            let due = i as f64 * CASCADE_STAGGER_SECS;
            m.animate(id(i), f32::from(u8::from(secs > due)), Role::Travel);
        }
        secs += dt;
        m.advance(dt);
        if (0..N_MAX).all(|i| m.get(id(i)).is_some_and(|v| (v - 1.0).abs() < 0.01)) {
            #[allow(clippy::cast_precision_loss)]
            let took = f as f64 * dt;
            assert!(took <= BUDGET_S, "a entrada demorou {took:.2} s");
            return;
        }
    }
    panic!("a entrada nao acabou");
}

/// ⚠️ **E ela é VISÍVEL — o piso que o SMOKE escreveu.** A primeira versão pedia *um* quadro, que
/// é condição **necessária e não suficiente**: a 60 Hz um quadro lê-se como simultâneo, e foi
/// exactamente assim que a wave foi reprovada. Três.
///
/// ⚠️ **Pin de COMPILAÇÃO e não teste**, porque os dois lados são constantes: o `clippy` apontou-o
/// (*«this assertion has a constant value»*) e tem razão — uma pergunta cuja resposta o compilador
/// já sabe deve falhar no compilador. *Mutação: `0.015`, o valor que esta wave shipou primeiro
/// ⇒ o crate deixa de compilar.*
const _: () = assert!(
    CASCADE_STAGGER_SECS >= 3.0 / 60.0,
    "CASCADE_STAGGER_SECS abaixo de tres quadros a 60 Hz: o Enio leu isso como SIMULTANEO"
);

/// ⭐ **O quadro da ABERTURA alveja ZERO — sem isto não há cascata, e a suíte ficaria verde.**
///
/// A lei do substrato é que a **primeira vista CHEGA ao alvo**; se o cartão 0 nascer alvejado em
/// `1.0` ele aparece assente e o gesto perde a cabeça. *Mutação: `>=` em vez de `>` ⇒ sangra.*
#[test]
fn every_card_is_targeted_at_zero_on_the_frame_the_palette_opens() {
    for i in 0..7 {
        assert_eq!(
            cascade_target(0.0, i),
            0.0,
            "o cartao {i} nasce ja alvejado — a entrada dele nao acontece"
        );
    }
    assert_eq!(cascade_target(0.001, 0), 1.0, "e o quadro seguinte parte");
}

/// A ordem CHEGA: o cartão `i` não parte antes da vez dele, e parte quando ela chega.
#[test]
fn a_later_card_leaves_later() {
    assert_eq!(cascade_target(CASCADE_STAGGER_SECS * 2.5, 0), 1.0);
    assert_eq!(cascade_target(CASCADE_STAGGER_SECS * 2.5, 2), 1.0);
    assert_eq!(cascade_target(CASCADE_STAGGER_SECS * 2.5, 3), 0.0);
}

/// ⚠️ **A subida é `Role::Travel`, logo o *reduced motion* MATA-A — e o fade sobrevive.**
///
/// Este é o PRIMEIRO consumidor de `Travel` no produto: até aqui o interruptor de acessibilidade
/// estava ligado a um cabo sem lâmpada. *Mutação: `cascade_rise` a ignorar `travels` ⇒ sangra.*
#[test]
fn reduced_motion_takes_the_rise_and_leaves_the_arrival() {
    assert!(
        cascade_rise(0.0, true) > 0.0,
        "com movimento, o cartao sobe"
    );
    assert_eq!(
        cascade_rise(0.0, false),
        0.0,
        "sem movimento, ele CHEGA sem viajar"
    );
    assert_eq!(cascade_rise(1.0, true), 0.0, "assente e assente");
}

// ─────────────────────────────────────────────────────────────────────────────
// A ROLAGEM SUAVE (E2) — a roda mexe um ALVO, a superfície desliza até lá.
// ─────────────────────────────────────────────────────────────────────────────

/// ⭐ **A roda acumula no ALVO, e é isso que faz girar depressa cobrir a distância toda.**
///
/// Somando ao valor EM VOO, cada nova volta parte de uma posição que ainda não chegou e a
/// superfície anda **menos** do que o dedo pediu — o defeito clássico de toda rolagem suave
/// escrita à pressa. *Mutação: a roda a ler `panel_scroll` (vivo) ⇒ o total encolhe.*
#[test]
fn spinning_the_wheel_fast_covers_the_whole_distance() {
    use crate::interaction::WidgetStore;
    let mut store = WidgetStore::with_capacity(4);
    let p = NodeId(7);
    // ⚠️ **A fixture TEM de conter o fenómeno.** Sem um valor VIVO atrasado, `panel_scroll` cai no
    //    alvo e ler um ou outro é a mesma coisa — a primeira versão deste gate ficou VERDE com a
    //    roda a ler o vivo, porque não havia vivo. Aqui a superfície fica sempre 40% para trás, que
    //    é o que a mola faz entre duas voltas de roda seguidas.
    for _ in 0..5 {
        let cur = store.panel_scroll_target(p);
        store.set_panel_scroll(p, cur + 100.0);
        let t = store.panel_scroll_target(p);
        store.set_panel_scroll_live(p, t * 0.6);
    }
    assert!(
        (store.panel_scroll_target(p) - 500.0).abs() < 1e-3,
        "o alvo tem de somar as cinco voltas, e somou {}",
        store.panel_scroll_target(p)
    );
}

/// ⚠️ **Sem substrato, `panel_scroll` É o alvo — o mundo pré-wave, byte-idêntico.**
///
/// É a neutralidade que torna esta wave segura: um store que ninguém tiquou desenha exactamente
/// onde desenhava. *Mutação: o `or_else` a devolver 0.0 ⇒ toda superfície salta para o topo.*
#[test]
fn an_untocked_store_paints_where_it_always_painted() {
    use crate::interaction::WidgetStore;
    let mut store = WidgetStore::with_capacity(4);
    let p = NodeId(7);
    store.set_panel_scroll(p, 240.0);
    assert!((store.panel_scroll(p) - 240.0).abs() < 1e-3);
}

/// E o vivo, quando existe, é o que o pintor vê — enquanto o alvo fica onde a roda o pôs.
#[test]
fn the_painter_sees_the_live_offset_and_the_wheel_sees_the_target() {
    use crate::interaction::WidgetStore;
    let mut store = WidgetStore::with_capacity(4);
    let p = NodeId(7);
    store.set_panel_scroll(p, 300.0);
    store.set_panel_scroll_live(p, 120.0);
    assert!(
        (store.panel_scroll(p) - 120.0).abs() < 1e-3,
        "o pintor vê o vivo"
    );
    assert!(
        (store.panel_scroll_target(p) - 300.0).abs() < 1e-3,
        "a roda vê o alvo"
    );
}
