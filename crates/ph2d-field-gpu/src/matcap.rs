//! ⭐⭐⭐ **O PASSE QUE PINTA UM MATCAP** — o modo de **omissão** do modelador, no dispositivo.
//!
//! # ⛔⛔⛔ Porque ele existe: o caminho que o artista de facto toma, medido
//!
//! O `#[default]` do `Shading` deste módulo é o **matcap** (*«a omissão de um modelador»*), e até
//! aqui o dispositivo só pintava o **`Render`**. ⇒ ao abrir uma cena, o arrasto ia **todo** pela
//! CPU: medido a `1920×1080` na cena `5`, **`90,17 ms`** e o prévio a escolher **`D=3`** — um nono
//! dos píxeis — contra **`16,63 ms`** e `D=1` do dispositivo.
//!
//! ⚠️⚠️ **E a cura NÃO era abrir a MARCHA ao matcap**, que foi a primeira leitura e é a errada: a
//! [`crate::trace::Tracer::frame`] devolve o G-buffer pelo barramento (`49,8 MB` a `1920×1080`,
//! `119`–`123 ms`) — *mais lento do que a CPU inteira*. O que ganha é **PINTAR** no dispositivo, e
//! é isso que este passe faz.
//!
//! # ⭐⭐⭐ Porque ele é um passe PRÓPRIO e não um modo do [`crate::paint`]
//!
//! Ver a nota do [`crate::matcap_wgsl`]: os **armazéns** (`9` contra o piso de `8` do WebGPU), o
//! matcap **não ler o campo da peça**, e compilar ser o caro.
//!
//! # ⚠️ A FOTOGRAFIA sobe UMA VEZ
//!
//! `749²×3` floats são `6,7 MB`. Subi-los por quadro seria quase o preço da imagem que este passe
//! veio poupar ⇒ [`crate::FieldPipelines::matcap_buffer`], com a mesma lei das grades das
//! esculturas e um contador para o gate observar.

/// Tudo o que o pintor de matcap precisa e que **não** sai da marcha.
pub struct MatcapSetup<'a> {
    /// `lado × lado × 3` valores **lineares** — o mesmo vector que a
    /// [`ph2d_field_render::shade_with`] indexa, sem repacote nenhum.
    pub rgb_linear: &'a [f32],
    /// O lado, em texels. ⛔ `0` não é pintável e o chamador cai na CPU — ver
    /// [`crate::supports_matcap`].
    pub side: u32,
    /// ⭐⭐⭐ **A identidade da FOTOGRAFIA** — o que decide se ela precisa de subir.
    ///
    /// ⚠️⚠️ **Ela é derivada do CONTEÚDO e calculada UMA vez por quem a carrega**, nunca aqui:
    /// resumir `6,7 MB` por quadro para decidir se se enviam `6,7 MB` é pagar o preço duas vezes —
    /// a lei que o doc da [`crate::FieldPipelines::grades`] já escreve.
    ///
    /// ⛔ **E é conteúdo e não endereço** de propósito: um `Arc` que morre e outro que nasce no
    /// mesmo sítio serviriam a fotografia errada **sem erro nenhum**, e as grades só escapam a isso
    /// por guardarem referências fortes — coisa que esta crate não pode fazer sobre um tipo que
    /// não conhece.
    pub chave: u64,
    /// A exposição, em paragens — o olhar da CENA, que vale para o matcap como no Blender.
    pub stops: f32,
    /// A vista, no código do [`ph2d_view_transform::wgsl::view_code`].
    pub view: u32,
    /// Os bytes EXACTOS que um pixel de fundo recebe — copiados, nunca reconvertidos.
    pub background: [u8; 4],
}

/// ⭐⭐⭐ **AS TRÊS ENTRADAS DO GRUPO `1` DESTE PASSE, numa lista NOMEADA** — o uniforme, a saída e
/// a fotografia.
#[must_use]
pub(crate) fn entradas() -> [wgpu::BindGroupLayoutEntry; 3] {
    use crate::trace::{armazem, uniforme};
    [uniforme(0), armazem(1, false), armazem(2, true)]
}

/// ⭐⭐⭐⭐ **QUANTOS ARMAZÉNS ESTE PASSE LIGA — CONTADOS** dos grupos que ele de facto liga: os do
/// grupo `0` da marcha mais os deste.
///
/// ⛔⛔ **Ele é uma FUNÇÃO e não um `const`, e a razão é um defeito medido:** o irmão
/// ([`crate::paint::armazens`]) era um literal que esteve **`3` abaixo** do real por três waves, e
/// o guarda que ele alimenta passou a aceitar placas onde a `wgpu` recusa o layout a meio de um
/// quadro. ⇒ *«número que soma se CONTA, nunca se escolhe»*.
///
/// ⚠️⚠️ **E é por isso que a lei dele é um GATE e não um `const _: () = assert!`:** uma asserção de
/// compilação sobre um número derivado é impossível (ele lê listas em tempo de execução), e sobre um
/// número ESCRITO ela mediria o literal em vez do layout. *A cerca mais forte é a que julga o que o
/// passe liga, não o que alguém escreveu que ele liga* — ver
/// [`matcap_tests::os_armazens_sao_contados_e_cabem_no_piso`].
#[must_use]
pub fn armazens() -> u32 {
    crate::trace::conta_armazens(&crate::trace::entradas_da_marcha())
        + crate::trace::conta_armazens(&entradas())
}

/// O piso que a `wgpu` garante em `max_storage_buffers_per_shader_stage`.
pub const PISO_DO_WEBGPU: u32 = 8;

