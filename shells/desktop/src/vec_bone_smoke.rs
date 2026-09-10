//! ⭐⭐⭐ **O DESENHO GANHA OSSOS** — `PH2D_VEC_BONE_SMOKE=1` (estudo 42, item 5).
//!
//! # O que a cena fecha
//!
//! Até 2026-09-06 um personagem deste editor só animava como **recorte de papel**: a timeline movia
//! a POSE de uma forma inteira, e mais nada. Dobrar um braço exigia desenhá-lo outra vez.
//!
//! # A cena, e o que cada peça prova
//!
//! | O quê | O que ela prova |
//! |---|---|
//! | **O BRAÇO** (barra com 3 ossos, já presa) | girar um osso DOBRA o desenho, e a base fica onde está |
//! | **O TENTÁCULO** (barra com 6 ossos, já presa) | a cadeia inteira: girar a RAIZ leva tudo, girar a PONTA leva só a ponta — a cinemática é a hierarquia |
//! | **A FOLHA SOLTA** (uma forma e um esqueleto **não ligados**) | o gesto do *Bind*: seleccionar a forma, carregar no botão, e ela passa a obedecer |
//!
//! ⚠️ **As duas primeiras são ligadas pela cena**, e é de propósito: sem uma peça que já obedece, o
//! primeiro gesto do artista seria montar um rig do zero para só então descobrir se ele funciona.
//!
//! ⚠️ Se a linha `[vec-bone-smoke]` não aparecer, PARE: a cena não montou.
//!
//! # ⚠️ Ela monta em DOIS tempos, e não é conforto
//!
//! Prender uma forma exige a **entidade** dela, e quem a cria é o `vec_entities::sync`, que corre
//! **depois** deste prólogo. Ligar no mesmo quadro em que se desenha encontraria o mapa vazio e
//! prenderia zero formas — em silêncio.

use ph2d_ecs::Entity;
use ph2d_vec_scene::ShapeKind;

use crate::build_smoke::shape;

/// ⭐⭐⭐ **A GEOMETRIA DO BRAÇO — uma tabela, dois consumidores.**
///
/// A cena monta-a e o gate `the_smoke_scene_gives_the_anchor_a_side_to_defend` mede-a. ⚠️ Escrita
/// duas vezes, ela divergiria em silêncio e o gate passaria a aprovar uma cena que já não existe —
/// que é exactamente o modo de falha do `CLAUDE.md` §5.0 (*uma cena que ensina o contrário do que
/// acontece é pior que uma cena ausente*).
pub(crate) const ARM_A: [f64; 2] = [-8.2, 2.5];
/// A ponta do braço da cena. Ver [`ARM_A`].
pub(crate) const ARM_B: [f64; 2] = [-1.8, 2.5];
/// Quantos ossos o braço da cena tem. Ver [`ARM_A`].
pub(crate) const ARM_BONES: usize = 3;
/// ⭐ **A DOBRA DO COTOVELO**, em radianos — o que dá à âncora um lado para capturar.
///
/// ⚠️ **`0` aqui apaga a wave do lado da dobra em silêncio:** o `add` capturaria `Auto` sobre uma
/// corrente recta, e o dono veria o joelho inverter exactamente como antes. O gate acima existe
/// para esse zero ser vermelho em vez de invisível.
pub(crate) const ARM_ELBOW_BEND: f32 = 0.45; // LITERAL-PX-OK: ângulo do documento (rad)

/// Qual osso do tentáculo nasce com limite de ângulo — o 2.º, que fica bem no meio da parte visível
/// da cadeia. ⚠️ O vizinho fica SEM limite de propósito: é o contraste que ensina.
pub(crate) const TENTACLE_LIMITED_BONE: usize = 2;

/// Meia-faixa do limite da cena, em radianos (~17°). ⚠️ Estreita de propósito: uma faixa larga
/// obrigaria o dono a girar meia volta antes de sentir a parede, e o smoke ficaria mudo.
pub(crate) const TENTACLE_LIMIT_HALF: f64 = 0.3; // LITERAL-PX-OK: ângulo do documento (rad)

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
pub(crate) const TENTACLE_BONES: usize = 6;

