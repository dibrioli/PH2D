//! ⭐⭐⭐ **O PASSE DE SEPARAÇÃO NA PLACA** (doc 130) — o `Collide` do cartão `Shape`/`Object`.
//!
//! A CPU separa a corrente de cada sink no fim do cozimento
//! (`ph2d_contact::passe::separa_com_relatorio`); aqui a MESMA lei corre sobre a corrente que o
//! sink vai baixar, antes do lowering e do passe de formas. O oráculo é a CPU, por passo — a
//! aritmética vive em [`LEI`], porta a porta, e nada aqui a inventa.
//!
//! Por passe: `forma` + `massa` (o registo de cada peça e o `alcance_max`) → `inicia` → por
//! varredura `gira` (forma rodada + contagem na grelha) → `scan` → `espalha` → `ordena` (cada balde
//! por índice: os parceiros somam-se em ordem CRESCENTE, como na CPU) → `corrige` (Jacobi, lê uma
//! fotografia, escreve a outra) → `decide` (a paragem, numa bandeira) → no fim `escreve`.
//! ⚠️ **Sem leitura de volta**: uma varredura depois da paragem sai logo.

use crate::stream::{GpuColumn, GpuStream};
use crate::{GpuCook, create_pipeline};
use ph2d_gpu::GpuContext;
use ph2d_nodegraph::attr::{
    COLLIDER_BOX_COLUMN, COLLIDER_COLUMN, COLLIDER_OFFSET_COLUMN, INV_INERTIA_COLUMN,
};
use ph2d_nodegraph::port::Dim;
use std::sync::Arc;

const WG: u32 = crate::codegen::WORKGROUP_SIZE;
const LEI: &str = include_str!("passe_na_placa_lei.wgsl");

/// As colunas que o passe lê, com a dimensão que a CPU exige e o bit no `U.tem`.
const COLUNAS: [(&str, Dim, u32); 10] = [
    (COLLIDER_COLUMN, Dim::Scalar, 1),
    (COLLIDER_BOX_COLUMN, Dim::Vec2, 2),
    (COLLIDER_OFFSET_COLUMN, Dim::Vec2, 4),
    ("size", Dim::Vec2, 8),
    ("rot", Dim::Scalar, 16),
    ("inv_mass", Dim::Scalar, 32),
    (INV_INERTIA_COLUMN, Dim::Scalar, 64),
    ("falloff", Dim::Scalar, 128),
    ("rest", Dim::Vec2, 256),
    ("body", Dim::Scalar, 256),
];

/// O estado do passe entre cozimentos: as varreduras por sink, os pipelines e o que segura os
/// buffers até ao `submit`.
#[derive(Default)]
pub(crate) struct Passe {
    pipes: Option<Pipes>,
    /// As varreduras de cada sink, na ordem de [`crate::GpuPlan::sinks`] — `0` (ou ausente) não corre.
    pub(crate) varreduras: Vec<u32>,
    /// Quantos sinks o último cozimento separou.
    pub(crate) correu: u32,
    hold: Vec<Arc<wgpu::Buffer>>,
    hold_uni: Vec<wgpu::Buffer>,
    hold_scan: Vec<crate::scan::ScanScratch>,
    /// O `ctrl` do último passe (`[alcance_max, andou, parado, varreduras]`) — só o gate o lê.
    pub(crate) ctrl: Option<Arc<wgpu::Buffer>>,
}

impl Passe {
    pub(crate) fn larga(&mut self) {
        self.hold.clear();
        self.hold_uni.clear();
        self.hold_scan.clear();
        self.correu = 0;
    }

    pub(crate) fn vira(&self) {
        if let Some(p) = &self.pipes {
            p.scan.vira();
        }
    }
}

