//! ⭐⭐⭐ **AS TRÊS MÍDIAS PRESAS AO MESMO GESTO** — `PH2D_VEC_BONE_MEDIA_SMOKE=1` (F11, ordem do
//! dono de 2026-09-17: *«vamos lá: imagens em 9 fatias e folhas de quadros»*).
//!
//! Três imagens lado a lado, cada uma presa à corrente dela e dobrada pelo MESMO ângulo:
//!
//! 1. **simples** — o CONTROLO, que já dobrava antes desta wave;
//! 2. **folha de quadros** (`4×1`, a mostrar o quadro `1`);
//! 3. **9-slice** esticado ao triplo da largura.
//!
//! # ⛔ Porque as três, e não só as duas que a wave curou
//!
//! ⚠️ **O que a `2` e a `3` faziam antes era INVISÍVEL sem o controlo ao lado.** A folha dobrava —
//! **errado**, e sem aviso nenhum (a silhueta era a dos quatro quadros espremida no sítio de um), e
//! o 9-slice não dobrava de todo. *Sem a imagem simples ao lado, «dobrou» e «dobrou certo» leem-se
//! igual.*
//!
//! # ⚠️ A arte de cada uma é escolhida para tornar o defeito LEGÍVEL
//!
//! - A folha tem **a MORDIDA num lado diferente por quadro**, e a união das quatro é o disco
//!   inteiro. Com quatro formas iguais a silhueta errada seria indistinguível da certa — *uma cena
//!   cuja arte não distingue os dois estados não prova nenhum*.
//! - O 9-slice é uma **moldura oca**: o miolo é transparente, logo vê-se se o pedaço do meio foi
//!   desenhado onde não devia.

use ph2d_asset::{AssetDb, AssetId};
use ph2d_core::Vec2;
use ph2d_ecs::{Entity, SimWorld};
use ph2d_render::SpriteRenderer;
use std::collections::BTreeMap;

/// Lado de uma célula da folha e da moldura, em pixels.
/// ⚠️ **A resolução da arte é INDEPENDENTE do tamanho da cena, e a foto impôs as duas.** A `64 px`
/// a silhueta saía com dentes de escada visíveis (a malha é traçada da tinta); a `128` ela fecha.
/// O tamanho no mundo é o [`LADO_M`], escrito à parte — *ligar os dois faria melhorar a arte
/// empurrar a cena para fora do enquadramento.*
const LADO_PX: u32 = 128;

/// O lado de uma imagem no MUNDO, em metros — ver [`LADO_PX`].
const LADO_M: f32 = 0.64;

/// Quantos quadros a folha tem.
const QUADROS: u32 = 4;

/// O quadro que a sprite mostra — ⚠️ **não é o `0` de propósito**: com o primeiro, uma leitura que
/// ignorasse a grelha acertaria por acaso na coluna de origem.
const QUADRO_VIVO: u32 = 1;

/// ⭐⭐⭐ **A dobra desta cena, em graus por junta — MAIS SUAVE que a da cena do Painter (`25°`).**
///
/// ⛔⛔ **A FOTO impôs o número.** A `25°` por junta a ponta da corrente roda `50°` e a malha
/// **DOBRA SOBRE SI MESMA**: os triângulos invertidos somem do desenho e a arte lê-se RASGADA em
/// duas. ⚠️ *Topologicamente ela não pode rasgar* — a tinta é uma imagem contínua de um conjunto
/// conexo —, e é exactamente por isso que o rasgo na foto é o sinal da dobra, não da lei desta wave.
/// Aqui a pergunta é **QUE arte é desenhada**, e uma cena que se lê rasgada não a responde.
const DOBRA: f32 = 13.0;

/// A borda do 9-slice, em pixels da fonte.
const BORDA_PX: f32 = 20.0;

/// Quantas vezes o 9-slice é esticado na largura — ⚠️ **é aqui que ele faz alguma coisa**: no
/// tamanho intrínseco ele é a identidade geométrica e a cena não mostraria nada.
///
/// ⚠️ **`2` e não `3`, e a FOTO é que o disse:** a `3` a fileira mede `12,2 m` e o enquadramento
/// automático mostra `~8` — a imagem da esquerda ficava **fora do ecrã**. *Uma cena que sai do
/// enquadramento ensina o contrário do que diz.*
const ESTICA: f32 = 2.0;

