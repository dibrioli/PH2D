//! O núcleo de **TRÊS CAIXAS** do blur — o gémeo barato do binomial do irmão [`super::blur`].
//!
//! ⭐ Ele existe por ordem do dono (2026-09-20): *«veja se abaixando a qualidade do blur não fica
//! bem mais leve. Mas só no Blur do composite. O Blur como ferramenta isolada não deve ser
//! modificado.»* — logo ele é pedido por **um** sítio de toda a crate da ferramenta, o laço da
//! pilha, e há censo a afirmá-lo (`o_nucleo_de_caixa_e_do_blur_da_pilha_e_so_dele`).
//!
//! **A lei:** três passagens de caixa por somas correntes aproximam uma gaussiana, e as larguras
//! saem de **CASAMENTO DE VARIÂNCIA** com o binomial que substituem (`σ² = k/2`) — é isso que faz
//! dela a *mesma quantidade de borrão* e não um borrão mais fraco disfarçado de optimização. O
//! custo é `~6` ops/pixel **independente de `k`**, contra `2(2k+1)` taps.
//!
//! **MEDIDO** (`--release`, canvas `1024²`, traço de 720 px, **pareado**, `load 3,4`–`4,0`):
//!
//! | raio | binomial | caixa | ganho p50 (p10..p90) | pior byte | média |
//! |---|---|---|---|---|---|
//! | 12 | `4,03` | `4,09` | **`×0,98`** (`0,95`..`1,00`) | `1` | `0,001` |
//! | 24 | `8,84` | `7,04` | `×1,26` (`1,23`..`1,27`) | `3` | `0,017` |
//! | 48 | `24,21` | `13,31` | `×1,83` (`1,76`..`1,86`) | `1` | `0,001` |
//! | 96 | `84,23` | `26,28` | **`×3,19`** (`3,09`..`3,26`) | `1` | `0,007` |
//!
//! ⛔⛔ **A leitura anterior dizia `×1,21` no raio 12 e estava ERRADA** — ela era `min(B)/min(A)` de
//! corridas SEPARADAS a `load 15`. Pareada, ali não há ganho nenhum: três passagens com avental
//! próprio custam o que um binomial de lado `9` custa, e a caixa só se paga a partir do raio `~24`.
//!
//! ⚠️ Este módulo nasceu de um **CORTE** do `blur.rs` (`822` linhas contra o tecto de `700`), nunca
//! de uma entrada nova no `FILE_OVERAGE_OK`.

use super::blur::src_coord;

/// As três larguras de caixa cuja variância somada mais se aproxima da do binomial de raio `k`.
///
/// Uma caixa de largura ímpar `w` tem variância `(w² − 1)/12`; três delas somam `Σ(wᵢ² − 1)/12`, e
/// o alvo é `σ² = k/2` (a variância do binomial de raio `k`). A largura ideal comum sai de
/// `3(w² − 1)/12 = k/2` ⇒ `w = √(1 + 2k)`, que quase nunca é um ímpar inteiro — por isso as três
/// caixas são **duas larguras vizinhas misturadas**, e a mistura é escolhida por MEDIÇÃO (a que
/// minimiza o erro de variância), nunca por arredondamento.
#[must_use]
pub(crate) fn box_radii(k: usize) -> [usize; 3] {
    #[allow(clippy::cast_precision_loss)]
    let alvo = k as f32 / 2.0;
    let ideal = (1.0 + 2.0 * k as f32).sqrt();
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let mut wl = ideal.floor() as usize;
    if wl.is_multiple_of(2) {
        wl = wl.saturating_sub(1);
    }
    let wl = wl.max(1);
    let wu = wl + 2;
    let var = |w: usize| {
        #[allow(clippy::cast_precision_loss)]
        let w = w as f32;
        (w * w - 1.0) / 12.0
    };
    let mut melhor = (f32::MAX, 0usize);
    for m in 0..=3usize {
        #[allow(clippy::cast_precision_loss)]
        let v = (3 - m) as f32 * var(wl) + m as f32 * var(wu);
        let erro = (v - alvo).abs();
        if erro < melhor.0 {
            melhor = (erro, m);
        }
    }
    let m = melhor.1;
    std::array::from_fn(|i| if i < m { (wu - 1) / 2 } else { (wl - 1) / 2 })
}

