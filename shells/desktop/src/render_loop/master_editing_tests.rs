//! Os gates do MODO de edição de receita (F4.6).
//!
//! ⚠️ **O oráculo é `is_off_canvas`**, e não a marca: um gate sobre a marca mede o mecanismo que eu
//! escolhi, e não o fim que a frase promete (*«a receita aparece enquanto se mexe nela»*) — a lição
//! de 26/08.

use super::mark;
use ph2d_ecs::{ChildOf, Entity, MasterRoot, Name, SimWorld, Transform, Visibility};
use ph2d_entity_visibility::off_canvas::is_off_canvas;

/// Uma receita de duas peças, e uma entidade solta que nunca participa.
fn scene() -> (SimWorld, Entity, Entity, Entity) {
    let mut sim = SimWorld::new();
    let root = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Badge"), MasterRoot))
        .id();
    let piece = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Box"), ChildOf(root)))
        .id();
    let loose = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Loose")))
        .id();
    ph2d_ecs::assign_master_pieces(sim.world_mut());
    (sim, root, piece, loose)
}

/// ⭐⭐⭐ **A receita sai da cena, e VOLTA enquanto está selecionada.**
///
/// As duas verdades que se contradiziam: esconder sempre torna a forma do mestre impossível de
/// mudar; desenhar sempre põe dois objetos empilhados.
///
/// (Mutação: `is_off_canvas` ignorar o `MasterEditing` ⇒ RED na 2.ª metade.)
#[test]
fn the_recipe_comes_back_while_it_is_being_edited() {
    let (mut sim, root, piece, _) = scene();
    mark(&mut sim, None::<u64>, &mut None);
    assert!(
        is_off_canvas(sim.world(), root) && is_off_canvas(sim.world(), piece),
        "a receita esta' na cena sem ninguem a editar — dois objetos empilhados"
    );
    // O gesto: escolher uma PEÇA dela na Hierarquia.
    mark(&mut sim, Some(piece.to_bits()), &mut None);
    assert!(
        !is_off_canvas(sim.world(), root) && !is_off_canvas(sim.world(), piece),
        "a receita nao voltou ao escolher uma peca dela — a forma do mestre fica inalcancavel"
    );
}

/// ⚠️⚠️ **As DUAS metades do passe** — marcar sem desmarcar deixa a receita visível para sempre
/// depois de o artista mudar de selecção.
///
/// (Mutação: apagar o laço do `difference` inverso ⇒ RED.)
#[test]
fn changing_the_selection_puts_the_recipe_back_out_of_the_scene() {
    let (mut sim, root, piece, loose) = scene();
    mark(&mut sim, Some(root.to_bits()), &mut None);
    assert!(!is_off_canvas(sim.world(), piece));
    mark(&mut sim, Some(loose.to_bits()), &mut None);
    assert!(
        is_off_canvas(sim.world(), root) && is_off_canvas(sim.world(), piece),
        "a receita ficou visivel depois de o artista mudar de selecao"
    );
}

/// ⭐⭐⭐ **Uma receita seleccionada com Shift também acende** — auditoria §1.6.
///
/// O passe lia só o **primário**, e duas rotas correntes deixam a receita seleccionada sem o ser:
/// `add_to_selection` (Shift/Ctrl-clique) e o atalho `preserves_multi` do ramo `Replace`. A linha
/// ficava realçada na Hierarquia e o canvas continuava vazio.
///
/// ⚠️ **O caso mede as DUAS coisas ao mesmo tempo** — a receita acende **e** a outra coisa
/// seleccionada continua na cena —, senão um `mark` que ignorasse o primário passaria na metade
/// que interessa.
///
/// (Mutação que o mata: `.take(1)` **logo a seguir** ao `selection.into_iter()` — o `Option<u64>`
/// de antes. ⚠️ **A 1.ª mutação que tentei SOBREVIVEU, e ela é que estava errada:** pôr o `take(1)`
/// *depois* do `filter_map(master_root_of)` é no-op, porque o objeto solto já tinha sido descartado
/// ali por não ser receita. *Uma mutação a jusante do filtro não mede o que o filtro recebeu.*
/// O chamador tem arch-gate próprio, `the_recipe_mark_is_fed_the_extra_selection_too`.)
#[test]
fn a_recipe_selected_as_an_extra_lights_up_too() {
    let (mut sim, root, piece, loose) = scene();
    // O gesto: clicar no objeto solto e depois Shift-clicar a linha da receita.
    mark(&mut sim, [loose.to_bits(), root.to_bits()], &mut None);
    assert!(
        !is_off_canvas(sim.world(), root) && !is_off_canvas(sim.world(), piece),
        "a receita ficou escondida por nao ser a selecao PRIMARIA — e a linha dela esta' realcada"
    );
    assert!(
        !is_off_canvas(sim.world(), loose),
        "acender a receita apagou o outro selecionado"
    );
    // E as duas metades continuam a valer com N: largar tudo apaga.
    mark(&mut sim, None::<u64>, &mut None);
    assert!(
        is_off_canvas(sim.world(), root),
        "a receita ficou acesa depois de a selecao esvaziar"
    );
}

