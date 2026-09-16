//! Os gates da 2.ª mídia — a régua da imagem contra a régua do quad.

use super::*;
use ph2d_ecs::Transform;
use ph2d_poly2d::GridOptions;

/// O `pixels_per_meter` de omissão do projecto.
const PPM: f32 = 100.0;

/// A escala da câmera das fixturas, em pixels de ecrã por metro de mundo.
const PX_POR_METRO: f64 = 50.0;

/// Uma sprite de `size` metros locais, com a âncora dada.
fn sprite(w: f32, h: f32, ax: f32, ay: f32) -> Sprite {
    Sprite {
        anchor: [ax, ay],
        ..Sprite::atlas(0, [w, h], [1.0; 4])
    }
}

/// Tinta opaca num rectângulo `w×h`, com as margens dadas.
fn tinta(w: u32, h: u32, mx: usize, my: usize) -> Vec<u8> {
    let (w, h) = (w as usize, h as usize);
    let mut rgba = vec![0u8; w * h * 4];
    for y in my..h - my {
        for x in mx..w - mx {
            rgba[(y * w + x) * 4 + 3] = 255;
        }
    }
    rgba
}

/// A instância que o extract emite para a sprite `s`: o quad DELA, com a âncora resolvida e o espelho.
fn instancia_de(s: &Sprite) -> RenderInstance {
    RenderInstance {
        world_pos: [0.0, 0.0],
        size: s.size,
        atlas_uv: [0.0, 0.0, 1.0, 1.0],
        tint: [1.0; 4],
        basis: RenderInstance::IDENTITY_BASIS,
        texture_id: 0,
        premultiplied: 0.0,
        anchor: s.resolve_anchor(PPM),
        per_corner_tint: [[1.0; 4]; 4],
        opacity: 1.0,
        flip_uv: RenderInstance::pack_flip_flags(s.flip_x, s.flip_y, false),
        z_order: 0,
        sampling: 0,
        uv_xform: RenderInstance::IDENTITY_UV_XFORM,
        clip_group: RenderInstance::CLIP_GROUP_NONE,
        clip_meta: 0,
        sub_order: 0,
    }
}

/// ⭐⭐⭐ **O CANTO DA IMAGEM É O CANTO DO QUAD, E O `y` VIRA.**
///
/// ⛔⛔ **Esquecer a inversão desenha o personagem de cabeça para baixo** — um defeito que passa por
/// TODO gate de geometria (a malha está certa, a deformação está certa, e a imagem está ao
/// contrário). Uma imagem tem o `y` a crescer para baixo; o mundo tem-no a crescer para cima.
///
/// (Mutação: tirar o sinal de `-sy / h` ⇒ RED nos dois cantos de `y`.)
#[test]
fn the_image_corner_is_the_quad_corner_and_the_y_axis_flips() {
    // Uma imagem de 100×50 pixels ocupando 4×2 unidades de mundo, centrada no pivô.
    let s = sprite(4.0, 2.0, 0.0, 0.0);
    let m = pixel_to_local(&s, [100, 50], PPM).expect("a imagem tem lado");
    let perto = |a: [f64; 2], b: [f64; 2]| (a[0] - b[0]).abs() < 1e-9 && (a[1] - b[1]).abs() < 1e-9;
    // (0,0) é o canto SUPERIOR esquerdo da textura ⇒ o canto de cima-esquerda do quad.
    assert!(
        perto(m.apply([0.0, 0.0]), [-2.0, 1.0]),
        "o pixel (0,0) saiu em {:?} — ele e' o canto de CIMA a' esquerda",
        m.apply([0.0, 0.0])
    );
    // (w,h) é o canto inferior direito.
    assert!(
        perto(m.apply([100.0, 50.0]), [2.0, -1.0]),
        "o pixel (w,h) saiu em {:?}",
        m.apply([100.0, 50.0])
    );
    // E o centro da imagem cai no pivô.
    assert!(perto(m.apply([50.0, 25.0]), [0.0, 0.0]));
}

