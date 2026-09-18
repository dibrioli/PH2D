//! ⏱️ **A SONDA DO «QUAD QUE NÃO É O DELA»** — filho do [`super`] para herdar as fixturas dele.
//!
//! ⚠️ **Ela existe porque a queixa tem DUAS formas e só UMA se queixa.** O
//! [`super::super::avisa_quad_que_nao_e_o_da_sprite`] fala quando a instância não é o quad da
//! sprite (o 9-slice), e **cala-se** quando a instância É o quad e a MALHA é que não lhe pertence
//! (a folha de quadros, cuja malha é traçada sobre a folha INTEIRA). *Medir é o que separa «não
//! desenha» de «desenha errado em silêncio».*

use super::*;

/// Tinta de uma FOLHA `cells × 1`: cada célula tem uma barra opaca de altura diferente, e as
/// colunas entre elas ficam vazias — assim a silhueta diz QUE células entraram na malha.
fn folha(cells: u32, cw: u32, ch: u32) -> Vec<u8> {
    let (w, h) = ((cells * cw) as usize, ch as usize);
    let mut rgba = vec![0u8; w * h * 4];
    for c in 0..cells as usize {
        let alto = ch as usize * (c + 1) / (cells as usize + 1);
        for y in 0..alto {
            for x in c * cw as usize + 1..(c + 1) * cw as usize - 1 {
                rgba[(y * w + x) * 4 + 3] = 255;
            }
        }
    }
    rgba
}

/// A caixa de `local` e a de `uv` de uma malha desenhada.
fn caixas(m: &SpriteMesh) -> ([f32; 4], [f32; 4]) {
    let mut l = [f32::MAX, f32::MAX, f32::MIN, f32::MIN];
    let mut u = [f32::MAX, f32::MAX, f32::MIN, f32::MIN];
    for p in &m.local {
        l[0] = l[0].min(p[0]);
        l[1] = l[1].min(p[1]);
        l[2] = l[2].max(p[0]);
        l[3] = l[3].max(p[1]);
    }
    for p in &m.uv {
        u[0] = u[0].min(p[0]);
        u[1] = u[1].min(p[1]);
        u[2] = u[2].max(p[0]);
        u[3] = u[3].max(p[1]);
    }
    (l, u)
}

/// ⏱️ **SONDA (`--ignored`) — o que HOJE acontece às três formas de sprite presa a um osso.**
#[test]
#[ignore = "sonda: imprime a tabela, sem barra"]
fn sonda_as_tres_formas() {
    // (a) CONTROLO: a sprite simples, a arte inteira num quad só.
    medir("simples", ph2d_ecs::SpriteGrid::SINGLE, |_s, i| {
        vec![(i, i.atlas_uv)]
    });
    // (b) A FOLHA de 4 quadros: a instância É o quad da sprite (passa o guarda), e o `atlas_uv`
    //     recorta a célula viva. ⚠️ A malha do bind foi traçada sobre a folha INTEIRA.
    medir(
        "folha 4x1 (celula 1)",
        ph2d_ecs::SpriteGrid {
            hframes: 4,
            vframes: 1,
            frame: 1,
        },
        |_s, mut i| {
            i.atlas_uv = [0.25, 0.0, 0.5, 1.0];
            vec![(i, i.atlas_uv)]
        },
    );
    // (c) O 9-SLICE: nove instâncias, cada uma com o seu sub-rect, tamanho e deslocamento.
    medir("9-slice", ph2d_ecs::SpriteGrid::SINGLE, |s, i| {
        let slice = ph2d_ecs::SliceNine {
            draw_mode: ph2d_ecs::SliceDrawMode::Sliced,
            // `[left, top, right, bottom]`, em pixels da FONTE.
            borders: [8.0, 4.0, 8.0, 4.0],
            ..ph2d_ecs::SliceNine::INERT
        };
        ph2d_render::nine_slice::nine_slice_patches(
            [0.0, 0.0, 1.0, 1.0],
            [40.0, 20.0],
            &slice,
            s.size,
            PPM,
            [1.0, 1.0],
        )
        .iter()
        .filter_map(|p| {
            let p = p.as_ref()?;
            let mut ri = i;
            ri.atlas_uv = p.uv;
            ri.size = p.size;
            ri.anchor = [
                i.anchor[0] + p.center_offset[0],
                i.anchor[1] + p.center_offset[1],
            ];
            // ⚠️ A fracção da célula, como o extract a publica — aqui à mão, porque a porta dele
            // vive na shell. Quem prova a COSTURA (o extract publica ⇒ isto consome) é o gate do
            // lado de lá; esta sonda mede o que a deformação faz com o facto.
            Some((ri, p.uv))
        })
        .collect()
    });
}

