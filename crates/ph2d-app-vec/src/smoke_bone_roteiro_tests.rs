//! Os gates do roteiro da cena do OSSO — irmão do `smoke_bone.rs` pelo tecto de LOC, e o corte é
//! por RESPONSABILIDADE: ali mora o que a cena MONTA; aqui, o que ela PROMETE ao artista.
//!
//! ⚠️ **Um roteiro é uma AFIRMAÇÃO sobre o ecrã** — ele nomeia linhas de painel, botões e peças —, e
//! é isso que o faz envelhecer sozinho. *O dono aprova o smoke com o passo impossível dentro.*

/// ⚠️⚠️ **O nível tem DUAS leis e elas não são a mesma** (a lição que a família das mídias já
/// pagou): **ilegível ou ausente ⇒ `1`**, o caminho de OMISSÃO — a cena que o dono já aprovou —,
/// e **legível e fora de faixa ⇒ COAGIDO** à faixa, que preserva *«ele pediu uma alta»*.
///
/// ⭐ A última asserção é a que impede a [`super::NIVEIS`] de mentir: o topo declarado tem de ser
/// ALCANÇÁVEL, senão acrescentar uma cena e esquecer a constante deixa-a inatingível.
///
/// ⚠️ **E a ausência tem de dar `1` e não `0`:** a env ERA de presença (`is_some`), e todo
/// comando que o dono já tem escrito é `PH2D_VEC_BONE_SMOKE=1` — ⛔ mas também há quem a arme
/// com `=`, que é a env VAZIA.
#[test]
fn o_nivel_da_cena_dos_ossos_e_coagido_a_faixa() {
    for (v, esperado, porque) in [
        (Some("1"), 1, "a cena que o dono ja' aprovou"),
        (Some("2"), 2, "a cena do ENVELOPE"),
        (
            Some("9"),
            super::NIVEIS,
            "legivel e alto demais: COAGIDO ao topo, nao mandado para o principio",
        ),
        (Some("0"), 1, "legivel e baixo demais: coagido ao piso"),
        (Some(""), 1, "a env vazia e' como um `env VAR=` a arma"),
        (Some("sim"), 1, "um valor ilegivel cai na cena de omissao"),
        (None, 1, "sem env"),
    ] {
        assert_eq!(super::nivel_de(v), esperado, "{porque} (pedido: {v:?})");
    }
    assert_eq!(
        super::nivel_de(Some(&super::NIVEIS.to_string())),
        super::NIVEIS,
        "o topo declarado tem de ser alcancavel — senao a `NIVEIS` mente sobre quantas cenas ha'"
    );
}

/// ⭐⭐⭐ **O ROTEIRO MANDA CLICAR NO OSSO DO MEIO DO BRAÇO PINTADO, e não na PONTA.**
///
/// ⛔⛔⛔ **Isto foi MEDIDO por fotografia depois de eu ter suposto o contrário** (2026-09-19,
/// 3.º report do dono sobre o pincel de peso). A 1.ª redacção do roteiro reaproveitava o nome
/// que a lição do *Onion* já tinha à mão — o da **ponta** —, e na foto a parte visível do braço
/// lê-se quase toda **AZUL**: a zona que a ponta governa sozinha cai atrás do painel *Bones*.
/// Com o osso do MEIO a rampa inteira (azul → ciano → verde → amarelo → vermelho) cabe no
/// enquadramento, porque ele tem território dos dois lados.
///
/// ⚠️⚠️ **É a MESMA armadilha do «Bone 2» do 1.º report** — *mandar o artista julgar a
/// ferramenta no osso em que ela tem menos a mostrar*. Ela voltou porque o nome mais fácil de
/// alcançar no código não era o nome certo para o gesto.
///
/// ⭐ **As duas metades, porque as regressões são diferentes:** o índice tem de ser o do meio
/// (uma cadeia de três), e a linha do roteiro tem de nomear ESSE e não o da ponta — a segunda
/// lê o ficheiro por [`include_str!`], logo deixa de compilar se ele mudar de sítio.
#[test]
fn o_roteiro_do_pincel_de_peso_nomeia_o_osso_do_meio() {
    // (a) o índice: numa cadeia de três, `len / 2` é o do meio, qualquer que seja a ordem em
    // que o passeio a devolve.
    let mut sim = ph2d_ecs::SimWorld::default();
    let raiz = super::cadeia(&mut sim, [3.6, 2.5], [7.4, 2.5], 3).expect("a cadeia monta");
    let ossos = ph2d_skeleton_live::esqueletos::ossos_desde(&sim, raiz);
    assert_eq!(
        ossos.len(),
        3,
        "a cadeia do braco pintado deixou de ter tres ossos"
    );
    let ponta = super::ponta_da_cadeia(&sim, raiz);
    // ⚠️ **A PORTA do produto, nunca uma copia da conta** — a 1.ª redacção deste gate escrevia
    // `ossos[ossos.len() / 2]` aqui, e a mutação que trocava o índice do PRODUTO por `.last()`
    // sobrevivia: *uma cópia da lei julga a cópia.*
    let meio = super::osso_do_meio(&sim, raiz).expect("a porta responde numa cadeia de tres");
    assert_ne!(meio, raiz, "o osso escolhido e' a RAIZ da cadeia");
    assert_ne!(
        meio, ponta,
        "o osso escolhido e' a PONTA — e' a foto de 19/09 a` letra: dali o braco le^-se quase \
         todo azul, porque o vermelho cai atras do painel"
    );

    // (b) a linha do roteiro nomeia o do meio.
    let texto = include_str!("smoke_bone.rs");
    let linha = texto.split("PINCEL DE PESO:").nth(1).expect(
        "o roteiro deixou de ensinar o pincel de peso — e o dono ja' reportou 3x sobre ele",
    );
    let cabeca = &linha[..linha.len().min(120)];
    assert!(
        cabeca.contains("{meio}"),
        "o roteiro do pincel voltou a nomear outro osso que nao o do MEIO: {cabeca:?}"
    );
}

