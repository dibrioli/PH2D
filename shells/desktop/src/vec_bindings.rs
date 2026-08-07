//! **O RESOLVEDOR de tokens** — o `ph2d_ecs::VecBindings` de cada forma, resolvido contra o modo
//! vigente, na forma que o renderer consome.
//!
//! É a metade que só a shell pode fazer: o documento não conhece tema, e a crate de desenho não
//! conhece o ECS onde os bindings moram.
//!
//! ⚠️ **Ele NÃO entra no [`crate::vec_entities::view_state`]**, e a razão é custo: aquela porta é
//! chamada por todo caminho de HIT-TEST e gesto (o pick, o marquee, a linha de corte), e nenhum
//! deles pergunta de que cor a forma é. Resolver token ali seria trabalho de desenho pago por quem
//! só quer geometria. Quem publica é o passe de desenho, uma vez por frame.

use ph2d_ecs::{BoundProp, Entity, SimWorld, VecBindings};
use ph2d_tokens::{ColorToken, NumToken, Theme};
use ph2d_vec_scene::{BoundStyle, Rgba8};

use crate::vec_entities::VecEntityMap;

/// A cor concreta de um token neste modo.
///
/// ⚠️ **A porta ÚNICA de *"que cor tem este token?"***, e ela tem três consumidores: este
/// resolvedor (para DESENHAR), o painel (para a swatch MOSTRAR) e os gates. Uma segunda tabela em
/// qualquer um deles é a swatch que mostra uma cor e a arte que desenha outra — divergência que só
/// aparece num screenshot.
///
/// Chave desconhecida devolve `None` em vez de uma cor de emergência: um token que sumiu da tabela
/// tem de deixar o LITERAL valer (a arte volta ao que o artista escreveu), e nunca pintar de rosa
/// choque uma forma que estava certa.
#[must_use]
pub(crate) fn token_color(key: &str, theme: Theme) -> Option<Rgba8> {
    let c = ColorToken::from_key(key)?.resolve(theme);
    Some(Rgba8::new(c.r, c.g, c.b, c.a))
}

/// **O que é preciso para resolver um token neste frame**: o MODO, e a RÉGUA.
///
/// ⚠️ Um par nomeado, e não dois escalares soltos, porque ele atravessa a shell inteira — o passe
/// de desenho e o passe de AUTO LAYOUT resolvem tokens em sítios diferentes, e dois `f32`/`Theme`
/// soltos numa lista de argumentos são o par que alguém troca de ordem sem o compilador reclamar.
///
/// ⚠️ **A cor não usa a régua**, e é por isso que [`token_color`] continua a tomar só o [`Theme`]:
/// uma cor é adimensional. Dar-lhe a régua sugeriria que ela tem uma.
#[derive(Clone, Copy, Debug)]
pub(crate) struct TokenCtx {
    /// O modo vigente — a metade que já existia.
    pub theme: Theme,
    /// `ProjectSettings::pixels_per_meter` — ver [`token_world`].
    pub pixels_per_meter: f32,
}

impl TokenCtx {
    /// **O modo e a régua de FÁBRICA** — para fixtures que não autoram nenhum dos dois.
    ///
    /// ⚠️ Um construtor NOMEADO, e não um `Default`: um `TokenCtx::default()` alcançável do
    /// caminho de produto seria a porta por onde alguém resolveria um comprimento com a régua
    /// errada sem o compilador dizer nada, e o sintoma — um traço com a espessura de outro
    /// projeto — não se vê em teste nenhum.
    #[cfg(test)]
    pub(crate) fn factory() -> Self {
        Self {
            theme: Theme::default(),
            pixels_per_meter: ph2d_editor::project::DEFAULT_PIXELS_PER_METER,
        }
    }
}

/// **O comprimento de MUNDO que um token de escala vale** — a irmã numérica do [`token_color`],
/// e a porta ÚNICA da fronteira px↔mundo desta feature (W4c.4).
///
/// # ⚠️ A régua não é escolhida aqui: ela já tem dono
///
/// `ProjectSettings::pixels_per_meter` é a única px↔mundo do projeto (ADR-0131 D4 — *"um 2º
/// `PIXELS_PER_METER` seria a segunda porta que diverge"*), e é ela. Com o default de 100,
/// `stroke.default = 1.5 px` vale **0,015** unidades — 1,58 pt numa moldura de telefone, que mede
/// 8 unidades no lado maior.
///
/// ⚠️ **NÃO é o `px_to_world` da câmera**, embora a row *Width* do painel fale nele: aquele número
/// é px de TELA no zoom do momento, então resolver por ele faria o traço mudar de espessura quando
/// o artista se aproximasse — e o valor SALVO dependeria de onde ele estava olhando.
///
/// Chave desconhecida devolve `None` pelo mesmo motivo que a cor: o LITERAL do documento tem de
/// valer, e nunca um comprimento de emergência.
#[must_use]
pub(crate) fn token_world(key: &str, tok: TokenCtx) -> Option<f64> {
    let px = f64::from(NumToken::from_key(key)?.px(tok.theme));
    let ppm = f64::from(tok.pixels_per_meter);
    // O `set_pixels_per_meter` clampa em `MIN_PIXELS_PER_METER = 1.0`, mas o campo é público: uma
    // escrita direta de zero daria `inf`, e uma espessura infinita pinta a tela inteira.
    (ppm > 0.0).then_some(px / ppm)
}