/// Monta a cena (a arte presa a uma corrente DOBRADA), emite as instâncias que `emite` devolve, e
/// imprime o que cada uma recebeu.
fn medir(
    nome: &str,
    grid: ph2d_ecs::SpriteGrid,
    emite: impl Fn(&Sprite, RenderInstance) -> Vec<(RenderInstance, [f32; 4])>,
) {
    let cells = grid.hframes.max(1) * grid.vframes.max(1);
    let mut sim = SimWorld::default();
    let raiz = crate::bone::create(&mut sim, None, [-2.0, 0.0], [0.0, 0.0]).expect("raiz");
    let raiz = Entity::from_bits(raiz);
    let ponta = crate::bone::create(&mut sim, Some(raiz), [0.0, 0.0], [2.0, 0.0]).expect("ponta");
    let s = sprite(4.0, 2.0, 0.0, 0.0);
    let e = sim.world_mut().spawn((Transform::IDENTITY, s, grid)).id();
    let (cw, ch) = (40u32, 20u32);
    assert!(crate::skin_live::bind_image(
        &mut sim,
        e,
        &folha(cells, cw, ch),
        [cells * cw, ch],
        PPM,
        ph2d_poly2d::GridOptions::default(),
        Some(raiz),
    ));
    sim.world_mut()
        .get_mut::<Transform>(Entity::from_bits(ponta))
        .expect("pose")
        .rotation = 0.6;

    let mut present = PresentWorld::new();
    let base = instancia_de(&s);
    let celula = base.atlas_uv;
    let (du, dv) = (celula[2] - celula[0], celula[3] - celula[1]);
    let ps: Vec<Entity> = emite(&s, base)
        .into_iter()
        .map(|(ri, uv)| {
            let fonte = ph2d_render::nine_slice::SlicePatchSource {
                frac: [
                    (uv[0] - celula[0]) / du,
                    (uv[1] - celula[1]) / dv,
                    (uv[2] - celula[0]) / du,
                    (uv[3] - celula[1]) / dv,
                ],
            };
            present.world_mut().spawn((SimRef(e), ri, fonte)).id()
        })
        .collect();
    let feitas = attach_skin_meshes(&sim, &mut present, PPM, &[]);
    println!("-- {nome}: {} instancia(s), {feitas} com malha", ps.len());
    for (k, p) in ps.iter().enumerate() {
        match present.world().get::<SpriteMesh>(*p) {
            Some(m) => {
                let (l, u) = caixas(m);
                println!(
                    "   [{k}] {:>5} pecas | local [{:+.3},{:+.3}]..[{:+.3},{:+.3}] | uv \
                     [{:.3},{:.3}]..[{:.3},{:.3}]",
                    m.tris.len(),
                    l[0],
                    l[1],
                    l[2],
                    l[3],
                    u[0],
                    u[1],
                    u[2],
                    u[3]
                );
            }
            None => println!("   [{k}] SEM MALHA — desenha-se como quad, sem deformar"),
        }
    }
}

