//! **Os gates do PARALELO do núcleo de três caixas** — cortados do `blur_caixa_tests.rs` pelo
//! tecto de LOC, por assunto: o paralelo tem de dar o MESMO `f32` que a série, e aqui mora essa
//! prova. Incluído por `#[path]` de dentro do `blur_caixa.rs`, como o irmão — os `mod` daqui
//! continuam DESCENDENTES do módulo que julgam.

#[cfg(test)]
mod paralelo_tests {
    use super::super::{PIXEIS_PARA_PARALELIZAR, blur_region_caixa_com};

    /// Uma tela com conteúdo que não é plano — um campo constante esconderia toda diferença de
    /// ordem de soma, e é precisamente a ordem que o paralelo preserva ou não.
    fn tela(w: usize, h: usize) -> Vec<u8> {
        let mut v = vec![0u8; w * h * 4];
        for y in 0..h {
            for x in 0..w {
                let i = (y * w + x) * 4;
                v[i] = ((x * 7 + y * 13) % 251) as u8;
                v[i + 1] = ((x * 31 + y * 3) % 253) as u8;
                v[i + 2] = ((x ^ y) % 249) as u8;
                v[i + 3] = ((x * 5 + y * 11) % 255) as u8;
            }
        }
        v
    }

    /// ⭐⭐ **AS DUAS ROTAS DÃO O MESMO `f32`, BIT A BIT** — é isso que faz do paralelo uma
    /// optimização e não um segundo produto.
    ///
    /// A propriedade não é um acidente, é a FORMA das fatias: na horizontal uma linha de saída é
    /// função só da linha de entrada dela; na vertical a fatia é de **COLUNAS**, e uma coluna
    /// recebe exactamente a mesma sequência de adições nas duas rotas.
    ///
    /// ⛔ **Com fatias de LINHAS a igualdade cai**, e é por isso que a implementação não as usa:
    /// cada fatia teria de re-semear o acumulador somando `lado` linhas de fresco, e esse `f32`
    /// não é o que a soma corrida acumulou até ali. *A mesma não-associatividade que a cerca do
    /// `rayon` desta crate já nomeia para o depósito das arestas.*
    ///
    /// **Mutação que sangra:** trocar a banda de colunas por uma de linhas com semente refeita.
    #[test]
    fn as_duas_rotas_dao_o_mesmo_f32() {
        let (w, h) = (813usize, 197usize);
        let buf = tela(w, h);
        // Várias larguras de banda e vários `k` — ⚠️ uma região estreita de mais tem MENOS bandas
        // que threads, e um `k` pequeno faz `box_radii` devolver raios diferentes.
        // ⚠️⚠️ **`(800, 120)` fecha um buraco que o próprio doc desta função nomeava:** as outras
        // quatro regiões estão TODAS abaixo do piso de `LARGURA_MINIMA_DA_BANDA`, logo a rota
        // paralela tomava a saída antecipada e `bandas_da_vertical` nunca chegava a partir nada
        // aqui. *Uma paridade cuja fixtura não alcança o regime que ela julga é verde a afirmar
        // nada* — e é essa a largura em que o produto vive.
        for (bw, bh) in [
            (64usize, 64usize),
            (200, 150),
            (291, 195),
            (7, 190),
            (800, 120),
        ] {
            for k in [1usize, 3, 8, 24, 57] {
                for wrap in [[false, false], [true, true]] {
                    let a = blur_region_caixa_com(
                        &buf, w as i64, h as i64, 1, 1, bw, bh, k, wrap, false,
                    );
                    let b = blur_region_caixa_com(
                        &buf, w as i64, h as i64, 1, 1, bw, bh, k, wrap, true,
                    );
                    assert_eq!(a.len(), b.len(), "{bw}×{bh} k={k}: tamanhos");
                    let difs = a.iter().zip(&b).filter(|(x, y)| x != y).count();
                    assert_eq!(
                        difs, 0,
                        "{bw}×{bh} k={k} wrap={wrap:?}: {difs} pixels diferem entre as rotas"
                    );
                    // CONTROLO: a saída não é trivial — um borrão que devolvesse tudo zero
                    // satisfaria a igualdade sem afirmar nada.
                    assert!(
                        a.iter().any(|p| p[3] > 1.0),
                        "{bw}×{bh} k={k}: a fixtura não contém borrão nenhum"
                    );
                }
            }
        }
    }

