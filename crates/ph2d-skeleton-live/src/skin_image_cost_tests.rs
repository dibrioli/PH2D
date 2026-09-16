//! ⏱️ **O QUE A PELE DE IMAGEM CUSTA À CPU NUM QUADRO** — irmão do [`super`] pelo tecto de LOC,
//! cortado por RESPONSABILIDADE: *a 2.ª mídia desenha o que deve* é uma pergunta, *quanto ela custa
//! por peça* é outra, e a segunda é uma sonda sem barra.
//!
//! ⚠️ Filho do arnês do irmão (`#[path]`) para herdar as fixturas dele — uma cópia divergiria no
//! primeiro ajuste.

use super::*;

/// Uma malha SINTÉTICA de `cols × rows` células (dois triângulos cada) sobre a imagem `size_px` — o
/// que a sonda de custo precisa para escolher o número de peças.
fn malha_grelha(size_px: [u32; 2], cols: u32, rows: u32) -> Mesh2d {
    let (w, h) = (f64::from(size_px[0]), f64::from(size_px[1]));
    let mut m = Mesh2d {
        rest: Vec::new(),
        tris: Vec::new(),
        size: size_px,
    };
    for j in 0..=rows {
        for i in 0..=cols {
            m.rest.push([
                w * f64::from(i) / f64::from(cols),
                h * f64::from(j) / f64::from(rows),
            ]);
        }
    }
    let id = |i: u32, j: u32| j * (cols + 1) + i;
    for j in 0..rows {
        for i in 0..cols {
            m.tris.push([id(i, j), id(i + 1, j), id(i + 1, j + 1)]);
            m.tris.push([id(i, j), id(i + 1, j + 1), id(i, j + 1)]);
        }
    }
    m
}