/// ⭐⭐ **A ÂNCORA DESLOCA O QUAD INTEIRO, e não a régua da imagem.**
///
/// ⚠️ Ela é o vector do pivô ao **centro** do quad, então move os quatro cantos por igual — se
/// entrasse na escala, uma sprite com pivô fora do centro sairia esticada.
#[test]
fn the_anchor_moves_the_whole_quad_and_not_the_scale() {
    let sem = pixel_to_local(&sprite(4.0, 2.0, 0.0, 0.0), [100, 50], PPM).expect("lado");
    let com = pixel_to_local(&sprite(4.0, 2.0, 1.5, -0.25), [100, 50], PPM).expect("lado");
    for p in [[0.0, 0.0], [100.0, 50.0], [37.0, 11.0]] {
        let (a, b) = (sem.apply(p), com.apply(p));
        assert!(
            (b[0] - a[0] - 1.5).abs() < 1e-9 && (b[1] - a[1] + 0.25).abs() < 1e-9,
            "o ponto {p:?} deslocou {:?} em vez do vector da ancora",
            [b[0] - a[0], b[1] - a[1]]
        );
    }
    // ⛔ Uma imagem de lado zero não tem régua.
    assert_eq!(
        pixel_to_local(&sprite(4.0, 2.0, 0.0, 0.0), [0, 50], PPM),
        None
    );
}

/// ⭐⭐ **SÓ O CANAL ALFA DECIDE A SILHUETA.**
///
/// ⚠️ Passar as três cores junto daria ao traçador três respostas para a mesma pergunta. Este gate
/// mede-o pelo caso que separa: um quadrado **preto** e um **branco** com o mesmo alfa dão a mesma
/// malha, e um transparente não dá nenhuma.
#[test]
fn only_the_alpha_channel_decides_the_silhouette() {
    let faz = |cor: [u8; 3], alfa: u8| -> Vec<u8> {
        let mut v = vec![0u8; 20 * 20 * 4];
        for y in 5..15 {
            for x in 5..15 {
                let i = (y * 20 + x) * 4;
                v[i..i + 3].copy_from_slice(&cor);
                v[i + 3] = alfa;
            }
        }
        v
    };
    let opts = GridOptions::default();
    let sem_focos: &[[f64; 2]] = &[];
    let preto = mesh_from_rgba(&faz([0, 0, 0], 255), 20, 20, sem_focos, opts).expect("ha' alfa");
    let branco =
        mesh_from_rgba(&faz([255, 255, 255], 255), 20, 20, sem_focos, opts).expect("ha' alfa");
    assert_eq!(
        preto, branco,
        "a cor mudou a malha — so' o ALFA pode decidir a silhueta"
    );
    assert_eq!(
        mesh_from_rgba(&faz([255, 255, 255], 0), 20, 20, sem_focos, opts),
        None,
        "tinta branca com alfa ZERO nao e' tinta"
    );
}

/// ⭐⭐⭐ **UMA IMAGEM PRESA A UM ESQUELETO PARADO NÃO SE MEXE UM PIXEL.**
///
/// ⚠️ É a mesma lei que o bind de uma forma já declara (*«a pose de repouso é a identidade por
/// construção, logo prender não move um pixel»*), e ela é o **controlo** de tudo o resto: sem
/// ela, um erro de sinal na régua da imagem passaria despercebido no meio da deformação.
///
/// (Mutação: qualquer troca de sinal no [`pixel_to_local`] ⇒ RED.)
#[test]
fn binding_an_image_to_a_still_skeleton_moves_nothing() {
    let mut sim = SimWorld::default();
    // Um osso deitado sobre o eixo X, e uma imagem por cima dele.
    let osso = crate::bone::create(&mut sim, None, [0.0, 0.0], [4.0, 0.0]).expect("osso");
    let e = sim
        .world_mut()
        .spawn((Transform::IDENTITY, sprite(4.0, 2.0, 0.0, 0.0)))
        .id();
    assert!(
        crate::skin_live::bind_image(
            &mut sim,
            e,
            &tinta(40, 20, 4, 4),
            [40, 20],
            PPM,
            GridOptions::default(),
            Some(Entity::from_bits(osso)),
        ),
        "o bind tinha de acontecer: ha' osso, ha' tinta e a pose nao e' singular"
    );
    let malha = skinned_mesh_of(&sim, e).expect("a malha esta' guardada nos bytes opacos");
    let posados = posed_local(&sim, e, &malha).expect("a pele resolve");
    let p2l = pixel_to_local(
        sim.world().get::<Sprite>(e).expect("sprite"),
        malha.mesh.size,
        PPM,
    )
    .expect("regua");
    for (i, &r) in malha.mesh.rest.iter().enumerate() {
        let repouso = p2l.apply(r);
        let agora = posados[i];
        assert!(
            (agora[0] - repouso[0]).abs() < 1e-6 && (agora[1] - repouso[1]).abs() < 1e-6,
            "o vertice {i} saiu de {repouso:?} para {agora:?} sem ninguem mexer no osso"
        );
    }
}

