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

/// ⭐⭐⭐ **O ROTEIRO DIZ QUE OSSOS GOVERNAM A BARRA, E ENSINA A CURA ONDE ELA APARECE.**
///
/// ⛔⛔⛔ **A metade dos ossos nasceu do report do dono de 2026-09-19** — *«Bone 14 está ligado à
/// imagem e não ao vetor. Bones 1, 2 e 3 estão ligados na barra laranja. O que vc mandou fazer não
/// funcionou»*. A cena tem **dois** esqueletos de três ossos (um na barra vectorial, outro no braço
/// pintado) e o roteiro nomeava um osso do segundo ao lado de uma lição sobre a primeira: *seguir o
/// roteiro à letra não podia funcionar*.
///
/// ⚠️ **A redacção anterior prendia a lição ao aviso «na BARRA laranja o mesmo pincel mostra 8
/// pontos e mais nada», e essa premissa MORREU no mesmo dia:** desde a cura do pincel a mancha
/// pousa no CONTORNO, logo o meio da barra deixou de ser um sítio onde não há o que pintar — ele
/// passou a ser o sítio que a F30 existe para servir.
///
/// ⛔⛔ **As três metades são três regressões diferentes:** o roteiro pode deixar de dizer que
/// ossos tocam a barra (e o artista escolhe um do braço pintado, como eu fiz); pode deixar de
/// nomear a ferramenta da outra saída (e ele não sabe com que mão fazer); e pode deixar de ligar a
/// lição ao pincel, que é o gesto que a wave existe para destravar.
#[test]
fn o_roteiro_ensina_a_acrescentar_um_ponto_onde_falta_controlo() {
    let texto = include_str!("smoke_bone.rs");
    for agulha in [
        "CANETA",
        "SOBREVIVE",
        "Weight",
        "BARRA LARANJA obedece",
        "{nos} PONTOS",
    ] {
        assert!(
            texto.contains(agulha),
            "o roteiro deixou de dizer «{agulha}» — sem ele o artista fica com a barra e nenhuma \
             saida, que e' exactamente o estado que o dono reportou"
        );
    }
    // ⛔ E a lição vive JUNTO da linha que nomeia os ossos da barra, não solta no fim: quem lê
    // *quem governa a barra* tem de encontrar ali *o que fazer com ela*. ⚠️ A régua é a distância
    // entre as duas — *duas linhas separadas por vinte lêem-se como dois assuntos*.
    let quem = texto
        .find("BARRA LARANJA obedece")
        .expect("quem governa a barra");
    let pincel = texto
        .find("PINCEL DE PESO NA BARRA")
        .expect("a licao do pincel");
    let caneta = texto.find("A OUTRA SAIDA").expect("a outra saida");
    // ⚠️⚠️ **A régua é o VÃO entre as lições, e não a distância entre os princípios delas** — a 1.ª
    // redacção media `inicio → inicio` e reprovou no dia em que a lição do meio **cresceu**, sobre
    // um roteiro correcto. *Uma régua que cresce com o tamanho do que ela separa mede a coisa
    // errada.*
    let vao = |a: usize, b: usize| {
        let fim = texto[a..].find(");").map_or(a, |k| a + k);
        b.saturating_sub(fim)
    };
    assert!(
        pincel > quem && vao(quem, pincel) < 400,
        "a licao do pincel ficou a {} bytes do fim de quem governa a barra",
        vao(quem, pincel)
    );
    assert!(
        caneta > pincel && vao(pincel, caneta) < 400,
        "a outra saida ficou a {} bytes do fim da licao do pincel",
        vao(pincel, caneta)
    );
}

/// ⭐⭐⭐ **OS DOIS ESQUELETOS DA CENA NÃO PARTILHAM UM ÚNICO OSSO** — o facto que o report do dono
/// expôs, e que nenhum gate afirmava.
///
/// ⚠️ **Ele mede a CENA, não o texto:** o roteiro pode nomear os ossos certos e a cena mudar por
/// baixo dele. A barra vectorial e o braço pintado têm cadeias próprias, e é por isso que escolher
/// um osso de uma e pintar na outra **não faz nada e está certo**.
#[test]
fn a_barra_e_o_braco_pintado_tem_esqueletos_separados() {
    let mut sim = ph2d_ecs::SimWorld::default();
    let barra = super::cadeia(
        &mut sim,
        ph2d_skeleton_demo::ARM_A,
        ph2d_skeleton_demo::ARM_B,
        ph2d_skeleton_demo::ARM_BONES,
    )
    .expect("a cadeia da barra monta");
    let pintado = super::cadeia(&mut sim, [3.6, 2.5], [7.4, 2.5], 3).expect("a do braco pintado");
    let da_barra = ph2d_skeleton_live::esqueletos::ossos_desde(&sim, barra);
    let do_braco = ph2d_skeleton_live::esqueletos::ossos_desde(&sim, pintado);
    assert_eq!(da_barra.len(), 3, "a barra deixou de ter tres ossos");
    assert_eq!(
        do_braco.len(),
        3,
        "o braco pintado deixou de ter tres ossos"
    );
    assert!(
        da_barra.iter().all(|e| !do_braco.contains(e)),
        "os dois esqueletos partilham um osso — a licao «escolher um osso do braco pintado e \
         pintar na barra nao faz nada» deixou de ser verdade"
    );
}

