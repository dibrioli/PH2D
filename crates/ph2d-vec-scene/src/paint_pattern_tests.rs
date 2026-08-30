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

pub(super) fn fill() -> PatternFill {
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
    // ⚠️⚠️ **O canto medido é a BASE do ladrilho (`py = th`), e não a linha 0** — desde a cura da
    // inversão vertical (`ph2d_vec_pattern::place_tests::the_tile_is_not_upside_down`, report do
    // Enio de 2026-08-30). A linha `0` do assado é o **topo** do desenho e a âncora é o canto
    // **inferior** da caixa: este teste media o par errado, e passava porque o produto os casava —
    // que era precisamente o defeito. *Uma régua que mede o canto que o código escolheu confirma a
    // escolha, não a lei.*
    let base = [m[2] * 16.0 + m[4], m[3] * 16.0 + m[5]];
    assert_eq!(base, [100.0, 50.0], "a copia nao assenta no canto da forma");
    // O canto oposto do ladrilho: `size` reescalado para cobrir 40x40 com aspecto 1:2 ⇒ [40, 80].
    let far = [m[0] * 8.0 + base[0], m[5]];
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

// ── O TAMANHO POR EIXO e o cadeado de proporção (plano 33, W10) ───────────────────────
//
// ⛔ O `set_longer_side` (um número, aspecto sempre preservado) foi SUBSTITUÍDO: o artista pediu
// para poder achatar de propósito. A protecção não desapareceu — mudou de lei imposta para gesto
// escolhido, e o default do cadeado é LIGADO.

/// ⭐⭐ **COM o cadeado, os dois eixos escalam pelo MESMO factor — a razão ACTUAL sobrevive.**
///
/// ⚠️ Um cadeado que voltasse ao aspecto natural da arte **desfaria o achatamento** que o artista
/// autorou, no instante em que ele mexesse no outro número. É a lei do Photoshop e do Figma.
#[test]
fn the_lock_preserves_the_current_ratio_not_the_arts_natural_one() {
    let mut f = fill(); // size [10, 20] — já 1:2
    f.size = [4.0, 1.0]; // o artista ACHATOU para 4:1
    f.set_axis(0, 8.0, true);
    assert_eq!(
        f.size,
        [8.0, 2.0],
        "o cadeado desfez o achatamento em vez de o escalar"
    );
    // E pelo outro eixo, com o mesmo resultado de razão.
    f.set_axis(1, 1.0, true);
    assert_eq!(f.size, [4.0, 1.0]);
}

/// **SEM o cadeado, cada eixo é independente** — é isto que o artista pediu.
#[test]
fn without_the_lock_an_axis_moves_alone() {
    let mut f = fill(); // [10, 20]
    f.set_axis(0, 5.0, false);
    assert_eq!(f.size, [5.0, 20.0], "o outro eixo mexeu-se");
    f.set_axis(1, 5.0, false);
    assert_eq!(f.size, [5.0, 5.0]);
}

/// ⚠️ **Um valor inválido não escreve nada** — e o `NaN` é o caso que escorrega pela porta de trás,
/// porque ele reprova toda desigualdade. Um `size` de `NaN` apaga a forma sem erro nenhum.
#[test]
fn a_non_positive_or_nan_size_writes_nothing() {
    for mau in [0.0, -3.0, f64::NAN, f64::INFINITY] {
        for lock in [true, false] {
            let mut f = fill();
            f.set_axis(0, mau, lock);
            assert_eq!(f.size, [10.0, 20.0], "{mau} escreveu (lock={lock})");
        }
    }
    // ⚠️ E um eixo que não existe também não escreve — o índice vem do painel.
    let mut f = fill();
    f.set_axis(2, 5.0, false);
    assert_eq!(f.size, [10.0, 20.0]);
}

/// ⚠️ **Um `size` degenerado não define razão nenhuma**, e o cadeado não pode dividir por ele. Os
/// dois eixos passam a medir `v` — o único par que satisfaz *"a razão de antes"* quando não havia.
#[test]
fn a_degenerate_size_under_the_lock_becomes_square() {
    let mut f = fill();
    f.size = [0.0, 4.0];
    f.set_axis(0, 3.0, true);
    assert_eq!(f.size, [3.0, 3.0]);
}

/// ⭐ **SONDA: o que cada peça de tinta PESA** — o número que decide se o padrão no TRAÇO cabe
/// dentro do `StrokeSpec` ou tem de ser indirecto (plano 35).
///
/// ⚠️ Uma sonda, não uma afirmação: ela imprime e não julga. O gate que julga é o
/// `the_paint_enum_does_not_grow_when_pattern_lands`, e ele mede outra coisa.
#[test]
#[ignore = "sonda: imprime tamanhos, nao afirma nada"]
fn measure_the_paint_sizes() {
    println!("Rgba8        {:>4}", std::mem::size_of::<Rgba8>());
    println!("Paint        {:>4}", std::mem::size_of::<Paint>());
    println!("PatternFill  {:>4}", std::mem::size_of::<PatternFill>());
    println!(
        "StrokeSpec   {:>4}",
        std::mem::size_of::<crate::StrokeSpec>()
    );
    println!("VecPath      {:>4}", std::mem::size_of::<VecPath>());
    println!(
        "Option<Box<PatternFill>> {:>4}",
        std::mem::size_of::<Option<Box<PatternFill>>>()
    );
    println!(
        "Option<PatternFill>      {:>4}",
        std::mem::size_of::<Option<PatternFill>>()
    );
}

// ── O PADRÃO NO TRAÇO — wave A: o DADO (plano 35) ─────────────────────────────────────

use crate::{StrokePaint, StrokeSpec};

pub(super) fn stroke_com_padrao() -> StrokeSpec {
    let mut s = StrokeSpec::new(Rgba8::new(9, 9, 9, 255), 2.0);
    s.paint = StrokePaint::Pattern(Box::new(fill()));
    s
}

/// ⭐⭐ **UM TRAÇO PODE CARREGAR UM PADRÃO** — o buraco inteiro que o plano 35 fecha.
#[test]
fn a_stroke_can_carry_a_pattern() {
    let s = stroke_com_padrao();
    assert!(s.pattern().is_some(), "o traco nao carrega o padrao");
    // CONTROLO: um traço sólido continua a não ter nenhum — senão o gate mediria o `Some` de tudo.
    assert!(
        StrokeSpec::new(Rgba8::new(1, 2, 3, 255), 1.0)
            .pattern()
            .is_none()
    );
}

/// ⭐ **A COR continua a ter resposta num traço com padrão** — a `fallback`, que é literalmente o que
/// ele pinta enquanto o ladrilho não resolve.
///
/// ⚠️ É esta porta que faz a troca de `color: Rgba8` por `StrokePaint` custar **um caractere** em
/// cada leitor que só quer uma cor (a swatch, o token, o `StrokeStyle` da shell), em vez de um
/// `match` espalhado por quinze ficheiros.
#[test]
fn the_stroke_colour_still_answers_for_a_patterned_stroke() {
    assert_eq!(stroke_com_padrao().color(), fill().fallback);
    assert_eq!(
        StrokeSpec::new(Rgba8::new(7, 8, 9, 255), 1.0).color(),
        Rgba8::new(7, 8, 9, 255)
    );
}

/// **O padrão do traço sobrevive ao save**, com a lei intacta.
#[test]
fn the_stroke_pattern_survives_the_save() {
    let s = stroke_com_padrao();
    let bytes = postcard::to_allocvec(&s).expect("serializa");
    let back: StrokeSpec = postcard::from_bytes(&bytes).expect("desserializa");
    assert_eq!(back, s);
    assert_eq!(
        back.pattern().map(|p| p.source),
        Some(fill().source),
        "a fonte da arte nao voltou"
    );
}

/// ⚠️ **Desvanecer um traço com padrão baixa a OPACIDADE dele** — a MESMA lei que o preenchimento
/// já obedecia, e por isso o mesmo gate do outro lado.
///
/// Escalar só a `fallback` faria a linha manter o ladrilho a cheio e clarear apenas o instante em
/// que ele ainda não resolveu, que é o contrário do que se vê.
#[test]
fn fading_a_patterned_stroke_dims_the_pattern_not_just_its_fallback() {
    let mut path = VecPath {
        verts: [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        stroke: Some(stroke_com_padrao()),
        ..VecPath::default()
    };
    path.id = VecPathId::default();
    let mut scene = VecScene::default();
    let id = scene.push_path(path);
    let bound = BoundStyle {
        path: id,
        alpha: Some(128),
        ..BoundStyle::default()
    };
    let drawn = scene.paths()[0].painted(Some(&bound));
    let p = drawn
        .stroke
        .as_ref()
        .and_then(StrokeSpec::pattern)
        .expect("padrao");
    assert!(
        (p.alpha - 128.0 / 255.0).abs() < 1e-6,
        "a opacidade do padrao do traco nao desceu: {}",
        p.alpha
    );
    assert_eq!(p.fallback.a, 128, "a cor de recurso desce junto");
}

/// ⚠️ **Um TOKEN de cor no traço substitui a tinta por uma COR** — a mesma lei que a linha do
/// preenchimento já obedecia (ela troca o `Paint` inteiro por um `Solid`).
///
/// Pintar só a `fallback` de um padrão seria escolher uma cor que ninguém vê.
#[test]
fn a_colour_token_on_a_patterned_stroke_replaces_the_paint() {
    let mut path = VecPath {
        verts: [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        stroke: Some(stroke_com_padrao()),
        ..VecPath::default()
    };
    path.id = VecPathId::default();
    let mut scene = VecScene::default();
    let id = scene.push_path(path);
    let tok = Rgba8::new(200, 30, 40, 255);
    let bound = BoundStyle {
        path: id,
        stroke: Some(tok),
        ..BoundStyle::default()
    };
    let drawn = scene.paths()[0].painted(Some(&bound));
    let s = drawn.stroke.as_ref().expect("traco");
    assert!(s.pattern().is_none(), "o padrao sobreviveu ao token");
    assert_eq!(s.color(), tok);
}

// ───────────── o TIJOLO nasce MORTO (2026-08-30) ─────────────

/// ⭐⭐⭐ **UM PADRÃO QUE NASCE TIJOLO TEM DE LADRILHAR COMO UM TIJOLO.**
///
/// ⛔⛔ **Dois dos quatro reticulados estavam MORTOS AO NASCER**, e a cadeia é aritmética:
/// [`PatternFill::new`] nascia com `offset_denom = 1`, e a
/// [`ph2d_vec_pattern::TileLaw::period`] devolve `offset_denom.max(1)` para os tijolos ⇒ `1` ⇒
/// `cells() = [1, 1]` ⇒ **o ladrilho assado sai byte-idêntico ao da grade**. O artista carrega em
/// *Brick* e vê uma grade.
///
/// ⚠️ **A `Hex` escapava porque o braço dela devolve `2` sem olhar o campo** — é isso que fazia o
/// defeito parecer impossível: dos quatro chips, dois funcionavam.
///
/// ⚠️⚠️ **E o painel não o podia curar: a faixa do slider do *Offset* começa em `2`**
/// (`TEXPAT_DENOM_MIN`), então o `1` que o documento tinha era **inalcançável pelo controlo** —
/// e a fileira pintava `"1/2"` a partir do valor semeado no store, não do documento. *O artista
/// lia «1/2», via uma grade, e não tinha gesto nenhum que o tirasse de lá.*
///
/// ⛔⛔⛔ **E a cena de smoke ESCONDIA-O**: a `=76` põe `f.offset_denom = 2` à mão, então ela
/// demonstrava tijolos e colmeias a funcionar sobre um produto em que o chip era inerte. *Uma cena
/// que compensa o defeito do produto aprova-o.*
#[test]
fn a_pattern_born_as_a_brick_actually_tiles_as_one() {
    let px = [16u32, 16];
    for kind in [TileKind::BrickRow, TileKind::BrickCol] {
        let mut p = PatternFill::new(src(), [1.0, 1.0], Rgba8::new(0, 0, 0, 255));
        p.kind = kind;
        let lei = p.law(px);
        assert!(
            lei.period() > 1,
            "{kind:?} nasce com periodo {} - o ladrilho e' byte-identico ao da grade e o chip nao \
             muda um pixel",
            lei.period()
        );
        assert_ne!(
            lei.cells(),
            [1, 1],
            "{kind:?} nasce a assar UMA celula - e' uma grade com outro nome"
        );
    }
}

/// ⭐⭐ **CONTROLO: a GRADE continua a ser uma célula, e a COLMEIA continua a ser duas.**
///
/// ⚠️ É a metade que impede a cura de ser *"pôr tudo a 2"*: o `offset_denom` é **inerte** nos dois
/// reticulados que não desfasam (o `period()` deles não o lê), então mudar o nascimento não pode
/// mexer num pixel deles. Sem esta linha, uma cura que mudasse a grade passaria.
#[test]
fn the_grid_and_the_hex_are_untouched_by_where_the_offset_is_born() {
    let px = [16u32, 16];
    let base = PatternFill::new(src(), [1.0, 1.0], Rgba8::new(0, 0, 0, 255));
    for (kind, esperado) in [(TileKind::Grid, [1u32, 1]), (TileKind::Hex, [1, 2])] {
        let mut p = base.clone();
        p.kind = kind;
        assert_eq!(p.law(px).cells(), esperado, "{kind:?} mudou de ladrilho");
        // E mexer no campo continua a não os tocar.
        let mut outro = p.clone();
        outro.offset_denom = 7;
        assert_eq!(
            outro.law(px).cells(),
            esperado,
            "{kind:?} passou a ler o campo"
        );
    }
}

/// ⚠️ **O valor em que ele nasce tem de estar DENTRO do que o painel exprime.**
///
/// O slider do *Offset* vai de `2` a `8` (`ph2d-panel-vector`, `TEXPAT_DENOM_MIN`/`MAX`), e um
/// documento que nasce fora dessa faixa é **estado inalcançável**: o controlo não o mostra e não o
/// pode devolver. *Um modelo que aceita o que o painel não exprime produz estado que o artista não
/// consegue desfazer* — a mesma lei que o `ANIM_TAGS_MAX` da §11 do Sprite já pagou.
#[test]
fn the_offset_is_born_inside_the_range_the_panel_can_express() {
    let d = PatternFill::new(src(), [1.0, 1.0], Rgba8::new(0, 0, 0, 255)).offset_denom;
    assert!(
        (2..=8).contains(&d),
        "o padrao nasce com offset_denom = {d}, fora da faixa 2..=8 do slider - o artista nao tem \
         gesto que o alcance"
    );
}
