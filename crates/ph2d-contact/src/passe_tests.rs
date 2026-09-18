//! Os gates do passe automático (doc 115 W5).
//!
//! ⚠️ **A fixtura declara pela convenção do OBJECTO** (a caixa em geometria mais o `size` que a
//! converte, doc 115 §12.1) e não por um número de mundo cravado: é a rota que o produto usa desde
//! a W4, e uma fixtura que escrevesse a meia de mundo directamente passaria no dia em que a
//! conversão se partisse.

use super::separa_o_que_se_desenha;
use crate::Forma;
use ph2d_nodegraph::attr::{COLLIDER_BOX_COLUMN, COLLIDER_COLUMN, Column, Stream};

/// A meia que um OBJECTO declara — geometria, convertida pelo `size` da linha.
const MEIA: [f32; 2] = [0.5, 0.5];

/// `n` peças de lado `1` nas posições dadas, cada uma a declarar a caixa dela.
fn cena(p: &[[f32; 2]]) -> Stream {
    let n = p.len();
    Stream::new(n)
        .with("P", Column::Vec2(p.to_vec()))
        .with("size", Column::Vec2(vec![[1.0, 1.0]; n]))
        .with(COLLIDER_BOX_COLUMN, Column::Vec2(vec![MEIA; n]))
}

fn posicoes(s: &Stream) -> Vec<[f32; 2]> {
    match s.get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => panic!("a saida tem de trazer P"),
    }
}

/// ⭐⭐⭐ **A LEI: duas peças sobrepostas deixam de se sobrepor.** O controlo é o vão — elas ficam a
/// tocar-se, não empilhadas nem atiradas para longe.
#[test]
fn duas_pecas_sobrepostas_separam_se() {
    // Lado `1`, centros a `0,5` ⇒ metade de cada uma dentro da outra.
    let s = cena(&[[0.0, 0.0], [0.5, 0.0]]);
    let out = separa_o_que_se_desenha(&s, 32).expect("uma cena sobreposta separa-se");
    let p = posicoes(&out);
    let vao = (p[1][0] - p[0][0]).abs();
    assert!(
        (vao - 1.0).abs() < 1e-3,
        "duas caixas de lado 1 assentam com os centros a 1 de distancia: {vao}"
    );
    // ⚠️ E o eixo em que NÃO havia sobreposição não se mexe — sem isto, um solver que
    // empurrasse na diagonal passaria a metade de cima deste gate.
    assert!(
        p[0][1].abs() < 1e-6 && p[1][1].abs() < 1e-6,
        "nada empurra em y: {p:?}"
    );
}

/// ⛔ **Uma corrente que NÃO declara colisor devolve `None`** — e é isto que mantém toda cena sem
/// colisão byte-idêntica: o passe sai antes de clonar sequer a corrente.
#[test]
fn sem_declaracao_o_passe_nao_toca_em_nada() {
    let s = Stream::new(2)
        .with("P", Column::Vec2(vec![[0.0, 0.0], [0.5, 0.0]]))
        .with("size", Column::Vec2(vec![[1.0, 1.0]; 2]));
    assert!(separa_o_que_se_desenha(&s, 32).is_none());
}

/// ⚠️ **Uma cena JÁ separada também devolve `None`.** *Nada se mexeu ⇒ nada se escreve* — sem esta
/// metade, toda cena com colisor pagaria uma corrente nova por quadro para entregar os mesmos bits.
#[test]
fn uma_cena_ja_separada_nao_escreve_corrente_nova() {
    let s = cena(&[[0.0, 0.0], [3.0, 0.0]]);
    assert!(separa_o_que_se_desenha(&s, 32).is_none());
}

/// ⚠️ `0` varreduras é *«não corras»*, e não *«corre e não mudes nada»*.
#[test]
fn zero_varreduras_nao_e_um_passe() {
    let s = cena(&[[0.0, 0.0], [0.5, 0.0]]);
    assert!(separa_o_que_se_desenha(&s, 0).is_none());
}