/// **O vão que o auto layout desta moldura tem NESTE frame** — `[principal, transversal]`, e
/// `None` num eixo = o número autorado em `VecLayout::gap` vale.
///
/// ⚠️ Ele lê o mesmo `VecBindings` que a tinta, pela mesma porta de conversão. Um segundo caminho
/// para *"quanto vale este token em mundo?"* dentro do passe de layout divergiria da largura do
/// traço no dia em que a régua mudasse — e o sintoma seria uma moldura a espaçar por uma régua e
/// um traço a engrossar por outra.
#[must_use]
pub(crate) fn bound_gap(sim: &SimWorld, frame: Entity, tok: TokenCtx) -> [Option<f64>; 2] {
    let Some(b) = sim.world().get::<VecBindings>(frame) else {
        return [None, None];
    };
    [
        b.get(BoundProp::LayoutGapMain)
            .and_then(|k| token_world(k, tok)),
        b.get(BoundProp::LayoutGapCross)
            .and_then(|k| token_world(k, tok)),
    ]
}

/// As tintas que os tokens produzem neste frame, para o `VecViewState`.
///
/// Vazio quando nada está bindado — que é todo documento que já existe, e é o que faz o desenho
/// deles ficar byte-idêntico ao mundo pré-token.
#[must_use]
pub(crate) fn resolve(sim: &SimWorld, map: &VecEntityMap, tok: TokenCtx) -> Vec<BoundStyle> {
    let theme = tok.theme;
    let w = sim.world();
    let mut out = Vec::new();
    for (&id, &bits) in map {
        let e = Entity::from_bits(bits);
        if w.get_entity(e).is_err() {
            continue;
        }
        let Some(b) = w.get::<VecBindings>(e) else {
            continue;
        };
        let paint = BoundStyle {
            path: id,
            fill: b.get(BoundProp::Fill).and_then(|k| token_color(k, theme)),
            stroke: b
                .get(BoundProp::StrokeColor)
                .and_then(|k| token_color(k, theme)),
            // A opacidade VIVA não vem de token nenhum — quem a produz é uma row autorada, e ela
            // é fundida nesta mesma entrada pelo `vec_widget_drive::apply` (W8b.3).
            alpha: None,
            width: b
                .get(BoundProp::StrokeWidth)
                .and_then(|k| token_world(k, tok)),
        };
        // Uma entrada que não resolveu nada não descreve desenho nenhum — publicá-la faria o
        // renderer perguntar por ela em toda forma da cena sem nunca ter o que responder.
        if !paint.is_noop() {
            out.push(paint);
        }
    }
    out
}

/// **Que escolha do picker este id é** — a propriedade, e o token (ou `None` = SOLTAR).
///
/// ⚠️ Ele ENUMERA os ids gerados em vez de os inverter, porque um `NodeId` é um hash e um hash não
/// se inverte. É o mesmo desenho do `frames::device_preset`, e o custo é 162 comparações num
/// clique — não num frame.
#[must_use]
pub(crate) fn token_choice(id: ph2d_editor::NodeId) -> Option<(BoundProp, Option<&'static str>)> {
    for slot in ph2d_editor::ids::TOKEN_SLOTS {
        // ⚠️ O alvo vem do CÓDIGO da tabela pela porta do modelo, e não de um `match` escrito aqui:
        // um slot novo nasce ligado, em vez de virar uma linha de picker que não faz nada.
        let prop = BoundProp::from_code(slot.code)?;
        if id == ph2d_editor::ids::vector_token_option_id(slot.code, 0) {
            return Some((prop, None));
        }
        for i in 0..slot.table.len() {
            if id == ph2d_editor::ids::vector_token_option_id(slot.code, i + 1) {
                return Some((prop, slot.table.key(i)));
            }
        }
    }
    None
}

