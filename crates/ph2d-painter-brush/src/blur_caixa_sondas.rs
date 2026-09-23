//! **As SONDAS de relógio do núcleo de três caixas** — cortadas do `blur_caixa_tests.rs` pelo
//! tecto de LOC (`1 320` contra `700`), por responsabilidade: lá moram os gates, aqui as medições
//! que se correm à mão numa máquina calma (e o diagnóstico do miolo, que é a régua delas).

/// **ONDE O BORRÃO GASTA** — a sonda que a ordem do dono de 2026-09-22 (*«atacar o Blur»*) pede
/// antes da 1.ª linha de cura.
///
/// O [ADR-0171](../../../docs/architecture/decisions/0171-o-borrao-de-caixa-da-pilha-parte-se-em-fatias-e-a-largura-da-banda-e-medida.md)
/// já mediu o que NÃO é (o núcleo, o avental, a alocação, o cover por blocos) e deixou o custo
/// por pixel: `~224 B/px` em sete passagens de `[f32; 4]`. ⚠️ **Mas ele nunca partiu esse número
/// entre as passagens** — e sem isso «atacar o Blur» escolhe a metade errada com 50 % de chance.
///
/// ⭐⭐ **O CONTROLO é o que torna esta sonda honesta:** as partes são medidas chamando as MESMAS
/// funções privadas na MESMA ordem, e a soma delas é comparada com o total medido pela **porta do
/// produto** (`blur_region_caixa`). Se as duas discordarem, a decomposição está a medir outro
/// programa — *que é exactamente como uma sonda mente*.
///
/// ```text
/// bash scripts/ph2d-run.sh cargo test -p ph2d-painter-brush --release --lib \
///     diag_onde_o_borrao_gasta -- --ignored --nocapture --test-threads=1
/// ```
#[test]
#[ignore = "sonda de relógio: corre à mão, em --release e com a máquina CALMA"]
fn diag_onde_o_borrao_gasta() {
    let carga = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!("\n  ONDE O BORRÃO GASTA   (load {})", carga.trim());
    println!(
        "\n   bw × bh |   k | avental | 3×H sep | 1×H fund |  ganho | 3H serie | 1H serie | ganho |     3×V | páginas |  desfaz | BLUR sep | BLUR fund | ganho | porta1 | porta2 | erro"
    );
    println!(
        "  ---------+-----+---------+---------+----------+--------+---------+---------+--------+---------+---------+---------+----------+-----------+-------+--------+--------+------"
    );

    for &(bw, bh, k) in &[
        (912usize, 240usize, 24usize),
        (912, 400, 96),
        (1024, 1024, 96),
    ] {
        let (fw, fh) = (2048i64, 2048i64);
        let buf: Vec<u8> = (0..(fw * fh * 4) as usize)
            .map(|i| ((i * 37) % 256) as u8)
            .collect();
        let ms = |f: &mut dyn FnMut()| -> f64 {
            let mut melhor = f64::MAX;
            for _ in 0..3 {
                let t0 = std::time::Instant::now();
                f();
                melhor = melhor.min(t0.elapsed().as_secs_f64() * 1e3);
            }
            melhor
        };

        // A porta do PRODUTO — o total contra o qual a decomposição é conferida.
        let mut porta = 0.0;
        let t_porta = ms(&mut || {
            let v = super::blur_region_caixa(&buf, fw, fh, 0, 0, bw, bh, k, [false, false]);
            porta = v.len() as f64;
        });
        let _ = porta;

        let raios = super::box_radii(k);
        let r_total: usize = raios.iter().sum();
        let (ap_w, ap_h) = (bw + 2 * r_total, bh + 2 * r_total);
        let paralelo = super::vale_a_pena_partir(bw, bh);

        // (1) o AVENTAL — replicado, e é a única parte que não é a função do produto; o controlo
        //     da soma abaixo é o que impede esta réplica de mentir.
        let mut apron = Vec::new();
        let t_ap = ms(&mut || {
            apron = super::avental(
                &buf,
                fw,
                fh,
                0,
                0,
                ap_w,
                ap_h,
                r_total,
                [false, false],
                paralelo,
            );
        });

        // ⚠️⚠️ **Nenhum `clone()` dentro do cronómetro, e isto foi APANHADO pelo controlo:** a
        //    1.ª redacção clonava a entrada de cada etapa lá dentro, e a soma das partes leu
        //    `+97 %` da porta. *As caixas recebem `&[…]` e não mutam a entrada, logo não há nada
        //    a clonar; só o `desfaz` precisa de um alvo, e ele é preparado FORA.*
        let (mut cur, mut w, mut h) = (Vec::new(), ap_w, ap_h);
        let t_h = ms(&mut || {
            let (mut c, mut ww) = (Vec::new(), ap_w);
            let mut ent: &[[f32; 4]] = &apron;
            for r in raios {
                let (n, nw) = super::caixa_h(ent, ww, ap_h, r, paralelo);
                c = n;
                ww = nw;
                ent = &c;
            }
            cur = c;
            w = ww;
            h = ap_h;
        });

        // (2-bis) **O A/B DA FUSÃO, na MESMA corrida** — a única forma honesta de o medir com
        //         esta máquina, que está partilhada com outra linha: as duas rotas veem o mesmo
        //         estado de cache, o mesmo escalonador e a mesma contenção.
        let t_h3 = ms(&mut || {
            let _ = super::caixa_h3(&apron, ap_w, ap_h, raios, paralelo);
        });

        // (2-ter) **O MESMO PAR, EM SÉRIE** — e é este que decide. O par paralelo mede o
        //     ESCALONADOR tanto como o código: com outra linha a martelar a máquina, a mesma
        //     porta lida duas vezes na mesma corrida deu `10,2` e `14,2 ms`. Em série o que
        //     sobra é o TRÁFEGO DE MEMÓRIA, que é exactamente o que a fusão corta, e a razão
        //     sobrevive à contenção porque as duas leituras a sofrem por igual.
        let t_h_serie = ms(&mut || {
            let (mut c, mut ww) = (Vec::new(), ap_w);
            let mut ent: &[[f32; 4]] = &apron;
            for r in raios {
                let (n, nw) = super::caixa_h(ent, ww, ap_h, r, false);
                c = n;
                ww = nw;
                ent = &c;
            }
            std::hint::black_box(c.len());
        });
        let t_h3_serie = ms(&mut || {
            let v = super::caixa_h3(&apron, ap_w, ap_h, raios, false);
            std::hint::black_box(v.0.len());
        });

        // (3) as três VERTICAIS.
        let entrada_v = std::mem::take(&mut cur);
        let mut saida = Vec::new();
        let t_v = ms(&mut || {
            let (mut c, mut hh) = (Vec::new(), h);
            let mut ent: &[[f32; 4]] = &entrada_v;
            for r in raios {
                let fatias = if paralelo {
                    super::bandas_da_vertical(w)
                } else {
                    1
                };
                let (n, nh) = super::caixa_v(ent, w, hh, r, fatias);
                c = n;
                hh = nh;
                ent = &c;
            }
            saida = c;
        });

        // (4) o DESFAZ da premultiplicação — o alvo é preparado FORA do cronómetro.
        let mut alvo = saida.clone();
        let t_d = ms(&mut || {
            for p in &mut alvo {
                let a = p[3];
                let inv = if a > 1e-4 { 255.0 / a } else { 0.0 };
                *p = [p[0] * inv, p[1] * inv, p[2] * inv, a];
            }
        });

        // (3-bis) **A VERTICAL, os dois lados, EM SÉRIE** — a mesma lei do par horizontal.
        let t_v_serie = ms(&mut || {
            let (mut c, mut hh) = (Vec::new(), h);
            let mut ent: &[[f32; 4]] = &entrada_v;
            for r in raios {
                let (n, nh) = super::caixa_v(ent, w, hh, r, 1);
                c = n;
                hh = nh;
                ent = &c;
            }
            std::hint::black_box(c.len());
        });
        let t_v3_serie = ms(&mut || {
            let v = super::caixa_v3(
                &entrada_v,
                w,
                h,
                raios,
                super::LARGURA_DA_BANDA_FUNDIDA,
                false,
            );
            std::hint::black_box(v.0.len());
        });

        // (4-bis) **QUANTO DAS TRÊS VERTICAIS É SÓ MEMÓRIA NOVA?** Cada `caixa_v` devolve um
        //     `Vec` FRESCO (`vec![[0f32; 4]; …]` = `alloc_zeroed`), logo as páginas chegam
        //     preguiçosas e a PRIMEIRA escrita de cada uma é uma falha de página. Isto mede
        //     exactamente esse chão: alocar os mesmos três tamanhos e TOCAR cada página.
        //     *Se ele for a maior parte do `3×V`, a cura é reaproveitar o buffer e não fundir
        //     o laço — e fundir seria construir a coisa cara para não pagar a barata.*
        let t_paginas = ms(&mut || {
            let mut hh = ap_h;
            let mut toque = 0.0f32;
            for r in raios {
                let oh = hh - 2 * r;
                let mut v = vec![[0f32; 4]; w * oh];
                // uma escrita por página de 4 KiB (= 256 pixels de 16 B)
                for i in (0..v.len()).step_by(256) {
                    v[i][0] = 1.0;
                    toque += v[i][0];
                }
                hh = oh;
            }
            std::hint::black_box(toque);
        });

        // (5) **A PORTA OUTRA VEZ, no FIM** — o controlo do CONTROLO. A 1.ª leitura dela é a
        //     primeira coisa que toca os 16 MiB do `buf`; as partes correm todas depois, com
        //     ele já quente. *Se esta 2.ª leitura cair para perto da soma, o desvio é da ORDEM
        //     e não de trabalho em falta na decomposição.*
        let t_porta2 = ms(&mut || {
            let v = super::blur_region_caixa(&buf, fw, fh, 0, 0, bw, bh, k, [false, false]);
            porta = v.len() as f64;
        });

        // ⚠️ O controlo soma o que a PORTA de facto corre, que desde a fusão é a `caixa_h3`.
        //    Somar o `t_h` separado aqui seria comparar a decomposição de um programa com o
        //    relógio de OUTRO — e a 1.ª redacção desta linha fazia exactamente isso.
        let soma = t_ap + t_h3 + t_v + t_d;
        let erro = (soma - t_porta2) / t_porta2 * 100.0;
        let separado = t_ap + t_h + t_v + t_d;
        // ⭐ **O VEREDITO DA JORNADA, em SÉRIE** — o borrão inteiro antes e depois das duas
        //   fusões, com as duas leituras a sofrer a contenção por igual.
        let antes = t_ap + t_h_serie + t_v_serie + t_d;
        let depois = t_ap + t_h3_serie + t_v3_serie + t_d;
        println!(
            "  >> {bw}×{bh} k={k}  BORRÃO EM SÉRIE  antes {antes:6.2} ms  depois {depois:6.2} ms   {:.2}×  \
             (H {:.2}× · V {:.2}×)",
            antes / depois.max(f64::MIN_POSITIVE),
            t_h_serie / t_h3_serie.max(f64::MIN_POSITIVE),
            t_v_serie / t_v3_serie.max(f64::MIN_POSITIVE)
        );
        println!(
            "  {bw:4}×{bh:4} | {k:3} | {t_ap:7.2} | {t_h:7.2} | {t_h3:7.2} | {:5.2}× | {t_h_serie:7.2} | {t_h3_serie:7.2} | {:5.2}× | {t_v:7.2} | {t_paginas:7.2} | {t_d:7.2} | {separado:5.1} | {soma:5.1} | {:5.2}× | {t_porta:5.1} | {t_porta2:5.1} | {erro:+5.1}%",
            t_h / t_h3.max(f64::MIN_POSITIVE),
            t_h_serie / t_h3_serie.max(f64::MIN_POSITIVE),
            separado / soma.max(f64::MIN_POSITIVE)
        );
    }
    println!(
        "\n  (o ERRO é o controlo: se a soma das partes não bater a porta do produto,\n   \
         a decomposição está a medir outro programa)\n"
    );
}

