//! Os gates do resolvedor de tokens.

use super::*;
use crate::vec_entities::{VecEntityMap, sync};
use ph2d_vec_scene::{VecPathId, VecScene, rectangle};

/// O modo pedido, com a régua de FÁBRICA — o que quase toda fixture quer.
fn ctx(theme: Theme) -> TokenCtx {
    TokenCtx {
        theme,
        ..TokenCtx::factory()
    }
}

/// Uma cena com uma forma, e o mapa `VecPathId → entidade`.
fn scene() -> (SimWorld, VecScene, VecEntityMap, VecPathId) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let id = scene.push_path(rectangle([0.0, 0.0], [2.0, 1.0]));
    sync(&mut sim, &mut scene, &mut map);
    (sim, scene, map, id)
}

fn bind(sim: &mut SimWorld, map: &VecEntityMap, id: VecPathId, prop: BoundProp, token: &str) {
    let e = Entity::from_bits(map[&id]);
    let mut b = sim
        .world()
        .get::<VecBindings>(e)
        .cloned()
        .unwrap_or_default();
    b.set(prop, token);
    sim.world_mut().entity_mut(e).insert(b);
}

/// **Sem binding, a tabela sai VAZIA** — e é isso que faz todo documento que já existe desenhar
/// byte-idêntico ao mundo pré-token.
#[test]
fn a_document_with_no_bindings_publishes_nothing() {
    let (sim, _scene, map, _id) = scene();
    assert!(resolve(&sim, &map, ctx(Theme::Forge)).is_empty());
}

/// **O MODO decide a cor.** É a entrega inteira da wave: a mesma arte, dois modos, duas cores — e
/// sem tocar no documento.
#[test]
fn the_same_binding_resolves_to_a_different_colour_in_another_mode() {
    let (mut sim, _scene, map, id) = scene();
    bind(&mut sim, &map, id, BoundProp::Fill, "accent");

    let forge = resolve(&sim, &map, ctx(Theme::Forge));
    let sunstone = resolve(&sim, &map, ctx(Theme::Sunstone));
    assert_eq!(forge.len(), 1);
    assert_eq!(sunstone.len(), 1);
    assert_eq!(forge[0].path, id);
    assert_ne!(
        forge[0].fill, sunstone[0].fill,
        "trocar de modo tem de re-vestir a arte; iguais, o binding nao serve para nada"
    );
    assert_eq!(
        forge[0].fill,
        token_color("accent", Theme::Forge),
        "a cor vem da porta unica, nao de uma segunda tabela"
    );
}

/// **Um token que não existe deixa o LITERAL valer.**
///
/// A alternativa (uma cor de emergência) pintaria de errado uma forma que estava certa, e o
/// artista veria a arte mudar sem ter mexido nela.
#[test]
fn an_unknown_token_falls_back_to_the_literal() {
    let (mut sim, _scene, map, id) = scene();
    bind(&mut sim, &map, id, BoundProp::Fill, "no-such-token");
    assert!(token_color("no-such-token", Theme::Forge).is_none());
    assert!(
        resolve(&sim, &map, ctx(Theme::Forge)).is_empty(),
        "nada resolvido = nada publicado = o desenho usa o literal do documento"
    );
}

/// As duas propriedades chegam juntas na mesma entrada.
#[test]
fn fill_and_stroke_ride_the_same_entry() {
    let (mut sim, _scene, map, id) = scene();
    bind(&mut sim, &map, id, BoundProp::Fill, "accent");
    bind(&mut sim, &map, id, BoundProp::StrokeColor, "border");
    let out = resolve(&sim, &map, ctx(Theme::Forge));
    assert_eq!(out.len(), 1, "uma forma, uma entrada");
    assert_eq!(out[0].fill, token_color("accent", Theme::Forge));
    assert_eq!(out[0].stroke, token_color("border", Theme::Forge));
}

/// ⚠️ **A chave do documento tem de casar com a chave que o token EMITE.**
///
/// O gate percorre a lista inteira e afirma o round-trip. Sem ele, um token cuja chave o
/// `from_key` não reconhecesse ficaria para sempre no fallback: o artista o escolheria no picker e
/// a arte não mudaria, em silêncio.
#[test]
fn every_token_the_picker_offers_resolves_by_its_own_key() {
    for &t in ColorToken::ALL {
        assert_eq!(
            ColorToken::from_key(t.key()),
            Some(t),
            "token {:?} nao volta pela propria chave",
            t
        );
        assert!(token_color(t.key(), Theme::Forge).is_some());
    }
    assert!(
        ColorToken::ALL.len() >= 60,
        "a lista encolheu — o picker perdeu tokens"
    );
}

