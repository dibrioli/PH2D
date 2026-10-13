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

#[path = "passe_na_placa_corrige.rs"]
mod corrige;
pub use corrige::Corrige;

const WG: u32 = crate::codegen::WORKGROUP_SIZE;
/// Até quantas peças o passe corre num só grupo (doc 130 §3.1). MEDIDO (11/10, release, as duas
/// variantes no mesmo processo): a `1 000` peças empatam (`14,4` × `14,4 ms` a 64 varreduras); a
/// `2 048` os despachos ganham (`15,7` × `29,1 ms`). O recurso é o de UM multiprocessador.
const PECAS_PARA_UM_GRUPO: u32 = 1024;
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

/// Uma leitura do `ctrl` em voo: o buffer, o aviso do mapeamento e as peças.
type Leitura = (
    wgpu::Buffer,
    std::sync::mpsc::Receiver<Result<(), wgpu::BufferAsyncError>>,
    u32,
);

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
    /// O tecto do caminho de UM grupo — `None` = [`PECAS_PARA_UM_GRUPO`]; só a sonda o muda (as
    /// duas variantes no MESMO processo, `MEDIR_VELOCIDADE.md`).
    pub(crate) um_grupo_ate: Option<u32>,
    /// A forma do `corrige` ([`Corrige`]) — só a sonda e o gate a mudam.
    pub(crate) corrige: Corrige,
    /// Quanto o processador levou a ENCOMENDAR o último passe (a sonda lê).
    pub(crate) encode: std::time::Duration,
    /// O `ctrl` do último passe (`[alcance_max, andou, parado, varreduras, candidatos]`).
    pub(crate) ctrl: Option<Arc<wgpu::Buffer>>,
    /// A cópia do `ctrl` encomendada no cozimento, por mapear (o `submit` ainda não correu).
    copia: Option<(wgpu::Buffer, u32)>,
    /// A leitura em voo: o buffer, o aviso do mapeamento e as peças.
    leitura: Option<Leitura>,
    /// O último `Relatorio` lido da placa.
    ultimo: Option<ph2d_contact::passe::Relatorio>,
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
    /// Por [`Corrige::indice`]: o passo do caminho de despachos e o caminho de um grupo.
    corrige: Vec<wgpu::ComputePipeline>,
    decide: wgpu::ComputePipeline,
    uno: Vec<wgpu::ComputePipeline>,
    escreve: wgpu::ComputePipeline,
    scan: crate::scan::Scan,
}

fn cabeca(ligacoes: &[(&str, &str)]) -> String {
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
        "{LEI}\nconst PNP_GRAUS: f32 = {graus:?};\nconst PNP_POR_GRAU: f32 = {por_grau:?};\n{s}"
    )
}

fn modulo(ligacoes: &[(&str, &str)], corpo: &str, extra: &str) -> String {
    format!(
        "{}{extra}\n@compute @workgroup_size({WG})\n\
         fn main(@builtin(global_invocation_id) gid: vec3<u32>) {{\n    let i = gid.x;\n{corpo}\n}}\n",
        cabeca(ligacoes)
    )
}

const R: &str = "read";
const RW: &str = "read_write";

/// As ligações de toda a VARREDURA — o mesmo leiaute nos dois caminhos (ver `pnp_toca`).
const VARREDURA: [(&str, &str); 7] = [
    ("dec: array<vec4<f32>>", R),
    ("ago: array<vec4<f32>>", RW),
    ("est: array<vec4<f32>>", RW),
    ("starts: array<atomic<u32>>", RW),
    ("cursor: array<atomic<u32>>", RW),
    ("sorted: array<u32>", RW),
    ("ctrl: array<atomic<u32>>", RW),
];
const FNS: &str = include_str!("passe_na_placa_varredura.wgsl");

/// As funções da varredura com o `corrige` na forma `c`.
fn fns(c: Corrige) -> String {
    format!("{FNS}\n{}", c.fonte())
}

/// O passo `corrige` do caminho de despachos e o módulo de um grupo, com o `corrige` na forma `c`.
fn fontes_do_corrige(c: Corrige) -> (String, String) {
    let corrige = modulo(
        &VARREDURA,
        "    pnp_toca();\n    let alc_max = bitcast<f32>(atomicLoad(&ctrl[0]));\n\
         \x20   if (i >= u.n) { return; }\n\
         \x20   if (atomicLoad(&ctrl[2]) != 0u) {\n\
         \x20       est[u.bo + i] = est[u.bi + i];\n\
         \x20       return;\n\
         \x20   }\n\
         \x20   let a = pnp_corrige(i, u.bi, u.bo, alc_max);\n\
         \x20   if (a >= 0.0) { atomicMax(&ctrl[1], pnp_bits(a)); }",
        &fns(c),
    );
    let uno = format!(
        "{}{}\n{}",
        cabeca(&VARREDURA),
        fns(c),
        include_str!("passe_na_placa_uno.wgsl")
    );
    (corrige, uno)
}

