//! ⭐⭐⭐ **OS GATES DAS LÂMPADAS** — várias, com sombra, e o tecto que sai da placa.
//!
//! ⚠️ **Irmão por ASSUNTO do [`super::paint_parity_tests`]:** aquele mede a lei do PINTOR (o
//! material, o céu, o olhar, o dono) com uma lâmpada; estes medem a lei da LUZ — quantas cabem,
//! quanto custam, e o que acontece acima do tecto.

use super::{FUNDO, H, W, dois_caminhos, fixtura};

/// As `n` primeiras lâmpadas de uma lista FIXA — cada uma com posição e cor próprias.
///
/// ⚠️⚠️ **A lista é um PREFIXO e as posições vivem à FRENTE da peça, e as duas coisas são o
/// controlo.** A primeira redacção espalhava-as num círculo completo: passar de `1` para `2`
/// punha a segunda **atrás** da peça, movia `811` canais, e o gate reprovou. ⛔ A leitura fácil
/// — *«o dispositivo ignora a segunda»* — estava **refutada na linha de cima**: a paridade com a
/// CPU lia `100 %` com `pior 0`, e a CPU usa as duas. *O que estava errado era o corpus, não a
/// lei — e é o controlo que distingue as duas coisas.*
///
/// ⚠️ **Cores diferentes de propósito:** com lâmpadas brancas, perder uma no meio devolveria quase
/// a mesma imagem.
fn constelacao(cam: &ph2d_field_render::Orbit, n: usize) -> Vec<ph2d_field_render::PointLamp> {
    let r = 2.2 * cam.half_extent;
    let (right, up, toward_eye) = cam.basis();
    // Oito direcções no hemisfério que o olho vê — todas com componente positiva para o olho.
    const POSTOS: [[f32; 3]; 8] = [
        [-0.55, 0.66, 0.50],
        [0.62, 0.35, 0.70],
        [-0.20, -0.62, 0.75],
        [0.75, -0.30, 0.59],
        [-0.80, 0.10, 0.59],
        [0.10, 0.80, 0.59],
        [0.40, -0.70, 0.59],
        [-0.45, -0.25, 0.86],
    ];
    const CORES: [[f32; 3]; 8] = [
        [2.4, 1.6, 1.0],
        [1.0, 2.0, 2.4],
        [2.2, 0.8, 2.0],
        [1.2, 2.4, 1.0],
        [2.4, 2.0, 0.6],
        [0.8, 1.4, 2.4],
        [2.0, 2.4, 1.8],
        [2.4, 1.0, 1.4],
    ];
    // ⚠️ **A lista CICLA acima de oito** — é o que deixa o gate do tecto pedir `MAX_LAMPS + 1`
    // lâmpadas sem inventar posições novas. Com `.take(n)` sobre oito, pedir nove devolvia oito e o
    // gate media outra coisa.
    (0..n)
        .map(|i| ph2d_field_render::PointLamp {
            world: {
                let e = POSTOS[i % POSTOS.len()];
                [0, 1, 2].map(|c| {
                    cam.target[c] + r * (e[0] * right[c] + e[1] * up[c] + e[2] * toward_eye[c])
                })
            },
            radiance_at_one: CORES[i % CORES.len()],
        })
        .collect()
}

