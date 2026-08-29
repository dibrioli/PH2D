//! **OS GATES DO PADRÃO NO TRAÇO** (plano 35) — irmão do [`super::pattern_tests`] pelo teto de 700
//! LOC, e o corte é por SUJEITO: ali o padrão é a tinta de uma ÁREA, aqui a de uma FAIXA.
//!
//! ⚠️ E as duas perguntam coisas diferentes: uma área mostra o reticulado inteiro, uma faixa mostra
//! **uma fatia** dele — é por isso que a fase, o modo de repetição e o enquadramento do `Clamp`
//! têm respostas próprias deste lado, e por isso o report de 28/08 (*"o stroke inverte"*) só
//! podia nascer aqui.

use super::pattern_tests::tile;
use super::stroke_uniform::is_conformal;
use crate::build_bezpath;
use ph2d_vec_scene::{StrokePaint, StrokePiece, StrokeSpec};
use ph2d_vector::{Affine, ImageQuality, VectorScene};
use std::borrow::Cow;

// ── O PADRÃO NO TRAÇO — wave B (plano 35) ─────────────────────────────────────────────

/// Uma forma **só com traço** (sem preenchimento), com a tinta que se pedir.
fn so_traco(paint: StrokePaint) -> ph2d_vec_scene::VecScene {
    let mut scene = ph2d_vec_scene::VecScene::default();
    let mut s = StrokeSpec::new(ph2d_vec_scene::Rgba8::new(9, 9, 9, 255), 1.0);
    s.paint = paint;
    scene.push_path(ph2d_vec_scene::VecPath {
        verts: [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]]
            .map(ph2d_vec_scene::VecVertex::corner)
            .to_vec(),
        closed: true,
        stroke: Some(s),
        ..ph2d_vec_scene::VecPath::default()
    });
    scene
}

fn pat_do_traco() -> StrokePaint {
    StrokePaint::Pattern(Box::new(ph2d_vec_scene::PatternFill::new(
        ph2d_vec_scene::PatternSource::Shape(1),
        [4.0, 4.0],
        ph2d_vec_scene::Rgba8::new(200, 30, 30, 255),
    )))
}

fn tile_de_teste() -> crate::PatternTile {
    crate::PatternTile {
        image: tile(),
        cells: [1, 1],
        tile_px: [1, 1],
        quality: ImageQuality::Medium,
    }
}

fn desenha(scene: &ph2d_vec_scene::VecScene, tiles: &crate::PatternTiles) -> VectorScene {
    let mut target = VectorScene::new();
    crate::dispatch(
        scene,
        &ph2d_vec_scene::VecViewState::default(),
        &ph2d_vec_scene::VecXforms::new(),
        &crate::LiveGeometry::new(),
        &crate::FxImages::new(),
        &crate::WidgetSkins::new(),
        tiles,
        Affine::IDENTITY,
        &mut target,
    );
    target
}

/// ⭐⭐ **UM TRAÇO COM PADRÃO DESENHA O LADRILHO** — e sem ladrilho pinta a `fallback`, byte a byte
/// como um traço sólido daquela cor.
///
/// As duas metades são desenho CERTO: a segunda é o que o artista vê enquanto a arte carrega.
/// ⛔ Desenhar NADA seria pior — uma linha invisível não se distingue de uma forma sem contorno.
#[test]
fn a_patterned_stroke_draws_the_tile_and_falls_back_to_the_colour() {
    let com_padrao = so_traco(pat_do_traco());
    let solido = so_traco(StrokePaint::Solid(ph2d_vec_scene::Rgba8::new(
        200, 30, 30, 255,
    )));
    let vazio = crate::PatternTiles::new();

    // 1. Sem ladrilho: BYTE-A-BYTE o encode de um traço sólido da cor de recurso.
    let a = desenha(&com_padrao, &vazio);
    let b = desenha(&solido, &vazio);
    assert!(a.inner().encoding().draw_tags == b.inner().encoding().draw_tags);
    assert_eq!(
        a.inner().encoding().draw_data,
        b.inner().encoding().draw_data
    );

    // 2. Com ladrilho: deixa de o ser.
    let mut tiles = crate::PatternTiles::new();
    let id = com_padrao.paths()[0].id;
    tiles.insert((id, crate::PatternSlot::Stroke), tile_de_teste());
    let c = desenha(&com_padrao, &tiles);
    assert!(
        c.inner().encoding().draw_tags != b.inner().encoding().draw_tags,
        "com ladrilho o traco continuou a encodar um solido"
    );
}

