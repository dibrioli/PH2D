//! ⭐⭐⭐ **O PASSE QUE PINTA** — do G-buffer que ficou no dispositivo aos bytes que a tela recebe.
//!
//! # ⛔⛔ Porque ele existe: o barramento, medido
//!
//! Até aqui o dispositivo marchava e **devolvia o G-buffer**: `t`, normal, sombra e oclusão por
//! pixel, `49,8 MB` a `1920×1080`. A CPU reconstruía o ponto de cada pixel, resolvia o material,
//! corria o OpenPBR, aplicava o olhar e escrevia os bytes — `36` dos `61 ms` do quadro assente.
//!
//! ⇒ as três leis do pintor já atravessaram ([`ph2d_material::wgsl`], o céu do chamador e o
//! [`ph2d_view_transform::wgsl`]) e a quarta é a do **dono** ([`ph2d_field_eval::owners::wgsl`]).
//! Com as quatro no dispositivo, o que volta é a **imagem**: `8,3 MB`.
//!
//! # ⚠️ Ele lê o grupo `0` da marcha e não o refaz
//!
//! O centro, a luz e a lista de bordas já lá estão. O pintor acrescenta um **grupo `1`** com o que
//! só ele lê — o céu, as tabelas, os materiais e a saída. *Uma segunda declaração do centro seria a
//! segunda resposta à mesma pergunta, e ela divergiria no dia em que um dos dois mudasse de
//! formato.*
//!
//! # ⚠️⚠️ As DUAS aproximações da borda são as da CPU, declaradas
//!
//! Um pixel de borda tem quatro sub-amostras marchadas, e delas guarda-se a **normal** — não o
//! ponto. ⇒ o **ponto** e o **material** do centro servem às quatro, exactamente como o
//! [`ph2d_field_render::shade_render`] faz. Numa silhueta entre duas peças de cores diferentes isto
//! pinta a borda com a cor da que o centro apanhou.

use ph2d_field_eval::owners::{Owners, wgsl::OwnersWgsl};

/// Tudo o que o pintor precisa e que **não** sai da marcha.
pub struct PaintSetup<'a> {
    /// A lei do dono desta peça. `None` numa peça de UMA folha — ver [`Owners`].
    pub owners: Option<&'a Owners>,
    /// Um material por folha, empacotado com o [`ph2d_material::wgsl::pack`] e concatenado.
    ///
    /// ⚠️ **A ordem é a das folhas do [`Owners`]**, e quem constrói um constrói o outro: uma lista
    /// com outra ordem pinta cada peça com a cor da vizinha, sem erro nenhum.
    pub materials: &'a [f32],
    /// O corpo que preenche o [`ph2d_material::wgsl::ENV_SLOT`] — o céu de quem chama.
    ///
    /// ⚠️ **Ele vem de fora de propósito:** o céu do produto vive na família `field3d`, que é
    /// composição, e esta crate não desenha estúdio nenhum.
    pub env_source: &'a str,
    /// O uniforme que o [`Self::env_source`] lê.
    pub env_consts: &'a [f32],
    /// O armazém que o [`Self::env_source`] lê.
    pub env_tables: &'a [f32],
    /// A radiância que cada lâmpada entrega a **uma** unidade de distância — o
    /// [`ph2d_field_render::PointLamp::radiance_at_one`]. As posições são as
    /// [`crate::trace::MarchSetup::lamps`], e as duas listas **têm de ter o mesmo comprimento e a
    /// mesma ordem**: quem monta uma monta a outra.
    pub lamp_radiance: [[f32; 3]; crate::trace::MAX_LAMPS],
    /// A exposição, em paragens.
    pub stops: f32,
    /// A vista, no código do [`ph2d_view_transform::wgsl::view_code`].
    pub view: u32,
    /// Os bytes EXACTOS que um pixel de fundo recebe — copiados, nunca reconvertidos.
    pub background: [u8; 4],
    /// A largura em MUNDO da fronteira entre dois materiais — ver o `BOUNDARY_PIXELS` do
    /// [`ph2d_field_render::shade_render`], que é quem a deriva.
    pub pixel_world: f32,
}

