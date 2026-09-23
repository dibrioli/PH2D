//! ⭐⭐ **As passagens POR PIXEL da composição da pilha — em linhas disjuntas, na equipa de threads.**
//!
//! A [`super::composite_acumulado`] decide O QUE cada camada faz; este módulo é COMO os píxeis de uma
//! passagem são percorridos. Cortado do irmão por responsabilidade (e pelo tecto de LOC dele).
//!
//! # Porque em paralelo, e porque isto é byte-idêntico (ADR-0172, com os invariantes do ADR-0109)
//!
//! Medido no build do PRODUTO (exemplo `mede_a_pilha`, a pilha do dono no Composite Brush, amostrada
//! por `gdb`): a thread principal passava o quadro inteiro a compor, em série, e as passagens por
//! pixel (tinta, borracha) eram a parte que não tinha equipa nenhuma — o borrão e o depósito em
//! banda já corriam nela. Cada pixel de saída destas passagens é função PURA de entradas que a
//! passagem só LÊ (o plano da camada, o `pre`) e do próprio pixel da tela; nenhuma soma atravessa
//! píxeis, nenhuma aleatoriedade entra ⇒ os três invariantes do ADR-0109 valem à letra, e a saída é
//! a mesma para qualquer número de threads e qualquer escalonamento.
//!
//! # E sem a biblioteca matemática no laço
//!
//! ⛔ Quando isto foi escrito o repo compilava para o `x86-64` BASE (sem SSE4.1), onde `f32::round`
//! **não** é uma instrução: é uma chamada ao `roundf` de software do `compiler_builtins`. Medido na
//! mesma amostragem: o arredondamento era a maior folha do laço da tinta. [`redondo_u8`] dá o MESMO
//! byte com um truncamento (`cvttss2si`) e uma comparação — gate de equivalência ao bit ao lado.
//! ⚠️ Desde o ADR-0174 o produto compila para `x86-64-v2`, que TEM `roundss`: a vantagem desta
//! função encolheu para a do `clamp` evitado, e ela FICA porque continua exacta e porque um build
//! com `RUSTFLAGS` definido à mão perde o nível (os `RUSTFLAGS` substituem o `.cargo/config.toml`).

use super::Region;
use rayon::prelude::*;

/// `0..255` → `0..1`, como `f32::from(b) / 255.0` — a tabela é essa divisão feita uma vez por valor,
/// logo o número é o mesmo, bit a bit.
const DEC: [f32; 256] = {
    let mut t = [0.0f32; 256];
    let mut i = 0;
    while i < 256 {
        t[i] = i as f32 / 255.0;
        i += 1;
    }
    t
};

/// `0..255` → `0..1`.
#[inline]
pub(super) fn dec(b: u8) -> f32 {
    DEC[b as usize]
}

/// `0..1` → `0..255`, com o mesmo arredondamento do resto da casa.
#[inline]
pub(super) fn enc(v: f32) -> u8 {
    redondo_u8(v * 255.0)
}

/// **`x.round().clamp(0.0, 255.0) as u8`, sem chamar o `roundf`.**
///
/// ⭐ Exacto em todo `f32`: abaixo de `0` (e em `−0` e `NaN`) as duas dão `0`; de `255` para cima as
/// duas dão `255`; entre elas `x < 2²⁴`, logo `x − ⌊x⌋` é EXACTO em `f32` e `round` (meio para
/// longe do zero) é o truncamento mais um quando a fracção chega a `½`.
#[inline]
pub(super) fn redondo_u8(x: f32) -> u8 {
    if x >= 255.0 {
        return 255;
    }
    if x > 0.0 {
        let t = x as u32;
        return (t + u32::from(x - t as f32 >= 0.5)) as u8;
    }
    // `x ≤ 0`, `−0` e `NaN` — as duas dão `0`.
    0
}

/// Percorrer as linhas de `r` em paralelo: `linha(i, pixeis)` recebe o índice da linha DENTRO de `r`
/// e a fatia RGBA dela na tela (só a largura de `r`).
fn por_linhas<F>(buf: &mut [u8], stride: usize, r: Region, linha: F)
where
    F: Fn(usize, &mut [u8]) + Sync + Send,
{
    let (y0, h) = (r.y as usize, r.h as usize);
    let (x0, rw) = (r.x as usize * 4, r.w as usize * 4);
    buf[y0 * stride..(y0 + h) * stride]
        .par_chunks_mut(stride)
        .enumerate()
        // Uma linha é pouco trabalho para uma tarefa; oito é a ordem de grandeza de um bloco de
        // cache e deixa `h / 8` tarefas — muitas mais do que núcleos em toda região real.
        .with_min_len(8)
        .for_each(|(i, l)| linha(i, &mut l[x0..x0 + rw]));
}

