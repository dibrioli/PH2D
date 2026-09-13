//! Os gates do retrato de um Prefab ([`super`]).
//!
//! ⚠️ **O oráculo é o PIXEL**, e não «a função devolveu `Some`»: um compositor que desenhasse uma
//! peça só, ou que espelhasse tudo, devolveria `Some` na mesma.

use super::compose;
use ph2d_asset_index::Thumb;
use ph2d_ecs::{ChildOf, Entity, SimWorld, Transform};

/// Pixels por metro — o valor não importa enquanto ninguém autorar `offset`, e o gate do pivô
/// escolhe-o de propósito.
const PPM: f32 = 100.0;
const RED: [u8; 4] = [255, 0, 0, 255];
const BLUE: [u8; 4] = [0, 0, 255, 255];

fn solid(c: [u8; 4]) -> Thumb {
    Thumb {
        rgba: std::sync::Arc::new(c.repeat(16 * 16)),
        w: 16,
        h: 16,
    }
}

/// Uma receita com peças em `(x, y)`, cada uma 1×1, e a cor que a miniatura dela devolve.
fn recipe(pieces: &[([f32; 2], [u8; 4])]) -> (SimWorld, Entity, Vec<Entity>, Vec<[u8; 4]>) {
    let mut sim = SimWorld::new();
    let root = sim
        .world_mut()
        .spawn((Transform::IDENTITY, ph2d_ecs::MasterRoot))
        .id();
    let mut ents = Vec::new();
    let mut colors = Vec::new();
    for (i, (at, c)) in pieces.iter().enumerate() {
        let e = sim
            .world_mut()
            .spawn((
                Transform::from_translation(ph2d_core::Vec2::new(at[0], at[1])),
                // ⛔⛔ **DE ÁTLAS, e SEM `SpritePixels`** — report do Enio de 2026-09-01 com
                // foto: *«objeto pai é o objeto vazio com 3 texturas como filhas»*, e o cartão
                // ficou cinzento. O `SpritePixels` é o carimbo da MINORIA; o caminho normal de
                // todo import e de todo canvas novo é o átlas. *Uma fixtura que usasse o carimbo
                // raro ficava verde sobre o defeito que o report mostra.*
                ph2d_render::Sprite::atlas(i as u32, [1.0, 1.0], [1.0; 4]),
                ChildOf(root),
            ))
            .id();
        ents.push(e);
        colors.push(*c);
    }
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    (sim, root, ents, colors)
}

/// A arte de uma peça, resolvida como o produto a resolve: **pela chave do átlas**, que é a forma
/// que o report expôs.
fn art(sim_pieces: Vec<Entity>, colors: Vec<[u8; 4]>) -> impl FnMut(Entity) -> Option<Thumb> {
    move |e| {
        let i = sim_pieces.iter().position(|p| *p == e)?;
        colors.get(i).copied().map(solid)
    }
}

fn pixel(t: &Thumb, x: u32, y: u32) -> [u8; 4] {
    let i = ((y * t.w + x) * 4) as usize;
    [t.rgba[i], t.rgba[i + 1], t.rgba[i + 2], t.rgba[i + 3]]
}

/// ⭐⭐⭐ **AS DUAS PEÇAS APARECEM** — é a razão de existir do retrato.
///
/// Até 2026-09-01 o cartão mostrava a peça MAIOR, e um prefab de duas peças aparecia como uma.
///
/// (Mutação: compor só a primeira peça ⇒ RED.)
#[test]
fn both_pieces_show_up_in_the_portrait() {
    let (sim, root, pieces, colors) = recipe(&[([-1.0, 0.0], RED), ([1.0, 0.0], BLUE)]);
    let t = compose(&sim, root, &pieces, PPM, art(pieces.clone(), colors)).expect("o retrato");
    let mid = t.h / 2;
    let left = pixel(&t, 1, mid);
    let right = pixel(&t, t.w - 2, mid);
    assert_eq!(
        left[0], 255,
        "a peca da esquerda nao foi desenhada: {left:?}"
    );
    assert_eq!(
        right[2], 255,
        "a peca da direita nao foi desenhada: {right:?}"
    );
}

