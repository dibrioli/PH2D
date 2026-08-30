//! **As RÉGUAS deixam de partilhar coordenada com o chrome** — a primeira lei do modelo de
//! áreas (D5, `docs/UI_New_and_Simple/spec/01_modelo_de_areas.md` §4).
//!
//! # O defeito que este portão fixa
//!
//! Até 2026-08-30 a régua era ancorada em `HeroLayout::canvas`, que **é a viewport inteira**
//! (`layout.rs`, `let canvas = Rect::new(viewport.x, viewport.y, viewport.w, viewport.h)`).
//! O trilho de ferramentas nasce em `x = viewport.x` e a barra de topo em `y = viewport.y + 14`,
//! e as duas são pintadas **depois** da régua. ⇒ no viewport de referência (1366 × 1024, que é
//! o iPad Pro 12,9" — o alvo declarado dos tokens) a régua da esquerda ficava **87,8 % tapada** e
//! a de cima **29,4 %** (medição em `docs/UI_New_and_Simple/medicoes/02_a_area_tapada.md`).
//!
//! ⭐ **A cura não é uma verificação — é a ausência de coordenada partilhada.** As réguas passam
//! a ser regiões da [`HeroLayout::draw_area`], que começa depois da coluna da esquerda e acaba
//! antes da da direita. Duas regiões irmãs não se tapam porque não ocupam o mesmo espaço.
//!
//! # ⛔ E há uma segunda metade, que é de INPUT e não se vê
//!
//! A régua **não está no `HitIndex`**: o gesto de guia é geométrico
//! (`ruler::hit(host, p)`) e corre em `input_dispatch.rs` **antes** do hit-test de chrome, com
//! um `return` quando acerta. Enquanto o hospedeiro foi a janela inteira, um press nos
//! **6 px de cima de qualquer botão da barra** (a banda de cima é `y ∈ [0, 20]`, a barra começa
//! em `y = 14`) ou nos **3 px da esquerda de qualquer chip do trilho** (a banda esquerda é
//! `x ∈ [0, 20]`, o chip começa em `x = 17`) **nascia uma guia em vez de carregar no botão** —
//! e nenhum gate do repo media isto, porque todos perguntam pelo `HitIndex`, onde a régua não
//! está.
//!
//! Os dois testes de CONTROLO abaixo reproduzem os dois defeitos com a âncora antiga. Sem eles
//! a lei podia passar por a função de medida devolver zero por engano — *um zero de «não medido»
//! e um de «perfeito» são o mesmo byte*.

use ph2d_editor_core::ruler;
use ph2d_editor_core::screens::layout::{
    DockSides, HERO_VIEWPORT_H, HERO_VIEWPORT_W, HeroLayout, LEFT_DOCK_PANELS, rail_w,
};
use ph2d_editor_core::zones::Rect;

fn reference_viewport() -> Rect {
    Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H)
}

/// Área da intersecção de dois rects (0 quando não se tocam).
fn overlap_area(a: Rect, b: Rect) -> f32 {
    let w = (a.x + a.w).min(b.x + b.w) - a.x.max(b.x);
    let h = (a.y + a.h).min(b.y + b.h) - a.y.max(b.y);
    if w <= 0.0 || h <= 0.0 { 0.0 } else { w * h }
}

/// O chrome **docado** deste layout — o que ocupa faixa fixa, e por isso o que uma região
/// irmã nunca pode tocar. ⚠️ Os painéis só entram quando a coluna deles está aberta: um
/// painel fechado não é pintado, e reservar-lhe espaço seria o defeito simétrico.
fn docked_chrome(l: &HeroLayout, docks: DockSides, _mirrored: bool) -> Vec<(&'static str, Rect)> {
    let mut v = vec![
        ("top_bar", l.top_bar),
        ("left_rail", l.left_rail),
        ("bottom_hud", l.bottom_hud),
        // ⭐⭐ A coluna do TAKEOVER entra SEMPRE, e nao atras de um flag. A 1a versao deste
        // oraculo perguntava `if docks.right`, isto e', partilhava com o produto a premissa que
        // devia estar sob julgamento — e quando a lista de inquilinos estava errada, o painel
        // saia da exclusao E da acusacao ao mesmo tempo, e o gate devolvia 0.0 por nao olhar.
        // `l.inspector` E' o rect da coluna, esteja la' quem estiver.
        ("takeover_column", l.inspector),
    ];
    if docks.hierarchy_open {
        v.push(("hierarchy", l.hierarchy));
    }
    v
}

