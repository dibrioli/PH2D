//! Gates da suavização do traço.

use super::*;

/// Um traço de mão: um círculo perfeito mais tremor determinístico.
///
/// ⚠️ **O tremor é de ALTA frequência de propósito** — é isso que uma mão faz
/// num arrasto rápido, e é a única coisa que esta lei tem de tirar. Um ruído
/// lento seria indistinguível de uma forma que o artista quis.
fn tracado(n: usize, raio: f32, tremor: f32) -> (Vec<[f32; 2]>, Vec<[f32; 2]>) {
    let mut verdade = Vec::with_capacity(n);
    let mut mao = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f32 / n as f32 * std::f32::consts::TAU;
        let p = [raio * t.cos(), raio * t.sin()];
        verdade.push(p);
        // ⚠️ **PERIÓDICO no anel, e isso não é detalhe:** a 1.ª redacção usava
        // `sin(i · 2,3999)`, que não fecha em `i = n` — a fixtura trazia uma
        // DESCONTINUIDADE real no ponto onde o laço fecha, e o gate do anel
        // fechado lia o resíduo dela (`0,1148 px`) como prova de que a lei
        // tratava o anel como aberto. *Uma régua que acusa a lei sobre um
        // defeito da própria fixtura não afirma nada.* As harmónicas são
        // INTEIRAS e coprimas com `n`, logo altas e sem costura.
        // ⚠️ **BANDA LARGA, e isso é a segunda correcção da fixtura:** com duas
        // harmónicas altas o tremor é aniquilado em `8` pares e a curva do
        // tecto fica plana — *uma entrada fácil demais faz a lei parecer melhor
        // do que ela é*. Uma mão treme em várias escalas ao mesmo tempo, então
        // aqui somam-se cinco harmónicas inteiras de `7` a `53`, com peso a cair
        // como `1/k` (o espectro de um ruído de mão, não branco).
        let r = tremor
            * [7.0f32, 13.0, 23.0, 37.0, 53.0]
                .into_iter()
                .map(|k| (k * t + k).sin() / (k / 7.0))
                .sum::<f32>();
        mao.push([p[0] + r * t.cos(), p[1] + r * t.sin()]);
    }
    (verdade, mao)
}

fn area(anel: &[[f32; 2]]) -> f32 {
    let n = anel.len();
    (0..n)
        .map(|i| {
            let (a, b) = (anel[i], anel[(i + 1) % n]);
            a[0] * b[1] - b[0] * a[1]
        })
        .sum::<f32>()
        * 0.5
}

/// Quanto do tremor sobra, e quanto a FORMA se moveu — as duas grandezas que a
/// escolha do tecto pesa uma contra a outra.
fn mede(verdade: &[[f32; 2]], saida: &[[f32; 2]]) -> (f32, f32) {
    let n = verdade.len();
    // ⭐⭐ **O TREMOR é a RUGOSIDADE LOCAL — quanto cada ponto se afasta da
    // média dos vizinhos —, e não o desvio ao círculo verdadeiro.**
    //
    // ⛔ A 1.ª régua media o desvio radial, e sobre uma entrada de banda larga
    // ela lia `97,8 %` de *«tremor que sobra»* sobre uma lei a funcionar: a
    // maior parte daquele desvio é uma harmónica BAIXA, que é **forma** e que a
    // lei tem de preservar. *Uma régua que chama tremor a uma feição acusa o
    // alisador de não destruir o desenho.* O que o olho lê como tremor é local,
    // e mede-se localmente.
    let mut soma = 0.0f32;
    for i in 0..n {
        let (a, c) = (saida[(i + n - 1) % n], saida[(i + 1) % n]);
        let m = [(a[0] + c[0]) * 0.5, (a[1] + c[1]) * 0.5];
        soma += (saida[i][0] - m[0]).powi(2) + (saida[i][1] - m[1]).powi(2);
    }
    let rms = (soma / n as f32).sqrt();
    // A forma é a ÁREA que o anel encerra.
    let d_area = (area(saida).abs() / area(verdade).abs() - 1.0).abs() * 100.0;
    (rms, d_area)
}

