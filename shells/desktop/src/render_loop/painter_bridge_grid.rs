//! **A grade do Grid Stamp na tela.**
//!
//! O método carimba no centro da célula de uma rede própria; sem desenhá-la, a regra de encaixe é uma
//! coisa que o artista tem de adivinhar pelo resultado. Irmão de `draw_symmetry_overlay`, com a mesma
//! armação (canvas + seleção + o afim da sprite) e o mesmo traço tracejado em px de TELA.
//!
//! ⚠️ **As linhas vêm da porta do carimbo** (`PainterTool::grid_line_run` → `BrushSpec::grid_line_run`),
//! nunca de uma re-derivação aqui. Um overlay que calculasse a própria rede desenharia linhas que a
//! tinta não respeita — e um desenho parece autoritativo, então o artista acreditaria nele em vez do
//! carimbo. O gate que ancora as duas é `the_drawn_line_is_the_boundary_the_stamp_lands_between`.
//!
//! ⚠️ **O piso de legibilidade é o teto de custo, e não é um `MAX_LINES` inventado.** Duas linhas mais
//! próximas do que alguns pixels de TELA não são uma grade: são um cinza. Então a pergunta é feita em
//! espaço de tela — *esta célula ainda se vê?* — e a resposta cobra o número de traços por tabela:
//! numa janela de 1600 px, `MIN_CELL_SCREEN_PX = 3` limita cada eixo a ~533 linhas, e é o zoom que
//! decide, como em todo DCC. Um teto em contagem faria a grade sumir por um motivo que o artista não
//! consegue ver (quantas linhas há) em vez de um que ele vê (quão perto elas estão).

use ph2d_ecs::SimWorld;
use ph2d_editor::HeroScreen;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_tool_painter::PainterTool;
use ph2d_vector::VectorScene;

/// Abaixo disto duas linhas vizinhas deixam de se distinguir na tela — o traço tem 1 px e o tracejado
/// precisa de folga entre os dois. Medido contra o tracejado que este overlay usa (`4 on / 4 off`).
const MIN_CELL_SCREEN_PX: f64 = 3.0;

/// O comprimento de um traço do tracejado, em px de TELA. Ele é a unidade das DUAS canetas: uma
/// desenha em `[0, DASH)` e a outra em `[DASH, 2·DASH)`, então o par cobre a linha inteira.
const DASH_PX: f64 = 4.0; // LITERAL-PX-OK: tracejado em px de tela

/// **As DUAS canetas do guia** — `(cor, traço)` em fases complementares.
///
/// ⚠️ **A grade se adapta ao fundo por CONSTRUÇÃO, não por amostragem** (Enio, 2026-08-09: *"a cor do
/// grid é quase invisível em fundo branco … precisa se adaptar ao fundo"*). O guia era UMA caneta
/// azul-clara a 42% de alfa: sobre a arte escura ela lia, sobre uma folha branca ela desaparecia — e
/// uma grade que só se vê em metade dos documentos é uma regra que o artista tem de adivinhar
/// justamente onde ela governa.
///
/// A cura é o tracejado de **dois tons** (o *marching ants* de Photoshop / GIMP / Inkscape): o mesmo
/// padrão desenhado duas vezes, um escuro onde o outro tem vão. Toda posição da linha é coberta por
/// EXATAMENTE um dos dois, então existe contraste sobre qualquer fundo — branco, preto ou a arte no
/// meio — e a resposta não depende de nada que possa estar errado.
///
/// ⛔ **A alternativa — ler os pixels sob a linha e escolher o tom — foi recusada com motivo:** ela
/// decide POR LINHA o que varia AO LONGO dela (uma vertical atravessa céu e sombra), custa uma leitura
/// do composite por quadro, e **pisca** quando o artista pinta debaixo da grade. Um guia que muda de
/// cor porque a tinta mudou lê como defeito.
///
/// ⚠️ **Preço, em aritmética e não em promessa:** cada linha passa a custar DOIS traços. O pior caso
/// da varredura do `no_axis_ever_draws_more_lines_than_the_window_can_resolve` é **1281 linhas num
/// eixo**, logo **2562 traços** — o piso de legibilidade continua sendo o teto, e é ele que impede o
/// número de crescer com o zoom ou com a célula.
fn guide_pens() -> [(ph2d_vector::Color, ph2d_vector::Stroke); 2] {
    use ph2d_vector::{Color, Stroke};
    [
        // O tom ESCURO: é ele que aparece na folha branca, o caso reportado.
        (
            Color::new([0.10, 0.11, 0.14, 0.55]), // LITERAL-COLOR-OK: guia discreto da grade
            Stroke::new(1.0).with_dashes(0.0, [DASH_PX, DASH_PX]),
        ),
        // O tom CLARO, na fase oposta: o guia que já existia, agora ocupando os vãos do outro.
        (
            Color::new([0.88, 0.92, 1.00, 0.55]), // LITERAL-COLOR-OK: guia discreto da grade
            Stroke::new(1.0).with_dashes(DASH_PX, [DASH_PX, DASH_PX]),
        ),
    ]
}