    /// ⭐ **A DECISÃO DE PARTIR SEGUE O JOELHO MEDIDO** — o único observável, sem relógio, de uma
    /// rota que é byte-idêntica por construção.
    ///
    /// ⚠️ Sem este gate, apagar o paralelo do produto não acorda nada: as outras seis réguas
    /// afirmam que as duas rotas CONCORDAM, e desligar uma delas mantém-nas a concordar. *Uma
    /// optimização bit-idêntica não tem régua de valor; o que ela tem é a DECISÃO.*
    ///
    /// **Mutação que sangra:** `bw * bh >= usize::MAX` (a rota paralela nunca arma).
    #[test]
    fn a_decisao_de_partir_segue_o_joelho_medido() {
        use super::super::{PIXEIS_PARA_PARALELIZAR as JOELHO, vale_a_pena_partir};
        // Abaixo do joelho o paralelo PERDE (medido: `0,44×` a `64²`) ⇒ não se parte.
        for (w, h) in [(1usize, 1usize), (64, 64), (128, 128), (192, 192)] {
            assert!(
                !vale_a_pena_partir(w, h),
                "{w}×{h} = {} px está abaixo do joelho ({JOELHO}) e não se parte",
                w * h
            );
        }
        // No joelho e acima dele, parte-se.
        for (w, h) in [(256usize, 256usize), (384, 384), (1484, 1484), (4096, 512)] {
            assert!(
                vale_a_pena_partir(w, h),
                "{w}×{h} = {} px está no joelho ({JOELHO}) ou acima e parte-se",
                w * h
            );
        }
        // E a região que o PRODUTO compõe num carimbo de figura a `2048²` está do lado que parte
        // — senão a cura não alcança o report que a motivou.
        assert!(vale_a_pena_partir(1484, 1484));
    }

    /// ⛔⛔⛔ **AS BANDAS DE COLUNAS DÃO O MESMO QUE UMA BANDA SÓ** — e este gate existe porque
    /// uma mutação sobreviveu a TODOS os outros.
    ///
    /// ⚠️⚠️ **Nenhuma das outras réguas chegava a correr o código que parte as bandas.** O número
    /// delas é `w / LARGURA_MINIMA_DA_BANDA`, e as regiões das fixturas (`59`–`291` px) estão
    /// todas ABAIXO do piso de `384` ⇒ `bandas_da_vertical` devolve `1`, a passagem toma a saída
    /// antecipada e o `split_at_mut` **nunca executa**. *Uma paridade cuja fixtura não alcança o
    /// regime que ela julga é verde a afirmar nada* — a mutação que dava a TODA banda largura `1`
    /// (logo a maior parte das colunas nunca escrita) passava nos cinco gates.
    ///
    /// ⇒ aqui a contagem é um ARGUMENTO, e não o que a região e a máquina decidem.
    ///
    /// **Mutação que sangra:** `resto.split_at_mut(lw.min(1))`.
    #[test]
    fn as_bandas_de_colunas_dao_o_mesmo_que_uma_banda_so() {
        use super::super::caixa_v;
        let (w, h) = (167usize, 97usize);
        let src: Vec<[f32; 4]> = (0..w * h)
            .map(|p| {
                let (j, i) = ((p / w) as f32, (p % w) as f32);
                [i * 1.5 + j, i - j * 2.0, i * j * 0.01, 200.0 + i * 0.1]
            })
            .collect();
        for r in [1usize, 3, 7] {
            let (base, oh) = caixa_v(&src, w, h, r, 1);
            for nb in [2usize, 3, 4, 7, 16, 167] {
                let (got, oh2) = caixa_v(&src, w, h, r, nb);
                assert_eq!((got.len(), oh2), (base.len(), oh), "r={r} nb={nb}: forma");
                let difs = got.iter().zip(&base).filter(|(a, b)| a != b).count();
                assert_eq!(
                    difs, 0,
                    "r={r} nb={nb}: {difs} pixels diferem de uma banda só"
                );
            }
        }
        // CONTROLO: a saída não é toda zero — senão «igual» seria verdade por vácuo, que é
        // exactamente o que a mutação das bandas de largura `1` produzia.
        let (base, _) = caixa_v(&src, w, h, 3, 1);
        assert!(
            base.iter().filter(|p| p[3] > 1.0).count() > base.len() / 2,
            "a fixtura não contém saída nenhuma"
        );
    }

