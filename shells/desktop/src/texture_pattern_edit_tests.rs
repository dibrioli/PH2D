//! Os gates da autoria do padrão (plano 33, W5).

use super::*;
use ph2d_vec_scene::{Rgba8, VecPath, VecPathId, VecVertex};

fn scene_with(f: PatternFill) -> (VecScene, ph2d_vec_edit::PenTool, VecPathId) {
    let mut scene = VecScene::default();
    let id = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        fill: Some(Paint::Pattern(Box::new(f))),
        ..VecPath::default()
    });
    let mut pen = ph2d_vec_edit::PenTool::default();
    pen.select_many(&[id]);
    (scene, pen, id)
}

fn fill() -> PatternFill {
    // ⚠️ Arte NÃO quadrada de propósito: um `size` `[8, 2]` é o único que mostra se o aspecto
    // sobreviveu a um Size novo. Com `[4, 4]` toda aritmética errada passa.
    let mut f = PatternFill::new(
        PatternSource::Shape(1),
        [8.0, 2.0],
        Rgba8::new(1, 2, 3, 255),
    );
    f.kind = TileKind::Grid;
    f
}

fn pattern_of(scene: &VecScene, id: VecPathId) -> PatternFill {
    match scene.path(id).and_then(|p| p.fill.as_ref()) {
        Some(Paint::Pattern(p)) => (**p).clone(),
        _ => panic!("a forma deixou de ter padrao"),
    }
}

/// ⭐ **COM o cadeado, mexer num eixo preserva a razão** — a protecção de sempre, agora como gesto.
#[test]
fn with_the_lock_an_axis_keeps_the_ratio() {
    let (mut scene, pen, id) = scene_with(fill()); // size [8, 2] — 4:1
    let mut h = ph2d_vec_edit::History::default();
    apply(
        &mut scene,
        &mut h,
        &pen,
        PatternSlot::Fill,
        TexPatCmd::Axis(0, 4.0, true),
    );
    let p = pattern_of(&scene, id);
    assert_eq!(p.size, [4.0, 1.0], "o cadeado nao preservou a razao 4:1");
}

/// ⭐⭐ **SEM o cadeado, a arte ACHATA — e é isto que o Enio pediu** (2026-08-27).
#[test]
fn without_the_lock_the_art_squashes_on_purpose() {
    let (mut scene, pen, id) = scene_with(fill()); // [8, 2]
    let mut h = ph2d_vec_edit::History::default();
    apply(
        &mut scene,
        &mut h,
        &pen,
        PatternSlot::Fill,
        TexPatCmd::Axis(1, 8.0, false),
    );
    let p = pattern_of(&scene, id);
    assert_eq!(p.size, [8.0, 8.0], "o outro eixo mexeu-se, ou este nao");
    assert_eq!(h.undo_len(), 1, "achatar e' UM passo de undo");
    // ⚠️ E o MESMO valor não grava passo — o slider re-publica a cada quadro em que está agarrado.
    apply(
        &mut scene,
        &mut h,
        &pen,
        PatternSlot::Fill,
        TexPatCmd::Axis(1, 8.0, false),
    );
    assert_eq!(h.undo_len(), 1, "o mesmo valor gravou um passo espurio");
}

/// **Cada mudança é UM passo de undo, e um valor repetido NÃO é passo nenhum.**
///
/// ⚠️ Sem a comparação, o slider a re-publicar o mesmo número faria todo quadro virar um passo — o
/// defeito que o `canonicalize` do editor curou para o mundo inteiro.
#[test]
fn a_repeated_value_records_no_undo_step() {
    let (mut scene, pen, _) = scene_with(fill());
    let mut h = ph2d_vec_edit::History::default();
    apply(
        &mut scene,
        &mut h,
        &pen,
        PatternSlot::Fill,
        TexPatCmd::Angle(30.0),
    );
    let after_first = h.undo_len();
    apply(
        &mut scene,
        &mut h,
        &pen,
        PatternSlot::Fill,
        TexPatCmd::Angle(30.0),
    );
    assert_eq!(
        h.undo_len(),
        after_first,
        "o mesmo valor gravou um passo espurio"
    );
    apply(
        &mut scene,
        &mut h,
        &pen,
        PatternSlot::Fill,
        TexPatCmd::Angle(31.0),
    );
    assert_eq!(h.undo_len(), after_first + 1, "um valor NOVO tem de gravar");
}