fn fontes() -> [(&'static str, String); 10] {
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
    // Os passos de UMA varredura no caminho de vários despachos: cada um sai se o passe parou.
    let passo = |corpo: &str| {
        modulo(
            &VARREDURA,
            &format!(
                "    pnp_toca();\n    let alc_max = bitcast<f32>(atomicLoad(&ctrl[0]));\n\
                 \x20   let lado = 2.0 * alc_max;\n{corpo}"
            ),
            FNS,
        )
    };
    let gira = passo(
        "    if (i >= u.n || atomicLoad(&ctrl[2]) != 0u) { return; }\n    pnp_gira_conta(i, u.bi, lado);",
    );
    let espalha = passo(
        "    if (i >= u.n || atomicLoad(&ctrl[2]) != 0u) { return; }\n    pnp_espalha(i, u.bi, lado);",
    );
    let ordena = passo(
        "    if (i >= u.num_buckets || atomicLoad(&ctrl[2]) != 0u) { return; }\n    pnp_ordena(i);",
    );
    let (corrige, uno) = fontes_do_corrige(Corrige::default());
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
         \x20   let s = est[u.bi + i];\n\
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
        ("uno", uno),
        ("escreve", escreve),
    ]
}

/// As fontes WGSL, para o gate que as valida sem placa.
#[doc(hidden)]
#[must_use]
pub fn fontes_wgsl() -> Vec<(&'static str, String)> {
    let mut v: Vec<_> = fontes().into();
    for c in Corrige::TODAS {
        let (corrige, uno) = fontes_do_corrige(c);
        let nome = match c {
            Corrige::Matriz => continue,
            Corrige::Ramos => ["corrige · ramos", "uno · ramos"],
            Corrige::Escolha => ["corrige · escolha", "uno · escolha"],
        };
        v.extend([(nome[0], corrige), (nome[1], uno)]);
    }
    v
}

