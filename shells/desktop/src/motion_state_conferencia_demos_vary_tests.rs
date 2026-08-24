//! Gates da cena `=94` — a forma desenhada e a variação por elemento.

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
    let sinks = build_vary_demo_document(&mut doc, &reg).expect("a cena monta");
    doc.graph.validate(&reg).expect("bem-tipada");
    (doc, reg, sinks)
}

/// As colunas de um sink num instante.
fn at(doc: &MotionDoc, reg: &NodeRegistry, sink: NodeId, t: f64) -> ph2d_nodegraph::attr::Stream {
    let mut cook = Cook::new();
    cook.advance_tick(&doc.graph, reg, t).expect("avanca");
    let out = cook.cook(&doc.graph, reg, sink, t).expect("coze");
    out[0].as_stream().clone()
}

/// Quantos valores distintos uma leitura produziu.
fn distinct(v: impl Iterator<Item = u32>) -> usize {
    let mut x: Vec<u32> = v.collect();
    x.sort_unstable();
    x.dedup();
    x.len()
}

/// **A CENA MONTA AS DEZ BANDAS**, e as dez cospem.
#[test]
fn the_vary_scene_builds_all_ten_bands() {
    let (doc, reg, sinks) = scene();
    assert_eq!(sinks.len(), 10, "cinco pares");
    assert_eq!(band_labels().count(), 10, "um rotulo por banda");
    for (k, s) in sinks.iter().enumerate() {
        let st = at(&doc, &reg, *s, 1.4);
        assert!(st.count() > 0, "banda {k} vazia");
    }
}

/// ⭐⭐ **O PAR 1 É O DEFEITO CURADO: a forma DESENHADA conduz.**
///
/// A esquerda é a `Sine`, a direita é a `Custom` com um V autorado. Se a curva não fosse
/// lida, as duas seriam a mesma onda — que é exactamente o que o artista via.
#[test]
fn the_drawn_wave_drives_where_the_sine_does_not() {
    let (doc, reg, sinks) = scene();
    // Uma fatia de tempo em que as duas ondas discordam de facto.
    let mut apart = 0.0_f32;
    for k in 0..12 {
        let t = f64::from(k) * 0.17;
        let (a, b) = (at(&doc, &reg, sinks[0], t), at(&doc, &reg, sinks[1], t));
        let (Some(Column::Vec2(pa)), Some(Column::Vec2(pb))) = (a.get("P"), b.get("P")) else {
            panic!("P")
        };
        let d = pa
            .iter()
            .zip(pb)
            .map(|(x, y)| (x[1] - y[1]).abs())
            .fold(0.0_f32, f32::max);
        apart = apart.max(d);
    }
    assert!(
        apart > 0.3,
        "a onda desenhada tinha de conduzir para outro sitio que a Sine: {apart:.4}"
    );
}

/// ⭐⭐⭐ **OS QUATRO PARES DE PARTÍCULA VARIAM, cada um no seu canal** — e o par de
/// controlo NÃO varia.
#[test]
fn each_channel_varies_on_the_right_and_holds_on_the_left() {
    let (doc, reg, sinks) = scene();
    // (par, o que ler)
    let read = |s: &ph2d_nodegraph::attr::Stream, ch: usize| -> Vec<u32> {
        match ch {
            0 => match s.get("rot") {
                Some(Column::Scalar(r)) => r.iter().map(|x| x.to_bits()).collect(),
                _ => Vec::new(),
            },
            3 => match s.get("size") {
                Some(Column::Vec2(z)) => z.iter().map(|q| q[0].to_bits()).collect(),
                _ => Vec::new(),
            },
            _ => match s.get("tint") {
                Some(Column::Vec4(t)) => t
                    .iter()
                    .map(|c| {
                        if ch == 1 {
                            c[3].to_bits()
                        } else {
                            ph2d_color::rgb_to_hsv(*c).0.to_bits()
                        }
                    })
                    .collect(),
                _ => Vec::new(),
            },
        }
    };
    for ch in 0..4 {
        let (l, r) = (sinks[2 + ch * 2], sinks[3 + ch * 2]);
        let (sl, sr) = (at(&doc, &reg, l, 1.6), at(&doc, &reg, r, 1.6));
        let n = sr.count();
        assert!(n > 20, "banda {ch} tem particulas: {n}");
        let (dl, dr) = (
            distinct(read(&sl, ch).into_iter()),
            distinct(read(&sr, ch).into_iter()),
        );
        assert!(
            dl <= 2,
            "CONTROLE: a esquerda do par {ch} tinha de ser uniforme, e deu {dl} valores"
        );
        assert!(
            dr > n / 2,
            "o par {ch} tinha de variar: {dr} valores distintos em {n}"
        );
    }
}

/// ⚠️ **A variação NÃO PISCA enquanto a partícula vive** — ela é da IDENTIDADE, e num
/// emissor a janela viva desliza a cada tique.
///
/// É a armadilha que o `value.instance_field(Random)` tem por default (ele chaveia pelo
/// ÍNDICE), e a razão de o `motion.randomize` não oferecer esse knob.
#[test]
fn a_particle_keeps_its_draw_while_it_lives() {
    let (doc, reg, sinks) = scene();
    let snap = |t: f64| -> Vec<(u32, u32)> {
        let s = at(&doc, &reg, sinks[3], t); // o par da rotação
        let (Some(Column::Scalar(ids)), Some(Column::Scalar(rot))) = (s.get("id"), s.get("rot"))
        else {
            panic!("id/rot")
        };
        ids.iter()
            .zip(rot)
            .map(|(i, r)| (*i as u32, r.to_bits()))
            .collect()
    };
    let (a, b) = (snap(1.5), snap(1.62));
    let mut shared = 0;
    for (id, r) in &a {
        if let Some((_, r2)) = b.iter().find(|(j, _)| j == id) {
            assert_eq!(r, r2, "a particula {id} trocou de angulo enquanto vivia");
            shared += 1;
        }
    }
    assert!(
        shared > 10,
        "CONTROLE: as duas fotos partilham particulas ({shared})"
    );
}

/// As fichas do canvas: uma por banda, curta.
#[test]
fn every_band_carries_its_caption() {
    let caps = captions();
    assert_eq!(caps.len(), 10, "uma ficha por banda");
    for c in &caps {
        assert!(!c.text.contains("--"), "a ficha e' curta: {:?}", c.text);
        assert!(!c.text.is_empty(), "ficha vazia");
    }
}
