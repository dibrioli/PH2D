//! ⭐⭐⭐ **O SOLVER** — e cada gate aqui mata uma maneira diferente de estar errado.

use crate::{
    ClothDrive, ClothMaterial, ClothRest, ClothState, ClothTopology, StepConfig, V3, energy,
    fixtures, step,
};

const N: usize = 8;

fn mat() -> ClothMaterial {
    ClothMaterial {
        density: 1.0,
        young: 400.0,
        poisson: 0.3,
        bending: 2.0e-3,
        damping: 0.05,
    }
}

fn cena() -> (Vec<V3>, ClothTopology, ClothRest, Vec<bool>) {
    let (x, t) = fixtures::dome(N);
    let topo = ClothTopology::build(&t, x.len());
    let rest = ClothRest::measure(&topo, &x, &mat());
    (x, topo, rest, fixtures::border(N))
}

fn max_desloc(a: &[V3], b: &[V3]) -> f64 {
    a.iter()
        .zip(b)
        .map(|(p, q)| (0..3).map(|c| (p[c] - q[c]).abs()).fold(0.0f64, f64::max))
        .fold(0.0f64, f64::max)
}

/// ⭐⭐⭐ **GATE — o repouso é PONTO FIXO, ao bit.**
///
/// ⛔⛔ É o controle positivo da suíte inteira: sem força externa nenhuma, o pano
/// parado na pose em que o traço começou **não se mexe**. Um solver que "relaxa" o
/// que ninguém tocou faria a peça mudar só por o artista encostar o pincel — e a
/// barra é `0.0`, não um epsilon, porque no repouso todo termo é exatamente zero.
#[test]
fn o_repouso_e_ponto_fixo() {
    let (x, topo, rest, _) = cena();
    let mut st = ClothState::at_rest(&x);
    let cfg = StepConfig {
        substeps: 4,
        iterations: 4,
        ..StepConfig::default()
    };
    for _ in 0..20 {
        step(
            &topo,
            &rest,
            &mat(),
            &[],
            &ClothDrive::default(),
            &cfg,
            &mut st,
        );
    }
    assert_eq!(max_desloc(&st.x, &x), 0.0, "o repouso andou");
    assert!(st.v.iter().all(|v| *v == [0.0; 3]), "nasceu velocidade");
}

/// ⭐⭐⭐ **GATE — MAIS iterações nunca pioram.**
///
/// ⚠️ **É a propriedade central do VBD, na forma em que ela é observável de
/// fora:** cada energia local é garantidamente reduzida, e a soma das reduções
/// locais é a redução global ⇒ gastar mais orçamento no mesmo sub-passo não pode
/// deixar o pano mais deformado. *Um solver que oscilasse com o número de
/// iterações não poderia prometer «trunque à vontade».*
#[test]
fn mais_iteracoes_nunca_pioram() {
    let (x, topo, rest, pin) = cena();
    let mut antes = f64::INFINITY;
    let mut caiu = false;
    for it in [1u32, 2, 4, 8, 16] {
        let mut st = ClothState::at_rest(&x);
        // Puxa o miolo para fora do repouso e deixa o solver responder.
        for (i, p) in st.x.iter_mut().enumerate() {
            if !pin[i] {
                p[2] += 0.25;
            }
        }
        let cfg = StepConfig {
            substeps: 1,
            iterations: it,
            ..StepConfig::default()
        };
        step(
            &topo,
            &rest,
            &mat(),
            &pin,
            &ClothDrive::default(),
            &cfg,
            &mut st,
        );
        let e = energy(&topo, &rest, &mat(), &st.x);
        assert!(
            e <= antes * (1.0 + 1e-9),
            "{it} iteracoes pioraram: {e:.6e} contra {antes:.6e}"
        );
        caiu |= e < antes * 0.999;
        antes = e;
    }
    assert!(
        caiu,
        "a energia nunca caiu -- o gate estaria verde por inercia"
    );
}

/// ⭐⭐⭐ **GATE — UMA iteração só é estável, e é esse o regime do pincel.**
///
/// ⛔ É a promessa que fez este método ser escolhido no lugar do XPBD: quem trunca
/// o orçamento é o relógio do quadro, e o solver tem de continuar correto quando
/// isso acontece.
#[test]
fn uma_iteracao_so_e_estavel() {
    let (x, topo, rest, pin) = cena();
    let mut st = ClothState::at_rest(&x);
    let cfg = StepConfig {
        substeps: 1,
        iterations: 1,
        gravity: [0.0, 0.0, -9.8],
        ..StepConfig::default()
    };
    for _ in 0..500 {
        step(
            &topo,
            &rest,
            &mat(),
            &pin,
            &ClothDrive::default(),
            &cfg,
            &mut st,
        );
    }
    assert!(
        st.x.iter().flatten().all(|c| c.is_finite()),
        "divergiu para NaN/inf"
    );
    assert!(
        max_desloc(&st.x, &x) < 2.0,
        "o pano fugiu: {:.3}",
        max_desloc(&st.x, &x)
    );
}

