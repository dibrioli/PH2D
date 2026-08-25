//! Gates da cena `=95` — o que uma força não sabia dizer (folha 02).

use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

const TICKS: usize = 150;
const DT: f64 = 1.0 / 60.0;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registram");
    reg
}

fn scene() -> (MotionDoc, NodeRegistry, Vec<NodeId>) {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_forces_demo_document(&mut doc, &reg).expect("a cena monta");
    doc.graph.validate(&reg).expect("bem-tipada");
    (doc, reg, sinks)
}

/// Corre a cena e devolve a última pose de cada sink pedido.
fn settle(doc: &MotionDoc, reg: &NodeRegistry, sinks: &[NodeId]) -> Vec<Vec<[f32; 2]>> {
    let mut cook = Cook::new();
    let mut last = vec![Vec::new(); sinks.len()];
    for k in 0..TICKS {
        let t = k as f64 * DT;
        cook.advance_tick(&doc.graph, reg, t).expect("avanca");
        for (i, &s) in sinks.iter().enumerate() {
            let out = cook.cook(&doc.graph, reg, s, t).expect("coze");
            if let Some(Column::Vec2(p)) = out[0].as_stream().get("P") {
                last[i] = p.clone();
            }
        }
    }
    last
}

/// A distância do ponto mais afastado ao centroide.
fn spread(p: &[[f32; 2]]) -> f32 {
    if p.is_empty() {
        return 0.0;
    }
    let c = p
        .iter()
        .fold([0.0_f32; 2], |a, q| [a[0] + q[0], a[1] + q[1]])
        .map(|v| v / p.len() as f32);
    p.iter()
        .map(|q| (q[0] - c[0]).hypot(q[1] - c[1]))
        .fold(0.0_f32, f32::max)
}

/// **A CENA MONTA AS OITO BANDAS**, e as oito cospem sem explodir.
#[test]
fn the_forces_scene_builds_all_eight_bands() {
    let (doc, reg, sinks) = scene();
    assert_eq!(sinks.len(), 8, "quatro pares");
    assert_eq!(band_labels().count(), 8, "um rotulo por banda");
    let poses = settle(&doc, &reg, &sinks);
    for (k, p) in poses.iter().enumerate() {
        assert!(!p.is_empty(), "banda {k} vazia");
        for q in p {
            assert!(q[0].is_finite() && q[1].is_finite(), "banda {k} explodiu");
        }
    }
}

/// SONDA — imprime o que cada par de facto faz, para as barras saírem de medição.
#[test]
#[ignore = "sonda de medicao, nao gate"]
fn measure_the_force_pairs() {
    let (doc, reg, sinks) = scene();
    let poses = settle(&doc, &reg, &sinks);
    // ⚠️ **Ao CENTROIDE, não à origem do mundo** — o `finish` desloca cada banda para o
    // quadrante dela, e uma régua ancorada na origem media o deslocamento do quadrante.
    let closest = |p: &[[f32; 2]]| {
        let c = p
            .iter()
            .fold([0.0_f32; 2], |a, q| [a[0] + q[0], a[1] + q[1]])
            .map(|v| v / p.len() as f32);
        p.iter()
            .map(|q| (q[0] - c[0]).hypot(q[1] - c[1]))
            .fold(f32::MAX, f32::min)
    };
    let far = |p: &[[f32; 2]]| p.iter().map(|q| q[0].hypot(q[1])).fold(0.0_f32, f32::max);
    for (k, p) in poses.iter().enumerate() {
        println!(
            "banda {k}: dispersao {:.4} · mais perto do centro {:.4} · mais longe {:.4}",
            spread(p),
            closest(p),
            far(p)
        );
    }
    let mut cook = Cook::new();
    for k in 0..TICKS {
        let t = k as f64 * DT;
        cook.advance_tick(&doc.graph, &reg, t).expect("avanca");
        for &s in &sinks {
            let _ = cook.cook(&doc.graph, &reg, s, t);
        }
        if k == 60 || k == TICKS - 1 {
            for (j, &s) in sinks.iter().enumerate().skip(2).take(4) {
                let out = cook.cook(&doc.graph, &reg, s, t).expect("coze");
                if let Some(Column::Vec2(v)) = out[0].as_stream().get("vel") {
                    let hi = v.iter().map(|q| q[0].hypot(q[1])).fold(0.0_f32, f32::max);
                    println!("  tique {k} banda {j}: maior velocidade {hi:.4}");
                }
            }
        }
    }
}

