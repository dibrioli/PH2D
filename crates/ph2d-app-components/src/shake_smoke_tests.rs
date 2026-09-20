//! Os gates da cena do abanão. ⚠️ Eles medem o que a cena **MONTA**, não o que ela imprime — e a
//! régua mais importante é a que nenhuma das irmãs tem: *o abanão chega a ser VISTO?*

use super::*;
use ph2d_ecs::{CameraRuntime, SimWorld, StableId};

fn montada() -> (SimWorld, Montada) {
    let mut sim = SimWorld::new();
    let m = montar(sim.world_mut(), 1);
    (sim, m)
}

/// ⭐⭐⭐ **O PÁTIO existe, e a régua é a POPULAÇÃO** — sobre um chão de cor chapada um abanão é
/// rigorosamente invisível, e a cena ensinaria *«o abanão não funciona»*.
#[test]
fn a_cena_tem_com_que_se_ver_o_abanao() {
    let (mut sim, _) = montada();
    let mundo = sim.world_mut();
    let postes = mundo
        .query::<&Name>()
        .iter(mundo)
        .filter(|n| n.0.starts_with("Post "))
        .count();
    let esperado = usize::try_from(POSTES_X * POSTES_Y).unwrap();
    assert_eq!(postes, esperado, "o pátio é o que torna o abanão visível");
    assert!(
        esperado >= 20,
        "com poucos postes o dono pode parar num canto vazio e não ver nada: {esperado}"
    );
}

/// ⚠️ **E eles têm de COBRIR a vista** — a régua é a do doc: a grelha tem de ser mais larga que o
/// enquadramento de fábrica, senão há sítios onde o abanão fica invisível.
#[test]
fn o_patio_cobre_mais_do_que_a_vista_enquadra() {
    #[allow(clippy::cast_precision_loss)]
    let largura = (POSTES_X - 1) as f32 * POSTE_PASSO;
    #[allow(clippy::cast_precision_loss)]
    let altura = (POSTES_Y - 1) as f32 * POSTE_PASSO;
    let vista_h = ph2d_ecs::GameCamera::default().height_world;
    assert!(
        altura >= vista_h,
        "a grelha tem {altura} m de alto e a vista enquadra {vista_h}"
    );
    assert!(largura > altura, "e o ecrã é mais largo que alto");
    // ⭐⭐ **E o PASSO é menor que METADE da vista, e a FOTO é que o corrigiu:** esta régua media a
    // JANELA, e o que o dono vê é a **BANDA** que sobra com a timeline aberta — ~metade dela. Com
    // `5` m de passo a foto mostrou **uma** fileira de postes atrás da bomba.
    assert!(
        POSTE_PASSO < vista_h * 0.5,
        "com passo {POSTE_PASSO} numa BANDA de ~{:.1} m há sítios com uma fileira só",
        vista_h * 0.5
    );
}

/// ⭐⭐⭐ **A cena tem as TRÊS peças do abanão, e nenhuma sozinha faz alguma coisa.**
#[test]
fn a_cena_tem_quem_grite_quem_ouca_e_quem_trema() {
    let (mut sim, _) = montada();
    let mundo = sim.world_mut();
    assert_eq!(
        mundo.query::<&SignalOnAction>().iter(mundo).count(),
        1,
        "quem GRITA"
    );
    assert_eq!(
        mundo.query::<&ShakeEmitter>().iter(mundo).count(),
        1,
        "quem OUVE"
    );
    assert_eq!(
        mundo.query::<&CameraShake>().iter(mundo).count(),
        1,
        "quem TREME"
    );
}

/// ⚠️ **O gatilho e a fonte falam o MESMO nome** — duas consts diferentes seriam uma cena que monta
/// bem e não faz nada, e nenhum gate de presença o veria.
#[test]
fn o_nome_do_sinal_casa_dos_dois_lados() {
    let (mut sim, _) = montada();
    let mundo = sim.world_mut();
    let publicado: Vec<String> = mundo
        .query::<&SignalOnAction>()
        .iter(mundo)
        .flat_map(|t| t.0.iter().map(|r| r.signal.clone()))
        .collect();
    let ouvido: Vec<String> = mundo
        .query::<&ShakeEmitter>()
        .iter(mundo)
        .flat_map(|e| e.0.iter().map(|f| f.on.clone()))
        .collect();
    assert_eq!(
        publicado, ouvido,
        "quem grita e quem ouve têm de dizer o mesmo"
    );
    assert!(!publicado[0].is_empty());
}