/// ⛔⛔ **O RETRATO NÃO É ESPELHADO** — a peça da esquerda no mundo fica à esquerda no retrato, e a
/// de cima fica em cima.
///
/// ⚠️ **É o defeito que passa por bom até alguém comparar:** o Y do mundo cresce para cima e o de
/// uma imagem cresce para baixo, e sem a inversão o retrato sai virado sem nada o denunciar.
///
/// (Mutação: tirar o `h - y` do `blit` ⇒ RED na metade vertical.)
#[test]
fn the_portrait_is_not_mirrored_on_either_axis() {
    let (sim, root, pieces, colors) = recipe(&[([-1.0, 1.0], RED), ([1.0, -1.0], BLUE)]);
    let t = compose(&sim, root, &pieces, PPM, art(pieces.clone(), colors)).expect("o retrato");
    // A vermelha está em cima-à-esquerda no MUNDO ⇒ em cima-à-esquerda na IMAGEM (linha 0).
    let top_left = pixel(&t, 1, 1);
    let bottom_right = pixel(&t, t.w - 2, t.h - 2);
    assert_eq!(
        top_left[0], 255,
        "a peca de cima-a-esquerda nao esta' la': {top_left:?}"
    );
    assert_eq!(
        bottom_right[2], 255,
        "a peca de baixo-a-direita nao esta' la': {bottom_right:?}"
    );
}

/// ⚠️ **A escala é UNIFORME** — uma disposição larga não estica a peça quadrada.
///
/// Sem isto o objecto que o artista tenta reconhecer sai deformado, que é a única coisa que um
/// retrato tem de não fazer.
#[test]
fn a_wide_layout_does_not_stretch_the_pieces() {
    let (sim, root, pieces, colors) = recipe(&[([-4.0, 0.0], RED), ([4.0, 0.0], BLUE)]);
    let t = compose(&sim, root, &pieces, PPM, art(pieces.clone(), colors)).expect("o retrato");
    assert!(
        t.w > t.h,
        "uma disposicao larga tem de dar um retrato largo: {}x{}",
        t.w,
        t.h
    );
    // A caixa é 10×1 em mundo ⇒ o retrato tem de manter essa razão (a menos de arredondamento).
    let ratio = t.w as f32 / t.h as f32;
    assert!(
        (7.0..=13.0).contains(&ratio),
        "o aspecto do retrato ({ratio:.2}) nao segue o da disposicao (10:1)"
    );
}

/// ⛔ **Sem peça com pixels não há retrato** — e o cartão fica com a cor dominante.
///
/// ⚠️ Inventar um cinzento diria que o prefab tem retrato. *Ele não tem.*
#[test]
fn a_recipe_with_no_pixels_has_no_portrait() {
    let mut sim = SimWorld::new();
    let root = sim.world_mut().spawn((Transform::IDENTITY,)).id();
    let child = sim
        .world_mut()
        .spawn((Transform::IDENTITY, ChildOf(root)))
        .id();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    assert!(compose(&sim, root, &[child], PPM, |_| None).is_none());
}

/// ⛔⛔ **O retrato é DETERMINÍSTICO** — duas composições do mesmo mundo dão os mesmos bytes.
///
/// ⚠️ Sem a ordem total das peças elas trocavam de camada entre quadros ao sabor da ordem de
/// arquétipo, e **um cartão que pisca é pior que um cartão parcial**. ⚠️ E o memo do pintor
/// revalida por identidade de `Arc`: bytes iguais com `Arc` novo custam um reenvio ao atlas, mas
/// bytes DIFERENTES a cada quadro fariam o cartão tremer.
#[test]
fn the_portrait_is_deterministic() {
    let (sim, root, pieces, colors) = recipe(&[([-1.0, 0.0], RED), ([1.0, 0.0], BLUE)]);
    let a = compose(
        &sim,
        root,
        &pieces,
        PPM,
        art(pieces.clone(), colors.clone()),
    )
    .expect("a");
    let b = compose(&sim, root, &pieces, PPM, art(pieces.clone(), colors)).expect("b");
    assert_eq!(
        a.rgba, b.rgba,
        "duas composicoes iguais deram bytes diferentes"
    );
    assert_eq!((a.w, a.h), (b.w, b.h));
}

