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

/// **A dobra desta cena, em graus por junta.**
///
/// ⛔⛔⛔ **ESTA NOTA JÁ ESTEVE ERRADA, e a redacção anterior fica registada porque o erro é do tipo
/// mais caro que existe: um número de PRODUTO baixado para esconder um defeito cuja causa era
/// outra.** Ela dizia — a `13`, em 2026-09-17 — que *«a `25°` a malha DOBRA SOBRE SI MESMA: os
/// triângulos invertidos somem do desenho e a arte lê-se RASGADA em duas»*.
///
/// **MEDIDO em 2026-09-18** ([`ph2d_skeleton_live`], sonda `sonda_da_dobra`, sobre a arte do braço
/// desta cena e pela porta do produto): a dobra **não vira um único triângulo** até `75°` por junta
/// (`150°` na ponta); a primeira inversão aparece a `90°`, com `19` de `3 593`, e ali a corrente
/// está dobrada por completo sobre si.
///
/// | graus/junta | `0` | `13` | `25` | `50` | `75` | `90` |
/// |---|---:|---:|---:|---:|---:|---:|
/// | triângulos do avesso | `0` | `0` | `0` | `0` | `0` | **`19`** |
/// | pior factor de área | `1,44` | `1,22` | `0,99` | `0,49` | `0,03` | `−0,18` |
///
/// ⭐⭐⭐ **O rasgo da foto era REAL e tinha OUTRA causa — duas, na verdade, e as duas curadas na
/// mesma jornada:** a arte da folha tinha **gargalos finos** na união dos quadros (ver [`folha`]) e
/// a cena **configurava a sprite DEPOIS de prender** (ver [`uma`]), o que traçava a malha contra um
/// quad `4 ×` mais largo. *Baixar o ângulo fez o sintoma encolher, e por isso pareceu uma cura.*
///
/// ⚠️ **A lição é a do `CLAUDE.md` §0.0 duas vezes:** um limite legítimo diz **de que recurso é** —
/// este não dizia de nenhum —, e *quem cura o defeito que tornava um número inalcançável tem de
/// reconferir a nota*. O número volta a `25`, que é onde a dobra se LÊ: no 9-slice a `13°` a moldura
/// quase não arqueia.
const DOBRA: f32 = 25.0;

/// A borda do 9-slice, em pixels da fonte.
const BORDA_PX: f32 = 20.0;

/// Quantas vezes o 9-slice é esticado na largura — ⚠️ **é aqui que ele faz alguma coisa**: no
/// tamanho intrínseco ele é a identidade geométrica e a cena não mostraria nada.
///
/// ⚠️ **`2` e não `3`, e a FOTO é que o disse:** a `3` a fileira mede `12,2 m` e o enquadramento
/// automático mostra `~8` — a imagem da esquerda ficava **fora do ecrã**. *Uma cena que sai do
/// enquadramento ensina o contrário do que diz.*
const ESTICA: f32 = 2.0;

/// ⭐⭐⭐ **TRÊS NÍVEIS, e cada um faz uma pergunta DIFERENTE.**
///
/// - **`=1` — as três FORMAS:** *que arte é desenhada?* Artes diferentes de propósito, para se ver
///   que a folha mostra UM quadro e que a moldura mantém os cantos.
/// - **`=2` — os três BRAÇOS:** *elas dobram IGUAL?* Mesmo tamanho, mesma largura, mesma arte —
///   um **TESTE NULO**: as três TÊM de sair idênticas, e qualquer diferença é o defeito.
/// - **`=3` — o braço ANIMA:** *a pose que a mão faz é GRAVADA?* Um braço só, em repouso, com a
///   timeline aberta e o AutoKey armado — quem dobra aqui é o **dono**.
///
/// ⛔⛔ **O `=3` existe porque a máquina dele estava pronta e NINGUÉM a tinha visto.** O AutoKey
/// grava a corrente inteira (e o alvo de uma restrição de IK) desde 2026-09-14, com seis gates na
/// shell — e **nenhuma cena do app armava o AutoKey**, logo o dono nunca lhe chegou. *Uma feature
/// construída, gateada e sem smoke é uma feature que o artista não tem* (`CLAUDE.md` §0.8: o smoke
/// é onde ele as APRENDE).
pub const NIVEIS: u32 = 4;