/// ⭐⭐⭐ **A CENA DO SMOKE, no quadro 0: a receita não tem um pixel** — auditoria §1.7.
///
/// ⛔ Este é o gate que faltava, e a sua ausência custou o report inteiro. Os textos das duas cenas
/// diziam *«receita 'Ragdoll' lá em cima»* e *«receita 'Badge' à ESQUERDA, longe das cópias»*, com
/// as coordenadas escolhidas precisamente para ela ficar visível — e desde a F4.6 aquelas
/// coordenadas têm **zero pixels**, porque nada está seleccionado no arranque. *O instrumento que
/// existe para dar o meio-caminho entregava exactamente o report que ele existe para evitar.*
///
/// ⚠️ **Nenhum gate atravessava a cena do smoke.** Os que existem medem os INGREDIENTES (que
/// `VecPathId` cada peça recebeu) numa fixtura de duas entidades feita à mão; nenhum perguntava
/// *«quantas peças desta cena desenham no quadro 0?»*. É a mesma distância entre a marca e o fim
/// que os outros sete achados têm.
///
/// (Mutação: `want.insert(root)` em vez de `want.extend(subtree(..))` ⇒ RED na 2.ª metade — o
/// PASSO 1 do smoke acenderia meia receita. ⚠️ A mutação que apaga o laço de **desmarcar**
/// sobrevive aqui de propósito: neste gate o `mark(None)` corre primeiro num mundo onde nada está
/// marcado, e quem a mata é `changing_the_selection_puts_the_recipe_back_out_of_the_scene`.)
#[test]
fn the_smoke_scene_shows_its_recipe_only_after_the_row_is_clicked() {
    let mut sim = SimWorld::new();
    let r = crate::init::build_component_registry();
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    let (master, _roots) = crate::instance_smoke::spawn_ragdoll_scene(
        &mut sim,
        &r,
        &mut crate::instance_docs::OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
    );
    let recipe: Vec<Entity> = {
        let mut out = vec![master];
        let mut i = 0;
        while i < out.len() {
            if let Some(kids) = sim.world().get::<ph2d_ecs::Children>(out[i]) {
                let kids: Vec<Entity> = kids.iter().copied().collect();
                out.extend(kids);
            }
            i += 1;
        }
        out
    };
    assert!(recipe.len() > 1, "a receita do smoke nao tem pecas");

    // O quadro 0 do smoke: ninguém escolheu nada.
    mark(&mut sim, None::<u64>, &mut None);
    for &e in &recipe {
        assert!(
            is_off_canvas(sim.world(), e),
            "uma peca da receita desenha no arranque — o texto do smoke e o canvas concordam \
             outra vez, mas por acidente"
        );
    }
    // O PASSO 1 que o texto agora manda dar.
    mark(&mut sim, Some(master.to_bits()), &mut None);
    for &e in &recipe {
        assert!(
            !is_off_canvas(sim.world(), e),
            "clicar na linha da receita nao a trouxe — o PASSO 1 do smoke aponta para nada"
        );
    }
}

/// ⛔ **Escolher uma coisa qualquer não acende receita nenhuma**, e o olho do artista continua a
/// valer por cima do modo.
#[test]
fn a_loose_object_lights_nothing_and_the_eye_still_wins() {
    let (mut sim, root, _, loose) = scene();
    mark(&mut sim, Some(loose.to_bits()), &mut None);
    assert!(!is_off_canvas(sim.world(), loose), "o objeto solto sumiu");
    // O olho fechado esconde mesmo a receita que está a ser editada — ele é autoria do artista.
    mark(&mut sim, Some(root.to_bits()), &mut None);
    sim.world_mut()
        .entity_mut(root)
        .insert(Visibility::hidden());
    assert!(
        is_off_canvas(sim.world(), root),
        "o modo de edicao passou por cima do olho da Hierarquia"
    );
}

