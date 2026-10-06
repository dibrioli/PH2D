//! Os gates do contacto no passo (doc 109 W2) — pela porta do produto, o [`crate::step`].

use crate::step;
use ph2d_nodegraph::attr::{
    COLLIDER_BOX_COLUMN, COLLIDER_COLUMN, Column, INV_INERTIA_COLUMN, Stream,
};

/// Duas peças em `x = ±meio`, com velocidades, relógio e (opcional) colisor de raio `r`.
pub(super) fn par(meio: f32, v: f32, r: Option<f32>) -> Stream {
    let s = Stream::new(2)
        .with("P", Column::Vec2(vec![[-meio, 0.0], [meio, 0.0]]))
        .with("vel", Column::Vec2(vec![[v, 0.0], [-v, 0.0]]))
        .with("sim_t", Column::Scalar(vec![0.0, 0.0]));
    match r {
        Some(r) => s.with(COLLIDER_COLUMN, Column::Scalar(vec![r, r])),
        None => s,
    }
}

pub(super) fn col(s: &Stream, name: &str) -> Vec<[f32; 2]> {
    match s.get(name) {
        Some(Column::Vec2(v)) => v.clone(),
        _ => panic!("sem {name}"),
    }
}

pub(super) const DT: f32 = 1.0 / 60.0;

/// ⭐ **Um colisor de raio ZERO passa exactamente como nenhum** — posições e velocidades ao bit.
#[test]
fn a_zero_collider_steps_exactly_like_no_collider() {
    let sem = step(&par(0.1, 1.0, None), DT, 1.0, 0.0, 0.0, 1.0);
    let zero = step(&par(0.1, 1.0, Some(0.0)), DT, 1.0, 0.0, 0.0, 1.0);
    for c in ["P", "vel"] {
        let (a, b) = (col(&sem, c), col(&zero, c));
        for i in 0..2 {
            assert_eq!(
                (a[i][0].to_bits(), a[i][1].to_bits()),
                (b[i][0].to_bits(), b[i][1].to_bits()),
                "{c}[{i}]"
            );
        }
    }
}

/// **Marcha `tiques` passos pela porta do produto, com a MEMÓRIA do mundo de contacto** — o que o
/// `Cook` faz entre dois tiques (o mundo do rapier vive entre eles, doc 121 §9.20).
pub(super) fn corre(s: &Stream, tiques: u32) -> Stream {
    let t0 = match s.get("sim_t") {
        Some(Column::Scalar(v)) => v.first().copied().unwrap_or(0.0),
        _ => 0.0,
    };
    let (mut s, mut mundo) = (s.clone(), None);
    for k in 1..=tiques {
        #[expect(clippy::cast_precision_loss, reason = "um índice de tique")]
        let t = t0 + k as f32 * DT;
        s = crate::step_com(&s, t, 1.0, 0.0, 0.0, 1.0, &mut mundo);
    }
    s
}

/// ⭐ **A FOLGA do contacto do motor da casa** — a sobreposição que o rapier ADMITE de propósito
/// para o contacto persistir entre tiques (`allowed_linear_error`, `0,005` u à escala `1`). ⚠️ A lei
/// de antes separava ao bit num passo; esta guarda os contactos, e a pilha ganha com isso (doc 121
/// §9.20). Uma barra mais apertada que isto mediria o solver, não o produto.
const FOLGA: f32 = 0.005;

/// ⭐ **Duas peças sobrepostas saem à soma dos raios** (`size` ausente ⇒ escala 1) — num segundo, e
/// sem passar dela: o rapier corrige a penetração por uma mola amortecida (a meio segundo, de `0,8`
/// de sobreposição sobra `0,018`), nunca de um salto.
#[test]
fn overlapping_pieces_are_pushed_apart_to_their_radii() {
    let p = col(&corre(&par(0.1, 0.0, Some(0.5)), 60), "P");
    let d = p[1][0] - p[0][0];
    assert!(d >= 1.0 - FOLGA && d <= 1.0 + 1e-3, "{p:?}");
}

