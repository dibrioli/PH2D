//! **EXPORTAR UMA SPRITE** para qualquer um dos formatos que a engine suporta — plano
//! [`docs/Sprite_projeto/18`](../../../docs/Sprite_projeto/18_precisao_de_16_bits_nas_sprites.md) W9.
//!
//! > Enio, 2026-08-21: *"2) exportação nos vários formatos suportados."*
//!
//! # O que faltava, e não era o que parecia
//!
//! ⚠️ **Os dezasseis exportadores já existiam e já estavam registados** — PNG, JPEG, WebP, GIF,
//! APNG, TIFF, PSD, ORA, JXL, EXR, HDR-Radiance, AVIF, SVG, TGA, QOI e o nativo. O `init` monta a
//! [`ph2d_imageio::ExporterRegistry`] com todos eles desde a ADR-0054, e a anotação no `app_state`
//! dizia, palavra por palavra, *"W0.T6 stages the registry; W1+ wires Save into find_for"*.
//!
//! O que não existia era **a porta**. Nenhuma linha da shell chamava `find_for`. *Um exportador que
//! nenhum gesto alcança não é um formato suportado — é código a compilar.*
//!
//! # O formato vem da EXTENSÃO que o artista escreve
//!
//! ⛔ **Não há um dropdown de formato**, e é decisão: o diálogo do sistema já tem um sítio para isso
//! (o nome do ficheiro), e um segundo sítio criaria o estado em que os dois discordam — o artista
//! escreve `heroi.png`, o dropdown diz `JPEG`, e o ficheiro sai a mentir sobre si próprio no nome.
//! A extensão é a fonte única.
//!
//! # A precisão: tenta-se o EXACTO primeiro, e só se cai quando o formato recusa
//!
//! ⚠️ **É aqui que a wave dos 16 bits fecha o círculo.** A importação preserva 16 bits desde a W2.4;
//! até hoje não havia por onde os fazer sair. Uma sprite `Rgba16Float` é oferecida ao exportador
//! como [`DecodedImage::FlatHdr`](ph2d_imageio::DecodedImage) — `f32` linear, sem passar por 8 bits.
//!
//! ⛔ **E NÃO há uma lista de «formatos HDR» escrita aqui.** Uma lista dessas nasce certa e
//! apodrece no dia em que um exportador ganha suporte de HDR. Em vez disso: **oferece-se o exacto, e
//! se ele devolver [`Error::HdrUnsupported`](ph2d_imageio::Error) oferece-se o de 8 bits** — e aí
//! avisa-se, porque um ficheiro que perde precisão em silêncio é o defeito que esta linha inteira
//! existe para não repetir (`docs/Sprite_projeto/19` §5). *Quem sabe se um formato aguenta HDR é o
//! formato.*

use std::path::{Path, PathBuf};

use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::{Entity, SimWorld};
use ph2d_editor::{Toast, ToastQueue};
use ph2d_imageio::{DecodedImage, ExportFormat, ExportOpts, ExporterRegistry, ImageBuffer};
use ph2d_render::SpriteRenderer;
use std::collections::BTreeMap;

/// Todos os formatos que a engine sabe escrever, na ordem em que aparecem no diálogo.
///
/// ⚠️ **A ordem é de USO, não alfabética:** primeiro os que um artista de sprites quer (PNG à
/// cabeça, que é o default), depois os de alta precisão, depois os de nicho. Um diálogo que abre no
/// formato errado é um clique extra em todas as exportações.
///
/// ⚠️ **A lista inclui formatos que vão RECUSAR uma sprite** — o SVG é vectorial, o PSD e o ORA são
/// de camadas. Eles ficam porque a recusa vem **do exportador, com a razão dele**; omiti-los aqui
/// seria esta lista a decidir por eles, e a decisão apodreceria no dia em que um deles aprendesse a
/// achatar. *A pergunta «este formato aguenta isto?» tem um dono, e não é este ficheiro.*
const OFFERED: [ExportFormat; 16] = [
    ExportFormat::Png,
    ExportFormat::Webp,
    ExportFormat::Jpeg,
    ExportFormat::Tga,
    ExportFormat::Qoi,
    ExportFormat::Tiff,
    ExportFormat::Gif,
    ExportFormat::Apng,
    // Alta precisão — os que podem receber os 16 bits sem os rebaixar.
    ExportFormat::Exr,
    ExportFormat::HdrRadiance,
    ExportFormat::Jxl,
    ExportFormat::Avif,
    // De camadas e de nicho.
    ExportFormat::Psd,
    ExportFormat::Ora,
    ExportFormat::Svg,
    ExportFormat::Ph2dNative,
];

