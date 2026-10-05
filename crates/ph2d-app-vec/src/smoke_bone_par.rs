//! ⭐⭐⭐ **O PAR — a MESMA barra como DESENHO e como IMAGEM** — `PH2D_VEC_BONE_SMOKE=3` (ordem do
//! dono, 2026-09-29: *«monte a cena com apenas dois esqueletos iguais, um com vector outro com
//! img»*).
//!
//! # Porque esta cena existe
//!
//! O report anterior dele (*«o osso do meio provoca ondulações discretas, que diferem da deformação
//! de imagens»*) só se julga com as duas mídias **lado a lado e em condições iguais** — e a cena
//! `=1` não as dá: lá o braço vectorial tem `7 × 1` m e o pintado `3,2 × 0,96`, com esqueletos de
//! tamanhos diferentes e sítios diferentes do ecrã. *Duas peças que diferem em tudo não dizem qual
//! diferença é da mídia.*
//!
//! # O que a cena monta, e o que é IGUAL por construção
//!
//! | | em cima | em baixo |
//! |---|---|---|
//! | a peça | um `RoundRect` **vectorial** | uma **imagem** pintada da mesma cápsula |
//! | o tamanho | [`peca`] — UMA porta para os dois | idem |
//! | a cor | [`COR`] | idem |
//! | o esqueleto | [`OSSOS`] ossos pelo eixo da cápsula | idem, deslocado só em `y` |
//! | a pose | [`DOBRA`] em Z, aplicada DEPOIS de prender | idem |
//!
//! ⚠️ **A dobra é aplicada no 2.º tempo, depois de prender**, e é load-bearing: prender fotografa a
//! pose de REPOUSO, e com o esqueleto já dobrado as duas peças aprenderiam o dobrado como repouso e
//! abririam RECTAS.
//!
//! ⭐⭐⭐ **E a `=4` é A DOBRA FORTE** (ordem do dono, 2026-09-29: *«a dobra forte do cotovelo:
//! tente o estado da arte diretamente»*): o MESMO par, empilhado e aberto já a [`DOBRA_FORTE`] por
//! junta — onde a face de dentro de dois membros se SOBREPÕE. O desenho sai como a SILHUETA dos
//! membros (canto em «V»), que é o que a imagem de baixo já mostra de graça; com
//! `PH2D_SKIN_CONTACTO=0` volta o contorno com o «olho» por dentro, e é essa a comparação.
//!
//! ⚠️ **Nada mais na cena, de propósito** — o dono pediu *«apenas»*. Sem folha solta, sem
//! bifurcação, sem barra por cima: *uma cena de comparação com uma terceira peça obriga o olho a
//! decidir o que comparar*.

use ph2d_ecs::{Entity, SimWorld};
use ph2d_vec_scene::{ShapeKind, VecScene, cook_tinted as shape};

/// Quantos ossos cada esqueleto tem — os três do braço da cena `=1`, que é a corrente em que o
/// report nasceu.
pub(crate) const OSSOS: u32 = 3;

/// ⭐ **A dobra por junta, em graus, em Z** (o 2.º osso sobe, o 3.º volta a deitar) — a forma da
/// pose da foto do dono.
///
/// ⚠️ `40°` e não os `25°` da família das mídias: ali a pergunta é *«dobram?»*; aqui é *«dobram
/// IGUAL?»*, e a diferença entre as duas mídias (a transição entre ossos) só tem tamanho para se ver
/// com a dobra franca. ⛔ Acima de `~90°` o vinco de dentro do cotovelo nasce nas DUAS mídias
/// (`det J`, pesquisa 04 §1.3), e a cena passaria a mostrar esse, que é outra pergunta.
pub(crate) const DOBRA: f32 = 40.0;

/// ⭐⭐⭐ **A dobra da `=4`, em graus por junta** — a pose em que há CONTACTO.
///
/// ⚠️ **Medido, não escolhido:** na barra da cena (a mesma proporção `6 × 1` desta peça) o desenho
/// fiel NÃO se cruza até `90°` e cruza-se de `110°` para cima (gate
/// `skin_desenho::tests::numa_dobra_forte_o_desenho_nao_se_cruza`). `120°` fica dentro da faixa
/// com folga, e em Z o 3.º osso volta a deitar por CIMA do 1.º sem lhe tocar (vão de `~0,4`
/// espessuras) — acima de `~130°` os dois membros das pontas começam a tocar-se, que é outra
/// pergunta.
pub(crate) const DOBRA_FORTE: f32 = 120.0;

