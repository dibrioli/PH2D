//! **A família da física** — os 32 componentes de `ph2d-physics-ecs`.
//!
//! ⚠️ **São CONFIG, nunca estado vivo do solver** ([ADR-0131](../../../../docs/architecture/decisions/0131-physics-global-runtime-truth-rapier-ecs-bridge.md)):
//! *"o undo ordena por bytes"*. Isso é o que os torna anexáveis com segurança — anexar um
//! `RigidBody` é declarar uma intenção, não injetar um corpo no meio de um passo do solver.
//!
//! # ⭐⭐ UMA PORTA, e 31 LINHAS DE SECÇÃO (report do dono, 2026-09-13 e 14)
//!
//! *«Os componentes da física foram colocados no modal que é aberto ao clicar no `+` do inspector.
//! Contudo foi erroneamente picotado, dividido em inúmeros supostos componentes que na verdade são
//! apenas seções das opções de física.»*
//!
//! Medido: dos **32** tipos descritos aqui, **30 estavam `Authored`** — `30` de `85` itens da
//! paleta inteira, **35 %**, e de longe a maior família (a seguinte, `core`, oferece 22). E
//! **nenhum** deles, menos um, é uma escolha do artista: são as **rows** que a §11/§12/§13/§14 já
//! pinta *e já anexa*, ou gestos que a paleta genérica não sabe exprimir.
//!
//! ⭐⭐ **E a decisão final é do dono** (2026-09-14, depois de ver a poda de três):
//! *«um objeto de física (Physics Body) e todas as opções aparecem com ele (inclusive Collision
//! Shape e Platform Player)»*. ⇒ **`30 → 1`**.
//!
//! ⚠️ **A régua não é nova — é a razão 2 do próprio [`crate::Attach::Intrinsic`]**, que já estava
//! escrita e aplicada a dois destes (`Dominance`, `MassOverride`): *o neutro existe e anexá-lo
//! seria um **NO-OP**; a PRESENÇA é que carrega o sentido, e o valor de anexação vem do CONTEXTO,
//! que a paleta genérica não conhece.* O que faltava era **correr a régua sobre a família inteira**
//! — o helper [`i`] nasceu `p` (autorado) e ninguém classificou item a item.
//!
//! ⚠️ **Medição do outro lado, e é ela que autoriza a poda:** cada um dos 27 tem um gesto do
//! Inspector que o anexa e o destaca, pelo idioma da presença-override. `PhysicsFieldEdit` (o
//! vocabulário da §11) nomeia-os um a um — `Ccd`, `LockPositionX/Y`, `LockRotation`,
//! `GravityScale`, `InitialVelocity`, `MaterialCombine`, `DampingOverride`, `OneWayPlatform`,
//! `NoWallCling`, `WalkSurface` e as **sete** da zona (`AreaEffector`, `AreaDrag`, `AreaBuoyancy`,
//! `AreaFormDrag`, `AreaTorque`, `AreaFalloff`, `AreaForceWorldAxes`); o `PlayerMode` e o
//! `PlayerSignals` saem dos toggles da §14; o `SignalOnHit`/`SignalOnLeave` das duas rows de texto;
//! e o `PhysicsJoint`, o `PulleyWheel`, o `WestonAxle`, o `JointWorldAnchor` e o `RopeStops` nascem
//! de **gestos** (*Join* · *Join by drawing* · *Rig* · a roldana · o pino de mundo · o batente).
//!
//! ⛔ **Um deles anexado pelo `+` é pior que ausente**, e é isso que separa esta poda de uma
//! arrumação: um `PhysicsJoint::default()` prende `StableId 0` a `StableId 0` — uma junta que não
//! prende nada, num objeto que pode nem ser corpo. As zonas e os overrides são mais benignos e têm
//! o mesmo defeito de fundo: **o componente entra e nada acontece**, porque o valor neutro é
//! exatamente o que a ausência já dizia.
//!
//! ⇒ a paleta oferece **UMA** entrada de física, e tudo o resto chega com ela ou por um gesto da
//! secção que fala dele:
//!
//! | Onde | O que | Porque não é um item de paleta |
//! |---|---|---|
//! | **`+` → Physics → Physics Body** (`RigidBody`) | *este objeto é simulado* | é a porta |
//! | **§11, ao nascer** (`Collider`) | forma · meias-extensões · offset · densidade · quique · atrito · camada · Solid\|Sensor | as opções **são** as rows da §11, e ela nasce com o corpo |
//! | **§11, face vazia** (`Collider` sozinho) | *Add Shape to X* — esta forma é mais uma peça do corpo acima | ⛔ a paleta não sabe **NOMEAR o dono**, e sem o nome o gesto não existe |
//! | **§14, face vazia** (`PlatformPlayer`) | *Make Platform Player* | ⛔ só faz sentido num corpo `Dynamic` (a mola é um impulso), e o `+` genérico não sabe perguntar isso |
//!
//! ⚠️ **As duas faces vazias VOLTARAM**, e é uma reversão parcial e declarada da F3 do ADR-0166
//! (*«o Inspector mostra o que o objecto TEM»*): a §11 volta a aparecer num filho de um corpo, e a
//! §14 em todo corpo `Dynamic`. O que **não** volta é a doença que a F3 curou — um objecto pelado
//! não mostra secção de física nenhuma, e as doze secções de zeros continuam mortas.
//!
//! ⚠️ **O rótulo passa a nomear a SECÇÃO que nascerá**, que é a regra do campo
//! [`crate::ComponentDesc::display_name`] (*nomeado pelo resultado, não pelo tipo Rust*):
//! `Rigid Body` → **Physics Body** (o cabeçalho da §11, e a palavra do botão que o dono conhecia).
//! O `Collider` fica **Collision Shape** para o rótulo *«brings …»* dizer o que de facto chega.
//!
//! ⚠️ **`applies_to` é `ANY` de propósito, e é a medição que o manda:** a ponte da física vê
//! as entidades por *query* (`BodyQuery = (Entity, &RigidBody, &Collider, &Transform)`), e
//! nada nela pergunta que tipo de objeto é — **uma peça materializada com `RigidBody`
//! funciona sem tocar na ponte** (doc 04 §1.2, item 4). Um caminho vetorial, um objeto Flip
//! e um objeto vazio podem todos ser corpos. Estreitar isto para `IMAGE` seria escrever na
//! tabela uma limitação que o código não tem.
//!
//! # ✅ As quatro referências por identidade — DECLARADAS (F1 migrou, F4.2 remapeia)
//!
//! `PhysicsJoint.body_a/b` e `PulleyWheel.rope/.body` **eram** `stable_name_id` — hash do
//! `Name` —, e era por isso que *a junta de uma cópia prendia os corpos do MESTRE* (a cópia
//! recebe nome `" (1)"`, o hash muda, e a referência continuava a nomear o original). A F1
//! migrou-os para [`ph2d_ecs::StableId`], e desde a **F4.2** eles estão declarados aqui com
//! [`crate::RefKind::Object`].
//!
//! ⚠️ **É esta declaração que FAZ o remap acontecer**, e não uma lista de casos escrita à mão
//! do outro lado: a tabela de remapeadores da shell é conferida contra estes campos por um
//! censo de dois lados, então declarar uma referência nova sem quem a reescreva **reprova**.
//! [`ph2d_ecs::StableId`]: https://github.com/dibrioli/PH2D/blob/main/crates/ph2d-ecs/src/stable_id.rs

