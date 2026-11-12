//! O RESTAURO (undo/redo) suja só a caixa do que escreve, volta à grade nova ao bit, e a variante sem zerar o
//! fluxo dá o mesmo estado (filho de [`super`], LOC cap).

use super::*;

/// As células (índice com o pad) em que algum plano que a tela desenha difere AO BIT entre `a` e `b`.
fn celulas_que_diferem(a: &crate::grid::GridSnapshot, b: &crate::grid::GridSnapshot) -> Vec<usize> {
    fn bits<T: std::fmt::Debug>(v: &[T], i: usize) -> String {
        v.get(i).map(|x| format!("{x:?}")).unwrap_or_default()
    }
    let f32s = |i: usize| {
        [
            (&a.film, &b.film),
            (&a.susp, &b.susp),
            (&a.sett, &b.sett),
            (&a.relevo, &b.relevo),
            (&a.altura, &b.altura),
            (&a.espessura, &b.espessura),
            (&a.crista, &b.crista),
            (&a.nivelado, &b.nivelado),
            (&a.vela, &b.vela),
        ]
        .iter()
        .any(|(x, y)| x.get(i).map(|v| v.to_bits()) != y.get(i).map(|v| v.to_bits()))
    };
    let (apa, apb) = (a.aparencia.para_vec(), b.aparencia.para_vec());
    let (ola, olb) = (a.oleo.para_vec(), b.oleo.para_vec());
    let (maa, mab) = (a.material.para_vec(), b.material.para_vec());
    (0..a.film.len())
        .filter(|&i| {
            f32s(i)
                || bits(&a.susp_rgb, i) != bits(&b.susp_rgb, i)
                || bits(&a.sett_rgb, i) != bits(&b.sett_rgb, i)
                || bits(&maa, i) != bits(&mab, i)
                || bits(&apa, i) != bits(&apb, i)
                || bits(&ola, i) != bits(&olb, i)
                || bits(&a.tinta, i) != bits(&b.tinta, i)
                || bits(&a.pigmento, i) != bits(&b.pigmento, i)
                || a.wet.get(i) != b.wet.get(i)
                || a.bloom.get(i) != b.bloom.get(i)
        })
        .collect()
}

/// **B12.2 — o restauro suja a CAIXA do que escreve, não a tela** (a porta `Full`): cada célula que o undo/redo muda
/// AO BIT (todos os planos que a tela desenha) cai dentro do sujo, que é um `Rect` — o composite da caixa dá a mesma
/// tela que o do `Full`. Um passo vazio não suja nada.
#[test]
fn historia_o_restauro_suja_a_caixa_e_nao_a_tela() {
    let mut e = Engine::new(600, 400);
    e.sliders.water = 1.0;
    traco(&mut e, 40.0, 40.0, 5);
    corre(&mut e, 5);
    traco(&mut e, 300.0, 200.0, 5);
    corre(&mut e, 5);
    e.drain_step();
    for refaz in [false, true] {
        let antes = snapshot_grid(e.active_grid());
        e.take_dirty();
        assert!(if refaz { e.redo() } else { e.undo() });
        let depois = snapshot_grid(e.active_grid());
        let d = e.take_dirty();
        let Dirty::Rect { x0, y0, x1, y1 } = d else {
            panic!("refaz={refaz}: o sujo do restauro é {d:?}, não a caixa");
        };
        let s = e.active_grid().s;
        let mudadas = celulas_que_diferem(&antes, &depois);
        assert!(
            !mudadas.is_empty(),
            "refaz={refaz}: o restauro não mudou nada"
        );
        let fora = mudadas
            .iter()
            .filter(|&&i| {
                let (x, y) = ((i % s) as i32, (i / s) as i32);
                !(x0..=x1).contains(&x) || !(y0..=y1).contains(&y)
            })
            .count();
        assert_eq!(fora, 0, "refaz={refaz}: células mudadas fora do sujo {d:?}");
    }
    // Passo vazio: capturar sem escrever e desfazer não suja nada.
    e.capture_history();
    e.take_dirty();
    assert!(e.undo());
    assert_eq!(e.take_dirty(), Dirty::Clean, "o passo vazio sujou a tela");
}

