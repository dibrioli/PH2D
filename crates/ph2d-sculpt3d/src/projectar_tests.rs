//! Os gates da **lei do raio** do [`crate::Verb::SceneProject`], cobrada por
//! FORMA FECHADA — `SPEC_unblocked_brushes.md` §6.3 e §6.4.
//!
//! ⚠️ **Aqui a lei está SOZINHA no numerador:** um plano a uma distância que se
//! escreve à mão, e a resposta certa é essa distância. A cadeia de peso, a
//! curva e a força entram na bancada do corpus, não aqui — *um traço inteiro
//! mistura seis aplicações da curva com a re-medição da §6.6, e um desvio nele
//! não diz qual dos dois falhou* (espec §8.4).

use super::distancia;
use crate::FolgaModo;
use ph2d_mesh::{Face, Mesh, Pose};

/// Um quadrado grande em `z = alt`, virado para cima — o alvo mais simples que
/// existe, e aquele cuja distância se lê sem calcular nada.
fn plano(alt: f32) -> Mesh {
    let l = 8.0;
    Mesh::from_parts(
        vec![[-l, -l, alt], [l, -l, alt], [l, l, alt], [-l, l, alt]],
        vec![Face::tri(0, 1, 2), Face::tri(0, 2, 3)],
    )
    .expect("o plano do gate")
}

/// O de baixo do dab: para dentro do ecrã numa vista de topo.
const PARA_BAIXO: [f32; 3] = [0.0, 0.0, -1.0];
const ORIGEM: [f32; 3] = [0.0, 0.0, 0.0];

/// ⭐⭐ **A DISTÂNCIA É A DO ALVO MAIS PERTO, e a folga entra DEPOIS do
/// vencedor** (espec §6.3, regras 1 e 4).
///
/// ⛔⛔ **A ordem é load-bearing e a segunda metade deste gate é o que a prende:**
/// com a folga aplicada ANTES da competição, os dois candidatos encolhiam por
/// igual e o vencedor seria o mesmo — por acidente. O que separa as duas ordens
/// é o **valor**, e é por isso que ele é afirmado e não só a escolha.
#[test]
fn ganha_o_alvo_mais_perto_e_a_folga_entra_depois() {
    let alvos = vec![(plano(-0.8), Pose::IDENTITY), (plano(-0.3), Pose::IDENTITY)];
    let d = distancia(
        ORIGEM,
        PARA_BAIXO,
        Pose::IDENTITY,
        &alvos,
        false,
        0.0,
        FolgaModo::DoAlvo,
    )
    .expect("o raio tem de acertar em alguma coisa");
    assert!(
        (d - 0.3).abs() < 1e-6,
        "ganhou o alvo errado: {d} (o perto está a 0,3 e o longe a 0,8)"
    );
    // ⭐ Com folga, o vencedor continua a ser o perto E o valor é o dele menos
    // a folga — `0,3 − 0,1`, nunca `0,8 − 0,1`.
    let d = distancia(
        ORIGEM,
        PARA_BAIXO,
        Pose::IDENTITY,
        &alvos,
        false,
        0.1,
        FolgaModo::DoAlvo,
    )
    .expect("acerto");
    assert!(
        (d - 0.2).abs() < 1e-6,
        "a folga entrou no sítio errado da ordem: {d}"
    );
}