/// ⭐⭐ **Peso `0` é um OBSTÁCULO: ele não se move e o outro contorna-o.**
///
/// ⚠️ A régua é a peça PARADA e não o vão: um solver que movesse as duas metade para cada lado dá
/// exactamente o mesmo vão, e é precisamente esse o defeito que isto apanha.
#[test]
fn uma_peca_de_peso_zero_nao_se_move() {
    let s = cena(&[[0.0, 0.0], [0.5, 0.0]]).with("inv_mass", Column::Scalar(vec![0.0, 1.0]));
    let out = separa_o_que_se_desenha(&s, 32).expect("separa");
    let p = posicoes(&out);
    assert_eq!(p[0], [0.0, 0.0], "a peca de peso zero e' um obstaculo");
    assert!(
        (p[1][0] - 1.0).abs() < 1e-3,
        "a outra contorna-a inteira: {p:?}"
    );
}

/// ⭐⭐⭐ **O passe honra a CAIXA declarada e não um disco** — o discriminador é uma caixa COMPRIDA,
/// onde as duas leis dão respostas diferentes por um factor que o olho vê.
///
/// Uma caixa `4 × 1` tratada como disco teria raio `max(2; 0,5) = 2` e assentaria com os centros a
/// `4`; como caixa, dois rectângulos lado a lado assentam a `4` no eixo longo e a **`1`** no curto.
/// A fixtura empurra-os no eixo CURTO, onde a diferença é `4×`.
#[test]
fn a_caixa_declarada_ganha_ao_disco_que_a_conteria() {
    let n = 2;
    let s = Stream::new(n)
        .with("P", Column::Vec2(vec![[0.0, 0.0], [0.0, 0.5]]))
        .with("size", Column::Vec2(vec![[4.0, 1.0]; n]))
        .with(COLLIDER_BOX_COLUMN, Column::Vec2(vec![MEIA; n]));
    let out = separa_o_que_se_desenha(&s, 32).expect("separa");
    let p = posicoes(&out);
    let vao = (p[1][1] - p[0][1]).abs();
    assert!(
        (vao - 1.0).abs() < 1e-3,
        "no eixo CURTO duas caixas de altura 1 assentam a 1 — um disco daria 4: {vao}"
    );
}

/// ⚠️ **A corrente de saída mantém TODA coluna que a de entrada tinha** — o passe é um acabamento,
/// não um filtro. Sem isto, a aparência (`tint`, `uv_rect`, `texture_id`…) evaporava no dia em que
/// a cena ganhasse um colisor.
#[test]
fn o_passe_devolve_todas_as_colunas_que_recebeu() {
    let s = cena(&[[0.0, 0.0], [0.5, 0.0]])
        .with("tint", Column::Vec4(vec![[1.0, 0.0, 0.0, 1.0]; 2]))
        .with("texture_id", Column::Scalar(vec![7.0, 7.0]));
    let out = separa_o_que_se_desenha(&s, 32).expect("separa");
    for nome in ["size", COLLIDER_BOX_COLUMN, "tint", "texture_id"] {
        assert!(out.get(nome).is_some(), "a coluna `{nome}` evaporou");
    }
    assert_eq!(out.count(), s.count());
}