    /// ⛔⛔⛔ **A CAIXA TRIPLA CONCORDA COM UMA CONVOLUÇÃO DIRECTA** — o único gate desta crate
    /// que julga os VALORES da passagem, e ele nasceu de **duas mutações sobreviventes**.
    ///
    /// As outras réguas são todas cegas à soma corrida, cada uma por um mecanismo diferente:
    /// o campo constante é invariante a um deslocamento · a variância é uma propriedade das
    /// LARGURAS e nunca corre a passagem · a paridade compara duas rotas que **partilham o corpo
    /// da banda** · e o impulso tem a região centrada nele, logo **as linhas da SEMENTE
    /// (`0..lado`) não contêm sinal nenhum** e uma mutação ali não se vê.
    ///
    /// ⇒ o oráculo tem de ser **independente da soma corrida**: a mesma convolução escrita como
    /// somas directas da janela. ⚠️ A fixtura é **OPACA** de propósito — com `α = 255` a
    /// premultiplicação e a divisão que a desfaz são a identidade, logo a referência é o box³ dos
    /// bytes e não é preciso reconstruir o avental.
    ///
    /// **Mutações que sangram:** a semente da banda desalinhada por uma coluna · as bandas de
    /// saída desalinhadas das de entrada.
    #[test]
    fn a_caixa_tripla_concorda_com_uma_convolucao_directa() {
        use super::super::box_radii;
        let (w, h) = (81usize, 81usize);
        // Conteúdo que não é plano NEM simétrico — os dois esconderiam um deslocamento.
        let mut buf = vec![255u8; w * h * 4];
        for y in 0..h {
            for x in 0..w {
                let i = (y * w + x) * 4;
                buf[i] = ((x * 17 + y * 5) % 241) as u8;
                buf[i + 1] = ((x * x + y * 3) % 239) as u8;
                buf[i + 2] = ((x + y * y) % 233) as u8;
            }
        }
        let k = 8usize;
        let raios = box_radii(k);
        let r_total: usize = raios.iter().sum();
        let lado = 81 - 2 * r_total; // a região cabe com o avental TODO dentro da tela
        let off = r_total as i64;
        assert!(lado >= 9, "a fixtura tem de deixar uma região utilizável");

        // A REFERÊNCIA: box³ por somas DIRECTAS da janela, sobre o avental da região.
        let ap = lado + 2 * r_total;
        let mut cur: Vec<[f32; 4]> = (0..ap * ap)
            .map(|p| {
                let (j, i) = (p / ap, p % ap);
                let si = ((j * w) + i) * 4;
                [
                    f32::from(buf[si]),
                    f32::from(buf[si + 1]),
                    f32::from(buf[si + 2]),
                    f32::from(buf[si + 3]),
                ]
            })
            .collect();
        let (mut cw, mut ch) = (ap, ap);
        for r in raios {
            let (nw, n) = (cw - 2 * r, 2 * r + 1);
            let mut out = vec![[0f32; 4]; nw * ch];
            for j in 0..ch {
                for i in 0..nw {
                    let mut a = [0f64; 4];
                    for t in 0..n {
                        for c in 0..4 {
                            a[c] += f64::from(cur[j * cw + i + t][c]);
                        }
                    }
                    for c in 0..4 {
                        out[j * nw + i][c] = (a[c] / f64::from(n as u32)) as f32;
                    }
                }
            }
            cur = out;
            cw = nw;
        }
        for r in raios {
            let (nh, n) = (ch - 2 * r, 2 * r + 1);
            let mut out = vec![[0f32; 4]; cw * nh];
            for j in 0..nh {
                for i in 0..cw {
                    let mut a = [0f64; 4];
                    for t in 0..n {
                        for c in 0..4 {
                            a[c] += f64::from(cur[(j + t) * cw + i][c]);
                        }
                    }
                    for c in 0..4 {
                        out[j * cw + i][c] = (a[c] / f64::from(n as u32)) as f32;
                    }
                }
            }
            cur = out;
            ch = nh;
        }
        assert_eq!((cw, ch), (lado, lado));

        for paralelo in [false, true] {
            let got = blur_region_caixa_com(
                &buf,
                w as i64,
                h as i64,
                off,
                off,
                lado,
                lado,
                k,
                [false, false],
                paralelo,
            );
            // A barra é o ERRO DE ARREDONDAMENTO de somas em `f32` contra o mesmo em `f64`, e
            // não um epsilon escolhido: o pior caso é `lado³` adições de valores `< 256`.
            let mut pior = 0f32;
            for (g, r) in got.iter().zip(&cur) {
                for c in 0..4 {
                    pior = pior.max((g[c] - r[c]).abs());
                }
            }
            assert!(
                pior < 0.05,
                "paralelo={paralelo}: a passagem discorda da convolução directa por {pior}"
            );
            // CONTROLO: a saída não é o próprio sinal — houve borrão. ⚠️ Medido sobre a região
            // INTEIRA e não num ponto: numa rampa suave um pixel qualquer borra para quase ele
            // próprio, e um controlo de um ponto reprova sobre produto correcto (medido).
            let mut maior = 0f32;
            for j in 0..lado {
                for i in 0..lado {
                    let cru = f32::from(buf[((j + r_total) * w + i + r_total) * 4]);
                    maior = maior.max((got[j * lado + i][0] - cru).abs());
                }
            }
            assert!(
                maior > 8.0,
                "paralelo={paralelo}: a fixtura não contém borrão nenhum (pior desvio {maior})"
            );
        }
    }

