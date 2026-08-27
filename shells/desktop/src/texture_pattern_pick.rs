//! **ESCOLHER A ARTE de um padrão de textura** (plano 33, W4) — a porta que o chip *Tile* abre.
//!
//! # A 4ª condição de uma costura de UI
//!
//! *O componente existe · é pintado e registado · o clique chega ao barramento · e a **SEQUÊNCIA
//! leva a algum lugar**.* As três primeiras têm gate de costura nesta casa; a quarta é a que se
//! esquece — e aqui ela é o assunto inteiro: **escolher *Tile* numa forma que ainda não tem padrão
//! não pode não fazer nada.** Um chip que muda o tipo de preenchimento para algo invisível é o
//! defeito que esta linha já recebeu de report três vezes.
//!
//! ⚠️ **Desistir do diálogo NÃO muda o preenchimento.** O gesto que destrói o trabalho do artista
//! pergunta; um `Cancel` que apagasse o gradiente dele seria o pior dos dois mundos.
//!
//! # As duas listas são UMA
//!
//! Os filtros do diálogo saem de [`ph2d_asset::SUPPORTED_IMAGE_EXTENSIONS`], a mesma lista de que o
//! `import_router` deriva o predicado do *drop*. ⚠️ Uma lista escrita à mão ao lado de um predicado
//! são duas respostas à mesma pergunta, e a que o artista vê é a que envelhece — foi o defeito de
//! 23/08 em que o `.gif`/`.psd`/`.ora` estavam invisíveis no *Import…* havia meses.

use ph2d_asset::AssetDb;
use ph2d_vec_scene::{Paint, PatternSource, Rgba8, VecPathId, VecScene};

/// Quantas cópias do padrão cabem, por omissão, no lado MENOR da forma.
///
/// ⚠️ **É um default de produto, não uma medição** — e a escolha é conservadora de propósito: com
/// uma cópia só, um padrão não se lê como padrão (parece uma imagem esticada); com vinte, a arte
/// vira textura de grão e o artista não vê o que escolheu. Três mostra o motivo E a repetição.
const DEFAULT_TILES_ACROSS: f64 = 3.0;

/// A fonte a usar quando o artista escolhe *Tile* na forma `sel`.
///
/// - a forma **já** tem padrão -> a fonte dele (trocar de chip e voltar não perde a arte);
/// - senão -> abre o diálogo. `None` = desistiu, e quem chama **não muda nada**.
///
/// ⚠️ **Funções livres sobre `&AssetDb`, e não métodos de `App`, e a razão é o EMPRÉSTIMO:** no
/// quadro, o `self.gfx` está mutavelmente emprestado desde o topo e vive até ao fim — um `&mut
/// self` aqui não compila. Quem chama passa o `asset_db` que já tem desestruturado.
pub(crate) fn source_for(
    assets: &AssetDb,
    scene: &VecScene,
    sel: VecPathId,
) -> Option<PatternSource> {
    if let Some(Paint::Pattern(p)) = scene.path(sel).and_then(|p| p.fill.as_ref()) {
        return Some(p.source);
    }
    let dialog = rfd::FileDialog::new().add_filter(
        "Image",
        &ph2d_asset::SUPPORTED_IMAGE_EXTENSIONS
            .iter()
            .map(|e| e.trim_start_matches('.'))
            .collect::<Vec<_>>(),
    );
    // ⚠️ Pela PORTA (`modal::pick_file`), nunca `dialog.pick_file()` direto: o diálogo congela o
    // laço, e quem congela **declara** — senão a mensagem escrita a seguir vive um quadro só e o
    // artista não a lê.
    let path = crate::modal::pick_file(dialog)?;
    let bytes = std::fs::read(&path)
        .map_err(|e| {
            eprintln!(
                "[ph2d-vec] padrao: nao consegui ler {}: {e}",
                path.display()
            )
        })
        .ok()?;
    // ⚠️⚠️ **O id tem de ser o dos PIXELS, não o do FICHEIRO — e isto é a persistência a decidir a
    // autoria.**
    //
    // O `insert_image_bytes` cunha `blake3(bytes do ficheiro)`; o `insert_image_rgba8` cunha
    // `blake3(dims + RGBA)`. O save de um padrão embute **pixels** (o ficheiro do artista pode ter
    // mudado de sítio, ou nunca mais existir), então reabrir o projecto re-insere RGBA — e só o
    // segundo id volta **igual**. Com o primeiro, a fonte do padrão deixaria de resolver ao reabrir
    // e a forma pintaria a `fallback` para sempre, sem erro nenhum a que agarrar.
    //
    // A dupla inserção é deliberada e barata: a primeira **descodifica**, a segunda dá a
    // identidade durável. As duas são `or_insert_with` (HR-6), então nada se duplica no disco.
    let decoded = assets
        .insert_image_bytes(&bytes)
        .map_err(|e| eprintln!("[ph2d-vec] padrao: {} nao e' imagem: {e:?}", path.display()))
        .ok()?;
    // ⚠️ O `Arc<Asset>` tem de viver numa ligação: `assets.get(..)?.image_rgba8()?` empresta de um
    // temporário que morre no fim da expressão.
    let asset = assets.get(&decoded)?;
    let (w, h, px) = asset.image_rgba8()?;
    let rgba = px.into_owned();
    Some(PatternSource::Image(assets.insert_image_rgba8(w, h, rgba)))
}

/// O TAMANHO de mundo com que um padrão novo nasce, para a forma `sel`.
///
/// ⭐ **Preserva o aspecto da arte.** Nascer esticado é a primeira coisa que o artista veria, e ele
/// leria isso como *"a ferramenta deformou a minha imagem"* — não como um default.
///
/// O lado maior fica em `1/DEFAULT_TILES_ACROSS` do lado MENOR da forma, então uma forma comprida
/// não recebe um ladrilho gigante.
#[must_use]
pub(crate) fn default_size(
    assets: &AssetDb,
    scene: &VecScene,
    sel: VecPathId,
    source: &PatternSource,
) -> [f64; 2] {
    let (lo, hi) = scene.path_bbox(sel).unwrap_or(([0.0, 0.0], [1.0, 1.0]));
    let target =
        ((hi[0] - lo[0]).abs().min((hi[1] - lo[1]).abs()) / DEFAULT_TILES_ACROSS).max(f64::EPSILON);
    let art = art_dims(assets, source).unwrap_or([1, 1]);
    let (aw, ah) = (f64::from(art[0].max(1)), f64::from(art[1].max(1)));
    let s = target / aw.max(ah);
    [aw * s, ah * s]
}

/// As dimensões em pixels da arte de uma fonte (`None` se ela ainda não resolve).
fn art_dims(assets: &AssetDb, source: &PatternSource) -> Option<[u32; 2]> {
    let PatternSource::Image(id) = source else {
        return None;
    };
    let asset = assets.get(id)?;
    let (w, h, _) = asset.image_rgba8()?;
    Some([w, h])
}

/// A cor de recurso de um padrão novo: a que a forma já pintava.
///
/// ⚠️ **Não é decoração.** Ela é o que se vê enquanto o ladrilho não resolve, e herdá-la da tinta
/// anterior faz a troca de chip parecer contínua em vez de um piscar para uma cor arbitrária.
#[must_use]
pub(crate) fn fallback_of(cur: Option<&Paint>) -> Rgba8 {
    cur.map_or(Rgba8::new(255, 255, 255, 255), Paint::primary_color)
}
