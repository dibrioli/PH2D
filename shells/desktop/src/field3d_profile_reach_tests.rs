//! ⭐ **A FAMÍLIA DO PERFIL ALCANÇA O PAINEL?** — os gates de alcance das formas desenhadas
//! (W53), separados do isolamento com que tinham partilhado arquivo.
//!
//! ⚠️ A razão do corte é o teto de LOC do shell (HR-18), e a fronteira **não é arbitrária**: estes
//! gates perguntam *o painel oferece o que o motor sabe fazer?*, e os do arquivo irmão perguntam
//! *o isolamento diz-se e tem volta?*. Duas leis, dois arquivos.

/// ⭐⭐ **TODA FORMA QUE O MOTOR SABE FAZER TEM BOTÃO.**
///
/// # A lei que faltava, e por que a da W34 não a apanhava
///
/// `Primitive::Extrude` e `Primitive::Revolve` existem no motor **desde a W3**, medidos contra
/// oráculos independentes — e **nenhum botão os alcançava**: só as cenas de smoke os construíam. O
/// plano do módulo chama-lhes a razão de existir (*"é aqui que o fluxo do MoI renasce"*).
///
/// ⚠️ **A lei da W34 tem uma exclusão escrita** que os deixava de fora: a tabela dela cobre só as
/// fileiras que **dependem da seleção**, e as formas foram postas de lado como *"ações sempre
/// disponíveis"*. A pergunta certa para esta fileira é outra — *o painel oferece tudo o que o motor
/// sabe fazer?* — e a exclusão da outra lei escondia-a.
///
/// ⭐ A régua é [`ph2d_field::PrimitiveKind::ALL`] — a lista que o motor de facto tem, e que o
/// compilador obriga a crescer: uma primitiva nova aparece aqui **sozinha**, no dia em que nascer.
///
/// ⚠️ **Ela já foi duas coisas piores.** Até 26/08 era uma lista **literal** (*«uma de cada,
/// construída à mão»*) e a contagem no fim só a defendia de si mesma — um `Primitive` novo ficava
/// sem botão e este gate ficava **verde**. Depois passou a ser `key.ends_with(kind.key())`, uma
/// **convenção de nome**, que a W101 partiu (ver abaixo).
#[test]
fn every_primitive_the_engine_can_make_has_a_button() {
    use crate::field3d_shapes::SHAPES;
    use ph2d_field::PrimitiveKind;
    // ⭐⭐⭐ **A RÉGUA É O QUE O CONSTRUTOR DEVOLVE, e não o nome da chave** (W101).
    //
    // ⛔ Ela era `s.key.ends_with(k.key())` — uma **convenção de nome** —, e a W101 partiu-a com
    // uma linha honesta: o `panel.model3d.add.cone_truncated` constrói um `PrimitiveKind::Cone` e
    // não acaba em «cone». Uma régua de string reprova sobre um catálogo correto e, pior, **aprova**
    // uma chave que calhe de acabar bem sem construir nada daquilo.
    //
    // ⭐ Perguntar ao `shape_at` é perguntar o FACTO: *que forma é que este botão faz?* Isso mata a
    // convenção, aceita duas portas para a mesma primitiva (que é o que o cone e o tronco são), e
    // torna estruturalmente impossível duas primitivas partilharem um botão — um slot constrói
    // exatamente uma. *A metade do gate que defendia isso deixou de ter o que defender.*
    let construido: Vec<Option<PrimitiveKind>> = SHAPES.iter().map(|s| s.make.builds()).collect();
    for k in PrimitiveKind::ALL {
        assert!(
            construido.contains(&Some(k)),
            "o motor sabe fazer «{}» e o catálogo não tem linha nenhuma que a construa — é uma \
             feature completa e invisível, que é o defeito que a W53 pagou",
            k.key()
        );
    }
    // ⭐⭐⭐ **…e o CONTROLE, re-derivado na W101 porque a premissa dele DISSOLVEU.**
    //
    // ⚠️ Ele dizia `SHAPES.len() == PrimitiveKind::ALL.len() + 2` — *«além das primitivas, só as
    // duas esculturas»* —, e isso pressupunha **uma porta por primitiva**. O `Cone` e o `Truncated
    // Cone` são a MESMA primitiva com dois defaults, e a contagem passou a reprovar sobre um
    // catálogo correto. ⛔ Afrouxá-la para `>=` seria apagar o controle; a cura é perguntar o que
    // ele de facto defendia: *o painel não promete o que o motor não tem*.
    //
    // ⇒ toda linha do catálogo ou **nomeia** uma primitiva, ou é uma das quatro que não saem de um
    // raio (as duas de perfil, as duas esculturas). Uma chave inventada não cai em nenhum dos dois.
    for (i, shape) in SHAPES.iter().enumerate() {
        let escultura = matches!(
            shape.make,
            crate::field3d_shapes::Make::Sculpt | crate::field3d_shapes::Make::SculptScene
        );
        assert!(
            construido[i].is_some() || escultura,
            "{} não produz primitiva nenhuma e não é uma escultura — o painel promete uma forma \
             que o motor não tem",
            shape.key
        );
    }
}