/// ⭐ **O `rot` ACUMULA sobre o que já lá estava, nunca o substitui** — a peça pode chegar ao passe
/// já rodada por um `motion.rotate` a montante, e escrever só o giro apagaria essa autoria.
///
/// ⚠️ E numa cena sem rotação **a coluna não nasce**: acrescentá-la muda a resposta de todo
/// consumidor que distinga *«sem rotação»* de *«rotação zero»* (a lei estrutural do `motion.collide`).
///
/// # ⛔ A 1.ª redacção deste gate tinha uma premissa MINHA errada, e ele apanhou-a
///
/// Ela punha `rot = 30` em duas CAIXAS e afirmava que a saída seria `30` intacto — *«aqui nada
/// roda»*. **Roda:** duas caixas inclinadas tocam-se por uma quina, o braço da força não passa pelo
/// centro, e o solver devolve `−9,46°` a cada uma (medido: a saída lê `20,54`). ⇒ a régua da
/// PRESERVAÇÃO tem de correr sobre uma forma que **não pode** rodar, e a da ACUMULAÇÃO sobre uma que
/// roda — são duas fixturas, não duas asserções sobre a mesma.
#[test]
fn o_rot_acumula_e_so_nasce_quando_alguma_peca_roda() {
    // (a) Sem rotação nenhuma: caixas alinhadas aos eixos, empurrão no eixo ⇒ giro zero.
    let sem_giro = separa_o_que_se_desenha(&cena(&[[0.0, 0.0], [0.5, 0.0]]), 32).expect("separa");
    assert!(
        sem_giro.get("rot").is_none(),
        "nenhuma peca rodou — a coluna nao pode nascer"
    );

    // (b) A PRESERVAÇÃO, numa forma que não tem quina: dois DISCOS tocam-se ao longo da linha dos
    // centros, o braço é exactamente zero, e o `rot` autorado sai intacto ao bit.
    let n = 2;
    let discos = Stream::new(n)
        .with("P", Column::Vec2(vec![[0.0, 0.0], [0.5, 0.0]]))
        .with("size", Column::Vec2(vec![[1.0, 1.0]; n]))
        .with(COLLIDER_COLUMN, Column::Scalar(vec![0.5; n]))
        .with("rot", Column::Scalar(vec![30.0, 30.0]));
    let out = separa_o_que_se_desenha(&discos, 32).expect("separa");
    match out.get("rot") {
        Some(Column::Scalar(v)) => assert_eq!(
            *v,
            vec![30.0, 30.0],
            "o `rot` autorado a montante tem de sobreviver ao passe"
        ),
        outro => panic!("o `rot` de entrada evaporou: {outro:?}"),
    }

    // (c) A ACUMULAÇÃO, na forma que roda: a saída fica na vizinhança do valor AUTORADO e não no
    // giro sozinho. ⚠️ **A banda sai da medição** — o giro deste arranjo é `−9,46°`, logo acumular
    // lê `20,54` e SUBSTITUIR leria `−9,46`: a barra de `15°` separa as duas com folga de `5,5`
    // para um lado e `24,5` para o outro, e é a mutação que ela existe para matar.
    let caixas = cena(&[[0.0, 0.0], [0.5, 0.0]]).with("rot", Column::Scalar(vec![30.0, 30.0]));
    let out = separa_o_que_se_desenha(&caixas, 32).expect("separa");
    match out.get("rot") {
        Some(Column::Scalar(v)) => {
            for (i, r) in v.iter().enumerate() {
                assert!(
                    (r - 30.0).abs() < 15.0,
                    "a peca {i} leu `{r}` — isto e' o GIRO sozinho, nao o autorado mais o giro"
                );
                assert!(
                    *r != 30.0,
                    "a peca {i} nao rodou nada, e uma quina tem de rodar"
                );
            }
        }
        outro => panic!("as caixas inclinadas rodaram e a coluna nao saiu: {outro:?}"),
    }
}