/// Prende `rgba` (`src` px) a uma corrente, com a grelha dada, e devolve a malha guardada.
fn malha_do_bind(
    rgba: &[u8],
    src: [u32; 2],
    grid: ph2d_ecs::SpriteGrid,
) -> crate::skinned_mesh::SkinnedMesh {
    let mut sim = SimWorld::default();
    let raiz = crate::bone::create(&mut sim, None, [-2.0, 0.0], [2.0, 0.0]).expect("osso");
    let s = sprite(4.0, 2.0, 0.0, 0.0);
    let e = sim.world_mut().spawn((Transform::IDENTITY, s, grid)).id();
    assert!(crate::skin_live::bind_image(
        &mut sim,
        e,
        rgba,
        src,
        PPM,
        ph2d_poly2d::GridOptions::default(),
        Some(Entity::from_bits(raiz)),
    ));
    skinned_mesh_of(&sim, e).expect("a pele guarda a malha")
}

/// Tinta de uma folha `cells × 1` em que SÓ a célula `viva` tem tinta, e só na METADE dela que
/// `esquerda` escolhe — o discriminador da união.
fn folha_meia_celula(cells: u32, cw: u32, ch: u32, viva: u32, esquerda: bool) -> Vec<u8> {
    let (w, h) = ((cells * cw) as usize, ch as usize);
    let mut rgba = vec![0u8; w * h * 4];
    let x0 = viva as usize * cw as usize + if esquerda { 0 } else { cw as usize / 2 };
    for y in 2..h - 2 {
        for x in x0 + 1..x0 + cw as usize / 2 - 1 {
            rgba[(y * w + x) * 4 + 3] = 255;
        }
    }
    rgba
}

/// A caixa dos vértices de repouso de uma malha.
fn caixa_rest(m: &ph2d_poly2d::Mesh2d) -> [f64; 4] {
    let mut b = [f64::MAX, f64::MAX, f64::MIN, f64::MIN];
    for p in &m.rest {
        b[0] = b[0].min(p[0]);
        b[1] = b[1].min(p[1]);
        b[2] = b[2].max(p[0]);
        b[3] = b[3].max(p[1]);
    }
    b
}

/// ⭐⭐⭐ **A MALHA DE UMA FOLHA É A DA CÉLULA, NUNCA A DA FOLHA** (F11, 2026-09-17).
///
/// ⛔⛔ **O defeito que isto fecha era MUDO:** a malha era traçada sobre a folha inteira e o
/// `pixel_to_local` espremia-a no quad de UMA célula — medido, uma folha `4×1` desenhava `1 277`
/// peças recortadas dos quatro quadros dentro do sítio de um, **sem aviso nenhum**. O `size` da
/// malha é a testemunha: ele é a régua que o desenho lê (`deform_field(.., mesh.mesh.size, ..)`).
///
/// ⚠️ **E o CONTROLO é a mesma arte sem grelha** — sem ele isto passaria com um bind que ignorasse
/// a imagem e cravasse um tamanho qualquer.
///
/// (Mutação: ler `size_px` em vez de `cells.cell_px()` ⇒ RED com `160×20`.)
#[test]
fn a_malha_de_uma_folha_e_a_da_celula() {
    let arte = folha(4, 40, 20);
    let folha_4 = malha_do_bind(
        &arte,
        [160, 20],
        ph2d_ecs::SpriteGrid {
            hframes: 4,
            vframes: 1,
            frame: 0,
        },
    );
    assert_eq!(
        folha_4.mesh.size,
        [40, 20],
        "a malha de uma folha 4x1 tem de medir UMA celula — com a folha inteira ela e' espremida \
         no quad de uma celula, que e' o defeito mudo de 2026-09-17"
    );
    let sem_grelha = malha_do_bind(&arte, [160, 20], ph2d_ecs::SpriteGrid::SINGLE);
    assert_eq!(
        sem_grelha.mesh.size,
        [160, 20],
        "sem grelha a celula E' a imagem: este e' o controlo que impede um tamanho cravado"
    );
}