/// ⚠️⚠️ **O ladrilho do TRAÇO não é o do PREENCHIMENTO** — a chave do mapa tem de os separar.
///
/// Uma chave só pela forma entregaria o ladrilho do preenchimento ao traço, e o desenho ficaria
/// certo **por acidente** enquanto os dois fossem iguais.
#[test]
fn the_fill_tile_is_not_handed_to_the_stroke() {
    let scene = so_traco(pat_do_traco());
    let id = scene.paths()[0].id;
    let vazio = crate::PatternTiles::new();
    let base = desenha(&scene, &vazio);

    // Um ladrilho no slot do PREENCHIMENTO não pode mudar o traço.
    let mut errado = crate::PatternTiles::new();
    errado.insert((id, crate::PatternSlot::Fill), tile_de_teste());
    let a = desenha(&scene, &errado);
    assert!(
        a.inner().encoding().draw_tags == base.inner().encoding().draw_tags,
        "o ladrilho do preenchimento vazou para o traco"
    );
    // CONTROLO: no slot certo, ele muda — senão este gate mediria um mapa que nunca é lido.
    let mut certo = crate::PatternTiles::new();
    certo.insert((id, crate::PatternSlot::Stroke), tile_de_teste());
    assert!(
        desenha(&scene, &certo).inner().encoding().draw_tags != base.inner().encoding().draw_tags
    );
}

/// ⭐ **O KILL-CRITERION do plano 35:** um traço com padrão custa o que um traço sólido custa —
/// **zero** camadas de clip.
///
/// ⚠️ O `n_clips` conta **duas** por camada (o `begin` e o `end`), e é por isso que a barra é a
/// IGUALDADE com o sólido, e não um número escrito à mão.
#[test]
fn a_patterned_stroke_pushes_no_clip_layer() {
    let scene = so_traco(pat_do_traco());
    let id = scene.paths()[0].id;
    let mut tiles = crate::PatternTiles::new();
    tiles.insert((id, crate::PatternSlot::Stroke), tile_de_teste());
    let com = desenha(&scene, &tiles);
    let solido = desenha(
        &so_traco(StrokePaint::Solid(ph2d_vec_scene::Rgba8::new(9, 9, 9, 255))),
        &crate::PatternTiles::new(),
    );
    assert_eq!(
        com.inner().encoding().n_clips,
        solido.inner().encoding().n_clips,
        "o padrao no traco empurrou camada - o kill-criterion do plano 35 caiu"
    );
    assert_eq!(
        com.inner().encoding().n_paths,
        solido.inner().encoding().n_paths,
        "o padrao no traco custou mais um desenho que o solido"
    );
}