/// Os buffers do grupo `0`, que a marcha já criou e escreveu.
pub(crate) struct Alvos<'a> {
    pub bgl: &'a wgpu::BindGroupLayout,
    pub setup: &'a wgpu::Buffer,
    pub k: &'a wgpu::Buffer,
    pub centro: &'a wgpu::Buffer,
    pub luz: &'a wgpu::Buffer,
    pub conta: &'a wgpu::Buffer,
    pub borda: &'a wgpu::Buffer,
}

/// ⛔ **A lei do dono numa peça de UMA folha** — a mesma resposta que o `Surfaces::owners: None` dá
/// na CPU, e não um caso especial: não perguntar é exactamente o custo zero.
const DONO_DE_UMA_FOLHA: &str = r"
struct Dono { a: u32, b: u32, t: f32 };
fn dono_mix(p: vec3<f32>, width: f32) -> Dono { return Dono(0u, 0u, 0.0); }
";

/// O corpo do pintor — o grupo `1`, as leis de leitura e as duas entradas.
///
/// ⚠️ `{BLUR_COS}` e `{PISO_LUZ}` são **lidos do ficheiro** que os declara, nunca escritos aqui: uma
/// constante transcrita é uma divergência à espera de um dia em que alguém mexa na outra.
const PINTOR: &str = r"
// ── o grupo 1: o que só o pintor lê ───────────────────────────────────────────────────────────
struct Pintor {
    knobs: vec4<f32>,  // stops, pixel_world, _, _
    fundo: vec4<f32>,  // o fundo em LINEAR pré-multiplicado, para a média da borda
    modo: vec4<u32>,   // view, bordas, fundo empacotado, materiais
    // ⭐ **Uma radiância por lâmpada**, na MESMA ordem das posições do `Setup`.
    lamp: array<vec4<f32>, {MAX_LAMPS}>,
};
@group(1) @binding(0) var<uniform> ceu: Ceu;
@group(1) @binding(1) var<uniform> pintor: Pintor;
@group(1) @binding(2) var<storage, read> tabela: array<f32>;
@group(1) @binding(3) var<storage, read> materiais: array<f32>;
@group(1) @binding(4) var<storage, read_write> saida: array<u32>;

const BLUR_COS: f32 = {BLUR_COS};
const PISO_LUZ: f32 = {PISO_LUZ};
const PACKED: u32 = {PACKED}u;

// ⚠️ **A rede é `materiais[0]`**, e ela não é decorativa: o dono pode vir de uma peça que já mudou
// entre a marcha e a pintura. Ler fora do buffer devolveria lixo; pintar com o primeiro material é
// o que o artista lê como «ainda não actualizou», que é o que de facto aconteceu.
fn ler_mat(i: u32) -> Mat {
    var j = i;
    if (j >= pintor.modo.w) { j = 0u; }
    let o = j * PACKED;
    var m: Mat;
    m.base_color_weight    = vec4<f32>(materiais[o +  0u], materiais[o +  1u], materiais[o +  2u], materiais[o +  3u]);
    m.specular_color_metal = vec4<f32>(materiais[o +  4u], materiais[o +  5u], materiais[o +  6u], materiais[o +  7u]);
    m.coat_color_diffrough = vec4<f32>(materiais[o +  8u], materiais[o +  9u], materiais[o + 10u], materiais[o + 11u]);
    m.emission_specweight  = vec4<f32>(materiais[o + 12u], materiais[o + 13u], materiais[o + 14u], materiais[o + 15u]);
    m.darkening_coatweight = vec4<f32>(materiais[o + 16u], materiais[o + 17u], materiais[o + 18u], materiais[o + 19u]);
    m.attenuation_coatior  = vec4<f32>(materiais[o + 20u], materiais[o + 21u], materiais[o + 22u], materiais[o + 23u]);
    m.prepared             = vec4<f32>(materiais[o + 24u], materiais[o + 25u], materiais[o + 26u], materiais[o + 27u]);
    m.emissive             = vec4<f32>(materiais[o + 28u], materiais[o + 29u], materiais[o + 30u], materiais[o + 31u]);
    return m;
}

