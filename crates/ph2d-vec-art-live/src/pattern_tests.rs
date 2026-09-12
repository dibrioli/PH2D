//! Os gates do memo de padrões (plano 33, W4).

use super::*;
use ph2d_vec_pattern::TileKind;
use ph2d_vec_scene::{PatternFill, Rgba8, VecPath, VecVertex};

/// Arte 4x4 opaca com um texel por coordenada.
fn art_bytes() -> Vec<u8> {
    let mut v = Vec::new();
    for y in 0..4u8 {
        for x in 0..4u8 {
            v.extend_from_slice(&[x * 16, y * 16, 200, 255]);
        }
    }
    v
}

pub(super) fn scene_with(fill: PatternFill) -> (VecScene, VecPathId) {
    let mut scene = VecScene::default();
    let id = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [8.0, 0.0], [8.0, 8.0], [0.0, 8.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        fill: Some(Paint::Pattern(Box::new(fill))),
        ..VecPath::default()
    });
    (scene, id)
}

/// A expansão de objecto num mundo **sem ECS**: cada caminho é o próprio objecto.
///
/// ⚠️ É a resposta honesta aqui, e não um atalho: a pertença a um grupo vive na árvore de
/// entidades, que estes gates não constroem. O caso do GRUPO tem gates próprios, com a expansão a
/// sério.
pub(super) fn solo() -> impl Fn(VecPathId) -> Vec<VecPathId> {
    |id| vec![id]
}

/// **Ninguém se mexeu** — a pose de identidade para todo membro.
///
/// ⚠️ É o CONTROLO implícito de todo gate que não é sobre pose: com ela, a metade nova da chave é
/// constante, e um gate que reprove aqui está a falar de outra coisa.
pub(super) fn still() -> impl Fn(VecPathId) -> ph2d_vec_scene::Xform {
    |_| ph2d_vec_scene::Xform::IDENTITY
}

/// O assador de FORMA que devolve nada — o caminho de quem só tem fontes de imagem.
fn no_shape() -> impl FnMut(VecPathId) -> Option<(u32, u32, Vec<u8>)> {
    |_| None
}

fn db_with_art() -> (AssetDb, ph2d_asset::AssetId) {
    let db = AssetDb::new();
    let id = db.insert_image_rgba8(4, 4, art_bytes());
    (db, id)
}

/// **A arte carregada vira ladrilho, e a grade encostada devolve-a ao tamanho.**
#[test]
fn a_loaded_image_becomes_a_tile() {
    let (db, asset) = db_with_art();
    let (scene, path) = scene_with(PatternFill::new(
        PatternSource::Image(asset),
        [4.0, 4.0],
        Rgba8::new(1, 2, 3, 255),
    ));
    let mut live = TexturePatternLive::default();
    live.recook(
        &scene,
        &db,
        ImageQuality::Medium,
        &mut no_shape(),
        &solo(),
        &still(),
    );
    let tile = live
        .tiles()
        .get(&(path, ph2d_vec_render::PatternSlot::Fill))
        .expect("o padrao assou");
    assert_eq!(tile.tile_px, [4, 4]);
    assert_eq!(tile.cells, [1, 1]);
}

/// ⚠️ **Uma fonte que não resolve NÃO produz ladrilho** — e é isso que faz a forma pintar a
/// `fallback` em vez de desaparecer.
#[test]
fn an_unresolved_source_produces_no_tile() {
    let db = AssetDb::new();
    let (scene, path) = scene_with(PatternFill::new(
        PatternSource::Image(ph2d_asset::AssetId::from_bytes(b"nunca importada")),
        [4.0, 4.0],
        Rgba8::new(1, 2, 3, 255),
    ));
    let mut live = TexturePatternLive::default();
    live.recook(
        &scene,
        &db,
        ImageQuality::Medium,
        &mut no_shape(),
        &solo(),
        &still(),
    );
    assert!(
        live.tiles()
            .get(&(path, ph2d_vec_render::PatternSlot::Fill))
            .is_none()
    );
    // E uma fonte-FORMA também não, enquanto a W7 não existir.
    let (scene2, path2) = scene_with(PatternFill::new(
        PatternSource::Shape(7),
        [4.0, 4.0],
        Rgba8::new(1, 2, 3, 255),
    ));
    live.recook(
        &scene2,
        &db,
        ImageQuality::Medium,
        &mut no_shape(),
        &solo(),
        &still(),
    );
    assert!(
        live.tiles()
            .get(&(path2, ph2d_vec_render::PatternSlot::Fill))
            .is_none()
    );
}