/// **A SEQUÊNCIA leva a algum lugar** — a 4ª condição de UI, e a que nenhuma das outras três
/// implica: todo edit pode ter gate, todo widget pode estar registado e clicável, e o gesto ainda
/// não chegar a lado nenhum.
///
/// A corrente inteira: **o id da opção → a propriedade + o token → o componente no ECS → a tinta
/// resolvida → o `VecPath` que o renderer recebe** — e trocar de modo re-veste.
#[test]
fn the_whole_chain_from_the_click_to_the_drawn_paint() {
    use ph2d_vec_scene::{Paint, VecViewState};

    let (mut sim, mut scene, map, id) = scene();
    let literal = ph2d_vec_scene::Rgba8::new(9, 9, 9, 255);
    if let Some(p) = scene.path_mut(id) {
        p.fill = Some(Paint::Solid(literal));
    }

    // 1. O id que o picker pinta para "accent" — decodificado pela porta do PRODUTO.
    let row = 1 + ColorToken::ALL
        .iter()
        .position(|t| t.key() == "accent")
        .expect("o token accent existe na tabela");
    let opt = ph2d_editor::ids::vector_token_option_id(0, row);
    let (prop, token) = token_choice(opt).expect("o id e' uma escolha do picker");
    assert_eq!(prop, BoundProp::Fill);
    assert_eq!(token, Some("accent"));

    // 2. A shell escreve no ECS.
    set_selected_binding(&mut sim, &map, &[id], prop, token);

    // 3+4. O resolvedor produz a tinta, o desenho a usa — e ela DIFERE entre dois modos.
    let mut view = VecViewState::default();
    for theme in [Theme::Forge, Theme::Sunstone] {
        view.bound = resolve(&sim, &map, ctx(theme));
        let path = scene.path(id).expect("a forma existe");
        assert_eq!(
            path.painted(view.bound_style(id)).fill,
            token_color("accent", theme).map(Paint::Solid),
            "o desenho tem de mostrar o token resolvido no modo {theme:?}"
        );
        assert_ne!(
            path.painted(view.bound_style(id)).fill,
            Some(Paint::Solid(literal)),
            "e nao o literal"
        );
    }

    // 5. Soltar devolve o LITERAL — bindar nunca o apagou.
    set_selected_binding(&mut sim, &map, &[id], prop, None);
    view.bound = resolve(&sim, &map, ctx(Theme::Forge));
    let path = scene.path(id).expect("a forma existe");
    assert_eq!(
        path.painted(view.bound_style(id)).fill,
        Some(Paint::Solid(literal)),
        "soltar tem de devolver a cor que o artista escreveu"
    );
}

/// **O componente DESANEXA quando fica vazio.**
///
/// Um `VecBindings` sem entradas viaja no save e entra no diff do undo — e então duas cenas
/// logicamente iguais comparam diferente, que é o passo espúrio que o `canonicalize` do undo
/// global existe para matar.
#[test]
fn unbinding_the_last_property_detaches_the_component() {
    let (mut sim, _scene, map, id) = scene();
    set_selected_binding(&mut sim, &map, &[id], BoundProp::Fill, Some("accent"));
    let e = Entity::from_bits(map[&id]);
    assert!(sim.world().get::<VecBindings>(e).is_some());
    set_selected_binding(&mut sim, &map, &[id], BoundProp::Fill, None);
    assert!(
        sim.world().get::<VecBindings>(e).is_none(),
        "um componente VAZIO ficou anexado — ele entra no diff do undo e no save"
    );
}