/// ⛔⛔ **NENHUM ACERTO ⇒ O VÉRTICE NÃO SE MOVE** (espec §6.3.3), e as três
/// maneiras de lá chegar.
///
/// ⚠️ **`None` e `Some(0.0)` não são a mesma coisa**, e este gate é metade da
/// razão de o tipo os separar: um acerto a distância zero ainda subtrai a folga
/// e pode **afastar** a peça (§6.4), enquanto um não-acerto é inerte.
#[test]
fn sem_acerto_nao_ha_lei() {
    let vazio: Vec<(Mesh, Pose)> = Vec::new();
    assert_eq!(
        distancia(
            ORIGEM,
            PARA_BAIXO,
            Pose::IDENTITY,
            &vazio,
            false,
            0.0,
            FolgaModo::DoAlvo
        ),
        None,
        "sem outra peça na cena o pincel tem de ser inerte"
    );
    // O alvo está ATRÁS e os dois sentidos estão desligados.
    let acima = vec![(plano(0.5), Pose::IDENTITY)];
    assert_eq!(
        distancia(
            ORIGEM,
            PARA_BAIXO,
            Pose::IDENTITY,
            &acima,
            false,
            0.0,
            FolgaModo::DoAlvo
        ),
        None,
        "um alvo do lado errado sem os dois sentidos não pode ser alcançado"
    );
    // ⭐ O CONTROLO: com os dois sentidos ele é alcançado, e com sinal
    // NEGATIVO. Sem esta metade o gate acima ficaria verde sobre uma lei que
    // nunca acerta em nada.
    let d = distancia(
        ORIGEM,
        PARA_BAIXO,
        Pose::IDENTITY,
        &acima,
        true,
        0.0,
        FolgaModo::DoAlvo,
    )
    .expect("com os dois sentidos ele acerta");
    assert!(
        (d + 0.5).abs() < 1e-6,
        "um acerto para trás tem de vir com `d` negativo, e veio {d}"
    );
}

/// ⛔⛔ **AS DUAS ARMADILHAS DA FOLGA, as duas MEDIDAS no alvo** (espec §6.4) —
/// e este gate existe para elas serem **reproduzidas de propósito**, não
/// descobertas por alguém a ler o rótulo.
///
/// | caso | o que o rótulo *«distância mínima»* sugere | o que acontece |
/// |---|---|---|
/// | folga `0,6` sobre um vão de `0,5` | o barro pára a `0,1` do alvo | ele **AFASTA-SE** `0,1` |
/// | folga `0,1` num acerto para TRÁS | a excursão encolhe `0,1` | ela **CRESCE** `0,1` |
///
/// ⭐⭐ **E desde 17/09 a alternativa simétrica SHIPA, como o outro botão** —
/// decisão do dono (*«cada modo com opção»*), depois de a espec §10.3 a deixar
/// em aberto. ⚠️ **Este gate mudou de forma com ela:** ele comparava a lei do
/// alvo contra uma função SOLTA, e hoje corre **as duas pela mesma porta**
/// ([`crate::FolgaModo`]) — *uma lei medida fora do caminho do produto não
/// afirma nada sobre o produto*, que é a armadilha que esta linha já pagou na
/// ponte da curva do contorno e na da pose.
///
/// ⚠️ A 3.ª metade é o **CONTROLO**: no sentido de avanço as duas leis têm de
/// dar o MESMO número. Sem ela, um modo que fosse a outra lei com outro nome
/// passaria nas duas primeiras.
#[test]
fn a_folga_so_e_minima_no_sentido_de_avanco() {
    let abaixo = vec![(plano(-0.5), Pose::IDENTITY)];
    let acima = vec![(plano(0.5), Pose::IDENTITY)];

    // (a) A folga maior que o vão INVERTE o sentido.
    let d = distancia(
        ORIGEM,
        PARA_BAIXO,
        Pose::IDENTITY,
        &abaixo,
        false,
        0.6,
        FolgaModo::DoAlvo,
    )
    .expect("acerto");
    assert!(
        (d + 0.1).abs() < 1e-6,
        "a folga maior que o vão tinha de dar `-0,1` (o barro afasta-se), e deu {d}"
    );
    let s = distancia(
        ORIGEM,
        PARA_BAIXO,
        Pose::IDENTITY,
        &abaixo,
        false,
        0.6,
        FolgaModo::Simetrica,
    )
    .expect("acerto");
    assert!(
        (s - 0.0).abs() < 1e-6,
        "a lei simétrica tem de PARAR em zero, nunca inverter — deu {s}"
    );

    // (b) Num acerto para trás a folga SOMA em magnitude.
    let d = distancia(
        ORIGEM,
        PARA_BAIXO,
        Pose::IDENTITY,
        &acima,
        true,
        0.1,
        FolgaModo::DoAlvo,
    )
    .expect("acerto");
    assert!(
        (d + 0.6).abs() < 1e-6,
        "a folga tinha de CRESCER a excursão para `-0,6`, e deu {d}"
    );
    let s = distancia(
        ORIGEM,
        PARA_BAIXO,
        Pose::IDENTITY,
        &acima,
        true,
        0.1,
        FolgaModo::Simetrica,
    )
    .expect("acerto");
    assert!(
        (s + 0.4).abs() < 1e-6,
        "a lei simétrica tem de ENCOLHER a excursão para `-0,4` — deu {s}"
    );

    // ⭐ E o CONTROLO de que as duas leis não são a mesma função com outro
    // nome: no sentido de avanço elas **concordam**, e é só nas duas
    // armadilhas que se separam.
    let d = distancia(
        ORIGEM,
        PARA_BAIXO,
        Pose::IDENTITY,
        &abaixo,
        false,
        0.1,
        FolgaModo::DoAlvo,
    )
    .expect("acerto");
    let s = distancia(
        ORIGEM,
        PARA_BAIXO,
        Pose::IDENTITY,
        &abaixo,
        false,
        0.1,
        FolgaModo::Simetrica,
    )
    .expect("acerto");
    assert!(
        (d - s).abs() < 1e-6,
        "no sentido de avanço as duas leis têm de dar o mesmo número: {d} contra {s}"
    );
}

