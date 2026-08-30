//! **A ARTE DOS PADRÕES dentro do ficheiro de projecto** (plano 33, W4) — irmão do
//! [`crate::project_sprite_pixels`], e pela mesma razão que ele existe.
//!
//! # O que quebraria sem isto
//!
//! O documento guarda **qual** imagem um padrão usa (um `AssetId`), nunca os pixels. Reabrir o
//! projecto noutra sessão — ou noutra máquina — encontraria o `AssetDb` vazio, a fonte não
//! resolveria, e toda forma com padrão pintaria a `fallback` **para sempre e sem erro nenhum a que
//! agarrar**. É literalmente o defeito que o `project_sprite_pixels` curou para as sprites.
//!
//! # A identidade é o CONTEÚDO, e ela tem de SOBREVIVER
//!
//! ⚠️⚠️ **É por isto que a autoria usa `insert_image_rgba8` e não `insert_image_bytes`.** O
//! primeiro cunha `blake3(dims + RGBA)`; o segundo, `blake3(bytes do ficheiro)`. Aqui embutimos
//! **pixels** (o ficheiro do artista pode ter mudado de sítio, ou nunca mais existir), então o
//! restore re-insere RGBA — e só o id dos pixels volta **igual**. Com o outro, o round-trip daria
//! um id novo e a fonte do documento apontaria para o nada.
//!
//! O gate `a_saved_pattern_reopens_with_the_same_asset_id` prende exactamente isso.
//!
//! # Dois padrões com a mesma arte custam UMA entrada
//!
//! A colheita é chaveada pelo `AssetId` (um `BTreeMap`), então a arte partilhada não se duplica no
//! ficheiro — a mesma propriedade que o `project_sprite_pixels` documenta.

use ph2d_asset::AssetId;
use ph2d_vec_scene::{Paint, PatternSource};
use std::collections::BTreeMap;

/// A versão do blob. ⚠️ Ele carrega a própria, como o `timeline` e o `sculpt` — é isso que faz um
/// campo novo AQUI dentro não voltar a bumpar o `PROJECT_SCHEMA`.
const PATTERN_ART_DOC_VERSION: u32 = 1;

/// A arte de um padrão embutida no projecto.
#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct SavedPatternArt {
    /// O `AssetId` que o documento nomeia — e que o restore tem de reproduzir.
    pub(crate) id: AssetId,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) rgba: Vec<u8>,
}

/// ⭐⭐⭐ **A metade PURA da LEITURA** — descodifica o blob sem tocar no `AssetDb` (2026-08-30).
///
/// # Por que ela existe: para o load poder RECUSAR antes de mutar a sessão
///
/// ⚠️ **A lei já estava escrita neste app, e este era o único blob que não a obedecia.** A timeline
/// e a escultura fazem o load inteiro ser **RECUSADO** quando o documento delas é ilegível, com a
/// razão no [`crate::project_load`]: *"abrir sem ela seria a pior das opções: a cena aparece,
/// parece certa, … e o próximo Ctrl+S grava esse vazio POR CIMA do arquivo"*. A arte dos padrões
/// respondia com um `eprintln!` e seguia — e o resto da cadeia é idêntico, com o sujeito trocado:
/// a fonte deixa de resolver, toda forma com estampa pinta a `fallback`, e o
/// [`super::App::collect_texture_pattern_art`] do save seguinte **salta o `AssetId` que já não está
/// no `AssetDb`** ⇒ os pixels deixam de existir no disco. *A arte não some por um defeito; some
/// porque o app abriu, mentiu e salvou.*
///
/// ⭐ **E isto fecha um buraco de FUTURO de graça:** o [`PATTERN_ART_DOC_VERSION`] vive fora da
/// escada do `PROJECT_SCHEMA` (é o que faz um campo novo aqui dentro não bumpar o schema do
/// projecto), e o preço era que, no dia em que ele subisse, um ficheiro anterior abriria **sem a
/// arte** em vez de ser recusado. Com a recusa, esse preço desaparece.
///
/// ⚠️ **Um blob VAZIO não é erro** — é o caso normal (um projecto sem padrão nenhum não paga byte
/// nenhum), e tratá-lo como erro fecharia a porta a quase todo ficheiro do repo.
///
/// ⭐⭐ **E ela torna o caminho de leitura GATEÁVEL**, que é a segunda razão. Instalar precisa de um
/// `AppGfx` que segura uma surface de janela real ⇒ não é alcançável de um teste; descodificar não
/// precisa de nada. É a MESMA cirurgia que a escrita já sofreu ([`art_ids_named_by`]), e pela mesma
/// razão.
pub(crate) fn decode_texture_pattern_art(blob: &[u8]) -> Result<Vec<SavedPatternArt>, String> {
    if blob.is_empty() {
        return Ok(Vec::new());
    }
    let (ver, arts) = postcard::from_bytes::<(u32, Vec<SavedPatternArt>)>(blob)
        .map_err(|e| format!("blob ilegivel: {e}"))?;
    if ver != PATTERN_ART_DOC_VERSION {
        return Err(format!(
            "versao {ver}, este binario le {PATTERN_ART_DOC_VERSION}"
        ));
    }
    Ok(arts)
}