/// ⚠️⚠️ **O PADRÃO CAI NO MESMO SÍTIO nos dois caminhos do `stroke_uniform`** — e este gate existe
/// porque o segundo caminho é uma armadilha real.
///
/// O Vello compõe `transform * brush_transform`. No caminho rápido (afim conforme) a geometria é
/// local e o afim é o `transform` ⇒ a colocação local chega certa. No caminho não-conforme a
/// geometria **já foi levada à tela** e o afim que chega ao Vello é `IDENTITY` ⇒ sem pré-compor, o
/// padrão ficaria no espaço LOCAL sobre uma geometria de TELA: encolhido no canto do mundo.
///
/// ⭐ A régua é a IGUALDADE dos afins que chegam ao encoding — não uma imagem, não um relógio.
#[test]
fn the_stroke_pattern_lands_in_the_same_place_under_a_non_conformal_affine() {
    let scene = so_traco(pat_do_traco());
    let id = scene.paths()[0].id;
    let mut tiles = crate::PatternTiles::new();
    tiles.insert((id, crate::PatternSlot::Stroke), tile_de_teste());

    let desenhar = |xf: Affine| {
        let mut target = VectorScene::new();
        crate::dispatch(
            &scene,
            &ph2d_vec_scene::VecViewState::default(),
            &ph2d_vec_scene::VecXforms::new(),
            &crate::LiveGeometry::new(),
            &crate::FxImages::new(),
            &crate::WidgetSkins::new(),
            &tiles,
            xf,
            &mut target,
        );
        target.inner().encoding().transforms.clone()
    };
    // ⚠️ A MESMA escala não-uniforme que parte a caneta (bug #27) — é ela que manda o traço pelo
    // caminho lento. O controlo é a versão uniforme do mesmo afim.
    let partido = Affine::scale_non_uniform(3.0, 1.0);
    let conforme = Affine::scale(3.0);
    assert!(
        !is_conformal(partido),
        "a fixtura deixou de conter o fenomeno: este afim ja' e' conforme"
    );
    assert!(is_conformal(conforme));

    // O afim do PINCEL que chega ao Vello tem de ser o mesmo nos dois caminhos, a menos do afim da
    // geometria — que é exactamente o que o caminho lento pré-compõe.
    let a = desenhar(partido);
    let b = desenhar(conforme);
    assert!(
        !a.is_empty() && !b.is_empty(),
        "nenhum dos dois encodou transform nenhum"
    );
    // ⭐ A afirmação forte: o caminho lento **não** deixa a colocação no espaço local. Se deixasse,
    // o afim do pincel seria idêntico ao do caso identidade — e não é.
    let identidade = desenhar(Affine::IDENTITY);
    assert!(
        a != identidade,
        "o caminho nao-conforme deixou a colocacao no espaco LOCAL - o padrao encolhe no canto"
    );
}

/// ⭐⭐ **O PADRÃO NÃO ESCALA COM A LARGURA DO TRAÇO** (gate nº 4 do plano 35 §4) — a queixa que o
/// Illustrator colhe há anos, do lado certo.
///
/// *A largura decide a **faixa**; o padrão decide **o que a preenche**.* São duas grandezas, e
/// juntá-las faria engrossar a linha mudar o motivo debaixo dela.
///
/// ⚠️ **A régua é o afim do PINCEL que chega ao encoding**, e não uma imagem: se a colocação
/// passasse a ler a largura, ele mudaria entre as duas corridas.
///
/// ⚠️⚠️ **O CONTROLE é a metade que importa** — as duas corridas têm de diferir em ALGUMA coisa,
/// senão este gate ficaria verde sobre um produto que ignora a largura por completo (e aí ele não
/// mediria nada).
#[test]
fn the_stroke_pattern_does_not_scale_with_the_stroke_width() {
    let desenhar = |w: f64| {
        let mut scene = ph2d_vec_scene::VecScene::default();
        let mut s = StrokeSpec::new(ph2d_vec_scene::Rgba8::new(9, 9, 9, 255), w);
        s.paint = pat_do_traco();
        let id = scene.push_path(ph2d_vec_scene::VecPath {
            verts: [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]]
                .map(ph2d_vec_scene::VecVertex::corner)
                .to_vec(),
            closed: true,
            stroke: Some(s),
            ..ph2d_vec_scene::VecPath::default()
        });
        let mut tiles = crate::PatternTiles::new();
        tiles.insert((id, crate::PatternSlot::Stroke), tile_de_teste());
        let alvo = desenha(&scene, &tiles);
        let e = alvo.inner().encoding();
        (e.transforms.clone(), e.styles.clone())
    };
    let (xf_fino, estilo_fino) = desenhar(0.5);
    let (xf_grosso, estilo_grosso) = desenhar(4.0);
    assert_eq!(
        xf_fino, xf_grosso,
        "o afim do PINCEL mudou com a largura - engrossar a linha mexe no motivo (a queixa do \
         Illustrator, do lado errado)"
    );
    // CONTROLE: a largura CHEGOU ao desenho. Sem esta metade, o gate acima ficaria verde sobre um
    // produto que nunca lê a largura — e aí ele não estaria a medir nada.
    assert_ne!(
        estilo_fino, estilo_grosso,
        "as duas larguras encodaram o MESMO estilo - a fixtura nao contem o fenomeno"
    );
}

