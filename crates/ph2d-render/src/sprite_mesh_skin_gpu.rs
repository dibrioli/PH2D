//! ⭐⭐⭐ **A METADE DO DISPOSITIVO DA PELE** (F9 W2) — os QUATRO buffers que o `posa_pela_pele` do
//! `sprite.wgsl` lê, a COSTURA que os enche e a CONJUGAÇÃO que leva um afim **e um ponto** da pele
//! ao espaço do quad.
//!
//! ⛔⛔ **O quarto buffer é a tabela de JUNTAS, e ela existe porque a lei não é linear** — o
//! [`ph2d_skeleton::Skin::blend`] roda em torno da junta desde 2026-09-19. Ver o cabeçalho do
//! [`crate::sprite_mesh_skin`] para o que o gate de paridade mediu antes disto existir.
//!
//! ⚠️ **Irmão do [`crate::sprite_mesh`] pelo tecto de 700 LOC, e o corte é por RESPONSABILIDADE:**
//! ali mora *o que é uma sprite desenhada como malha*; aqui, *o que a placa precisa de receber para
//! a posar*. ⛔ A cura de um tecto vermelho é o CORTE, nunca uma entrada no `FILE_OVERAGE_OK`.
//!
//! ⚠️ **Eles são PARALELOS ao buffer de vértices das malhas**, e é isso que dispensa um offset por
//! chamada: numa chamada não-indexada o `@builtin(vertex_index)` é o índice ABSOLUTO no buffer
//! ligado, logo o vértice `j` da tira lê `skin_w[j]` e `skin_b[j]`. *Um vector paralelo resolve o
//! que um `@location` novo não podia* — os `0..15` do dispositivo estão cheios (a
//! `RenderInstance` ocupa `2..15` e o `QuadVertex` o `0..1`).
//!
//! ⚠️ **O grupo é RECONSTRUÍDO quando qualquer um dos três cresce**, porque um `BindGroup` do wgpu
//! guarda o buffer e não o slot. ⛔ Recriá-lo por quadro seria trabalho por quadro para uma coisa
//! que só muda quando a cena engorda.

use crate::sprite_mesh::{MeshFrame, SpriteMesh};
use ph2d_gpu::GpuContext;

/// Um registo de OSSO para o dispositivo — o afim conjugado, o ângulo CRU da pose dele, e o que o
/// shader precisa para achar a tabela de juntas da malha a que ele pertence.
///
/// ⚠️ **A convenção do afim é a do `ph2d_affine::Xform`** (`x' = a·x + c·y + e`), e ela é a mesma na
/// CPU, no payload e aqui — *três convenções para o mesmo afim é onde um sinal se perde*.
///
/// ⚠️⚠️ **O `idx` é o índice LOCAL deste osso na malha dele, e não é redundante com o índice do
/// array:** o `skin_b` guarda índices já DESLOCADOS pela base do quadro, e a tabela de juntas é
/// indexada em local. ⛔ Guardar a base em vez do índice custava o mesmo e obrigava a uma subtracção
/// por par; assim o shader lê `idx` de cada osso que já foi buscar.
///
/// ⚠️ **As duas razões são da INSTÂNCIA e repetem-se por osso dela, de propósito:** conjugar uma
/// ROTAÇÃO por um `size` não-uniforme não dá uma rotação (`S⁻¹RS` tem os termos fora da diagonal
/// multiplicados por `sy/sx` e `sx/sy`), e o `θ̄` só existe DEPOIS de o shader misturar os ângulos.
/// ⛔ *Um valor que só o shader pode calcular não pode chegar pré-conjugado.*
///
/// Alinhamento: `vec4` a `0`, `vec2` a `16` e `24`, `vec4<u32>` a `32`, `vec4<f32>` a `48` ⇒ passo
/// `64`, que é múltiplo de `16`. ⚠️ **`24` bones custam `1,5 KB`** — o registo é do QUADRO e a
/// generosidade aqui não é medível.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct SkinAfimGpu {
    pub(crate) lin: [f32; 4],
    pub(crate) tra: [f32; 2],
    pub(crate) ang: [f32; 2],
    /// `(índice local, ossos da malha, base da tabela de juntas, 0)`.
    pub(crate) info: [u32; 4],
    /// `(sx/sy, sy/sx, 0, 0)` da instância para que este registo foi conjugado.
    pub(crate) razao: [f32; 4],
}

impl SkinAfimGpu {
    /// Do `[a, b, c, d, e, f]` já conjugado para o que o shader lê.
    pub(crate) fn de(m: [f32; 6], ang: [f32; 2], info: [u32; 4], razao: [f32; 2]) -> Self {
        Self {
            lin: [m[0], m[1], m[2], m[3]],
            tra: [m[4], m[5]],
            ang,
            info,
            razao: [razao[0], razao[1], 0.0, 0.0],
        }
    }
}

/// Os quatro buffers e o grupo que os liga.
pub(crate) struct SkinBuffers {
    pesos: wgpu::Buffer,
    ossos: wgpu::Buffer,
    afins: wgpu::Buffer,
    juntas: wgpu::Buffer,
    caps: [u32; 4],
    grupo: wgpu::BindGroup,
}

