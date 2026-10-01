//! ⭐⭐⭐⭐ **OS GATES DA OCLUSÃO NO TEMPO** — o céu de um ponto calculado UMA vez e reaproveitado
//! enquanto a câmara gira (`ph2d_field_gpu::ceu_tempo`; ordem do dono de 2026-09-29: *«o render
//! ainda não está em tempo real — somos uma game engine»*).
//!
//! Relógio: a tabela no doc do [`ph2d_field_gpu::ceu_tempo`] (girar, aproximar e afastar, com o
//! chão sem céu ao lado).
//!
//! As metades que os gates afirmam são as coisas que a cache pode fazer mal: ler uma célula errada
//! (o quadro parado), deixar a imagem derivar num gesto de câmara, RECOMEÇAR a tabela num gesto de
//! câmara (o zoom que não era tempo real), herdar de uma peça que já não é a mesma, e ter menos
//! entradas do que a vista pede.

use super::super::*;

/// Um quadro da cena `cena` com a câmara `cam`, pela porta da sonda.
#[allow(clippy::too_many_arguments)]
fn quadro(
    t: &crate::gpu_frame::SharedTracer,
    doc: &ph2d_field::FieldDoc,
    cam: &ph2d_field_render::Orbit,
    assente: bool,
    sonda: crate::gpu_frame::Sonda,
) -> Vec<u8> {
    let reg = crate::smoke::sampled_registry();
    let materiais = [ph2d_material::OpenPbr::default().prepare()];
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: None,
    };
    let pres = ph2d_field_render::Presentation::of(ph2d_view_transform::Look::default());
    // ⚠️ A luz fica FIXA no mundo: girar não pode trocar a luz, senão o quadro mediria outra cena.
    let luz = [crate::gpu_frame::tests_lampada(
        &ph2d_field_render::Orbit::default(),
    )];
    let chao = ph2d_field_render::lowest_point(doc, &reg)
        .map(|height| ph2d_field_render::Ground { height });
    crate::gpu_frame::paint_com(
        t,
        doc,
        &reg,
        cam,
        &luz,
        &surfaces,
        &pres,
        [40, 40, 40, 255],
        chao,
        LW,
        LH,
        assente,
        sonda,
    )
    .expect("o pintor")
    .rgba
}

/// A cache ligada, com o resto da fábrica.
fn com_cache() -> crate::gpu_frame::Sonda {
    crate::gpu_frame::Sonda {
        ceu_no_tempo: true,
        ..crate::gpu_frame::Sonda::default()
    }
}

/// A régua: a oclusão EXACTA (os `48` cones em todo pixel), sem cache.
fn exacta() -> crate::gpu_frame::Sonda {
    crate::gpu_frame::Sonda {
        ceu_no_tempo: false,
        ceu_passo: 1,
        ..crate::gpu_frame::Sonda::default()
    }
}

/// O quadro de movimento de antes da cache: a oclusão a passo `2`.
fn a_passo() -> crate::gpu_frame::Sonda {
    crate::gpu_frame::Sonda {
        ceu_no_tempo: false,
        ..crate::gpu_frame::Sonda::default()
    }
}

/// Quantos canais diferem, quantos mais de `8` níveis, e o pior.
fn diferenca(a: &[u8], b: &[u8]) -> (usize, usize, u8) {
    let (mut algum, mut acima, mut pior) = (0usize, 0usize, 0u8);
    for (x, y) in a.iter().zip(b) {
        let d = x.abs_diff(*y);
        pior = pior.max(d);
        if d > 0 {
            algum += 1;
        }
        if d > 8 {
            acima += 1;
        }
    }
    (algum, acima, pior)
}

/// Quantos pixels são um PONTO CLARO isolado: mais de `12` níveis acima do mais claro dos oito
/// vizinhos — a régua dos *«pontos»* das fotos do dono.
fn lum(img: &[u8], x: usize, y: usize) -> u32 {
    let i = (y * LW as usize + x) * 4;
    (u32::from(img[i]) + u32::from(img[i + 1]) + u32::from(img[i + 2])) / 3
}

