//! ⭐⭐⭐ **O RELÓGIO DO CARIMBO** — o terceiro irmão do [`crate::motion_carimbo_probe`], e o corte
//! é por RESPONSABILIDADE: a rota e a população são **decisões** (imunes à carga da máquina), e o
//! que vive aqui são **relógios**.
//!
//! ⚠️ É a distinção que o `CLAUDE.md` §5.0 cobra — *nenhuma leitura de relógio desta workstation
//! vale nada acima de `load ~5`* —, e é por isso que toda sonda deste ficheiro imprime a carga ao
//! lado do número. Separá-las em ficheiros diferentes é o que impede alguém de ler uma tabela
//! desta e uma tabela de lá como se as duas tivessem o mesmo valor probatório.
//!
//! ⛔⛔ **E TODA sonda daqui mede o quadro QUENTE, porque é esse o programa que corre.** A shell
//! **reaproveita** a cena vectorial de um quadro para o seguinte (`fase_frame_open.rs`:
//! `vector_scene.reset()`), e o `reset` do Vello limpa os fluxos **mantendo a capacidade** — logo
//! um quadro em regime **não paga o crescimento dos buffers**. Uma sonda que construa uma
//! `VectorScene::new()` por medição paga-o em toda leitura, e mede *outro programa*: é a mesma
//! família do sucedâneo que a `line/components` registou (*«uma sonda que mede um sucedâneo para
//! sempre mede outro programa»*), e ela mordeu aqui — a escada de 2026-09-20 saiu **fria**.
//!
//! À mão, em RELEASE e com a máquina calma:
//!
//! ```text
//! cargo test -p ph2d-app-motion --lib --release -- --ignored --nocapture audit_the_stamp_encode
//! ```

/// ⛔⛔ **A FATIA — as sondas deste ficheiro NÃO podem correr ao mesmo tempo.**
///
/// O `libtest` corre os testes de um binário em PARALELO por omissão, e três relógios no mesmo
/// binário medem-se **uns aos outros**: medido em 2026-09-20, a mesma célula (`102 400` cópias pelo
/// `fill`) leu `9,79`, `11,89` e `23,39 ms` nas três sondas da MESMA corrida — um factor de `2,4×`
/// que não é da lei nenhuma.
///
/// ⚠️ **A cura é a fatia e não uma instrução no cabeçalho.** Pedir `--test-threads=1` a quem corre
/// é a mesma espécie de nota que o `CLAUDE.md` §2 mede a morrer: *uma ferramenta que nenhum passo
/// obrigatório invoca pelo nome não é usada*. Aqui o passo obrigatório é o próprio `lock`.
///
/// ⚠️ O `unwrap_or_else` é deliberado: uma sonda que entre em pânico envenena o mutex, e a irmã
/// seguinte deve **medir na mesma** — o veredito dela não depende da que morreu.
static FATIA: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Toma a fatia para esta sonda. Todo `fn` de relógio deste ficheiro começa por aqui.
fn fatia() -> std::sync::MutexGuard<'static, ()> {
    FATIA
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// O `loadavg` de 1 minuto — ao lado de todo relógio (`CLAUDE.md` §5.0).
///
/// ⚠️ **Ele MENTE a decair** (memória `project-memory/`, integração de 2026-09-20): depois de uma
/// compilação em 32 núcleos ele fica alto durante minutos com a máquina já parada. Quem duvidar de
/// uma leitura confirma a ociosidade real (`vmstat 1 3`) antes de a deitar fora.
fn carga() -> String {
    std::fs::read_to_string("/proc/loadavg")
        .unwrap_or_default()
        .split_whitespace()
        .next()
        .unwrap_or("?")
        .to_string()
}

/// A estrela de `pontas` pontas — a forma da escada, cozida pela porta do documento vectorial.
fn estrela(pontas: f64) -> ph2d_vec_scene::VecPath {
    ph2d_vec_scene::cook(
        ph2d_vec_scene::ShapeKind::Star,
        [-0.5, -0.5],
        [0.5, 0.5],
        &[pontas, 0.5],
    )
}

