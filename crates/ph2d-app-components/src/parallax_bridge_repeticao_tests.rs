//! Os gates da ponte da paralaxe, parte 2: a repetição (W2) e o confinamento (W3).
//!
//! ⚠️ **Módulo-FILHO do [`super`] por tecto de LOC** (o ficheiro-pai chegou a `1359` contra `700`),
//! e o corte é por WAVE, que é como ele já se lia. ⭐ Filho e não irmão de propósito: os ajudantes
//! (`cena`, `pose_em`, `com_camera`, as constantes `PARADO`/`SEM_LIMITE`) ficam UMA vez, no pai —
//! duas cópias de um arnês divergem, e um gate que mede o arnês errado fica verde a medir nada.

#![allow(clippy::wildcard_imports)]
use super::*;

// ══ A REPETIÇÃO INFINITA (plano 24, W2) ════════════════════════════════════════════════════════

fn cena_rep(k: [f32; 2], tile: [f32; 2], pose: Transform) -> (SimWorld, ph2d_ecs::Entity) {
    let mut sim = SimWorld::default();
    let e = sim
        .world_mut()
        .spawn((ScrollFactor { k }, ScrollRepeat { tile }, pose))
        .id();
    (sim, e)
}

/// ⭐⭐⭐ **A CORRECÇÃO É UM NÚMERO INTEIRO DE LADRILHOS** — a lei medida no alvo, e a única metade
/// dela que a sonda de facto gravou.
///
/// ⚠️ **É isto que faz a costura não poder abrir:** a imagem a seguir ao salto é a mesma. ⛔ Somar
/// um RESTO faria o erro de `f32` acumular, e ao décimo milésimo ladrilho a costura estaria aberta.
#[test]
fn a_correccao_e_um_numero_inteiro_de_ladrilhos() {
    let tile = 256.0_f32;
    let rep = ScrollRepeat { tile: [tile, tile] };
    let mut corrigiu = 0;
    for i in -400_i16..=400 {
        let d = f32::from(i) * 37.0; // ⚠️ um passo que NÃO divide o ladrilho, de propósito
        let e = rep.envolve([d, -d]);
        for (bruto, envolvido) in [(d, e[0]), (-d, e[1])] {
            let n = (bruto - envolvido) / tile;
            assert!(
                (n - n.round()).abs() < 1e-3,
                "a correccao de {bruto} nao e' um inteiro de ladrilhos: {n}"
            );
            if n.abs() > 0.5 {
                corrigiu += 1;
            }
        }
    }
    // O CONTROLO: a varredura de facto passou por ladrilhos — sem ele as asserções acima passariam
    // sobre uma lei que nunca corrige nada.
    assert!(
        corrigiu > 700,
        "so' {corrigiu} correccoes: a varredura nao sai do 1.o ladrilho"
    );
}