/// Os píxeis mais claros do que os OITO vizinhos por mais de `12` níveis.
fn claros(img: &[u8]) -> Vec<(usize, usize)> {
    let (w, h) = (LW as usize, LH as usize);
    let mut v = Vec::new();
    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let c = lum(img, x, y);
            let mut viz = 0;
            for (dx, dy) in [
                (0, 1),
                (2, 1),
                (1, 0),
                (1, 2),
                (0, 0),
                (2, 0),
                (0, 2),
                (2, 2),
            ] {
                viz = viz.max(lum(img, x + dx - 1, y + dy - 1));
            }
            if c > viz + 12 {
                v.push((x, y));
            }
        }
    }
    v
}

fn pontos_claros(img: &[u8]) -> usize {
    claros(img).len()
}

/// ⭐⭐⭐ **Os pontos claros NOVOS** — claros na imagem da cache e mais de `8` níveis acima da exacta
/// no MESMO pixel. ⛔ A contagem crua (`pontos_claros`) contra a da exacta media o LIMIAR: a peça já
/// tem faíscas na silhueta, e uma delas a `129` contra `128` da exacta (com o vizinho a `116`) entra
/// de um lado e não do outro — medido na rosca a girar, `16` «a mais» de que só `3` passam `8` níveis.
fn pontos_novos(img: &[u8], certo: &[u8]) -> usize {
    claros(img)
        .into_iter()
        .filter(|&(x, y)| lum(img, x, y) > lum(certo, x, y) + 8)
        .count()
}

/// Assenta a cena (o quadro que GRAVA a tabela), esquecendo a de antes.
fn assenta(t: &crate::gpu_frame::SharedTracer, doc: &ph2d_field::FieldDoc) {
    if let Ok(mut g) = t.lock() {
        g.esquece_o_ceu();
    }
    let _ = quadro(
        t,
        doc,
        &ph2d_field_render::Orbit::default(),
        true,
        com_cache(),
    );
}

/// ⭐⭐⭐⭐ **PARADO, O MOVIMENTO LÊ A OCLUSÃO DO ASSENTE** — sem girar, o quadro de movimento que
/// herda da tabela tem de dar a oclusão EXACTA (a que o assente gravou), a menos da média de uma
/// célula.
///
/// ⭐ **A barra NÃO é escolhida: é o quadro que a cache SUBSTITUI.** O de passo `2` contra a exacta
/// lê `31` canais acima de `8` níveis, pior `14` — uma cache pior do que isso seria uma regressão
/// comprada com relógio. Medido no nó: `0` acima de `8`, pior `7`. ⛔ Com a 1.ª redacção do assente
/// (a célula guardava o céu do ÚLTIMO pixel a escrever) lia `203`–`253`, pior `24`–`26`.
///
/// ⛔ **O CONTROLO vem primeiro:** o quadro herdado tem de DIFERIR do de passo `2` — se a tabela não
/// fosse lida, o quadro de movimento seria o de antes e a barra passaria sobre ele POR IGUALDADE.
#[test]
#[ignore = "precisa de GPU"]
fn parado_o_movimento_le_a_oclusao_do_assente() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let doc = crate::smoke::scene(28);
    let cam = ph2d_field_render::Orbit::default();
    let certo = quadro(t, &doc, &cam, false, exacta());
    let antes = quadro(t, &doc, &cam, false, a_passo());
    assenta(t, &doc);
    let herdado = quadro(t, &doc, &cam, false, com_cache());
    let (mexe, ..) = diferenca(&herdado, &antes);
    let (algum, acima, pior) = diferenca(&herdado, &certo);
    let (_, acima_p2, pior_p2) = diferenca(&antes, &certo);
    println!(
        "herdado contra a exacta: {algum} canais · {acima} acima de 8 · pior {pior}; contra o de \
         passo 2: {mexe} canais; o de passo 2 contra a exacta: {acima_p2} acima de 8 · pior \
         {pior_p2}"
    );
    assert!(
        mexe > 1_000,
        "CONTROLO: o quadro herdado é o de passo 2 ({mexe} canais de diferença) — a tabela não foi \
         lida, e a barra abaixo não afirmaria nada"
    );
    assert!(
        acima <= acima_p2 && pior <= pior_p2,
        "parado, o quadro herdado é PIOR do que o de passo 2 que ele substitui: {acima} canais \
         acima de 8 contra {acima_p2}, pior {pior} contra {pior_p2}"
    );
}