/// ⭐⭐⭐ **VÁRIAS LÂMPADAS, e todas com sombra** — o item que ficava na CPU.
///
/// # ⛔⛔ O que o formato antigo não conseguia dizer
///
/// O canal de luz do dispositivo era um `vec2` por pixel: o céu e **UMA** sombra. Com duas
/// lâmpadas a segunda ficava sem sombra **em silêncio** — e por isso o chamador caía na CPU
/// inteira em vez de a ignorar. *Um formato que não tem onde pôr a segunda resposta é um tecto
/// escrito em bytes.*
///
/// ⇒ o canal passa a ter passo `1 + n_lamps`, e este gate mede-o até ao tecto.
#[test]
#[ignore = "precisa de GPU"]
fn varias_lampadas_sombreiam_todas_e_concordam_com_a_cpu() {
    if crate::gpu_frame::shared().is_none() {
        println!("sem adaptador — saltado");
        return;
    }
    let (doc, postas, materiais) = fixtura();
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let cam = ph2d_field_render::Orbit::default();
    let owners = ph2d_field_eval::owners::Owners::new(
        &postas,
        &reg,
        ph2d_field_render::hit_tolerance(
            cam.half_extent,
            f32::from(u16::try_from(W.min(H)).expect("a tela cabe")),
        ),
    );
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: Some(&owners),
    };

    let mut anterior: Option<Vec<u8>> = None;
    let cabem =
        crate::gpu_frame::lamps_that_fit(crate::gpu_frame::shared().expect("o adaptador"), W, H);
    for n in [1usize, 2, 3, ph2d_field_gpu::trace::MAX_LAMPS.min(cabem)] {
        let luz = constelacao(&cam, n);
        let (cpu, gpu, _) = dois_caminhos(&surfaces, &doc, &luz)
            .unwrap_or_else(|| panic!("o dispositivo tem de tomar {n} lâmpada(s)"));
        let pior = cpu
            .iter()
            .zip(gpu.iter())
            .map(|(a, b)| a.abs_diff(*b))
            .max()
            .unwrap_or(0);
        #[allow(clippy::cast_precision_loss)]
        let fraccao = cpu
            .iter()
            .zip(gpu.iter())
            .filter(|(a, b)| a.abs_diff(**b) <= 1)
            .count() as f64
            / cpu.len() as f64;
        println!(
            "  {n} lâmpada(s) · ≤1 nível em {:.3} % · pior {pior}",
            fraccao * 100.0
        );
        assert!(
            fraccao >= 0.995 && pior <= 2,
            "com {n} lâmpadas: {:.3} % a ≤1 nível, pior {pior}",
            fraccao * 100.0
        );
        // ⭐ **O CONTROLO: acrescentar uma lâmpada MUDA a imagem.** Sem ele, um shader que
        // ignorasse as lâmpadas acima da primeira leria `100 %` em todas as linhas — e o gate
        // estaria a comparar duas cópias do mesmo defeito.
        if let Some(antes) = &anterior {
            let mexeu = antes
                .iter()
                .zip(gpu.iter())
                .filter(|(a, b)| a.abs_diff(**b) > 2)
                .count();
            assert!(
                mexeu > 1_000,
                "passar para {n} lâmpadas moveu só {mexeu} canais — as lâmpadas extra não chegam \
                 ao pixel"
            );
        }
        anterior = Some(gpu);
    }
}

