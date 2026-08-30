//! ⭐⭐⭐ **O ATLAS DE IMAGEM PERSISTENTE** — a sonda e os dois gates que a `vello` 0.10 passou a
//! exigir de quem redesenha a mesma imagem em muitos quadros.
//!
//! # O que mudou por baixo de nós, sem uma linha nossa se mexer
//!
//! Até à `vello` 0.8 o atlas de imagens era **limpo a cada render**. Isso tornava a porta
//! [`VectorScene::draw_image_rgba`] — que constrói uma `Blob` nova, logo um **id novo**, a cada
//! chamada — barata *por construção*: ninguém acumulava, porque ninguém residia.
//!
//! A **0.10 tornou o atlas persistente** ([`vello_encoding`], `image_cache.rs`):
//!
//! - o atlas é **quadrado**, nasce a `1024` e **dobra** até `8192` (`MAX_ATLAS_SIZE`);
//! - uma imagem entra pelo `id` da `Blob` e fica **residente**;
//! - só sai quando não é usada há `EVICT_AFTER_GENERATIONS = 2` passes de resolução, **e** só
//!   quando uma alocação falha (o despejo é reactivo, não periódico);
//! - crescer o atlas é um **repack completo** que marca **toda** imagem residente como suja ⇒
//!   re-envio de tudo;
//! - e quando nem o despejo nem o crescimento chegam, o `resolve_pending_images` põe
//!   `xy = None` e a imagem **não é desenhada — em silêncio**. Um modo de falha que na 0.8 era
//!   inalcançável.
//!
//! ⇒ Um produtor que emita **um id novo por quadro** deixou de pagar só o envio: ele passa a
//! encher um recurso partilhado, com uma cauda de 2 a 3 quadros, competindo com todos os outros
//! desenhos de imagem do app pelo mesmo `8192²`.
//!
//! # Porque a régua é o `Resolver`, e não um relógio nem a GPU
//!
//! As três perguntas — *quanto cresceu o atlas · quantos envios por quadro · quantos despejos* —
//! são **decididas na CPU**, de forma determinística, dentro do [`vello_encoding::Resolver`]. Medir
//! isto pelo `Renderer` exigiria adaptador de GPU, e os gates de GPU desta casa são `#[ignore]`,
//! logo o CI nunca os corre (`docs/Atualizar Stack/04_registro.md` §20.2: **428 testes** passam sem
//! testar nada por isso). ⭐ *Uma medição que só corre onde ninguém a corre não é uma medição.*
//!
//! ⚠️ E a régua **não é um relógio**: nenhum número aqui é tempo, então nada disto entra na família
//! de flakes de carga do `CLAUDE.md` §5.0.
//!
//! # A honestidade do arnês
//!
//! O laço abaixo espelha o que o app faz, e a lista é curta de propósito — *uma fixtura prova o que
//! amostrou*:
//!
//! | o app | o arnês |
//! |---|---|
//! | uma [`VectorScene`] por quadro, `reset()` no início | uma `VectorScene` nova por quadro |
//! | **um** `Resolver` vivo dentro do `Renderer`, entre quadros | **um** `Resolver` fora do laço |
//! | N vistas a desenhar N imagens do tamanho da área | `viewports` imagens de `w × h` |
//!
//! ⛔ O que o arnês **não** encena: os outros desenhos de imagem do app (que competem pelo mesmo
//! atlas) e os glifos (que têm cache próprio, `GlyphCache`, e não tocam neste). ⇒ os números da
//! sonda são um **piso** do que acontece na tela, nunca um tecto.

use super::*;
use std::sync::Arc;
use vello::kurbo::Affine;
use vello::peniko::ImageQuality;