/// **Escolher uma cor SOLTA o token daquela propriedade** — o *detach* do Figma (Enio,
/// 2026-08-02: *"ao selecionar uma nova cor no Fill/Stroke, o Token deve voltar para None"*).
///
/// ⚠️ **E o CONTROLE é a metade que importa:** o read-back do picker corre em TODO frame em que
/// ele está aberto, então um flag armado incondicionalmente soltaria o token no instante em que o
/// picker abrisse — antes de o artista tocar em coisa nenhuma. É por isso que o `colour_authored`
/// do tool arma só quando o valor MUDA, e é isso que a primeira metade deste gate mede.
#[test]
fn authoring_a_colour_detaches_that_token_and_only_that_one() {
    use ph2d_tool_vector::VectorTool;

    // CONTROLE: re-escrever a MESMA cor não é autoria — o picker aberto não solta nada.
    let mut t = VectorTool::default();
    let same = t.ui_snapshot().fill;
    let _ = t.take_colour_authored();
    t.set_fill_rgba(same);
    assert_eq!(
        t.take_colour_authored(),
        (false, false),
        "o picker apenas ABERTO nao pode soltar o token — ele re-escreve a mesma cor todo frame"
    );

    // E uma cor DIFERENTE é autoria, só do lado que mudou.
    t.set_fill_rgba([1, 2, 3, 255]);
    assert_eq!(t.take_colour_authored(), (true, false));
    t.set_stroke_rgba([9, 9, 9, 255]);
    assert_eq!(t.take_colour_authored(), (false, true));

    // A ponta da corrente: o one-shot solta a propriedade autorada e DEIXA a outra.
    let (mut sim, _scene, map, id) = scene();
    set_selected_binding(&mut sim, &map, &[id], BoundProp::Fill, Some("accent"));
    set_selected_binding(
        &mut sim,
        &map,
        &[id],
        BoundProp::StrokeColor,
        Some("border"),
    );
    note_authored(BoundProp::Fill);
    detach_on_authored(&mut sim, &map, &[id]);
    let e = Entity::from_bits(map[&id]);
    let b = sim
        .world()
        .get::<VecBindings>(e)
        .expect("o traco sobrevive");
    assert_eq!(
        b.get(BoundProp::Fill),
        None,
        "a cor autorada soltou o token"
    );
    assert_eq!(
        b.get(BoundProp::StrokeColor),
        Some("border"),
        "e NAO soltou a outra propriedade"
    );
}

// ─────────────────────────── W4c.4 — OS TOKENS DE ESCALA ───────────────────────────

/// ⚠️ **A RÉGUA — e este é o gate central da wave.**
///
/// Um token de escala fala PIXELS; o documento fala MUNDO. O oráculo **não** é a fórmula (isso
/// seria o espelho da função sob teste): é o ABSURDO que a conversão evita, medido contra o
/// tamanho de uma moldura de telefone.
///
/// Sem régua, `stroke.default = 1.5` viraria 1,5 unidades num aparelho que mede
/// `frames::LONG_SIDE = 8` no lado maior — **19% da altura da tela, como espessura de traço**. Com
/// a régua de fábrica (100 px/unidade) ele vale 0,015, que é o cabelo que o token promete.
#[test]
fn a_length_token_crosses_into_world_by_the_projects_ruler() {
    let tok = ctx(Theme::Forge);
    let w = token_world("stroke.default", tok).expect("o token existe");

    let px = f64::from(
        ph2d_tokens::NumToken::from_key("stroke.default")
            .unwrap()
            .px(Theme::Forge),
    );
    assert!(
        (w - px / f64::from(ph2d_editor::project::DEFAULT_PIXELS_PER_METER)).abs() < 1e-9,
        "a conversao tem de ser a regua do projeto"
    );

    // O ABSURDO que ela evita, no tamanho REAL de uma moldura.
    let phone = ph2d_tool_vector::frames::LONG_SIDE;
    assert!(
        px / phone > 0.15,
        "premissa do gate: sem regua o numero cru seria uma fracao GRANDE da moldura ({:.0}%)",
        px / phone * 100.0
    );
    assert!(
        w / phone < 0.01,
        "com a regua o traco tem de ser um cabelo, e nao {:.0}% da moldura",
        w / phone * 100.0
    );
}

