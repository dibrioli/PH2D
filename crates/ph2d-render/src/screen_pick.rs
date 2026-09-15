//! ⭐⭐⭐ **A COR QUE O ARTISTA VÊ** — o que um conta-gotas tem de devolver.
//!
//! ⛔⛔⛔ **O conta-gotas lia a camada ERRADA, e por isso devolvia `#00000000` em quase todo o
//! ecrã** (report do dono, 2026-09-15: *«não funciona de maneira nenhuma e em nenhum lugar»*).
//!
//! O quadro deste app tem **duas** metades e elas vivem em texturas diferentes:
//!
//! | metade | textura | o que está lá |
//! |---|---|---|
//! | o MUNDO | a saída do [`crate::Tonemap`] (`Bgra8UnormSrgb`) | sprites, imagens, a pré-visualização do Painter |
//! | o CHROME | a intermédia do [`crate::VelloPass`] (`Rgba8Unorm`) | os painéis **e a arte VECTORIAL do documento** |
//!
//! A leitura de reserva do conta-gotas pedia **só a segunda**. Sobre uma sprite ela é
//! transparente por construção — e transparente é exactamente `#00000000`. ⚠️ **O patch que
//! existia curava UM canto** (o Painter activo sobre a sprite seleccionada, a amostrar a
//! composição de camadas dele); em todo o resto — outra ferramenta, outro painel de cor, o fundo
//! do canvas — o artista recebia transparente. *Um patch num canto lê-se como «funciona», e o
//! report que o desmente chega meses depois.*
//!
//! ⇒ a resposta é **compor as duas**, exactamente como o `compositor.wgsl` faz para o ecrã.
//!
//! # ⚠️ Porque a composição pode ser feita em BYTES
//!
//! O shader re-codifica o mundo para sRGB antes de misturar, porque os bytes do Vello já são
//! valores de DESIGNER (sRGB, alfa **directo**). A saída do tonemap é `*Srgb`, logo **os bytes
//! dela já são esse mesmo espaço**. ⇒ misturar os bytes com a
//! [`crate::compositor::composite_straight`] dá o byte que o ecrã mostra, sem uma conversão pelo
//! meio que só voltaria atrás.
//!
//! ⚠️ **Nota de espaço de cor do lado do Vello** (herdada do `read_pixel` que esta porta
//! substituiu): a intermédia é declarada `Rgba8Unorm` (linear) e o Vello escreve bytes
//! **sRGB-codificados** directamente nela (o `Color::from_rgba8` do peniko é sRGB). ⇒ os bytes
//! lidos já estão na codificação que o `ColorValue::from_rgba8` espera, e uma passagem
//! linear→sRGB aqui **clarearia os meios-tons** (`docs/UI_Bugs/README.md` §4.3).
//!
//! ⚠️ **E a saída do tonemap é `B G R A`** — trocar os canais é metade desta porta, e é a metade
//! que uma leitura distraída do `read_pixel` vizinho (que é `RGBA`) não faz.

use ph2d_gpu::GpuContext;

/// UM texel de uma textura, em bytes crus, pela ordem em que ela os guarda.
///
/// ⚠️ **Bloqueia** até a cópia da GPU aterrar e o buffer mapear (poucos ms). Uma escolha de
/// conta-gotas é um clique; ⛔ isto não se chama por quadro.
#[must_use]
pub fn read_texel(gpu: &GpuContext, texture: &wgpu::Texture, x: u32, y: u32) -> Option<[u8; 4]> {
    if x >= texture.width() || y >= texture.height() {
        return None;
    }
    // O wgpu exige `bytes_per_row` múltiplo de 256 numa cópia textura→buffer: pede-se a linha
    // inteira para levar 4 bytes.
    const BYTES_PER_ROW: u32 = 256;
    let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ph2d-render screen-pick readback"),
        size: u64::from(BYTES_PER_ROW),
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("ph2d-render screen-pick encoder"),
        });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d { x, y, z: 0 },
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(BYTES_PER_ROW),
                rows_per_image: Some(1),
            },
        },
        wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
    );
    gpu.queue.submit([encoder.finish()]);
    let slice = buffer.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |res| {
        let _ = tx.send(res);
    });
    gpu.device.poll(wgpu::PollType::wait_indefinitely()).ok()?;
    rx.recv().ok()?.ok()?;
    let view = slice.get_mapped_range();
    let texel = [view[0], view[1], view[2], view[3]];
    drop(view);
    buffer.unmap();
    Some(texel)
}

/// ⭐⭐⭐ **A composição das duas metades, em bytes** — a lei do `compositor.wgsl`, em CPU.
///
/// `mundo` chega em **`B G R A`** (a saída do tonemap) e `chrome` em `R G B A` com alfa **directo**
/// (a intermédia do Vello). Devolve `R G B A` com **`a = 255`**: o ecrã é opaco, e um conta-gotas
/// que devolvesse a alfa da camada de cima diria *«a cor que viste é meio transparente»* sobre um
/// pixel perfeitamente sólido.
#[must_use]
pub fn compose_screen_bytes(mundo_bgra: [u8; 4], chrome_rgba: [u8; 4]) -> [u8; 4] {
    let a = f32::from(chrome_rgba[3]) / 255.0;
    let mundo = [
        f32::from(mundo_bgra[2]),
        f32::from(mundo_bgra[1]),
        f32::from(mundo_bgra[0]),
    ];
    let chrome = [
        f32::from(chrome_rgba[0]),
        f32::from(chrome_rgba[1]),
        f32::from(chrome_rgba[2]),
    ];
    let c = crate::compositor::composite_straight(mundo, chrome, a);
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "uma mistura convexa de dois bytes cai em 0..=255 por construção"
    )]
    let byte = |v: f32| v.round().clamp(0.0, 255.0) as u8;
    [byte(c[0]), byte(c[1]), byte(c[2]), 255]
}

/// ⭐⭐ **QUAL das duas texturas é o MUNDO neste quadro** — a mesma pergunta que o compositor faz.
///
/// ⛔⛔ **Ela existe como PORTA e não como um `if` no chamador:** o quadro tem dois modos (o
/// simples, em que o compositor lê a saída do tonemap; e o INTERCALADO — ou com o vidro do *Edit
/// Prefab* —, em que o mundo inteiro já está no acumulador), e duas respostas à mesma pergunta
/// divergem no dia em que nascer um terceiro modo. *O conta-gotas tem de ler exactamente o que o
/// ecrã mostra.*
#[must_use]
pub fn world_source<'a>(
    compositor_reads_world: bool,
    world_rt: &'a crate::WorldRt,
    tonemap: &'a crate::Tonemap,
) -> &'a wgpu::Texture {
    if compositor_reads_world {
        world_rt.texture()
    } else {
        tonemap.output_texture()
    }
}

/// ⭐⭐⭐ **A cor do ECRÃ em `(x, y)`** — as duas metades lidas e compostas. `None` fora do quadro.
///
/// Ver o cabeçalho do módulo para o porquê de serem duas.
#[must_use]
pub fn screen_color(
    gpu: &GpuContext,
    mundo: &wgpu::Texture,
    chrome: &wgpu::Texture,
    x: u32,
    y: u32,
) -> Option<[u8; 4]> {
    let m = read_texel(gpu, mundo, x, y)?;
    let c = read_texel(gpu, chrome, x, y)?;
    Some(compose_screen_bytes(m, c))
}

#[cfg(test)]
#[path = "screen_pick_tests.rs"]
mod tests;