/// ⚠️ **Nenhum buffer nasce com tamanho zero:** o wgpu recusa um binding vazio, e uma cena sem pele
/// nenhuma é o caso NORMAL. Uma entrada basta, e ela nunca é lida (o `SEM_PELE` decide antes).
const MINIMO: u32 = 1;

impl SkinBuffers {
    pub(crate) fn new(gpu: &GpuContext, bgl: &wgpu::BindGroupLayout) -> Self {
        let pesos = Self::aloca(gpu, "pesos", MINIMO, 16);
        let ossos = Self::aloca(gpu, "ossos", MINIMO, 16);
        let afins = Self::aloca(gpu, "afins", MINIMO, 64);
        let juntas = Self::aloca(gpu, "juntas", MINIMO, 8);
        let grupo = Self::grupo(gpu, bgl, &pesos, &ossos, &afins, &juntas);
        Self {
            pesos,
            ossos,
            afins,
            juntas,
            caps: [MINIMO; 4],
            grupo,
        }
    }

    fn aloca(gpu: &GpuContext, nome: &str, capacidade: u32, passo: u64) -> wgpu::Buffer {
        gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("ph2d-render skin {nome}")),
            size: u64::from(capacidade.max(MINIMO)) * passo,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    fn grupo(
        gpu: &GpuContext,
        bgl: &wgpu::BindGroupLayout,
        pesos: &wgpu::Buffer,
        ossos: &wgpu::Buffer,
        afins: &wgpu::Buffer,
        juntas: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-render skin bg"),
            layout: bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: pesos.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: ossos.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: afins.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: juntas.as_entire_binding(),
                },
            ],
        })
    }

    /// ⭐ **Sobe o que o quadro montou.** Recria o grupo **só** quando algum buffer cresceu.
    pub(crate) fn upload(
        &mut self,
        gpu: &GpuContext,
        bgl: &wgpu::BindGroupLayout,
        pesos: &[[f32; 4]],
        ossos: &[[u32; 4]],
        afins: &[SkinAfimGpu],
        juntas: &[[f32; 2]],
    ) {
        let pedidos = [
            u32::try_from(pesos.len()).unwrap_or(u32::MAX),
            u32::try_from(ossos.len()).unwrap_or(u32::MAX),
            u32::try_from(afins.len()).unwrap_or(u32::MAX),
            u32::try_from(juntas.len()).unwrap_or(u32::MAX),
        ];
        let mut cresceu = false;
        for (i, pedido) in pedidos.iter().enumerate() {
            if *pedido <= self.caps[i] {
                continue;
            }
            let mut cap = self.caps[i].max(MINIMO);
            while cap < *pedido {
                cap = cap.saturating_mul(2);
            }
            let (nome, passo) = [
                ("pesos", 16u64),
                ("ossos", 16),
                ("afins", 64),
                ("juntas", 8),
            ][i];
            let novo = Self::aloca(gpu, nome, cap, passo);
            match i {
                0 => self.pesos = novo,
                1 => self.ossos = novo,
                2 => self.afins = novo,
                _ => self.juntas = novo,
            }
            self.caps[i] = cap;
            cresceu = true;
        }
        if cresceu {
            self.grupo = Self::grupo(
                gpu,
                bgl,
                &self.pesos,
                &self.ossos,
                &self.afins,
                &self.juntas,
            );
        }
        if !pesos.is_empty() {
            gpu.queue
                .write_buffer(&self.pesos, 0, bytemuck::cast_slice(pesos));
        }
        if !ossos.is_empty() {
            gpu.queue
                .write_buffer(&self.ossos, 0, bytemuck::cast_slice(ossos));
        }
        if !afins.is_empty() {
            gpu.queue
                .write_buffer(&self.afins, 0, bytemuck::cast_slice(afins));
        }
        if !juntas.is_empty() {
            gpu.queue
                .write_buffer(&self.juntas, 0, bytemuck::cast_slice(juntas));
        }
    }

    pub(crate) fn grupo_ligado(&self) -> &wgpu::BindGroup {
        &self.grupo
    }
}