/// ⭐⭐⭐ **A DISTÂNCIA ATRAVESSA A ESCALA DAS DUAS PEÇAS** — o gate que o
/// cabeçalho do [`crate::projectar`] nomeia.
///
/// ⚠️⚠️ **O `t` de um acerto vem em unidades da peça CONSULTADA** (o
/// [`ph2d_mesh::Ray`] normaliza a direcção, e a
/// [`ph2d_mesh::Pose::ray_to_local`] leva o raio ao espaço da malha) ⇒ dois
/// candidatos em peças com escalas diferentes **não são comparáveis** pelo `t`
/// cru, e a resposta tem de vir na régua do objecto ACTIVO. *Sem a conversão os
/// dois números continuam `f32` plausíveis, e o `min |d|` da espec escolhe pelo
/// número errado — em silêncio.*
///
/// A fixtura: um alvo **escalado 2×** cujo plano local está em `z = −0,25`, ou
/// seja em `z = −0,5` no mundo. A resposta certa é `0,5`; sem a conversão seria
/// `0,25`.
#[test]
fn a_distancia_atravessa_a_escala_das_duas_pecas() {
    let alvo = vec![(plano(-0.25), Pose::new([0.0, 0.0, 0.0], 2.0))];
    let d = distancia(
        ORIGEM,
        PARA_BAIXO,
        Pose::IDENTITY,
        &alvo,
        false,
        0.0,
        FolgaModo::DoAlvo,
    )
    .expect("acerto");
    assert!(
        (d - 0.5).abs() < 1e-6,
        "a escala do ALVO não atravessou: {d} (o `t` cru daria 0,25)"
    );

    // ⭐ E a do ACTIVO pelo outro lado: com o activo a `2×`, o mesmo vão de
    // `0,5` no mundo mede `0,25` na régua dele — que é a régua em que o dab
    // escreve.
    let alvo = vec![(plano(-0.5), Pose::IDENTITY)];
    let d = distancia(
        ORIGEM,
        PARA_BAIXO,
        Pose::new([0.0, 0.0, 0.0], 2.0),
        &alvo,
        false,
        0.0,
        FolgaModo::DoAlvo,
    )
    .expect("acerto");
    assert!(
        (d - 0.25).abs() < 1e-6,
        "a escala do ACTIVO não atravessou: {d}"
    );
}

