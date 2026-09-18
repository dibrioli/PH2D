//! ⭐⭐⭐ **`=2` — OS TRÊS BRAÇOS: o TESTE NULO**, filho do [`super`] para herdar as fixturas dele.
//!
//! Ordem do dono (2026-09-18): *«crie os 3 tipos do mesmo tamanho e mesma largura para eu ver como
//! se dobram, se se dobram igual uma a outra»*.
//!
//! ⚠️ **A pergunta é OUTRA e por isso o ficheiro é outro.** O pai monta a cena `=1`, que pergunta
//! *que arte cada mídia desenha* — artes diferentes de propósito. Aqui a arte, o tamanho, o
//! esqueleto e a dobra são os MESMOS, e o certo é **não se ver diferença nenhuma**.
//!
//! ⚠️ Ele é filho e não irmão porque o motor de que precisa — `uma`, `tamanho`, `eixos_em`,
//! `corrente_em`, `dobra` — é do pai, e **duplicá-lo poria as duas cenas a correr leis diferentes.**

use super::*;

/// A arte de um braço, em pixels — comprida e estreita (`4:1`), como o dono pediu.
///
/// ⚠️⚠️ **Estes números são o MUNDO também, e é isso que faz a cena ser um teste nulo.** A
/// `spawn_rgba` mede o quad em `pixels / ppm`, e o 9-slice só é a **identidade geométrica** quando o
/// alvo é esse tamanho intrínseco — ali os cantos não esticam, as bordas não esticam, e os nove
/// pedaços recompõem a imagem original. ⇒ *qualquer diferença entre as três É o defeito.*
const BRACO_PX: [u32; 2] = [240, 60];

/// As bordas do 9-slice do braço, `[esquerda, cima, direita, baixo]` em pixels da fonte.
///
/// ⚠️ **Assimétricas de propósito:** num braço de `60 px` de altura, `24` em cima e em baixo
/// deixariam a fileira do meio com `12` — e uma fileira fina é onde um erro de mapa se esconde.
const BRACO_BORDAS: [f32; 4] = [24.0, 12.0, 24.0, 12.0];

/// **`=2` — OS TRÊS BRAÇOS, empilhados** (ordem do dono, 2026-09-18: *«crie os 3 tipos do mesmo
/// tamanho e mesma largura para eu ver como se dobram, se se dobram igual uma a outra»*).
///
/// ⭐⭐⭐ **É um TESTE NULO**: mesmo tamanho, mesma largura, **a mesma arte**, o mesmo esqueleto e a
/// mesma dobra — as três TÊM de sair idênticas. *Uma cena em que o certo é «não se vê diferença» é a
/// régua mais dura que existe, porque não há nada para interpretar.*
///
/// ⚠️ **A coluna é imposta pela mesma lei de enquadramento da cena `=1`** (ver o doc do [`formas`]):
/// três braços de `4:1` numa FILEIRA mediriam `12:1` e nunca caberiam. Empilhados, o bloco mede
/// `~1:1`.
pub(super) fn bracos(
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    cell_idx: u32,
    pixels_per_meter: f32,
    atlas_asset_map: &mut BTreeMap<u32, AssetId>,
) -> Option<(u64, u32)> {
    let ppm = f64::from(pixels_per_meter.max(f32::MIN_POSITIVE));
    let (larg, alt) = (f64::from(BRACO_PX[0]) / ppm, f64::from(BRACO_PX[1]) / ppm);
    let vao = alt * 0.5;
    let passo = alt + vao;
    let ys = [passo, 0.0, -passo];
    let celula = [
        BRACO_PX[0] as f32 / pixels_per_meter.max(f32::MIN_POSITIVE),
        BRACO_PX[1] as f32 / pixels_per_meter.max(f32::MIN_POSITIVE),
    ];

    let mut primeiro = None;
    let mut gastas = 0_u32;

    // (1) O CONTROLO — a arte, tal e qual.
    if let Some(bits) = uma(
        sim,
        renderer,
        asset_db,
        cell_idx,
        pixels_per_meter,
        atlas_asset_map,
        [0.0, ys[0]],
        "Braco simples",
        BRACO_PX[0],
        BRACO_PX[1],
        braco(0),
        larg,
        |_, _| {},
    ) {
        primeiro.get_or_insert(bits);
        gastas += 1;
    }

    // (2) A FOLHA — as quatro células são o MESMO braço com listras de outra cor.
    //
    // ⚠️ **A silhueta das quatro é igual de propósito:** a malha do bind é traçada sobre a UNIÃO
    // delas, e com silhuetas diferentes a malha desta seria diferente da do controlo — *a cena
    // deixaria de comparar a DEFORMAÇÃO e passaria a comparar duas malhas*.
    if let Some(bits) = uma(
        sim,
        renderer,
        asset_db,
        cell_idx + 1,
        pixels_per_meter,
        atlas_asset_map,
        [0.0, ys[1]],
        "Braco folha 4 quadros",
        BRACO_PX[0] * QUADROS,
        BRACO_PX[1],
        folha_de_bracos(),
        larg,
        move |sim, e| {
            sim.world_mut().entity_mut(e).insert(ph2d_ecs::SpriteGrid {
                hframes: QUADROS,
                vframes: 1,
                frame: QUADRO_VIVO,
            });
            tamanho(sim, e, celula);
        },
    ) {
        primeiro.get_or_insert(bits);
        gastas += 1;
    }

    // (3) O 9-SLICE — a MESMA arte, no tamanho INTRÍNSECO.
    //
    // ⭐⭐⭐ **Nada de esticão aqui, e é essa a força da cena:** no tamanho intrínseco o 9-slice é a
    // identidade geométrica, então os nove pedaços recompõem exactamente a imagem do controlo. ⛔ Um
    // esticão faria a comparação da dobra ficar refém da comparação da textura.
    //
    // ⚠️ **O tamanho NÃO é escrito:** a `spawn_rgba` já o mede em `pixels / ppm`, que é exactamente
    // o intrínseco — escrevê-lo à mão seria uma segunda resposta à mesma pergunta.
    if let Some(bits) = uma(
        sim,
        renderer,
        asset_db,
        cell_idx + 2,
        pixels_per_meter,
        atlas_asset_map,
        [0.0, ys[2]],
        "Braco 9-slice",
        BRACO_PX[0],
        BRACO_PX[1],
        braco(0),
        larg,
        |sim, e| {
            sim.world_mut().entity_mut(e).insert(ph2d_ecs::SliceNine {
                draw_mode: ph2d_ecs::SliceDrawMode::Sliced,
                borders: BRACO_BORDAS,
                ..ph2d_ecs::SliceNine::INERT
            });
        },
    ) {
        primeiro.get_or_insert(bits);
        gastas += 1;
    }

    anuncia_bracos(gastas, larg, alt);
    Some((primeiro?, gastas))
}