/// ⭐⭐⭐ **GIRAR O OSSO LEVA A IMAGEM COM ELE** — a prova de que a 2.ª mídia de facto deforma.
///
/// ⚠️ E o **controlo** está dentro do gate: um ponto longe do alcance do osso tem de ficar onde
/// estava. Sem ele, um bug que movesse TUDO por igual (uma translação global) passaria.
#[test]
fn turning_the_bone_carries_the_image() {
    let mut sim = SimWorld::default();
    let osso = crate::bone::create(&mut sim, None, [-2.0, 0.0], [2.0, 0.0]).expect("osso");
    let e = sim
        .world_mut()
        .spawn((Transform::IDENTITY, sprite(4.0, 2.0, 0.0, 0.0)))
        .id();
    let raiz = Entity::from_bits(osso);
    assert!(crate::skin_live::bind_image(
        &mut sim,
        e,
        &tinta(40, 20, 4, 4),
        [40, 20],
        PPM,
        GridOptions::default(),
        Some(raiz),
    ));
    let malha = skinned_mesh_of(&sim, e).expect("malha");
    let antes = posed_local(&sim, e, &malha).expect("pele");
    // Gira o osso um quarto de volta.
    {
        let mut t = sim
            .world_mut()
            .get_mut::<Transform>(raiz)
            .expect("Transform");
        t.rotation = std::f32::consts::FRAC_PI_2;
    }
    let depois = posed_local(&sim, e, &malha).expect("pele");
    let mexeu = antes
        .iter()
        .zip(&depois)
        .map(|(a, b)| (a[0] - b[0]).hypot(a[1] - b[1]))
        .fold(0.0f64, f64::max);
    assert!(
        mexeu > 0.5,
        "girar o osso nao moveu a malha (maior deslocamento {mexeu}) — a imagem nao obedece"
    );
}

/// A sprite de um caso do gate de repouso: `4×2` metros com a âncora deslocada, e as três escolhas
/// de autoria que mudam onde o quad se desenha.
fn sprite_do_caso(centered: bool, offset: [f32; 2], flip_x: bool, flip_y: bool) -> Sprite {
    Sprite {
        centered,
        offset,
        flip_x,
        flip_y,
        ..sprite(4.0, 2.0, 0.25, -0.5)
    }
}