fn all_dock_states() -> [DockSides; 2] {
    [DockSides::BOTH, DockSides::NONE]
}

/// **A LEI.** Nas duas orientações e nos quatro estados de coluna, nenhuma das duas faixas de
/// régua partilha um único pixel com chrome docado.
#[test]
fn the_rulers_never_share_a_pixel_with_docked_chrome() {
    for mirrored in [false, true] {
        for docks in all_dock_states() {
            let l = HeroLayout::for_viewport_docked(
                reference_viewport(),
                mirrored,
                rail_w(),
                ph2d_editor_core::screens::layout::CenterSplit::None,
                docks,
            );
            assert!(
                l.draw_area.w > 0.0 && l.draw_area.h > 0.0,
                "a area de desenho colapsou (mirrored={mirrored}, docks={docks:?}): {:?}",
                l.draw_area
            );
            for (band_name, band) in [
                ("top", ruler::top_band(l.draw_area)),
                ("left", ruler::left_band(l.draw_area)),
            ] {
                for (chrome_name, rect) in docked_chrome(&l, docks, mirrored) {
                    let a = overlap_area(band, rect);
                    assert_eq!(
                        a, 0.0,
                        "a regua '{band_name}' {band:?} partilha {a} px2 com '{chrome_name}' \
                         {rect:?} (mirrored={mirrored}, docks={docks:?})"
                    );
                }
            }
        }
    }
}

/// **CONTROLO nº 1 — a medida vê uma sobreposição quando ela existe.**
///
/// Com a âncora antiga (`layout.canvas`, a viewport inteira) a régua da esquerda é comida pelo
/// trilho e a de cima pela barra. Os números vêm da medição de 2026-08-30 e são reproduzidos
/// aqui a partir do próprio layout: se um dia a geometria do chrome mudar, este controlo
/// muda de número mas **não pode ir a zero** — se fosse, o teste da lei acima estaria a
/// afirmar o vazio.
#[test]
fn the_control_the_old_anchor_was_covered_and_the_measure_sees_it() {
    let l = HeroLayout::for_viewport_docked(
        reference_viewport(),
        false,
        rail_w(),
        ph2d_editor_core::screens::layout::CenterSplit::None,
        DockSides::BOTH,
    );
    let old_left = ruler::left_band(l.canvas);
    let old_top = ruler::top_band(l.canvas);

    let left_covered = overlap_area(old_left, l.left_rail)
        + overlap_area(old_left, l.top_bar)
        + overlap_area(old_left, l.bottom_hud);
    let top_covered = overlap_area(old_top, l.top_bar);

    let left_frac = left_covered / old_left.area();
    let top_frac = top_covered / old_top.area();

    assert!(
        left_frac > 0.8,
        "a ancora antiga da regua esquerda deixou de estar tapada ({left_frac:.3}) - \
         ou a geometria do chrome mudou, ou este controlo deixou de medir o que dizia"
    );
    assert!(
        top_frac > 0.2,
        "a ancora antiga da regua de cima deixou de estar tapada ({top_frac:.3})"
    );
    // E a mesma medida, sobre a area de desenho, da' zero — a cura, lado a lado com o defeito.
    assert_eq!(
        overlap_area(ruler::left_band(l.draw_area), l.left_rail),
        0.0,
        "a regua esquerda ancorada na area de desenho ainda toca o trilho"
    );
}