/// **O que a SHELL tem de armar para a cena `n`** — o prólogo, que não é da crate.
///
/// ⚠️ **Ele é uma LEI PURA aqui e um efeito lá, de propósito.** Abrir um painel, ligar um
/// interruptor da timeline e parar o relógio são três coisas da `App`, e a cena não a tem; mas a
/// DECISÃO — *quais destas três, para que nível* — é da cena, e escrita aqui ela é gateável sem
/// janela nenhuma. O molde é o da física: *o que sai são os CORPOS; o que decide a ordem do quadro
/// fica na shell.*
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Prologo {
    /// A timeline tem de estar **visível**: o `autokey_pass` exige `panel_open` — *gravar é um modo
    /// de autoria, e não faz sentido com a interface dele escondida.*
    pub timeline_aberta: bool,
    /// O interruptor **AutoKey** da timeline.
    pub auto_key: bool,
    /// ⚠️ **O relógio NASCE A ANDAR** (`Playhead::new` põe `playing: true`) — medido, não suposto.
    /// Uma cena de autoria que não o pare grava a pose num instante que já passou.
    pub relogio_parado: bool,
    /// ⛔⛔⛔ **Pedir o *Frame All* — e a cena que ANIMA não pede, por MEDIÇÃO.**
    ///
    /// O `ViewFocusKind::All` ajusta ao rectângulo da **JANELA** e os painéis são desenhados POR
    /// CIMA dela; ele enche `110 %` da janela com o conteúdo, logo **o que os painéis tapam fica
    /// fora**. Com as colunas laterais (`~37 %` da largura) isso já morde, e com a timeline aberta
    /// (`~33 %` da altura) **nenhum tamanho de cena sobrevive**: o ajuste é derivado do próprio
    /// conteúdo, então encolher a cena encolhe o enquadramento junto.
    ///
    /// ⭐ *Medido na foto de 2026-09-18:* a `=3` com o `All` mostrava `±80 px` de mundo sobre um
    /// braço que mede `±120` — cortado nas duas pontas. ⇒ ela fica na **câmera de omissão**
    /// (`Camera2d::default().height_world = 10 m`), onde a área livre é folgada, e dimensiona-se
    /// para lá dentro.
    ///
    /// ⚠️ **Isto NÃO é a cura do defeito** — ele é do verbo *Frame All* e vale para toda a casa
    /// (abrir a timeline e carregar em *Frame All* corta a cena de qualquer módulo). A cura pede o
    /// rectângulo LIVRE em vez do da janela, muda o enquadramento de todas as cenas do app, e por
    /// isso é decisão do dono e não de uma linha a meio de uma wave.
    pub enquadrar: bool,
}

