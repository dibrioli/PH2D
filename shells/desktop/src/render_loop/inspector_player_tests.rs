//! **A SEQUÊNCIA leva a algum lugar** (W5) — a quarta condição de UI que a
//! política deste módulo exige, e a que as outras três não implicam.
//!
//! O seam prova que o clique chega ao barramento; a paridade prova que o widget
//! é registrado; o `every_physics_component_is_authorable` prova que alguém o
//! escreve. **Nenhum dos três prova que o gesto INTEIRO produz um personagem que
//! anda** — foi essa a categoria que pegou o passo *"converta para Capsule"* que
//! quase entrou num roteiro de smoke: geometricamente correto, e destruía o
//! tronco.

use super::inspector_player::{apply_player_edit, attach_player, build_player_info};
use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_editor::PlayerFieldEdit;
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, PlatformPlayer, RigidBody};

/// **A premissa desta fixture, declarada uma vez** — todo corpo aqui é
/// `Dynamic` e vira player pela porta do Inspector, então a lei corre nele com
/// a perna ELÁSTICA. Passá-la a cada chamada seria repetir trinta vezes o que
/// é um fato do arquivo; passá-la ERRADA deixaria verdes, pelo motivo errado,
/// os gates que leem `reaction_is_live`/`push_is_live`/`spring_is_live`.
const SPRUNG: ph2d_physics_ecs::PlayerLiveness = ph2d_physics_ecs::PlayerLiveness::SPRING;

fn body(kind: BodyKind, shape: ColliderShape) -> (SimWorld, u64) {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((
            Name::new("Hero"),
            RigidBody { kind },
            Collider {
                shape,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 0.0)),
        ))
        .id();
    (sim, e.to_bits())
}

const CAPSULE: ColliderShape = ColliderShape::Capsule {
    half_height: 0.3,
    radius: 0.2,
};

/// **O gesto inteiro:** um corpo Dynamic **não tem** a §14, anexar o componente fá-la aparecer, e
/// os números vêm do ponto de partida da LEI.
///
/// ⚠️ **A primeira metade INVERTEU na F3** (ADR-0166). Este gate afirmava *"todo corpo Dynamic tem
/// a §14"* — a **face vazia**, cujo botão «Make Platform Player» era a única rota para a feature.
/// Hoje a seção segue o componente, e a rota é o `+` do cabeçalho.
#[test]
fn attaching_the_player_opens_the_section_at_the_laws_starting_point() {
    let (mut sim, bits) = body(BodyKind::Dynamic, CAPSULE);
    assert!(
        build_player_info(&sim, bits, 0.0, 0.0, None, SPRUNG).is_none(),
        "um corpo SEM o componente nao pode ter a §14 — era a face vazia da pre-F3"
    );

    attach_player(&mut sim, bits);
    let after = build_player_info(&sim, bits, 0.0, 0.0, None, SPRUNG)
        .expect("com o componente a seccao aparece");
    assert!(after.has_player);
    assert_eq!(after.speed, 6.0, "a velocidade do ponto de partida");
    assert_eq!(after.max_slope_deg, 45.0);
}

/// ⚠️ **E ele nasce PAIRANDO, não tangente.**
///
/// O ponto de partida do modelo (`0,5`) deixa esta cápsula exatamente tangente
/// ao chão — ela não flutua, e só uma rampa revela. O `Add` conhece a forma e
/// sobe a altura acima do piso; sem isso a primeira impressão do artista seria
/// um personagem encostado num app cuja tese é que ele paira.
#[test]
fn a_new_player_floats_over_its_own_collider() {
    let (mut sim, bits) = body(BodyKind::Dynamic, CAPSULE);
    attach_player(&mut sim, bits);
    let info = build_player_info(&sim, bits, 0.0, 0.0, None, SPRUNG).unwrap();
    assert!(
        info.min_float_known,
        "uma capsula tem piso computavel — sem isto o resto do gate nao diz nada"
    );
    assert!(
        info.float_height > info.min_float_height,
        "ele tem de nascer ACIMA do piso: {:.3} vs o minimo {:.3}",
        info.float_height,
        info.min_float_height
    );
}