/// ⭐⭐⭐⭐ **OS GESTOS DE CÂMARA FICAM PERTO DA EXACTA E NÃO RECOMEÇAM A TABELA** — `12` quadros de
/// cada gesto (girar `2°`, aproximar `3 %`, afastar `3 %`) e o último comparado com a oclusão exacta
/// na mesma câmara, em DUAS cenas: o nó (tubos que se cruzam) e a rosca (filetes paralelos, onde as
/// células vizinhas discordam mais).
///
/// ⭐ **A metade que é o TEMPO REAL:** nenhum gesto de câmara recomeça a tabela. ⛔⛔ Até 2026-09-30
/// o alcance da oclusão e os limiares de pixel vinham da câmara e entravam na chave, e cada quadro de
/// aproximar recomeçava a tabela e pagava a oclusão inteira — o gate de antes só GIRAVA, à distância
/// fixa, e não o podia ver (`ph2d_field_render::OCCLUSION_REACH`).
///
/// ⭐ **As barras saem do VALE medido** (`diag_os_gestos_da_camara`, três repetições por célula):
/// ver [`BARRAS`].
#[test]
#[ignore = "precisa de GPU"]
fn os_gestos_da_camara_ficam_perto_da_exacta() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    for &(cena, qual, acima_max, pior_max) in BARRAS {
        let doc = crate::smoke::scene(cena);
        let g = depois_do_gesto(t, &doc, qual);
        println!(
            "cena {cena} gesto {qual}: {} acima de 8 · pior {} · pontos claros {} (a exacta tem {}) \
             · {} reinícios",
            g.acima, g.pior, g.pontos, g.pontos_certos, g.reinicios
        );
        assert_eq!(
            g.reinicios, 0,
            "cena {cena} gesto {qual}: o gesto de câmara RECOMEÇOU a tabela {} vezes — a oclusão foi \
             paga inteira em vez de herdada",
            g.reinicios
        );
        assert!(
            g.acima <= acima_max && g.pior <= pior_max,
            "cena {cena} gesto {qual}: a oclusão herdada afasta-se da exacta: {} canais acima de 8 \
             (tecto {acima_max}), pior {} (tecto {pior_max})",
            g.acima,
            g.pior
        );
        assert!(
            g.novos <= NOVOS_MAX,
            "cena {cena} gesto {qual}: a cache pinta PONTOS CLAROS que a exacta não tem: {} (tecto \
             {NOVOS_MAX}; claros {} contra {} da exacta)",
            g.novos,
            g.pontos,
            g.pontos_certos
        );
    }
}

/// ⭐ **`(cena, gesto, tecto dos canais acima de 8, tecto do pior)`** — o vale entre a lei e as que
/// a prova de mutação separa. Medido a `1920×1080` (mín–máx de três repetições), **2026-09-30 com a
/// lei nova** (o chão na tabela, as fatias por quadro, a cópia entre níveis):
///
/// | cena · gesto | a lei | concordância desligada | tabela a `2` por pixel |
/// |---|---|---|---|
/// | nó · girar | `15`–`33` · pior `11`–`12` | `12`–`48` | `173`–`196` · pior `77` |
/// | rosca · girar | `75`–`104` · pior `23` | `80`–`98` | `383`–`427` · pior `34` |
/// | nó · aproximar | `89`–`178` · pior `22`–`26` | `70`–`184` | `1 020`–`1 044` |
/// | rosca · aproximar | `415`–`537` · pior `21`–`33` | `424`–`515` | `603`–`718` |
/// | nó · afastar | `0`–`10` · pior `7`–`17` | `0`–`19` | `21`–`62` · pior `11`–`24` |
/// | rosca · afastar | `44`–`54` · pior `22` | `34`–`77` | `80`–`103` |
///
/// ⛔ **A CONCORDÂNCIA deixou de ser visível nestes gestos** (a coluna dela cai dentro da da lei): com
/// a cópia entre níveis e as fatias por quadro o erro que sobra mora noutro sítio, e a mutação que a
/// desliga SOBREVIVE a este gate — dito aqui em vez de uma barra que fingisse vê-la. ⚠️ A rosca a
/// afastar também não separa a tabela a `2` por pixel; a barra dela guarda a capacidade da tabela.
/// A herança com UMA vizinha (`quantas < 1`) sangra pela contagem: nó a girar `109`–`122`, nó a
/// afastar `149`–`173`, rosca a girar `496`–`552`.
const BARRAS: &[(u32, u32, usize, u8)] = &[
    (28, 0, 45, 30),
    (29, 0, 260, 30),
    (28, 1, 360, 45),
    (29, 1, 570, 35),
    (28, 2, 20, 22),
    (29, 2, 110, 45),
];