/// Um blob **bem-formado** de arte de padrão, para os gates do load.
///
/// ⚠️ Ele vive aqui, ao lado do decodificador, e **não** na suíte que o usa: escrever o formato à
/// mão noutro ficheiro seria um segundo escritor do blob, e ele divergiria no primeiro campo novo
/// — que é exactamente a razão pela qual o `PATTERN_ART_DOC_VERSION` existe.
#[cfg(test)]
pub(crate) fn encode_for_test(w: u32, h: u32) -> Vec<u8> {
    let rgba: Vec<u8> = (0..w * h)
        .flat_map(|i| [(i % 251) as u8, 9, 40, 255])
        .collect();
    postcard::to_allocvec(&(
        PATTERN_ART_DOC_VERSION,
        vec![SavedPatternArt {
            id: AssetId::from_bytes(&rgba),
            width: w,
            height: h,
            rgba,
        }],
    ))
    .expect("o proprio formato escreve-se")
}

/// ⭐⭐ **QUE ARTES esta cena nomeia** — a metade PURA da colheita (plano 35, wave D).
///
/// ⚠️ **Extraída para poder ser GATEADA.** A colheita precisa do `AssetDb`, que vive num `AppGfx`
/// que segura uma surface de janela real ⇒ ela não é alcançável de um teste, e a única defesa
/// possível era um gate de FONTE a procurar a palavra `stroke`. Esta função responde à pergunta que
/// de facto importa — *o coletor vê as duas tintas?* — sobre uma cena construída à mão.
///
/// ⚠️ **As DUAS tintas.** Enquanto isto lia só o `fill`, uma forma cujo TRAÇO tinha padrão gravava o
/// `AssetId` no documento e **nunca os pixels**: reabrir dava uma linha pintada a cor de recurso,
/// para sempre e **sem erro nenhum a que agarrar** — exactamente o defeito que este ficheiro existe
/// para curar, com o sujeito trocado. *Um coletor que varre metade do modelo é indistinguível de um
/// que varre tudo, até alguém usar a outra metade.*
///
/// ⏳ Uma fonte-FORMA (W7) não entra: ela É o documento, e o documento já viaja. Só a imagem precisa
/// dos pixels.
#[must_use]
pub(crate) fn art_ids_named_by(
    scene: &ph2d_vec_scene::VecScene,
) -> std::collections::BTreeSet<AssetId> {
    let mut ids = std::collections::BTreeSet::new();
    for path in scene.paths() {
        let da_forma = match path.fill.as_ref() {
            Some(Paint::Pattern(p)) => Some(p.as_ref()),
            _ => None,
        };
        let do_traco = path
            .stroke
            .as_ref()
            .and_then(ph2d_vec_scene::StrokeSpec::pattern);
        for p in da_forma.into_iter().chain(do_traco) {
            if let PatternSource::Image(id) = p.source {
                ids.insert(id);
            }
        }
    }
    ids
}