/// **CONTROLO nº 2 — o roubo de CLIQUE, que nenhuma sonda do repo via.**
///
/// O gesto da guia é geométrico e corre antes do hit-test de chrome. Com a âncora antiga, o
/// topo de um botão da barra e a esquerda de um chip do trilho respondiam «régua». Com a área
/// de desenho, respondem `None` — e o botão volta a ser um botão.
#[test]
fn the_ruler_no_longer_steals_the_click_from_the_top_bar_and_the_rail() {
    let l = HeroLayout::for_viewport_docked(
        reference_viewport(),
        false,
        rail_w(),
        ph2d_editor_core::screens::layout::CenterSplit::None,
        DockSides::BOTH,
    );
    // Um ponto na metade de CIMA da barra de topo, e um na coluna esquerda de um chip do
    // trilho — os dois dentro de chrome que o artista quer carregar.
    let in_top_bar = (l.top_bar.x + l.top_bar.w * 0.5, l.top_bar.y + 1.0);
    let in_rail = (l.left_rail.x + 18.0, l.left_rail.y + 10.0);

    assert!(
        ruler::hit(l.canvas, in_top_bar).is_some(),
        "o controlo falhou: com a ancora antiga o topo da barra TINHA de responder regua"
    );
    assert!(
        ruler::hit(l.canvas, in_rail).is_some(),
        "o controlo falhou: com a ancora antiga a esquerda do chip TINHA de responder regua"
    );

    assert!(
        ruler::hit(l.draw_area, in_top_bar).is_none(),
        "a regua continua a roubar o clique da barra de topo"
    );
    assert!(
        ruler::hit(l.draw_area, in_rail).is_none(),
        "a regua continua a roubar o clique do trilho"
    );
}

/// **CENSO — a coluna da ESQUERDA tem UM inquilino, e é isso que sustenta a
/// [`LEFT_DOCK_PANELS`].**
///
/// ⛔⛔ Este gate substitui um que estava ERRADO por construção (auditoria de 2026-08-30). O
/// anterior varria o `layout.rs` por `let X = inspector;` e concluía que a coluna da direita
/// tinha **cinco** inquilinos. Ela tem **dezassete**: os outros doze não têm alias nenhum no
/// `layout.rs` — eles lêem `ctx.layout.inspector` directamente, de **outra crate**, e o painel
/// **Vector** é um deles. O censo tinha a *forma* de uma conferência de dois lados e media um
/// subconjunto que não era a pergunta.
///
/// ⇒ a coluna da direita deixou de ter lista (é sempre reservada, ver [`DockSides`]) e o que
/// ficou por defender é a da esquerda, cuja lista **de facto** tem um nome. Este censo pergunta
/// ao que o produto faz: *que crates de painel desenham no rect de cada coluna?*
#[test]
fn the_left_dock_column_still_has_exactly_one_tenant() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/");
    let mut left: Vec<String> = Vec::new();
    let mut right: Vec<String> = Vec::new();
    for entry in std::fs::read_dir(root).expect("ler crates/") {
        let dir = entry.expect("entry").path();
        let Some(name) = dir.file_name().and_then(|n| n.to_str()).map(str::to_owned) else {
            continue;
        };
        if !name.starts_with("ph2d-panel-") {
            continue;
        }
        let src = dir.join("src");
        let mut takes_left = false;
        let mut takes_right = false;
        let mut stack = vec![src];
        while let Some(d) = stack.pop() {
            let Ok(rd) = std::fs::read_dir(&d) else {
                continue;
            };
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    stack.push(p);
                    continue;
                }
                if p.extension().and_then(|x| x.to_str()) != Some("rs") {
                    continue;
                }
                let Ok(txt) = std::fs::read_to_string(&p) else {
                    continue;
                };
                for raw in txt.lines() {
                    let l = raw.trim_start();
                    if l.starts_with("//") {
                        continue; // ⚠️ descascar comentarios: documentar a cura nao pode acusar
                    }
                    // O rect INTEIRO da coluna, nunca uma dimensao dela (`.inspector.w` e' o
                    // que a widget-gallery le' para casar a largura, e nao ocupa a coluna).
                    if l.contains("layout.hierarchy;") {
                        takes_left = true;
                    }
                    if l.contains("layout.inspector;") || l.contains("layout.padding;") {
                        takes_right = true;
                    }
                }
            }
        }
        if takes_left {
            left.push(name.clone());
        }
        if takes_right {
            right.push(name);
        }
    }
    left.sort();
    right.sort();

    assert!(
        !left.is_empty() && !right.is_empty(),
        "controlo positivo falhou: a varredura nao achou inquilino nenhum em nenhuma coluna \
         (esquerda={left:?}, direita={right:?}) — a forma do codigo mudou e este censo deixou \
         de medir o que diz"
    );
    assert_eq!(
        left,
        vec!["ph2d-panel-hierarchy".to_string()],
        "a coluna da ESQUERDA ganhou um segundo inquilino. A LEFT_DOCK_PANELS ({:?}) passa a \
         ter o MESMO defeito que matou a lista da direita: um painel que a tome sem estar la' \
         faz a area de desenho crescer para dentro dele. Ou acrescente a chave, ou torne a \
         coluna sempre reservada, como a da direita.",
        LEFT_DOCK_PANELS
    );
    assert!(
        right.len() > 1,
        "a coluna da DIREITA passou a ter um inquilino so' ({right:?}). O motivo de ela ser \
         SEMPRE reservada era ser um slot de takeover multi-inquilino; com um dono unico, \
         vale a pena voltar a perguntar pela visibilidade dele"
    );
}