/// A dobra por junta de cada nível desta cena — na `=4`, `PH2D_VEC_BONE_DOBRA=<graus>` abre-a
/// noutra dobra (F39, 2026-09-30: a quina do contacto só se lê numa ESCADA de ângulos, e a foto
/// não tem mão para escrever no Inspector).
#[must_use]
pub(crate) fn dobra_do_nivel(nivel: u32) -> f32 {
    dobra_de(nivel, std::env::var("PH2D_VEC_BONE_DOBRA").ok().as_deref())
}

/// A lei pura de [`dobra_do_nivel`]: o pedido só vale na `=4` e só se for um número finito.
#[must_use]
pub(crate) fn dobra_de(nivel: u32, pedido: Option<&str>) -> f32 {
    if nivel < 4 {
        return DOBRA;
    }
    pedido
        .and_then(|s| s.trim().parse::<f32>().ok())
        .filter(|g| g.is_finite())
        .unwrap_or(DOBRA_FORTE)
}

/// A dobra das duas juntas das cenas `=5` e `=6`: `PH2D_VEC_BONE_DOBRA=<graus>` (as duas) ou
/// `<g1>,<g2>` (05/10: com os ossos presos ao Edit do vetor o dono não os conseguia mexer; o vinco
/// lê-se com uma junta a `170°` e a outra a `110°`, e as duas a `≥ 140°` dobram a barra em TRÊS
/// camadas — o aberto A13). Sem pedido válido, a `omissao` da cena nas duas.
#[must_use]
pub(crate) fn dobra_da_cena(omissao: f32) -> (f32, f32) {
    dobra_pedida(omissao, std::env::var("PH2D_VEC_BONE_DOBRA").ok().as_deref())
}

/// A lei pura de [`dobra_da_cena`]: só números finitos valem; um só vale para as duas juntas.
#[must_use]
pub(crate) fn dobra_pedida(omissao: f32, pedido: Option<&str>) -> (f32, f32) {
    let num = |s: &str| s.trim().parse::<f32>().ok().filter(|g| g.is_finite());
    match pedido.map(|s| s.split_once(',').map_or((s, None), |(a, b)| (a, Some(b)))) {
        Some((a, None)) => num(a).map_or((omissao, omissao), |g| (g, g)),
        Some((a, Some(b))) => match (num(a), num(b)) {
            (Some(x), Some(y)) => (x, y),
            _ => (omissao, omissao),
        },
        None => (omissao, omissao),
    }
}

/// A junta do contorno da `=4` — `PH2D_VEC_BONE_JUNTA=miter|round|bevel` (F39: a foto precisa de
/// ver as três juntas no mesmo vinco). Qualquer outra coisa é a de fábrica do traço.
#[must_use]
pub(crate) fn junta_de(pedido: Option<&str>) -> ph2d_vec_scene::LineJoin {
    match pedido.map(str::trim) {
        Some("round") => ph2d_vec_scene::LineJoin::Round,
        Some("bevel") => ph2d_vec_scene::LineJoin::Bevel,
        _ => ph2d_vec_scene::LineJoin::Miter,
    }
}

/// A espessura do contorno da `=4` — `PH2D_VEC_BONE_TRACO=<fracção da espessura da peça>` troca a
/// de fábrica ([`ESPESSURA_DO_CONTORNO`]). A junta em bico só se LÊ num traço grosso (F39).
#[must_use]
pub(crate) fn espessura_do_contorno(pedido: Option<&str>) -> f64 {
    pedido
        .and_then(|s| s.trim().parse::<f64>().ok())
        .filter(|f| f.is_finite() && *f > 0.0)
        .unwrap_or(ESPESSURA_DO_CONTORNO)
}

/// A cor das duas peças — a do braço da cena `=1`.
pub(crate) const COR: [u8; 3] = [230, 170, 90];

/// A cor do CONTORNO do desenho na `=4` — o mesmo laranja escurecido, para ler como borda da peça.
pub(crate) const CONTORNO: [u8; 3] = [120, 70, 25];