/// Esta célula ainda se VÊ? `step_px` é o lado da célula em px de imagem, `px_scale` quantos px de
/// tela vale um px de imagem.
///
/// Extraída para ser gateável: a propriedade que ela garante — *o número de traços por eixo é limitado
/// pela LARGURA DA JANELA em px de tela ÷ o piso, seja qual for o zoom ou o tamanho da célula* — é o
/// que faz do piso um limite de recurso de verdade em vez de uma esperança.
fn axis_is_legible(step_px: f32, px_scale: f64) -> bool {
    f64::from(step_px) * px_scale >= MIN_CELL_SCREEN_PX
}

/// Desenha a rede do Grid Stamp sobre a sprite selecionada. No-op fora do método, com a grade
/// desligada, sem seleção, sem canvas, ou quando a célula ficou pequena demais para se ver.
pub(super) fn draw_grid_overlay(
    painter: &PainterTool,
    hero: &HeroScreen,
    sim: &SimWorld,
    camera: &Camera2d,
    window_size: WindowSize,
    vector_scene: &mut VectorScene,
) {
    // As DUAS metades: o método tem de ser o da grade **e** o artista tem de a querer vendo. Desenhá-la
    // noutro método mostraria uma regra que não governa nada — o mesmo controle morto que o painel
    // recusa ao não pintar as rows.
    if !painter.stamps_on_a_grid() || !painter.grid_show() {
        return;
    }
    let Some(bits) = hero.gizmo.selection else {
        return;
    };
    let (iw, ih) = painter.canvas_size();
    if iw == 0 || ih == 0 {
        return;
    }
    let entity = ph2d_ecs::Entity::from_bits(bits);
    let (Some(tr), Some(sprite)) = (
        ph2d_ecs::world_transform(sim.world(), entity),
        sim.world().get::<ph2d_render::Sprite>(entity),
    ) else {
        return;
    };
    // A grelha desta sprite (ADR-0164 F1 passo 6) — ausente = uma célula, e aí o quad do
    // afim é o de sempre, byte-idêntico.
    let sprite_grid = sim.world().get::<ph2d_ecs::SpriteGrid>(entity).copied();
    let affine = super::bgremoval_preview::sprite_image_to_screen_affine(
        iw,
        ih,
        tr,
        sprite,
        sprite_grid,
        camera,
        window_size,
    );
    // Escala de tela por pixel de imagem — a mesma leitura que a tolerância de agarre usa
    // (`|coluna linear 0|`), então "quão grande é isto na tela" tem uma resposta só no shell.
    let c = affine.as_coeffs();
    let px_scale = (c[0] * c[0] + c[1] * c[1]).sqrt();
    if px_scale <= 0.0 {
        return;
    }

    use ph2d_vector::{Affine, BezPath, Brush, Point};
    let map = |x: f64, y: f64| affine * Point::new(x, y);
    let scene = vector_scene.inner_mut();
    // A JANELA, em px de imagem — as quatro quinas da tela levadas de volta pelo inverso do afim, e a
    // caixa delas recortada ao canvas.
    //
    // ⚠️ Sem isto o piso de legibilidade **não limita nada** com zoom alto, e o gate irmão foi quem
    // disse: um canvas de 2048 com célula de 1 px a zoom 4 desenhava **2049 traços**, quase todos fora
    // da tela, porque a rede era percorrida sobre o CANVAS inteiro. Recortada à janela, o número volta
    // a ser o que o piso promete — *quantas linhas cabem na tela*, e não quantas o canvas tem.
    let inv = affine.inverse();
    let (win_w, win_h) = (f64::from(window_size.width), f64::from(window_size.height));
    let corners = [(0.0, 0.0), (win_w, 0.0), (0.0, win_h), (win_w, win_h)]
        .map(|(x, y)| inv * Point::new(x, y));
    let view = |sel: fn(&Point) -> f64, extent: f64| {
        let lo = corners.iter().map(sel).fold(f64::INFINITY, f64::min);
        let hi = corners.iter().map(sel).fold(f64::NEG_INFINITY, f64::max);
        (lo.max(0.0), hi.min(extent))
    };
    // Guia discreto, tracejado em px de TELA (o caminho já vem mapeado e é traçado sob IDENTITY), então
    // o tracejado lê igual em qualquer zoom — a convenção do overlay de simetria ao lado. As DUAS
    // canetas do par de dois tons vêm da porta única (`guide_pens`), que é quem explica o porquê.
    let pens = guide_pens();
    let (w, h) = (f64::from(iw), f64::from(ih));

    for axis in 0..2 {
        let extent = if axis == 0 { w } else { h };
        let (lo, hi) = if axis == 0 {
            view(|p| p.x, extent)
        } else {
            view(|p| p.y, extent)
        };
        let (first, step, n) = painter.grid_line_run(axis, lo as f32, hi as f32);
        // O piso é POR EIXO: uma célula 400×4 continua desenhando as verticais e cala as horizontais,
        // que é a leitura honesta — metade da rede ainda se vê.
        if !axis_is_legible(step, px_scale) {
            continue;
        }
        for k in 0..n {
            let t = f64::from(first) + f64::from(step) * f64::from(k);
            let mut path = BezPath::new();
            if axis == 0 {
                path.move_to(map(t, 0.0));
                path.line_to(map(t, h));
            } else {
                path.move_to(map(0.0, t));
                path.line_to(map(w, t));
            }
            for (color, dash) in &pens {
                scene.stroke(dash, Affine::IDENTITY, &Brush::Solid(*color), None, &path);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_painter_brush::BrushSpec;

    fn spec(cell: f32) -> BrushSpec {
        BrushSpec {
            grid_cell_px: [cell, cell],
            ..BrushSpec::default()
        }
    }

    /// Quantos traços um eixo desenha de fato. **Espelha o laço do produto pela MESMA porta** — uma
    /// contagem escrita à mão aqui mediria a aritmética do teste, não a do overlay — incluindo o
    /// recorte à JANELA, que é o que faz do piso um limite (sem ele o produto percorria o canvas
    /// inteiro e desenhava milhares de linhas fora da tela).
    fn lines_drawn(cell: f32, canvas: u32, px_scale: f64, window_px: f64) -> u32 {
        let s = spec(cell);
        // A janela em px de IMAGEM, recortada ao canvas — o `view()` do produto, sem rotação.
        let hi = (window_px / px_scale).min(f64::from(canvas));
        let (_, step, n) = s.grid_line_run(0, 0.0, hi as f32);
        if axis_is_legible(step, px_scale) {
            n
        } else {
            0
        }
    }

    /// **O piso de legibilidade É o teto de custo.**
    ///
    /// Com a célula no piso de 1 px de imagem e uma tela de 4096, a rede ingênua tem **8192 traços por
    /// quadro** — e o artista não veria grade nenhuma, porque linhas a menos de dois ou três pixels de
    /// tela são um cinza. O limite não é um `MAX_LINES` inventado: é a pergunta que o desenho já
    /// obriga a fazer, feita em espaço de TELA. A propriedade que o torna um limite de verdade é que o
    /// número de traços passa a ser limitado pela **janela ÷ o piso**, e por mais nada — nem pela
    /// célula, nem pelo zoom, nem pelo canvas.
    ///
    /// **Mutação que deve sangrar:** apagar o `axis_is_legible` do laço (desenhar sempre).
    #[test]
    fn no_axis_ever_draws_more_lines_than_the_window_can_resolve() {
        let window_px = 3840.0_f64;
        let ceiling = (window_px / MIN_CELL_SCREEN_PX).ceil() as u32;
        let mut worst = 0;
        for canvas in [1024u32, 2048, 4096] {
            let fit = window_px / f64::from(canvas);
            for zoom in [0.05, 0.25, fit, 1.0, 4.0] {
                for cell in [1.0f32, 2.0, 4.0, 8.0, 32.0, 128.0, 512.0] {
                    let n = lines_drawn(cell, canvas, zoom, window_px);
                    assert!(
                        n <= ceiling,
                        "célula {cell} px, canvas {canvas}, zoom {zoom}: {n} traços contra o teto \
                         {ceiling} — o piso de legibilidade deixou de limitar o custo"
                    );
                    worst = worst.max(n);
                }
            }
        }
        assert!(
            worst > 0,
            "a varredura não desenhou linha nenhuma — a fixture não contém o fenômeno"
        );
        // O número MEDIDO, ao lado do limite (§0): não o teto, mas o pior caso que a varredura de
        // fato alcança — é ele que diz quanto a grade custa no dia mais caro.
        assert!(
            worst <= 1281,
            "o pior caso da varredura era 1281 traços num eixo e passou a {worst} — o número mudou e \
             a nota ao lado do limite deixou de valer"
        );
    }

    /// A outra metade: **a grade que se vê É desenhada.** Um piso alto demais calaria a rede
    /// justamente onde ela serve, e passaria pelo gate do teto sem ruído.
    ///
    /// **Mutação que deve sangrar:** subir `MIN_CELL_SCREEN_PX` para 40.
    #[test]
    fn a_cell_the_artist_can_see_is_drawn() {
        assert!(
            lines_drawn(32.0, 1024, 1.0, 1600.0) > 0,
            "uma célula de 32 px a 1:1 não desenhou nada — o piso está calando a grade útil"
        );
        assert!(
            lines_drawn(128.0, 2048, 0.1, 1600.0) > 0,
            "uma célula de 128 px a 10% de zoom são 12,8 px de tela — legível, e mesmo assim calada"
        );
    }

    /// **O laço do produto usa as duas peças que os gates acima medem.**
    ///
    /// ⚠️ O `lines_drawn` deste módulo ESPELHA o laço em vez de o chamar (o laço precisa de janela,
    /// sprite e câmera), então sozinho ele não vê o produto deixar de recortar ou de perguntar o piso —
    /// mediria a própria aritmética, verde para sempre. Este gate fecha o vão lendo a fonte, e começa
    /// por um **controle positivo**: sem a âncora do laço ele falha alto em vez de varrer o nada.
    ///
    /// **Mutações que devem sangrar:** trocar `if !axis_is_legible(..)` por `if false`; trocar
    /// `grid_line_run(axis, lo as f32, hi as f32)` por `(axis, 0.0, extent as f32)`.
    #[test]
    fn the_drawing_loop_clips_to_the_window_and_asks_the_floor() {
        // ⚠️ SÓ a metade do PRODUTO. O arquivo se inclui a si mesmo, então um `contains` sobre ele
        // inteiro casa com as agulhas escritas AQUI, nos próprios asserts — e o gate passa a olhar
        // para si em vez de para o laço. Foi o que ele fez: a mutação `if false` sobreviveu, verde,
        // porque a string continuava existindo dentro deste teste. (A mesma doença que o gate do
        // `stamps: media` já pagou, por outra via: *um oráculo que casa com a documentação de si
        // mesmo não está olhando para o produto*.)
        const FILE: &str = include_str!("painter_bridge_grid.rs");
        let src = FILE
            .split_once("#[cfg(test)]")
            .expect("controle positivo: o módulo de testes sumiu — o corte deixaria de existir")
            .0;
        assert!(
            src.contains("for axis in 0..2 {"),
            "controle positivo: o laço por eixo mudou de forma — este gate estaria varrendo o nada"
        );
        assert!(
            src.contains("painter.grid_line_run(axis, lo as f32, hi as f32)"),
            "o laço deixou de recortar à janela: com zoom alto ele volta a percorrer o canvas inteiro \
             e o piso de legibilidade deixa de limitar o custo"
        );
        assert!(
            src.contains("if !axis_is_legible(step, px_scale) {"),
            "o laço deixou de perguntar o piso — a grade vira um cinza e cobra milhares de traços"
        );
    }

    /// ⚠️ **O guia contrasta com QUALQUER fundo, e a propriedade é o que o gate afirma** — o report do
    /// Enio de 2026-08-09 (*"a cor do grid é quase invisível em fundo branco"*).
    ///
    /// Duas metades, e nenhuma sozinha basta: **(a)** as duas luminâncias ficam em lados OPOSTOS do
    /// cinza médio, senão as duas somem juntas no mesmo fundo (era literalmente o mundo anterior — uma
    /// caneta só, clara); **(b)** as fases são COMPLEMENTARES sobre o mesmo padrão, senão o tom escuro
    /// pinta em cima do claro e a linha volta a ter um tom só, com o segundo traço custando por nada.
    ///
    /// **Mutações que devem sangrar:** pôr os dois tons claros (mata (a)); dar a mesma fase às duas
    /// canetas (mata (b)).
    #[test]
    fn the_guide_has_a_tone_for_a_light_ground_and_one_for_a_dark_ground() {
        let [(dark, sd), (light, sl)] = guide_pens();
        // Luminância perceptual (Rec.709) — a régua de *"isto se vê sobre o quê?"*.
        let lum = |c: ph2d_vector::Color| {
            let [r, g, b, _] = c.components;
            0.2126 * r + 0.7152 * g + 0.0722 * b
        };
        let (ld, ll) = (lum(dark), lum(light));
        assert!(
            ld < 0.35 && ll > 0.65,
            "os dois tons do guia são {ld:.3} e {ll:.3} — eles têm de ficar em lados opostos do \
             cinza médio, senão existe um fundo que apaga os DOIS"
        );
        assert!(
            dark.components[3] > 0.3 && light.components[3] > 0.3,
            "um tom quase transparente não contrasta com nada: alfas {} e {}",
            dark.components[3],
            light.components[3]
        );
        // (b) O mesmo padrão em fases complementares: juntos cobrem o período inteiro, sem sobrepor.
        assert_eq!(
            sd.dash_pattern.as_slice(),
            sl.dash_pattern.as_slice(),
            "as duas canetas têm de compartilhar o padrão — com padrões diferentes 'complementar' \
             deixa de querer dizer alguma coisa"
        );
        assert_eq!(
            sd.dash_pattern.as_slice(),
            [DASH_PX, DASH_PX],
            "o padrão é meio-a-meio: é o que faz de UMA fase deslocada o complemento da outra"
        );
        assert!(
            (sl.dash_offset - sd.dash_offset - DASH_PX).abs() < 1e-9,
            "as fases são {} e {} — elas têm de diferir por exatamente um traço, senão os dois tons \
             pintam em cima um do outro e o segundo traço custa por nada",
            sd.dash_offset,
            sl.dash_offset
        );
    }

    /// **O laço desenha as DUAS canetas.** As cores certas num par que só é metade usado não
    /// contrastam com nada — e um gate sobre `guide_pens` sozinho fica verde exatamente aí.
    ///
    /// ⚠️ Lê só a metade do PRODUTO do arquivo, pela mesma razão do gate irmão (o arquivo se inclui a
    /// si mesmo, e um `contains` sobre ele inteiro casaria com a agulha escrita aqui).
    ///
    /// **Mutação que deve sangrar:** trocar o laço por um `scene.stroke` de uma caneta só.
    #[test]
    fn the_drawing_loop_strokes_both_pens() {
        const FILE: &str = include_str!("painter_bridge_grid.rs");
        let src = FILE
            .split_once("#[cfg(test)]")
            .expect("controle positivo: o módulo de testes sumiu — o corte deixaria de existir")
            .0;
        assert!(
            src.contains("let pens = guide_pens();"),
            "controle positivo: o laço deixou de pedir o par à porta — este gate estaria varrendo o nada"
        );
        assert!(
            src.contains("for (color, dash) in &pens {"),
            "o laço deixou de percorrer as duas canetas: o guia volta a ter um tom só, e some no \
             fundo que aquele tom não enfrenta"
        );
    }

    /// O piso é **por EIXO**: uma célula 400×4 continua desenhando as verticais e cala as
    /// horizontais. Metade da rede visível é a leitura honesta — calar as duas perderia informação
    /// que se vê.
    #[test]
    fn the_floor_is_per_axis_not_per_grid() {
        let s = BrushSpec {
            grid_cell_px: [400.0, 1.0],
            ..BrushSpec::default()
        };
        let (_, wide, _) = s.grid_line_run(0, 0.0, 2048.0);
        let (_, thin, _) = s.grid_line_run(1, 0.0, 2048.0);
        assert!(
            axis_is_legible(wide, 1.0),
            "o eixo largo (400 px) devia continuar desenhando"
        );
        assert!(
            !axis_is_legible(thin, 1.0),
            "o eixo fino (1 px a 1:1) não se vê e devia calar"
        );
    }
}