/// ⚠️ **Uma peça sem miniatura em cache é SALTADA, não é um retrato falhado** — o orçamento de
/// redução pode ter acabado no quadro, e o retrato parcial que sobra é melhor que nenhum.
#[test]
fn a_piece_without_art_is_skipped_and_the_rest_still_draws() {
    let (sim, root, pieces, _colors) = recipe(&[([-1.0, 0.0], RED), ([1.0, 0.0], BLUE)]);
    // Só a segunda tem arte.
    let only_second = pieces[1];
    let t = compose(&sim, root, &pieces, PPM, |e| {
        (e == only_second).then(|| solid(BLUE))
    })
    .expect("o retrato parcial");
    assert!(t.w > 0 && t.h > 0);
}

/// ⭐⭐⭐ **UMA PEÇA DE FOLHA MOSTRA A CÉLULA VIVA, e não a folha inteira** — report do Enio,
/// 2026-09-01 (2.ª foto): *«não ficou idêntico»*, com as peças a saírem estreitas.
///
/// ⚠️ **A causa era eu desenhar a imagem INTEIRA onde a tela desenha um PEDAÇO.** Uma sprite pode
/// mostrar uma célula de uma grelha (`SpriteGrid`) ou um sub-rectângulo (`SpriteRegion`), e o quad
/// não muda de tamanho por isso — o que muda é a janela. Espremer a folha toda no quad encolhe o
/// desenho na horizontal, que é exactamente o que a foto mostra.
///
/// ⚠️ **A fixtura CARREGA O FENÓMENO**: metade esquerda vermelha, metade direita azul, e a célula
/// viva é a **1** (a direita). Um retrato que ignore a grelha devolve as DUAS cores.
///
/// **Mutação que deve sangrar:** o `uv_window` devolver sempre o rectângulo unitário.
#[test]
fn a_sheet_piece_shows_the_live_cell_and_not_the_whole_sheet() {
    let mut sim = SimWorld::new();
    let root = sim
        .world_mut()
        .spawn((Transform::IDENTITY, ph2d_ecs::MasterRoot))
        .id();
    let piece = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
            // Duas colunas, e a VIVA é a segunda.
            ph2d_ecs::SpriteGrid {
                hframes: 2,
                vframes: 1,
                frame: 1,
            },
            ChildOf(root),
        ))
        .id();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());

    // Uma miniatura de duas metades: esquerda VERMELHA, direita AZUL.
    let mut rgba = Vec::new();
    for _ in 0..16 {
        for x in 0..16 {
            rgba.extend_from_slice(if x < 8 { &RED } else { &BLUE });
        }
    }
    let half = Thumb {
        rgba: std::sync::Arc::new(rgba),
        w: 16,
        h: 16,
    };
    let t = compose(&sim, root, &[piece], PPM, |e| {
        (e == piece).then(|| half.clone())
    })
    .expect("o retrato");
    // A célula viva é a direita ⇒ o retrato INTEIRO tem de ser azul.
    for x in [1, t.w / 2, t.w - 2] {
        let px = pixel(&t, x, t.h / 2);
        assert_eq!(
            px[2], 255,
            "a coluna {x} nao e' da celula viva — a folha inteira foi espremida no quad: {px:?}"
        );
        assert_eq!(
            px[0], 0,
            "sobrou vermelho da celula que NAO esta' viva: {px:?}"
        );
    }
}