/// ⭐⭐⭐ **DOIS NÍVEIS, e o `=2` é uma pergunta DIFERENTE** (ordem do dono, 2026-09-18).
///
/// - **`=1` — as três FORMAS:** *que arte é desenhada?* Artes diferentes de propósito, para se ver
///   que a folha mostra UM quadro e que a moldura mantém os cantos.
/// - **`=2` — os três BRAÇOS:** *elas dobram IGUAL?* Mesmo tamanho, mesma largura, mesma arte —
///   um **TESTE NULO**: as três TÊM de sair idênticas, e qualquer diferença é o defeito.
pub const NIVEIS: u32 = 2;

/// O nível pedido, coagido a `1..=NIVEIS`.
///
/// ⚠️ Um valor ilegível (ou a env vazia, que é como um `env VAR=` a arma) cai em `1`: *o caminho de
/// omissão é a cena que o dono já aprovou, nunca uma que ele não pediu.*
#[must_use]
pub fn nivel() -> u32 {
    nivel_de(std::env::var("PH2D_VEC_BONE_MEDIA_SMOKE").ok().as_deref())
}

/// A LEI do [`nivel`], sem a env — ela não se escreve num gate (`set_var` é `unsafe` na edição 2024
/// e corre numa árvore com threads).
#[must_use]
pub(crate) fn nivel_de(v: Option<&str>) -> u32 {
    v.and_then(|v| v.trim().parse::<u32>().ok())
        .unwrap_or(1)
        .clamp(1, NIVEIS)
}

/// Este smoke está armado?
#[must_use]
pub fn armed() -> bool {
    std::env::var_os("PH2D_VEC_BONE_MEDIA_SMOKE").is_some()
}

/// Tinta LISTRADA — o controlo. As listras transversais tornam a dobra legível (uma barra chapada
/// dobrada lê-se quase igual à mesma barra rodada).
fn listrada(w: u32, h: u32) -> Vec<u8> {
    let (wu, hu) = (w as usize, h as usize);
    let mut rgba = vec![0u8; wu * hu * 4];
    for y in 0..hu {
        for x in 0..wu {
            let i = (y * wu + x) * 4;
            let escura = (x / 12) % 2 == 0;
            rgba[i] = if escura { 40 } else { 235 };
            rgba[i + 1] = if escura { 90 } else { 235 };
            rgba[i + 2] = if escura { 180 } else { 235 };
            rgba[i + 3] = 255;
        }
    }
    rgba
}

/// A FOLHA: `QUADROS` células de `LADO_PX`, cada uma um DISCO com a mordida num lado diferente.
///
/// ⭐⭐⭐ **A união das quatro é o DISCO INTEIRO, e essa é a razão da forma.** A malha do bind é
/// traçada sobre a UNIÃO dos quadros (é o que impede um quadro de ser recortado ao dar play), então
/// a união é geometria de verdade — e quatro formas SOBREPOSTAS de famílias diferentes (disco,
/// triângulo, cruz, barra) davam uma união com **gargalos finos**, onde o traçador deixa fendas.
///
/// ⛔⛔ **A foto apanhou-o e eu quase o li como defeito do produto:** a arte desenhava-se RASGADA ao
/// meio, e ao afinar a resolução o rasgo ficou **mais** visível — *é assim que se distingue um
/// artefacto do traçador de um defeito da lei: afinar a malha piora um e cura o outro.*
///
/// ⚠️ Cada quadro tira um QUADRANTE diferente, logo nenhum ponto é tirado em mais de um ⇒ a união
/// é exactamente o disco.
fn folha() -> Vec<u8> {
    let lado = LADO_PX as usize;
    let w = lado * QUADROS as usize;
    let mut rgba = vec![0u8; w * lado * 4];
    for c in 0..QUADROS as usize {
        let (r, g, b) = match c {
            0 => (230u8, 80, 80),
            1 => (80, 200, 120),
            2 => (90, 140, 240),
            _ => (240, 200, 70),
        };
        for y in 0..lado {
            for x in 0..lado {
                let (dx, dy) = (x as f32 / lado as f32 - 0.5, y as f32 / lado as f32 - 0.5);
                if dx.hypot(dy) >= 0.45 {
                    continue;
                }
                // O quadrante que ESTE quadro tira — `atan2` em `0..4`, a começar à direita.
                let q = ((dy.atan2(dx) / std::f32::consts::FRAC_PI_2).rem_euclid(4.0)) as usize;
                if q == c {
                    continue;
                }
                let i = ((y * w) + c * lado + x) * 4;
                rgba[i] = r;
                rgba[i + 1] = g;
                rgba[i + 2] = b;
                rgba[i + 3] = 255;
            }
        }
    }
    rgba
}

