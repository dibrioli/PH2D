//! **As passagens VERTICAIS do núcleo de três caixas** — a fundida por bandas e a separada que é
//! o oráculo dela. Cortadas do [`super`] pelo tecto de LOC (`848` contra `700`), por
//! responsabilidade: lá mora a lei (as larguras, a horizontal, a porta), aqui o percurso vertical.

#[cfg(test)]
use super::BANDAS_PERCORRIDAS;

/// **As TRÊS passagens verticais, uma BANDA de colunas de cada vez** (2026-09-22).
///
/// ⭐⭐ **A fusão da vertical NÃO é «uma coluna de cada vez»** — isso leria a memória com passo `w`
/// e pagaria a cache, que é precisamente a razão por que a [`caixa_v`] usa um acumulador de LINHA.
/// O que ela faz é correr as três passagens **dentro da banda**, com os dois intermédios do tamanho
/// da banda em vez do tamanho da imagem: o acesso continua contíguo e o buffer que é atravessado
/// três vezes passa a caber na cache.
///
/// ⭐⭐⭐ **E é BYTE-IDÊNTICA para QUALQUER partição, pelo argumento que a [`caixa_v`] já escreve:**
/// o acumulador `acc[i]` só toca a coluna `c0 + i`, logo a largura da banda é ordem de laço e nunca
/// aritmética. Cada coluna recebe exactamente a mesma sequência de adições, na mesma ordem, com os
/// mesmos `inv`. O gate `as_tres_verticais_fundidas_dao_o_mesmo_f32` prova-o contra as três
/// chamadas separadas sobre um corpus de tamanhos e raios.
///
/// ⚠️ **A largura da banda é MEDIDA e é de CACHE, não de threads** (ver [`LARGURA_DA_BANDA_FUNDIDA`]):
/// com a banda larga os intermédios voltam a ser lidos da DRAM e a fusão não compra nada; com ela
/// estreita de mais o laço interno fica curto de mais para amortizar o percurso por linha.
pub(super) fn caixa_v3(
    src: &[[f32; 4]],
    w: usize,
    h: usize,
    raios: [usize; 3],
    largura_da_banda: usize,
    paralelo: bool,
) -> (Vec<[f32; 4]>, usize) {
    let r_total: usize = raios.iter().sum();
    if r_total == 0 {
        return (src.to_vec(), h);
    }
    let oh = h - 2 * r_total;
    let mut out = vec![[0f32; 4]; w * oh];

    // UMA passagem vertical sobre uma banda: lê `largura` colunas a partir de `c0` numa grelha de
    // passo `stride`, e escreve COMPACTO com largura `largura`. É o laço da `caixa_v` com a origem
    // parametrizada — o que permite encadeá-la sobre os próprios intermédios, que são compactos.
    let passo = |ent: &[[f32; 4]],
                 stride: usize,
                 c0: usize,
                 largura: usize,
                 hh: usize,
                 r: usize,
                 acc: &mut Vec<[f32; 4]>,
                 dest: &mut [[f32; 4]]| {
        if r == 0 {
            for j in 0..hh {
                dest[j * largura..][..largura].copy_from_slice(&ent[j * stride + c0..][..largura]);
            }
            return;
        }
        let lado = 2 * r + 1;
        #[allow(clippy::cast_precision_loss)]
        let inv = 1.0 / lado as f32;
        acc.clear();
        acc.resize(largura, [0f32; 4]);
        for j in 0..lado {
            for i in 0..largura {
                let sv = ent[j * stride + c0 + i];
                for c in 0..4 {
                    acc[i][c] += sv[c];
                }
            }
        }
        let oh_l = hh - 2 * r;
        for j in 0..oh_l {
            for i in 0..largura {
                for c in 0..4 {
                    dest[j * largura + i][c] = acc[i][c] * inv;
                }
            }
            if j + 1 < oh_l {
                for i in 0..largura {
                    let sai = ent[j * stride + c0 + i];
                    let entra = ent[(j + lado) * stride + c0 + i];
                    for c in 0..4 {
                        acc[i][c] += entra[c] - sai[c];
                    }
                }
            }
        }
    };

    let h1 = h - 2 * raios[0];
    let h2 = h1 - 2 * raios[1];
    // ⚠️ Os rascunhos são por TRABALHADOR e nunca por banda — a mesma lei que a `caixa_h3` pagou.
    type Rascunho = (Vec<[f32; 4]>, Vec<[f32; 4]>, Vec<[f32; 4]>, Vec<[f32; 4]>);
    let banda = |r: &mut Rascunho, c0: usize, largura: usize, dest: &mut [&mut [[f32; 4]]]| {
        // ⚠️ A CONTA, e não o valor: a identidade ao bit vale para toda partição, logo nenhuma
        //    régua de VALOR consegue ver alguém cravar uma banda só — e uma banda só (a largura
        //    CHEIA) mediu entre `0,45×` e `1,41×` das três passagens separadas conforme a corrida:
        //    nunca melhor do que a banda de cache, e às vezes pior do que antes da fusão.
        #[cfg(test)]
        BANDAS_PERCORRIDAS.with(|c| c.set(c.get() + 1));
        let (t1, t2, t3, acc) = r;
        t1.clear();
        t1.resize(largura * h1, [0f32; 4]);
        t2.clear();
        t2.resize(largura * h2, [0f32; 4]);
        t3.clear();
        t3.resize(largura * oh, [0f32; 4]);
        passo(src, w, c0, largura, h, raios[0], acc, t1);
        passo(t1, largura, 0, largura, h1, raios[1], acc, t2);
        passo(t2, largura, 0, largura, h2, raios[2], acc, t3);
        for (j, linha) in dest.iter_mut().enumerate() {
            linha.copy_from_slice(&t3[j * largura..][..largura]);
        }
    };

    // As bandas são de CACHE: `w / largura_da_banda`, arredondado para cima, e nunca zero.
    let nb = w.div_ceil(largura_da_banda.max(1)).max(1).min(w.max(1));
    let base = w / nb;
    let resto_col = w % nb;
    let larguras: Vec<usize> = (0..nb).map(|b| base + usize::from(b < resto_col)).collect();
    let mut bandas: Vec<Vec<&mut [[f32; 4]]>> =
        larguras.iter().map(|_| Vec::with_capacity(oh)).collect();
    for linha in out.chunks_mut(w) {
        let mut resto = linha;
        for (b, &lw) in larguras.iter().enumerate() {
            let (esq, dir) = resto.split_at_mut(lw);
            bandas[b].push(esq);
            resto = dir;
        }
    }
    let mut c = 0usize;
    let inicios: Vec<usize> = larguras
        .iter()
        .map(|lw| {
            let v = c;
            c += lw;
            v
        })
        .collect();
    let vazio = || -> Rascunho { (Vec::new(), Vec::new(), Vec::new(), Vec::new()) };
    if paralelo {
        use rayon::prelude::*;
        bandas
            .par_iter_mut()
            .zip(inicios)
            .zip(larguras)
            .for_each_init(vazio, |r, ((dest, c0), lw)| banda(r, c0, lw, dest));
    } else {
        let mut r = vazio();
        for ((dest, c0), lw) in bandas.iter_mut().zip(inicios).zip(larguras) {
            banda(&mut r, c0, lw, dest);
        }
    }
    (out, oh)
}