/// Um traço com QUATRO CANTOS que o artista quis — um quadrado, com tremor.
///
/// ⚠️ **É a metade que o círculo não pode medir:** uma lei que tira tremor tira
/// também feição, e um círculo não tem feição nenhuma para perder. *Um corpus
/// sem o que a lei pode destruir mede só metade dela.*
fn quadrado(lado: f32, por_lado: usize, tremor: f32) -> (Vec<[f32; 2]>, Vec<[f32; 2]>) {
    let cantos = [[-lado, -lado], [lado, -lado], [lado, lado], [-lado, lado]];
    let (mut verdade, mut mao) = (Vec::new(), Vec::new());
    for c in 0..4 {
        let (a, b) = (cantos[c], cantos[(c + 1) % 4]);
        for i in 0..por_lado {
            let t = i as f32 / por_lado as f32;
            let p = [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t];
            verdade.push(p);
            let k = verdade.len() as f32;
            let r = tremor * ((k * 2.399_9).sin() + 0.5 * (k * 5.11).cos());
            mao.push([p[0] + r, p[1] + r * 0.5]);
        }
    }
    (verdade, mao)
}

/// Quanto o CANTO se arredondou: a distância do ponto do canto ao sítio onde
/// ele estava.
fn canto_perdido(verdade: &[[f32; 2]], saida: &[[f32; 2]], por_lado: usize) -> f32 {
    (0..4)
        .map(|c| {
            let i = c * por_lado;
            (saida[i][0] - verdade[i][0]).hypot(saida[i][1] - verdade[i][1])
        })
        .fold(0.0f32, f32::max)
}

#[test]
#[ignore = "sonda: de onde sai o PARES_MAX"]
fn diag_o_tecto_das_passagens() {
    let (verdade, mao) = tracado(128, 100.0, 3.0);
    let (rms0, _) = mede(&verdade, &mao);
    println!("\ntremor de entrada: rms {rms0:.4} px");
    for pares in [1usize, 8, 32, 64, 128, 192] {
        // A sonda varre o tecto correndo a lei `pares` vezes com grau 1.
        let mut a = mao.clone();
        let mut b = a.clone();
        for _ in 0..pares {
            super::passo(&a, &mut b, LAMBDA);
            super::passo(&b, &mut a, MU);
        }
        let (rms, d_area) = mede(&verdade, &a);
        // O CONTROLO: o laplaciano puro, o mesmo número de passagens.
        let mut l = mao.clone();
        let mut m = l.clone();
        for _ in 0..pares {
            super::passo(&l, &mut m, LAMBDA);
            super::passo(&m, &mut l, LAMBDA);
        }
        let (_, d_lap) = mede(&verdade, &l);
        // E o que a lei DESTRÓI: um canto que o artista desenhou.
        // ⚠️ **Tremor ZERO aqui, de propósito:** com tremor a régua mede duas
        // coisas somadas (o tremor que saiu do canto e o canto que se
        // arredondou) e não sabe separá-las. *O que a lei DESTRÓI mede-se numa
        // entrada que não tem nada a corrigir.*
        let (vq, mq) = quadrado(100.0, 50, 0.0);
        let mut qa = mq.clone();
        let mut qb = qa.clone();
        for _ in 0..pares {
            super::passo(&qa, &mut qb, LAMBDA);
            super::passo(&qb, &mut qa, MU);
        }
        println!(
            "pares={pares:3}  tremor que sobra {:5.1}%  forma moveu {d_area:.2}%  \
             CANTO perdido {:5.2} px  | CONTROLO laplaciano puro: forma moveu {d_lap:.2}%",
            rms / rms0 * 100.0,
            canto_perdido(&vq, &qa, 50),
        );
    }
}