/// A distância do ponto MAIS PERTO do centroide — o quão vazio está o miolo.
fn hollow(p: &[[f32; 2]]) -> f32 {
    if p.is_empty() {
        return 0.0;
    }
    let c = p
        .iter()
        .fold([0.0_f32; 2], |a, q| [a[0] + q[0], a[1] + q[1]])
        .map(|v| v / p.len() as f32);
    p.iter()
        .map(|q| (q[0] - c[0]).hypot(q[1] - c[1]))
        .fold(f32::MAX, f32::min)
}

/// A maior velocidade da banda, em dois instantes.
fn speeds(doc: &MotionDoc, reg: &NodeRegistry, sinks: &[NodeId], at: &[usize]) -> Vec<Vec<f32>> {
    let mut cook = Cook::new();
    let mut out = vec![Vec::new(); sinks.len()];
    for k in 0..TICKS {
        let t = k as f64 * DT;
        cook.advance_tick(&doc.graph, reg, t).expect("avanca");
        for (i, &s) in sinks.iter().enumerate() {
            let o = cook.cook(&doc.graph, reg, s, t).expect("coze");
            if at.contains(&k)
                && let Some(Column::Vec2(v)) = o[0].as_stream().get("vel")
            {
                out[i].push(v.iter().map(|q| q[0].hypot(q[1])).fold(0.0_f32, f32::max));
            }
        }
    }
    out
}

/// ⭐⭐ **O PAR 1: a rampa COLAPSA num ponto e o perfil assenta num ANEL.**
///
/// ⚠️ **As duas réguas são ancoradas no CENTROIDE**, e não na origem do mundo: o `finish`
/// desloca cada banda para o quadrante dela, e uma régua da origem mediria o deslocamento.
///
/// ⚠️ **E a fixture teve de ser corrigida duas vezes antes de medir alguma coisa**, o que é
/// o registo mais útil aqui: (a) sem ARRASTO nada assenta — a nuvem atravessa o centro e
/// sai —, e (b) com a nuvem MAIOR que o raio de influência os cantos ficam fora da força e
/// o que se media era o que ela não alcança. *Um par ANTES/DEPOIS mede a diferença entre
/// dois estados de equilíbrio; sem equilíbrio ele mede o transiente.*
///
/// Medido: rampa `0,3315` de dispersão com o miolo a `0,0011`; perfil `1,3535` com o miolo
/// a `0,3266`.
#[test]
fn the_ramp_collapses_to_a_point_where_the_profile_settles_on_a_ring() {
    let (doc, reg, sinks) = scene();
    let poses = settle(&doc, &reg, &sinks[..2]);
    let (ramp, profiled) = (spread(&poses[0]), spread(&poses[1]));
    assert!(
        profiled > ramp * 3.0,
        "o perfil tinha de deixar um anel onde a rampa colapsa: {ramp:.4} contra {profiled:.4}"
    );
    // E o MIOLO do anel está vazio — é o que a inversão compra, e é o que uma dispersão
    // maior sozinha não distingue de uma nuvem que simplesmente não colapsou.
    let (core_r, core_p) = (hollow(&poses[0]), hollow(&poses[1]));
    assert!(
        core_r < 0.05,
        "CONTROLE: a rampa junta tudo, entao o miolo dela e' cheio ({core_r:.4})"
    );
    assert!(
        core_p > 0.15,
        "o miolo do anel tinha de ficar VAZIO: {core_p:.4}"
    );
}