/// Uma passagem de caixa HORIZONTAL por soma corrente: a saída perde `r` de cada lado.
///
/// ⚠️ **`#[cfg(test)]` de propósito: desde a FUSÃO (2026-09-22) ela não tem chamador de produto** —
/// quem o produto corre é a [`caixa_h3`], que faz as três numa passagem sobre a memória. Ela FICA
/// porque é o **ORÁCULO** do gate `as_tres_horizontais_fundidas_dao_o_mesmo_f32`: a fusão prova-se
/// contra as três chamadas separadas, e apagá-la levaria a régua junto.
///
/// ⛔ Sem o `cfg`, ela é **código morto numa crate de biblioteca** e o CI reprova
/// (`build.warnings = "deny"`), que é a armadilha que o §5.0 do roteador já regista para os
/// `#[cfg(target_os)]`: *o `cargo check` local com `--all-targets` vê o uso do teste; o passe do CI
/// não corre com `cfg(test)` e vê um `fn` sem chamadores.*
#[cfg(test)]
fn caixa_h(
    src: &[[f32; 4]],
    w: usize,
    h: usize,
    r: usize,
    paralelo: bool,
) -> (Vec<[f32; 4]>, usize) {
    if r == 0 {
        return (src.to_vec(), w);
    }
    let ow = w - 2 * r;
    let lado = 2 * r + 1;
    #[allow(clippy::cast_precision_loss)]
    let inv = 1.0 / lado as f32;
    let mut out = vec![[0f32; 4]; ow * h];
    // ⭐ Uma LINHA de saída é função só da linha de entrada dela, e a ordem das somas dentro da
    //    linha não muda ⇒ o paralelo é **byte-idêntico**, e não «igual a menos de um epsilon».
    let linha = |j: usize, dest: &mut [[f32; 4]]| {
        let base = j * w;
        let mut acc = [0f32; 4];
        for i in 0..lado {
            let s = src[base + i];
            for c in 0..4 {
                acc[c] += s[c];
            }
        }
        for i in 0..ow {
            for c in 0..4 {
                dest[i][c] = acc[c] * inv;
            }
            if i + 1 < ow {
                let sai = src[base + i];
                let entra = src[base + i + lado];
                for c in 0..4 {
                    acc[c] += entra[c] - sai[c];
                }
            }
        }
    };
    if paralelo {
        use rayon::prelude::*;
        out.par_chunks_mut(ow)
            .enumerate()
            .for_each(|(j, dest)| linha(j, dest));
    } else {
        out.chunks_mut(ow)
            .enumerate()
            .for_each(|(j, dest)| linha(j, dest));
    }
    (out, ow)
}