/// Pixels distintos por semente — para que duas imagens de quadros diferentes não sejam
/// acidentalmente o mesmo conteúdo (o `id` da `Blob` é por alocação, não por conteúdo, mas uma
/// fixtura que devolvesse sempre os mesmos bytes convidaria a próxima leitura ao engano).
fn rgba(w: u32, h: u32, seed: u8) -> Arc<Vec<u8>> {
    Arc::new(vec![seed; (w as usize) * (h as usize) * 4])
}

/// Como o produtor entrega a imagem ao Vello.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    /// A porta crua: `Arc` novo ⇒ `Blob` nova ⇒ **id novo** a cada quadro.
    Raw,
    /// O [`StableImage`]: construído UMA vez por traçado e **clonado** por quadro (clone de `Blob`
    /// = refcount + MESMO id) ⇒ o `ImageCache` acerta e salta o envio.
    Stable,
}

/// O que a sonda colhe ao longo de `frames` quadros.
#[derive(Debug, Default)]
struct Report {
    /// Lado do atlas quadrado no fim (px).
    atlas_side: u32,
    /// Quantas imagens foram **enviadas** à GPU, somadas sobre todos os quadros.
    uploads: usize,
    /// Quantos bytes esses envios custaram.
    upload_bytes: u64,
    /// Quantas residentes foram despejadas, somadas sobre todos os quadros.
    evictions: usize,
    /// ⚠️ Ids **distintos** que o produtor cunhou — é a causa, e é o que o produto controla.
    distinct_ids: usize,
    /// ⛔⛔ **Ids que foram DESENHADOS e nunca chegaram ao atlas** — a imagem que não aparece na
    /// tela, sem erro nenhum.
    ///
    /// A detecção é exacta e não precisa de espreitar o interior do `Resolver`: a primeira vez que
    /// um id consegue lugar, o `ImageCache` empurra-o para a lista de envios (`Entry::Vacant`).
    /// ⇒ um id desenhado que **nunca** aparece em envio nenhum é um id que nunca teve lugar.
    never_uploaded: usize,
}

/// Corre `frames` quadros com `viewports` imagens de `w × h` cada, e devolve o que o atlas fez.
///
/// ⚠️ **Um `Resolver` só, fora do laço** — é o que o `Renderer` faz, e é o que torna o atlas
/// *persistente*. Um `Resolver` por quadro mediria a 0.8.
fn run(mode: Mode, frames: usize, viewports: usize, w: u32, h: u32) -> Report {
    run_with_neighbour(mode, frames, viewports, w, h, None)
}