/// ⭐⭐ **A cerca é `Myself`, e isso é LEI da cena:** com `Anyone` uma segunda bomba ouviria o
/// estrondo da primeira e a distância deixaria de ser a variável.
#[test]
fn a_fonte_ouve_so_o_proprio_estrondo() {
    let (mut sim, _) = montada();
    let mundo = sim.world_mut();
    let de = mundo.query::<&ShakeEmitter>().iter(mundo).next().unwrap().0[0].de;
    assert_eq!(de, SignalFrom::Myself);
}

/// ⭐⭐⭐ **O alcance da fonte tem de COBRIR a vista e ACABAR dentro do pátio** — as duas metades.
///
/// ⚠️ Sem a primeira, um estrondo ao pé não abana nada e o passo (1) mente; sem a segunda, o dono
/// não consegue **andar para fora** do alcance e o passo (2) é inalcançável.
#[test]
fn o_alcance_deixa_a_cena_ensinar_as_duas_coisas() {
    let (mut sim, _) = montada();
    let mundo = sim.world_mut();
    let f = mundo.query::<&ShakeEmitter>().iter(mundo).next().unwrap().0[0].clone();
    assert!(f.fora > f.dentro, "{:?}", (f.dentro, f.fora));
    assert!(
        ph2d_shake::atenuacao(0.0, f.dentro, f.fora) > 0.9,
        "ao pé tem de chegar inteiro"
    );
    #[allow(clippy::cast_precision_loss)]
    let meia_grelha = (POSTES_X / 2) as f32 * POSTE_PASSO;
    assert!(
        f.fora < meia_grelha * 2.0,
        "o dono tem de conseguir ANDAR para fora do alcance sem sair do pátio: \
         fora = {}, meia-grelha = {meia_grelha}",
        f.fora
    );
}

/// ⭐ **A câmera SEGUE o herói** — sem isso andar não afasta a vista, e o passo (2) da cena não
/// ensina nada.
#[test]
fn a_camera_segue_o_heroi_que_o_dono_conduz() {
    let (mut sim, _) = montada();
    let mundo = sim.world_mut();
    let (_, follow) = mundo
        .query::<(&GameCamera, &CameraFollow)>()
        .iter(mundo)
        .next()
        .expect("a câmera tem de seguir alguém");
    let alvo = follow.target.clone();
    let existe = mundo.query::<&Name>().iter(mundo).any(|n| n.0 == alvo);
    assert!(existe, "a câmera segue «{alvo}», que ninguém na cena tem");
    // ⭐ E o alvo é o objecto que o dono CONDUZ.
    let mundo = sim.world_mut();
    let conduzido = mundo
        .query::<(&Name, &ph2d_physics_ecs::TopDownPlayer)>()
        .iter(mundo)
        .next()
        .map(|(n, _)| n.0.clone());
    assert_eq!(conduzido.as_deref(), Some(alvo.as_str()));
}

/// ⚠️ **Quem nasce escolhido tem a secção que o roteiro nomeia** — a lição do #15.
#[test]
fn quem_nasce_escolhido_tem_a_seccao_do_roteiro() {
    let (sim, m) = montada();
    let e = Entity::from_bits(m.escolhido);
    assert!(
        sim.world().get::<ShakeEmitter>(e).is_some(),
        "o passo (3) manda ver SHAKE EMITTER no escolhido"
    );
    assert!(
        sim.world().get::<Sprite>(e).is_some(),
        "e ele tem de ter CORPO, senão o dedo do dono não lhe chega no canvas"
    );
}

