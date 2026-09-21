//! ⭐⭐⭐ **A ATRIBUIÇÃO DO QUADRO — o que é NOSSO, o que é da PLACA, e sobre o quê os dois
//! trabalham.**
//!
//! ⛔⛔ Este ficheiro existe por uma pergunta do dono de 2026-09-21 (*«não aguenta arredondar
//! 90 000 mas aguenta 20 000. talvez isso não seja um problema. talvez seja o limite normal.
//! avalie»*) e pela aritmética que ela obrigou a fazer: o irmão
//! [`crate::motion_carimbo_relogio_probe`] media `3,5 ms` de CPU a `102 400` cópias com as quinas
//! redondas, e o quadro que o dono vê mede `19,53 ms` ⇒ **`~15 ms` não estavam atribuídos a
//! ninguém**. *Dizer «é o limite da máquina» com quatro quintos do quadro por explicar é
//! exactamente o que o `CLAUDE.md` §0.0 proíbe.*
//!
//! ⭐ O corte em relação ao irmão é por RESPONSABILIDADE e não por tamanho: lá medem-se as
//! **ROTAS** do encode (qual das duas é mais barata); aqui mede-se a **ATRIBUIÇÃO** — quanto do
//! quadro é nosso, e sobre que população.
//!
//! ⚠️ As duas sondas daqui tomam a MESMA fatia do irmão (`super::fatia`) e imprimem a carga ao
//! lado do número, pela mesma lei: *nenhuma leitura de relógio desta workstation vale nada acima
//! de `load ~5`*.
//!
//! ## (1) A CPU inteira do carimbo — 2026-09-21, `--release`, `90 %` de CPU ociosa, `90 000` cópias
//!
//! | corner | vértices | cozer | desenho | resolver | **CPU** | % de um quadro | p/ a placa |
//! |---:|---:|---:|---:|---:|---:|---:|---:|
//! | `0,00` | `12` | `0,65 ms` | `1,80 ms` | `1,06 ms` | **`3,52 ms`** | `21 %` | `30,8 MB` |
//! | `0,50` | `30` | `0,61 ms` | `2,28 ms` | `1,65 ms` | **`4,54 ms`** | `27 %` | `44,3 MB` |
//!
//! ⭐⭐⭐ **E daqui sai a ATRIBUIÇÃO, que é a razão de este ficheiro existir.** O quadro que o dono
//! mede na app é `9,99 ms` a corner `0` e `19,53 ms` a corner `0,5` ⇒
//!
//! | | corner `0` | corner `0,5` | cresceu |
//! |---|---:|---:|---:|
//! | o quadro que o dono vê | `9,99 ms` | `19,53 ms` | `1,96×` |
//! | a nossa CPU (esta tabela) | `3,52 ms` | `4,54 ms` | `1,29×` |
//! | **o que sobra (a PLACA)** | **`6,5 ms`** | **`15,0 ms`** | **`2,3×`** |
//! | segmentos da forma | `12` | `30` | `2,5×` |
//!
//! ⇒ **o que sobra segue os SEGMENTOS quase ao número, e a nossa CPU não.** Os painéis e o resto
//! do chrome não crescem com os segmentos de uma estrela ⇒ o que cresce é a RASTERIZAÇÃO. *A cura
//! do carimbo (2026-09-20) caiu numa metade que hoje vale um quarto do quadro; o tecto está na
//! outra.*
//!
//! ## (2) E sobre O QUÊ os dois lados trabalham — a mesma corrida, `90 000` cópias, corner `0,5`
//!
//! | | todas (`90 000`) | à VISTA (`8 736`) |
//! |---|---:|---:|
//! | desenho + resolver | `3,79 ms` | `0,21 ms` |
//! | segmentos | `1 620 000` | `157 248` |
//! | bytes para a placa | `44,3 MB` | `4,3 MB` |
//!
//! ⇒ **o artista vê `9,7 %` das cópias e a conta é paga pelas `90 000`**, porque o passe vectorial
//! não tem recorte por câmara (campo `38,8 × 38,8` unidades, janela da câmara `21,8 × 6,8`).
//!
//! ⚠️⚠️ **Na cena `=126` isso é DELIBERADO** — é o que a faz medir o que ela diz medir, e o
//! cabeçalho dela escreve-o. ⛔ **Num projecto a sério é o tecto**, e é essa a resposta à pergunta
//! do dono: *o que trava não é a máquina no limite dela, é entregarmos-lhe dez vezes o trabalho que
//! alguém pode ver* (§0.0 — *nunca deixe o fallback definir o produto*).
//!
//! ⛔ **O que estas sondas NÃO medem é a metade maior: a PLACA.** A linha `o que sobra` acima é uma
//! SUBTRACÇÃO, não uma medição — ela é sólida porque a coluna `cresceu` a separa do resto do
//! chrome, mas o número dela por dentro (quanto é flatten, quanto é binning, quanto é fine raster)
//! pede o dispositivo, e no dia em que isto se mediu a placa tinha a app do dono e os gates de
//! outra linha em cima.

