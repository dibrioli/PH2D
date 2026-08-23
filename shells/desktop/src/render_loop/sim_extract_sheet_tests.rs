//! Gates da folha aberta — irmão de [`super`] pelo teto de LOC do shell.
//!
//! # ⛔ Um gate que existiu e SAIU, com o motivo
//!
//! Houve aqui um `painting_lays_the_cells_out_where_the_ghosts_put_them`, a afirmar que a folha
//! desdobrada (pintura) e os fantasmas (`Show sheet on canvas`) punham cada célula no mesmo sítio.
//! **Ele deixou de ser verdade de propósito**: as duas âncoras divergiram quando se mediu que o
//! `Sprite::frame` continua a andar durante a pintura — ancorar a folha desdobrada na célula viva
//! fá-la-ia deslizar debaixo do pincel. Hoje a pintura centra a folha no pivô e a
//! pré-visualização dispõe-na à volta da célula viva, e **a diferença é qual dos dois desenha essa
//! célula**. Ver o doc de [`super::unfolded_quad`].
//!
//! *Um gate apagado sem o motivo escrito lê-se como um gate inconveniente.*

use super::*;
use ph2d_render::Sprite;

/// Uma sprite de uma célula de `2×2` metros, numa grelha `hf × vf`, parada no frame `live`.
fn spr(hf: u32, vf: u32, live: u32) -> Sprite {
    let mut s = Sprite::atlas(0, [2.0, 2.0], [1.0; 4]);
    s.hframes = hf;
    s.vframes = vf;
    s.frame = live;
    s
}

fn base() -> RenderInstance {
    RenderInstance {
        world_pos: [0.0, 0.0],
        size: [2.0, 2.0],
        atlas_uv: [0.0, 0.0, 1.0, 1.0],
        tint: [1.0; 4],
        basis: [1.0, 0.0, 0.0, 1.0],
        premultiplied: 0.0,
        // ⚠️ **Um pivô AUTORADO, não zero**: o deslocamento SOMA-SE a ele, e um anchor de origem
        // deixaria verde uma implementação que o substituísse em vez de somar.
        anchor: [0.5, -0.25],
        per_corner_tint: [[1.0; 4]; 4],
        opacity: 0.8,
        flip_uv: 0,
        uv_xform: RenderInstance::IDENTITY_UV_XFORM,
        texture_id: 0,
        z_order: 7,
        sampling: RenderInstance::SAMPLING_DEFAULT,
        clip_group: RenderInstance::CLIP_GROUP_NONE,
        clip_meta: 0,
    }
}

const FULL: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

/// **Um sprite SEM grelha não é uma folha** — e a ausência é a resposta.
#[test]
fn a_sprite_without_a_grid_has_no_sheet_to_open() {
    assert_eq!(cell_count(&spr(1, 1, 0)), None);
    assert_eq!(
        cell_count(&spr(0, 0, 0)),
        None,
        "o piso de 1 vale aqui tambem"
    );
    assert_eq!(cell(&spr(1, 1, 0), FULL, 0), None);
    // E uma grelha de verdade conta.
    assert_eq!(cell_count(&spr(4, 2, 0)), Some(8));
}

/// **A CÉLULA VIVA não ganha fantasma** — o caminho normal já a desenha.
///
/// ⚠️ Desenhá-la duas vezes somaria alfa e dar-lhe-ia um realce que ninguém pediu — a mesma
/// armadilha que a precedência do overlay de âncoras pagou em 2026-08-23.
#[test]
fn the_live_cell_is_never_ghosted() {
    for live in 0..8u32 {
        let s = spr(4, 2, live);
        assert_eq!(cell(&s, FULL, live), None, "o frame {live} e' o vivo");
        // ⚠️ Controlo positivo: as OUTRAS sete existem — sem isto um `None` sempre passaria.
        let others = (0..8)
            .filter(|i| *i != live)
            .filter(|i| cell(&s, FULL, *i).is_some())
            .count();
        assert_eq!(others, 7, "as outras celulas tem de existir");
    }
    // Fora da grelha não existe.
    assert_eq!(cell(&spr(4, 2, 0), FULL, 8), None);
    // ⚠️ Um `frame` fora da grelha (a grelha encolheu debaixo dele) fixa na ULTIMA — e é essa que
    // fica sem fantasma, senão a folha abriria com duas células vivas.
    assert_eq!(cell(&spr(4, 2, 99), FULL, 7), None);
}