/// **B12.2, a régua a 3840×2160:** o Ctrl+Z de um traço molhado compõe a caixa dele, não as 8 294 400 células.
#[test]
fn historia_o_undo_a_4k_compoe_a_caixa_do_traco() {
    let mut e = Engine::new(3840, 2160);
    e.sliders.water = 1.0;
    traco(&mut e, 1900.0, 1000.0, 8);
    corre(&mut e, 3);
    e.drain_step();
    e.take_dirty();
    assert!(e.undo());
    let area = match e.take_dirty() {
        Dirty::Rect { x0, y0, x1, y1 } => i64::from(x1 - x0 + 1) * i64::from(y1 - y0 + 1),
        Dirty::Full => 3840 * 2160,
        Dirty::Clean => 0,
    };
    eprintln!(
        "B12.2 undo 3840x2160: celulas sujas {area} (Full = {})",
        3840 * 2160
    );
    assert!(
        area > 0 && area < 3840 * 2160 / 20,
        "o undo compõe {area} células"
    );
}

/// **B12.1, o anel:** o 1.º traço numa camada NOVA (sem nenhum passo antes) desfaz-se até à grade nova ao bit,
/// porque o anel de dreno já nasce selado e o 1.º passo não escreve fora do registo.
#[test]
fn historia_o_undo_do_primeiro_traco_volta_a_grade_nova() {
    for rf in [1, 2] {
        let mut e = Engine::with_flow_ratio(960, 640, rf);
        e.sliders.water = 1.0;
        let nova = hash_grid(e.active_grid());
        traco(&mut e, 480.0, 320.0, 6);
        corre(&mut e, 4);
        e.drain_step();
        assert_ne!(
            hash_grid(e.active_grid()),
            nova,
            "o traço não pousou (rf {rf})"
        );
        assert!(e.undo());
        e.drain_step();
        assert_eq!(
            hash_grid(e.active_grid()),
            nova,
            "undo do 1.º traço (rf {rf})"
        );
    }
}

/// O undo vivo do Painter reescreve o fluxo inteiro depois do restauro: a variante que não o zera tem de dar o MESMO
/// estado (grade, fluxo, passo) que o restauro de sempre seguido da mesma reescrita — e de facto não o zerar.
fn sem_zerar_o_fluxo_da_o_mesmo_estado(rf: usize) {
    let monta = || {
        let mut e = Engine::with_flow_ratio(300, 220, rf);
        e.sliders.water = 1.0;
        for (x, y) in [(40.0, 40.0), (200.0, 60.0), (60.0, 150.0)] {
            traco(&mut e, x, y, 7);
            corre(&mut e, 20);
        }
        e
    };
    let (mut a, mut b) = (monta(), monta());
    let foto = |e: &Engine| {
        (
            e.active_grid().flow_x.clone(),
            e.active_grid().flow_y.clone(),
        )
    };
    let repoe = |e: &mut Engine, f: &(Vec<f32>, Vec<f32>)| {
        let l = e.active_layer;
        e.layers[l].grid.flow_x.clone_from(&f.0);
        e.layers[l].grid.flow_y.clone_from(&f.1);
    };
    for passo in ["undo", "undo", "redo", "undo", "redo", "redo"] {
        let f = foto(&a);
        assert_eq!(
            f,
            foto(&b),
            "{passo} (rf {rf}): partiram de fluxos diferentes"
        );
        let l = b.active_layer;
        b.layers[l].grid.flow_x.fill(7.0);
        let (fa, fb) = if passo == "undo" {
            (a.undo(), b.undo_sem_zerar_o_fluxo())
        } else {
            (a.redo(), b.redo_sem_zerar_o_fluxo())
        };
        assert!(fa && fb, "{passo} (rf {rf}): sem passo");
        assert!(
            b.active_grid().flow_x.iter().all(|v| *v == 7.0),
            "{passo} (rf {rf}): a variante zerou o fluxo — a cura não está lá"
        );
        assert!(
            a.active_grid().flow_x.iter().all(|v| *v == 0.0),
            "controlo: o restauro de sempre zera"
        );
        repoe(&mut a, &f);
        repoe(&mut b, &f);
        assert_eq!(
            hash_grid(a.active_grid()),
            hash_grid(b.active_grid()),
            "{passo} (rf {rf}): grade"
        );
        assert_eq!(foto(&a), foto(&b), "{passo} (rf {rf}): fluxo");
        assert_eq!(a.dirty, b.dirty, "{passo} (rf {rf}): sujo");
        corre(&mut a, 5);
        corre(&mut b, 5);
        assert_eq!(
            hash_grid(a.active_grid()),
            hash_grid(b.active_grid()),
            "{passo} (rf {rf}): água depois"
        );
    }
}

#[test]
fn historia_sem_zerar_o_fluxo_da_o_mesmo_estado_que_o_restauro_de_sempre() {
    sem_zerar_o_fluxo_da_o_mesmo_estado(1);
    sem_zerar_o_fluxo_da_o_mesmo_estado(2);
}