/// **Os índices do painel e os enums do documento são a MESMA lista, nos dois sentidos.**
///
/// ⚠️ Uma tradução escrita só num sentido é onde um chip passa a acender no reticulado errado — e
/// isso lê-se como *"o painel mostra Brick e o desenho é Hex"*, que é um report sem causa aparente.
#[test]
fn the_panel_indices_round_trip_through_the_document_enums() {
    for k in [
        TileKind::Grid,
        TileKind::BrickRow,
        TileKind::BrickCol,
        TileKind::Hex,
    ] {
        let (mut scene, pen, id) = scene_with(fill());
        let mut h = ph2d_vec_edit::History::default();
        apply(
            &mut scene,
            &mut h,
            &pen,
            PatternSlot::Fill,
            TexPatCmd::Tile(tile_index(k)),
        );
        assert_eq!(
            pattern_of(&scene, id).kind,
            k,
            "ida e volta partiu em {k:?}"
        );
    }
    for m in [PatternMode::Tile, PatternMode::Mirror, PatternMode::Clamp] {
        let (mut scene, pen, id) = scene_with(fill());
        let mut h = ph2d_vec_edit::History::default();
        apply(
            &mut scene,
            &mut h,
            &pen,
            PatternSlot::Fill,
            TexPatCmd::Mode(mode_index(m)),
        );
        assert_eq!(
            pattern_of(&scene, id).mode,
            m,
            "ida e volta partiu em {m:?}"
        );
    }
}

/// ⚠️ **O denominador é INTEIRO e nunca desce abaixo de 1.** Um `1/0` seria uma divisão por zero no
/// assador, e um `1/2,7` um desfasamento que nenhum reticulado exprime.
#[test]
fn the_offset_denominator_is_a_whole_number_and_never_zero() {
    let (mut scene, pen, id) = scene_with(fill());
    let mut h = ph2d_vec_edit::History::default();
    apply(
        &mut scene,
        &mut h,
        &pen,
        PatternSlot::Fill,
        TexPatCmd::OffsetDenom(2.7),
    );
    assert_eq!(pattern_of(&scene, id).offset_denom, 3);
    apply(
        &mut scene,
        &mut h,
        &pen,
        PatternSlot::Fill,
        TexPatCmd::OffsetDenom(-5.0),
    );
    assert_eq!(pattern_of(&scene, id).offset_denom, 1);
}

/// **O ângulo do painel é em GRAUS e o documento guarda RADIANOS** — a conversão vive numa porta só.
#[test]
fn the_angle_crosses_from_degrees_to_radians_in_one_door() {
    let (mut scene, pen, id) = scene_with(fill());
    let mut h = ph2d_vec_edit::History::default();
    apply(
        &mut scene,
        &mut h,
        &pen,
        PatternSlot::Fill,
        TexPatCmd::Angle(90.0),
    );
    assert!(
        (pattern_of(&scene, id).angle - std::f64::consts::FRAC_PI_2).abs() < 1e-12,
        "90 graus nao viraram pi/2"
    );
}

/// ⚠️ **Uma forma SEM padrão não é tocada** — a secção nem sobe para ela, mas o comando pode chegar
/// por um caminho que o painel não controla (um atalho, um replay), e um `Paint::Solid` que virasse
/// padrão por causa de um slider seria uma edição que o artista não pediu.
#[test]
fn a_shape_without_a_pattern_is_left_alone() {
    let mut scene = VecScene::default();
    let id = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        fill: Some(Paint::solid(Rgba8::new(9, 9, 9, 255))),
        ..VecPath::default()
    });
    let mut pen = ph2d_vec_edit::PenTool::default();
    pen.select_many(&[id]);
    let mut h = ph2d_vec_edit::History::default();
    apply(
        &mut scene,
        &mut h,
        &pen,
        PatternSlot::Fill,
        TexPatCmd::Axis(0, 4.0, true),
    );
    assert_eq!(h.undo_len(), 0, "gravou um passo sobre uma forma solida");
    assert!(matches!(
        scene.path(id).and_then(|p| p.fill.as_ref()),
        Some(Paint::Solid(_))
    ));
}