/// ⭐⭐⭐ **A TINTA DA CÉLULA É A UNIÃO DE TODOS OS QUADROS — e a razão é a ANIMAÇÃO.**
///
/// ⛔⛔ **Com o quadro VIVO em vez da união, o artista prende no quadro `0`, dá play, e a arte do
/// quadro `1` some** — ela cai fora da malha e é recortada. Aqui os dois quadros têm tinta em
/// METADES OPOSTAS da célula: a união cobre a largura toda, o quadro vivo cobriria metade.
///
/// (Mutação: varrer só `cells.count() == 1` ⇒ RED, a caixa fica na metade esquerda.)
#[test]
fn a_tinta_da_celula_cobre_todos_os_quadros() {
    let (cw, ch) = (40u32, 20u32);
    let mut arte = folha_meia_celula(2, cw, ch, 0, true);
    for (i, b) in folha_meia_celula(2, cw, ch, 1, false).iter().enumerate() {
        arte[i] = arte[i].max(*b);
    }
    let m = malha_do_bind(
        &arte,
        [2 * cw, ch],
        ph2d_ecs::SpriteGrid {
            hframes: 2,
            vframes: 1,
            frame: 0,
        },
    );
    let b = caixa_rest(&m.mesh);
    assert!(
        b[0] < f64::from(cw) * 0.3 && b[2] > f64::from(cw) * 0.7,
        "a malha cobriu [{:.1}, {:.1}] de {cw} px — ela tem de cobrir a UNIAO dos dois quadros, \
         senao a arte de um deles e' recortada ao dar play",
        b[0],
        b[2]
    );
    // ⛔ O CONTROLO: a mesma folha com tinta SÓ no quadro 0 cobre só metade — é o que prova que a
    // asserção acima mede a união e não a largura da célula.
    let so_um = malha_do_bind(
        &folha_meia_celula(2, cw, ch, 0, true),
        [2 * cw, ch],
        ph2d_ecs::SpriteGrid {
            hframes: 2,
            vframes: 1,
            frame: 0,
        },
    );
    let b1 = caixa_rest(&so_um.mesh);
    assert!(
        b1[2] < f64::from(cw) * 0.7,
        "o controlo cobriu ate' {:.1} de {cw} px — a regua acima nao esta' a medir a uniao",
        b1[2]
    );
}

/// ⭐⭐ **COM UMA CÉLULA SÓ, A TINTA É A IMAGEM — byte a byte.**
///
/// ⚠️ É isto que mantém toda sprite normal exactamente como estava: a porta nova entra no caminho de
/// TODA imagem presa, e a identidade não é uma promessa, é uma comparação.
#[test]
fn com_uma_celula_a_tinta_e_a_imagem_ao_bit() {
    let arte = folha(1, 40, 20);
    let cells = ph2d_render::SourceCells::of([40, 20], None, 1, 1).expect("lado");
    let uniao = crate::skin_image::cell_alpha(&arte, [40, 20], &cells);
    let cru: Vec<u8> = arte.iter().skip(3).step_by(4).copied().collect();
    assert_eq!(
        uniao, cru,
        "a uniao de UMA celula tem de ser a propria alfa"
    );
}

/// As instâncias de um 9-slice desta sprite, com a fracção de cada pedaço — o que o extract emite.
fn pedacos(
    s: &Sprite,
    base: RenderInstance,
    bordas: [f32; 4],
    alvo: [f32; 2],
) -> Vec<(RenderInstance, ph2d_render::nine_slice::SlicePatchSource)> {
    let slice = ph2d_ecs::SliceNine {
        draw_mode: ph2d_ecs::SliceDrawMode::Sliced,
        borders: bordas,
        ..ph2d_ecs::SliceNine::INERT
    };
    let celula = base.atlas_uv;
    let (du, dv) = (celula[2] - celula[0], celula[3] - celula[1]);
    ph2d_render::nine_slice::nine_slice_patches(celula, [40.0, 20.0], &slice, alvo, PPM, [1.0, 1.0])
        .iter()
        .filter_map(|p| {
            let p = p.as_ref()?;
            let mut ri = base;
            ri.atlas_uv = p.uv;
            ri.size = p.size;
            ri.anchor = [
                base.anchor[0] + p.center_offset[0],
                base.anchor[1] + p.center_offset[1],
            ];
            let _ = s;
            Some((
                ri,
                ph2d_render::nine_slice::SlicePatchSource {
                    frac: [
                        (p.uv[0] - celula[0]) / du,
                        (p.uv[1] - celula[1]) / dv,
                        (p.uv[2] - celula[0]) / du,
                        (p.uv[3] - celula[1]) / dv,
                    ],
                },
            ))
        })
        .collect()
}

