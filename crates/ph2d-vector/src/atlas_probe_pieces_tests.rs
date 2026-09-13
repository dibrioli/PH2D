//! ⭐⭐⭐ **UMA IMAGEM EM N PEÇAS NO MESMO QUADRO** — a sonda que nomeia o recurso do `Smooth` da
//! pele de imagem (esqueleto, F6-d) e os dois gates que ficam.
//!
//! # O report que a pede (dono, 2026-09-10)
//!
//! *«Smooth bugado quebrando a forma»* — o braço quase recto, a silhueta com **degraus** e a ponta
//! **comida**, com a malha provadamente certa (área ao cêntimo, zero peças saltadas). A nota daquela
//! wave atribuiu o limite à **camada de recorte** do Vello e escreveu *«não há como o medir sem
//! ecrã»*.
//!
//! # ⛔⛔ O que esta sonda mede em vez disso
//!
//! A pele desenha **um recorte mais um afim por triângulo**, e o afim desenhava a imagem pela porta
//! CRUA ([`VectorScene::draw_image_rgba_transformed`]), que cunha uma `Blob` — logo um **id** —
//! **por chamada**. O atlas da `vello` 0.10 é por id (`atlas_probe_tests`): N peças são **N cópias
//! inteiras** da imagem, o atlas pára em `8192²`, e o que não cabe **não é desenhado, em silêncio**.
//!
//! ⚠️ **E é por quadro, com cauda:** um residente só é despejado depois de
//! `EVICT_AFTER_GENERATIONS = 2` passes sem uso, logo o atlas tem de guardar as cópias de **três**
//! quadros ao mesmo tempo.
//!
//! ⭐ **A decisão do atlas é da CPU** (`vello_encoding::Resolver`), determinística e sem adaptador —
//! não é um relógio e não é GPU, então nada disto entra na família de flakes do `CLAUDE.md` §5.0.

use super::*;
use std::collections::BTreeSet;
use std::sync::Arc;
use vello::kurbo::{Affine, BezPath};
use vello::peniko::ImageQuality;

/// Como a pele entrega cada peça ao Vello.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Porta {
    /// [`VectorScene::draw_image_rgba_transformed`] — o `Arc` é o MESMO, a `Blob` é nova.
    Crua,
    /// [`VectorScene::draw_stable_image_transformed`] — um [`StableImage`] clonado por peça.
    Estavel,
}

/// O que o atlas fez, lido no ÚLTIMO quadro (o regime, não o arranque).
#[derive(Debug, Default)]
struct Relato {
    /// Lado do atlas quadrado (px).
    atlas_lado: u32,
    /// Quantos desenhos de imagem a cena emitiu no último quadro — o **controlo**: tem de ser o
    /// número de peças, senão a sonda não mediu a pele.
    desenhos: usize,
    /// Ids **distintos** entre esses desenhos.
    ids: usize,
    /// ⛔⛔ Desenhos do último quadro cujo id **nunca teve lugar** no atlas — peças da imagem que
    /// não aparecem na tela, sem erro nenhum.
    sem_lugar: usize,
    /// MB enviados à GPU por quadro, em média, **depois** do primeiro.
    envio_mb_por_quadro: f64,
}

