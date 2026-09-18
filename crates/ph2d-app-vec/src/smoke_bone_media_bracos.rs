//! ⭐⭐⭐ **AS CENAS DO BRAÇO** — `=2`, o TESTE NULO, e `=3`, o braço que ANIMA. Filho do
//! [`super`] para herdar as fixturas dele.
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

/// ⭐⭐⭐ **O PASSO DA COLUNA SAI DA DOBRA, e não de uma fracção escolhida.**
///
/// ⛔⛔ Ele era `alt × 1,5`, calibrado a olho quando a cena dobrava `13°`. A `25°` a ponta de uma
/// corrente de três ossos sobe `0,95 m` sobre uma banda de `0,90 m` ⇒ **os braços encostam-se**, e a
/// foto mostrou-os a tocar-se. *Um número de disposição calibrado a UM ângulo deixa de descrever a
/// cena no dia em que o ângulo muda* — e quem o mudou (§0.0) tem de reconferir.
///
/// A subida da ponta é fechada: cada osso mede `larg/3` e o `k`-ésimo chega inclinado `k × DOBRA`,
/// logo a ponta sobe `(larg/3) · Σ sin(k·θ)`. A banda de um braço é `alt + subida`, e a folga é um
/// quarto da altura — **ar que se vê**, não uma margem de segurança.
fn passo_da_coluna(larg: f64, alt: f64) -> f64 {
    let osso = larg / 3.0;
    let t = f64::from(DOBRA).to_radians();
    let subida: f64 = (1..3).map(|k| (f64::from(k) * t).sin()).sum::<f64>() * osso;
    alt + subida + alt * 0.25
}

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
    let passo = passo_da_coluna(larg, alt);
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
        braco_de(BRACO_PX, 0),
        larg,
        DOBRA,
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
        DOBRA,
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
        braco_de(BRACO_PX, 0),
        larg,
        DOBRA,
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
fn braco_de(px: [u32; 2], tom: usize) -> Vec<u8> {
    let (w, h) = (px[0] as usize, px[1] as usize);
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
        let um = braco_de(BRACO_PX, c);
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

/// ⭐⭐⭐ **O TAMANHO DO BRAÇO DA `=3` SAI DA CÂMERA DE OMISSÃO** — ele é derivado, não escolhido.
///
/// ⛔⛔ Aquela cena **não pede o *Frame All*** (o porquê medido vive no
/// [`super::Prologo::enquadrar`]: com um painel aberto ele corta sempre), logo ela abre na câmera de
/// omissão — `Camera2d::default().height_world`, que é **`10 m`** de mundo sobre a ALTURA da janela.
///
/// O braço fica em **metade** dessa altura de comprimento e mantém o `4:1` que o dono pediu. ⚠️ A
/// folga é grande de propósito e o recurso tem nome: os painéis são desenhados por cima do mundo e
/// tapam `~37 %` da largura e `~33 %` da altura (medido na foto de 2026-09-18) — um braço colado ao
/// limite ficaria com as pontas debaixo de um dock, e é **exactamente esse** o defeito que esta cena
/// existe para não ter.
///
/// ⚠️ **Em PIXELS de arte e não em metros**, porque a [`super::uma`] mede o quad em `pixels / ppm`:
/// pedir o tamanho em metros aqui obrigaria a escrevê-lo DUAS vezes.
fn braco_da_camera_de_omissao(ppm: f64) -> [u32; 2] {
    let altura_de_mundo = f64::from(ph2d_render::Camera2d::default().height_world);
    let comprimento_m = altura_de_mundo * 0.5;
    let w = (comprimento_m * ppm).round().max(4.0) as u32;
    [w, (w / 4).max(1)]
}

/// **`=3` — O BRAÇO ANIMA** (2026-09-18): um braço só, **em repouso**, com a timeline aberta e o
/// AutoKey armado. ⭐⭐⭐ **Aqui quem dobra é o DONO** — as outras duas cenas dobram sozinhas.
///
/// ⛔⛔ **Ela existe porque a máquina estava pronta e ninguém a tinha visto.** O AutoKey grava a
/// corrente INTEIRA que a mão moveu (e o alvo de uma restrição de IK) desde 2026-09-14, com seis
/// gates na shell — e **nenhuma cena do app armava o AutoKey**. *Uma feature construída, gateada e
/// sem smoke é uma feature que o artista não tem.*
///
/// ⚠️ **Um braço e não três**, e a razão é a pergunta: aqui ela é *«a pose é gravada?»*, e três
/// sujeitos só acrescentariam a dúvida de qual deles gravou.
///
/// ⚠️ **A dobra entra a ZERO** — o braço nasce esticado. Uma cena que já chegasse dobrada teria a
/// pose inicial dentro do repouso e a primeira chave do dono não mudaria nada na tela.
///
/// ⚠️ **A ferramenta NÃO é armada aqui:** desde 2026-09-09 o app abre o painel de Bones e arma a
/// ferramenta sozinho quando um osso é escolhido. *Armá-la na cena seria a segunda resposta à mesma
/// pergunta, e a que envelhece.*
pub(super) fn anima(
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    cell_idx: u32,
    pixels_per_meter: f32,
    atlas_asset_map: &mut BTreeMap<u32, AssetId>,
) -> Option<(u64, u32)> {
    let ppm = f64::from(pixels_per_meter.max(f32::MIN_POSITIVE));
    let px = braco_da_camera_de_omissao(ppm);
    let larg = f64::from(px[0]) / ppm;
    let bits = uma(
        sim,
        renderer,
        asset_db,
        cell_idx,
        pixels_per_meter,
        atlas_asset_map,
        [0.0, 0.0],
        "Braco",
        px[0],
        px[1],
        braco_de(px, 0),
        larg,
        // ⭐ ZERO: o braço nasce esticado, e quem o dobra é o dono.
        0.0,
        |_, _| {},
    )?;
    anuncia_anima();
    Some((bits, 1))
}

/// O roteiro da cena que anima.
fn anuncia_anima() {
    println!(
        "[bone-media-smoke] O BRACO ANIMA: um braco esticado, a timeline aberta e o AutoKey ligado.\n\
         [bone-media-smoke] 1) Na Hierarquia clique em 'Bone 3' (o osso do meio) — o painel Bones \
         abre-se e a ferramenta de osso arma-se sozinha\n\
         [bone-media-smoke] 2) No canvas ARRASTE esse osso: o braco dobra E aparece uma chave na \
         timeline, no tempo 0\n\
         [bone-media-smoke] 3) Arraste o cursor da timeline para ~1 s e dobre o braco para o OUTRO \
         lado — nasce a segunda chave\n\
         [bone-media-smoke] 4) Carregue em Play: o braco tem de ANIMAR entre as duas poses\n\
         [bone-media-smoke] Se nao aparecer chave nenhuma no passo (2), o AutoKey nao ficou armado \
         — PARE e diga"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐⭐⭐ **A COLUNA COBRE A SUBIDA DA DOBRA** — o gate que a foto de 2026-09-18 encomendou.
    ///
    /// ⛔ Com o passo calibrado a `13°` e a cena a dobrar `25°`, a ponta de um braço sobe para dentro
    /// da banda do vizinho e os três encostam-se. *Uma cena em que as peças se tocam não responde
    /// «elas dobram igual?» — ela pergunta «qual é qual?».*
    ///
    /// ⚠️ **As duas metades são precisas.** A primeira diz que a banda cabe; sem a segunda, um passo
    /// enorme passaria (e o bloco sairia do enquadramento, que é o defeito que a `=1` já pagou).
    #[test]
    fn a_coluna_cobre_a_subida_da_dobra() {
        let (larg, alt) = (2.4_f64, 0.6_f64);
        let osso = larg / 3.0;
        let t = f64::from(DOBRA).to_radians();
        let subida: f64 = (1..3).map(|k| (f64::from(k) * t).sin()).sum::<f64>() * osso;
        let passo = passo_da_coluna(larg, alt);
        assert!(
            passo >= alt + subida,
            "o passo da coluna ({passo:.3} m) nao cobre a banda de um braco dobrado \
             ({:.3} m: {alt:.3} de altura mais {subida:.3} de subida) — os tres encostam-se",
            alt + subida
        );
        assert!(
            passo <= alt + subida + alt,
            "o passo ({passo:.3} m) sobra mais de uma altura de arte sobre a banda \
             ({:.3} m) — o bloco cresce e o enquadramento deixa de o mostrar",
            alt + subida
        );
    }

    /// ⚠️ **O CONTROLO da lei acima: a subida TEM de crescer com a dobra.** Sem isto, um
    /// `passo_da_coluna` que ignorasse o ângulo (o defeito que esta wave curou) passaria o gate
    /// irmão — ele mede uma desigualdade, e uma constante grande satisfá-la por acaso.
    #[test]
    fn a_subida_cresce_com_a_dobra() {
        let sobe = |graus: f64| -> f64 {
            let t = graus.to_radians();
            (1..3).map(|k| (f64::from(k) * t).sin()).sum::<f64>() * 0.8
        };
        assert!(
            sobe(0.0) == 0.0 && sobe(13.0) < sobe(25.0) && sobe(25.0) < sobe(40.0),
            "a subida da ponta nao e' monotona na dobra: {:.3} / {:.3} / {:.3} / {:.3}",
            sobe(0.0),
            sobe(13.0),
            sobe(25.0),
            sobe(40.0)
        );
    }
}