/// ⭐⭐⭐ **A FASE é a mesma ao DÉCIMO MILÉSIMO ladrilho** — o gate que o plano encomendou.
///
/// Com `k = 0,5` e um ladrilho de `256`, a câmera avança `512` por ladrilho. A varredura vai a
/// `10 240` unidades (**vinte** ladrilhos) e a pose tem de cair na mesma fase — ⚠️ **módulo o
/// ladrilho**, que é o que «a mesma fase» quer dizer: `+128` e `−128` são o MESMO ponto de uma
/// imagem que se repete a cada `256`, e uma régua que os comparasse como números crus acusaria a
/// lei certa.
///
/// # ⭐ E o tecto de `f32` está MEDIDO, e não é da lei
///
/// A fase mantém-se exacta até câmeras de `5 × 10¹⁰` para uma fase de zero; a que se perde primeiro
/// é a **coordenada da câmera** — a `5,12 × 10⁹` um `f32` já não consegue guardar um desvio de
/// `256` sobre ela, e o `+256` do meio-ladrilho evapora ANTES de chegar à lei. *O limite não é o
/// envolvimento; é representar onde a câmera está.*
#[test]
fn a_fase_e_a_mesma_ao_decimo_milesimo_ladrilho() {
    let tile = 256.0_f32;
    let autorada = pose_em(-3.0, 7.0);
    for fase_inicial in [0.0_f32, 256.0, 61.0] {
        let mut poses = Vec::new();
        for n in 0_i16..=20 {
            let (mut sim, e) = cena_rep([0.5, 1.0], [tile, 0.0], autorada);
            let mut drive = PreviewDrive::default();
            let centro = [fase_inicial + 512.0 * f32::from(n), 0.0];
            drive_parallax(&mut sim, Some((centro, SEM_LIMITE)), PARADO, &mut drive);
            poses.push(sim.world().get::<Transform>(e).expect("pose").translation.x);
        }
        for (n, p) in poses.iter().enumerate() {
            let delta = ph2d_ecs::envolve_eixo(p - poses[0], tile);
            assert!(
                delta.abs() < 1e-3,
                "fase {fase_inicial}: ao ladrilho {n} a pose e' {p} contra {} — a costura ABRIU \
                 (desvio de {delta} dentro do ladrilho)",
                poses[0]
            );
            // ⭐⭐⭐ **A METADE ABSOLUTA, e ela nasceu de uma MUTAÇÃO SOBREVIVENTE:** apagar a
            // repetição da ponte deixava a asserção de cima VERDE, porque a varredura anda
            // exactamente um ladrilho por passo e a diferença envolve para zero. *Uma régua que
            // compara a fase MÓDULO o ladrilho não distingue «a costura fechou» de «o fundo fugiu
            // um número inteiro de ladrilhos»* — e fugir é precisamente o que a repetição existe
            // para impedir. ⚠️ O controlo do fim mede outra CENA (sem o componente): ele prova que
            // a varredura mexe, nunca que é o componente que a segura.
            assert!(
                (p - autorada.translation.x).abs() <= tile / 2.0 + 1e-3,
                "fase {fase_inicial}: ao ladrilho {n} a pose FUGIU para {p} — o fundo deixou de \
                 ser infinito (autorada {}, meio ladrilho {})",
                autorada.translation.x,
                tile / 2.0
            );
        }
        // ⭐ O CONTROLO: **sem** a repetição a mesma varredura afasta-se `5 120` unidades. Sem ele
        // este gate passaria sobre uma paralaxe que nunca desloca nada.
        let (mut sim, e) = cena([0.5, 1.0], autorada);
        let mut drive = PreviewDrive::default();
        drive_parallax(
            &mut sim,
            Some(([fase_inicial + 512.0 * 20.0, 0.0], SEM_LIMITE)),
            PARADO,
            &mut drive,
        );
        let solto = sim.world().get::<Transform>(e).expect("pose").translation.x;
        assert!(
            (solto - poses[0]).abs() > 1000.0,
            "sem repeticao a pose devia ter fugido, e leu {solto} contra {}",
            poses[0]
        );
    }
}

/// ⛔ **A ausência e o `0` são a mesma coisa** — a convenção do alvo, e é ela que permite repetir
/// só em X, que é o caso de quase todo fundo.
#[test]
fn um_ladrilho_zero_nao_corrige_e_o_eixo_livre_desloca() {
    let autorada = pose_em(0.0, 0.0);
    let centro = [4000.0_f32, 4000.0];
    // Repete em X (ladrilho 256) e NÃO em Y.
    let (mut sim, e) = cena_rep([0.5, 0.5], [256.0, 0.0], autorada);
    let mut drive = PreviewDrive::default();
    drive_parallax(&mut sim, Some((centro, SEM_LIMITE)), PARADO, &mut drive);
    let t = *sim.world().get::<Transform>(e).expect("pose");
    assert!(
        t.translation.x.abs() <= 128.0 + 1e-3,
        "o eixo REPETIDO fugiu do ladrilho: x = {}",
        t.translation.x
    );
    assert!(
        (t.translation.y - 2000.0).abs() < 1e-2,
        "o eixo LIVRE deixou de deslocar: y = {} contra 2000",
        t.translation.y
    );
    // E o CONTROLO da própria convenção: sem o componente os dois eixos fogem.
    let (mut sim2, e2) = cena([0.5, 0.5], autorada);
    let mut drive2 = PreviewDrive::default();
    drive_parallax(&mut sim2, Some((centro, SEM_LIMITE)), PARADO, &mut drive2);
    let t2 = *sim2.world().get::<Transform>(e2).expect("pose");
    assert!(
        (t2.translation.x - 2000.0).abs() < 1e-2,
        "x = {}",
        t2.translation.x
    );
}