/// ⚠️ **Uma peça que não declara NADA não participa, e as que declaram continuam a separar-se.**
///
/// ⛔ É a diferença explícita para o `motion.collide`, que dá a quem não declara o disco do
/// `Radius` do cartão dele — *aqui não há cartão de onde tirar um raio, e inventar um seria
/// afirmar que uma peça colide quando ninguém o disse*.
///
/// # ⛔⛔ A 1.ª fixtura punha a peça muda no ponto de SIMETRIA, e a mutação SOBREVIVEU
///
/// Ela estava em `0,25`, **exactamente a meio** das outras duas (`0` e `0,5`). O solver é Jacobi
/// com médias: ali os empurrões dos dois vizinhos cancelam-se ao bit, logo a peça fica onde está
/// **participe ou não** — e a mutação que lhe dava um disco inventado passou no gate. ⇒ a posição é
/// **assimétrica** agora, que é a única em que as duas leis dão respostas diferentes.
///
/// *É a mesma família de «um corpus no ponto neutro de um knob não testa esse knob», uma camada
/// abaixo: aqui o ponto neutro é da GEOMETRIA.*
#[test]
fn quem_nao_declara_nao_participa() {
    let n = 3;
    let s = Stream::new(n)
        .with("P", Column::Vec2(vec![[0.0, 0.0], [0.5, 0.0], [0.2, 0.0]]))
        .with("size", Column::Vec2(vec![[1.0, 1.0]; n]))
        // A terceira declara `[0, 0]`, que a porta da leitura lê como *«não colide»*.
        .with(
            COLLIDER_BOX_COLUMN,
            Column::Vec2(vec![MEIA, MEIA, [0.0, 0.0]]),
        );
    let out = separa_o_que_se_desenha(&s, 32).expect("separa");
    let p = posicoes(&out);
    assert_eq!(p[2], [0.2, 0.0], "a peca sem forma fica onde estava");
    assert!(
        (p[1][0] - p[0][0]).abs() > 0.9,
        "as duas que declaram separam-se na mesma: {p:?}"
    );
}