/// **A porta pergunta pela HIERARCHY, e a coluna do takeover não tem estado.**
///
/// ⚠️ Sem este teste, trocar o ramo de [`DockSides::resolve`] deixa a suite verde: a lei
/// geométrica constrói os `DockSides` à mão e nunca passa por aqui.
#[test]
fn the_dock_sides_ask_about_the_hierarchy_and_the_takeover_column_has_no_state() {
    assert_eq!(
        DockSides::resolve(|k| k == "hierarchy"),
        DockSides::BOTH,
        "com a Hierarchy aberta a coluna dela e' reservada"
    );
    assert_eq!(
        DockSides::resolve(|_| false),
        DockSides::NONE,
        "com a Hierarchy fechada a area reclama a coluna dela"
    );
    // ⭐ E a coluna do takeover NAO responde a ninguem: qualquer inquilino dela — o Inspector,
    // o Vector, o Physics… — deixa a resposta igual. E' o teorema do doc de `DockSides`.
    for tenant in ["inspector", "vector", "physics", "sculpt3d", "audio_editor"] {
        assert_eq!(
            DockSides::resolve(|k| k == tenant),
            DockSides::NONE,
            "'{tenant}' vive na coluna do takeover, que nao tem estado — e a Hierarchy esta' \
             fechada neste caso"
        );
    }
}

/// ⭐⭐⭐ **O TEOREMA que autoriza reservar sempre a coluna da direita, gateado no fonte.**
///
/// *Régua viva ⇒ painel Vector visível ⇒ coluna da direita ocupada.* Se uma das duas metades
/// deixar de valer, reservar a coluna sempre passa a ser um palpite conservador em vez de uma
/// dedução — e o custo (a régua acabar 318 px antes da borda) deixa de ser zero.
#[test]
fn the_rulers_only_live_while_the_takeover_column_is_occupied() {
    const OFFERS: &str = include_str!("../src/screens/hero/offers.rs");
    assert!(
        OFFERS.contains("self.view.rulers_visible && self.is_panel_visible(\"vector\")"),
        "a porta `rulers_live` mudou de condicao. O teorema que autoriza reservar sempre a \
         coluna da direita e' 'regua viva => painel Vector visivel => coluna ocupada'; se a \
         regua passar a viver noutro modo, releia o doc de `DockSides` antes de shipar"
    );
    let vector_paint = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/")
        .join("ph2d-panel-vector/src/paint.rs");
    let src = std::fs::read_to_string(&vector_paint).expect("ph2d-panel-vector/src/paint.rs");
    assert!(
        src.contains("ctx.layout.inspector;"),
        "o painel Vector deixou de desenhar no rect da coluna da direita — a segunda metade do \
         teorema caiu"
    );
}

