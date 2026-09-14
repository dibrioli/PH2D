//! **O ESFREGÃO DE DESLOCAMENTO** — a lei dele, medida contra a espec
//! `SPEC_unblocked_brushes.md` §§1 e 5.
//!
//! ⭐ **Nenhum destes gates precisa de GPU nem de uma pilha de multiresolução:**
//! a referência entra pelo [`SculptStroke::reference`], como no irmão
//! [`super::verb_erase`]. Fotografá-la é trabalho da shell; o que **ela diz** é
//! medido aqui.
//!
//! ⭐⭐⭐ **E a fixtura principal é um DEGRAU, porque ela torna a lei uma conta
//! de quatro linhas que se fecha à mão.** Num plano com a referência plana e o
//! relevo a valer `h` só em `x < −0,05`, com a mão a andar em `+X`, a espec §5.2
//! prevê — e o motor entrega — exactamente isto:
//!
//! | `x` | `D` antes | `D′` depois | porquê |
//! |---|---|---|---|
//! | `−0,20` | `h` | **`h/(1+G)`** | a ORLA (§5.4): os vizinhos a montante estão FORA da pegada e entram com **zero** |
//! | `−0,10` | `h` | **`h`** | a montante é tudo `h` ⇒ a média devolve `h` |
//! | `0,00` | `0` | **`h·G/(1+G)`** | o relevo VIAJA para onde a mão vai |
//! | `+0,10` | `0` | **`0`** | a jusante não contribui — `g` é a parte NEGATIVA do cosseno |
//!
//! com `G = 1 + 1/√2` — o vizinho de aresta (`ê = −x̂`, `g = 1`) mais o vizinho
//! **diagonal** que esta triangulação dá (`ê = (−x̂+ŷ)/√2`, `g = 1/√2`); os
//! vizinhos em `±ŷ` têm `g = 0` por ortogonalidade, e os de `+x̂` por serem a
//! jusante.
//!
//! ⛔⛔ **A LINHA `+0,10` É A LEI INTEIRA NUMA CÉLULA:** com `g` escrito como
//! `|cos|` em vez da parte negativa, ela lê `h·G/(1+G)` e o pincel passa a
//! BORRAR em vez de TRANSPORTAR. *É a diferença entre as duas ferramentas.*

use super::*;
use crate::SmearMode;

/// O passo da grelha da fixtura do degrau.
const PASSO: f32 = 0.1;
/// A altura do degrau.
const H: f32 = 0.2;
/// O raio do pincel das fixturas — ⚠️ escolhido para que o vizinho a montante
/// de `x = −0,20` caia **FORA** da pegada, que é o que torna a linha da ORLA
/// observável. *Uma fixtura que não contém o fenómeno não o mede.*
const RAIO: f32 = 0.35;
/// `G = 1 + 1/√2` — a soma dos pesos a montante nesta triangulação.
const G: f32 = 1.0 + std::f32::consts::FRAC_1_SQRT_2;

/// **O PLANO COM DEGRAU** — `(malha viva, referência)`.
///
/// A referência é o plano liso; as posições vivas trazem o deslocamento `deslocar`
/// aplicado a quem está em `x < −0,05`.
///
/// ⚠️ **A triangulação é a anti-diagonal** (`b–c`), e ela é load-bearing: é ela
/// que põe **um** vizinho diagonal a montante em vez de dois, e é daí que sai o
/// `G` da tabela do cabeçalho.
fn degrau(deslocar: impl Fn(&mut [f32; 3])) -> (Mesh, Vec<[f32; 3]>) {
    const N: usize = 21;
    let mut pos = Vec::with_capacity(N * N);
    for j in 0..N {
        for i in 0..N {
            pos.push([(i as f32 - 10.0) * PASSO, (j as f32 - 10.0) * PASSO, 0.0f32]);
        }
    }
    let referencia = pos.clone();
    for p in &mut pos {
        if p[0] < -0.05 {
            deslocar(p);
        }
    }
    let mut faces = Vec::with_capacity((N - 1) * (N - 1) * 2);
    for j in 0..N - 1 {
        for i in 0..N - 1 {
            let a = (j * N + i) as u32;
            let (b, c) = (a + 1, a + N as u32);
            faces.push(ph2d_mesh::Face::tri(a, c, b));
            faces.push(ph2d_mesh::Face::tri(b, c, c + 1));
        }
    }
    (
        Mesh::from_parts(pos, faces).expect("o plano do degrau"),
        referencia,
    )
}