/// ⭐⭐⭐ **«ABRIU AGORA» é reportado UMA vez, e não em todo quadro em que a receita está aberta.**
///
/// É a diferença entre a câmera **ir** à receita quando ela abre e a câmera ficar **presa** a ela:
/// o segundo quadro tem a mesma selecção e a mesma marca, e um `opened` que continuasse cheio
/// puxaria a vista de volta a cada quadro — o artista não conseguiria dar pan enquanto edita.
///
/// **Mutação que deve sangrar:** o `opened` deixar de filtrar pelo `have` (passa a ser *«o que
/// está aberto»*, que é verdade sempre).
#[test]
fn a_recipe_reports_itself_opened_once_and_not_every_frame() {
    let (mut sim, root, piece, _) = scene();
    assert!(
        mark(&mut sim, None::<u64>, &mut None).opened.is_empty(),
        "sem selecção não há nada a abrir"
    );
    let first = mark(&mut sim, Some(piece.to_bits()), &mut None).opened;
    assert_eq!(
        first,
        vec![root],
        "abrir pela peça tem de reportar a RAIZ — é ela que a câmera enquadra"
    );
    assert!(
        mark(&mut sim, Some(piece.to_bits()), &mut None)
            .opened
            .is_empty(),
        "o segundo quadro reportou a mesma abertura — a vista ficaria presa à receita"
    );
    // E fechar e reabrir volta a reportar: a transição é o facto, não a primeira vez.
    mark(&mut sim, None::<u64>, &mut None);
    assert_eq!(
        mark(&mut sim, Some(root.to_bits()), &mut None).opened,
        vec![root],
        "reabrir depois de fechar tem de voltar a enquadrar"
    );
}

/// ⭐⭐⭐ **UMA RECEITA SÓ DE IMAGENS também levanta o vidro.**
///
/// ⚠️ **É a família que a primeira redacção deixava de fora:** o interruptor perguntava à vista do
/// VETOR (`VecViewState::isolated`), que é enchida a partir das formas vectoriais marcadas — e um
/// ragdoll ou um cartão de sprite não tem nenhuma. O vidro nunca subiria para eles.
///
/// **Mutação que deve sangrar:** o `any_open` filtrar por qualquer coisa além do `MasterEditing`.
#[test]
fn a_recipe_made_only_of_images_still_raises_the_glass() {
    let (mut sim, root, _piece, _) = scene();
    assert!(
        !super::any_open(&mut sim),
        "sem receita aberta o vidro nao pode subir — o quadro comum pagaria os passes"
    );
    mark(&mut sim, Some(root.to_bits()), &mut None);
    assert!(
        super::any_open(&mut sim),
        "a receita esta' aberta e o vidro nao subiu — as pecas raster dela ficariam no fundo"
    );
    mark(&mut sim, None::<u64>, &mut None);
    assert!(
        !super::any_open(&mut sim),
        "fechar a receita tem de baixar o vidro"
    );
}

/// ⭐⭐⭐ **A BARRA aparece exactamente quando o VIDRO sobe** — as duas perguntas têm de concordar.
///
/// ⚠️ Elas são feitas por funções diferentes ([`super::any_open`] responde existência, o
/// [`super::open_view`] responde identidade), e uma divergência dá um canvas borrado **sem barra**
/// (ou uma barra sobre um canvas nítido). ⛔ Nenhum dos dois é diagnosticável a olho: o artista vê
/// *«o modo está meio ligado»*.
///
/// **Mutação que deve sangrar:** o `open_view` deixar de exigir `MasterRoot` ou `MasterEditing`.
#[test]
fn the_bar_and_the_glass_answer_the_same_question() {
    let (mut sim, root, piece, _) = scene();
    sim.world_mut()
        .entity_mut(root)
        .insert(ph2d_ecs::StableId(77));
    assert_eq!(
        super::any_open(&mut sim),
        super::open_view(&mut sim).is_some(),
        "fechada: uma das duas ja' se acha aberta"
    );
    mark(&mut sim, Some(piece.to_bits()), &mut None);
    assert!(super::any_open(&mut sim), "controle: a receita abriu");
    let view = super::open_view(&mut sim).expect("a barra nao viu a receita que o vidro ve");
    assert_eq!(view.name, "Badge", "a barra nomeia a receita errada");
    assert_eq!(view.copies, 0, "sem copias, a barra diz zero — nao mente");
}

/// ⭐⭐ **E ela CONTA as cópias** — é a promessa do modo, e a única coisa que o distingue de editar
/// um objecto qualquer.
#[test]
fn the_bar_counts_the_copies_that_follow() {
    let (mut sim, root, _piece, _) = scene();
    sim.world_mut()
        .entity_mut(root)
        .insert(ph2d_ecs::StableId(77));
    for _ in 0..3 {
        sim.world_mut()
            .spawn((Transform::IDENTITY, ph2d_ecs::InstanceOf { master: 77 }));
    }
    // E uma cópia de OUTRA receita, que não pode entrar na conta.
    sim.world_mut()
        .spawn((Transform::IDENTITY, ph2d_ecs::InstanceOf { master: 99 }));
    mark(&mut sim, Some(root.to_bits()), &mut None);
    assert_eq!(
        super::open_view(&mut sim).map(|v| v.copies),
        Some(3),
        "a conta apanhou copias de outra receita, ou perdeu as desta"
    );
}