/// **AS TRÊS CAIXAS HORIZONTAIS, FUNDIDAS NUMA PASSAGEM SOBRE A MEMÓRIA** (2026-09-22, ordem do
/// dono *«atacar o Blur»*).
///
/// ⭐ **A fusão é possível por uma propriedade que o [`caixa_h`] já declara:** *uma linha de saída é
/// função só da linha de entrada dela*. Logo as três caixas de uma linha podem correr enquanto ela
/// está quente na cache, em vez de atravessarem o buffer inteiro três vezes.
///
/// ⭐⭐ **E é BYTE-IDÊNTICA, não «igual a menos de um epsilon»:** as somas por linha são as mesmas,
/// na mesma ordem, com os mesmos `inv` — *o que muda é onde os intermédios vivem, nunca a
/// aritmética*. O gate `as_tres_horizontais_fundidas_dao_o_mesmo_f32` compara-a com as três
/// passagens separadas sobre um corpus de tamanhos e raios.
///
/// ⚠️ **O que ela compra é TRÁFEGO, e o ADR-0171 mediu que é disso que este núcleo vive**
/// (`~224 B/px` a `6,8 GB/s`, da ordem de um núcleo): três passagens separadas leem e escrevem o
/// buffer inteiro três vezes; esta lê-o uma vez e escreve-o uma vez, com os dois intermédios a
/// caberem num rascunho por linha.
fn caixa_h3(
    src: &[[f32; 4]],
    w: usize,
    h: usize,
    raios: [usize; 3],
    paralelo: bool,
) -> (Vec<[f32; 4]>, usize) {
    let r_total: usize = raios.iter().sum();
    if r_total == 0 {
        return (src.to_vec(), w);
    }
    let ow = w - 2 * r_total;
    let mut out = vec![[0f32; 4]; ow * h];
    // Uma caixa sobre uma fatia, escrita num destino — o MESMO laço do `caixa_h`, sem o buffer.
    let caixa = |ent: &[[f32; 4]], r: usize, dest: &mut [[f32; 4]]| {
        if r == 0 {
            dest.copy_from_slice(&ent[..dest.len()]);
            return;
        }
        let lado = 2 * r + 1;
        #[allow(clippy::cast_precision_loss)]
        let inv = 1.0 / lado as f32;
        let mut acc = [0f32; 4];
        for s in ent.iter().take(lado) {
            for c in 0..4 {
                acc[c] += s[c];
            }
        }
        let n = dest.len();
        for i in 0..n {
            for c in 0..4 {
                dest[i][c] = acc[c] * inv;
            }
            if i + 1 < n {
                for c in 0..4 {
                    acc[c] += ent[i + lado][c] - ent[i][c];
                }
            }
        }
    };
    let w1 = w - 2 * raios[0];
    let w2 = w1 - 2 * raios[1];
    // ⚠️⚠️ **O rascunho é por THREAD e nunca por linha.** A 1.ª redacção alocava os dois intermédios
    //    dentro do laço — `2 × h` alocações por passagem —, e isso comeria o tráfego que a fusão
    //    existe para poupar. O `for_each_init` do rayon dá um rascunho por trabalhador; em série ele
    //    é um só, fora do laço.
    let linha =
        |t1: &mut Vec<[f32; 4]>, t2: &mut Vec<[f32; 4]>, j: usize, dest: &mut [[f32; 4]]| {
            let ent = &src[j * w..j * w + w];
            caixa(ent, raios[0], t1);
            caixa(t1, raios[1], t2);
            caixa(t2, raios[2], dest);
        };
    if paralelo {
        use rayon::prelude::*;
        out.par_chunks_mut(ow).enumerate().for_each_init(
            || (vec![[0f32; 4]; w1], vec![[0f32; 4]; w2]),
            |(t1, t2), (j, dest)| linha(t1, t2, j, dest),
        );
    } else {
        let (mut t1, mut t2) = (vec![[0f32; 4]; w1], vec![[0f32; 4]; w2]);
        out.chunks_mut(ow)
            .enumerate()
            .for_each(|(j, dest)| linha(&mut t1, &mut t2, j, dest));
    }
    (out, ow)
}

