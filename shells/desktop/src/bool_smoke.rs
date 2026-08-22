//! **A cena da BOOLEANA VIVA** — `PH2D_BUILD_SMOKE=48`.
//!
//! Módulo irmão do [`crate::build_smoke`] pelo teto de LOC (HR-18), como os `*_smoke` vizinhos.
//!
//! ⚠️ **Ela dá o MATERIAL e não arma modo nenhum** — a cicatriz que o `impasto_smoke` prega: um
//! smoke que arma o estado por baixo do pano pula justamente a costura que existe para provar.
//! Nenhuma forma nasce agrupada, o modo **Live nasce OFF** (o default do produto), e é o artista
//! quem liga e quem clica.
//!
//! # A pergunta desta cena é UMA, e é de olho
//!
//! *Depois da operação, os operandos continuam lá — e mexer num deles muda o resultado enquanto a
//! mão se move.* Tudo o mais é consequência.
//!
//! O que ela monta, e por quê:
//! - **o PAR** (um quadrado e um círculo que se cruzam) — o material das oito operações;
//! - a **ROSQUINHA e a barra** — o par onde `Subtract` e `Intersect` dão figuras claramente
//!   diferentes, para o re-mirar ser visível num clique;
//! - uma **LINHA ABERTA** sobre o par, o CONTROLE: ela não é operando, e uma booleana à volta
//!   dela não pode fazê-la sumir.

use ph2d_vec_scene::{Paint, Rgba8, VecPath, VecVertex, ellipse, rectangle};

/// Os três rigs, em `x`. Folgados o bastante para um resultado não encostar no vizinho.
const RIG_X: [f64; 3] = [-3.2, 0.4, 3.6];

fn tint(mut p: VecPath, rgb: [u8; 3]) -> VecPath {
    p.fill = Some(Paint::Solid(Rgba8::new(rgb[0], rgb[1], rgb[2], 255)));
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
    let s = &mut gfx.vec_scene;
    // Rig 1 — o PAR: quadrado + círculo que se cruzam.
    s.push_path(tint(
        rectangle([RIG_X[0] - 0.9, 0.6], [RIG_X[0] + 0.5, 2.0]),
        [235, 120, 120],
    ));
    s.push_path(tint(
        ellipse([RIG_X[0] + 0.4, 1.3], 0.8, 0.8),
        [120, 200, 235],
    ));
    // Rig 2 — a ROSQUINHA e a barra: `Subtract` abre uma fenda, `Intersect` deixa dois toquinhos.
    s.push_path(tint(ellipse([RIG_X[1], 1.3], 1.0, 1.0), [235, 200, 120]));
    s.push_path(tint(
        rectangle([RIG_X[1] - 1.4, 1.05], [RIG_X[1] + 1.4, 1.55]),
        [160, 235, 160],
    ));
    // Rig 3 — o CONTROLE: duas formas que se cruzam, e uma LINHA ABERTA por cima delas.
    s.push_path(tint(
        rectangle([RIG_X[2] - 0.8, 0.6], [RIG_X[2] + 0.4, 2.0]),
        [200, 160, 235],
    ));
    s.push_path(tint(
        rectangle([RIG_X[2] - 0.2, 1.1], [RIG_X[2] + 1.0, 2.4]),
        [235, 235, 160],
    ));
    let mut line = VecPath {
        verts: vec![
            VecVertex::corner([RIG_X[2] - 1.2, 0.3]),
            VecVertex::corner([RIG_X[2] + 1.4, 2.7]),
        ],
        closed: false,
        ..VecPath::default()
    };
    line.stroke = Some(ph2d_vec_scene::StrokeSpec::new(
        Rgba8::new(30, 30, 30, 255),
        0.06,
    ));
    s.push_path(line);
}