/// ⭐⭐⭐ **GATE — o arrasto na velocidade máxima não explode.**
///
/// ⚠️ **É o risco NOMEADO da escolha de método:** o VBD **não projeta** a Hessiana
/// indefinida, e quem validou essa decisão foi a bancada dos autores, não a nossa.
/// Aqui a fixtura é o pior caso do produto — o artista arrastando com força
/// absurda, que é o gesto que um pincel de facto recebe.
#[test]
fn o_arrasto_na_velocidade_maxima_nao_explode() {
    let (x, topo, rest, pin) = cena();
    let mut st = ClothState::at_rest(&x);
    // ⚠️ **A mão conduz por META e PESO** — ver [`ClothDrive`]. A 1.ª redação
    // destes gates empurrava por aceleração externa, e a forma mudou quando a
    // medição mostrou que uma força por MASSA desaparece ao refinar a malha.
    let mut alvo = vec![[0.0f64; 3]; x.len()];
    let mut peso = vec![0.0f64; x.len()];
    let cfg = StepConfig {
        substeps: 2,
        iterations: 2,
        ..StepConfig::default()
    };
    for k in 0..300 {
        // Uma força enorme, a mudar de direção como um traço a serpentear.
        let a = f64::from(k) * 0.37;
        for (i, g) in alvo.iter_mut().enumerate() {
            peso[i] = if pin[i] { 0.0 } else { 1.0 };
            let p = st.x[i];
            *g = [
                p[0] + 0.9 * a.cos(),
                p[1] + 0.9 * a.sin(),
                p[2] + 0.6 * (a * 0.5).sin(),
            ];
        }
        step(
            &topo,
            &rest,
            &mat(),
            &pin,
            &ClothDrive {
                goal: &alvo,
                weight: &peso,
                stiffness: 1.0e4,
            },
            &cfg,
            &mut st,
        );
        assert!(
            st.x.iter().flatten().all(|c| c.is_finite()),
            "explodiu no passo {k}"
        );
    }
    assert!(
        max_desloc(&st.x, &x) < 50.0,
        "a malha foi para o infinito: {:.3}",
        max_desloc(&st.x, &x)
    );
}

/// ⭐⭐⭐ **GATE — pregado é pregado, e não «quase».**
///
/// ⛔ O anel de falloff é a feature (é ele que faz a transição para o resto da
/// peça não estourar). Um pregado que escorrega um ULP por sub-passo escorrega
/// visivelmente num traço de mil eventos. ⚠️ E aqui pregar não é uma mola forte:
/// é o vértice **não ser atualizado** — massa infinita de verdade, sem termo de
/// penalidade e sem constante para afinar.
#[test]
fn pregado_e_pregado() {
    let (x, topo, rest, pin) = cena();
    let mut st = ClothState::at_rest(&x);
    let alvo: Vec<V3> = x
        .iter()
        .map(|p| [p[0] + 0.4, p[1] - 0.3, p[2] + 0.5])
        .collect();
    let peso: Vec<f64> = (0..x.len()).map(|_| 1.0).collect();
    let cfg = StepConfig {
        substeps: 4,
        iterations: 3,
        gravity: [0.0, 0.0, -20.0],
        ..StepConfig::default()
    };
    for _ in 0..100 {
        step(
            &topo,
            &rest,
            &mat(),
            &pin,
            &ClothDrive {
                goal: &alvo,
                weight: &peso,
                stiffness: 1.0e4,
            },
            &cfg,
            &mut st,
        );
    }
    for (i, p) in pin.iter().enumerate() {
        if *p {
            assert_eq!(st.x[i], x[i], "o pregado {i} andou");
        }
    }
    // Controle: o miolo TEM de ter andado, senão o gate estaria verde por nada
    // se mexer.
    let miolo = (0..x.len()).filter(|i| !pin[*i]).fold(0.0f64, |m, i| {
        m.max(
            (0..3)
                .map(|c| (st.x[i][c] - x[i][c]).abs())
                .fold(0.0, f64::max),
        )
    });
    assert!(miolo > 1e-3, "o miolo nao se mexeu: {miolo:.3e}");
}

