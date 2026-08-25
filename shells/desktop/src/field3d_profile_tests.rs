//! A sonda da tolerância do perfil — ver [`super`].

use ph2d_field_render::Orbit;

/// O contorno das duas sondas: um círculo de raio `0,5` em quatro segmentos de Bézier — o caso do
/// torno do Enio.
///
/// ⚠️ **Deslocado do eixo**: o torno recusa um perfil com `x < 0` (regra do documento), e é essa a
/// forma de um perfil de torno a sério — um anel em volta do eixo.
fn circle_path() -> ph2d_vec_scene::VecPath {
    use ph2d_vec_scene::{VecPath, VecVertex};
    let k = 0.552_284_75_f64;
    let r = 0.5_f64;
    let cx = 0.9_f64;
    let mut verts = Vec::new();
    for (i, (ax, ay)) in [(cx + r, 0.0), (cx, r), (cx - r, 0.0), (cx, -r)]
        .into_iter()
        .enumerate()
    {
        let (tx, ty) = [(0.0, k * r), (-k * r, 0.0), (0.0, -k * r), (k * r, 0.0)][i];
        verts.push(VecVertex {
            in_handle: [ax - tx, ay - ty],
            out_handle: [ax + tx, ay + ty],
            ..VecVertex::corner([ax, ay])
        });
    }
    VecPath {
        verts,
        closed: true,
        ..VecPath::default()
    }
}

/// ⭐⭐ **A SONDA QUE ESCOLHEU A TOLERÂNCIA** (W54) — imprime a tabela que vive no doc do
/// [`ph2d_field_profile::TOLERANCE_RATIO`].
///
/// ⚠️ `#[ignore]` porque ela **mede relógio**: 35 traçados a 640×480 são ~8 s, e uma leitura desta
/// máquina não vale nada acima de `load ~5` (a lei do `CLAUDE.md` §5). Corre-se à mão, com a máquina
/// calma, quando o número tiver de ser reconferido:
///
/// ```text
/// cargo test -p ph2d-host-desktop --bin ph2d-host-desktop --release -- \
///     --exact field3d_profile::tests::the_table_that_chose_the_tolerance --ignored --nocapture
/// ```
///
/// ⭐ *Uma tabela num doc-comment sem a sonda ao lado envelhece em silêncio* — e foi exactamente o
/// que aconteceu com a de 2026-08-19, que esta wave desmentiu por 2,4×.
#[test]
#[ignore]
fn the_table_that_chose_the_tolerance() {
    let r = 0.5_f64;
    let path = circle_path();

    println!("ratio | arestas | salto de normal | EXTRUSAO | TORNO | bandas");
    let reg = ph2d_field_eval::hybrid::Registry::new();
    for ratio in [1e-3_f64, 3e-4, 1e-4, 3e-5, 1e-5] {
        let d = 2.0 * r;
        let prof = ph2d_field_profile::cook_path(&path, ratio * d).expect("perfil");
        let n = prof.segment_count();
        let jump = 360.0 / n as f64;
        let sag = r * (1.0 - (std::f64::consts::PI / n as f64).cos());
        // ⚠️ **AS DUAS**: a tabela de 19/08 no doc do `TOLERANCE_RATIO` diz 24,1 ms para 64
        // arestas, e o torno mede 66 ms para 56. Ou o traçado ficou mais caro, ou aquela mediu a
        // EXTRUSÃO. Medir as duas responde, em vez de sobrescrever.
        let mk = |k: ph2d_field::Primitive| {
            ph2d_field::FieldDoc::new(
                vec![ph2d_field::Node {
                    xform: ph2d_field::Xform::IDENTITY,
                    kind: ph2d_field::NodeKind::Leaf(k),
                    mods: Vec::new(),
                }],
                ph2d_field::NodeId(0),
            )
            .expect("a peça")
        };
        let ext = mk(ph2d_field::Primitive::Extrude {
            profile: prof.clone(),
            half_height: 0.2,
            round: 0.0,
        });
        let cam0 = Orbit::from_yaw_pitch(0.72, 0.52);
        let mut ext_ms = Vec::new();
        for _ in 0..7 {
            let t0 = std::time::Instant::now();
            let _ = ph2d_field_render::trace(&ext, &reg, &cam0, 640, 480);
            ext_ms.push(t0.elapsed().as_secs_f64() * 1000.0);
        }
        ext_ms.sort_by(f64::total_cmp);
        let doc = mk(ph2d_field::Primitive::Revolve { profile: prof });
        let cam = Orbit::from_yaw_pitch(0.72, 0.52);
        // Sete corridas, a MEDIANA — uma leitura só é ruído, e três ainda deram uma tabela
        // não-monótona (305 arestas a 978 ms contra 528 a 594).
        let mut ms = Vec::new();
        let mut g = None;
        for _ in 0..7 {
            let t0 = std::time::Instant::now();
            let out = ph2d_field_render::trace(&doc, &reg, &cam, 640, 480);
            ms.push(t0.elapsed().as_secs_f64() * 1000.0);
            g = Some(out);
        }
        ms.sort_by(f64::total_cmp);
        // ⭐ A RÉGUA DAS BANDAS: pixels vizinhos, os dois na peça, cuja NORMAL salta mais que 3°.
        // É o que o olho lê como degrau — e não o erro da silhueta.
        let g = g.expect("traçou");
        let mut bands = 0usize;
        for y in 0..480usize {
            for x in 1..640usize {
                let (a, b) = (y * 640 + x - 1, y * 640 + x);
                if g.hit[a] && g.hit[b] {
                    let d = g.normal[a][0] * g.normal[b][0]
                        + g.normal[a][1] * g.normal[b][1]
                        + g.normal[a][2] * g.normal[b][2];
                    if d.clamp(-1.0, 1.0).acos().to_degrees() > 3.0 {
                        bands += 1;
                    }
                }
            }
        }
        let _ = sag;
        let _ = d;
        println!(
            "{ratio:>5.0e} | {n:>7} | {jump:>10.2}° | {:>7.1} ms | {:>6.1} ms | {bands:>6}",
            ext_ms[3], ms[3]
        );
    }
}

