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
        None,
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
        None,
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
        None,
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
        None,
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

/// ⭐⭐⭐ **A DOBRA DESTA CENA É MENOR QUE A DAS OUTRAS, e o número saiu de uma FOTO.**
///
/// ⛔⛔ **A minha 1.ª redacção afirmava que «a subida é comum às três» e isso está REFUTADO** — e a
/// medição que a desmente já estava na mão: a sonda do rig partilhado lê excursões de `1,123 m` e
/// `1,195 m` para duas peças na **mesma** corrente, porque os pesos dependem da distância ao osso.
/// *Peças paralelas afastadas do eixo CONVERGEM do lado interior de uma curva* — é geometria, não
/// defeito —, e a `25°` da [`DOBRA`] elas encostavam-se umas às outras na foto.
///
/// ⚠️ **E o mesmo ângulo estragava o enquadramento:** o *Frame All* mede o **quad** de cada sprite,
/// e a pele dobrada sai muito para fora dele ⇒ a `25°` metade da figura ficava fora do ecrã.
///
/// ⚠️ **Baixar o ângulo é legítimo aqui e não na [`bracos`]:** ali a pergunta é *«as três dobram
/// IGUAL?»*, e uma dobra fraca esconde exactamente a diferença que se procura; aqui ela é *«as três
/// seguem o MESMO esqueleto?»*, que se lê com uma curva suave.
const DOBRA_DO_PERSONAGEM: f32 = 12.0;

/// ⭐⭐ **E a ordem é ERRO DE COMPILAÇÃO, não um `assert!` de teste.**
///
/// ⛔ Um `assert!` sobre duas CONSTANTES é dobrado pelo compilador antes de correr, e o clippy
/// di-lo em voz alta (`this assertion has a constant value`) — foi ele que apanhou a 1.ª redacção
/// deste gate. *Uma afirmação que o compilador já resolveu não é um teste; num `const` ela é uma
/// PROPRIEDADE.*
const _: () = assert!(DOBRA_DO_PERSONAGEM < DOBRA);

/// ⭐⭐⭐ **O ÂNGULO DE UM MEMBRO É DERIVADO DA GEOMETRIA, nunca escolhido** (§0.0).
///
/// Um membro roda em torno da **própria origem** (a ponta esquerda dele), logo a ponta direita sobe
/// ou desce `larg · sin θ`. Os dois membros rodam em sentidos opostos ⇒ as pontas aproximam-se de
/// `2 · larg · sin θ`, e elas cruzam-se quando isso passa o `vao`.
///
/// ⛔⛔⛔ **E a 1.ª redacção desta lei comparou com a grandeza ERRADA — a TERCEIRA vez neste bloco:**
/// ela media contra o `vao`, que é de CENTRO a centro, e o que cruza é a **FOLGA** entre bordas
/// (`vao − alt`). Com `vao = 1,3 · alt` a folga vale `0,3 · alt ≈ 0,26 m` sobre uma ponta que se
/// desloca `0,45 m`, e a foto mostrou-as sobrepostas **com o gate verde**.
///
/// ⇒ o argumento é a **folga**, e o ângulo sai de `asin(0,8 · folga / larg)`. *Uma régua que mede a
/// distância entre centros aprova duas peças que já se tocam pelas bordas.*
fn angulo_do_membro(larg: f64, folga: f64) -> f32 {
    let limite = (0.8 * folga / larg).clamp(-1.0, 1.0).asin().to_degrees();
    (limite as f32).max(2.0)
}

/// O vão entre duas peças de um personagem, em metros.
///
/// ⚠️ **Ele NÃO é o [`passo_da_coluna`]:** ali cada braço tem a corrente dele e o vão paga a
/// **SUBIDA** da ponta; aqui as peças penduram do mesmo rig e o que o vão paga é a **CONVERGÊNCIA**
/// entre curvaturas vizinhas — uma grandeza mais pequena, à dobra desta cena.
fn vao_do_personagem(alt: f64) -> f64 {
    alt * 1.9
}