/// ⭐ **GRAU ZERO É O ANEL, AO BIT — e EMPRESTADO.**
///
/// ⚠️ A segunda metade não é decoração: `Cow::Borrowed` é a prova de que o
/// caminho de omissão não copia nem toca nada. *Um no-op que aloca ainda é um
/// caminho novo.*
#[test]
fn grau_zero_e_o_anel_ao_bit() {
    let (_, mao) = tracado(64, 100.0, 3.0);
    for grau in [0.0f32, -1.0, f32::NAN] {
        let out = suaviza(&mao, grau);
        assert!(
            matches!(out, std::borrow::Cow::Borrowed(_)),
            "grau {grau} copiou o anel"
        );
        assert_eq!(
            out.iter()
                .map(|p| [p[0].to_bits(), p[1].to_bits()])
                .collect::<Vec<_>>(),
            mao.iter()
                .map(|p| [p[0].to_bits(), p[1].to_bits()])
                .collect::<Vec<_>>(),
            "grau {grau} moveu um bit"
        );
    }
    // Menos de três pontos não é um anel.
    let dois = [[0.0f32, 0.0], [1.0, 1.0]];
    assert!(matches!(suaviza(&dois, 1.0), std::borrow::Cow::Borrowed(_)));
}

/// ⛔⛔ **A LEI INTEIRA, e o CONTROLO é metade dela: o tremor sai E a forma
/// fica.**
///
/// ⚠️ Sem a segunda metade, um laplaciano puro passaria este gate a tirar
/// tremor — e encolheria o contorno `46 ×` mais, fazendo a ferramenta cortar
/// **por dentro** da linha que o artista desenhou.
#[test]
fn o_tremor_sai_e_a_forma_fica() {
    let (verdade, mao) = tracado(128, 100.0, 3.0);
    let (rms0, _) = mede(&verdade, &mao);
    let out = suaviza(&mao, 1.0);
    let (rms, d_area) = mede(&verdade, &out);
    // ⚠️ **A barra é a MEDIÇÃO** (`24,1 %` no tecto, tabela no doc do
    // `PARES_MAX`), com margem para a deriva de `f32`. ⛔ Ela não é apertada até
    // ao valor de hoje: um gate colado ao número reprova na primeira wave que
    // mexa na fixtura, e o que ele afirma é *«o tremor sai»*, não *«sai
    // exactamente isto»*.
    assert!(
        rms < rms0 * 0.30,
        "o tremor mal saiu: {rms:.4} contra {rms0:.4} de entrada"
    );
    assert!(
        d_area < 0.5,
        "a forma encolheu {d_area:.2}% — é o defeito que o par de passagens existe para não ter"
    );
    // O CONTROLO: o laplaciano puro, o mesmo trabalho.
    let mut l = mao.clone();
    let mut m = l.clone();
    for _ in 0..PARES_MAX {
        super::passo(&l, &mut m, LAMBDA);
        super::passo(&m, &mut l, LAMBDA);
    }
    let (_, d_lap) = mede(&verdade, &l);
    assert!(
        d_lap > 5.0 * d_area.max(0.01),
        "CONTROLO: o laplaciano puro tinha de encolher MUITO mais ({d_lap:.2}% \
         contra {d_area:.2}%) — se ele já não encolhe, esta lei deixou de ser \
         necessária e o par de passagens pode sair"
    );
}

/// ⛔ **O ANEL É FECHADO** — o vizinho do último é o primeiro.
///
/// Tratá-lo como aberto prende as duas pontas e deixa uma bossa **exactamente
/// onde o artista fechou o laço**, que é o sítio onde a mão mais treme.
#[test]
fn o_anel_e_fechado() {
    let (verdade, mao) = tracado(128, 100.0, 3.0);
    let out = suaviza(&mao, 1.0);
    let n = out.len();
    // O desvio radial junto ao fecho tem de ser da mesma ordem do resto.
    let raio = 100.0f32;
    let desvio = |i: usize| (out[i][0].hypot(out[i][1]) - raio).abs();
    let no_fecho = desvio(0).max(desvio(n - 1));
    let mediana_do_resto = {
        let mut v: Vec<f32> = (2..n - 2).map(desvio).collect();
        v.sort_by(f32::total_cmp);
        v[v.len() / 2]
    };
    assert!(
        no_fecho < mediana_do_resto * 4.0 + 0.05,
        "o fecho do laço ficou com uma bossa: {no_fecho:.4} contra {mediana_do_resto:.4} \
         do resto — o anel está a ser tratado como ABERTO"
    );
    let _ = verdade;
}