/// ⛔⛔ **REPORT DO ENIO (2026-08-27, o SEGUNDO): *"quando volta para tile o aspecto fica de clamp
/// até mudar o parâmetro Size"*.**
///
/// A 1.ª cura do *"clamp deixa tudo em branco"* **ESCREVIA** `size`/`origin` ao entrar no modo. Isso
/// destruía a lei que o artista tinha afinado, e voltar a `Tile` não a devolvia. ⇒ o enquadramento
/// passou a ser **DERIVADO no desenho** (`PatternFill::placement_in`), e trocar de modo é agora uma
/// escrita de **um** campo. *Um modo de APRESENTAÇÃO não consome o documento.*
#[test]
fn switching_modes_never_touches_the_authored_law() {
    let (mut scene, pen, id) = scene_with(fill());
    let mut h = ph2d_vec_edit::History::default();
    let antes = pattern_of(&scene, id);
    for m in [2u8, 0, 1, 2, 0] {
        apply(
            &mut scene,
            &mut h,
            &pen,
            PatternSlot::Fill,
            TexPatCmd::Mode(m),
        );
    }
    let depois = pattern_of(&scene, id);
    assert_eq!(depois.size, antes.size, "trocar de modo mexeu no tamanho");
    assert_eq!(
        depois.origin, antes.origin,
        "trocar de modo mexeu na origem"
    );
    assert_eq!(
        depois.kind, antes.kind,
        "trocar de modo mexeu no reticulado"
    );
    assert_eq!(
        depois.mode,
        PatternMode::Tile,
        "e o ultimo modo tem de valer"
    );
}

/// ⭐⭐ **A PORTA POR-ID do Picker** (plano 33, W7) — ela resolve por identidade e **não** pela
/// seleção, e é essa separação que faz o gesto de duas mãos funcionar.
///
/// ⚠️ O alvo é capturado no *arm*; o clique seguinte cai noutra forma, e ela passa a ser a
/// selecionada. Ler a seleção aqui apontaria o padrão para a forma errada — o *"escolhendo a si
/// mesmo"* que o doc do `vec_pick` nomeia como a razão de o Picker existir.
#[test]
fn the_by_id_door_writes_the_captured_shape_and_nothing_else() {
    // ⚠️ A fonte inicial aponta para um id que NÃO pode nascer nesta cena: a 1.ª redacção usava o
    // `fill()` partilhado (que aponta para `Shape(1)`) e a segunda forma nascia **com o id 1** — a
    // escrita virava um no-op e o gate media a colisão da fixtura, não a porta.
    let mut inicial = fill();
    inicial.source = PatternSource::Shape(999);
    let (mut scene, _pen, alvo) = scene_with(inicial);
    // Uma SEGUNDA forma, que é quem estaria selecionada depois do clique.
    let outra = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        fill: Some(Paint::solid(Rgba8::new(9, 9, 9, 255))),
        ..VecPath::default()
    });
    let mut h = ph2d_vec_edit::History::default();
    assert!(set_source(
        &mut scene,
        &mut h,
        alvo,
        PatternSlot::Fill,
        PatternSource::Shape(outra)
    ));
    assert_eq!(pattern_of(&scene, alvo).source, PatternSource::Shape(outra));
    assert_eq!(h.undo_len(), 1, "escrever a fonte e' UM passo de undo");

    // ⚠️ O MESMO valor não grava passo — senão re-armar o picker e escolher a mesma forma encheria
    // a pilha de undo com passos que não mudam nada.
    assert!(!set_source(
        &mut scene,
        &mut h,
        alvo,
        PatternSlot::Fill,
        PatternSource::Shape(outra)
    ));
    assert_eq!(h.undo_len(), 1, "o mesmo valor gravou um passo espurio");

    // ⚠️ E uma forma SEM padrão não é tocada: o picker escreve numa forma que TEM um.
    assert!(!set_source(
        &mut scene,
        &mut h,
        outra,
        PatternSlot::Fill,
        PatternSource::Shape(alvo)
    ));
    assert_eq!(h.undo_len(), 1);
    assert!(matches!(
        scene.path(outra).and_then(|p| p.fill.as_ref()),
        Some(Paint::Solid(_))
    ));
}

