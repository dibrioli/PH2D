//! Gates da cena `=29` — e a SONDA que produziu os números da mensagem de anúncio.
//!
//! A regra do plano 89: *toda wave ganha cena com números MEDIDOS, e a sonda headless roda ANTES
//! de a mensagem ser escrita*.

use super::*;
use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

/// A queda por unidade de `x` da rampa — `tan(18°)`, **do lado do gate e não da cena**.
///
/// Ele é o ORÁCULO de *onde a rampa está*, e um oráculo derivado da normal que o produto computa
/// concordaria com ela por construção: seria um espelho, não uma medida. Aqui é um número de
/// tabela trigonométrica, que o colisor nunca vê.
const RAMP_SLOPE: f32 = 0.3249;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    reg
}

/// Cozinha a cena até assentar e devolve a posição de cada elemento.
fn settled(ramp_deg: f32, secs: f64) -> Vec<[f32; 2]> {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks =
        build_gpu_ramp_demo_document(&mut doc, &reg, ramp_deg).expect("a cena é bem tipada");
    let mut cook = Cook::new();
    let last = (secs * 60.0) as u64;
    let mut out = Vec::new();
    for k in 0..=last {
        let t = k as f64 / 60.0;
        let s = cook.cook(&doc.graph, &reg, sinks[0], t).expect("cozinha")[0]
            .as_stream()
            .clone();
        if k == last {
            out = match s.get("P") {
                Some(Column::Vec2(v)) => v.clone(),
                _ => vec![],
            };
        }
        cook.advance_tick(&doc.graph, &reg, t).expect("avança");
    }
    out
}

fn centroid_x(ps: &[[f32; 2]]) -> f32 {
    ps.iter().map(|p| p[0]).sum::<f32>() / ps.len() as f32
}

/// **A cena é bem tipada e a chuva inteira está lá** — o mínimo que separa uma cena de um
/// documento que o `validate` recusa na abertura.
#[test]
fn the_scene_builds_with_the_whole_shower() {
    let ps = settled(RAMP_DEG, 4.0);
    assert_eq!(ps.len(), (ROWS * COLS) as usize, "27 discos");
}

/// **A RAMPA TRANSPORTA, e um chão não.**
///
/// É a capacidade inteira da wave numa comparação: a MESMA cena, o mesmo `sim.collide`, o mesmo
/// tudo — e um único param diferente. Com o plano horizontal os discos pousam onde caíram; com
/// ele inclinado eles descem.
///
/// FALSIFICADO por um contato que empurra sempre para CIMA: a componente tangencial que a
/// gravidade injeta seria cancelada a cada passo e a rampa se comportaria como um chão com uma
/// inclinação desenhada em cima.
#[test]
fn the_ramp_carries_the_shower_downhill_and_a_floor_does_not() {
    let ramp = centroid_x(&settled(RAMP_DEG, 4.0));
    let floor = centroid_x(&settled(0.0, 4.0));
    assert!(
        ramp - floor > 1.5,
        "a rampa tem de levar a chuva para a direita: centroide {ramp} contra {floor} no chão"
    );
}

/// **E a PAREDE a para** — o mesmo nó, um quarto de volta, e a única coisa entre a chuva e o
/// infinito à direita.
///
/// FALSIFICADO por um `offset` que não alcança a 90°: os discos passariam direto, e a razão de a
/// forma de Hesse ter sido escolhida em vez do pivô deixaria de estar demonstrada.
#[test]
fn the_wall_stops_what_the_ramp_delivers() {
    let ps = settled(RAMP_DEG, 4.0);
    // O disco colide pelo SPRITE, então o centro para meio disco antes da parede.
    let limit = WALL_X - DISC * 0.5 + 1e-3;
    for p in &ps {
        assert!(p[0] <= limit, "disco em x = {} passou da parede", p[0]);
    }
    let max = ps.iter().fold(f32::MIN, |m, p| m.max(p[0]));
    assert!(
        max > WALL_X - DISC,
        "…e alguém tem de CHEGAR nela, senão a parede não está sendo testada; maior x = {max}"
    );
}

/// **Quem não está na parede está SOBRE a rampa** — a linha `y = -RAMP_SLOPE * x`, a menos de
/// meio disco de raio, que é onde um sprite que pousa fica.
///
/// Este é o gate que a razão de existir da wave exige: sem ele, "os discos foram para a direita"
/// seria satisfeito por uma cena em que eles simplesmente caem através de tudo.
#[test]
fn the_shower_rests_on_the_ramp_it_slid_down() {
    let ps = settled(RAMP_DEG, 4.0);
    // Os que ainda não alcançaram a parede — os empilhados na quina sobem uns sobre os outros.
    let free: Vec<_> = ps.iter().filter(|p| p[0] < WALL_X - DISC * 2.0).collect();
    assert!(free.len() >= 4, "a fixture precisa conter discos livres");
    for p in free {
        let on_ramp = -RAMP_SLOPE * p[0];
        assert!(
            (p[1] - on_ramp).abs() < DISC,
            "disco em {p:?} deveria estar sobre a rampa (y = {on_ramp})"
        );
    }
}

/// **A SONDA** — imprime o que a cena faz, de onde saem os números do anúncio e do doc.
///
/// `cargo test -p ph2d-host-desktop --lib ramp_demo::tests::probe -- --ignored --nocapture`
#[test]
#[ignore]
fn probe_ramp_chute() {
    for (name, deg) in [("chao (angulo 0)", 0.0), ("rampa", RAMP_DEG)] {
        let ps = settled(deg, 4.0);
        let min = ps.iter().fold(f32::MAX, |m, p| m.min(p[0]));
        let max = ps.iter().fold(f32::MIN, |m, p| m.max(p[0]));
        eprintln!(
            "{name}: centroide x = {:.4}  faixa [{min:.4}, {max:.4}]  altura media {:.4}",
            centroid_x(&ps),
            ps.iter().map(|p| p[1]).sum::<f32>() / ps.len() as f32,
        );
    }
}