/// ⭐⭐⭐ **EM REPOUSO, CADA PIXEL DA IMAGEM PRESA É LIDO ONDE O QUAD O LÊ** — com *Centered*
/// desligado, *Offset* e espelho.
///
/// ⛔⛔ **O quinto defeito da F6-h:** a régua da imagem lia a âncora CRUA, e o quad desenha-se na
/// RESOLVIDA ([`Sprite::resolve_anchor`]) — com *Centered* desligado a malha ficava meio quad ao lado
/// da arte. E o espelho: o shader espelha a UV do quad, então a régua tem de espelhar a POSIÇÃO.
///
/// ⚠️ **O gate escreve a lei do shader por extenso** (`quad_pos = (local − anchor)/size`,
/// `uv = (qx + ½, ½ − qy)`, e o espelho aplicado à UV) e faz duas perguntas a cada vértice: a UV dele
/// é a do quad no ponto onde ele está? e o TEXEL que o shader lê com ela é o pixel de onde o vértice
/// veio? O controlo é a sprite de sempre (centrada, sem offset, sem espelho).
///
/// (Mutações: a âncora crua no `pixel_to_local` ⇒ RED nos dois casos deslocados; sem espelho na
/// posição ⇒ RED nos dois espelhados.)
#[test]
fn at_rest_each_pixel_of_a_bound_image_is_read_where_the_quad_reads_it() {
    let casos: [(&str, bool, [f32; 2], bool, bool); 4] = [
        ("controlo", true, [0.0, 0.0], false, false),
        ("nao centrada + offset", false, [12.0, -7.0], false, false),
        ("espelho em X + offset", true, [5.0, 3.0], true, false),
        (
            "nao centrada + espelho em Y",
            false,
            [0.0, 0.0],
            false,
            true,
        ),
    ];
    for (nome, centered, offset, flip_x, flip_y) in casos {
        let mut sim = SimWorld::default();
        let osso = crate::bone::create(&mut sim, None, [-2.0, 0.0], [2.0, 0.0]).expect("osso");
        let e = sim
            .world_mut()
            .spawn((
                Transform::IDENTITY,
                sprite_do_caso(centered, offset, flip_x, flip_y),
            ))
            .id();
        assert!(
            crate::skin_live::bind_image(
                &mut sim,
                e,
                &tinta(40, 20, 4, 4),
                [40, 20],
                PPM,
                GridOptions::default(),
                Some(Entity::from_bits(osso)),
            ),
            "{nome}: o bind tinha de acontecer"
        );
        let inst = instancia_de(&sprite_do_caso(centered, offset, flip_x, flip_y));
        let mut present = PresentWorld::new();
        let p = present.world_mut().spawn((SimRef(e), inst)).id();
        assert_eq!(
            attach_skin_meshes(&sim, &mut present, PPM, None, PX_POR_METRO, &[]),
            1,
            "{nome}: a instancia da imagem presa nao recebeu malha"
        );
        let malha = present
            .world()
            .get::<SpriteMesh>(p)
            .expect("a malha esta' na instancia")
            .clone();
        let guardada = mesh_of(&sim, e).expect("malha guardada");
        assert_eq!(malha.local.len(), guardada.rest.len(), "{nome}");
        let (w, h) = (f64::from(guardada.size[0]), f64::from(guardada.size[1]));
        for (i, r) in guardada.rest.iter().enumerate() {
            let (l, uv) = (malha.local[i], malha.uv[i]);
            // A lei do shader, por extenso.
            let q = [
                (l[0] - inst.anchor[0]) / inst.size[0],
                (l[1] - inst.anchor[1]) / inst.size[1],
            ];
            let do_quad = [q[0] + 0.5, 0.5 - q[1]];
            assert!(
                (uv[0] - do_quad[0]).abs() < 1e-4 && (uv[1] - do_quad[1]).abs() < 1e-4,
                "{nome}: o vertice {i} leva a UV {uv:?} e o quad naquele ponto leva {do_quad:?}"
            );
            let texel = [
                if flip_x { 1.0 - uv[0] } else { uv[0] },
                if flip_y { 1.0 - uv[1] } else { uv[1] },
            ];
            let pixel = [f64::from(texel[0]) * w, f64::from(texel[1]) * h];
            assert!(
                (pixel[0] - r[0]).abs() < 1e-2 && (pixel[1] - r[1]).abs() < 1e-2,
                "{nome}: o vertice {i} veio do pixel {r:?} e o shader le o pixel {pixel:?}"
            );
        }
    }
}

