//! **A CENA DE DEMONSTRAÇÃO DO ESQUELETO** — a geometria do braço e do tentáculo, a cadeia de
//! ossos e a acção que os Smart Bones percorrem.
//!
//! ⭐ Nasceu na auditoria de arquitectura de 2026-09-12 (A1). Estas peças moravam no módulo da cena
//! `PH2D_VEC_BONE_SMOKE` (`ph2d_app_vec::smoke_bone`) e têm DOIS consumidores de propósito: a cena
//! monta-as, e os gates da família do esqueleto medem-nas (*uma tabela escrita duas vezes
//! divergiria em silêncio e o gate passaria a aprovar uma cena que já não existe*). Para os ter, a
//! `ph2d-app-skeleton` dependia — em dev — da família Vector inteira, e o gate
//! `architecture_no_dependency_climbs_a_layer` mediu-o como família → família. O que atravessava era
//! CONTEÚDO de cena, e conteúdo com dois donos é uma folha.
#![forbid(unsafe_code)]

use ph2d_ecs::Entity;

/// ⭐⭐⭐ **A GEOMETRIA DO BRAÇO — uma tabela, dois consumidores.**
///
/// A cena monta-a e o gate `the_smoke_scene_gives_the_anchor_a_side_to_defend` mede-a. ⚠️ Escrita
/// duas vezes, ela divergiria em silêncio e o gate passaria a aprovar uma cena que já não existe —
/// que é exactamente o modo de falha do `CLAUDE.md` §5.0 (*uma cena que ensina o contrário do que
/// acontece é pior que uma cena ausente*).
pub const ARM_A: [f64; 2] = [-8.2, 2.5];
/// A ponta do braço da cena. Ver [`ARM_A`].
pub const ARM_B: [f64; 2] = [-1.8, 2.5];
/// Quantos ossos o braço da cena tem. Ver [`ARM_A`].
pub const ARM_BONES: usize = 3;
/// ⭐ **A DOBRA DO COTOVELO**, em radianos — o que dá à âncora um lado para capturar.
///
/// ⚠️ **`0` aqui apaga a wave do lado da dobra em silêncio:** o `add` capturaria `Auto` sobre uma
/// corrente recta, e o dono veria o joelho inverter exactamente como antes. O gate acima existe
/// para esse zero ser vermelho em vez de invisível.
pub const ARM_ELBOW_BEND: f32 = 0.45; // LITERAL-PX-OK: ângulo do documento (rad)

/// ⭐⭐⭐ **A DOBRA DO OMBRO, PARA O LADO CONTRÁRIO** — o que dá ao modo **MISTO** um sujeito na
/// cena (ordem do dono, 2026-09-14: *«ossos com ângulos para os dois lados»*).
///
/// ⛔⛔ **Sem ela a cena não sabe exprimir o que o modo faz:** com uma só junta dobrada, `Mixed`,
/// `Ccw` e `Cw` entregam a MESMA pose, e o dono escolheria os três sem ver diferença nenhuma —
/// *uma cena que não distingue os modos ensina que eles não existem*. Com o braço em **S**, subir
/// o `IK Chain` para `3` põe as duas juntas sob a âncora: o `Ccw` e o `Cw` alinham-nas, o `Mixed`
/// deixa cada uma no lado em que está.
///
/// ⚠️ O sinal é o que importa, não o valor: ela tem de ser **oposta** à [`ARM_ELBOW_BEND`], e a
/// cerca abaixo faz disso um erro de compilação.
pub const ARM_SHOULDER_BEND: f32 = -0.45; // LITERAL-PX-OK: ângulo do documento (rad)

/// ⭐ A cerca das duas dobras do braço — em TEMPO DE COMPILAÇÃO, ao lado do que guarda.
///
/// ⚠️ Zero em qualquer uma delas apaga uma wave **em silêncio**: sem a do cotovelo o `add` captura
/// uma corrente recta (e o joelho volta a inverter); com as duas do MESMO lado o braço deixa de ser
/// um S e o modo misto fica indistinguível do `Ccw`.
const _: () = {
    assert!(ARM_ELBOW_BEND != 0.0);
    assert!(ARM_SHOULDER_BEND != 0.0);
    assert!((ARM_ELBOW_BEND > 0.0) != (ARM_SHOULDER_BEND > 0.0));
    // O braço precisa de TRÊS ossos para ter duas juntas interiores.
    assert!(ARM_BONES >= 3);
};

/// Qual osso do tentáculo nasce com limite de ângulo — o 2.º, que fica bem no meio da parte visível
/// da cadeia. ⚠️ O vizinho fica SEM limite de propósito: é o contraste que ensina.
pub const TENTACLE_LIMITED_BONE: usize = 2;

