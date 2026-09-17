//! **O emissor sob uma AGENDA** — o modo `Scheduled` (TOP-20 #18, W0), irmão do `burst`.
//!
//! As leis do relógio vêm do oráculo (Godot 4.7.2 MIT, `godot_particles_probe.gd`, doc 14 §2).

use super::*;
use crate::schedule::{EMIT_SCHEDULED, SCHEDULE_KEY};
use crate::{MANIFEST, MotionEmitter, Schedule, Spawn, emit, window_in};

fn agendado(rate: f32, life: f32, texto: &str) -> Spec {
    let mut s = spec();
    s.spawn = Spawn::Scheduled { rate };
    s.life = life;
    s.schedule = Schedule::parse(texto).expect("agenda");
    s
}

fn ages_of(s: &Stream) -> Vec<f32> {
    match s.get("age").unwrap() {
        Column::Scalar(v) => v.clone(),
        other => panic!("age is Scalar, got {other:?}"),
    }
}

/// ⭐⭐⭐ **A agenda vazia É o contínuo, ao bit** — o braço que sempre shipou corre intacto.
#[test]
fn a_agenda_vazia_e_o_continuo_ao_bit() {
    let mut cont = spec();
    cont.spawn = Spawn::Continuous { rate: 37.0 };
    cont.life = 1.3;
    let sch = agendado(37.0, 1.3, "");
    for i in 0..90 {
        let t = i as f32 * 0.071;
        assert_eq!(emit(&cont, t), emit(&sch, t), "t = {t}");
    }
}

/// ⭐⭐ **L1 e L2 do oráculo** — uma rajada de `n = 8` com explosividade `e`: a emissão dura
/// `(1 − e)·vida`, nascem EXACTAMENTE `n`, e a última morre em `vida·(1 + (n−1)/n·(1 − e))`.
#[test]
fn a_rajada_unica_reproduz_o_relogio_do_oraculo() {
    let (n, life) = (8.0_f32, 1.0_f32);
    for (e, fim_oraculo) in [(0.0_f32, 1.875_f32), (0.5, 1.4375)] {
        let d = (1.0 - e) * life;
        let s = agendado(n / d, life, &format!("0-{d}"));
        let ids: std::collections::BTreeSet<u32> = (0..400)
            .flat_map(|i| {
                let out = emit(&s, i as f32 * 0.005);
                // Um fluxo vazio não tem colunas — a varredura passa por instantes sem ninguém.
                let ids = if out.count() == 0 {
                    Vec::new()
                } else {
                    ids_of(&out)
                };
                ids.into_iter().map(|x| x as u32)
            })
            .collect();
        assert_eq!(ids.len(), n as usize, "e = {e}: nasceram {ids:?}");
        let vivas = |t: f32| emit(&s, t).count();
        assert_eq!(
            vivas(fim_oraculo),
            1,
            "e = {e}: a última ainda vive no fim exacto"
        );
        assert_eq!(
            vivas(fim_oraculo + 0.001),
            0,
            "e = {e}: e morre logo a seguir"
        );
        assert!(vivas(fim_oraculo - 0.2) >= 1);
    }
}

/// ⭐⭐ **L5 e L6 — desligar não mata as vivas; religar continua a numeração.**
#[test]
fn desligar_nao_mata_e_religar_continua() {
    let s = agendado(10.0, 3.0, "0-1 2-");
    let meio = emit(&s, 1.5);
    assert_eq!(
        ids_of(&meio),
        (0..10).map(|i| i as f32).collect::<Vec<_>>(),
        "na pausa: as dez"
    );
    let volta = emit(&s, 2.0);
    let ids = ids_of(&volta);
    assert_eq!(ids.len(), 11, "no instante em que religa nasce a 11.ª");
    assert!(
        ids.windows(2).all(|w| w[1] == w[0] + 1.0),
        "contíguas: {ids:?}"
    );
    let ages = ages_of(&volta);
    assert!(
        (ages[10] - 0.0).abs() < 1e-6,
        "a nova tem idade zero: {ages:?}"
    );
    assert!(
        (ages[9] - 1.1).abs() < 1e-5,
        "a 10.ª nasceu em 0,9: {ages:?}"
    );
    assert!(
        ages.windows(2).all(|w| w[1] <= w[0]),
        "da mais velha para a mais nova"
    );
}