struct Pipes {
    forma: wgpu::ComputePipeline,
    massa: wgpu::ComputePipeline,
    inicia: wgpu::ComputePipeline,
    gira: wgpu::ComputePipeline,
    espalha: wgpu::ComputePipeline,
    ordena: wgpu::ComputePipeline,
    corrige: wgpu::ComputePipeline,
    decide: wgpu::ComputePipeline,
    escreve: wgpu::ComputePipeline,
    scan: crate::scan::Scan,
}

fn modulo(ligacoes: &[(&str, &str)], corpo: &str, extra: &str) -> String {
    let mut s = String::from("@group(0) @binding(0) var<uniform> u: U;\n");
    for (k, (nome, tipo)) in ligacoes.iter().enumerate() {
        s += &format!(
            "@group(0) @binding({}) var<storage, {tipo}> {nome};\n",
            k + 1
        );
    }
    let graus = ph2d_contact::GRAUS;
    let por_grau = core::f32::consts::PI / 180.0;
    format!(
        "{LEI}\nconst PNP_GRAUS: f32 = {graus:?};\nconst PNP_POR_GRAU: f32 = {por_grau:?};\n{s}{extra}\n\
         @compute @workgroup_size({WG})\n\
         fn main(@builtin(global_invocation_id) gid: vec3<u32>) {{\n    let i = gid.x;\n{corpo}\n}}\n"
    )
}

const R: &str = "read";
const RW: &str = "read_write";