/// ⭐ **As duas formas de perfil só são oferecíveis com um contorno FECHADO escolhido** — a lei da
/// W34 aplicada à segunda família cuja disponibilidade não é constante.
///
/// ⚠️ **Mede o CATÁLOGO desde a W100**, como a irmã da escultura da cena: a fileira de chips virou
/// um botão que abre a paleta, e a filtragem mudou-se para lá.
#[test]
fn the_profile_buttons_appear_only_with_a_closed_outline_selected() {
    use crate::field3d_shapes::{SHAPES, available, slot_of};
    let ex = slot_of("panel.model3d.add.extrude").expect("extrude existe");
    let rev = slot_of("panel.model3d.add.revolve").expect("revolve existe");
    assert!(
        !available(&SHAPES[ex], false, false) && !available(&SHAPES[rev], false, false),
        "sem contorno escolhido, «Extrude» e «Revolve» não têm o que extrudar"
    );
    assert!(
        available(&SHAPES[ex], false, true) && available(&SHAPES[rev], false, true),
        "com um contorno escolhido, as duas têm de estar disponíveis"
    );
    // ⛔ **O CONTROLE**: exatamente DUAS formas seguem o contorno — sem ele, um `available` que
    // devolvesse `profile` para tudo passaria as duas afirmações acima.
    let seguem = SHAPES
        .iter()
        .filter(|s| available(s, false, false) != available(s, false, true))
        .count();
    assert_eq!(seguem, 2, "só as duas de perfil dependem do contorno");
}

/// ⭐⭐⭐ **CADA forma constrói a SUA, e a posição não decide nada** (W100).
///
/// # ⚠️ O que este gate defendia, e por que a cerca mudou de sítio
///
/// Ele defendia **quatro constantes derivadas do fim da lista** (`SHAPES.len() - 4` … `- 1`): um
/// `-3` trocado por um `-4` faria dois botões serem o mesmo, e o comentário delas mandava, com
/// todas as letras, acrescentar formas *«antes das esculturas»* — ⛔ **acrescentar no fim fazia o
/// *Extrude* abrir o diálogo de escultura, sem erro nenhum.**
///
/// ⭐ Com o construtor na própria linha ([`crate::field3d_shapes::Make`]) essa colisão **deixou de
/// ser exprimível**. O que fica a medir é a propriedade que a substituiu: *nenhuma posição é lida
/// para saber o que uma forma é* — e isso lê-se exigindo que cada `Make` não-fórmula apareça
/// **exatamente uma vez**, que é o que uma lista de 60 linhas copiadas-e-coladas pode partir.
#[test]
fn the_four_derived_slots_are_distinct_and_in_range() {
    use crate::field3d_shapes::{Make, SHAPES};
    /// O predicado que reconhece **uma** porta não-fórmula. ⚠️ Um `type` e não o tipo cru: o clippy
    /// recusa a assinatura literal, e um alias diz melhor o que ela é.
    type Porta = (&'static str, fn(&Make) -> bool);
    let porta: [Porta; 4] = [
        ("Extrude", |m| matches!(m, Make::Extrude)),
        ("Revolve", |m| matches!(m, Make::Revolve)),
        ("Sculpt", |m| matches!(m, Make::Sculpt)),
        ("SculptScene", |m| matches!(m, Make::SculptScene)),
    ];
    for (nome, quantos) in porta.map(|(n, p)| (n, SHAPES.iter().filter(|s| p(&s.make)).count())) {
        assert_eq!(
            quantos, 1,
            "o catálogo tem {quantos} linhas com o `Make::{nome}` - duas fariam a segunda ser \
             inalcançável, e zero faria a feature desaparecer em silêncio"
        );
    }
}
