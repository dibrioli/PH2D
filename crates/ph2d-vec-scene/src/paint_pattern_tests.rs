//! Os gates do **dado** do Texture Pattern (plano 33, W3).
//!
//! ⚠️ A fixtura contém os fenómenos que a §5.1 do plano declarou: escala não-uniforme, rotação,
//! e um vão que a colmeia tem de ignorar.

use super::{Paint, PatternFill, PatternSource, Rgba8, VecPath, VecPathId, VecScene, VecVertex};
use crate::paint_bind::BoundStyle;
use ph2d_asset_id::AssetId;
use ph2d_vec_pattern::{PatternMode, TileKind};

fn src() -> PatternSource {
    PatternSource::Image(AssetId::from_bytes(b"tijolo"))
}

fn fill() -> PatternFill {
    PatternFill::new(src(), [10.0, 20.0], Rgba8::new(200, 30, 30, 255))
}

/// Uma forma quadrada com o padrão de referência já vestido.
fn scene_with_pattern(f: PatternFill) -> (VecScene, VecPathId) {
    let mut scene = VecScene::default();
    let verts = [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]]
        .map(VecVertex::corner)
        .to_vec();
    let id = scene.push_path(VecPath {
        verts,
        closed: true,
        fill: Some(Paint::Pattern(Box::new(f))),
        ..VecPath::default()
    });
    (scene, id)
}

/// ⚠️ **O `Paint` NÃO ENGORDA** — e o número é medido, não estimado.
///
/// Ele mora dentro de todo `VecPath`, e todo `VecPath` entra em toda fotografia de undo. Medido em
/// 2026-08-27, ANTES desta wave: `size_of::<Paint>() == 56`. Um [`PatternFill`] em linha levá-lo-ia
/// a mais do dobro — pago por cada forma da cena a cada passo de undo, **inclusive pelas que não
/// têm padrão nenhum**.
///
/// ⭐ A desigualdade é a afirmação portátil (`Paint` mais pequeno que o `PatternFill` que ele
/// referencia ⇒ há indirecção); o `56` ao lado é o número que a medição deu.
#[test]
fn the_paint_enum_does_not_grow_when_pattern_lands() {
    let paint = std::mem::size_of::<Paint>();
    assert!(
        paint < std::mem::size_of::<PatternFill>(),
        "o PatternFill entrou em linha no Paint: {paint} bytes"
    );
    assert_eq!(
        paint, 56,
        "o Paint mudou de tamanho ({paint} != 56) - reconfira o custo por passo de undo antes de \
         actualizar este numero"
    );
}

/// ⭐⭐ **A colmeia DERIVA o passo vertical, e o `gap[1]` autorado é ignorado nela.**
///
/// Dois sítios a decidir o mesmo passo dariam um desenho num instante e um espaçamento noutro.
/// Aqui a lei vive na [`PatternFill::period`], e este gate mede que ela **manda** — mexer no vão
/// vertical de uma colmeia não pode mover coisa nenhuma.
#[test]
fn the_hex_period_is_derived_and_the_authored_y_gap_is_ignored() {
    let mut f = fill();
    f.kind = TileKind::Hex;
    f.gap = [0.0, 0.0];
    let base = f.period();
    f.gap = [0.0, 999.0];
    assert_eq!(base, f.period(), "a colmeia leu o vao vertical autorado");
    assert!(
        (base[1] - base[0] * ph2d_vec_pattern::HEX_ROW_RATIO).abs() < 1e-12,
        "o passo vertical da colmeia nao e' o derivado"
    );
    // Controlo: numa grade o vão vertical MANDA — senão este gate estaria a medir uma função morta.
    let mut g = fill();
    g.kind = TileKind::Grid;
    let a = g.period();
    g.gap = [0.0, 999.0];
    assert_ne!(a, g.period(), "numa grade o vao vertical tem de contar");
}

