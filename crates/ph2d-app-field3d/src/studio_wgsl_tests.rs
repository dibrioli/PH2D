//! O gate do céu — irmão do `o_material_do_dispositivo_e_o_da_cpu`, e pela mesma razão.

const ARNES: &str = r#"
// A ordem é a do `ph2d_field_gpu::probe::evaluate`: uniformes, depois storages, entrada e saída.
@group(0) @binding(0) var<uniform> ceu: Ceu;
@group(0) @binding(1) var<storage, read> tabela: array<f32>;
@group(0) @binding(2) var<storage, read> entrada: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read_write> saida: array<vec4<f32>>;

@compute @workgroup_size(64, 1, 1)
fn avalia(@builtin(global_invocation_id) g: vec3<u32>) {
    let i = g.x;
    if (i >= arrayLength(&entrada)) { return; }
    let a = entrada[i];
    // `a.xyz` é a direcção; `a.w` é o `alpha`. O `shrink` chega a `1` — o que este gate mede é a
    // TABELA e a rampa, e o encolhimento tem a lei dele na CPU (ver o `ENV_SLOT`).
    saida[i * 2u + 0u] = vec4<f32>(ceu_radiance(a.xyz, a.w, 1.0), 0.0);
    saida[i * 2u + 1u] = vec4<f32>(ceu_irradiance(a.xyz), 0.0);
}
"#;

/// ⭐⭐⭐ **O CÉU DO DISPOSITIVO É O DO PRODUTO** — a rampa e as duas tabelas da caixa de luz.
///
/// ⚠️ **O `shrink` é forçado a `1` dos dois lados.** Ele é `f64` na CPU e constante por material,
/// logo viaja pronto no material — o que este gate tem de medir é a **TABELA** (`49 × 513` células
/// com interpolação bilinear) e a rampa. *Medir aqui uma constante que chega de fora seria medir o
/// mensageiro.*
///
/// ⚠️ **A grelha varre o eixo da caixa**, que é `+y`: é `cos ψ` que endereça a tabela, e uma
/// interpolação trocada de eixo lê-se como um céu quase certo.
#[test]
#[ignore = "precisa de GPU"]
fn o_ceu_do_dispositivo_e_o_do_produto() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };

    // A grelha: direcções que varrem o eixo da caixa, e rugosidades que varrem a tabela.
    let mut amostras: Vec<[f32; 4]> = Vec::new();
    for iy in 0..33u8 {
        let y = -1.0 + 2.0 * f32::from(iy) / 32.0;
        let s = (1.0 - y * y).max(0.0).sqrt();
        for ia in 0..8u8 {
            let a = f32::from(ia) * 0.9;
            for alpha in [0.0_f32, 0.0004, 0.01, 0.09, 0.25, 0.64, 1.0] {
                amostras.push([s * a.cos(), y, s * a.sin(), alpha]);
            }
        }
    }
    assert!(amostras.len() > 1_000, "grelha pobre: {}", amostras.len());

    let fonte = format!("{}\n{ARNES}", crate::studio_wgsl::SOURCE);
    let guarda = t.lock().expect("o traçador");
    let (device, queue) = guarda.parts();
    let saida = ph2d_field_gpu::probe::evaluate(
        device,
        queue,
        &fonte,
        "avalia",
        &[&crate::studio_wgsl::constants()],
        &[&crate::studio_wgsl::tables()],
        &amostras,
        2,
    );
    drop(guarda);

    // ⚠️ **O lado da CPU com o `shrink` neutralizado**: a rampa nua mais a caixa, que é exactamente
    // o que a `Studio::radiance` faz com `lobe_shrink == 1`.
    let estudio = crate::studio::Studio::of_the_product();
    let mut pior = (0.0f64, 0usize, "");
    let mut vivos = 0usize;
    for (i, a) in amostras.iter().enumerate() {
        let dir = [a[0], a[1], a[2]];
        // `radiance` com `shrink = 1` — a mesma expressão, com o factor trocado por um.
        let cpu_r = {
            let (_, _, share, amp) = estudio.softbox.expect("caixa").tables();
            let up = dir[1];
            let base = 1.0 - share + amp * estudio.softbox.expect("caixa").specular(a[3], dir[1]);
            [0, 1, 2].map(|c| {
                ph2d_light::AMBIENT
                    * (ph2d_light::ENV_BASE[c] * base + 1.5 * ph2d_light::ENV_SLOPE[c] * up)
            })
        };
        let cpu_i = estudio.irradiance(dir);
        if cpu_r.iter().chain(&cpu_i).any(|c| *c > 1e-4) {
            vivos += 1;
        }
        for (k, (nome, cpu)) in [("radiance", cpu_r), ("irradiance", cpu_i)]
            .iter()
            .enumerate()
        {
            for c in 0..3 {
                let g = saida[i * 2 + k][c];
                let rel = f64::from((cpu[c] - g).abs()) / f64::from(cpu[c].abs().max(g).max(1e-3));
                if rel > pior.0 {
                    pior = (rel, i, nome);
                }
            }
        }
    }
    assert!(
        vivos * 2 > amostras.len(),
        "o céu respondeu em {vivos} de {} amostras — a grelha não o exercita",
        amostras.len()
    );
    println!(
        "  céu · pior desvio relativo {:.3e} · na amostra {} ({})",
        pior.0, pior.1, pior.2
    );
    // A mesma folga do material, e pela mesma razão: `f32` com contracção, sobre uma interpolação
    // bilinear. ⛔ Um eixo trocado na tabela move DÉCIMAS.
    assert!(
        pior.0 < 1e-4,
        "o céu dos dois motores difere {:.3e} ({}) — não é `f32`, é outra tabela ou outro eixo",
        pior.0,
        pior.2
    );
}