/// **A grelha abre-se no lugar certo: a coluna anda em `+X`, a linha anda em `−Y`.**
///
/// ⚠️ O `V` da textura cresce para BAIXO e o `Y` do mundo cresce para CIMA — é a inversão que
/// põe a segunda linha da folha **abaixo** da primeira, e não acima dela.
#[test]
fn the_grid_opens_right_and_down() {
    let s = spr(4, 2, 0); // celula viva = coluna 0, linha 0
    // Mesma linha, uma coluna à direita: +1 largura de célula.
    assert_eq!(cell(&s, FULL, 1).unwrap().1, [2.0, 0.0]);
    assert_eq!(cell(&s, FULL, 3).unwrap().1, [6.0, 0.0]);
    // Linha de baixo, mesma coluna: uma altura de célula para BAIXO.
    assert_eq!(cell(&s, FULL, 4).unwrap().1, [0.0, -2.0]);
    assert_eq!(cell(&s, FULL, 7).unwrap().1, [6.0, -2.0]);

    // E é RELATIVO à célula viva: com o frame no meio, há vizinhas dos dois lados.
    let s = spr(4, 2, 5); // coluna 1, linha 1
    assert_eq!(
        cell(&s, FULL, 4).unwrap().1,
        [-2.0, 0.0],
        "a de tras fica a' esquerda"
    );
    assert_eq!(
        cell(&s, FULL, 1).unwrap().1,
        [0.0, 2.0],
        "a linha de cima fica ACIMA"
    );
}

/// **A sub-UV de cada fantasma é a MESMA que o extract daria àquele frame.**
///
/// ⚠️ É a afirmação que impede a pré-visualização de ter uma segunda resposta a *«onde está a
/// célula N»*: as duas saem da mesma função, e este gate prende-o para o dia em que alguém
/// «otimizar» uma delas.
#[test]
fn a_ghosts_uv_is_the_one_the_extract_would_give_that_frame() {
    let s = spr(4, 2, 0);
    for i in 1..8u32 {
        let (uv, _) = cell(&s, FULL, i).unwrap();
        let want = super::super::sim_extract::sprite_sheet_subrect(FULL, 4, 2, i);
        assert_eq!(uv, want, "a celula {i} diverge do extract");
    }
    // E ela respeita um `base_uv` já estreitado por REGIÃO — a folha vive dentro da região.
    let region = [0.25, 0.0, 0.75, 0.5];
    let (uv, _) = cell(&s, region, 1).unwrap();
    assert_eq!(
        uv,
        super::super::sim_extract::sprite_sheet_subrect(region, 4, 2, 1)
    );
    assert!(
        uv[0] >= 0.25 && uv[2] <= 0.75,
        "a folha nao sai da regiao: {uv:?}"
    );
}

/// **O fantasma SOMA-SE ao pivô autorado e esmaece — e mais nada muda.**
#[test]
fn a_ghost_keeps_everything_but_its_cell_place_and_weight() {
    let b = base();
    let g = ghost(&b, [0.5, 0.0, 1.0, 1.0], [2.0, -2.0]);
    assert_eq!(g.atlas_uv, [0.5, 0.0, 1.0, 1.0]);
    assert_eq!(
        g.anchor,
        [0.5 + 2.0, -0.25 - 2.0],
        "SOMA ao pivo, nao substitui"
    );
    assert!(
        (g.opacity - 0.8 * GHOST_OPACITY).abs() < 1.0e-6,
        "o fantasma escala a opacidade da base, nao a substitui: {}",
        g.opacity
    );
    // O resto é a base, byte a byte — um fantasma não é um sprite diferente.
    assert_eq!(g.size, b.size);
    assert_eq!(g.basis, b.basis);
    assert_eq!(g.tint, b.tint);
    assert_eq!(g.texture_id, b.texture_id);
    assert_eq!(g.z_order, b.z_order);
    assert_eq!(g.sampling, b.sampling);
}