/// ⭐ O tecto dos [`pontos_novos`] — medido `0` no nó e `1`–`9` na rosca, nos três gestos, com a lei e
/// com as duas mutações da tabela acima: **ele não separa essas mutações** (quem as apanha são as
/// barras de contagem). Ele é a guarda contra o SAL — pontos claros que a exacta não tem —, a
/// assinatura de duas células diferentes fundidas numa entrada (a impressão da chave, 2026-09-30).
const NOVOS_MAX: usize = 12;

/// ⭐⭐⭐⭐ **UMA EDIÇÃO DA PEÇA NÃO HERDA NADA** — o quadro de movimento cuja peça mudou (a mão a
/// arrastar um parâmetro) faz a oclusão de antes, BYTE A BYTE, e a tabela recomeça. ⛔ Sem isto a
/// tabela vazia daria a cada pixel UMA fatia de cones, ou — pior — o céu da peça de antes.
#[test]
#[ignore = "precisa de GPU"]
fn uma_edicao_da_peca_nao_herda_nada() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let antes_doc = crate::smoke::scene(28);
    let depois_doc = crate::smoke::scene(29);
    let cam = ph2d_field_render::Orbit::default();
    let esperado = quadro(t, &depois_doc, &cam, false, a_passo());
    assenta(t, &antes_doc);
    let r0 = t.lock().map_or(0, |g| g.ceu_tempo_reinicios());
    let editado = quadro(t, &depois_doc, &cam, false, com_cache());
    let r1 = t.lock().map_or(0, |g| g.ceu_tempo_reinicios());
    let (algum, _, pior) = diferenca(&editado, &esperado);
    assert_eq!(
        r1,
        r0 + 1,
        "a peça mudou e a tabela não recomeçou ({r0} → {r1} reinícios)"
    );
    assert_eq!(
        algum, 0,
        "o quadro da peça editada não é a oclusão de antes: {algum} canais, pior {pior}"
    );
}

/// ⭐⭐⭐⭐ **O PRODUTO LIGA A CACHE** — a porta de fábrica pede a oclusão no tempo.
/// ⛔ Sem esta metade a cura podia existir só na sonda.
#[test]
fn o_produto_liga_a_oclusao_no_tempo() {
    if std::env::var("PH2D_FIELD_CEU_TEMPO").is_ok() {
        println!("PH2D_FIELD_CEU_TEMPO definido — a fábrica não é a que corre; saltado");
        return;
    }
    assert!(
        crate::gpu_frame::Sonda::default().ceu_no_tempo,
        "a Sonda de fábrica não liga a oclusão no tempo"
    );
}

/// Os três gestos de câmara que a cache tem de aguentar: girar, aproximar e afastar.
fn gesto(cam: &mut ph2d_field_render::Orbit, qual: u32) {
    match qual {
        0 => cam.turn_world([0.0, 1.0, 0.0], 2f32.to_radians()),
        1 => cam.half_extent *= 0.97,
        _ => cam.half_extent *= 1.03,
    }
}

/// O que um gesto deixa: canais acima de `8` níveis e o pior contra a exacta, os pontos claros dos
/// dois lados, e quantas vezes a tabela recomeçou DURANTE o gesto.
struct Gesto {
    acima: usize,
    pior: u8,
    pontos: usize,
    pontos_certos: usize,
    novos: usize,
    reinicios: usize,
}