/// Quantos vértices o caminho COZIDO tem — **contados**, nunca inferidos do número de pontas.
fn segmentos(p: &ph2d_vec_scene::VecPath) -> usize {
    let c = p.cooked();
    (0..c.contour_count())
        .filter_map(|i| c.contour(i).map(|(v, _)| v.len()))
        .sum()
}

/// A pose de cada cópia — a MESMA em todas as rotas, para a única diferença ser o encode.
fn pose(i: usize) -> ph2d_vector::Affine {
    let x = f64::from(u32::try_from(i % 320).unwrap_or(0)) * 0.01;
    ph2d_vector::Affine::translate((x, 0.0))
}

/// **O melhor de `rep` medições do quadro QUENTE**, em milissegundos.
///
/// ⭐ A cena é construída **uma vez**, enchida **uma vez** (o quadro frio, que paga o crescimento
/// dos buffers) e só depois `reset`ada e cronometrada — que é exactamente o que
/// `fase_frame_open.rs` faz a cada quadro do produto. *Sem este aquecimento a sonda mede o
/// alocador, e o alocador não é o que o artista espera.*
fn melhor_quente(rep: u32, mut encoda: impl FnMut(&mut ph2d_vector::VectorScene)) -> f64 {
    let mut cena = ph2d_vector::VectorScene::new();
    encoda(&mut cena); // frio: cresce os buffers até ao tamanho do quadro
    let mut melhor = f64::INFINITY;
    for _ in 0..rep {
        cena.reset();
        let t = std::time::Instant::now();
        encoda(&mut cena);
        melhor = melhor.min(t.elapsed().as_secs_f64() * 1e3);
        std::hint::black_box(&cena);
    }
    melhor
}

/// O mesmo, mas **FRIO** — uma cena nova por medição. Fica porque é o contraste que prova que o
/// aquecimento não é cosmético, e porque é o que o 1.º quadro depois de um `Reset Layout` paga.
fn melhor_frio(rep: u32, mut encoda: impl FnMut(&mut ph2d_vector::VectorScene)) -> f64 {
    let mut melhor = f64::INFINITY;
    for _ in 0..rep {
        let mut cena = ph2d_vector::VectorScene::new();
        let t = std::time::Instant::now();
        encoda(&mut cena);
        melhor = melhor.min(t.elapsed().as_secs_f64() * 1e3);
        std::hint::black_box(&cena);
    }
    melhor
}

/// A rota de HOJE: `N` chamadas a `Scene::fill`, pela porta de lote do produto.
fn rota_fill(caminho: &ph2d_vec_scene::VecPath, n: usize, cena: &mut ph2d_vector::VectorScene) {
    ph2d_vec_render::draw_shared_instances(
        (0..n).map(|i| (1u32, pose(i), [1.0, 1.0, 1.0, 1.0])),
        |_| Some(caminho),
        cena,
    );
}

/// A rota CANDIDATA: encodar a forma **uma vez** num fragmento e `Scene::append` com a pose de cada
/// cópia — no `vello_encoding` isso é `extend_from_slice` dos cinco fluxos (um **memcpy**) mais
/// **um** produto de afins.
fn rota_append(caminho: &ph2d_vec_scene::VecPath, n: usize, cena: &mut ph2d_vector::VectorScene) {
    let mut frag = ph2d_vector::VectorScene::new();
    ph2d_vec_render::draw_shape_instance(
        caminho,
        ph2d_vector::Affine::IDENTITY,
        [1.0, 1.0, 1.0, 1.0],
        &mut frag,
    );
    for i in 0..n {
        cena.inner_mut().append(frag.inner(), Some(pose(i)));
    }
}