/// ⭐⭐ **A SONDA DO CONTORNO VIVO** (W55) — as duas perguntas que o vínculo levanta, num sítio só.
///
/// 1. **Quanto custa reconferir por quadro?** O vínculo não guarda cache nenhum: ele **recoze** o
///    contorno e compara com o que está no nó (ver [`crate::field3d_profile_live`]). Se isso custasse
///    um milissegundo, o desenho é que teria de avisar — e o desenho não conhece a peça.
/// 2. **Onde fica o TETO do knob de resolução?** O recurso é o traçado **assente**, e ele cresce com
///    as arestas. O número tem de sair da tabela, não do conforto (`CLAUDE.md` §0).
///
/// ⚠️ `#[ignore]` porque mede relógio — máquina calma, `load < 3`:
///
/// ```text
/// cargo test -p ph2d-host-desktop --bin ph2d-host-desktop --release -- \
///     --exact field3d_profile::tests::the_table_that_chose_the_resolution_ceiling --ignored --nocapture
/// ```
#[test]
#[ignore]
fn the_table_that_chose_the_resolution_ceiling() {
    let path = circle_path();
    let reg = ph2d_field_eval::hybrid::Registry::new();
    println!("nivel | ratio | arestas | recozer | comparar | EXTRUSAO");
    // ⚠️ **A escada vai muito além do teto de propósito** (W60): o teto de `16` foi escolhido
    // quando o traçado custava `0,95–1,10 ms/aresta`, e as waves W56e–W59 baixaram esse número. A
    // `CLAUDE.md` §0 obriga quem move o custo a **reconferir a nota** que o custo tornava
    // inalcançável — e reconferir exige medir onde ela agora cai.
    for level in [1u32, 2, 4, 8, 16, 32, 64, 128] {
        let ratio = ph2d_field_profile::TOLERANCE_RATIO / f64::from(level);
        let d = 1.0_f64;
        // O recozimento inteiro, como o vínculo o faz — achatar + validar o documento.
        let mut cook_us = Vec::new();
        let mut prof = None;
        for _ in 0..21 {
            let t0 = std::time::Instant::now();
            let p = ph2d_field_profile::cook_path(&path, ratio * d).expect("perfil");
            cook_us.push(t0.elapsed().as_secs_f64() * 1e6);
            prof = Some(p);
        }
        cook_us.sort_by(f64::total_cmp);
        let prof = prof.expect("perfil");
        let n = prof.segment_count();
        // A comparação que decide se alguma coisa mudou — o caso comum é IGUAL, que é o pior caso
        // dela (percorre tudo antes de responder).
        let twin = ph2d_field_profile::cook_path(&path, ratio * d).expect("perfil");
        let mut cmp_us = Vec::new();
        for _ in 0..21 {
            let t0 = std::time::Instant::now();
            assert!(prof == twin);
            cmp_us.push(t0.elapsed().as_secs_f64() * 1e6);
        }
        cmp_us.sort_by(f64::total_cmp);
        let doc = ph2d_field::FieldDoc::new(
            vec![ph2d_field::Node {
                xform: ph2d_field::Xform::IDENTITY,
                kind: ph2d_field::NodeKind::Leaf(ph2d_field::Primitive::Extrude {
                    profile: prof,
                    half_height: 0.2,
                    round: 0.0,
                }),
                mods: Vec::new(),
            }],
            ph2d_field::NodeId(0),
        )
        .expect("a peça");
        let cam = Orbit::from_yaw_pitch(0.72, 0.52);
        let mut ms = Vec::new();
        for _ in 0..7 {
            let t0 = std::time::Instant::now();
            let _ = ph2d_field_render::trace(&doc, &reg, &cam, 640, 480);
            ms.push(t0.elapsed().as_secs_f64() * 1000.0);
        }
        ms.sort_by(f64::total_cmp);
        println!(
            "{level:>5} | {ratio:>5.0e} | {n:>7} | {:>6.1} us | {:>6.2} us | {:>7.1} ms",
            cook_us[10], cmp_us[10], ms[3]
        );
    }
}