use crate::{
    ComponentCategory as C, ComponentDesc as D, FieldDesc, FieldKind as K, ObjectKinds as O,
    Propagation, RefKind,
};

/// **Uma referência a outro objeto**, por `StableId` — ver o cabeçalho do módulo.
///
/// ⚠️ `Propagate`: o campo segue o mestre **depois de remapeado**. Não é `RuntimeOwned` — a
/// referência é AUTORIA (*"prende neste corpo"*), e quem o solver possui é a pose, não o elo.
const fn r(field_id: u16, name: &'static str) -> FieldDesc {
    FieldDesc {
        field_id,
        name,
        kind: K::Ref,
        policy: Propagation::Propagate,
        is_ref: Some(RefKind::Object),
    }
}

/// **`PhysicsJoint`** — os dois corpos que ele prende. ⚠️ Os `field_id` seguem a ordem de
/// declaração da struct (`joint.rs`), para que uma wave que descreva o resto do tipo não tenha
/// de saltar por cima destes dois.
const JOINT: &[FieldDesc] = &[r(1, "Body A"), r(2, "Body B")];

/// **`PulleyWheel`** — a CORDA a que ela pertence (`rope`) e o CORPO em que é montada
/// (`body`, `0` = pregada no cenário). Os ids seguem a struct (`components/rope.rs`):
/// `rope` é o 1.º campo e `body` é o 7.º — os do meio ficam por descrever, e o id declarado
/// é precisamente o que torna isso seguro.
const PULLEY_WHEEL: &[FieldDesc] = &[r(1, "Rope"), r(7, "Body")];

