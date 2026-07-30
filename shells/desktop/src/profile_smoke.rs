//! **A cena pronta para o smoke da LARGURA VIVA** — `PH2D_BUILD_SMOKE=41` (ADR-0145).
//!
//! Módulo irmão do [`crate::build_smoke`] pelo teto de LOC (HR-18), como os `*_smoke` vizinhos.
//!
//! ⚠️ **Esta cena NÃO arma perfil nenhum**, e é deliberado: o gesto que o smoke tem de provar
//! começa nos SLIDERS do painel. Armar o componente em código pularia exatamente a costura que
//! ele existe para exercer — a cicatriz que o `impasto_smoke` do Painter prega.
//!
//! O que ela dá é **o material**: três traços iguais em geometria e diferentes em nada mais,
//! para o artista arrastar os knobs num e comparar com os vizinhos; e uma forma **com
//! preenchimento E traço**, que é onde o Power Stroke tem duas camadas a produzir (o miolo fica,
//! o traço vira fita) e onde uma regra de camada errada aparece na hora.

use ph2d_vec_scene::{Rgba8, StrokeSpec, VecPath, VecVertex};

/// Metade da largura de um S, em unidades de mundo.
const HALF_W: f64 = 1.5;
/// Amplitude do S.
const AMP: f64 = 0.55;
/// A largura do traço das referências — grossa o bastante para a fita ser legível de longe.
const STROKE_W: f64 = 0.16;

/// Um S aberto, deslocado em `dy`, com traço da cor dada.
fn ess(dy: f64, rgb: [u8; 3]) -> VecPath {
    let verts: Vec<VecVertex> = (0..9)
        .map(|i| {
            let t = f64::from(i) / 8.0;
            VecVertex::corner([
                -HALF_W + 2.0 * HALF_W * t,
                dy + AMP * (t * std::f64::consts::TAU).sin(),
            ])
        })
        .collect();
    let mut p = VecPath {
        verts,
        closed: false,
        ..VecPath::default()
    };
    p.stroke = Some(StrokeSpec::new(
        Rgba8::new(rgb[0], rgb[1], rgb[2], 255),
        STROKE_W,
    ));
    p
}

pub(crate) fn frame(app: &mut crate::App, f: u32) {
    match f {
        3 => build(app),
        4 => announce(app),
        _ => {}
    }
}

fn build(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("vector"));
    let scene = &mut gfx.vec_scene;
    // Três traços IGUAIS: o artista arrasta os knobs num deles e os outros dois ficam de
    // controle — sem controle, "mudou" e "sempre foi assim" são indistinguíveis numa screenshot.
    for (dy, rgb) in [
        (1.4_f64, [70u8, 150, 220]),
        (0.0, [120, 190, 120]),
        (-1.4, [220, 150, 90]),
    ] {
        scene.push_path(ess(dy, rgb));
    }
    // E uma forma com PREENCHIMENTO e traço: aqui o Power Stroke produz DUAS camadas (o miolo
    // continua existindo, sem traço, e a fita entra por cima). É o caso em que a regra de camada
    // aparece — se ela estivesse errada, o preenchimento sumiria no primeiro arrasto.
    let mut disc = ph2d_vec_scene::ellipse([3.4, -1.8], 0.8, 0.8);
    disc.fill = Some(ph2d_vec_scene::Paint::solid(Rgba8::new(210, 210, 235, 255)));
    disc.stroke = Some(StrokeSpec::new(Rgba8::new(40, 40, 60, 255), STROKE_W));
    scene.push_path(disc);
}

fn announce(app: &mut crate::App) {
    let n = app.gfx.as_ref().map_or(0, |g| g.vec_scene.paths().len());
    eprintln!(
        "[smoke] largura viva (ADR-0145): {n} formas -- tres S iguais (azul/verde/laranja) e um \
         DISCO com preenchimento E traco. Nenhum perfil esta' armado: o gesto comeca nos \
         sliders. (1) clique o S do MEIO; (2) na secao **Expand** do painel Vector arraste **W \
         Mid** -- o traco tem de engrossar DESDE o arrasto, com os outros dois S parados como \
         controle; se so' mudar ao clicar o botao, PARE: o perfil nao esta' vivo; (3) arraste \
         **W Start** e **W End** para ~0,25: o traco afina nas duas pontas (a caligrafia); (4) \
         **W Pos** desloca o ponto mais GROSSO ao longo do traco; (5) troque para o modo **Node** \
         -- as ancoras sao as MESMAS de antes (a curva autorada nao mudou; o que se ve e' \
         desenho); (6) clique o S de CIMA (ele nao tem \
         perfil dele proprio), e clicar DE NOVO no do meio tem de trazer os numeros DELE de volta \
         -- sobre uma forma SEM perfil o painel mantem o que voce acabou de escolher, que e' o \
         valor que o proximo arrasto aplicaria (a lei do slider de Offset); (7) \
         selecione o DISCO e engrosse: o preenchimento continua la', e so' o contorno vira \
         fita; (8) com o S do meio selecionado clique **Apply Power Stroke** -- a forma nao pode \
         SALTAR (o que estava na tela e' o que fica) e um Ctrl+Z desfaz tudo num passo; (9) \
         arraste os knobs de volta ao neutro (1,00) num S ainda vivo: a fita some e o traco \
         normal volta -- o perfil neutro nao fica pendurado no documento."
    );
}