/// Espalha o pedaço `k` como o extract o espalha: o `[0]` vai para o espelho principal e os outros
/// **oito levam `SlicePatchMirror`**.
///
/// ⚠️⚠️ **A 1.ª redacção da fixtura não punha a marca, e por isso ela NÃO continha o fenómeno:** a
/// mutação que tira o pedaço da lista de alvos sobrevivia, porque sem a marca ele entrava pela porta
/// da instância BASE. *Uma fixtura que não reproduz o que o produto emite mede outro programa.*
fn espalha(
    w: &mut ph2d_ecs::World,
    e: Entity,
    ri: RenderInstance,
    f: ph2d_render::nine_slice::SlicePatchSource,
    k: usize,
) -> Entity {
    if k == 0 {
        w.spawn((SimRef(e), ri, f)).id()
    } else {
        w.spawn((SimRef(e), ri, f, SlicePatchMirror)).id()
    }
}

/// Monta a cena em REPOUSO (sem dobra) com os pedaços dados e devolve as malhas desenhadas.
fn malhas_em_repouso(tamanho: [f32; 2], bordas: [f32; 4], nove: bool) -> Vec<SpriteMesh> {
    let mut sim = SimWorld::default();
    let raiz = crate::bone::create(&mut sim, None, [-2.0, 0.0], [2.0, 0.0]).expect("osso");
    let s = sprite(tamanho[0], tamanho[1], 0.0, 0.0);
    let e = sim.world_mut().spawn((Transform::IDENTITY, s)).id();
    assert!(crate::skin_live::bind_image(
        &mut sim,
        e,
        &folha(1, 40, 20),
        [40, 20],
        PPM,
        ph2d_poly2d::GridOptions::default(),
        Some(Entity::from_bits(raiz)),
    ));
    let mut present = PresentWorld::new();
    let base = instancia_de(&s);
    let ps: Vec<Entity> = if nove {
        pedacos(&s, base, bordas, tamanho)
            .into_iter()
            .enumerate()
            .map(|(k, (ri, f))| espalha(present.world_mut(), e, ri, f, k))
            .collect()
    } else {
        vec![present.world_mut().spawn((SimRef(e), base)).id()]
    };
    attach_skin_meshes(&sim, &mut present, PPM, &[]);
    ps.iter()
        .filter_map(|p| present.world().get::<SpriteMesh>(*p).cloned())
        .collect()
}

/// A área de uma malha desenhada, em metros locais.
fn area_local(m: &SpriteMesh) -> f64 {
    m.tris
        .iter()
        .map(|t| {
            let [a, b, c] = t.map(|i| m.local[i as usize]);
            f64::from((b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1])).abs() / 2.0
        })
        .sum()
}

/// ⭐⭐⭐ **UM 9-SLICE COM BORDAS A ZERO DESENHA COMO A SPRITE** — a identidade embutida no modelo.
///
/// ⚠️ Com bordas a zero o anel tem área nula e sobra **um** quad com a UV inteira; a fracção dele é
/// `[0,0,1,1]`, logo a malha passa pelo curto-circuito do corte e sai a do bind, intocada. *É esta
/// célula que prova que pôr o 9-slice no caminho não muda uma sprite que não o usa.*
#[test]
fn um_9slice_com_bordas_a_zero_desenha_como_a_sprite() {
    let nove = malhas_em_repouso([4.0, 2.0], [0.0; 4], true);
    let simples = malhas_em_repouso([4.0, 2.0], [0.0; 4], false);
    assert_eq!(nove.len(), 1, "bordas a zero tinham de dar UM quad");
    assert_eq!(simples.len(), 1);
    assert_eq!(nove[0].tris, simples[0].tris);
    assert_eq!(nove[0].local, simples[0].local);
    assert_eq!(nove[0].uv, simples[0].uv);
}

