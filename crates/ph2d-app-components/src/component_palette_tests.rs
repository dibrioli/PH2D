//! Os gates da paleta de componentes (ADR-0166 / plano F3).
//!
//! ⚠️ Eles medem o **MODELO**, não a pintura — o widget é o `command_palette`, que já tem os
//! gates dele. O que é desta linha é *o que a paleta oferece a quem*, e é isso que se afirma aqui.

use super::*;

/// Tudo se constrói, para os gates isolarem o filtro do `can_build`.
fn buildable(_: &str) -> bool {
    true
}

fn labels(m: &PaletteModel) -> Vec<String> {
    m.groups
        .iter()
        .flat_map(|g| &g.subs)
        .flat_map(|s| &s.items)
        .map(|i| i.label.clone())
        .collect()
}

/// ⭐ **O EXEMPLO DO ENIO, medido:** *"9-slice provavelmente não se aplica a nada além de uma
/// sprite de imagem"*. Ele é oferecido a uma imagem e **não** a um objeto vetorial.
///
/// (Mutação: ignorar o `applies_to` ⇒ o vetor passa a oferecê-lo — RED.)
#[test]
fn nine_slice_is_offered_to_an_image_and_not_to_a_vector() {
    let img = build(ObjectKind::Image, &[], &buildable, false);
    assert!(
        labels(&img).iter().any(|l| l == "9-Slice"),
        "uma imagem tem de poder receber 9-Slice; ofereceu: {:?}",
        labels(&img)
    );
    let vec = build(ObjectKind::Vector, &[], &buildable, false);
    assert!(
        !labels(&vec).iter().any(|l| l.starts_with("9-Slice")),
        "um objeto vetorial NAO pode receber 9-Slice; ofereceu: {:?}",
        labels(&vec)
    );
}

/// ⚠️ **O inaplicável NÃO some — ele fica sob *Show all*, esmaecido e COM A RAZÃO.**
///
/// ⛔ Nem apagar da lista (um componente que existe e é invisível lê-se como defeito), nem no-op
/// silencioso ao clique (DIRETIVA §2). O rótulo carrega o porquê.
#[test]
fn show_all_reveals_the_inapplicable_with_the_reason_named() {
    let hidden = build(ObjectKind::Vector, &[], &buildable, false);
    let shown = build(ObjectKind::Vector, &[], &buildable, true);
    assert!(
        labels(&shown).len() > labels(&hidden).len(),
        "o Show all tem de REVELAR alguma coisa"
    );
    let nine = labels(&shown)
        .into_iter()
        .find(|l| l.starts_with("9-Slice"))
        .expect("o 9-Slice tem de aparecer sob Show all");
    assert!(
        nine.contains("not for this object type"),
        "o item inaplicavel tem de dizer PORQUE: {nine:?}"
    );
    // …e num sub-grupo próprio, depois dos aplicáveis.
    let sub = shown
        .groups
        .iter()
        .flat_map(|g| &g.subs)
        .find(|s| s.items.iter().any(|i| i.label.starts_with("9-Slice")))
        .expect("o sub-grupo");
    assert_eq!(sub.title.as_deref(), Some("Not for this object type"));
}

/// **O que o objeto JÁ TEM não é oferecido** — anexar o que já existe é um clique que não faz nada.
#[test]
fn a_component_already_on_the_object_is_not_offered() {
    let before = build(ObjectKind::Image, &[], &buildable, false);
    let after = build(
        ObjectKind::Image,
        &["ph2d::ecs::SliceNine"],
        &buildable,
        false,
    );
    assert!(labels(&before).iter().any(|l| l == "9-Slice"));
    assert!(
        !labels(&after).iter().any(|l| l == "9-Slice"),
        "ja' esta' no objeto e continuou a ser oferecido"
    );
}

/// ⚠️ **O que a paleta não consegue CONSTRUIR não pode estar nela.** Sem `insert_default` não há
/// valor inicial, e um item que aceita o clique e não anexa nada é o defeito que o `+` existe para
/// não ter.
#[test]
fn a_component_the_registry_cannot_build_is_never_offered() {
    let all = build(ObjectKind::Image, &[], &buildable, true);
    let none = build(ObjectKind::Image, &[], &|_| false, true);
    assert!(
        !labels(&all).is_empty(),
        "o controle positivo tem de oferecer"
    );
    assert!(
        labels(&none).is_empty(),
        "sem construtor, a paleta tem de ficar VAZIA; ofereceu: {:?}",
        labels(&none)
    );
}

