//! ⭐⭐⭐ **O BRILHO NO DISPOSITIVO** — o gémeo do [`ph2d_field_render::brilho`]
//! (`docs/Render3d/12` §11).
//!
//! # ⛔⛔⛔ Porque ele existe: o caminho de OMISSÃO é este
//!
//! O modelador pinta no dispositivo desde `05` §39 — a marcha e o pintor correm lá e o que volta é a
//! **imagem**. O brilho vivia só na cauda do sombreador de CPU, logo **o artista nunca o alcançava**
//! sem abrir o app com `PH2D_FIELD_GPU=0`, e as onze fileiras do painel apareciam apagadas a dizê-lo.
//! ⇒ *uma feature que só existe no caminho de referência é uma feature que o produto não tem.*
//!
//! ⚠️ **E trazer a imagem para a CPU só para isto está MEDIDO e fora de questão:** a cadeia em CPU
//! custa `24,1 ms` a `445×305` (o quadro de MOVIMENTO) e `347,2 ms` a `1898×916` (o ASSENTE) — ver
//! a sonda `quanto_custa_a_cadeia` da `ph2d-bloom`.
//!
//! # ⚠️ A LEI é a mesma e vem da folha
//!
//! Tudo o que decide pixels vem de [`ph2d_bloom::wgsl`], que é a mesma crate de onde o caminho de
//! CPU tira a dele. Aqui mora **só** o que é do dispositivo: os buffers, os bind groups e a ordem
//! dos despachos. *A lei atravessa; a plumbagem não.*
//!
//! # ⚠️ A cadeia, degrau a degrau (o gémeo do `ph2d_bloom::halo`)
//!
//! ```text
//! cena (w×h)  --desce+corte-->  mip[0] (w/2)  --desce-->  mip[1]  ...  mip[n-1]
//!                                                                          |
//!                        acc = mip[n-1];  para k = n-2..0:  acc = tenda(acc→mip[k]) + mip[k]
//!                                                                          |
//!                                        halo (w×h) = cor(tenda(acc→w×h))
//! ```
//!
//! ⭐ **O corte entra DENTRO da porta de leitura do primeiro degrau** — ver a nota da
//! [`ph2d_bloom::wgsl`]: assim ele não custa nem um passe nem um buffer do tamanho do quadro.
//!
//! ⭐ **O halo é escrito POR CIMA do buffer da cena**, que a esta altura já ninguém lê — o último
//! degrau lê o acumulador, nunca a cena. *Um quadro de `1898×916` são `27,8 MB`; escrevê-lo duas
//! vezes seria pagar duas.*

use ph2d_bloom::Bloom;

/// O que o chamador tem de dizer sobre o OLHAR — os mesmos dois números que o pintor já recebe.
pub(crate) struct Olhar {
    pub stops: f32,
    pub view: u32,
}

/// Os buffers da cadeia, devolvidos para viverem até à submissão.
pub(crate) struct Cadeia {
    _mips: Vec<wgpu::Buffer>,
    _acc: Vec<wgpu::Buffer>,
    _ub: Vec<wgpu::Buffer>,
}

