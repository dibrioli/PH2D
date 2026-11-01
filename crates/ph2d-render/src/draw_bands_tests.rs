use super::*;
use Family::{Flip as F, Motion as M, Palco as K, Pixel as P, Sprite as S, Vector as V};

/// ⭐ **O caso do report**: uma forma desenhada primeiro, uma imagem importada a seguir. Na
/// hierarquia a imagem fica ABAIXO ⇒ ela desenha DEPOIS ⇒ por cima.
#[test]
fn the_report_case_puts_the_image_after_the_shape() {
    let (b, degraded) = plan(&[V, S]);
    assert!(!degraded);
    assert_eq!(
        b,
        vec![
            Band {
                family: V,
                lo: 0,
                hi: 1
            },
            Band {
                family: S,
                lo: 1,
                hi: 2
            },
        ],
        "o vetor tem de sair na PRIMEIRA faixa — e ate' hoje ele saia sempre na ultima"
    );
}

/// A ordem de sempre continua a ser exprimível: importar a imagem antes de desenhar.
#[test]
fn the_old_order_is_still_expressible() {
    let (b, _) = plan(&[S, V]);
    assert_eq!(b.len(), 2);
    assert_eq!(b[0].family, S);
    assert_eq!(b[1].family, V);
}

/// Ranks contíguos da mesma família colapsam **numa** passagem.
#[test]
fn a_run_of_the_same_family_is_one_pass() {
    let (b, _) = plan(&[S, S, S, V, V, S]);
    assert_eq!(b.len(), 3);
    assert_eq!((b[0].lo, b[0].hi), (0, 3));
    assert_eq!((b[1].lo, b[1].hi), (3, 5));
    assert_eq!((b[2].lo, b[2].hi), (5, 6));
}

/// ⚠️ **As faixas cobrem a ordem inteira, sem buraco e sem sobreposição** — uma passagem que
/// salta um rank perde um objecto, e nenhuma régua de contagem o veria.
#[test]
fn the_bands_tile_the_whole_order() {
    let fams = [S, V, S, S, V, S, V, V, S];
    let (b, _) = plan(&fams);
    let mut next = 0u32;
    for band in &b {
        assert_eq!(band.lo, next, "buraco ou sobreposicao nas faixas: {b:?}");
        assert!(band.hi > band.lo, "faixa vazia: {band:?}");
        next = band.hi;
    }
    assert_eq!(next as usize, fams.len(), "as faixas nao cobrem o fim");
}

/// Uma cena sem vetor nenhum é **uma** faixa — o caminho de sempre, sem custo novo.
#[test]
fn a_scene_with_no_vectors_is_a_single_pass() {
    let (b, degraded) = plan(&[S, S, S, S]);
    assert!(!degraded);
    assert_eq!(b.len(), 1);
    assert_eq!(b[0].family, S);
}

/// E uma cena vazia não pede passagem nenhuma.
#[test]
fn an_empty_order_asks_for_no_pass() {
    let (b, degraded) = plan(&[]);
    assert!(b.is_empty());
    assert!(!degraded);
}

/// ⛔ **Acima do teto a lei DEGRADA para a ordem de sempre**, e diz que degradou.
#[test]
fn beyond_the_ceiling_it_falls_back_to_the_phase_one_order_and_says_so() {
    let fams: Vec<Family> = (0..MAX_BANDS + 2)
        .map(|i| if i % 2 == 0 { S } else { V })
        .collect();
    assert!(
        bands(&fams).len() > MAX_BANDS,
        "a fixtura nao estoura o teto"
    );
    let (b, degraded) = plan(&fams);
    assert!(degraded, "estourou o teto e nao disse");
    assert_eq!(b.len(), 2, "a degradacao e' sprite-tudo, depois vetor-tudo");
    assert_eq!(b[0].family, S);
    assert_eq!(b[1].family, V);
    // ⚠️ E as duas faixas cobrem a ordem INTEIRA — o consumidor filtra por família.
    assert_eq!((b[0].lo, b[0].hi), (0, fams.len() as u32));
    assert_eq!((b[1].lo, b[1].hi), (0, fams.len() as u32));
}

