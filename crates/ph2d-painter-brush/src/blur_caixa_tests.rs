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
    /// vale para toda partição — e o borrão passa a medir `0,57×`, ou seja **pior do que as três
    /// passagens separadas que a fusão veio substituir**. *A lei da identidade não pode gatear a
    /// lei do custo; quem a gateia é a contagem.*
    ///
    /// ⚠️ **Corre em SÉRIE de propósito:** o contador é por THREAD (senão o fan-out da suíte conta
    /// as bandas de outra corrida), e em série todas as bandas passam por esta.
    ///
    /// **Mutações que sangram:** `nb = 1` · ignorar o `largura_da_banda` · arredondar para baixo
    /// em vez de `div_ceil` (a última coluna ficaria fora de toda banda).
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
                let v =
                    super::super::blur_region_caixa(&buf, fw, fh, 0, 0, bw, bh, k, [false, false]);
                porta = v.len() as f64;
            });
            let _ = porta;

            let raios = super::super::box_radii(k);
            let r_total: usize = raios.iter().sum();
            let (ap_w, ap_h) = (bw + 2 * r_total, bh + 2 * r_total);
            let paralelo = super::super::vale_a_pena_partir(bw, bh);

            // (1) o AVENTAL — replicado, e é a única parte que não é a função do produto; o controlo
            //     da soma abaixo é o que impede esta réplica de mentir.
            let mut apron = Vec::new();
            let t_ap = ms(&mut || {
                apron = super::super::avental(
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
                    let (n, nw) = super::super::caixa_h(ent, ww, ap_h, r, paralelo);
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
                let _ = super::super::caixa_h3(&apron, ap_w, ap_h, raios, paralelo);
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
                    let (n, nw) = super::super::caixa_h(ent, ww, ap_h, r, false);
                    c = n;
                    ww = nw;
                    ent = &c;
                }
                std::hint::black_box(c.len());
            });
            let t_h3_serie = ms(&mut || {
                let v = super::super::caixa_h3(&apron, ap_w, ap_h, raios, false);
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
                        super::super::bandas_da_vertical(w)
                    } else {
                        1
                    };
                    let (n, nh) = super::super::caixa_v(ent, w, hh, r, fatias);
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
                    let (n, nh) = super::super::caixa_v(ent, w, hh, r, 1);
                    c = n;
                    hh = nh;
                    ent = &c;
                }
                std::hint::black_box(c.len());
            });
            let t_v3_serie = ms(&mut || {
                let v = super::super::caixa_v3(
                    &entrada_v,
                    w,
                    h,
                    raios,
                    super::super::LARGURA_DA_BANDA_FUNDIDA,
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
                let v =
                    super::super::blur_region_caixa(&buf, fw, fh, 0, 0, bw, bh, k, [false, false]);
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

    /// **A LARGURA DA BANDA DA VERTICAL FUNDIDA, varrida** — o número da
    /// [`super::super::LARGURA_DA_BANDA_FUNDIDA`] sai daqui e de mais lado nenhum.
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
            let raios = super::super::box_radii(k);
            let r_total: usize = raios.iter().sum();
            let (ap_w, ap_h) = (bw + 2 * r_total, bh + 2 * r_total);
            let (fw, fh) = (2048i64, 2048i64);
            let buf: Vec<u8> = (0..(fw * fh * 4) as usize)
                .map(|i| ((i * 37) % 256) as u8)
                .collect();
            let apron = super::super::avental(
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
            let (ent, w) = super::super::caixa_h3(&apron, ap_w, ap_h, raios, false);

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
                    let (n, nh) = super::super::caixa_v(e, w, hh, r, 1);
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
                    let v = super::super::caixa_v3(&ent, w, ap_h, raios, largura, false);
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
}