/// ⭐ **Uma LINHA DE SECÇÃO, não um item de paleta** — ver a tabela das três portas no cabeçalho.
///
/// O componente existe, é editadíssimo e **não se escolhe**: quem o anexa (e o destaca no neutro)
/// é o knob da §11/§12/§13/§14 que fala dele. Anexá-lo pelo `+` seria, no melhor caso, um clique
/// que não muda nada — e no pior (`PhysicsJoint`) uma junta a prender coisa nenhuma.
///
/// ⚠️ **Continua a poder ter secção** ([`crate::ComponentDesc::may_have_section`]): `Intrinsic` diz
/// *não se oferece*, nunca *não se edita*. É precisamente a distinção que a terceira variante
/// comprou.
const fn i(canonical_name: &'static str, display_name: &'static str) -> D {
    D::intrinsic(canonical_name, display_name, C::Physics, &[])
}

/// **A PORTA, e o que ela não funciona sem** — ver [`D::requires`].
///
/// ⚠️ Irmã do [`i`] pelo lado oposto: ali o componente não se escolhe, aqui ele escolhe-se *e traz
/// companhia*. (Era *«irmã do `p`»*, o helper autorado que a poda de 13/09 apagou — quando o vizinho
/// que dá nome a uma nota deixa de existir, o que se corrige é a nota.)
///
/// ⚠️ **UMA entrada em toda a família, e ela é uma QUERY.** A ponte consulta
/// `(Entity, &RigidBody, &Collider, &Transform)`: um corpo sem collider nunca entra no solver. ⛔ A
/// barra é *inerte sem aquele*, nunca boa prática — as zonas, os joints e os markers ficam de fora
/// de propósito. (Eram duas até 2026-09-14: o `PlatformPlayer` exigia o `RigidBody` pela mesma
/// query, e deixou de precisar de o declarar quando a porta dele passou a ser um botão que só
/// existe **sobre** um corpo.)
const fn pr(
    canonical_name: &'static str,
    display_name: &'static str,
    requires: &'static [&'static str],
) -> D {
    D::authored_requiring(
        canonical_name,
        display_name,
        C::Physics,
        O::ANY,
        &[],
        requires,
    )
}