use super::{carga, fatia, melhor_quente, segmentos};

/// ⭐⭐⭐ **O QUADRO INTEIRO DO LADO DA CPU, COM O PEDAÇO QUE FALTAVA** — a sonda que a pergunta do
/// dono de 2026-09-21 obrigou a escrever (*«não aguenta arredondar 90 000 mas aguenta 20 000.
/// talvez seja o limite normal. avalie»*).
///
/// ⛔⛔ **A aritmética que a abriu:** a [`audit_the_corner_radius_cost`] mede `3,5 ms` de CPU a
/// `102 400` cópias com as quinas redondas, e o quadro que o dono vê mede `19,53 ms` a `90 000`
/// ⇒ **`~15 ms` não estavam atribuídos a ninguém**. Declarar um tecto de HARDWARE com `80 %` do quadro por
/// explicar é exactamente o que o `CLAUDE.md` §0.0 proíbe.
///
/// ⭐ A coluna nova é o **`resolver`** — o passo de CPU que o `vello::Renderer` corre DENTRO do
/// `render_to_texture`, e que nenhuma sonda desta casa media: ele percorre os fluxos da cena e
/// monta o `layout`, as rampas e o atlas que a placa vai ler. Ele entra pela porta nova
/// [`ph2d_vector::SceneResolver`], que existe precisamente porque o `Resolver` não é alcançável
/// fora da `ph2d-vector`.
///
/// ⚠️ **O que esta sonda NÃO mede continua a ser a PLACA**, e é de propósito que ela o diz na
/// última linha: o que sobrar depois de `cozer + desenho + resolver` é rasterização, e medi-la
/// pede o dispositivo.
#[test]
#[ignore = "sonda de medição — corra à mão, em RELEASE e com a máquina calma"]
fn audit_the_cpu_frame_with_resolve() {
    let _fatia = fatia();
    let (mut m, saida) = crate::motion_carimbo_probe::monta("grade + carimbo");
    let forma = m
        .doc
        .graph
        .nodes()
        .iter()
        .find(|n| n.type_name == "source.shape")
        .map(|n| n.id)
        .expect("a cadeia do carimbo tem uma `source.shape`");
    // ⚠️ **A POPULAÇÃO É A DA CENA `=126`** e não a do arnês (`320 × 320`). O relógio de um encode
    // não depende do VÃO, mas depende do NÚMERO — e reportar `102 400` quando o dono tem `90 000`
    // convida ao erro que a sonda irmã já pagou: *uma tabela na população errada compara-se com a
    // do dono como se fosse a mesma experiência*.
    let grade = m
        .doc
        .graph
        .nodes()
        .iter()
        .find(|n| n.type_name == "motion.grid")
        .map(|n| n.id)
        .expect("a cadeia do carimbo tem uma `motion.grid`");
    #[expect(clippy::cast_precision_loss, reason = "o lado da grelha da cena")]
    let lado = crate::motion_state::carimbo_demo::LADO_N as f32;
    m.doc.graph.set_param(grade, "rows", lado);
    m.doc.graph.set_param(grade, "cols", lado);

    let uv = [0.0, 0.0, 1.0, 1.0];
    let tam = [1.0, 1.0];
    eprintln!(
        "\n  ═══ O QUADRO DO LADO DA CPU, COM O `resolver` DENTRO (load {}) ═══\n",
        carga()
    );
    eprintln!(
        "   corner | vértices |   cozer | desenho | resolver |     CPU | % quadro |   p/ a placa"
    );
    eprintln!(
        "  --------|----------|---------|---------|----------|---------|----------|-------------"
    );
    for c in [0.0f64, 0.5] {
        #[expect(clippy::cast_possible_truncation, reason = "duas posições do slider")]
        m.doc
            .graph
            .set_param(forma, ph2d_node_motion_shape::param::CORNER, c as f32);
        let mut cozer = f64::INFINITY;
        for t in 1..4u64 {
            let ph = f64::from(u32::try_from(t).unwrap_or(0)) / 60.0;
            crate::motion_shape_gen::publish(&mut m, ph);
            m.pump.mark_dirty();
            let inicio = std::time::Instant::now();
            assert!(
                m.pump
                    .pump(&m.doc.graph, &m.registry, &[saida], t, ph, uv, tam),
                "o quadro tem de cozinhar"
            );
            cozer = cozer.min(inicio.elapsed().as_secs_f64() * 1e3);
        }
        let insts = &m.pump.vector_instances;
        let esperadas = (crate::motion_state::carimbo_demo::LADO_N as usize).pow(2);
        assert_eq!(
            insts.len(),
            esperadas,
            "controlo: a cadeia tem de trazer as {esperadas} linhas da CENA"
        );
        let store = &m.shape_store;
        let segs = insts
            .first()
            .and_then(|i| store.get(i.geometry_id))
            .map_or(0, segmentos);
        let encoda = |cena: &mut ph2d_vector::VectorScene| {
            let mut sem_arte = |_: u32, _: [f32; 4]| None;
            crate::motion_shape_gen::encode(
                insts,
                store,
                &mut sem_arte,
                ph2d_vector::Affine::IDENTITY,
                cena,
            );
        };
        let desenho = melhor_quente(3, encoda);

        // ⚠️ **Um resolvedor só, e a cena enchida UMA vez** — é o que o `Renderer` faz entre
        // quadros, e é o que mede o quadro em REGIME. *Um resolvedor por medição mediria o
        // nascimento do atlas, e uma cena nova por medição mediria o alocador.*
        let mut cena = ph2d_vector::VectorScene::new();
        encoda(&mut cena);
        let mut resolvedor = ph2d_vector::SceneResolver::new();
        let mut tamanho = resolvedor.resolve(&cena); // frio: cresce o buffer de saída
        let mut resolver = f64::INFINITY;
        for _ in 0..3 {
            let inicio = std::time::Instant::now();
            tamanho = resolvedor.resolve(&cena);
            resolver = resolver.min(inicio.elapsed().as_secs_f64() * 1e3);
        }
        assert_eq!(
            tamanho.draw_objects,
            insts.len(),
            "controlo: o resolvedor tem de ver um objecto de desenho por cópia"
        );
        #[expect(clippy::cast_precision_loss, reason = "um tamanho de buffer")]
        let mb = tamanho.scene_bytes as f64 / 1e6;
        let cpu = cozer + desenho + resolver;
        eprintln!(
            "  {c:>7.2} | {segs:>8} | {cozer:>5.2} ms | {desenho:>4.2} ms | {resolver:>5.2} ms | {cpu:>4.2} ms | {:>7.0}% | {mb:>8.1} MB",
            cpu / 16.67 * 100.0
        );
    }
    eprintln!(
        "\n  ⛔ O que NÃO está nesta tabela é a PLACA. O quadro que o dono mede (`19,53 ms` a\n  \
         corner `0,5`) menos a coluna `CPU` é rasterização — e é ela que decide se `20 000` é o\n  \
         tecto da MÁQUINA ou o tecto do caminho que escolhemos.\n"
    );
    eprintln!("  load no fim: {}\n", carga());
}