/// A espessura do contorno da `=4`, em fracção da espessura da peça — fina o bastante para não
/// esconder o canto, grossa o bastante para o «olho» se ver à câmera de omissão.
pub(crate) const ESPESSURA_DO_CONTORNO: f64 = 0.06;

/// A arte da imagem, em pixels. ⚠️ O tamanho no MUNDO sai daqui pelo `ppm` do projecto ([`peca`]),
/// e o desenho vectorial copia esse tamanho — logo os dois são iguais em qualquer `ppm`.
pub(crate) const IMG_W: u32 = 600;
/// Ver [`IMG_W`].
pub(crate) const IMG_H: u32 = 100;

/// ⭐ **O afastamento vertical entre os dois centros, em múltiplos da espessura da peça.**
///
/// ⚠️ A cena abre na **câmera de omissão** (o `Frame All` só conta imagens e enquadraria a de baixo
/// sozinha — ver o prólogo em [`crate::smoke_bone_envelope::prologo_do_nivel`]), e este número põe
/// as duas peças dobradas com um vão de `~1,5` espessuras entre elas, dentro dela (gate [`tests`]).
pub(crate) const AFASTAMENTO: f64 = 3.6;

/// ⭐⭐ **O tamanho de UMA peça no mundo** — `(largura, espessura)`, a porta que as DUAS leem.
#[must_use]
pub(crate) fn peca(ppm: f32) -> (f64, f64) {
    let ppm = f64::from(ppm.max(1e-3));
    (f64::from(IMG_W) / ppm, f64::from(IMG_H) / ppm)
}

/// Os dois centros (`y`) — o desenho em cima, a imagem em baixo.
#[must_use]
pub(crate) fn centros(ppm: f32) -> [f64; 2] {
    let (_, t) = peca(ppm);
    let d = AFASTAMENTO * t / 2.0;
    [d, -d]
}

/// ⭐ **A caixa do EIXO dobrado** de uma peça centrada em `centro` (antes de dobrar), alargada pela
/// meia espessura — a pose que o [`dobra`] produz: o 2.º osso sobe `graus`, o 3.º volta a deitar.
#[must_use]
pub(crate) fn caixa_dobrada(ppm: f32, centro: [f64; 2], graus: f32) -> [f64; 4] {
    let (l, t) = peca(ppm);
    let passo = (l - t) / f64::from(OSSOS);
    let mut p = [centro[0] - l / 2.0 + t / 2.0, centro[1]];
    let mut ang = 0.0_f64;
    let mut caixa = [p[0], p[1], p[0], p[1]];
    for k in 0..OSSOS {
        if k > 0 {
            let sinal = if k % 2 == 1 { 1.0 } else { -1.0 };
            ang += sinal * f64::from(graus).to_radians();
        }
        p = [p[0] + passo * ang.cos(), p[1] + passo * ang.sin()];
        caixa = [
            caixa[0].min(p[0]),
            caixa[1].min(p[1]),
            caixa[2].max(p[0]),
            caixa[3].max(p[1]),
        ];
    }
    let r = t / 2.0;
    [caixa[0] - r, caixa[1] - r, caixa[2] + r, caixa[3] + r]
}

/// ⭐⭐ **Onde nasce cada peça** (`[x, y]` do centro, RECTA) — o desenho primeiro.
///
/// A `=3` é a disposição de sempre ([`centros`]). A `=4` empilha as duas peças DOBRADAS com um vão
/// de uma espessura e centra a caixa de cada uma em `x`: em Z a `120°` a peça dobrada é mais alta
/// do que larga, e a disposição da `=3` poria a imagem por cima do desenho.
#[must_use]
pub(crate) fn origens(ppm: f32, nivel: u32) -> [[f64; 2]; 2] {
    if nivel < 4 {
        let [y_vec, y_img] = centros(ppm);
        return [[0.0, y_vec], [0.0, y_img]];
    }
    let (_, t) = peca(ppm);
    let [x0, y0, x1, y1] = caixa_dobrada(ppm, [0.0, 0.0], dobra_do_nivel(nivel));
    let cx = -(x0 + x1) / 2.0;
    [[cx, t / 2.0 - y0], [cx, -t / 2.0 - y1]]
}