/// Exactamente no teto **não** degrada — a cerca fica no lado perigoso, não nos dois.
#[test]
fn exactly_at_the_ceiling_it_does_not_degrade() {
    let fams: Vec<Family> = (0..MAX_BANDS)
        .map(|i| if i % 2 == 0 { S } else { V })
        .collect();
    assert_eq!(bands(&fams).len(), MAX_BANDS);
    assert!(!plan(&fams).1);
}

/// A degradação de uma cena só-de-sprites continua a ser **uma** faixa.
#[test]
fn the_fallback_of_a_sprite_only_scene_is_one_band() {
    assert_eq!(collapsed(&[S, S]).len(), 1);
    assert_eq!(collapsed(&[V, V]).len(), 1);
    assert!(collapsed(&[]).is_empty());
}

/// ⭐ **A ordem chega FORA DE ORDEM e o registo tem de a colocar por índice.** O ordenador
/// devolve `(entidade, rank)`, e a travessia que os regista é a do mundo, não a do rank.
#[test]
fn recording_out_of_order_still_lands_each_family_at_its_rank() {
    let mut o = FrameOrder::default();
    o.record(2, None);
    o.record(0, Some(7));
    o.record(1, None);
    assert_eq!(o.complete(), Some(vec![V, S, S]));
    assert_eq!(
        o.vector_ids_in(Band {
            family: V,
            lo: 0,
            hi: 1
        }),
        vec![7]
    );
    assert!(
        o.vector_ids_in(Band {
            family: V,
            lo: 1,
            hi: 3
        })
        .is_empty()
    );
}

/// Uma cena sem forma nenhuma diz que não tem — é o que manda o presente pelo caminho de
/// sempre, sem pagar faixa nenhuma.
#[test]
fn a_scene_with_no_shapes_says_so() {
    let mut o = FrameOrder::default();
    o.record(0, None);
    assert!(!o.has_vectors());
    o.record(1, Some(3));
    assert!(o.has_vectors());
}

/// `clear` esvazia as DUAS metades — deixar `vector_ranks` para trás faria uma forma apagada
/// continuar a pedir uma faixa, e a faixa desenharia o nada por cima de um sprite.
#[test]
fn clear_empties_both_halves() {
    let mut o = FrameOrder::default();
    o.record(0, Some(1));
    o.record(1, None);
    o.clear();
    assert!(o.families.is_empty());
    assert!(!o.has_vectors());
}

/// ⛔⛔ **UM BURACO TEM NOME.** Este é o gate que a primeira versão do modelo não conseguia
/// ter: com o preenchimento por omissão a `Sprite`, saltar um rank de sprite era invisível.
///
/// **Mutação que deve sangrar:** no `build_frame_order`, saltar o `record` quando o objecto
/// não é uma forma.
#[test]
fn a_rank_that_was_never_recorded_is_a_hole_and_the_plan_degrades() {
    let mut o = FrameOrder::default();
    // O rank 0 (um sprite) NÃO é registado; só o 1.
    o.record(1, Some(9));
    assert_eq!(o.complete(), None, "o buraco no rank 0 nao foi visto");
    let (b, degraded) = o.plan();
    assert!(
        degraded,
        "uma ordem com buraco tem de degradar, nao de adivinhar"
    );
    assert_eq!(b.len(), 2, "a degradacao e' a ordem da Fase 1");
    assert_eq!(b[0].family, S);
    assert_eq!(b[1].family, V);
}

/// E uma ordem sem buracos **não** degrada.
#[test]
fn a_complete_order_does_not_degrade() {
    let mut o = FrameOrder::default();
    o.record(0, None);
    o.record(1, Some(9));
    assert_eq!(o.complete(), Some(vec![S, V]));
    assert!(!o.plan().1);
}