/// O botão de ajuste conserta uma altura curta autorada à mão.
#[test]
fn fit_to_collider_raises_a_short_float_height() {
    let (mut sim, bits) = body(BodyKind::Dynamic, CAPSULE);
    attach_player(&mut sim, bits);
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::FloatHeight(0.2));
    let short = build_player_info(&sim, bits, 0.0, 0.0, None, SPRUNG).unwrap();
    assert!(
        short.float_height < short.min_float_height,
        "a fixture TEM de conter o defeito"
    );

    apply_player_edit(&mut sim, bits, PlayerFieldEdit::FitFloatHeight);
    let fixed = build_player_info(&sim, bits, 0.0, 0.0, None, SPRUNG).unwrap();
    assert!(
        fixed.float_height > fixed.min_float_height,
        "o ajuste tem de passar do piso: {:.3} vs {:.3}",
        fixed.float_height,
        fixed.min_float_height
    );
}

/// ⚠️ **A §14 segue o COMPONENTE, e não o tipo do corpo** — e isto é uma REVERSÃO medida da F3.
///
/// Até aqui a regra era *"Dynamic, com ou sem o componente; e nunca um Static"*, porque a mola é um
/// impulso e um impulso não move massa infinita. Aquela era a condição de **OFERECER O BOTÃO**, e o
/// botão mudou-se para a paleta: mantê-la produziria o pior dos dois mundos — o artista anexa o
/// componente pelo `+` e **nada aparece**. *Um componente presente e invisível lê-se como defeito.*
///
/// A física continua verdadeira; ela é assunto da §11, que é onde o tipo do corpo se muda.
///
/// (Mutação: pôr `kind != Static` de volta na `player_section_applies` ⇒ o `Static` reprova.)
#[test]
fn the_section_follows_the_component_not_the_body_kind() {
    for kind in [BodyKind::Static, BodyKind::Kinematic, BodyKind::Dynamic] {
        let (mut sim, bits) = body(kind, CAPSULE);
        assert!(
            build_player_info(&sim, bits, 0.0, 0.0, None, SPRUNG).is_none(),
            "sem o componente nao ha' §14, nem num {kind:?}"
        );
        attach_player(&mut sim, bits);
        assert!(
            sim.world()
                .get::<PlatformPlayer>(ph2d_ecs::Entity::from_bits(bits))
                .is_some(),
            "a porta da paleta anexa em qualquer corpo — {kind:?}"
        );
        assert!(
            build_player_info(&sim, bits, 0.0, 0.0, None, SPRUNG).is_some(),
            "com o componente a §14 aparece — mesmo num {kind:?}"
        );
    }
}

/// **Remover devolve o corpo a um corpo comum — e FECHA a seção.**
///
/// ⚠️ **A segunda metade inverteu na F3:** ela dizia *"a seção continua viva, com a face vazia,
/// para que ele possa voltar a ser um player"*. A rota de volta é agora o `+` do cabeçalho, e uma
/// seção vazia sobre um componente ausente é exatamente o que a fase apaga.
#[test]
fn removing_the_behaviour_closes_the_section() {
    let (mut sim, bits) = body(BodyKind::Dynamic, CAPSULE);
    attach_player(&mut sim, bits);
    assert!(build_player_info(&sim, bits, 0.0, 0.0, None, SPRUNG).is_some());
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::Remove);
    assert!(
        build_player_info(&sim, bits, 0.0, 0.0, None, SPRUNG).is_none(),
        "sem o componente a §14 tem de sumir"
    );
}

