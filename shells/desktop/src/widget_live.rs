//! **A PELE de widget, viva** — o degrau 2 do §2 do plano UI/UX (W6.2).
//!
//! Uma forma marcada com [`ph2d_ecs::VecWidget`] deixa de ser desenhada como vetor e passa a ser
//! pintada pelo **pintor REAL** do catálogo, no retângulo dela. Este módulo é a ponte, e ele mora
//! na shell por uma razão de arquitetura: só aqui as duas metades se encontram — a
//! `ph2d-editor-core` (que sabe o que é um botão) e a `ph2d-vec-render` (que sabe onde a forma
//! está no z). A crate de render recebe um fragmento **opaco** e nunca aprende o que é um widget.
//!
//! # Três perguntas, três donos, e nenhuma resposta duplicada
//!
//! - **ONDE** — [`ph2d_vec_render::path_screen_bounds`], a MESMA função que o produtor de FX
//!   raster usa para dimensionar o scratch dele. Uma segunda projeção descreveria a forma noutro
//!   lugar da tela no primeiro zoom.
//! - **O QUÊ** — o `kind` do componente, traduzido UMA vez por
//!   [`ph2d_editor_core::widget::WidgetKind::from_code`]; código desconhecido devolve `None` e a
//!   forma **desenha como sempre** (é a compatibilidade, não um erro).
//! - **O RÓTULO** — o [`ph2d_ecs::Name`] da entidade, o nome que o artista já digita na
//!   Hierarquia. Sem `Name`, rótulo vazio: o widget desenha a moldura e mais nada, que é a
//!   resposta honesta para um objeto sem nome.
//!
//! ⚠️ **E o GLIFO de um `IconButton` é a QUARTA resposta, dada pelo próprio desenho** — a forma
//! que veste o botão *é* o ícone, normalizada por [`crate::widget_icon::icon_face`], a MESMA porta
//! que o painel gerado percorre. Ver o doc dela para o que a pose não faz.
//!
//! # A APARÊNCIA não vem daqui, e isso é decisão
//!
//! Cor, raio e tipografia saem dos **tokens** (`ph2d_tokens::resolve`, que a W6.1 tornou
//! autorável) — o desenho responde ONDE e O QUÊ. Um mapeamento por-tipo (*"o preenchimento da
//! forma vira a cor da swatch"*) é uma tabela de casos especiais e está deliberadamente **não
//! construído**; ver o doc de [`ph2d_editor_core::widget::paint_widget_skin`].
//!
//! # O widget é pintado em pixels de TELA
//!
//! Um token é um número em **px** — `Radius::Md.px()`, `ICON_BTN_SIZE_PX`, o corpo do texto em pt.
//! Pintar a pele numa escala inventada mostraria ao artista um número que não é o número que o
//! app usa, que é exactamente a falha que a W6.1 existe para impedir. ⚠️ A consequência honesta:
//! **o zoom do canvas amplia a MOLDURA, não os detalhes do controle** — a caixa cresce, os cantos
//! ficam do tamanho que o token diz. Um widget é autorado em px porque um token é px; magnificação
//! fiel exigiria uma constante px↔mundo, e essa constante já tem dono noutro lugar
//! (`ProjectSettings::pixels_per_meter`) — puxá-la para cá seria uma segunda resposta a *"quanto
//! mede um pixel"*.

use ph2d_ecs::{Entity, Name, SimWorld, VecWidget, VecWidgetIcon};
use ph2d_editor::icons::IconId;
use ph2d_editor::widget::{SkinParam, WidgetKind, icon_glyph, paint_widget_skin};
use ph2d_editor::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vec_render::{LiveGeometry, WidgetSkins};
use ph2d_vec_scene::{Paint, VecPathId, VecScene, VecXforms};
use ph2d_vector::Affine;

use crate::vec_entities::VecEntityMap;
use crate::widget_icon::icon_face;

/// A pele autorada de `id`, se houver. **Porta única**: o cozimento e o painel perguntam AQUI.
pub(crate) fn spec_of(sim: &SimWorld, map: &VecEntityMap, id: VecPathId) -> Option<VecWidget> {
    let &bits = map.get(&id)?;
    sim.world()
        .get::<VecWidget>(Entity::from_bits(bits))
        .copied()
}