/// **O borrão de uma SUB-REGIÃO é igual ao miolo do borrão da região MAIOR?**
///
/// ⚠️ É a pergunta que decide se a aplicação do borrão pode encolher: se a resposta for NÃO, a
/// soma corrente da caixa carrega o sítio onde COMEÇOU e o resultado passa a depender da região
/// pedida — que é a não-associatividade do `f32` que a cerca das bandas desta crate já nomeia.
///
/// ⚠️ **Gate desde a auditoria de 2026-09-23** (o nome `diag_` fica porque o handoff e três docs
/// o citam): ele corria na suíte sem afirmar NADA, e é a premissa sobre que a barra `pior ≤ 1`
/// do `a_ordem_e_da_pilha_e_nao_da_taxa_do_rato` assenta. As duas metades:
/// * **não é idêntico** — se um dia for (outra soma, outra ordem), aquela barra pode voltar a
///   `pior == 0`, e isso tem de ser dito;
/// * **difere menos de `1e-2`** (numa escala `0..255`) — o bastante para mudar um byte só quando o
///   valor exacto cai junto a uma fronteira de arredondamento, e nunca dois.
#[test]
fn diag_o_borrao_de_uma_sub_regiao_e_o_miolo_do_maior() {
    let (fw, fh) = (512i64, 512i64);
    let buf: Vec<u8> = (0..(fw * fh * 4) as usize)
        .map(|i| ((i * 37 + (i / 97) * 11) % 256) as u8)
        .collect();
    for k in [8usize, 24, 96] {
        let r_total: usize = super::box_radii(k).iter().sum();
        // a região GRANDE e a sub-região no meio dela
        let (gx, gy, gw, gh) = (60i64, 60i64, 300usize, 300usize);
        let pad = 40usize;
        let (sx, sy, sw, sh) = (gx + pad as i64, gy + pad as i64, gw - 2 * pad, gh - 2 * pad);
        let grande = super::blur_region_caixa(&buf, fw, fh, gx, gy, gw, gh, k, [false, false]);
        let pequena = super::blur_region_caixa(&buf, fw, fh, sx, sy, sw, sh, k, [false, false]);
        let mut pior = 0f32;
        let mut iguais = 0usize;
        for j in 0..sh {
            for i in 0..sw {
                let a = grande[(j + pad) * gw + i + pad];
                let b = pequena[j * sw + i];
                let d = (0..4).fold(0f32, |m, c| m.max((a[c] - b[c]).abs()));
                pior = pior.max(d);
                iguais += usize::from(a == b);
            }
        }
        println!(
            "  k={k:3} r_total={r_total:3}  pior |Δ| = {pior:.3e}  ·  {iguais} de {} idênticos ao bit",
            sw * sh
        );
        assert!(
            pior > 0.0 && iguais < sw * sh,
            "k={k}: a sub-região saiu IDÊNTICA ao miolo — a premissa do byte tolerado morreu"
        );
        assert!(
            pior < 1e-2,
            "k={k}: a sub-região difere do miolo mais do que o arredondamento explica ({pior:.3e})"
        );
    }
}