fn fontes() -> [(&'static str, String); 9] {
    let forma = modulo(
        &[
            ("c_p: array<vec2<f32>>", R),
            ("c_raio: array<f32>", R),
            ("c_caixa: array<vec2<f32>>", R),
            ("c_desvio: array<vec2<f32>>", R),
            ("c_size: array<vec2<f32>>", R),
            ("c_rot: array<f32>", R),
            ("dec: array<vec4<f32>>", RW),
            ("ctrl: array<atomic<u32>>", RW),
        ],
        include_str!("passe_na_placa_forma.wgsl"),
        "",
    );
    let massa = modulo(
        &[
            ("c_p: array<vec2<f32>>", R),
            ("c_w: array<f32>", R),
            ("c_ii: array<f32>", R),
            ("c_fall: array<f32>", R),
            ("c_rest: array<vec2<f32>>", R),
            ("c_body: array<f32>", R),
            ("dec: array<vec4<f32>>", RW),
            ("est: array<vec4<f32>>", RW),
        ],
        include_str!("passe_na_placa_massa.wgsl"),
        "",
    );
    let inicia = modulo(
        &[("ctrl: array<atomic<u32>>", RW)],
        "    if (i != 0u || u.n == 0u) { return; }\n\
         \x20   let alc = bitcast<f32>(atomicLoad(&ctrl[0]));\n\
         \x20   atomicStore(&ctrl[1], 0u);\n\
         \x20   atomicStore(&ctrl[3], 0u);\n\
         \x20   atomicStore(&ctrl[2], select(1u, 0u, alc > 0.0));",
        "",
    );
    let gira = modulo(
        &[
            ("dec: array<vec4<f32>>", R),
            ("est_in: array<vec4<f32>>", R),
            ("ago: array<vec4<f32>>", RW),
            ("starts: array<atomic<u32>>", RW),
            ("ctrl: array<u32>", R),
        ],
        "    if (i >= u.n || ctrl[2] != 0u) { return; }\n\
         \x20   let d0 = dec[4u * i];\n\
         \x20   if (u32(d0.x) == 0u) { return; }\n\
         \x20   let d1 = dec[4u * i + 1u];\n\
         \x20   let s = est_in[i];\n\
         \x20   var a = d1;\n\
         \x20   if (s.z != 0.0) {\n\
         \x20       let e = pnp_eixo_de(s.z);\n\
         \x20       a = vec4<f32>(pnp_gira(d1.xy, e), a.zw);\n\
         \x20       if (!pnp_zero(d1.zw)) { a = vec4<f32>(a.xy, pnp_gira(d1.zw, e)); }\n\
         \x20   }\n\
         \x20   ago[i] = a;\n\
         \x20   let lado = 2.0 * bitcast<f32>(ctrl[0]);\n\
         \x20   atomicAdd(&starts[pnp_balde(pnp_celula(s.xy, lado), u.num_buckets)], 1u);",
        "",
    );
    let espalha = modulo(
        &[
            ("dec: array<vec4<f32>>", R),
            ("est_in: array<vec4<f32>>", R),
            ("starts: array<u32>", R),
            ("cursor: array<atomic<u32>>", RW),
            ("sorted: array<u32>", RW),
            ("ctrl: array<u32>", R),
        ],
        "    if (i >= u.n || ctrl[2] != 0u) { return; }\n\
         \x20   if (u32(dec[4u * i].x) == 0u) { return; }\n\
         \x20   let lado = 2.0 * bitcast<f32>(ctrl[0]);\n\
         \x20   let b = pnp_balde(pnp_celula(est_in[i].xy, lado), u.num_buckets);\n\
         \x20   sorted[starts[b] + atomicAdd(&cursor[b], 1u)] = i;",
        "",
    );
    let ordena = modulo(
        &[
            ("starts: array<u32>", R),
            ("sorted: array<u32>", RW),
            ("ctrl: array<u32>", R),
        ],
        "    if (i >= u.num_buckets || ctrl[2] != 0u) { return; }\n\
         \x20   let lo = starts[i];\n\
         \x20   let hi = starts[i + 1u];\n\
         \x20   for (var a = lo + 1u; a < hi; a = a + 1u) {\n\
         \x20       let x = sorted[a];\n\
         \x20       var b = a;\n\
         \x20       while (b > lo && sorted[b - 1u] > x) { sorted[b] = sorted[b - 1u]; b = b - 1u; }\n\
         \x20       sorted[b] = x;\n\
         \x20   }",
        "",
    );
    let corrige = modulo(
        &[
            ("dec: array<vec4<f32>>", R),
            ("ago: array<vec4<f32>>", R),
            ("est_in: array<vec4<f32>>", R),
            ("est_out: array<vec4<f32>>", RW),
            ("starts: array<u32>", R),
            ("sorted: array<u32>", R),
            ("ctrl: array<atomic<u32>>", RW),
        ],
        include_str!("passe_na_placa_corrige.wgsl"),
        include_str!("passe_na_placa_corrige_fns.wgsl"),
    );
    let decide = modulo(
        &[("ctrl: array<atomic<u32>>", RW)],
        "    if (i != 0u || atomicLoad(&ctrl[2]) != 0u) { return; }\n\
         \x20   atomicAdd(&ctrl[3], 1u);\n\
         \x20   let m = bitcast<f32>(atomicLoad(&ctrl[1]));\n\
         \x20   let alc = bitcast<f32>(atomicLoad(&ctrl[0]));\n\
         \x20   atomicStore(&ctrl[1], 0u);\n\
         \x20   if (m == 0.0 || m < u.repouso * alc) { atomicStore(&ctrl[2], 1u); }",
        "",
    );
    let escreve = modulo(
        &[
            ("c_p: array<vec2<f32>>", R),
            ("c_rot: array<f32>", R),
            ("dec: array<vec4<f32>>", R),
            ("est: array<vec4<f32>>", R),
            ("p_out: array<vec2<f32>>", RW),
            ("rot_out: array<f32>", RW),
        ],
        "    if (i >= u.n) { return; }\n\
         \x20   let p = c_p[i];\n\
         \x20   let s = est[i];\n\
         \x20   let k = dec[4u * i + 2u].z;\n\
         \x20   var pos = s.xy;\n\
         \x20   var giro = s.z;\n\
         \x20   if (k < 1.0) {\n\
         \x20       pos = vec2<f32>(p.x + (pos.x - p.x) * k, p.y + (pos.y - p.y) * k);\n\
         \x20       giro = giro * k;\n\
         \x20   }\n\
         \x20   p_out[i] = pos;\n\
         \x20   var antes = 0.0;\n\
         \x20   if ((u.tem & 16u) != 0u) { antes = c_rot[i]; }\n\
         \x20   rot_out[i] = antes + giro;",
        "",
    );
    [
        ("forma", forma),
        ("massa", massa),
        ("inicia", inicia),
        ("gira", gira),
        ("espalha", espalha),
        ("ordena", ordena),
        ("corrige", corrige),
        ("decide", decide),
        ("escreve", escreve),
    ]
}

/// As fontes WGSL, para o gate que as valida sem placa.
#[doc(hidden)]
#[must_use]
pub fn fontes_wgsl() -> Vec<(&'static str, String)> {
    fontes().into()
}

impl Pipes {
    fn new(gpu: &GpuContext) -> Self {
        let [f, m, i, g, e, o, c, d, w] = fontes();
        let p = |(nome, src): (&str, String)| create_pipeline(gpu, &src, nome);
        Self {
            forma: p(f),
            massa: p(m),
            inicia: p(i),
            gira: p(g),
            espalha: p(e),
            ordena: p(o),
            corrige: p(c),
            decide: p(d),
            escreve: p(w),
            scan: crate::scan::Scan::new(gpu),
        }
    }
}

fn grupo(
    gpu: &GpuContext,
    pipe: &wgpu::ComputePipeline,
    buffers: &[&wgpu::Buffer],
) -> wgpu::BindGroup {
    let entries: Vec<wgpu::BindGroupEntry> = buffers
        .iter()
        .enumerate()
        .map(|(k, b)| wgpu::BindGroupEntry {
            binding: k as u32,
            resource: b.as_entire_binding(),
        })
        .collect();
    gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("ph2d-passe"),
        layout: &pipe.get_bind_group_layout(0),
        entries: &entries,
    })
}

