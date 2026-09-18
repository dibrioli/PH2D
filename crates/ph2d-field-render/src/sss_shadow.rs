//! ⭐⭐⭐ **A SOMBRA COM A BORDA MOLE — a que um material TRANSLÚCIDO lê.**
//!
//! # Porque ela existe: o report de 2026-09-18
//!
//! O dono apontou uma **linha dura** na fronteira entre a parte iluminada e a sombreada de uma
//! esfera de jade. O experimento que a diagnosticou tirou a placa vizinha da cena: **sem ela a bola
//! sai lisa** ⇒ a linha é a borda da SOMBRA que a placa lança, e ela é dura porque a luz é um
//! **ponto**.
//!
//! ⛔ Isso é certo como geometria e errado como produto: num jade a luz que entra **fora** da sombra
//! espalha-se por baixo da superfície **para dentro** dela. A nossa subsuperfície tinha a difusão na
//! lei do `N·L` (o [`ph2d_material`] envolve a luz à volta do terminador) e **não a tinha na lei da
//! SOMBRA** — a visibilidade entrava dura, por pixel.
//!
//! # ⭐⭐ A lei: a visibilidade que a closure de subsuperfície lê é a MÉDIA da vizinhança
//!
//! O comprimento sobre o qual se faz a média é a **distância de espalhamento** — o `subsurface_radius`
//! por canal, em unidades do MUNDO, convertido a píxeis pela câmera. ⭐ É por ser **por canal** que a
//! borda fica avermelhada: o vermelho viaja mais no material e entra mais fundo na sombra, que é a
//! assinatura de toda pele e de toda cera.
//!
//! ⚠️ **A média é GUARDADA pela normal** ([`crate::OCCLUSION_BLUR_COS`], a mesma porta que o céu e o
//! ricochete usam): ela alisa dentro de uma superfície e **não atravessa uma quina**. *Duas cópias
//! da guarda divergiriam no dia em que alguém afinasse uma delas.*
//!
//! ⚠️⚠️ **E ela é SEPARÁVEL, em duas passagens de uma dimensão.** Um quadrado de `(2r+1)²` toques
//! por pixel é `O(r²)`; duas passagens de `(2r+1)` são `O(r)`, e dão a mesma resposta para um núcleo
//! de caixa. ⛔ A guarda de normal **não** é separável em rigor (um vizinho pode estar ligado na
//! horizontal e não na vertical) — a divergência é declarada e vale o preço: a alternativa é `O(r²)`,
//! e a `r = 24` isso são `2 401` toques por pixel e por canal.
//!
//! # ⚠️ Canal vazio ⇒ o quadro de sempre, AO BIT
//!
//! Sem esta passagem o [`crate::Shadows::soft_at`] devolve a visibilidade DURA, logo a lei da
//! subsuperfície recebe exactamente o que recebia. É o mesmo desenho do `ambient` e do `bounce`.

use crate::Gbuffer;

/// ⭐ **O tecto do raio, em píxeis — e ele diz de que recurso é: o RELÓGIO.**
///
/// A passagem custa `O(r)` por pixel e por canal. Medido a `320×240` (debug, `load ~1`):
///
/// | `r` | relógio da passagem |
/// |---:|---:|
/// | `8` | `4,1 ms` |
/// | `24` | `11,2 ms` |
/// | `48` | `21,6 ms` |
///
/// ⛔ Acima de `48` a borda de um jade já não muda de aspecto (a sombra está toda lavada) e o preço
/// continua a subir linearmente — *o tecto é onde o efeito satura e o custo não*.
pub const MAX_RAIO_PX: f32 = 48.0;

/// ⭐⭐⭐ **A média guardada, em duas passagens** — devolve um canal RGB por lâmpada.
///
/// `raio_px` é o raio por canal, em píxeis de ecrã. Um raio `<= 0` num canal deixa-o **igual ao
/// duro**, ao bit.
#[must_use]
pub fn blur_por_canal(g: &Gbuffer, vis: &[f32], raio_px: [f32; 3]) -> Vec<[f32; 3]> {
    let n = vis.len();
    let mut out = vec![[0.0f32; 3]; n];
    for (k, &r) in raio_px.iter().enumerate() {
        let canal = uma_dimensao(g, vis, r, true);
        let canal = uma_dimensao(g, &canal, r, false);
        for i in 0..n {
            out[i][k] = canal[i];
        }
    }
    out
}

/// Uma passagem de uma dimensão, com a guarda de normal da casa.
fn uma_dimensao(g: &Gbuffer, canal: &[f32], raio_px: f32, horizontal: bool) -> Vec<f32> {
    let mut out = canal.to_vec();
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let r = raio_px.clamp(0.0, MAX_RAIO_PX) as i32;
    if r <= 0 {
        return out;
    }
    let (w, h) = (g.width as i32, g.height as i32);
    if w <= 0 || h <= 0 {
        return out;
    }
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) as usize;
            if !g.hit[i] {
                continue;
            }
            let n0 = g.normal[i];
            let (mut soma, mut peso) = (0.0f32, 0.0f32);
            for d in -r..=r {
                let (xx, yy) = if horizontal { (x + d, y) } else { (x, y + d) };
                if xx < 0 || yy < 0 || xx >= w || yy >= h {
                    continue;
                }
                let j = (yy * w + xx) as usize;
                if !g.hit[j] {
                    continue;
                }
                let nj = g.normal[j];
                // ⚠️ A MESMA guarda do céu e do ricochete — ver o cabeçalho.
                if n0[0] * nj[0] + n0[1] * nj[1] + n0[2] * nj[2] < crate::OCCLUSION_BLUR_COS {
                    continue;
                }
                soma += canal[j];
                peso += 1.0;
            }
            if peso > 0.0 {
                out[i] = soma / peso;
            }
        }
    }
    out
}