/// **Uma régua diferente dá um comprimento diferente** — é o que faz dela uma régua e não uma
/// constante disfarçada.
#[test]
fn the_ruler_is_read_from_the_project_not_baked_in() {
    let a = token_world(
        "spacing.md",
        TokenCtx {
            pixels_per_meter: 100.0,
            ..TokenCtx::factory()
        },
    );
    let b = token_world(
        "spacing.md",
        TokenCtx {
            pixels_per_meter: 200.0,
            ..TokenCtx::factory()
        },
    );
    assert!(a.is_some() && b.is_some());
    assert!(
        (a.unwrap() - b.unwrap() * 2.0).abs() < 1e-9,
        "dobrar a regua tem de METADE o comprimento"
    );

    // ⚠️ Uma régua ZERO não devolve infinito: o campo é público, e uma espessura infinita pinta a
    // tela inteira.
    assert_eq!(
        token_world(
            "spacing.md",
            TokenCtx {
                pixels_per_meter: 0.0,
                ..TokenCtx::factory()
            }
        ),
        None
    );
}

/// **Uma chave de COR não é um comprimento, e uma de comprimento não é uma cor.**
///
/// ⚠️ As duas famílias partilham o slot do arquivo (a lista `tokens`), então este gate é o que
/// impede um `"accent"` preso a uma espessura de resolver para um número qualquer.
#[test]
fn the_two_families_do_not_answer_for_each_other() {
    let tok = ctx(Theme::Forge);
    assert!(
        token_world("accent", tok).is_none(),
        "cor nao tem comprimento"
    );
    assert!(
        token_color("spacing.md", Theme::Forge).is_none(),
        "comprimento nao tem cor"
    );
}

/// **A ESPESSURA atravessa a corrente inteira** — do clique do picker ao `VecPath` desenhado.
///
/// ⚠️ E o CONTROLE é a segunda metade: uma forma **sem traço** não recebe espessura nenhuma. Um
/// token de largura ali teria de inventar a COR, que é a mesma metade-que-falta da cor do traço.
#[test]
fn a_width_token_reaches_the_drawn_stroke_and_never_invents_one() {
    use ph2d_vec_scene::VecViewState;

    let (mut sim, mut scene, map, id) = scene();
    let tok = ctx(Theme::Forge);
    let expected = token_world("stroke.heavy", tok).expect("o token existe");

    // CONTROLE: sem traço, nada resolve — a entrada nem é publicada.
    set_selected_binding(
        &mut sim,
        &map,
        &[id],
        BoundProp::StrokeWidth,
        Some("stroke.heavy"),
    );
    let mut view = VecViewState {
        bound: resolve(&sim, &map, tok),
        ..VecViewState::default()
    };
    let path = scene.path(id).expect("a forma existe");
    assert!(
        path.stroke.is_none(),
        "premissa: a forma da fixture nasce sem traco"
    );
    assert!(
        matches!(
            path.painted(view.bound_style(id)),
            std::borrow::Cow::Borrowed(_)
        ),
        "sem traco a largura nao tem o que engrossar — e nao pode nem clonar"
    );

    // Com traço, ela chega ao desenho.
    if let Some(p) = scene.path_mut(id) {
        p.stroke = Some(ph2d_vec_scene::StrokeSpec::new(
            ph2d_vec_scene::Rgba8::new(0, 0, 0, 255),
            0.5,
        ));
    }
    view.bound = resolve(&sim, &map, tok);
    let path = scene.path(id).expect("a forma existe");
    let drawn = path.painted(view.bound_style(id));
    assert!(
        drawn
            .stroke
            .is_some_and(|s| (s.width - expected).abs() < 1e-9),
        "o traco desenhado tem de ter a espessura do token"
    );
    assert!(
        path.stroke.is_some_and(|s| (s.width - 0.5).abs() < 1e-9),
        "e o DOCUMENTO continua com a largura que o artista escreveu"
    );
}

/// **O picker numérico chega ao componente** — a 4ª condição de UI, para os slots novos.
///
/// ⚠️ Ela não é implicada pelas outras três: o chip pode estar pintado, registado e clicável, e o
/// id da opção ainda não decodificar para alvo nenhum. Era exactamente esse o estado antes desta
/// wave, porque o `token_choice` enumerava `[(Fill, 0), (StrokeColor, 1)]` à mão.
#[test]
fn every_token_slot_decodes_a_click_into_its_own_target() {
    for slot in ph2d_editor::ids::TOKEN_SLOTS {
        let prop = BoundProp::from_code(slot.code).expect("todo slot nomeia um alvo do modelo");

        let unbind = ph2d_editor::ids::vector_token_option_id(slot.code, 0);
        assert_eq!(
            token_choice(unbind),
            Some((prop, None)),
            "a linha de SOLTAR"
        );

        // A PRIMEIRA e a ÚLTIMA linha da tabela deste slot — as duas pontas do intervalo.
        for i in [0, slot.table.len() - 1] {
            let id = ph2d_editor::ids::vector_token_option_id(slot.code, i + 1);
            assert_eq!(
                token_choice(id),
                Some((prop, slot.table.key(i))),
                "a linha {i} do slot {} tem de virar o par (alvo, chave)",
                slot.code
            );
        }
    }
}