/// ⏱️ **SONDA (`--ignored`) — O QUE A PELE DE IMAGEM CUSTA À CPU NUM QUADRO** (plano 03, W4).
///
/// O `SKIN_FRAME_PIECES` foi derivado do buffer do Vello, que este caminho já não gasta. O recurso
/// que sobra do lado da CPU é o TEMPO do quadro, e é isto que ele mede, por número de peças:
/// descodificar a malha guardada (postcard, **por quadro**), deformar cada vértice, refinar
/// (`Smooth`) e montar o `SpriteMesh`.
///
/// ⚠️ **Acima de `load ~5` uma leitura de relógio desta workstation não vale nada** (`CLAUDE.md`
/// §5.0) — a carga é impressa ao lado. Corre com:
/// `cargo test -p ph2d-skeleton-live --lib -- --ignored --nocapture measure_the_cpu_cost`
#[test]
#[ignore = "sonda: imprime a tabela do custo, sem barra"]
fn measure_the_cpu_cost_of_a_skinned_frame() {
    use std::time::Instant;
    // ⛔⛔ **A zoom `1` esta arte mede `200 × 100` px de ecrã, e só a malha de `72` peças chegava a
    // partir** (medido 2026-09-16): as outras quatro linhas mediam *decidir não partir* com o nome de
    // «refinamento». A zoom `8` o refinamento trabalha em toda linha que cabe no orçamento, e a
    // coluna diz `NAO PARTIU` onde não trabalhou — *uma sonda cujo sujeito deixou de fazer a coisa
    // medida mede outra coisa com o mesmo nome.*
    const ZOOM: f64 = 8.0;

    let carga = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!(
        "carga: {}",
        carga
            .split_whitespace()
            .take(3)
            .collect::<Vec<_>>()
            .join(" ")
    );
    println!("ms: MINIMO/mediana de 40 corridas (a media sob contencao mede o vizinho)");
    println!(
        "{:>8} | {:>13} | {:>13} | {:>30} | {:>30}",
        "pecas", "descodificar", "Fast", "Smooth UNIFORME", "Smooth ADAPTATIVO"
    );

    for (cols, rows) in [(6_u32, 6_u32), (12, 12), (24, 24), (48, 48), (72, 72)] {
        let mut sim = SimWorld::default();
        let raiz = crate::bone::create(&mut sim, None, [-2.0, 0.0], [0.0, 0.0]).expect("raiz");
        let raiz = Entity::from_bits(raiz);
        let ponta =
            crate::bone::create(&mut sim, Some(raiz), [0.0, 0.0], [2.0, 0.0]).expect("ponta");
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
        // A malha guardada é substituída pela sintética: é ela que fixa o número de peças.
        //
        // ⚠️⚠️ **Ela leva uma TABELA DE PESOS, e a razão é que o produto mudou por baixo desta
        // sonda:** desde que a pele de imagem passou ao padrão-ouro o `source` guarda uma
        // [`SkinnedMesh`], e escrever aqui uma `Mesh2d` crua produzia bytes que o
        // `skinned_mesh_of` **recusa** — a sonda é `#[ignore]`, logo isso ficava verde a medir
        // ZERO peles. *Uma sonda que o CI nunca corre é o sítio onde um formato novo se esconde.*
        //
        // ⛔⛔⛔ **OS PESOS TÊM DE VARIAR NO ESPAÇO, e a 1.ª redacção desta sonda não o fazia.**
        // Ela escrevia a tabela **UNIFORME** (`1/n` em todo o vértice) com a justificação de que
        // era o pior caso do `blend` — e é, para o `Fast`. ⚠️ Mas com pesos iguais em todo o lado
        // a mistura das poses é a **MESMA** em todo o ponto, logo o campo é um AFIM — e um
        // triângulo desenhado com um afim reproduz um afim **exactamente**. ⇒ o desvio é zero,
        // **nenhuma** das duas leis refina, e a coluna do `Smooth` media o custo de *decidir não
        // fazer nada*. Medido: `saiu == pecas` nas cinco linhas.
        //
        // ⭐ A tabela agora é uma rampa em `x` (o osso da raiz manda à esquerda, a ponta à
        // direita), que é a forma de uma pele de verdade — e é ela que põe curvatura no campo.
        let malha = malha_grelha([40, 20], cols, rows);
        let pecas = malha.tris.len();
        let ossos_n = sim
            .world()
            .get::<ph2d_skeleton_ecs::SkinBind>(e)
            .expect("pele")
            .tendons
            .len();
        let largura = f64::from(malha.size[0]).max(f64::MIN_POSITIVE);
        let mut pesos = vec![0.0; malha.rest.len() * ossos_n];
        for (v, p) in malha.rest.iter().enumerate() {
            let t = (p[0] / largura).clamp(0.0, 1.0);
            for b in 0..ossos_n {
                // ⚠️ Partição da unidade por construção: a rampa entre os dois primeiros ossos, e
                // zero nos outros. *Uma tabela que não soma `1` mede outra lei.*
                pesos[v * ossos_n + b] = match b {
                    0 => 1.0 - t,
                    1 => t,
                    _ => 0.0,
                };
            }
        }
        let guardada = crate::skinned_mesh::SkinnedMesh { pesos, mesh: malha };
        let bytes = postcard::to_allocvec(&guardada).expect("serializa");
        {
            let mut skin = sim
                .world_mut()
                .get_mut::<ph2d_skeleton_ecs::SkinBind>(e)
                .expect("pele");
            skin.source = bytes;
        }
        sim.world_mut()
            .get_mut::<Transform>(Entity::from_bits(ponta))
            .expect("Transform da ponta")
            .rotation = 1.0;
        let s = sprite(4.0, 2.0, 0.0, 0.0);
        let mut present = PresentWorld::new();
        let p = present
            .world_mut()
            .spawn((SimRef(e), instancia_de(&s)))
            .id();

        const RONDAS: usize = 40;
        let mut descodif = Vec::with_capacity(RONDAS);
        for _ in 0..RONDAS {
            let t = Instant::now();
            let m = mesh_of(&sim, e).expect("malha");
            descodif.push(t.elapsed().as_secs_f64() * 1e3);
            assert_eq!(m.tris.len(), pecas);
        }
        // ⚠️ O `present` entra por ARGUMENTO (e não capturado): a sonda lê a malha que ficou
        // **entre** duas medições, e um empréstimo mutável preso no fecho proíbe isso.
        let medir = |present: &mut PresentWorld, modo: Option<RefineOptions>| -> Vec<f64> {
            let mut ms = Vec::with_capacity(RONDAS);
            for _ in 0..RONDAS {
                present.world_mut().entity_mut(p).remove::<SpriteMesh>();
                let t = Instant::now();
                attach_skin_meshes(&sim, present, PPM, modo, PX_POR_METRO * ZOOM, &[]);
                ms.push(t.elapsed().as_secs_f64() * 1e3);
            }
            ms
        };
        let descodif = melhor(descodif);
        let fast = melhor(medir(&mut present, None));
        let opcoes = |adaptativo: bool| RefineOptions {
            tolerance_px: 0.5,
            max_pieces: SKIN_FRAME_PIECES,
            adaptativo,
        };
        // ⚠️ **As DUAS leis, e o número que interessa é o µs POR PEÇA ENTREGUE** — é dele que o
        // `CUSTO_POR_PECA_NS` sai, e a lei que o produto corre é a adaptativa.
        let uniforme = melhor(medir(&mut present, Some(opcoes(false))));
        let saiu_u = present
            .world()
            .get::<SpriteMesh>(p)
            .map_or(0, |m| m.tris.len());
        let adaptativo = melhor(medir(&mut present, Some(opcoes(true))));
        let saiu_a = present
            .world()
            .get::<SpriteMesh>(p)
            .map_or(0, |m| m.tris.len());
        #[expect(clippy::cast_precision_loss, reason = "contagens de peças")]
        let por_peca =
            |ms: f64, n: usize| -> f64 { if n == 0 { 0.0 } else { ms * 1e3 / n as f64 } };
        let aviso = if saiu_a == pecas && pecas < SKIN_FRAME_PIECES {
            "  NAO PARTIU"
        } else {
            ""
        };
        println!(
            "{pecas:>8} | {:>6.3}/{:<6.3} | {:>6.3}/{:<6.3} | {:>6.3} ({saiu_u:>6}, {:>5.3} us/p) \
             | {:>6.3} ({saiu_a:>6}, {:>5.3} us/p){aviso}",
            descodif.0,
            descodif.1,
            fast.0,
            fast.1,
            uniforme.0,
            por_peca(uniforme.0, saiu_u),
            adaptativo.0,
            por_peca(adaptativo.0, saiu_a),
        );
    }
}

