//! Testes do [`crate::label_live`] — módulo irmão (teto de 600 LOC por arquivo da shell).
//!
//! O que eles provam, em ordem de importância:
//!
//! 1. **O rótulo SEGUE a forma** (`moving_the_host_shape_drags_the_label_along`) — a feature
//!    inteira numa asserção.
//! 2. **O rótulo de um CONECTOR fica no meio da linha por COMPRIMENTO DE ARCO**, e numa rota em
//!    "L" isso NÃO é a quina (`a_label_on_a_connector_sits_at_the_arclength_middle_of_the_route`).
//! 3. **Arrastar o rótulo o desloca, e ele FICA lá** — sem saltar de volta no frame seguinte
//!    (`dragging_the_label_sticks_and_it_does_not_snap_back`) — **e continua seguindo o
//!    hospedeiro** (`a_dragged_label_still_follows_its_host`). Os dois juntos são o antídoto do
//!    par de bugs do cache de pose: quem sempre dirige perde o arrasto; quem sempre absorve
//!    perde o vínculo.
//! 4. Apagar o hospedeiro deixa o TEXTO vivo — só o vínculo morre.

use super::*;
use ph2d_ecs::{VecShape, VecTextParams};
use ph2d_vec_scene::{VecScene, rectangle};

/// Um documento de teste com o ciclo de frame da shell dentro.
struct Doc {
    sim: SimWorld,
    scene: VecScene,
    map: VecEntityMap,
    sides: crate::connector_live::SideCache,
    poses: LabelPoses,
    /// O arm do duplo-clique: o hospedeiro cujo rótulo ainda não tem geometria.
    pending: Option<VecPathId>,
    /// O path da sessão de texto ABERTA (o que a shell passa como `editing`).
    editing: Option<VecPathId>,
}

/// Os params de um objeto de texto qualquer — o rótulo É um objeto de texto, e é o `VecShape`
/// que o `settle_origins` enxerga para pulá-lo.
fn text_params() -> VecTextParams {
    VecTextParams {
        text: "Label".to_owned(),
        origin: [0.0, 0.0],
        family: None,
        size: 1.0,
        weight: 400.0,
        line_height: 1.2,
        tracking: 0.0,
        align: 0,
        axes: Vec::new(),
    }
}

impl Doc {
    fn new() -> Self {
        Self {
            sim: SimWorld::default(),
            scene: VecScene::new(),
            map: VecEntityMap::new(),
            sides: crate::connector_live::SideCache::new(),
            poses: LabelPoses::new(),
            pending: None,
            editing: None,
        }
    }

    /// Uma forma (o hospedeiro).
    fn shape(&mut self, lo: [f64; 2], hi: [f64; 2]) -> VecPathId {
        self.scene.push_path(rectangle(lo, hi))
    }

    /// Um conector entre duas formas (rota ortogonal, o default).
    fn connector(&mut self, a: VecPathId, b: VecPathId) -> VecPathId {
        let c = self
            .scene
            .push_path(ph2d_vec_scene::line([0.0, 0.0], [0.0, 0.0]));
        crate::vec_entities::sync(&mut self.sim, &mut self.scene, &mut self.map);
        assert!(crate::connector_live::attach(
            &mut self.sim,
            &self.map,
            c,
            &VecConnector::between(a, b)
        ));
        c
    }

    /// Um RÓTULO de `host`: um path (a caixa das letras), o `VecShape::Text` que o faz um objeto
    /// de texto, e o vínculo. A geometria é um retângulo — o que o passe move é a BBOX, e um
    /// glyph de verdade só a tornaria mais lenta de calcular.
    fn label(&mut self, lo: [f64; 2], hi: [f64; 2], host: VecPathId) -> VecPathId {
        let id = self.scene.push_path(rectangle(lo, hi));
        crate::vec_entities::sync(&mut self.sim, &mut self.scene, &mut self.map);
        let e = self.entity(id);
        if let Ok(mut ent) = self.sim.world_mut().get_entity_mut(e) {
            ent.insert(VecShape::Text(text_params()));
        }
        assert!(attach(&mut self.sim, &self.map, id, host));
        id
    }