/// ⭐⭐⭐ **SÓ A INSTÂNCIA BASE DO QUAD DA SPRITE RECEBE A MALHA** — e só se o extract a emitiu.
///
/// ⚠️ **A visibilidade não é perguntada outra vez:** uma sprite escondida pelo olho não tem
/// instância, e é por isso que não recebe malha. Três imagens presas, três destinos:
/// - `a` tem a instância base ⇒ malha (o controlo); a célula fantasma dela (`SlicePatchMirror`), com
///   a MESMA `SimRef` e o mesmo quad, fica no quad;
/// - a do meio não tem instância (escondida) ⇒ nada;
/// - `c` só tem uma instância cujo quad não é o dela (o 1.º patch de um 9-slice) ⇒ fica no quad.
///
/// (Mutações: tirar o `Without<SlicePatchMirror>` ⇒ a fantasma rouba a malha à base; tirar a
/// comparação com o quad da sprite ⇒ o patch de `c` recebe a malha do quad inteiro.)
#[test]
fn only_the_base_instance_of_the_sprites_own_quad_gets_the_mesh() {
    let mut sim = SimWorld::default();
    let osso = crate::bone::create(&mut sim, None, [-2.0, 0.0], [2.0, 0.0]).expect("osso");
    let raiz = Some(Entity::from_bits(osso));
    let mut presas = Vec::new();
    for _ in 0..3 {
        let e = sim
            .world_mut()
            .spawn((Transform::IDENTITY, sprite(4.0, 2.0, 0.0, 0.0)))
            .id();
        assert!(crate::skin_live::bind_image(
            &mut sim,
            e,
            &tinta(40, 20, 4, 4),
            [40, 20],
            PPM,
            GridOptions::default(),
            raiz,
        ));
        presas.push(e);
    }
    let &[a, _escondida, c] = presas.as_slice() else {
        panic!("fixtura: tres imagens presas");
    };
    let s = sprite(4.0, 2.0, 0.0, 0.0);
    let mut present = PresentWorld::new();
    let base_a = present
        .world_mut()
        .spawn((SimRef(a), instancia_de(&s)))
        .id();
    let fantasma_a = present
        .world_mut()
        .spawn((SimRef(a), instancia_de(&s), SlicePatchMirror))
        .id();
    let mut patch = instancia_de(&s);
    patch.size = [1.0, 0.5];
    patch.anchor = [-1.5, 0.75];
    let patch_c = present.world_mut().spawn((SimRef(c), patch)).id();

    assert_eq!(
        attach_skin_meshes(&sim, &mut present, PPM, None, PX_POR_METRO, &[]),
        1,
        "so' a imagem com o quad dela emitido podia receber malha"
    );
    assert!(
        present.world().get::<SpriteMesh>(base_a).is_some(),
        "a instancia base da imagem visivel nao recebeu malha"
    );
    assert!(
        present.world().get::<SpriteMesh>(fantasma_a).is_none(),
        "a celula fantasma (mesma SimRef, SlicePatchMirror) recebeu a malha"
    );
    assert!(
        present.world().get::<SpriteMesh>(patch_c).is_none(),
        "o patch de um 9-slice recebeu a malha do quad inteiro da sprite"
    );
}