/// ⭐ **A TRANSLAÇÃO DO ALVO também atravessa** — o irmão barato do gate acima,
/// e ele existe porque as duas metades de uma pose falham por razões
/// diferentes: trocar o ponto pelo vector (ou vice-versa) dá um `d` plausível e
/// errado, que é exactamente o que a espec §6.3 avisa por escrito.
#[test]
fn a_translacao_do_alvo_atravessa_e_a_direccao_nao_a_apanha() {
    // O plano local está em `z = 0`, e a peça está pousada em `z = −0,5`.
    let alvo = vec![(plano(0.0), Pose::new([0.0, 0.0, -0.5], 1.0))];
    let d = distancia(
        ORIGEM,
        PARA_BAIXO,
        Pose::IDENTITY,
        &alvo,
        false,
        0.0,
        FolgaModo::DoAlvo,
    )
    .expect("acerto");
    assert!(
        (d - 0.5).abs() < 1e-6,
        "a translação do alvo não atravessou: {d}"
    );
    // ⛔ O CONTROLO: a partir de um ponto de partida deslocado, a distância
    // muda pelo mesmo tanto — se a translação entrasse na DIREÇÃO em vez de no
    // PONTO, este número não se mexia.
    let d = distancia(
        [0.0, 0.0, 0.25],
        PARA_BAIXO,
        Pose::IDENTITY,
        &alvo,
        false,
        0.0,
        FolgaModo::DoAlvo,
    )
    .expect("acerto");
    assert!(
        (d - 0.75).abs() < 1e-6,
        "a origem do raio não é tratada como PONTO: {d}"
    );
}

/// Um tampo fino no plano `z = 0` — a peça que o gesto esculpe.
///
/// ⚠️ **Grelha e não esfera:** aqui a pergunta é *para que lado o barro vai*, e
/// num plano chato a resposta lê-se no sinal de um `z` sem nenhuma geometria a
/// misturar-se com ela.
fn tampo() -> Mesh {
    const N: usize = 21;
    let f = |k: usize| -1.0 + (k as f32) * (2.0 / (N as f32 - 1.0));
    let mut pos = Vec::with_capacity(N * N);
    for j in 0..N {
        for i in 0..N {
            pos.push([f(i), f(j), 0.0]);
        }
    }
    let mut faces = Vec::with_capacity((N - 1) * (N - 1) * 2);
    for j in 0..N - 1 {
        for i in 0..N - 1 {
            let idx = |a: usize, b: usize| u32::try_from(b * N + a).expect("cabe");
            let (a, b, c, d) = (idx(i, j), idx(i + 1, j), idx(i + 1, j + 1), idx(i, j + 1));
            faces.push(Face::tri(a, b, c));
            faces.push(Face::tri(a, c, d));
        }
    }
    Mesh::from_parts(pos, faces).expect("o tampo do gate")
}

// ⛔⛔⛔ **AQUI VIVIA `a_inversao_nega_a_translacao_e_nao_vira_o_raio`, e ele
// foi APAGADO em 15/09 porque o SUJEITO dele deixou de existir** — ordem do
// dono depois do smoke da `=45`: *«não vi utilidade na feature Scene Project +
// CTRL. Melhor retirá-la e documentá-la como indesejada.»*
//
// ⚠️ **Um gate cujo sujeito foi retirado não se afrouxa nem se marca
// `#[ignore]`: apaga-se.** Deixá-lo a afirmar uma lei que o produto já não tem
// seria um vermelho permanente, e mantê-lo verde exigiria afrouxar a asserção
// até ela não dizer nada. Quem guarda a decisão agora é o gate
// `o_ctrl_saiu_do_projectar_e_o_corpus_diz_quais_fixturas_isso_custou`, na
// banca do oráculo, que mede o **oposto**: que o gesto é inerte.
//
// ⭐⭐ **O que ele provava fica escrito, porque foi caro e porque uma recusa
// sem mecanismo convida a reconstrução.** As duas leis candidatas eram
// indistinguíveis no caso comum, e só uma célula as separa:
//
// | lei | alvo ABAIXO, `Ctrl` | alvo ACIMA, `Ctrl`, sem os dois sentidos |
// |---|---|---|
// | virar o raio | sobe | **sobe** (o raio virado encontra-o) |
// | **negar a translação** (a certa) | sobe | **nada se move** (o raio para baixo não acerta) |
//
// ⇒ com o alvo abaixo as duas dão o mesmo sinal no primeiro dab, e foi por
// isso que a errada shipou. O oráculo concordou com a segunda
// (`1,415e-1 → 2,384e-7` nas duas fixturas invertidas), e num dab a negação é
// **exacta** (espec §6.6). *Se alguém reabrir esta feature, começa daqui e não
// do zero.*