/// ⭐⭐ **A LISTA DE OSSOS DA BARRA CONTA DO PRIMEIRO PARA O ÚLTIMO.**
///
/// ⛔ A 1.ª redacção usava a [`ph2d_skeleton_live::esqueletos::ossos_desde`], que ordena por
/// `to_bits` — no bevy, a criação **invertida** —, e a frase saía *«Bone 3, Bone 2, Bone 1»*.
/// *Uma lista que conta ao contrário lê-se como um defeito, e o dono não tem como saber que não é.*
///
/// ⚠️ **O CONTROLO é a 2.ª asserção:** sem ela, uma cadeia cujos nomes já viessem ordenados por
/// acaso deixaria este gate verde sobre a lei errada.
#[test]
fn a_lista_de_ossos_da_barra_conta_do_primeiro_para_o_ultimo() {
    let mut sim = ph2d_ecs::SimWorld::default();
    let raiz = super::cadeia(
        &mut sim,
        ph2d_skeleton_demo::ARM_A,
        ph2d_skeleton_demo::ARM_B,
        ph2d_skeleton_demo::ARM_BONES,
    )
    .expect("a cadeia monta");
    let ordem = super::cadeia_em_ordem(&sim, raiz);
    assert_eq!(ordem.len(), 3, "a cadeia da barra deixou de ter tres ossos");
    assert_eq!(ordem[0], raiz, "a lista nao comeca na raiz");
    assert_eq!(
        ordem[2],
        super::ponta_da_cadeia(&sim, raiz),
        "a lista nao acaba na ponta"
    );
    // ⭐ O CONTROLO: o conjunto que a casa usa como RÉGUA vem noutra ordem, e é por isso que esta
    // porta existe.
    let conjunto = ph2d_skeleton_live::esqueletos::ossos_desde(&sim, raiz);
    assert_ne!(
        conjunto, ordem,
        "as duas ordens coincidiram — ou o bevy mudou a numeracao, ou esta porta deixou de ser \
         necessaria e a frase pode voltar a sair do conjunto"
    );
}

/// ⭐⭐⭐ **O ROTEIRO CONTA OS PONTOS DA BARRA, NUNCA OS AFIRMA.**
///
/// ⛔⛔ **A frase «os oito nós da barra» morreu no mesmo dia em que foi escrita** (2026-09-19): o
/// dono mandou *«criar a subdivisão visível logo na associação com os ossos»*, e desde então a
/// contagem é função do ESQUELETO — o osso mais curto a dividir por três. *Um roteiro que afirma um
/// número que a cena deriva envelhece na wave seguinte.*
#[test]
fn o_roteiro_nao_afirma_uma_contagem_de_nos() {
    let texto = include_str!("smoke_bone.rs");
    let linha = texto
        .split("PINCEL DE PESO NA BARRA")
        .nth(1)
        .expect("a licao do pincel na barra");
    let cabeca = &linha[..linha.len().min(900)];
    assert!(
        cabeca.contains("{nos} PONTOS"),
        "a licao do pincel deixou de CONTAR os pontos: {cabeca:?}"
    );
    for morta in ["oito nos da barra", "oito pontos coloridos"] {
        assert!(
            !texto.contains(morta),
            "o roteiro voltou a AFIRMAR «{morta}» — a contagem e' derivada do esqueleto desde que a \
             subdivisao do bind existe"
        );
    }
}

/// ⭐⭐⭐ **O ROTEIRO SÓ NOMEIA RÓTULOS QUE O PAINEL DE FACTO PINTA** (F29).
///
/// ⚠️⚠️ **Um passo que manda carregar numa linha AFIRMA que ela está lá** — e *o dono aprova o
/// smoke com o passo impossível dentro*. É a lição que a família da escultura pagou quando um
/// roteiro mandou subir um controlo que vivia num nível do painel que a cena não abre.
///
/// ⚠️ **A régua é a tabela de TEXTO e não uma lista escrita à mão:** cada nome citado tem de ser o
/// que a [`ph2d_i18n::tr`] devolve para a chave que o painel usa. ⛔ Mudar o rótulo no catálogo e
/// esquecer o roteiro reprova aqui, que é o dia certo.
#[test]
fn o_roteiro_do_pincel_nomeia_rotulos_que_existem() {
    let texto = include_str!("smoke_bone.rs");
    let bloco = texto
        .split("DOIS MODOS DE PESO")
        .nth(1)
        .expect("a licao dos dois modos do pincel de peso");
    let bloco = &bloco[..bloco.len().min(1200)];
    for chave in [
        "panel.vector.bone.weight.mode",
        "panel.vector.bone.weight.mode.cumulative",
        "panel.vector.bone.weight.mode.absolute",
        "panel.vector.bone.weight.target",
        "panel.vector.bone.weight.direction",
    ] {
        let rotulo = ph2d_i18n::tr(chave);
        assert_ne!(rotulo, chave, "a chave `{chave}` nao tem texto no catalogo");
        assert!(
            bloco.contains(rotulo),
            "o roteiro dos dois modos nao nomeia «{rotulo}» (a chave `{chave}`) — ou ele deixou de \
             ensinar o controlo, ou o rotulo mudou no catalogo e o passo ficou impossivel"
        );
    }
    // ⛔ E a metade NEGATIVA: ele não pode nomear um rótulo que não existe.
    for inventado in ["Weight Target", "Absolute Mode", "Replace"] {
        assert!(
            !bloco.contains(inventado),
            "o roteiro nomeia «{inventado}», que nao e' rotulo nenhum deste painel"
        );
    }
}