/// Como [`run`], mas com um **segundo produtor** a desenhar `(w, h)` no mesmo quadro — o vizinho
/// que compete pelo mesmo atlas. É ele que responde se o padrão do módulo 3D **rouba a tela** de
/// outra ferramenta, ou se apenas gasta largura de banda.
///
/// ⚠️ O vizinho usa sempre o [`StableImage`]: ele é o **bom vizinho**, o que já faz a coisa certa.
/// Se até ele desaparece, a acusação é do produtor barulhento e não da vítima.
fn run_with_neighbour(
    mode: Mode,
    frames: usize,
    viewports: usize,
    w: u32,
    h: u32,
    vizinho: Option<(u32, u32)>,
) -> Report {
    let mut resolver = vello_encoding::Resolver::new();
    let mut packed: Vec<u8> = Vec::new();
    let mut report = Report::default();
    let mut ids: std::collections::BTreeSet<u64> = std::collections::BTreeSet::new();
    let mut uploaded: std::collections::BTreeSet<u64> = std::collections::BTreeSet::new();
    let vizinho = vizinho.map(|(vw, vh)| {
        StableImage::from_rgba(rgba(vw, vh, 0xAB), vw, vh).expect("dimensoes batem")
    });

    // ⭐ No modo estável a imagem é construída UMA vez por *traçado*, e este arnês encena o caso
    // extremo — e comum — de **nenhum traçado novo**: a peça parada, a mesma imagem redesenhada
    // para sempre. É onde a porta crua desperdiça 100% do que gasta.
    let stable: Vec<StableImage> = (0..viewports)
        .map(|v| StableImage::from_rgba(rgba(w, h, v as u8), w, h).expect("dimensoes batem"))
        .collect();

    for frame in 0..frames {
        let mut scene = VectorScene::new();
        for (v, handle) in stable.iter().enumerate() {
            let dest = (0.0, 0.0, f64::from(w), f64::from(h));
            match mode {
                Mode::Raw => {
                    // O byte muda por quadro **e** por vista: um `Arc` novo, como no produto.
                    let seed = (frame * viewports + v) as u8;
                    scene.draw_image_rgba_premultiplied_transformed(
                        &rgba(w, h, seed),
                        w,
                        h,
                        Affine::translate((dest.0, dest.1)),
                        ImageQuality::Medium,
                    );
                }
                Mode::Stable => {
                    scene.draw_stable_image(handle, dest, ImageQuality::Medium);
                }
            }
        }

        // ⚠️ O vizinho desenha **depois**, como um painel que se pinta por cima do canvas: é a
        // ordem que o app usa, e a que dá ao produtor barulhento a primeira escolha de lugar.
        if let Some(v) = &vizinho {
            scene.draw_stable_image(
                v,
                (0.0, 0.0, f64::from(v.width()), f64::from(v.height())),
                ImageQuality::Medium,
            );
        }

        for patch in &scene.inner().encoding().resources.patches {
            if let vello_encoding::Patch::Image { image, .. } = patch {
                ids.insert(image.data.id());
            }
        }

        let (_layout, _ramps, images) = resolver.resolve(scene.inner().encoding(), &mut packed);
        report.atlas_side = images.width;
        report.uploads += images.images.len();
        report.evictions += images.evicted;
        for (img, _, _) in images.images {
            uploaded.insert(img.data.id());
            report.upload_bytes += u64::from(img.width) * u64::from(img.height) * 4;
        }
    }

    report.distinct_ids = ids.len();
    report.never_uploaded = ids.difference(&uploaded).count();
    report
}

/// ⭐ **A SONDA** — imprime a tabela que responde *«o que o módulo 3D custa ao atlas?»*.
///
/// Não afirma nada: é uma medição (`CLAUDE.md` §5.0 — 64% dos `#[ignore]` deste repo são sondas,
/// e tratá-las como portões produz mil falsos alarmes).
///
/// ```text
/// cargo test -p ph2d-vector --lib -- --ignored measure_the_image_atlas
/// ```
#[test]
#[ignore = "sonda: imprime a tabela do atlas, nao afirma"]
fn measure_the_image_atlas_under_the_viewport_pattern() {
    // Os tamanhos saem do produto: o traçado do módulo 3D sai **no tamanho real da área**
    // (`field3d_smoke_draw::viewport_pass`), e a divisão em quatro dá quartos dela.
    let casos: [(&str, usize, u32, u32); 4] = [
        ("1 vista, 1920x1080", 1, 1920, 1080),
        ("4 vistas, 960x540", 4, 960, 540),
        ("1 vista, 2560x1440", 1, 2560, 1440),
        ("4 vistas, 1280x720", 4, 1280, 720),
    ];
    const FRAMES: usize = 60;

    println!(
        "\n{:<22} {:>6} {:>7} {:>9} {:>9} {:>9} {:>9}",
        "caso", "modo", "atlas", "ids", "envios", "MB", "despejos"
    );
    for (nome, vps, w, h) in casos {
        for modo in [Mode::Raw, Mode::Stable] {
            let r = run(modo, FRAMES, vps, w, h);
            println!(
                "{:<22} {:>6} {:>9} {:>9} {:>9} {:>9.1} {:>9}",
                nome,
                match modo {
                    Mode::Raw => "cru",
                    Mode::Stable => "estavel",
                },
                r.atlas_side,
                r.distinct_ids,
                r.uploads,
                r.upload_bytes as f64 / (1024.0 * 1024.0),
                r.evictions
            );
        }
    }
    println!("\n({FRAMES} quadros por caso — cerca de um segundo de tela)\n");
}