/// **A LARGURA DA BANDA DA VERTICAL FUNDIDA, varrida** — o número da
/// [`super::LARGURA_DA_BANDA_FUNDIDA`] sai daqui e de mais lado nenhum.
///
/// ⚠️ **A coluna que decide é a de SÉRIE.** Em paralelo o relógio mede o escalonador tanto como
/// o código — medido nesta máquina, a MESMA porta lida duas vezes na mesma corrida deu `10,2` e
/// `14,2 ms` a `load 14`. Em série o que sobra é o TRÁFEGO DE MEMÓRIA, que é exactamente o que a
/// fusão corta, e a razão contra a linha de base sofre a contenção por igual dos dois lados.
///
/// `cargo test -p ph2d-painter-brush --release --lib diag_a_largura_da_banda -- --ignored --nocapture`
#[test]
#[ignore = "sonda de relógio: corre à mão numa máquina calma"]
fn diag_a_largura_da_banda_fundida() {
    let carga = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!(
        "\n  A LARGURA DA BANDA DA VERTICAL   (load {})",
        carga.trim()
    );

    for &(bw, bh, k) in &[(912usize, 400usize, 96usize), (1024, 1024, 96)] {
        let raios = super::box_radii(k);
        let r_total: usize = raios.iter().sum();
        let (ap_w, ap_h) = (bw + 2 * r_total, bh + 2 * r_total);
        let (fw, fh) = (2048i64, 2048i64);
        let buf: Vec<u8> = (0..(fw * fh * 4) as usize)
            .map(|i| ((i * 37) % 256) as u8)
            .collect();
        let apron = super::avental(
            &buf,
            fw,
            fh,
            0,
            0,
            ap_w,
            ap_h,
            r_total,
            [false, false],
            false,
        );
        // A entrada da vertical é a saída das horizontais.
        let (ent, w) = super::caixa_h3(&apron, ap_w, ap_h, raios, false);

        let ms = |f: &mut dyn FnMut()| -> f64 {
            let mut melhor = f64::MAX;
            for _ in 0..3 {
                let t0 = std::time::Instant::now();
                f();
                melhor = melhor.min(t0.elapsed().as_secs_f64() * 1e3);
            }
            melhor
        };

        // A linha de BASE: as três passagens separadas, como o produto corria até hoje.
        let base_serie = ms(&mut || {
            let (mut c, mut hh) = (Vec::new(), ap_h);
            let mut e: &[[f32; 4]] = &ent;
            for r in raios {
                let (n, nh) = super::caixa_v(e, w, hh, r, 1);
                c = n;
                hh = nh;
                e = &c;
            }
            std::hint::black_box(c.len());
        });

        println!("\n  {bw}×{bh}  k={k}   (3×V separadas, série: {base_serie:.2} ms)");
        println!("   largura | fundida | ganho | KiB/intermédio");
        println!("  ---------+---------+-------+----------------");
        for largura in [16usize, 32, 64, 128, 256, 512, w] {
            if largura > w {
                continue;
            }
            let t = ms(&mut || {
                let v = super::caixa_v3(&ent, w, ap_h, raios, largura, false);
                std::hint::black_box(v.0.len());
            });
            let kib = largura * (ap_h - 2 * raios[0]) * 16 / 1024;
            println!(
                "  {largura:8} | {t:7.2} | {:5.2}× | {kib:14}",
                base_serie / t.max(f64::MIN_POSITIVE)
            );
        }
    }
    println!();
}