/// ⭐⭐⭐ **OS PARES 2 e 3: o modo alvo SATURA onde a aceleração constante não para.**
///
/// ⚠️ **A régua é a VELOCIDADE, e não a distância percorrida** — que foi a primeira que
/// escrevi e estava errada: com uma resistência alta o modo alvo chega depressa e pode
/// **andar mais** nos primeiros segundos. A afirmação é sobre a derivada, e é ela que se
/// mede: entre o tique 60 e o 149, o modo `Force` sobe e o modo alvo fica onde está.
///
/// Medido no vento: `2,0000 → 4,9667` contra `1,9079 → 1,9990` — e o `1,999` **é** a
/// `strength` do vento, que é exactamente o que «saturar» quer dizer.
#[test]
fn the_target_velocity_saturates_where_the_constant_force_keeps_accelerating() {
    let (doc, reg, sinks) = scene();
    let v = speeds(&doc, &reg, &sinks[2..6], &[60, TICKS - 1]);
    // ⚠️ **A subida vale só para o VENTO, e a razão é GEOMÉTRICA.** Um vento não tem
    // extensão: uma aceleração constante ali acelera para sempre (`2,0000 → 4,9667`). Um
    // vórtice TEM raio, então quem sai dele deixa de receber força — a versão `Force` dele
    // também estabiliza (`4,6723 → 4,9472`, 6%), mas **por ter fugido do campo** e não por
    // uma lei. *Duas saturações que se parecem e têm causas diferentes: exigir a mesma
    // subida das duas seria medir a geometria da fixture, não o modo.* No vórtice a
    // afirmação que se pode fazer é a de baixo — o modo alvo fica bem ABAIXO do constante.
    let (wf0, wf1) = (v[0][0], v[0][1]);
    assert!(
        wf1 > wf0 * 1.2,
        "o vento constante tinha de CRESCER ({wf0:.4} -> {wf1:.4})"
    );
    assert!(
        v[3][1] < v[2][1] * 0.5,
        "vortice: o modo alvo tinha de ficar bem abaixo do constante ({:.4} contra {:.4})",
        v[2][1],
        v[3][1]
    );
    // E a saturação, que vale para os dois.
    for (k, name) in [(0_usize, "vento"), (2, "vortice")] {
        let (t0, t1) = (v[k + 1][0], v[k + 1][1]);
        assert!(
            t1 <= t0 * 1.05,
            "{name}: o modo alvo tinha de SATURAR ({t0:.4} -> {t1:.4})"
        );
        assert!(
            t1 > 0.1,
            "{name}: e ele tem de andar alguma coisa ({t1:.4})"
        );
    }
}

/// ⭐ **O PAR 4: as duas superfícies não são a mesma.**
///
/// ⚠️ **A afirmação sobre a FORMA das cristas vive no gate do crate**
/// (`the_spectrum_breaks_the_single_wavelength`, que mede a distância entre cristas
/// vizinhas). Aqui, ao nível da cena, o que se pode afirmar é mais fraco e é o que importa
/// para o olho: as peças assentam noutro sítio, e nenhuma das duas explode.
#[test]
fn the_two_seas_are_not_the_same_sea() {
    let (doc, reg, sinks) = scene();
    let poses = settle(&doc, &reg, &sinks[6..8]);
    assert_eq!(poses[0].len(), poses[1].len(), "a mesma contagem");
    let apart = poses[0]
        .iter()
        .zip(&poses[1])
        .map(|(a, b)| (a[1] - b[1]).abs())
        .fold(0.0_f32, f32::max);
    assert!(apart > 0.3, "as duas superficies coincidiram ({apart:.4})");
    for (k, p) in poses.iter().enumerate() {
        for q in p {
            assert!(q[1].abs() < 40.0, "o mar {k} atirou uma peca para {q:?}");
        }
    }
}

/// As fichas do canvas: uma por banda, curta.
#[test]
fn every_band_carries_its_caption() {
    let caps = captions();
    assert_eq!(caps.len(), 8, "uma ficha por banda");
    for c in &caps {
        assert!(!c.text.contains("--"), "a ficha e' curta: {:?}", c.text);
        assert!(!c.text.is_empty(), "ficha vazia");
    }
}