/// ⛔⛔⛔ **A SONDA QUE DECIDE SE ISTO É DESPERDÍCIO OU DEFEITO** — um segundo produtor, bem
/// comportado, a competir pelo mesmo atlas.
///
/// A pergunta é a que o registo da subida deixou aberta: *«quando não cabe, a imagem não é
/// desenhada, em silêncio»* — mas **cabe?** Só uma corrida com vizinho responde.
///
/// ⚠️ **Que número a resposta CONTRÁRIA imprimiria?** `nao coube = 0` em todas as linhas: o atlas
/// tem `8192²` e o padrão do módulo, sozinho, não o esgota.
#[test]
#[ignore = "sonda: imprime se o vizinho perde a vez no atlas"]
fn measure_whether_a_neighbour_still_fits_beside_the_viewport_pattern() {
    const FRAMES: usize = 60;
    // O vizinho é uma pré-visualização de painel do tamanho de um dock — o `ph2d-panel-sculpt3d`
    // desenha exactamente isto, e é o candidato mais próximo a partilhar a tela com o MODEL.
    const VIZINHO: (u32, u32) = (512, 512);

    let casos: [(&str, usize, u32, u32); 4] = [
        ("1 vista, 1920x1080", 1, 1920, 1080),
        ("4 vistas, 960x540", 4, 960, 540),
        ("1 vista, 2560x1440", 1, 2560, 1440),
        ("1 vista, 3200x1800", 1, 3200, 1800),
    ];

    println!(
        "\n{:<22} {:>8} {:>7} {:>8} {:>10} {:>10}",
        "caso", "modo", "atlas", "despejos", "desenhados", "NAO COUBE"
    );
    for (nome, vps, w, h) in casos {
        for modo in [Mode::Raw, Mode::Stable] {
            let r = run_with_neighbour(modo, FRAMES, vps, w, h, Some(VIZINHO));
            println!(
                "{:<22} {:>8} {:>7} {:>8} {:>10} {:>10}",
                nome,
                match modo {
                    Mode::Raw => "cru",
                    Mode::Stable => "estavel",
                },
                r.atlas_side,
                r.evictions,
                r.distinct_ids,
                r.never_uploaded
            );
        }
    }
    println!();

    // ⭐⭐ **A QUE DISTÂNCIA DA BORDA?** No pior caso medido acima o atlas já está no TECTO
    // (`8192`), e um tecto atingido não tem para onde crescer. Esta varredura sobe o vizinho até
    // alguém ficar de fora — é o número que separa *«desperdiça»* de *«desperdiça e está a um
    // passo de partir»*.
    println!(
        "{:<22} {:>8} {:>7} {:>10}",
        "vizinho (1 vista 2560x1440)", "modo", "atlas", "NAO COUBE"
    );
    for lado in [512u32, 1024, 2048, 3072, 4096] {
        for modo in [Mode::Raw, Mode::Stable] {
            let r = run_with_neighbour(modo, 30, 1, 2560, 1440, Some((lado, lado)));
            println!(
                "{:<22} {:>8} {:>7} {:>10}",
                format!("{lado}x{lado}"),
                match modo {
                    Mode::Raw => "cru",
                    Mode::Stable => "estavel",
                },
                r.atlas_side,
                r.never_uploaded
            );
        }
    }
    println!();
}