/// A arte de um braço: um rectângulo OPACO com listras transversais, no matiz `tom`.
///
/// ⚠️ **Opaco e rectangular de propósito:** a malha é traçada da tinta, então uma silhueta
/// rectangular dá a MESMA malha às três — *num teste nulo, a malha é uma variável que tem de ser
/// controlada, não medida.*
///
/// ⚠️ **As listras são TRANSVERSAIS** (colunas): dobrar o cotovelo abre-as em leque, que é o que o
/// olho vê. Uma barra chapada dobrada lê-se quase igual à mesma barra rodada.
fn braco(tom: usize) -> Vec<u8> {
    let (w, h) = (BRACO_PX[0] as usize, BRACO_PX[1] as usize);
    let mut rgba = vec![0u8; w * h * 4];
    // ⭐ O número de listras é FIXO e não derivado da largura: assim a mesma arte em qualquer
    // resolução abre o mesmo leque, e a cena não muda de leitura ao afinar a arte.
    let listras = 12usize;
    let claro = [
        (235u8, 235, 235),
        (200, 240, 210),
        (215, 225, 250),
        (245, 235, 205),
    ][tom % 4];
    let escuro = [
        (40u8, 90, 180),
        (40, 150, 90),
        (70, 80, 200),
        (200, 150, 40),
    ][tom % 4];
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) * 4;
            let c = if (x * listras / w).is_multiple_of(2) {
                escuro
            } else {
                claro
            };
            rgba[i] = c.0;
            rgba[i + 1] = c.1;
            rgba[i + 2] = c.2;
            rgba[i + 3] = 255;
        }
    }
    rgba
}

/// A folha de `QUADROS` braços: a mesma silhueta, um matiz por quadro.
fn folha_de_bracos() -> Vec<u8> {
    let (w, h) = (BRACO_PX[0] as usize, BRACO_PX[1] as usize);
    let total = w * QUADROS as usize;
    let mut rgba = vec![0u8; total * h * 4];
    for c in 0..QUADROS as usize {
        let um = braco(c);
        for y in 0..h {
            let dst = (y * total + c * w) * 4;
            let src = y * w * 4;
            rgba[dst..dst + w * 4].copy_from_slice(&um[src..src + w * 4]);
        }
    }
    rgba
}

/// O roteiro da cena dos braços.
fn anuncia_bracos(gastas: u32, larg: f64, alt: f64) {
    if gastas < 3 {
        eprintln!("[bone-media-smoke] PARE: montei so' {gastas} de 3 bracos");
    }
    println!(
        "[bone-media-smoke] TESTE NULO: tres bracos de {larg:.2} x {alt:.2} m, a MESMA arte, o \
         mesmo esqueleto e a mesma dobra ({DOBRA}° por junta):\n\
         [bone-media-smoke] 1) 'Braco simples' (em cima) — o controlo\n\
         [bone-media-smoke] 2) 'Braco folha 4 quadros' (meio) — o quadro {QUADRO_VIVO} de \
         {QUADROS}, que e' o MESMO braco noutro matiz (verde)\n\
         [bone-media-smoke] 3) 'Braco 9-slice' (em baixo) — a mesma arte, no tamanho INTRINSECO: \
         ali o 9-slice e' a identidade, e os nove pedacos recompoem a imagem do controlo\n\
         [bone-media-smoke] ⇒ AS TRES TEM DE DOBRAR IGUAL. Se uma delas abrir o leque de listras de \
         outra maneira, ou tiver a silhueta noutro sitio, e' ELA que esta' errada.\n\
         [bone-media-smoke] (com PH2D_BONE_LOG=1 a de baixo imprime NOVE linhas de 'pele:', uma por \
         pedaco — e' isso que prova que ela esta' mesmo a ser fatiada)"
    );
}
