//! Gates do produtor da PILHA de FX raster (plano 24 W2) — a metade que roda **sem GPU**.
//!
//! O que se prova aqui é a FRONTEIRA: o que o componente autorado (mundo, com degraus ligados e
//! desligados) vira quando chega ao passe (pixels), e que os ids por-linha do painel decodificam
//! de volta no controle certo. O desenho em si é dos gates de GPU (`ph2d-render`), e o gesto é do
//! seam do painel — três lentes, três arquivos.

use super::{FilterHit, hit_of, resolve_ops};
use ph2d_ecs::{FxOp, VecFilter};
use ph2d_vector::Affine;

fn op(kind: u8, radius: f32) -> FxOp {
    FxOp {
        radius,
        ..FxOp::new(kind)
    }
}

/// **O painel e o motor concordam sobre os TETOS.** O painel não alcança o `ph2d-ecs` (vive de
/// snapshots), então os dois números são cópias — e uma cópia menor deixa os últimos degraus
/// inalcançáveis (o `.take()` do paint os corta), sem erro nenhum. Este é o único lugar do repo
/// que vê os dois lados.
#[test]
fn the_panel_and_the_engine_agree_on_the_ceilings() {
    assert_eq!(
        ph2d_editor::ids::MAX_FILTER_ROWS,
        VecFilter::MAX_OPS,
        "o teto de LINHAS do painel tem de bater com o `VecFilter::MAX_OPS`"
    );
    assert_eq!(
        ph2d_editor::ids::MAX_FILTER_KINDS,
        FxOp::KINDS,
        "o teto de TIPOS do painel tem de bater com o `FxOp::KINDS`"
    );
    assert_eq!(
        ph2d_editor::ids::MAX_FILTER_BLENDS,
        usize::from(FxOp::BLEND_KINDS),
        "o teto de LEIS DE MISTURA do painel tem de bater com o `FxOp::BLEND_KINDS` — um teto \
         menor deixa as ultimas leis sem opcao no popover, em silencio"
    );
    assert_eq!(
        ph2d_panel_vector::FILTER_DETAIL_MAX,
        f64::from(FxOp::MAX_DETAIL),
        "o teto de OITAVAS do painel tem de bater com o `FxOp::MAX_DETAIL` — um teto maior deixa \
         o artista pedir oitavas que o produtor clampa em silencio, e o slider mente"
    );
    // ⚠️ **O teto de STOPS: esta asserção NÃO é a defesa primária, e dizer o contrário seria vender
    // um gate que não pode falhar.** Medido: baixar o `MAX_FILTER_STOPS` para 4 **não compila** — o
    // snapshot do painel declara `[f32; MAX_FILTER_STOPS]` e a shell o preenche de um
    // `[_; MAX_GRADIENT_STOPS]`, então os dois tamanhos são o MESMO tipo e o compilador os casa. Ela
    // fica como cinto: se o snapshot algum dia virar `Vec` (desacoplando os tamanhos), é ela que
    // nomeia o porquê — um teto menor deixa os últimos stops sem punho (autorados e inalcançáveis),
    // um maior faz o painel pintar punhos que o uniform não carrega.
    assert_eq!(
        ph2d_editor::ids::MAX_FILTER_STOPS,
        FxOp::MAX_GRADIENT_STOPS,
        "os tetos de STOPS divergiram — e como os arrays sao tipados por eles, isto so e alcancavel \
         se o snapshot deixou de ser um array de tamanho fixo"
    );
    let widest = FxOp::SPECS.iter().map(|s| s.modes.len()).max().unwrap_or(0);
    assert!(
        widest <= ph2d_editor::ids::MAX_FILTER_MODES,
        "o teto de MODOS do painel ({}) nao cobre o tipo mais largo ({widest}) — os ultimos \
         modos ficariam sem chip, em silencio",
        ph2d_editor::ids::MAX_FILTER_MODES
    );
}

