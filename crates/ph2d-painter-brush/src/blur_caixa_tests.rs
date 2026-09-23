//! **Os gates do núcleo de TRÊS CAIXAS** — cortados do [`super`] em 2026-09-21, quando o
//! paralelo o levou de `700` para `1 028` linhas.
//!
//! ⚠️ Ele é incluído por `#[path]` **de dentro** do `blur_caixa.rs` e não declarado no `lib.rs`:
//! assim os `mod` daqui continuam a ser DESCENDENTES do módulo que julgam, e nenhum `fn` privado
//! teve de alargar a visibilidade só para hospedar um teste. *Mover um ficheiro troca o REGIME DE
//! VISIBILIDADE que o governa, e num `lib` `pub(crate)` é API.*

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::blur::BLUR_KERNEL_MAX;

    /// **A caixa tripla tem a VARIÂNCIA do binomial que ela substitui** — é isso que faz dela a
    /// «mesma quantidade de borrão», e não um borrão mais fraco disfarçado de optimização.
    ///
    /// Um binomial de raio `k` tem `σ² = k/2`; três caixas de larguras `wᵢ` somam `Σ(wᵢ²−1)/12`. O
    /// erro tem de ficar dentro do degrau da própria grelha de larguras (elas são ímpares, logo a
    /// variância só toma valores discretos) — a barra é **meia largura de degrau**, derivada, e não
    /// um epsilon escolhido.
    #[test]
    fn a_caixa_tripla_tem_a_variancia_do_binomial() {
        for k in 1..=BLUR_KERNEL_MAX {
            let r = box_radii(k);
            let var: f32 = r
                .iter()
                .map(|&ri| {
                    let w = (2 * ri + 1) as f32;
                    (w * w - 1.0) / 12.0
                })
                .sum();
            let alvo = k as f32 / 2.0;
            // O degrau: trocar UMA caixa de largura `w` por `w+2` muda a variância em
            // `((w+2)² − w²)/12 = (4w + 4)/12`. Com `w ≈ √(1+2k)`, meia dessas.
            let w = (1.0 + 2.0 * k as f32).sqrt();
            let degrau = (4.0 * w + 4.0) / 12.0;
            assert!(
                (var - alvo).abs() <= degrau * 0.5 + 1e-3,
                "k={k}: variância {var:.3} contra o alvo {alvo:.3} (degrau {degrau:.3})"
            );
        }
    }

    /// **A caixa tripla é uma MÉDIA: sobre um campo constante ela devolve a constante.**
    ///
    /// ⚠️ Esta é a metade que apanha um erro de normalização ou de avental — os dois deixariam a
    /// régua da variância acima VERDE, porque ela mede só as larguras.
    ///
    /// ⛔⛔ **E o ALFA é obrigatório, não é zelo: uma MUTAÇÃO SOBREVIVEU sem ele.** Com a versão que
    /// varria só `0..3`, trocar `1/lado` por `1/(lado − ½)` — um ganho uniforme de `20 %` na média —
    /// passava VERDE, porque este é um borrão em espaço **premultiplicado** e o último passo
    /// un-premultiplica por `255/α`: *um ganho uniforme entra no numerador e no denominador e
    /// divide-se a si próprio*. O canal que não é dividido por nada é o **α**, e é só ele que vê a
    /// normalização. ⇒ *num borrão premultiplicado, um campo constante de RGB não é régua de
    /// normalização nenhuma.*
    #[test]
    fn a_caixa_tripla_preserva_um_campo_constante() {
        let (w, h) = (48u32, 48u32);
        let buf = vec![137u8; (w * h * 4) as usize];
        for k in [1usize, 4, 8, 32] {
            let out = blur_region_caixa(
                &buf,
                i64::from(w),
                i64::from(h),
                8,
                8,
                24,
                24,
                k,
                [false, false],
            );
            for p in &out {
                for v in &p[..3] {
                    assert!(
                        (v - 137.0).abs() < 0.6,
                        "k={k}: a caixa mudou um campo constante para {v:.2}"
                    );
                }
                assert!(
                    (p[3] - 137.0).abs() < 0.6,
                    "k={k}: a caixa não é uma MÉDIA — o alfa saiu {:.2} de 137",
                    p[3]
                );
            }
        }
    }
}

