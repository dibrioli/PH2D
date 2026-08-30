//! ⭐ **A REDUÇÃO a uma miniatura de cartão** — uma lei, três consumidores.
//!
//! Ela nasceu dentro do `motion_object_bake` e servia dois assadores (o vetor e o Flip). Em
//! 2026-08-30 o navegador de assets passou a ser o terceiro, e o nome do módulo deixou de descrever
//! quem a usa. ⇒ ela muda-se para aqui, e o assador passa a ser um **chamador** como os outros.
//!
//! ⚠️ **O que fica lá é a construção do `PreviewThumb`**, que é vocabulário de um painel: este
//! módulo devolve as **partes** (bytes + tamanho) e não conhece painel nenhum. É o que permite ao
//! `ph2d-asset-index` — uma crate folha, sem UI — receber a mesma redução sem herdar a dependência.

/// Lado máximo (px) de uma miniatura de cartão — grande o suficiente para se ler a forma, pequeno
/// o suficiente para os bytes viajarem por quadro de graça (~37 KB a 96²).
pub(crate) const THUMB_MAX: u32 = 96;

/// Reduz RGBA8 **reta** (`w`×`h`) a uma miniatura: no máximo [`THUMB_MAX`] no lado longo, aspecto
/// preservado, **nunca amplia**.
///
/// ⚠️ **Média de caixa em espaço PRÉ-MULTIPLICADO** (`Σ c·a / Σ a`): sem isso uma borda
/// transparente sangra um halo escuro para dentro da forma encolhida — a armadilha do
/// pré-multiplicado que a lição do overlay nomeia (vizinhança do ADR-0120). É a mesma família do
/// defeito que a cor do cartão pagou (a média crua de uma sprite recortada é a cor do **nada**).
///
/// # ⭐⭐ O PREÇO, medido (release, 2026-08-30 — `measure_thumbnail_cost`)
///
/// | textura | saída | relógio |
/// |---|---|---|
/// | 256² | 96² | **0,273 ms** |
/// | 512² | 96² | **0,383 ms** |
/// | 1024² | 96² | **0,750 ms** |
/// | 2048² | 96² | **3,584 ms** |
/// | 4096² | 96² | **12,079 ms** |
///
/// ⚠️ **É `O(pixels da FONTE)`, e é isso que torna a memória por conteúdo obrigatória** — a
/// `TextureLibrary` reescreve a entrada de cada textura a cada quadro, então sem ela uma imagem de
/// 4096² custaria **72% de um quadro de 60 fps, 60×/s**. Com ela custa isso **uma vez**, no quadro
/// em que a imagem entra — que é o mesmo quadro em que ela já foi descodificada.
///
/// ⛔ **O tecto de amostras da cor MEDIDO e NÃO adoptado aqui:** o `swatch_for` percorre a imagem
/// com passo e paga ~4 µs em qualquer tamanho, mas ele responde UM número. Saltar pixels numa
/// miniatura apaga exactamente aquilo que se quer ver — *a resposta dela é a FORMA*. O que se
/// compraria era relógio numa passagem que já corre uma vez só.
///
/// Devolve `(bytes, largura, altura)`.
pub(crate) fn reduce(rgba: &[u8], w: u32, h: u32) -> (std::sync::Arc<Vec<u8>>, u32, u32) {
    let (w, h) = (w.max(1), h.max(1));
    let long = w.max(h);
    let (tw, th) = if long <= THUMB_MAX {
        (w, h)
    } else {
        let s = THUMB_MAX as f32 / long as f32;
        (
            ((w as f32 * s).round() as u32).max(1),
            ((h as f32 * s).round() as u32).max(1),
        )
    };
    let mut out = vec![0u8; (tw * th * 4) as usize];
    for oy in 0..th {
        let sy0 = oy * h / th;
        let sy1 = ((oy + 1) * h / th).max(sy0 + 1).min(h);
        for ox in 0..tw {
            let sx0 = ox * w / tw;
            let sx1 = ((ox + 1) * w / tw).max(sx0 + 1).min(w);
            let (mut sr, mut sg, mut sb, mut sa, mut n) = (0u64, 0u64, 0u64, 0u64, 0u64);
            for sy in sy0..sy1 {
                for sx in sx0..sx1 {
                    let i = ((sy * w + sx) * 4) as usize;
                    let a = u64::from(rgba[i + 3]);
                    sr += u64::from(rgba[i]) * a;
                    sg += u64::from(rgba[i + 1]) * a;
                    sb += u64::from(rgba[i + 2]) * a;
                    sa += a;
                    n += 1;
                }
            }
            let o = ((oy * tw + ox) * 4) as usize;
            // `sa == 0` ⇒ o bloco era todo transparente; a cor fica a 0 (já zerada), que é o que um
            // texel transparente de miniatura deve carregar.
            if let (Some(r), Some(g), Some(b)) =
                (sr.checked_div(sa), sg.checked_div(sa), sb.checked_div(sa))
            {
                out[o] = r as u8;
                out[o + 1] = g as u8;
                out[o + 2] = b as u8;
            }
            out[o + 3] = (sa / n.max(1)) as u8;
        }
    }
    (std::sync::Arc::new(out), tw, th)
}

#[cfg(test)]
mod probe {
    /// ⭐ **O instrumento que produziu a tabela do [`super::reduce`]** — `#[ignore]` porque mede um
    /// relógio, e um relógio sob o fan-out do portão de fecho não mede nada (§5.0 do `CLAUDE.md`).
    ///
    /// Corra-o sozinho, em **release** e com a máquina calma:
    /// `cargo test --release -p ph2d-host-desktop --bins -- --ignored --nocapture measure_thumbnail_cost`
    ///
    /// ⚠️ **Ele não é um gate** — não tem barra e não reprova. *Uma regra sem instrumento é uma
    /// nota que envelhece*, e é para a tabela poder ser reconferida que ele fica.
    #[test]
    #[ignore]
    fn measure_thumbnail_cost() {
        for side in [256u32, 512, 1024, 2048, 4096] {
            let rgba = vec![0x80u8; (side as usize) * (side as usize) * 4];
            let t0 = std::time::Instant::now();
            let (_bytes, w, h) = super::reduce(&rgba, side, side);
            let dt = t0.elapsed();
            eprintln!(
                "[thumb] {side}x{side} -> {w}x{h}  {:.3} ms",
                dt.as_secs_f64() * 1000.0
            );
        }
    }
}