/// **O FLIP espelha a grelha, não cada célula no lugar dela.**
///
/// ⚠️ A lição que o 9-slice pagou: o bit de flip já inverte o conteúdo de cada quad no shader, e
/// o que falta é geométrico. Sem negar o deslocamento, uma folha espelhada abre-se com o desenho
/// virado e as células na ordem original — o espelho fica pela metade.
#[test]
fn a_flipped_sheet_mirrors_where_the_cells_go() {
    let mut b = base();
    b.flip_uv = RenderInstance::FLIP_X_BIT;
    let g = ghost(&b, FULL, [2.0, -2.0]);
    assert_eq!(g.anchor[0], base().anchor[0] - 2.0, "o X espelha");
    assert_eq!(g.anchor[1], base().anchor[1] - 2.0, "e o Y nao");

    let mut b = base();
    b.flip_uv = RenderInstance::FLIP_Y_BIT;
    let g = ghost(&b, FULL, [2.0, -2.0]);
    assert_eq!(g.anchor[0], base().anchor[0] + 2.0);
    assert_eq!(g.anchor[1], base().anchor[1] + 2.0, "o Y espelha");
}

/// **A DECISÃO de abrir a folha** — as três razões de não abrir, e a de abrir.
///
/// ⚠️ Este gate existe porque o laço que emite as células **não é alcançável de um teste** (o
/// `sim_extract::run` pede um `SpriteRenderer` vivo). Ele não prova que os fantasmas chegam ao
/// ecrã; prova que a **pergunta** está certa, e deixa no extract um resíduo curto o bastante para
/// se conferir a olho. *Dizer qual metade está gateada é a diferença entre uma cobertura e uma
/// alegação de cobertura.*
#[test]
fn the_sheet_opens_only_for_the_previewed_sprite_that_has_a_grid() {
    let e = ph2d_ecs::Entity::from_bits(0x1_0000_0007);
    let other = ph2d_ecs::Entity::from_bits(0x1_0000_0009);
    let sheet = spr(4, 2, 0);
    let plain = spr(1, 1, 0);

    assert_eq!(
        should_open(Some(e), e, false, &sheet),
        Some(8),
        "o caso de ABRIR"
    );
    assert_eq!(
        should_open(None, e, false, &sheet),
        None,
        "a caixa esta' desmarcada"
    );
    assert_eq!(
        should_open(Some(other), e, false, &sheet),
        None,
        "marcada sobre OUTRA entidade nao abre esta"
    );
    assert_eq!(
        should_open(Some(e), e, true, &sheet),
        None,
        "sob pre-visualizacao de ferramenta a textura nao e' uma folha"
    );
    assert_eq!(
        should_open(Some(e), e, false, &plain),
        None,
        "1x1 nao e' folha"
    );
}