/// ⭐⭐ **O PIVÔ desloca o quad** — uma sprite não-centrada não é desenhada sobre a translação.
///
/// ⚠️ A lei vive no [`ph2d_render::Sprite::resolve_anchor`], que é a porta que a tela usa. A 1.ª
/// versão do retrato usava a translação, e o objecto saía deslocado de meia peça.
///
/// **Mutação que deve sangrar:** ignorar o `resolve_anchor` e usar a translação.
#[test]
fn a_non_centered_piece_is_placed_where_the_screen_places_it() {
    let mut sim = SimWorld::new();
    let root = sim
        .world_mut()
        .spawn((Transform::IDENTITY, ph2d_ecs::MasterRoot))
        .id();
    // Uma peça CENTRADA na origem e outra NÃO centrada na origem: a segunda desenha-se meia peça
    // à direita e meia para baixo, então a caixa envolvente deixa de ser simétrica.
    let a = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
            ChildOf(root),
        ))
        .id();
    let mut off = ph2d_render::Sprite::atlas(1, [1.0, 1.0], [1.0; 4]);
    off.centered = false;
    let b = sim
        .world_mut()
        .spawn((Transform::IDENTITY, off, ChildOf(root)))
        .id();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());

    let t = compose(&sim, root, &[a, b], PPM, |e| {
        Some(solid(if e == a { RED } else { BLUE }))
    })
    .expect("o retrato");
    // ⚠️ **O oráculo é a peça de baixo ficar VISÍVEL**, e a 1.ª redacção deste gate não o era: ela
    // só perguntava se o canto direito era azul, e com o pivô ignorado as duas peças caem uma em
    // cima da outra — a azul é desenhada depois e tapa a vermelha, e a resposta continuava azul.
    // *A mutação sobreviveu, e foi ela que o disse.*
    //
    // Com o pivô: a centrada ocupa `[-0.5, +0.5]` e a não-centrada `[0, +1]` em x ⇒ há região onde
    // **só** a vermelha existe. Sem o pivô elas coincidem e o vermelho desaparece do retrato.
    let red_seen = (0..t.w).any(|x| {
        (0..t.h).any(|y| {
            let px = pixel(&t, x, y);
            px[0] > 200 && px[2] < 50
        })
    });
    assert!(
        red_seen,
        "a peca CENTRADA sumiu — as duas caíram no mesmo sitio, logo o pivô foi ignorado"
    );
}