/// ⭐⭐ **O contacto NUNCA acrescenta velocidade** — duas peças que nascem sobrepostas, paradas,
/// separam-se em posição e continuam paradas. Somar `Δp/dt` inteiro faria delas uma explosão.
///
/// ⚠️ As CAIXAS também, e ao longo da separação toda: com a omissão do rapier (`1` estabilização)
/// duas caixas paradas e sobrepostas saíam a `0,84` u/s (ver
/// [`ph2d_contact_world::ESTABILIZACOES`]).
#[test]
fn the_contact_never_adds_speed() {
    let s = step(&par(0.1, 0.0, Some(0.5)), DT, 1.0, 0.0, 0.0, 1.0);
    assert_eq!(col(&s, "vel"), vec![[0.0, 0.0], [0.0, 0.0]]);
    let caixas = Stream::new(2)
        .with("P", Column::Vec2(vec![[-0.3, 0.0], [0.3, 0.0]]))
        .with("vel", Column::Vec2(vec![[0.0, 0.0], [0.0, 0.0]]))
        .with("sim_t", Column::Scalar(vec![0.0, 0.0]))
        .with(
            COLLIDER_BOX_COLUMN,
            Column::Vec2(vec![[0.5, 2.0], [0.5, 2.0]]),
        );
    let out = corre(&caixas, 120);
    let (p, v) = (col(&out, "P"), col(&out, "vel"));
    let d = p[1][0] - p[0][0];
    assert!(
        d >= 1.0 - FOLGA && d <= 1.0 + 0.02,
        "separaram-se a' face e PARARAM: {p:?}"
    );
    assert!(
        (v[1][0] - v[0][0]).abs() < 0.02,
        "a velocidade que sobra da separacao: {v:?}"
    );
}

/// ⭐⭐ **A aproximação é CANCELADA, não reflectida** — duas peças que se encostam a `±2 u/s`
/// ficam encostadas e param; nenhuma ressalta para trás.
#[test]
fn an_approach_is_cancelled_not_reflected() {
    // Encostadas (distância 1 = a soma dos raios) e a vir uma para a outra.
    let s = step(&par(0.5, 2.0, Some(0.5)), DT, 1.0, 0.0, 0.0, 1.0);
    let (p, v) = (col(&s, "P"), col(&s, "vel"));
    let d = p[1][0] - p[0][0];
    assert!(d >= 1.0 - FOLGA && d <= 1.0 + 1e-4, "encostadas: {p:?}");
    for (i, vi) in v.iter().enumerate() {
        assert!(
            vi[0].abs() < 1e-3,
            "a peca {i} parou em x, sem ressalto: {vi:?}"
        );
    }
}

/// ⭐⭐ **Duas CAIXAS encostam pela FACE e param** — a distância é a soma das meias larguras, não a
/// dos círculos à volta delas (o `41 %` de ar do report do doc 109 §5), e a aproximação é cancelada.
///
/// ⚠️ Caixas `0,5 × 2,0` (aspecto `4:1`): a lei de antes ficava `3,5 %` curta às `8` varreduras (o
/// braço de cada extremo entrava na massa efectiva); o rapier encosta-as pela face à folga dele.
#[test]
fn two_boxes_rest_face_to_face_and_stop() {
    let s = Stream::new(2)
        .with("P", Column::Vec2(vec![[-0.3, 0.0], [0.3, 0.0]]))
        .with("vel", Column::Vec2(vec![[1.0, 0.0], [-1.0, 0.0]]))
        .with("sim_t", Column::Scalar(vec![0.0, 0.0]))
        .with(
            COLLIDER_BOX_COLUMN,
            Column::Vec2(vec![[0.5, 2.0], [0.5, 2.0]]),
        );
    let out = corre(&s, 60);
    let (p, v) = (col(&out, "P"), col(&out, "vel"));
    let d = p[1][0] - p[0][0];
    assert!(
        d >= 1.0 - FOLGA && d <= 1.0 + 1e-3,
        "elas encostam pela face, a menos da folga do contacto: {p:?}"
    );
    // ⚠️ Elas nascem `0,4` SOBREPOSTAS e a vir uma para a outra: a separação é por velocidade, e a
    // que sobra ao fim de um segundo é a da correcção (`0,005` u/s medido; `0,84` com a omissão do
    // rapier — ver `ESTABILIZACOES`).
    for (i, vi) in v.iter().enumerate() {
        assert!(vi[0].abs() < 1e-2, "a caixa {i} parou em x: {vi:?}");
    }
    // E nenhuma TOMBOU: o encosto é de face, os dois pontos dele não dão binário (sobra o
    // arredondamento da ordem dos dois pontos, longe de um grau).
    if let Some(Column::Scalar(r)) = out.get("rot") {
        assert!(
            r.iter().all(|a| a.abs() < 0.5),
            "caixas de frente nao tombam: {r:?}"
        );
    }
}