/// ⭐ **O caminho de sempre NÃO é banded** — e isto é o que mantém todo o golden byte-idêntico.
#[test]
fn the_phase_one_orders_do_not_need_banding() {
    for fams in [
        vec![],
        vec![S],
        vec![V],
        vec![S, S],
        vec![S, V],
        vec![S, S, V, V],
    ] {
        let (b, _) = plan(&fams);
        assert!(!needs_banding(&b), "{fams:?} nao devia precisar de faixas");
    }
}

/// ⭐⭐ **O caso do report PRECISA** — e é a única diferença entre os dois conjuntos.
#[test]
fn the_report_case_needs_banding() {
    for fams in [vec![V, S], vec![S, V, S], vec![V, S, V]] {
        let (b, _) = plan(&fams);
        assert!(needs_banding(&b), "{fams:?} tinha de precisar de faixas");
    }
}

/// `doc_bands_of` devolve **só** as faixas de vetor, e vazio no caminho de sempre.
#[test]
fn doc_bands_are_the_vector_runs_and_empty_on_the_old_path() {
    let mut o = FrameOrder::default();
    o.record(0, None);
    o.record(1, Some(5));
    assert!(
        doc_bands_of(&o, false).is_empty(),
        "[S, V] e' o caminho de sempre"
    );
    // ⭐ Com as formas na placa o mesmo quadro manda o documento às faixas.
    assert_eq!(doc_bands_of(&o, true).len(), 1);

    let mut o = FrameOrder::default();
    o.record(0, Some(5));
    o.record(1, None);
    let b = doc_bands_of(&o, false);
    assert_eq!(b.len(), 1);
    assert_eq!(b[0].family, V);
    assert_eq!(o.vector_ids_in(b[0]), vec![5]);
}

/// ⭐⭐ **O clique lê a pilha do DESENHO** (dono 01/11): intercalado, a altura é o rank; degradado,
/// todo vetor fica por cima de todo sprite — como o presente o desenha.
#[test]
fn the_stack_key_is_the_drawn_stack_in_both_regimes() {
    let mut o = FrameOrder::default();
    // rank 0 vetor (bits 10) · rank 1 sprite (bits 11) · rank 2 vetor (bits 12)
    for (r, bits, path) in [(0u32, 10u64, Some(1u64)), (1, 11, None), (2, 12, Some(2))] {
        o.record(r, path);
        o.enrol(bits, r);
    }
    let (b, degraded) = o.plan();
    assert!(!degraded);
    let k = |bits| o.stack_key(&b, bits).expect("entrou na ordem");
    assert!(
        k(12) > k(11) && k(11) > k(10),
        "intercalado: a pilha e' o rank"
    );

    let b = collapsed(&[V, S, V]);
    let k = |bits| o.stack_key(&b, bits).expect("entrou na ordem");
    assert!(
        k(10) > k(11),
        "degradado: o vetor desenha-se por cima de todo sprite"
    );
    assert_eq!(o.stack_key(&b, 99), None, "fora da ordem nao tem altura");
}

/// ⭐ GEF 19 E2: três famílias — as faixas cobrem a ordem, o pixel intercala com o vetor.
#[test]
fn three_family_bands_tile_the_order() {
    let fams = [P, V, S, P, P, V];
    let (b, degraded) = plan(&fams);
    assert!(!degraded);
    let got: Vec<_> = b.iter().map(|b| (b.family, b.lo, b.hi)).collect();
    assert_eq!(
        got,
        vec![(P, 0, 1), (V, 1, 2), (S, 2, 3), (P, 3, 5), (V, 5, 6)]
    );
    assert!(needs_banding(&b));
}

/// A degradação (teto ou buraco) põe o pixel POR ÚLTIMO — o quadro de antes da E2.
#[test]
fn the_degraded_order_puts_pixels_last() {
    let fams: Vec<Family> = (0..MAX_BANDS + 3).map(|i| [P, V, S][i % 3]).collect();
    let (b, degraded) = plan(&fams);
    assert!(degraded);
    assert_eq!(
        b.iter().map(|b| b.family).collect::<Vec<_>>(),
        vec![S, V, P]
    );
    let mut o = FrameOrder::default();
    o.record_pixel(0, 40);
    o.record(2, Some(9));
    let (b, degraded) = o.plan();
    assert!(degraded, "rank 1 e' buraco");
    assert_eq!(
        b.iter().map(|b| b.family).collect::<Vec<_>>(),
        vec![S, V, P]
    );
}