/// ⛔⛔ **ACIMA DO TECTO O DISPOSITIVO RECUSA — em voz alta, e não pela metade.**
///
/// ⚠️ **A recusa É a feature.** Sombrear as primeiras `MAX_LAMPS` e deixar as outras sem sombra
/// devolveria uma imagem plausível e errada, que é exactamente o defeito que esta wave curou um
/// nível abaixo. ⇒ `None`, e o chamador cai na CPU, que sabe sombrear qualquer número.
#[test]
#[ignore = "precisa de GPU"]
fn acima_do_tecto_de_lampadas_o_dispositivo_recusa() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let (doc, _, materiais) = fixtura();
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let cam = ph2d_field_render::Orbit::default();
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: None,
    };
    let olhar = ph2d_view_transform::Look::default();
    let pinta = |n: usize| {
        crate::gpu_frame::paint(
            t,
            &doc,
            &reg,
            &cam,
            &constelacao(&cam, n),
            &surfaces,
            &ph2d_field_render::Presentation::of(olhar),
            FUNDO,
            None,
            W,
            H,
            true,
        )
    };
    let cabem = crate::gpu_frame::lamps_that_fit(t, W, H);
    // ⭐ **O tecto é do QUADRO**, e este gate prova-o nos dois sentidos: no número ele toma, um
    // acima recusa. ⚠️ Ele também tem de ser MENOR que a lista cíclica consegue produzir — senão
    // a linha de baixo mediria outra coisa.
    assert!(
        cabem > 0 && cabem <= ph2d_field_gpu::trace::MAX_LAMPS,
        "o tecto deste quadro leu {cabem} — fora da faixa que o uniforme comporta"
    );
    assert!(
        pinta(cabem).is_some(),
        "no tecto ({cabem}) o dispositivo tem de tomar — senão a linha abaixo não afirma nada"
    );
    assert!(
        pinta(cabem + 1).is_none(),
        "acima do tecto ({cabem}) o dispositivo tomou o quadro — as lâmpadas extra ficariam sem \
         sombra"
    );
    // ⭐⭐ **E o tecto ENCOLHE com a tela** — a mesma placa, uma tela maior, menos lâmpadas.
    //
    // ⚠️⚠️ **A régua usa o PISO GARANTIDO da `wgpu` (`128 MiB`) e não o limite desta placa**, e a
    // primeira redacção usava o da placa e reprovou: ao pedir `adapter.limits()` em vez do
    // `Limits::default()`, o tamanho de ligação subiu e o `MAX_LAMPS` passou a saturar os dois
    // lados (`32` e `32`). *A propriedade que este gate afirma é da LEI, e medi-la num número que
    // a máquina escolhe faz dela uma medida da máquina.*
    let piso: u64 = 134_217_728;
    let (pequena, grande) = (
        ph2d_field_gpu::trace::lamps_that_fit(piso, W, H),
        ph2d_field_gpu::trace::lamps_that_fit(piso, 1920, 1080),
    );
    println!(
        "  tecto · esta placa liga {} B · ao piso da wgpu: {W}×{H} → {pequena} · 1920×1080 → {grande}",
        crate::gpu_frame::binding_limit(t)
    );
    assert!(
        grande < pequena,
        "ao piso da wgpu, {W}×{H} cabem {pequena} e 1920×1080 cabem {grande} — o tecto não está a \
         olhar para a tela"
    );
    // ⚠️ E ZERO lâmpadas também recusa: a peça sai acesa só pelo céu, e esse caminho é da CPU.
    assert!(
        pinta(0).is_none(),
        "sem lâmpada nenhuma o dispositivo tem de recusar"
    );
}

/// ⏱️⭐ **O QUE CADA LÂMPADA CUSTA** — a tabela de que o `MAX_LAMPS` sai.
#[test]
#[ignore = "medição — precisa de GPU e de máquina calma"]
fn mede_o_que_cada_lampada_custa_no_dispositivo() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    const LW: u32 = 1920;
    const LH: u32 = 1080;
    let (doc, _, materiais) = fixtura();
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let cam = ph2d_field_render::Orbit::default();
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: None,
    };
    let olhar = ph2d_view_transform::Look::default();
    println!(
        "\n  {LW}×{LH} · load {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    println!("  lâmpadas      min   mediana");
    let cabem = crate::gpu_frame::lamps_that_fit(t, LW, LH);
    println!("  (cabem {cabem} nesta tela — ver `lamps_that_fit`)");
    for n in [
        1usize,
        2,
        4,
        8,
        12,
        16,
        24,
        ph2d_field_gpu::trace::MAX_LAMPS,
    ]
    .into_iter()
    .filter(|n| *n <= cabem)
    {
        let luz = constelacao(&cam, n);
        let mut v: Vec<f64> = Vec::new();
        for _ in 0..7 {
            let t0 = std::time::Instant::now();
            let p = crate::gpu_frame::paint(
                t,
                &doc,
                &reg,
                &cam,
                &luz,
                &surfaces,
                &ph2d_field_render::Presentation::of(olhar),
                FUNDO,
                None,
                LW,
                LH,
                true,
            )
            .expect("o pintor");
            std::hint::black_box(p.rgba.len());
            v.push(t0.elapsed().as_secs_f64() * 1e3);
        }
        v.sort_by(f64::total_cmp);
        println!("  {n:>8}   {:6.2}   {:6.2} ms", v[0], v[v.len() / 2]);
    }
}