#[cfg(test)]
mod fatias_tests {
    use super::super::{blur_region_caixa_com, box_radii, caixa_v};

    /// **QUANTAS FATIAS a passagem VERTICAL quer** — a sonda por etapa mediu `1,12×` com uma fatia
    /// por thread, contra `3,6×` da horizontal. A causa é o acesso: em série a vertical percorre
    /// cada linha INTEIRA e é um fluxo sequencial; partida em `nb` bandas de colunas ela vira `nb`
    /// fluxos com passo `w`. ⇒ *a banda tem de ser larga o bastante para voltar a ser sequencial.*
    /// `cargo test -p ph2d-painter-brush --release fatias_tests::mede -- --ignored --nocapture`
    #[test]
    #[ignore = "sonda de relógio"]
    fn mede_quantas_fatias_a_vertical_quer() {
        let (w, h) = (2048usize, 2048usize);
        let mut buf = vec![0u8; w * h * 4];
        for (i, b) in buf.iter_mut().enumerate() {
            *b = ((i * 37) % 251) as u8;
        }
        let (lado, k) = (1484usize, 24usize);
        let r = box_radii(k)[0];
        let apron = blur_region_caixa_com(
            &buf,
            w as i64,
            h as i64,
            8,
            8,
            lado,
            lado,
            1,
            [false, false],
            false,
        );
        let mut base = f64::MAX;
        for nb in [1usize, 2, 3, 4, 6, 8, 12, 16] {
            let mut t_min = f64::MAX;
            for _ in 0..7 {
                let t = std::time::Instant::now();
                let out = caixa_v(&apron, lado, lado, r, nb);
                t_min = t_min.min(t.elapsed().as_secs_f64() * 1e3);
                assert!(!out.0.is_empty());
            }
            if nb == 1 {
                base = t_min;
            }
            eprintln!(
                "{nb:>2} fatias | {t_min:>7.3} ms | {:>5.2}× | largura da banda {:>5} px",
                base / t_min,
                lado / nb
            );
        }
    }

    /// **AS TRÊS HORIZONTAIS FUNDIDAS DÃO O MESMO `f32`** — o gate da cura de 2026-09-22 (ordem do
    /// dono: *«atacar o Blur»*).
    ///
    /// A fusão corre as três caixas de uma LINHA enquanto ela está quente, em vez de atravessar o
    /// buffer inteiro três vezes. ⭐ **Ela é byte-idêntica por construção** — as mesmas somas, na
    /// mesma ordem, com os mesmos `inv` —, e este gate é o que transforma esse «por construção» numa
    /// propriedade: ele corre as TRÊS passagens separadas (`caixa_h`, que fica no ficheiro
    /// exactamente para ser este oráculo) e exige igualdade **ao bit**.
    ///
    /// ⚠️ **O corpus não é um tamanho:** ele varre raios (incluindo `k` pequeno, onde algum dos três
    /// raios é `0` e o braço de cópia arma), larguras pares e ímpares, e as duas rotas (série e
    /// paralelo) — *uma fusão que só fosse igual no caso do meio não seria uma fusão*.
    #[test]
    fn as_tres_horizontais_fundidas_dao_o_mesmo_f32() {
        let mut casos = 0usize;
        for k in [1usize, 2, 3, 8, 24, 96] {
            let raios = super::super::box_radii(k);
            let r_total: usize = raios.iter().sum();
            for (w_extra, h) in [(1usize, 3usize), (0, 7), (5, 11), (32, 5)] {
                let w = 2 * r_total + 1 + w_extra;
                let src: Vec<[f32; 4]> = (0..w * h)
                    .map(|i| {
                        #[allow(clippy::cast_precision_loss)]
                        let f = i as f32;
                        [f * 0.37, f * 0.11 + 1.0, (f % 13.0) * 7.5, (f % 251.0)]
                    })
                    .collect();
                for paralelo in [false, true] {
                    // O ORÁCULO: as três passagens separadas, que é o que o produto fazia.
                    let (mut c, mut ww) = (src.clone(), w);
                    for r in raios {
                        let (n, nw) = super::super::caixa_h(&c, ww, h, r, paralelo);
                        c = n;
                        ww = nw;
                    }
                    let (fundido, fw) = super::super::caixa_h3(&src, w, h, raios, paralelo);
                    assert_eq!(
                        fw, ww,
                        "a largura de saída tem de ser a mesma (k={k}, w={w})"
                    );
                    assert_eq!(
                        fundido, c,
                        "a fusão tem de ser BYTE-IDÊNTICA às três passagens (k={k}, w={w}, h={h}, \
                         paralelo={paralelo})"
                    );
                    casos += 1;
                }
            }
        }
        // PISO DE POPULAÇÃO: sem ele, um corpus que encolhesse para zero passaria em silêncio.
        assert!(casos >= 48, "o corpus encolheu: {casos} casos");
    }

