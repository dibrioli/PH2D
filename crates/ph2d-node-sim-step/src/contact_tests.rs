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

/// ⭐ **Duas peças sobrepostas saem à soma dos raios** (`size` ausente ⇒ escala 1).
#[test]
fn overlapping_pieces_are_pushed_apart_to_their_radii() {
    let s = step(&par(0.1, 0.0, Some(0.5)), DT, 1.0, 0.0, 0.0, 1.0);
    let p = col(&s, "P");
    assert!(((p[1][0] - p[0][0]) - 1.0).abs() < 1e-4, "{p:?}");
}

/// ⭐⭐ **O contacto NUNCA acrescenta velocidade** — duas peças que nascem sobrepostas, paradas,
/// separam-se em posição e continuam paradas. Somar `Δp/dt` inteiro faria delas uma explosão.
#[test]
fn the_contact_never_adds_speed() {
    let s = step(&par(0.1, 0.0, Some(0.5)), DT, 1.0, 0.0, 0.0, 1.0);
    assert_eq!(col(&s, "vel"), vec![[0.0, 0.0], [0.0, 0.0]]);
}

/// ⭐⭐ **A aproximação é CANCELADA, não reflectida** — duas peças que se encostam a `±2 u/s`
/// ficam encostadas e param; nenhuma ressalta para trás.
#[test]
fn an_approach_is_cancelled_not_reflected() {
    // Encostadas (distância 1 = a soma dos raios) e a vir uma para a outra.
    let s = step(&par(0.5, 2.0, Some(0.5)), DT, 1.0, 0.0, 0.0, 1.0);
    let (p, v) = (col(&s, "P"), col(&s, "vel"));
    assert!(
        ((p[1][0] - p[0][0]) - 1.0).abs() < 1e-4,
        "encostadas: {p:?}"
    );
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
/// ⚠️⚠️ **A tolerância da distância é o RESÍDUO DE CONVERGÊNCIA do encosto de dois pontos, medido**
/// (doc 111 §5.11). Estas caixas são `0,5 × 2,0` — aspecto `4:1` —, e o braço de cada extremo do
/// trecho é `2,0`: com o braço a entrar na massa efectiva de cada ponto (`k = w + invI·b²`), cada
/// varredura corrige menos e o produto, às `8` varreduras, fica **`3,5 %` curto**:
///
/// ```text
///   varreduras | distancia | residuo
///            8 |  0,964626 | 3,5e-2
///           16 |  0,996872 | 3,1e-3
///           32 |  0,999976 | 2,4e-5
///           64 |  1,000000 | 0,0      ← exacto
/// ```
///
/// ⛔ **O resíduo ENCOLHE com as varreduras ⇒ é convergência, não viés** — e por isso o gate mede
/// as DUAS pontas em vez de afrouxar a barra até a de `8` passar. ⭐ A `=114` usa quadrados (aspecto
/// `1:1`) e fica `0,15 %` curta às `8`, exacta às `32`: *o preço é do ASPECTO da caixa, e uma cena
/// que empilhe formas esguias paga-o em sub-passos.*
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
    let out = step(&s, DT, 1.0, 0.0, 0.0, 1.0);
    let (p, v) = (col(&out, "P"), col(&out, "vel"));
    // Às `8` varreduras do produto: o resíduo MEDIDO acima, e nunca uma sobreposição maior.
    let d = p[1][0] - p[0][0];
    assert!(
        (0.96..=1.0).contains(&d),
        "elas encostam pela face, a menos do residuo de convergencia: {p:?}"
    );
    for (i, vi) in v.iter().enumerate() {
        assert!(vi[0].abs() < 1e-3, "a caixa {i} parou em x: {vi:?}");
    }
    // ⭐ E com varreduras a chegar, a distância é EXACTA — a prova de que não há viés.
    //
    // ⚠️⚠️ **A BARRA MUDOU DE DONO em 2026-09-18 (doc 115 §20), e a mudança é o PREÇO do
    // `REPOUSO_VISIVEL`:** o `separate` pára quando mais nenhuma peça se mexe de forma visível, e
    // deixa uma cauda da ordem do limiar. Medido nesta fixtura: `1,1e-4` sobre uma caixa de `1,0`
    // de largura — **`0,011 %` dela**, ou `0,02 px` num desenho de 200 px.
    //
    // ⛔ **A barra é ABSOLUTA e medida, nunca derivada da constante:** uma barra que escalasse com
    // o `REPOUSO_VISIVEL` ficaria verde se alguém o subisse `1000×` — foi uma mutação sobrevivente
    // que ensinou isso, no gate irmão da `ph2d-contact`.
    let convergida = {
        let mut p = vec![[-0.3_f32, 0.0], [0.3, 0.0]];
        let c = vec![
            Some(ph2d_contact::Colisor::caixa([0.5, 2.0], [1.0, 0.0])),
            Some(ph2d_contact::Colisor::caixa([0.5, 2.0], [1.0, 0.0])),
        ];
        let w = [1.0_f32, 1.0];
        let inv: Vec<f32> = c
            .iter()
            .zip(w)
            .map(|(c, w)| c.map_or(0.0, |c| c.inv_inercia(w)))
            .collect();
        let mut g = vec![0.0; 2];
        ph2d_contact::separate(
            &mut p,
            &mut ph2d_contact::Saida { giro: &mut g },
            &ph2d_contact::Pecas::novas(&c, &w, &inv),
            64,
        );
        p[1][0] - p[0][0]
    };
    assert!(
        (convergida - 1.0).abs() < 2e-4,
        "a 64 varreduras o residuo tem de caber no repouso visivel: {convergida}"
    );
    // ⭐ **E o CONTROLO do viés, no caminho que NUNCA pára cedo** — sem esta metade um viés
    // sistemático esconder-se-ia atrás da barra nova.
    let sem_atalho = {
        let mut p = vec![[-0.3_f32, 0.0], [0.3, 0.0]];
        let c = vec![
            Some(ph2d_contact::Colisor::caixa([0.5, 2.0], [1.0, 0.0])),
            Some(ph2d_contact::Colisor::caixa([0.5, 2.0], [1.0, 0.0])),
        ];
        let w = [1.0_f32, 1.0];
        let inv: Vec<f32> = c
            .iter()
            .zip(w)
            .map(|(c, w)| c.map_or(0.0, |c| c.inv_inercia(w)))
            .collect();
        let mut g = vec![0.0; 2];
        ph2d_contact::separate_all_pairs(
            &mut p,
            &mut ph2d_contact::Saida { giro: &mut g },
            &ph2d_contact::Pecas::novas(&c, &w, &inv),
            256,
        );
        p[1][0] - p[0][0]
    };
    assert!(
        (sem_atalho - 1.0).abs() < 1e-5,
        "sem atalho nenhum a distancia e' EXACTA: {sem_atalho}"
    );
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
    const EPS: f32 = 1e-4;
    for e in [0.0_f32, 0.25, 0.5, 0.75, 1.0] {
        let s = Stream::new(2)
            .with("P", Column::Vec2(vec![[-0.20, 0.0], [0.0, 0.0]]))
            .with("vel", Column::Vec2(vec![[1.0, 0.0], [0.0, 0.0]]))
            .with("sim_t", Column::Scalar(vec![0.0, 0.0]))
            .with(BOUNCE_COLUMN, Column::Scalar(vec![e, e]))
            .with(
                COLLIDER_BOX_COLUMN,
                Column::Vec2(vec![[0.11, 0.11], [0.11, 0.11]]),
            );
        let v = col(&step(&s, DT, 1.0, 0.0, 0.0, 1.0), "vel");
        let (bate, parada) = ((1.0 - e) * 0.5, (1.0 + e) * 0.5);
        assert!(
            (v[0][0] - bate).abs() < EPS,
            "com salto {e}, a que bate fica em {} e a conta da' {bate}",
            v[0][0]
        );
        assert!(
            (v[1][0] - parada).abs() < EPS,
            "com salto {e}, a PARADA fica em {} e a conta da' {parada} — se ler 0, ela virou parede",
            v[1][0]
        );
        // ⭐ E o MOMENTO conserva-se: é ele que distingue uma troca de uma parede.
        assert!(
            (v[0][0] + v[1][0] - 1.0).abs() < EPS,
            "o momento tem de ficar em 1,0 e ficou em {}",
            v[0][0] + v[1][0]
        );
    }
}

