//! Gates dos **VARIANTS** (plano UI/UX W5c).
//!
//! ⚠️ O keystone é o último: escolher um chip tem de **religar a instância e mudar o DESENHO**.
//! Os gates de projeção provam que as fileiras descrevem o catálogo; só o do desenho prova que o
//! gesto leva a algum lado, que é a quarta condição da política de UI — e ela não é implicada
//! pelas outras três.

use super::*;
use ph2d_ecs::{Name, VecInstance};
use ph2d_vec_scene::{Paint, Rgba8, VecScene, rectangle};

use crate::vec_entities::VecEntityMap;

/// **Um pai com `n` mestres irmãos**, nomeados por `names`, mais uma instância do primeiro.
///
/// ⚠️ O pai é uma forma comum e NÃO um mestre: um conjunto de variants é *os mestres irmãos*, e
/// se o pai também se declarasse mestre a fixture responderia a duas perguntas ao mesmo tempo.
fn variant_set(names: &[&str]) -> (SimWorld, VecScene, VecEntityMap, Vec<VecPathId>, VecPathId) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let parent = scene.push_path(rectangle([-1.0, -1.0], [0.0, 0.0]));
    let mut kids = Vec::new();
    for (i, name) in names.iter().enumerate() {
        // Larguras DIFERENTES por versão: é o que faz o gate do desenho poder distinguir uma da
        // outra sem depender de cor. Uma fixture de versões idênticas seria verde por vácuo.
        let w = 10.0 + (i as f64) * 6.0;
        let mut p = rectangle([0.0, 0.0], [w, 10.0]);
        p.fill = Some(Paint::Solid(Rgba8::new(200, 40, 40, 255)));
        kids.push((scene.push_path(p), (*name).to_string()));
    }
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let pe = Entity::from_bits(map[&parent]);
    let mut paths = Vec::new();
    for (path, name) in &kids {
        let e = Entity::from_bits(map[path]);
        crate::vec_transform::reparent_keeping_world(&mut sim, e, pe);
        sim.world_mut()
            .entity_mut(e)
            .insert(ph2d_ecs::VecComponentMain)
            .insert(Name(name.clone()));
        paths.push(*path);
    }
    let main = paths[0];
    let at = crate::vec_component_edit::place_instance(&sim, &mut scene, &map, &[main])
        .expect("Place recusou");
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    crate::vec_component_edit::arm_instance(&mut sim, &map, at, main, [40.0, 0.0]);
    (sim, scene, map, paths, at)
}

/// As fileiras que o painel pintaria para uma instância que deriva de `main`.
fn rows(sim: &SimWorld, map: &VecEntityMap, main: VecPathId) -> VariantRows {
    rows_and_targets(sim, map, main).0
}

// ── O PARSE ──────────────────────────────────────────────────────────────────────────────

/// **`Size=Small, State=Idle` é uma combinação; `Botão` não é.**
#[test]
fn a_name_is_a_combination_only_when_every_part_is_a_pair() {
    assert_eq!(
        parse_combo("Size=Small, State=Idle"),
        Some(vec![
            ("Size".into(), "Small".into()),
            ("State".into(), "Idle".into()),
        ])
    );
    assert_eq!(parse_combo("Botao"), None, "sem `=` nao e' combinacao");
    assert_eq!(parse_combo("Size="), None, "valor vazio");
    assert_eq!(parse_combo("=Small"), None, "propriedade vazia");
    assert_eq!(parse_combo("Size=Small=Big"), None, "dois `=` numa parte");
    // ⚠️ O espaço é aparado: `Size=Small, State=Idle` e `Size=Small,State=Idle` TÊM de dar a
    // mesma combinação, senão dois irmãos escritos por mãos diferentes deixariam de casar.
    assert_eq!(parse_combo("a=b,c=d"), parse_combo(" a = b , c = d "));
}

// ── O CONJUNTO ───────────────────────────────────────────────────────────────────────────

/// **Um mestre SOZINHO não é um conjunto** — o controle.
///
/// Sem ele, todo gate abaixo poderia estar a medir uma fileira que aparece sempre.
#[test]
fn a_lone_main_offers_no_variant_rows() {
    let (sim, _sc, map, paths, _at) = variant_set(&["Botao"]);
    assert!(
        rows(&sim, &map, paths[0]).axes.is_empty(),
        "um mestre sem irmaos ofereceu uma fileira que nao escolhe nada"
    );
}