use super::{overlap_bar, painted_arm_rect};

/// ⭐⭐ **A peça que ensina a ORDEM tem de ATRAVESSAR a imagem — e não a tapar.**
///
/// ⚠️ A cena existe para mostrar que a imagem presa está na ORDEM do quadro (até 2026-09-13 ela
/// era desenhada por cima de tudo). Uma barra ao lado não distingue as duas coisas, e uma que a
/// tapasse inteira também não. Vale em qualquer `pixels_per_meter` porque a barra é DERIVADA.
#[test]
fn the_bar_that_teaches_order_crosses_the_painted_arm_without_hiding_it() {
    for ppm in [50.0_f32, 100.0, 200.0] {
        let (c, s) = painted_arm_rect(ppm);
        let (min, max) = overlap_bar(ppm);
        let (ax0, ax1) = (c[0] - s[0] * 0.5, c[0] + s[0] * 0.5);
        let (ay0, ay1) = (c[1] - s[1] * 0.5, c[1] + s[1] * 0.5);
        assert!(
            min[0] > ax0 && max[0] < ax1,
            "ppm {ppm}: a barra {min:?}..{max:?} nao deixa a imagem ver-se dos dois lados \
             ({ax0}..{ax1})"
        );
        assert!(
            min[1] < ay0 && max[1] > ay1,
            "ppm {ppm}: a barra nao atravessa a imagem de cima a baixo"
        );
    }
}

/// ⭐⭐⭐ **O ROTEIRO ENSINA A CURA ONDE A LIMITAÇÃO APARECE** — a 1.ª saída da F26, que o dono
/// escolheu em 2026-09-19 (*«primeiro 1 e depois o 2»*).
///
/// A barra laranja tem **oito** nós, todos nas duas pontas, e é ali que o artista descobre que o
/// pincel de peso não tem onde pegar. ⚠️ *Um aviso que nomeia um limite e não diz o que fazer com
/// ele é meia lição* — e o passo que faltava é o mais barato de todos: **pegar na caneta e pôr um
/// ponto onde se quer controlo**.
///
/// ⛔⛔ **As três metades são três regressões diferentes:** o roteiro pode deixar de nomear a
/// ferramenta (o artista não sabe com que mão fazer); pode deixar de dizer que o ponto **sobrevive**
/// (e aí ele reproduz o defeito de 19/09 sem saber que foi curado); e pode deixar de o ligar ao
/// pincel (que é o gesto que a wave existe para destravar).
#[test]
fn o_roteiro_ensina_a_acrescentar_um_ponto_onde_falta_controlo() {
    let texto = include_str!("smoke_bone.rs");
    for agulha in ["CANETA", "SOBREVIVE", "Weight"] {
        assert!(
            texto.contains(agulha),
            "o roteiro deixou de dizer «{agulha}» — sem ele o artista fica com a barra de oito nos \
             e nenhuma saida, que e' exactamente o estado que a F26 devolveu ao dono"
        );
    }
    // ⛔ E a lição vive JUNTO do aviso que a motiva, não solta no fim: quem lê o aviso da barra tem
    // de encontrar a cura na mesma frase. *Duas linhas separadas por vinte lêem-se como dois
    // assuntos.*
    let aviso = texto.find("BARRA laranja").expect("o aviso da barra");
    let cura = texto.find("CANETA").expect("a cura");
    assert!(
        cura > aviso && cura - aviso < 400,
        "a cura da barra de oito nos ficou longe do aviso que a motiva ({} bytes) — o artista le^ o \
         limite e nao encontra a saida",
        cura.saturating_sub(aviso)
    );
}
