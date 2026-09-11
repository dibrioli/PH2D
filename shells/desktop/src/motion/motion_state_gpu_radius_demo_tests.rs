//! Gates da cena `=28` — e a SONDA que produziu os números da mensagem de anúncio.
//!
//! A regra do plano 89: *toda wave ganha cena com números MEDIDOS, e a sonda headless roda
//! ANTES de a mensagem ser escrita*.

use super::*;
use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    reg
}

/// Cozinha a cena até assentar e devolve `(P, size)` de cada elemento.
fn settled(secs: f64) -> Vec<Rested> {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_gpu_radius_demo_document(&mut doc, &reg).expect("a cena é bem tipada");
    let mut cook = Cook::new();
    let mut out = Vec::new();
    for k in 0..=((secs * 60.0) as u64) {
        let t = k as f64 / 60.0;
        let s = cook.cook(&doc.graph, &reg, sinks[0], t).expect("cozinha")[0]
            .as_stream()
            .clone();
        if k == (secs * 60.0) as u64 {
            let p = match s.get("P") {
                Some(Column::Vec2(v)) => v.clone(),
                _ => vec![],
            };
            let sz = match s.get("size") {
                Some(Column::Vec2(v)) => v.clone(),
                _ => vec![],
            };
            out = p.into_iter().zip(sz).collect();
        }
        cook.advance_tick(&doc.graph, &reg, t).expect("avança");
    }
    out
}

/// Um elemento assentado: onde o centro dele parou, e que tamanho o renderizador desenha ali.
type Rested = ([f32; 2], [f32; 2]);

/// A metade ESQUERDA são os `COLS` primeiros (`motion.combine` concatena na ordem das portas).
fn halves(all: &[Rested]) -> (&[Rested], &[Rested]) {
    all.split_at(COLS as usize)
}

/// **A cena é bem tipada e tem as duas fileiras** — o mínimo que separa uma cena de um
/// documento que o `validate` recusa na abertura.
#[test]
fn the_scene_builds_with_both_rows() {
    let all = settled(4.0);
    assert_eq!(all.len(), (COLS * 2.0) as usize, "cinco discos por lado");
}

/// **OS TAMANHOS VARIAM — sem isto a cena inteira é vácua.**
///
/// O oráculo desta wave é *"as bordas de baixo alinham"*, e com discos todos iguais um `height`
/// subido à mão daria o mesmo desenho. Este gate é o que torna a comparação capaz de falhar.
#[test]
fn the_row_is_a_ramp_of_sizes_or_nothing_below_means_anything() {
    let all = settled(4.0);
    let (left, _) = halves(&all);
    let h: Vec<f32> = left.iter().map(|(_, s)| s[1]).collect();
    for pair in h.windows(2) {
        assert!(pair[1] > pair[0], "a rampa tem de crescer: {h:?}");
    }
    assert!(
        (h[0] - SIZE_MIN).abs() < 1e-4 && (h[COLS as usize - 1] - SIZE_MAX).abs() < 1e-4,
        "a faixa autorada é [{SIZE_MIN}, {SIZE_MAX}]; medido {h:?}"
    );
}

/// **O DEFEITO E A CURA, lado a lado, na mesma cena.**
///
/// À esquerda (`Point`) os CENTROS pousam na linha do chão e as bordas de baixo ficam espalhadas
/// por meia altura cada. À direita (`Sprite Size`) as BORDAS pousam e os centros é que se
/// espalham. Um oráculo é a negação exata do outro, o que é o que torna a foto legível.
///
/// FALSIFICADO por um raio que não alcança o contato: os dois lados teriam a mesma dispersão.
#[test]
fn the_point_row_sinks_and_the_sized_row_rests_on_top() {
    let all = settled(4.0);
    let (left, right) = halves(&all);
    let spread = |row: &[Rested], f: fn(&Rested) -> f32| -> f32 {
        let v: Vec<f32> = row.iter().map(f).collect();
        v.iter().cloned().fold(f32::MIN, f32::max) - v.iter().cloned().fold(f32::MAX, f32::min)
    };
    let centre = |e: &Rested| e.0[1];
    let bottom = |e: &Rested| e.0[1] - e.1[1] * 0.5;

    // ESQUERDA: os centros concordam, as bordas não.
    assert!(
        spread(left, centre) < 1e-3,
        "Point: todo centro pousa no chão; medido {:?}",
        left.iter().map(centre).collect::<Vec<_>>()
    );
    assert!(
        spread(left, bottom) > (SIZE_MAX - SIZE_MIN) * 0.4,
        "Point: as bordas de baixo TÊM de estar espalhadas — é o defeito; medido {:?}",
        left.iter().map(bottom).collect::<Vec<_>>()
    );

    // DIREITA: exatamente o contrário.
    assert!(
        spread(right, bottom) < 1e-3,
        "Sprite Size: as bordas alinham; medido {:?}",
        right.iter().map(bottom).collect::<Vec<_>>()
    );
    assert!(
        spread(right, centre) > (SIZE_MAX - SIZE_MIN) * 0.4,
        "Sprite Size: os centros é que se espalham agora; medido {:?}",
        right.iter().map(centre).collect::<Vec<_>>()
    );
}

/// **E as bordas pousam NO CHÃO, não numa linha qualquer.**
///
/// O gate acima prova que as cinco concordam entre si; este prova que elas concordam com o
/// número que o artista autorou. Sem ele, um raio com o dobro do tamanho passaria — as cinco
/// ainda alinhariam, meia unidade acima do chão.
#[test]
fn the_sized_row_rests_on_the_floor_the_artist_authored() {
    let all = settled(4.0);
    let (_, right) = halves(&all);
    for (p, s) in right {
        let bottom = p[1] - s[1] * 0.5;
        assert!(
            (bottom - FLOOR).abs() < 1e-3,
            "borda de baixo em {bottom}, chão em {FLOOR}"
        );
    }
}

/// **A SONDA** — imprime as duas fileiras, de onde saem os números do anúncio e do doc.
///
/// `cargo test -p ph2d-host-desktop --lib radius_demo::tests::probe -- --ignored --nocapture`
#[test]
#[ignore]
fn probe_radius_rest() {
    let all = settled(4.0);
    let (left, right) = halves(&all);
    for (name, row) in [("Point", left), ("Sprite Size", right)] {
        eprintln!("{name}:");
        for (p, s) in row {
            eprintln!(
                "   size={:.2}  centro y={:.4}  borda y={:.4}  (afunda {:.4})",
                s[1],
                p[1],
                p[1] - s[1] * 0.5,
                FLOOR - (p[1] - s[1] * 0.5)
            );
        }
    }
}