/// **O rectângulo que a câmara de arranque da cena `=126` mostra, em unidades de MUNDO.**
///
/// ⚠️ Ele é **citado** do cabeçalho do [`crate::motion_state_carimbo_demo`], que o mediu da foto da
/// cena `=124` (`55,5` píxeis por unidade), e nunca re-estimado aqui. *Um número medido numa foto e
/// re-derivado noutro ficheiro é a segunda resposta à mesma pergunta.*
const JANELA: [f64; 2] = [21.8, 6.8];

/// ⭐⭐⭐ **QUANTAS DAS ESTRELAS O ARTISTA CONSEGUE VER — e o que custa o resto.**
///
/// ⛔⛔ Nasceu da pergunta do dono de 2026-09-21 (*«talvez seja o limite normal. avalie»*), e é a
/// medição que a decide. A [`audit_the_cpu_frame_with_resolve`] mostra que a CPU vale `36 %` do
/// quadro dele e que a placa faz o resto; esta responde **sobre o quê** os dois lados trabalham.
///
/// ⚠️⚠️ **O `grep` não serve aqui.** Uma varredura por `cull`/`viewport` no passe vectorial devolve
/// ZERO, e *uma ausência afirmada pelo endereço do código é um palpite com cara de medição*
/// (`CLAUDE.md` §5). ⇒ esta sonda **CONTA**: ela mede a caixa que as instâncias de facto ocupam,
/// conta quantas caem na janela da câmara, e cronometra o mesmo encode sobre as duas populações.
///
/// ⭐ A coluna que decide é a última: se encodar **só as visíveis** custar uma fracção do que custa
/// encodar todas, então o tecto não é da MÁQUINA — é de estarmos a entregar-lhe trabalho que
/// ninguém pode ver (§0.0: *nunca deixe o fallback definir o produto*).
///
/// ⛔ **O que ela NÃO mede é a metade maior:** a placa. Cortar a população corta o que ela recebe
/// na mesma proporção de OBJECTOS, mas quanto isso vale em milissegundos de rasterização é outra
/// medição, e ela pede o dispositivo.
#[test]
#[ignore = "sonda de medição — corra à mão, em RELEASE e com a máquina calma"]
fn audit_what_the_camera_can_see() {
    let _fatia = fatia();
    let (mut m, saida) = crate::motion_carimbo_probe::monta("grade + carimbo");
    let forma = m
        .doc
        .graph
        .nodes()
        .iter()
        .find(|n| n.type_name == "source.shape")
        .map(|n| n.id)
        .expect("a cadeia do carimbo tem uma `source.shape`");
    m.doc
        .graph
        .set_param(forma, ph2d_node_motion_shape::param::CORNER, 0.5);

    // ⛔⛔ **A GEOMETRIA É A DA CENA, e não a do arnês** — e isto foi um defeito MEDIDO desta
    // sonda: o `monta` põe `320 × 320` posições com o vão de FÁBRICA (`1,0`), logo o campo dele
    // mede `319 × 319` unidades e a janela da câmara apanhava `0,1 %` das cópias. A cena `=126`
    // põe [`LADO_N`] posições com o vão [`VAO`] (a pegada da estrela mais `20 %` de ar), e o campo
    // dela mede `~39`. *A pergunta «quantas é que o artista vê?» é sobre a CENA — medi-la no arnês
    // responde sobre outro programa.*
    let grade = m
        .doc
        .graph
        .nodes()
        .iter()
        .find(|n| n.type_name == "motion.grid")
        .map(|n| n.id)
        .expect("a cadeia do carimbo tem uma `motion.grid`");
    #[expect(clippy::cast_precision_loss, reason = "o lado da grelha da cena")]
    let lado = crate::motion_state::carimbo_demo::LADO_N as f32;
    m.doc.graph.set_param(grade, "rows", lado);
    m.doc.graph.set_param(grade, "cols", lado);
    m.doc
        .graph
        .set_param(grade, "gap_x", crate::motion_state::carimbo_demo::VAO);
    m.doc
        .graph
        .set_param(grade, "gap_y", crate::motion_state::carimbo_demo::VAO);

    let uv = [0.0, 0.0, 1.0, 1.0];
    let tam = [1.0, 1.0];
    crate::motion_shape_gen::publish(&mut m, 0.0);
    m.pump.mark_dirty();
    assert!(
        m.pump
            .pump(&m.doc.graph, &m.registry, &[saida], 0, 0.0, uv, tam),
        "o quadro tem de cozinhar"
    );
    let insts = m.pump.vector_instances.clone();
    let esperadas = (crate::motion_state::carimbo_demo::LADO_N as usize).pow(2);
    assert_eq!(
        insts.len(),
        esperadas,
        "controlo: a cadeia tem de trazer as {esperadas} linhas da CENA"
    );

    // A caixa que as instâncias OCUPAM — medida delas, nunca inferida do `gap` vezes o número.
    let (mut x0, mut x1, mut y0, mut y1) = (f64::MAX, f64::MIN, f64::MAX, f64::MIN);
    for i in &insts {
        let (x, y) = (f64::from(i.world_pos[0]), f64::from(i.world_pos[1]));
        x0 = x0.min(x);
        x1 = x1.max(x);
        y0 = y0.min(y);
        y1 = y1.max(y);
    }
    // A janela é CENTRADA no campo — é onde a câmara de arranque aponta.
    let (cx, cy) = ((x0 + x1) * 0.5, (y0 + y1) * 0.5);
    let (hw, hh) = (JANELA[0] * 0.5, JANELA[1] * 0.5);
    let visiveis: Vec<_> = insts
        .iter()
        .filter(|i| {
            let (x, y) = (f64::from(i.world_pos[0]), f64::from(i.world_pos[1]));
            (x - cx).abs() <= hw && (y - cy).abs() <= hh
        })
        .cloned()
        .collect();
    assert!(
        (x1 - x0) > 30.0 && (x1 - x0) < 50.0,
        "controlo: o campo da CENA mede ~39 unidades de lado, e mediu {:.1}",
        x1 - x0
    );
    assert!(
        !visiveis.is_empty() && visiveis.len() < insts.len(),
        "controlo: a janela tem de apanhar ALGUMAS e não TODAS — apanhou {} de {}",
        visiveis.len(),
        insts.len()
    );

    let store = &m.shape_store;
    let mede = |quais: &[ph2d_eval_motion::VectorInstance]| {
        let encoda = |cena: &mut ph2d_vector::VectorScene| {
            let mut sem_arte = |_: u32, _: [f32; 4]| None;
            crate::motion_shape_gen::encode(
                quais,
                store,
                &mut sem_arte,
                ph2d_vector::Affine::IDENTITY,
                cena,
            );
        };
        let desenho = melhor_quente(3, encoda);
        let mut cena = ph2d_vector::VectorScene::new();
        encoda(&mut cena);
        let mut r = ph2d_vector::SceneResolver::new();
        let mut tamanho = r.resolve(&cena);
        let mut resolver = f64::INFINITY;
        for _ in 0..3 {
            let inicio = std::time::Instant::now();
            tamanho = r.resolve(&cena);
            resolver = resolver.min(inicio.elapsed().as_secs_f64() * 1e3);
        }
        (desenho, resolver, tamanho)
    };

    let (d_todas, r_todas, t_todas) = mede(&insts);
    let (d_vis, r_vis, t_vis) = mede(&visiveis);
    #[expect(clippy::cast_precision_loss, reason = "tamanhos de buffer")]
    let (mb_todas, mb_vis) = (
        t_todas.scene_bytes as f64 / 1e6,
        t_vis.scene_bytes as f64 / 1e6,
    );

    #[expect(clippy::cast_precision_loss, reason = "contagens de cena")]
    let fraccao = visiveis.len() as f64 / insts.len() as f64 * 100.0;
    eprintln!(
        "\n  ═══ O QUE A CÂMARA VÊ, E O QUE PAGAMOS PELO RESTO (load {}) ═══\n",
        carga()
    );
    eprintln!(
        "  campo ocupado: {:.1} × {:.1} unidades   ·   janela da câmara: {:.1} × {:.1}",
        x1 - x0,
        y1 - y0,
        JANELA[0],
        JANELA[1]
    );
    eprintln!("\n   população |  cópias |  desenho | resolver |   soma | segmentos | p/ a placa");
    eprintln!("  -----------|---------|----------|----------|--------|-----------|-----------");
    eprintln!(
        "  TODAS      | {:>7} | {d_todas:>5.2} ms | {r_todas:>5.2} ms | {:>3.2} ms | {:>9} | {:>6.1} MB",
        insts.len(),
        d_todas + r_todas,
        t_todas.path_segments,
        mb_todas
    );
    eprintln!(
        "  à VISTA    | {:>7} | {d_vis:>5.2} ms | {r_vis:>5.2} ms | {:>3.2} ms | {:>9} | {:>6.1} MB",
        visiveis.len(),
        d_vis + r_vis,
        t_vis.path_segments,
        mb_vis
    );
    eprintln!(
        "\n  ⇒ o artista vê {fraccao:.1}% das cópias, e a conta é paga pelas {}.\n  \
         O encode e a resolução das INVISÍVEIS custam {:.2} ms de CPU por quadro.\n",
        insts.len(),
        (d_todas + r_todas) - (d_vis + r_vis)
    );
    eprintln!("  load no fim: {}\n", carga());
}
