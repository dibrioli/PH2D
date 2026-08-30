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

/// A fonte a usar quando o artista escolhe *Pattern* na forma `sel`.
///
/// - a forma **já** tem padrão -> a fonte dele (trocar de chip e voltar não perde a arte);
/// - senão -> [`PatternSource::None`]: o padrão nasce **sem arte escolhida**.
///
/// # ⛔⛔ Ela ABRIA O DIÁLOGO DE IMAGEM, e isso decidia pelo artista
///
/// Report do Enio (2026-08-30): *"ao apertar pattern o usuário é obrigado a selecionar uma img no
/// dialog. não tem a opção de usar shape até que se use a img em pattern"*.
///
/// As duas artes de um padrão nascem por portas **diferentes** — uma imagem por diálogo de ficheiro,
/// uma forma pelo gesto de duas mãos —, e um chip só pode abrir uma. Abrindo a da imagem, a da forma
/// ficava **atrás** dela: para usar uma forma era preciso primeiro escolher uma imagem que se ia
/// deitar fora. *Uma porta que serve dois destinos e conhece um só não é uma porta: é um desvio.*
///
/// ⭐ ⇒ o chip deixa de escolher, e quem escolhe é o **painel**, que já pinta *Source…* e
/// *Use Shape…* lado a lado com a dica por cima — a UI que a W11 construiu para a arte APAGADA
/// serve, sem uma linha nova, a arte AINDA NÃO ESCOLHIDA.
///
/// ⚠️ **PREÇO NOMEADO: o caminho da imagem passa a ter um clique a mais** (chip *Pattern*, depois
/// *Source…*). É o custo de a escolha ser explícita, e é a decisão desta wave — não um descuido.
///
/// ⭐ **E o gesto deixou de poder não fazer nada.** Antes, desistir do diálogo devolvia `None` e o
/// preenchimento não mudava: o artista carregava em *Pattern* e **a app não fazia nada visível**.
/// Hoje ele vê sempre a secção nascer, e desfazê-la é um `Ctrl+Z`.
///
/// ⚠️ **Funções livres sobre `&AssetDb`, e não métodos de `App`, e a razão é o EMPRÉSTIMO:** no
/// quadro, o `self.gfx` está mutavelmente emprestado desde o topo e vive até ao fim — um `&mut
/// self` aqui não compila. Quem chama passa o `asset_db` que já tem desestruturado.
pub(crate) fn source_for(scene: &VecScene, sel: VecPathId) -> Option<PatternSource> {
    if let Some(Paint::Pattern(p)) = scene.path(sel).and_then(|p| p.fill.as_ref()) {
        return Some(p.source);
    }
    Some(PatternSource::None)
}

/// **Abre o diálogo, SEMPRE** — a porta do botão *Source…*, que existe para TROCAR a arte.
///
/// ⚠️ É a metade crua da [`source_for`], e as duas são portas do mesmo gesto em situações
/// diferentes: o chip *Pattern* pergunta *"há arte?"* primeiro; o botão *Source…* já sabe que há e
/// quer outra. ⛔ Chamar a `source_for` no botão devolveria a arte que já lá está e o botão seria
/// **mudo** — o defeito que esta linha recebeu três vezes.
pub(crate) fn pick_source(assets: &AssetDb) -> Option<PatternSource> {
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
///
/// ⛔⛔ **E O CANTO É O DA FORMA, não a origem do MUNDO** — report do Enio (2026-08-27:
/// *"clamp deixa tudo em branco"*).
///
/// A 1.ª versão nascia em `[0, 0]`. Com `Tile`/`Mirror` isso é invisível (o padrão repete-se por
/// toda a parte); com **`Clamp`** é catastrófico: o ladrilho fica na origem do mundo, a forma
/// amostra `uv` a centenas de texels de distância, e o `Extend::Pad` devolve a **borda** da arte
/// esticada — um borrão chapado, nunca a imagem. Medido na cena de smoke: as seis formas caem em
/// `uv.x` de **−331 a +331** com o ladrilho a cobrir `0..32`.
///
/// ⚠️ **É a metade que faltava da lei que este plano se gabava de honrar.** A §1.2 do
/// [plano 33](../../docs/Vector%20Module/33_plano_texture_pattern.md) diz que a ancoragem do
/// Illustrator à origem da régua é *"o erro clássico da categoria"* — eu evitei a metade do
/// TRANSFORM (o padrão anda com a forma) e reproduzi a metade do NASCIMENTO.
#[must_use]
pub(crate) fn default_placement(
    assets: &AssetDb,
    scene: &VecScene,
    sel: VecPathId,
    source: &PatternSource,
) -> ([f64; 2], [f64; 2]) {
    let (lo, hi) = scene.path_bbox(sel).unwrap_or(([0.0, 0.0], [1.0, 1.0]));
    let target =
        ((hi[0] - lo[0]).abs().min((hi[1] - lo[1]).abs()) / DEFAULT_TILES_ACROSS).max(f64::EPSILON);
    let art = art_dims(assets, source).unwrap_or([1, 1]);
    let (aw, ah) = (f64::from(art[0].max(1)), f64::from(art[1].max(1)));
    let s = target / aw.max(ah);
    ([aw * s, ah * s], lo)
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

#[cfg(test)]
#[path = "texture_pattern_pick_tests.rs"]
mod tests;