/// A ordem de sempre com pixel por cima não pede faixas (byte-idêntico); fora dela, pede.
#[test]
fn the_old_stack_with_pixels_on_top_needs_no_banding() {
    for fams in [
        vec![P],
        vec![S, P],
        vec![V, P],
        vec![S, V, P],
        vec![S, S, V, P, P],
    ] {
        assert!(!needs_banding(&plan(&fams).0), "{fams:?}");
    }
    for fams in [vec![P, S], vec![P, V], vec![S, P, V], vec![V, P, S]] {
        assert!(needs_banding(&plan(&fams).0), "{fams:?}");
    }
}

/// `doc_bands_of` devolve as faixas de vetor E de pixel; `pixel_ids_in` vai por rank.
#[test]
fn doc_bands_carry_the_pixel_runs() {
    let mut o = FrameOrder::default();
    o.record_pixel(1, 21);
    o.record_pixel(0, 20);
    o.record(2, None);
    o.record(3, Some(5));
    let b = doc_bands_of(&o, false);
    assert_eq!(b.iter().map(|b| b.family).collect::<Vec<_>>(), vec![P, V]);
    assert_eq!(o.pixel_ids_in(b[0]), vec![20, 21]);
    assert!(o.vector_ids_in(b[0]).is_empty());
    assert_eq!(o.pixel_ids(), vec![20, 21]);
    o.clear();
    assert!(o.pixel_ids().is_empty());
}

/// O clique (`stack_key`) lê o pixel na pilha desenhada, nos dois regimes.
#[test]
fn the_stack_key_works_for_pixel_sprites() {
    let mut o = FrameOrder::default();
    o.record_pixel(0, 10);
    o.enrol(10, 0);
    o.record(1, Some(1));
    o.enrol(11, 1);
    let (b, _) = o.plan();
    let k = |b: &[Band], bits| o.stack_key(b, bits).expect("na ordem");
    assert!(
        k(&b, 11) > k(&b, 10),
        "intercalado: o pixel de rank 0 fica atras"
    );
    let c = collapsed(&[P, V]);
    assert!(k(&c, 10) > k(&c, 11), "degradado: o pixel por cima de tudo");
}

/// ⭐ GEF 19 E3: a quarta família — as corridas cruas cobrem a ordem e o Motion tem rank próprio.
#[test]
fn four_family_bands_tile_the_order() {
    let fams = [M, S, V, M, P];
    let got: Vec<_> = bands(&fams)
        .iter()
        .map(|b| (b.family, b.lo, b.hi))
        .collect();
    assert_eq!(
        got,
        vec![(M, 0, 1), (S, 1, 2), (V, 2, 3), (M, 3, 4), (P, 4, 5)]
    );
    assert!(!M.is_scene(), "o Motion e' placa, nao cena Vello");
}

/// Sem a placa o plano põe o Motion por cima de tudo e cola as vizinhas que ele partia — o
/// resto é o plano de antes (byte-idêntico); com a placa, a faixa fica no rank dela.
#[test]
fn the_plan_keeps_motion_on_top_and_the_rest_unchanged() {
    let mut o = FrameOrder::default();
    o.record(0, None);
    o.record_motion(1, 40);
    o.record(2, None);
    o.record(3, Some(9));
    let (b, degraded) = o.plan();
    assert!(!degraded);
    let got: Vec<_> = b.iter().map(|b| (b.family, b.lo, b.hi)).collect();
    assert_eq!(got, vec![(S, 0, 3), (V, 3, 4), (M, 0, 4)]);
    assert!(
        !needs_banding(&b),
        "[S, V] + Motion no topo = o caminho de sempre"
    );
    o.motion_na_placa = true;
    let got: Vec<_> = o.plan().0.iter().map(|b| (b.family, b.lo, b.hi)).collect();
    assert_eq!(got, vec![(S, 0, 1), (M, 1, 2), (S, 2, 3), (V, 3, 4)]);
    assert!(
        !needs_banding(&plan(&[S, V, M]).0),
        "Motion por ultimo = o de sempre"
    );
    assert!(needs_banding(&plan(&[M, S]).0));
    let fams: Vec<Family> = (0..MAX_BANDS + 3).map(|i| [M, V, S][i % 3]).collect();
    let (b, _) = plan(&fams);
    assert_eq!(b.last().map(|b| b.family), Some(M));
    let mut o = FrameOrder::default();
    o.record_motion(0, 40);
    o.record(2, Some(9));
    o.motion_na_placa = true;
    let (b, degraded) = o.plan();
    assert!(degraded, "rank 1 e' buraco");
    assert_eq!(
        b.iter().map(|b| b.family).collect::<Vec<_>>(),
        vec![S, V, M]
    );
}