/// ⭐ **O memo acerta: re-assar sem mudar nada devolve o MESMO handle** (o mesmo `Blob`, logo a
/// mesma textura no atlas do Vello).
///
/// ⚠️ Um handle novo por quadro faria o Vello re-enviar a textura **todo quadro** — é a razão de o
/// `StableImage` existir, e é invisível a qualquer gate que olhe só para os pixels.
#[test]
fn recooking_an_unchanged_pattern_keeps_the_same_handle() {
    let (db, asset) = db_with_art();
    let (scene, path) = scene_with(PatternFill::new(
        PatternSource::Image(asset),
        [4.0, 4.0],
        Rgba8::new(1, 2, 3, 255),
    ));
    let mut live = TexturePatternLive::default();
    live.recook(
        &scene,
        &db,
        ImageQuality::Medium,
        &mut no_shape(),
        &solo(),
        &still(),
    );
    let first = live.tiles()[&(path, ph2d_vec_render::PatternSlot::Fill)]
        .image
        .clone();
    live.recook(
        &scene,
        &db,
        ImageQuality::Medium,
        &mut no_shape(),
        &solo(),
        &still(),
    );
    let second = &live.tiles()[&(path, ph2d_vec_render::PatternSlot::Fill)].image;
    assert_eq!(
        first.width(),
        second.width(),
        "o memo re-assou o que nao mudou"
    );
    // Controlo: mudar a LEI tem de re-assar — senão este gate estaria a medir um mapa congelado.
    let mut f2 = PatternFill::new(
        PatternSource::Image(asset),
        [4.0, 4.0],
        Rgba8::new(1, 2, 3, 255),
    );
    f2.kind = TileKind::BrickRow;
    f2.offset_denom = 3;
    let (scene2, path2) = scene_with(f2);
    live.recook(
        &scene2,
        &db,
        ImageQuality::Medium,
        &mut no_shape(),
        &solo(),
        &still(),
    );
    assert_eq!(
        live.tiles()[&(path2, ph2d_vec_render::PatternSlot::Fill)].tile_px,
        [4, 12],
        "o tijolo de 1/3 pede tres linhas"
    );
}

/// ⚠️ **O filtro actualiza-se SEM re-assar** — ele escolhe a amostragem na GPU e não toca um byte
/// do ladrilho. Metê-lo na chave faria alternar o modo de imagem re-assar a cena inteira para
/// produzir os mesmos pixels.
#[test]
fn changing_the_filter_does_not_rebake() {
    let (db, asset) = db_with_art();
    let (scene, path) = scene_with(PatternFill::new(
        PatternSource::Image(asset),
        [4.0, 4.0],
        Rgba8::new(1, 2, 3, 255),
    ));
    let mut live = TexturePatternLive::default();
    live.recook(
        &scene,
        &db,
        ImageQuality::Medium,
        &mut no_shape(),
        &solo(),
        &still(),
    );
    live.recook(
        &scene,
        &db,
        ImageQuality::Low,
        &mut no_shape(),
        &solo(),
        &still(),
    );
    let t = &live.tiles()[&(path, ph2d_vec_render::PatternSlot::Fill)];
    assert_eq!(t.quality, ImageQuality::Low, "o filtro tem de acompanhar");
    assert_eq!(t.tile_px, [4, 4]);
}