    /// **O rótulo NASCENDO como no app** — e a diferença é o bug inteiro.
    ///
    /// O duplo-clique só ARMA (`pending = host`): um rótulo nasce vazio, e sem geometria não há
    /// path, nem entidade, nem onde pendurar o vínculo. É a 1ª letra que materializa o objeto — e
    /// o `VecLabel` só aparece no [`upkeep_pending`] do frame SEGUINTE ao push, que a shell roda
    /// **depois** do `connector_live::recook`.
    ///
    /// Pendurar o vínculo à mão (o [`Self::label`] acima) pula justamente a janela em que o texto
    /// existe e o vínculo não — a janela em que a linha enxergava a própria legenda.
    fn label_born_the_real_way(
        &mut self,
        lo: [f64; 2],
        hi: [f64; 2],
        host: VecPathId,
    ) -> VecPathId {
        self.pending = Some(host); // o duplo-clique no hospedeiro
        // A 1ª letra: o `vec_text_regen` empurra o path, e o `upsert_text_shape` do frame põe o
        // `VecShape::Text` (é ele, e não o vínculo, que chega a tempo de o `recook` enxergar).
        let id = self.scene.push_path(rectangle(lo, hi));
        crate::vec_entities::sync(&mut self.sim, &mut self.scene, &mut self.map);
        let e = self.entity(id);
        if let Ok(mut ent) = self.sim.world_mut().get_entity_mut(e) {
            ent.insert(VecShape::Text(text_params()));
        }
        self.editing = Some(id); // a sessão de digitação está aberta
        id
    }

    fn entity(&self, id: VecPathId) -> Entity {
        Entity::from_bits(*self.map.get(&id).expect("a entidade do path"))
    }

    /// **O FRAME da shell, PASSO A PASSO, na ordem EXATA do `render_loop`** (mod.rs §2566..2718).
    ///
    /// Rodar uma aproximação aqui é o que deixou o laço do rótulo passar: a 1ª versão deste
    /// harness chamava `sync → settle → build → recook → upkeep` e **pulava o
    /// [`upkeep_pending`]** — que é o passe que PENDURA o `VecLabel`, e que a shell roda **depois
    /// do `recook`**. Com o vínculo pendurado à mão antes do 1º frame, o gate media um mundo em
    /// que a isenção já valia desde sempre: exatamente o único mundo em que o bug não aparece.
    ///
    /// A ordem abaixo é a verdade, com os passes que faltavam marcados:
    ///
    /// ```text
    /// vec_entities::sync
    /// vec_text::upsert_text_shape      ← faltava (a pose do texto EM SESSÃO, todo frame)
    /// connector_live::upkeep           ← faltava (pendura o VecConnector)
    /// vec_transform::settle_origins
    /// vec_transform::build
    /// connector_live::recook           ← LÊ o VecLabel (as paredes)
    /// label_live::upkeep_pending       ← faltava — e PENDURA o VecLabel, DEPOIS de quem o lê
    /// label_live::upkeep
    /// ```
    fn frame(&mut self) {
        crate::vec_entities::sync(&mut self.sim, &mut self.scene, &mut self.map);
        crate::connector_live::upkeep(&mut self.sim, &self.scene, &self.map, None, &mut None);
        crate::vec_transform::settle_origins(&mut self.sim, &mut self.scene, &self.map, &[]);
        let mut xf = crate::vec_transform::build(&self.sim, &self.map);
        crate::connector_live::recook(
            &mut self.sim,
            &mut self.scene,
            &self.map,
            &xf,
            &mut self.sides,
        );
        upkeep_pending(
            &mut self.sim,
            &self.map,
            &mut self.pending,
            self.editing,
            self.editing.is_some(),
        );
        upkeep(
            &mut self.sim,
            &self.scene,
            &self.map,
            &mut xf,
            self.editing,
            &mut self.poses,
        );
    }