/// **Uma área estreita demais não pinta régua NEM responde a uma** — a porta única
/// [`ruler::live_bands`].
///
/// ⛔ Achado da auditoria de 2026-08-30: o desenho tinha a guarda `<= RULER_PX` e o hit-test
/// não. A wave das áreas tornou a faixa alcançável (deixou de exigir uma janela de 20 px e
/// passou a exigir uma de ~735 px de largura). *Visível ⇔ vivo*, e o inverso — responder sem
/// aparecer — é a forma pior.
#[test]
fn a_band_too_narrow_to_paint_is_also_too_narrow_to_answer() {
    let px = ph2d_editor_core::ruler::RULER_PX;
    for (w, h) in [
        (0.0, 500.0),
        (px - 1.0, 500.0),
        (px, 500.0),
        (500.0, px),
        (500.0, px - 1.0),
    ] {
        let r = Rect::new(10.0, 20.0, w, h);
        assert!(
            ruler::live_bands(r).is_none(),
            "uma area {w}x{h} nao comporta regua (RULER_PX={px}) e mesmo assim ofereceu faixas"
        );
        // O ponto no canto superior esquerdo, que e' onde a faixa nasceria.
        assert!(
            ruler::hit(r, (r.x + 1.0, r.y + 1.0)).is_none(),
            "a area {w}x{h} nao pinta regua e mesmo assim RESPONDE a uma - chrome morto sob o \
             dedo, ao contrario"
        );
    }
    // Controlo: logo acima do limiar as duas metades acordam JUNTAS.
    let ok = Rect::new(10.0, 20.0, px + 1.0, px + 1.0);
    assert!(ruler::live_bands(ok).is_some());
    assert!(ruler::hit(ok, (ok.x + 1.0, ok.y + 1.0)).is_some());
}

/// **Uma faixa docada no fundo não corre por baixo da régua** — o `timeline` nasce exactamente
/// no `area_x0` e partilhava 20 × 240 px² com a régua da esquerda.
#[test]
fn a_bottom_dock_takes_the_height_it_occupies_from_the_drawing_area() {
    let mut l = HeroLayout::for_viewport_docked(
        reference_viewport(),
        false,
        rail_w(),
        ph2d_editor_core::screens::layout::CenterSplit::None,
        DockSides::BOTH,
    );
    let before = overlap_area(ruler::left_band(l.draw_area), l.timeline);
    assert!(
        before > 0.0,
        "controlo: sem reserva, a regua esquerda TINHA de partilhar pixels com o dock do \
         timeline (partilhou {before})"
    );
    l.reserve_bottom_strip(l.timeline);
    assert_eq!(
        overlap_area(ruler::left_band(l.draw_area), l.timeline),
        0.0,
        "a regua esquerda continua a correr por baixo do dock do timeline"
    );
    assert!(l.draw_area.h > 0.0, "a area colapsou ao reservar a faixa");
    // Idempotente, e inerte para uma faixa vazia.
    let h = l.draw_area.h;
    l.reserve_bottom_strip(l.timeline);
    l.reserve_bottom_strip(Rect::new(0.0, 0.0, 0.0, 0.0));
    assert_eq!(l.draw_area.h, h);
}

/// **A reserva da faixa de fundo está FIADA** — a lei geométrica acima chama
/// `reserve_bottom_strip` à mão, então sem este gate apagar a chamada do produto deixa a suite
/// inteira verde e a régua volta a correr por baixo do timeline.
///
/// ⚠️ E a ORDEM é load-bearing: tem de vir **depois** do `dock_timeline_into_motion`, que MOVE
/// o rect do timeline — reservar antes reservaria o sítio errado.
#[test]
fn the_bottom_strip_reservation_is_wired_and_runs_after_the_motion_dock() {
    const HERO_PAINT: &str = include_str!("../src/screens/hero/paint.rs");
    let dock = HERO_PAINT
        .find("layout.dock_timeline_into_motion();")
        .expect("o timeline docado no Motion");
    let timeline = HERO_PAINT
        .find("layout.reserve_bottom_strip(layout.timeline);")
        .expect(
            "o dock do timeline nao e' reservado: a regua da esquerda volta a correr por baixo \
             dele (20 x 240 px2 no viewport de referencia)",
        );
    let flip = HERO_PAINT
        .find("layout.reserve_bottom_strip(layout.flip_strip);")
        .expect("a tira do Flip nao e' reservada — o irmao do timeline");
    assert!(
        timeline > dock && flip > dock,
        "a reserva corre ANTES do `dock_timeline_into_motion`, que move o rect do timeline — \
         reservaria o sitio errado"
    );
}