/// ⭐⭐⭐ **A ACÇÃO QUE A CENA JÁ TRAZ** — o nome que aparece no selector *Action* do painel.
///
/// ⚠️⚠️ **Ela existe por causa do report do dono de 2026-09-08** (*«não há meios de selecionar nem
/// o objeto alvo nem a animação»*): sem uma acção com CONTEÚDO na cena, o único caminho para provar
/// um osso inteligente era o artista gravar uma animação primeiro — e o smoke passava a testar a
/// timeline em vez do osso. *Uma cena que só produz o fenómeno depois de o artista acertar OUTRO
/// gesto não prova nada quando esse gesto falha* (`CLAUDE.md` §5.0).
pub(crate) const DEMO_ACTION: &str = "Leaf Rises";

/// Quanto a folha sobe ao longo da acção da cena, nas unidades do documento.
///
/// ⚠️ Grande o suficiente para o percurso se ler numa forma que mede ~2 de altura: uma subida de
/// meia forma leria-se como tremor, e o smoke ficaria mudo sobre um motor correcto.
const DEMO_RISE: f32 = 3.0; // LITERAL-PX-OK: distância do documento, não medida de UI

/// A duração da acção da cena, em segundos. ⚠️ **Ela é o que o `clamp` do `action_time` divide**,
/// e não uma escolha estética: o giro do osso mapeia-se nesta faixa inteira.
const DEMO_SECONDS: f64 = 2.0;

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
pub(crate) fn seed_demo_action(doc: &mut ph2d_timeline::TimelineDoc, bits: u64) {
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

/// Uma cadeia de `n` ossos de `a` a `b` (mundo), o 1.º sem pai. Devolve a RAIZ.
pub(crate) fn cadeia(
    sim: &mut ph2d_ecs::SimWorld,
    a: [f64; 2],
    b: [f64; 2],
    n: usize,
) -> Option<Entity> {
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
        let bits = crate::bone_gesture::create(sim, pai, o, p)?;
        pai = Some(Entity::from_bits(bits));
        raiz = raiz.or(pai);
    }
    raiz
}

/// A PONTA de uma cadeia — desce pelo 1.º filho até não haver osso abaixo.
fn ponta_da_cadeia(sim: &ph2d_ecs::SimWorld, raiz: Entity) -> Entity {
    let mut e = raiz;
    while let Some(f) = sim.world().get::<ph2d_ecs::Children>(e).and_then(|c| {
        c.iter()
            .find(|c| sim.world().get::<ph2d_skeleton_ecs::Bone>(**c).is_some())
    }) {
        e = *f;
    }
    e
}

impl crate::App {
    /// No prólogo do frame. No-op sem a env.
    pub(crate) fn vec_bone_smoke(&mut self) {
        if std::env::var_os("PH2D_VEC_BONE_SMOKE").is_none() || self.gfx.is_none() {
            return;
        }
        match self.vec_bone_smoke_step {
            0 => self.bone_smoke_build(),
            // ⚠️⚠️ **NÃO se conta QUADROS aqui, pergunta-se o FATO.** Prender exige a ENTIDADE de
            // cada forma, e quem a cria (`vec_entities::sync`) corre no MEIO do quadro — que um
            // quadro inicial pode nunca alcançar (superfície ainda por configurar). Um contador
            // acertaria na máquina que testou e prenderia **zero** noutra, em silêncio, e o
            // sintoma seria exactamente *"nenhuma forma pode ser deformada"*.
            1 => {
                let prontas = self
                    .vec_bone_smoke_pend
                    .as_ref()
                    .is_some_and(|p| p.iter().all(|(id, _)| self.vec_entities.contains_key(id)));
                if prontas {
                    self.bone_smoke_bind();
                }
            }
            _ => {}
        }
    }