    /// Move a pose de um objeto — é EXATAMENTE o que o gizmo de sprite faz (ADR-0111), tanto
    /// para arrastar uma forma quanto para arrastar o próprio rótulo.
    fn nudge(&mut self, id: VecPathId, dx: f32, dy: f32) {
        let e = self.entity(id);
        if let Some(mut t) = self.sim.world_mut().get_mut::<Transform>(e) {
            t.translation.x += dx;
            t.translation.y += dy;
        }
    }

    /// O centro da bbox de MUNDO de um path — o que o usuário vê.
    fn centre(&self, id: VecPathId) -> [f64; 2] {
        let xf = crate::vec_transform::build(&self.sim, &self.map);
        let (lo, hi) = self
            .scene
            .path_world_curve_bbox(&xf, id)
            .expect("a bbox de mundo");
        [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5]
    }

    /// A rota do conector `id`, achatada em mundo (o mesmo achatamento do passe).
    fn route(&self, id: VecPathId) -> Vec<[f64; 2]> {
        let xf = crate::vec_transform::build(&self.sim, &self.map);
        polyline_world(&self.scene, &xf, id)
    }

    fn label_of_host(&self, host: VecPathId) -> Option<VecLabel> {
        self.map.iter().find_map(|(_, &bits)| {
            let l = *self.sim.world().get::<VecLabel>(Entity::from_bits(bits))?;
            (l.host == host).then_some(l)
        })
    }
}

/// Igualdade de pontos até `1e-3` de mundo.
///
/// A tolerância não é preguiça: o `Transform` é `f32` e a geometria é `f64`, então a pose que o
/// passe escreve volta com um resíduo da ordem do ulp (~`2e-7` nestas magnitudes). Exigir
/// igualdade exata testaria a precisão do `f32`, não a feature.
fn near(a: [f64; 2], b: [f64; 2]) -> bool {
    (a[0] - b[0]).abs() < 1e-3 && (a[1] - b[1]).abs() < 1e-3
}

/// **O TESTE.** Move-se a forma; o rótulo vai junto.
///
/// É a feature inteira numa asserção. Se o passe não rodar, se ele ler a bbox local em vez da de
/// mundo, se ele mover a TRANSLAÇÃO em vez do CENTRO da caixa, ou se rodar antes do
/// `vec_transform::build` — este teste fica vermelho.
#[test]
fn moving_the_host_shape_drags_the_label_along() {
    let mut d = Doc::new();
    // Hospedeiro: [0,0]-[4,2], centro (2, 1). O rótulo nasce longe, de propósito.
    let host = d.shape([0.0, 0.0], [4.0, 2.0]);
    let label = d.label([20.0, 20.0], [21.0, 20.4], host);

    d.frame();
    assert!(
        near(d.centre(label), [2.0, 1.0]),
        "o rotulo tem de cair no CENTRO da forma: {:?}",
        d.centre(label)
    );

    // O gizmo arrasta a forma. O rótulo tem de ir junto — sozinho.
    d.nudge(host, 10.0, 5.0);
    d.frame();
    assert!(
        near(d.centre(host), [12.0, 6.0]),
        "a forma andou: {:?}",
        d.centre(host)
    );
    assert!(
        near(d.centre(label), [12.0, 6.0]),
        "o rotulo TEM de seguir a forma: {:?}",
        d.centre(label)
    );
}