/// Ordenado por `canonical_name` (gate `the_catalog_is_sorted_and_unique`).
///
/// ⚠️ **UMA porta e 31 `Intrinsic`** — a tabela vive no cabeçalho, e o gate
/// [`tests::the_physics_family_offers_one_door_and_not_its_rows`] impede que a segunda entre sem
/// alguém decidir.
pub const DESCS: &[D] = &[
    // As SETE da zona — todas rows do bloco de área da §11, pintadas quando o collider é
    // *Sensor* e escritas pelos `PhysicsFieldEdit::Force*/AreaDrag/AreaDensity/AreaFormDrag/
    // AreaTorque/AreaFalloff/ForceWorldAxes`. ⚠️ Fora de um sensor elas **não têm leitor**: a
    // ponte não lê `AreaEffector` de uma peça nem de um corpo sólido.
    i("ph2d::physics::AreaBuoyancy", "Buoyancy Zone"),
    i("ph2d::physics::AreaDrag", "Drag Zone"),
    i("ph2d::physics::AreaEffector", "Force Zone"),
    i("ph2d::physics::AreaFalloff", "Zone Falloff"),
    i("ph2d::physics::AreaForceWorldAxes", "Zone World Axes"),
    i("ph2d::physics::AreaFormDrag", "Form Drag Zone"),
    i("ph2d::physics::AreaTorque", "Torque Zone"),
    i("ph2d::physics::Ccd", "Continuous Collision"),
    // ⭐⭐ **A forma do corpo — e ela CHEGA COM ELE** (ordem do dono, 2026-09-14). Todas as opções
    // dela (forma, meias-extensões, offset, densidade, quique, atrito, camada, Solid|Sensor) são
    // rows da §11, que nasce com o `Physics Body`. A outra vida dela — *esta forma é mais uma peça
    // do corpo ancestral* (W-Compound) — é o botão **Add Shape to X** da face vazia da §11, que
    // NOMEIA o dono; a paleta genérica não sabe nomeá-lo, e por isso aquela é a porta.
    //
    // ⚠️ **Ser `Intrinsic` e ser dependência do `RigidBody` deixou de ser contraditório:** o gate
    // `every_declared_requirement_names_a_real_component` exigia `Authored` e o recurso de que ele
    // falava é OUTRO — quem constrói a cascata é o `insert_default` do REGISTO, que não consulta o
    // `attach`. Hoje ele exige o que de facto é preciso: que o registo saiba construir o alvo.
    i("ph2d::physics::Collider", "Collision Shape"),
    i("ph2d::physics::DampingOverride", "Damping"),
    // ⚠️⚠️ `Dominance` e `MassOverride` são `Intrinsic` por uma CERCA, não por falta de
    // desenho — e a cerca está escrita no doc-comment deles (`components/overrides.rs`):
    // *"absent = the neutral default and the Inspector detaches it at 0 (a project file
    // stays free of the no-op)"*. Neles a PRESENÇA é que carrega o sentido, e o valor de
    // anexação teria de vir do CONTEXTO (a massa que o corpo tem agora) — que a paleta
    // genérica não conhece. ⇒ **A porta por-seção deles não é redundante com o `+`**: ela
    // SEMEIA do valor vivo, que é uma coisa que o `+` não pode fazer (ADR-0166).
    D::intrinsic("ph2d::physics::Dominance", "Dominance", C::Physics, &[]),
    i("ph2d::physics::GravityScale", "Gravity Scale"),
    i("ph2d::physics::InitialVelocity", "Initial Velocity"),
    // O pino de MUNDO: ele chega com o gesto que prega a junta ao cenário (`joint_world`), e
    // sozinho não nomeia ponto nenhum.
    i("ph2d::physics::JointWorldAnchor", "World Anchor"),
    i("ph2d::physics::LockPositionX", "Lock Position X"),
    i("ph2d::physics::LockPositionY", "Lock Position Y"),
    i("ph2d::physics::LockRotation", "Lock Rotation"),
    // Irmã do `Dominance` acima, pela mesma cerca (a massa e a densidade são a mesma
    // grandeza por dois caminhos; ausente = a densidade manda).
    D::intrinsic("ph2d::physics::MassOverride", "Mass", C::Physics, &[]),
    i("ph2d::physics::MaterialCombine", "Material Combine"),
    i("ph2d::physics::NoWallCling", "No Wall Cling"),
    i("ph2d::physics::OneWayPlatform", "One-Way Platform"),
    // ⛔⛔ **A junta é o caso em que oferecer é PIOR que não oferecer.** Ela nasce dos gestos que
    // conhecem os DOIS corpos (*Join Selected Bodies* · *Join by drawing* · *Rig N Parts*), e o
    // ponto neutro do tipo prende `StableId 0` a `StableId 0`: uma junta que não prende nada, num
    // objeto que pode nem ser corpo. ✅ `body_a`/`body_b` seguem DECLARADOS (F4.2) — `Intrinsic`
    // **pode** ter campos, e sem eles a junta de uma instância prende os corpos do mestre (gate
    // `the_instance_joint_binds_the_instances_own_bodies`).
    D::intrinsic("ph2d::physics::PhysicsJoint", "Joint", C::Physics, JOINT),
    // ⭐⭐ **O comportamento — e ele também chega com o corpo** (ordem do dono, 2026-09-14). A §14
    // volta a pintar-se sobre **todo corpo Dynamic**, com ou sem o componente, e a face vazia dela é
    // o botão *Make Platform Player* — que foi a porta original (W5) e morreu na F3.
    //
    // ⚠️ **A porta por-secção faz o que a paleta não podia:** ela só existe onde a física honra o
    // gesto (num `Static` a mola do player é um impulso sobre massa infinita), e o `+` genérico não
    // sabe perguntar isso. ⛔ E anexá-lo pelo `+` a um corpo qualquer era o pior dos dois mundos —
    // um crate com lei de personagem.
    // ⚠️ **`Intrinsic` E com `requires`**, que não é contradição: o `attach` diz *quem escolhe* e o
    // `requires` diz *é inerte sem o quê* — e a porta `attach_by_name` honra a cascata sem
    // consultar o `attach`. Sem esta linha, um chamador que anexasse o player a um objecto pelado
    // deixava-o lá a não fazer nada, em silêncio. Ver [`D::intrinsic_requiring`].
    D::intrinsic_requiring(
        "ph2d::physics::PlatformPlayer",
        "Platform Player",
        C::Physics,
        &[],
        &["ph2d::physics::RigidBody"],
    ),
    i("ph2d::physics::PlayerMode", "Player Mode"),
    i("ph2d::physics::PlayerSignals", "Player Signals"),
    // ⭐⭐⭐ **O PROJÉCTIL de arcade** (TOP-20 #14). ⚠️ `RigidBody` é requerido pela mesma razão
    // dos dois movers irmãos: sem corpo não há o que mover nem em que bater, e a paleta anexa os
    // dois de uma vez em vez de entregar um componente inerte.
    pr(
        "ph2d::physics::ProjectileMotion",
        "Projectile Motion",
        &["ph2d::physics::RigidBody"],
    ),
    // ✅ `rope`/`body` idem — e a roldana é a SEXTA consulta da ponte, a que a refutação não
    // nomeava (F4.1): ela é alcançada pelo nome da corda, então uma referência por remapear
    // faria a roldana da instância disputar a corda do mestre. ⛔ `Intrinsic` pela razão da junta:
    // ela chega pelo botão da §12, que já tem a corda em mãos.
    D::intrinsic(
        "ph2d::physics::PulleyWheel",
        "Pulley Wheel",
        C::Physics,
        PULLEY_WHEEL,
    ),
    // ⭐ **PORTA 1 — a que o dono conhecia pelo nome.** Era o botão *Add Physics Body* da face
    // vazia da §11; desde o ADR-0166/F3 a rota é o `+`, e o rótulo passa a dizer o que a secção
    // que nasce diz.
    pr(
        "ph2d::physics::RigidBody",
        "Physics Body",
        &["ph2d::physics::Collider"],
    ),
    i("ph2d::physics::RopeStops", "Rope Stops"),
    i("ph2d::physics::SignalOnHit", "Signal on Hit"),
    i("ph2d::physics::SignalOnLeave", "Signal on Leave"),
    i("ph2d::physics::SignalTagFilter", "Signal Tag Filter"),
    // ⭐⭐⭐ **O mover de VISTA DE CIMA** (TOP-20 #13). ⚠️ `RigidBody` é requerido pela
    // mesma razão do irmão `PlatformPlayer`: sem corpo não há o que mover, e a paleta
    // anexa os dois de uma vez em vez de entregar um componente inerte.
    pr(
        "ph2d::physics::TopDownPlayer",
        "Top-Down Player",
        &["ph2d::physics::RigidBody"],
    ),
    i("ph2d::physics::WalkSurface", "Walk Surface"),
    i("ph2d::physics::WestonAxle", "Weston Axle"),
];

