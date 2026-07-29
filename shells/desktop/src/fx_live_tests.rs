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
    let widest = FxOp::SPECS.iter().map(|s| s.modes.len()).max().unwrap_or(0);
    assert!(
        widest <= ph2d_editor::ids::MAX_FILTER_MODES,
        "o teto de MODOS do painel ({}) nao cobre o tipo mais largo ({widest}) — os ultimos \
         modos ficariam sem chip, em silencio",
        ph2d_editor::ids::MAX_FILTER_MODES
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
    }
}
