//! ⭐⭐⭐ **A PORTA QUE DESENHA A CENA** — as duas metades do desenho vectorial do documento.
//!
//! ⚠️ **Cortado do `lib.rs` em 2026-09-07 pelo tecto de LOC da workspace (779/700), e o corte é por
//! RESPONSABILIDADE:** aquele ficheiro declara os TIPOS que atravessam o desenho (a geometria viva,
//! as imagens de FX, os ladrilhos, as peles) e as portas de OVERLAY (gizmos, guias, marcas); este
//! declara *«desenhe o documento»*.
//!
//! As duas metades são complementares e existem por causa do vidro jateado do *Edit Prefab*: o
//! [`dispatch`] desenha o MUNDO (saltando a receita aberta) e o [`dispatch_isolated`] desenha **só**
//! a receita, numa cena que o presente compõe depois do borrão.

use super::{
    BrushArts, Derived, DilatedPaints, FxImage, FxImages, LiveGeometry, PatternSlot, PatternTiles,
    WidgetSkins, blend, draw_path_tiled, frame_clip, path_to_screen,
};
use ph2d_vec_scene::{VecPath, VecScene, VecViewState, VecXforms};
use ph2d_vector::{Affine, VectorScene};

/// Desenha toda a `scene` no `target` (o `VectorScene` do frame) sob `camera`
/// (o world→screen). Fill primeiro, stroke por cima.
///
/// `view` diz quem a ÁRVORE do editor esconde — a visibilidade é da entidade ECS
/// do path e dos ancestrais dela, não do documento (ADR-0110). `xforms` diz onde
/// cada path está — o `Transform` da entidade dele (ADR-0111). O stroke escala
/// junto com a forma, como o contorno de um sprite escalado.
///
/// `live` ([`LiveGeometry`]) troca a geometria de um caminho pela DERIVADA dele **no z dele** —
/// não num passe por cima de tudo. É o que mantém a promessa do Offset vivo: o documento guarda
/// a curva autorada (o modo Node edita os nós DELA) e o que se vê é o resultado, empilhado
/// exatamente onde a forma sempre esteve.
///
/// `fx` ([`FxImages`]) injeta o FX raster de uma forma **no z dela**, SUBSTITUINDO o desenho
/// vetorial: a pilha de filtros já compôs sombra, brilho e forma numa imagem só (o halo entra por
/// baixo DENTRO do op — ver `ph2d_render::fx_stack`), então não há um "atrás" que o compositor
/// precise conhecer. As imagens já vêm em coordenadas de tela — o `dispatch` só as encoda, sem
/// tocar GPU. Vazio = sem FX (o caminho comum é byte-idêntico ao mundo pré-FX).
///
/// `skins` ([`WidgetSkins`]) injeta a PELE de widget de uma forma **no z dela**, também
/// SUBSTITUINDO o desenho vetorial — ver o tipo para o porquê de o fragmento ser opaco.
///
/// ⚠️ Oito argumentos, e agrupá-los num struct seria pior: cada um é uma FONTE independente com
/// dono próprio na shell (a cena, a árvore, as poses, a geometria derivada, as imagens de FX, as
/// peles, a câmera) e um struct-de-argumentos convidaria alguém a guardá-lo entre frames — que é
/// exactamente como um deles ficaria velho sem que nada dissesse.
#[allow(clippy::too_many_arguments)]
pub fn dispatch(
    scene: &VecScene,
    view: &VecViewState,
    xforms: &VecXforms,
    live: &LiveGeometry,
    fx: &FxImages,
    skins: &WidgetSkins,
    patterns: &PatternTiles,
    brushes: &BrushArts,
    dilated: &DilatedPaints,
    camera: Affine,
    target: &mut VectorScene,
) {
    // As MOLDURAS abertas (`frame_clip`). Vazio no caminho comum, e então tudo abaixo é o desenho
    // de sempre — `open_after` e `close_after` não fazem nada sem `view.clips`.
    let mut frames = frame_clip::OpenClips::default();
    // ⭐⭐⭐ **O ISOLAMENTO do *Edit Prefab*** (Enio, 2026-09-07): com uma receita aberta, o mundo
    // fica atrás de um **vidro jateado** e ela desenha-se por cima, nítida. ⛔ Sem receita aberta
    // isto é `false` e cada linha abaixo é a de sempre.
    //
    // ⚠️⚠️ **Aqui só existe a metade do MUNDO.** O borrão não é uma camada do Vello — é um passe
    // sobre a textura em que este desenho aterra (`ph2d_render::FrostPass`), e o que fica por cima
    // dele é a cena que o [`dispatch_isolated`] produz. *Uma camada de opacidade aqui recuaria a
    // arte vectorial e deixaria o fundo do canvas e as sprites nítidos por baixo dela — metade do
    // mundo atrás do vidro e metade à frente.*
    let isolating = view.isolating();
    for path in scene.paths() {
        // ⚠️ **A ordem dentro do laço é a LEI**: desenha, depois abre, depois fecha. A moldura é o
        // PRIMEIRO membro da própria sub-árvore (a pilha de z é o DFS na ordem — o filho desenha
        // sobre o pai), então o preenchimento dela é o fundo do card **por sair primeiro**; abrir
        // antes de desenhar recortaria a moldura pela própria silhueta.
        if !view.is_hidden(path.id) && !(isolating && view.is_isolated(path.id)) {
            draw_one(
                path, scene, view, xforms, live, fx, skins, patterns, brushes, dilated, camera,
                target,
            );
        }
        // ⚠️ FORA do filtro de escondido: push e pop de camada têm de se emparelhar mesmo quando
        // a moldura não desenha (ver `frame_clip`).
        frames.open_after(path.id, scene, view, xforms, live, camera, target);
        frames.close_after(path.id, view, target);
    }
    frames.close_all(target);
}