    /// O 1.º tempo: a arte e os três esqueletos.
    fn bone_smoke_build(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("vector"));
        // ⭐ O BRAÇO e o TENTÁCULO: barras deitadas, com a cadeia pelo MEIO delas.
        let braco = gfx.vec_scene.push_path(shape(
            ShapeKind::RoundRect,
            [-8.5, 2.0],
            [-1.5, 3.0],
            &[0.5],
            [230, 170, 90],
        ));
        let tentaculo = gfx.vec_scene.push_path(shape(
            ShapeKind::RoundRect,
            [-8.5, -0.5],
            [0.5, 0.3],
            &[0.4],
            [110, 190, 160],
        ));
        // A FOLHA SOLTA: a forma e o esqueleto existem, e **não se conhecem**.
        let folha = gfx.vec_scene.push_path(shape(
            ShapeKind::Ellipse,
            [2.5, -4.5],
            [8.5, -2.5],
            &[],
            [180, 140, 220],
        ));
        let a = cadeia(&mut gfx.sim, ARM_A, ARM_B, ARM_BONES);
        let t = cadeia(&mut gfx.sim, [-8.2, -0.1], [0.2, -0.1], TENTACLE_BONES);
        let f = cadeia(&mut gfx.sim, [3.0, -3.5], [8.0, -3.5], 2);
        // ⭐⭐⭐ **A 4.ª PEÇA: um desenho PINTADO** — a 2.ª mídia (ordem do dono, 2026-09-09).
        //
        // ⚠️ Ela nasce **solta**, como a folha: o gesto do *Bind* é o que a cena ensina. E o
        // esqueleto dela fica por baixo do braço pintado, na mesma pose relativa do braço vectorial
        // — assim as duas mídias vêem-se lado a lado a responder ao MESMO gesto.
        let ppm = gfx.hero_screen.as_ref().map_or(64.0, |h| {
            h.project.pixels_per_meter.max(crate::EPS_PIXELS_PER_METER)
        });
        let px = arm_pixels();
        let img = match gfx.renderer.acquire_individual(IMG_W, IMG_H, &px) {
            Ok(texture_id) => {
                let pixels_id = gfx.asset_db.insert_image_rgba8(IMG_W, IMG_H, px);
                let (_, bits) = crate::image_import::spawn_sprite(
                    &mut gfx.sim,
                    crate::image_import::PackedSource::Individual {
                        texture_id,
                        pixels_id,
                    },
                    ph2d_core::Vec2::new(5.5, 2.5),
                    [f64::from(IMG_W) as f32 / ppm, f64::from(IMG_H) as f32 / ppm],
                    "Painted arm",
                );
                let raiz = cadeia(&mut gfx.sim, [3.6, 2.5], [7.4, 2.5], 3);
                Some((bits, raiz))
            }
            Err(e) => {
                eprintln!("[vec-bone-smoke] a imagem nao subiu para a GPU: {e}");
                None
            }
        };
        self.vec_bone_smoke_img = img;
        self.vec_bone_smoke_pend = Some([(braco, a), (tentaculo, t), (folha, f)]);
        self.vec_bone_smoke_step = 1;
    }

    /// O 2.º tempo: prende as DUAS primeiras. A folha fica solta de propósito.
    fn bone_smoke_bind(&mut self) {
        self.vec_bone_smoke_step = 2;
        let Some(pecas) = self.vec_bone_smoke_pend.take() else {
            return;
        };
        let gfx = self.gfx.as_mut().expect("gfx");
        eprintln!(
            "[vec-bone-smoke] mapa de entidades = {} forma(s); cena = {} caminho(s)",
            self.vec_entities.len(),
            gfx.vec_scene.paths().len()
        );
        // ⭐⭐⭐ **A IMAGEM É PRESA PELA CENA, como o braço e o tentáculo** — e pela mesma razão
        // escrita no cabeçalho: sem uma peça que já obedece, o primeiro gesto do artista seria
        // montar tudo do zero para só então descobrir se funciona.
        if let Some((bits, raiz)) = self.vec_bone_smoke_img
            && let Some(e) = ph2d_ecs::Entity::try_from_bits(bits)
        {
            let arte = gfx
                .sim
                .world()
                .get::<ph2d_ecs::SpritePixels>(e)
                .map(|p| p.0)
                .and_then(|id| gfx.asset_db.get(&id));
            let feito = arte
                .as_ref()
                .and_then(|a| a.image_rgba8())
                .is_some_and(|(w, h, cow)| {
                    crate::skeleton_live::bind_image(
                        &mut gfx.sim,
                        e,
                        &cow,
                        [w, h],
                        ph2d_poly2d::GridOptions::default(),
                        raiz,
                    )
                });
            eprintln!(
                "[vec-bone-smoke] o desenho PINTADO {}",
                if feito {
                    "esta' preso ao esqueleto"
                } else {
                    "NAO prendeu -- PARE, a 2a midia nao montou"
                }
            );
        }
        let mut presas = 0;
        for (id, raiz) in pecas.iter().take(2) {
            presas += crate::skeleton_live::bind(
                &mut gfx.sim,
                &gfx.vec_scene,
                &self.vec_entities,
                &[*id],
                *raiz,
            );
        }
        // ⭐⭐⭐ **O TENTÁCULO ABRE JÁ CURVADO** (report do Enio, 2026-09-06: *"nenhuma forma pode
        // ser deformada"*). A medição mostrou o motor intacto — o que faltava era **ver** que ele
        // funciona sem depender de acertar um gesto que ainda se está a aprender.
        //
        // ⚠️ É a lei do `CLAUDE.md` §5.0 levada a sério: *uma cena que só prova o motor DEPOIS de
        // o artista acertar o gesto não prova nada quando o gesto falha* — e as duas falhas leem-se
        // exactamente igual na tela.
        if let Some((_, Some(raiz))) = pecas.get(1).copied() {
            let mut e = raiz;
            let mut n = 0;
            let mut limitado = None;
            // Deixa a raiz quieta e curva do 2.º em diante: a base ancorada faz a curva LER-SE.
            while let Some(filhos) = gfx.sim.world().get::<ph2d_ecs::Children>(e) {
                let Some(f) = filhos.iter().next() else {
                    break;
                };
                e = *f;
                n += 1;
                if n >= 1
                    && let Some(mut t) = gfx.sim.world_mut().get_mut::<ph2d_ecs::Transform>(e)
                {
                    t.rotation = 0.30; // LITERAL-PX-OK: ângulo do documento (rad), não medida de UI
                }
                // ⭐⭐⭐ **UM OSSO DO TENTÁCULO NASCE COM LIMITE DE ÂNGULO**, e o vizinho não.
                //
                // ⚠️ É o CONTRASTE que ensina: girar este pára numa parede, girar o de baixo gira
                // livre. Uma cena em que tudo tem limite não distingue *«o limite funciona»* de
                // *«o osso não roda»*.
                //
                // ⛔ E ele vai no TENTÁCULO, não no braço: o braço tem a âncora de IK, e uma
                // corrente que não alcança o alvo por causa de um limite lê-se, para quem está a
                // aprender, como a IK partida. As duas coisas provam-se separadas.
                if n == TENTACLE_LIMITED_BONE {
                    let centro = f64::from(
                        gfx.sim
                            .world()
                            .get::<ph2d_ecs::Transform>(e)
                            .map_or(0.0, |t| t.rotation),
                    );
                    gfx.sim
                        .world_mut()
                        .entity_mut(e)
                        .insert(ph2d_skeleton_ecs::BoneLimit {
                            min: centro - TENTACLE_LIMIT_HALF,
                            max: centro + TENTACLE_LIMIT_HALF,
                        });
                    limitado = Some(n);
                }
            }
            if let Some(n) = limitado {
                eprintln!(
                    "[vec-bone-smoke] o {n}.o osso do TENTACULO nasce com LIMITE DE ANGULO de +-{:.0} \
                     graus -- gire-o e ele PARA; gire o vizinho e ele gira livre.",
                    TENTACLE_LIMIT_HALF.to_degrees()
                );
            }
        }
        // ⭐⭐⭐ **O BRAÇO NASCE COM ÂNCORA DE IK** — a restrição que persiste.
        //
        // ⚠️ **Ela nasce COINCIDENTE com a ponta**, então a cena abre com o braço exactamente onde
        // estava: o losango é a única coisa nova na tela, e o artista descobre o que ele faz
        // arrastando-o. ⛔ Uma cena que abrisse já dobrada não distinguiria *«a âncora funciona»* de
        // *«a cena montou torta»*.
        //
        // ⭐⭐⭐ **E O COTOVELO NASCE DOBRADO, porque uma corrente RECTA NÃO TEM LADO.**
        //
        // ⚠️⚠️ **Sem isto a cena ensinaria o CONTRÁRIO do que o app faz** (`CLAUDE.md` §5.0): o
        // `add` **captura** de que lado a corrente já está, e sobre um braço recto ele captura
        // `Auto` — que é precisamente o modo em que o joelho **inverte** ao passar pela posição
        // esticada. O dono arrastaria o losango, veria a inversão, e a cura estaria lá, desligada,
        // porque a cena não lhe deu um lado para defender.
        //
        // ⛔ E a dobra vai no ÚLTIMO osso, não em toda a cadeia: a corrente que a âncora governa é
        // a de `DEFAULT_CHAIN` (dois ossos), e dobrar acima dela não lhe daria lado nenhum.
        //
        // ⚠️ A cerca do bloco de baixo continua inteira: o que não pode abrir deslocado é a
        // ÂNCORA (ela nasce coincidente com a ponta, e o braço não se mexe quando ela aparece).
        // Um cotovelo dobrado é a pose que o artista autorou — é o que um braço tem.
        if let Some((_, Some(raiz))) = pecas.first().copied() {
            let ponta = ponta_da_cadeia(&gfx.sim, raiz);
            if let Some(mut t) = gfx.sim.world_mut().get_mut::<ph2d_ecs::Transform>(ponta) {
                t.rotation = ARM_ELBOW_BEND;
            }
        }
        let ancorado = pecas
            .first()
            .and_then(|(_, raiz)| *raiz)
            .map(|raiz| ponta_da_cadeia(&gfx.sim, raiz))
            .and_then(|ponta| crate::skeleton_goal::add(&mut gfx.sim, ponta))
            .is_some();
        // ⚠️ O lado CAPTURADO sai no log: sem esta linha, um `Auto` capturado por engano (a cena a
        // montar-se recta) lê-se exactamente como a cura a funcionar — até o dono arrastar.
        let lado = pecas
            .first()
            .and_then(|(_, raiz)| *raiz)
            .map(|raiz| ponta_da_cadeia(&gfx.sim, raiz))
            .and_then(|ponta| gfx.sim.world().get::<ph2d_skeleton_ecs::IkGoal>(ponta))
            .map_or(ph2d_skeleton::BendSide::Keep, |g| g.bend);
        // ⭐⭐⭐ **A ACÇÃO PRONTA**, para o osso inteligente ter o que percorrer sem o artista ter de
        // gravar uma animação primeiro. A folha roxa sobe ao longo dela.
        //
        // ⚠️ **A acção fica FECHADA** (o clip activo continua a ser o `"Main"`): um controlo não
        // percorre a acção que está aberta na timeline — ali o artista está a gravá-la. Abrir esta
        // deixaria o osso inteligente inerte e o smoke a ensinar o contrário do que o app faz.
        let folha_bits = pecas
            .get(2)
            .and_then(|(id, _)| self.vec_entities.get(id).copied());
        if let Some(bits) = folha_bits {
            seed_demo_action(&mut self.timeline.doc, bits);
            eprintln!(
                "[vec-bone-smoke] a cena traz a accao \"{DEMO_ACTION}\" (a FOLHA roxa sobe {DEMO_RISE} \
                 em {DEMO_SECONDS}s). Escolha um osso e carregue em `Add Smart Bone`: ele nasce \
                 VAZIO. Depois `Pick Object` + clique na folha roxa (no canvas ou na Hierarquia) e \
                 escolha \"{DEMO_ACTION}\" em `Action` -- girar esse osso passa a percorrer a \
                 animacao inteira."
            );
        }
        eprintln!(
            "[vec-bone-smoke] {presas} forma(s) presa(s): o BRACO (3 ossos, COTOVELO DOBRADO, com \
             ANCORA DE IK: {ancorado}, lado capturado: {lado:?}) e o TENTACULO (6, ja' CURVADO \
             pela cena -- e' o motor a trabalhar sem gesto nenhum). A FOLHA roxa tem esqueleto e \
             NAO esta' presa -- seleccione-a e carregue em `Bind to Skeleton`. Para POSAR, fique \
             na ferramenta Bone: arraste o CORPO de um osso para o girar, a BOLINHA da junta para \
             o deslocar, e o LOSANGO na ponta do braco para a corrente inteira o seguir -- esse \
             fica. ⚠️ Se `lado capturado` disser `Keep`, a cena montou-se RECTA e o smoke do lado \
             da dobra nao tem sujeito."
        );
    }
}