/// Uma passagem de caixa VERTICAL por soma corrente. ⚠️ O acumulador é uma LINHA inteira e desliza
/// para baixo — uma coluna de cada vez leria a memória com passo `w` e pagaria a cache.
///
/// ⚠️ **Ela é o ORÁCULO da [`caixa_v3`] E o caminho da porta de bissecção** — a mesma razão que a
/// [`caixa_h`] já traz escrita.
pub(super) fn caixa_v(
    src: &[[f32; 4]],
    w: usize,
    h: usize,
    r: usize,
    fatias: usize,
) -> (Vec<[f32; 4]>, usize) {
    if r == 0 {
        return (src.to_vec(), h);
    }
    let oh = h - 2 * r;
    let lado = 2 * r + 1;
    #[allow(clippy::cast_precision_loss)]
    let inv = 1.0 / lado as f32;
    let mut out = vec![[0f32; 4]; w * oh];

    // Uma BANDA de colunas `[c0, c0 + largura)`, ao longo de todas as linhas de saída.
    let banda = |c0: usize, dest: &mut [&mut [[f32; 4]]]| {
        let largura = dest.first().map_or(0, |d| d.len());
        let mut acc = vec![[0f32; 4]; largura];
        for j in 0..lado {
            for i in 0..largura {
                let sv = src[j * w + c0 + i];
                for c in 0..4 {
                    acc[i][c] += sv[c];
                }
            }
        }
        for j in 0..oh {
            for i in 0..largura {
                for c in 0..4 {
                    dest[j][i][c] = acc[i][c] * inv;
                }
            }
            if j + 1 < oh {
                for i in 0..largura {
                    let sai = src[j * w + c0 + i];
                    let entra = src[(j + lado) * w + c0 + i];
                    for c in 0..4 {
                        acc[i][c] += entra[c] - sai[c];
                    }
                }
            }
        }
    };

    if fatias <= 1 {
        let mut linhas: Vec<&mut [[f32; 4]]> = out.chunks_mut(w).collect();
        banda(0, &mut linhas);
        return (out, oh);
    }

    // ⛔⛔ **A banda tem de ser de COLUNAS, e é isso que a torna byte-idêntica.** Com bandas de
    //     LINHAS cada uma teria de re-semear o acumulador somando `lado` linhas de fresco, e esse
    //     `f32` **não é** o que a soma corrida acumulou até ali — a mesma não-associatividade que
    //     a cerca do `rayon` desta crate já nomeia para o depósito das arestas. A coluna, essa,
    //     recebe exactamente a mesma sequência de adições nas duas rotas.
    //
    // ⚠️ `out` é row-major, logo uma banda de colunas **não é contígua**: as fatias disjuntas
    //     saem de partir cada LINHA com `split_at_mut` (a crate é `forbid(unsafe_code)`).
    let nb = fatias.min(w.max(1));
    let base = w / nb;
    let resto_col = w % nb;
    let larguras: Vec<usize> = (0..nb).map(|b| base + usize::from(b < resto_col)).collect();
    let mut bandas: Vec<Vec<&mut [[f32; 4]]>> =
        larguras.iter().map(|_| Vec::with_capacity(oh)).collect();
    for linha in out.chunks_mut(w) {
        let mut resto = linha;
        for (b, &lw) in larguras.iter().enumerate() {
            let (esq, dir) = resto.split_at_mut(lw);
            bandas[b].push(esq);
            resto = dir;
        }
    }
    let mut c0 = 0usize;
    let inicios: Vec<usize> = larguras
        .iter()
        .map(|lw| {
            let v = c0;
            c0 += lw;
            v
        })
        .collect();
    {
        use rayon::prelude::*;
        bandas
            .par_iter_mut()
            .zip(inicios)
            .for_each(|(dest, c0)| banda(c0, dest));
    }
    (out, oh)
}