/// ⭐⭐ **O QUE CUSTA ENCODAR `N` CÓPIAS DA MESMA FORMA** — a escada que o ADR-0154 Fase 3 pede.
///
/// ⭐ O lote passa pela **porta do produto** ([`ph2d_vec_render::draw_shared_instances`]), que já
/// tessela cada geometria DISTINTA uma vez — logo o que esta escada mede é o que SOBRA depois
/// dessa economia: o encode de `N` preenchimentos no Vello.
///
/// ⚠️ **As duas colunas são o achado:** a FRIA é a que esta sonda imprimia sozinha em 2026-09-20, e
/// ela paga o crescimento dos buffers da cena; a QUENTE é o quadro que o artista de facto vê. Ver
/// o cabeçalho do módulo.
#[test]
#[ignore = "sonda de medição — corra à mão, em RELEASE e com a máquina calma"]
fn audit_the_stamp_encode_cost() {
    let _fatia = fatia();
    let caminho = estrela(5.0);
    eprintln!(
        "\n  ═══ O ENCODE DE `N` CÓPIAS DA MESMA ESTRELA ({} segmentos, load {}) ═══\n",
        segmentos(&caminho),
        carga()
    );
    eprintln!("     cópias |  FRIO (cena nova) |  QUENTE (o produto) |  % quadro |  por cópia");
    eprintln!("  ----------|-------------------|---------------------|-----------|-----------");
    for n in [1_000usize, 10_000, 102_400, 1_000_000] {
        let frio = melhor_frio(3, |c| rota_fill(&caminho, n, c));
        let quente = melhor_quente(3, |c| rota_fill(&caminho, n, c));
        #[expect(clippy::cast_precision_loss, reason = "uma contagem de cena")]
        let por = quente * 1e3 / n as f64;
        eprintln!(
            "  {n:>9} | {frio:>14.2} ms | {quente:>16.2} ms | {:>8.0}% | {por:>7.3} µs",
            quente / 16.67 * 100.0
        );
    }
    eprintln!("\n  load no fim: {}\n", carga());
}

/// ⭐⭐⭐ **DUAS ROTAS PARA O MESMO ENCODE** — a investigação que o dono ordenou em 2026-09-20
/// (*«manter a nitidez e ir procurar uma cura que não a custe»*), depois de RECUSAR assar a forma
/// numa imagem acima de um tecto.
///
/// ⚠️ **Isto mede o TECTO do prémio, não a cura.** Um fragmento carrega o `brush` ASSADO, logo esta
/// rota, tal como está aqui, só serve cópias da MESMA cor — a pergunta da tinta vem depois, e só
/// vale a pena fazê-la se esta coluna ganhar. *Medir o tecto antes de desenhar a cura é o que
/// impede uma wave inteira sobre um ganho que não existe.*
///
/// ⚠️ **E o que nenhuma das duas colunas mede é a PLACA.** O
/// [`crate::motion_custo_do_quadro_probe`] já escreve a lei: *se o encode for barato, o que sobra é
/// a placa, e o que a governa não é o número de formas — é quantos PIXEIS elas cobrem*.
#[test]
#[ignore = "sonda de medição — corra à mão, em RELEASE e com a máquina calma"]
fn audit_the_stamp_encode_routes() {
    let _fatia = fatia();
    let caminho = estrela(5.0);
    eprintln!(
        "\n  ═══ `fill` POR CÓPIA contra `append` DE UM FRAGMENTO — quadro QUENTE (load {}) ═══\n",
        carga()
    );
    eprintln!("     cópias |      fill |    append |  por cópia |  razão");
    eprintln!("  ----------|-----------|-----------|------------|-------");
    for n in [10_000usize, 102_400, 1_000_000] {
        let a = melhor_quente(3, |c| rota_fill(&caminho, n, c));
        let b = melhor_quente(3, |c| rota_append(&caminho, n, c));
        #[expect(clippy::cast_precision_loss, reason = "uma contagem de cena")]
        let por = b * 1e3 / n as f64;
        eprintln!(
            "  {n:>9} | {a:>6.2} ms | {b:>6.2} ms | {por:>7.3} µs | {:>5.2}x",
            a / b.max(f64::MIN_POSITIVE)
        );
    }
    eprintln!("\n  load no fim: {}\n", carga());
}