/// ⭐ **O texto do pintor de matcap** — as declarações do grupo `0`, o olhar, o empacotamento e o
/// corpo.
///
/// ⚠️ **Ele NÃO contém o [`crate::FIELD_SLOT`]**, e isso é a propriedade que faz o cache de
/// pipelines acertar para sempre: ver a nota do [`crate::matcap_wgsl`], ponto 2, e o gate
/// `o_passe_do_matcap_nao_le_o_campo_da_peca`.
#[must_use]
pub fn fonte() -> String {
    format!(
        "{}{}\n{}\n{}",
        crate::trace_wgsl::comum(),
        ph2d_view_transform::wgsl::SOURCE,
        crate::empacota_wgsl::EMPACOTA,
        crate::matcap_wgsl::MATCAP,
    )
}

/// Os bytes do uniforme, por esta ordem exacta.
///
/// ⛔ **Um uniforme lido com a compensação errada não estoura: PINTA** — a mesma lei do
/// [`crate::paint_uniforme`], e por isso a ordem daqui é a lei.
fn arruma(mc: &MatcapSetup<'_>, n_bordas: u32) -> Vec<u8> {
    let bg = mc.background;
    let a = f32::from(bg[3]) / 255.0;
    let mut u: Vec<u8> = Vec::with_capacity(48);
    // `olhar`: a exposição, e três de reserva a ZERO — uma posição sem dono é onde o campo
    // seguinte aterra por engano.
    for f in [mc.stops, 0.0, 0.0, 0.0] {
        u.extend_from_slice(&f.to_le_bytes());
    }
    // ⚠️ **O fundo da BORDA é LINEAR e PRÉ-MULTIPLICADO** — é ele que entra na média das quatro
    // sub-amostras, exactamente como o `bg` do `ph2d_field_render::shade_with`.
    for f in [
        ph2d_color::srgb::srgb_to_linear_byte(bg[0]) * a,
        ph2d_color::srgb::srgb_to_linear_byte(bg[1]) * a,
        ph2d_color::srgb::srgb_to_linear_byte(bg[2]) * a,
        a,
    ] {
        u.extend_from_slice(&f.to_le_bytes());
    }
    let empacotado = u32::from(bg[0])
        | (u32::from(bg[1]) << 8)
        | (u32::from(bg[2]) << 16)
        | (u32::from(bg[3]) << 24);
    for v in [mc.view, n_bordas, empacotado, mc.side] {
        u.extend_from_slice(&v.to_le_bytes());
    }
    u
}

/// ⭐⭐⭐ **A imagem, pintada.** RGBA8 pré-multiplicado em ecrã, pronto para a tela.
// O dispositivo, a fila, o cache, a fotografia, os alvos, a tela e a contagem de bordas — sete
// coisas independentes, e uma struct só as renomearia (a mesma nota que o irmão `paint::pinta`
// carrega).
#[allow(clippy::too_many_arguments)]
pub(crate) fn pinta(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    cache: &mut crate::FieldPipelines,
    mc: &MatcapSetup<'_>,
    alvos: &crate::paint::Alvos<'_>,
    width: u32,
    height: u32,
    bordas: u64,
) -> Vec<u8> {
    // ⭐ As entradas saem das listas nomeadas (`entradas` / `crate::trace::entradas_da_marcha`) —
    // e é por isso que os construtores de ligação já não são importados aqui.
    use wgpu::util::DeviceExt;

    let bgl1 = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("matcap"),
        entries: &entradas(),
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("matcap"),
        bind_group_layouts: &[Some(alvos.bgl), Some(&bgl1)],
        immediate_size: 0,
    });
    let fonte = fonte();
    // ⚠️ **A fita vai porque a porta a pede, e é IGNORADA**: este texto não tem o
    // [`crate::FIELD_SLOT`], logo o `replace` é um no-op e a chave do cache não muda quando o
    // artista acrescenta uma forma. *É essa ausência que o gate afirma.*
    let p_pinta = cache
        .entry_with_layout(device, &fonte, alvos.fita, "pinta_matcap", Some(&layout))
        .clone();
    // ⭐ **Compilar é o caro** — a borda só nasce quando ela vai de facto correr.
    let p_bordas = (bordas > 0).then(|| {
        cache
            .entry_with_layout(
                device,
                &fonte,
                alvos.fita,
                "pinta_matcap_bordas",
                Some(&layout),
            )
            .clone()
    });

    #[allow(clippy::cast_possible_truncation)]
    let n_bordas = bordas as u32;
    let ub = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("matcap"),
        contents: &arruma(mc, n_bordas),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let n = u64::from(width) * u64::from(height);
    let b_saida = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("imagem"),
        size: (n * 4).max(16),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    // ⭐⭐⭐ **A fotografia, do cache** — ver [`crate::FieldPipelines::matcap_buffer`].
    let b_foto = cache.matcap_buffer(device, mc).clone();

    fn recurso(b: &wgpu::Buffer, i: u32) -> wgpu::BindGroupEntry<'_> {
        wgpu::BindGroupEntry {
            binding: i,
            resource: b.as_entire_binding(),
        }
    }
    let bg0 = crate::trace_grupo::grupo_da_marcha(
        device,
        alvos.bgl,
        alvos.setup,
        alvos.k,
        alvos.centro,
        alvos.luz,
        alvos.conta,
        alvos.borda,
        alvos.grades,
    );
    let bg1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bgl1,
        entries: &[recurso(&ub, 0), recurso(&b_saida, 1), recurso(&b_foto, 2)],
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
    // ⚠️ **A borda DEPOIS do interior, e a ordem é a lei**: ela SOBRESCREVE o pixel que o interior
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

#[cfg(test)]
#[path = "matcap_tests.rs"]
mod matcap_tests;