/// ⭐⭐⭐ **SÓ a receita aberta** — o que fica ACIMA do vidro jateado (Enio, 2026-09-07).
///
/// Irmã do [`dispatch`] e **exactamente o complemento dela**: aquela salta o que está em
/// [`ph2d_vec_scene::VecViewState::isolated`], esta desenha só isso. ⛔ Sem receita aberta ela não
/// desenha nada, e o presente nem a compõe.
///
/// ⚠️ **Cena própria, e não a mesma com um `push_layer` no meio:** o que separa as duas metades é
/// um passe de GPU (o borrão) sobre a textura do mundo, e um passe de GPU não cabe no meio de uma
/// codificação de Vello. *É isto que faz a receita ficar acima do borrão em vez de dentro dele.*
///
/// ⚠️ **Sem as MOLDURAS**, de propósito: um recorte de moldura é um intervalo sobre a pilha de z
/// do mundo, e a receita já não está nessa pilha — ela flutua acima de tudo. Cortá-la por uma
/// moldura de que ela não faz parte era o defeito que a primeira versão desta lei já evitava, ao
/// desenhá-la depois do `close_all`.
#[allow(clippy::too_many_arguments)]
pub fn dispatch_isolated(
    scene: &VecScene,
    view: &VecViewState,
    xforms: &VecXforms,
    live: &LiveGeometry,
    fx: &FxImages,
    skins: &WidgetSkins,
    patterns: &PatternTiles,
    brushes: &BrushArts,
    dilated: &DilatedPaints,
    camera: Affine,
    target: &mut VectorScene,
) {
    if !view.isolating() {
        return;
    }
    for path in scene.paths() {
        if view.is_isolated(path.id) && !view.is_hidden(path.id) {
            draw_one(
                path, scene, view, xforms, live, fx, skins, patterns, brushes, dilated, camera,
                target,
            );
        }
    }
}