/// **Um variant é um IRMÃO, nunca um FILHO** — e um mestre-raiz não tem variants.
///
/// ⚠️ **A primeira versão deste gate era verde por vácuo, e a mutação apanhou-a:** ela soltava um
/// mestre-FOLHA do pai, e uma folha não tem `Children` — então *"o pai é exigido"* e *"o pai é eu
/// mesmo"* davam a mesma lista vazia. A fixture tem de conter o fenômeno: um mestre-raiz **com
/// dois mestres dentro** é um componente aninhado, e as peças dele não são versões dele.
#[test]
fn a_variant_is_a_sibling_never_a_child() {
    let (mut sim, mut scene, mut map, paths, _at) = variant_set(&["A", "B", "C"]);
    let (root, k1, k2) = (
        Entity::from_bits(map[&paths[0]]),
        Entity::from_bits(map[&paths[1]]),
        Entity::from_bits(map[&paths[2]]),
    );
    sim.world_mut()
        .entity_mut(root)
        .remove::<ph2d_ecs::ChildOf>();
    crate::vec_transform::reparent_keeping_world(&mut sim, k1, root);
    crate::vec_transform::reparent_keeping_world(&mut sim, k2, root);
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    assert!(
        rows(&sim, &map, paths[0]).axes.is_empty(),
        "um mestre solto na raiz recebeu os PRÓPRIOS filhos como versões de si mesmo"
    );
    // E os dois filhos SÃO variants um do outro — o controle que torna o acima não-vazio.
    assert_eq!(rows(&sim, &map, paths[1]).axes.len(), 1);
}

/// **Os irmãos MESTRES são os variants, e a fileira acende o vigente.**
#[test]
fn the_sibling_mains_are_the_variants() {
    let (sim, _sc, map, paths, _at) = variant_set(&["Small", "Medium", "Large"]);
    let r = rows(&sim, &map, paths[1]);
    assert_eq!(r.axes.len(), 1, "nomes crus dao UMA fileira");
    assert_eq!(r.axes[0].values, vec!["Small", "Medium", "Large"]);
    assert_eq!(r.axes[0].selected, 1, "a fileira nao acendeu o vigente");
}

/// **Um irmão que NÃO é mestre não é um variant.**
///
/// ⚠️ Sem isto, qualquer forma largada dentro do grupo de versões viraria um chip — e clicar nele
/// religaria a instância a algo que o produtor recusa desenhar (ele exige o marcador).
#[test]
fn a_sibling_that_is_not_a_main_is_not_a_variant() {
    let (mut sim, mut scene, mut map, paths, _at) = variant_set(&["Small", "Medium"]);
    let e = Entity::from_bits(map[&paths[1]]);
    sim.world_mut()
        .entity_mut(e)
        .remove::<ph2d_ecs::VecComponentMain>();
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    assert!(
        rows(&sim, &map, paths[0]).axes.is_empty(),
        "uma forma comum ao lado do mestre virou variant"
    );
}

// ── OS EIXOS ─────────────────────────────────────────────────────────────────────────────

/// **Os nomes viram PROPRIEDADES**, e cada uma ganha a sua fileira.
#[test]
fn the_names_become_property_axes() {
    let (sim, _sc, map, paths, _at) = variant_set(&[
        "Size=Small, State=Idle",
        "Size=Large, State=Idle",
        "Size=Small, State=Hover",
        "Size=Large, State=Hover",
    ]);
    let r = rows(&sim, &map, paths[0]);
    assert_eq!(r.axes.len(), 2, "a matriz cheia tem de dar DOIS eixos");
    assert_eq!(r.axes[0].name, "Size");
    assert_eq!(r.axes[0].values, vec!["Small", "Large"]);
    assert_eq!(r.axes[1].name, "State");
    assert_eq!(r.axes[1].values, vec!["Idle", "Hover"]);
    assert_eq!((r.axes[0].selected, r.axes[1].selected), (0, 0));
    // ⚠️ **E a metade que faltava, que uma mutação nomeou:** medido só do PRIMEIRO variant,
    // `selected = 0` está certo por acidente em todo eixo — a fixture não continha o fenômeno.
    // Do ÚLTIMO (`Large, Hover`) os dois eixos têm de acender o segundo chip.
    let r = rows(&sim, &map, paths[3]);
    assert_eq!(
        (r.axes[0].selected, r.axes[1].selected),
        (1, 1),
        "a fileira nao acendeu a combinacao vigente"
    );
}