const ARNES_OLHAR: &str = r#"
struct Qual { view: vec4<f32> };
@group(0) @binding(0) var<uniform> qual: Qual;
@group(0) @binding(1) var<storage, read> entrada: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> saida: array<vec4<f32>>;

@compute @workgroup_size(64, 1, 1)
fn avalia(@builtin(global_invocation_id) g: vec3<u32>) {
    let i = g.x;
    if (i >= arrayLength(&entrada)) { return; }
    let a = entrada[i];
    // ⚠️ `a.xyz` é a luz da cena e `a.w` são os STOPS, crus. A vista vem por uniforme e há um
    // despacho por variante — a 1.ª redacção dobrou as duas num só número e não sabia representar
    // `stops = 1,5`. *Um codificador é uma lei a mais para o gate ter de estar certa.*
    saida[i] = vec4<f32>(vt_to_display(a.xyz, a.w, u32(qual.view.x)), 0.0);
}
"#;

/// ⭐⭐⭐ **O OLHAR DO DISPOSITIVO É O DA CPU** — a exposição e as duas transformações.
///
/// ⚠️ **A `Neutral` é a que importa**: ela tem três degraus (o desvio com joelho quadrático, a
/// compressão do pico e a dessaturação), e cada um tem uma fronteira onde um `<` trocado por `<=`
/// muda a resposta. ⇒ a grelha varre **as fronteiras**: `0,08` do joelho e `0,76` da compressão.
#[test]
#[ignore = "precisa de GPU"]
fn o_olhar_do_dispositivo_e_o_da_cpu() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let mut amostras: Vec<[f32; 4]> = Vec::new();
    let mut esperado: Vec<[f32; 3]> = Vec::new();
    // ⚠️ As fronteiras da lei, e não uma grelha uniforme que passaria ao lado das duas.
    let niveis = [
        0.0_f32, 1e-4, 0.03, 0.0799, 0.08, 0.0801, 0.3, 0.7599, 0.76, 0.7601, 0.95, 1.0, 1.7, 9.0,
    ];
    for r in niveis {
        for g in [0.0_f32, 0.05, 0.4, 0.9, 2.5] {
            for b in [0.0_f32, 0.12, 0.66, 1.4] {
                for stops in [-3.0_f32, 0.0, 1.5] {
                    amostras.push([r, g, b, stops]);
                }
            }
        }
    }
    assert!(
        amostras.len() >= 840,
        "grelha pobre: {} (14 níveis × 5 × 4 × 3 exposições; as duas vistas são despachos à parte)",
        amostras.len()
    );

    let fonte = format!("{}\n{ARNES_OLHAR}", ph2d_view_transform::wgsl::SOURCE);
    let guarda = t.lock().expect("o traçador");
    let (device, queue) = guarda.parts();
    // ⭐ Um despacho por variante do olhar — a vista é uniforme, não um bit escondido no `w`.
    let mut saida = Vec::new();
    for view in ph2d_view_transform::ViewTransform::ALL {
        #[allow(clippy::cast_precision_loss)]
        let codigo = [
            f32::from(u8::try_from(ph2d_view_transform::wgsl::view_code(view)).unwrap_or(0)),
            0.0,
            0.0,
            0.0,
        ];
        saida.push(ph2d_field_gpu::probe::evaluate(
            device,
            queue,
            &fonte,
            "avalia",
            &[&codigo],
            &[],
            &amostras,
            1,
        ));
        for a in &amostras {
            esperado.push(
                ph2d_view_transform::Look {
                    exposure_stops: a[3],
                    view,
                }
                .apply([a[0], a[1], a[2]]),
            );
        }
    }
    let saida: Vec<[f32; 4]> = saida.into_iter().flatten().collect();
    drop(guarda);

    // ⭐ O controlo: a `Neutral` tem de MOVER alguma coisa, senão o gate compara duas identidades.
    assert_eq!(
        esperado.len(),
        saida.len(),
        "as duas listas têm de ter o mesmo tamanho"
    );
    let mexeu = esperado
        .iter()
        .zip(amostras.iter().chain(&amostras))
        .filter(|(e, a)| (e[0] - a[0]).abs() > 1e-3)
        .count();
    assert!(
        mexeu * 8 > esperado.len(),
        "o olhar só mexeu em {mexeu} de {} amostras — a grelha não o exercita",
        esperado.len()
    );

    let mut pior = (0.0f64, 0usize);
    for (i, (e, g)) in esperado.iter().zip(&saida).enumerate() {
        for c in 0..3 {
            let rel = f64::from((e[c] - g[c]).abs()) / f64::from(e[c].abs().max(g[c]).max(1e-3));
            if rel > pior.0 {
                pior = (rel, i);
            }
        }
    }
    println!(
        "  olhar · pior desvio relativo {:.3e} · na amostra {}",
        pior.0, pior.1
    );
    assert!(
        pior.0 < 1e-4,
        "o olhar dos dois motores difere {:.3e} — um degrau da `Neutral` está noutro sítio",
        pior.0
    );
}