// A base é ortonormal, logo a transposta é a inversa — a mesma `ViewBasis::world_to_view` da CPU.
fn mundo_para_vista(w: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(dot(w, s.right), dot(w, s.up), dot(w, s.fwd));
}

// ⭐ **A OCLUSÃO SUAVIZADA** — o `ph2d_field_render::blur_occlusion`, célula a célula.
//
// ⚠️ Os vizinhos leem a oclusão CRUA (a CPU suaviza para uma cópia), e um pixel que não acerta
// devolve o valor dele sem tocar em nada — ali `n0` é o vector zero e a cerca da normal fecha
// sozinha, que é exactamente o `continue` da CPU.
fn ceu_em(x: u32, y: u32, i: u32, n0: vec3<f32>) -> f32 {
    var soma = 0.0;
    var cont = 0u;
    for (var dy = -1; dy <= 1; dy = dy + 1) {
        for (var dx = -1; dx <= 1; dx = dx + 1) {
            let xx = i32(x) + dx;
            let yy = i32(y) + dy;
            if (xx < 0 || yy < 0 || xx >= i32(s.w) || yy >= i32(s.h)) { continue; }
            let j = u32(yy) * s.w + u32(xx);
            let c = centro[j];
            if (c.x < 0.0) { continue; }
            if (dot(n0, c.yzw) < BLUR_COS) { continue; }
            soma = soma + luz[j * passo_da_luz()];
            cont = cont + 1u;
        }
    }
    if (cont > 0u) { return soma / f32(cont); }
    return luz[i * passo_da_luz()];
}

// A luz que UM material devolve ao olho, já com o olhar — o `shade_render::radiance` da CPU.
fn luz_do_material(m: Mat, n: vec3<f32>, v: vec3<f32>, p: vec3<f32>, i: u32, ceu_vis: f32) -> vec3<f32> {
    // ⭐⭐⭐ **A OCLUSÃO É A SOMBRA DO CÉU** — ela multiplica o que o AMBIENTE entrega, e mais nada.
    // Não toca nas lâmpadas (que têm sombra a sério) nem na emissão.
    var rgb = mx_indirect(m, n, v) * ceu_vis;
    let piso = PISO_LUZ * PISO_LUZ;
    let base = i * passo_da_luz();
    // ⭐⭐⭐ **AS LUZES-OBJECTO, uma a uma** — a direcção e a distância de cada saem do PONTO deste
    // pixel, e a soma é sobre a RADIÂNCIA, como a CPU faz.
    for (var l: u32 = 0u; l < s.n_lamps; l = l + 1u) {
        let d = s.lamps[l].xyz - p;
        let cru = dot(d, d);
        // ⚠️ **O piso protege DUAS grandezas.** Abaixo dele a direcção é a NORMAL: com a luz sobre
        // o ponto, `d` é o vector ZERO, a direcção normalizada sai `(0,0,0)` e o pixel ficaria PRETO.
        var to_light = n;
        if (cru > piso) {
            let inv = 1.0 / sqrt(cru);
            to_light = mundo_para_vista(d * inv);
        }
        // ⭐ **A sombra entra na radiância que CHEGA** — não no `N·L` e não no resultado.
        let chega = pintor.lamp[l].rgb * luz[base + 1u + l] / max(cru, piso);
        rgb = rgb + mx_direct(m, n, v, to_light, chega);
    }
    return vt_to_display(rgb + mx_emission(m, n, v), pintor.knobs.x, pintor.modo.x);
}