/// ⭐⭐⭐ **Uma caixa cujo CENTRO passa da beira tomba** (doc 109 §6 — *«precisa destravar a rot»*),
/// e a coluna `inv_inertia` a zero (o botão `Lock Rotation` do cartão) trava-a.
///
/// ⚠️ **As duas metades num gate:** *«roda»* passa com uma peça que gira sempre, e *«trava»* passa
/// com uma que nunca gira. E travada **nem a coluna `rot` nasce** — uma cena sem rotação sai como
/// sempre saiu.
///
/// ⚠️⚠️ **A 1.ª redacção punha o centro da caixa em `0,9`, DENTRO da prancha que acaba em `1,0`, e
/// exigia que ela tombasse** — ela passava porque o contacto de UM ponto dá binário a uma caixa
/// apoiada, que é o defeito que o encosto de dois pontos veio curar (doc 111 §5.10). *O gate tinha
/// o defeito escrito dentro dele.* Hoje o centro está em `1,1`, para lá da beira, que é a condição
/// em que tombar é a resposta certa — e o irmão em [`ph2d_contact`] mede o contraste.
///
/// ⚠️⚠️ **E ele mudou de ENDEREÇO em 2026-09-16, quando a rotação passou a ser uma VELOCIDADE**
/// (doc 111 §9): a lei antiga escrevia o ângulo do tombo **no tique do contacto**, então um passo
/// só bastava e uma caixa parada em penetração pura já rodava. Hoje o contacto entrega
/// **velocidade angular**, e a posição não roda ninguém — ⇒ a fixtura tem de PRESSIONAR a caixa
/// contra a beira (uma velocidade para baixo, que é o que a gravidade faz) e marchar. *A exigência
/// é a mesma; o que mudou é onde a lei é lida.*
#[test]
fn a_box_caught_off_centre_turns_unless_the_column_locks_it() {
    let angulos = |travada: bool| -> Option<Vec<f32>> {
        let (mut p, mut v) = (
            vec![[0.0_f32, 0.0], [1.1, 0.30]],
            vec![[0.0_f32, 0.0], [0.0, -0.5]],
        );
        let (mut rot, mut spin) = (vec![0.0_f32, 0.0], vec![0.0_f32, 0.0]);
        let mut nasceu = false;
        for _ in 0..12 {
            let s = Stream::new(2)
                .with("P", Column::Vec2(p.clone()))
                .with("vel", Column::Vec2(v.clone()))
                .with("rot", Column::Scalar(rot.clone()))
                .with(crate::SPIN, Column::Scalar(spin.clone()))
                .with("sim_t", Column::Scalar(vec![0.0, 0.0]))
                // A prancha é um pino; a caixa livre pousa na ponta dela.
                .with("inv_mass", Column::Scalar(vec![0.0, 1.0]))
                .with(
                    COLLIDER_BOX_COLUMN,
                    Column::Vec2(vec![[1.0, 0.1], [0.25, 0.25]]),
                );
            let s = if travada {
                s.with(INV_INERTIA_COLUMN, Column::Scalar(vec![0.0, 0.0]))
            } else {
                s
            };
            let out = step(&s, DT, 1.0, 0.0, 0.0, 1.0);
            p = col(&out, "P");
            v = col(&out, "vel");
            // ⚠️ A fixtura semeia `rot`/`spin`, logo as colunas existem sempre à saída. O que a
            // metade TRAVADA tem de provar é que elas ficam em ZERO — a coluna nascer é inevitável
            // desde que alguém a autore, e exigir a ausência mediria a fixtura, não a lei.
            if let Some(Column::Scalar(r)) = out.get("rot") {
                rot = r.clone();
                nasceu = true;
            }
            if let Some(Column::Scalar(sp)) = out.get(crate::SPIN) {
                spin = sp.clone();
            }
        }
        nasceu.then_some(rot)
    };
    let solta = angulos(false).expect("sem travar, o passo escreve o angulo");
    assert_eq!(solta[0], 0.0, "o pino nao roda: {solta:?}");
    assert!(solta[1].abs() > 1.0, "a caixa da ponta tombou: {solta:?}");
    let presa = angulos(true).expect("a fixtura autora `rot`, logo a coluna sai sempre");
    assert!(
        presa.iter().all(|r| *r == 0.0),
        "travada pela coluna, nada roda: {presa:?}"
    );
}