/// ⭐⭐ **E um OBSTÁCULO continua a ser uma parede** — a metade que o gate acima não mede, e sem ela
/// «partilhar o choque» podia ter sido escrito de forma a amolecer um pino.
///
/// Com `inv_mass = 0` a peça é imóvel por declaração, e a que bate devolve **exactamente** o salto:
/// `v' = −e·v`. ⚠️ É o mesmo `j = (1+e)·vrel/(w_a + w_b)` com `w_b = 0` — *uma lei, os dois casos*.
#[test]
fn an_obstacle_is_still_a_wall_and_returns_the_bounce() {
    use ph2d_nodegraph::attr::BOUNCE_COLUMN;
    for e in [0.0_f32, 0.5, 1.0] {
        let s = Stream::new(2)
            .with("P", Column::Vec2(vec![[-0.20, 0.0], [0.0, 0.0]]))
            .with("vel", Column::Vec2(vec![[1.0, 0.0], [0.0, 0.0]]))
            .with("sim_t", Column::Scalar(vec![0.0, 0.0]))
            .with("inv_mass", Column::Scalar(vec![1.0, 0.0]))
            .with(BOUNCE_COLUMN, Column::Scalar(vec![e, e]))
            .with(
                COLLIDER_BOX_COLUMN,
                Column::Vec2(vec![[0.11, 0.11], [0.11, 0.11]]),
            );
        let v = col(&step(&s, DT, 1.0, 0.0, 0.0, 1.0), "vel");
        assert!(
            (v[0][0] + e).abs() < 1e-4,
            "contra um obstaculo, com salto {e}, ela devolve {} e a conta da' {}",
            v[0][0],
            -e
        );
        assert!(v[1][0].abs() < 1e-6, "o obstaculo nao se mexe: {:?}", v[1]);
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