// ⭐⭐ **Sombreia DUAS vezes e mistura o RESULTADO**, nunca os materiais: um metal e um dieléctrico
// a meio caminho não são um meio-metal. E só paga o dobro onde há fronteira.
fn radiancia(p: vec3<f32>, n: vec3<f32>, v: vec3<f32>, i: u32, ceu_vis: f32) -> vec3<f32> {
    let d = dono_mix(p, pintor.knobs.y);
    let ca = luz_do_material(ler_mat(d.a), n, v, p, i, ceu_vis);
    if (d.t <= 0.0) { return ca; }
    let cb = luz_do_material(ler_mat(d.b), n, v, p, i, ceu_vis);
    return ca + (cb - ca) * d.t;
}

// A curva do `ph2d_color::srgb::linear_to_srgb_byte`, com o mesmo arredondamento.
fn srgb_byte(linear: f32) -> u32 {
    let v = clamp(linear, 0.0, 1.0);
    var e = v * 12.92;
    if (v > 0.0031308) { e = 1.055 * pow(v, 1.0 / 2.4) - 0.055; }
    return u32(clamp(e * 255.0 + 0.5, 0.0, 255.0));
}

fn empacota(c: vec4<f32>) -> u32 {
    let a = u32(clamp(clamp(c.w, 0.0, 1.0) * 255.0 + 0.5, 0.0, 255.0));
    return srgb_byte(c.x) | (srgb_byte(c.y) << 8u) | (srgb_byte(c.z) << 16u) | (a << 24u);
}

// A direcção PARA o observador — o raio do traçado, ao contrário.
fn direccao_de_vista(d: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(-dot(d, s.right), -dot(d, s.up), -dot(d, s.fwd));
}

// ⭐⭐⭐ **O INTERIOR: um pixel, um material, uma escrita.**
@compute @workgroup_size(8, 8, 1)
fn pinta(@builtin(global_invocation_id) g: vec3<u32>) {
    if (g.x >= s.w || g.y >= s.h) { return; }
    let i = g.y * s.w + g.x;
    let c = centro[i];
    // ⚠️ **O fundo é COPIADO**, e não passa pela conversão — a mesma cerca da CPU.
    if (c.x < 0.0) { saida[i] = pintor.modo.z; return; }
    let r = ray_at_plane(raio(f32(g.x) + 0.5, f32(g.y) + 0.5));
    // ⭐ O PONTO reconstrói-se do `t` — a mesma álgebra do `Rays::point_at`.
    let p = r.o + r.d * c.x;
    let rgb = radiancia(p, c.yzw, direccao_de_vista(r.d), i, ceu_em(g.x, g.y, i, c.yzw));
    saida[i] = empacota(vec4<f32>(rgb, 1.0));
}

// ⭐⭐⭐ **A BORDA: quatro sub-amostras, média em LINEAR DE ECRÃ.**
//
// ⚠️ A média é das luzes **já transformadas** — é isso que o olho vê, e transformar a média de duas
// luzes da cena não é a mesma coisa.
@compute @workgroup_size(64, 1, 1)
fn pinta_bordas(@builtin(global_invocation_id) g: vec3<u32>) {
    let slot = g.x;
    if (slot >= pintor.modo.y) { return; }
    let i = bitcast<u32>(borda[slot * 5u].x);
    if (i >= s.w * s.h) { return; }
    let x = i % s.w;
    let y = i / s.w;
    let c = centro[i];
    let r = ray_at_plane(raio(f32(x) + 0.5, f32(y) + 0.5));
    let v = direccao_de_vista(r.d);
    let p = r.o + r.d * c.x;
    let ceu_vis = ceu_em(x, y, i, c.yzw);
    var acc = vec4<f32>(0.0);
    for (var j = 0u; j < 4u; j = j + 1u) {
        let q = borda[slot * 5u + 1u + j];
        var cor = pintor.fundo;
        if (q.x >= 0.0) { cor = vec4<f32>(radiancia(p, q.yzw, v, i, ceu_vis), 1.0); }
        acc = acc + cor * 0.25;
    }
    saida[i] = empacota(acc);
}
";