/// ⚠️ **O amortecimento é clampado no TETO MEDIDO da lei** — acima dele o boost
/// inverte a velocidade em vez de matá-la, e o personagem pipoca.
///
/// Duas camadas: a porta da lei também clampa. Esta existe para o número
/// AUTORADO nunca guardar algo que o motor não vai honrar — um campo que mente
/// sobre si mesmo é pior que um clamp invisível.
#[test]
fn the_damping_is_clamped_to_the_measured_ceiling() {
    let (mut sim, bits) = body(BodyKind::Dynamic, CAPSULE);
    attach_player(&mut sim, bits);
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::SpringDamping(5.0));
    let info = build_player_info(&sim, bits, 0.0, 0.0, None, SPRUNG).unwrap();
    assert_eq!(
        info.spring_damping,
        ph2d_physics_ecs::RideConfig::MAX_DAMPING
    );
}

/// ⚠️ **Uma CAIXA não tem piso computável, e o info o diz** em vez de devolver
/// a fórmula da cápsula.
///
/// A extensão de uma cápsula ao longo de uma normal é `radius + hh·cos θ` (o
/// raio é isotrópico) e a de uma caixa é `half_x·sin θ + half_y·cos θ` — outra
/// fórmula, outro piso. Um número errado apresentado como certo é pior que a
/// ausência dele.
#[test]
fn a_box_reports_no_known_floor() {
    let (mut sim, bits) = body(
        BodyKind::Dynamic,
        ColliderShape::Cuboid {
            half_x: 0.3,
            half_y: 0.5,
        },
    );
    attach_player(&mut sim, bits);
    let info = build_player_info(&sim, bits, 0.0, 0.0, None, SPRUNG).unwrap();
    assert!(!info.min_float_known);
}

/// ⚠️ **As duas assistências da W10 chegam ao COMPONENTE, e o clamp é o do
/// barramento** — a quarta condição para a wave nova.
///
/// O seam prova que o número levanta a edição; este prova que a edição pousa no
/// personagem, e que um valor negativo (que o chip aceita digitar) vira zero em
/// vez de um alcance ao contrário.
#[test]
fn the_two_w10_assists_land_on_the_component_and_are_clamped() {
    let (mut sim, bits) = body(BodyKind::Dynamic, CAPSULE);
    attach_player(&mut sim, bits);

    apply_player_edit(&mut sim, bits, PlayerFieldEdit::CornerReach(0.2));
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::LiftMomentum(0.8));
    let info = build_player_info(&sim, bits, 0.0, 0.0, None, SPRUNG).unwrap();
    assert!((info.corner_reach - 0.2).abs() < 1.0e-6, "{info:?}");
    assert!((info.lift_momentum - 0.8).abs() < 1.0e-6, "{info:?}");

    // Negativo não é uma direção nem uma janela: vira o desligado.
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::CornerReach(-1.0));
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::LiftMomentum(-1.0));
    let clamped = build_player_info(&sim, bits, 0.0, 0.0, None, SPRUNG).unwrap();
    assert_eq!((clamped.corner_reach, clamped.lift_momentum), (0.0, 0.0));
}

/// **O `Fit Crouch` semeia a altura agachada pelo MESMO piso da perna de pé**
/// (W18).
///
/// ⚠️ **A mesma função, e é o desenho inteiro:** o piso é da FORMA e da rampa, não
/// de qual perna está em uso. Uma segunda fórmula para a de baixo divergiria no
/// dia em que a caixa ganhasse a dela.
///
/// ⚠️ **A PERNA DE PÉ é deliberadamente ALTA na fixture, e é isso que dá dentes
/// ao gate.** A primeira versão fazia `FitFloatHeight` antes, então a perna já
/// estava no valor ajustado — e *"semear do piso"* e *"copiar a perna de pé"*
/// davam **o mesmo número**: a mutação `p.crouch_height = p.float_height`
/// **sobreviveu**, sobre um gate escrito exatamente para a pegar. Com a perna em
/// `1,50` as duas respostas ficam a um metro de distância.
///
/// ⚠️ **Mutações medidas:** `p.float_height` no lugar do `fitted_float` dá `1,500`
/// contra os `0,600` do piso; e um `fitted_float` sem o `min` deixa o agachar
/// passar da perna (gate irmão).
#[test]
fn fitting_the_crouch_seeds_it_from_the_floor_not_from_the_standing_leg() {
    let (mut sim, bits) = body(BodyKind::Dynamic, CAPSULE);
    attach_player(&mut sim, bits);
    // Uma perna de pé BEM acima do que o piso pede — a premissa que separa as
    // duas respostas possíveis.
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::FloatHeight(1.50));
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::CrouchHeight(0.10));

    apply_player_edit(&mut sim, bits, PlayerFieldEdit::FitCrouchHeight);
    let info = build_player_info(&sim, bits, 0.0, 0.0, None, SPRUNG).unwrap();

    assert!(
        info.crouch_height > info.min_float_height,
        "o Fit deixou o agachar ABAIXO do piso ({:.3} <= {:.3}): e' exatamente o \
         estado que ele existe para consertar",
        info.crouch_height,
        info.min_float_height
    );
    assert!(
        info.crouch_height < info.float_height * 0.5,
        "o Fit copiou a PERNA DE PE ({:.3} contra {:.3}) em vez de derivar o piso -- \
         um agachar que nao agacha",
        info.crouch_height,
        info.float_height
    );
}