/// ⭐⭐⭐ **Grava a cadeia e a composição no encoder**, ou devolve `None` quando não há o que fazer.
///
/// ⚠️ **`None` é o caminho de omissão e tem de custar zero:** sem brilho ligado nada aqui é criado,
/// nenhum shader é compilado e o `cena` do chamador fica com o texel de rede que ele ligou.
#[allow(clippy::too_many_arguments)]
pub(crate) fn encadeia(
    device: &wgpu::Device,
    cache: &mut crate::FieldPipelines,
    fita: &ph2d_field_eval::wgsl::TapeWgsl,
    enc: &mut wgpu::CommandEncoder,
    cena: &wgpu::Buffer,
    saida: &wgpu::Buffer,
    (w, h): (u32, u32),
    bloom: &Bloom,
    olhar: &Olhar,
) -> Option<Cadeia> {
    use wgpu::util::DeviceExt;

    if !bloom.contributes() {
        return None;
    }
    let n = ph2d_bloom::levels_that_fit(w as usize, h as usize);
    if n == 0 {
        return None;
    }
    let p = &bloom.params;
    #[allow(clippy::cast_precision_loss)]
    let aspecto = w as f32 / h.max(1) as f32;
    let base = p.upsample_basis(aspecto);

    // ⚠️ **Os tamanhos são os do gémeo de CPU, à letra** (`lw / 2`, inteiro) — uma divisão
    // arredondada de outra maneira daria outra cadeia e a paridade deixaria de fechar.
    let mut dims: Vec<(u32, u32)> = Vec::with_capacity(n);
    let (mut lw, mut lh) = (w, h);
    for _ in 0..n {
        lw /= 2;
        lh /= 2;
        dims.push((lw, lh));
    }

    let buffer = |lado: (u32, u32), rot: &str| {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(rot),
            size: u64::from(lado.0) * u64::from(lado.1) * 16,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        })
    };
    let mips: Vec<wgpu::Buffer> = dims.iter().map(|d| buffer(*d, "brilho.mip")).collect();
    // ⚠️ **DOIS acumuladores e não um:** o degrau que sobe LÊ o acumulador e ESCREVE o seguinte, e
    // um buffer não pode estar ligado como leitura e escrita no mesmo grupo. Eles alternam.
    let acc: Vec<wgpu::Buffer> = (0..2).map(|_| buffer(dims[0], "brilho.acc")).collect();

    let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("brilho"),
        entries: &[
            crate::trace::armazem(0, true),
            crate::trace::armazem(1, false),
            crate::trace::armazem(2, true),
            crate::trace::uniforme(3),
        ],
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("brilho"),
        bind_group_layouts: &[Some(&bgl)],
        immediate_size: 0,
    });
    let fonte = fonte_da_cadeia();
    let p_desce = cache
        .entry_with_layout(device, &fonte, fita, "desce", Some(&layout))
        .clone();
    let p_sobe = cache
        .entry_with_layout(device, &fonte, fita, "sobe", Some(&layout))
        .clone();
    let p_final = cache
        .entry_with_layout(device, &fonte, fita, "sobe_final", Some(&layout))
        .clone();

    let mut ubs: Vec<wgpu::Buffer> = Vec::new();
    let mut uniforme = |d: (u32, u32, u32, u32), corta: bool| {
        let mut v: Vec<f32> = Vec::with_capacity(20);
        // `dims` viaja como u32 — o `bitcast` do lado do shader é o `vec4<u32>` declarado lá.
        for x in [d.0, d.1, d.2, d.3] {
            v.push(f32::from_bits(x));
        }
        v.extend_from_slice(&base);
        v.extend_from_slice(&[
            p.threshold,
            p.knee,
            p.clamp_limit(),
            if corta { 1.0 } else { 0.0 },
        ]);
        v.extend_from_slice(&[p.saturation, p.intensity, 0.0, 0.0]);
        v.extend_from_slice(&[p.tint[0], p.tint[1], p.tint[2], 0.0]);
        let b = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("brilho.passe"),
            contents: &v.iter().flat_map(|f| f.to_le_bytes()).collect::<Vec<u8>>(),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        ubs.push(b);
        ubs.len() - 1
    };

    // (2) DESCE. ⭐ O primeiro degrau lê a CENA e aplica o corte na porta.
    let mut plano: Vec<(usize, &wgpu::Buffer, &wgpu::Buffer, &wgpu::Buffer)> = Vec::new();
    let mut ids_desce: Vec<usize> = Vec::with_capacity(n);
    for k in 0..n {
        let origem = if k == 0 { (w, h) } else { dims[k - 1] };
        ids_desce.push(uniforme((origem.0, origem.1, dims[k].0, dims[k].1), k == 0));
    }
    for k in 0..n {
        let origem: &wgpu::Buffer = if k == 0 { cena } else { &mips[k - 1] };
        plano.push((ids_desce[k], origem, &mips[k], origem));
    }
    let descidas = plano.len();

    // (3) SOBE, um degrau de cada vez, somando o mip daquele nível.
    let mut ids_sobe: Vec<usize> = Vec::new();
    for k in (0..n.saturating_sub(1)).rev() {
        let origem = if k + 1 == n - 1 {
            dims[n - 1]
        } else {
            dims[k + 1]
        };
        ids_sobe.push(uniforme((origem.0, origem.1, dims[k].0, dims[k].1), false));
    }
    let mut destino_par = 0usize;
    for (passo, k) in (0..n.saturating_sub(1)).rev().enumerate() {
        let origem: &wgpu::Buffer = if passo == 0 {
            &mips[n - 1]
        } else {
            &acc[1 - destino_par]
        };
        plano.push((ids_sobe[passo], origem, &acc[destino_par], &mips[k]));
        destino_par = 1 - destino_par;
    }
    let subidas = plano.len() - descidas;

    // (4) o último degrau, de volta ao quadro — e a COR do halo no mesmo passe.
    let origem_final = if subidas == 0 { dims[n - 1] } else { dims[0] };
    let id_final = uniforme((origem_final.0, origem_final.1, w, h), false);
    let fonte_final: &wgpu::Buffer = if subidas == 0 {
        &mips[n - 1]
    } else {
        &acc[1 - destino_par]
    };
    plano.push((id_final, fonte_final, cena, fonte_final));

    for (i, (ub, origem, destino, nivel)) in plano.iter().enumerate() {
        let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("brilho"),
            layout: &bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: origem.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: destino.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: nivel.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: ubs[*ub].as_entire_binding(),
                },
            ],
        });
        let (dw, dh) = if i < descidas {
            dims[i]
        } else if i < descidas + subidas {
            dims[n - 2 - (i - descidas)]
        } else {
            (w, h)
        };
        let pipeline = if i < descidas {
            &p_desce
        } else if i < descidas + subidas {
            &p_sobe
        } else {
            &p_final
        };
        let mut cp = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("brilho"),
            timestamp_writes: None,
        });
        cp.set_pipeline(pipeline);
        cp.set_bind_group(0, &bg, &[]);
        cp.dispatch_workgroups(dw.div_ceil(8), dh.div_ceil(8), 1);
    }

    // (5) A COMPOSIÇÃO — o gémeo do `ph2d_field_render::brilho::soma_halo`.
    let bgl_c = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("brilho.compoe"),
        entries: &[
            crate::trace::armazem(0, true),
            crate::trace::armazem(1, false),
            crate::trace::uniforme(2),
        ],
    });
    let layout_c = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("brilho.compoe"),
        bind_group_layouts: &[Some(&bgl_c)],
        immediate_size: 0,
    });
    let p_compoe = cache
        .entry_with_layout(
            device,
            &fonte_da_composicao(),
            fita,
            "compoe",
            Some(&layout_c),
        )
        .clone();
    let ub_c = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("brilho.compoe"),
        contents: &[
            w.to_le_bytes(),
            h.to_le_bytes(),
            olhar.view.to_le_bytes(),
            0u32.to_le_bytes(),
            olhar.stops.to_le_bytes(),
            0f32.to_le_bytes(),
            0f32.to_le_bytes(),
            0f32.to_le_bytes(),
        ]
        .concat(),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let bg_c = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("brilho.compoe"),
        layout: &bgl_c,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: cena.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: saida.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: ub_c.as_entire_binding(),
            },
        ],
    });
    {
        let mut cp = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("brilho.compoe"),
            timestamp_writes: None,
        });
        cp.set_pipeline(&p_compoe);
        cp.set_bind_group(0, &bg_c, &[]);
        cp.dispatch_workgroups(w.div_ceil(8), h.div_ceil(8), 1);
    }
    ubs.push(ub_c);
    Some(Cadeia {
        _mips: mips,
        _acc: acc,
        _ub: ubs,
    })
}