/// O clique (`stack_key`) acha o objecto Motion — no rank dele com a placa, por cima sem ela.
#[test]
fn the_stack_key_works_for_motion_objects() {
    let mut o = FrameOrder::default();
    o.record_motion(0, 10);
    o.enrol(10, 0);
    o.record(1, None);
    o.enrol(11, 1);
    let (b, _) = o.plan();
    let k = |bits| o.stack_key(&b, bits).expect("na ordem");
    assert!(k(10) > k(11), "sem placa: por cima do mundo");
    assert_eq!(o.motion_ids_in(b[b.len() - 1]), vec![10]);
    o.motion_na_placa = true;
    let (b, _) = o.plan();
    let k = |bits| o.stack_key(&b, bits).expect("na ordem");
    assert!(k(10) < k(11), "com placa: o rank da Hierarchy");
    assert!(doc_bands_of(&o, true).iter().all(|b| b.family != M));
}

/// ⭐ GEF 19 E4: o Flip logo acima dos sprites é o `game_rt` de sempre (sem faixas); abaixo deles,
/// ou acima do vetor, intercala. A degradação e o buraco põem-no logo a seguir aos sprites.
#[test]
fn the_flip_sits_on_the_sprites_in_the_old_stack_and_bands_elsewhere() {
    assert!(
        !needs_banding(&plan(&[S, F, V, P, M]).0),
        "a ordem de sempre"
    );
    assert!(!needs_banding(&plan(&[F, V]).0));
    assert!(
        needs_banding(&plan(&[F, S]).0),
        "Flip por baixo das imagens"
    );
    assert!(needs_banding(&plan(&[S, V, F]).0), "Flip por cima do vetor");
    assert!(!F.is_scene(), "o Flip e' game_rt, nao cena Vello");
    let fams: Vec<Family> = (0..MAX_BANDS + 3).map(|i| [F, V, S][i % 3]).collect();
    let (b, degraded) = plan(&fams);
    assert!(degraded);
    assert_eq!(
        b.iter().map(|b| b.family).collect::<Vec<_>>(),
        vec![S, F, V]
    );
    let mut o = FrameOrder::default();
    o.record_flip(0, 77);
    o.record(2, None);
    let (b, degraded) = o.plan();
    assert!(degraded, "rank 1 e' buraco");
    assert_eq!(b.iter().map(|b| b.family).collect::<Vec<_>>(), vec![S, F]);
}

/// Os objectos Flip de uma faixa saem do de trás para o da frente; o clique acha-os pelo rank.
#[test]
fn the_flip_ids_of_a_band_come_back_to_front_and_the_click_finds_them() {
    let mut o = FrameOrder::default();
    o.record_flip(2, 70);
    o.enrol(1070, 2);
    o.record(1, None);
    o.enrol(11, 1);
    o.record_flip(0, 60);
    o.enrol(1060, 0);
    let (b, _) = o.plan();
    assert_eq!(
        b.iter().map(|b| (b.family, b.lo, b.hi)).collect::<Vec<_>>(),
        vec![(F, 0, 1), (S, 1, 2), (F, 2, 3)]
    );
    assert_eq!(o.flip_ids_in(b[0]), vec![60]);
    assert_eq!(o.flip_ids_in(b[2]), vec![70]);
    let all = Band {
        family: F,
        lo: 0,
        hi: 3,
    };
    assert_eq!(o.flip_ids_in(all), vec![60, 70], "rank crescente");
    let k = |bits| o.stack_key(&b, bits).expect("na ordem");
    assert!(k(1060) < k(11) && k(11) < k(1070));
    o.clear();
    assert!(o.flip_ranks.is_empty());
}