/// ⭐⭐⭐ **GATE — a razão de massa INFINITA é servida.**
///
/// ⚠️⚠️ **É o gate que justifica a troca de método.** O XPBD é documentado como
/// sofrendo *particularmente* sob razões de massa altas — e um pincel a fabrica
/// toda vez que prega o anel: massa infinita na borda contra massa finita no
/// miolo, com o pano rígido a ligar as duas. Aqui a fixtura é o pior caso
/// (material duro, pouca iteração) e o que se cobra é **convergência**, não
/// sobrevivência.
#[test]
fn a_razao_de_massa_infinita_e_servida() {
    let (x, topo, _, pin) = cena();
    let duro = ClothMaterial {
        young: 2.0e4,
        ..mat()
    };
    let rest = ClothRest::measure(&topo, &x, &duro);
    let mut st = ClothState::at_rest(&x);
    for (i, p) in st.x.iter_mut().enumerate() {
        if !pin[i] {
            p[2] += 0.2;
        }
    }
    let e0 = energy(&topo, &rest, &duro, &st.x);
    let cfg = StepConfig {
        substeps: 2,
        iterations: 2,
        ..StepConfig::default()
    };
    for _ in 0..200 {
        step(
            &topo,
            &rest,
            &duro,
            &pin,
            &ClothDrive::default(),
            &cfg,
            &mut st,
        );
    }
    let e1 = energy(&topo, &rest, &duro, &st.x);
    assert!(
        st.x.iter().flatten().all(|c| c.is_finite()),
        "divergiu com material duro contra borda pregada"
    );
    assert!(
        e1 < e0 * 1e-2,
        "nao relaxou contra a borda pregada: {e0:.4e} -> {e1:.4e}"
    );
}

/// ⭐⭐⭐ **GATE — a mesma entrada dá os mesmos BITS.**
///
/// ⚠️ A ordem de Gauss-Seidel é a coloração, e a coloração é derivada da malha —
/// esta é a metade observável daquela decisão. A casa tem hash de replay a cobrar
/// determinismo.
#[test]
fn o_passo_e_deterministico() {
    let (x, topo, rest, pin) = cena();
    let alvo: Vec<V3> = x
        .iter()
        .map(|p| [p[0] + 0.2, p[1] + 0.1, p[2] - 0.3])
        .collect();
    let peso: Vec<f64> = (0..x.len())
        .map(|i| 0.5 + 0.5 * ((i % 3) as f64 / 2.0))
        .collect();
    let cfg = StepConfig {
        substeps: 3,
        iterations: 2,
        gravity: [0.0, 0.0, -9.8],
        ..StepConfig::default()
    };
    let corrida = || {
        let mut st = ClothState::at_rest(&x);
        for _ in 0..30 {
            step(
                &topo,
                &rest,
                &mat(),
                &pin,
                &ClothDrive {
                    goal: &alvo,
                    weight: &peso,
                    stiffness: 1.0e4,
                },
                &cfg,
                &mut st,
            );
        }
        st.x
    };
    assert_eq!(corrida(), corrida());
}

/// ⭐⭐⭐ **GATE — o custo segue a PEGADA, não a malha.**
///
/// ⚠️ **Um pincel simula a região que ele toca**, e é isso que o torna viável numa
/// escultura de milhões de vértices. Este gate mede a FORMA do custo, e ⛔ **não é
/// um gate de relógio** — a lei desta casa é que nenhuma leitura de tempo vale
/// sob carga.
///
/// ⚠️ **A barra é MEDIDA, e a primeira redação dela era um palpite:** eu escrevi
/// `< 16` visitas por vértice e a grade lê **`16,50`**. O número não é arbitrário
/// — um vértice interior de valência `6` toca `6` triângulos e `12` dobradiças
/// (`6` como ponta da aresta, `6` como ápice da aresta oposta), mais ele próprio:
/// **`19`**, e a média sobe com a fração de interior. ⇒ o que se cobra não é um
/// teto inventado: é o trabalho por vértice **CONVERGIR** com o tamanho da região,
/// que é a afirmação *«o custo é da pegada»* na forma em que ela é falsificável.
#[test]
fn o_custo_segue_a_pegada() {
    let medir = |n: usize| {
        let (x, t) = fixtures::dome(n);
        let topo = ClothTopology::build(&t, x.len());
        // O trabalho de um sub-passo é uma visita por vértice mais uma por
        // (vértice, elemento incidente) — que é o comprimento dos dois CSR.
        let visitas: usize = (0..x.len())
            .map(|i| 1 + topo.tri_of.of(i).len() + topo.hinge_of.of(i).len())
            .sum();
        (visitas, visitas as f64 / x.len() as f64)
    };
    let (v8, p8) = medir(8);
    let (v16, p16) = medir(16);
    let (v32, p32) = medir(32);
    assert!(v8 < v16 && v16 < v32, "a regiao nao cresceu");
    // ⚠️ A região cresce **16×** em vértices de `n = 8` para `n = 32` (medido:
    // 81 → 1089). O trabalho POR VÉRTICE tem de ficar PRESO — ele sobe só porque
    // a fração de interior sobe (14,43 → 16,50 → 17,69, a caminho de 19), e nunca
    // ultrapassa o valor da malha regular.
    for (n, p) in [(8, p8), (16, p16), (32, p32)] {
        assert!(
            (10.0..19.5).contains(&p),
            "trabalho por vertice fora do limite estrutural em n={n}: {p:.2}"
        );
    }
    assert!(
        p32 / p8 < 1.5,
        "o trabalho por vertice segue a MALHA e nao a pegada: {p8:.2} -> {p32:.2} \
         ({:.2}x) enquanto os vertices fizeram {:.0}x",
        p32 / p8,
        v32 as f64 / v8 as f64
    );
}