/// ⛔⛔ **A SAÍDA NÃO DEPENDE DE ONDE O ANEL COMEÇA** — a prova do buffer duplo.
///
/// ⚠️ Um Gauss-Seidel leria o valor NOVO de um vizinho e o VELHO do outro, e a
/// saída passaria a depender do ponto em que o gesto começou — que é uma
/// propriedade da mão, não da forma. *Duas pessoas a desenhar o mesmo laço a
/// partir de cantos diferentes têm de obter o mesmo corte.*
#[test]
fn a_saida_nao_depende_de_onde_o_anel_comeca() {
    let (_, mao) = tracado(64, 100.0, 3.0);
    let direito = suaviza(&mao, 1.0).into_owned();
    const R: usize = 17;
    let rodado: Vec<[f32; 2]> = mao[R..].iter().chain(&mao[..R]).copied().collect();
    let suave_rodado = suaviza(&rodado, 1.0).into_owned();
    for i in 0..mao.len() {
        let a = direito[(i + R) % mao.len()];
        let b = suave_rodado[i];
        assert!(
            (a[0] - b[0]).abs() < 1e-4 && (a[1] - b[1]).abs() < 1e-4,
            "rodar o anel mudou a saída no ponto {i}: {a:?} contra {b:?}"
        );
    }
}

/// **Mais grau, menos tremor** — a pista tem de ser monótona, senão ela mente.
#[test]
fn mais_grau_tira_mais_tremor() {
    let (verdade, mao) = tracado(128, 100.0, 3.0);
    let mut anterior = f32::INFINITY;
    for passo in 0..=4 {
        let g = passo as f32 / 4.0;
        let (rms, _) = mede(&verdade, &suaviza(&mao, g));
        assert!(
            rms <= anterior + 1e-4,
            "grau {g}: o tremor SUBIU ({rms:.4} contra {anterior:.4}) — a pista \
             não é monótona e o artista não consegue prever o que ela faz"
        );
        anterior = rms;
    }
}

#[test]
#[ignore = "sonda: o MU e' observavel?"]
fn diag_o_mu() {
    let (verdade, mao) = tracado(128, 100.0, 3.0);
    let (rms0, _) = mede(&verdade, &mao);
    let (vq, mq) = quadrado(100.0, 50, 0.0);
    for mu in [-0.50f32, -0.505, -0.52, -0.55, -0.60] {
        let mut a = mao.clone();
        let mut b = a.clone();
        for _ in 0..PARES_MAX {
            super::passo(&a, &mut b, LAMBDA);
            super::passo(&b, &mut a, mu);
        }
        let (rms, d_area) = mede(&verdade, &a);
        let mut qa = mq.clone();
        let mut qb = qa.clone();
        for _ in 0..PARES_MAX {
            super::passo(&qa, &mut qb, LAMBDA);
            super::passo(&qb, &mut qa, mu);
        }
        println!(
            "mu={mu:6.3}  tremor que sobra {:5.1}%  area moveu {d_area:6.3}%  canto perdido {:5.2} px",
            rms / rms0 * 100.0,
            canto_perdido(&vq, &qa, 50),
        );
    }
}