/// ⭐ Uma Macro sem palco a tocar não parte a ordem: o rank dela passa ao vizinho de baixo (ou de
/// cima, ou sprite sozinha); com palco, faixa própria no rank dela.
#[test]
fn a_macro_without_a_playing_stage_gives_its_rank_to_a_neighbour() {
    let ordem = || {
        let mut o = FrameOrder::default();
        o.record(0, None);
        o.record_palco(1, 50);
        o.record(2, None);
        o.record_palco(3, 51);
        o.record(4, Some(9));
        o
    };
    let mut o = ordem();
    o.adormece_palcos(|_| false);
    assert_eq!(o.complete(), Some(vec![S, S, S, S, V]));
    assert!(o.palco_ranks.is_empty());
    assert!(!needs_banding(&o.plan().0), "o quadro de sempre");
    let mut o = ordem();
    o.adormece_palcos(|m| m == 50);
    let (b, _) = o.plan();
    assert_eq!(
        b.iter().map(|b| (b.family, b.lo, b.hi)).collect::<Vec<_>>(),
        vec![(S, 0, 1), (K, 1, 2), (S, 2, 4), (V, 4, 5)]
    );
    assert_eq!(o.palco_ids_in(b[1]), vec![50]);
    let mut o = FrameOrder::default();
    o.record_palco(0, 50);
    o.record(1, Some(9));
    o.adormece_palcos(|_| false);
    assert_eq!(
        o.complete(),
        Some(vec![V, V]),
        "sem vizinho de baixo: o de cima"
    );
    let mut o = FrameOrder::default();
    o.record_palco(0, 50);
    o.adormece_palcos(|_| false);
    assert_eq!(o.complete(), Some(vec![S]));
    assert!(
        !needs_banding(&plan(&[S, V, M, K]).0),
        "palco por ultimo = o de sempre"
    );
    assert!(needs_banding(&plan(&[K, S]).0));
}

/// ⭐ GEF 19 — **o passe de baixo cola cada faixa pelo seu motor**: a faixa Motion abaixo dos sprites
/// vai à colagem do Motion, a Palco também, e as cenas contam-se por ordem. (Mutação: saltar a faixa
/// Motion ⇒ RED.)
#[test]
fn the_lower_pass_glues_a_motion_band_below_the_sprites() {
    let b = |family, lo| Band {
        family,
        lo,
        hi: lo + 1,
    };
    let bands = [b(M, 0), b(V, 1), b(K, 2), b(F, 3), b(P, 4), b(S, 5)];
    let colas: Vec<Cola> = passe_de_baixo(&bands).map(|(_, c)| c).collect();
    assert_eq!(
        colas,
        vec![
            Cola::Motion,
            Cola::Cena(0),
            Cola::Motion,
            Cola::Flip,
            Cola::Cena(1),
            Cola::Sprites
        ]
    );
    let faixas: Vec<Band> = passe_de_baixo(&bands).map(|(f, _)| f).collect();
    assert_eq!(faixas, bands, "cada faixa uma vez, na ordem de desenho");
}

/// ⭐ GEF 19 — **os palcos de fora**: com dois palcos a tocar, o que tem rank na ordem pousa na faixa
/// Palco dele, o outro no fim do mundo. (Mutação: `palco_fora` sempre `true` ⇒ RED.)
#[test]
fn a_stage_without_a_rank_is_the_only_one_outside() {
    let mut o = FrameOrder::default();
    o.record_palco(0, 41);
    assert!(!o.palco_fora(41), "o palco com rank cola na faixa dele");
    assert!(o.palco_fora(42), "o palco sem rank pousa no fim");
}