/// **E o `Fit Crouch` nunca passa da perna de pé** — a metade que só morde numa
/// cápsula cujo piso já está acima da altura autorada.
///
/// ⚠️ Sem o `min`, um corpo gordo (piso alto) com uma perna curta receberia um
/// agachar mais alto do que estar em pé, e o `crouch_step` passaria a LEVANTAR o
/// personagem quando ele segurasse BAIXO.
#[test]
fn a_fitted_crouch_never_rises_above_the_standing_leg() {
    let (mut sim, bits) = body(BodyKind::Dynamic, CAPSULE);
    attach_player(&mut sim, bits);
    // Uma perna deliberadamente MAIS CURTA que o piso da forma.
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::FloatHeight(0.20));
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::CrouchHeight(0.10));
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::FitCrouchHeight);
    let info = build_player_info(&sim, bits, 0.0, 0.0, None, SPRUNG).unwrap();
    assert!(
        info.crouch_height <= info.float_height,
        "o agachar ({:.3}) passou da perna de pe ({:.3})",
        info.crouch_height,
        info.float_height
    );
}

/// **A §14 NÃO É CASA PARA A CORRIDA GRAVADA** (W25) — a medição que abriu a
/// wave, e ela afirma um DEFEITO do desenho antigo, não uma cura.
///
/// A fita de entrada é um fato do **DOCUMENTO**: ela viaja no arquivo (W17) e é
/// o que o Bake replaya (W16). Os dois botões que a governam moravam só aqui,
/// numa seção que **só existe sobre um corpo Dynamic selecionado** — e este gate
/// mede exatamente isso: apagar o personagem, ou selecionar qualquer outra
/// coisa, e a corrida fica presa no documento sem gesto que a alcance.
///
/// ⚠️ **O oráculo é a AUSÊNCIA da seção**, não a ausência do botão: com
/// `build_player_info` em `None` o painel não pinta nada, então nenhum readout e
/// nenhum verbo existem — é por isso que a cura teve de ser uma segunda VISTA
/// noutro painel, e não um botão a mais dentro deste.
///
/// **Mutação que deve sangrar:** fazer o `build_player_info` devolver `Some`
/// para qualquer entidade — o que consertaria o alcance e traria de volta a §14
/// sobre um corpo Static, onde a mola (um impulso) não move massa infinita.
#[test]
fn the_player_section_is_no_home_for_a_document_wide_run() {
    // 1. O corpo não é Dynamic: a §14 não existe, e com ela nenhum verbo de fita.
    let (sim, bits) = body(BodyKind::Static, CAPSULE);
    assert!(
        build_player_info(&sim, bits, 4.0, 0.0, None, SPRUNG).is_none(),
        "a §14 nasceu sobre um corpo Static -- a mola e' um impulso e nao move \
         massa infinita"
    );

    // 2. E o personagem foi APAGADO, que é o caso que fecha o argumento: a
    //    corrida sobrevive a ele (está no arquivo), a seção não.
    let (mut sim, bits) = body(BodyKind::Dynamic, CAPSULE);
    attach_player(&mut sim, bits);
    assert!(
        build_player_info(&sim, bits, 4.0, 0.0, None, SPRUNG).is_some(),
        "o controle: enquanto o player existe a §14 mostra a corrida"
    );
    sim.world_mut().despawn(ph2d_ecs::Entity::from_bits(bits));
    assert!(
        build_player_info(&sim, bits, 4.0, 0.0, None, SPRUNG).is_none(),
        "apagar o personagem tinha de levar a §14 embora -- e' isso que prendia \
         a corrida"
    );
}