/// **Mundo -> pixels atravessa UMA porta**, e ela alimenta-se do período (não do vão autorado) —
/// é isso que faz a colmeia usar a mesma conta que todo o resto.
#[test]
fn the_pattern_law_crosses_one_door_from_world_to_pixels() {
    let mut f = fill();
    f.gap = [5.0, 10.0];
    // A arte mede 10x20 no mundo e tem 100x200 px ⇒ 10 px por unidade nos dois eixos.
    let law = f.law([100, 200]);
    assert_eq!(law.gap_px, [50, 100]);
    // E a colmeia entra pela MESMA porta, com o vão vertical que o período dela impõe.
    let mut h = fill();
    h.kind = TileKind::Hex;
    h.size = [10.0, 10.0];
    let hl = h.law([100, 100]);
    assert_eq!(hl.gap_px[0], 0, "sem vao horizontal autorado");
    assert!(
        hl.gap_px[1] < 0,
        "a colmeia aperta as linhas (sqrt(3)/2 < 1), deu {}",
        hl.gap_px[1]
    );
}

/// ⚠️ **Desvanecer um padrão baixa a OPACIDADE dele, não uma cor.** Um padrão não tem cor para
/// escalar — se só a `fallback` descesse, a forma manteria o ladrilho a cheio e clarearia apenas o
/// instante em que ele ainda não resolveu, que é o contrário do que se vê.
#[test]
fn fading_a_pattern_dims_the_pattern_not_just_its_fallback() {
    let (scene, id) = scene_with_pattern(fill());
    let bound = BoundStyle {
        path: id,
        alpha: Some(128),
        ..BoundStyle::default()
    };
    let painted = scene.paths()[0].painted(Some(&bound));
    let Some(Paint::Pattern(p)) = &painted.fill else {
        panic!("o desvanecimento trocou a especie do preenchimento")
    };
    assert!(
        (p.alpha - 128.0 / 255.0).abs() < 1e-6,
        "a opacidade do padrao nao desceu: {}",
        p.alpha
    );
    // (255*128 + 127) / 255 = 128 — a conta arredondada do `fade`, não a minha estimativa.
    assert_eq!(p.fallback.a, 128, "a cor de recurso desce junto");
}

/// ⭐⭐ **O padrão CONSERVA A ORIENTAÇÃO quando a forma roda** — e é o único preenchimento desta
/// casa que o faz.
///
/// O gradiente radial não pode: um radial do peniko **é circular** e não tem onde guardar um
/// ângulo, e é por isso que o `transform_fill_geometry` lhe passa um `radius_scale` médio. O padrão
/// tem o campo, então a sonda do afim (as imagens dos dois eixos unitários) dá-lhe a resposta exacta.
#[test]
fn rotating_the_shape_rotates_the_pattern_with_it() {
    let (mut scene, id) = scene_with_pattern(fill());
    let quarter = std::f64::consts::FRAC_PI_2;
    assert!(scene.rotate_path_by(id, quarter, [0.0, 0.0]));
    let Some(Paint::Pattern(p)) = &scene.paths()[0].fill else {
        panic!("especie trocada")
    };
    assert!(
        (p.angle - quarter).abs() < 1e-9,
        "o padrao nao rodou com a forma: {}",
        p.angle
    );
    assert!(
        (p.size[0] - 10.0).abs() < 1e-9 && (p.size[1] - 20.0).abs() < 1e-9,
        "uma rotacao nao pode mudar o tamanho do ladrilho: {:?}",
        p.size
    );
}