    /// **A vertical fundida numa banda dá o MESMO `f32` que as três passagens separadas** — e para
    /// TODA largura de banda, que é a metade que a torna uma lei e não um acidente do número que se
    /// escolheu.
    ///
    /// ⭐ **O argumento é o que a [`super::super::caixa_v`] já escreve para o paralelo:** o
    /// acumulador `acc[i]` só toca a coluna `c0 + i`, logo a largura da banda é ORDEM DE LAÇO e
    /// nunca aritmética — cada coluna recebe a mesma sequência de adições, na mesma ordem.
    ///
    /// ⚠️ **A largura varre `1`, `2`, `3` e a largura CHEIA de propósito:** a
    /// [`super::super::LARGURA_DA_BANDA_FUNDIDA`] shipa `128`, e se o gate só a medisse ali a lei
    /// ficava afirmada num ponto só — *uma propriedade que vale para toda partição prova-se em mais
    /// do que a partição que o produto usa*.
    ///
    /// **Mutações que sangram:** semear o acumulador com a banda inteira em vez de por coluna ·
    /// trocar a ordem das três passagens · devolver `h` em vez de `h − 2·Σr`.
    #[test]
    fn as_tres_verticais_fundidas_dao_o_mesmo_f32() {
        let mut casos = 0usize;
        for k in [1usize, 2, 3, 8, 24, 96] {
            let raios = super::super::box_radii(k);
            let r_total: usize = raios.iter().sum();
            for (w, h_extra) in [(1usize, 3usize), (7, 0), (11, 5), (5, 32)] {
                let h = 2 * r_total + 1 + h_extra;
                let src: Vec<[f32; 4]> = (0..w * h)
                    .map(|i| {
                        #[allow(clippy::cast_precision_loss)]
                        let f = i as f32;
                        [f * 0.37, f * 0.11 + 1.0, (f % 13.0) * 7.5, (f % 251.0)]
                    })
                    .collect();
                // O ORÁCULO: as três passagens separadas, que é o que o produto fazia até hoje.
                let (mut c, mut hh) = (src.clone(), h);
                for r in raios {
                    let (n, nh) = super::super::caixa_v(&c, w, hh, r, 1);
                    c = n;
                    hh = nh;
                }
                for largura in [1usize, 2, 3, w] {
                    for paralelo in [false, true] {
                        let (fundido, fh) =
                            super::super::caixa_v3(&src, w, h, raios, largura, paralelo);
                        assert_eq!(
                            fh, hh,
                            "a altura de saída tem de ser a mesma (k={k}, h={h}, largura={largura})"
                        );
                        assert_eq!(
                            fundido, c,
                            "a fusão tem de ser BYTE-IDÊNTICA às três passagens (k={k}, w={w}, \
                             h={h}, largura={largura}, paralelo={paralelo})"
                        );
                        casos += 1;
                    }
                }
            }
        }
        // PISO DE POPULAÇÃO: sem ele, um corpus que encolhesse para zero passaria em silêncio.
        assert!(casos >= 160, "o corpus encolheu: {casos} casos");
    }

