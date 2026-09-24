//! **O campo da reserva GUARDADO entre quadros dá o byte do campo REFEITO a cada quadro** (ADR-0173,
//! 3.ª ronda). O controlo é o caminho de antes, ligado por `SEM_CACHE`: o composite refaz o campo na
//! janela de leitura inteira. A sessão tem o que mexe no plano guardado — o Smudge a arrastar os
//! níveis sobre o traço vivo, o pen-up que recompõe o traço inteiro, e quatro traços na MESMA sessão
//! molhada, cada um com outro raio (o plano renasce) e a cruzar tinta anterior numa das quatro
//! direcções (cada faixa nova é o único caminho por onde essa tinta entra no plano).
//!
//! Prova de mutação (2026-09-23), cada uma a sangrar: apagar a faixa de CIMA (`16 320` bytes), a de
//! BAIXO (`21 098`), a da ESQUERDA (`13 078`), a da DIREITA (`24 825`) e o recálculo do SUJO do
//! quadro (`67 189`).

use super::*;
use crate::tool::paint::watercolor_reserve::SEM_CACHE;

/// Um traço recto de `a` a `b`, fechando um quadro a cada dois eventos, SEM pen-up.
fn traco(t: &mut PainterTool, a: [f32; 2], b: [f32; 2]) {
    assert!(t.on_canvas_pointer(cp(a, PointerPhase::Down)));
    let passos = 80;
    for i in 1..=passos {
        let f = i as f32 / passos as f32;
        t.on_canvas_pointer(cp(
            [a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f],
            PointerPhase::Move,
        ));
        if i % 2 == 0 {
            frame(t);
        }
    }
}

fn fecha(t: &mut PainterTool, b: [f32; 2]) {
    t.on_canvas_pointer(cp(b, PointerPhase::Up));
    for _ in 0..4 {
        frame(t);
    }
}

/// Um traço da sessão: onde vai, com que raio e que Rewet (o par muda o raio do campo ⇒ o conjunto
/// de raios da sessão muda ⇒ o plano guardado RENASCE, e as faixas novas passam a ser o único
/// caminho por onde a tinta dos traços ANTERIORES entra nele).
type Traco = ([f32; 2], [f32; 2], f32, f32);

/// ⚠️ **A carga é ALTA de propósito** (`0,95`): a 1.ª redacção corria a `0,4`, a reserva do 1.º traço
/// esgotava-se a um terço do caminho, e o campo VERDADEIRO debaixo de toda faixa velha era `0` — a
/// mutação que apagava a faixa da esquerda SOBREVIVEU porque o zero velho e o zero certo são o mesmo
/// byte. ⚠️ **E a GEOMETRIA também**: a 2.ª redacção tinha três traços em diagonal e as faixas de
/// CIMA e de BAIXO sobreviviam — nenhuma varria tinta anterior. Aqui o 1.º é a base (horizontal) e
/// cada um dos outros CRUZA tinta anterior numa das quatro direcções, com um raio próprio (os cinco
/// `R` são distintos: `24 · 5 · 12 · 18 · 8`), logo cada um RECONSTRÓI o plano e só as faixas trazem
/// a tinta de antes para dentro dele.
const TRACOS: [Traco; 5] = [
    ([40.0, 250.0], [470.0, 250.0], 32.0, 1.0),
    ([180.0, 490.0], [180.0, 30.0], 20.0, 0.2),
    ([330.0, 30.0], [330.0, 490.0], 44.0, 0.5),
    ([470.0, 120.0], [40.0, 120.0], 28.0, 0.75),
    ([40.0, 390.0], [470.0, 390.0], 36.0, 0.35),
];

/// Uma fotografia da tela a meio de cada traço e depois de cada pen-up, e se o plano guardado
/// existia a meio do 1.º.
fn sessao() -> (Vec<Vec<u8>>, bool) {
    let size = 512u32;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    t.paint.brush.color = [0.85, 0.12, 0.10];
    t.apply_brush_preset(1);
    t.paint.brush.wet_charge = 0.95;
    t.paint.brush.wet_smudge = 0.5;
    t.paint.brush.edge_spread = 24.0;
    t.paint.brush.warp = 24.0;
    let mut fotos = Vec::new();
    let mut guardado = false;
    for (k, (a, b, r, rewet)) in TRACOS.into_iter().enumerate() {
        t.paint.brush.radius_px = r;
        t.paint.brush.wet_rewet = rewet;
        let seed = t.paint.brush;
        t.paint.brush_by_mode.fill(seed);
        traco(&mut t, a, b);
        if k == 0 {
            guardado = t.paint.wet_reserve_cache.is_some();
        }
        fotos.push(t.canvas_rgba.to_vec());
        fecha(&mut t, b);
        fotos.push(t.canvas_rgba.to_vec());
    }
    (fotos, guardado)
}

#[test]
fn o_campo_guardado_da_o_byte_do_campo_refeito() {
    let (fotos, guardado) = sessao();
    SEM_CACHE.with(|c| c.set(true));
    let (fotos0, guardado0) = sessao();
    SEM_CACHE.with(|c| c.set(false));
    // CONTROLO: os dois caminhos são de facto dois — senão o gate compara o controlo consigo mesmo.
    assert!(
        guardado,
        "o composite não guardou o campo: o caminho novo nunca correu"
    );
    assert!(
        !guardado0,
        "o controlo guardou o campo: SEM_CACHE não desligou nada"
    );
    // CONTROLO: cada traço mexe na tela, senão comparamos papéis parados.
    for k in 1..TRACOS.len() {
        assert_ne!(
            fotos[2 * k - 1],
            fotos[2 * k + 1],
            "o traço {} não pintou nada",
            k + 1
        );
    }
    for (i, (a, b)) in fotos.iter().zip(&fotos0).enumerate() {
        let dif = a.iter().zip(b.iter()).filter(|(x, y)| x != y).count();
        let quando = if i % 2 == 0 { "vivo" } else { "pen-up" };
        assert_eq!(
            dif,
            0,
            "traço {} ({quando}): {dif} bytes entre o campo guardado e o refeito",
            i / 2 + 1
        );
    }
}