    /// ⛔⛔ **A RESPOSTA A UM IMPULSO É SIMÉTRICA E CENTRADA** — e este gate existe porque uma
    /// MUTAÇÃO SOBREVIVEU.
    ///
    /// O gate da paridade compara as duas rotas, e **as duas partilham o corpo da banda**: uma
    /// mutação DENTRO dele move os dois lados por igual e a igualdade continua a valer. *Um
    /// oráculo de IGUALDADE só afirma sobre o que as duas rotas fazem DIFERENTE.* Medido:
    /// desalinhar a semente da banda por uma coluna (`src[… + i + 1]`) deixava os quatro gates
    /// desta crate VERDES — a passagem vertical não tinha régua nenhuma sobre os VALORES dela,
    /// e a que havia (um campo constante) é cega a um deslocamento por construção.
    ///
    /// A régua que separa é o **impulso**: um pixel aceso no centro tem de sair como um perfil
    /// simétrico à volta dele, nos dois eixos. Um deslocamento de UMA coluna quebra-a.
    ///
    /// **Mutação que sangra:** `src[j * w + c0 + i + 1]` na semente da banda.
    #[test]
    fn a_resposta_a_um_impulso_e_simetrica_e_centrada() {
        let (w, h) = (129usize, 129usize);
        let mut buf = vec![0u8; w * h * 4];
        let centro = (h / 2) * w + w / 2;
        buf[centro * 4..centro * 4 + 4].copy_from_slice(&[255, 255, 255, 255]);
        let (lado, k) = (97usize, 12usize);
        let off = ((w - lado) / 2) as i64;
        for paralelo in [false, true] {
            let out = blur_region_caixa_com(
                &buf,
                w as i64,
                h as i64,
                off,
                off,
                lado,
                lado,
                k,
                [false, false],
                paralelo,
            );
            let c = lado / 2;
            // O impulso está no centro da região (a região é ímpar e centrada nele).
            assert_eq!(
                w / 2 - off as usize,
                c,
                "a fixtura tem de centrar o impulso"
            );
            // CONTROLO: há borrão — um `k` que devolvesse o impulso intacto satisfaria a
            // simetria sem afirmar nada sobre a passagem.
            let vizinho = out[c * lado + c + 3][3];
            assert!(
                vizinho > 0.0,
                "paralelo={paralelo}: o impulso não se espalhou, a fixtura não contém borrão"
            );
            for d in 1..=20usize {
                let (e, dd) = (out[c * lado + c - d][3], out[c * lado + c + d][3]);
                assert!(
                    (e - dd).abs() <= f32::EPSILON * 8.0,
                    "paralelo={paralelo}: assimetria horizontal a {d}: {e} contra {dd}"
                );
                let (ci, ba) = (out[(c - d) * lado + c][3], out[(c + d) * lado + c][3]);
                assert!(
                    (ci - ba).abs() <= f32::EPSILON * 8.0,
                    "paralelo={paralelo}: assimetria vertical a {d}: {ci} contra {ba}"
                );
            }
            // E o máximo está NO centro, não ao lado dele.
            let pico = out[c * lado + c][3];
            assert!(
                pico >= out[c * lado + c + 1][3] && pico >= out[(c + 1) * lado + c][3],
                "paralelo={paralelo}: o pico não está no centro"
            );
        }
    }