/// ⭐⭐ **O SHIFT MOVE O PADRÃO, e a base é o canto da CAIXA da forma** (Enio, 2026-08-27).
///
/// Esta fileira substitui a alça de MOVER do W6, retirada por decisão dele. ⚠️ A base tem de ser
/// ligada à FORMA: com uma base no mundo, a fase de um padrão dependeria de onde a forma está.
#[test]
fn the_shift_command_moves_the_pattern_by_a_fraction_of_one_repeat() {
    // A forma é o quadrado `[0,0]..[10,10]`; a arte mede `[8, 2]` sem vão ⇒ período `[8, 2]`.
    let (mut scene, pen, id) = scene_with(fill());
    let mut h = ph2d_vec_edit::History::default();
    apply(
        &mut scene,
        &mut h,
        &pen,
        PatternSlot::Fill,
        TexPatCmd::Shift(0, 25.0),
    );
    let p = pattern_of(&scene, id);
    assert!(
        (p.origin[0] - 2.0).abs() < 1e-9,
        "25% de um periodo de 8 sao 2 unidades: {:?}",
        p.origin
    );
    assert!(
        p.origin[1].abs() < 1e-12,
        "o eixo Y mexeu-se: {:?}",
        p.origin
    );
    assert_eq!(h.undo_len(), 1);

    apply(
        &mut scene,
        &mut h,
        &pen,
        PatternSlot::Fill,
        TexPatCmd::Shift(1, 50.0),
    );
    let p = pattern_of(&scene, id);
    assert!(
        (p.origin[1] - 1.0).abs() < 1e-9,
        "50% de um periodo de 2 e' 1 unidade: {:?}",
        p.origin
    );
    assert_eq!(h.undo_len(), 2);

    // ⚠️ **O MESMO valor não grava passo.** O slider re-publica a cada quadro em que está agarrado;
    // sem isto, arrastar uma vez encheria a pilha de undo.
    apply(
        &mut scene,
        &mut h,
        &pen,
        PatternSlot::Fill,
        TexPatCmd::Shift(0, 25.0),
    );
    apply(
        &mut scene,
        &mut h,
        &pen,
        PatternSlot::Fill,
        TexPatCmd::Shift(1, 50.0),
    );
    assert_eq!(h.undo_len(), 2, "o mesmo valor gravou um passo espurio");
}

// ── O ALVO da secção (plano 35, wave D) ────────────────────────────────────────
//
// ⚠️⚠️ **A fixtura tem de conter os DOIS**: uma forma com padrão só no preenchimento, e uma com
// padrão nos dois. É a exigência escrita no §4 do plano — um gate só sobre a primeira passa com a
// wave inteira por construir.

/// Uma forma com padrão no preenchimento (`no_fill`) e/ou no traço (`no_traco`).
///
/// ⚠️ **Os dois padrões são DIFERENTES** (`size` `[8,2]` contra `[3,5]`): com dois iguais, entregar
/// o padrão errado dá o resultado certo por acidente — a mesma armadilha que a chave do memo da
/// wave C teve de evitar.
fn cena_alvos(no_fill: bool, no_traco: bool) -> (VecScene, ph2d_vec_edit::PenTool, VecPathId) {
    let mut scene = VecScene::default();
    let cor = Rgba8::new(1, 2, 3, 255);
    let mut s = ph2d_vec_scene::StrokeSpec::new(cor, 0.5);
    if no_traco {
        s.paint = ph2d_vec_scene::StrokePaint::Pattern(Box::new(PatternFill::new(
            PatternSource::Shape(2),
            [3.0, 5.0],
            cor,
        )));
    }
    let id = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        fill: no_fill
            .then(|| Paint::Pattern(Box::new(fill())))
            .or_else(|| Some(Paint::solid(cor))),
        stroke: Some(s),
        ..VecPath::default()
    });
    let mut pen = ph2d_vec_edit::PenTool::default();
    pen.select_many(&[id]);
    (scene, pen, id)
}