/// O passo da grelha de pontos da imagem, em pixels dela — `5` pontos na espessura da peça.
const PONTO_PASSO: f64 = 20.0;
/// O raio de cada ponto, em pixels — com o centro a `PONTO_PASSO / 2`, a borda dele fica a `6 px` da
/// borda comprida da peça.
const PONTO_RAIO: f64 = 4.0;

/// A arte da imagem: a cápsula deitada, borda suave de um pixel, com uma GRELHA DE PONTOS.
///
/// ⭐ **Os pontos são ordem do dono** (2026-10-02: *«a textura deveria ser listrada ou pontilhada
/// para vermos melhor a deformação interna»*) — a cor chapada mostrava só a silhueta. Pontos e não
/// listras: uma grelha mostra o esticão nos DOIS eixos. ⚠️ Eles ficam a [`PONTO_RAIO`] + `2 px` das
/// bordas compridas, logo a tinta da BEIRA (a que o fecho da imagem estica para dentro de um vão)
/// continua a ser a [`COR`].
fn pixels() -> Vec<u8> {
    let (w, h) = (f64::from(IMG_W), f64::from(IMG_H));
    let raio = h / 2.0;
    let (ax, bx) = (raio, w - raio);
    let mut px = vec![0u8; (IMG_W * IMG_H * 4) as usize];
    for y in 0..IMG_H {
        for x in 0..IMG_W {
            let p = [f64::from(x) + 0.5, f64::from(y) + 0.5];
            let d = (p[0] - p[0].clamp(ax, bx)).hypot(p[1] - raio);
            let cobertura = (raio - d + 0.5).clamp(0.0, 1.0);
            if cobertura <= 0.0 {
                continue;
            }
            let i = ((y * IMG_W + x) * 4) as usize;
            // O ponto mais perto da grelha e a cobertura dele (borda suave de um pixel).
            let centro = |c: f64| {
                (c / PONTO_PASSO)
                    .floor()
                    .mul_add(PONTO_PASSO, PONTO_PASSO / 2.0)
            };
            let no_ponto = (PONTO_RAIO - (p[0] - centro(p[0])).hypot(p[1] - centro(p[1])) + 0.5)
                .clamp(0.0, 1.0);
            for k in 0..3 {
                let mistura =
                    f64::from(CONTORNO[k]).mul_add(no_ponto, f64::from(COR[k]) * (1.0 - no_ponto));
                #[expect(
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss,
                    reason = "um canal"
                )]
                let c = mistura.round() as u8;
                px[i + k] = c;
            }
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "cobertura em [0, 1]"
            )]
            let a = (cobertura * 255.0).round() as u8;
            px[i + 3] = a;
        }
    }
    px
}

/// Um esqueleto de [`OSSOS`] ossos pelo EIXO da cápsula centrada em `y`, recto (o repouso), com os
/// ossos nomeados `«{prefixo} bone k»` — o nome que o roteiro manda escolher na Hierarquia.
fn esqueleto(sim: &mut SimWorld, ppm: f32, centro: [f64; 2], prefixo: &str) -> Option<Entity> {
    let (l, t) = peca(ppm);
    let y = centro[1];
    // ⚠️ Das pontas do EIXO, e não das pontas da peça: a tampa redonda é meia espessura de cada lado.
    let x0 = centro[0] - l / 2.0 + t / 2.0;
    let passo = (l - t) / f64::from(OSSOS);
    let mut pai = None;
    let mut raiz = None;
    for k in 0..OSSOS {
        let a = [x0 + passo * f64::from(k), y];
        let b = [x0 + passo * f64::from(k + 1), y];
        let e = Entity::try_from_bits(ph2d_skeleton_live::bone::create(sim, pai, a, b)?)?;
        sim.world_mut()
            .entity_mut(e)
            .insert(ph2d_ecs::Name::new(format!("{prefixo} bone {}", k + 1)));
        raiz.get_or_insert(e);
        pai = Some(e);
    }
    raiz
}

