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

use ph2d_vec_scene::ShapeKind;

use ph2d_vec_scene::cook_tinted as shape;
// ⭐ A TABELA da cena, a cadeia e a acção dos Smart Bones moram na folha `ph2d-skeleton-demo`
// (auditoria de arquitectura A1, 2026-09-12): a cena monta-as e os gates da família do esqueleto
// medem-nas — dois consumidores, de propósito, sem uma família depender da outra.
use ph2d_skeleton_demo::{
    ARM_A, ARM_B, ARM_BONES, ARM_ELBOW_BEND, DEMO_ACTION, DEMO_RISE, DEMO_SECONDS, TENTACLE_BONES,
    TENTACLE_LIMIT_HALF, TENTACLE_LIMITED_BONE, cadeia, ponta_da_cadeia, seed_demo_action,
};

/// **O roteador desta cena** — lê a `PH2D_VEC_BONE_SMOKE`.
#[must_use]
pub fn armed() -> bool {
    std::env::var_os("PH2D_VEC_BONE_SMOKE").is_some()
}

/// O 1.º tempo: a arte e os três esqueletos.
pub fn build(
    scene: &mut ph2d_vec_scene::VecScene,
    sim: &mut ph2d_ecs::SimWorld,
    renderer: &mut ph2d_render::SpriteRenderer,
    assets: &mut ph2d_asset::AssetDb,
    ppm: f32,
    st: &mut crate::state::VecState,
) {
    // ⭐ O BRAÇO e o TENTÁCULO: barras deitadas, com a cadeia pelo MEIO delas.
    let braco = scene.push_path(shape(
        ShapeKind::RoundRect,
        [-8.5, 2.0],
        [-1.5, 3.0],
        &[0.5],
        [230, 170, 90],
    ));
    let tentaculo = scene.push_path(shape(
        ShapeKind::RoundRect,
        [-8.5, -0.5],
        [0.5, 0.3],
        &[0.4],
        [110, 190, 160],
    ));
    // A FOLHA SOLTA: a forma e o esqueleto existem, e **não se conhecem**.
    let folha = scene.push_path(shape(
        ShapeKind::Ellipse,
        [2.5, -4.5],
        [8.5, -2.5],
        &[],
        [180, 140, 220],
    ));
    let a = cadeia(sim, ARM_A, ARM_B, ARM_BONES);
    let t = cadeia(sim, [-8.2, -0.1], [0.2, -0.1], TENTACLE_BONES);
    let f = cadeia(sim, [3.0, -3.5], [8.0, -3.5], 2);
    // ⭐⭐⭐ **A 4.ª PEÇA: um desenho PINTADO** — a 2.ª mídia (ordem do dono, 2026-09-09).
    //
    // ⚠️ Ela nasce **solta**, como a folha: o gesto do *Bind* é o que a cena ensina. E o
    // esqueleto dela fica por baixo do braço pintado, na mesma pose relativa do braço vectorial
    // — assim as duas mídias vêem-se lado a lado a responder ao MESMO gesto.
    // ⚠️ O `ppm` chega por PARÂMETRO: ele sai do `hero_screen`, que é chrome da shell.
    let px = arm_pixels();
    let img = match renderer.acquire_individual(IMG_W, IMG_H, &px) {
        Ok(texture_id) => {
            let pixels_id = assets.insert_image_rgba8(IMG_W, IMG_H, px);
            let (_, bits) = ph2d_image_import::spawn_sprite(
                sim,
                ph2d_image_import::PackedSource::Individual {
                    texture_id,
                    pixels_id,
                },
                ph2d_core::Vec2::new(5.5, 2.5),
                [f64::from(IMG_W) as f32 / ppm, f64::from(IMG_H) as f32 / ppm],
                "Painted arm",
            );
            let raiz = cadeia(sim, [3.6, 2.5], [7.4, 2.5], 3);
            Some((bits, raiz))
        }
        Err(e) => {
            eprintln!("[vec-bone-smoke] a imagem nao subiu para a GPU: {e}");
            None
        }
    };
    st.bone_smoke_img = img;
    st.bone_smoke_pend = Some([(braco, a), (tentaculo, t), (folha, f)]);
    st.bone_smoke_step = 1;
}