/// ⭐ **SONDA do report de 2026-08-28** (*"ao mudar a posição dos nós ... o stroke inverte"*).
///
/// Números EXACTOS da cena de smoke (`texture_pattern_smoke`): `BOX = 2,2`, ladrilho `BOX/6`,
/// faixa `ladrilho * 1,2`, e o modo `Mirror` que a forma 9 usa no traço. Imprime a colocação e a
/// contagem de peças ANTES e DEPOIS de mover um nó.
#[test]
#[ignore = "sonda: imprime, nao afirma"]
fn measure_what_a_node_move_does_to_the_stroke_pattern() {
    const BOX: f64 = 2.2;
    let lado = BOX / 6.0;
    let cor = ph2d_vec_scene::Rgba8::new(40, 40, 55, 255);
    let mk = |v3: [f64; 2]| {
        let mut scene = ph2d_vec_scene::VecScene::default();
        let mut f = ph2d_vec_scene::PatternFill::new(
            ph2d_vec_scene::PatternSource::Shape(1),
            [lado, lado],
            cor,
        );
        f.mode = ph2d_vec_pattern::PatternMode::Mirror;
        f.origin = [0.0, 0.0];
        let mut s = ph2d_vec_scene::StrokeSpec::new(cor, lado * 1.2);
        s.paint = ph2d_vec_scene::StrokePaint::Pattern(Box::new(f));
        let id = scene.push_path(ph2d_vec_scene::VecPath {
            verts: [[0.0, 0.0], [BOX, 0.0], v3, [0.0, BOX]]
                .map(ph2d_vec_scene::VecVertex::corner)
                .to_vec(),
            closed: true,
            stroke: Some(s),
            ..ph2d_vec_scene::VecPath::default()
        });
        (scene, id)
    };
    for (rotulo, v3) in [
        ("nos originais", [BOX, BOX]),
        ("no movido", [BOX * 1.6, BOX * 0.8]),
    ] {
        let (scene, id) = mk(v3);
        let path = &scene.paths()[0];
        let pat = path
            .stroke
            .as_ref()
            .and_then(ph2d_vec_scene::StrokeSpec::pattern)
            .expect("padrao no traco");
        let t = tile_de_teste();
        let bp = build_bezpath(path);
        let b = ph2d_vector::Shape::bounding_box(&bp);
        let place = pat.placement_in(t.cells, t.tile_px, ([b.x0, b.y0], [b.x1, b.y1]));
        let pecas: Vec<&'static str> =
            ph2d_vec_scene::stroke_plan(path, path.stroke.as_ref().unwrap())
                .into_iter()
                .map(|p| match p {
                    StrokePiece::Line {
                        path: Cow::Borrowed(_),
                    } => "Line(emprestada)",
                    StrokePiece::Line {
                        path: Cow::Owned(_),
                    } => "Line(propria)",
                    StrokePiece::Symbol { .. } => "Symbol",
                    StrokePiece::Fill { .. } => "Fill",
                })
                .collect();
        let mut tiles = crate::PatternTiles::new();
        tiles.insert((id, crate::PatternSlot::Stroke), tile_de_teste());
        let alvo = desenha(&scene, &tiles);
        let e = alvo.inner().encoding();
        println!(
            "[{rotulo}] bbox=({:.3},{:.3})..({:.3},{:.3})  placement={place:?}\n  \
             tile_px={:?} cells={:?}  UM ladrilho cobre {:.4} x {:.4} unidades de mundo\n  \
             pecas={pecas:?}  n_paths={} n_clips={} transforms={}",
            b.x0,
            b.y0,
            b.x1,
            b.y1,
            t.tile_px,
            t.cells,
            place[0].hypot(place[1]) * f64::from(t.tile_px[0]),
            place[2].hypot(place[3]) * f64::from(t.tile_px[1]),
            e.n_paths,
            e.n_clips,
            e.transforms.len(),
        );
    }
}