fn announce(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_ref() else {
        return;
    };
    eprintln!(
        "[bool] cena montada: {} formas — o PAR, a ROSQUINHA+barra, e o rig do CONTROLE (duas \
         formas + uma LINHA ABERTA por cima).",
        gfx.vec_scene.paths().len()
    );
    eprintln!("[bool] o modo Live nasce OFF — o default do produto. Quem liga é você.");
    eprintln!("[bool] o roteiro (pegue a ferramenta VECTOR primeiro):");
    eprintln!("  1. Seção Boolean: a row 'Live'. Deixe em Off, selecione o PAR (Shift+clique) e");
    eprintln!("     clique Union. ⚠️ É o mundo de sempre: os operandos SOMEM, vira uma forma só.");
    eprintln!("     Ctrl+Z para devolvê-los.");
    eprintln!("  2. Agora ligue Live=On, selecione o PAR e clique Subtract. A forma combinada");
    eprintln!("     aparece — e ⚠️ os DOIS operandos continuam na Hierarquia, dentro de um grupo");
    eprintln!("     chamado 'Boolean'.");
    eprintln!("  3. Abra o grupo, selecione o CÍRCULO e ARRASTE. ⚠️ A pergunta da wave: o");
    eprintln!("     recorte segue a mão, quadro a quadro, enquanto você arrasta.");
    eprintln!("  3b. Agora SEM a Hierarquia: clique na forma combinada, no canvas. Clique de");
    eprintln!("     novo no MESMO ponto e a seleção passa ao operando seguinte — inclusive o");
    eprintln!("     que foi COMIDO por um Subtract e não desenha nada. ⚠️ E o contrário tem de");
    eprintln!("     valer: clicar DENTRO do buraco não pega nada, porque ali não há tinta.");
    eprintln!("  4. Com o grupo selecionado, clique Intersect. ⚠️ Ele TROCA a operação — não cria");
    eprintln!("     um segundo grupo e não consome nada. Os oito botões são o seletor.");
    eprintln!("  5. Faça o mesmo na ROSQUINHA+barra: Subtract abre uma fenda, Intersect deixa");
    eprintln!("     dois toquinhos. Alternar entre os dois é instantâneo.");
    eprintln!("  6. No rig do CONTROLE: selecione as DUAS formas e a LINHA, e faça Union viva.");
    eprintln!("     ⚠️ A linha aberta NÃO pode sumir — ela não é operando, e uma booleana não");
    eprintln!("     apaga o que não consumiu.");
    eprintln!("  7. Com um grupo selecionado aparece 'Apply Boolean'. Clique. ⚠️ A arte não pode");
    eprintln!("     mover um pixel: o que estava na tela vira caminho comum, no MESMO z.");
    eprintln!("  8. Ctrl+Z depois de cada passo: criar, re-mirar e consolidar desfazem.");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A cena contém os fenômenos que o roteiro pede.** Sem eles os passos 3, 5 e 6 não provam
    /// nada — e um roteiro que pede o que a geometria não tem engana exactamente quem o corre.
    #[test]
    fn the_fixture_contains_what_the_script_asks_about() {
        let mut s = ph2d_vec_scene::VecScene::new();
        s.push_path(tint(
            rectangle([RIG_X[2] - 0.8, 0.6], [RIG_X[2] + 0.4, 2.0]),
            [1, 2, 3],
        ));
        s.push_path(tint(
            rectangle([RIG_X[2] - 0.2, 1.1], [RIG_X[2] + 1.0, 2.4]),
            [1, 2, 3],
        ));
        // As duas do rig 3 têm de se CRUZAR (senão a união não é uma união).
        let a = ph2d_vec_boolean::area(&s.paths()[0]).abs();
        let b = ph2d_vec_boolean::area(&s.paths()[1]).abs();
        let refs: Vec<&VecPath> = s.paths().iter().collect();
        let u: f64 = ph2d_vec_boolean::pathfinder(&refs, ph2d_vec_boolean::PathfinderOp::Union)
            .unwrap()
            .iter()
            .map(|p| ph2d_vec_boolean::area(p).abs())
            .sum();
        assert!(
            u < a + b - 1e-6,
            "as duas formas do rig do CONTROLE não se cruzam (união {u:.3} = soma {:.3})",
            a + b
        );
    }
}