/// ⛔⛔ **O CONTROLE, e ele é a metade que dá sentido ao gate seguinte.**
///
/// A porta crua cunha um id **por chamada**. Se um dia ela deixar de o fazer (o upstream a mudar,
/// ou alguém a memoizar), este teste fica vermelho — e é o aviso de que o gate irmão passou a ser
/// verde **por acidente**, medindo uma diferença que já não existe.
#[test]
fn the_raw_image_port_mints_a_new_id_on_every_call() {
    const FRAMES: usize = 8;
    let r = run(Mode::Raw, FRAMES, 1, 64, 64);
    assert_eq!(
        r.distinct_ids, FRAMES,
        "a porta crua devia cunhar um id por quadro; cunhou {} em {FRAMES}",
        r.distinct_ids
    );
    assert_eq!(
        r.uploads, FRAMES,
        "cada id novo e' uma residente nova, logo um envio; houve {} envios",
        r.uploads
    );
}

/// ⭐⭐⭐ **A LEI:** redesenhar a MESMA imagem custa **um** envio, não um por quadro.
///
/// ⚠️ Esta é a propriedade que o app perdeu ao subir para a 0.10 sem mudar uma linha — não porque
/// algo se partiu, mas porque *uma afirmação verdadeira por construção deixou de o ser*.
#[test]
fn redrawing_a_stable_image_uploads_it_once_not_once_per_frame() {
    const FRAMES: usize = 60;
    const VISTAS: usize = 4;
    let r = run(Mode::Stable, FRAMES, VISTAS, 640, 360);

    assert_eq!(
        r.distinct_ids, VISTAS,
        "quatro vistas paradas deviam ter QUATRO ids no total, e nao um por quadro"
    );
    assert_eq!(
        r.uploads, VISTAS,
        "a mesma imagem foi enviada {} vezes em {FRAMES} quadros — o cache do atlas nao acertou",
        r.uploads
    );
    assert_eq!(
        r.evictions, 0,
        "nada devia ser despejado: {VISTAS} imagens residentes e o atlas nao encheu"
    );
}

/// ⛔⛔ **O TIPO DE ALFA VIAJA DENTRO DO HANDLE, e enganar-se nele não dá erro nenhum.**
///
/// [`StableImage::from_rgba`] carimba `Alpha`; [`StableImage::from_rgba_premultiplied`] carimba
/// `AlphaPremultiplied`. Os dois aceitam os MESMOS bytes e compilam igual — o que muda é o Vello
/// multiplicar (ou não) antes de amostrar. ⇒ o único sítio onde a diferença é observável sem GPU é
/// o **encoding**, e é aí que este gate olha.
///
/// ⚠️ **Sem ele, escrever `Alpha` no construtor pré-multiplicado é um defeito silencioso** cujo
/// sintoma é a borda da peça a escurecer — exactamente o artefacto que a
/// `draw_image_rgba_premultiplied_transformed` nasceu para curar (Enio, 2026-05-26: *"linha clara
/// contornando a forma"*).
#[test]
fn the_two_constructors_do_not_encode_the_same_alpha() {
    let px = rgba(4, 4, 0x40);
    let reto = StableImage::from_rgba(px.clone(), 4, 4).expect("dimensoes batem");
    let premul = StableImage::from_rgba_premultiplied(px, 4, 4).expect("dimensoes batem");

    let desenha = |img: &StableImage| {
        let mut s = VectorScene::new();
        s.draw_stable_image(img, (0.0, 0.0, 4.0, 4.0), ImageQuality::Medium);
        s.inner().encoding().draw_data.clone()
    };

    assert_ne!(
        desenha(&reto),
        desenha(&premul),
        "os dois construtores encodaram o MESMO tipo de alfa — um deles esta' a mentir sobre os \
         bytes que recebeu, e o Vello vai pre-multiplicar duas vezes"
    );
}