/// ⛔⛔ **O RAMO DO PADRÃO TEM DE DESENHAR AS MESMAS PEÇAS QUE O SÓLIDO** — só a tinta muda.
///
/// ⚠️ O `stroke_plan` distingue um marcador **cheio** (`StrokePiece::Fill`) de um **aberto**
/// (`StrokePiece::Symbol`, um contorno que se TRAÇA). A wave B tratou os dois como preenchimento:
/// uma seta aberta virava um losango **maciço** assim que o traço ganhava padrão.
///
/// ⭐ **A régua é a IGUALDADE com o ramo sólido**, e não uma contagem escrita à mão: os dois têm de
/// listar os mesmos estilos (traçar × preencher), porque a decisão de *o que* desenhar é do
/// `stroke_plan` e não de quem pinta. *Uma segunda cópia da lista de peças diverge no 1.º ajuste.*
#[test]
fn the_pattern_branch_draws_the_same_pieces_as_the_solid_one() {
    let com_seta = |paint: StrokePaint| {
        let mut scene = ph2d_vec_scene::VecScene::default();
        let mut s = StrokeSpec::new(ph2d_vec_scene::Rgba8::new(9, 9, 9, 255), 0.4);
        s.paint = paint;
        // ⚠️ Um marcador ABERTO nas duas pontas — é ele que produz o `StrokePiece::Symbol`.
        s.marker_start = ph2d_vec_scene::Marker::DiamondOpen;
        s.marker_end = ph2d_vec_scene::Marker::DiamondOpen;
        let id = scene.push_path(ph2d_vec_scene::VecPath {
            verts: [[0.0, 0.0], [4.0, 0.0], [4.0, 3.0]]
                .map(ph2d_vec_scene::VecVertex::corner)
                .to_vec(),
            closed: false,
            stroke: Some(s),
            ..ph2d_vec_scene::VecPath::default()
        });
        (scene, id)
    };
    let (cena_pat, id) = com_seta(pat_do_traco());
    let mut tiles = crate::PatternTiles::new();
    tiles.insert((id, crate::PatternSlot::Stroke), tile_de_teste());
    let com = desenha(&cena_pat, &tiles);
    let (cena_sol, _) = com_seta(StrokePaint::Solid(ph2d_vec_scene::Rgba8::new(9, 9, 9, 255)));
    let solido = desenha(&cena_sol, &crate::PatternTiles::new());

    // CONTROLO: a fixtura contém o fenómeno — há mais de uma peça, senão o gate não mede nada.
    assert!(
        com.inner().encoding().n_paths >= 3,
        "a fixtura nao produziu as pecas de marcador ({} paths)",
        com.inner().encoding().n_paths
    );
    assert_eq!(
        com.inner().encoding().n_paths,
        solido.inner().encoding().n_paths,
        "o ramo do padrao desenha um numero DIFERENTE de pecas que o solido"
    );
    assert_eq!(
        com.inner().encoding().styles,
        solido.inner().encoding().styles,
        "o ramo do padrao TRATOU uma peca de outra maneira - uma seta ABERTA sai macica assim que \
         o traco ganha padrao"
    );
}

/// ⭐ **SONDA: o PINCEL que chega ao encoding é o mesmo num preenchimento e num traço?**
///
/// Mesmo ladrilho, mesma colocação, mesmo modo — só muda quem pinta. Se os `draw_data` diferirem, o
/// amostrador recebe outra lei (outro `Extend`, outra alfa, outro afim de pincel) e é aí que mora
/// o *"um ladrilho e transparente em volta"* da foto de 28/08.
#[test]
#[ignore = "sonda: imprime, nao afirma"]
fn measure_the_brush_a_stroke_gets_against_the_one_a_fill_gets() {
    let quadrado = {
        let mut s = ph2d_vec_scene::VecScene::default();
        s.push_path(ph2d_vec_scene::VecPath {
            verts: [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]]
                .map(ph2d_vec_scene::VecVertex::corner)
                .to_vec(),
            closed: true,
            ..ph2d_vec_scene::VecPath::default()
        });
        s
    };
    let bp = build_bezpath(&quadrado.paths()[0]);
    let place = Affine::new([2.0, 0.0, 0.0, 2.0, 0.5, 0.5]);
    let img = tile();
    for modo in [
        ph2d_vec_pattern::PatternMode::Tile,
        ph2d_vec_pattern::PatternMode::Mirror,
        ph2d_vec_pattern::PatternMode::Clamp,
    ] {
        let e = crate::pattern::extend_of(modo);
        let mut f = VectorScene::new();
        f.fill_path_image(
            &bp,
            ph2d_vector::Fill::NonZero,
            Affine::IDENTITY,
            &img,
            place,
            e,
            e,
            ImageQuality::Medium,
            1.0,
        );
        let mut t = VectorScene::new();
        t.stroke_path_image(
            &bp,
            &ph2d_vector::Stroke::new(1.0),
            Affine::IDENTITY,
            &img,
            place,
            e,
            e,
            ImageQuality::Medium,
            1.0,
        );
        let (ef, et) = (f.inner().encoding(), t.inner().encoding());
        println!(
            "  {modo:?} -> {e:?}\n    fill  : draw_data={:?}\n    stroke: draw_data={:?}\n    \
             IGUAIS? tags={} data={}  (n_tags fill={} stroke={})",
            ef.draw_data,
            et.draw_data,
            ef.draw_tags.len() == et.draw_tags.len(),
            ef.draw_data == et.draw_data,
            ef.draw_tags.len(),
            et.draw_tags.len(),
        );
    }
}