/// **O que decide COMO este personagem é movido** — o `PlayerMode` e as duas
/// consequências dele (a 3ª lei e o empurrão lateral), cortadas para um irmão
/// por TETO DE LOC (a shell mede 600).
///
/// ⚠️ **FILHO e não irmão de módulo**, o precedente do `pose_owner_tests`: o
/// `body()` e a `CAPSULE` deste arquivo são a porta pela qual as duas metades
/// montam a mesma cena, e uma segunda cópia delas é como as duas famílias
/// passariam a testar corpos diferentes sem ninguém notar.
#[path = "inspector_player_mode_tests.rs"]
mod mode;

/// **REMOVER devolve o corpo ao que ele era, e o caso que decide é o
/// CINEMÁTICO** (auditoria de 2026-08-15).
///
/// ⚠️ **O `Mode` escreve DUAS metades** (o `PlayerMode` e o `RigidBody.kind`) e o
/// `Remove` desfazia UMA. O que sobrava era um corpo `Kinematic` sem
/// `PlatformPlayer` — e esse é exatamente o estado que a §14 **NÃO OFERECE**
/// (ele é dirigido pela cena), então o artista removia o comportamento e ficava
/// **preso**, sem controle nenhum na tela para o trazer de volta. É o mesmo beco
/// sem saída que a W-KinMove consertou para o gesto do MODO, reaberto pela porta
/// do lado.
///
/// ⚠️ E a segunda consequência é silenciosa: o corpo **deixa de cair**, e um
/// `PlayerMode` órfão viaja no arquivo a descrever um personagem que não existe.
#[test]
fn removing_the_behaviour_gives_a_plain_dynamic_body_back() {
    let (mut sim, bits) = body(BodyKind::Dynamic, CAPSULE);
    attach_player(&mut sim, bits);
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::Mode(1));
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::EmitSignals(true));
    let e = ph2d_ecs::Entity::from_bits(bits);
    // A fixture TEM de conter o fenômeno: as duas metades foram escritas.
    assert_eq!(
        sim.world().get::<RigidBody>(e).unwrap().kind,
        BodyKind::Kinematic,
        "o Mode escreve o kind -- sem isso o resto do gate nao diz nada"
    );

    apply_player_edit(&mut sim, bits, PlayerFieldEdit::Remove);

    assert!(sim.world().get::<PlatformPlayer>(e).is_none());
    assert!(
        sim.world().get::<ph2d_physics_ecs::PlayerMode>(e).is_none(),
        "um PlayerMode orfao descreve um personagem que nao existe -- e VIAJA no arquivo"
    );
    assert!(
        sim.world()
            .get::<ph2d_physics_ecs::PlayerSignals>(e)
            .is_none(),
        "e o opt-in de sinais e' da §14: sem ela ele nao tem quem o desligue"
    );
    assert_eq!(
        sim.world().get::<RigidBody>(e).unwrap().kind,
        BodyKind::Dynamic,
        "⚠️ o corpo tem de VOLTAR a cair -- Kinematic sem player e' dirigido pela \
         CENA, e a §14 nao e' oferecida ali: o artista fica PRESO"
    );
    // ⚠️ **E a seção FECHA** (F3): a porta de volta é o `+` do cabeçalho, não uma face vazia.
    assert!(
        build_player_info(&sim, bits, 0.0, 0.0, None, SPRUNG).is_none(),
        "sem o componente a §14 tem de sumir"
    );
}
