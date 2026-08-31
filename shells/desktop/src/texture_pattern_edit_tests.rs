//! Os gates da autoria do padrão (plano 33, W5).

use super::*;
use ph2d_vec_scene::{Rgba8, VecPath, VecPathId, VecVertex};

pub(super) fn scene_with(f: PatternFill) -> (VecScene, ph2d_vec_edit::PenTool, VecPathId) {
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

pub(super) fn fill() -> PatternFill {
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

pub(super) fn pattern_of(scene: &VecScene, id: VecPathId) -> PatternFill {
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
        PatternSource::Shape(outra),
        // ⚠️ A fixtura JÁ tem arte, então este tamanho **não** pode ser adoptado — é o controlo da
        // metade "trocar a arte preserva o do artista".
        [77.0, 77.0],
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
        PatternSource::Shape(outra),
        // ⚠️ A fixtura JÁ tem arte, então este tamanho **não** pode ser adoptado — é o controlo da
        // metade "trocar a arte preserva o do artista".
        [77.0, 77.0],
    ));
    assert_eq!(h.undo_len(), 1, "o mesmo valor gravou um passo espurio");

    // ⚠️ E uma forma SEM padrão não é tocada: o picker escreve numa forma que TEM um.
    assert!(!set_source(
        &mut scene,
        &mut h,
        outra,
        PatternSlot::Fill,
        PatternSource::Shape(alvo),
        [77.0, 77.0],
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

/// ⭐⭐⭐ **O PAINEL DIZ O MESMO QUE O DESENHO, mesmo com o ângulo a acumular** (auditoria de
/// 2026-08-30).
///
/// # O defeito
///
/// O `angle` do documento **acumula**: o `transform_fill_geometry` faz `pat.angle += atan2(..)` a
/// cada rotação, sem dar a volta — e isso é o que torna rodar duas vezes o mesmo que rodar pela
/// soma. ⇒ ele sai de `0..360` por **uso ordinário**.
///
/// ⛔ O slider e o campo numérico estão registados em `0..=360` e **os dois coagem**: o slider
/// encostava num extremo (visualmente igual a `0`) enquanto o número mostrava `400` — os dois a
/// discordar no ecrã —, e o primeiro toque em qualquer um deles **SALTAVA** o padrão.
///
/// ⚠️ O gradiente linear, na MESMA função de publicação, já normalizava. *Duas respostas à mesma
/// pergunta, e só uma delas tinha sido escrita.*
#[test]
fn the_panel_reads_the_same_angle_the_drawing_shows() {
    use std::f64::consts::TAU;
    for (rad, esperado, o_que) in [
        (0.0, 0.0, "zero"),
        (TAU * 0.25, 90.0, "um quarto"),
        // ⭐ O caso do defeito: duas voltas e um quarto acumuladas.
        (TAU * 2.25, 90.0, "duas voltas e um quarto"),
        // ⭐ E o NEGATIVO, que o `atan2` produz metade das vezes.
        (-TAU * 0.25, 270.0, "um quarto NEGATIVO"),
    ] {
        let v = super::panel_angle_deg(rad);
        assert!(
            (v - esperado).abs() < 1e-9,
            "{o_que}: o painel mostraria {v} e o desenho mostra {esperado} - fora de `0..360` o \
             slider encosta e o numero discorda dele, e o primeiro toque SALTA o padrao"
        );
        assert!(
            (0.0..360.0).contains(&v),
            "{o_que}: {v} esta' fora da faixa REGISTADA dos dois controlos"
        );
    }
}

/// ⚠️ **E a ida-e-volta fecha:** o que o painel mostra, escrito de volta, dá o mesmo ângulo.
///
/// A escrita é `deg.to_radians()` (`TexPatCmd::Angle`), e sem esta folha a normalização podia
/// deslocar o padrão de propósito — *normalizar a LEITURA não pode mudar o que a ESCRITA significa*.
#[test]
fn what_the_panel_shows_writes_back_to_the_same_angle() {
    for graus in [0.0_f64, 37.5, 180.0, 359.9] {
        let ida = graus.to_radians();
        let volta = super::panel_angle_deg(ida);
        assert!(
            (volta - graus).abs() < 1e-9,
            "{graus} deg foi e voltou como {volta} - a leitura e a escrita discordam"
        );
    }
}

/// ⭐⭐⭐ **O VÃO TEM DOIS EIXOS, e o ELO é o comportamento de sempre** (report do Enio, 2026-08-30).
///
/// # Porque isto deixou de ser cosmética
///
/// O vão era **um número para os dois eixos**, e isso tornou-se load-bearing na mesma jornada: desde
/// que o passo vertical da colmeia passou a ler o `gap[1]`, abrir as **fileiras** do favo é a única
/// saída do encaixe de `13,4 %` — e com um controlo só ela abria também as **colunas**.
///
/// # ⚠️ O elo NÃO é o cadeado do tamanho, e a diferença é o ZERO
///
/// O cadeado de aspecto preserva a **razão** actual. Um vão nasce em `0`, e uma razão sobre zero não
/// significa nada. ⇒ aqui o elo é *"o mesmo número"* — que é **exactamente** o que o controlo único
/// fazia, e é por isso que o elo LIGADO é o comportamento de sempre, ao bit.
///
/// ⚠️ **As duas metades são o controlo uma da outra**: sem a de cima, um elo que nunca ligasse
/// passaria a de baixo; sem a de baixo, um elo que ligasse SEMPRE passaria a de cima — e era esse o
/// defeito.
#[test]
fn the_gap_has_two_axes_and_the_link_is_the_old_behaviour_exactly() {
    let (mut scene, pen, id) = scene_with(fill());
    let aplica = |scene: &mut VecScene, cmd| {
        apply(
            scene,
            &mut ph2d_vec_edit::History::default(),
            &pen,
            PatternSlot::Fill,
            cmd,
        );
    };
    // ⭐ LIGADO: mexer num eixo leva o outro ao MESMO número — o controlo único de sempre.
    aplica(&mut scene, TexPatCmd::Gap(1, 7.0, true));
    assert_eq!(
        pattern_of(&scene, id).gap,
        [7.0, 7.0],
        "com o elo LIGADO os dois vaos tem de ficar iguais - e' o comportamento que existia antes"
    );
    // ⭐⭐ DESLIGADO: o eixo Y abre SOZINHO — é isto que o favo de mel precisava.
    aplica(&mut scene, TexPatCmd::Gap(1, 3.0, false));
    assert_eq!(
        pattern_of(&scene, id).gap,
        [7.0, 3.0],
        "com o elo DESLIGADO abrir as fileiras abriu tambem as colunas - e' o report de 30/08"
    );
    // E o eixo X também, sem tocar no Y.
    aplica(&mut scene, TexPatCmd::Gap(0, -2.0, false));
    assert_eq!(pattern_of(&scene, id).gap, [-2.0, 3.0]);
    // ⚠️ CONTROLO: um eixo fora de alcance não escreve nada — senão um índice novo escreveria em
    // silêncio no primeiro elemento.
    aplica(&mut scene, TexPatCmd::Gap(9, 99.0, false));
    assert_eq!(
        pattern_of(&scene, id).gap,
        [-2.0, 3.0],
        "um eixo fora de alcance escreveu"
    );
}

/// ⭐⭐ **E O VÃO VERTICAL ABRE AS FILEIRAS DA COLMEIA** — a razão de existir desta wave.
///
/// ⚠️ A régua é o **período**, que é a lei que o desenho lê ([`ph2d_vec_scene::PatternFill::period`]),
/// e não um número que eu escreva: com o vão vertical no valor que compensa o aperto de `√3/2`, as
/// fileiras encostam. ⛔ E o eixo X tem de ficar onde estava — era isso que o controlo único não
/// conseguia.
#[test]
fn opening_the_hex_rows_no_longer_opens_its_columns() {
    let mut f = fill();
    f.kind = ph2d_vec_pattern::TileKind::Hex;
    f.size = [10.0, 10.0];
    f.gap = [0.0, 0.0];
    let (mut scene, pen, id) = scene_with(f);
    let antes = pattern_of(&scene, id).period();
    // O vão que compensa exactamente o aperto do favo.
    let abre = 10.0 * (1.0 / ph2d_vec_pattern::HEX_ROW_RATIO - 1.0);
    apply(
        &mut scene,
        &mut ph2d_vec_edit::History::default(),
        &pen,
        PatternSlot::Fill,
        TexPatCmd::Gap(1, abre, false),
    );
    let depois = pattern_of(&scene, id).period();
    assert!(
        (depois[1] - 10.0).abs() < 1e-9,
        "as fileiras nao encostaram: passo {} contra 10,0",
        depois[1]
    );
    assert!(
        (depois[0] - antes[0]).abs() < 1e-12,
        "abrir as FILEIRAS mexeu nas COLUNAS ({} -> {}) - e' exactamente o report",
        antes[0],
        depois[0]
    );
    assert!(antes[1] < 10.0, "o controlo: o favo apertava mesmo");
}