/// Meia-faixa do limite da cena, em radianos (~17°). ⚠️ Estreita de propósito: uma faixa larga
/// obrigaria o dono a girar meia volta antes de sentir a parede, e o smoke ficaria mudo.
pub const TENTACLE_LIMIT_HALF: f64 = 0.3; // LITERAL-PX-OK: ângulo do documento (rad)

/// ⭐⭐⭐ **As duas cercas das constantes de cima, em TEMPO DE COMPILAÇÃO** — e elas vivem aqui, ao
/// lado do que guardam, e não num teste noutro ficheiro.
///
/// ⚠️ Uma meia-faixa fora de `(0, π)` ou **trava** o osso (zero) ou **não o limita** (meia volta ou
/// mais), e nos dois casos o smoke fica mudo sem dizer porquê. E o osso limitado tem de ter um
/// vizinho ACIMA dele, senão não há o contraste que ensina.
///
/// ⭐ Como são constantes, isto é `const {}`: quem as editar para um valor mudo **não compila**, em
/// vez de descobrir num teste que ele podia não ter corrido.
const _: () = {
    assert!(TENTACLE_LIMIT_HALF > 0.0);
    assert!(TENTACLE_LIMIT_HALF < std::f64::consts::PI);
    assert!(TENTACLE_LIMITED_BONE >= 1);
    // ⛔⛔ **A metade que FALTAVA** (auditoria de 2026-09-08): o doc da cerca prometia *«o osso
    // limitado tem de ter um vizinho ACIMA dele»* e só afirmava o piso. Pôr `TENTACLE_LIMITED_BONE`
    // acima da contagem da cadeia faz o `if n == …` **nunca disparar** — a cena nasce sem limite
    // nenhum, o smoke fica mudo, e nada acusa. *Uma cerca com metade das paredes é uma cerca que se
    // atravessa por um lado.*
    assert!(TENTACLE_LIMITED_BONE < TENTACLE_BONES - 1);
};

/// Quantos ossos o tentáculo da cena tem — a fonte da contagem, lida pela cena **e** pela cerca de
/// cima. ⚠️ Escrita duas vezes, ela divergiria no dia em que a cadeia crescesse.
pub const TENTACLE_BONES: usize = 6;

/// ⭐⭐⭐ **A ACÇÃO QUE A CENA JÁ TRAZ** — o nome que aparece no selector *Action* do painel.
///
/// ⚠️⚠️ **Ela existe por causa do report do dono de 2026-09-08** (*«não há meios de selecionar nem
/// o objeto alvo nem a animação»*): sem uma acção com CONTEÚDO na cena, o único caminho para provar
/// um osso inteligente era o artista gravar uma animação primeiro — e o smoke passava a testar a
/// timeline em vez do osso. *Uma cena que só produz o fenómeno depois de o artista acertar OUTRO
/// gesto não prova nada quando esse gesto falha* (`CLAUDE.md` §5.0).
pub const DEMO_ACTION: &str = "Leaf Rises";

/// Quanto a folha sobe ao longo da acção da cena, nas unidades do documento.
///
/// ⚠️ Grande o suficiente para o percurso se ler numa forma que mede ~2 de altura: uma subida de
/// meia forma leria-se como tremor, e o smoke ficaria mudo sobre um motor correcto.
pub const DEMO_RISE: f32 = 3.0; // LITERAL-PX-OK: distância do documento, não medida de UI

/// A duração da acção da cena, em segundos. ⚠️ **Ela é o que o `clamp` do `action_time` divide**,
/// e não uma escolha estética: o giro do osso mapeia-se nesta faixa inteira.
pub const DEMO_SECONDS: f64 = 2.0;

/// ⭐⭐⭐ **SEMEIA A ACÇÃO DA CENA** no documento — a folha `bits` sobe [`DEMO_RISE`] ao longo de
/// [`DEMO_SECONDS`].
///
/// ⚠️⚠️ **Ela DEVOLVE o documento à acção que estava aberta**, e isso é load-bearing, não arrumação:
/// o `insert_key` escreve no clip **activo**, e um controlo **não percorre a acção aberta** (ali o
/// artista está a gravá-la). Deixá-la aberta faria o osso inteligente nascer inerte na própria cena
/// que existe para o demonstrar — *uma cena que ensina o contrário do que acontece é pior que uma
/// cena ausente* (`CLAUDE.md` §5.0).
///
/// ⭐ Vive **fora** do `bone_smoke_bind` para o gate a poder correr: aquela função precisa do `gfx`,
/// que segura uma surface de janela real.
pub fn seed_demo_action(doc: &mut ph2d_timeline::TimelineDoc, bits: u64) {
    let antes = doc.active_index();
    let i = doc.add_clip(DEMO_ACTION.to_string());
    doc.set_active(i);
    for (t, v) in [(0.0, 0.0_f32), (DEMO_SECONDS, DEMO_RISE)] {
        doc.insert_key(
            bits,
            ph2d_timeline::PropKind::TranslationY,
            ph2d_anim::RationalTime::from_seconds(t),
            ph2d_anim::AnimValue::Float(v),
            ph2d_anim::Interp::Linear,
        );
    }
    doc.set_active(antes);
}