impl MeshFrame {
    /// ⭐⭐⭐ **A TABELA DA PELE DESTA MALHA, costurada em lockstep com os vértices dela.**
    ///
    /// ⚠️ **Sem pele ela escreve [`SEM_PELE`] em todo vértice** — e é isso que torna o caminho de
    /// toda malha que não é uma pele byte-idêntico *por construção*: o shader devolve o `quad_pos`
    /// que recebeu, sem uma multiplicação.
    ///
    /// ⚠️ **Os índices são DESLOCADOS pela base do quadro:** o payload guarda-os locais à malha
    /// (`0..n_ossos`) e a tabela do dispositivo é a concatenação de todas as malhas. ⛔ Sem o
    /// deslocamento, a segunda imagem presa da cena posava-se pelos ossos da primeira.
    ///
    /// ⚠️⚠️ **E o registo de cada osso leva o índice LOCAL dele de volta**, porque a tabela de
    /// juntas é indexada em local — *o deslocamento que a concatenação faz tem de ser desfeito
    /// exactamente onde ele atrapalha, e em lado nenhum mais*.
    pub(crate) fn costura_da_pele(&mut self, malha: &SpriteMesh, anchor: [f32; 2], size: [f32; 2]) {
        let n = malha.local.len().min(malha.uv.len());
        let pele = malha.skin.as_ref().filter(|p| p.valida(malha.local.len()));
        let Some(pele) = pele else {
            crate::sprite_mesh::caminho_da_tira(&malha.tris, n, |_| {
                self.pesos
                    .push([0.0; crate::sprite_mesh_skin::OSSOS_POR_VERTICE]);
                self.ossos.push(
                    [crate::sprite_mesh_skin::SEM_PELE; crate::sprite_mesh_skin::OSSOS_POR_VERTICE],
                );
            });
            return;
        };
        let base = u32::try_from(self.afins.len()).unwrap_or(0);
        let base_juntas = u32::try_from(self.juntas.len()).unwrap_or(0);
        let ossos_da_malha = u32::try_from(pele.afins.len()).unwrap_or(0);
        // ⚠️ `size` com um lado nulo nunca chega aqui: o `quad_pos` já recusou a malha antes.
        let razao = [size[0] / size[1], size[1] / size[0]];
        for (k, m) in pele.afins.iter().enumerate() {
            let ang = pele.angulos.get(k).copied().unwrap_or([1.0, 0.0]);
            let info = [
                u32::try_from(k).unwrap_or(0),
                ossos_da_malha,
                base_juntas,
                0,
            ];
            self.afins.push(SkinAfimGpu::de(
                conjuga_para_o_quad(*m, anchor, size),
                ang,
                info,
                razao,
            ));
        }
        for j in &pele.juntas {
            self.juntas
                .push(conjuga_ponto_para_o_quad(*j, anchor, size));
        }
        let (pesos, ossos) = (&mut self.pesos, &mut self.ossos);
        crate::sprite_mesh::caminho_da_tira(&malha.tris, n, |v| {
            pesos.push(pele.pesos[v]);
            ossos.push(std::array::from_fn(|k| base + pele.ossos[v][k]));
        });
    }
}

/// ⭐⭐ **UM PONTO DA PELE, LEVADO AO ESPAÇO DO QUAD** — `Q(l) = (l − anchor)/size`, a MESMA `Q` de
/// que a [`conjuga_para_o_quad`] é a conjugação.
///
/// ⚠️ **Ele é ponto e não vector:** a junta é um SÍTIO, logo a translação entra. ⛔ Conjugá-la como
/// direcção (sem o `− anchor`) punha o centro de rotação no sítio errado em toda sprite cuja âncora
/// não fosse a origem, *e a dobra sairia deslocada só nessas*.
///
/// ⭐ **E é porque ela é afim que a base da mistura sobrevive:** `Q(Σ ŵ_i x_i) = Σ ŵ_i Q(x_i)` vale
/// porque `Σ ŵ = 1` — a renormalização da truncagem a `K` é o que torna esta igualdade exacta, e
/// não uma aproximação.
#[must_use]
pub(crate) fn conjuga_ponto_para_o_quad(p: [f32; 2], anchor: [f32; 2], size: [f32; 2]) -> [f32; 2] {
    [(p[0] - anchor[0]) / size[0], (p[1] - anchor[1]) / size[1]]
}

/// ⭐⭐⭐ **UM AFIM DA PELE, LEVADO DO ESPAÇO LOCAL PARA O DO QUAD** — `A = Q ∘ M ∘ Q⁻¹`, com
/// `Q(l) = (l − anchor) / size`.
///
/// ⚠️⚠️ **É esta conjugação que deixa o payload ser do BIND e não da INSTÂNCIA:** o mesmo bind serve
/// **nove** instâncias num 9-slice, cada uma com o seu `anchor`/`size`. Guardar os afins já em
/// espaço de quad amarraria a pele ao quad de uma delas, e as outras oito desenhariam a dobra
/// deslocada.
///
/// ⚠️ **Um `size` NEGATIVO é o espelho da sprite, e a conjugação lida com ele sozinha** — os
/// factores `sy/sx` e `sx/sy` trocam de sinal em par, que é o que mantém a dobra do mesmo lado da
/// arte espelhada. ⛔ Um `abs()` aqui desenharia a dobra ao contrário numa sprite virada.
///
/// `size` com um lado nulo nunca chega aqui: o [`quad_pos`] já recusou a malha antes.
#[must_use]
pub(crate) fn conjuga_para_o_quad(m: [f32; 6], anchor: [f32; 2], size: [f32; 2]) -> [f32; 6] {
    let [a, b, c, d, e, f] = m;
    let (sx, sy) = (size[0], size[1]);
    let (ax, ay) = (anchor[0], anchor[1]);
    [
        a,
        b * sx / sy,
        c * sy / sx,
        d,
        (a * ax + c * ay + e - ax) / sx,
        (b * ax + d * ay + f - ay) / sy,
    ]
}

#[cfg(test)]
#[path = "sprite_mesh_skin_gpu_tests.rs"]
mod sprite_mesh_skin_gpu_tests;