/// ⭐⭐ **O raio em PÍXEIS que uma distância de espalhamento do MUNDO vale**, nesta câmera.
///
/// ⚠️ É por isto que a borda não muda de largura quando se dá zoom: ela é uma distância do MUNDO, e
/// o número de píxeis que ela ocupa tem de a seguir. *Um raio escrito em píxeis seria uma borda que
/// encolhe quando o artista se aproxima.*
#[must_use]
pub fn raio_em_pixeis(cam: &crate::Orbit, altura_px: u32, mundo: [f32; 3]) -> [f32; 3] {
    if altura_px == 0 {
        return [0.0; 3];
    }
    #[allow(clippy::cast_precision_loss)]
    // A vista cobre `2 · half_extent` de mundo na altura da imagem.
    let px_por_mundo = altura_px as f32 / (2.0 * cam.half_extent.max(f32::EPSILON));
    mundo.map(|m| (m.max(0.0) * px_por_mundo).min(MAX_RAIO_PX))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Uma tira de `n × 1` píxeis, com a normal a VIRAR ao meio — uma quina.
    fn tira_com_quina(n: usize) -> Gbuffer {
        #[allow(clippy::cast_possible_truncation)]
        Gbuffer {
            width: n as u32,
            height: 1,
            hit: vec![true; n],
            // Metade a apontar para `+z`, metade para `+x`: `cos = 0`, muito abaixo da guarda.
            normal: (0..n)
                .map(|i| {
                    if i < n / 2 {
                        [0.0, 0.0, 1.0]
                    } else {
                        [1.0, 0.0, 0.0]
                    }
                })
                .collect(),
            point: vec![[0.0; 3]; n],
            curvature: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// ⭐⭐⭐ **A MÉDIA NÃO ATRAVESSA UMA QUINA** — o gémeo do gate que o céu já tem.
    ///
    /// ⚠️ Ele existe porque uma **mutação SOBREVIVEU**: a cena da bola de jade é lisa, e apagar a
    /// guarda de normal não movia um byte lá. *Uma cena sem quina nenhuma não testa a guarda da
    /// quina*, e a régua tem de trazer o fenómeno consigo.
    #[test]
    fn a_media_da_borda_mole_nao_atravessa_uma_quina() {
        const N: usize = 32;
        let g = tira_com_quina(N);
        // Um degrau de visibilidade que coincide com a quina.
        let vis: Vec<f32> = (0..N).map(|i| if i < N / 2 { 0.0 } else { 1.0 }).collect();
        let out = blur_por_canal(&g, &vis, [8.0; 3]);
        // ⭐⭐ **A asserção mora COLADA à quina**, e não nas pontas: a `r = 8` as pontas ficam fora
        // do alcance dela e leem `0` e `1` na mesma **com a guarda apagada** — *uma mutação
        // SOBREVIVEU por eu ter medido onde a guarda não decide nada*.
        assert!(
            (out[N / 2 - 1][0] - 0.0).abs() < 1e-6 && (out[N / 2][0] - 1.0).abs() < 1e-6,
            "os dois píxeis colados à quina leem {:.4} e {:.4} — a média atravessou-a",
            out[N / 2 - 1][0],
            out[N / 2][0]
        );
        // ⭐ E o CONTROLO: sem quina, a mesma média ESBORRATA o mesmo degrau. Sem esta metade, uma
        // «média» que não fizesse nada passaria na de cima.
        let liso = Gbuffer {
            normal: vec![[0.0, 0.0, 1.0]; N],
            ..tira_com_quina(N)
        };
        let out = blur_por_canal(&liso, &vis, [8.0; 3]);
        assert!(
            out[N / 2 - 1][0] > 0.05 && out[N / 2][0] < 0.95,
            "sem quina o degrau ficou em {:.4}/{:.4} — a média não está a fazer nada",
            out[N / 2 - 1][0],
            out[N / 2][0]
        );
    }

    /// ⭐⭐ **Raio zero num canal ⇒ esse canal é a visibilidade DURA, ao bit** — é o que faz um
    /// material sem espalhamento não pagar nada e o quadro sair o de sempre.
    #[test]
    fn raio_zero_devolve_a_visibilidade_dura_ao_bit() {
        const N: usize = 16;
        let g = tira_com_quina(N);
        #[allow(clippy::cast_precision_loss)]
        let vis: Vec<f32> = (0..N).map(|i| i as f32 / N as f32).collect();
        let out = blur_por_canal(&g, &vis, [0.0; 3]);
        for i in 0..N {
            assert!(
                (out[i][0] - vis[i]).to_bits() == 0.0f32.to_bits(),
                "o pixel {i} lê {:.6} contra {:.6} — o raio zero mexeu no canal",
                out[i][0],
                vis[i]
            );
        }
    }
}