/// ⭐⭐⭐ **A DIRECÇÃO DO RAIO NÃO DERIVA COM A TRINCHEIRA QUE O TRAÇO ABRE.**
///
/// No modo [`crate::ProjectMode::Plane`] a direcção é o oposto da normal da
/// ÁREA sob o dab — e se essa normal for lida da superfície VIVA, ela inclina
/// sobre o vale que os dabs anteriores cavaram: o raio parte de lado e o barro
/// é **transportado**, não empurrado.
///
/// ⛔⛔ **É o defeito que o corpus mediu:** a fixtura `projectar_normal_plano_area`
/// desviava `1,281e-1` do oráculo e o desvio era **inteiramente lateral** — no
/// oráculo aquela fixtura é **byte-idêntica** à `projectar_base` (`max abs(Δ) =
/// 0,0` nos `1 681` vértices), ou seja a direcção não se mexe ao longo dos seis
/// dabs.
///
/// ⚠️ **A cura tem DUAS metades e a segunda quase passou despercebida:**
/// ajustar o plano sobre a superfície congelada baixou o desvio para `1,036e-2`
/// e não a zero, porque o `base_nrm` é o **PRIMEIRO TOQUE** e não o pen-down —
/// um vértice que entra na pegada no 3.º dab é fotografado já inclinado.
/// ⇒ [`crate::SculptStroke::nrm0_do_pen_down`], e o desvio fecha em `1,639e-7`.
///
/// ⚠️ **A régua é o DESLOCAMENTO LATERAL e não a posição**, porque é ele que
/// nomeia o mecanismo: um desvio vertical seria força a mais, e este é o barro
/// a andar para o lado.
#[test]
fn a_direccao_do_raio_nao_deriva_com_a_trincheira() {
    let b = crate::Brush {
        verb: crate::Verb::SceneProject,
        mode: crate::RefMode::B,
        radius: 0.35,
        strength: 1.0,
        falloff: crate::Falloff::Smooth,
        project_mode: crate::ProjectMode::Plane,
        ..crate::Brush::default()
    };
    let mut mesh = tampo();
    let repouso = mesh.positions().to_vec();
    let mut s = crate::SculptStroke::default();
    s.begin(&mesh);
    s.pecas_da_cena = vec![(plano(-0.5), Pose::IDENTITY)];
    s.pose_activa = Pose::IDENTITY;
    // Seis dabs ao longo de `x`, que é o percurso que cava a trincheira.
    for k in 0..6 {
        let x = -0.3 + 0.12 * f32::from(u8::try_from(k).expect("cabe"));
        s.dab(
            &mut mesh,
            &b,
            &crate::Dab::at([x, 0.0, 0.0], b.radius, PARA_BAIXO),
            crate::Symmetry::default(),
        );
    }
    let (mut lateral, mut fundo) = (0.0f32, 0.0f32);
    for (a, r) in mesh.positions().iter().zip(&repouso) {
        lateral = lateral.max((a[0] - r[0]).abs()).max((a[1] - r[1]).abs());
        fundo = fundo.max((a[2] - r[2]).abs());
    }
    // O controlo positivo: a cena tem de conter o fenómeno.
    assert!(
        fundo > 0.4,
        "a trincheira não foi cavada (fundo {fundo:.4}) — sem ela este gate mede \
         o nada"
    );
    assert!(
        lateral < 1e-6,
        "o raio derivou: o barro andou {lateral:.3e} de LADO ao longo do traço. \
         A normal do plano está a ser lida da superfície viva (ou do primeiro \
         toque), e ela inclina sobre o vale que o próprio traço abre"
    );
}