/// **Escalar a forma escala o ladrilho, POR EIXO** — e a escala não-uniforme é o fenómeno que a
/// fixtura tinha de conter (plano 33 §5.1).
#[test]
fn scaling_the_shape_scales_the_tile_per_axis() {
    let (mut scene, id) = scene_with_pattern(fill());
    assert!(scene.scale_path(id, 3.0, 0.5, [0.0, 0.0]));
    let Some(Paint::Pattern(p)) = &scene.paths()[0].fill else {
        panic!("especie trocada")
    };
    assert!(
        (p.size[0] - 30.0).abs() < 1e-9 && (p.size[1] - 10.0).abs() < 1e-9,
        "o ladrilho nao seguiu a escala por eixo: {:?}",
        p.size
    );
    assert!(
        p.angle.abs() < 1e-9,
        "uma escala positiva nao pode rodar o padrao"
    );
}

/// **Mover a forma move o padrão com ela** — a lei que o `paint.rs` já escreveu para os gradientes,
/// e o oposto do defeito da origem-da-régua do Illustrator.
#[test]
fn moving_the_shape_moves_the_pattern_with_it() {
    let (mut scene, id) = scene_with_pattern(fill());
    assert!(scene.translate_path(id, 7.0, -3.0));
    let Some(Paint::Pattern(p)) = &scene.paths()[0].fill else {
        panic!("especie trocada")
    };
    assert_eq!(p.origin, [7.0, -3.0]);
    assert!(p.angle.abs() < 1e-9 && (p.size[0] - 10.0).abs() < 1e-9);
}

/// **A cor representativa de um padrão é a `fallback`** — uma resposta EXACTA, e não uma
/// aproximação: é literalmente a cor que ele pinta enquanto o ladrilho não resolve.
#[test]
fn the_swatch_colour_of_a_pattern_is_its_fallback() {
    let f = fill();
    let c = f.fallback;
    assert_eq!(Paint::Pattern(Box::new(f)).primary_color(), c);
}

/// **O padrão sobrevive ao save**, com a fonte e a lei intactas.
#[test]
fn a_pattern_round_trips_through_postcard() {
    let mut f = fill();
    f.kind = TileKind::BrickRow;
    f.offset_denom = 3;
    f.mode = PatternMode::Mirror;
    f.angle = 0.75;
    let (scene, _) = scene_with_pattern(f.clone());
    let bytes = scene.to_bytes().expect("serializa");
    let back = VecScene::from_bytes(&bytes).expect("desserializa");
    let Some(Paint::Pattern(p)) = &back.paths()[0].fill else {
        panic!("o padrao nao sobreviveu ao save")
    };
    assert_eq!(**p, f);
}

/// ⛔⛔ **O `Clamp` ENQUADRA a cópia na forma, e a lei é DERIVADA** — report do Enio (2026-08-27):
/// *"clamp deixa tudo em branco"*, e depois *"quando volta para tile o aspecto fica de clamp"*.
///
/// Os dois reports são a mesma lei vista de dois lados: enquadrar é necessário (senão o `Pad` mostra
/// só a borda esticada) e **escrever** o enquadramento é errado (senão voltar não devolve nada).
#[test]
fn clamp_frames_the_copy_over_the_shape_without_touching_the_authored_law() {
    let mut f = fill(); // size [10, 20] (aspecto 1:2), origem [0,0]
    let bbox = ([100.0, 50.0], [140.0, 90.0]); // 40x40, LONGE da origem
    let autorada = f.placement([1, 1], [8, 16]);

    // Tile: a colocação é a autorada, AO BIT.
    f.mode = PatternMode::Tile;
    assert_eq!(f.placement_in([1, 1], [8, 16], bbox), autorada);
    f.mode = PatternMode::Mirror;
    assert_eq!(f.placement_in([1, 1], [8, 16], bbox), autorada);

    // Clamp: enquadra — a origem vira o canto da forma e a cópia COBRE a caixa.
    f.mode = PatternMode::Clamp;
    let m = f.placement_in([1, 1], [8, 16], bbox);
    assert_eq!(
        [m[4], m[5]],
        [100.0, 50.0],
        "a copia nao foi ao canto da forma"
    );
    // O canto oposto do ladrilho: `size` reescalado para cobrir 40x40 com aspecto 1:2 ⇒ [40, 80].
    let far = [m[0] * 8.0 + m[4], m[3] * 16.0 + m[5]];
    assert!(
        (far[0] - 140.0).abs() < 1e-9 && far[1] >= 90.0 - 1e-9,
        "a copia nao COBRE a caixa: canto oposto {far:?}"
    );
    // ⚠️ E o DOCUMENTO não se mexeu: `placement_in` é uma leitura.
    assert_eq!(f.size, [10.0, 20.0]);
    assert_eq!(f.origin, [0.0, 0.0]);
}