/// **Um nome que não parseia derruba TODOS para o modo de nomes crus.**
///
/// ⚠️ E não para uma interseção: com `{Size}` num irmão e `{Size,State}` noutro, esconder o
/// `State` faria o artista perder um eixo sem nada a dizer porquê. Nos nomes crus tudo aparece.
#[test]
fn a_name_that_does_not_parse_falls_back_to_raw_names() {
    let (sim, _sc, map, paths, _at) = variant_set(&["Size=Small", "Size=Large", "O especial"]);
    let r = rows(&sim, &map, paths[0]);
    assert_eq!(r.axes.len(), 1);
    assert!(r.axes[0].name.is_empty(), "o modo cru nao nomeia o eixo");
    assert_eq!(
        r.axes[0].values,
        vec!["Size=Small", "Size=Large", "O especial"]
    );
}

/// **Um valor que não leva a lado nenhum NÃO é oferecido** — o buraco da matriz.
///
/// De `Size=Small, State=Idle` alcança-se `Large` (existe `Large, Idle`) mas **não** `Hover`, que
/// só existe em `Large`. Um chip `Hover` ali seria clicável e não faria nada.
#[test]
fn a_value_that_leads_nowhere_is_not_offered() {
    let (sim, _sc, map, paths, _at) = variant_set(&[
        "Size=Small, State=Idle",
        "Size=Large, State=Idle",
        "Size=Large, State=Hover",
    ]);
    let r = rows(&sim, &map, paths[0]);
    let state = r
        .axes
        .iter()
        .find(|a| a.name == "State")
        .map(|a| a.values.clone());
    assert!(
        state.is_none(),
        "o eixo State so' tem um valor alcancavel daqui e mesmo assim foi oferecido: {state:?}"
    );
    // E a partir de `Large` ele existe, com os dois — o controle que torna o acima não-vazio.
    let r = rows(&sim, &map, paths[1]);
    let state = r
        .axes
        .iter()
        .find(|a| a.name == "State")
        .expect("de Large, o eixo State tem de existir");
    assert_eq!(state.values, vec!["Idle", "Hover"]);
}

// ── O TETO ───────────────────────────────────────────────────────────────────────────────

/// **O excedente é CONTADO, nunca truncado em silêncio.**
#[test]
fn the_options_beyond_the_id_table_are_counted() {
    let names: Vec<String> = (0..ph2d_editor::ids::MAX_VARIANT_VALUES + 3)
        .map(|i| format!("V{i}"))
        .collect();
    let refs: Vec<&str> = names.iter().map(String::as_str).collect();
    let (sim, _sc, map, paths, _at) = variant_set(&refs);
    let r = rows(&sim, &map, paths[0]);
    assert_eq!(r.axes[0].values.len(), ph2d_editor::ids::MAX_VARIANT_VALUES);
    assert_eq!(r.beyond, 3, "as tres versoes a mais nao foram contadas");
}

// ── O KEYSTONE ───────────────────────────────────────────────────────────────────────────

/// **Escolher um chip religa a instância e MUDA O DESENHO.**
///
/// ⚠️ O oráculo é a largura do que a cópia desenha, e não o campo `main`: um `main` novo com o
/// desenho igual seria a feature partida com todos os gates de projeção verdes. As três versões da
/// fixture têm larguras diferentes justamente para este gate poder distingui-las.
#[test]
fn picking_a_value_relinks_the_instance_and_changes_what_it_draws() {
    let (mut sim, mut scene, map, paths, at) = variant_set(&["Small", "Medium", "Large"]);
    let before = drawn_width(&sim, &scene, &map, at);
    let e = Entity::from_bits(map[&at]);
    // As DUAS chamadas que o handler faz, na mesma ordem.
    let target = target_of(&sim, &map, paths[0], 0, 2).expect("o chip Large nao enderecou nada");
    assert_eq!(target, paths[2]);
    crate::vec_component_pieces::swap_main(&mut sim, &scene, &map, e, target)
        .expect("swap recusou");
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map.clone());
    let after = drawn_width(&sim, &scene, &map, at);
    assert!(
        after > before + 1.0,
        "escolher Large nao mudou o desenho da copia: {before} -> {after}"
    );
    let main = sim
        .world()
        .get::<VecInstance>(e)
        .expect("a instancia sumiu")
        .main;
    assert_eq!(main, paths[2], "o vinculo nao foi religado");
    // E a fileira passa a acender o novo — sem isto o chip aceso mentiria depois do clique.
    assert_eq!(rows(&sim, &map, main).axes[0].selected, 2);
}