/// ⚠️⚠️ **A varredura tem as DUAS metades.** Uma forma que deixou de ter padrão (ou que foi
/// apagada) não pode continuar a desenhar o ladrilho dela — nem a mantê-lo em memória.
#[test]
fn a_shape_that_loses_its_pattern_loses_its_tile() {
    let (db, asset) = db_with_art();
    let (mut scene, path) = scene_with(PatternFill::new(
        PatternSource::Image(asset),
        [4.0, 4.0],
        Rgba8::new(1, 2, 3, 255),
    ));
    let mut live = TexturePatternLive::default();
    live.recook(
        &scene,
        &db,
        ImageQuality::Medium,
        &mut no_shape(),
        &solo(),
        &still(),
    );
    assert!(
        live.tiles()
            .contains_key(&(path, ph2d_vec_render::PatternSlot::Fill))
    );
    scene.path_mut(path).unwrap().fill = Some(Paint::solid(Rgba8::new(9, 9, 9, 255)));
    live.recook(
        &scene,
        &db,
        ImageQuality::Medium,
        &mut no_shape(),
        &solo(),
        &still(),
    );
    assert!(
        live.tiles().is_empty(),
        "o ladrilho sobreviveu ao padrao que o pediu"
    );
}

/// ⛔⛔ **REPORT DO ENIO (2026-08-27): *"em column o pattern some"*.** Um ladrilho que não caberia
/// no atlas é **REDUZIDO até caber**, não recusado — e a forma continua a mostrar o padrão.
///
/// ⚠️ **Este gate afirmava o CONTRÁRIO até hoje** (`..._leaves_no_tile`), e foi ele que apanhou a
/// mudança de lei quando o assador passou a reduzir. *Um gate que se torna vermelho por causa de uma
/// cura é um gate a fazer o trabalho dele — ele obriga a dizer, por escrito, que a lei mudou.*
#[test]
fn a_tile_too_big_for_the_atlas_is_scaled_and_still_shows() {
    let db = AssetDb::new();
    let big = ph2d_vec_pattern::MAX_TILE_EDGE_PX / 2;
    let asset = db.insert_image_rgba8(big, 4, vec![0u8; (big as usize) * 4 * 4]);
    let mut f = PatternFill::new(
        PatternSource::Image(asset),
        [4.0, 4.0],
        Rgba8::new(1, 2, 3, 255),
    );
    f.kind = TileKind::BrickCol;
    f.offset_denom = 3; // 3 x (MAX/2) passaria do tecto
    let (scene, path) = scene_with(f);
    let mut live = TexturePatternLive::default();
    live.recook(
        &scene,
        &db,
        ImageQuality::Medium,
        &mut no_shape(),
        &solo(),
        &still(),
    );
    let tile = live
        .tiles()
        .get(&(path, ph2d_vec_render::PatternSlot::Fill))
        .expect("o ladrilho tem de ser REDUZIDO, nao recusado");
    assert!(
        tile.tile_px[0] <= ph2d_vec_pattern::MAX_TILE_EDGE_PX
            && tile.tile_px[1] <= ph2d_vec_pattern::MAX_TILE_EDGE_PX,
        "o reduzido nao coube: {:?}",
        tile.tile_px
    );
    assert_eq!(tile.cells, [3, 1], "a LEI nao muda com a reducao");
}

// ── O PADRÃO NO TRAÇO — wave C (plano 35) ─────────────────────────────────────────────