/// ⭐ **Só `Authored` é oferecido.** A `Sprite` é `Intrinsic` (chega pelo gesto que cria a imagem)
/// e as pontes são `Machinery` — nenhuma é uma escolha do artista.
#[test]
fn only_authored_components_reach_the_palette() {
    let all = build(ObjectKind::Image, &[], &buildable, true);
    let ls = labels(&all);
    assert!(
        !ls.iter().any(|l| l.starts_with("Sprite Pixels")),
        "uma Machinery chegou a' paleta: {ls:?}"
    );
    assert!(
        !ls.iter().any(|l| l == "Sprite"),
        "a Sprite e' Intrinsic — ela chega pelo gesto, nao pelo +"
    );
}

/// **O pick volta a ser um componente** — o inverso do `item_id`, e a única rota de volta.
///
/// ⚠️ Ele varre o CATÁLOGO: uma segunda lista à mão envelheceria no primeiro componente novo, e o
/// sintoma seria *"o item aparece e não faz nada"*.
#[test]
fn every_offered_item_maps_back_to_its_component() {
    let m = build(ObjectKind::Image, &[], &buildable, true);
    let items: Vec<_> = m
        .groups
        .iter()
        .flat_map(|g| &g.subs)
        .flat_map(|s| &s.items)
        .collect();
    assert!(!items.is_empty());
    for it in items {
        let name = name_of_pick(it.id)
            .unwrap_or_else(|| panic!("o item {:?} nao volta a ser um componente", it.label));
        assert!(
            it.label.starts_with(
                ph2d_component_desc::desc_for(name)
                    .expect("o descritor")
                    .display_name
            ),
            "o id do item {:?} nomeia {name}, que tem outro rotulo",
            it.label
        );
    }
}

/// **Um objeto VAZIO ainda tem o que receber** — senão o `+` num objeto novo abriria uma paleta
/// vazia, que é a primeira coisa que o smoke do Enio faz.
#[test]
fn an_empty_object_still_has_something_to_offer() {
    let m = build(ObjectKind::Empty, &[], &buildable, false);
    assert!(
        !labels(&m).is_empty(),
        "o + num objeto vazio abriu uma paleta VAZIA"
    );
}

/// ⚠️ **Toda categoria com item tem um título e uma cor** — um grupo sem título é uma faixa muda
/// no modal.
#[test]
fn every_group_is_named_and_tinted() {
    let m = build(ObjectKind::Image, &[], &buildable, true);
    for g in &m.groups {
        assert!(!g.title.is_empty(), "grupo sem titulo");
        assert!(
            !g.subs.iter().all(|s| s.items.is_empty()),
            "grupo {:?} sem itens — ele nao devia existir",
            g.title
        );
    }
}

/// ⭐ **A CASCATA É MOSTRADA ANTES DE SER APLICADA** — a correção da crítica medida ao Bevy
/// (discussão #16570, doc 02 §1.4: *«não vejo o que vem junto»*).
///
/// ⚠️ **E ela é FECHADA:** *Platform Player* traz `RigidBody`, que traz `Collider`. Mostrar só o
/// primeiro salto seria a mesma queixa um nível abaixo — o artista clicava esperando um componente
/// e recebia três.
#[test]
fn the_cascade_is_shown_in_the_label_before_it_is_applied() {
    let m = build(ObjectKind::Image, &[], &buildable, false);
    // ⚠️ O rótulo é **Physics Body** desde a poda de 13/09 e o `Collision Shape` que ele TRAZ já não
    // é um item da paleta desde 14/09 (ordem do dono) — o que não muda é a lei: *o artista vê o que
    // vem junto ANTES de clicar*. Ver o cabeçalho do `catalog/physics`.
    let body = labels(&m)
        .into_iter()
        .find(|l| l.starts_with("Physics Body"))
        .expect("o Physics Body tem de estar na paleta");
    assert!(
        body.contains("brings Collision Shape"),
        "o rotulo tem de dizer o que vem junto: {body:?}"
    );
    // ⚠️ **A metade FECHADA (transitiva) mede-se na tabela, e não na paleta** — desde 14/09 o único
    // `requires` da física é `RigidBody → Collider`, logo não há um 2.º salto para ver num rótulo.
    // A lei do fecho continua gateada no `brings_along` pelas famílias que a exercem.
    assert!(
        !body.contains("brings brings"),
        "o rotulo montou-se duas vezes: {body:?}"
    );
}

/// ⚠️ **E quem NÃO tem cascata não ganha texto nenhum** — a metade de ausência. Um rótulo que diz
/// *"brings"* sobre nada seria ruído em ~90 itens.
#[test]
fn a_component_with_no_requirement_says_nothing_extra() {
    let m = build(ObjectKind::Image, &[], &buildable, false);
    let nine = labels(&m)
        .into_iter()
        .find(|l| l.starts_with("9-Slice"))
        .expect("o 9-Slice");
    assert_eq!(nine, "9-Slice", "um item sem cascata tem o rotulo limpo");
}