/// O 1.º tempo: as duas peças e os dois esqueletos, rectos.
pub(crate) fn build(
    scene: &mut VecScene,
    sim: &mut SimWorld,
    renderer: &mut ph2d_render::SpriteRenderer,
    assets: &mut ph2d_asset::AssetDb,
    ppm: f32,
    nivel: u32,
    st: &mut crate::state::VecState,
) {
    let (l, t) = peca(ppm);
    let [o_vec, o_img] = origens(ppm, nivel);
    let mut peca_vec = shape(
        ShapeKind::RoundRect,
        [o_vec[0] - l / 2.0, o_vec[1] - t / 2.0],
        [o_vec[0] + l / 2.0, o_vec[1] + t / 2.0],
        &[t / 2.0],
        COR,
    );
    // ⭐⭐⭐ **Na `=4` o desenho leva CONTORNO, e é ele que mostra o fenómeno** (FOTOGRAFADO,
    // 2026-09-29): o preenchimento (não-zero) de um contorno sobreposto JÁ pinta a união dos
    // membros — sem traço, as fotos com e sem `PH2D_SKIN_CONTACTO` saíam iguais ao pixel. O que a
    // sobreposição estraga é o TRAÇO, que desenha o «olho» por dentro da junta.
    if nivel >= 4 {
        let mut traco = ph2d_vec_scene::StrokeSpec::new(
            ph2d_vec_scene::Rgba8::new(CONTORNO[0], CONTORNO[1], CONTORNO[2], 255),
            t * espessura_do_contorno(std::env::var("PH2D_VEC_BONE_TRACO").ok().as_deref()),
        );
        traco.join = junta_de(std::env::var("PH2D_VEC_BONE_JUNTA").ok().as_deref());
        peca_vec.stroke = Some(traco);
    }
    let desenho = scene.push_path(peca_vec);
    let osso_vec = esqueleto(sim, ppm, o_vec, "Vector");
    let px = pixels();
    st.bone_smoke_img = match renderer.acquire_individual(IMG_W, IMG_H, &px) {
        Ok(texture_id) => {
            let pixels_id = assets.insert_image_rgba8(IMG_W, IMG_H, px);
            #[expect(clippy::cast_possible_truncation, reason = "metros de uma cena")]
            let (_, bits) = ph2d_image_import::spawn_sprite(
                sim,
                ph2d_image_import::PackedSource::Individual {
                    texture_id,
                    pixels_id,
                },
                ph2d_core::Vec2::new(o_img[0] as f32, o_img[1] as f32),
                [l as f32, t as f32],
                "Image",
            );
            Some((bits, esqueleto(sim, ppm, o_img, "Image")))
        }
        Err(e) => {
            eprintln!("[vec-bone-smoke] PARE: a imagem nao subiu para a GPU: {e}");
            None
        }
    };
    st.bone_smoke_pend = Some(vec![(desenho, osso_vec)]);
    st.bone_smoke_step = 1;
}

/// O 2.º tempo: prende as duas peças com os esqueletos RECTOS e só depois dobra-os — iguais.
pub(crate) fn bind(
    scene: &mut VecScene,
    sim: &mut SimWorld,
    assets: &ph2d_asset::AssetDb,
    ppm: f32,
    nivel: u32,
    st: &mut crate::state::VecState,
) {
    st.bone_smoke_step = 2;
    let Some(pecas) = st.bone_smoke_pend.take() else {
        return;
    };
    let mut raizes = Vec::new();
    let mut vector_preso = false;
    for (id, raiz) in &pecas {
        vector_preso |=
            ph2d_skeleton_live::skin_live::bind(sim, scene, &st.entities, &[*id], *raiz) > 0;
        if let Some(e) = st.entities.get(id).and_then(|b| Entity::try_from_bits(*b)) {
            sim.world_mut()
                .entity_mut(e)
                .insert(ph2d_ecs::Name::new("Vector"));
        }
        raizes.extend(*raiz);
    }
    let mut imagem_presa = false;
    if let Some((bits, raiz)) = st.bone_smoke_img
        && let Some(e) = Entity::try_from_bits(bits)
    {
        let arte = sim
            .world()
            .get::<ph2d_ecs::SpritePixels>(e)
            .map(|p| p.0)
            .and_then(|id| assets.get(&id));
        imagem_presa = arte
            .as_ref()
            .and_then(|a| a.image_rgba8())
            .is_some_and(|(w, h, cow)| {
                ph2d_skeleton_live::skin_image_bind::bind_image(
                    sim,
                    e,
                    &cow,
                    [w, h],
                    ppm,
                    ph2d_poly2d::GridOptions::default(),
                    raiz,
                )
            });
        raizes.extend(raiz);
    }
    let g1 = dobra_do_nivel(nivel);
    // ⚠️ `PH2D_VEC_BONE_DOBRA2` dobra a SEGUNDA junta à parte (F39): o vinco mais agudo nasce logo
    // DEPOIS do contacto, e com as duas juntas iguais ele fica escondido debaixo do traço — a
    // fotografia do dono tinha a de baixo forte e a de cima a mal tocar.
    let g2 = segunda_dobra(
        nivel,
        g1,
        std::env::var("PH2D_VEC_BONE_DOBRA2").ok().as_deref(),
    );
    for raiz in &raizes {
        dobra_duas(sim, *raiz, g1, g2);
    }
    anuncia(vector_preso, imagem_presa, nivel);
}