impl Pipes {
    fn new(gpu: &GpuContext) -> Self {
        let [f, m, i, g, e, o, _, d, _, w] = fontes();
        let p = |(nome, src): (&str, String)| create_pipeline(gpu, &src, nome);
        let (corrige, uno): (Vec<_>, Vec<_>) = Corrige::TODAS
            .iter()
            .map(|&c| {
                let (corrige, uno) = fontes_do_corrige(c);
                (p(("corrige", corrige)), p(("uno", uno)))
            })
            .unzip();
        Self {
            forma: p(f),
            massa: p(m),
            inicia: p(i),
            gira: p(g),
            espalha: p(e),
            ordena: p(o),
            corrige,
            decide: p(d),
            uno,
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

    /// Força o tecto do caminho de um grupo (`None` = o do produto) — só a sonda.
    #[doc(hidden)]
    pub fn passe_um_grupo_ate(&mut self, n: Option<u32>) {
        self.passe.um_grupo_ate = n;
    }

    /// A forma do `corrige` das próximas corridas ([`Corrige`]) — só a sonda e o gate.
    #[doc(hidden)]
    pub fn passe_corrige(&mut self, c: Corrige) {
        self.passe.corrige = c;
    }

    /// ⭐ **O `Relatorio` do passe na placa** (doc 130 §5) — o que o perfilador do quadro mostra
    /// (varreduras · peças · vizinhos), lido SEM espera: chame depois de cada cozimento; a cópia
    /// encomendada nele é mapeada aqui e lida num quadro seguinte. `None` até à 1.ª leitura.
    /// ⚠️ Os vizinhos são os da grelha da PLACA (célula `2·alcance_max`), não os da da CPU.
    pub fn relatorio_do_passe(
        &mut self,
        gpu: &GpuContext,
    ) -> Option<ph2d_contact::passe::Relatorio> {
        let _ = gpu.device.poll(wgpu::PollType::Poll);
        if let Some((staging, pronto, pecas)) = &self.passe.leitura {
            match pronto.try_recv() {
                Ok(Ok(())) => {
                    let c: Vec<u32> =
                        bytemuck::pod_collect_to_vec(&staging.slice(..).get_mapped_range());
                    staging.unmap();
                    self.passe.ultimo = Some(ph2d_contact::passe::Relatorio {
                        varreduras: c[3] as usize,
                        pecas: *pecas as usize,
                        candidatos: c[4] as usize,
                        ..Default::default()
                    });
                    self.passe.leitura = None;
                }
                Ok(Err(_)) | Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.passe.leitura = None;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
            }
        }
        if self.passe.leitura.is_none()
            && let Some((staging, pecas)) = self.passe.copia.take()
        {
            let (tx, rx) = std::sync::mpsc::channel();
            staging.slice(..).map_async(wgpu::MapMode::Read, move |r| {
                let _ = tx.send(r);
            });
            self.passe.leitura = Some((staging, rx, pecas));
        }
        self.passe.ultimo
    }

    /// Quanto o processador levou a encomendar o último passe — só a sonda.
    #[doc(hidden)]
    #[must_use]
    pub fn passe_encode(&self) -> std::time::Duration {
        self.passe.encode
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
        let est = novo(pool, n64 * 32);
        let ago = novo(pool, n64 * 16);
        let starts = novo(pool, (u64::from(nb) + 1) * 4);
        let cursor = novo(pool, u64::from(nb) * 4);
        let sorted = novo(pool, n64 * 4);
        let ctrl = novo(pool, 32);
        let p_out = novo(pool, n64 * 8);
        let rot_out = novo(pool, n64 * 4);
        let ler = |nome: &str, dim: Dim| -> Arc<wgpu::Buffer> {
            col(nome, dim).map_or_else(|| Arc::clone(&nulo), |c: &GpuColumn| Arc::clone(&c.buffer))
        };
        let [raio, caixa, desvio, size, rot, w, ii, fall, rest, body] =
            COLUNAS.map(|(nome, dim, _)| ler(nome, dim));

        // ⭐ O caminho: poucas peças ⇒ TODAS as varreduras num despacho de um só grupo (§3.1).
        let um_grupo = n <= self.passe.um_grupo_ate.unwrap_or(PECAS_PARA_UM_GRUPO);
        let fim = if um_grupo || varreduras.is_multiple_of(2) {
            0
        } else {
            n
        };
        let uni = |bi: u32, bo: u32| {
            let b = gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("ph2d-passe u"),
                size: 32,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let campos = [n, nb, repouso.to_bits(), tem, bi, bo, varreduras, 0];
            gpu.queue.write_buffer(&b, 0, bytemuck::cast_slice(&campos));
            b
        };
        let (u0, u1, uf) = (uni(0, n), uni(n, 0), uni(fim, 0));

        let t0 = std::time::Instant::now();
        let pipes = self.passe.pipes.get_or_insert_with(|| Pipes::new(gpu));
        enc.clear_buffer(&ctrl, 0, None);
        let pb = &p.buffer;
        let g = |pipe, bufs: &[&wgpu::Buffer]| grupo(gpu, pipe, bufs);
        let gf = g(
            &pipes.forma,
            &[&u0, pb, &raio, &caixa, &desvio, &size, &rot, &dec, &ctrl],
        );
        despacha(enc, &pipes.forma, &gf, n);
        let gm = g(
            &pipes.massa,
            &[&u0, pb, &w, &ii, &fall, &rest, &body, &dec, &est],
        );
        despacha(enc, &pipes.massa, &gm, n);
        let gi = g(&pipes.inicia, &[&u0, &ctrl]);
        despacha(enc, &pipes.inicia, &gi, 1);
        let mut scratch = crate::scan::ScanScratch::default();
        let varre = |pipe, u: &wgpu::Buffer| {
            grupo(
                gpu,
                pipe,
                &[u, &dec, &ago, &est, &starts, &cursor, &sorted, &ctrl],
            )
        };
        if um_grupo {
            let uno = &pipes.uno[self.passe.corrige.indice()];
            despacha(enc, uno, &varre(uno, &u0), 1);
        } else {
            // As duas fases do ping-pong, ligadas UMA vez por passe.
            let fases = [&u0, &u1].map(|u| {
                (
                    varre(&pipes.gira, u),
                    varre(&pipes.espalha, u),
                    varre(&pipes.ordena, u),
                    varre(&pipes.corrige[self.passe.corrige.indice()], u),
                )
            });
            let gd = g(&pipes.decide, &[&u0, &ctrl]);
            for v in 0..varreduras as usize {
                let (gg, ge, go, gc) = &fases[v % 2];
                enc.clear_buffer(&starts, 0, None);
                enc.clear_buffer(&cursor, 0, None);
                despacha(enc, &pipes.gira, gg, n);
                pipes
                    .scan
                    .exclusive(gpu, enc, &starts, nb + 1, &mut scratch);
                despacha(enc, &pipes.espalha, ge, n);
                despacha(enc, &pipes.ordena, go, nb);
                despacha(enc, &pipes.corrige[self.passe.corrige.indice()], gc, n);
                despacha(enc, &pipes.decide, &gd, 1);
            }
        }
        let gw = g(
            &pipes.escreve,
            &[&uf, pb, &rot, &dec, &est, &p_out, &rot_out],
        );
        despacha(enc, &pipes.escreve, &gw, n);
        self.passe.hold_uni.extend([u0, u1, uf]);
        self.passe.hold_scan.push(scratch);
        // O `Relatorio` sai da placa SEM espera: uma cópia por vez, mapeada no quadro seguinte.
        if self.passe.copia.is_none() && self.passe.leitura.is_none() {
            let staging = gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("ph2d-passe relatorio"),
                size: 32,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            });
            enc.copy_buffer_to_buffer(&ctrl, 0, &staging, 0, 32);
            self.passe.copia = Some((staging, n));
        }
        self.passe.ctrl = Some(ctrl);
        self.passe.encode = t0.elapsed();

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