/// A MOLDURA do 9-slice: um anel opaco com o miolo TRANSPARENTE, e os cantos marcados.
fn moldura() -> Vec<u8> {
    let lado = LADO_PX as usize;
    let b = BORDA_PX as usize;
    let mut rgba = vec![0u8; lado * lado * 4];
    for y in 0..lado {
        for x in 0..lado {
            let no_anel = x < b || y < b || x >= lado - b || y >= lado - b;
            if !no_anel {
                continue;
            }
            // ⭐ O canto é de outra cor: é ele que tem de ficar do MESMO tamanho quando a moldura
            // estica, e sem contraste ninguém vê se ele esticou.
            let canto = (x < b || x >= lado - b) && (y < b || y >= lado - b);
            let i = (y * lado + x) * 4;
            rgba[i] = if canto { 250 } else { 120 };
            rgba[i + 1] = if canto { 170 } else { 130 };
            rgba[i + 2] = if canto { 60 } else { 200 };
            rgba[i + 3] = 255;
        }
    }
    rgba
}

/// Monta a cena do nível pedido. Devolve `(bits da 1.ª imagem, quantas células do atlas foram
/// gastas)`.
pub fn build(
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    cell_idx: u32,
    pixels_per_meter: f32,
    atlas_asset_map: &mut BTreeMap<u32, AssetId>,
) -> Option<(u64, u32)> {
    if nivel() == 2 {
        return bracos(
            sim,
            renderer,
            asset_db,
            cell_idx,
            pixels_per_meter,
            atlas_asset_map,
        );
    }
    formas(
        sim,
        renderer,
        asset_db,
        cell_idx,
        pixels_per_meter,
        atlas_asset_map,
    )
}

/// **`=1` — AS TRÊS FORMAS**: artes diferentes, para se ver QUE arte cada uma desenha.
fn formas(
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    cell_idx: u32,
    pixels_per_meter: f32,
    atlas_asset_map: &mut BTreeMap<u32, AssetId>,
) -> Option<(u64, u32)> {
    let m = f64::from(LADO_M);
    // ⭐⭐⭐ **DUAS FILEIRAS, e a disposição é IMPOSTA pela lei do enquadramento** — não é gosto.
    //
    // ⛔⛔ **Uma fileira de três NUNCA cabe, e a aritmética di-lo:** o `ViewFocusKind::All` ajusta a
    // altura a `max(span_y, span_x / aspecto_da_JANELA) × 1,1`, e os painéis tapam ~37 % da largura
    // ⇒ a largura VISÍVEL é `altura × aspecto_do_CANVAS`. Com o `span_x` a mandar, isso dá
    // `span_x × 0,81 < span_x` **para qualquer tamanho** — encolher a cena não ajuda, e três fotos
    // provaram-no (a da esquerda ficava sempre cortada pela borda do painel).
    //
    // ⇒ a cena só cabe se o `span_y` mandar, isto é, se ela for quase QUADRADA: duas em cima, a
    // moldura (que é `ESTICA ×` mais larga) sozinha em baixo.
    let vao = m * 0.4;
    let linha_y = (m + vao) / 2.0;
    let xs = [-(m + vao) / 2.0, (m + vao) / 2.0, 0.0];
    let ys = [linha_y, linha_y, -linha_y];

    let mut primeiro = None;
    let mut gastas = 0_u32;

    // (1) O CONTROLO.
    if let Some(bits) = uma(
        sim,
        renderer,
        asset_db,
        cell_idx,
        pixels_per_meter,
        atlas_asset_map,
        [xs[0], ys[0]],
        "Simples",
        LADO_PX,
        LADO_PX,
        listrada(LADO_PX, LADO_PX),
        f64::from(LADO_M),
        |sim, e| tamanho(sim, e, [LADO_M, LADO_M]),
    ) {
        primeiro.get_or_insert(bits);
        gastas += 1;
    }

    // (2) A FOLHA de quadros.
    if let Some(bits) = uma(
        sim,
        renderer,
        asset_db,
        cell_idx + 1,
        pixels_per_meter,
        atlas_asset_map,
        [xs[1], ys[1]],
        "Folha 4 quadros",
        LADO_PX * QUADROS,
        LADO_PX,
        folha(),
        f64::from(LADO_M),
        |sim, e| {
            sim.world_mut().entity_mut(e).insert(ph2d_ecs::SpriteGrid {
                hframes: QUADROS,
                vframes: 1,
                frame: QUADRO_VIVO,
            });
            // ⚠️ **O quad é o de UMA célula** — a `spawn_rgba` mede-o pela imagem inteira, que numa
            // folha é a grelha toda.
            tamanho(sim, e, [LADO_M, LADO_M]);
        },
    ) {
        primeiro.get_or_insert(bits);
        gastas += 1;
    }

    // (3) O 9-SLICE esticado.
    if let Some(bits) = uma(
        sim,
        renderer,
        asset_db,
        cell_idx + 2,
        pixels_per_meter,
        atlas_asset_map,
        [xs[2], ys[2]],
        "Moldura 9-slice",
        LADO_PX,
        LADO_PX,
        moldura(),
        f64::from(LADO_M) * f64::from(ESTICA),
        |sim, e| {
            sim.world_mut().entity_mut(e).insert(ph2d_ecs::SliceNine {
                draw_mode: ph2d_ecs::SliceDrawMode::Sliced,
                borders: [BORDA_PX; 4],
                ..ph2d_ecs::SliceNine::INERT
            });
            tamanho(sim, e, [LADO_M * ESTICA, LADO_M]);
        },
    ) {
        primeiro.get_or_insert(bits);
        gastas += 1;
    }

    anuncia(gastas);
    Some((primeiro?, gastas))
}