// ── A FASE (`shift`) — a posição do padrão depois de as alças de canvas saírem ────────
//
// ⛔ As três alças do plano 33 W6 foram RETIRADAS por decisão do Enio (2026-08-27: *"não ficou
// legal. vamos retirar e deixar os ajustes apenas no painel"*). A posição passou para as fileiras
// *Shift X/Y*, e é esta lei que elas atravessam.

/// **A fase mede-se ao longo dos eixos DO PADRÃO, em unidades de UMA repetição.**
///
/// ⚠️ Os eixos do padrão e não os do mundo: é ao longo deles que a repetição é periódica. Com um
/// quarto de volta os dois papéis trocam, e é isso que este gate mede.
#[test]
fn the_shift_is_a_phase_along_the_patterns_own_axes() {
    let mut f = fill(); // size [10, 20], gap [0, 0] ⇒ período [10, 20]
    f.origin = [2.5, 5.0];
    let s = f.shift([0.0, 0.0]);
    assert!((s[0] - 0.25).abs() < 1e-12, "{s:?}");
    assert!((s[1] - 0.25).abs() < 1e-12, "{s:?}");

    // Um quarto de volta: o eixo X do padrão passa a apontar para +Y do mundo.
    f.angle = std::f64::consts::FRAC_PI_2;
    f.origin = [0.0, 2.5];
    let s = f.shift([0.0, 0.0]);
    assert!(
        (s[0] - 0.25).abs() < 1e-12,
        "com o padrao rodado, andar em +Y do MUNDO e' andar no eixo X DELE: {s:?}"
    );
    assert!(s[1].abs() < 1e-12, "e o outro eixo nao se mexeu: {s:?}");
}

/// ⭐ **Uma repetição inteira é a IDENTIDADE** — é isso que fecha a faixa do slider em `0..100 %`.
///
/// ⚠️ Se um período não fosse a identidade, a faixa seria um limite de conforto (um palpite), e não
/// o recurso. A faixa é a periodicidade do reticulado.
#[test]
fn a_whole_period_of_shift_reads_the_same_phase() {
    let mut f = fill(); // período [10, 20]
    f.origin = [3.0, 4.0];
    let antes = f.shift([0.0, 0.0]);
    f.origin = [3.0 + 10.0 * 3.0, 4.0 - 20.0 * 2.0];
    let depois = f.shift([0.0, 0.0]);
    assert!(
        (antes[0] - depois[0]).abs() < 1e-12 && (antes[1] - depois[1]).abs() < 1e-12,
        "{antes:?} != {depois:?}"
    );
}

/// ⚠️⚠️ **Escrever a fase só mexe na parte FRACCIONÁRIA — a origem não é teleportada para junto da
/// base.**
///
/// No `Tile` teleportar seria invisível (um período é a identidade), mas no `Mirror` a identidade
/// são DOIS períodos e o reflexo trocaria de fase sozinho. *Uma escrita que só está certa num dos
/// modos não é a lei.*
#[test]
fn setting_the_shift_moves_only_the_fractional_part() {
    let mut f = fill(); // período [10, 20]
    f.origin = [37.0, 4.0]; // 3 períodos inteiros + 0,7
    f.set_shift_axis([0.0, 0.0], 0, 0.2);
    assert!(
        (f.origin[0] - 32.0).abs() < 1e-9,
        "a origem saltou de celula: {:?}",
        f.origin
    );
    assert!((f.shift([0.0, 0.0])[0] - 0.2).abs() < 1e-12);
}