/// **O meio da rota é por COMPRIMENTO DE ARCO**, não o vértice do meio da polilinha.
///
/// A distinção não é acadêmica: numa rota ortogonal em "L" — o caso comum de um fluxograma — o
/// vértice do meio é exatamente a QUINA, e o rótulo pousaria em cima do cotovelo. O teste fixa a
/// propriedade (o rótulo está SOBRE a linha, à metade do comprimento) **e prova que a resposta
/// ingênua seria outra** — sem isso ele passaria mesmo com o bug, numa rota reta.
///
/// E o conjunto todo se move: arrastar uma das formas refaz a rota, e o rótulo acompanha.
#[test]
fn a_label_on_a_connector_sits_at_the_arclength_middle_of_the_route() {
    let mut d = Doc::new();
    // Duas caixas em L, com as pernas MUITO desiguais — de propósito. A rota que sai daqui é
    // `[2, 0.5] → [2.175, 0.5] → [2.175, 9.5] → [30, 9.5]`: um toco de jetty, um degrau de 9 e
    // um corredor de 27.8. É essa desigualdade que separa a resposta certa (arco) da preguiçosa
    // (contagem) — em duas caixas simétricas as duas coincidem, e o gate não provaria nada.
    let a = d.shape([0.0, 0.0], [2.0, 1.0]);
    let b = d.shape([30.0, 9.0], [32.0, 10.0]);
    let conn = d.connector(a, b);
    let label = d.label([50.0, 50.0], [51.0, 50.4], conn);

    d.frame();
    let pts = d.route(conn);
    assert!(pts.len() > 2, "a rota ortogonal DOBRA");
    check_on_route_midpoint(&pts, d.centre(label));

    // Mexer numa das formas refaz a rota — e o rótulo se reposiciona nela.
    let before = d.centre(label);
    d.nudge(b, 8.0, 6.0);
    d.frame();
    let after = d.centre(label);
    assert!(
        !near(before, after),
        "a rota mudou; o rotulo tinha de ir junto (ficou em {after:?})"
    );
    check_on_route_midpoint(&d.route(conn), after);
}

/// `p` está SOBRE a polilinha, e à METADE do comprimento dela? E a resposta ingênua (o vértice
/// do meio) seria diferente — senão este gate não morde.
fn check_on_route_midpoint(pts: &[[f64; 2]], p: [f64; 2]) {
    let seg = |a: [f64; 2], b: [f64; 2]| (b[0] - a[0]).hypot(b[1] - a[1]);
    let total: f64 = pts.windows(2).map(|w| seg(w[0], w[1])).sum();
    assert!(total > 1.0, "a rota tem comprimento");

    // O comprimento de arco até o ponto mais próximo de `p` na polilinha, por **projeção exata**
    // em cada segmento. (Varrer o segmento em N amostras — que foi a 1ª versão deste teste —
    // mede a distância com erro da ORDEM DO PASSO: ~6e-3 aqui, o que afogaria a asserção e a
    // faria acusar um bug que não existe.)
    let mut best = (f64::INFINITY, 0.0_f64); // (distância, arco)
    let mut acc = 0.0;
    for w in pts.windows(2) {
        let (a, b) = (w[0], w[1]);
        let len = seg(a, b);
        if len > 1e-12 {
            let t = (((p[0] - a[0]) * (b[0] - a[0]) + (p[1] - a[1]) * (b[1] - a[1])) / (len * len))
                .clamp(0.0, 1.0);
            let q = [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t];
            let d = (q[0] - p[0]).hypot(q[1] - p[1]);
            if d < best.0 {
                best = (d, acc + len * t);
            }
        }
        acc += len;
    }
    assert!(
        best.0 < 1e-3,
        "o rotulo tem de ficar SOBRE a linha (dist {:.5})",
        best.0
    );
    let frac = best.1 / total;
    assert!(
        (frac - 0.5).abs() < 1e-2,
        "o rotulo tem de ficar a METADE do comprimento de arco (ficou a {frac:.3})"
    );

    // **A prova de que o gate MORDE.** A resposta preguiçosa é o ponto do MEIO da polilinha por
    // contagem — que é o que sai de um `pts[len / 2]`, e o análogo exato do "vértice do meio".
    // Nesta rota ela cai num arco bem longe de 0.5 (as pernas do "L" têm comprimentos
    // diferentes), então trocar o cálculo por ela derruba o assert de cima. Se um dia as duas
    // coincidirem, é ESTE assert que avisa que o teste parou de distingui-las.
    let k = pts.len() / 2;
    let naive_arc: f64 = pts[..=k].windows(2).map(|w| seg(w[0], w[1])).sum();
    let naive_frac = naive_arc / total;
    assert!(
        (naive_frac - 0.5).abs() > 0.05,
        "esta rota precisa DISTINGUIR arco-medio ({frac:.3}) de meio-por-contagem \
         ({naive_frac:.3}) — senao o gate passa com o bug"
    );
}