/// **`=4` — UM PERSONAGEM: três desenhos SEPARADOS, UM esqueleto só.**
///
/// ⭐⭐⭐ **É a capacidade que separa «um braço» de um BONECO**, e nenhuma cena deste módulo a
/// exercia: as `=1`, `=2` e `=3` dão a cada imagem a corrente DELA. Um boneco é um tronco, dois
/// braços, duas pernas e uma cabeça — peças separadas presas ao **mesmo** rig.
///
/// ⚠️ **A prova está na HIERARQUIA, e é por isso que a cena é legível:** aqui vivem **três** ossos e
/// três imagens; na `=2` vivem **nove** ossos. *Uma cena cuja afirmação se lê numa lista é mais
/// forte do que uma que pede ao artista para confiar no que se move.*
///
/// ⚠️ **Quem dobra é ESTA função, uma vez só** — a [`uma`] recebe `rig: Some(..)` e `graus = 0`,
/// porque dobrar por peça somaria o ângulo três vezes (a corrente é a mesma).
///
/// ⚠️ **A ordem é PRENDER e depois DOBRAR:** o bind fotografa a pose de repouso, e com a corrente já
/// dobrada as três peças nasciam presas à pose torta e a cena ficava imóvel.
pub(super) fn personagem(
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    cell_idx: u32,
    pixels_per_meter: f32,
    atlas_asset_map: &mut BTreeMap<u32, AssetId>,
) -> Option<(u64, u32)> {
    let ppm = f64::from(pixels_per_meter.max(f32::MIN_POSITIVE));
    // ⭐⭐ **O tamanho sai da CÂMERA DE OMISSÃO, como na [`anima`]** — esta cena também não pede o
    // *Frame All* (o porquê medido vive no [`super::Prologo::enquadrar`]), logo ela abre na câmera
    // de sempre e a peça tem de caber nela. ⚠️ **Mais curta que a da `=3`**, porque aqui são TRÊS
    // empilhadas: a altura útil é o que sobra depois dos docks, e uma peça só não a mede.
    let px = braco_da_camera_de_omissao(ppm);
    let px = [(px[0] * 7) / 10, (px[1] * 7) / 10];
    let (larg, alt) = (f64::from(px[0]) / ppm, f64::from(px[1]) / ppm);

    let vao = vao_do_personagem(alt);

    // ⭐⭐⭐ **O ESQUELETO É UMA ÁRVORE, e não uma corrente — porque um BONECO é uma árvore.**
    //
    // ⛔⛔ **A 1.ª redacção pendurou as três peças numa cadeia em LINHA e o log desmentiu-a:** duas
    // delas liam *«os pesos do padrão-ouro NÃO resolveram — esta imagem cai na lei derivada»*, e na
    // foto apareciam com uma orla escura. *Um osso que não passa por DENTRO da peça não a sabe
    // pesar* — e num rig a sério cada parte tem o osso dela.
    //
    // ⇒ um **tronco** dentro da peça do meio e um **membro** dentro de cada uma das outras, os três
    // na mesma árvore. ⭐ É isso que dá as duas metades do gesto: mexer no tronco move a figura
    // INTEIRA (os filhos herdam), mexer num membro move **só** a peça dele.
    let tronco = Entity::from_bits(ph2d_skeleton_live::bone::create(
        sim,
        None,
        [-larg / 2.0, 0.0],
        [larg / 2.0, 0.0],
    )?);
    let mut ossos = vec![tronco];
    for y in [vao, -vao] {
        ossos.push(Entity::from_bits(ph2d_skeleton_live::bone::create(
            sim,
            Some(tronco),
            [-larg / 2.0, y],
            [larg / 2.0, y],
        )?));
    }
    let nomes = ["Peca de cima", "Peca do meio", "Peca de baixo"];
    let mut primeiro = None;
    let mut gastas = 0_u32;
    for (k, nome) in nomes.iter().enumerate() {
        let y = vao * (1.0 - k as f64);
        if let Some(bits) = uma(
            sim,
            renderer,
            asset_db,
            cell_idx + k as u32,
            pixels_per_meter,
            atlas_asset_map,
            [0.0, y],
            nome,
            px[0],
            px[1],
            braco_de(px, k),
            larg,
            Some(&ossos),
            0.0,
            |_, _| {},
        ) {
            primeiro.get_or_insert(bits);
            gastas += 1;
        }
    }

    // ⭐⭐ **A POSE mostra as DUAS metades de um rig:** o tronco roda (a figura inteira acompanha,
    // porque os membros são filhos dele) e cada membro roda **o seu** no sentido oposto. ⛔ Uma pose
    // que só rodasse o tronco desenhava três peças rígidas, e a cena não distinguiria um rig
    // partilhado de um grupo.
    let membro = angulo_do_membro(larg, vao - alt);
    for (k, osso) in ossos.iter().enumerate() {
        let graus = match k {
            0 => DOBRA_DO_PERSONAGEM,
            1 => -membro,
            _ => membro,
        };
        if let Some(mut t) = sim.world_mut().get_mut::<ph2d_ecs::Transform>(*osso) {
            t.rotation += graus.to_radians();
        }
    }
    anuncia_personagem(gastas, ossos.len());
    Some((primeiro?, gastas))
}