/// ⭐⭐⭐ **A CENA INTEIRA, pelo caminho do PRODUTO:** um estrondo tem de abanar, e o mesmo
/// estrondo a `20` m **não**. ⛔ Este é o único gate que percorre a cena montada com a ponte.
#[test]
fn na_cena_montada_o_estrondo_abana_e_a_distancia_conta() {
    use ph2d_runtime::{Signal, SignalOutbox, SignalReader};
    const DT: f32 = 1.0 / 60.0;

    fn treme_com_a_camera_em(x: f32) -> f32 {
        let mut sim = SimWorld::new();
        montar(sim.world_mut(), 1);
        // A câmera vive onde o `follow` a puser; aqui pousa-se o `CameraRuntime` à mão, que é o que
        // a fase da shell faz no primeiro quadro.
        let cam = ph2d_ecs::active_camera_of(sim.world_mut()).unwrap();
        sim.world_mut().entity_mut(cam).insert(CameraRuntime {
            center: [x, 0.0],
            anchor: [x, 0.0],
            last_target: None,
            velocity: [0.0, 0.0],
            settled: true,
        });
        let bomba = {
            let mundo = sim.world_mut();
            mundo
                .query::<(Entity, &ShakeEmitter)>()
                .iter(mundo)
                .next()
                .unwrap()
                .0
        };
        let mut out = SignalOutbox::new();
        let mut r = SignalReader::at(&out);
        out.publish(Signal::from_action(SINAL, bomba.to_bits(), 0));
        crate::shake_bridge::drive_camera_shake(&mut sim, &out, &mut r, DT).trauma
    }

    let perto = treme_com_a_camera_em(0.0);
    let longe = treme_com_a_camera_em(30.0);
    assert!(
        perto > 0.8,
        "ao pé da bomba tem de abanar forte: {perto:.3}"
    );
    assert_eq!(longe, 0.0, "a 30 m o mesmo estrondo não pode chegar");
    // ⭐ E o MEIO existe — senão a cena é um interruptor e não uma lei.
    let meio = treme_com_a_camera_em(11.0);
    assert!(
        meio > 0.0 && meio < perto,
        "a meio caminho tem de abanar MENOS e não nada: {meio:.3}"
    );
}

/// ⚠️ **Os `StableId` existem** — sem eles a `active_camera_of` devolve `None` e a cena monta sem
/// câmera activa, que é um defeito mudo (a cena parece certa e nada treme).
#[test]
fn a_cena_atribui_identidade() {
    let (mut sim, _) = montada();
    let cam = ph2d_ecs::active_camera_of(sim.world_mut());
    assert!(cam.is_some(), "a cena tem de ter uma câmera ACTIVA");
    assert!(sim.world().get::<StableId>(cam.unwrap()).is_some());
}

/// ⭐⭐⭐ **A BOMBA CABE NA VISTA quando a cena abre** — a régua que a FOTO obrigou a escrever.
///
/// ⛔⛔ **Ela nasceu de um defeito que os DEZ gates acima não viam:** a 1.ª redacção punha o herói a
/// `5` m da bomba, a câmera segue o herói, e a banda de canvas que sobra com a timeline aberta
/// enquadra **~6 m** — logo o quadrado vermelho do passo (1) ficava FORA DO ECRÃ. *Nenhum gate de
/// «a cena tem as três peças» vê isso: as três peças estavam lá.*
///
/// ⚠️ **A barra é a META-ALTURA da vista dividida por DOIS**, e a divisão não é folga: ela é o preço
/// medido de a timeline abrir junto (o prólogo abre-a, porque o passo (6) manda parar a corrida).
#[test]
fn a_bomba_cabe_na_vista_quando_a_cena_abre() {
    let (mut sim, m) = montada();
    let bomba = Entity::from_bits(m.escolhido);
    let pos_bomba = sim.world().get::<Transform>(bomba).unwrap().translation;
    // Onde a câmera vai pousar no 1.º quadro: em cima de quem ela segue (`settled = false`).
    let mundo = sim.world_mut();
    let alvo = mundo
        .query::<(&Name, &ph2d_physics_ecs::TopDownPlayer, &Transform)>()
        .iter(mundo)
        .next()
        .map(|(_, _, t)| t.translation)
        .expect("a cena tem de ter quem a câmera siga");
    let meia = ph2d_ecs::GameCamera::default().height_world * 0.5;
    let dy = (pos_bomba.y - alvo.y).abs();
    assert!(
        dy < meia * 0.5,
        "a bomba está a {dy:.2} m de quem a câmera segue, e com a timeline aberta só cabem \
         ~{:.2} m: o quadrado vermelho do passo (1) fica FORA DO ECRÃ",
        meia
    );
    // ⭐ E o CONTROLO: ela não nasce EM CIMA dele, senão o herói e o realce tapam-na.
    assert!(
        dy > 0.5,
        "a bomba tem de estar VISÍVEL ao lado do herói, não debaixo dele: {dy:.2} m"
    );
}