fn despacha(
    enc: &mut wgpu::CommandEncoder,
    pipe: &wgpu::ComputePipeline,
    g: &wgpu::BindGroup,
    n: u32,
) {
    let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
        label: Some("ph2d-passe"),
        timestamp_writes: None,
    });
    pass.set_pipeline(pipe);
    pass.set_bind_group(0, g, &[]);
    pass.dispatch_workgroups(n.div_ceil(WG).max(1), 1, 1);
}

impl GpuCook {
    /// As varreduras do passe de cada sink, na ordem de [`crate::GpuPlan::sinks`]. Vazio (o
    /// default) = nenhum passe — o que toda chamada anterior ao doc 130 pedia.
    pub fn set_passe(&mut self, varreduras: Vec<u32>) {
        self.passe.varreduras = varreduras;
    }

    /// Quantos sinks do último cozimento a placa VIU sem divergir da CPU (o leitor é a cerca da rota):
    /// os que chegaram com colisor declarado e os vazios.
    #[must_use]
    pub fn passe_correu(&self) -> u32 {
        self.passe.correu
    }

    /// ⭐ **Separa `stream` na placa**, ou `None` quando a CPU devolveria a corrente intacta sem
    /// olhar (zero varreduras, nenhuma coluna de colisor, sem `P`). `repouso` é o
    /// [`ph2d_contact::REPOUSO_VISIVEL`] no produto; o gate pede `0` para medir só o ponto fixo.
    pub(crate) fn separa_na_placa(
        &mut self,
        gpu: &GpuContext,
        enc: &mut wgpu::CommandEncoder,
        stream: &GpuStream,
        varreduras: u32,
        repouso: f32,
    ) -> Option<GpuStream> {
        let n = stream.count;
        // Uma saída VAZIA não tem o que separar nem o que perder: a CPU também não mexe (um
        // objecto sem imagem nem declara — `com_colisor` não põe coluna de zero linhas).
        if n == 0 {
            self.passe.correu += 1;
            return None;
        }
        let col = |nome: &str, dim: Dim| stream.get(nome).filter(|c| c.dim == dim);
        let p = col("P", Dim::Vec2)?;
        let declara = col(COLLIDER_COLUMN, Dim::Scalar).is_some()
            || col(COLLIDER_BOX_COLUMN, Dim::Vec2).is_some();
        if !declara || n > 65_535 * WG {
            return None;
        }
        // A placa VIU a declaração (o leitor é a cerca da rota): uma corrente vazia ou zero
        // varreduras não têm nada a separar, como na CPU.
        self.passe.correu += 1;
        if varreduras == 0 {
            return None;
        }
        let ligados = col("rest", Dim::Vec2).is_some() && col("body", Dim::Scalar).is_some();
        let mut tem = 0u32;
        for (nome, dim, bit) in COLUNAS {
            if col(nome, dim).is_some() && (bit != 256 || ligados) {
                tem |= bit;
            }
        }
        let nb = crate::grid::num_buckets(n);
        let n64 = u64::from(n);
        let mut novo = |pool: &mut crate::stream::BufferPool, bytes: u64| {
            let b = pool.acquire(gpu, bytes);
            self.passe.hold.push(Arc::clone(&b));
            b
        };
        let pool = &mut self.pool;
        let nulo = novo(pool, 16);
        let dec = novo(pool, n64 * 64);
        let est_a = novo(pool, n64 * 16);
        let est_b = novo(pool, n64 * 16);
        let ago = novo(pool, n64 * 16);
        let starts = novo(pool, (u64::from(nb) + 1) * 4);
        let cursor = novo(pool, u64::from(nb) * 4);
        let sorted = novo(pool, n64 * 4);
        let ctrl = novo(pool, 16);
        let p_out = novo(pool, n64 * 8);
        let rot_out = novo(pool, n64 * 4);
        let ler = |nome: &str, dim: Dim| -> Arc<wgpu::Buffer> {
            col(nome, dim).map_or_else(|| Arc::clone(&nulo), |c: &GpuColumn| Arc::clone(&c.buffer))
        };
        let [raio, caixa, desvio, size, rot, w, ii, fall, rest, body] =
            COLUNAS.map(|(nome, dim, _)| ler(nome, dim));

        let uni = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-passe u"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut ub = [0u8; 16];
        ub[0..4].copy_from_slice(&n.to_le_bytes());
        ub[4..8].copy_from_slice(&nb.to_le_bytes());
        ub[8..12].copy_from_slice(&repouso.to_le_bytes());
        ub[12..16].copy_from_slice(&tem.to_le_bytes());
        gpu.queue.write_buffer(&uni, 0, &ub);

        let pipes = self.passe.pipes.get_or_insert_with(|| Pipes::new(gpu));
        enc.clear_buffer(&ctrl, 0, None);
        let pb = &p.buffer;
        let g = |pipe, bufs: &[&wgpu::Buffer]| grupo(gpu, pipe, bufs);
        let gf = g(
            &pipes.forma,
            &[&uni, pb, &raio, &caixa, &desvio, &size, &rot, &dec, &ctrl],
        );
        despacha(enc, &pipes.forma, &gf, n);
        let gm = g(
            &pipes.massa,
            &[&uni, pb, &w, &ii, &fall, &rest, &body, &dec, &est_a],
        );
        despacha(enc, &pipes.massa, &gm, n);
        let gi = g(&pipes.inicia, &[&uni, &ctrl]);
        despacha(enc, &pipes.inicia, &gi, 1);
        // As duas fases do ping-pong, ligadas UMA vez por passe.
        let fases = [(&est_a, &est_b), (&est_b, &est_a)].map(|(a, b)| {
            (
                g(&pipes.gira, &[&uni, &dec, a, &ago, &starts, &ctrl]),
                g(
                    &pipes.espalha,
                    &[&uni, &dec, a, &starts, &cursor, &sorted, &ctrl],
                ),
                g(
                    &pipes.corrige,
                    &[&uni, &dec, &ago, a, b, &starts, &sorted, &ctrl],
                ),
            )
        });
        let go = g(&pipes.ordena, &[&uni, &starts, &sorted, &ctrl]);
        let gd = g(&pipes.decide, &[&uni, &ctrl]);
        let mut scratch = crate::scan::ScanScratch::default();
        for v in 0..varreduras as usize {
            let (gg, ge, gc) = &fases[v % 2];
            enc.clear_buffer(&starts, 0, None);
            enc.clear_buffer(&cursor, 0, None);
            despacha(enc, &pipes.gira, gg, n);
            pipes
                .scan
                .exclusive(gpu, enc, &starts, nb + 1, &mut scratch);
            despacha(enc, &pipes.espalha, ge, n);
            despacha(enc, &pipes.ordena, &go, nb);
            despacha(enc, &pipes.corrige, gc, n);
            despacha(enc, &pipes.decide, &gd, 1);
        }
        let fim = if varreduras % 2 == 1 { &est_b } else { &est_a };
        let gw = g(
            &pipes.escreve,
            &[&uni, pb, &rot, &dec, fim, &p_out, &rot_out],
        );
        despacha(enc, &pipes.escreve, &gw, n);
        self.passe.hold_uni.push(uni);
        self.passe.hold_scan.push(scratch);
        self.passe.ctrl = Some(ctrl);

        let mut out = stream.clone();
        out.cols.insert(
            "P".into(),
            GpuColumn {
                buffer: p_out,
                dim: Dim::Vec2,
            },
        );
        out.cols.insert(
            "rot".into(),
            GpuColumn {
                buffer: rot_out,
                dim: Dim::Scalar,
            },
        );
        Some(out)
    }
}