/// ⭐⭐⭐ **OS NOVE PEDAÇOS COBREM A ARTE, SEM BURACOS E SEM SOBREPOSIÇÃO.**
///
/// ⚠️ No tamanho INTRÍNSECO o 9-slice é a identidade geométrica (nada estica), logo a soma das áreas
/// dos pedaços tem de dar a área da malha inteira. **A régua apanha os dois defeitos opostos de uma
/// vez:** um corte que perde um pedaço soma a menos, e um mapa errado que repete arte soma a mais.
#[test]
fn os_nove_pedacos_somam_a_arte_inteira() {
    // ⚠️⚠️ **O TAMANHO INTRÍNSECO é `pixels / ppm`, e a 1.ª redacção deste gate usava `4×2 m`** —
    // que com `40×20 px` a `100 px/m` é o 9-slice a esticar **10×**. Ali as áreas NÃO têm de somar,
    // e o gate reprovava sobre produto correcto (`5,76` contra `5,10 m²`). *Uma régua de identidade
    // tem de correr no ponto em que a lei É a identidade.*
    let intrinseco = [40.0 / PPM, 20.0 / PPM];
    let nove = malhas_em_repouso(intrinseco, [8.0, 4.0, 8.0, 4.0], true);
    let simples = malhas_em_repouso(intrinseco, [8.0, 4.0, 8.0, 4.0], false);
    assert!(nove.len() >= 4, "so' {} pedacos com malha", nove.len());
    let soma: f64 = nove.iter().map(area_local).sum();
    let inteira = area_local(&simples[0]);
    assert!(
        (soma - inteira).abs() < 1e-6 * inteira,
        "os pedacos somam {soma:.9} m2 contra {inteira:.9} da arte inteira"
    );
}

/// ⭐⭐⭐ **CADA PEDAÇO DESENHA DENTRO DO QUAD DELE** — a régua que prova que o mapa é o do PEDAÇO.
///
/// ⛔⛔ **Sem ela, um mapa errado passa em tudo o resto:** as áreas ainda somam, as UVs ainda vão de
/// `0` a `1`, e a arte sai com o miolo pintado por cima dos cantos. Aqui o alvo é `3×` mais largo que
/// o intrínseco — que é o caso em que o 9-slice de facto faz alguma coisa.
#[test]
fn cada_pedaco_desenha_dentro_do_quad_dele() {
    let s = sprite(4.0, 2.0, 0.0, 0.0);
    let base = instancia_de(&s);
    let bordas = [8.0, 4.0, 8.0, 4.0];
    let alvo = [12.0, 2.0];
    let quads: Vec<_> = pedacos(&s, base, bordas, alvo)
        .into_iter()
        .map(|(ri, _)| (ri.anchor, ri.size))
        .collect();
    let mut sim = SimWorld::default();
    let raiz = crate::bone::create(&mut sim, None, [-2.0, 0.0], [2.0, 0.0]).expect("osso");
    let e = sim.world_mut().spawn((Transform::IDENTITY, s)).id();
    assert!(crate::skin_live::bind_image(
        &mut sim,
        e,
        &folha(1, 40, 20),
        [40, 20],
        PPM,
        ph2d_poly2d::GridOptions::default(),
        Some(Entity::from_bits(raiz)),
    ));
    let mut present = PresentWorld::new();
    let ps: Vec<Entity> = pedacos(&s, base, bordas, alvo)
        .into_iter()
        .enumerate()
        .map(|(k, (ri, f))| espalha(present.world_mut(), e, ri, f, k))
        .collect();
    attach_skin_meshes(&sim, &mut present, PPM, &[]);
    let mut vistos = 0;
    for (k, p) in ps.iter().enumerate() {
        let Some(m) = present.world().get::<SpriteMesh>(*p) else {
            continue;
        };
        vistos += 1;
        let (a, sz) = quads[k];
        let (lo, hi) = (
            [a[0] - sz[0] / 2.0, a[1] - sz[1] / 2.0],
            [a[0] + sz[0] / 2.0, a[1] + sz[1] / 2.0],
        );
        for v in &m.local {
            assert!(
                v[0] >= lo[0] - 1e-4
                    && v[0] <= hi[0] + 1e-4
                    && v[1] >= lo[1] - 1e-4
                    && v[1] <= hi[1] + 1e-4,
                "o pedaco {k} desenhou em {v:?}, fora do quad dele [{lo:?}..{hi:?}] — o mapa nao \
                 e' o do PEDACO, e a arte sai com o miolo por cima dos cantos"
            );
        }
    }
    assert!(
        vistos >= 4,
        "so' {vistos} pedacos desenharam: a regua mede pouco"
    );
}