/// **Cada slot de cor recebe a escolha do picker, e SÓ ele.**
///
/// ⚠️ **Este gate existe porque o arch-gate irmão não conseguia prová-lo.** A rota morava dentro do
/// `render_frame` (window-gated), então o que a cobria era uma varredura do FONTE — e uma varredura
/// vê forma, não comportamento: a mutação que dobrava o stop na ponta escura **manteve o nome
/// `ColourSlot::SelectedStop` num braço inalcançável** e passou verde. Extraída para uma função
/// pura, a rota é observável e a mutação sangra aqui.
#[test]
fn each_colour_slot_gets_the_picked_colour_and_only_it() {
    // ⚠️ `ColourSlot` vem do módulo que o DEFINE, não do re-export: o produto recebe um valor dele
    // (do `colour_target`) e o passa adiante sem nunca nomear o tipo, então um `pub(crate) use` só
    // para o teste seria um import que o build sem testes reporta como morto.
    use crate::fx_live::apply_picked_colour;
    use crate::fx_live_hit::ColourSlot;
    const PICKED: [f32; 4] = [0.25, 0.5, 0.75, 1.0];
    let base = {
        let mut op = FxOp::new(FxOp::GRADIENT_MAP);
        op.stop_count = 3;
        op.color = [1.0, 0.0, 0.0, 1.0];
        op.color_b = [0.0, 1.0, 0.0, 1.0];
        op.stops[0] = [0.1, 0.1, 0.1, 1.0];
        op.stops[1] = [0.2, 0.2, 0.2, 1.0];
        op.stops[2] = [0.3, 0.3, 0.3, 1.0];
        op
    };
    // O stop em foco é o do MEIO — nem 0 nem o último, senão um off-by-one acertaria por acidente.
    let mut op = base;
    apply_picked_colour(&mut op, ColourSlot::SelectedStop, 1, PICKED);
    assert_eq!(op.stops[1], PICKED, "o stop em foco nao recebeu a cor");
    assert_eq!(op.color, base.color, "pintar um STOP mexeu na ponta escura");
    assert_eq!(
        op.color_b, base.color_b,
        "pintar um STOP mexeu na ponta clara"
    );
    assert_eq!(
        op.stops[0], base.stops[0],
        "pintar o stop 1 mexeu no stop 0"
    );
    assert_eq!(
        op.stops[2], base.stops[2],
        "pintar o stop 1 mexeu no stop 2"
    );

    let mut op = base;
    apply_picked_colour(&mut op, ColourSlot::First, 1, PICKED);
    assert_eq!(op.color, PICKED);
    assert_eq!(
        op.stops[1], base.stops[1],
        "a ponta escura escreveu num STOP"
    );

    let mut op = base;
    apply_picked_colour(&mut op, ColourSlot::Second, 1, PICKED);
    assert_eq!(op.color_b, PICKED);
    assert_eq!(
        op.stops[1], base.stops[1],
        "a ponta clara escreveu num STOP"
    );

    // E um foco OBSOLETO (a rampa encolheu desde o clique) pousa no último stop VIVO, nunca fora.
    let mut op = base;
    apply_picked_colour(&mut op, ColourSlot::SelectedStop, 7, PICKED);
    assert_eq!(
        op.stops[2], PICKED,
        "um foco obsoleto nao foi clampado a contagem viva — a cor cairia num slot que a rampa \
         nao usa, e o artista veria o picker nao fazer nada"
    );
}