/// **O QUAD DESDOBRADO cobre a folha inteira, e NÃO depende do frame vivo.**
///
/// ⚠️ A segunda metade é a que carrega o desenho, e ela custou uma versão: a primeira ancorava a
/// folha na célula viva, para a arte não saltar ao pegar no pincel. **O `Sprite::frame` continua a
/// andar enquanto se pinta**, então o desvio mudaria a cada quadro e a folha **deslizaria debaixo
/// do pincel**. Hoje ela fica centrada no pivô, e o mesmo sprite em frames diferentes dá o mesmo
/// quad.
#[test]
fn the_unfolded_quad_covers_the_sheet_and_ignores_the_live_frame() {
    assert_eq!(
        unfolded_quad(&spr(1, 1, 0)),
        None,
        "sem grelha nao ha' o que desdobrar"
    );
    let first = unfolded_quad(&spr(4, 2, 0)).unwrap();
    assert_eq!(first, [8.0, 4.0], "4x2 celulas de 2 m sao 8x4 m");
    for live in 1..8u32 {
        assert_eq!(
            unfolded_quad(&spr(4, 2, live)).unwrap(),
            first,
            "o frame {live} nao pode mover o quad -- a folha deslizaria sob o pincel"
        );
    }
    // E uma coluna também desdobra.
    assert_eq!(unfolded_quad(&spr(1, 4, 0)).unwrap(), [2.0, 8.0]);
}

/// **A PRÉ-VISUALIZAÇÃO ANIMADA mostra a célula VIVA, fora da folha, e ela ANDA.**
///
/// ⚠️ A razão de existir: com o quad desdobrado a mostrar tudo, o `Sprite::frame` deixa de ter
/// efeito visível. Se a sub-UV deste quad não seguisse o frame, ele seria mais um retrato parado —
/// e o artista pintaria oito desenhos sem nunca ver a animação que eles formam.
#[test]
fn the_animated_preview_follows_the_live_frame_and_sits_outside_the_sheet() {
    assert_eq!(
        anim_preview_quad(&spr(1, 1, 0), FULL),
        None,
        "sem grelha nao ha' preview"
    );

    let mut seen = std::collections::BTreeSet::new();
    for live in 0..8u32 {
        let s = spr(4, 2, live);
        let (uv, off) = anim_preview_quad(&s, FULL).unwrap();
        // A sub-UV é a do frame vivo — a MESMA que o extract daria.
        assert_eq!(
            uv,
            super::super::sim_extract::sprite_sheet_subrect(FULL, 4, 2, live),
            "o preview do frame {live} tem de mostrar o frame {live}"
        );
        seen.insert(uv.map(f32::to_bits));
        // Fica ACIMA, e fora da folha: meia folha (2 m) + meia célula + folga.
        assert_eq!(off[0], 0.0, "alinhado com o centro da folha");
        assert!(
            f64::from(off[1]) >= f64::from(s.vframes) * 0.5 * f64::from(s.size[1]),
            "tem de ficar FORA da folha, senao tapa o que se esta' a pintar (viu {})",
            off[1]
        );
    }
    assert_eq!(
        seen.len(),
        8,
        "os oito frames dao oito imagens diferentes -- ele ANDA"
    );
}

/// **A pergunta «uma ferramenta pré-visualiza esta entidade?» tem UMA resposta.**
///
/// ⚠️ Ela é lida pelo tique (uma folha em pintura toca), pelo extract (o quad desdobra-se) e pelo
/// overlay (as linhas seguem o quad desdobrado) — e chegou a estar escrita nos três. Uma
/// discordância dá **linhas sobre um quad que não desdobrou**, que é o defeito fotografado.
#[test]
fn one_function_answers_who_a_tool_is_previewing() {
    let a = ph2d_ecs::Entity::from_bits(0x1_0000_0001);
    let b = ph2d_ecs::Entity::from_bits(0x1_0000_0002);
    assert!(!is_tool_previewed(&[], a), "sem pre-visualizacao nenhuma");
    assert!(!is_tool_previewed(&[None, None], a), "com todas vazias");
    assert!(is_tool_previewed(&[None, Some(a.to_bits()), None], a));
    assert!(
        !is_tool_previewed(&[Some(b.to_bits())], a),
        "a pre-visualizacao do VIZINHO nao e' a desta"
    );
    // ⚠️ Várias ao mesmo tempo é o caso real (o sprite activo E o usado como Shape do pincel).
    assert!(is_tool_previewed(
        &[Some(b.to_bits()), Some(a.to_bits())],
        a
    ));
}