/// **Arrastar o rótulo o desloca — e ele FICA lá.**
///
/// Este é o gate do cache de pose. Quem SEMPRE dirige a pose a partir da âncora faz o rótulo
/// **saltar de volta ao centro** no instante em que o usuário o solta: o gesto é impossível.
/// O passe distingue "fui eu que escrevi esta pose" de "o gizmo mexeu nela" e, no segundo caso,
/// **absorve** o deslocamento no offset — a pose que o usuário deu vira a nova verdade.
#[test]
fn dragging_the_label_sticks_and_it_does_not_snap_back() {
    let mut d = Doc::new();
    let host = d.shape([0.0, 0.0], [4.0, 2.0]); // centro (2, 1)
    let label = d.label([0.0, 0.0], [1.0, 0.4], host);
    d.frame();
    assert!(near(d.centre(label), [2.0, 1.0]));

    // O gizmo arrasta o RÓTULO (não a forma).
    d.nudge(label, 3.0, 4.0);
    d.frame();
    assert!(
        near(d.centre(label), [5.0, 5.0]),
        "o rotulo tem de FICAR onde o usuario o largou: {:?}",
        d.centre(label)
    );
    assert!(
        near(d.label_of_host(host).expect("o vinculo").offset, [3.0, 4.0]),
        "o deslocamento foi ABSORVIDO no offset"
    );

    // E no frame seguinte (sem tocar em nada) ele NÃO salta de volta.
    d.frame();
    d.frame();
    assert!(
        near(d.centre(label), [5.0, 5.0]),
        "o rotulo saltou de volta ao centro: {:?}",
        d.centre(label)
    );
}

/// **E depois de arrastado, ele continua seguindo o hospedeiro.**
///
/// O par do teste acima, e sem ele aquele passaria com o rótulo simplesmente SOLTO. Quem SEMPRE
/// absorve o deslocamento mata o vínculo: o offset engoliria o movimento da forma (o alvo andaria
/// junto com o rótulo, e o delta seria eternamente zero), e o texto ficaria parado enquanto a
/// caixa se afasta.
#[test]
fn a_dragged_label_still_follows_its_host() {
    let mut d = Doc::new();
    let host = d.shape([0.0, 0.0], [4.0, 2.0]); // centro (2, 1)
    let label = d.label([0.0, 0.0], [1.0, 0.4], host);
    d.frame();
    d.nudge(label, 3.0, 4.0); // o usuário desloca o rótulo
    d.frame();
    assert!(near(d.centre(label), [5.0, 5.0]));

    // Agora a FORMA se move. O rótulo tem de ir junto, MANTENDO o deslocamento.
    d.nudge(host, 10.0, -1.0);
    d.frame();
    assert!(
        near(d.centre(host), [12.0, 0.0]),
        "a forma andou: {:?}",
        d.centre(host)
    );
    // Centro da forma (12, 0) + o offset que o usuário autorou (3, 4) = (15, 4).
    assert!(
        near(d.centre(label), [15.0, 4.0]),
        "o rotulo deslocado TEM de seguir a forma (centro + offset): {:?}",
        d.centre(label)
    );
    assert!(
        near(d.label_of_host(host).expect("o vinculo").offset, [3.0, 4.0]),
        "o offset e o do usuario — o movimento do hospedeiro nao pode ser engolido por ele"
    );
}