/// ⭐ **Uma peça sem colisor ATRAVESSA** — só as duas com colisor se afastam.
#[test]
fn a_piece_without_a_collider_passes_through() {
    let s = par(0.1, 0.0, None).with(COLLIDER_COLUMN, Column::Scalar(vec![0.5, 0.0]));
    let out = step(&s, DT, 1.0, 0.0, 0.0, 1.0);
    assert_eq!(col(&out, "P"), vec![[-0.1, 0.0], [0.1, 0.0]]);
}

// ───────────────────────── §7 · O MATERIAL, PELA PORTA DO PRODUTO ─────────────────────────

use ph2d_nodegraph::attr::{BOUNCE_COLUMN, FRICTION_COLUMN};

/// Uma bola livre a deslizar sobre um OBSTÁCULO (`inv_mass = 0`) que não se move, com o material
/// pedido. `spin` semeia a rotação própria da bola.
fn bola_sobre_obstaculo(atrito: f32, vx: f32, spin: Option<f32>) -> Stream {
    // O obstáculo é um disco grande, a bola um disco pequeno pousado nele com folga mínima.
    let (grande, pequeno) = (2.0_f32, 0.25_f32);
    let s = Stream::new(2)
        .with(
            "P",
            Column::Vec2(vec![[0.0, -grande], [0.0, pequeno - 0.001]]),
        )
        .with("vel", Column::Vec2(vec![[0.0, 0.0], [vx, -0.2]]))
        .with("inv_mass", Column::Scalar(vec![0.0, 1.0]))
        .with("sim_t", Column::Scalar(vec![0.0, 0.0]))
        .with(COLLIDER_COLUMN, Column::Scalar(vec![grande, pequeno]))
        .with(FRICTION_COLUMN, Column::Scalar(vec![atrito, atrito]))
        .with(BOUNCE_COLUMN, Column::Scalar(vec![0.0, 0.0]));
    match spin {
        Some(g) => s.with("spin", Column::Scalar(vec![0.0, g])),
        None => s,
    }
}