/// `12` quadros do gesto `qual` a partir do assente, e o último contra a exacta na mesma câmara.
/// `PH2D_SONDA_DIR` grava as duas imagens.
fn depois_do_gesto(
    t: &crate::gpu_frame::SharedTracer,
    doc: &ph2d_field::FieldDoc,
    qual: u32,
) -> Gesto {
    assenta(t, doc);
    let r0 = t.lock().map_or(0, |g| g.ceu_tempo_reinicios());
    let mut cam = ph2d_field_render::Orbit::default();
    let mut img = Vec::new();
    for _ in 0..12 {
        gesto(&mut cam, qual);
        img = quadro(t, doc, &cam, false, com_cache());
    }
    let reinicios = t.lock().map_or(0, |g| g.ceu_tempo_reinicios()) - r0;
    let certo = quadro(t, doc, &cam, false, exacta());
    let (_, acima, pior) = diferenca(&img, &certo);
    if let Ok(dir) = std::env::var("PH2D_SONDA_DIR") {
        for (nome, im) in [("tempo", &img), ("exacta", &certo)] {
            let mut ppm = format!("P6\n{LW} {LH}\n255\n").into_bytes();
            for px in im.as_chunks::<4>().0 {
                ppm.extend_from_slice(&px[..3]);
            }
            std::fs::write(format!("{dir}/g{qual}_{nome}.ppm"), ppm).expect("grava");
        }
    }
    Gesto {
        acima,
        pior,
        pontos: pontos_claros(&img),
        pontos_certos: pontos_claros(&certo),
        novos: pontos_novos(&img, &certo),
        reinicios,
    }
}

/// Sonda: os três gestos nas duas cenas, três repetições — a variância da corrida, de onde saem as
/// [`BARRAS`].
#[test]
#[ignore = "sonda — precisa de GPU"]
fn diag_os_gestos_da_camara() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    for cena in [28u32, 29] {
        let doc = crate::smoke::scene(cena);
        for qual in 0..3 {
            for rep in 0..3 {
                let g = depois_do_gesto(t, &doc, qual);
                println!(
                    "DIAG cena {cena} gesto {qual} rep {rep}: {} acima · pior {} · pontos {}/{} · \
                     novos {} · {} reinícios",
                    g.acima, g.pior, g.pontos, g.pontos_certos, g.novos, g.reinicios
                );
            }
        }
    }
}

/// ⭐⭐⭐ **A TABELA SEGUE A VISTA, E O TECTO É O DO DISPOSITIVO** — `4` entradas por pixel (ver
/// [`ph2d_field_gpu::ceu_tempo::ENTRADAS_POR_PIXEL`]), e nunca mais do que o maior buffer que a placa
/// deixa ligar. ⛔ Até 2026-09-30 era a constante `2²¹`, e a `1920×1080` ela enchia (o sal nas fendas).
#[test]
fn a_tabela_segue_a_vista_e_o_tecto_do_dispositivo() {
    use ph2d_field_gpu::ceu_tempo::{LAMPADAS_NA_TABELA, entradas_para, palavras_para};
    let hd = 1920 * 1080;
    let grande = 4 << 30;
    let piso_wgpu = 128 << 20;
    let ceu = palavras_para(0);
    assert_eq!(ceu, 5, "sem lâmpadas, as cinco palavras do céu");
    assert_eq!(entradas_para(hd, grande, ceu), 4 * 1920 * 1080);
    assert_eq!(
        entradas_para(4 * hd, grande, ceu),
        16 * 1920 * 1080,
        "a 4K a tabela cresce com a vista"
    );
    assert_eq!(
        entradas_para(hd, piso_wgpu, ceu),
        u32::try_from(piso_wgpu / 20).expect("cabe"),
        "no piso da wgpu quem manda é o tamanho da ligação"
    );
    // ⭐ As LÂMPADAS: uma palavra de contagem e uma por par; acima do tecto voltam ao pixel.
    assert_eq!(palavras_para(1), 7);
    assert_eq!(palavras_para(2), 7);
    assert_eq!(palavras_para(3), 8);
    assert_eq!(palavras_para(LAMPADAS_NA_TABELA), 8);
    assert_eq!(palavras_para(LAMPADAS_NA_TABELA + 1), 5);
    assert_eq!(
        entradas_para(hd, piso_wgpu, 8),
        u32::try_from(piso_wgpu / 32).expect("cabe"),
        "com lâmpadas a entrada é maior e cabem menos na ligação"
    );
    assert_eq!(
        entradas_para(16, grande, ceu),
        1 << 16,
        "uma vista minúscula tem um piso"
    );
}