/// A dobra da SEGUNDA junta: a mesma da primeira, salvo um pedido na `=4`.
#[must_use]
pub(crate) fn segunda_dobra(nivel: u32, primeira: f32, pedido: Option<&str>) -> f32 {
    if nivel < 4 {
        return primeira;
    }
    pedido
        .and_then(|s| s.trim().parse::<f32>().ok())
        .filter(|g| g.is_finite())
        .unwrap_or(primeira)
}

/// ⭐ **A MESMA pose nos dois**: o 2.º osso sobe `graus`, o 3.º volta a deitar `segunda` — em Z
/// (as duas iguais, salvo o pedido de [`segunda_dobra`]).
pub(crate) fn dobra_duas(sim: &mut SimWorld, raiz: Entity, graus: f32, segunda: f32) {
    let mut e = raiz;
    let mut k = 0;
    while let Some(f) = sim.world().get::<ph2d_ecs::Children>(e).and_then(|c| {
        c.iter()
            .find(|c| sim.world().get::<ph2d_skeleton_ecs::Bone>(**c).is_some())
            .copied()
    }) {
        e = f;
        k += 1;
        let sinal = if k % 2 == 1 { 1.0 } else { -1.0 };
        let g = if k >= 2 { segunda } else { graus };
        if let Some(mut t) = sim.world_mut().get_mut::<ph2d_ecs::Transform>(e) {
            t.rotation += sinal * g.to_radians();
        }
    }
}

fn anuncia(vector_preso: bool, imagem_presa: bool, nivel: u32) {
    if !vector_preso || !imagem_presa {
        eprintln!(
            "[vec-bone-smoke] PARE: o desenho {} e a imagem {} -- a cena nao montou",
            if vector_preso {
                "prendeu"
            } else {
                "NAO prendeu"
            },
            if imagem_presa {
                "prendeu"
            } else {
                "NAO prendeu"
            },
        );
    }
    if nivel >= 4 {
        println!(
            "[vec-bone-smoke] A DOBRA FORTE: o MESMO par, com as duas juntas dobradas a \
             {DOBRA_FORTE}° (em cima o DESENHO VECTORIAL «Vector», em baixo a IMAGEM «Image»).\n\
             [vec-bone-smoke] 1) Olhe o lado de DENTRO de cada junta do desenho de cima: o \
             CONTORNO escuro faz um canto em «V», com a mesma forma da imagem de baixo -- sem um \
             laco nem um «olho» desenhado por dentro da curva.\n\
             [vec-bone-smoke] 2) Para ver o que era antes: feche o app e volte a abri-lo com \
             PH2D_SKIN_CONTACTO=0 no comando -- o contorno do desenho passa a cruzar-se por \
             dentro das duas juntas.\n\
             [vec-bone-smoke] 3) Para outras dobras: na Hierarquia clique em «Vector bone 2» e \
             escreva outro numero no campo de rotacao do Inspector. Abaixo de ~100 nao ha' \
             contacto e o desenho fica como sempre."
        );
        return;
    }
    println!(
        "[vec-bone-smoke] O PAR: a MESMA barra duas vezes, com o MESMO tamanho, a MESMA cor e um \
         esqueleto IGUAL de {OSSOS} ossos em cada. Em cima e' um DESENHO VECTORIAL («Vector»), em \
         baixo uma IMAGEM («Image»). Os dois esqueletos abrem dobrados da MESMA forma ({DOBRA}° por \
         junta, em Z).\n\
         [vec-bone-smoke] 1) Compare as duas barras: a forma das bordas, sobretudo junto das duas \
         juntas. O que for diferente entre elas e' a diferenca entre as duas midias.\n\
         [vec-bone-smoke] 2) Para outra pose, IGUAL nas duas: na Hierarquia clique em «Vector bone \
         2», e no Inspector escreva um numero no campo de rotacao. Depois clique em «Image bone 2» e \
         escreva o MESMO numero. Faca o mesmo com o osso 3 se quiser.\n\
         [vec-bone-smoke] 3) Tambem pode arrastar os ossos com a ferramenta Bone (painel Bones, \
         «Transform»), mas a mao nao repete o mesmo angulo nas duas -- para comparar, use os \
         numeros."
    );
}