    /// **A vertical fundida PARTE em bandas de cache — e isto mede a CONTA, não o valor.**
    ///
    /// ⭐⭐ **Ele existe porque uma MUTAÇÃO SOBREVIVEU:** cravar `nb = 1` na
    /// [`super::super::caixa_v3`] deixa o gate da identidade ao bit **VERDE**, porque a igualdade
    /// vale para toda partição — e o borrão passa a medir entre `0,45×` e `1,41×` das três passagens
    /// separadas conforme a corrida (re-medido na auditoria de 2026-09-23), ou seja **às vezes pior
    /// do que o motor que a fusão veio substituir**. *A lei da identidade não pode gatear a
    /// lei do custo; quem a gateia é a contagem.*
    ///
    /// ⚠️ **Corre em SÉRIE de propósito:** o contador é por THREAD (senão o fan-out da suíte conta
    /// as bandas de outra corrida), e em série todas as bandas passam por esta.
    ///
    /// **Mutações que sangram:** `nb = 1` · ignorar o `largura_da_banda` · arredondar para baixo
    /// em vez de `div_ceil` — ⚠️ esta sangra pela CONTAGEM (`912/128` dá `7` bandas contra `8`) e
    /// NÃO por deixar uma coluna de fora, que esta nota dizia até 2026-09-23 e é falso: a
    /// [`super::super::caixa_v3`] reparte `base + resto` e cobre sempre as `w` colunas.
    #[test]
    fn a_vertical_fundida_parte_em_bandas_de_cache() {
        use super::super::{BANDAS_PERCORRIDAS, LARGURA_DA_BANDA_FUNDIDA, caixa_v3};
        let raios = super::super::box_radii(8);
        let r_total: usize = raios.iter().sum();
        let h = 2 * r_total + 9;
        let mut casos = 0usize;
        for (w, largura) in [
            (1024usize, 128usize),
            (912, 128),
            (300, 64),
            (100, 128),
            (7, 2),
        ] {
            let src: Vec<[f32; 4]> = (0..w * h)
                .map(|i| {
                    #[allow(clippy::cast_precision_loss)]
                    let f = i as f32;
                    [f * 0.37, f * 0.11, f % 13.0, f % 251.0]
                })
                .collect();
            BANDAS_PERCORRIDAS.with(|c| c.set(0));
            let _ = caixa_v3(&src, w, h, raios, largura, false);
            let n = BANDAS_PERCORRIDAS.with(std::cell::Cell::get);
            assert_eq!(
                n,
                w.div_ceil(largura).max(1).min(w),
                "a partição tem de seguir a largura pedida (w={w}, largura={largura})"
            );
            casos += 1;
        }
        // A metade que nomeia o PRODUTO: na região grande, a constante que ship de facto PARTE.
        BANDAS_PERCORRIDAS.with(|c| c.set(0));
        let w = 1024usize;
        let src = vec![[1f32; 4]; w * h];
        let _ = caixa_v3(&src, w, h, raios, LARGURA_DA_BANDA_FUNDIDA, false);
        let n = BANDAS_PERCORRIDAS.with(std::cell::Cell::get);
        assert!(
            n >= 8,
            "a constante do produto tem de partir uma região de 1024 px em bandas de cache: {n}"
        );
        assert!(casos >= 5, "o corpus encolheu: {casos} casos");
    }

    /// **O ALCANCE que a caixa declara é EXACTAMENTE até onde ela lê** — nem um pixel a mais (o
    /// avental do composite pagaria área morta), nem um a menos (a orla leria bytes por compor).
    ///
    /// ⭐ A régua é o IMPULSO: um só pixel aceso a `d` píxeis da região muda a saída dela se e só
    /// se `d ≤ alcance`. É ela que dá direito ao avental estreito do composite (2026-09-23), que
    /// passou de `k·P + 1` para [`crate::BlurKernel::alcance`] `+ 1`.
    ///
    /// **Mutações que sangram:** o alcance ser um raio só · o alcance ser `k` (o do binomial).
    #[test]
    fn o_alcance_da_caixa_e_exactamente_ate_onde_ela_le() {
        let (fw, fh) = (1400i64, 3i64);
        let mut casos = 0usize;
        for k in [1usize, 2, 3, 8, 24, 96, 256] {
            let alcance = crate::BlurKernel::Caixa.alcance(k);
            let (rx, ry) = (40i64, 1i64);
            let le = |d: usize| {
                let mut buf = vec![0u8; (fw * fh * 4) as usize];
                let i = ((ry * fw + rx + d as i64) * 4) as usize;
                buf[i..i + 4].copy_from_slice(&[255, 255, 255, 255]);
                let out =
                    blur_region_caixa_com(&buf, fw, fh, rx, ry, 1, 1, k, [false, false], false);
                out[0][3] > 0.0
            };
            assert!(
                le(alcance),
                "k={k}: o pixel a {alcance} px (o alcance) tem de ser lido"
            );
            assert!(
                !le(alcance + 1),
                "k={k}: o pixel a {} px foi lido — o alcance declarado ({alcance}) é CURTO",
                alcance + 1
            );
            casos += 1;
        }
        assert!(casos >= 7, "o corpus encolheu: {casos}");
    }