/// ⭐⭐⭐ **A TRAVA segura a sessão quando a selecção sai** (Enio, 2026-09-07: *«só permita sair da
/// edição apertando Done ou a tecla Enter»*).
///
/// ⚠️ **O controlo negativo é a metade que importa:** sem a trava, a MESMA chamada fecha a receita
/// — é o comportamento que o report descreve, e um gate sem ele passaria com a trava ignorada.
///
/// **Mutação que deve sangrar:** o `.chain(*latch)` sair da montagem do `editing`.
#[test]
fn the_latch_keeps_the_recipe_open_when_the_selection_leaves() {
    let (mut sim, root, piece, loose) = scene();
    sim.world_mut()
        .entity_mut(root)
        .insert(ph2d_ecs::StableId(4242));
    let mut latch = None;
    mark(&mut sim, Some(piece.to_bits()), &mut latch);
    // A trava guarda a identidade DURÁVEL da receita — ver o doc do `mark`.
    latch = Some(4242);
    // O gesto: clicar noutro objecto (ou no vazio) — a selecção sai da receita.
    mark(&mut sim, Some(loose.to_bits()), &mut latch);
    assert!(
        !is_off_canvas(sim.world(), piece),
        "a sessao fechou ao clicar fora — a saida volta a ser um acidente"
    );
    // Controlo: sem a trava, a mesma chamada fecha.
    mark(&mut sim, Some(loose.to_bits()), &mut None);
    assert!(
        is_off_canvas(sim.world(), piece),
        "controlo: sem trava, sair da seleccao TEM de fechar"
    );
}

/// ⚠️ **A trava solta-se sozinha quando a receita MORRE.** Apagá-la a meio da sessão deixaria o
/// artista num modo sem barra (ela é derivada do mundo) e portanto **sem saída**.
#[test]
fn the_latch_lets_go_when_the_recipe_dies() {
    let (mut sim, root, _piece, _) = scene();
    sim.world_mut()
        .entity_mut(root)
        .insert(ph2d_ecs::StableId(4242));
    let mut latch = Some(4242);
    sim.world_mut().entity_mut(root).despawn();
    mark(&mut sim, None::<u64>, &mut latch);
    assert_eq!(latch, None, "a trava ficou presa a uma receita que morreu");
}

/// ⛔⛔⛔ **A SESSÃO SOBREVIVE A UM `Ctrl+Z`** — e a 1.ª versão da trava não sobrevivia.
///
/// O undo **respawna tudo com bits novos** (é a lei escrita do módulo do editor: *«referência
/// durável entre objectos é o `StableId`, nunca os bits»*), e uma trava guardada em bits aponta,
/// depois de um passo, para uma entidade morta. ⇒ desfazer uma edição feita DENTRO da receita
/// **expulsava o artista da sessão**, e nada na tela dizia porquê.
///
/// ⚠️ **A fixtura reproduz o restauro pelo mecanismo**: a entidade morre e outra nasce com o MESMO
/// `StableId` — que é exactamente o que o `apply_project` faz.
///
/// **Mutação que deve sangrar:** a trava voltar a resolver-se por bits.
#[test]
fn the_session_survives_an_undo_step() {
    let (mut sim, root, piece, _) = scene();
    sim.world_mut()
        .entity_mut(root)
        .insert(ph2d_ecs::StableId(4242));
    let mut latch = None;
    mark(&mut sim, Some(root.to_bits()), &mut latch);
    latch = Some(4242);
    assert!(
        !is_off_canvas(sim.world(), piece),
        "controlo: a sessao abriu"
    );

    // O `Ctrl+Z`: o mundo é reconstruído — entidades novas, MESMOS `StableId`.
    sim.world_mut().entity_mut(piece).despawn();
    sim.world_mut().entity_mut(root).despawn();
    let root2 = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Name::new("Badge"),
            MasterRoot,
            ph2d_ecs::StableId(4242),
        ))
        .id();
    let piece2 = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Box"), ChildOf(root2)))
        .id();
    ph2d_ecs::assign_master_pieces(sim.world_mut());

    // A selecção morreu com o passo (o restauro transporta-a por id estável, e a peça podia nem
    // estar seleccionada) — é a TRAVA que tem de segurar a sessão.
    mark(&mut sim, None::<u64>, &mut latch);
    assert_eq!(latch, Some(4242), "a trava largou a receita ao desfazer");
    assert!(
        !is_off_canvas(sim.world(), piece2),
        "o `Ctrl+Z` expulsou o artista da sessao"
    );
}