/// Lado da imagem da 4.ª peça, em pixels.
const IMG_W: u32 = 320;
/// Altura da imagem da 4.ª peça. Ver [`IMG_W`].
const IMG_H: u32 = 96;

/// ⭐⭐⭐ **A ARTE DA 4.ª PEÇA — um braço PINTADO, com listras.**
///
/// ⚠️ **As listras são o que torna a dobra legível.** Uma barra de cor chapada deformada lê-se
/// quase igual à mesma barra rodada; com listras transversais, dobrar o cotovelo **abre um leque**
/// que o olho vê de imediato. *Uma cena que não distingue a deformação da rotação não prova a
/// deformação.*
///
/// ⚠️ **A borda é SUAVE** (o alfa sobe ao longo de ~2 px), e é de propósito: é o caso que o limiar
/// de `1` do traçador defende, e uma arte de borda dura não o testaria.
fn arm_pixels() -> Vec<u8> {
    let (w, h) = (IMG_W as f64, IMG_H as f64);
    let raio = h / 2.0 - 3.0;
    let (ax, bx) = (raio + 3.0, w - raio - 3.0);
    let mut px = vec![0u8; (IMG_W * IMG_H * 4) as usize];
    for y in 0..IMG_H {
        for x in 0..IMG_W {
            let p = [f64::from(x) + 0.5, f64::from(y) + 0.5];
            // Distância à cápsula deitada — o eixo vai de `ax` a `bx` a meia altura.
            let cx = p[0].clamp(ax, bx);
            let d = (p[0] - cx).hypot(p[1] - h / 2.0);
            let cobertura = (raio + 1.0 - d).clamp(0.0, 1.0);
            if cobertura <= 0.0 {
                continue;
            }
            // Listras transversais a cada 24 px, mais um degradê ao longo do braço.
            let faixa = ((p[0] / 24.0).floor() as i32).rem_euclid(2) == 0;
            let t = (p[0] / w).clamp(0.0, 1.0);
            let base: [f64; 3] = if faixa {
                [235.0, 180.0, 95.0]
            } else {
                [190.0, 120.0, 70.0]
            };
            let i = ((y * IMG_W + x) * 4) as usize;
            for k in 0..3 {
                let v = base[k] * 0.25f64.mul_add(-t, 1.0);
                #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let b = v.clamp(0.0, 255.0) as u8;
                px[i + k] = b;
            }
            #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let a = (cobertura * 255.0) as u8;
            px[i + 3] = a;
        }
    }
    px
}