/// ⭐⭐⭐ **CADA PEDAÇO DESENHA A TINTA DELE — e onde não há tinta ele não desenha nada.**
///
/// ⛔⛔ **É a régua que apanha o defeito que a ÁREA não vê.** Sem o corte, cada pedaço recebe a arte
/// INTEIRA encolhida no quad dele — e a soma das áreas fica **exactamente igual**, porque os nove
/// quads ladrilham o da sprite (`Σ inteira × área_k/área_total = inteira`). *Uma régua invariante ao
/// defeito fica verde sobre ele.*
///
/// A fixtura tem tinta só na METADE DE CIMA da arte ⇒ a fileira de baixo não pode desenhar nada.
#[test]
fn a_fileira_sem_tinta_nao_desenha() {
    let intrinseco = [40.0 / PPM, 20.0 / PPM];
    let s = sprite(intrinseco[0], intrinseco[1], 0.0, 0.0);
    let base = instancia_de(&s);
    let bordas = [8.0, 4.0, 8.0, 4.0];
    let mut sim = SimWorld::default();
    let raiz = crate::bone::create(&mut sim, None, [-2.0, 0.0], [2.0, 0.0]).expect("osso");
    let e = sim.world_mut().spawn((Transform::IDENTITY, s)).id();
    assert!(crate::skin_live::bind_image(
        &mut sim,
        e,
        &folha(1, 40, 20),
        [40, 20],
        PPM,
        ph2d_poly2d::GridOptions::default(),
        Some(Entity::from_bits(raiz)),
    ));
    let mut present = PresentWorld::new();
    let pedacos_e_quads = pedacos(&s, base, bordas, intrinseco);
    let ps: Vec<Entity> = pedacos_e_quads
        .iter()
        .enumerate()
        .map(|(k, (ri, f))| espalha(present.world_mut(), e, *ri, *f, k))
        .collect();
    attach_skin_meshes(&sim, &mut present, PPM, &[]);
    let (mut baixo, mut cima) = (0, 0);
    for (p, (_, f)) in ps.iter().zip(&pedacos_e_quads) {
        let tem = present.world().get::<SpriteMesh>(*p).is_some();
        // A fileira de baixo é a que começa depois da metade da arte.
        if f.frac[1] >= 0.5 {
            baixo += usize::from(tem);
        } else {
            cima += usize::from(tem);
        }
    }
    assert_eq!(
        baixo, 0,
        "{baixo} pedaco(s) da fileira de BAIXO desenharam, e ali nao ha' tinta — cada pedaco tem \
         de receber a arte DELE, e nao a arte inteira encolhida no quad dele"
    );
    assert!(
        cima >= 2,
        "so' {cima} pedacos de cima desenharam: a regua mede pouco"
    );
}