/// O formato que a extensão do caminho escolhido nomeia.
///
/// ⚠️ **`jpg` é o mesmo que `jpeg`, e `htm`-style aliases param aqui.** O
/// [`ExportFormat::extension`] devolve **a** forma canónica; o mundo escreve as duas, e recusar
/// `heroi.jpg` porque a canónica é `jpeg` seria o app a corrigir o artista sobre o nome do próprio
/// ficheiro dele.
pub(crate) fn format_for_path(path: &Path) -> Option<ExportFormat> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    let ext = match ext.as_str() {
        "jpg" => "jpeg",
        "tif" => "tiff",
        other => other,
    };
    OFFERED.into_iter().find(|f| f.extension() == ext)
}

/// Os pixels de uma sprite prontos a exportar, nas DUAS precisões que ela pode ter.
///
/// ⚠️ **Os 8 bits vêm sempre; os 16 só quando existem.** Não é redundância: o caminho de alta
/// precisão pode ser recusado pelo formato, e aí o de 8 bits tem de estar já na mão — relê-lo
/// significaria uma segunda ida à GPU no meio de um `match`.
struct SpriteSource {
    /// A imagem de 8 bits, em alfa RETO.
    flat: ImageBuffer<ph2d_color::SrgbRgba>,
    /// Os meios-floats, quando a textura é de 16 bits.
    halves: Option<Vec<u16>>,
    width: u32,
    height: u32,
}

/// Os pixels da sprite, na melhor precisão que ela tem.
///
/// PRECISION-READONLY: exporta — LÊ os pixels e escreve um FICHEIRO; nunca escreve de volta na
/// sprite. A precisão que se perca perde-se no ficheiro (e é dita ao artista), nunca na cena.
fn source_for(
    entity: Entity,
    sim: &SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    atlas_asset_map: &BTreeMap<u32, AssetId>,
) -> Option<SpriteSource> {
    let src = crate::hero_intents::texture_edit::read_sprite_source(
        entity,
        sim,
        renderer,
        asset_db,
        atlas_asset_map,
    )?;
    // ⚠️ **Alfa RETO.** Um ficheiro não amostra nada, e todo programa que o abrir assume reto —
    // gravar pré-multiplicado daria a imagem escura no Aseprite, que é o defeito que o
    // `sheet_export` já pagou uma vez.
    let straight = src.image.into_straight();
    let (w, h) = (straight.width, straight.height);
    let flat = ImageBuffer {
        width: w,
        height: h,
        pixels: bytemuck::allocation::cast_vec(straight.pixels),
        color_profile: ph2d_imageio::ColorProfile::Srgb,
    };
    Some(SpriteSource {
        flat,
        halves: src.pixels_16,
        width: w,
        height: h,
    })
}

/// Constrói a oferta de alta precisão a partir dos meios-floats.
///
/// ⚠️ **Os halves são LINEARES** (é o que o `Rgba16Float` guarda), e o `LinearRgba` também — por isso
/// não há curva nenhuma a atravessar aqui. Aplicar uma seria o erro clássico, e escureceria o
/// ficheiro inteiro.
fn hdr_offer(halves: &[u16], width: u32, height: u32) -> DecodedImage {
    let pixels = halves
        .chunks_exact(4)
        .map(|px| {
            ph2d_color::LinearRgba::new(
                ph2d_color::half_to_f32(px[0]),
                ph2d_color::half_to_f32(px[1]),
                ph2d_color::half_to_f32(px[2]),
                ph2d_color::half_to_f32(px[3]),
            )
        })
        .collect();
    DecodedImage::FlatHdr(ImageBuffer {
        width,
        height,
        pixels,
        // ⚠️ **`LinearRec709`, não `Srgb`.** Os primários são os mesmos do sRGB; o que muda é a
        // ausência da curva — e é isso que estes valores são. Declarar `Srgb` num buffer linear
        // diria ao exportador para aplicar a transferência outra vez.
        color_profile: ph2d_imageio::ColorProfile::LinearRec709,
    })
}