/// ⭐⭐⭐ **O ORÇAMENTO DO `Smooth` É DO QUADRO, e reparte-se pelas imagens presas.**
///
/// ⚠️ Um tecto por IMAGEM não é um tecto do quadro: N imagens presas multiplicam-no.
///
/// ⚠️ **A fixtura escolhe o orçamento para as duas leis darem respostas diferentes:** com
/// `B = 9 t` (`t` triângulos por imagem), um tecto por imagem deixa cada uma refinar a `k = 3` e o
/// quadro emite `18 t > B`; repartido, cada uma recebe `4,5 t` e fica em `k = 2` (`8 t ≤ B`). ⚠️ E
/// as duas têm de refinar — um orçamento que coubesse por dar tudo a uma e nada à outra passaria na
/// primeira metade.
///
/// ⚠️ **Dois ossos, e o de baixo dobrado**: com um osso só a pele é um movimento rígido, o campo é
/// linear, o `Smooth` pede `k = 1` e o gate compararia `Fast` com `Fast`.
#[test]
fn the_smooth_pieces_of_all_skinned_images_share_one_frame_budget() {
    let mut sim = SimWorld::default();
    let raiz = crate::bone::create(&mut sim, None, [-2.0, 0.0], [0.0, 0.0]).expect("raiz");
    let raiz = Entity::from_bits(raiz);
    let ponta = crate::bone::create(&mut sim, Some(raiz), [0.0, 0.0], [2.0, 0.0]).expect("ponta");
    let s = sprite(4.0, 2.0, 0.0, 0.0);
    let mut present = PresentWorld::new();
    let mut instancias = Vec::new();
    for _ in 0..2 {
        let e = sim
            .world_mut()
            .spawn((Transform::IDENTITY, sprite(4.0, 2.0, 0.0, 0.0)))
            .id();
        assert!(crate::skin_live::bind_image(
            &mut sim,
            e,
            &tinta(40, 20, 2, 4),
            [40, 20],
            PPM,
            GridOptions::default(),
            Some(raiz),
        ));
        instancias.push(
            present
                .world_mut()
                .spawn((SimRef(e), instancia_de(&s)))
                .id(),
        );
    }
    sim.world_mut()
        .get_mut::<Transform>(Entity::from_bits(ponta))
        .expect("Transform da ponta")
        .rotation = 1.0;

    let mut desenha = |modo: Option<RefineOptions>| -> Vec<usize> {
        for &p in &instancias {
            present.world_mut().entity_mut(p).remove::<SpriteMesh>();
        }
        assert_eq!(
            attach_skin_meshes(&sim, &mut present, PPM, modo, PX_POR_METRO, &[]),
            2,
            "as duas imagens presas tinham de receber malha"
        );
        instancias
            .iter()
            .map(|&p| {
                present
                    .world()
                    .get::<SpriteMesh>(p)
                    .expect("malha")
                    .tris
                    .len()
            })
            .collect()
    };

    let fast = desenha(None);
    assert_eq!(
        fast[0], fast[1],
        "fixtura: as duas malhas tinham de ter o mesmo tamanho"
    );
    let t = fast[0];
    assert!(
        t > 1,
        "fixtura: a malha guardada tem de ter mais de um triangulo"
    );

    let orcamento = 9 * t;
    let suave = desenha(Some(RefineOptions {
        tolerance_px: 1e-3,
        max_pieces: orcamento,
        ..RefineOptions::default()
    }));
    let total: usize = suave.iter().sum();
    assert!(
        total <= orcamento,
        "o quadro emitiu {total} pecas contra um orcamento de {orcamento} ({suave:?}) — o tecto \
         esta' a ser aplicado POR IMAGEM, e N imagens presas multiplicam-no"
    );
    assert!(
        suave.iter().all(|&n| n > t),
        "o orcamento coube por nao refinar uma das imagens ({suave:?} contra {t} no Fast)"
    );
}
/// ⏱️ **A SONDA DO CUSTO por quadro** — filha por ASSUNTO (e pelo tecto de LOC).
#[path = "skin_image_cost_tests.rs"]
mod custo;

/// **A malha, já POSADA** — os vértices de repouso levados pela pele para onde eles estão agora.
///
/// ⚠️ Vive no ARNÊS: no produto quem pergunta ao campo é o [`attach_skin_meshes`], que pode pedir-lhe
/// **mais** pontos que os vértices da malha (o `Smooth`). Uma segunda porta no produto que só sabe
/// perguntar pelos vértices seria a segunda resposta à mesma pergunta.
/// ⚠️⚠️ **Ela lê a MALHA GUARDADA INTEIRA (com os pesos), e não só a geometria** — desde que a pele
/// de imagem passou ao padrão-ouro, medir com `pele.point` mediria a lei **euclidiana** enquanto o
/// produto desenha com a tabela resolvida no bind. *Um arnês que não usa a lei do produto mede um
/// programa que ninguém corre.*
fn posed_local(sim: &SimWorld, e: Entity, sm: &SkinnedMesh) -> Option<Vec<[f64; 2]>> {
    let (p2l, pele) = deform_field(sim, e, sm.mesh.size, PPM)?;
    let mut w = pele.scratch();
    Some(
        sm.mesh
            .rest
            .iter()
            .enumerate()
            .map(|(v, &p)| {
                let q = p2l.apply(p);
                let pesos = sm.pesos_de(v);
                if pesos.is_empty() {
                    pele.point(q, &mut w)
                } else {
                    pele.point_with(q, pesos, &mut w)
                }
            })
            .collect(),
    )
}

/// A pele NOUTRO INSTANTE (a fonte de poses injectada) — filho por assunto, e aqui dentro para
/// herdar as fixturas deste arnês (uma cópia delas divergiria no primeiro ajuste).
#[path = "skin_at_time_tests.rs"]
mod at_time;

/// A SUSPENSÃO da pele (pintar achata a arte) — filho por assunto, e aqui dentro para herdar as
/// fixturas deste arnês (uma cópia delas divergiria no primeiro ajuste). Saiu para ficheiro próprio
/// em 2026-09-15, pelo teto de LOC por ficheiro.
#[path = "skin_suspend_tests.rs"]
mod suspensao;