/// ⛔⛔ **O TETO DE `Resolution` EM FUNÇÃO DO ZOOM — o eixo que NÃO existe** (W60).
///
/// ⚠️ **Esta sonda ficou como REGISTO de três refutações**, e não como régua viva. Ela nasceu para
/// responder *"o teto de 16 pode subir agora que o traçado ficou 2,2× mais barato?"*, e as três
/// respostas que ela deu foram todas *"não é aqui que se pergunta"*.
///
/// # Por que esta sonda existe
///
/// A `CLAUDE.md` §0 obriga: *"quem move o número que tornava algo inalcançável tem de reconferir a
/// nota"*. As waves W56e–W59 baixaram o traçado ~`2,2×`, e o teto de `16` tinha **duas** pernas:
///
/// 1. **o relógio** — 664 arestas custavam `648,7 ms`, e meio segundo é onde o artista lê *"o app
///    prendeu"* em vez de *"está a afinar"*. ⇒ **essa perna caiu**: hoje custam `288`–`317 ms`.
/// 2. **o olho** — *"o nível 32 não compra nada que se veja"*. ⇒ essa continua de pé, **e é mais
///    forte do que o doc dizia**: a régua das bandas põe o joelho em **168 arestas**, que é o
///    próprio default.
///
/// ⚠️ **Mas aquela régua correu num enquadramento só.** O doc do `DEFAULT_PROFILE_RESOLUTION` já
/// escrevia a suspeita — *"o knob existe para a peça que é grande ou vista de perto"* —, e é
/// exactamente isso que nunca foi medido. Um degrau que se esconde a `half_extent = 0,8` pode
/// aparecer a `0,1`.
///
/// # As três refutações, na ordem em que apareceram
///
/// 1. ⛔ **A régua das bandas SATURA.** Ela conta pixels vizinhos com salto de normal acima de
///    **3°**, e o salto do nível 1 já é `2,14°` — logo ela devolve o mesmo número do nível 1 ao 64
///    (`91`, `117`, `98`, `102`, `97`, `98`, `96`). *Uma régua com limiar não distingue nada que
///    esteja todo abaixo dele*, e os ~100 pixels que ela conta são o **aro** da extrusão, uma quina
///    de 90° que é geometria de verdade.
/// 2. ⛔ **Sem limiar ela é engolida pelo aro.** O `p99,9` do salto dá `11°` a `half_extent = 0,8` e
///    `79°` a `0,4`, também plano em todos os níveis — é o aro outra vez, agora sem nada a
///    escondê-lo.
/// 3. ⛔ **E o eixo do zoom não é alcançável assim: a câmera ENTRA na peça.** Na lente convergente
///    `olho = half_extent / tan(meia abertura)`, e com `tan(0,3454) = 0,360` isso dá `0,556` a
///    `half_extent = 0,2` — contra uma bola de raio `0,539`. As três linhas de baixo da tabela são
///    **quadros vazios**, e por isso `0,00`.
///
/// # ⭐⭐⭐ E a razão de fundo: o facetamento é INVARIANTE À ESCALA
///
/// O cozimento **não conhece a câmera** — `cook_path_at` deriva a tolerância de `span × ratio`, com
/// `span` a extensão do **desenho**. ⇒ o perfil que sai é o mesmo em qualquer enquadramento, e o
/// salto de normal de um círculo de `n` lados é `360/n` **sempre**. *«O knob existe para a peça
/// vista de perto» é uma afirmação sobre a SILHUETA, não sobre a luz — e a W54 mediu que a silhueta
/// erra `0,079 %` da peça enquanto a normal salta `6,43°`.*
///
/// ⇒ **A pergunta que sobra não se responde num círculo.** Ela pede um contorno de curvatura
/// **variável** (uma quina apertada ao lado de um arco longo), onde o achatamento gasta segmentos
/// de forma desigual. Fica ⏸️, com este registo no lugar de um palpite.
///
/// ```text
/// cargo test -p ph2d-host-desktop --release -- --exact \
///     field3d_profile::tests::the_table_of_where_the_banding_knee_moves_with_zoom \
///     --ignored --nocapture
/// ```
#[test]
#[ignore]
fn the_table_of_where_the_banding_knee_moves_with_zoom() {
    let path = circle_path();
    let reg = ph2d_field_eval::hybrid::Registry::new();
    println!("half_extent | nível=1 | 2 | 4 | 8 | 16 | 32 | 64   (p99,9 do salto de normal, °)");
    for half in [0.8_f32, 0.4, 0.2, 0.1, 0.05] {
        let mut cells = Vec::new();
        for level in [1u32, 2, 4, 8, 16, 32, 64] {
            let ratio = ph2d_field_profile::TOLERANCE_RATIO / f64::from(level);
            let prof = ph2d_field_profile::cook_path(&path, ratio).expect("perfil");
            let doc = ph2d_field::FieldDoc::new(
                vec![ph2d_field::Node {
                    xform: ph2d_field::Xform::IDENTITY,
                    kind: ph2d_field::NodeKind::Leaf(ph2d_field::Primitive::Extrude {
                        profile: prof,
                        half_height: 0.2,
                        round: 0.0,
                    }),
                    mods: Vec::new(),
                }],
                ph2d_field::NodeId(0),
            )
            .expect("a peça");
            let mut cam = Orbit::from_yaw_pitch(0.72, 0.52);
            cam.half_extent = half;
            let g = ph2d_field_render::trace(&doc, &reg, &cam, 640, 480);
            // ⛔ **A régua da tabela irmã CONTA pixels acima de 3°, e ela satura aqui:** o salto
            // de normal do nível 1 já é `2,14°`, então ela devolve o mesmo número do nível 1 ao 64 —
            // *uma régua com limiar não distingue nada que esteja todo abaixo dele*. A régua desta
            // sonda é o **percentil 99,9** do salto entre vizinhos, sem limiar nenhum.
            let mut jumps: Vec<f32> = Vec::new();
            for y in 0..480usize {
                for x in 1..640usize {
                    let (a, b) = (y * 640 + x - 1, y * 640 + x);
                    if g.hit[a] && g.hit[b] {
                        let d = g.normal[a][0] * g.normal[b][0]
                            + g.normal[a][1] * g.normal[b][1]
                            + g.normal[a][2] * g.normal[b][2];
                        jumps.push(d.clamp(-1.0, 1.0).acos().to_degrees());
                    }
                }
            }
            jumps.sort_by(f32::total_cmp);
            // ⚠️ **p99,9 e não o MÁXIMO**: o máximo é a quina viva do aro da extrusão (90°), que é
            // geometria de verdade e não faceta — ela mascara tudo o que a sonda quer ver.
            let p = jumps
                .get(jumps.len().saturating_sub(1).saturating_mul(999) / 1000)
                .copied()
                .unwrap_or(0.0);
            cells.push(format!("{p:>6.2}"));
        }
        println!("{half:>11.2} | {}", cells.join(" |"));
    }
}