/// **A SONDA: o que o painel mostraria para a cena de smoke `=58`.**
///
/// ⚠️ Ela roda os NOMES da cena real pelo motor real e imprime as fileiras — é o que torna o
/// roteiro do smoke uma afirmação MEDIDA em vez de uma promessa. Rode com
/// `cargo test -p ph2d-host-desktop measure_the_smoke_rows -- --ignored --nocapture`.
#[test]
#[ignore = "sonda: imprime as fileiras da cena =58"]
fn measure_the_smoke_rows() {
    let names = [
        "Size=Small, State=Idle",
        "Size=Large, State=Idle",
        "Size=Small, State=Hover",
        "Size=Large, State=Hover",
    ];
    let (sim, _sc, map, paths, _at) = variant_set(&names);
    for (i, name) in names.iter().enumerate() {
        let r = rows(&sim, &map, paths[i]);
        let shown: Vec<String> = r
            .axes
            .iter()
            .map(|a| {
                let opts: Vec<String> = a
                    .values
                    .iter()
                    .enumerate()
                    .map(|(v, s)| {
                        if v == a.selected {
                            format!("[{s}]")
                        } else {
                            s.clone()
                        }
                    })
                    .collect();
                format!("{}: {}", a.name, opts.join(" "))
            })
            .collect();
        eprintln!("  de `{name}` -> {}", shown.join("  |  "));
    }
    // E o solitário, a metade da AUSÊNCIA do roteiro.
    let (sim, _sc, map, lone, _at) = variant_set(&["Solo"]);
    eprintln!(
        "  um mestre SEM irmaos -> {} fileira(s)",
        rows(&sim, &map, lone[0]).axes.len()
    );
}

/// **O id de um chip vira um VERBO na shell.**
///
/// ⚠️ Este gate nasceu de uma mutação SOBREVIVENTE, e o buraco que ela nomeou é a quarta condição
/// da política de UI — a que **não é implicada** pelas outras três. Com o roteador cego ao id, o
/// chip é pintado, vive sob o mouse, o Click atravessa o barramento **e não vira nada**: os doze
/// gates de projeção e os quatro de seam ficavam todos VERDES.
#[test]
fn a_variant_chip_id_becomes_a_verb() {
    use crate::vec_component_edit::{ComponentEdit, component_edit_for_id};
    for axis in 0..ph2d_editor::ids::MAX_VARIANT_AXES {
        for value in 0..ph2d_editor::ids::MAX_VARIANT_VALUES {
            let id = ph2d_editor::ids::vector_variant_option_id(axis, value);
            assert_eq!(
                component_edit_for_id(id),
                Some(ComponentEdit::Variant(axis, value)),
                "o chip ({axis},{value}) nao vira verbo — o clique chega a' shell e morre"
            );
        }
    }
    // ⚠️ E os verbos VIZINHOS não são engolidos: a varredura da tabela roda depois das peças,
    // e um id de peça que caísse nela viraria um variant em silêncio.
    assert_eq!(
        component_edit_for_id(ph2d_editor::ids::vector_instance_piece_show_id(0)),
        Some(ComponentEdit::PieceVisible(0))
    );
    assert_eq!(
        component_edit_for_id(ph2d_editor::ids::VECTOR_COMPONENT_SWAP),
        Some(ComponentEdit::Swap)
    );
}

/// A largura do que a instância `at` desenha.
fn drawn_width(sim: &SimWorld, scene: &VecScene, map: &VecEntityMap, at: VecPathId) -> f64 {
    let xf: ph2d_vec_scene::VecXforms = crate::vec_transform::build(sim, map);
    let inst = sim
        .world()
        .get::<VecInstance>(Entity::from_bits(map[&at]))
        .expect("a instancia sumiu")
        .clone();
    let (paths, _) = crate::instance_live::cook_one(scene, sim, map, &xf, at, &inst)
        .expect("a copia nao cozinhou");
    let (mut lo, mut hi) = (f64::MAX, f64::MIN);
    for p in &paths {
        for v in p.verts_all() {
            lo = lo.min(v.anchor[0]);
            hi = hi.max(v.anchor[0]);
        }
    }
    hi - lo
}