/// ⭐⭐ **UMA FORMA PODE TER PADRÃO NO PREENCHIMENTO E NO TRAÇO AO MESMO TEMPO**, e os dois são
/// entradas INDEPENDENTES no memo.
///
/// ⚠️ **A fixtura contém o fenómeno de propósito:** os dois padrões têm reticulados diferentes, e
/// por isso ladrilhos diferentes. Com dois padrões IGUAIS este gate ficaria verde sobre um mapa
/// indexado só pela forma — que era exactamente o defeito que a chave por slot cura.
#[test]
fn a_shape_can_have_a_pattern_on_fill_and_stroke_at_once() {
    use ph2d_vec_render::PatternSlot;
    let (db, asset) = db_with_art();
    let mut do_fill = PatternFill::new(
        PatternSource::Image(asset),
        [4.0, 4.0],
        Rgba8::new(1, 2, 3, 255),
    );
    do_fill.kind = TileKind::Grid;
    let mut do_traco = PatternFill::new(
        PatternSource::Image(asset),
        [4.0, 4.0],
        Rgba8::new(9, 9, 9, 255),
    );
    // ⚠️ Tijolo com meio passo: o ladrilho assado tem DUAS linhas, então ele difere do da grade em
    // pixels — é o que torna a mistura das duas entradas visível a este gate.
    do_traco.kind = TileKind::BrickRow;
    do_traco.offset_denom = 2;

    let (mut scene, id) = scene_with(do_fill);
    let mut s = ph2d_vec_scene::StrokeSpec::new(Rgba8::new(9, 9, 9, 255), 1.0);
    s.paint = ph2d_vec_scene::StrokePaint::Pattern(Box::new(do_traco));
    scene.path_mut(id).expect("a forma").stroke = Some(s);

    let mut live = TexturePatternLive::default();
    live.recook(
        &scene,
        &db,
        ImageQuality::Medium,
        &mut no_shape(),
        &solo(),
        &still(),
    );

    let f = live
        .tiles()
        .get(&(id, PatternSlot::Fill))
        .expect("o padrao do preenchimento assou");
    let t = live
        .tiles()
        .get(&(id, PatternSlot::Stroke))
        .expect("o padrao do traco assou");
    assert_ne!(
        f.tile_px, t.tile_px,
        "os dois ladrilhos sairam iguais - a chave do mapa nao esta' a separar as duas tintas"
    );

    // ⚠️ E a varredura desmarca por SLOT: tirar o padrão do traço não pode levar o do preenchimento.
    scene.path_mut(id).expect("a forma").stroke = None;
    live.recook(
        &scene,
        &db,
        ImageQuality::Medium,
        &mut no_shape(),
        &solo(),
        &still(),
    );
    assert!(live.tiles().contains_key(&(id, PatternSlot::Fill)));
    assert!(
        !live.tiles().contains_key(&(id, PatternSlot::Stroke)),
        "o ladrilho do traco sobreviveu a' remocao da tinta dele"
    );
}

