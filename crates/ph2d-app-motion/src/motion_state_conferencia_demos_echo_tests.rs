//! Gates da cena `=88` — **o eco que vê o futuro** (doc 89, folha 07).
//!
//! ⚠️ **O oráculo é a DIREÇÃO da cauda contra o movimento, e não «a figura
//! mudou».** As três linhas desenham o mesmo número de peças, no mesmo caminho,
//! com a mesma decadência: contagem, caixa envolvente e excursão dão verde nas
//! três e não dizem nada. O que as separa é de que lado da cabeça o eco pousa —
//! e isso mede-se projectando o eco VIZINHO sobre a velocidade do elemento (o
//! mais velho está longe demais: numa figura que vira, ele não prediz nada).

use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registram");
    reg
}

fn scene() -> (MotionDoc, NodeRegistry, Vec<NodeId>) {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_echo_demo_document(&mut doc, &reg).expect("a cena monta");
    doc.graph.validate(&reg).expect("bem-tipada");
    (doc, reg, sinks)
}

const DT: f64 = 1.0 / 60.0;

/// Coze `ticks` quadros com os LEQUES montados — a mesma costura que o shell faz
/// uma vez por quadro. Devolve as posições do último quadro.
///
/// ⚠️ **Sem o `set_time_fans` isto mediria o ring nas três linhas**, com a suíte
/// verde sobre uma cena que não prova nada: o `eval` cai no `Remembered` quando o
/// leque está ausente, de propósito.
fn run(doc: &MotionDoc, reg: &NodeRegistry, sink: NodeId, ticks: u32) -> Vec<[f32; 2]> {
    let fans = ph2d_node_motion_trail::time_fans(&doc.graph, reg, DT);
    let mut cook = Cook::new();
    let mut last = Vec::new();
    for k in 0..ticks {
        let t = f64::from(k) * DT;
        let out = cook
            .cook_scoped_fanned(&doc.graph, reg, sink, t, &Default::default(), &fans)
            .expect("cozinha");
        if let Some(Column::Vec2(p)) = out[0].as_stream().get("P") {
            last = p.clone();
        }
        cook.advance_tick_fanned(&doc.graph, reg, t, &Default::default(), &fans)
            .expect("avanca o quadro");
    }
    last
}

fn row_of(echo: Echo) -> usize {
    ROWS_TABLE
        .iter()
        .position(|r| r.echo == echo)
        .expect("existe")
}

/// A cabeça é a ÚLTIMA linha — ela pinta sobre os próprios ecos.
fn head(p: &[[f32; 2]]) -> [f32; 2] {
    *p.last().expect("a cabeca existe")
}
/// O eco MAIS PRÓXIMO da cabeça — a penúltima linha (a última é a cabeça).
///
/// ⚠️ **É este que a régua da direção usa, e não o mais velho (`p[0]`, que por
/// isso deixou de ter chamador).** A projecção sobre
/// a velocidade só prediz onde uma coisa está enquanto a direção não mudou, e o
/// eco mais velho está a `(L−1)·s` tiques daqui — meio quinto de volta desta
/// figura. Medido: com o mais velho a projecção sai **−0,31** numa cauda que vai
/// à frente, porque o caminho já virou. *Uma régua de direção mede um passo, não
/// uma viagem.*
fn nearest(p: &[[f32; 2]]) -> [f32; 2] {
    p[p.len() - 2]
}

/// O quanto a cabeça anda num tique — a régua com que toda distância desta cena
/// se compara.
fn travel_per_tick(doc: &MotionDoc, reg: &NodeRegistry, sink: NodeId) -> f32 {
    let a = head(&run(doc, reg, sink, 120));
    let b = head(&run(doc, reg, sink, 121));
    (b[0] - a[0]).hypot(b[1] - a[1])
}

/// **A CENA CONSTRÓI AS TRÊS LINHAS**, e cada uma desenha a cauda inteira.
#[test]
fn the_echo_scene_builds_every_row() {
    let (doc, reg, sinks) = scene();
    assert_eq!(sinks.len(), ROWS_TABLE.len(), "uma sink por linha");
    for (k, &s) in sinks.iter().enumerate() {
        let p = run(&doc, &reg, s, 90);
        assert_eq!(
            p.len(),
            LENGTH as usize,
            "linha {k}: a cauda tem de estar cheia"
        );
    }
}