/// ⭐⭐⭐ **O HERÓI ANDA — o gate que os DOZE acima não tinham, e que o dono apanhou**
/// (*«vc esqueceu de colocar física no jogador»*, 19/09).
///
/// ⛔⛔ **A cena montava as peças certas e o passo (2) do roteiro era IMPOSSÍVEL:** o herói tinha
/// `TopDownPlayer` e **nenhum corpo**, e o [`ph2d_physics_ecs::PhysicsBridge::drive_topdown`] varre
/// `self.bodies` — quem não tem corpo **nunca entra no laço**. Medido na cena de então: com a seta
/// segurada durante `60` tiques o herói andou `dx = 0` e `dy = 0`.
///
/// ⚠️⚠️ **Nenhum dos doze gates o via, e a cegueira tem forma:** eles perguntam *«a entidade tem o
/// COMPONENTE?»* (o `a_camera_segue_o_heroi_que_o_dono_conduz` pergunta exactamente isso) e o
/// consumidor pergunta *«a entidade tem um CORPO?»* — **a lente do gate é mais larga que a do
/// consumidor**, que é a família que o `CLAUDE.md` §5.0 nomeia. A cura é medir o BARRO: conduzir a
/// cena montada pela ponte REAL e olhar o `Transform`.
///
/// **Mutações que devem sangrar:** tirar o `RigidBody` do herói · pô-lo `Dynamic` ou `Static` ·
/// tirar o `Collider` · apagar o `TopDownPlayer`.
#[test]
fn o_heroi_anda_quando_o_dono_carrega_na_seta() {
    use ph2d_physics_ecs::{PhysicsBridge, PlayerInput};
    const TIQUES: u64 = 60;

    let mut sim = SimWorld::new();
    montar(sim.world_mut(), 1);
    let (heroi, lei) = {
        let m = sim.world_mut();
        m.query::<(Entity, &Name, &ph2d_physics_ecs::TopDownPlayer)>()
            .iter(m)
            .find(|(_, n, _)| n.0 == "Heroi")
            .map(|(e, _, c)| (e, c.law()))
            .expect("a cena tem de ter o herói que o roteiro manda conduzir")
    };
    let antes = sim.world().get::<Transform>(heroi).unwrap().translation;
    let mut bridge = PhysicsBridge::new();
    for t in 1..=TIQUES {
        // ⚠️ **A entrada é escrita a CADA tique**, como o quadro faz: a ponte lê-a e não a guarda.
        bridge.set_player_input(
            heroi,
            PlayerInput {
                drive: 1.0,
                ..PlayerInput::default()
            },
        );
        bridge.dispatch(&mut sim, true, t);
    }
    let depois = sim.world().get::<Transform>(heroi).unwrap().translation;
    let andou = (depois.x - antes.x).hypot(depois.y - antes.y);

    // O orçamento de um segundo de seta, à velocidade que a cena autora.
    #[allow(clippy::cast_precision_loss)]
    let orcamento = lei.speed * (TIQUES as f32) / 60.0;
    assert!(
        andou > orcamento * 0.5,
        "com a seta segurada {TIQUES} tiques o herói andou {andou:.4} m de um orçamento de \
         {orcamento:.4}.\n⚠️ `0` quer dizer que ele não tem CORPO (a ponte varre corpos, não \
         componentes) ou que o corpo não é cinemático — e o passo (2) do roteiro fica impossível."
    );
    // ⛔ **E ele não pode CAIR:** um corpo dinâmico numa cena de vista de cima é levado pela
    // gravidade, a câmera segue-o, e em ~2 s não há pátio nenhum no ecrã. Medido no defeito:
    // `y = −492 m` ao fim de dez segundos.
    assert!(
        (depois.y - antes.y).abs() < 0.05,
        "o herói desceu {:.3} m sem ninguém lhe pedir — o corpo é DINÂMICO e a gravidade manda \
         nele; a vista segue-o e o pátio sai do ecrã",
        antes.y - depois.y
    );
}