/// ⭐⭐ **A repetição não toca na POSE AUTORADA** — o artista continua a poder arrastar o fundo, e o
/// que ele arrasta é o documento.
///
/// ⚠️ É esta a razão de a lei envolver o DESLOCAMENTO e não a soma: envolver a soma envolveria
/// também o que ele autorou, e o fundo saltaria para a origem assim que ele o puxasse para além de
/// meio ladrilho.
#[test]
fn a_repeticao_nao_envolve_a_pose_autorada() {
    let tile = 256.0_f32;
    // ⚠️ A pose autorada está MUITO para lá de meio ladrilho, que é onde a lei errada morde.
    let autorada = pose_em(4000.0, 0.0);
    let (mut sim, e) = cena_rep([0.5, 1.0], [tile, 0.0], autorada);
    let mut drive = PreviewDrive::default();
    drive_parallax(&mut sim, Some(([0.0, 0.0], SEM_LIMITE)), PARADO, &mut drive);
    assert_eq!(
        *sim.world().get::<Transform>(e).expect("pose"),
        autorada,
        "com a camera na origem a repeticao mexeu na pose que o artista autorou"
    );
    // E com a câmera longe, o que se vê é a autorada mais uma fase — nunca a autorada envolvida.
    drive_parallax(
        &mut sim,
        Some(([10_000.0, 0.0], SEM_LIMITE)),
        PARADO,
        &mut drive,
    );
    let x = sim.world().get::<Transform>(e).expect("pose").translation.x;
    assert!(
        (x - 4000.0).abs() <= 128.0 + 1e-3,
        "a pose fugiu do ladrilho a` volta do AUTORADO: {x} contra 4000 ± 128"
    );
    let Some(Driven::ParallaxPose(memo)) = drive.authored(e.to_bits(), Driver::ParallaxPose) else {
        panic!("tinha de continuar a ser conduzido");
    };
    assert_eq!(memo, autorada, "o autorado foi envolvido");
}

// ══ O CONFINAMENTO (plano 24, W3) ══════════════════════════════════════════════════════════════

/// A cena da tabela medida no alvo: região `−600..600`, k `0,5`.
fn cena_lim(k: f32, min: f32, max: f32) -> (SimWorld, ph2d_ecs::Entity) {
    let mut sim = SimWorld::default();
    let e = sim
        .world_mut()
        .spawn((
            ScrollFactor { k: [k, 1.0] },
            ScrollLimits {
                min: [min, 0.0],
                max: [max, 0.0],
            },
            pose_em(0.0, 0.0),
        ))
        .id();
    (sim, e)
}

fn pose_com_vista(k: f32, min: f32, max: f32, cam: f32, meia: f32) -> f32 {
    let (mut sim, e) = cena_lim(k, min, max);
    let mut drive = PreviewDrive::default();
    drive_parallax(
        &mut sim,
        Some(([cam, 0.0], [meia, 0.0])),
        PARADO,
        &mut drive,
    );
    sim.world().get::<Transform>(e).expect("pose").translation.x
}

/// ⭐⭐⭐ **A CURVA COM OS DOIS JOELHOS, pedaço a pedaço** — a tabela medida no alvo, reproduzida.
///
/// ```text
/// cam.x  −1200 … −360   declive +1,0000     ← a camada CONGELA no ecrã
/// cam.x   −240 …  +240   declive +0,5000     ← a paralaxe autorada
/// cam.x   +360 … +1200   declive +1,0000     ← congela outra vez
/// ```
///
/// ⚠️⚠️ **O gate amostra os PEDAÇOS e nunca a média, e isso não é estilo:** duas leis foram
/// construídas e REFUTADAS antes desta porque eu media um declive MÉDIO sobre uma curva que tem
/// joelhos. *Um clamp não tem um declive; tem pedaços* — e a média de dois pedaços não é nenhum
/// deles. Uma régua média aqui lê `~0,8` e aprova qualquer lei que passe pelos extremos.
#[test]
fn a_curva_do_confinamento_tem_os_dois_joelhos() {
    let (k, min, max, meia) = (0.5_f32, -600.0_f32, 600.0_f32, 360.0_f32);
    let declive = |a: f32, b: f32| {
        (pose_com_vista(k, min, max, b, meia) - pose_com_vista(k, min, max, a, meia)) / (b - a)
    };
    for (a, b, esperado, nome) in [
        (-1200.0_f32, -360.0_f32, 1.0_f32, "congelada a` esquerda"),
        (-240.0, 240.0, 0.5, "a paralaxe autorada"),
        (360.0, 1200.0, 1.0, "congelada a` direita"),
    ] {
        let m = declive(a, b);
        assert!(
            (m - esperado).abs() < 1e-4,
            "{nome} ({a} .. {b}): declive {m} contra {esperado} — a tabela do alvo"
        );
    }
    // ⭐ E o CONTROLO que prova que a régua vê os pedaços: a MÉDIA da curva inteira não é nenhum
    // deles, e é ela que aprovaria a lei errada.
    let media = declive(-1200.0, 1200.0);
    assert!(
        (media - 0.5).abs() > 0.1 && (media - 1.0).abs() > 0.1,
        "a media da curva inteira ({media}) caiu em cima de um dos pedacos: este gate deixou de \\
         poder distinguir a lei da media dela"
    );
}