/// O índice do vértice da linha central em `x = (i − 10)·PASSO`.
const fn central(i: usize) -> usize {
    10 * 21 + i
}

/// O pincel do esfregão, com a curva **Constant** — a mesma escolha do irmão:
/// com ela o peso vale `1` em toda a pegada e a fracção percorrida fica a
/// depender **só** da força, que é a grandeza que a §1.1 pina.
fn pincel(modo: SmearMode, strength: f32) -> Brush {
    Brush {
        verb: Verb::SmearMultires,
        radius: RAIO,
        strength,
        falloff: Falloff::Constant,
        smear_mode: modo,
        ..Brush::default()
    }
}

/// Corre `n` dabs a andar em `+X` a partir da origem e devolve as posições.
///
/// ⚠️ **`n = 1` deixa o [`Dab::path`] a ZERO** — o primeiro dab de um traço não
/// tem antecessor —, que é exactamente a degenerescência que o
/// [`SmearMode::Drag`] tem por lei (§5.3). Os gates que a medem usam-no de
/// propósito; os outros usam `n = 2`, onde o segundo dab é o que trabalha.
fn esfregar(mesh: &mut Mesh, referencia: &[[f32; 3]], brush: &Brush, n: usize) -> Vec<[f32; 3]> {
    let mut s = SculptStroke::default();
    s.begin(mesh);
    s.reference = referencia.to_vec();
    for k in 0..n {
        let c = [PASSO * 0.5 * k as f32, 0.0, 0.0];
        let d = Dab::at(c, brush.radius, [0.0, 0.0, 1.0]);
        s.dab(mesh, brush, &d, Symmetry::default());
    }
    mesh.positions().to_vec()
}

/// ⭐⭐⭐ **A LEI, FECHADA À MÃO** — a tabela do cabeçalho deste ficheiro,
/// medida no motor (espec §5.2 e §5.4).
#[test]
fn a_lei_do_esfregao_e_a_media_ponderada_a_montante() {
    let (mut mesh, referencia) = degrau(|p| p[2] = H);
    let b = pincel(SmearMode::Drag, 1.0);
    let z = esfregar(&mut mesh, &referencia, &b, 2);
    let esperado = [
        (8usize, H / (1.0 + G)), // a ORLA come deslocamento
        (9, H),                  // o miolo do degrau não se mexe
        (10, H * G / (1.0 + G)), // o relevo VIAJA para +X
        (11, 0.0),               // a jusante não contribui
    ];
    for (i, alvo) in esperado {
        let lido = z[central(i)][2];
        assert!(
            (lido - alvo).abs() < 1e-5,
            "x = {:+.2}: lido {lido:.6}, a conta da espec dá {alvo:.6}",
            (i as f32 - 10.0) * PASSO
        );
    }
}

/// ⭐⭐⭐ **A FORÇA ENTRA AO QUADRADO** (espec §1.1), medida na mesma célula da
/// tabela: a metade do curso percorre **um QUARTO** do caminho, não metade.
///
/// ⛔⛔ **E o padrão do [`Verb::Thumb`] NÃO transfere**, exactamente como no
/// apagador: no [`crate::Grip::Stamp`] o `unit_accum` é `true` — o alvo já traz
/// o peso e o `accum` vale `1` —, então o quadrado tem de ser **escrito no
/// alvo**. À maneira do polegar este pincel media `0,500` a meio curso.
#[test]
fn a_forca_do_esfregao_entra_ao_quadrado() {
    let cheia = {
        let (mut mesh, r) = degrau(|p| p[2] = H);
        esfregar(&mut mesh, &r, &pincel(SmearMode::Drag, 1.0), 2)[central(10)][2]
    };
    let meia = {
        let (mut mesh, r) = degrau(|p| p[2] = H);
        esfregar(&mut mesh, &r, &pincel(SmearMode::Drag, 0.5), 2)[central(10)][2]
    };
    assert!(cheia > 1e-3, "a fixtura não moveu nada: {cheia}");
    let fraccao = meia / cheia;
    assert!(
        (fraccao - 0.25).abs() < 1e-4,
        "a meio curso ele percorreu {fraccao:.6} do caminho — `força¹` daria \
         0,500 e a espec §1.1 mede 0,250"
    );
}