fn escalar(s: &Stream, name: &str) -> Vec<f32> {
    match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// ⭐⭐⭐ **A COSTURA do material está ligada** (doc 109 §7): o passo tem de entregar ao contacto o
/// DESLIZE (onde a peça estava antes de ele a mover) e o MATERIAL — senão a lei existe na folha e
/// não acontece no produto.
///
/// ⛔⛔ **É este gate, e só este, que morre se o `sim.step` deixar de passar o `antes_do_passo` ou
/// os materiais.** Os gates da `ph2d-contact` chamam o solver directamente e ficariam todos verdes:
/// *a costura é o que se perde num merge, não a matemática.*
#[test]
fn the_step_hands_the_contact_the_slide_and_the_material() {
    let rola = step(
        &bola_sobre_obstaculo(1.0, 1.0, None),
        DT,
        1.0,
        0.0,
        0.0,
        1.0,
    );
    let gelo = step(
        &bola_sobre_obstaculo(0.0, 1.0, None),
        DT,
        1.0,
        0.0,
        0.0,
        1.0,
    );
    // ⚠️⚠️ **A LEITURA mudou de COLUNA em 2026-09-16** (doc 111 §9): o atrito entrega hoje uma
    // VELOCIDADE angular (`spin`), e o `rot` só se move quando ela é integrada no passo SEGUINTE —
    // depois de um passo só ele está em zero, e o gate lia o sítio vazio da lei nova. *Um gate cujo
    // sujeito muda de unidade muda de endereço, nunca de exigência* (a mesma lição que o
    // `delta_vel` da `ph2d-contact` já tinha pago quando o atrito saiu da posição).
    let g = escalar(&rola, crate::SPIN);
    assert!(
        g.get(1).copied().unwrap_or(0.0) < -1e-4,
        "com atrito a bola tem de RODAR (horario, a deslizar para +x): {g:?}"
    );
    // ⚠️ **A barra é de RUÍDO e não zero, e o número tem mecanismo** (doc 109 §7.1): a alavanca da
    // normal num disco é zero em aritmética exacta, e o `ponto − centro` de uma peça longe da
    // origem é uma subtracção que deixa cancelamento de `f32` — medido aqui, **`9,1e-10`**, contra
    // os `~1e-2` que o atrito produz. *Uma barra de zero mediria a aritmética.*
    let gelado = escalar(&gelo, crate::SPIN).get(1).copied().unwrap_or(0.0);
    assert!(
        gelado.abs() < 1e-6,
        "com atrito 0 a bola nao pode rodar, e rodou {gelado}"
    );
}

/// ⭐⭐ **E o passo diz ao contacto quanto o `spin` JÁ rodou neste tique** — uma bola a girar derrapa
/// contra o chão mesmo sem se deslocar, e sem este canal o atrito não a veria.
///
/// ⚠️ O controlo é a MESMA cena sem `spin`: ali o contacto não tem deslize nenhum a opor.
#[test]
fn the_step_tells_the_contact_how_much_the_spin_already_turned() {
    let girando = step(
        &bola_sobre_obstaculo(1.0, 0.0, Some(900.0)),
        DT,
        1.0,
        0.0,
        0.0,
        1.0,
    );
    let parada = step(
        &bola_sobre_obstaculo(1.0, 0.0, Some(0.0)),
        DT,
        1.0,
        0.0,
        0.0,
        1.0,
    );
    // A bola patina: o chão TRAVA-a para o lado contrário ao varrimento do ponto de baixo.
    // ⚠️ **Em VELOCIDADE** (doc 111 §7): o atrito deixou de ser uma correcção de posição e passou a
    // ser um impulso. *Um gate cujo sujeito muda de unidade muda de endereço, nunca de exigência.*
    let (a, b) = (col(&girando, "vel"), col(&parada, "vel"));
    assert!(
        a[1][0] < b[1][0] - 1e-6,
        "o atrito tinha de travar a bola que patina: {:?} contra {:?}",
        a[1],
        b[1]
    );
}

/// ⭐⭐ **O SALTO da peça chega à velocidade** (doc 109 §7): duas bolas saltitantes a aproximarem-se
/// separam-se com MAIS velocidade do que duas mortas — e com `bounce = 0` a lei é a de sempre.
#[test]
fn the_bounce_of_the_pieces_reaches_the_velocity() {
    let com = |b: f32| {
        par(0.4, 1.0, Some(0.5))
            .with(FRICTION_COLUMN, Column::Scalar(vec![0.0, 0.0]))
            .with(BOUNCE_COLUMN, Column::Scalar(vec![b, b]))
    };
    let morta = step(&com(0.0), DT, 1.0, 0.0, 0.0, 1.0);
    let viva = step(&com(0.9), DT, 1.0, 0.0, 0.0, 1.0);
    let (m, v) = (col(&morta, "vel"), col(&viva, "vel"));
    assert!(
        m[0][0].abs() < 1e-4,
        "morta: a aproximacao e' CANCELADA, e sobrou {:?}",
        m[0]
    );
    assert!(
        v[0][0] < -0.5,
        "viva: ela tem de voltar para tras, e ficou em {:?}",
        v[0]
    );
}

/// ⭐⭐⭐ **DUAS CAIXAS IGUAIS PARTILHAM O CHOQUE** — report do dono (2026-09-15): *«quando aumento
/// bounciness as colisões não são realistas. Quando uma caixa bate na outra, não parecem ter a mesma
/// massa. É como se uma fosse muito mais pesada que a outra?»*
///
/// ⚠️⚠️ **A barra NÃO é um número escolhido: é a fórmula.** Com massas iguais, o impulso clássico dá
/// `v_a' = v(1−e)/2` e `v_b' = v(1+e)/2` — sem salto partilham a meias, com salto máximo **trocam**.
/// O gate compara com a conta, não com uma tolerância inventada.
///
/// ⚠️ As caixas nascem a ENCOSTAR (`0,22` = a soma das meias), não sobrepostas: com o motor da casa
/// uma sobreposição de partida é corrigida por VELOCIDADE ao longo de alguns tiques, e essa seria
/// outra lei a sujar a conta do choque.
///
/// ## O defeito que ele apanhou, medido
///
/// A lei anterior era **por PEÇA**, sobre a velocidade ABSOLUTA de cada uma: a caixa PARADA lia
/// `vn = 0` e o guarda *«só se responde a quem se aproxima»* disparava ⇒ ela **nunca** recebia
/// velocidade. Nunca havia troca de momento, e quem era atingido era uma PAREDE:
///
/// ```text
///   bounciness |  a que bate |  a PARADA |  momento (era 1,00)
///         0,00 |      0,0000 |    0,0000 |        0,00
///         0,50 |     −0,5000 |    0,0000 |       −0,50
///         1,00 |     −1,0000 |    0,0000 |       −1,00     ← o momento INVERTIA-SE
/// ```
#[test]
fn two_equal_boxes_share_the_blow_instead_of_one_being_a_wall() {
    use ph2d_nodegraph::attr::BOUNCE_COLUMN;
    /// `f32` sobre uma conta de duas divisões — a folga é de arredondamento, não de lei.
    const EPS: f32 = 2e-3;
    for (forma, k) in [("discos", 1.0_f32), ("caixas", SALTO_DA_FACE)] {
        for e in [0.0_f32, 0.25, 0.5, 0.75, 1.0] {
            let v = col(&corre(&choque(forma, e, 1.0), 3), "vel");
            let (bate, parada) = ((1.0 - k * e) * 0.5, (1.0 + k * e) * 0.5);
            assert!(
                (v[0][0] - bate).abs() < EPS,
                "{forma}, salto {e}: a que bate fica em {} e a conta da' {bate}",
                v[0][0]
            );
            assert!(
                (v[1][0] - parada).abs() < EPS,
                "{forma}, salto {e}: a PARADA fica em {} e a conta da' {parada} — se ler 0, ela virou parede",
                v[1][0]
            );
            // ⭐ E o MOMENTO conserva-se: é ele que distingue uma troca de uma parede.
            assert!(
                (v[0][0] + v[1][0] - 1.0).abs() < EPS,
                "{forma}: o momento tem de ficar em 1,0 e ficou em {}",
                v[0][0] + v[1][0]
            );
        }
    }
}

/// ⭐ **O salto de uma CAIXA que bate de FACE é `11/25 ÷ 1/2 = 0,88` do pedido** — a lei da família
/// do Box2D (o rapier aplica o salto no fim do passo, UMA passagem de Gauss–Seidel pelos DOIS pontos
/// da face), e não um defeito: para quadrados iguais o sistema dos dois pontos é
/// `K = [[5, −1], [−1, 5]]/m`, a passagem única dá `λ = (1/5 + 6/25)·m·v = 11/25·m·v` contra o
/// exacto `1/2·m·v`. Um disco (um ponto) dá a conta exacta. Medido: `e = 1` devolve `0,880`,
/// `e = 0,5` `0,440`, com qualquer número de iterações e estabilizações (doc 121 §9.20).
const SALTO_DA_FACE: f32 = 0.88;

/// Duas peças IGUAIS a ENCOSTAR (a soma das meias), a da esquerda a `1 u/s`; `parada_w` é o
/// `inv_mass` da da direita (`0` = um pino). ⚠️ Encostadas e não sobrepostas: com o motor da casa uma
/// sobreposição de partida é corrigida por VELOCIDADE ao longo de alguns tiques, e essa seria outra
/// lei a sujar a conta do choque.
fn choque(forma: &str, e: f32, parada_w: f32) -> Stream {
    use ph2d_nodegraph::attr::BOUNCE_COLUMN;
    let s = Stream::new(2)
        .with("P", Column::Vec2(vec![[-0.22, 0.0], [0.0, 0.0]]))
        .with("vel", Column::Vec2(vec![[1.0, 0.0], [0.0, 0.0]]))
        .with("sim_t", Column::Scalar(vec![0.0, 0.0]))
        .with("inv_mass", Column::Scalar(vec![1.0, parada_w]))
        .with(BOUNCE_COLUMN, Column::Scalar(vec![e, e]));
    if forma == "discos" {
        s.with(COLLIDER_COLUMN, Column::Scalar(vec![0.11, 0.11]))
    } else {
        s.with(
            COLLIDER_BOX_COLUMN,
            Column::Vec2(vec![[0.11, 0.11], [0.11, 0.11]]),
        )
    }
}

/// ⭐⭐ **E um OBSTÁCULO continua a ser uma parede** — a metade que o gate acima não mede, e sem ela
/// «partilhar o choque» podia ter sido escrito de forma a amolecer um pino.
///
/// Com `inv_mass = 0` a peça é imóvel por declaração, e a que bate devolve o salto: `v' = −e·v`
/// (`−0,88·e·v` numa caixa de face — ver [`SALTO_DA_FACE`]).
#[test]
fn an_obstacle_is_still_a_wall_and_returns_the_bounce() {
    for (forma, k) in [("discos", 1.0_f32), ("caixas", SALTO_DA_FACE)] {
        for e in [0.0_f32, 0.5, 1.0] {
            let v = col(&corre(&choque(forma, e, 0.0), 3), "vel");
            assert!(
                (v[0][0] + k * e).abs() < 2e-3,
                "{forma}: contra um obstaculo, com salto {e}, ela devolve {} e a conta da' {}",
                v[0][0],
                -k * e
            );
            assert!(v[1][0].abs() < 1e-6, "o obstaculo nao se mexe: {:?}", v[1]);
        }
    }
}

/// Um disco de raio `R` já a ROLAR a `1 u/s` sobre um obstáculo fixo, com `Rolling = rolar`, pela
/// porta do produto — os segundos até parar (`|v| < 0,01`), ou `None` em 30 s.
///
/// ⚠️ **As condições são as da tabela do `ROLLING_MAX`** (raio `0,2`, gravidade `4`, `μ = 1`), que
/// a taça (`sim.collide`) mediu: é a mesma pergunta às duas metades do app.
///
/// ⛔⛔ **A prancha tem de ser mais comprida que o caminho, e a 1.ª não era.** Com meia-largura `4`
/// o disco a `Rolling = 0,02` lia *«não pára em 30 s»* contra os `18,57` da taça — e a série no
/// tempo mostrou-o a desacelerar **exactamente** ao ritmo da taça (`0,053 u/s²`) até aos `5 s`, e
/// depois a CAIR PELA PONTA (`y = −1098` aos 28 s). A taça mede contra um plano INFINITO; um
/// rolamento fraco anda `~9` unidades antes de parar. *Uma fixtura mais curta que o fenómeno lê o
/// fim dela como um defeito do produto.*
const PRANCHA: f32 = 40.0;

pub(super) fn disco_ate_parar(rolar: f32, sub: u32) -> Option<f32> {
    use crate::SPIN;
    use ph2d_nodegraph::attr::{COLLIDER_COLUMN, FRICTION_COLUMN, ROLLING_COLUMN};
    const G: f32 = 4.0;
    const R: f32 = 0.2;
    #[expect(
        clippy::cast_precision_loss,
        reason = "uma contagem de sub-passos pequena"
    )]
    let dt = DT / sub as f32;
    let mut p = vec![[0.0_f32, -0.5], [0.0, R]];
    let mut v = vec![[0.0_f32, 0.0], [1.0, 0.0]];
    // A rolar para a direita: ponto de contacto parado, `ω = −v/R`.
    let mut spin = vec![0.0_f32, -(1.0 / R).to_degrees()];
    for k in 0..(60 * 30 * sub) {
        let s = Stream::new(2)
            .with("P", Column::Vec2(p.clone()))
            .with("vel", Column::Vec2(v.clone()))
            .with(SPIN, Column::Scalar(spin.clone()))
            .with("accel", Column::Vec2(vec![[0.0, -G], [0.0, -G]]))
            .with("sim_t", Column::Scalar(vec![0.0, 0.0]))
            .with("inv_mass", Column::Scalar(vec![0.0, 1.0]))
            .with(FRICTION_COLUMN, Column::Scalar(vec![1.0, 1.0]))
            .with(ROLLING_COLUMN, Column::Scalar(vec![0.0, rolar]))
            .with(COLLIDER_COLUMN, Column::Scalar(vec![0.0, R]))
            .with(
                COLLIDER_BOX_COLUMN,
                Column::Vec2(vec![[PRANCHA, 0.5], [0.0, 0.0]]),
            );
        let out = step(&s, dt, 1.0, 0.0, 0.0, 1.0);
        p = col(&out, "P");
        v = col(&out, "vel");
        if let Some(Column::Scalar(sp)) = out.get(SPIN) {
            spin = sp.clone();
        }
        if v[1][0].abs() < 0.01 {
            #[expect(clippy::cast_precision_loss, reason = "uma contagem de tiques pequena")]
            let t = k as f32 * dt;
            return Some(t);
        }
    }
    None
}