/// ⭐⭐⭐ **O texto do pintor, composto** — as quatro leis mais o corpo.
///
/// ⚠️ **A ordem é a que o WGSL precisa para o `struct Ceu` existir antes do binding que o nomeia.**
/// O material traz a ranhura do ambiente já preenchida pelo céu de quem chama.
pub(crate) fn fonte(pintor: &PaintSetup<'_>, lei_do_dono: Option<&OwnersWgsl>) -> String {
    let dono = lei_do_dono.map_or(DONO_DE_UMA_FOLHA, |l| l.source.as_str());
    let material =
        ph2d_material::wgsl::SOURCE.replace(ph2d_material::wgsl::ENV_SLOT, pintor.env_source);
    let corpo = PINTOR
        .replace(
            "{BLUR_COS}",
            &formata(ph2d_field_render::OCCLUSION_BLUR_COS),
        )
        .replace(
            "{PISO_LUZ}",
            &formata(ph2d_field_render::POINT_LAMP_MIN_DISTANCE),
        )
        .replace("{PACKED}", &ph2d_material::wgsl::PACKED.to_string())
        .replace("{MAX_LAMPS}", &crate::trace::MAX_LAMPS.to_string());
    format!(
        "{}{material}\n{}\n{dono}\n{corpo}",
        crate::trace_wgsl::comum(),
        ph2d_view_transform::wgsl::SOURCE
    )
}

/// ⚠️ Um `f32` que o WGSL leia como `f32` — sem isto um `0.9` inteiro sairia `0.9` e um `2` sairia
/// `2`, que ali é um literal **inteiro**.
fn formata(v: f32) -> String {
    let s = format!("{v:?}");
    if s.contains('.') || s.contains('e') {
        s
    } else {
        format!("{s}.0")
    }
}