    /// ⭐ **UMA BANDA NUNCA É MAIS ESTREITA DO QUE O PISO MEDIDO** — a lei que a tabela de
    /// [`super::super::LARGURA_MINIMA_DA_BANDA`] comprou, afirmada sem um relógio.
    ///
    /// ⚠️⚠️ **A premissa dele foi dada por MORTA e RENASCEU no mesmo dia — e isto fica escrito
    /// porque é a lição.** Quando a vertical fundiu (2026-09-22) o caminho de omissão deixou de
    /// chamar a `bandas_da_vertical`, e eu declarei a premissa morta; horas depois a **porta de
    /// bissecção** ([`super::super::sem_fusao`], que a casa exige a toda troca de motor) devolveu
    /// aquela rota ao produto, e com ela a lei. ⇒ *uma premissa só morre quando o código que a
    /// realiza deixa de ser alcançável, e «o caminho de omissão mudou» não é isso.*
    ///
    /// ⭐ Hoje há **duas** leis de banda e cada uma tem o gate dela: esta, sobre a rota de
    /// bissecção, e `a_vertical_fundida_parte_em_bandas_de_cache` sobre a
    /// [`super::super::LARGURA_DA_BANDA_FUNDIDA`], que é a do caminho de omissão.
    ///
    /// ⚠️ **Ela não pode ser «`bandas_da_vertical(1484) == 4`»:** a contagem é limitada pela pool,
    /// logo esse número é da MÁQUINA e o gate mediria o escalonador. O que é da LEI é a relação —
    /// *a banda que sai tem pelo menos a largura do piso* —, e ela vale em qualquer máquina.
    ///
    /// **Mutação que sangra:** `bandas_da_vertical` devolver `fatias_do_soquete()` (o piso
    /// apagado) — numa região larga a banda fica com `w / threads` px, abaixo do piso.
    #[test]
    fn uma_banda_nunca_fica_abaixo_do_piso() {
        use super::super::{LARGURA_MINIMA_DA_BANDA as PISO, bandas_da_vertical};
        let mut houve_larga = false;
        for w in [1usize, 7, 100, 383, 384, 385, 800, 1484, 4096, 16384] {
            let nb = bandas_da_vertical(w);
            assert!(nb >= 1, "w={w}: pelo menos uma banda");
            if w < PISO {
                assert_eq!(nb, 1, "w={w}: abaixo do piso não se parte nada");
            } else {
                assert!(
                    w / nb >= PISO,
                    "w={w}: {nb} bandas dão {} px, abaixo do piso de {PISO}",
                    w / nb
                );
                houve_larga = true;
            }
        }
        // CONTROLO: a varredura chegou a conter uma região larga — senão a metade que importa
        // nunca correu e o gate passaria por vácuo.
        assert!(
            houve_larga,
            "a varredura não conteve nenhuma região acima do piso"
        );
    }

