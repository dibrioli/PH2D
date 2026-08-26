//! ⭐⭐ **O que este objecto OFERECE e o que ele VESTE** — o irmão do
//! [`field3d_scene_panel`](super::panel) por assunto.
//!
//! ⚠️ **O corte nasceu de um teto** (HR-18, `600` LOC) e é por responsabilidade: o irmão monta o
//! **retrato** que o painel lê; este ficheiro responde às duas perguntas que dependem de **quem
//! está escolhido** — *que verbos este objecto de facto oferece?* e *que selo ele veste na
//! Hierarquia?*. As duas partilham a mesma lei (uma fileira que deixou de ser fixa) e os mesmos
//! consumidores: o painel **e** o despacho.

/// As ações que valem para **qualquer** objeto destacável, na ordem do seletor.
///
/// ⚠️ **O terceiro é um INTERRUPTOR, os dois primeiros são ações** — e a diferença aparece no
/// `active`: *Isolate* acende quando aquele nó é o que está isolado. Misturar os dois tipos numa
/// fileira só é deliberado: a pergunta que ela responde é *"o que faço com o que está escolhido?"*,
/// e isolar é uma resposta a ela.
pub(crate) const ACTS: [&str; 3] = [ACT_DUPLICATE, ACT_DELETE, ACT_ISOLATE];

pub(crate) const ACT_DUPLICATE: &str = "panel.model3d.act.duplicate";
pub(crate) const ACT_DELETE: &str = "panel.model3d.act.delete";
pub(crate) const ACT_ISOLATE: &str = "panel.model3d.act.isolate";
/// ⭐⭐ **LARGAR o desenho** (W57) — a forma deixa de o seguir e fica com a última que teve.
pub(crate) const ACT_UNLINK: &str = "panel.model3d.act.unlink";
/// ⭐⭐ **LIGAR ao contorno escolhido** (W57) — a metade que faltava do vínculo.
pub(crate) const ACT_LINK: &str = "panel.model3d.act.link";

/// ⭐⭐⭐ **RELIGAR uma escultura cujo arquivo sumiu** (W76) — a saída que o aviso não tinha.
///
/// ⚠️ **O aviso existia desde a W23 e era um beco:** reabrir um projeto cuja malha mudou de sítio
/// diz *«Sculpture bunny.obj is missing»* e a peça abre sem ela — e a única cura era **pôr o arquivo
/// de volta no caminho exacto**. *Um aviso que nomeia o problema e não oferece o gesto manda o
/// artista consertar o disco.*
pub(crate) const ACT_RELINK: &str = "panel.model3d.act.relink";

/// ⭐⭐⭐ **AS AÇÕES QUE ESTE OBJETO DE FACTO OFERECE** (W57) — uma porta, dois consumidores.
///
/// ⚠️ **Ela existe porque a fileira deixou de ser fixa.** Até a W56 as ações eram três, sempre as
/// mesmas, e quem drenava a intenção casava o `slot` por **número** (`0`, `1`, `ISOLATE_SLOT`). Com
/// verbos que só aparecem às vezes, o número de um slot passa a depender do que foi publicado — e
/// duas listas escritas em sítios diferentes fariam um botão executar o verbo do vizinho **sem erro
/// nenhum**. ⇒ o painel e o despacho chamam **esta** função, e o slot resolve-se em **chave**.
///
/// ⚠️ Vazia quando o escolhido **não se destaca da peça**, pela mesma razão da fileira de operações:
/// um controle que aparece e não faz nada é pior do que um que não aparece. ⭐ **A RAIZ era o caso
/// que a lia errado** (W34): `selection.is_empty()` deixava a fileira aparecer com a peça inteira
/// escolhida, e ali `duplicate` e `remove` recusam os dois — por decisão escrita, não por acaso.
/// Quem responde é [`ph2d_field_ecs::can_detach`], a mesma função que os dois gestos consomem: *a
/// recusa era uma decisão; a affordance que a ignorava era um defeito.*
pub(crate) fn acts_for(
    world: &bevy_ecs::world::World,
    selection: &[bevy_ecs::entity::Entity],
) -> Vec<&'static str> {
    let Some(&one) = selection.first() else {
        return Vec::new();
    };
    if !ph2d_field_ecs::can_detach(world, one) {
        return Vec::new();
    }
    let mut out = ACTS.to_vec();
    // ⚠️ **Largar é oferecido a quem TEM vínculo**, e a pergunta é feita ao componente — a mesma
    // porta que faz a linha «Resolution» aparecer. Perguntar pela FORMA ofereceria o verbo a uma
    // extrusão solta, que não tem o que largar.
    if world
        .get::<ph2d_field_ecs::FieldProfileSource>(one)
        .is_some()
    {
        out.push(ACT_UNLINK);
    }
    // ⚠️ **Ligar precisa das DUAS pontas**: um contorno fechado escolhido no editor vetorial, e uma
    // forma que saiba o que fazer com um perfil. Um `Box` ligado a um desenho é estado inalcançável
    // — o recozimento escreveria um perfil onde a forma não tem onde o pôr.
    if crate::field3d_smoke::profile_pick().is_some() && takes_a_profile(world, one) {
        out.push(ACT_LINK);
    }
    // ⚠️ **Religar é oferecido só a quem PERDEU o arquivo** — a pergunta é ao registo, que é a
    // mesma resposta parcial que o `field3d_reload::missing_keys` lê. Oferecê-lo a uma escultura
    // que está lá seria um verbo que não tem o que consertar; escondê-lo quando falta é o beco que
    // esta wave veio fechar.
    if missing_sculpture(world, one) {
        out.push(ACT_RELINK);
    }
    out
}