/// ⭐⭐⭐ **A CURA DESENHA O MESMO QUE A PORTA QUE SUBSTITUIU — e este é o gate que responde
/// *«partiste a imagem?»*.**
///
/// Os dois gates acima medem o **custo**. Nenhum deles diz que o pixel é o mesmo, e é exactamente
/// isso que um artista repara primeiro. ⭐ **O oráculo existe e é o código de ontem:**
/// `draw_image_rgba_premultiplied_transformed` com o afim escrito à mão. Se o encoding bater, a
/// troca é invisível na tela por construção — não por promessa.
///
/// ⚠️ **Compara TRÊS fluxos, não um.** O `draw_data` carrega a qualidade e o tipo de alfa; os
/// `transforms` carregam o enquadramento (é onde um `dest` mal composto apareceria); os `styles`
/// carregam o modo de preenchimento. Olhar só para um deixaria a colocação por medir — e a
/// colocação é a metade que este módulo mudou (o afim explícito virou um rectângulo).
#[test]
fn the_stable_draw_encodes_exactly_what_the_raw_premultiplied_port_did() {
    const W: u32 = 8;
    const H: u32 = 5;
    // Um rectângulo de destino deslocado e com escala NÃO uniforme — se fosse 1:1 na origem, um
    // `dest` trocado passaria despercebido.
    let (x, y, w, h) = (17.0_f64, 23.0_f64, 40.0_f64, 15.0_f64);
    let px = rgba(W, H, 0x7F);

    let mut antiga = VectorScene::new();
    antiga.draw_image_rgba_premultiplied_transformed(
        &px,
        W,
        H,
        Affine::translate((x, y)) * Affine::scale_non_uniform(w / f64::from(W), h / f64::from(H)),
        ImageQuality::Medium,
    );

    let handle = StableImage::from_rgba_premultiplied(px, W, H).expect("dimensoes batem");
    let mut nova = VectorScene::new();
    nova.draw_stable_image(&handle, (x, y, x + w, y + h), ImageQuality::Medium);

    let (a, b) = (antiga.inner().encoding(), nova.inner().encoding());
    assert_eq!(
        a.draw_data, b.draw_data,
        "o desenho estavel encodou draw_data diferente da porta crua — qualidade ou tipo de alfa \
         divergiram, e a borda da peca muda"
    );
    assert_eq!(
        a.transforms, b.transforms,
        "o `dest` em rectangulo nao compoe o mesmo afim que o translate x scale_non_uniform — a \
         imagem sai noutro sitio ou noutro tamanho"
    );
    assert_eq!(a.styles, b.styles, "o modo de preenchimento divergiu");
    // ⛔ O controlo: os dois fluxos não podem estar vazios, senão a igualdade é vácua.
    assert!(
        !a.draw_data.is_empty() && !a.transforms.is_empty(),
        "o oraculo nao encodou nada — a comparacao acima nao mediu coisa nenhuma"
    );
}

/// ⭐⭐ **O preço da porta crua, afirmado como DESIGUALDADE contra a estável.**
///
/// ⚠️ Um número absoluto envelheceria com a política de despejo do upstream. A razão não: enquanto
/// a porta crua cunhar um id por chamada, ela paga `frames ×` o que a estável paga `1 ×`.
///
/// *É a forma que a casa exige de um gate sobre um recurso de terceiros* — a lei é a
/// **desigualdade**, e o oráculo é a corrida gémea, não uma constante escrita à mão.
#[test]
fn the_raw_port_costs_a_whole_frame_of_uploads_where_the_stable_one_costs_none() {
    const FRAMES: usize = 30;
    const VISTAS: usize = 4;
    let cru = run(Mode::Raw, FRAMES, VISTAS, 640, 360);
    let estavel = run(Mode::Stable, FRAMES, VISTAS, 640, 360);

    assert!(
        cru.upload_bytes >= estavel.upload_bytes * 10,
        "a porta crua devia custar uma ordem de grandeza mais bytes de envio: \
         cru {} B contra estavel {} B",
        cru.upload_bytes,
        estavel.upload_bytes
    );
    assert!(
        cru.atlas_side >= estavel.atlas_side,
        "a porta crua nunca devia pedir um atlas MENOR que a estavel: \
         cru {} contra estavel {}",
        cru.atlas_side,
        estavel.atlas_side
    );
}