#[test]
#[ignore = "sonda: o MU amplifica harmonicas baixas?"]
fn diag_a_amplificacao() {
    // Um anel com UMA harmónica baixa: uma forma que o artista QUIS.
    let n = 128usize;
    let (raio, amp, ordem) = (100.0f32, 6.0f32, 3.0f32);
    let entrada: Vec<[f32; 2]> = (0..n)
        .map(|i| {
            let t = i as f32 / n as f32 * std::f32::consts::TAU;
            let r = raio + amp * (ordem * t).cos();
            [r * t.cos(), r * t.sin()]
        })
        .collect();
    let medir = |v: &[[f32; 2]]| {
        // A amplitude da harmónica `ordem` no raio.
        let (mut c, mut sq) = (0.0f32, 0.0f32);
        for (i, p) in v.iter().enumerate() {
            let t = i as f32 / n as f32 * std::f32::consts::TAU;
            let r = p[0].hypot(p[1]);
            c += r * (ordem * t).cos();
            sq += r * (ordem * t).sin();
        }
        2.0 * (c * c + sq * sq).sqrt() / n as f32
    };
    println!(
        "\nentrada: amplitude da ordem {ordem} = {:.4}",
        medir(&entrada)
    );
    for mu in [-0.50f32, -0.505, -0.52, -0.55] {
        let mut a = entrada.clone();
        let mut b = a.clone();
        for _ in 0..PARES_MAX {
            super::passo(&a, &mut b, LAMBDA);
            super::passo(&b, &mut a, mu);
        }
        let saida = medir(&a);
        println!(
            "mu={mu:6.3}  amplitude {saida:.4}  ({:+.1}%)",
            (saida / medir(&entrada) - 1.0) * 100.0
        );
    }
}

/// ⛔⛔⛔ **A LEI NUNCA AMPLIFICA UMA FORMA QUE O ARTISTA DESENHOU.**
///
/// # Ela nasceu de uma MUTAÇÃO SOBREVIVENTE, e a mutação tinha razão
///
/// O [`MU`] era `−0,52` com um doc a dizer que `μ = −λ` fazia *«as duas
/// passagens cancelarem-se»*. Trocar a constante passava a suíte inteira — e ao
/// medir, a afirmação era **falsa**: a composição de um par tem resposta
/// `H(k) = (1 − λk)(1 − μk)` com `k = 1 − cos θ`, e com `μ` **mais** negativo
/// que `−λ` aparece um termo linear positivo ⇒ **`H > 1` nas frequências
/// baixas**. Medido com uma harmónica de ordem `3`: `+0,6 %` a `μ = −0,52` e
/// `+1,8 %` a `−0,55`, contra `−0,2 %` a `−0,50`.
///
/// ⚠️ **Amplificar é pior do que alisar de menos:** o artista desenhou uma
/// ondulação e a ferramenta devolve-a MAIOR — isto é, ela inventa forma. *Uma
/// lei de suavização pode errar por preservar de menos; nunca por criar.*
#[test]
fn a_lei_nunca_amplifica_uma_forma_que_o_artista_desenhou() {
    let n = 128usize;
    let (raio, amp) = (100.0f32, 6.0f32);
    for ordem in [2.0f32, 3.0, 5.0, 8.0] {
        let entrada: Vec<[f32; 2]> = (0..n)
            .map(|i| {
                let t = i as f32 / n as f32 * std::f32::consts::TAU;
                let r = raio + amp * (ordem * t).cos();
                [r * t.cos(), r * t.sin()]
            })
            .collect();
        let amplitude = |v: &[[f32; 2]]| {
            let (mut c, mut s) = (0.0f32, 0.0f32);
            for (i, p) in v.iter().enumerate() {
                let t = i as f32 / n as f32 * std::f32::consts::TAU;
                let r = p[0].hypot(p[1]);
                c += r * (ordem * t).cos();
                s += r * (ordem * t).sin();
            }
            2.0 * c.hypot(s) / n as f32
        };
        let antes = amplitude(&entrada);
        let depois = amplitude(&suaviza(&entrada, 1.0));
        assert!(
            depois <= antes * 1.001,
            "ordem {ordem}: a lei AMPLIFICOU a forma ({antes:.4} → {depois:.4}, \
             {:+.2}%) — com `μ` mais negativo que `−λ` a resposta passa de 1 nas \
             frequências baixas, e o alisador inventa forma",
            (depois / antes - 1.0) * 100.0
        );
        // ⭐ E a metade que impede a cura barata (uma lei que apague tudo):
        // uma feição desta escala tem de SOBREVIVER.
        assert!(
            depois > antes * 0.90,
            "ordem {ordem}: a lei comeu a forma ({antes:.4} → {depois:.4})"
        );
    }
}