/// ⭐ **A REDUÇÃO, medida pelo produto:** o rastro RE-COZIDO desenha a mesma cauda
/// que o ring — mesma cabeça, mesmos ecos, no mesmo sítio.
///
/// ⚠️ Não ao bit, e a razão está escrita no `Decay::at_age`: o ring chega ao
/// desbote por `n` multiplicações e a re-cozedura de uma vez. O que este gate
/// afirma é a GEOMETRIA, que é o que o artista vê.
#[test]
fn the_resampled_tail_lands_where_the_remembered_one_does() {
    let (doc, reg, sinks) = scene();
    let a = run(&doc, &reg, sinks[row_of(Echo::Remembered)], 120);
    let b = run(&doc, &reg, sinks[row_of(Echo::Resampled)], 120);
    assert_eq!(a.len(), b.len(), "as duas caudas tem o mesmo comprimento");
    let dy = row_y(row_of(Echo::Resampled)) - row_y(row_of(Echo::Remembered));
    let worst = a
        .iter()
        .zip(&b)
        .map(|(p, q)| (q[0] - p[0]).abs().max((q[1] - dy - p[1]).abs()))
        .fold(0.0f32, f32::max);
    let tick = travel_per_tick(&doc, &reg, sinks[row_of(Echo::Remembered)]);
    assert!(
        tick > 0.0,
        "CONTROLE: se o elemento nao anda, a barra e' zero e o gate e' vacuo"
    );

    // ⭐ **A CABEÇA é EXACTA**, e é a metade da afirmação que não tem desculpa: as
    // duas linhas leem a entrada viva no mesmo instante, então a peça viva tem de
    // pousar no mesmo pixel.
    let (ha, hb) = (head(&a), head(&b));
    let head_gap = (hb[0] - ha[0]).abs().max((hb[1] - dy - ha[1]).abs());
    assert!(head_gap < 1e-4, "a cabeca divergiu {head_gap:.6}");

    // ⚠️ **E os ECOS ficam dentro de um CICLO DE PROMOÇÃO, que é mecanismo e não
    // folga.** O ring promove a cabeça a fantasma a cada `spacing` tiques, então
    // as idades dos ecos dele passeiam por `1..=spacing` conforme a fase do
    // quadro: a cauda lembrada carrega até `spacing − 1` tiques de erro de fase, o
    // tempo todo. A re-cozida não tem fase nenhuma — ela lê `t − k·s` exacto.
    // Medido nesta cena: a pior diferença é **1,9×** o passo de um tique, dentro
    // de um ciclo de 4.
    //
    // ⛔ Uma barra apertada aqui não seria mais rigorosa: ela mediria o erro de
    // fase do modo ANTIGO, e reprovaria sempre que o quadro calhasse noutro ponto
    // do ciclo. *A cauda re-cozida é a mais certa das duas.*
    let cycle = f32::from(SPACING as u16);
    assert!(
        worst < tick * cycle,
        "o rastro re-cozido divergiu do lembrado em {worst:.5}, contra um ciclo de {cycle} passos de {tick:.5}"
    );
}

/// ⭐⭐ **O ECO VAI À FRENTE.** A régua é a projecção do vector cabeça→eco sobre a
/// VELOCIDADE do elemento: negativa quando o eco fica para trás (as duas
/// primeiras linhas) e positiva quando ele vai adiante (a terceira).
///
/// ⚠️ **Uma régua de posição não serviria** — num caminho que se cruza, o eco
/// mais velho pode calhar de qualquer lado da tela. O que define *à frente* é o
/// sentido da marcha, e ele mede-se contra a velocidade.
#[test]
fn the_forward_echo_leads_the_element_and_the_others_trail_it() {
    let (doc, reg, sinks) = scene();
    // A velocidade da cabeça: dois quadros consecutivos da MESMA linha.
    let vel_of = |k: usize| {
        let a = head(&run(&doc, &reg, sinks[k], 120));
        let b = head(&run(&doc, &reg, sinks[k], 121));
        [b[0] - a[0], b[1] - a[1]]
    };
    let step = f32::from(SPACING as u16);
    for (echo, want_ahead) in [
        (Echo::Remembered, false),
        (Echo::Resampled, false),
        (Echo::Forward, true),
    ] {
        let k = row_of(echo);
        let p = run(&doc, &reg, sinks[k], 120);
        let (h, o) = (head(&p), nearest(&p));
        let v = vel_of(k);
        let speed = v[0].hypot(v[1]).max(1e-9);
        let along = ((o[0] - h[0]) * v[0] + (o[1] - h[1]) * v[1]) / speed;
        assert!(
            (along > 0.0) == want_ahead,
            "linha {k}: o eco vizinho projecta {along:.4} sobre a marcha (esperava {})",
            if want_ahead { "a FRENTE" } else { "atras" }
        );
        // E não marginalmente: o eco vizinho está a `spacing` tiques de distância,
        // então a projecção tem de ser da ordem de `spacing` passos.
        assert!(
            along.abs() > speed * step * 0.5,
            "linha {k}: {along:.4} e' ruido ao lado de {step} passos de {speed:.4}"
        );
    }
}

/// ⚠️ **O rastro re-cozido é EXACTO sob scrub** — nada nele é estado, então
/// chegar ao quadro 90 saltando é o mesmo que chegar lá andando. O de LEMBRAR
/// não tem essa propriedade (ele depende do ring), e é isso que o controle diz.
#[test]
fn the_resampled_tail_is_the_same_whether_you_walk_or_jump_to_the_frame() {
    let (doc, reg, sinks) = scene();
    let k = row_of(Echo::Resampled);
    let fans = ph2d_node_motion_trail::time_fans(&doc.graph, &reg, DT);
    let walked = run(&doc, &reg, sinks[k], 91);
    let jumped = {
        let mut cook = Cook::new();
        let t = 90.0 * DT;
        let out = cook
            .cook_scoped_fanned(&doc.graph, &reg, sinks[k], t, &Default::default(), &fans)
            .expect("cozinha");
        match out[0].as_stream().get("P") {
            Some(Column::Vec2(p)) => p.clone(),
            _ => panic!("P"),
        }
    };
    assert_eq!(walked.len(), jumped.len(), "o salto deu outra cauda");
    let worst = walked
        .iter()
        .zip(&jumped)
        .map(|(a, b)| (a[0] - b[0]).abs().max((a[1] - b[1]).abs()))
        .fold(0.0f32, f32::max);
    assert!(worst < 1e-4, "o salto divergiu da caminhada em {worst:.6}");
}

/// As fichas do canvas: uma por linha, na altura da linha.
#[test]
fn every_row_carries_its_caption() {
    let caps = captions();
    assert_eq!(caps.len(), ROWS_TABLE.len(), "uma ficha por linha");
    for (k, c) in caps.iter().enumerate() {
        assert!(
            c.world[1] > row_y(k),
            "a ficha {k} tem de ficar ACIMA da sua linha"
        );
    }
}