/// ⭐⭐⭐ **A imagem, pintada onde os dados estão.**
// O dispositivo, a fila, o cache, o pedido, a lei do dono, os alvos da marcha, a tela e a contagem
// de bordas — oito coisas independentes, e uma struct só as renomearia.
#[allow(clippy::too_many_arguments)]
pub(crate) fn pinta(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    cache: &mut crate::FieldPipelines,
    pintor: &PaintSetup<'_>,
    lei_do_dono: Option<&OwnersWgsl>,
    alvos: &Alvos<'_>,
    width: u32,
    height: u32,
    bordas: u64,
) -> Vec<u8> {
    use crate::trace::{armazem, uniforme};
    use wgpu::util::DeviceExt;

    let bgl1 = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("pintor"),
        entries: &[
            uniforme(0),
            uniforme(1),
            armazem(2, true),
            armazem(3, true),
            armazem(4, false),
        ],
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("pintor"),
        bind_group_layouts: &[Some(alvos.bgl), Some(&bgl1)],
        immediate_size: 0,
    });
    // ⚠️ **O cache é o mesmo do traçado**, e a chave é o TEXTO: um arrasto de slider muda números e
    // não recompila nada, exactamente como na marcha.
    let fonte = fonte(pintor, lei_do_dono);
    let vazia = ph2d_field_eval::wgsl::TapeWgsl {
        source: String::new(),
        consts: Vec::new(),
    };
    let p_pinta = cache
        .entry_with_layout(device, &fonte, &vazia, "pinta", Some(&layout))
        .clone();
    let p_bordas = (bordas > 0).then(|| {
        cache
            .entry_with_layout(device, &fonte, &vazia, "pinta_bordas", Some(&layout))
            .clone()
    });

    let bytes = |v: &[f32]| -> Vec<u8> { v.iter().flat_map(|f| f.to_le_bytes()).collect() };
    let ceu = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("ceu"),
        contents: &bytes(pintor.env_consts),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let tabela = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("tabela"),
        contents: &bytes(pintor.env_tables),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let mats = if pintor.materials.is_empty() {
        vec![0.0f32; ph2d_material::wgsl::PACKED]
    } else {
        pintor.materials.to_vec()
    };
    let materiais = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("materiais"),
        contents: &bytes(&mats),
        usage: wgpu::BufferUsages::STORAGE,
    });

    let bg = pintor.background;
    let a = f32::from(bg[3]) / 255.0;
    let mut u: Vec<u8> = Vec::with_capacity(64 + crate::trace::MAX_LAMPS * 16);
    for f in [
        pintor.stops,
        pintor.pixel_world,
        0.0,
        0.0,
        // ⚠️ **O fundo da BORDA é LINEAR e PRÉ-MULTIPLICADO** — a média das quatro amostras corre
        // em linear de ecrã, e o alfa entra nela como as outras três componentes.
        ph2d_color::srgb::srgb_to_linear_byte(bg[0]) * a,
        ph2d_color::srgb::srgb_to_linear_byte(bg[1]) * a,
        ph2d_color::srgb::srgb_to_linear_byte(bg[2]) * a,
        a,
    ] {
        u.extend_from_slice(&f.to_le_bytes());
    }
    #[allow(clippy::cast_possible_truncation)]
    let n_bordas = bordas as u32;
    let empacotado = u32::from(bg[0])
        | (u32::from(bg[1]) << 8)
        | (u32::from(bg[2]) << 16)
        | (u32::from(bg[3]) << 24);
    #[allow(clippy::cast_possible_truncation)]
    let n_mats = (mats.len() / ph2d_material::wgsl::PACKED) as u32;
    for v in [pintor.view, n_bordas, empacotado, n_mats] {
        u.extend_from_slice(&v.to_le_bytes());
    }
    // ⚠️ **O array vai INTEIRO** — a mesma razão do `MarchSetup::lamps`: um `array<vec4, 8>` de
    // uniforme tem tamanho fixo.
    for r in pintor.lamp_radiance {
        for f in r {
            u.extend_from_slice(&f.to_le_bytes());
        }
        u.extend_from_slice(&0f32.to_le_bytes());
    }
    let ub_pintor = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("pintor"),
        contents: &u,
        usage: wgpu::BufferUsages::UNIFORM,
    });

    let n = u64::from(width) * u64::from(height);
    let b_saida = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("imagem"),
        size: (n * 4).max(16),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    fn recurso(b: &wgpu::Buffer, i: u32) -> wgpu::BindGroupEntry<'_> {
        wgpu::BindGroupEntry {
            binding: i,
            resource: b.as_entire_binding(),
        }
    }
    let bg0 = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: alvos.bgl,
        entries: &[
            recurso(alvos.setup, 0),
            recurso(alvos.k, 1),
            recurso(alvos.centro, 2),
            recurso(alvos.luz, 3),
            recurso(alvos.conta, 4),
            recurso(alvos.borda, 5),
        ],
    });
    let bg1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bgl1,
        entries: &[
            recurso(&ceu, 0),
            recurso(&ub_pintor, 1),
            recurso(&tabela, 2),
            recurso(&materiais, 3),
            recurso(&b_saida, 4),
        ],
    });

    let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut cp = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });
        cp.set_pipeline(&p_pinta);
        cp.set_bind_group(0, &bg0, &[]);
        cp.set_bind_group(1, &bg1, &[]);
        cp.dispatch_workgroups(width.div_ceil(8), height.div_ceil(8), 1);
    }
    // ⚠️ **A borda depois do interior, e a ordem é a lei**: ela SOBRESCREVE o pixel que o interior
    // acabou de escrever, exactamente como o laço em série da CPU faz depois das linhas.
    if let Some(p) = &p_bordas {
        let mut cp = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });
        cp.set_pipeline(p);
        cp.set_bind_group(0, &bg0, &[]);
        cp.set_bind_group(1, &bg1, &[]);
        cp.dispatch_workgroups(n_bordas.div_ceil(64), 1, 1);
    }
    let leitura = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("leitura"),
        size: (n * 4).max(16),
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    enc.copy_buffer_to_buffer(&b_saida, 0, &leitura, 0, (n * 4).max(16));
    queue.submit([enc.finish()]);
    leitura.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    device.poll(wgpu::PollType::wait_indefinitely()).ok();
    let dados = leitura.slice(..).get_mapped_range();
    #[allow(clippy::cast_possible_truncation)]
    let out = dados[..(n as usize) * 4].to_vec();
    drop(dados);
    out
}