fn le_bytes(gpu: &GpuContext, buf: &wgpu::Buffer, bytes: u64) -> Vec<u8> {
    let staging = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ph2d-passe leitura"),
        size: bytes.max(4),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut enc = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    enc.copy_buffer_to_buffer(buf, 0, &staging, 0, bytes.max(4));
    gpu.queue.submit(Some(enc.finish()));
    let slice = staging.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    let out = slice.get_mapped_range()[..bytes as usize].to_vec();
    staging.unmap();
    out
}

/// O que o gate lê de volta de um passe: `P`, `rot` e as varreduras que correram.
pub type Lido = (Vec<[f32; 2]>, Vec<f32>, u32);

impl GpuCook {
    /// ⭐ **A porta do GATE** (doc 130 §6): sobe `stream`, corre o passe e LÊ DE VOLTA. `None`
    /// quando o passe não corre — a mesma resposta do `None` da CPU sem colisor declarado.
    #[doc(hidden)]
    pub fn separa_e_le(
        &mut self,
        gpu: &GpuContext,
        stream: &ph2d_nodegraph::attr::Stream,
        varreduras: u32,
        repouso: f32,
    ) -> Option<Lido> {
        self.pool.reclaim();
        self.passe.larga();
        let up = crate::stream::upload_stream(gpu, &mut self.pool, stream);
        let mut enc = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        let out = self.separa_na_placa(gpu, &mut enc, &up, varreduras, repouso)?;
        gpu.queue.submit(Some(enc.finish()));
        let n = u64::from(up.count);
        let p = le_bytes(gpu, &out.cols["P"].buffer, n * 8);
        let r = le_bytes(gpu, &out.cols["rot"].buffer, n * 4);
        let c = le_bytes(gpu, self.passe.ctrl.as_ref()?, 16);
        let p: Vec<f32> = bytemuck::pod_collect_to_vec(&p);
        let c: Vec<u32> = bytemuck::pod_collect_to_vec(&c);
        Some((
            p.as_chunks::<2>().0.to_vec(),
            bytemuck::pod_collect_to_vec(&r),
            c[3],
        ))
    }
}
