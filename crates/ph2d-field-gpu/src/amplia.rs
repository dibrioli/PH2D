//! ⭐⭐⭐⭐ **A AMPLIAÇÃO NA PLACA** — quando o quadro de movimento é traçado mais pequeno do que a
//! área (a resolução dinâmica da `ph2d-app-field3d::preview`), a imagem sobe ao tamanho CHEIO aqui,
//! antes de sair do dispositivo, e o ecrã desenha-a `1:1`.
//!
//! ⛔⛔ **Até 2026-10-01 a imagem mais pequena saía assim e o ecrã esticava-a em BILINEAR**, com a
//! escala em três degraus (`1`, `½`, `⅓` da largura) — a foto do dono (*«a resolução ainda cai»*)
//! era isso: metade da largura esticada, a aresta em escada. ⇒ a escala passou a contínua e a
//! ampliação é a **bicúbica de Catmull-Rom (16 amostras) com ANTI-ANEL**: o resultado de cada canal
//! fica preso entre o mínimo e o máximo dos QUATRO texels mais perto do ponto. ⚠️ É essa trava que
//! a torna segura aqui: a aresta sai anti-serrilhada do traçador, e uma bicúbica crua TOCA (*ringing*)
//! numa aresta de alto contraste — o halo que o comentário do `draw_stable_image` já recusava. Com
//! a trava, um valor novo nunca sai da gama dos vizinhos que o pixel de facto cobre.
//!
//! ⚠️ A interpolação é sobre os BYTES em sRGB (o que o olho vê), e os quatro canais por igual —
//! o alfa é a cobertura do fundo, e interpolá-lo junto mantém a borda da peça coerente com a cor.

/// O WGSL do passe.
const WGSL: &str = r"
struct Dims { origem: vec4<u32> };
@group(0) @binding(0) var<storage, read> entrada: array<u32>;
@group(0) @binding(1) var<storage, read_write> saida: array<u32>;
@group(0) @binding(2) var<uniform> D: Dims;

fn texel(x: i32, y: i32) -> vec4<f32> {
    let w = i32(D.origem.x);
    let h = i32(D.origem.y);
    let px = entrada[u32(clamp(y, 0, h - 1) * w + clamp(x, 0, w - 1))];
    return vec4<f32>(f32(px & 255u), f32((px >> 8u) & 255u), f32((px >> 16u) & 255u), f32(px >> 24u));
}

// Os quatro pesos de Catmull-Rom para a fracção `t`.
fn pesos(t: f32) -> vec4<f32> {
    let t2 = t * t;
    let t3 = t2 * t;
    return vec4<f32>(
        -0.5 * t3 + t2 - 0.5 * t,
        1.5 * t3 - 2.5 * t2 + 1.0,
        -1.5 * t3 + 2.0 * t2 + 0.5 * t,
        0.5 * t3 - 0.5 * t2,
    );
}

@compute @workgroup_size(8, 8, 1)
fn amplia(@builtin(global_invocation_id) g: vec3<u32>) {
    let tw = D.origem.z;
    let th = D.origem.w;
    if (g.x >= tw || g.y >= th) { return; }
    // O centro do pixel de saída, nas coordenadas de texel da entrada.
    let u = (f32(g.x) + 0.5) * f32(D.origem.x) / f32(tw) - 0.5;
    let v = (f32(g.y) + 0.5) * f32(D.origem.y) / f32(th) - 0.5;
    let x0 = i32(floor(u));
    let y0 = i32(floor(v));
    let wx = pesos(u - f32(x0));
    let wy = pesos(v - f32(y0));
    var soma = vec4<f32>(0.0);
    for (var j: i32 = 0; j < 4; j = j + 1) {
        var linha = vec4<f32>(0.0);
        for (var i: i32 = 0; i < 4; i = i + 1) {
            linha = linha + texel(x0 - 1 + i, y0 - 1 + j) * wx[i];
        }
        soma = soma + linha * wy[j];
    }
    // ⭐ O ANTI-ANEL: preso à gama dos quatro texels que o ponto cobre.
    let a = texel(x0, y0);
    let b = texel(x0 + 1, y0);
    let c = texel(x0, y0 + 1);
    let d = texel(x0 + 1, y0 + 1);
    let lo = min(min(a, b), min(c, d));
    let hi = max(max(a, b), max(c, d));
    let r = clamp(soma, lo, hi);
    let q = vec4<u32>(clamp(r + vec4<f32>(0.5), vec4<f32>(0.0), vec4<f32>(255.0)));
    saida[g.y * tw + g.x] = q.x | (q.y << 8u) | (q.z << 16u) | (q.w << 24u);
}
";