/// ⭐⭐⭐ **DE QUE É FEITO O CUSTO DE UMA CÓPIA** — a decomposição que decide se a rota do `append`
/// serve o produto **com a tinta por cópia**, ou só cópias da mesma cor.
///
/// A [`audit_the_stamp_encode_routes`] mede o tecto do prémio e deixa uma pergunta em aberto: um
/// fragmento carrega o `brush` ASSADO. A cura utilizável tem de ser *«copiar o CAMINHO e re-encodar
/// o PINCEL»* — e o que decide se ela vale é a partição do custo entre os dois.
///
/// ⭐ **A partição lê-se da INCLINAÇÃO, não de um cronómetro dentro do encoder:** varrendo o número
/// de PONTAS da estrela (logo o número de segmentos) com `N` fixo, a recta de cada rota dá
///
/// * a **inclinação** = o que custa cada segmento (percorrer o `BezPath` no `fill`; `memcpy` no
///   `append`);
/// * a **ordenada na origem** = o que custa uma cópia **sem caminho nenhum** — a transformação, o
///   estilo, o objecto de desenho e o pincel.
///
/// ⇒ a rota candidata custa aproximadamente `origem(fill) + inclinação(append)`. *Uma cura desenhada
/// sem esta partição seria escolhida pela elegância e não pela medição.*
#[test]
#[ignore = "sonda de medição — corra à mão, em RELEASE e com a máquina calma"]
fn audit_the_stamp_encode_split() {
    let _fatia = fatia();
    const N: usize = 102_400;
    eprintln!(
        "\n  ═══ O CUSTO DE UMA CÓPIA CONTRA O NÚMERO DE SEGMENTOS (N = {N}, QUENTE, load {}) ═══\n",
        carga()
    );
    eprintln!("   pontas | segmentos |      fill |    append |  fill µs/cópia | append µs/cópia");
    eprintln!("  --------|-----------|-----------|-----------|----------------|----------------");
    let mut amostras: Vec<(f64, f64, f64)> = Vec::new();
    for pontas in [3.0f64, 5.0, 10.0, 20.0] {
        let caminho = estrela(pontas);
        let segs = segmentos(&caminho);
        let a = melhor_quente(3, |c| rota_fill(&caminho, N, c));
        let b = melhor_quente(3, |c| rota_append(&caminho, N, c));
        #[expect(clippy::cast_precision_loss, reason = "uma contagem de cena")]
        let (pa, pb) = (a * 1e3 / N as f64, b * 1e3 / N as f64);
        #[expect(clippy::cast_precision_loss, reason = "uma contagem de segmentos")]
        amostras.push((segs as f64, pa, pb));
        eprintln!(
            "  {pontas:>7.0} | {segs:>9} | {a:>6.2} ms | {b:>6.2} ms | {pa:>13.4} | {pb:>15.4}"
        );
    }
    // Recta por mínimos quadrados sobre as quatro amostras — a partição que a sonda existe para dar.
    let recta = |f: &dyn Fn(&(f64, f64, f64)) -> f64| {
        #[expect(clippy::cast_precision_loss, reason = "quatro amostras")]
        let n = amostras.len() as f64;
        let sx: f64 = amostras.iter().map(|s| s.0).sum();
        let sy: f64 = amostras.iter().map(f).sum();
        let sxx: f64 = amostras.iter().map(|s| s.0 * s.0).sum();
        let sxy: f64 = amostras.iter().map(|s| s.0 * f(s)).sum();
        let m = (n * sxy - sx * sy) / (n * sxx - sx * sx);
        (m, (sy - m * sx) / n)
    };
    let (ma, oa) = recta(&|s| s.1);
    let (mb, ob) = recta(&|s| s.2);
    eprintln!(
        "\n  fill  : {oa:.4} µs/cópia + {ma:.5} µs/segmento\n  \
         append: {ob:.4} µs/cópia + {mb:.5} µs/segmento"
    );
    eprintln!(
        "  ⇒ copiar o CAMINHO e re-encodar o PINCEL custaria ≈ {oa:.4} µs/cópia + {mb:.5} µs/segmento"
    );
    eprintln!("\n  load no fim: {}\n", carga());
}