/// **Este nó é uma escultura cujo campo o registo não conhece?**
///
/// ⚠️ Uma chave `scene:` fica de FORA: ela nomeia a escultura viva da cena, que não veio de arquivo
/// nenhum — pedir um `.obj` para a substituir seria mandar o artista procurar o que nunca existiu.
/// Quem a repõe é o `+ Sculpt from scene`, e o `resolve_missing` já lhe pede sozinho.
pub(crate) fn missing_sculpture(
    world: &bevy_ecs::world::World,
    e: bevy_ecs::entity::Entity,
) -> bool {
    let Some(ph2d_field_ecs::FieldNode {
        shape: ph2d_field::NodeShape::Sampled { key },
    }) = world.get::<ph2d_field_ecs::FieldNode>(e)
    else {
        return false;
    };
    !key.starts_with(crate::field3d_import::SCENE_PREFIX)
        && !crate::field3d_smoke::sampled_registry().contains_key(key)
}

/// ⭐⭐ **O SELO DE QUEM SEGUE UM DESENHO** (W57) — o vínculo passa a ver-se na Hierarquia.
///
/// ⚠️ **A linha «Resolution» do painel dizia-o, e ninguém abre um painel para perguntar.** Quem
/// olha a árvore não via diferença nenhuma entre uma extrusão **viva** (que muda quando a curva
/// muda) e uma **solta** (uma fotografia dela) — duas coisas que se comportam de forma oposta e
/// liam-se igual. *Um estado que só o inspector conta é um estado que se descobre por acidente.*
///
/// ⚠️ **Três letras, não uma frase** — é a convenção da fileira de selos (`SUB`, `INT`, `EXC`), e
/// um selo é um **código**, nunca um rótulo traduzido: quem o pinta dá-lhe o tom, e o olho lê o tom
/// antes de decifrar as letras.
/// ⚠️ **Ele é PUBLICADO, não perguntado** — a mesma ponte do retrato do painel, e pela mesma razão.
/// Quem pinta a Hierarquia tem o mundo emprestado no meio do quadro, e uma consulta da `bevy_ecs`
/// pede-o **mutável** (ela guarda o estado dela lá dentro). O módulo já percorre a árvore uma vez
/// por quadro para publicar o retrato; o selo sai dessa mesma travessia. *Um empréstimo mutável
/// pedido só para ler é onde um `RefCell` de shell nasce.*
pub(crate) fn link_badges() -> std::collections::BTreeMap<u64, &'static str> {
    LINK_BADGES.with(|c| c.borrow().clone())
}

/// **Publica os selos do quadro** — a porta de escrita, ao lado da de leitura.
///
/// ⚠️ Ela existe porque o corte da W76 deixou o **armazenamento** aqui e o **produtor** no irmão: um
/// `thread_local` visível de fora seria a fronteira a não dizer nada. *Quem escreve chama uma
/// função; o estado não sai de casa.*
pub(crate) fn publish_badges(m: std::collections::BTreeMap<u64, &'static str>) {
    LINK_BADGES.with(|c| *c.borrow_mut() = m);
}

thread_local! {
    static LINK_BADGES: std::cell::RefCell<std::collections::BTreeMap<u64, &'static str>> =
        const { std::cell::RefCell::new(std::collections::BTreeMap::new()) };
}

/// ⭐⭐⭐ **O código do selo do ISOLAMENTO**, e ele **GANHA do vínculo na mesma linha.**
///
/// # ⚠️ O buraco que ele fecha
///
/// O painel do MODEL já dizia *"Isolated: X"* desde a W44 — mas a Hierarquia, que é onde o artista
/// olha quando pergunta *"por que só isto aparece?"*, não dizia nada. ⛔ *Um estado que esconde
/// trabalho e não se anuncia onde a ausência se vê é uma armadilha, não uma feature.*
///
/// # ⛔ A precedência, e por que ela não é um empate
///
/// O campo do selo é **um por linha**, e o comentário do merge no `render_loop` afirmava que *"as
/// duas famílias nunca caem na mesma entidade"* — verdade enquanto as famílias eram forma vetorial
/// e nó do modelador. ⚠️ **`ISO` e `LNK` caem**: um nó isolado pode seguir um desenho.
///
/// ⇒ **`ISO` ganha**, e a razão não é gosto: *o `LNK` é uma propriedade daquele nó, e o `ISO` é um
/// estado da VISTA que explica por que todo o resto desapareceu.* Quando as duas competem, a
/// pergunta que o artista tem é a segunda. ⚠️ E o `LNK` daquela linha não se perde de vista: ele
/// volta assim que o isolamento cair, e o gesto que o tira está na mesma fileira.
pub(crate) const ISOLATE_BADGE: &str = "ISO";

/// O código do selo do vínculo. ⚠️ **Novo, e não um dos que a tabela de tons já tinha**: reusar
/// `PRF` (que existe lá com outro dono) faria duas famílias partilharem um tom, e mudar o tom de uma
/// para acomodar a outra é repintar um selo alheio.
pub(crate) const LINK_BADGE: &str = "LNK";

/// Esta forma sabe o que fazer com um contorno? — as duas de perfil, e só elas.
fn takes_a_profile(world: &bevy_ecs::world::World, e: bevy_ecs::entity::Entity) -> bool {
    matches!(
        world.get::<ph2d_field_ecs::FieldNode>(e).map(|n| &n.shape),
        Some(ph2d_field::NodeShape::Leaf(
            ph2d_field::Primitive::Extrude { .. } | ph2d_field::Primitive::Revolve { .. }
        ))
    )
}