/// Corre `quadros` quadros com uma imagem `w × h` partida em `pecas` recortes.
///
/// ⚠️ **Um `Resolver` só, fora do laço** — é o que o `Renderer` faz, e é o que torna o atlas
/// persistente (a mesma honestidade do arnês irmão).
fn corre(porta: Porta, quadros: usize, pecas: usize, w: u32, h: u32) -> Relato {
    let px: Arc<Vec<u8>> = Arc::new(vec![0x7F; (w as usize) * (h as usize) * 4]);
    let estavel = StableImage::from_rgba(Arc::clone(&px), w, h).expect("dimensoes batem");
    let mut resolver = vello_encoding::Resolver::new();
    let mut packed: Vec<u8> = Vec::new();
    let mut enviados: BTreeSet<u64> = BTreeSet::new();
    let mut relato = Relato::default();
    let mut bytes_depois_do_primeiro = 0_u64;

    for q in 0..quadros {
        let mut cena = VectorScene::new();
        for i in 0..pecas {
            // A geometria do recorte não entra na conta do atlas; a SEQUÊNCIA é a do produto —
            // recorte, imagem inteira com o afim da peça, fim do recorte.
            let x = f64::from(u32::try_from(i).unwrap_or(0) % w.max(1));
            let mut tri = BezPath::new();
            tri.move_to((x, 0.0));
            tri.line_to((x + 1.0, 0.0));
            tri.line_to((x, 1.0));
            tri.close_path();
            cena.push_clip(&tri);
            match porta {
                Porta::Crua => cena.draw_image_rgba_transformed(
                    &px,
                    w,
                    h,
                    Affine::IDENTITY,
                    ImageQuality::Medium,
                ),
                Porta::Estavel => cena.draw_stable_image_transformed(
                    &estavel,
                    Affine::IDENTITY,
                    ImageQuality::Medium,
                ),
            }
            cena.pop_layer();
        }
        let emitidos = cena.probe_image_ids();
        let (_layout, _ramps, imagens) = resolver.resolve(cena.inner().encoding(), &mut packed);
        let lado = imagens.width;
        let mut bytes = 0_u64;
        for (img, _, _) in imagens.images {
            enviados.insert(img.data.id());
            bytes += u64::from(img.width) * u64::from(img.height) * 4;
        }
        if q > 0 {
            bytes_depois_do_primeiro += bytes;
        }
        if q + 1 == quadros {
            relato.atlas_lado = lado;
            relato.desenhos = emitidos.len();
            relato.ids = emitidos.iter().copied().collect::<BTreeSet<_>>().len();
            relato.sem_lugar = emitidos.iter().filter(|id| !enviados.contains(id)).count();
        }
    }
    #[expect(
        clippy::cast_precision_loss,
        reason = "megabytes para uma tabela de sonda"
    )]
    {
        relato.envio_mb_por_quadro = bytes_depois_do_primeiro as f64
            / (1024.0 * 1024.0)
            / (quadros.saturating_sub(1).max(1) as f64);
    }
    relato
}

/// ⭐ **A SONDA** — a tabela que responde *«quantas peças cabem, e de que recurso é o tecto?»*.
///
/// ```text
/// cargo test -p ph2d-vector --lib -- --ignored --nocapture a_skinned_image_split_into_pieces
/// ```
#[test]
#[ignore = "sonda: imprime a tabela do atlas da pele de imagem, nao afirma"]
fn measure_a_skinned_image_split_into_pieces() {
    // `320 × 96` é a imagem do smoke do osso (`ph2d-app-vec::smoke_bone`, `IMG_W`/`IMG_H`); `216`
    // é a malha guardada dela, `864` o que o orçamento de omissão (`1024`) dá a `k = 2`, `7 776`
    // o que o report correu. As duas imagens grandes são o `Fast` com arte de tamanho comum.
    let casos: [(u32, u32, usize); 9] = [
        (320, 96, 216),
        (320, 96, 864),
        (320, 96, 1_024),
        (320, 96, 2_048),
        (320, 96, 7_776),
        (512, 512, 216),
        (1_024, 1_024, 16),
        (1_024, 1_024, 64),
        (1_024, 1_024, 216),
    ];
    const QUADROS: usize = 4;
    println!(
        "\n{:>11} {:>6} {:>8} {:>6} {:>8} {:>9} {:>11}",
        "imagem", "pecas", "porta", "atlas", "ids", "SEM LUGAR", "MB/quadro"
    );
    for (w, h, pecas) in casos {
        for porta in [Porta::Crua, Porta::Estavel] {
            let r = corre(porta, QUADROS, pecas, w, h);
            assert_eq!(r.desenhos, pecas, "a sonda nao emitiu uma imagem por peca");
            println!(
                "{:>11} {:>6} {:>8} {:>6} {:>8} {:>9} {:>11.1}",
                format!("{w}x{h}"),
                pecas,
                match porta {
                    Porta::Crua => "crua",
                    Porta::Estavel => "estavel",
                },
                r.atlas_lado,
                r.ids,
                r.sem_lugar,
                r.envio_mb_por_quadro
            );
        }
    }
    println!("\n({QUADROS} quadros por caso; SEM LUGAR = pecas do ultimo quadro que nao aparecem)\n");
}