/// ⭐ **Amplia `entrada` (`origem`, um `u32` RGBA por pixel) para `destino`** no encoder dado, e
/// devolve o buffer da imagem grande (`destino.0 × destino.1` pixels, `COPY_SRC`).
pub(crate) fn amplia(
    device: &wgpu::Device,
    cache: &mut crate::FieldPipelines,
    fita: &ph2d_field_eval::wgsl::TapeWgsl,
    enc: &mut wgpu::CommandEncoder,
    entrada: &wgpu::Buffer,
    origem: (u32, u32),
    destino: (u32, u32),
) -> wgpu::Buffer {
    use wgpu::util::DeviceExt;
    let n = u64::from(destino.0) * u64::from(destino.1);
    let saida = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("amplia"),
        size: (n * 4).max(16),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("amplia"),
        entries: &[
            crate::trace::armazem(0, true),
            crate::trace::armazem(1, false),
            crate::trace::uniforme(2),
        ],
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("amplia"),
        bind_group_layouts: &[Some(&bgl)],
        immediate_size: 0,
    });
    let p = cache
        .entry_with_layout(device, WGSL, fita, "amplia", Some(&layout))
        .clone();
    let ub = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("amplia"),
        contents: &[origem.0, origem.1, destino.0, destino.1]
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect::<Vec<u8>>(),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("amplia"),
        layout: &bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: entrada.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: saida.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: ub.as_entire_binding(),
            },
        ],
    });
    let mut crono = cache.cronometro.take();
    {
        let mut cp = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("amplia"),
            timestamp_writes: crono.as_mut().and_then(|c| c.marca("amplia")),
        });
        cp.set_pipeline(&p);
        cp.set_bind_group(0, &bg, &[]);
        cp.dispatch_workgroups(destino.0.div_ceil(8), destino.1.div_ceil(8), 1);
    }
    cache.cronometro = crono;
    saida
}

/// ⭐ **A mesma lei na CPU** — o gémeo que os gates usam para afirmar o passe sem placa, e a régua
/// do anti-anel.
#[must_use]
pub fn amplia_cpu(entrada: &[u8], origem: (u32, u32), destino: (u32, u32)) -> Vec<u8> {
    let (w, h) = (origem.0 as i32, origem.1 as i32);
    let texel = |x: i32, y: i32| -> [f32; 4] {
        let i = ((y.clamp(0, h - 1) * w + x.clamp(0, w - 1)) * 4) as usize;
        [
            f32::from(entrada[i]),
            f32::from(entrada[i + 1]),
            f32::from(entrada[i + 2]),
            f32::from(entrada[i + 3]),
        ]
    };
    let pesos = |t: f32| -> [f32; 4] {
        let t2 = t * t;
        let t3 = t2 * t;
        [
            -0.5 * t3 + t2 - 0.5 * t,
            1.5 * t3 - 2.5 * t2 + 1.0,
            -1.5 * t3 + 2.0 * t2 + 0.5 * t,
            0.5 * t3 - 0.5 * t2,
        ]
    };
    let mut out = vec![0u8; destino.0 as usize * destino.1 as usize * 4];
    for gy in 0..destino.1 {
        for gx in 0..destino.0 {
            let u = (gx as f32 + 0.5) * origem.0 as f32 / destino.0 as f32 - 0.5;
            let v = (gy as f32 + 0.5) * origem.1 as f32 / destino.1 as f32 - 0.5;
            let (x0, y0) = (u.floor() as i32, v.floor() as i32);
            let (wx, wy) = (pesos(u - x0 as f32), pesos(v - y0 as f32));
            let mut soma = [0.0f32; 4];
            for (j, wyj) in wy.iter().enumerate() {
                for (i, wxi) in wx.iter().enumerate() {
                    let t = texel(x0 - 1 + i as i32, y0 - 1 + j as i32);
                    for c in 0..4 {
                        soma[c] += t[c] * wxi * wyj;
                    }
                }
            }
            let viz = [
                texel(x0, y0),
                texel(x0 + 1, y0),
                texel(x0, y0 + 1),
                texel(x0 + 1, y0 + 1),
            ];
            let o = (gy as usize * destino.0 as usize + gx as usize) * 4;
            for c in 0..4 {
                let lo = viz.iter().map(|t| t[c]).fold(f32::INFINITY, f32::min);
                let hi = viz.iter().map(|t| t[c]).fold(f32::NEG_INFINITY, f32::max);
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                {
                    out[o + c] = (soma[c].clamp(lo, hi) + 0.5).clamp(0.0, 255.0) as u8;
                }
            }
        }
    }
    out
}

#[cfg(test)]
#[path = "amplia_tests.rs"]
mod amplia_tests;