/// ⭐⭐⭐ **O JOELHO é `(região − ecrã)/2`** — o ponto em que a borda da VISTA alcança a da REGIÃO.
///
/// ⚠️ **É por isto que a meia-janela tem de atravessar** (e é a premissa da W1 que morreu): o joelho
/// move-se com o ZOOM, e uma lei que só visse o centro poria-o sempre no mesmo sítio.
#[test]
fn o_joelho_esta_onde_a_borda_da_vista_alcanca_a_regiao() {
    for (largura, ecra) in [(1200.0_f32, 720.0_f32), (1200.0, 200.0), (400.0, 100.0)] {
        let (min, max, meia) = (-largura / 2.0, largura / 2.0, ecra / 2.0);
        let joelho = (largura - ecra) / 2.0;
        // ⚠️ Mede-se DENTRO e FORA do joelho, com uma margem, porque o ponto exacto é a fronteira
        // dos dois pedaços e não pertence a nenhum.
        let m_dentro = (pose_com_vista(0.5, min, max, joelho - 1.0, meia)
            - pose_com_vista(0.5, min, max, joelho - 21.0, meia))
            / 20.0;
        let m_fora = (pose_com_vista(0.5, min, max, joelho + 21.0, meia)
            - pose_com_vista(0.5, min, max, joelho + 1.0, meia))
            / 20.0;
        assert!(
            (m_dentro - 0.5).abs() < 1e-3,
            "largura {largura} ecra {ecra}: a {} unidades o declive e' {m_dentro} e devia ser a \\
             paralaxe (0,5) — o joelho chegou CEDO",
            joelho - 11.0
        );
        assert!(
            (m_fora - 1.0).abs() < 1e-3,
            "largura {largura} ecra {ecra}: a {} unidades o declive e' {m_fora} e devia ser o \\
             congelamento (1,0) — o joelho chegou TARDE",
            joelho + 11.0
        );
    }
}

/// ⛔ **Sem limites a saída é BYTE-IDÊNTICA** — o segundo termo é `k · 0`, e é essa a forma que a
/// lei tem escrita. ⚠️ A forma equivalente `centro − k·confinado` diferiria por um ULP, e mudava o
/// que a W1 e a W2 já shipam.
#[test]
fn sem_limites_a_saida_e_byte_identica() {
    for centro in [[400.0_f32, -250.0], [0.1, 3.7], [-1e4, 1e4]] {
        for k in [[0.3_f32, 0.7], [0.0, 2.0], [-0.5, 1.0]] {
            let autorada = pose_em(-3.0, 7.0);
            let (mut a, ea) = cena(k, autorada);
            let mut da = PreviewDrive::default();
            drive_parallax(&mut a, Some((centro, [360.0, 360.0])), PARADO, &mut da);

            let mut b = SimWorld::default();
            // ⚠️ Uma região VAZIA (`max == min`) é a omissão — o componente presente e inerte.
            let eb = b
                .world_mut()
                .spawn((ScrollFactor { k }, ScrollLimits::default(), autorada))
                .id();
            let mut db = PreviewDrive::default();
            drive_parallax(&mut b, Some((centro, [360.0, 360.0])), PARADO, &mut db);

            assert_eq!(
                a.world().get::<Transform>(ea).expect("pose").translation,
                b.world().get::<Transform>(eb).expect("pose").translation,
                "com limites INERTES a pose mudou (centro {centro:?}, k {k:?}) — o segundo termo \\
                 deixou de ser `k · 0`"
            );
        }
    }
}