/// Quanto o osso da ponta do braço gira ao longo da acção da cena, em radianos.
///
/// ⚠️ Grande o suficiente para os fantasmas do onion se distinguirem uns dos outros: com meio
/// décimo de radiano as quatro silhuetas empilham-se e o smoke fica mudo sobre um motor correcto.
pub const DEMO_SWING: f32 = 0.9; // LITERAL-PX-OK: radianos do documento, não medida de UI

/// ⭐⭐⭐ **A ACÇÃO DO BRAÇO — e ela vive no clip ABERTO, ao contrário da [`seed_demo_action`].**
///
/// ⚠️⚠️ **A diferença não é arrumação: é o consumidor.** O onion lê o clip **activo**
/// (`entity_key_times`/`animated_entities` perguntam ao `active_clip`), logo uma acção fechada
/// deixaria os fantasmas sem passado nem futuro a mostrar — *uma cena que só produz o fenómeno
/// depois de o artista acertar OUTRO gesto não prova nada quando esse gesto falha* (`CLAUDE.md`
/// §5.0). A acção do osso inteligente é o oposto: ela TEM de estar fechada, e o doc dela diz porquê.
///
/// ⚠️ **O canal é a ROTAÇÃO de um OSSO**, e é isso que faz a arte presa dobrar sem ninguém lhe
/// tocar: a imagem não tem key nenhuma, e é o esqueleto que a move.
pub fn seed_arm_swing(doc: &mut ph2d_timeline::TimelineDoc, bits: u64) {
    for (t, v) in [(0.0, 0.0_f32), (DEMO_SECONDS, DEMO_SWING)] {
        doc.insert_key(
            bits,
            ph2d_timeline::PropKind::Rotation,
            ph2d_anim::RationalTime::from_seconds(t),
            ph2d_anim::AnimValue::Float(v),
            ph2d_anim::Interp::Linear,
        );
    }
}

/// Uma cadeia de `n` ossos de `a` a `b` (mundo), o 1.º sem pai. Devolve a RAIZ.
pub fn cadeia(sim: &mut ph2d_ecs::SimWorld, a: [f64; 2], b: [f64; 2], n: usize) -> Option<Entity> {
    #[expect(
        clippy::cast_precision_loss,
        reason = "n é a contagem de ossos da cena, sempre um punhado"
    )]
    let passo = [(b[0] - a[0]) / n as f64, (b[1] - a[1]) / n as f64];
    let mut pai: Option<Entity> = None;
    let mut raiz: Option<Entity> = None;
    for i in 0..n {
        #[expect(clippy::cast_precision_loss, reason = "idem")]
        let t = i as f64;
        let o = [a[0] + passo[0] * t, a[1] + passo[1] * t];
        let p = [o[0] + passo[0], o[1] + passo[1]];
        let bits = ph2d_skeleton_live::bone::create(sim, pai, o, p)?;
        pai = Some(Entity::from_bits(bits));
        raiz = raiz.or(pai);
    }
    raiz
}

/// A PONTA de uma cadeia — desce pelo 1.º filho até não haver osso abaixo.
pub fn ponta_da_cadeia(sim: &ph2d_ecs::SimWorld, raiz: Entity) -> Entity {
    let mut e = raiz;
    while let Some(f) = sim.world().get::<ph2d_ecs::Children>(e).and_then(|c| {
        c.iter()
            .find(|c| sim.world().get::<ph2d_skeleton_ecs::Bone>(**c).is_some())
    }) {
        e = *f;
    }
    e
}

/// Quantos sub-ossos têm os ossos do MEIO da [`bifurcacao`] — o bastante para a curva se ler.
pub const BRANCH_SEGMENTS: u8 = 8;

/// Os dois ossos que a [`bifurcacao`] põe a curvar pela corrente, com os nomes que a Hierarquia
/// mostra.
#[derive(Clone, Copy, Debug)]
pub struct Bifurcacao {
    /// O osso do meio com **um** filho: a curva segue-o nas duas pontas.
    pub com_um_filho: Entity,
    /// O osso do meio com **dois** filhos: não há «o seguinte», e o lado da ponta fica recto.
    pub com_dois_filhos: Entity,
}