/// ⭐ **Os dois textos, para o gate que os valida sem placa** — ver
/// `tests/brilho_wgsl_valido.rs`. ⚠️ Eles são montados em tempo de EXECUÇÃO, logo um erro de
/// sintaxe só apareceria quando o artista ligasse o Bloom.
#[must_use]
pub fn fontes_para_gate() -> (String, String) {
    (fonte_da_cadeia(), fonte_da_composicao())
}

/// O texto da cadeia: a lei da folha, com a porta de leitura desta crate no meio.
fn fonte_da_cadeia() -> String {
    let porta = r#"
@group(0) @binding(0) var<storage, read> origem: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read_write> destino: array<vec4<f32>>;
// ⚠️ **O nível a SOMAR na subida** — nos degraus que descem ele é ligado à própria origem (um
// buffer pode estar em duas leituras), e o passe não lhe toca.
@group(0) @binding(2) var<storage, read> nivel: array<vec4<f32>>;
@group(0) @binding(3) var<uniform> P: Passe;

// ⭐⭐⭐ **A PORTA — e é ela que carrega o CORTE no primeiro degrau** (ver `ph2d_bloom::wgsl`):
// `corte.w` é `1` só no degrau que lê a cena, e ali cada texel entra já cortado. *O gémeo de CPU
// escreve o corte num quadro inteiro antes de descer; aqui isso seria um passe e `27,8 MB`.*
fn bl_le(i: u32) -> vec3<f32> {
    let c = origem[i].xyz;
    if (P.corte.w != 0.0) { return bl_corte(c, P.corte.x, P.corte.y, P.corte.z); }
    return c;
}
"#;
    format!(
        "struct Passe {{\n    dims: vec4<u32>,\n    base: vec4<f32>,\n    corte: vec4<f32>,\n    cor: \
         vec4<f32>,\n    tinta: vec4<f32>,\n}};\n{}\n{}\n",
        ph2d_bloom::wgsl::fonte(porta),
        ENTRADAS
    )
}

/// Os três pontos de entrada da cadeia. ⚠️ Os tamanhos vêm TODOS do uniforme — um shader por nível
/// seria um módulo por nível para compilar.
const ENTRADAS: &str = r#"
@compute @workgroup_size(8, 8, 1)
fn desce(@builtin(global_invocation_id) g: vec3<u32>) {
    if (g.x >= P.dims.z || g.y >= P.dims.w) { return; }
    destino[g.y * P.dims.z + g.x] =
        vec4<f32>(bl_desce13(P.dims.x, P.dims.y, P.dims.z, P.dims.w, g.x, g.y), 0.0);
}