/// ⭐⭐⭐ **A CENA DO REPORT, reproduzida com os construtores REAIS — e gravada como imagem.**
///
/// Report do Enio (2026-09-01, 3.ª foto): *«ainda não representa fielmente o objeto»*. Um objecto
/// vazio com três filhos de átlas: um quadrado preto de 128 px, um branco de 64 a sair pela borda
/// de baixo, um branco de 32 a sair pela de cima.
///
/// ⚠️ **Este gate existe para eu OLHAR**, não só para afirmar: ele grava o retrato em
/// `PH2D_PORTRAIT_DUMP` quando a variável estiver definida, ampliado 4× para se ler. As
/// afirmações abaixo são as que a foto refuta — e cada uma nasceu VERMELHA contra o código que a
/// foto mostrava.
#[test]
fn the_reported_scene_composes_faithfully() {
    const PPM: f32 = 100.0;
    let mut sim = SimWorld::new();
    let root = sim
        .world_mut()
        .spawn((Transform::IDENTITY, ph2d_ecs::MasterRoot))
        .id();
    // (chave de átlas, lado em px, centro em mundo)
    let specs: [(u32, u32, [f32; 2]); 3] = [
        (0, 128, [0.0, 0.0]),
        (1, 64, [0.0, -0.64 - 0.1]),
        (2, 32, [0.0, 0.64]),
    ];
    let mut pieces = Vec::new();
    for (key, side, at) in specs {
        let m = side as f32 / PPM;
        pieces.push(
            sim.world_mut()
                .spawn((
                    Transform::from_translation(ph2d_core::Vec2::new(at[0], at[1])),
                    ph2d_render::Sprite::atlas(key, [m, m], [1.0; 4]),
                    ChildOf(root),
                ))
                .id(),
        );
    }
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    // As miniaturas REAIS que a cache daria: o preto reduzido a 96, os brancos ao tamanho deles.
    let art_for = |side: u32, c: [u8; 4]| Thumb {
        rgba: std::sync::Arc::new(c.repeat((side * side) as usize)),
        w: side,
        h: side,
    };
    let black = art_for(96, [0, 0, 0, 255]);
    let w64 = art_for(64, [255, 255, 255, 255]);
    let w32 = art_for(32, [255, 255, 255, 255]);
    let (p0, p1, p2) = (pieces[0], pieces[1], pieces[2]);
    let t = compose(&sim, root, &pieces, PPM, |e| {
        Some(if e == p0 {
            black.clone()
        } else if e == p1 {
            w64.clone()
        } else if e == p2 {
            w32.clone()
        } else {
            unreachable!()
        })
    })
    .expect("o retrato");

    if let Ok(dir) = std::env::var("PH2D_PORTRAIT_DUMP") {
        let k = 4u32;
        let mut big = vec![0u8; (t.w * k * t.h * k * 4) as usize];
        for y in 0..t.h * k {
            for x in 0..t.w * k {
                let src = (((y / k) * t.w + (x / k)) * 4) as usize;
                let dst = ((y * t.w * k + x) * 4) as usize;
                big[dst..dst + 4].copy_from_slice(&t.rgba[src..src + 4]);
            }
        }
        let path = std::path::Path::new(&dir).join("portrait_repro.png");
        image::save_buffer(&path, &big, t.w * k, t.h * k, image::ColorType::Rgba8)
            .expect("gravar o retrato");
        eprintln!("[retrato] {}x{} gravado em {}", t.w, t.h, path.display());
    }

    // A caixa: 1,28 de largura; altura = do fundo do branco de baixo (−0,74−0,32) ao topo do de
    // cima (0,64+0,16) ⇒ 1,86 ⇒ o retrato é mais ALTO que largo.
    assert!(
        t.h > t.w,
        "a caixa envolvente e' mais alta que larga: {}x{}",
        t.w,
        t.h
    );
    // ⛔ **A mancha vertical**: um pixel do preto, fora das colunas dos brancos, tem de ser PRETO —
    // e um pixel ACIMA do branco de cima (mas dentro da caixa) tem de ser TRANSPARENTE.
    let s = t.h as f32 / 1.86;
    let col_left = 2; // coluna de borda: so' o preto vive aqui
    let y_mid = t.h / 2;
    let px = pixel(&t, col_left, y_mid);
    assert_eq!(px, [0, 0, 0, 255], "a borda do preto nao e' preta: {px:?}");
    // O branco de cima ocupa x ∈ [−0,16, 0,16] ⇒ colunas centrais; y de 0,48 a 0,80. Um pixel na
    // coluna central a MEIA ALTURA do preto (y=0) tem de ser preto, nao branco.
    let col_mid = t.w / 2;
    let y_zero = ((0.80 - 0.0) * s) as u32; // y=0 em mundo, contado do topo da caixa
    let px = pixel(&t, col_mid, y_zero);
    assert_eq!(
        px,
        [0, 0, 0, 255],
        "o branco de cima MANCHOU a coluna central ate' ao meio do preto: {px:?}"
    );
    // E o branco de cima ocupa metade fora do preto: um pixel em y=0,72 na coluna central e' BRANCO
    // e um pixel na mesma altura mas na coluna da borda e' TRANSPARENTE (fora de toda peça).
    let y_top_white = ((0.80 - 0.72) * s) as u32;
    assert_eq!(
        pixel(&t, col_mid, y_top_white)[0],
        255,
        "o branco de cima nao esta' la'"
    );
    assert_eq!(
        pixel(&t, col_left, y_top_white)[3],
        0,
        "ha' tinta fora de todas as pecas (acima do preto, na borda)"
    );
}