/// ⭐⭐ **O ARRASTO PARADO É INERTE, e os outros dois NÃO** (espec §5.3).
///
/// ⚠️ **Não é um caso de borda: é a lei.** Com `d̂` nulo nenhum vizinho passa o
/// teste do cosseno, a média colapsa em `D[v]` e o alvo é a posição viva ⇒ zero
/// **exacto**, não «pequeno». ⛔ O [`SmearMode::Pinch`] e o
/// [`SmearMode::Expand`] não têm essa degenerescência — a direcção deles nasce
/// da GEOMETRIA e não do movimento —, e é esse lado que impede a cura barata
/// *«fazer o pincel inteiro não mover nada»* de passar.
#[test]
fn o_arrasto_parado_e_inerte_e_os_outros_dois_nao() {
    let mexeu = |modo: SmearMode| {
        let (mut mesh, r) = degrau(|p| p[2] = H);
        let antes = mesh.positions().to_vec();
        // ⚠️ **UM dab**, que é o que deixa o `Dab::path` a zero.
        let depois = esfregar(&mut mesh, &r, &pincel(modo, 1.0), 1);
        antes
            .iter()
            .zip(&depois)
            .map(|(p, q)| {
                let d = [q[0] - p[0], q[1] - p[1], q[2] - p[2]];
                (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
            })
            .fold(0.0f32, f32::max)
    };
    assert_eq!(
        mexeu(SmearMode::Drag),
        0.0,
        "com o cursor parado o arrasto tem de ser inerte AO BIT"
    );
    for modo in [SmearMode::Pinch, SmearMode::Expand] {
        let d = mexeu(modo);
        assert!(
            d > 1e-3,
            "{modo:?} parado mediu {d:.3e} — a direcção dele é da GEOMETRIA e \
             não podia ter degenerado com o cursor"
        );
    }
}

/// ⭐ **OS TRÊS MODOS DÃO SAÍDAS DIFERENTES** — o selector CHEGA à lei.
///
/// ⛔⛔ **É o ponto cego do §5.0 do roteador**, e nenhum gate de registo o vê:
/// um chip pintado, hit-indexado e vivo sob o dedo cujo valor o consumidor
/// descarta lê-se exactamente como um chip ligado. *A pergunta é se o VALOR
/// chega a um efeito*, e a única resposta é medir a malha.
#[test]
fn o_selector_de_direccao_chega_a_lei() {
    let saida = |modo: SmearMode| {
        let (mut mesh, r) = degrau(|p| p[2] = H);
        esfregar(&mut mesh, &r, &pincel(modo, 1.0), 2)
    };
    let (a, b, c) = (
        saida(SmearMode::Drag),
        saida(SmearMode::Pinch),
        saida(SmearMode::Expand),
    );
    let difere = |x: &[[f32; 3]], y: &[[f32; 3]]| {
        x.iter()
            .zip(y)
            .map(|(p, q)| (q[2] - p[2]).abs())
            .fold(0.0f32, f32::max)
    };
    for (nome, d) in [
        ("arrastar↔apertar", difere(&a, &b)),
        ("arrastar↔espalhar", difere(&a, &c)),
        ("apertar↔espalhar", difere(&b, &c)),
    ] {
        assert!(
            d > 1e-3,
            "{nome}: as duas saídas diferem {d:.3e} — o chip é um controlo morto"
        );
    }
}

/// ⛔ **INVERTER NÃO FAZ NADA** (espec §1.2) — a saída é **byte-idêntica**.
///
/// ⚠️ *Não é uma feature em falta:* o factor de força deste pincel não tem
/// sinal, e *«esfregar ao contrário»* não nomeia nada — esfregar para o outro
/// lado é andar com a mão para o outro lado.
#[test]
fn inverter_nao_muda_um_bit_do_esfregao() {
    let com = |invert: bool| {
        let (mut mesh, r) = degrau(|p| p[2] = H);
        let mut b = pincel(SmearMode::Drag, 1.0);
        b.invert = invert;
        esfregar(&mut mesh, &r, &b, 2)
    };
    assert_eq!(
        com(false),
        com(true),
        "o Ctrl mudou a saída de um pincel cujo factor não tem sinal"
    );
}

/// ⛔ **SEM REFERÊNCIA ELE NÃO MOVE NADA** (espec §5.6).
///
/// ⚠️ **A recusa em voz alta é da shell**; esta linha é a rede que garante que
/// contorná-la **não inventa geometria**. *A lei não tem para onde apontar, e
/// não fabrica uma superfície.*
#[test]
fn sem_referencia_o_esfregao_e_inerte() {
    let (mut mesh, _) = degrau(|p| p[2] = H);
    let antes = mesh.positions().to_vec();
    // ⚠️ A referência VAZIA, que é a omissão do [`SculptStroke`].
    let depois = esfregar(&mut mesh, &[], &pincel(SmearMode::Drag, 1.0), 2);
    assert_eq!(antes, depois, "sem referência ele mexeu na malha");
}

/// ⭐⭐⭐ **A VIZINHANÇA É MEDIDA NA SUPERFÍCIE DE REFERÊNCIA, nunca nas
/// posições deslocadas** (espec §5.2) — é isso que faz esfregar repetidamente
/// não deformar a topologia.
///
/// **A fixtura discrimina de propósito:** o degrau desloca `+Ŷ` em vez de `+Ẑ`,
/// logo as direcções do anel VIVO junto do degrau deixam de ser as da grelha.
/// Com o anel da referência os pesos a montante valem `1` e `1/√2`; com o anel
/// vivo eles leem `0,707` e `0,447`, e o resultado cai **`15 %`**.
///
/// ⚠️ **O deslocamento é `0,1` e não `0,5`** — grande o bastante para virar as
/// direcções, pequeno o bastante para os vizinhos continuarem DENTRO da pegada.
/// *Com `0,5` eles saem do raio, a orla do §5.4 come tudo, e o gate mede a orla
/// em vez da lei.*
#[test]
fn a_vizinhanca_do_esfregao_e_medida_na_referencia() {
    const DY: f32 = 0.1;
    let (mut mesh, r) = degrau(|p| p[1] += DY);
    let z = esfregar(&mut mesh, &r, &pincel(SmearMode::Drag, 1.0), 2);
    // `D′ = (0 + G·(0, DY, 0)) / (1 + G)` — os mesmos pesos da tabela do
    // cabeçalho, porque o anel da REFERÊNCIA é a grelha lisa.
    let alvo = DY * G / (1.0 + G);
    let lido = z[central(10)][1];
    assert!(
        (lido - alvo).abs() < 1e-5,
        "o vértice da origem foi para y = {lido:.6}; com o anel da referência a \
         conta dá {alvo:.6} (com o anel VIVO daria ~0,0536)"
    );
}

/// ⛔⛔ **O `Accumulate` DESTE PINCEL É UMA LEI QUE REFERÊNCIA NENHUMA DECLARA
/// — medido ANTES de escondido.**
///
/// ⚠️⚠️ **Esconder um knob MORTO e esconder um knob VIVO leem-se igual numa
/// tabela de dívida**, e o que os separa é a medição escrita ao lado. Este está
/// **vivo**: nesta casa o `Accumulate` é o `from_live` do
/// [`crate::Grip::Stamp`] — *de onde a curva de queda mede a distância* —, e
/// ligá-lo muda a saída.
///
/// ⇒ **e é exactamente por isso que ele não pode ser oferecido.** Com um alvo
/// ancorado na superfície de referência, mandar a queda medir da posição JÁ
/// esfregada faria a pegada do pincel depender de quanto relevo ele já
/// transportou — e a espec §5 escreve a lei INTEIRA sem um acumulador. *Um chip
/// cuja lei nós inventámos é a LEI vestida com a autoridade de uma fonte que
/// não a declara*, que é a cerca que o `L` do kelvinlet já paga por escrito.
///
/// ⚠️ **A fixtura precisa de uma curva com QUEDA** — com a `Constant` o peso é
/// `1` em toda a pegada e a origem da distância deixa de ser observável: o gate
/// mediria zero sobre um knob vivo.
#[test]
fn o_acumular_do_esfregao_e_uma_lei_que_ninguem_declara() {
    let saida = |accumulate: bool| {
        let (mut mesh, r) = degrau(|p| p[2] = H);
        let mut b = pincel(SmearMode::Drag, 1.0);
        b.falloff = Falloff::Smooth;
        b.accumulate = accumulate;
        // ⚠️ **QUATRO dabs**: com dois o relevo mal viajou, e a distância
        // medida do vivo ainda coincide com a medida do `pre`.
        esfregar(&mut mesh, &r, &b, 4)
    };
    let (a, b) = (saida(false), saida(true));
    let d = a
        .iter()
        .zip(&b)
        .map(|(p, q)| (q[2] - p[2]).abs())
        .fold(0.0f32, f32::max);
    assert!(
        d > 1e-4,
        "o `Accumulate` mediu {d:.3e} — se ele fosse INERTE, escondê-lo seria \
         arrumação e não uma decisão; esta nota tem de mudar com a medição"
    );
}