/// **Apagar o hospedeiro NÃO mata o rótulo** — ele vira um objeto de texto comum.
///
/// Mesma filosofia do `freeze_missing` do conector: destruir trabalho do usuário (ele DIGITOU
/// aquilo) é pior que deixar um objeto largado. E manter o vínculo apontando para um id morto
/// deixaria o texto sem âncora — congelado num limbo que nada mais move.
#[test]
fn deleting_the_host_leaves_the_text_alive_without_the_link() {
    let mut d = Doc::new();
    let host = d.shape([0.0, 0.0], [4.0, 2.0]);
    let label = d.label([0.0, 0.0], [1.0, 0.4], host);
    d.frame();
    let where_it_was = d.centre(label);

    d.scene.remove_path(host); // Delete na forma
    d.frame();

    assert!(
        d.scene.paths().iter().any(|p| p.id == label),
        "o TEXTO tem de continuar na cena"
    );
    let e = d.entity(label);
    assert!(d.sim.world().get_entity(e).is_ok(), "a entidade esta viva");
    assert!(
        d.sim.world().get::<VecLabel>(e).is_none(),
        "so o VINCULO morre: o rotulo vira um texto comum"
    );
    assert!(
        d.sim.world().get::<VecShape>(e).is_some(),
        "e continua sendo um objeto de TEXTO (editavel, conversivel)"
    );
    assert!(
        near(d.centre(label), where_it_was),
        "e fica onde estava — nada de voar para a origem"
    );
    // Idempotente: o frame seguinte não tem mais nada a dissolver.
    d.frame();
    assert!(d.scene.paths().iter().any(|p| p.id == label));
}

/// **Um rótulo nunca é assentado** pelo `vec_transform::settle_origins` — e não porque alguém o
/// pôs numa lista de exceções, mas porque ele **é um objeto de texto** (`VecShape::Text`), e o
/// `settle` já pula toda Live Shape (geometria derivada dos parâmetros).
///
/// O teste fixa a propriedade, não o mecanismo: se um dia o rótulo deixar de ser um texto, ou o
/// `settle` deixar de pular Live Shapes, ele fica vermelho — e é aí que a exceção explícita
/// passa a ser necessária.
///
/// (E mesmo se fosse assentado, o passe sobreviveria: ele deriva o delta do **centro da bbox de
/// mundo** do rótulo, não da translação — não há suposição nenhuma sobre onde está o pivô.)
#[test]
fn a_label_is_never_settled() {
    let mut d = Doc::new();
    let host = d.shape([0.0, 0.0], [4.0, 2.0]);
    // Longe da origem: assentado, este rótulo ganharia translação (20.5, 20.2).
    let label = d.label([20.0, 20.0], [21.0, 20.4], host);
    let e = d.entity(label);

    crate::vec_transform::settle_origins(&mut d.sim, &mut d.scene, &d.map, &[]);

    assert_eq!(
        d.sim.world().get::<Transform>(e).copied().unwrap(),
        Transform::IDENTITY,
        "o rotulo e uma Live Shape (texto): o settle o PULA"
    );
}

/// O vínculo é achado pelo **hospedeiro**, não por quem está sob o cursor — senão um rótulo
/// arrastado para longe seria invisível ao duplo-clique, e um SEGUNDO rótulo nasceria por cima
/// do primeiro.
#[test]
fn the_label_of_a_host_is_found_even_after_being_dragged_away() {
    let mut d = Doc::new();
    let host = d.shape([0.0, 0.0], [4.0, 2.0]);
    let label = d.label([0.0, 0.0], [1.0, 0.4], host);
    d.frame();
    d.nudge(label, 30.0, 30.0); // bem longe da forma
    d.frame();

    let found = label_of(&d.sim, &d.map, host).expect("o rotulo do hospedeiro");
    assert_eq!(found.0, label);
    assert!(label_of(&d.sim, &d.map, 9999).is_none(), "host inexistente");
}