/// **Cada slot lista a tabela da UNIDADE dele.** Cores para uma cor, comprimentos para um
/// comprimento — e o oráculo é a CHAVE que o clique produz, não a etiqueta do slot.
///
/// ⚠️ Sem isto o picker da espessura ofereceria `"accent"`, o artista o escolheria, e o
/// `token_world` devolveria `None` — uma escolha que não faz nada, em silêncio.
#[test]
fn every_token_slot_paints_its_own_table() {
    let tok = ctx(Theme::Forge);
    for slot in ph2d_editor::ids::TOKEN_SLOTS {
        let first = ph2d_editor::ids::vector_token_option_id(slot.code, 1);
        let (_, key) = token_choice(first).expect("a 1a linha e' uma escolha");
        let key = key.expect("e nao a de SOLTAR");
        match slot.table {
            ph2d_editor::ids::TokenTable::Colour => assert!(
                token_color(key, Theme::Forge).is_some(),
                "o slot {} lista cor, mas '{key}' nao resolve como cor",
                slot.code
            ),
            ph2d_editor::ids::TokenTable::Length => assert!(
                token_world(key, tok).is_some(),
                "o slot {} lista comprimento, mas '{key}' nao resolve como comprimento",
                slot.code
            ),
        }
    }
}

/// **Digitar uma ESPESSURA solta o token dela** — arch-gate, porque a decisão mora no bridge.
///
/// ⚠️ O one-shot em si é testável (`note_authored` + `detach_on_authored` estão logo acima), mas
/// quem decide ARMÁ-LO é o `vector_bridge::dispatch`, que exige janela e tool: nenhum teste de
/// unidade o alcança. Sem esta afirmação, apagar a linha do produto deixa a suíte inteira VERDE
/// com o token a cobrir uma largura que o artista acabou de escrever.
///
/// ⚠️ A âncora é a ADJACÊNCIA de linha (*a nota é a primeira linha da guarda*), e não uma distância
/// em bytes — o proxy que expira assim que alguém põe uma linha no meio.
#[test]
fn authoring_a_width_arms_the_detach_in_the_bridge() {
    const BRIDGE: &str = include_str!("render_loop/vector_bridge.rs");
    let lines: Vec<&str> = BRIDGE.lines().collect();
    let n = lines
        .iter()
        .position(|l| l.contains("note_authored(ph2d_ecs::BoundProp::StrokeWidth)"))
        .expect(
            "o bridge nao arma o detach da espessura — digitar uma largura deixa o token a \
             cobri-la, e o campo mostra um valor que a arte nao usa",
        );
    let prev = lines[..n]
        .iter()
        .rev()
        .find(|l| !l.trim().is_empty())
        .copied()
        .unwrap_or("");
    assert!(
        prev.contains("width_authored"),
        "a nota nao esta' atras da pergunta *a largura MUDOU?* — armada sempre, ela soltaria o \
         token no frame em que o painel abrisse"
    );
}

/// **O VÃO resolvido chega ao passe de layout**, e por eixo.
#[test]
fn a_gap_token_resolves_per_axis_and_leaves_the_other_alone() {
    let (mut sim, _scene, map, id) = scene();
    let tok = ctx(Theme::Forge);
    let e = Entity::from_bits(map[&id]);
    sim.world_mut().entity_mut(e).insert(ph2d_ecs::VecLayout {
        gap: [7.0, 9.0],
        ..Default::default()
    });

    assert_eq!(
        bound_gap(&sim, e, tok),
        [None, None],
        "sem binding os dois eixos usam o numero autorado"
    );

    set_selected_binding(
        &mut sim,
        &map,
        &[id],
        BoundProp::LayoutGapMain,
        Some("spacing.lg"),
    );
    let g = bound_gap(&sim, e, tok);
    assert_eq!(g[0], token_world("spacing.lg", tok));
    assert_eq!(g[1], None, "o eixo transversal continua com o literal");
}