/// ⚠️ **O passe lê o que a corrente declara, e um RAIO também é uma declaração** — a forma com
/// `Collider Shape = Circle` escreve [`COLLIDER_COLUMN`], e o passe tem de a honrar.
#[test]
fn um_raio_declarado_tambem_separa() {
    let n = 2;
    let s = Stream::new(n)
        .with("P", Column::Vec2(vec![[0.0, 0.0], [0.5, 0.0]]))
        .with("size", Column::Vec2(vec![[1.0, 1.0]; n]))
        .with(COLLIDER_COLUMN, Column::Scalar(vec![0.5; n]));
    let out = separa_o_que_se_desenha(&s, 32).expect("separa");
    let p = posicoes(&out);
    let c = crate::colisores(&s).expect("declara")[0].expect("um disco");
    assert!(matches!(c.forma, Forma::Disco(_)), "a fixtura e' um disco");
    assert!(
        (p[1][0] - p[0][0]).abs() > 0.9,
        "dois discos de raio 0,5 assentam a 1: {p:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// doc 115 W6 §15.1 — A ATENUAÇÃO (o `falloff`), que a W5 recusou por erro de categoria
// ─────────────────────────────────────────────────────────────────────────────

/// A mesma cena, com uma atenuação por peça.
fn cena_atenuada(p: &[[f32; 2]], fall: &[f32]) -> Stream {
    assert_eq!(p.len(), fall.len(), "CONTROLO do arnes: uma per peca");
    cena(p).with("falloff", Column::Scalar(fall.to_vec()))
}

/// ⭐⭐⭐ **`falloff = 0` deixa a peça ONDE ESTAVA — é a resposta ao §10.4.**
///
/// Uma `source.shape` tem o `Collide` do cartão dela; um Sprite não tem cartão nenhum. O que ele
/// tem é a CORRENTE, e um campo a montante põe-lhe esta coluna — *o objecto passa a ter como não
/// ser mexido, sem um param novo, sem degrau de schema e sem uma linha de Inspector*.
///
/// ⚠️ **As duas metades:** a peça atenuada fica parada **e** a vizinha continua a mexer-se. Sem a
/// segunda, um passe que simplesmente desistisse da cena inteira passaria aqui.
#[test]
fn atenuacao_zero_deixa_a_peca_onde_estava_e_a_vizinha_mexe_se() {
    let p0 = [[0.0, 0.0], [0.5, 0.0]];
    let out = separa_o_que_se_desenha(&cena_atenuada(&p0, &[0.0, 1.0]), 32)
        .expect("a cena ainda tem o que separar");
    let p = posicoes(&out);
    assert_eq!(
        p[0], p0[0],
        "a peca com `falloff = 0` nao pode ser movida um bit"
    );
    assert!(
        (p[1][0] - p0[1][0]).abs() > 1e-4,
        "e a vizinha TEM de se mexer — senao este gate passaria sobre um passe inerte"
    );
}

/// ⭐⭐ **E a lei é a MISTURA, não um interruptor:** `0,5` anda exactamente metade do caminho.
///
/// ⛔ Sem este gate, `k` escrito como `if fall > 0 { 1 } else { 0 }` passaria o gate acima e
/// divergiria do `motion.collide` em todo valor intermédio — *em silêncio, e só numa cena que
/// alguém tivesse migrado*.
#[test]
fn atenuacao_meia_anda_metade_do_caminho() {
    let p0 = [[0.0, 0.0], [0.5, 0.0]];
    let cheio = posicoes(&separa_o_que_se_desenha(&cena(&p0), 32).expect("cheio"));
    let meio =
        posicoes(&separa_o_que_se_desenha(&cena_atenuada(&p0, &[0.5, 0.5]), 32).expect("atenuado"));
    for i in 0..2 {
        let esperado = p0[i][0] + (cheio[i][0] - p0[i][0]) * 0.5;
        assert!(
            (meio[i][0] - esperado).abs() < 1e-6,
            "peca {i}: `falloff = 0,5` anda metade — esperado {esperado}, veio {}",
            meio[i][0]
        );
    }
    // ⚠️ CONTROLO: o caminho cheio tem de ser mesmo diferente do meio, senão a conta acima é
    // trivialmente verdadeira sobre um deslocamento nulo.
    assert!(
        (cheio[0][0] - meio[0][0]).abs() > 1e-4,
        "a fixtura tem de distinguir meio de cheio"
    );
}

/// ⚠️ **Coluna AUSENTE lê-se `1`, e o caminho é BYTE-IDÊNTICO** — é a linha que impede esta wave
/// de mudar uma única cena que já existia.
///
/// ⛔ E um `falloff` **malformado** (do tipo errado, ou `NaN`) cai no mesmo braço: *uma peça que
/// deixasse de se mexer por um `NaN` seria um defeito mudo*.
///
/// ⚠️⚠️ **O caso do COMPRIMENTO errado não é testável, e isso é um facto sobre a casa e não uma
/// folga:** o `Stream::with` tem um `assert` (*«column length must equal stream element count»*) e
/// entra em pânico ao construir a fixtura. O braço continua no `match` de propósito — ele cobre
/// o TIPO errado, que é construtível e está medido aqui.
#[test]
fn sem_atenuacao_ou_com_ela_malformada_o_passe_e_byte_identico() {
    let p0 = [[0.0, 0.0], [0.5, 0.0]];
    let nua = posicoes(&separa_o_que_se_desenha(&cena(&p0), 32).expect("a nua separa"));

    let tipo_errado = cena(&p0).with("falloff", Column::Vec2(vec![[0.0, 0.0]; 2]));
    assert_eq!(
        posicoes(&separa_o_que_se_desenha(&tipo_errado, 32).expect("a de tipo errado separa")),
        nua,
        "um `falloff` que nao e' escalar nao descreve atenuacao nenhuma — lê-se ausente"
    );

    let nan = cena_atenuada(&p0, &[f32::NAN, f32::NAN]);
    assert_eq!(
        posicoes(&separa_o_que_se_desenha(&nan, 32).expect("a de NaN separa")),
        nua,
        "um `NaN` cai no braco de omissao (`1`), nunca em «nao mexer»"
    );
}

/// A penetração que o olho vê, em fracções do lado — a mesma barra que o doc 115 §15 usa.
const PENETRACAO_VISIVEL: f32 = 0.02;

/// Quantos pares de uma cadeia ainda se atravessam mais do que a barra.
fn pares_atravessados(p: &[[f32; 2]]) -> usize {
    let mut n = 0;
    for i in 0..p.len() {
        for j in (i + 1)..p.len() {
            let dx = (p[i][0] - p[j][0]).abs();
            let dy = (p[i][1] - p[j][1]).abs();
            // caixas de lado 1 ⇒ tocam-se quando a distância em ALGUM eixo chega a 1
            let pen = (1.0 - dx).min(1.0 - dy);
            if pen > PENETRACAO_VISIVEL {
                n += 1;
            }
        }
    }
    n
}

/// ⚠️ **SONDA, não gate** — ela imprime a escada e não afirma nada, de propósito: o número que ela
/// procura é *onde a cadeia converge com as varreduras LIVRES*, e escrever uma barra antes de o
/// saber seria o palpite que o §0.0 proíbe.
#[test]
#[ignore = "sonda de medição — corre à mão"]
fn sonda_onde_uma_cadeia_converge() {
    for n in [4usize, 8, 16, 32] {
        // uma CADEIA: peças de lado 1 a um quarto de passo, cada uma encavalitada nas vizinhas
        let p: Vec<[f32; 2]> = (0..n).map(|i| [i as f32 * 0.25, 0.0]).collect();
        let antes = pares_atravessados(&p);
        let mut linha = format!("n={n:<3} antes={antes:<5}");
        for v in [8usize, 32, 64, 128, 256, 512, 1024, 4096, 16384] {
            let fica = separa_o_que_se_desenha(&cena(&p), v)
                .map_or(antes, |s| pares_atravessados(&posicoes(&s)));
            linha.push_str(&format!(" {v}:{fica}"));
        }
        println!("{linha}");
    }
}

/// ⚠️ **SONDA de RELÓGIO** — imprime o custo por varredura. ⛔ Corre em `--release` e com a máquina
/// calma: o `/proc/loadavg` vai impresso ao lado, porque *a régua que desmente uma leitura sob
/// carga é a própria leitura sob carga*.
#[test]
#[ignore = "sonda de medição — corre à mão, em release e calmo"]
fn sonda_o_preco_de_uma_varredura() {
    let carga = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!("loadavg: {}", carga.trim());
    for n in [16usize, 64, 256, 1024, 4096] {
        let p: Vec<[f32; 2]> = (0..n).map(|i| [i as f32 * 0.25, 0.0]).collect();
        let s = cena(&p);
        // mínimo de cinco, que é o idioma de relógio desta casa
        let mut melhor = f64::INFINITY;
        for _ in 0..5 {
            let t = std::time::Instant::now();
            let _ = separa_o_que_se_desenha(&s, 64);
            melhor = melhor.min(t.elapsed().as_secs_f64());
        }
        let por_varredura_us = melhor * 1e6 / 64.0;
        println!(
            "n={n:<5} 64 varreduras: {:>9.3} ms   ⇒ {por_varredura_us:>8.3} us/varredura   \
             ⇒ cabem {:>7.0} varreduras em 8 ms",
            melhor * 1e3,
            8_000.0 / por_varredura_us
        );
    }
}

/// ⚠️ **SONDA da SOBRE-RELAXAÇÃO** — a hipótese é que o `~n²` é de JACOBI + MÉDIA, e não da lei.
/// Aqui ela é aplicada POR FORA (uma varredura de cada vez, com extrapolação `p + ω·(p′−p)`), que
/// é SOR à letra — se ela quebrar o `n²`, a cura tem endereço; se não, a hipótese morre barata.
#[test]
#[ignore = "sonda de medição — corre à mão"]
fn sonda_a_sobre_relaxacao_quebra_o_n2() {
    for n in [8usize, 16, 32] {
        let p0: Vec<[f32; 2]> = (0..n).map(|i| [i as f32 * 0.25, 0.0]).collect();
        for omega in [1.0_f32, 1.5, 1.8, 1.95] {
            let mut p = p0.clone();
            let mut quando = None;
            for k in 1..=4096usize {
                let Some(s) = separa_o_que_se_desenha(&cena(&p), 1) else {
                    break;
                };
                let np = posicoes(&s);
                for i in 0..n {
                    p[i][0] += omega * (np[i][0] - p[i][0]);
                    p[i][1] += omega * (np[i][1] - p[i][1]);
                }
                if pares_atravessados(&p) == 0 {
                    quando = Some(k);
                    break;
                }
            }
            println!(
                "n={n:<3} omega={omega:<5} -> {}",
                quando.map_or("NAO CONVERGIU em 4096".to_string(), |k| format!(
                    "convergiu em {k} varreduras"
                ))
            );
        }
    }
}

/// ⚠️⚠️ **A SONDA QUE DECIDE O DESENHO** — a ESCADA de varreduras contra o comprimento da cadeia.
///
/// A fixtura é a lei de uma corda: ela puxa para um REPOUSO (`0,5`, metade do diâmetro) e PARA —
/// ⛔ um aperto de 3 % por quadro para sempre é força ILIMITADA contra empurrão limitado, e uma
/// fixtura assim não contém o fenómeno: nenhum solver a ganha.
///
/// Mede-se o mesmo aperto em duas moradas: **FRIO** (o que o passe faz hoje — a saída é deitada
/// fora e o estado do solver continua encavalitado) e **QUENTE** (a saída separada É o estado do
/// quadro seguinte).
#[test]
#[ignore = "sonda de medição — corre à mão"]
fn sonda_o_frio_contra_o_quente() {
    const QUADROS: usize = 60;
    for (n, varreduras) in [
        (8usize, 8usize),
        (8, 32),
        (8, 64),
        (8, 256),
        (16, 8),
        (16, 64),
        (16, 256),
        (16, 1024),
        (32, 64),
        (32, 256),
        (32, 1024),
        (32, 4096),
    ] {
        let base: Vec<[f32; 2]> = (0..n).map(|i| [i as f32, 0.0]).collect();
        let centro = (n as f32 - 1.0) / 2.0;
        let aperta = |p: &[[f32; 2]]| -> Vec<[f32; 2]> {
            (0..n)
                .map(|i| {
                    let alvo = centro + (i as f32 - centro) * 0.5;
                    [p[i][0] + (alvo - p[i][0]) * 0.2, p[i][1]]
                })
                .collect()
        };
        let mut quente = base.clone();
        let mut frio = base.clone();
        let (mut pior_q, mut pior_f) = (0, 0);
        for _ in 0..QUADROS {
            let apertado = aperta(&quente);
            quente = separa_o_que_se_desenha(&cena(&apertado), varreduras)
                .map_or(apertado, |s| posicoes(&s));
            pior_q = pior_q.max(pares_atravessados(&quente));
            frio = aperta(&frio);
            let desenhado = separa_o_que_se_desenha(&cena(&frio), varreduras)
                .map_or_else(|| frio.clone(), |s| posicoes(&s));
            pior_f = pior_f.max(pares_atravessados(&desenhado));
        }
        println!(
            "n={n:<3} varreduras={varreduras:<5} pares que ficam: QUENTE={pior_q:<4} FRIO={pior_f}"
        );
    }
}

/// ⚠️⚠️ **O EXPOENTE É DO SOLVER OU DA LEI?** — modelo mínimo de UMA cadeia (discos num eixo, só
/// vizinhos), onde as duas leis se escrevem em dez linhas cada e o expoente se lê directo.
///
/// - **Jacobi MEDIADO** é o que o produto faz: lê a FOTO e cada peça aplica a MÉDIA do que os
///   contactos dela pedem.
/// - **Vermelho-preto** é Gauss-Seidel determinístico: os contactos pares primeiro, os ímpares
///   depois, cada metade a ler o que a anterior escreveu. ⭐ Sem média — dentro de uma cor nenhum
///   par partilha peça, logo não há o que mediar, e a ordem é a das cores (não a dos índices).
#[test]
#[ignore = "sonda de medição — corre à mão"]
fn sonda_o_expoente_e_do_solver() {
    const D: f32 = 1.0; // o diâmetro que cada par quer
    let converge = |n: usize, vermelho_preto: bool| -> Option<usize> {
        let mut x: Vec<f32> = (0..n).map(|i| i as f32 * 0.25).collect();
        for k in 1..=200_000usize {
            if vermelho_preto {
                for cor in [0usize, 1] {
                    for c in (cor..n.saturating_sub(1)).step_by(2) {
                        let pen = D - (x[c + 1] - x[c]);
                        if pen > 0.0 {
                            x[c] -= pen * 0.5;
                            x[c + 1] += pen * 0.5;
                        }
                    }
                }
            } else {
                let foto = x.clone();
                let mut d = vec![0.0f32; n];
                let mut cont = vec![0u32; n];
                for c in 0..n.saturating_sub(1) {
                    let pen = D - (foto[c + 1] - foto[c]);
                    if pen > 0.0 {
                        d[c] -= pen * 0.5;
                        d[c + 1] += pen * 0.5;
                        cont[c] += 1;
                        cont[c + 1] += 1;
                    }
                }
                for i in 0..n {
                    if cont[i] > 0 {
                        x[i] = foto[i] + d[i] / cont[i] as f32;
                    }
                }
            }
            if (0..n - 1).all(|c| x[c + 1] - x[c] >= D - 0.02) {
                return Some(k);
            }
        }
        None
    };
    for n in [8usize, 16, 32, 64, 128] {
        println!(
            "n={n:<5} jacobi-mediado={:<8} vermelho-preto={}",
            converge(n, false).map_or("—".into(), |k| k.to_string()),
            converge(n, true).map_or("—".into(), |k| k.to_string()),
        );
    }
}

/// ⚠️ **O RELÓGIO dos casos que de facto FECHAM** — é daqui que sai o tecto, e não de um palpite.
#[test]
#[ignore = "sonda de medição — corre à mão, em release e calmo"]
fn sonda_o_relogio_de_uma_cadeia_que_fecha() {
    println!(
        "loadavg: {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    for (n, v) in [(8usize, 256usize), (16, 1024), (32, 4096), (64, 16384)] {
        let p: Vec<[f32; 2]> = (0..n).map(|i| [i as f32 * 0.25, 0.0]).collect();
        let s = cena(&p);
        let mut melhor = f64::INFINITY;
        for _ in 0..3 {
            let t = std::time::Instant::now();
            let _ = separa_o_que_se_desenha(&s, v);
            melhor = melhor.min(t.elapsed().as_secs_f64());
        }
        println!(
            "n={n:<4} varreduras={v:<6} -> {:>9.2} ms   ({:>5.1} % de um quadro de 16,7 ms)   \
             trabalho = {} peca-varreduras",
            melhor * 1e3,
            melhor * 1e3 / 16.7 * 100.0,
            n * v
        );
    }
}

/// ⭐⭐⭐ **UMA CADEIA DE 16 PEÇAS FECHA — e o tecto herdado NÃO a fechava** (doc 115 §18, ordem do
/// dono: *«quero todas as possibilidades possíveis, não quero limitações no sistema»*).
///
/// ⚠️ **As duas metades são o gate.** Sem a de cima ele não afirma a capacidade nova; sem a de
/// BAIXO ele não afirma que ela era precisa — *e um gate que passa com o número antigo não mediu
/// a mudança*. O `64` era o tecto herdado do clamp do `motion.collide`, e ele fecha **quatro**
/// peças.
#[test]
fn uma_cadeia_de_dezasseis_fecha_no_tecto_novo_e_nao_no_herdado() {
    const N: usize = 16;
    let p: Vec<[f32; 2]> = (0..N).map(|i| [i as f32 * 0.25, 0.0]).collect();
    let antes = pares_atravessados(&p);
    assert!(
        antes >= N,
        "a fixtura tem de CONTER o fenomeno — ela traz {antes} pares atravessados"
    );
    let fica = |v: usize| {
        separa_o_que_se_desenha(&cena(&p), v).map_or(antes, |s| pares_atravessados(&posicoes(&s)))
    };
    assert_eq!(
        fica(1024),
        0,
        "a 1024 varreduras (o tecto do SLIDER) a cadeia tem de fechar"
    );
    let herdado = fica(64);
    assert!(
        herdado > 0,
        "CONTROLO: a 64 (o tecto HERDADO) ela nao fechava — se fecha, a fixtura deixou de conter \
         o fenomeno e a metade de cima nao afirma nada"
    );
}