/// **Acrescentar um stop NÃO muda o desenho** — a lei do `+`, e a razão de ele nascer no maior vão
/// com a cor que a rampa já tem ali.
///
/// O oráculo é a rampa AMOSTRADA (`ramp_preview`), que é a mesma função que o trilho pinta e cujo
/// acordo com o dispositivo já está medido em 1 nível de byte. Se o `+` mudasse a arte, o artista
/// clicaria para ganhar um ponto de controle e receberia uma edição que não pediu.
#[test]
fn adding_a_stop_does_not_change_the_ramp() {
    let mut op = FxOp::new(FxOp::GRADIENT_MAP);
    // Uma rampa de três, com vãos DESIGUAIS: o `+` tem de escolher o maior, e com vãos iguais o
    // gate não distinguiria "o maior" de "o primeiro".
    op.stop_count = 3;
    op.stops[0] = [0.0, 0.0, 0.0, 1.0];
    op.stops[1] = [0.9, 0.2, 0.2, 1.0];
    op.stops[2] = [1.0, 1.0, 1.0, 1.0];
    op.stop_pos = [0.0, 0.2, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    let before = crate::fx_live::ramp_preview(&op);
    crate::fx_live::add_stop(&mut op);
    assert_eq!(op.stop_count, 4, "o `+` nao acrescentou um stop");
    // O vão maior é `0,2..1,0` ⇒ o stop novo cai em 0,6.
    assert!(
        (op.stop_pos[3] - 0.6).abs() < 1e-6,
        "o stop novo caiu em {} — nao no meio do MAIOR vao (0,2..1,0), entao ele pode nascer \
         debaixo de outro punho e ficar inalcancavel",
        op.stop_pos[3]
    );
    let after = crate::fx_live::ramp_preview(&op);
    let worst = before
        .iter()
        .zip(&after)
        .flat_map(|(a, b)| (0..3).map(move |c| i32::from(a[c]).abs_diff(i32::from(b[c]))))
        .max()
        .unwrap_or(0);
    assert!(
        worst <= 1,
        "acrescentar um stop moveu a rampa em {worst} nivel(is) — o `+` esta a EDITAR a arte, e o \
         artista so pediu um ponto de controle"
    );
    // ⚠️ **E em SMOOTH ele NÃO é neutro — por construção, não por defeito.** O easing é por
    // SEGMENTO, então dividir um segmento reforma a curva; nenhuma escolha de cor conserta isso.
    // Este meio-gate existe para o número ficar MEDIDO em vez de ser descoberto como bug, e porque
    // foi ele que expôs o default errado: o `BLANK` compartilhado nasce em `MODE_CONTOUR` (= 1), e
    // `1` aqui é *Smooth* — o Gradient Map nascia com easing que ninguém pediu.
    let mut eased = FxOp::new(FxOp::GRADIENT_MAP);
    eased.mode = 1;
    eased.stop_count = 3;
    eased.stops[0] = [0.0, 0.0, 0.0, 1.0];
    eased.stops[1] = [0.9, 0.2, 0.2, 1.0];
    eased.stops[2] = [1.0, 1.0, 1.0, 1.0];
    eased.stop_pos = [0.0, 0.2, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    let smooth_before = crate::fx_live::ramp_preview(&eased);
    crate::fx_live::add_stop(&mut eased);
    let smooth_after = crate::fx_live::ramp_preview(&eased);
    let smooth_worst = smooth_before
        .iter()
        .zip(&smooth_after)
        .flat_map(|(a, b)| (0..3).map(move |c| i32::from(a[c]).abs_diff(i32::from(b[c]))))
        .max()
        .unwrap_or(0);
    assert!(
        smooth_worst > 8,
        "em Smooth o `+` moveu a rampa em apenas {smooth_worst} nivel(is) — se ele passou a ser \
         neutro ali, o easing deixou de ser por-SEGMENTO, e o modo perdeu o que o distingue"
    );
    assert_eq!(
        FxOp::new(FxOp::GRADIENT_MAP).mode,
        0,
        "um Gradient Map novo nasceu em Smooth — o `BLANK` compartilhado tem `MODE_CONTOUR` (= 1) e \
         aqui `1` e outro modo; o default TEM de ser declarado, senao o artista ganha um easing \
         que nao pediu e o `+` deixa de ser neutro"
    );
}

/// **O `−` tem PISO em dois, e o piso é a definição** — abaixo dele a rampa cai numa lei diferente
/// (o ramo vazio do `gradient_sample`, 73 níveis de byte distante do default de dois stops), e o
/// artista atravessaria uma descontinuidade que nada na tela explica.
#[test]
fn removing_stops_stops_at_two() {
    let mut op = FxOp::new(FxOp::GRADIENT_MAP);
    op.stop_count = 3;
    op.stop_pos = [0.0, 0.5, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    op.stops[1] = [0.9, 0.2, 0.2, 1.0];
    // Remover o do MEIO desloca os de cima para baixo — o índice de autoria dos que sobram muda, e
    // é por isso que a seleção do painel é clampada.
    crate::fx_live::remove_stop(&mut op, 1);
    assert_eq!(op.stop_count, 2);
    assert!(
        (op.stop_pos[1] - 1.0).abs() < 1e-6,
        "o stop que sobrou nao desceu de slot ({}) — o array deixou um buraco",
        op.stop_pos[1]
    );
    // E agora o piso morde: mais nenhum sai.
    for _ in 0..3 {
        crate::fx_live::remove_stop(&mut op, 0);
    }
    assert_eq!(
        op.stop_count, 2,
        "o `-` furou o piso de dois — abaixo dele a rampa muda de LEI, nao de forma"
    );
}

/// **Um degrau DESLIGADO nunca chega ao passe** — a pilha o salta, como a de geometria salta um
/// `FxEntry` desarmado. E uma pilha TODA desligada resolve para vazio, que é o que faz a forma
/// voltar a desenhar-se a si mesma: o produtor não cria imagem nenhuma.
#[test]
fn a_disabled_op_never_reaches_the_pass() {
    let mut f = VecFilter {
        ops: vec![op(FxOp::BLUR, 0.2), op(FxOp::GLOW, 0.3)],
    };
    assert_eq!(resolve_ops(&f, Affine::IDENTITY).len(), 2);
    f.ops[0].enabled = false;
    let ops = resolve_ops(&f, Affine::IDENTITY);
    assert_eq!(ops.len(), 1, "o degrau desligado tem de ser saltado");
    assert_eq!(
        ops[0].kind,
        FxOp::GLOW,
        "e o que sobra é o que ficou LIGADO"
    );
    f.ops[1].enabled = false;
    assert!(
        resolve_ops(&f, Affine::IDENTITY).is_empty() && !f.is_active(),
        "pilha toda desligada resolve para VAZIO — a forma sai nua"
    );
}

/// **O raio é de MUNDO e vira pixels pela CÂMERA** — é isto que torna o filtro *resolution-crisp*:
/// dar zoom aumenta o borrão na tela, na mesma proporção. Guardar pixels congelaria o efeito num
/// nível de zoom.
#[test]
fn the_world_radius_becomes_pixels_through_the_camera() {
    let f = VecFilter {
        ops: vec![op(FxOp::BLUR, 0.5)],
    };
    let one = resolve_ops(&f, Affine::IDENTITY)[0].sigma_px;
    let two = resolve_ops(&f, Affine::scale(2.0))[0].sigma_px;
    assert!(
        (one - 0.5).abs() < 1e-4,
        "sem zoom, o sigma em pixels é o raio de mundo (deu {one})"
    );
    assert!(
        (two - 2.0 * one).abs() < 1e-4,
        "a 2× de zoom o borrão tem de DOBRAR na tela (deu {two} contra {one})"
    );
}

/// **O deslocamento da sombra também cruza a câmera** (é um vetor de mundo), e chega ao passe em
/// pixels INTEIROS — o halo é amostrado por `textureLoad`, sem sampler.
#[test]
fn the_shadow_offset_crosses_the_camera_and_lands_on_whole_pixels() {
    let f = VecFilter {
        ops: vec![FxOp {
            offset: [0.1, -0.25],
            ..op(FxOp::DROP_SHADOW, 0.05)
        }],
    };
    let at10 = resolve_ops(&f, Affine::scale(10.0))[0].offset_px;
    assert_eq!(
        at10,
        [1, -3],
        "0.1 e -0.25 de mundo a 10× dão 1 e -2.5 px, arredondados ao pixel"
    );
}

/// **`hit_of` decodifica cada controle da pilha, e só eles.** É a porta única que os TRÊS sítios da
/// ponte usam (o comando, o valor, o alvo do picker); se ela confundisse duas linhas, um slider da
/// linha 2 editaria a linha 0 e nada pareceria quebrado.
#[test]
fn hit_of_decodes_every_row_control_and_nothing_else() {
    use ph2d_editor::ids as vid;
    for k in 0..FxOp::KINDS {
        #[allow(clippy::cast_possible_truncation)]
        let want = FilterHit::Add(k as u8);
        assert_eq!(hit_of(vid::filter_add_id(k)), Some(want));
    }
    for r in 0..VecFilter::MAX_OPS {
        for m in 0..vid::MAX_FILTER_MODES {
            #[allow(clippy::cast_possible_truncation)]
            let want = FilterHit::Mode(r, m as u8);
            assert_eq!(hit_of(vid::filter_mode_id(r, m)), Some(want));
        }
        assert_eq!(hit_of(vid::filter_remove_id(r)), Some(FilterHit::Remove(r)));
        assert_eq!(hit_of(vid::filter_up_id(r)), Some(FilterHit::Up(r)));
        assert_eq!(hit_of(vid::filter_down_id(r)), Some(FilterHit::Down(r)));
        assert_eq!(hit_of(vid::filter_hide_id(r)), Some(FilterHit::Hide(r)));
        assert_eq!(hit_of(vid::filter_color_id(r)), Some(FilterHit::Color(r)));
        assert_eq!(
            hit_of(vid::filter_color_b_id(r)),
            Some(FilterHit::ColorB(r))
        );
        assert_eq!(hit_of(vid::filter_radius_id(r)), Some(FilterHit::Radius(r)));
        assert_eq!(hit_of(vid::filter_offx_id(r)), Some(FilterHit::OffX(r)));
        assert_eq!(hit_of(vid::filter_offy_id(r)), Some(FilterHit::OffY(r)));
        assert_eq!(
            hit_of(vid::filter_opacity_id(r)),
            Some(FilterHit::Opacity(r))
        );
    }
    // E um id de OUTRA seção não é da pilha — senão a ponte roubaria eventos alheios.
    assert_eq!(hit_of(vid::VECTOR_SECTION_FILTERS), None);
    assert_eq!(hit_of(vid::VECTOR_STROKE_SWATCH), None);
    // Nem o campo NUMÉRICO gêmeo (ele viaja por outro canal, e confundi-lo com o slider faria a
    // ponte editar duas vezes o mesmo valor).
    assert_eq!(hit_of(vid::filter_radius_num_id(0)), None);
}

/// **As duas pontas da rampa são alvos de picker DISTINTOS, e a porta única as separa.**
///
/// ⚠️ O readback do picker é o ÚNICO consumidor que precisa saber QUAL swatch o artista abriu — e o
/// modo de falha de errar é mudo: escolher a segunda ponta escreveria na primeira, com a UI a
/// mostrar a cor certa no card errado. A resposta vem do ID do alvo, nunca do `kind` do degrau.
#[test]
fn the_three_colour_swatches_are_distinct_picker_targets() {
    use crate::fx_live_hit::ColourSlot;
    use ph2d_editor::ids as vid;
    for r in 0..VecFilter::MAX_OPS {
        assert_eq!(
            crate::fx_live::colour_target(vid::filter_color_id(r)),
            Some((r, ColourSlot::First))
        );
        assert_eq!(
            crate::fx_live::colour_target(vid::filter_color_b_id(r)),
            Some((r, ColourSlot::Second))
        );
        // ⚠️ **A TERCEIRA, e é a que o comentário do readback já previa:** *"derivar a ponta do
        // `kind` faria a segunda escrever na primeira em qualquer tipo que ganhasse uma rampa
        // depois"*. Ela existe agora, e o slot é o que a distingue — um `bool` dobraria o stop na
        // ponta escura em silêncio.
        assert_eq!(
            crate::fx_live::colour_target(vid::filter_stop_color_id(r)),
            Some((r, ColourSlot::SelectedStop))
        );
    }
    // Um controle que NÃO é cor não é alvo de picker — senão arrastar um slider abriria o OKLCH.
    assert_eq!(
        crate::fx_live::colour_target(vid::filter_radius_id(0)),
        None
    );
    assert_eq!(
        crate::fx_live::colour_target(vid::VECTOR_STROKE_SWATCH),
        None
    );
}

/// **A SEGUNDA cor atravessa o produtor sem passar pela câmara.**
///
/// Uma cor não é um comprimento: dar zoom não pode mudar a paleta. É a mesma afirmação que o irmão
/// dos três knobs de ajuste faz, e ela existe porque o `resolve_ops` multiplica por `cam_scale`
/// TUDO o que é comprimento — e a linha de cima (`tint`) fica ao lado.
#[test]
fn the_second_colour_crosses_the_camera_unscaled() {
    let mut op = FxOp::new(FxOp::DUOTONE);
    op.color = [0.1, 0.2, 0.3, 1.0];
    op.color_b = [0.7, 0.8, 0.9, 0.5];
    let f = VecFilter::single(op);
    for zoom in [0.25f64, 1.0, 4.0] {
        let out = resolve_ops(&f, Affine::scale(zoom));
        assert_eq!(
            out[0].tint, op.color,
            "a ponta escura mudou com o zoom {zoom}"
        );
        assert_eq!(
            out[0].tint_b, op.color_b,
            "a ponta clara mudou com o zoom {zoom}"
        );
    }
}

/// **A LEI DE MISTURA atravessa o produtor, e a que não é tomada vira Normal.**
///
/// É a metade de HONRAR da porta única `FxOp::takes_blend` — o painel tem a de OFERECER. Sem esta,
/// um degrau cuja lei sobreviveu a uma mudança de tipo (ou um arquivo de outra versão) mandaria ao
/// dispositivo um número que a UI não mostra, e a forma desenharia uma mistura que o artista não
/// pediu nem consegue ver de onde veio.
#[test]
fn the_law_reaches_the_pass_only_for_a_kind_that_takes_one() {
    // Screen (6) em TODO tipo — os que tomam a lei a levam, os outros a perdem.
    for kind in 0..FxOp::KINDS as u8 {
        let mut o = op(kind, 0.2);
        o.blend = 6;
        let f = VecFilter { ops: vec![o] };
        let got = crate::fx_live::resolve_ops(&f, Affine::IDENTITY);
        assert_eq!(got.len(), 1, "{}", FxOp::kind_name(kind));
        let want = if FxOp::spec(kind).takes_blend { 6 } else { 0 };
        assert_eq!(
            got[0].blend,
            want,
            "{}: takes_blend={} mas o passe recebeu {}",
            FxOp::kind_name(kind),
            FxOp::spec(kind).takes_blend,
            got[0].blend
        );
    }
}

/// **O decodificador conhece as opções de mistura, e só elas.** Espelho do irmão que varre os
/// controles de linha: um id que ele não decodifica é um clique que a ponte descarta em silêncio.
#[test]
fn hit_of_decodes_every_blend_option() {
    use crate::fx_live::{FilterHit, hit_of};
    for r in 0..ph2d_editor::ids::MAX_FILTER_ROWS {
        for m in 0..ph2d_editor::ids::MAX_FILTER_BLENDS {
            let id = ph2d_editor::ids::filter_blend_option_id(r, m);
            assert_eq!(
                hit_of(id),
                Some(FilterHit::Blend(r, m as u8)),
                "a opcao {m} da linha {r} nao decodifica"
            );
        }
        // ⚠️ O CHIP nao e uma opcao — ele e um `Dropdown`, e abrir/fechar e do dispatch generico.
        // Decodifica-lo aqui faria o clique de ABRIR virar uma edicao da pilha.
        assert_eq!(
            hit_of(ph2d_editor::ids::filter_blend_id(r)),
            None,
            "o CHIP de mistura da linha {r} nao pode decodificar como edicao"
        );
    }
}

/// **Os três knobs do RUÍDO atravessam a fronteira, e o Detail atravessa CLAMPADO.**
///
/// O `detail_clamped` é a metade de HONRAR da porta única (a de OFERECER é do painel): um arquivo
/// — ou um teste — com detalhe `0` faria o laço do `fbm` não correr uma vez, e a turbulência
/// desenharia NADA com todos os knobs preenchidos.
#[test]
fn the_noise_knobs_reach_the_pass_and_the_detail_arrives_clamped() {
    let mut o = op(FxOp::TURBULENCE, 0.2);
    o.scale = 0.5;
    o.seed = 42;
    for (given, want) in [(0u8, 1u8), (1, 1), (3, 3), (200, FxOp::MAX_DETAIL)] {
        o.detail = given;
        let got = crate::fx_live::resolve_ops(&VecFilter { ops: vec![o] }, Affine::IDENTITY);
        assert_eq!(
            got[0].detail, want,
            "detalhe {given} chegou ao passe como {}",
            got[0].detail
        );
        assert_eq!(got[0].seed, 42, "a semente nao atravessou");
        assert!(
            (got[0].noise_scale_px - 0.5).abs() < 1e-6,
            "o tamanho nao atravessou: {}",
            got[0].noise_scale_px
        );
    }
}

/// **O decodificador conhece os três knobs do ruído.** Um id que ele não decodifica é um arrasto
/// que a ponte descarta em silêncio — o slider vivo, o valor no lixo.
#[test]
fn hit_of_decodes_the_three_noise_knobs() {
    use crate::fx_live::{FilterHit, hit_of};
    for r in 0..ph2d_editor::ids::MAX_FILTER_ROWS {
        for (id, want) in [
            (ph2d_editor::ids::filter_scale_id(r), FilterHit::Scale(r)),
            (ph2d_editor::ids::filter_detail_id(r), FilterHit::Detail(r)),
            (ph2d_editor::ids::filter_seed_id(r), FilterHit::Seed(r)),
        ] {
            assert_eq!(
                hit_of(id),
                Some(want),
                "a linha {r} perdeu um knob de ruido"
            );
        }
    }
}

/// **O crescimento atravessa a câmera COM O SINAL.** Ele é um comprimento de MUNDO como o raio, e
/// o zoom o converte em pixels — mas ao contrário do raio ele pode ser negativo, e é o sinal que
/// diz se a silhueta engorda ou afina. Um `max(0.0)` no caminho (a forma como todo outro
/// comprimento desta pilha é resolvido) apagaria metade da operação em silêncio.
#[test]
fn the_grow_crosses_the_camera_with_its_sign() {
    let mut o = op(FxOp::MORPHOLOGY, 0.0);
    for world in [-0.5_f32, -0.06, 0.0, 0.06, 0.5] {
        o.grow = world;
        let got = crate::fx_live::resolve_ops(&VecFilter { ops: vec![o] }, Affine::scale(3.0));
        let want = world * 3.0;
        assert!(
            (got[0].grow_px - want).abs() < 1e-4,
            "grow {world} sob zoom 3 devia chegar como {want}, chegou {}",
            got[0].grow_px
        );
    }
}

/// **Os três do ajuste atravessam a câmara SEM escala — e o "sem" é a afirmação.**
///
/// ⚠️ Irmão do gate acima e o contrário dele: o `grow` é um COMPRIMENTO e tem de ser multiplicado
/// pelo zoom; a matiz é um ÂNGULO e a saturação/brilho são FRAÇÕES. Dar zoom não pode mudar a cor
/// de nada, e multiplicá-los pelo `cam_scale` é o mesmo erro que dividir o raio por ele — só que
/// invisível até alguém dar zoom.
#[test]
fn the_adjust_knobs_cross_the_camera_unscaled() {
    let mut o = op(FxOp::COLOR_ADJUST, 0.0);
    for (h, s, b) in [(0.25_f32, 0.4_f32, -0.3_f32), (-0.5, -1.0, 1.0)] {
        o.hue = h;
        o.sat = s;
        o.bright = b;
        for zoom in [1.0_f64, 3.0, 0.25] {
            let got = crate::fx_live::resolve_ops(&VecFilter { ops: vec![o] }, Affine::scale(zoom));
            assert!(
                (got[0].hue - h).abs() < 1e-6
                    && (got[0].sat - s).abs() < 1e-6
                    && (got[0].bright - b).abs() < 1e-6,
                "sob zoom {zoom} o ajuste ({h}, {s}, {b}) chegou como ({}, {}, {}) — ele nao e' \
                 um comprimento",
                got[0].hue,
                got[0].sat,
                got[0].bright
            );
        }
    }
}

/// **O decodificador conhece o Amount do Grow / Shrink.** Um id que ele não decodifica é um arrasto
/// que a ponte descarta em silêncio — o slider anda e o documento não.
#[test]
fn hit_of_decodes_the_grow_knob() {
    use crate::fx_live::{FilterHit, hit_of};
    for r in 0..ph2d_editor::ids::MAX_FILTER_ROWS {
        assert_eq!(
            hit_of(ph2d_editor::ids::filter_grow_id(r)),
            Some(FilterHit::Grow(r)),
            "o Amount da linha {r} nao e' decodificado"
        );
        for (id, want) in [
            (ph2d_editor::ids::filter_hue_id(r), FilterHit::Hue(r)),
            (ph2d_editor::ids::filter_sat_id(r), FilterHit::Sat(r)),
            (ph2d_editor::ids::filter_bright_id(r), FilterHit::Bright(r)),
        ] {
            assert_eq!(
                hit_of(id),
                Some(want),
                "um knob de ajuste de cor da linha {r} nao e' decodificado"
            );
        }
    }
}