impl crate::App {
    /// Colhe a arte de cada padrão da cena, para embutir no ficheiro.
    ///
    /// Vazio quando não há padrão nenhum — um projecto sem padrões não paga byte nenhum.
    pub(super) fn collect_texture_pattern_art(&self, scene: &ph2d_vec_scene::VecScene) -> Vec<u8> {
        let Some(gfx) = self.gfx.as_ref() else {
            return Vec::new();
        };
        // ⚠️ **A lista de quem embutir sai da porta PURA** (`art_ids_named_by`), e não de um segundo
        // passeio pela cena — duas travessias divergem no dia em que uma ganhar uma tinta.
        let mut by_id: BTreeMap<AssetId, SavedPatternArt> = BTreeMap::new();
        for id in art_ids_named_by(scene) {
            // ⚠️⚠️ **EM VOZ ALTA, os dois.** Estes eram dois `continue` mudos, e eles são o último
            // elo de uma cadeia que APAGA autoria: a fonte não resolve -> a forma pinta a
            // `fallback` -> o save reescreve o ficheiro **sem** os pixels -> a arte deixou de
            // existir no disco, com o documento bem-formado e o toast a dizer *"Project saved"*.
            //
            // ⭐ A recusa do load ([`decode_texture_pattern_art`]) tira o caminho COMUM até aqui;
            // o que sobra é o asset que nunca esteve no `AssetDb` (despejado, ou um documento
            // montado por outra rota), e para esse a única resposta honesta é dizer o nome.
            // ⛔ **Recusar o SAVE seria pior** — o artista perderia o trabalho da sessão para
            // proteger uma arte que já estava perdida.
            let Some(asset) = gfx.asset_db.get(&id) else {
                eprintln!(
                    "[proj] arte de padrao: o asset {} nao esta' na sessao - ele NAO vai para o \
                     ficheiro, e a forma que o nomeia reabre a pintar a cor de recurso",
                    id.to_hex()
                );
                continue;
            };
            let Some((width, height, rgba)) = asset.image_rgba8() else {
                eprintln!(
                    "[proj] arte de padrao: o asset {} nao e' uma imagem - ele NAO vai para o \
                     ficheiro",
                    id.to_hex()
                );
                continue;
            };
            by_id.insert(
                id,
                SavedPatternArt {
                    id,
                    width,
                    height,
                    rgba: rgba.into_owned(),
                },
            );
        }
        if by_id.is_empty() {
            return Vec::new();
        }
        let arts: Vec<SavedPatternArt> = by_id.into_values().collect();
        postcard::to_allocvec(&(PATTERN_ART_DOC_VERSION, arts)).unwrap_or_default()
    }

    /// Devolve a arte ao `AssetDb`, **sob o mesmo id**.
    ///
    /// ⚠️⚠️ **Recebe a arte JÁ DESCODIFICADA, e o tipo é a defesa.** Enquanto isto recebia um
    /// `&[u8]`, a única forma de tratar um blob ilegível era aqui dentro — e aqui dentro já é
    /// **tarde**: a sessão já foi mutada, então a única saída era `eprintln!` + seguir, que é
    /// exactamente *abrir a mentir*. Com [`decode_texture_pattern_art`] a devolver um `Result`, o
    /// caminho não-descodificado **deixa de existir**: quem quiser ignorar o erro tem de escrever
    /// um `unwrap_or_default()` visível. *Uma lei imposta pelo tipo não precisa de um gate a
    /// lembrá-la.*
    pub(super) fn install_texture_pattern_art(&mut self, arts: Vec<SavedPatternArt>) {
        if arts.is_empty() {
            return;
        }
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        for a in arts {
            let back = gfx
                .asset_db
                .insert_image_rgba8(a.width, a.height, a.rgba.clone());
            // ⚠️ **Em voz alta.** Um id que não volta igual é uma fonte que nunca mais resolve, e o
            // sintoma seria *"o meu padrão virou uma cor chapada"* sem nada no log.
            if back != a.id {
                eprintln!(
                    "[proj] arte de padrao: o id nao voltou igual ({} != {}) - o padrao vai pintar \
                     a cor de recurso",
                    back.to_hex(),
                    a.id.to_hex()
                );
            }
        }
    }
}

#[cfg(test)]
#[path = "project_texture_pattern_tests.rs"]
mod tests;
