//! ⭐⭐ **A CODIFICAÇÃO com que a forma viaja no ARQUIVO** — as quatro portas e o arredondamento.
//!
//! Irmão (`#[path]`) do [`super`], e o corte é de ASSUNTO: lá mora *quando a forma é re-acesa e por
//! que lei*; aqui *com que bytes ela é guardada*. Os dois crescem por motivos diferentes — o pai
//! quando a lei da luz muda, este quando o FORMATO muda, que é quase nunca.
//!
//! ⚠️ **O corte foi FORÇADO pelo tecto de LOC** (HR-18, `700`): o pai chegou a `745`. A cura de um
//! tecto é um corte para o IRMÃO, nunca uma entrada nova no `FILE_OVERAGE_OK`.
//!
//! ⚠️ **As quatro portas são DUAS ida-e-volta** (`form_to_rgba8`/`form_from_rgba8` ·
//! `occlusion_to_r8`/`occlusion_from_r8`), e vivem juntas por isso: *uma metade que mude sem a outra
//! é um arquivo que se lê ao contrário, em silêncio*.

/// **O G-BUFFER virado IMAGEM** — a codificação com que a forma viaja no arquivo.
///
/// ⚠️ **A escolha é MEDIDA, e o número está no gate** (`bake_form_bytes`): guardar a forma
/// como `f32` custa **4× o disco** (16 MiB por sprite a 1024²) e o que ela compra é *nada que a luz
/// enxergue* — baixar para 8 bits move o pixel aceso em **≤ 3 de 255** (pior caso medido, com
/// ~0,25 de média). Não é um palpite sobre precisão: é o preço da precisão, pago pelo consumidor.
///
/// É também o que a indústria inteira shipa — as *Secondary Textures* da Unity, o `normal_texture`
/// da `CanvasTexture` do Godot, o bake-to-texture do Blender, o Sprite DLight, o Spine. Ninguém
/// guarda a malha, e ninguém guarda normais em `f32`.
///
/// A normal é `n × 0,5 + 0,5` por canal; o peso vai no alfa, cru.
pub fn form_to_rgba8(form: &[f32]) -> Vec<u8> {
    let mut out = vec![0u8; form.len()];
    for (o, f) in out
        .as_chunks_mut::<4>()
        .0
        .iter_mut()
        .zip(form.as_chunks::<4>().0.iter())
    {
        for c in 0..3 {
            o[c] = quantise(f[c] * 0.5 + 0.5);
        }
        o[3] = quantise(f[3]);
    }
    out
}

/// **A OCLUSÃO virada IMAGEM** — um byte por texel, o mesmo argumento do irmão acima.
///
/// ⚠️ **`R8` e não `f32`, e a medição que decide é a mesma da W8.7:** a oclusão é uma FRAÇÃO em
/// `[0, 1]`, então 256 níveis sobre uma quantidade que o olho lê como sombra suave custam **um
/// quarto** do disco de um plano de `f32` — e a alternativa seria fazer um plano que nunca mais muda
/// depois do bake pesar tanto quanto o G-buffer inteiro.
pub fn occlusion_to_r8(occ: &[f32]) -> Vec<u8> {
    occ.iter().map(|o| quantise(*o)).collect()
}

/// A inversa de [`occlusion_to_r8`].
///
/// ⚠️ **Sem renormalização, ao contrário da irmã** — e a assimetria diz o que cada canal É: uma
/// normal é uma DIREÇÃO (comprimento errado é brilho errado), uma oclusão é um NÚMERO.
///
/// ⚠️ **E um plano VAZIO decodifica para vazio, não para zeros:** um documento anterior a esta wave
/// não traz oclusão nenhuma, e o neutro dela é `1.0`. Devolver `vec![0.0]` pintaria de preto toda
/// arte já assada; quem substitui o neutro é o `Option` do consumidor.
pub fn occlusion_from_r8(bytes: &[u8]) -> Vec<f32> {
    bytes.iter().map(|b| f32::from(*b) / 255.0).collect()
}

/// **A imagem virada G-BUFFER de volta** — a inversa de [`form_to_rgba8`].
///
/// ⚠️ **A RENORMALIZAÇÃO não é enfeite.** `n × 0,5 + 0,5` quantizado não decodifica um vetor
/// unitário: os três canais arredondam independentemente, e o que volta tem comprimento entre
/// ~0,996 e ~1,004. A luz lê a normal como direção, então um vetor de comprimento errado é um brilho
/// errado — e o consumidor renormalizaria de qualquer jeito. Fazê-lo aqui é fazê-lo **uma vez**.
pub fn form_from_rgba8(bytes: &[u8]) -> Vec<f32> {
    let mut out = vec![0f32; bytes.len()];
    for (o, b) in out
        .as_chunks_mut::<4>()
        .0
        .iter_mut()
        .zip(bytes.as_chunks::<4>().0.iter())
    {
        let (x, y, z) = (
            f32::from(b[0]) / 255.0 * 2.0 - 1.0,
            f32::from(b[1]) / 255.0 * 2.0 - 1.0,
            f32::from(b[2]) / 255.0 * 2.0 - 1.0,
        );
        // ⚠️ O piso existe para um texel VAZIO (peso 0, os três canais em `128`) não virar `NaN`:
        // ali a normal não significa nada, mas o resultado ainda tem de ser um número.
        let len = (x * x + y * y + z * z).sqrt().max(1e-6);
        o[0] = x / len;
        o[1] = y / len;
        o[2] = z / len;
        o[3] = f32::from(b[3]) / 255.0;
    }
    out
}

/// `[0,1] → u8`, com o `+0,5` que faz do arredondamento o mais próximo em vez de truncar.
pub(super) fn quantise(v: f32) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0 + 0.5).floor() as u8
}