/// ⭐⭐ **A escrita é IDEMPOTENTE, e é isso que a impede de encher a pilha de undo.**
///
/// A ida e volta `origin -> fase -> origin` acontece a **cada quadro** em que o slider está
/// agarrado. Sem tolerância, o último bit mudaria de cada vez e cada quadro viraria um passo — o
/// defeito que o `canonicalize` do editor curou para o mundo inteiro.
#[test]
fn writing_the_phase_that_is_already_there_writes_nothing() {
    let mut f = fill();
    f.origin = [37.0, 4.0];
    f.angle = 0.7; // um ângulo feio de propósito: a base ortonormal não é a canónica
    let fase = f.shift([1.5, -2.5]);
    let antes = f.origin;
    for _ in 0..64 {
        f.set_shift_axis([1.5, -2.5], 0, fase[0]);
        f.set_shift_axis([1.5, -2.5], 1, fase[1]);
    }
    assert_eq!(
        f.origin, antes,
        "64 re-escritas do MESMO valor moveram a origem - cada quadro seria um passo de undo"
    );
}

/// **Escrever um eixo não toca no outro** — o delta é aplicado ao longo de UMA direcção.
///
/// ⚠️ A 1.ª redacção reconstruía a origem a partir das duas projecções, e a ida e volta pela base
/// deixava lixo de vírgula flutuante no eixo que ninguém tinha pedido.
///
/// ⚠️⚠️ **E os DOIS sentidos são obrigatórios.** A 1.ª versão deste gate escrevia só o eixo `0`, e
/// uma mutação que mandava **os dois** deltas pela direcção do eixo `0` **SOBREVIVEU** — para o eixo
/// `0` ela é a identidade. *Uma fixtura que só exercita o caso em que a mutação é um no-op não mede
/// a lei.*
#[test]
fn writing_one_axis_leaves_the_other_untouched() {
    for (escrito, olhado) in [(0usize, 1usize), (1, 0)] {
        let mut f = fill();
        f.angle = 0.7;
        f.origin = [3.0, 4.0];
        let antes = f.shift([0.0, 0.0])[olhado];
        f.set_shift_axis([0.0, 0.0], escrito, 0.31);
        assert_eq!(
            f.shift([0.0, 0.0])[olhado],
            antes,
            "escrever o eixo {escrito} mexeu no {olhado}"
        );
        // CONTROLO: o eixo que se pediu de facto MUDOU — senão o gate passaria com uma escrita
        // que não faz nada.
        assert!(
            (f.shift([0.0, 0.0])[escrito] - 0.31).abs() < 1e-12,
            "o eixo {escrito} nao recebeu a fase pedida"
        );
    }
}

/// ⚠️ **Um período não-positivo não tem fase nenhuma.** O `gap` é BIPOLAR, então `size + gap` pode
/// ser zero ou negativo — e `rem_euclid` de um período negativo daria uma fase que anda para trás.
#[test]
fn a_non_positive_period_has_no_phase_and_no_write() {
    let mut f = fill(); // size [10, 20]
    f.gap = [-10.0, -20.0]; // período [0, 0]
    assert_eq!(f.shift([0.0, 0.0]), [0.0, 0.0]);
    let antes = f.origin;
    f.set_shift_axis([0.0, 0.0], 0, 0.4);
    f.set_shift_axis([0.0, 0.0], 1, 0.4);
    assert_eq!(f.origin, antes, "escreveu uma fase que nao existe");
    // ⚠️ E um eixo que não existe também não escreve — o índice vem do painel.
    f.gap = [0.0, 0.0];
    let antes = f.origin;
    f.set_shift_axis([0.0, 0.0], 2, 0.4);
    assert_eq!(f.origin, antes, "um eixo inexistente escreveu");
}