/// O nome, na Hierarquia, do osso da [`Bifurcacao::com_um_filho`].
pub const BRANCH_ONE_CHILD: &str = "Curve: one child";
/// O nome, na Hierarquia, do osso da [`Bifurcacao::com_dois_filhos`].
pub const BRANCH_TWO_CHILDREN: &str = "Curve: two children";

/// ⭐⭐⭐ **A BIFURCAÇÃO — a pergunta do `From Chain` posta em cena** (o dono pediu-a em 2026-09-16:
/// *«não entendi o que vc explicou, monte uma cena para eu entender»*).
///
/// Dois esqueletos pequenos, lado a lado, iguais menos numa coisa: o osso do MEIO (curvado pela
/// corrente, `Curve Handles: From Chain`) tem **um** filho à esquerda e **dois** à direita.
///
/// - à esquerda a curva sai do pai e **entra no filho** — as duas pontas do osso seguem a corrente;
/// - à direita a raiz segue o pai igual, e **a ponta fica recta**: com dois filhos não há «o
///   seguinte» (`ph2d_skeleton_live::bend_live::effective_spec`), e escolher um deles seria um
///   sorteio.
///
/// A pergunta ao dono é se quer ESCOLHER qual dos filhos manda na curva (o *custom handle* do
/// Blender), e ela move o formato do ficheiro.
///
/// ⚠️ Mora no canto de baixo à esquerda da cena (`x` de `-8,4` a `-1,5`, `y` de `-5,4` a `-3,8`),
/// onde as outras peças não chegam.
pub fn bifurcacao(sim: &mut ph2d_ecs::SimWorld) -> Option<Bifurcacao> {
    use ph2d_skeleton_live::bone::create;
    let mut lado = |x0: f64, dois: bool, nome: &str| -> Option<Entity> {
        const Y: f64 = -4.6;
        let pai = Entity::from_bits(create(sim, None, [x0, Y - 0.7], [x0 + 1.0, Y])?);
        let meio = Entity::from_bits(create(sim, Some(pai), [x0 + 1.0, Y], [x0 + 2.4, Y])?);
        create(sim, Some(meio), [x0 + 2.4, Y], [x0 + 3.1, Y + 0.8])?;
        if dois {
            create(sim, Some(meio), [x0 + 2.4, Y], [x0 + 3.1, Y - 0.8])?;
        }
        let mut osso = sim.world_mut().get_mut::<ph2d_skeleton_ecs::Bone>(meio)?;
        osso.segments = BRANCH_SEGMENTS;
        osso.handles = ph2d_skeleton::bend::Handles::Auto;
        sim.world_mut()
            .entity_mut(meio)
            .insert(ph2d_ecs::Name::new(nome.to_string()));
        Some(meio)
    };
    Some(Bifurcacao {
        com_um_filho: lado(-8.4, false, BRANCH_ONE_CHILD)?,
        com_dois_filhos: lado(-4.6, true, BRANCH_TWO_CHILDREN)?,
    })
}

#[cfg(test)]
mod tests {
    /// ⭐⭐ **A CENA MOSTRA O QUE PROMETE** — a raiz curva nos dois esqueletos, e a ponta só onde há
    /// UM filho (`CLAUDE.md` §5.0: *uma cena que ensina o contrário do que acontece é pior que uma
    /// cena ausente*).
    ///
    /// (Mutações: a bifurcação sem o segundo filho ⇒ RED; o osso do meio sem `From Chain` ⇒ RED.)
    #[test]
    fn the_branch_scene_shows_a_straight_tip_only_where_the_bone_has_two_children() {
        let mut sim = ph2d_ecs::SimWorld::default();
        let b = super::bifurcacao(&mut sim).expect("a cena monta");
        let curva = |e| {
            ph2d_skeleton_live::bend_live::effective_spec(&sim, e)
                .expect("osso")
                .curve
        };
        let (um, dois) = (curva(b.com_um_filho), curva(b.com_dois_filhos));
        println!("um filho: {um:?}\ndois filhos: {dois:?}");
        let arqueia = |v: [f64; 2]| v[1].abs() > 0.05;
        assert!(
            arqueia(um.inn) && arqueia(dois.inn),
            "a raiz tem de seguir o pai nos dois"
        );
        assert!(arqueia(um.out), "com um filho a ponta tem de entrar nele");
        assert_eq!(
            dois.out,
            [0.0, 0.0],
            "com dois filhos a ponta tem de ficar recta — e' a pergunta da cena"
        );
        for (e, nome) in [
            (b.com_um_filho, super::BRANCH_ONE_CHILD),
            (b.com_dois_filhos, super::BRANCH_TWO_CHILDREN),
        ] {
            assert_eq!(
                sim.world()
                    .get::<ph2d_ecs::Name>(e)
                    .map(|n| n.as_str().to_string()),
                Some(nome.to_string())
            );
        }
    }
}