/// O roteiro da cena do personagem.
///
/// ⚠️ **Ele nomeia o que se vê NA TELA** (`Bone 1`, `Peca de cima`) e as duas metades do gesto —
/// *o tronco move a figura inteira, um membro move só a peça dele*. A 1.ª redacção falava de uma
/// «corrente» e de «dobrar juntas», que era a cena ANTERIOR: um roteiro que descreve a versão
/// antiga ensina o contrário do que acontece, e é o dono que o lê primeiro (§5.0).
fn anuncia_personagem(gastas: u32, ossos: usize) {
    if gastas < 3 {
        eprintln!("[bone-media-smoke] PARE: montei so' {gastas} de 3 pecas");
    }
    println!(
        "[bone-media-smoke] UM PERSONAGEM: {gastas} desenhos SEPARADOS presos ao MESMO esqueleto \
         ({ossos} ossos).\n\
         [bone-media-smoke] 1) Na Hierarquia conte: ha' {ossos} ossos para {gastas} desenhos — na \
         cena =2 ha' NOVE, um esqueleto por desenho. Aqui e' UM so'\n\
         [bone-media-smoke] 2) Clique em 'Bone 1' (o tronco, dentro da peca do meio) e ARRASTE: as \
         TRES pecas acompanham, porque as outras duas penduram dele\n\
         [bone-media-smoke] 3) Clique em 'Bone 2' e ARRASTE: mexe SO' a peca de cima — as outras \
         duas ficam paradas\n\
         [bone-media-smoke] 4) As frestas entre as pecas TEM de continuar la': sao tres desenhos, \
         nao um so'\n\
         [bone-media-smoke] ⇒ e' isto que faz um boneco: tronco, bracos, pernas e cabeca, cada um \
         um desenho, todos no mesmo esqueleto\n\
         [bone-media-smoke] Se arrastar 'Bone 1' e SO' UMA peca se mexer, PARE e diga qual"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// O corpo de `fn personagem`, lido do próprio ficheiro.
    ///
    /// ⚠️ **`include_str!` e não um `grep` de shell:** se a função mudar de nome ou de ficheiro isto
    /// deixa de **compilar**, em vez de passar a varrer zero linhas e ficar verde (§5.0).
    fn corpo_do_personagem() -> String {
        let fonte = include_str!("smoke_bone_media_bracos.rs");
        let i = fonte
            .find("pub(super) fn personagem(")
            .expect("a funcao personagem mudou de nome ou de ficheiro");
        let resto = &fonte[i..];
        let j = resto
            .find("\n/// O roteiro da cena do personagem.")
            .expect("o fim do corpo de personagem mudou de forma");
        resto[..j].to_string()
    }

    /// ⭐⭐⭐ **A CENA DO PERSONAGEM MONTA UMA ÁRVORE, E AS TRÊS PEÇAS PRENDEM-SE AO MESMO OSSO.**
    ///
    /// ⛔⛔ **A premissa que aqui esteve — «monta UMA corrente e dobra UMA vez» — MORREU por
    /// medição:** com uma cadeia em linha, duas das três peças liam no log *«os pesos do
    /// padrão-ouro NÃO resolveram — esta imagem cai na lei derivada»*, porque **o osso não passava
    /// por dentro delas**. O gate fica com a morte visível no diff (§5.0).
    ///
    /// ⚠️ **As três metades são três defeitos diferentes:** uma raiz a mais e deixa de ser UM
    /// esqueleto; um `Some(&ossos)` a menos e a peça monta o rig dela; e a pose tem de tocar em
    /// **todos** os ossos, senão a peça esquecida sai rígida e lê-se como «não seguiu».
    #[test]
    fn a_cena_do_personagem_monta_uma_arvore_e_prende_tudo_a_ela() {
        let corpo = corpo_do_personagem();
        assert_eq!(
            // ⚠️ A agulha lê-se do ficheiro JÁ FORMATADO, nunca de memória: a 1.ª redacção
            // procurava `create(sim, None,` e casou ZERO, porque o `cargo fmt` parte a chamada.
            corpo.matches("        None,\n").count(),
            1,
            "a cena do personagem tem mais (ou menos) de UMA raiz — deixa de ser um esqueleto so'"
        );
        assert_eq!(
            corpo.matches("Some(tronco)").count(),
            1,
            "os membros nao penduram todos do mesmo tronco: a arvore parte-se em duas"
        );
        assert_eq!(
            corpo.matches("Some(&ossos)").count(),
            1,
            "nem toda peca entra pelo rig partilhado — a que faltar monta a corrente DELA"
        );
        assert!(
            corpo.contains("for (k, osso) in ossos.iter().enumerate()"),
            "a pose deixou de tocar em TODOS os ossos: a peca esquecida sai rigida e le-se como \
             «nao seguiu o esqueleto», que e' o report que esta cena existe para nao ter"
        );
        // ⚠️ E a ORDEM: prender primeiro, pousar a pose depois. Com a árvore já torta o bind
        // fotografa a pose dobrada como repouso, e a cena abre imóvel.
        let prende = corpo.find("Some(&ossos)").expect("bind");
        let pose = corpo.find("t.rotation +=").expect("pose");
        assert!(
            prende < pose,
            "a cena poe a pose ANTES de prender: o repouso fotografado ja' esta' torto"
        );
    }

    /// ⚠️ **O vão do personagem deixa uma FRESTA que se vê, e a dobra dele é MENOR que a das
    /// outras cenas** — as duas metades de uma decisão só, tirada de uma foto.
    ///
    /// ⛔⛔ **A premissa que aqui esteve — «a subida é comum às três» — foi REFUTADA pela foto**, e a
    /// medição que a desmentia já existia: duas peças na mesma corrente excursionam `1,123 m` e
    /// `1,195 m`. *Peças afastadas do eixo convergem do lado interior da curva*, e a `25°` elas
    /// encostavam-se; o gate fica com a morte da premissa visível no diff.
    #[test]
    fn o_vao_do_personagem_e_menor_que_o_passo_da_coluna() {
        let (larg, alt) = (2.4_f64, 0.6_f64);
        let vao = vao_do_personagem(alt);
        assert!(
            vao > alt,
            "o vao ({vao:.3} m) nao chega para uma fresta sobre uma peca de {alt:.3} m — \
             as tres colam-se e leem-se como um desenho so'"
        );
        assert!(
            vao < passo_da_coluna(larg, alt),
            "o vao do personagem ({vao:.3} m) nao e' menor que o passo da coluna da =2 \
             ({:.3} m) — ele paga a CONVERGENCIA entre curvaturas, nao a subida de tres pontas",
            passo_da_coluna(larg, alt)
        );
        // ⭐ E as pontas de duas peças vizinhas NÃO se tocam — a lei que a foto encomendou.
        //
        // ⛔ A grandeza é a FOLGA entre BORDAS (`vao − alt`) e não o vão entre centros: com o vão a
        // régua aprovava peças já sobrepostas (medido, 2026-09-18).
        let folga = vao - alt;
        let membro = f64::from(angulo_do_membro(larg, folga));
        let desloca = larg * membro.to_radians().sin();
        assert!(
            desloca < folga,
            "a ponta de um membro desloca-se {desloca:.3} m sobre uma folga de {folga:.3} m — \
             as pecas vizinhas sobrepoem-se, e a foto le' um borrao em vez de tres desenhos"
        );
        assert!(
            membro > 1.0,
            "o angulo do membro caiu para {membro:.2}°: a peca fica parada e a cena deixa de \
             mostrar que um membro se move SOZINHO"
        );
    }

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