/// **A TINTA de uma camada Brush** sobre `r`: `tela ← blend(tela, plano)`, pixel a pixel.
pub(super) fn tinta(
    buf: &mut [u8],
    plano: &[u8],
    stride: usize,
    r: Region,
    blend: ph2d_painter_brush::BrushBlend,
    alpha_locked: bool,
) {
    por_linhas(buf, stride, r, |i, linha| {
        let base = (r.y as usize + i) * stride + r.x as usize * 4;
        let p = &plano[base..base + linha.len()];
        for (px, pl) in linha
            .as_chunks_mut::<4>()
            .0
            .iter_mut()
            .zip(p.as_chunks::<4>().0)
        {
            let a = dec(pl[3]);
            if a <= 0.0 {
                continue;
            }
            let dst = [dec(px[0]), dec(px[1]), dec(px[2]), dec(px[3])];
            let mut out =
                ph2d_painter_brush::blend_over(blend, dst, [dec(pl[0]), dec(pl[1]), dec(pl[2])], a);
            if alpha_locked {
                out[3] = dst[3];
            }
            for k in 0..4 {
                px[k] = enc(out[k]);
            }
        }
    });
}

/// **A BORRACHA** sobre `r`, com a cobertura `c = 1 − α` medida no escudo (`plano`). As duas leis
/// são as da [`super::composite_acumulado`]: `Tudo` come o alfa, `Traco` devolve o `pre`.
pub(super) fn borracha(
    buf: &mut [u8],
    plano: &[u8],
    pre: &[u8],
    stride: usize,
    r: Region,
    escopo: super::composite::EscopoDaBorracha,
) {
    use super::composite::EscopoDaBorracha;
    por_linhas(buf, stride, r, |i, linha| {
        let base = (r.y as usize + i) * stride + r.x as usize * 4;
        let (pl, pr) = (
            &plano[base..base + linha.len()],
            &pre[base..base + linha.len()],
        );
        for ((px, e), p) in linha
            .as_chunks_mut::<4>()
            .0
            .iter_mut()
            .zip(pl.as_chunks::<4>().0)
            .zip(pr.as_chunks::<4>().0)
        {
            // O escudo nasce opaco e o depósito come-lhe o alfa ⇒ `c = 1 − α`.
            let c = 1.0 - dec(e[3]);
            if c <= 0.0 {
                continue;
            }
            match escopo {
                EscopoDaBorracha::Tudo => {
                    px[3] = enc(dec(px[3]) * (1.0 - c));
                }
                EscopoDaBorracha::Traco => {
                    for k in 0..4 {
                        let d = f32::from(px[k]);
                        let q = f32::from(p[k]);
                        px[k] = redondo_u8(d + (q - d) * c);
                    }
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A régua de antes, escrita como estava.
    fn antes(x: f32) -> u8 {
        x.round().clamp(0.0, 255.0) as u8
    }

    /// ⭐ **O arredondamento sem biblioteca dá o MESMO byte** — nas fronteiras onde ele pode falhar
    /// (cada meio inteiro e cada inteiro de `0` a `256`, ±`4096` ulps à volta), nos extremos e nos
    /// valores que não são números. *Um arredondamento que erra só no `½` erra exactamente onde uma
    /// amostragem uniforme não olha.*
    #[test]
    fn o_arredondamento_sem_biblioteca_da_o_mesmo_byte() {
        let mut vistos = 0u64;
        for k in 0..=512u32 {
            let centro = k as f32 * 0.5;
            let bits = centro.to_bits() as i64;
            for d in -4096i64..=4096 {
                let b = bits + d;
                if b < 0 {
                    continue;
                }
                let x = f32::from_bits(b as u32);
                assert_eq!(redondo_u8(x), antes(x), "x = {x:e} (bits {b:#x})");
                assert_eq!(redondo_u8(-x), antes(-x), "x = {:e}", -x);
                vistos += 2;
            }
        }
        for x in [
            f32::NAN,
            -f32::NAN,
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::MAX,
            f32::MIN,
            0.0,
            -0.0,
            f32::MIN_POSITIVE,
            1e-45,
            254.5,
            254.49998,
            255.0,
            255.5,
            1e9,
        ] {
            assert_eq!(redondo_u8(x), antes(x), "x = {x:e}");
        }
        // CONTROLO: a varredura tem de ter olhado para o que diz que olhou.
        assert!(vistos > 4_000_000, "a varredura viu só {vistos} valores");
        // E toda saída de um `v ∈ [0,1]` que a composição de facto produz, `v·255`.
        for b in 0..=u16::MAX {
            let v = f32::from(b) / f32::from(u16::MAX);
            assert_eq!(enc(v), antes(v * 255.0), "v = {v}");
        }
    }

    /// Bytes pseudo-aleatórios reprodutíveis (LCG) — a fixtura tem de ter alfa parcial, zero e cheio,
    /// senão os ramos `a <= 0` e `c <= 0` não são exercitados.
    fn ruido(n: usize, semente: u32) -> Vec<u8> {
        let mut s = semente;
        (0..n)
            .map(|_| {
                s = s.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                let b = (s >> 24) as u8;
                // Um quarto dos bytes nos extremos, para os ramos de saída rápida.
                match b & 3 {
                    0 => 0,
                    1 => 255,
                    _ => b,
                }
            })
            .collect()
    }

    const W: u32 = 97;
    const H: u32 = 61;
    /// Não começa em `(0,0)` e tem linhas que chegam para muitas tarefas (`with_min_len(8)`).
    const R: Region = Region {
        x: 5,
        y: 3,
        w: 80,
        h: 50,
    };

    /// ⭐⭐ **A TINTA em paralelo dá o MESMO byte que o laço em série de antes** — o laço de
    /// referência é o que a [`super::super::composite_acumulado`] tinha, copiado à letra, com a
    /// divisão e o `round` da biblioteca. Três modos de mistura (o de fábrica, um de cor e o que
    /// apaga) e o trinco de alfa nos dois estados. E o que está FORA da região não se move.
    #[test]
    fn a_tinta_em_paralelo_da_o_byte_da_serie() {
        use ph2d_painter_brush::BrushBlend;
        let stride = W as usize * 4;
        let plano = ruido(stride * H as usize, 7);
        for blend in [
            BrushBlend::Mix,
            BrushBlend::Multiply,
            BrushBlend::EraseAlpha,
        ] {
            for trinco in [false, true] {
                let tela = ruido(stride * H as usize, 11);
                let mut serie = tela.clone();
                let r = R;
                for row in 0..r.h as usize {
                    let base = (r.y as usize + row) * stride + r.x as usize * 4;
                    for col in 0..r.w as usize {
                        let i = base + col * 4;
                        let d = |b: u8| f32::from(b) / 255.0;
                        let a = d(plano[i + 3]);
                        if a <= 0.0 {
                            continue;
                        }
                        let dst = [
                            d(serie[i]),
                            d(serie[i + 1]),
                            d(serie[i + 2]),
                            d(serie[i + 3]),
                        ];
                        let mut out = ph2d_painter_brush::blend_over(
                            blend,
                            dst,
                            [d(plano[i]), d(plano[i + 1]), d(plano[i + 2])],
                            a,
                        );
                        if trinco {
                            out[3] = dst[3];
                        }
                        for k in 0..4 {
                            serie[i + k] = antes(out[k] * 255.0);
                        }
                    }
                }
                let mut par = tela.clone();
                tinta(&mut par, &plano, stride, r, blend, trinco);
                assert!(
                    par == serie,
                    "{blend:?} · trinco {trinco}: a rota paralela mudou bytes"
                );
                // CONTROLO: a fixtura pintou alguma coisa, senão a igualdade não diz nada. ⚠️ Menos
                // no par `EraseAlpha` + trinco, que por LEI não muda um byte (o apagar só mexe no
                // alfa, e o trinco devolve-o) — ali a igualdade é o que se afirma.
                if !(blend == BrushBlend::EraseAlpha && trinco) {
                    assert!(
                        par != tela,
                        "{blend:?}: a passagem não mudou um byte — fixtura inerte"
                    );
                }
            }
        }
    }

    /// ⭐⭐ **A BORRACHA em paralelo dá o MESMO byte que o laço em série de antes**, nos dois escopos.
    #[test]
    fn a_borracha_em_paralelo_da_o_byte_da_serie() {
        use super::super::composite::EscopoDaBorracha;
        let stride = W as usize * 4;
        let escudo = ruido(stride * H as usize, 3);
        let pre = ruido(stride * H as usize, 5);
        for escopo in [EscopoDaBorracha::Tudo, EscopoDaBorracha::Traco] {
            let tela = ruido(stride * H as usize, 13);
            let mut serie = tela.clone();
            let r = R;
            for row in 0..r.h as usize {
                let base = (r.y as usize + row) * stride + r.x as usize * 4;
                for col in 0..r.w as usize {
                    let i = base + col * 4;
                    let c = 1.0 - f32::from(escudo[i + 3]) / 255.0;
                    if c <= 0.0 {
                        continue;
                    }
                    match escopo {
                        EscopoDaBorracha::Tudo => {
                            let a = f32::from(serie[i + 3]) / 255.0;
                            serie[i + 3] = antes(a * (1.0 - c) * 255.0);
                        }
                        EscopoDaBorracha::Traco => {
                            for k in 0..4 {
                                let d = f32::from(serie[i + k]);
                                let p = f32::from(pre[i + k]);
                                serie[i + k] = (d + (p - d) * c).round().clamp(0.0, 255.0) as u8;
                            }
                        }
                    }
                }
            }
            let mut par = tela.clone();
            borracha(&mut par, &escudo, &pre, stride, r, escopo);
            assert!(par == serie, "{escopo:?}: a rota paralela mudou bytes");
            assert!(
                par != tela,
                "{escopo:?}: a passagem não mudou um byte — fixtura inerte"
            );
        }
    }

    /// A tabela é a divisão, byte a byte.
    #[test]
    fn a_tabela_e_a_divisao() {
        for b in 0..=255u8 {
            assert_eq!(
                dec(b).to_bits(),
                (f32::from(b) / 255.0).to_bits(),
                "b = {b}"
            );
        }
    }
}