#[cfg(test)]
#[path = "smoke_bone_par_fresta_tests.rs"]
mod fresta_tests;

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐⭐⭐ **O PAR CABE NA CÂMERA DE OMISSÃO e as duas peças não se tocam.**
    ///
    /// A câmera de omissão mostra `height_world` de alto, e a largura VISÍVEL é a do canvas entre
    /// as duas colunas laterais, que tapam `~37 %` da janela (medido pela família das mídias) — com
    /// uma janela `16:9` isso são `height × 16/9 × 0,63`. A barra exige `90 %` de cada um, para a
    /// peça não encostar à borda. ⚠️ No `ppm` de omissão do projecto, que é o da cena do dono.
    ///
    /// ⚠️ **Nos DOIS níveis desta cena** — a `=4` empilha peças dobradas a [`DOBRA_FORTE`], e a
    /// disposição da `=3` pô-las-ia uma por cima da outra.
    #[test]
    fn o_par_cabe_na_camera_de_omissao_e_nao_se_toca() {
        let ppm = ph2d_editor_core::DEFAULT_PIXELS_PER_METER;
        let cam = ph2d_render::Camera2d::default();
        for nivel in [3, 4] {
            let g = dobra_do_nivel(nivel);
            let [o_vec, o_img] = origens(ppm, nivel);
            let (a, b) = (caixa_dobrada(ppm, o_vec, g), caixa_dobrada(ppm, o_img, g));
            let (x0, x1) = (a[0].min(b[0]), a[2].max(b[2]));
            let (y0, y1) = (a[1].min(b[1]), a[3].max(b[3]));
            let h = f64::from(cam.height_world);
            let meia_largura = h * 16.0 / 9.0 * 0.63 * 0.9 / 2.0;
            let meia_altura = h * 0.9 / 2.0;
            for (v, lim, eixo) in [
                (x0.abs().max(x1.abs()), meia_largura, "x"),
                (y0.abs().max(y1.abs()), meia_altura, "y"),
            ] {
                assert!(
                    v <= lim,
                    "=`{nivel}`: o par vai a {v:.3} m do centro em {eixo}, e a camera de omissao \
                     mostra {lim:.3} — uma das pecas sai do ecra'"
                );
            }
            assert!(
                b[3] < a[1],
                "=`{nivel}`: a imagem (topo {:.3}) toca o desenho (base {:.3}) — a comparacao exige \
                 as duas SEPARADAS",
                b[3],
                a[1]
            );
        }
    }

    /// ⭐⭐ **A `=4` TEM contacto nas juntas e NÃO tem entre as pontas.** A dobra fica na faixa em que
    /// o desenho fiel se cruza (medida na barra da cena: de `110°` para cima), e o 3.º osso volta a
    /// deitar ACIMA do 1.º com mais de uma espessura entre os eixos — senão a cena mostraria dois
    /// membros das pontas a tocar-se, que é outra pergunta.
    #[test]
    fn a_dobra_forte_tem_contacto_nas_juntas_e_nao_nas_pontas() {
        let (l, t) = peca(ph2d_editor_core::DEFAULT_PIXELS_PER_METER);
        let passo = (l - t) / f64::from(OSSOS);
        assert!(
            (110.0..=130.0).contains(&DOBRA_FORTE),
            "a dobra forte saiu da faixa medida de contacto: {DOBRA_FORTE}"
        );
        let entre_eixos = passo * f64::from(DOBRA_FORTE).to_radians().sin();
        assert!(
            entre_eixos > t,
            "o 3.º osso deita a {entre_eixos:.3} do 1.º, com espessura {t:.3} — as pontas tocam-se"
        );
        assert!(
            (dobra_do_nivel(3) - DOBRA).abs() < f32::EPSILON,
            "a `=3` (a cena aprovada) mudou de pose"
        );
    }

    /// ⭐⭐ **As duas peças TÊM O MESMO TAMANHO** — o desenho copia o da imagem pela mesma porta, e
    /// a arte da imagem enche o quadro dela de ponta a ponta (sem margem, que a tornaria mais curta).
    #[test]
    fn as_duas_pecas_tem_o_mesmo_tamanho() {
        let px = pixels();
        let alfa = |x: u32, y: u32| px[((y * IMG_W + x) * 4 + 3) as usize];
        // O eixo da cápsula toca as duas bordas: o píxel do meio de cada ponta é (quase) opaco.
        assert!(alfa(0, IMG_H / 2) > 100 && alfa(IMG_W - 1, IMG_H / 2) > 100);
        // E a espessura é a altura inteira da imagem.
        assert!(alfa(IMG_W / 2, 0) > 100 && alfa(IMG_W / 2, IMG_H - 1) > 100);
        // ⛔ E fora da tampa é transparente — senão a imagem seria um rectângulo e não a cápsula.
        assert_eq!(alfa(0, 0), 0);
    }

    /// ⭐ **O prólogo desta cena fecha a timeline e abre o painel dos ossos — e NÃO enquadra.**
    ///
    /// ⛔ A metade negativa é a que a foto de 2026-09-29 escreveu: o `Frame All` conta só imagens e
    /// enquadrou a peça de baixo sozinha, com o desenho de cima fora do ecrã.
    #[test]
    fn o_prologo_do_par_fecha_a_timeline_e_nao_enquadra() {
        for nivel in [3, 4] {
            let p = crate::smoke_bone_envelope::prologo_do_nivel(nivel);
            assert!(
                p.timeline_fechada && p.painel_do_osso,
                "a cena do PAR (=`{nivel}`) abre sem o prologo de que precisa: {p:?}"
            );
            assert!(
                !p.enquadrar,
                "a cena do PAR (=`{nivel}`) voltou a pedir o Frame All, que enquadra so' a \
                 IMAGEM: {p:?}"
            );
        }
    }

    /// ⭐ **Os dois pedidos da foto só valem onde devem.** A dobra pedida só entra na `=4` (a `=3`
    /// é o PAR de comparação e tem de abrir sempre igual) e um número inválido cai no de fábrica —
    /// nunca num `NaN` que montasse a cena torta em silêncio.
    #[test]
    fn a_dobra_e_o_traco_pedidos_so_valem_onde_devem() {
        assert!((dobra_de(4, Some("145")) - 145.0).abs() < f32::EPSILON);
        assert_eq!(dobra_pedida(60.0, Some(" 100 ")), (100.0, 100.0));
        assert_eq!(dobra_pedida(60.0, Some("170,110")), (170.0, 110.0));
        assert_eq!(dobra_pedida(60.0, Some("170,x")), (60.0, 60.0));
        assert_eq!(dobra_pedida(60.0, None), (60.0, 60.0));
        assert_eq!(dobra_pedida(60.0, Some("inf")), (60.0, 60.0));
        assert!((dobra_de(4, None) - DOBRA_FORTE).abs() < f32::EPSILON);
        assert!((dobra_de(4, Some("nan")) - DOBRA_FORTE).abs() < f32::EPSILON);
        assert!((dobra_de(4, Some("x")) - DOBRA_FORTE).abs() < f32::EPSILON);
        assert!(
            (dobra_de(3, Some("145")) - DOBRA).abs() < f32::EPSILON,
            "a dobra pedida chegou ao nivel 3, o PAR de comparacao"
        );
        assert!((espessura_do_contorno(Some("0.3")) - 0.3).abs() < f64::EPSILON);
        assert!((espessura_do_contorno(Some("-1")) - ESPESSURA_DO_CONTORNO).abs() < f64::EPSILON);
        assert!((espessura_do_contorno(None) - ESPESSURA_DO_CONTORNO).abs() < f64::EPSILON);
    }
}