/// Quanta informação por desenho o `Resolver` conta para uma imagem estável em `pecas` recortes —
/// o `layout.bin_data_start`, que é o que o Vello subtrai do buffer FIXO `bin_data`.
fn palavras_de_informacao(pecas: usize, w: u32, h: u32) -> u32 {
    let estavel = StableImage::from_rgba(
        Arc::new(vec![0x7F; (w as usize) * (h as usize) * 4]),
        w,
        h,
    )
    .expect("dimensoes batem");
    let mut cena = VectorScene::new();
    for i in 0..pecas {
        let x = f64::from(u32::try_from(i).unwrap_or(0) % w.max(1));
        let mut tri = BezPath::new();
        tri.move_to((x, 0.0));
        tri.line_to((x + 1.0, 0.0));
        tri.line_to((x, 1.0));
        tri.close_path();
        cena.push_clip(&tri);
        cena.draw_stable_image_transformed(&estavel, Affine::IDENTITY, ImageQuality::Medium);
        cena.pop_layer();
    }
    let mut packed = Vec::new();
    let (layout, _ramps, _imagens) =
        vello_encoding::Resolver::new().resolve(cena.inner().encoding(), &mut packed);
    layout.bin_data_start
}

/// ⭐⭐ **A SONDA DO TECTO DURO** — quantas palavras de informação cada peça gasta do buffer que o
/// Vello fixa em `1 << 18` (`vello_encoding::BufferSizes::new`: *«hand picked to accommodate the
/// vello test scenes as well as paris-30k»*).
///
/// ⚠️ **O buffer é do QUADRO, não da pele:** `binning_size = bin_data − layout.bin_data_start`
/// (`config.rs`), e o `bin_data_start` soma a informação de TODOS os desenhos da cena — painéis,
/// texto, arte. Passar dele dá a volta a um `u32`: pânico em debug, lixo em release.
#[test]
#[ignore = "sonda: imprime as palavras de informacao por peca e o tecto do Vello, nao afirma"]
fn measure_vello_bin_info_words_per_skin_piece() {
    const BIN_DATA: u32 = 1 << 18;
    println!("\n{:>8} {:>14} {:>12} {:>10}", "pecas", "bin_data_start", "por peca", "cabe?");
    for pecas in [1_usize, 216, 864, 7_776, 20_000, 21_600, 30_000] {
        let palavras = palavras_de_informacao(pecas, 320, 96);
        #[expect(clippy::cast_precision_loss, reason = "uma razao para uma tabela")]
        let por_peca = f64::from(palavras) / pecas as f64;
        println!(
            "{:>8} {:>14} {:>12.3} {:>10}",
            pecas,
            palavras,
            por_peca,
            if palavras < BIN_DATA { "sim" } else { "NAO" }
        );
    }
    println!("(bin_data = {BIN_DATA} palavras, partilhado pelo QUADRO inteiro)\n");
}

/// ⛔⛔ **O CONTROLO: a porta crua cunha um id POR PEÇA, mesmo com o `Arc` partilhado.**
///
/// É a metade que dá sentido ao gate seguinte — se um dia a porta crua deixar de cunhar (o
/// upstream a memoizar a `Blob`), a lei abaixo passa a ser verde **por acidente**.
#[test]
fn the_raw_port_mints_one_atlas_copy_per_piece_even_from_one_arc() {
    let r = corre(Porta::Crua, 1, 32, 16, 16);
    assert_eq!(
        (r.desenhos, r.ids),
        (32, 32),
        "a porta crua devia cunhar um id por peca a partir do MESMO `Arc`"
    );
}

/// ⭐⭐⭐ **A LEI: uma imagem em N peças ocupa UM lugar no atlas, e nenhuma peça fica de fora.**
///
/// ⚠️ **O caso é o do report, e o oráculo é a corrida gémea** (a forma que a casa exige de um gate
/// sobre um recurso de terceiros): a mesma imagem, as mesmas peças, pela porta crua, **tem** de
/// deixar peças sem lugar — senão o caso não estressa o atlas e a igualdade de cima é vácua.
#[test]
fn a_stable_image_split_into_pieces_takes_one_atlas_slot_and_loses_no_piece() {
    const QUADROS: usize = 4;
    const PECAS: usize = 7_776;
    let estavel = corre(Porta::Estavel, QUADROS, PECAS, 320, 96);
    let crua = corre(Porta::Crua, QUADROS, PECAS, 320, 96);
    assert!(
        crua.sem_lugar > 0,
        "a fixtura nao estressa o atlas: a porta crua coube inteira ({} ids) — a lei abaixo \
         ficaria verde sem medir nada",
        crua.ids
    );
    assert_eq!(
        (estavel.desenhos, estavel.ids, estavel.sem_lugar),
        (PECAS, 1, 0),
        "a imagem estavel em {PECAS} pecas tem de ser UM residente e desenhar todas as pecas"
    );
    assert!(
        estavel.envio_mb_por_quadro == 0.0,
        "a imagem estavel foi reenviada depois do primeiro quadro ({} MB/quadro)",
        estavel.envio_mb_por_quadro
    );
}