/// ⭐⭐ **UM TRAÇO TRACEJADO COM PADRÃO CONTINUA A PINTAR O PADRÃO** — e custa o mesmo.
///
/// ⚠️⚠️ **Este gate nasce de quatro rondas de report** (Enio, 2026-08-28): o contorno da forma dele
/// estava **tracejado** (`dash=Some((0.88, 1.61))`, `cap=Round`), e cada traço cheio de arte lê-se
/// como *"bolhas"*. Com um contorno fino e sólido um tracejado é uma linha pontilhada discreta; com
/// uma faixa larga e estampada, é a aparência inteira.
///
/// ⭐ **E as duas queixas que pareciam impossíveis eram a mesma coisa:** o `StrokeSpec` guarda o
/// tracejado em **MÚLTIPLOS DA LARGURA** (daí *"depende do width"*), e a `dash_fit` **reajusta-o ao
/// comprimento do caminho** para a emenda fechar (daí *"não é consistente, pode ou não aparecer
/// para o mesmo width"* — mover um nó muda o comprimento).
///
/// ⛔ *Nada disto é defeito*, e é por isso que existe um gate: para a próxima janela não voltar a
/// caçar o padrão quando o que mudou foi o traço.
#[test]
fn a_dashed_patterned_stroke_still_paints_the_pattern() {
    let dashed = |paint: StrokePaint| {
        let mut scene = ph2d_vec_scene::VecScene::default();
        let mut s = StrokeSpec::new(ph2d_vec_scene::Rgba8::new(9, 9, 9, 255), 0.22);
        s.paint = paint;
        // Os números MEDIDOS na sessão do Enio — a fixtura contém o fenómeno, e não uma aproximação.
        s.dash = Some((0.881_610_572_338_104_2, 1.605_468_75));
        s.cap = ph2d_vec_scene::LineCap::Round;
        let id = scene.push_path(ph2d_vec_scene::VecPath {
            verts: [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]]
                .map(ph2d_vec_scene::VecVertex::corner)
                .to_vec(),
            closed: true,
            stroke: Some(s),
            ..ph2d_vec_scene::VecPath::default()
        });
        (scene, id)
    };
    let (cena, id) = dashed(pat_do_traco());
    let mut tiles = crate::PatternTiles::new();
    tiles.insert((id, crate::PatternSlot::Stroke), tile_de_teste());
    let com = desenha(&cena, &tiles);
    // ⭐ A `fallback` do padrão, num traço SÓLIDO tracejado igual: é o desenho que sairia se o
    // ladrilho fosse ignorado.
    let (cena_sol, _) = dashed(StrokePaint::Solid(ph2d_vec_scene::Rgba8::new(
        200, 30, 30, 255,
    )));
    let sem = desenha(&cena_sol, &crate::PatternTiles::new());

    assert_ne!(
        com.inner().encoding().draw_data,
        sem.inner().encoding().draw_data,
        "o traco tracejado pintou a cor de recurso - o tracejado engoliu o padrao"
    );
    // ⚠️ E **o preço não muda**: o tracejado é da CANETA (a kurbo aplica-o), não uma peça por
    // traço. Se este número subir, alguém passou a emitir um desenho por dash.
    assert_eq!(
        com.inner().encoding().n_paths,
        sem.inner().encoding().n_paths,
        "o padrao num traco tracejado passou a custar mais desenhos que a cor"
    );
    assert_eq!(
        com.inner().encoding().n_clips,
        0,
        "o traco tracejado com padrao empurrou camada"
    );
}