/// ⛔⛔ **Uma região mais ESTREITA que a vista fixa-a no CENTRO dela** — e não entra em pânico.
///
/// ⚠️ Sem esta metade o clamp seria `clamp(c, min+h, max−h)` com o limite de baixo ACIMA do de
/// cima, e o `f32::clamp` do Rust **entra em pânico** ali. *É o mesmo caso que a câmera do jogo já
/// pagou* (uma cerca mais estreita que a janela), com a mesma cura.
#[test]
fn uma_regiao_mais_estreita_que_a_vista_nao_entra_em_panico() {
    // Região de `100` com uma vista de `720`: ela cabe inteira, e o sítio honesto é o centro dela.
    let x = pose_com_vista(0.5, 150.0, 250.0, 5000.0, 360.0);
    // O centro da região é `200`; o confinado é `200` ⇒ `d = 5000·0,5 + 0,5·(5000 − 200) = 4900`.
    assert!((x - 4900.0).abs() < 1e-2, "x = {x} contra 4900");
    // E o CONTROLO: com a vista a caber na região o resultado é outro.
    let y = pose_com_vista(0.5, -5000.0, 5000.0, 0.0, 360.0);
    assert!(y.abs() < 1e-3, "y = {y}");
}

/// ⭐⭐ **A ordem DECLARADA: confinar e depois envolver.**
///
/// ⚠️ Os dois juntos são uma cena que se contradiz — *um fundo que repete não tem borda para
/// esconder* —, e o que este gate fixa é o que ela DÁ: a repetição vem por último, logo o
/// deslocamento final cai sempre dentro de meio ladrilho, mesmo com a camada congelada.
#[test]
fn com_limites_e_repeticao_a_repeticao_e_a_ultima() {
    let tile = 256.0_f32;
    let mut sim = SimWorld::default();
    let e = sim
        .world_mut()
        .spawn((
            ScrollFactor { k: [0.5, 1.0] },
            ScrollLimits {
                min: [-600.0, 0.0],
                max: [600.0, 0.0],
            },
            ScrollRepeat { tile: [tile, 0.0] },
            pose_em(0.0, 0.0),
        ))
        .id();
    let mut drive = PreviewDrive::default();
    // Muito para lá do joelho: sem a repetição a camada teria fugido `~4 900`.
    drive_parallax(
        &mut sim,
        Some(([5000.0, 0.0], [360.0, 0.0])),
        PARADO,
        &mut drive,
    );
    let x = sim.world().get::<Transform>(e).expect("pose").translation.x;
    assert!(
        x.abs() <= tile / 2.0 + 1e-3,
        "a repeticao deixou de ser a ULTIMA: x = {x}, fora de meio ladrilho"
    );
}

/// ⭐⭐⭐ **A FORMA da composição, e ela é BYTE-IDÊNTICA à da W1 — com o CONTROLO que prova que a
/// forma alternativa não é.**
///
/// # ⛔⛔ Porque este gate precisou de existir: o irmão comparava o mutante consigo próprio
///
/// O `sem_limites_a_saida_e_byte_identica` mede *«uma cena com limites inertes dá a mesma pose que
/// uma sem o componente»* — e a mutação que troca a lei por `centro − k·confinado` **SOBREVIVEU**,
/// porque os **dois** lados dela passam pela lei mutada e apanham o mesmo ULP. *Um gate que compara
/// duas corridas do mesmo código não pode ver a forma desse código mudar.*
///
/// ⇒ a régua é a IDENTIDADE entre as duas portas: com `confinado == centro`, a
/// `deslocamento_confinado` tem de devolver a `deslocamento` **ao bit**, porque o segundo termo é
/// `k · 0`.
#[test]
fn a_composicao_do_confinamento_e_byte_identica_a_lei_da_w1() {
    // ⚠️ O corpus CONTÉM os pares discriminantes — medido: das `70` células, `8` dão números
    // diferentes nas duas formas (`(3,7 · 0,3)` lê `2,5899999` contra `2,5900002`). *Um corpus sem
    // eles aprovaria as duas.*
    let centros = [0.1_f32, 3.7, 400.0, -250.0, 1e4, -1e4, 7.0, -3.0, 1234.567];
    let ks = [0.3_f32, 0.7, 0.5, 2.0, -0.5, 0.25, 0.1234];
    let mut discriminantes = 0;
    for c in centros {
        for k in ks {
            let f = ScrollFactor { k: [k, k] };
            let centro = [c, -c];
            assert_eq!(
                f.deslocamento_confinado(centro, centro),
                f.deslocamento(centro),
                "com a vista DENTRO da regiao a lei deixou de ser a da W1 (centro {c}, k {k}) — o \
                 segundo termo deixou de ser `k · 0`"
            );
            // O CONTROLO: a forma alternativa `centro − k·confinado` dá outro número aqui.
            if c * (1.0 - k) != c - k * c {
                discriminantes += 1;
            }
        }
    }
    assert!(
        discriminantes >= 5,
        "so' {discriminantes} pares discriminam as duas formas: este gate deixou de poder ver a \
         diferenca que ele existe para guardar"
    );
}