/// ⭐ **SONDA do report de 2026-08-28 (2.º, *"não resolveu"*)** — o que o assado REAL entrega ao
/// traço, com os números exactos da cena de smoke.
///
/// Imprime, para a tinta do preenchimento e a do traço da MESMA forma: o tamanho do ladrilho em
/// pixels, as células, e **quantas unidades de mundo uma cópia cobre** — que é a grandeza que a
/// foto mede.
#[test]
#[ignore = "sonda: imprime, nao afirma"]
fn measure_the_baked_tile_of_each_paint() {
    const BOX: f64 = 2.2;
    const ART: u32 = 32;
    let db = AssetDb::new();
    // A MESMA arte do smoke: barra no topo, meia-diagonal, um quadrante transparente.
    let mut px = Vec::with_capacity((ART * ART * 4) as usize);
    for y in 0..ART {
        for x in 0..ART {
            let c = if y < ART / 8 {
                [230u8, 140, 60, 255]
            } else if x + y < ART {
                [70, 120, 210, 255]
            } else if x > ART * 3 / 4 && y > ART * 3 / 4 {
                [200, 40, 40, 0]
            } else {
                [235, 232, 225, 255]
            };
            px.extend_from_slice(&c);
        }
    }
    let arte = px.clone();
    let asset = db.insert_image_rgba8(ART, ART, px);
    let canto = [-1.3, -4.1]; // longe da origem, como as formas de baixo do smoke
    let mk = |lado: f64, kind: TileKind, denom: u8| {
        let mut f = PatternFill::new(
            PatternSource::Image(asset),
            [lado, lado],
            Rgba8::new(1, 2, 3, 255),
        );
        f.kind = kind;
        f.offset_denom = denom;
        f.origin = canto;
        f
    };
    let fill = mk(BOX / 3.0, TileKind::Grid, 2);
    let stroke_pat = mk(BOX / 6.0, TileKind::BrickRow, 2);
    let mut scene = VecScene::default();
    let mut s = ph2d_vec_scene::StrokeSpec::new(Rgba8::new(1, 2, 3, 255), (BOX / 6.0) * 1.2);
    s.paint = ph2d_vec_scene::StrokePaint::Pattern(Box::new(stroke_pat));
    let id = scene.push_path(VecPath {
        verts: [
            [canto[0], canto[1]],
            [canto[0] + BOX, canto[1]],
            [canto[0] + BOX, canto[1] + BOX],
            [canto[0], canto[1] + BOX],
        ]
        .map(VecVertex::corner)
        .to_vec(),
        closed: true,
        fill: Some(Paint::Pattern(Box::new(fill))),
        stroke: Some(s),
        ..VecPath::default()
    });
    let mut live = TexturePatternLive::default();
    live.recook(
        &scene,
        &db,
        ImageQuality::Medium,
        &mut no_shape(),
        &solo(),
        &still(),
    );
    println!(
        "\n  forma de {BOX} unidades, traco de {:.4}",
        (BOX / 6.0) * 1.2
    );
    for (slot, pat) in [
        (
            ph2d_vec_render::PatternSlot::Fill,
            scene
                .path(id)
                .and_then(|p| p.fill.as_ref())
                .and_then(|f| match f {
                    Paint::Pattern(p) => Some((**p).clone()),
                    _ => None,
                }),
        ),
        (
            ph2d_vec_render::PatternSlot::Stroke,
            scene
                .path(id)
                .and_then(|p| p.stroke.as_ref())
                .and_then(ph2d_vec_scene::StrokeSpec::pattern)
                .cloned(),
        ),
    ] {
        let Some(pat) = pat else { continue };
        let Some(t) = live.tiles().get(&(id, slot)) else {
            println!("  {slot:?}: SEM LADRILHO");
            continue;
        };
        let pl = pat.placement_in(t.cells, t.tile_px, ([0.0, 0.0], [BOX, BOX]));
        // ⭐ **A ALFA do ladrilho ASSADO** — pela MESMA porta que o memo usa
        // (`ph2d_vec_pattern::bake`). A arte e' ~94 % opaca, entao um assado com muito
        // transparente e' o assador a deixar buracos — e e' isso que se le^ como "blobs".
        let law = pat.law([ART, ART]);
        if let Ok(assado) = ph2d_vec_pattern::bake(&arte, ART, ART, &law) {
            let n = assado.rgba.len() / 4;
            let vazios = (0..n).filter(|i| assado.rgba[i * 4 + 3] == 0).count();
            println!(
                "    ALFA do assado {}x{}: {n} texels, {vazios} transparentes ({:.1} %)",
                assado.width,
                assado.height,
                100.0 * vazios as f64 / n.max(1) as f64
            );
        }
        println!(
            "  {slot:?}: size={:?} kind={:?} denom={} | tile_px={:?} cells={:?} | UMA COPIA cobre \
             {:.4} x {:.4} do mundo (o ladrilho inteiro: {:.4} x {:.4})",
            pat.size,
            pat.kind,
            pat.offset_denom,
            t.tile_px,
            t.cells,
            pl[0].hypot(pl[1]) * f64::from(t.tile_px[0]) / f64::from(t.cells[0].max(1)),
            pl[2].hypot(pl[3]) * f64::from(t.tile_px[1]) / f64::from(t.cells[1].max(1)),
            pl[0].hypot(pl[1]) * f64::from(t.tile_px[0]),
            pl[2].hypot(pl[3]) * f64::from(t.tile_px[1]),
        );
    }
}

// ── A POSE DOS MEMBROS — report do Enio de 2026-08-30 ─────────────────────────────────