/// **UM objecto da cena, com tudo o que ele desenha** — a porta que as DUAS passagens do
/// [`dispatch`] partilham.
///
/// ⚠️ **Extraída, e não copiada:** a segunda passagem (a receita isolada) tem de desenhar
/// exactamente o mesmo — a camada do objecto, o FX que substitui o desenho, a pele de widget, os
/// ladrilhos, a arte de pincel e a geometria viva. *Uma segunda cópia deste corpo desenharia a
/// receita sem o FX dela no dia em que alguém lhe pusesse um, e ninguém veria porquê.*
#[allow(clippy::too_many_arguments)]
fn draw_one(
    path: &VecPath,
    scene: &VecScene,
    view: &VecViewState,
    xforms: &VecXforms,
    live: &LiveGeometry,
    fx: &FxImages,
    skins: &WidgetSkins,
    patterns: &PatternTiles,
    brushes: &BrushArts,
    dilated: &DilatedPaints,
    camera: Affine,
    target: &mut VectorScene,
) {
    // ⭐⭐⭐ **A CAMADA DO OBJECTO** (v19): opacidade e modo de mistura da forma compõem-na
    // UMA vez, com tudo o que ela desenha lá dentro. Opaca e `Normal` ⇒ nada é empurrado e
    // o desenho é byte-idêntico ao de sempre. ⚠️ Ela envolve os TRÊS braços de propósito —
    // a imagem de FX e a pele de widget substituem o desenho, mas continuam a ser **este
    // objecto**, e desvanecer só o braço do meio seria a opacidade a funcionar até alguém
    // ligar um filtro.
    let bound = view.bound_style(path.id);
    let layered = blend::open_object_layer(target, scene, xforms, live, fx, path, bound, camera);
    // O FX da forma, se houver, TOMA o lugar do desenho: a pilha já compôs tudo o que se
    // vê desta forma (halo incluído) numa imagem só, no z dela.
    if let Some(img) = fx.get(&path.id) {
        draw_fx_image(img, target);
    } else if let Some(skin) = skins.get(&path.id) {
        // ⚠️ A pele já foi pintada em coordenadas de TELA (o shell cruzou a câmera para
        // achar o retângulo da forma), então ela entra SEM transform — o mesmo contrato
        // da `FxImage` ao lado, e pela mesma razão: quem sabe onde a forma está na tela é
        // quem tem a câmera, e ele já respondeu.
        target.inner_mut().append(skin.inner(), None);
    } else {
        // (A TINTA que os tokens dão a esta forma foi perguntada UMA vez, acima — e ela
        // vale também para a geometria DERIVADA dela: as cópias de offset/pattern/espelho
        // têm id próprio, então procurá-las na tabela não acharia nada e o token pararia
        // na borda do primeiro efeito.)
        // A derivada já está em MUNDO (a shell assou a pose dentro dela), então ela sobe
        // pela CÂMERA e não pelo afim do path — aplicar a pose duas vezes foi bug real
        // desta linha.
        // ⚠️ **O ladrilho é procurado pelo id da FONTE, tal como a tinta dos tokens
        // logo acima e pela mesma razão**: as cópias derivadas (offset/pattern-on-path/
        // espelho) têm id próprio, então uma busca por elas não acharia nada e o padrão
        // pararia na borda do primeiro efeito.
        let tile = patterns.get(&(path.id, PatternSlot::Fill));
        let stroke_tile = patterns.get(&(path.id, PatternSlot::Stroke));
        // ⭐ **A arte do PINCEL, pelo id da FONTE — a mesma lei do ladrilho logo acima.**
        let art = brushes.get(&path.id).map(Vec::as_slice);
        if let Some(items) = live.get(&path.id) {
            for item in items {
                // ⛔ `None`: uma cópia DERIVADA tem id próprio, e a geometria
                // dilatada é indexada pelo id da FONTE — a mesma lei do ladrilho e da
                // arte de pincel logo acima. O censo `the_artless_draw_routes_are_
                // declared` conta esta rota.
                draw_path_tiled(
                    &item.painted(bound),
                    camera,
                    target,
                    Derived {
                        tile,
                        stroke_tile,
                        brush_art: art,
                        dilated: None,
                    },
                );
            }
        } else {
            let transform = path_to_screen(xforms, path.id, camera);
            draw_path_tiled(
                &path.painted(bound),
                transform,
                target,
                Derived {
                    tile,
                    stroke_tile,
                    brush_art: art,
                    dilated: Some(dilated),
                },
            );
        }
    }
    if layered {
        target.pop_layer();
    }
}

/// Encoda uma [`FxImage`] na cena, no retângulo de tela dela. RGBA reta (a mesma política do
/// overlay de Background-Removal).
fn draw_fx_image(img: &FxImage, target: &mut VectorScene) {
    // Id de Blob estável ⇒ o Vello reusa a textura do atlas (sem re-upload por frame).
    target.draw_stable_image(&img.image, img.rect, ph2d_vector::ImageQuality::Medium);
}