/// O 2.º tempo: prende as DUAS primeiras. A folha fica solta de propósito.
///
/// O mapa de entidades vem do próprio `st` (A9, 2026-09-12): desde que ele mora no `VecState`, a
/// shell não consegue emprestá-lo à parte do estado que o contém.
pub fn bind(
    scene: &mut ph2d_vec_scene::VecScene,
    sim: &mut ph2d_ecs::SimWorld,
    doc: &mut ph2d_timeline::TimelineDoc,
    assets: &ph2d_asset::AssetDb,
    st: &mut crate::state::VecState,
) {
    st.bone_smoke_step = 2;
    let Some(pecas) = st.bone_smoke_pend.take() else {
        return;
    };
    let map = &st.entities;
    eprintln!(
        "[vec-bone-smoke] mapa de entidades = {} forma(s); cena = {} caminho(s)",
        map.len(),
        scene.paths().len()
    );
    // ⭐⭐⭐ **A IMAGEM É PRESA PELA CENA, como o braço e o tentáculo** — e pela mesma razão
    // escrita no cabeçalho: sem uma peça que já obedece, o primeiro gesto do artista seria
    // montar tudo do zero para só então descobrir se funciona.
    if let Some((bits, raiz)) = st.bone_smoke_img
        && let Some(e) = ph2d_ecs::Entity::try_from_bits(bits)
    {
        let arte = sim
            .world()
            .get::<ph2d_ecs::SpritePixels>(e)
            .map(|p| p.0)
            .and_then(|id| assets.get(&id));
        let feito = arte
            .as_ref()
            .and_then(|a| a.image_rgba8())
            .is_some_and(|(w, h, cow)| {
                ph2d_skeleton_live::skin_live::bind_image(
                    sim,
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
        presas += ph2d_skeleton_live::skin_live::bind(sim, scene, map, &[*id], *raiz);
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
        while let Some(filhos) = sim.world().get::<ph2d_ecs::Children>(e) {
            let Some(f) = filhos.iter().next() else {
                break;
            };
            e = *f;
            n += 1;
            if n >= 1
                && let Some(mut t) = sim.world_mut().get_mut::<ph2d_ecs::Transform>(e)
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
                    sim.world()
                        .get::<ph2d_ecs::Transform>(e)
                        .map_or(0.0, |t| t.rotation),
                );
                sim.world_mut()
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
        let ponta = ponta_da_cadeia(sim, raiz);
        if let Some(mut t) = sim.world_mut().get_mut::<ph2d_ecs::Transform>(ponta) {
            t.rotation = ARM_ELBOW_BEND;
        }
    }
    let ancorado = pecas
        .first()
        .and_then(|(_, raiz)| *raiz)
        .map(|raiz| ponta_da_cadeia(sim, raiz))
        .and_then(|ponta| ph2d_skeleton_live::goal::add(sim, ponta))
        .is_some();
    // ⚠️ O lado CAPTURADO sai no log: sem esta linha, um `Auto` capturado por engano (a cena a
    // montar-se recta) lê-se exactamente como a cura a funcionar — até o dono arrastar.
    let lado = pecas
        .first()
        .and_then(|(_, raiz)| *raiz)
        .map(|raiz| ponta_da_cadeia(sim, raiz))
        .and_then(|ponta| sim.world().get::<ph2d_skeleton_ecs::IkGoal>(ponta))
        .map_or(ph2d_skeleton::BendSide::Keep, |g| g.bend);
    // ⭐⭐⭐ **A ACÇÃO PRONTA**, para o osso inteligente ter o que percorrer sem o artista ter de
    // gravar uma animação primeiro. A folha roxa sobe ao longo dela.
    //
    // ⚠️ **A acção fica FECHADA** (o clip activo continua a ser o `"Main"`): um controlo não
    // percorre a acção que está aberta na timeline — ali o artista está a gravá-la. Abrir esta
    // deixaria o osso inteligente inerte e o smoke a ensinar o contrário do que o app faz.
    let folha_bits = pecas.get(2).and_then(|(id, _)| map.get(id).copied());
    if let Some(bits) = folha_bits {
        seed_demo_action(&mut *doc, bits);
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