/// ⭐⭐⭐ **A janela é a enumeração** — a lei contígua contra a definição, partícula a partícula,
/// em agendas com pausas, períodos e fins, e com o tecto a cortar.
#[test]
fn a_janela_e_a_enumeracao_bruta() {
    let agendas = [
        "0-1 2-",
        "0.3-0.8 1-1.1 1.5-2",
        "pulse 0.25/0.6",
        "0-1.3 2- pulse 0.3/0.5",
        "1-3",
        "off",
    ];
    for texto in agendas {
        let sched = Schedule::parse(texto).unwrap();
        for (rate, life, max) in [(10.0_f32, 0.7_f32, 1024), (33.0, 2.0, 1024), (50.0, 1.0, 7)] {
            for i in 0..260 {
                let t = i as f32 * 0.0137;
                let w = window_in(Spawn::Scheduled { rate }, &sched, life, max, t);
                let bruta: Vec<u32> = (0..4000u32)
                    .filter(|&k| {
                        sched
                            .birth(f64::from(k) / f64::from(rate))
                            .is_some_and(|b| {
                                b <= f64::from(t) && b >= f64::from(t) - f64::from(life)
                            })
                    })
                    .collect();
                let esperado: Vec<u32> = bruta[bruta.len().saturating_sub(max)..].to_vec();
                let janela: Vec<u32> = (0..w.count as u32).map(|k| w.first + k).collect();
                assert_eq!(janela, esperado, "`{texto}` rate {rate} life {life} t {t}");
                if let Some(&k) = esperado.first() {
                    let idade = f64::from(t) - sched.birth(f64::from(k) / f64::from(rate)).unwrap();
                    assert!(
                        (f64::from(w.age_first) - idade).abs() < 1e-4,
                        "idade da primeira"
                    );
                }
            }
        }
    }
}

/// ⛔ **O device recusa o modo** — a agenda não tem espelho WGSL, e a recusa é declarada.
#[test]
fn o_device_recusa_a_agenda() {
    let app = crate::gpu::GPU_KERNEL.applicable.expect("a recusa existe");
    let com = |modo: f32| {
        move |n: &str| match n {
            "probability" => 1.0,
            "emit_mode" => modo,
            _ => 0.0,
        }
    };
    assert!(app(&com(0.0)), "contínuo: device");
    assert!(app(&com(1.0)), "rajada: device");
    assert!(
        !app(&com(EMIT_SCHEDULED as f32)),
        "agendado: CPU — o kernel não conhece a agenda"
    );
}

struct Ops;
impl ph2d_nodegraph::cook::OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        (ty == MANIFEST.id).then_some(&MotionEmitter as &dyn NodeOp)
    }
}

/// **O nó lê a agenda do TEXTO, só no modo que a lê** — e um texto malformado não emite.
#[test]
fn o_no_le_a_agenda_do_texto_e_falha_fechado() {
    use ph2d_nodegraph::cook::Cook;
    use ph2d_nodegraph::graph::Graph;
    let vivas = |modo: f32, texto: Option<&str>, t: f64| -> usize {
        let mut g = Graph::new();
        let em = g.add_node("motion.emitter");
        g.set_param(em, "rate", 10.0);
        g.set_param(em, "life", 3.0);
        g.set_param(em, "emit_mode", modo);
        if let Some(x) = texto {
            g.set_text_param(em, SCHEDULE_KEY, x);
        }
        let mut cook = Cook::new();
        cook.cook(&g, &Ops, em, t).unwrap()[0].as_stream().count()
    };
    let m = EMIT_SCHEDULED as f32;
    assert_eq!(vivas(m, Some("0-1"), 1.5), 10, "a agenda corta a emissão");
    assert_eq!(
        vivas(0.0, Some("0-1"), 1.5),
        16,
        "no modo contínuo o texto é inerte"
    );
    assert_eq!(vivas(m, None, 1.5), 16, "sem texto: a identidade");
    assert_eq!(vivas(m, Some("0-x"), 1.5), 0, "⛔ malformado não acende");
    assert_eq!(vivas(m, Some("off"), 1.5), 0);
}