/// Uma passagem de caixa VERTICAL por soma corrente. ⚠️ O acumulador é uma LINHA inteira e desliza
/// para baixo — uma coluna de cada vez leria a memória com passo `w` e pagaria a cache.
fn caixa_v(
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

/// **Em quantas fatias o trabalho se parte** — o número de threads que a pool de `rayon` tem.
///
/// ⚠️ Ele NÃO é fixado num literal: *fixar o número de pedaços não se nota com a máquina ocupada e
/// mata-a parada* (a lei que a `line/motion-value` pagou com o app do dono a piorar `18,6 → 30,4
/// ms`). Aqui ele é o que a pool de facto tem.
fn fatias_do_soquete() -> usize {
    rayon::current_num_threads().max(1)
}

/// **A LARGURA MÍNIMA de uma banda de colunas da passagem vertical, em pixels.**
///
/// ⛔⛔ **Aqui o número de fatias NÃO é o número de threads, e a medição é inequívoca.** Em série a
/// vertical percorre cada linha INTEIRA — é um fluxo sequencial —, e parti-la em `nb` bandas de
/// colunas transforma-a em `nb` fluxos com passo `w`. Abaixo de uma certa largura a banda deixa de
/// ser sequencial e o paralelo PERDE para si próprio:
///
/// | fatias | largura | ms | ganho |
/// |---|---|---|---|
/// | 1 | `1 484` | `8,417` | `1,00×` |
/// | 2 | `742` | `5,341` | `1,58×` |
/// | 3 | `494` | `4,203` | `2,00×` |
/// | **4** | **`371`** | **`3,749`** | **`2,25×`** |
/// | 6 | `247` | `3,973` | `2,12×` |
/// | 8 | `185` | `5,082` | `1,66×` |
/// | 12 | `123` | `6,055` | `1,39×` |
/// | 16 | `92` | `7,352` | `1,14×` |
///
/// (`--release`, região `1 484²`, `k = 24`, `91 %` de CPU ociosa, mínimo de 7 corridas.)
///
/// ⇒ o que se fixa é a **LARGURA** (o meio do planalto `3`–`6` bandas, `247`–`494` px) e a
/// CONTAGEM sai dela e da região, com tecto na pool. ⚠️ *Isto não contradiz a lei de não fixar o
/// número de pedaços: a contagem continua a seguir a máquina pelo tecto, e o que a medição
/// acrescenta é um PISO que vem do acesso à memória e não do escalonador.*
///
/// ⚠️ A passagem HORIZONTAL não tem este piso — ali uma linha de saída já é contígua, e ela escala
/// `3,6×` com uma fatia por thread (contra o tecto do soquete, que é `4,05×`).
pub(crate) const LARGURA_MINIMA_DA_BANDA: usize = 384;

/// Quantas bandas de colunas a passagem vertical usa numa região de largura `w`.
fn bandas_da_vertical(w: usize) -> usize {
    (w / LARGURA_MINIMA_DA_BANDA).clamp(1, fatias_do_soquete())
}

/// O gémeo de [`crate::blur::blur_region`] com o núcleo de **três caixas**, no mesmo espaço
/// premultiplicado e com o mesmo contrato de avental (`wrap`/clamp) — só a convolução muda.
///
/// ⚠️ Este doc-comment ficou para trás no `blur.rs` quando o corte de 2026-09-20 trouxe a função
/// para cá, colado ao doc do irmão que FICA: o clippy apanhou-o (`empty_line_after_doc_comments`),
/// e é a armadilha que o handoff §21.9 regista — *um corte que sobe por `///` deixa o vizinho com a
/// prosa de outro e a função que viajou sem a dela*.
#[allow(clippy::too_many_arguments)]
#[must_use]
pub(crate) fn blur_region_caixa(
    buf: &[u8],
    fw: i64,
    fh: i64,
    min_x: i64,
    min_y: i64,
    bw: usize,
    bh: usize,
    k: usize,
    wrap: [bool; 2],
) -> Vec<[f32; 4]> {
    blur_region_caixa_com(
        buf,
        fw,
        fh,
        min_x,
        min_y,
        bw,
        bh,
        k,
        wrap,
        vale_a_pena_partir(bw, bh),
    )
}

/// **Se esta região é grande o bastante para a passagem se partir em fatias.**
///
/// ⚠️ Ela é uma função PURA e nomeada de propósito: a decisão é a única coisa que uma rota
/// bit-idêntica tem de observável sem um relógio — *e um gate de relógio aqui seria mais um membro
/// da família de flakes de fan-out*. Sem ela, apagar o paralelo do produto não acorda gate nenhum.
#[must_use]
pub(crate) fn vale_a_pena_partir(bw: usize, bh: usize) -> bool {
    bw * bh >= PIXEIS_PARA_PARALELIZAR
}

/// **A partir de quantos pixels de REGIÃO a passagem se parte em fatias.**
///
/// ⚠️ Número MEDIDO e não escolhido — abaixo dele acordar a pool custa mais do que o trabalho que
/// ela poupa, e o paralelo PERDE (`--release`, `k = 24`, mínimo de 7 corridas):
///
/// | região | série | fatias | ganho |
/// |---|---|---|---|
/// | `64²` | `0,060` | `0,136` | `0,44×` |
/// | `128²` | `0,208` | `0,275` | `0,76×` |
/// | `192²` | `0,464` | `0,492` | `0,94×` |
/// | **`256²`** | **`0,832`** | **`0,827`** | **`1,01×`** |
/// | `384²` | `1,824` | `1,478` | `1,23×` |
/// | `768²` | `7,777` | `4,648` | `1,67×` |
/// | `1 484²` | `54,024` | `25,892` | **`2,09×`** |
///
/// ⇒ `256²` é o primeiro tamanho em que ele **deixa de perder**, e é esse o valor.
pub(crate) const PIXEIS_PARA_PARALELIZAR: usize = 256 * 256;

/// O gémeo com a rota ESCOLHIDA, que é o que torna a paridade gateável.
///
/// ⛔ A escolha é um ARGUMENTO e não uma variável de ambiente — *um gate que lê o ambiente mede a
/// máquina*, e a lei aqui é sobre os bytes de saída.
#[allow(clippy::too_many_arguments)]
#[must_use]
/// **O AVENTAL**: a região mais a margem que as três caixas vão consumir, lida do canvas e já
/// PREMULTIPLICADA. ⭐ Ele é por-pixel puro — *uma linha dele não lê nenhuma outra* —, logo parti-lo
/// em fatias é byte-idêntico (ADR-0171).
///
/// ⚠️ **Porta própria desde 2026-09-22, e não por estética:** a sonda que parte o relógio do borrão
/// entre as passagens (`diag_onde_o_borrao_gasta`) precisava de o cronometrar sozinho, e a
/// alternativa era **replicá-lo** no teste — *uma sonda que replica o passo que mede pode medir
/// outro programa, que é como as sondas desta casa já mentiram meia dúzia de vezes*.
#[allow(clippy::too_many_arguments)]
pub(crate) fn avental(
    buf: &[u8],
    fw: i64,
    fh: i64,
    min_x: i64,
    min_y: i64,
    ap_w: usize,
    ap_h: usize,
    r_total: usize,
    wrap: [bool; 2],
    paralelo: bool,
) -> Vec<[f32; 4]> {
    let mut apron = vec![[0f32; 4]; ap_w * ap_h];
    let linha_avental = |j: usize, dest: &mut [[f32; 4]]| {
        let sy = src_coord(min_y + j as i64 - r_total as i64, fh, wrap[1]);
        for (i, d) in dest.iter_mut().enumerate().take(ap_w) {
            let sx = src_coord(min_x + i as i64 - r_total as i64, fw, wrap[0]);
            let si = ((sy * fw + sx) * 4) as usize;
            let a = f32::from(buf[si + 3]);
            let af = a / 255.0;
            *d = [
                f32::from(buf[si]) * af,
                f32::from(buf[si + 1]) * af,
                f32::from(buf[si + 2]) * af,
                a,
            ];
        }
    };
    if paralelo {
        use rayon::prelude::*;
        apron
            .par_chunks_mut(ap_w)
            .enumerate()
            .for_each(|(j, dest)| linha_avental(j, dest));
    } else {
        apron
            .chunks_mut(ap_w)
            .enumerate()
            .for_each(|(j, dest)| linha_avental(j, dest));
    }
    apron
}

pub(crate) fn blur_region_caixa_com(
    buf: &[u8],
    fw: i64,
    fh: i64,
    min_x: i64,
    min_y: i64,
    bw: usize,
    bh: usize,
    k: usize,
    wrap: [bool; 2],
    paralelo: bool,
) -> Vec<[f32; 4]> {
    let raios = box_radii(k);
    let r_total: usize = raios.iter().sum();
    let ap_w = bw + 2 * r_total;
    let ap_h = bh + 2 * r_total;
    let apron = avental(
        buf, fw, fh, min_x, min_y, ap_w, ap_h, r_total, wrap, paralelo,
    );
    // Separável: as três caixas na horizontal — FUNDIDAS numa passagem (2026-09-22) —, depois as
    // três na vertical. ⚠️ O `caixa_h` fica: ele é o oráculo do gate da fusão.
    // ⚠️ A largura NÃO muda nas verticais (só a altura), logo `w` não é `mut`.
    let (mut cur, w) = caixa_h3(&apron, ap_w, ap_h, raios, paralelo);
    let mut h = ap_h;
    for r in raios {
        let (n, nh) = caixa_v(
            &cur,
            w,
            h,
            r,
            if paralelo { bandas_da_vertical(w) } else { 1 },
        );
        cur = n;
        h = nh;
    }
    debug_assert_eq!((w, h), (bw, bh));
    let desfaz = |p: &mut [f32; 4]| {
        let a = p[3];
        let inv = if a > 1e-4 { 255.0 / a } else { 0.0 };
        *p = [p[0] * inv, p[1] * inv, p[2] * inv, a];
    };
    if paralelo {
        use rayon::prelude::*;
        cur.par_iter_mut().for_each(desfaz);
    } else {
        cur.iter_mut().for_each(desfaz);
    }
    cur
}

#[cfg(test)]
#[path = "blur_caixa_tests.rs"]
mod tests_do_caixa;