/// ⭐⭐ **A SECÇÃO EDITA O ALVO QUE ESTÁ ACESO** (gate nº 6 do plano 35 §4).
///
/// ⚠️ **O controle é a outra metade**: com o preenchimento aceso, o traço tem de ficar INTACTO. Sem
/// ele, uma implementação que escrevesse nos dois passaria.
#[test]
fn the_pattern_section_edits_the_target_that_is_lit() {
    let (mut scene, pen, id) = cena_alvos(true, true);
    let mut h = ph2d_vec_edit::History::default();
    apply(
        &mut scene,
        &mut h,
        &pen,
        PatternSlot::Stroke,
        TexPatCmd::Angle(30.0),
    );
    assert!(
        (pattern_at(&scene, id, PatternSlot::Stroke)
            .expect("o traco tem padrao")
            .angle
            - 30f64.to_radians())
        .abs()
            < 1e-12,
        "o angulo nao entrou no TRACO - a seccao editou o outro sujeito"
    );
    assert_eq!(
        pattern_at(&scene, id, PatternSlot::Fill)
            .expect("o preenchimento tem padrao")
            .angle,
        0.0,
        "escrever no traco mexeu tambem no PREENCHIMENTO"
    );
    // E o simétrico, com o outro alvo aceso.
    apply(
        &mut scene,
        &mut h,
        &pen,
        PatternSlot::Fill,
        TexPatCmd::Angle(45.0),
    );
    assert!(
        (pattern_at(&scene, id, PatternSlot::Stroke)
            .expect("o traco tem padrao")
            .angle
            - 30f64.to_radians())
        .abs()
            < 1e-12,
        "escrever no preenchimento mexeu tambem no TRACO"
    );
}

/// ⭐⭐ **O CHIP DO ALVO só aparece quando os dois existem** (gate nº 7 do plano 35 §4) — com um só
/// não há escolha a oferecer.
#[test]
fn the_target_chip_only_shows_when_both_exist() {
    let want = PatternSlot::Fill;
    let (s, _, id) = cena_alvos(true, true);
    assert_eq!(lit_target(&s, id, want).map(|(_, ambos)| ambos), Some(true));
    let (s, _, id) = cena_alvos(true, false);
    assert_eq!(
        lit_target(&s, id, want).map(|(_, ambos)| ambos),
        Some(false),
        "o chip aparece com UM alvo so' - ele nao tem escolha a oferecer"
    );
    let (s, _, id) = cena_alvos(false, true);
    assert_eq!(
        lit_target(&s, id, want).map(|(_, ambos)| ambos),
        Some(false)
    );
    let (s, _, id) = cena_alvos(false, false);
    assert_eq!(
        lit_target(&s, id, want),
        None,
        "sem padrao nenhum a seccao ainda sobe"
    );
}

/// ⚠️⚠️ **A preferência de sessão é COAGIDA ao que a forma tem.**
///
/// Escolher *Stroke* numa forma e clicar noutra, cujo traço não tem padrão, não pode fazer a secção
/// **desaparecer** por se lembrar de uma escolha feita algures.
#[test]
fn a_sticky_target_is_coerced_to_what_the_shape_actually_has() {
    // Só o preenchimento tem padrão, e a sessão lembra-se do TRAÇO.
    let (s, _, id) = cena_alvos(true, false);
    assert_eq!(
        lit_target(&s, id, PatternSlot::Stroke).map(|(a, _)| a),
        Some(PatternSlot::Fill),
        "a preferencia venceu o que existe - a seccao sobe vazia"
    );
    // E o simétrico.
    let (s, _, id) = cena_alvos(false, true);
    assert_eq!(
        lit_target(&s, id, PatternSlot::Fill).map(|(a, _)| a),
        Some(PatternSlot::Stroke)
    );
    // CONTROLO: com os dois, a preferência MANDA — senão este gate ficaria verde num produto que
    // ignora o chip por completo.
    let (s, _, id) = cena_alvos(true, true);
    assert_eq!(
        lit_target(&s, id, PatternSlot::Stroke).map(|(a, _)| a),
        Some(PatternSlot::Stroke)
    );
    assert_eq!(
        lit_target(&s, id, PatternSlot::Fill).map(|(a, _)| a),
        Some(PatternSlot::Fill)
    );
}

/// ⚠️ **A troca de ARTE também honra o alvo** — o botão *Source…* e o picker de forma escrevem na
/// tinta acesa, e não sempre no preenchimento.
#[test]
fn changing_the_art_honours_the_target_too() {
    let (mut scene, _, id) = cena_alvos(true, true);
    let mut h = ph2d_vec_edit::History::default();
    assert!(set_source(
        &mut scene,
        &mut h,
        id,
        PatternSlot::Stroke,
        PatternSource::Shape(77)
    ));
    assert_eq!(
        pattern_at(&scene, id, PatternSlot::Stroke).map(|p| p.source),
        Some(PatternSource::Shape(77))
    );
    assert_eq!(
        pattern_at(&scene, id, PatternSlot::Fill).map(|p| p.source),
        Some(PatternSource::Shape(1)),
        "trocar a arte do traco trocou tambem a do preenchimento"
    );
}