/// `(mínimo, mediana)` de amostras de relógio, em ms.
///
/// ⚠️⚠️ **O MÍNIMO é a leitura que sobrevive a esta workstation.** O `CLAUDE.md` §5.0 diz que acima
/// de `load ~5` nenhum relógio daqui vale nada — e a carga de FUNDO desta máquina (o editor, o
/// rust-analyzer, o sccache, as outras sessões) fica em `~7` sem ninguém compilar. O mínimo de `N`
/// corridas é o custo quando o escalonador deu o núcleo; a mediana ao lado diz quanto a máquina
/// estava a roubar. *Uma média sob contenção mede o vizinho.*
fn melhor(mut ms: Vec<f64>) -> (f64, f64) {
    ms.sort_by(|a, b| a.partial_cmp(b).expect("sem NaN no relogio"));
    (ms[0], ms[ms.len() / 2])
}

/// Três imagens pequenas presas a uma corrente de DOIS ossos, a ponta dobrada `1 rad` — o campo
/// curva, logo o `Smooth` tem o que partir quando o orçamento deixa.
fn tres_dobradas() -> (SimWorld, PresentWorld, Vec<Entity>) {
    let mut sim = SimWorld::default();
    let raiz = crate::bone::create(&mut sim, None, [-2.0, 0.0], [0.0, 0.0]).expect("raiz");
    let raiz = Entity::from_bits(raiz);
    let ponta = crate::bone::create(&mut sim, Some(raiz), [0.0, 0.0], [2.0, 0.0]).expect("ponta");
    let s = sprite(4.0, 2.0, 0.0, 0.0);
    let mut present = PresentWorld::new();
    let mut instancias = Vec::new();
    for _ in 0..3 {
        let e = sim.world_mut().spawn((Transform::IDENTITY, s)).id();
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
    (sim, present, instancias)
}

/// As malhas que o quadro pôs, e quantas vezes o refinamento correu para as pôr.
fn quadro(
    sim: &SimWorld,
    present: &mut PresentWorld,
    instancias: &[Entity],
    smooth: Option<RefineOptions>,
) -> (Vec<SpriteMesh>, usize) {
    for p in instancias {
        present.world_mut().entity_mut(*p).remove::<SpriteMesh>();
    }
    crate::skin_refine::REFINAMENTOS.with(|c| c.set(0));
    attach_skin_meshes(sim, present, PPM, smooth, PX_POR_METRO * 8.0, &[]);
    let corridas = crate::skin_refine::REFINAMENTOS.with(std::cell::Cell::get);
    let malhas = instancias
        .iter()
        .map(|p| {
            present
                .world_mut()
                .entity_mut(*p)
                .take::<SpriteMesh>()
                .expect("malha posta")
        })
        .collect();
    (malhas, corridas)
}

/// ⭐⭐⭐ **SEM ESPAÇO NO ORÇAMENTO, O `Smooth` NÃO PAGA O REFINAMENTO** (F6-t, 2026-09-16).
///
/// ⛔⛔ **Medido antes da cura** (`measure_the_smooth_under_a_full_scene`): com as malhas guardadas
/// acima do orçamento do quadro, cada imagem recebe um orçamento IGUAL ao que guarda — nada pode
/// partir, a saída é a do `Fast` —, e o quadro pagava a avaliação inteira: `8` imagens do smoke
/// custavam `5,9 ms` (`35 %` de um quadro) para entregar o que o `Fast` entrega em `0,21 ms`.
///
/// ⚠️ **As três metades:** sem espaço a lei NÃO corre · a saída é a do `Fast` AO BIT · e com espaço
/// ela corre e parte (o controlo — sem ele, um curto-circuito que desligasse o `Smooth` sempre
/// passaria as duas primeiras).
///
/// (Mutação: o curto-circuito apagado ⇒ RED na contagem.)
#[test]
fn without_room_in_the_budget_the_smooth_pays_nothing_and_draws_the_fast_mesh() {
    let (sim, mut present, instancias) = tres_dobradas();
    let (fast, corridas_fast) = quadro(&sim, &mut present, &instancias, None);
    assert_eq!(corridas_fast, 0);
    let guardadas: usize = fast.iter().map(|m| m.tris.len()).sum();
    let opcoes = |max_pieces| RefineOptions {
        tolerance_px: 0.5,
        max_pieces,
        adaptativo: true,
    };

    let (sem_espaco, corridas) =
        quadro(&sim, &mut present, &instancias, Some(opcoes(guardadas - 1)));
    assert_eq!(
        corridas, 0,
        "com as malhas guardadas acima do orcamento nada pode partir, e o Smooth pagou a lei na mesma"
    );
    assert_eq!(
        sem_espaco, fast,
        "sem espaco, o Smooth tem de desenhar o Fast AO BIT"
    );

    // ⛔ O CONTROLO: com espaço, a lei corre nas três e parte.
    let (com_espaco, corridas) = quadro(
        &sim,
        &mut present,
        &instancias,
        Some(opcoes(guardadas * 16)),
    );
    assert_eq!(corridas, 3, "com espaco a lei corre em cada imagem");
    let entregues: usize = com_espaco.iter().map(|m| m.tris.len()).sum();
    assert!(
        entregues > guardadas,
        "a fixtura nao dobra o bastante para o Smooth partir ({entregues} de {guardadas})"
    );
}

/// A cena CHEIA: `n` imagens do tamanho da cena do smoke (`512 × 320` px opacos, a malha de bind do
/// PRODUTO), presas à mesma corrente de três ossos dobrada `graus` por junta. Devolve o mundo, as
/// instâncias desenhadas e as peças que cada imagem guarda.
fn cena_cheia(n: usize, graus: f32) -> (SimWorld, PresentWorld, usize) {
    const LARG: u32 = 512;
    const ALT: u32 = 320;
    let largura = f64::from(LARG) / f64::from(PPM);
    let passo = largura / 3.0;
    let mut sim = SimWorld::default();
    let mut ossos = Vec::new();
    for k in 0..3 {
        let x0 = -largura / 2.0 + passo * f64::from(k);
        let pai = ossos.last().copied();
        let b = crate::bone::create(&mut sim, pai, [x0, 0.0], [x0 + passo, 0.0]).expect("osso");
        ossos.push(Entity::from_bits(b));
    }
    let s = sprite(LARG as f32 / PPM, ALT as f32 / PPM, 0.0, 0.0);
    let tinta = tinta(LARG, ALT, 0, 0);
    let mut present = PresentWorld::new();
    let mut guardadas = 0;
    for _ in 0..n {
        let e = sim.world_mut().spawn((Transform::IDENTITY, s)).id();
        assert!(crate::skin_live::bind_image(
            &mut sim,
            e,
            &tinta,
            [LARG, ALT],
            PPM,
            GridOptions::default(),
            ossos.first().copied(),
        ));
        guardadas = mesh_of(&sim, e).expect("malha").tris.len();
        present.world_mut().spawn((SimRef(e), instancia_de(&s)));
    }
    for osso in ossos.iter().skip(1) {
        sim.world_mut()
            .get_mut::<Transform>(*osso)
            .expect("Transform")
            .rotation += graus.to_radians();
    }
    (sim, present, guardadas)
}

/// ⏱️ **SONDA (`--ignored`) — O `Smooth` COM A CENA CHEIA** (fila do esqueleto, F6-t).
///
/// ⚠️ **A pergunta que ninguém tinha feito:** o orçamento do `Smooth` é do QUADRO inteiro
/// (`SKIN_FRAME_PIECES`) e é repartido pelas imagens na proporção das peças que cada uma GUARDA —
/// logo, quando a soma das malhas guardadas passa o orçamento, **nenhuma** imagem refina. Esta sonda
/// mede quantas imagens do tamanho do smoke cabem antes disso, e o que o quadro custa dos dois lados.
///
/// `cargo test -p ph2d-skeleton-live --lib -- --ignored --nocapture measure_the_smooth_under_a_full_scene`
#[test]
#[ignore = "sonda: imprime a tabela, sem barra"]
fn measure_the_smooth_under_a_full_scene() {
    use std::time::Instant;
    let carga = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!(
        "carga: {} | orcamento do quadro: {SKIN_FRAME_PIECES} pecas | ms: MINIMO/mediana de 30",
        carga
            .split_whitespace()
            .take(3)
            .collect::<Vec<_>>()
            .join(" ")
    );
    println!(
        "{:>3} | {:>4} | {:>5} | {:>9} | {:>13} | {:>13} | {:>9} | {:>7}",
        "img", "graus", "zoom", "guardadas", "Fast ms", "Smooth ms", "entregues", "us/peca"
    );
    for graus in [25.0_f32, 60.0] {
        for n in [1_usize, 2, 4, 8] {
            let (sim, mut present, por_imagem) = cena_cheia(n, graus);
            for zoom in [1.0_f64, 4.0, 8.0] {
                let px = PX_POR_METRO * zoom;
                const RONDAS: usize = 30;
                let medir = |present: &mut PresentWorld, modo: Option<RefineOptions>| {
                    let mut ms = Vec::with_capacity(RONDAS);
                    let mut entregues = 0;
                    for _ in 0..RONDAS {
                        let ids: Vec<Entity> = present
                            .world_mut()
                            .query_filtered::<Entity, With<SpriteMesh>>()
                            .iter(present.world())
                            .collect();
                        for p in ids {
                            present.world_mut().entity_mut(p).remove::<SpriteMesh>();
                        }
                        let t = Instant::now();
                        attach_skin_meshes(&sim, present, PPM, modo, px, &[]);
                        ms.push(t.elapsed().as_secs_f64() * 1e3);
                        entregues = present
                            .world_mut()
                            .query::<&SpriteMesh>()
                            .iter(present.world())
                            .map(|m| m.tris.len())
                            .sum::<usize>();
                    }
                    (melhor(ms), entregues)
                };
                let (fast, _) = medir(&mut present, None);
                let opcoes = RefineOptions {
                    tolerance_px: 0.5,
                    max_pieces: SKIN_FRAME_PIECES,
                    adaptativo: true,
                };
                let (suave, entregues) = medir(&mut present, Some(opcoes));
                #[expect(clippy::cast_precision_loss, reason = "contagens de pecas")]
                let us = suave.0 * 1e3 / entregues.max(1) as f64;
                println!(
                    "{n:>3} | {graus:>4} | {zoom:>5} | {:>9} | {:>6.3}/{:<6.3} | {:>6.3}/{:<6.3} | \
                     {entregues:>9} | {us:>7.3}",
                    n * por_imagem,
                    fast.0,
                    fast.1,
                    suave.0,
                    suave.1,
                );
            }
        }
    }
}