/// Prende (ou solta) a propriedade nas formas SELECIONADAS.
///
/// ⚠️ **Desanexa o componente quando fica vazio.** Um `VecBindings` sem entradas viaja no save e
/// entra no diff do undo, e então duas cenas logicamente iguais comparam diferente — o passo
/// espúrio que o `canonicalize` do undo global existe para matar.
pub(crate) fn set_selected_binding(
    sim: &mut SimWorld,
    map: &VecEntityMap,
    selected: &[ph2d_vec_scene::VecPathId],
    prop: BoundProp,
    token: Option<&str>,
) {
    for id in selected {
        let Some(&bits) = map.get(id) else { continue };
        let e = Entity::from_bits(bits);
        if sim.world().get_entity(e).is_err() {
            continue;
        }
        let mut b = sim
            .world()
            .get::<VecBindings>(e)
            .cloned()
            .unwrap_or_default();
        match token {
            Some(t) => b.set(prop, t),
            None => b.clear(prop),
        }
        if b.is_empty() {
            sim.world_mut().entity_mut(e).remove::<VecBindings>();
        } else {
            sim.world_mut().entity_mut(e).insert(b);
        }
    }
}

/// O que o painel mostra nas rows de token: o que a seleção tem preso, e se ela tem traço.
///
/// `None` quando a seleção não é uma forma única — as rows não são pintadas, porque prender um
/// token a *"várias formas ao mesmo tempo"* precisa de uma resposta a *"e se elas discordarem?"*
/// que esta wave não dá.
#[must_use]
pub(crate) fn selected_bindings(
    sim: &SimWorld,
    scene: &ph2d_vec_scene::VecScene,
    map: &VecEntityMap,
    selected: &[ph2d_vec_scene::VecPathId],
) -> Option<ph2d_panel_vector::state::TokenBindings> {
    let [id] = selected else { return None };
    let &bits = map.get(id)?;
    let e = Entity::from_bits(bits);
    if sim.world().get_entity(e).is_err() {
        return None;
    }
    let b = sim.world().get::<VecBindings>(e);
    let key = |p: BoundProp| b.and_then(|b| b.get(p)).map(str::to_owned);
    Some(ph2d_panel_vector::state::TokenBindings {
        fill: key(BoundProp::Fill),
        stroke: key(BoundProp::StrokeColor),
        width: key(BoundProp::StrokeWidth),
        gap_main: key(BoundProp::LayoutGapMain),
        gap_cross: key(BoundProp::LayoutGapCross),
        stroke_exists: scene.path(*id).is_some_and(|p| p.stroke.is_some()),
        // ⚠️ O gêmeo do `stroke_exists` para os vãos: sem `VecLayout` a moldura não empilha, e um
        // token de vão preso ali seria uma escolha que não move um pixel.
        flows: sim.world().get::<ph2d_ecs::VecLayout>(e).is_some(),
    })
}

thread_local! {
    /// **Que propriedades foram AUTORADAS neste frame**, como máscara sobre o discriminante do
    /// [`BoundProp`] — cada uma tem de soltar o token dela.
    ///
    /// ⚠️ Uma máscara, e não um par de `bool`: ela cresce com a lista de alvos sem que o canal
    /// mude de forma, e foi o par `(fill, stroke)` que teria de virar tripla na W4c.4 — a lista
    /// paralela que envelhece assim que um alvo novo entra num lado só.
    static AUTHORED: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

/// A ponte drena os one-shots do tool aqui; o passe de aplicação os consome.
///
/// ⚠️ Um canal interno da shell, e não um argumento a mais: quem SABE que um valor foi autorado é
/// o tool (a ponte é quem fala com ele), e quem pode SOLTAR o token é o passe que tem o mundo e a
/// seleção na mão. Os dois correm no mesmo frame, em ordem — a ponte primeiro.
pub(crate) fn note_authored(prop: BoundProp) {
    AUTHORED.with(|c| c.set(c.get() | 1 << (prop as u16)));
}

/// **Autorar um valor SOLTA o token daquela propriedade** — o *detach* do Figma.
///
/// ⚠️ Sem isto o artista escolheria uma cor (ou digitaria uma espessura), o token continuaria a
/// cobri-la, e o controlo mostraria um valor que a arte não usa: o pior estado possível
/// (decisão do Enio, 2026-08-02). E é por isso que os one-shots do tool armam só quando o valor
/// MUDA — o read-back do picker corre em todo frame em que ele está aberto.
pub(crate) fn detach_on_authored(
    sim: &mut SimWorld,
    map: &VecEntityMap,
    selected: &[ph2d_vec_scene::VecPathId],
) {
    let mask = AUTHORED.with(std::cell::Cell::take);
    for &prop in BoundProp::ALL {
        if mask & (1 << (prop as u16)) != 0 {
            set_selected_binding(sim, map, selected, prop, None);
        }
    }
}

#[cfg(test)]
#[path = "vec_bindings_tests.rs"]
mod tests;
