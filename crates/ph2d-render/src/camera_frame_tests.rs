use super::*;
use std::path::PathBuf;

fn fixtures(dir: &str) -> Vec<(String, serde_json::Value)> {
    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/UI_New_and_Simple/vetor_nova_ui/oraculo_enquadrar")
        .join(dir);
    let mut out: Vec<_> = std::fs::read_dir(&d)
        .unwrap_or_else(|e| panic!("{}: {e}", d.display()))
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .map(|p| {
            let s = std::fs::read_to_string(&p).unwrap();
            (p.display().to_string(), serde_json::from_str(&s).unwrap())
        })
        .collect();
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

fn f(v: &serde_json::Value) -> f32 {
    #[allow(clippy::cast_possible_truncation)]
    let x = v.as_f64().unwrap() as f32;
    x
}

/// Os objectos da cena (px de página, y para baixo) como caixa de MUNDO (y para cima, 1 px = 1).
fn caixa(objs: &[serde_json::Value], so: Option<&[serde_json::Value]>) -> Option<WorldBbox> {
    let mut b = None;
    for (i, o) in objs.iter().enumerate() {
        if let Some(sel) = so
            && !sel.iter().any(|s| s.as_u64() == Some(i as u64))
        {
            continue;
        }
        let (x, y, w, h) = (f(&o[0]), f(&o[1]), f(&o[2]), f(&o[3]));
        b = Some(union(
            b,
            WorldBbox {
                min: [x, -(y + h)],
                max: [x + w, -y],
            },
        ));
    }
    b
}

/// A caixa no ecrã pela câmera de `v` — a mesma pergunta que o oráculo respondeu.
fn no_ecra(v: View, b: WorldBbox, win: WindowSize) -> [f32; 4] {
    let cam = Camera2d::new(v.center, v.height_world);
    let (x0, y0) = cam.world_to_screen([b.min[0], b.max[1]], win);
    let (x1, y1) = cam.world_to_screen([b.max[0], b.min[1]], win);
    [x0, y0, x1, y1]
}

/// ⭐⭐ **A mesma lei, com os parâmetros de cada oráculo, reproduz os números GRAVADOS deles** — em
/// três janelas e três câmaras de partida. Com o tldraw: a janela inteira, 64 px de margem, zoom em
/// `[0,05, 8]`. Com o Excalidraw: a área livre da interface dele (`getViewportOffsets`), margem 0,
/// zoom em `[0,1, 1]`. O que difere entre eles são PARÂMETROS; a estrutura (o eixo apertado manda,
/// o outro centra, um ponto vai ao zoom máximo, o que transborda centra) é uma só — e é esta.
#[test]
fn the_law_with_each_oracles_parameters_reproduces_its_recorded_frames() {
    let mut vistos = 0;
    for (dir, tldraw) in [
        ("fixtures_tldraw", true),
        ("fixtures_tldraw_1920x1080", true),
        ("fixtures_tldraw_700x900", true),
        ("fixtures_excalidraw", false),
        ("fixtures_excalidraw_1920x1080", false),
        ("fixtures_excalidraw_700x900", false),
    ] {
        for (nome, fx) in fixtures(dir) {
            let objs = fx["_cabecalho"]["objectos"]["objs"].as_array().unwrap();
            let Some(tudo) = caixa(objs, None) else {
                continue;
            };
            let sel = fx["_cabecalho"]["objectos"]["sel"].as_array();
            for (k, r) in fx["corridas"].as_object().unwrap() {
                let dep = &r["depois"];
                let vp = &dep["viewport"];
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let win = WindowSize::new(f(&vp["w"]) as u32, f(&vp["h"]) as u32);
                let wh = f(&vp["h"]);
                let (area, margem, z) = if tldraw {
                    (
                        Area {
                            x: 0.0,
                            y: 0.0,
                            w: f(&vp["w"]),
                            h: wh,
                        },
                        64.0,
                        (0.05, 8.0),
                    )
                } else {
                    let ui = &vp["ui"];
                    let (t, rr, bb, l) = (
                        f(&ui["top"]),
                        f(&ui["right"]),
                        f(&ui["bottom"]),
                        f(&ui["left"]),
                    );
                    (
                        Area {
                            x: l,
                            y: t,
                            w: f(&vp["w"]) - l - rr,
                            h: wh - t - bb,
                        },
                        0.0,
                        (0.1, 1.0),
                    )
                };
                let range = (wh / z.1, wh / z.0);
                // O «tudo» de toda corrida; a seleção só onde o oráculo a enquadra (o tldraw, fora
                // dos 100 %, vai a 100 % centrado nela — um ciclo dele, recusado: ver o doc da lei).
                let alvo = if k.ends_with("tecla_shift_1") || k.ends_with("api_zoomToFit") {
                    Some((tudo, "tudo_ecra"))
                } else if (k.ends_with("tecla_shift_2") || k.ends_with("api_zoomToSelection"))
                    && sel.is_some()
                    && (!tldraw || k.starts_with("z1_origem"))
                {
                    Some((caixa(objs, sel.map(Vec::as_slice)).unwrap(), "selecao_ecra"))
                } else {
                    None
                };
                let Some((b, campo)) = alvo else { continue };
                let v = frame_within(b, area, win, margem, range).unwrap();
                let nosso = no_ecra(v, b, win);
                let e = &dep[campo];
                let deles = [f(&e["x0"]), f(&e["y0"]), f(&e["x1"]), f(&e["y1"])];
                for i in 0..4 {
                    let tol = 0.01 * (1.0 + deles[i].abs() / 1000.0);
                    assert!(
                        (nosso[i] - deles[i]).abs() < tol,
                        "{nome} {k}: o conteúdo no ecrã {nosso:?} ≠ o do oráculo {deles:?}"
                    );
                }
                vistos += 1;
            }
        }
    }
    // Controlo positivo: um caminho errado leria zero fixtures e passaria por vácuo.
    assert!(vistos >= 200, "só {vistos} corridas comparadas");
}

fn caixa_de(min: [f32; 2], max: [f32; 2]) -> WorldBbox {
    WorldBbox { min, max }
}

#[test]
fn a_point_goes_to_the_cameras_closest_zoom_and_never_fills_the_screen() {
    let win = WindowSize::new(1600, 1000);
    let area = Area {
        x: 300.0,
        y: 40.0,
        w: 1000.0,
        h: 900.0,
    };
    let v = frame(caixa_de([2.0, 3.0], [2.0, 3.0]), area, win, 44.0).unwrap();
    assert_eq!(v.height_world, Camera2d::ZOOM_MIN_HEIGHT_WORLD);
    // E o ponto cai no centro da ÁREA, não da janela.
    let cam = Camera2d::new(v.center, v.height_world);
    let (x, y) = cam.world_to_screen([2.0, 3.0], win);
    assert!(
        (x - 800.0).abs() < 1e-3 && (y - 490.0).abs() < 1e-3,
        "{x},{y}"
    );
}

#[test]
fn the_area_not_the_window_receives_the_box_with_the_margin_on_the_tight_axis() {
    // Os painéis tapam 300 px à esquerda e 200 à direita: a caixa cabe nos 1100 do meio.
    let win = WindowSize::new(1600, 1000);
    let area = Area {
        x: 300.0,
        y: 0.0,
        w: 1100.0,
        h: 1000.0,
    };
    let b = caixa_de([-10.0, -1.0], [10.0, 1.0]);
    let v = frame(b, area, win, 44.0).unwrap();
    let [x0, y0, x1, y1] = no_ecra(v, b, win);
    assert!(
        (x0 - 344.0).abs() < 1e-2 && (x1 - 1356.0).abs() < 1e-2,
        "{x0} {x1}"
    );
    assert!(
        ((y0 + y1) * 0.5 - 500.0).abs() < 1e-2,
        "o eixo folgado centra: {y0} {y1}"
    );
}

#[test]
fn what_the_widest_zoom_cannot_hold_is_centred_and_overflows() {
    let win = WindowSize::new(1000, 1000);
    let area = Area {
        x: 0.0,
        y: 0.0,
        w: 1000.0,
        h: 1000.0,
    };
    let v = frame(caixa_de([0.0, 0.0], [4000.0, 10.0]), area, win, 44.0).unwrap();
    assert_eq!(v.height_world, Camera2d::ZOOM_MAX_HEIGHT_WORLD);
    assert_eq!(v.center, [2000.0, 5.0]);
}

#[test]
fn a_degenerate_area_frames_without_the_margin_and_nan_refuses() {
    let win = WindowSize::new(100, 100);
    let b = caixa_de([0.0, 0.0], [1.0, 1.0]);
    let estreita = Area {
        x: 0.0,
        y: 0.0,
        w: 60.0,
        h: 100.0,
    };
    let v = frame(b, estreita, win, 44.0).unwrap();
    assert!(
        (v.height_world - 100.0 / 60.0).abs() < 1e-5,
        "{}",
        v.height_world
    );
    assert!(frame(caixa_de([f32::NAN, 0.0], [1.0, 1.0]), estreita, win, 44.0).is_none());
}

/// ⭐ **O ponto fixo fica parado** — o ponto do mundo que está no mesmo sítio do ecrã nas duas vistas
/// está nesse sítio em TODO o percurso (o voo lê-se como «aproximar EM DIRECÇÃO a» e não como um
/// pan com zoom por cima); e o zoom anda em `ln(h)` linear.
#[test]
fn along_keeps_the_fixed_point_still_and_walks_the_zoom_in_log() {
    let win = WindowSize::new(1200, 800);
    let a = View {
        center: [3.0, -2.0],
        height_world: 10.0,
    };
    let b = View {
        center: [40.0, 25.0],
        height_world: 1.25,
    };
    // O ponto fixo: (c0·h1 − c1·h0)/(h1 − h0).
    let w = |i: usize| {
        (a.center[i] * b.height_world - b.center[i] * a.height_world)
            / (b.height_world - a.height_world)
    };
    let fixo = [w(0), w(1)];
    let em = |v: View| Camera2d::new(v.center, v.height_world).world_to_screen(fixo, win);
    let p0 = em(a);
    for k in 0..=20 {
        #[allow(clippy::cast_precision_loss)]
        let s = k as f32 / 20.0;
        let v = along(a, b, s);
        let p = em(v);
        assert!(
            (p.0 - p0.0).abs() < 1e-2 && (p.1 - p0.1).abs() < 1e-2,
            "s={s}: {p:?} ≠ {p0:?}"
        );
        let lh = (v.height_world / a.height_world).ln() / (b.height_world / a.height_world).ln();
        assert!((lh - s).abs() < 1e-5, "s={s}: ln(h) andou {lh}");
    }
    assert_eq!(along(a, b, 1.0), b);
    assert_eq!(along(a, b, 0.0), a);
    // Sem zoom é a recta.
    let c = View {
        center: [10.0, 0.0],
        height_world: 10.0,
    };
    let m = along(a, c, 0.25);
    assert!((m.center[0] - (3.0 + 7.0 * 0.25)).abs() < 1e-5 && m.height_world == 10.0);
}

/// Uma mola de brincar: anda `passo` para o alvo e chega exactamente nele; a 1.ª vista de um id CHEGA.
struct Mola {
    v: Option<f32>,
    passo: f32,
    chamadas: Vec<f32>,
}

impl Mola {
    fn step(&mut self, goal: f32) -> f32 {
        self.chamadas.push(goal);
        let v = match self.v {
            None => goal,
            Some(v) if (goal - v).abs() <= self.passo => goal,
            Some(v) => v + self.passo * (goal - v).signum(),
        };
        self.v = Some(v);
        v
    }
}

fn voa(f: &mut ViewFlight, cam: &mut View, mola: &mut Mola) -> usize {
    let mut n = 0;
    while let Some(v) = f.tick(*cam, |g| mola.step(g)) {
        *cam = v;
        n += 1;
        assert!(n < 100, "o voo não assenta");
    }
    n
}

#[test]
fn the_flight_walks_the_spring_and_lands_on_the_target_verbatim() {
    let a = View {
        center: [0.0, 0.0],
        height_world: 10.0,
    };
    let b = View {
        center: [7.0, 3.0],
        height_world: 2.5,
    };
    let mut f = ViewFlight::default();
    let mut mola = Mola {
        v: None,
        passo: 0.25,
        chamadas: Vec::new(),
    };
    f.go(b, None);
    let mut cam = a;
    let n = voa(&mut f, &mut cam, &mut mola);
    assert_eq!(cam, b, "assenta no destino, sem resíduo");
    assert_eq!(n, 4, "quatro quartos");
    // A mola nasceu plantada na base (0) e só depois foi ao destino (1): sem isso a 1.ª vista saltava.
    assert_eq!(mola.chamadas[..2], [0.0, 1.0]);
    assert!(!f.flying());
    // Uma perna nova parte do valor da mola (1) — sem replantar.
    mola.chamadas.clear();
    f.go(a, mola.v);
    voa(&mut f, &mut cam, &mut mola);
    assert_eq!(cam, a);
    assert!(
        mola.chamadas.iter().all(|&g| g == 2.0),
        "{:?}",
        mola.chamadas
    );
}

#[test]
fn a_foreign_write_mid_flight_wins_and_the_flight_lets_go() {
    let a = View {
        center: [0.0, 0.0],
        height_world: 10.0,
    };
    let b = View {
        center: [7.0, 3.0],
        height_world: 2.5,
    };
    let mut f = ViewFlight::default();
    let mut mola = Mola {
        v: None,
        passo: 0.25,
        chamadas: Vec::new(),
    };
    f.go(b, None);
    let mut cam = f.tick(a, |g| mola.step(g)).unwrap();
    cam.height_world *= 0.9; // a roda
    assert_eq!(f.tick(cam, |g| mola.step(g)), None);
    assert!(!f.flying());
    // ANTES do 1.º tique não há escrita estrangeira: o voo parte de onde a câmera estiver (o resto
    // da mola de uma roda acabada de girar não desfaz o pedido).
    f.go(b, mola.v);
    let outra = View {
        center: [1.0, 0.0],
        ..a
    };
    let v = f.tick(outra, |g| mola.step(g)).unwrap();
    assert_eq!(v, along(outra, b, 0.25));
}

/// ⭐ **O beliscão ao contrário** (Procreate: *«return to the exact view you had before your
/// Quick-Pinch»*) — volta à vista de antes do enquadramento, só se a câmera ainda está onde ele a
/// deixou, e uma vez só.
#[test]
fn the_reverse_quick_pinch_returns_to_the_exact_view_only_while_untouched() {
    let antes = View {
        center: [5.0, 5.0],
        height_world: 3.0,
    };
    let tudo = View {
        center: [0.0, 0.0],
        height_world: 40.0,
    };
    let mut mola = Mola {
        v: None,
        passo: 0.5,
        chamadas: Vec::new(),
    };
    let mut f = ViewFlight::default();
    let mut cam = antes;
    f.go_remembering(tudo, mola.v);
    voa(&mut f, &mut cam, &mut mola);
    f.gesture_starts(cam);
    assert!(f.go_back(mola.v));
    voa(&mut f, &mut cam, &mut mola);
    assert_eq!(cam, antes, "a vista EXACTA");
    assert!(!f.go_back(mola.v), "uma vez só");

    // Mexida depois de assentar ⇒ a memória morre no começo do gesto.
    f.go_remembering(tudo, mola.v);
    voa(&mut f, &mut cam, &mut mola);
    cam.center[0] += 1.0;
    f.gesture_starts(cam);
    assert!(!f.go_back(mola.v));

    // Um enquadramento sem memória (a seleção) não arma nada.
    let mut g = ViewFlight::default();
    g.go(tudo, None);
    let mut cam = antes;
    voa(&mut g, &mut cam, &mut mola);
    g.gesture_starts(cam);
    assert!(!g.go_back(mola.v));
}