/// **Exporta a sprite para `path`.** Devolve `Ok(bytes escritos)`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn export_to(
    entity: Entity,
    path: &Path,
    sim: &SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    atlas_asset_map: &BTreeMap<u32, AssetId>,
    exporters: &ExporterRegistry,
    toasts: &mut ToastQueue,
) -> Result<usize, String> {
    let Some(format) = format_for_path(path) else {
        return Err(format!(
            "unknown extension — try one of: {}",
            OFFERED
                .iter()
                .map(|f| f.extension())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    };
    let Some(src) = source_for(entity, sim, renderer, asset_db, atlas_asset_map) else {
        return Err("this sprite's pixels are unreadable".to_string());
    };
    let opts = ExportOpts {
        format,
        ..Default::default()
    };
    let Some(exporter) = exporters.find_for(&opts) else {
        return Err(format!("no exporter for .{}", format.extension()));
    };

    // ⚠️ **A ALTA PRECISÃO PRIMEIRO, e a queda só quando o formato a recusa.** Ver o cabeçalho: não
    // há lista de «formatos HDR» aqui de propósito — quem sabe se um formato aguenta HDR é ele.
    let bytes = if let Some(halves) = src.halves.as_ref() {
        match exporter.export(&hdr_offer(halves, src.width, src.height), &opts) {
            Ok(bytes) => bytes,
            Err(ph2d_imageio::Error::HdrUnsupported) => {
                // A perda é do FICHEIRO, não da sprite — e diz-se, como todo verbo que custa
                // precisão neste projeto (`docs/Sprite_projeto/19` §5).
                toasts.push(Toast::info(format!(
                    "Exported as RGBA8 — .{} cannot carry 16-bit; try .exr or .hdr",
                    format.extension()
                )));
                exporter
                    .export(&DecodedImage::Flat(src.flat), &opts)
                    .map_err(|e| e.to_string())?
            }
            Err(e) => return Err(e.to_string()),
        }
    } else {
        exporter
            .export(&DecodedImage::Flat(src.flat), &opts)
            .map_err(|e| e.to_string())?
    };

    std::fs::write(path, &bytes).map_err(|e| format!("could not write {}: {e}", path.display()))?;
    Ok(bytes.len())
}

/// O nome de ficheiro sugerido, a partir do nome da entidade.
///
/// ⚠️ Mesma higiene do [`crate::sheet_export::safe_stem`] e pela mesma razão: o artista escreve
/// `Herói / v2`, e uma barra abriria um directório que não existe.
pub(crate) fn suggested_name(sim: &SimWorld, entity: Entity) -> String {
    let base = sim
        .world()
        .get::<ph2d_ecs::Name>(entity)
        .map(|n| n.as_str().to_string())
        .unwrap_or_else(|| "sprite".to_string());
    format!("{}.png", crate::sheet_export::safe_stem(&base))
}

/// Abre o diálogo do sistema e exporta. `None` se o artista cancelou.
#[allow(clippy::too_many_arguments)]
pub(crate) fn export_with_dialog(
    entity: Entity,
    sim: &SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    atlas_asset_map: &BTreeMap<u32, AssetId>,
    exporters: &ExporterRegistry,
    toasts: &mut ToastQueue,
) -> Option<PathBuf> {
    let mut dialog = rfd::FileDialog::new().set_file_name(suggested_name(sim, entity));
    // Um filtro por formato: é o que faz o diálogo do sistema mostrar a lista e trocar a extensão.
    for f in OFFERED {
        dialog = dialog.add_filter(f.extension().to_ascii_uppercase(), &[f.extension()]);
    }
    let picked = dialog.save_file()?;
    match export_to(
        entity,
        &picked,
        sim,
        renderer,
        asset_db,
        atlas_asset_map,
        exporters,
        toasts,
    ) {
        Ok(n) => {
            // ⚠️ O caminho COMPLETO, e não «exportado com sucesso» — foi por não o dizer que o Enio
            // perguntou *"não sei onde foi parar"* do `sheet_export`.
            toasts.push(Toast::success(format!(
                "Exported: {} ({n} bytes)",
                picked.display()
            )));
            Some(picked)
        }
        Err(e) => {
            toasts.push(Toast::error(format!("Export failed: {e}")));
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Toda extensão oferecida resolve de volta para o seu formato.**
    ///
    /// ⚠️ É o par que impede a lista e o diálogo divergirem: o `add_filter` usa
    /// `format.extension()`, e o `format_for_path` tem de reconhecer exactamente o que ele escreveu.
    /// Um formato que o diálogo oferece e o resolvedor não conhece é um «unknown extension» sobre um
    /// nome que o próprio app sugeriu.
    #[test]
    fn every_offered_extension_resolves_back() {
        for f in OFFERED {
            let p = PathBuf::from(format!("x.{}", f.extension()));
            assert_eq!(
                format_for_path(&p),
                Some(f),
                "a extensao `.{}` que o dialogo oferece nao volta a resolver para o formato",
                f.extension()
            );
        }
    }

    /// ⚠️ **Os apelidos que o mundo escreve.** `heroi.jpg` é JPEG — recusá-lo porque a forma
    /// canónica é `jpeg` seria o app a corrigir o artista sobre o nome do ficheiro dele.
    #[test]
    fn the_common_aliases_resolve() {
        assert_eq!(
            format_for_path(Path::new("a.jpg")),
            Some(ExportFormat::Jpeg)
        );
        assert_eq!(
            format_for_path(Path::new("a.tif")),
            Some(ExportFormat::Tiff)
        );
    }

    /// **Maiúsculas resolvem** — o Windows escreve `.PNG` sem pedir licença.
    #[test]
    fn the_extension_is_case_insensitive() {
        assert_eq!(format_for_path(Path::new("A.PNG")), Some(ExportFormat::Png));
    }

    /// **Sem extensão, ou com uma que ninguém escreve, não se adivinha.**
    ///
    /// ⚠️ ⛔ Cair para PNG aqui seria pior que recusar: o artista escreveu `heroi.bmp`, receberia um
    /// PNG com nome de BMP, e descobriria noutro programa.
    #[test]
    fn an_unknown_extension_is_refused_and_not_guessed() {
        assert_eq!(format_for_path(Path::new("a.bmp")), None);
        assert_eq!(format_for_path(Path::new("a")), None);
    }

    /// **O nome sugerido é sempre gravável**, mesmo vindo de um nome com barras.
    #[test]
    fn the_suggested_name_survives_a_hostile_entity_name() {
        let mut sim = SimWorld::default();
        let e = sim
            .world_mut()
            .spawn((ph2d_ecs::Name::new("Herói / v2"),))
            .id();
        let name = suggested_name(&sim, e);
        assert!(
            !name.contains('/') && name.ends_with(".png"),
            "o nome sugerido `{name}` abriria um directorio que nao existe"
        );
    }

    /// ⚠️ **Os meios-floats atravessam SEM curva.** Aplicar sRGB aqui escureceria o ficheiro inteiro
    /// — é o erro clássico, e o único vestígio seria a imagem parecer «mais contrastada».
    #[test]
    fn the_hdr_offer_carries_the_linear_values_verbatim() {
        let half = ph2d_color::f32_to_half(0.25);
        let halves = vec![half, half, half, ph2d_color::f32_to_half(1.0)];
        let DecodedImage::FlatHdr(buf) = hdr_offer(&halves, 1, 1) else {
            panic!("o hdr_offer devia dar FlatHdr");
        };
        assert_eq!(buf.width, 1);
        assert!(
            (buf.pixels[0].r() - 0.25).abs() < 1e-3,
            "o valor linear saiu {} e devia ser 0.25 — alguem aplicou uma curva",
            buf.pixels[0].r()
        );
    }

    /// ⚠️ **A PROVA de que o círculo fecha: 16 bits saem para disco e voltam.**
    ///
    /// Os testes acima medem as peças; este mede a **cadeia**, contra os exportadores e
    /// importadores DE VERDADE que o `init` regista. É a afirmação central desta wave — *"a
    /// importação preservava e não havia por onde sair"* — e sem ele ela era uma promessa.
    ///
    /// ⚠️ Um valor **acima de 1.0** é o que separa «alta precisão» de «mais bits»: é ele que um
    /// formato LDR não sabe guardar, e é ele que faz uma sprite ser fonte de luz (W8).
    ///
    /// # A TABELA, medida em 2026-08-21, e por que ela é impressa em vez de escrita
    ///
    /// A sonda corre **todos** os 16 e imprime o que cada um respondeu. Escrever a lista num
    /// comentário seria uma verdade com prazo de validade; imprimi-la faz com que quem correr o
    /// teste veja o estado do dia.
    ///
    /// ```text
    ///   hdr  OK      avif OK      ph2d OK        <- carregam 16 bits hoje
    ///   exr  --  encode deferred "to first real export client"     ⚠️ ESSE CLIENTE CHEGOU
    ///   jxl  --  `jxl-oxide` e' decode-only
    ///   psd  --  export deferred (ADR-0054 §5.2)
    ///   svg  --  precisa dos tipos canonicos do ph2d-vector
    ///   png/webp/jpeg/tga/qoi/tiff/gif/apng/ora  --  LDR: recusam HDR sem tone-map
    /// ```
    ///
    /// ⚠️ **O `exr` diz, palavra por palavra, que o encode ficou à espera do «first real export
    /// client».** Esta wave É esse cliente — a nota disparou. ⛔ Não se implementa aqui: o `.hdr` e
    /// o `.avif` já entregam a capacidade que o Enio pediu, e escrever o encoder do `exr` (a API de
    /// canais tipados da crate 1.x) é trabalho da linha do `imageio`, não desta. O que esta wave
    /// deve é **dizer que o gatilho disparou**, e é o que este doc-comment faz.
    #[test]
    fn a_sixteen_bit_sprite_survives_a_real_round_trip_through_hdr() {
        let mut exporters = ph2d_imageio::ExporterRegistry::new();
        ph2d_imageio_registry_init::register_all_exporters(&mut exporters);
        let mut importers = ph2d_imageio::ImporterRegistry::new();
        ph2d_imageio_registry_init::register_all_importers(&mut importers);

        // Um pixel bem acima do branco — o que um PNG perderia.
        let hot = ph2d_color::f32_to_half(4.0);
        let halves = vec![hot, hot, hot, ph2d_color::f32_to_half(1.0)];
        let offer = hdr_offer(&halves, 1, 1);

        // A tabela do dia, impressa (ver o doc-comment).
        let mut carriers = Vec::new();
        for f in OFFERED {
            let opts = ExportOpts {
                format: f,
                ..Default::default()
            };
            if let Some(e) = exporters.find_for(&opts) {
                match e.export(&offer, &opts) {
                    Ok(b) => {
                        eprintln!("  {:>5} OK  ({} bytes)", f.extension(), b.len());
                        carriers.push(f);
                    }
                    Err(err) => eprintln!("  {:>5} --  {err}", f.extension()),
                }
            }
        }
        assert!(
            !carriers.is_empty(),
            "NENHUM formato aceita um FlatHdr — a exportacao de 16 bits nao existe, e o menu \
             `Export Image` esta' a prometer o que nao entrega"
        );

        // E a ida-e-volta de facto, pelo Radiance HDR (o interchange canonico de alta precisao).
        let opts = ExportOpts {
            format: ExportFormat::HdrRadiance,
            ..Default::default()
        };
        let exporter = exporters.find_for(&opts).expect("o .hdr esta' registado");
        let bytes = exporter
            .export(&offer, &opts)
            .expect("o Radiance HDR aceita FlatHdr — medido em 2026-08-21");

        // ⚠️ Despacha pelos BYTES MÁGICOS, não pela extensão: não há ficheiro nenhum aqui, e os
        // bytes são a autoridade (a extensão é `Weak` por convenção deste registry).
        let importer = importers
            .find_for(ph2d_imageio::MagicHint::Bytes(&bytes))
            .expect("o que o nosso exportador escreve, o nosso importador tem de reconhecer");
        let back = importer
            .import(&bytes, &ph2d_imageio::ImportOpts::default())
            .expect("o .hdr que acabamos de escrever tem de importar");
        let DecodedImage::FlatHdr(buf) = back else {
            panic!("um .hdr tem de voltar como FlatHdr, e nao rebaixado");
        };
        // ⚠️ A barra é 1% e vem do FORMATO, não de gosto: o Radiance é RGBE — mantissa de 8 bits
        // com expoente partilhado —, logo ele guarda a ORDEM DE GRANDEZA exacta e a mantissa a
        // ~0,4%. Exigir igualdade binária seria exigir do RGBE o que ele não é.
        assert!(
            (buf.pixels[0].r() - 4.0).abs() < 0.04,
            "o valor acima do branco voltou {} em vez de ~4.0 — o caminho de alta precisao \
             perde-o em algum lado, e com ele a unica razao de exportar HDR",
            buf.pixels[0].r()
        );
    }

    /// **Um valor ACIMA de 1.0 sobrevive** — é a razão de o caminho HDR existir.
    #[test]
    fn a_value_above_one_survives_the_hdr_offer() {
        let hot = ph2d_color::f32_to_half(4.0);
        let halves = vec![hot, hot, hot, ph2d_color::f32_to_half(1.0)];
        let DecodedImage::FlatHdr(buf) = hdr_offer(&halves, 1, 1) else {
            panic!("FlatHdr");
        };
        assert!(
            buf.pixels[0].r() > 1.0,
            "o valor acima do branco foi cortado ({}), e com ele a unica razao de exportar HDR",
            buf.pixels[0].r()
        );
    }
}