/// ⭐⭐⭐ **E O DEDO DO DONO ALCANÇA-O — a metade que o gate acima NÃO mede.**
///
/// ⛔⛔ **O irmão entra pelo canal INTERNO da ponte** (`set_player_input`), que fica **ABAIXO** da
/// rotura que o TOP-20 #13 pagou por report: ali a lei estava certa, os `24` gates eram verdes, e a
/// **ENTREGA** — quem varre o mundo à procura de quem lê o teclado — não conhecia o componente ⇒
/// *«nada se move»*, com a fita a nem gravar porque a contagem dava `0`.
///
/// ⇒ esta metade pergunta à **PORTA** ([`ph2d_physics_ecs::keyboard_driven`]), que é a mesma que a
/// shell varre todo o quadro. ⚠️ Ela responde `false` a um mover com `default_controls` desligado —
/// e um herói assim seria um motor puro à espera de alguém que lhe escrevesse a intenção, que nesta
/// cena não existe.
#[test]
fn a_porta_do_teclado_alcanca_o_heroi_da_cena() {
    let mut sim = SimWorld::new();
    montar(sim.world_mut(), 1);
    let heroi = {
        let m = sim.world_mut();
        m.query::<(Entity, &Name)>()
            .iter(m)
            .find(|(_, n)| n.0 == "Heroi")
            .map(|(e, _)| e)
            .expect("a cena tem de ter o herói")
    };
    let mut alcancados = Vec::new();
    ph2d_physics_ecs::keyboard_driven::for_each_keyboard_driven(sim.world(), |e| {
        alcancados.push(e);
    });
    assert!(
        alcancados.contains(&heroi),
        "a varredura do teclado não alcança o herói: as setas do dono não chegam a ele, e a fita \
         determinística nem grava (a contagem de players daria zero)"
    );
    // ⛔ **E só ele** — a bomba e a câmera não são conduzidas pelo dedo.
    assert_eq!(alcancados.len(), 1, "quem mais lê o teclado nesta cena?");
}

/// ⭐⭐⭐ **O roteiro nomeia chips que EXISTEM, e os nomes saem da tabela que o painel pinta.**
///
/// ⚠️ **As duas metades, e nenhuma basta:** o texto tem de CONTER os três nomes (senão o passo (4)
/// fala de um botão que ninguém encontra) **e** eles têm de vir resolvidos do i18n (uma chave por
/// traduzir aparece como `shake.perfil.recoil` no ecrã do dono, e um gate que só procurasse a
/// palavra «Recoil» num literal ficaria verde sobre isso).
///
/// **Mutações que devem sangrar:** trocar `nome_do_perfil` por um literal · apagar uma entrada da
/// tabela de i18n · tirar um perfil do roteiro.
#[test]
fn o_roteiro_nomeia_os_tres_perfis_pela_tabela_do_painel() {
    let texto = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/shake_smoke.rs"))
        .expect("o roteiro vive neste ficheiro");
    assert!(
        texto.contains("{recuo}") && texto.contains("{impacto}") && texto.contains("{explosao}"),
        "o passo (4) tem de nomear os TRE^S perfis, e por interpolacao"
    );
    for p in ph2d_shake::Perfil::ALL {
        let nome = ph2d_i18n::tr(p.label_key());
        assert_ne!(
            nome,
            p.label_key(),
            "{p:?}: a chave {} nao esta' na tabela de i18n — o dono le^-la-ia crua",
            p.label_key()
        );
        assert!(
            nome.chars().next().is_some_and(char::is_uppercase),
            "{p:?}: «{nome}» nao e' um ro'tulo que o artista le^"
        );
    }
    let rotulo = ph2d_i18n::tr("panel.inspector.shake.preset");
    assert_ne!(rotulo, "panel.inspector.shake.preset", "a fileira tem nome");
}