    /// **A ROTA do motor dá o MESMO `f32` fundida e separada** — o gate que a auditoria de
    /// 2026-09-23 achou em falta.
    ///
    /// Os dois gates acima provam as funções COMPONENTES (`caixa_h3` contra `caixa_h`, `caixa_v3`
    /// contra `caixa_v`), e a rota que a porta [`super::super::sem_fusao`] escolhe é código de
    /// PRODUTO por cima delas: ela ENCADEIA as larguras e as alturas entre as seis caixas. Esta
    /// cola não tinha gate — os irmãos reconstroem-na à mão —, e é ela que o A/B da porta mede.
    ///
    /// ⚠️ **O corpus inclui a largura em que o produto de facto parte** (`w = 300`, três bandas de
    /// `100` sob o máximo de `128`; `w = 1030`, nove): os gates componentes só vêem larguras até
    /// `11`.
    ///
    /// **Mutações que sangram:** esquecer o `w = nw` na rota separada · passar `ap_w` em vez de `w`
    /// às verticais fundidas · uma caixa a menos num dos laços.
    ///
    /// ⛔ **O que ele NÃO vê, e é declarado:** a escolha `bandas_da_vertical(w)` contra `1` no
    /// caminho paralelo da rota separada é ORDEM DE LAÇO (a identidade ao bit vale para toda
    /// partição, gate `as_bandas_de_colunas_dao_o_mesmo_que_uma_banda_so`), logo é CUSTO e não
    /// valor — e uma contagem por thread não a vê, porque as bandas correm nos trabalhadores do
    /// rayon. Quem a mede é o relógio da porta.
    #[test]
    fn as_duas_rotas_do_motor_dao_o_mesmo_f32() {
        let mut casos = 0usize;
        for k in [1usize, 3, 24, 96] {
            let raios = box_radii(k);
            let r_total: usize = raios.iter().sum();
            for (bw, bh) in [(1usize, 1usize), (7, 3), (300, 17), (1030, 9)] {
                let (ap_w, ap_h) = (bw + 2 * r_total, bh + 2 * r_total);
                let apron: Vec<[f32; 4]> = (0..ap_w * ap_h)
                    .map(|i| {
                        #[allow(clippy::cast_precision_loss)]
                        let f = ((i * 2_654_435_761) % 1_000_003) as f32;
                        [f * 1e-4, (f % 97.0) * 2.5, (f % 13.0) * 19.0, f % 256.0]
                    })
                    .collect();
                for paralelo in [false, true] {
                    let (a, aw, ah) = super::super::as_seis_caixas(
                        apron.clone(),
                        ap_w,
                        ap_h,
                        raios,
                        paralelo,
                        true,
                    );
                    let (b, bw2, bh2) = super::super::as_seis_caixas(
                        apron.clone(),
                        ap_w,
                        ap_h,
                        raios,
                        paralelo,
                        false,
                    );
                    assert_eq!(
                        (aw, ah),
                        (bw, bh),
                        "a rota fundida tem de entregar a região"
                    );
                    assert_eq!(
                        (bw2, bh2),
                        (bw, bh),
                        "a rota separada tem de entregar a região"
                    );
                    assert!(
                        a == b,
                        "as duas rotas têm de ser BYTE-IDÊNTICAS (k={k}, {bw}×{bh}, \
                         paralelo={paralelo})"
                    );
                    casos += 1;
                }
            }
        }
        assert!(casos >= 32, "o corpus encolheu: {casos} casos");
    }
}