/// **O ícone ESCOLHIDO** desta forma, se o artista escolheu um.
///
/// ⚠️ Um slug que este build não conhece devolve `None` — e o `None` cai no DESENHO pela porta da
/// precedência, que é o canal de compatibilidade de sempre. Recusar aqui deixaria o botão sem
/// glifo nenhum num documento que outro build escreveu.
/// **Os rótulos dos filhos de um controle de LISTA** — a mesma lei de posse do `ui_panel_spec`.
///
/// ⚠️ Ela é perguntada à ÁRVORE (o `Children` do ECS), e não à cena: quem contém quem é fato de
/// hierarquia, e é o mesmo lugar de onde o painel gerado tira a lista. Duas derivações dariam um
/// canvas e um painel que discordam sobre quantas abas o controle tem.
fn child_labels(sim: &SimWorld, map: &VecEntityMap, id: VecPathId) -> Vec<String> {
    let Some(&bits) = map.get(&id) else {
        return Vec::new();
    };
    let w = sim.world();
    w.get::<ph2d_ecs::Children>(Entity::from_bits(bits))
        .map(|c| {
            c.iter()
                .filter_map(|&k| w.get::<ph2d_ecs::Name>(k).map(|n| n.0.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

fn chosen_icon(sim: &SimWorld, map: &VecEntityMap, id: VecPathId) -> Option<IconId> {
    let &bits = map.get(&id)?;
    let c = sim.world().get::<VecWidgetIcon>(Entity::from_bits(bits))?;
    IconId::from_slug(&c.slug)
}

/// O rótulo que o widget mostra: o `Name` da entidade, ou vazio.
fn label_of(sim: &SimWorld, map: &VecEntityMap, id: VecPathId) -> String {
    let Some(&bits) = map.get(&id) else {
        return String::new();
    };
    sim.world()
        .get::<Name>(Entity::from_bits(bits))
        .map(|n| n.0.clone())
        .unwrap_or_default()
}

/// **A MOLDURA de uma forma vestida, em pixels de tela** — a porta única do *ONDE*.
///
/// ⚠️ Ela existe separada do [`build`] por causa de um gate que nasceu FRACO: o de comportamento
/// compara a pele de duas poses e exige que difiram, e uma ponte que pintasse na origem ERRADA
/// com um tamanho certo passava por ele — *diferir* e *pousar no lugar* não são a mesma pergunta.
/// Com a moldura nomeada, o gate a compara com o `path_screen_bounds` termo a termo.
///
/// `None` quando o caminho não desenha nada, ou quando a moldura é degenerada: um controle de
/// largura zero desenharia uma listra que o artista não consegue agarrar de volta.
#[must_use]
pub(crate) fn frame_of(
    scene: &VecScene,
    xforms: &VecXforms,
    live: &LiveGeometry,
    id: VecPathId,
    camera: Affine,
) -> Option<Rect> {
    let (x0, y0, x1, y1) = ph2d_vec_render::path_screen_bounds(scene, xforms, live, id, camera)?;
    let (w, h) = ((x1 - x0) as f32, (y1 - y0) as f32);
    (w > 1.0 && h > 1.0).then(|| Rect::new(x0 as f32, y0 as f32, w, h))
}

/// **A cor que uma swatch mostra** — o preenchimento da forma que a veste.
///
/// ⚠️ Um gradiente responde pela **primeira parada** (`primary_color`), que é a porta que o
/// documento já oferece *"pra swatch de UI"* — inventar aqui uma média ou uma amostra do meio
/// seria uma segunda resposta a *"que cor é esta pintura?"*.
fn colour_of(fill: &Paint) -> [u8; 4] {
    let c = fill.primary_color();
    [c.r, c.g, c.b, c.a]
}

/// Pinta as peles deste frame. Vazio = nenhuma forma veste um widget, e o `dispatch` desenha o
/// que sempre desenhou.
///
/// ⚠️ Chamado DEPOIS do `sync` (senão uma forma recém-marcada ainda não tem entidade e o
/// componente dela não seria encontrado) — a mesma ordem que o `offset_live::recook` exige.
#[allow(clippy::too_many_arguments)]
pub(crate) fn build(
    scene: &VecScene,
    sim: &SimWorld,
    map: &VecEntityMap,
    xforms: &VecXforms,
    live: &LiveGeometry,
    camera: Affine,
    text: &mut TextSystem,
    theme: Theme,
) -> WidgetSkins {
    let mut out = WidgetSkins::new();
    for path in scene.paths() {
        let Some(spec) = spec_of(sim, map, path.id) else {
            continue;
        };
        // Código que este build não conhece: a forma desenha como sempre. É o caminho de
        // compatibilidade, e por isso ele é um `continue` e não um retângulo vazio.
        let Some(kind) = WidgetKind::from_code(spec.kind) else {
            continue;
        };
        let Some(rect) = frame_of(scene, xforms, live, path.id, camera) else {
            continue;
        };
        let mut skin = ph2d_vector::VectorScene::new();
        let label = label_of(sim, map, path.id);
        // ⚠️ **A cor é VIVA aqui, e é um SNAPSHOT no painel gerado** — a diferença é honesta: no
        // canvas a moldura relê o documento a cada quadro, então recolorir a forma recolore a
        // swatch enquanto o artista arrasta; o painel gerado é código escrito uma vez, e mostra a
        // cor que a forma tinha quando alguém apertou o botão.
        // ⚠️ **O glifo vive no ESCOPO do laço, e tem de viver.** `IconGlyph::Path` empresta o
        // `BezPath`, então normalizá-lo dentro do construtor do `SkinParam` deixaria o temporário
        // morrer antes da chamada. Uma `Option` nomeada é o que dá ao empréstimo um dono.
        // ⚠️ **As opções vivem no escopo do laço pelo mesmo motivo do glifo:** a `SkinParam`
        // EMPRESTA a fatia, então construí-las dentro do construtor deixaria o temporário morrer
        // antes da chamada.
        let options: Vec<String> = if kind.takes_options() {
            child_labels(sim, map, path.id)
        } else {
            Vec::new()
        };
        let face = kind.takes_icon().then(|| icon_face(path)).flatten();
        let chosen = kind
            .takes_icon()
            .then(|| chosen_icon(sim, map, path.id))
            .flatten();
        let param = SkinParam {
            rgba: kind
                .takes_colour()
                .then(|| path.fill.as_ref().map(colour_of))
                .flatten(),
            // ⚠️ A precedência **não é decidida aqui**: ela mora na porta única do catálogo, que
            // o painel gerado percorre também. Um `if` local seria a terceira cópia dela.
            icon: icon_glyph(chosen, face.as_ref()),
            options: &options,
            // ⚠️ A prévia do canvas mostra sempre a PRIMEIRA marcada, e é a lei do `PREVIEW_VALUE`
            // da pele: um controle de opções sem nenhuma acesa lê-se como quebrado, não como
            // vazio. O valor VIVO é do painel, que tem o store; o canvas é prévia.
            selected: 0,
        };
        paint_widget_skin(kind, &label, param, rect, &mut skin, text, theme);
        out.insert(path.id, skin);
    }
    out
}

#[cfg(test)]
#[path = "widget_live_tests.rs"]
mod tests;