/// **Um rótulo nasce VAZIO**, e um texto vazio não tem geometria — logo não tem path, e sem path
/// não tem entidade. O vínculo espera a 1ª letra materializar o objeto; e a sessão morrer
/// (Esc, troca de modo) **desarma** a fila, para o arm não grudar no PRÓXIMO texto digitado.
#[test]
fn the_pending_link_waits_for_the_first_letter_and_dies_with_the_session() {
    let mut d = Doc::new();
    let host = d.shape([0.0, 0.0], [4.0, 2.0]);
    crate::vec_entities::sync(&mut d.sim, &mut d.scene, &mut d.map);

    // Sessão aberta, texto ainda vazio: nada a pendurar, e o arm SOBREVIVE.
    let mut pending = Some(host);
    upkeep_pending(&mut d.sim, &d.map, &mut pending, None, true);
    assert_eq!(pending, Some(host), "sem geometria, o arm espera");

    // A 1ª letra cria o path (e o sync, a entidade) ⇒ o vínculo é pendurado e o arm some.
    let text = d.scene.push_path(rectangle([0.0, 0.0], [1.0, 0.4]));
    crate::vec_entities::sync(&mut d.sim, &mut d.scene, &mut d.map);
    upkeep_pending(&mut d.sim, &d.map, &mut pending, Some(text), true);
    assert_eq!(pending, None, "pendurado: o arm desarma");
    let e = d.entity(text);
    assert_eq!(
        d.sim.world().get::<VecLabel>(e).map(|l| l.host),
        Some(host),
        "o vinculo aponta para o hospedeiro"
    );
    // O nome tem de ser reconhecível na Hierarquia — "Path 12" não diz nada ao usuário.
    assert!(
        d.sim
            .world()
            .get::<Name>(e)
            .is_some_and(|n| n.0.ends_with(" Label")),
        "o rotulo se chama '<hospedeiro> Label'"
    );

    // A sessão morre ⇒ o arm morre com ela.
    let mut pending = Some(host);
    upkeep_pending(&mut d.sim, &d.map, &mut pending, None, false);
    assert_eq!(pending, None, "sessao morta, arm desarmado");
}

/// **`attach` NUNCA sobrescreve um vínculo existente**: o offset lá dentro pode ser o arrasto que
/// o usuário acabou de dar, e re-anexar o zeraria (reabrir a edição do rótulo o recentraria).
#[test]
fn re_attaching_never_clobbers_the_offset_the_user_authored() {
    let mut d = Doc::new();
    let host = d.shape([0.0, 0.0], [4.0, 2.0]);
    let label = d.label([0.0, 0.0], [1.0, 0.4], host);
    d.frame();
    d.nudge(label, 7.0, -2.0);
    d.frame();
    assert!(near(
        d.label_of_host(host).expect("vinculo").offset,
        [7.0, -2.0]
    ));

    assert!(attach(&mut d.sim, &d.map, label, host), "idempotente");
    assert!(
        near(d.label_of_host(host).expect("vinculo").offset, [7.0, -2.0]),
        "o offset do usuario sobreviveu ao re-attach"
    );
}

/// O meio por arco de um "L" NÃO é a quina — o teste puro da regra, sem router nem ECS.
/// (0,0) → (10,0) → (10,2): comprimento 12, metade 6 ⇒ (6, 0). A quina é (10, 0).
#[test]
fn the_arclength_midpoint_of_an_l_is_not_the_elbow() {
    let l = [[0.0, 0.0], [10.0, 0.0], [10.0, 2.0]];
    let mid = arclength_midpoint(&l).expect("meio");
    assert!(near(mid, [6.0, 0.0]), "{mid:?}");
    assert!(!near(mid, [10.0, 0.0]), "a QUINA nao e o meio da linha");
    // Degenerados: um ponto só, e uma rota de comprimento zero.
    assert_eq!(arclength_midpoint(&[[3.0, 4.0]]), Some([3.0, 4.0]));
    assert_eq!(arclength_midpoint(&[]), None);
}

/// **Os gates do LAÇO** (a linha e o texto pulando sem parar) — módulo filho, teto de 600 LOC.
/// Herdam o [`Doc`] daqui, e é ele que os faz morder: o frame é a sequência EXATA do `render_loop`.
#[path = "label_live_loop_tests.rs"]
mod loops;