    /// **QUAL ETAPA SE RECUSA A ESCALAR** — a sonda do soquete deu `4,05×` de folga com `N`
    /// borrões INDEPENDENTES, e partir UM borrão em fatias dá `1,53×`. A diferença são as sete
    /// barreiras de junção e o desequilíbrio de cada etapa; esta mede-as uma a uma.
    /// `cargo test -p ph2d-painter-brush --release paralelo_tests::mede_a_etapa -- --ignored
    /// --nocapture`
    #[test]
    #[ignore = "sonda de relógio"]
    fn mede_a_etapa_que_nao_escala() {
        use super::super::{box_radii, caixa_h, caixa_v};
        let (w, h) = (2048usize, 2048usize);
        let buf = tela(w, h);
        let (lado, k) = (1484usize, 24usize);
        let raios = box_radii(k);
        let r_total: usize = raios.iter().sum();
        eprintln!("k={k} raios={raios:?} r_total={r_total}");
        // O avental, construído em série uma vez, é a entrada das seis passagens.
        let apron = super::super::blur_region_caixa_com(
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
        let (aw, ah) = (lado, lado);
        for (nome, vertical) in [("caixa_h", false), ("caixa_v", true)] {
            for r in raios {
                let mut ts = f64::MAX;
                let mut tp = f64::MAX;
                for _ in 0..7 {
                    let t = std::time::Instant::now();
                    let a = if vertical {
                        caixa_v(&apron, aw, ah, r, 1)
                    } else {
                        caixa_h(&apron, aw, ah, r, false)
                    };
                    ts = ts.min(t.elapsed().as_secs_f64() * 1e3);
                    let t = std::time::Instant::now();
                    let b = if vertical {
                        caixa_v(&apron, aw, ah, r, super::super::fatias_do_soquete())
                    } else {
                        caixa_h(&apron, aw, ah, r, true)
                    };
                    tp = tp.min(t.elapsed().as_secs_f64() * 1e3);
                    assert_eq!(a.0.len(), b.0.len());
                }
                eprintln!(
                    "{nome} r={r:>2} | série {ts:>7.3} ms | fatias {tp:>7.3} ms | {:>5.2}×",
                    ts / tp
                );
            }
        }
        eprintln!("fatias do soquete: {}", super::super::fatias_do_soquete());
    }

    /// **A ESCADA que fixa [`PIXEIS_PARA_PARALELIZAR`]** — abaixo do joelho acordar a pool custa
    /// mais do que o trabalho que ela poupa.
    /// `cargo test -p ph2d-painter-brush --release blur_caixa::paralelo_tests::mede -- --ignored
    /// --nocapture`
    #[test]
    #[ignore = "sonda de relógio"]
    fn mede_o_joelho_do_paralelo() {
        let (w, h) = (2048usize, 2048usize);
        let buf = tela(w, h);
        eprintln!("limiar em vigor: {PIXEIS_PARA_PARALELIZAR} px");
        for lado in [64usize, 128, 192, 256, 384, 512, 768, 1024, 1484] {
            let mut t_s = f64::MAX;
            let mut t_p = f64::MAX;
            for _ in 0..7 {
                let t = std::time::Instant::now();
                let a = blur_region_caixa_com(
                    &buf,
                    w as i64,
                    h as i64,
                    8,
                    8,
                    lado,
                    lado,
                    24,
                    [false, false],
                    false,
                );
                t_s = t_s.min(t.elapsed().as_secs_f64() * 1e3);
                let t = std::time::Instant::now();
                let b = blur_region_caixa_com(
                    &buf,
                    w as i64,
                    h as i64,
                    8,
                    8,
                    lado,
                    lado,
                    24,
                    [false, false],
                    true,
                );
                t_p = t_p.min(t.elapsed().as_secs_f64() * 1e3);
                assert_eq!(a.len(), b.len());
            }
            eprintln!(
                "{:>5}² = {:>8} px | série {t_s:>7.3} ms | fatias {t_p:>7.3} ms | {:>5.2}×",
                lado,
                lado * lado,
                t_s / t_p
            );
        }
    }
}