/// Escreve o tamanho do quad desta sprite, em metros.
///
/// ⚠️ **As TRÊS passam por aqui, e não só as duas que precisam:** a `spawn_rgba` deriva o tamanho
/// dos PIXELS, logo melhorar a resolução da arte mudaria a cena de sítio. *Separar as duas grandezas
/// é o que deixa a arte ficar mais fina sem a cena sair do ecrã.*
fn tamanho(sim: &mut SimWorld, e: Entity, size: [f32; 2]) {
    if let Some(mut s) = sim.world_mut().get_mut::<ph2d_render::Sprite>(e) {
        s.size = size;
    }
}

/// Uma imagem: sobe a tinta, monta a corrente DELA, prende e dobra.
///
/// ⚠️ **A ordem é lei:** prender define o repouso, então a dobra vem depois — dobrada antes, esta
/// pose SERIA o repouso e a imagem sairia recta.
#[expect(
    clippy::too_many_arguments,
    reason = "e' o construtor de uma cena: cada argumento e' uma porta do app, e agrupa-los numa               struct so' esconderia isso"
)]
fn uma(
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    cell_idx: u32,
    pixels_per_meter: f32,
    atlas_asset_map: &mut BTreeMap<u32, AssetId>,
    centro: [f64; 2],
    nome: &str,
    w_px: u32,
    h_px: u32,
    pixels: Vec<u8>,
    largura_do_osso_m: f64,
    prepara: impl FnOnce(&mut SimWorld, Entity),
) -> Option<u64> {
    let (_, bits) = ph2d_image_import::spawn_rgba(
        sim,
        renderer,
        asset_db,
        cell_idx,
        w_px,
        h_px,
        pixels,
        Vec2::new(centro[0] as f32, centro[1] as f32),
        pixels_per_meter,
        atlas_asset_map,
        nome,
    )
    .map_err(|e| eprintln!("[bone-media-smoke] '{nome}' nao subiu: {e}"))
    .ok()?;
    let e = Entity::try_from_bits(bits)?;
    // ⭐⭐⭐ **CONFIGURAR ANTES DE PRENDER, e a ordem é LEI.** O bind lê a grelha, a região e o
    // TAMANHO do quad para traçar a malha e pôr as articulações em pixels da imagem. ⛔⛔ A 1.ª
    // redacção desta cena escrevia o tamanho **depois** de prender: a malha era traçada contra um
    // quad `4 ×` mais largo, as articulações caíam no sítio errado, e o disco saía **RASGADO ao
    // meio** — *e eu quase o li como defeito da wave, porque a arte e a lei estavam as duas certas.*
    prepara(sim, e);
    let ossos = crate::smoke_bone_paint::corrente_em(sim, largura_do_osso_m, centro)?;
    let arte = atlas_asset_map
        .get(&cell_idx)
        .and_then(|id| asset_db.get(id));
    let preso = arte
        .as_ref()
        .and_then(|a| a.image_rgba8())
        .is_some_and(|(w, h, cow)| {
            ph2d_skeleton_live::skin_live::bind_image(
                sim,
                e,
                &cow,
                [w, h],
                pixels_per_meter,
                ph2d_poly2d::GridOptions::default(),
                ossos.first().copied(),
            )
        });
    if !preso {
        eprintln!("[bone-media-smoke] '{nome}' NAO prendeu ao esqueleto -- PARE");
        return Some(bits);
    }
    crate::smoke_bone_paint::dobra(sim, &ossos, DOBRA);
    Some(bits)
}