/// ⭐⭐⭐ **Uma peça a ROLAR sobre outra PÁRA com o `Rolling` do cartão — e pára no tempo da TAÇA.**
///
/// ⛔⛔ **Até 2026-09-16 este controlo era MORTO no contacto peça×peça**, e o contrato da coluna
/// dizia-o por escrito: a rotação era posicional e não havia velocidade angular a travar. O doc
/// 111 §9 deu-lha; o §10 ligou-lhe o rolamento pela MESMA porta da taça.
///
/// ⭐ **A barra é a TAÇA**, que o dono já smokou (tabela do `ROLLING_MAX`): é a mesma pergunta às
/// duas metades do app, e elas têm de dar a mesma resposta. Medido nesta porta, na faixa em que
/// quem trava é o rolamento:
///
/// ```text
///   rolamento | 1 passo | 8 sub-passos | a taça
///        0,02 |   18,58 |        18,56 |  18,57
///        0,05 |    7,43 |         7,43 |   7,43
///        0,10 |    3,72 |         3,71 |   3,72
/// ```
///
/// ⇒ o desvio máximo é `0,27 %`, e a folga é `2 %` (`7×`); o lado do defeito é **infinito** (não
/// pára). ⚠️ **Acima de `0,25` as duas metades separam-se por construção**, e o gate não o mede: ali
/// quem trava é o Coulomb (teórico `0,25 s`), e a taça lê `0,33` porque o contacto dela não
/// acontece em todos os tiques, enquanto este lê `0,23–0,25`. ⚠️ E as DUAS contagens de sub-passos,
/// porque a lei tem de ser linear no passo — é o que o doc 111 §7 exigiu ao atrito.
#[test]
fn a_piece_rolling_on_a_piece_stops_as_it_does_on_the_bowl() {
    /// Folga relativa — `7×` o maior desvio medido (ver acima).
    const FOLGA: f32 = 0.02;
    const TACA: [(f32, f32); 3] = [(0.02, 18.57), (0.05, 7.43), (0.10, 3.72)];
    assert_eq!(
        disco_ate_parar(0.0, 1),
        None,
        "sem rolamento a bola rola para sempre -- e' a lei de antes da coluna"
    );
    for (rolar, taca) in TACA {
        for sub in [1, 8] {
            let t = disco_ate_parar(rolar, sub).unwrap_or_else(|| {
                panic!(
                    "rolamento {rolar} ({sub} sub-passos): a bola nunca parou -- o Rolling morreu"
                )
            });
            assert!(
                (t - taca).abs() <= FOLGA * taca,
                "rolamento {rolar} ({sub} sub-passos): parou em {t:.2} s, a taca para em {taca} s"
            );
        }
    }
}