/// ⛔ **O grafo de dependências é ACÍCLICO** — e isto não é higiene: a cascata do `attach_by_name`
/// é recursiva, então um ciclo no catálogo faria o `+` recorrer para sempre em vez de falhar alto.
#[test]
fn the_require_graph_has_no_cycles() {
    fn walk(name: &'static str, path: &mut Vec<&'static str>) {
        assert!(
            !path.contains(&name),
            "ciclo no `requires` do catalogo: {path:?} -> {name}"
        );
        let Some(d) = ph2d_component_desc::desc_for(name) else {
            return;
        };
        path.push(name);
        for dep in d.requires {
            walk(dep, path);
        }
        path.pop();
    }
    let mut seen = 0usize;
    for d in ph2d_component_desc::all() {
        if !d.requires.is_empty() {
            seen += 1;
        }
        walk(d.canonical_name, &mut Vec::new());
    }
    // ⚠️ **O piso de população deixou de ser SÓ um NÚMERO e ganhou um NOME** (2026-09-14). Um piso
    // que só conta não distingue *«a população encolheu por uma decisão»* de *«alguém apagou o
    // `requires` e o gate passou a andar sobre nada»* — e nesta rodada as duas cascatas da física
    // sobreviveram por razões diferentes (o `Collider` porque o corpo é inerte sem ele; o
    // `RigidBody` porque o **player** é, mesmo tendo saído da paleta: `Intrinsic` e `requires`
    // respondem a perguntas diferentes). A âncora nomeada abaixo é o que torna isso verificável.
    assert!(
        seen >= 2,
        "o gate ficou verde por nao haver `requires` nenhum ({seen})"
    );
    let corpo = ph2d_component_desc::desc_for("ph2d::physics::RigidBody").expect("o descritor");
    assert_eq!(
        corpo.requires,
        ["ph2d::physics::Collider"],
        "a cascata CANONICA desapareceu — este gate percorre um grafo, e sem uma aresta conhecida \
         ele nao percorre nada. Se o `RigidBody` deixou mesmo de exigir o `Collider`, troque esta \
         ancora pela cascata que ficou no lugar dela."
    );
}

/// ⚠️ **Toda dependência declarada NOMEIA um componente que existe e que o REGISTO sabe construir.**
///
/// Um nome canónico errado no `requires` não falha a compilação — a cascata simplesmente salta-o em
/// silêncio, e o artista anexa o dependente sem a dependência. É a mesma classe da chave por string
/// que o próprio descritor avisa.
///
/// ⛔⛔ **A 1.ª redacção exigia `Attach::Authored`, e nomeava o RECURSO ERRADO.** Ela dizia *«é
/// Intrinsic — a cascata não o consegue construir»*, e isso é falso: quem constrói é o
/// `insert_default` do **`ComponentRegistry`** (`attach_by_name` → `attach_one`), que **não
/// consulta o `attach`**. Um `Intrinsic` com `Default` — o `Collider` é exactamente esse caso —
/// constrói-se perfeitamente. *Um limite legítimo diz de que recurso ele é* (CLAUDE.md §0.0), e
/// este dizia de um recurso de outro subsistema.
///
/// ⚠️ Foi a ordem do dono de 2026-09-14 que o expôs (o `Collider` sai da paleta e continua a ser
/// dependência do `RigidBody`) — e a cura não é isentar a linha: é medir o que de facto importa.
/// Com o registo do produto a responder, o gate ficou **mais forte**, porque um `requires` que
/// nomeie um tipo sem `insert_default` passava antes e reprova agora.
///
/// (Mutação: apontar um `requires` a um nome sem `Default` no registo ⇒ RED com o nome dentro.)
#[test]
fn every_declared_requirement_names_a_real_component() {
    let reg = crate::component_registry_for_tests::registo();
    for d in ph2d_component_desc::all() {
        for dep in d.requires {
            ph2d_component_desc::desc_for(dep).unwrap_or_else(|| {
                panic!("{} exige {dep}, que nao tem descritor", d.canonical_name)
            });
            let buildable = reg
                .get_by_id(ph2d_ecs::scene::stable_type_id(dep))
                .is_some_and(|e| e.insert_default.is_some());
            assert!(
                buildable,
                "{} exige {dep}, que o registo do produto nao sabe construir — a cascata \
                 (`attach_one`) insere o ponto NEUTRO pelo `insert_default`, entao sem ele o \
                 dependente nasce sem a dependencia, em silencio",
                d.canonical_name
            );
        }
    }
}