/// O roteiro.
fn anuncia(gastas: u32) {
    if gastas < 3 {
        eprintln!("[bone-media-smoke] PARE: montei so' {gastas} de 3 imagens");
    }
    println!(
        "[bone-media-smoke] tres imagens presas a ossos e dobradas {}° por junta:\n\
         [bone-media-smoke] 1) 'Simples' (em cima a' esquerda) e' o CONTROLO — ela ja' dobrava antes\n\
         [bone-media-smoke] 2) 'Folha 4 quadros' (em cima a' direita) mostra o quadro {QUADRO_VIVO} de {QUADROS} \
         — um DISCO verde com UMA mordida (em baixo a' esquerda). Se ele aparecer ESTICADO ao \
         comprido, ou com mais de uma mordida, a malha voltou a ser a da folha inteira\n\
         [bone-media-smoke] 3) 'Moldura 9-slice' (em baixo) esta' esticada {ESTICA}x: os quatro \
         CANTOS (laranja) tem de ficar do mesmo tamanho e as bordas (azul) esticam — e a moldura \
         inteira dobra. Se ela estiver DIREITA, o 9-slice voltou a nao deformar",
        DOBRA
    );
}

#[path = "smoke_bone_media_bracos.rs"]
mod bracos_cena;
use bracos_cena::bracos;

#[cfg(test)]
mod tests {
    /// ⭐⭐⭐ **CONFIGURAR VEM ANTES DE PRENDER** — a ordem que quatro fotos pagaram.
    ///
    /// ⛔⛔ O bind lê a grelha, a região e o TAMANHO do quad para traçar a malha e pôr as
    /// articulações em pixels da imagem. Escrever o tamanho **depois** traça a malha contra um quad
    /// `4 ×` mais largo: a arte sai **RASGADA ao meio**, e como a lei e a arte estão as duas certas,
    /// lê-se como defeito do produto.
    ///
    /// ⚠️ **Por posição no texto e não por comportamento**, e a razão é declarada: o corpo desta cena
    /// precisa de um `SpriteRenderer` e de um atlas, que só existem com uma janela. ⭐ E ela deixa de
    /// **compilar** se o ficheiro mudar de sítio, que é o que a torna honesta.
    #[test]
    fn a_cena_configura_antes_de_prender() {
        const FONTE: &str = include_str!("smoke_bone_media.rs");
        let prepara = FONTE
            .find("\n    prepara(sim, e);")
            .expect("a cena deixou de ter o passo que configura a sprite antes de prender");
        let bind = FONTE
            .find("skin_live::bind_image(")
            .expect("a cena deixou de prender");
        assert!(
            prepara < bind,
            "o `prepara` passou para DEPOIS do `bind_image` — a malha volta a ser tracada contra o \
             quad errado, e a arte sai rasgada ao meio"
        );
    }

    /// ⭐⭐ **A UNIÃO DOS QUATRO QUADROS É O DISCO INTEIRO** — a propriedade que faz a arte desta cena
    /// ser traçável sem fendas.
    ///
    /// ⛔ Quatro formas de famílias diferentes sobrepostas davam uma união com **gargalos finos**,
    /// onde o traçador deixa fendas — e afinar a resolução PIORAVA o sintoma. *É assim que se separa
    /// um artefacto do traçador de um defeito da lei.*
    #[test]
    fn a_uniao_dos_quadros_e_o_disco_inteiro() {
        let arte = super::folha();
        let lado = super::LADO_PX as usize;
        let w = lado * super::QUADROS as usize;
        let mut faltam = 0;
        for y in 0..lado {
            for x in 0..lado {
                let (dx, dy) = (x as f32 / lado as f32 - 0.5, y as f32 / lado as f32 - 0.5);
                if dx.hypot(dy) >= 0.42 {
                    continue;
                }
                let coberto = (0..super::QUADROS as usize)
                    .any(|c| arte[((y * w) + c * lado + x) * 4 + 3] > 0);
                if !coberto {
                    faltam += 1;
                }
            }
        }
        assert_eq!(
            faltam, 0,
            "{faltam} pixels do disco nao estao em quadro nenhum — a uniao deixa de ser o disco e o \
             tracador ganha gargalos"
        );
    }
}