@compute @workgroup_size(8, 8, 1)
fn sobe(@builtin(global_invocation_id) g: vec3<u32>) {
    if (g.x >= P.dims.z || g.y >= P.dims.w) { return; }
    let d = g.y * P.dims.z + g.x;
    let t = bl_sobe_tenda(P.dims.x, P.dims.y, P.dims.z, P.dims.w, g.x, g.y, P.base);
    destino[d] = vec4<f32>(t + nivel[d].xyz, 0.0);
}

// ⭐ O último degrau não soma nível nenhum (não há mip na resolução do quadro) e é onde a COR entra
// — exactamente o passo (5) do `ph2d_bloom::halo`.
@compute @workgroup_size(8, 8, 1)
fn sobe_final(@builtin(global_invocation_id) g: vec3<u32>) {
    if (g.x >= P.dims.z || g.y >= P.dims.w) { return; }
    let d = g.y * P.dims.z + g.x;
    let t = bl_sobe_tenda(P.dims.x, P.dims.y, P.dims.z, P.dims.w, g.x, g.y, P.base);
    destino[d] = vec4<f32>(bl_cor(t, P.cor.x, P.tinta.xyz, P.cor.y), 0.0);
}
"#;

/// O texto da composição — o gémeo do [`ph2d_field_render::brilho::soma_halo`], incluindo a
/// COBERTURA que a luz traz consigo (sem ela o canvas apaga o halo inteiro; ver a nota daquela
/// função e o `docs/Render3d/12` §10).
fn fonte_da_composicao() -> String {
    format!(
        "struct Comp {{\n    dims: vec4<u32>,\n    olhar: vec4<f32>,\n}};\n\
         @group(0) @binding(0) var<storage, read> halo: array<vec4<f32>>;\n\
         @group(0) @binding(1) var<storage, read_write> imagem: array<u32>;\n\
         @group(0) @binding(2) var<uniform> C: Comp;\n{}\n{}\n",
        ph2d_view_transform::wgsl::SOURCE,
        COMPOSICAO
    )
}

const COMPOSICAO: &str = r#"
// A curva do `ph2d_color::srgb::srgb_to_linear_unit`, sobre um byte.
fn bl_srgb_para_linear(b: u32) -> f32 {
    let v = f32(b) / 255.0;
    if (v <= 0.04045) { return v / 12.92; }
    return pow((v + 0.055) / 1.055, 2.4);
}

// A do `linear_to_srgb_byte`, com o mesmo arredondamento.
fn bl_linear_para_byte(linear: f32) -> u32 {
    let v = clamp(linear, 0.0, 1.0);
    var e = v * 12.92;
    if (v > 0.0031308) { e = 1.055 * pow(v, 1.0 / 2.4) - 0.055; }
    return u32(clamp(e * 255.0 + 0.5, 0.0, 255.0));
}

@compute @workgroup_size(8, 8, 1)
fn compoe(@builtin(global_invocation_id) g: vec3<u32>) {
    if (g.x >= C.dims.x || g.y >= C.dims.y) { return; }
    let i = g.y * C.dims.x + g.x;
    // ⚠️ **O olhar corre UMA vez por pixel e não uma por canal** — a mesma nota do gémeo de CPU.
    let d = vt_to_display(halo[i].xyz, C.olhar.x, C.dims.z);
    let px = imagem[i];
    var r = px & 255u;
    var v = (px >> 8u) & 255u;
    var b = (px >> 16u) & 255u;
    let a = (px >> 24u) & 255u;
    if (d.x > 0.0) { r = bl_linear_para_byte(bl_srgb_para_linear(r) + d.x); }
    if (d.y > 0.0) { v = bl_linear_para_byte(bl_srgb_para_linear(v) + d.y); }
    if (d.z > 0.0) { b = bl_linear_para_byte(bl_srgb_para_linear(b) + d.z); }
    // ⭐⭐⭐ **A COBERTURA que esta luz traz consigo**, composta como toda camada — e o
    // arredondamento é para CIMA, senão a cauda do halo evapora-se (a cor vai em sRGB e a cobertura
    // em linear, e as duas quantizam a ritmos muito diferentes).
    let c = clamp(max(d.x, max(d.y, d.z)), 0.0, 1.0);
    let antes = f32(a) / 255.0;
    let novo = min(u32(ceil((c + (1.0 - c) * antes) * 255.0)), 255u);
    imagem[i] = r | (v << 8u) | (b << 16u) | (novo << 24u);
}
"#;