#[cfg(test)]
mod tests {
    use super::DESCS;

    /// ⭐⭐ **A família da física oferece TRÊS portas, e nenhuma linha de secção.**
    ///
    /// O gate que faltava em 2026-08-24, quando a F3 escreveu a família inteira com o helper
    /// autorado: **30 itens na paleta, e as opções de UM objecto de física espalhadas por eles**. O
    /// report do dono (13/09) nomeou-o *«picotado, dividido em inúmeros supostos componentes»*, e o
    /// veredito dele (14/09) fixou o número: *«um objeto de física (Physics Body) e todas as opções
    /// aparecem com ele»*.
    ///
    /// ⚠️ **É uma catraca NOMEADA, e a lista é a afirmação** — não um número. Uma família de
    /// física que ganhe uma segunda porta reprova aqui e obriga a decidir *isto é uma intenção do
    /// artista, ou é uma row/gesto da secção?*; e se a porta desaparecer reprova pelo mesmo
    /// `assert_eq`, que é a **metade de obsolescência** (a lista deixaria de descrever a tabela).
    ///
    /// ⚠️ **O piso de população é a outra metade:** sem ele, apagar a família inteira deixaria o
    /// gate a comparar dois vazios — *um zero de «não medido» e um de «perfeito» são o mesmo byte*.
    ///
    /// (Mutação: trocar um `i(...)` por `D::authored(...)` ⇒ o `assert_eq` reprova, nomeando-o.)
    #[test]
    fn the_physics_family_offers_one_door_and_not_its_rows() {
        // ⭐ **TRÊS portas desde 16/09** — as duas novas são os CONTROLADORES canónicos
        //    (TOP-20 #13 e #14), e a resposta às duas metades da pergunta acima é a mesma: um
        //    mover é uma **intenção** do artista (ele escolhe *este objecto é um jogador de
        //    vista de cima*), nunca uma row que a §11 já pinta. ⚠️ Os dois pedem `RigidBody`
        //    junto (`pr`), senão a paleta entregava um componente inerte.
        const PORTAS: [&str; 3] = [
            "ph2d::physics::ProjectileMotion",
            "ph2d::physics::RigidBody",
            "ph2d::physics::TopDownPlayer",
        ];
        let offered: Vec<&str> = DESCS
            .iter()
            .filter(|d| d.is_offered())
            .map(|d| d.canonical_name)
            .collect();
        assert_eq!(
            offered,
            PORTAS.to_vec(),
            "a paleta de FISICA tem de oferecer exatamente estas TRES portas -- o `Physics Body` e os \
             dois controladores canonicos.\n\
             Um componente novo aqui e' uma pergunta, nao um esquecimento: e' uma INTENCAO do \
             artista (entao e' `pr`, e esta lista cresce com ele), ou e' uma ROW / um GESTO que a \
             §11/§12/§13/§14 ja' oferece (entao e' `i(..)`)?"
        );
        assert!(
            DESCS.len() >= 30,
            "a familia encolheu para {} — o gate acima passaria a comparar duas listas quase \
             vazias e a nao afirmar nada",
            DESCS.len()
        );
    }
}