/// A lei do [`Prologo`], por nível.
///
/// ⚠️ **As cenas `=1` e `=2` não armam NADA**, e isso é metade do valor deste gate: um prólogo que
/// arma sempre poria a timeline por cima de duas cenas que o dono já aprovou sem ela.
#[must_use]
pub fn prologo_do_nivel(n: u32) -> Prologo {
    let anima = n == 3;
    Prologo {
        timeline_aberta: anima,
        auto_key: anima,
        relogio_parado: anima,
        // ⛔⛔ **O `Frame All` NÃO é o default, é uma escolha por cena — e a `=4` saiu dela por
        // MEDIÇÃO.** Ele enquadra o **quad** de cada sprite, e uma pele dobrada sai muito para fora
        // do quad: na foto de 2026-09-18 a terceira peça do personagem ficava cortada pela borda.
        // *Duas cenas desta família já não o pedem, por duas razões diferentes* — a `=3` porque a
        // timeline aberta corta sempre, a `=4` porque a deformação não cabe no que ele mede.
        enquadrar: n <= 2,
    }
}

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
    match nivel() {
        2 => {
            return bracos(
                sim,
                renderer,
                asset_db,
                cell_idx,
                pixels_per_meter,
                atlas_asset_map,
            );
        }
        3 => {
            return anima(
                sim,
                renderer,
                asset_db,
                cell_idx,
                pixels_per_meter,
                atlas_asset_map,
            );
        }
        4 => {
            return personagem(
                sim,
                renderer,
                asset_db,
                cell_idx,
                pixels_per_meter,
                atlas_asset_map,
            );
        }
        _ => {}
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
        None,
        DOBRA,
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
        None,
        DOBRA,
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
        None,
        DOBRA,
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
    // ⭐⭐⭐ **O RIG PARTILHADO.** `None` = esta imagem monta a corrente DELA (as cenas `=1`..`=3`);
    // `Some` = ela prende-se a uma que já existe, que é o que faz um PERSONAGEM — várias peças, um
    // esqueleto só. ⚠️ Com `Some`, quem dobra é o CHAMADOR: dobrar aqui somaria o ângulo uma vez
    // por peça, e três peças dariam o triplo da dobra.
    rig: Option<&[Entity]>,
    // ⭐ **A dobra é do CHAMADOR e não uma leitura global da `DOBRA`.** A cena `=3` precisa dela a
    // ZERO — ali quem dobra é o DONO, com a mão —, e ler a const aqui dentro faria *«a cena que não
    // dobra»* ser inexprimível sem um segundo caminho.
    graus: f32,
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
    let proprios;
    let ossos: &[Entity] = match rig {
        Some(r) => r,
        None => {
            proprios = crate::smoke_bone_paint::corrente_em(sim, largura_do_osso_m, centro)?;
            &proprios
        }
    };
    let arte = atlas_asset_map
        .get(&cell_idx)
        .and_then(|id| asset_db.get(id));
    let preso = arte
        .as_ref()
        .and_then(|a| a.image_rgba8())
        .is_some_and(|(w, h, cow)| {
            ph2d_skeleton_live::skin_image_bind::bind_image(
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
    crate::smoke_bone_paint::dobra(sim, ossos, graus);
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

#[path = "smoke_bone_media_arte.rs"]
mod arte;
use arte::{folha, listrada, moldura};

#[path = "smoke_bone_media_bracos.rs"]
mod bracos_cena;
use bracos_cena::{anima, bracos, personagem};

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
            .find("skin_image_bind::bind_image(")
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

    /// ⭐⭐⭐ **O PRÓLOGO ARMA SÓ A CENA QUE ANIMA** — a lei que a shell obedece.
    ///
    /// ⛔ **As metades NEGATIVAS são metade do gate:** um prólogo que armasse sempre poria a
    /// timeline por cima das duas cenas que o dono já aprovou sem ela, e a `=2` (o teste nulo)
    /// perderia metade do ecrã para uma tira que não responde à pergunta dela.
    #[test]
    fn o_prologo_arma_so_a_cena_que_anima() {
        for n in [1_u32, 2] {
            let p = super::prologo_do_nivel(n);
            assert_eq!(
                p,
                super::Prologo {
                    timeline_aberta: false,
                    auto_key: false,
                    relogio_parado: false,
                    // ⭐ As duas cenas que o dono já aprovou CONTINUAM a enquadrar-se sozinhas —
                    // elas não abrem painel nenhum, e ali o `Frame All` faz o que promete.
                    enquadrar: true,
                },
                "a cena =\u{7b}n\u{7d} passou a armar um prologo que ela nao pediu: {p:?}"
            );
        }
        // ⚠️ E a do PERSONAGEM também não se enquadra, por OUTRA razão — a deformação sai do quad
        // que o `Frame All` mede. *Duas ausências com mecanismos diferentes, e só uma delas anima.*
        let q = super::prologo_do_nivel(4);
        assert!(
            !q.enquadrar && !q.timeline_aberta && !q.auto_key && !q.relogio_parado,
            "a cena do personagem so' pode dispensar o Frame All — todo o resto do prologo e' da \
             cena que anima, e arma-lo aqui abre um painel que ninguem pediu: {q:?}"
        );
        let p = super::prologo_do_nivel(3);
        assert!(
            !p.enquadrar,
            "a cena que anima NAO pode pedir o Frame All: com a timeline aberta ele corta sempre \
             (o mecanismo medido vive no doc do campo)"
        );
        assert!(
            p.timeline_aberta && p.auto_key && p.relogio_parado,
            "a cena que anima precisa das TRES (a timeline aberta, o AutoKey e o relogio parado) — \
             sem qualquer uma delas o arrasto do dono nao grava chave nenhuma: {p:?}"
        );
    }

    /// ⭐⭐⭐ **E A SHELL ARMA-O MESMO** — sem esta metade, a lei de cima afirma sobre código que
    /// ninguém corre.
    ///
    /// ⚠️ **Por texto e não por comportamento, com a razão declarada:** a fase vive no `render_frame`
    /// e pede uma janela. ⭐ E o `include_str!` deixa de **compilar** se o ficheiro mudar de sítio —
    /// é isso que separa esta agulha de um `grep` que passa a medir zero em silêncio.
    #[test]
    fn a_shell_arma_o_prologo_desta_cena() {
        const FASE: &str =
            include_str!("../../../shells/desktop/src/render_loop/fase_atlas_scene_smokes_late.rs");
        for (agulha, porque) in [
            (
                "smoke_bone_media::prologo_do_nivel(",
                "a shell deixou de perguntar a' cena o que armar — a decisao voltou a viver nela",
            ),
            (
                "TimelineIntent::SetAutoKey(true)",
                "o AutoKey deixou de ser armado: o arrasto do dono nao grava chave nenhuma",
            ),
            (
                "kind: ph2d_editor_core::ViewFocusKind::All",
                "o enquadramento saiu do prologo — ou ele voltou a ser pedido incondicionalmente, e a cena que abre a timeline fica cortada",
            ),
            (
                "playhead.pause()",
                "o relogio deixou de ser parado, e ele NASCE a andar: a pose e' gravada num instante que ja' passou",
            ),
        ] {
            assert!(
                FASE.contains(agulha),
                "{porque} (agulha ausente: {agulha:?})"
            );
        }
    }

    /// ⚠️⚠️ **O nível tem DUAS leis e elas não são a mesma** — e a 1.ª redacção deste gate
    /// confundiu-as, acusando o produto de um defeito que era uma suposição minha:
    ///
    /// - **ilegível ou ausente ⇒ `1`**, o caminho de OMISSÃO (a cena que o dono já aprovou);
    /// - **legível e fora de faixa ⇒ COAGIDO à faixa** (`clamp`), que é o que o doc do
    ///   [`super::nivel_de`] diz por escrito: um `=9` preserva *«ele pediu uma alta»* e mostra o
    ///   topo, em vez de o mandar, calado, para o princípio.
    ///
    /// ⭐ A última asserção é a que impede a [`super::NIVEIS`] de mentir: o topo declarado tem de
    /// ser ALCANÇÁVEL, senão acrescentar uma cena e esquecer a constante deixa-a inatingível.
    #[test]
    fn o_nivel_e_coagido_a_faixa_das_cenas() {
        for (v, esperado, porque) in [
            (Some("3"), 3, "o topo de hoje tem de passar"),
            (Some("2"), 2, "um nivel do meio"),
            (
                Some("9"),
                super::NIVEIS,
                "legivel e alto demais: COAGIDO ao topo, nao mandado para o principio",
            ),
            (Some("0"), 1, "legivel e baixo demais: coagido ao piso"),
            (Some(""), 1, "a env vazia e' como um `env VAR=` a arma"),
            (Some("dois"), 1, "um valor ilegivel cai na cena de omissao"),
            (None, 1, "sem env"),
        ] {
            assert_eq!(super::nivel_de(v), esperado, "{porque} (pedido: {v:?})");
        }
        assert_eq!(
            super::nivel_de(Some(&super::NIVEIS.to_string())),
            super::NIVEIS,
            "o topo declarado tem de ser alcancavel — senao a `NIVEIS` mente sobre quantas cenas ha'"
        );
    }
}
