//! **OS GATES DO FLIP DE ARESTA** — filho do [`super`], não irmão.
//!
//! Eles têm um assunto só: *a troca de diagonal*, cujo operador vive no seu
//! próprio módulo ([`crate::dyntopo_flip`]). O pai julga a LEI do corte (a
//! ausência de rachadura, o alcance do pincel, o padrão do corte); aqui julgam-se
//! as **quatro recusas** do flip e o que ele existe para entregar.
//!
//! ⚠️ **FILHO e não irmão** para que `tri_sphere`, `cracks` e `scratch`
//! continuem sendo uma PORTA e não uma segunda cópia — a mesma razão do
//! [`super::splice`]. *Uma fixture duplicada é como dois gates passam a testar
//! duas malhas diferentes com o mesmo nome.*
//!
//! ⚠️ **Nasceram no ficheiro do pai e mudaram-se em 2026-08-21**, quando a quarta
//! recusa levou o `dyntopo_tests.rs` a **740 LOC** contra o teto de 700. ⛔ A cura
//! de um teto de ficheiro é o corte para o IRMÃO, nunca uma entrada na lista de
//! exceções.

use super::*;

/// **O FLIP PERGUNTA SÓ PELO QUE O CORTE MEXEU.**
///
/// ⚠️ **A fixture é construída para CONTER o fenômeno, e sem isso o gate seria
/// vazio:** uma esfera UV já é estável a flip fora da região do dab (medido — um
/// dab a 28k altera 3648 faces, todas dentro da esfera do pincel e nenhuma
/// fora), então sobre ela um flip global e um flip local dão o MESMO resultado e
/// nenhum oráculo os separa. Aqui um par de faces é deliberadamente virado para
/// a pior diagonal, longe de tudo: uma varredura global o encontraria e o
/// consertaria, uma varredura da região não.
///
/// As duas metades são independentes e as duas são precisas:
///
/// 1. **Sem sementes ele não sai à procura** — é o escopo.
/// 2. **Apontado para o estrago ele repara** — é o controle positivo, e sem ele
///    a primeira metade passaria com um operador que simplesmente não funciona.
#[test]
fn the_flip_asks_only_about_the_faces_the_cut_touched() {
    let (mut m, pair) = wreck_the_worst_pair(&tri_sphere(16, 24));
    assert!(
        has_pair(&m, pair),
        "o controle: a fixture tem de conter o estrago"
    );

    crate::dyntopo_flip::relax(&mut m, &[], &mut scratch());
    assert!(
        has_pair(&m, pair),
        "o flip varreu a malha atrás de trabalho que ninguém pediu"
    );

    let seed = face_of(&m, pair[0]).expect("a face estragada está na malha");
    crate::dyntopo_flip::relax(&mut m, &[seed], &mut scratch());
    assert!(
        !has_pair(&m, pair),
        "o operador tem de QUERER reparar isto — sem esta metade, a de cima é vazia"
    );
}

/// ⭐ **A RECUSA 4: duas trocas da mesma rodada não podem criar a MESMA
/// diagonal.**
///
/// ⚠️ **A recusa 2 não alcança isto, e a razão é o instante em que ela pergunta.**
/// Ela consulta o anel de `c` na adjacência de ENTRADA; duas trocas sobre pares
/// de faces disjuntos — logo invisíveis ao `spent`, que só protege a face —
/// produzem `c—d` sem que nenhuma das duas veja a outra. A malha sai com **duas
/// arestas entre o mesmo par**, o que aqui aparece como uma aresta de valência
/// **4** (a assinatura de *criada duas vezes*: uma criada por cima de uma que já
/// existia teria 3).
///
/// **A fixture é a MENOR que contém o fenómeno**, e isso foi medido, não
/// escolhido — uma rodada de `relax_valence`, com a recusa 4 desligada:
///
/// | fixture | vértices | trocas | arestas de valência ≠ 2 |
/// |---|---|---|---|
/// | `uv_sphere(*)` (lisa, qualquer tamanho) | — | 0 | 0 — *não flipa, não prova nada* |
/// | `uv_sphere_shuffled(48,72)` | 3 386 | 2 390 | **0** |
/// | ⭐ `uv_sphere_noisy(24,36)` | **830** | 457 | **7** |
/// | `uv_sphere_noisy(96,144)` | 13 682 | 8 590 | **185** |
/// | `uv_sphere_shuffled(96,144)` | 13 682 | 9 968 | 1 |
///
/// ⚠️ **A esfera LISA é o controle negativo que quase enganou:** ela não aceita
/// troca nenhuma (0 flips), então passaria com o operador inteiro apagado. É o
/// **ruído** que dá ao par `c,d` a valência alta de que a colisão precisa.
#[test]
fn a_round_of_flips_never_creates_the_same_diagonal_twice() {
    let mut m = shapes::uv_sphere_noisy(24, 36, 1.0, 0.02);
    m.triangulate();
    assert_eq!(cracks(&m), 0, "o controle: a fixture nasce variedade");

    let flips = crate::dyntopo_flip::relax_valence(&mut m, &mut scratch());
    // ⚠️ O controle POSITIVO: sem trocas nenhumas a asserção de baixo é vazia, e
    // é exatamente assim que uma esfera lisa passaria com o operador desligado.
    assert!(
        flips > 100,
        "a fixture tem de FLIPAR para que este gate afirme algo — só {flips}"
    );
    assert_eq!(
        cracks(&m),
        0,
        "duas trocas da mesma rodada criaram a mesma diagonal: a malha deixou de ser variedade"
    );
}

/// **Troca a diagonal do par vizinho cuja troca mais PIORA a qualidade** — o
/// estrago que o operador vai querer desfazer. Devolve a malha e as duas faces
/// novas por conjunto de vértices (que sobrevive a renumeração).
fn wreck_the_worst_pair(m: &Mesh) -> (Mesh, [[u32; 3]; 2]) {
    let pos = m.positions();
    let adj = m.adjacency();
    let src = m.faces();
    let mut best: Option<(f32, usize, usize, [u32; 4])> = None;
    for (i0, f0) in src.iter().enumerate() {
        if !f0.is_tri() {
            continue;
        }
        let v0 = f0.verts();
        for k in 0..3 {
            let (ea, eb) = (v0[k], v0[(k + 1) % 3]);
            let Some(i1) = adj
                .vert_faces
                .neighbours(ea as usize)
                .iter()
                .copied()
                .find(|&j| j as usize != i0 && src[j as usize].verts().contains(&eb))
                .map(|j| j as usize)
            else {
                continue;
            };
            if !src[i1].is_tri() {
                continue;
            }
            let Some((a, b, c, d)) = quad_of(src, i0, i1) else {
                continue;
            };
            let p = |v: u32| pos[v as usize];
            let old = min_angle(p(a), p(b), p(c)).min(min_angle(p(b), p(a), p(d)));
            let new = min_angle(p(a), p(d), p(c)).min(min_angle(p(d), p(b), p(c)));
            let loss = old - new;
            if best.is_none_or(|(l, ..)| loss > l) {
                best = Some((loss, i0, i1, [a, b, c, d]));
            }
        }
    }
    let (loss, i0, i1, [a, b, c, d]) = best.expect("a esfera tem pares vizinhos");
    assert!(
        loss > 1.0,
        "a fixture precisa de um par cuja troca piore de verdade: {loss} grau(s)"
    );
    let (n0, n1) = (Face::tri(a, d, c), Face::tri(d, b, c));
    let mut faces = src.to_vec();
    faces[i0] = n0;
    faces[i1] = n1;
    let wrecked = Mesh::from_parts(pos.to_vec(), faces).expect("a troca não inventa índice");
    (wrecked, [sorted(n0), sorted(n1)])
}

/// Os quatro cantos do quadrilátero de duas faces vizinhas — o mesmo desenho do
/// `dyntopo_flip::quad`, escrito aqui porque uma FIXTURE constrói um estado; ela
/// não pode chamar a função sob teste para decidir o que espera.
fn quad_of(faces: &[Face], i0: usize, i1: usize) -> Option<(u32, u32, u32, u32)> {
    let (t0, t1) = (faces[i0].verts(), faces[i1].verts());
    let k = (0..3).find(|&k| !t1.contains(&t0[k]))?;
    let c = t0[k];
    let (a, b) = (t0[(k + 1) % 3], t0[(k + 2) % 3]);
    let d = *t1.iter().find(|v| **v != a && **v != b)?;
    Some((a, b, c, d))
}

fn min_angle(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3]) -> f32 {
    let pts = [p0, p1, p2];
    let mut worst = 180.0f32;
    for k in 0..3 {
        let (o, u, v) = (pts[k], pts[(k + 1) % 3], pts[(k + 2) % 3]);
        let a = [u[0] - o[0], u[1] - o[1], u[2] - o[2]];
        let b = [v[0] - o[0], v[1] - o[1], v[2] - o[2]];
        let la = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt();
        let lb = (b[0] * b[0] + b[1] * b[1] + b[2] * b[2]).sqrt();
        if la < 1e-12 || lb < 1e-12 {
            return 0.0;
        }
        let c = ((a[0] * b[0] + a[1] * b[1] + a[2] * b[2]) / (la * lb)).clamp(-1.0, 1.0);
        worst = worst.min(c.acos().to_degrees());
    }
    worst
}

fn sorted(f: Face) -> [u32; 3] {
    let v = f.verts();
    let mut k = [v[0], v[1], v[2]];
    k.sort_unstable();
    k
}

fn face_of(m: &Mesh, key: [u32; 3]) -> Option<u32> {
    m.faces()
        .iter()
        .position(|f| f.is_tri() && sorted(*f) == key)
        .map(|i| u32::try_from(i).unwrap_or(u32::MAX))
}

fn has_pair(m: &Mesh, pair: [[u32; 3]; 2]) -> bool {
    pair.iter().all(|k| face_of(m, *k).is_some())
}

/// ⭐⭐⭐ **SEM PREFERÊNCIA, A PORTA NOVA NÃO EXISTE — o controlo do
/// [`crate::alinha_arestas`].**
///
/// Uma preferência CONSTANTE nunca satisfaz `nova > antiga + ganho`, logo o passe
/// é inerte **ao bit**. ⛔ Sem esta metade, tudo o que os gates abaixo afirmam
/// poderia ser obra de um passe que troca diagonais por conta própria.
#[test]
fn uma_preferencia_constante_deixa_a_malha_ao_bit() {
    let base = tri_sphere(16, 24);
    let mut m = base.clone();
    let plana = |_: [f32; 3]| 1.0f32;
    let trocas = crate::alinha_arestas(&mut m, [0.0, 0.0, 0.0], 10.0, &plana, &mut scratch());
    assert_eq!(
        trocas, 0,
        "uma preferencia constante trocou {trocas} arestas"
    );
    assert_eq!(
        m.faces(),
        base.faces(),
        "a ligacao mudou com uma preferencia que nao prefere nada"
    );
    assert_eq!(m.positions(), base.positions());
}

/// ⭐⭐⭐ **ELE NÃO CRIA NEM APAGA VÉRTICE, e é isso que o torna grátis.**
///
/// As outras duas metades do pente alinham **criando** arestas — e medido, isso
/// adensa a malha ou afina o triângulo. ⚠️ *Esta é a propriedade inteira desta
/// porta*, e sem gate ela é uma frase num cabeçalho.
#[test]
fn alinhar_por_troca_nao_muda_a_contagem() {
    let base = tri_sphere(16, 24);
    let mut m = base.clone();
    // Uma preferência de quatro dobras à volta de `+x`, que é a forma que o
    // pente usa — ver `ph2d_rake::preferencia_do_pente`.
    let pref = |u: [f32; 3]| {
        let c2 = u[0] * u[0];
        8.0 * c2 * c2 - 8.0 * c2 + 1.0
    };
    let trocas = crate::alinha_arestas(&mut m, [0.0, 0.0, 0.0], 10.0, &pref, &mut scratch());
    // O controlo POSITIVO: sem trocas as igualdades abaixo são triviais.
    assert!(trocas > 0, "a fixtura nao contem o fenomeno: zero trocas");
    assert_eq!(
        m.vert_count(),
        base.vert_count(),
        "a contagem de vertices mudou"
    );
    assert_eq!(
        m.face_count(),
        base.face_count(),
        "a contagem de faces mudou"
    );
    assert_eq!(
        m.positions(),
        base.positions(),
        "um vertice moveu-se — esta porta so' muda a LIGACAO"
    );
}

/// ⭐⭐⭐ **A REGIÃO É UMA CERCA: fora da esfera nem uma ligação muda.**
///
/// ⚠️ É a mesma promessa do [`relax`] e do corte, e ela é o que faz a topologia
/// dinâmica ser *local* — *um passe que alinha o modelo inteiro a cada dab é o
/// oposto exacto da promessa deste modo*.
#[test]
fn o_alinhamento_para_na_borda_da_esfera() {
    let base = tri_sphere(16, 24);
    let mut m = base.clone();
    let pref = |u: [f32; 3]| {
        let c2 = u[0] * u[0];
        8.0 * c2 * c2 - 8.0 * c2 + 1.0
    };
    // ⚠️⚠️ **O CENTRO É ACHADO, nunca escolhido.** O ganho exigido é `0,20` em
    // `cos 4α`, logo há calotas inteiras desta esfera onde não existe uma única
    // troca que o atinja — e um `trocas > 0` sobre uma região dessas acusaria a
    // PORTA em vez da fixtura. ⇒ corre-se o passe GLOBAL numa cópia, pega-se
    // numa face que ele mudou, e a calota nasce em cima dela. *Uma fixtura que
    // se localiza sozinha não envelhece com a malha.*
    let onde = {
        let mut sonda = base.clone();
        crate::alinha_arestas(&mut sonda, [0.0; 3], 10.0, &pref, &mut scratch());
        let i = base
            .faces()
            .iter()
            .zip(sonda.faces())
            .position(|(a, b)| a != b)
            .expect("o passe global nao mudou nada — a fixtura nao contem o fenomeno");
        let v = sonda.faces()[i].verts();
        let p = |k: usize| sonda.positions()[v[k] as usize];
        [
            (p(0)[0] + p(1)[0] + p(2)[0]) / 3.0,
            (p(0)[1] + p(1)[1] + p(2)[1]) / 3.0,
            (p(0)[2] + p(1)[2] + p(2)[2]) / 3.0,
        ]
    };
    let centro = onde;
    let raio = 0.35f32;
    let trocas = crate::alinha_arestas(&mut m, centro, raio, &pref, &mut scratch());
    assert!(trocas > 0, "a fixtura nao contem o fenomeno: zero trocas");
    // ⚠️⚠️ **A cerca NÃO é a esfera do dab, e escrever que era seria uma
    // promessa que o motor nunca fez:** cada rodada semeia a seguinte com as
    // faces que acabou de mudar (é assim que o [`relax`] repara o corte), logo a
    // região CRESCE alguns anéis. ⇒ o que este gate afirma é a propriedade que
    // importa — **o passe é LOCAL**: o lado oposto da peça fica intacto, e um
    // passe que alinhasse o modelo inteiro a cada dab seria o oposto exacto da
    // promessa deste modo.
    //
    // ⛔⛔ **A CERCA ERA `1,4` E ESTAVA NO FIO DA NAVALHA.** Medido o alcance
    // real nesta fixtura (o vértice mais distante de uma face que mudou), por
    // chão do alinhamento: `24° → 1,5371` · `20° → 1,5371` · `18° → 1,7254` ·
    // **`16° → 1,7254`** · `14° → 1,7144`. A `1,4` o gate passava por **não
    // existir face com TODOS os vértices lá fora**, e o degrau seguinte de chão
    // criava uma — *uma cerca que depende de nenhuma face cair inteira num anel
    // é uma cerca que a próxima constante move*.
    //
    // ⭐⭐ **E a fixtura é GROSSEIRA, o que não é o regime do artista:** medido
    // no regime que a `=49` dá (`diag_o_alcance_do_flip`), o alcance é o **MESMO
    // nos dois chãos** — `0,3896` de mundo, `2,38` raios de pincel, `8,1`
    // arestas — e o que muda é só quantas trocas acontecem lá dentro (`98` a
    // `24°` contra `167` a `16°`). *O que cresce com o chão é a densidade do
    // trabalho, não a pegada.*
    let longe: Vec<usize> = base
        .faces()
        .iter()
        .enumerate()
        .filter(|(_, f)| {
            f.verts().iter().all(|v| {
                let p = base.positions()[*v as usize];
                let d = [p[0] - centro[0], p[1] - centro[1], p[2] - centro[2]];
                (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt() > LONGE
            })
        })
        .map(|(i, _)| i)
        .collect();
    // O controlo: se não houver faces longe, a asserção abaixo é vácuo.
    assert!(
        longe.len() > 100,
        "so' {} faces estao longe do dab (cerca {LONGE}) — a fixtura nao separa \
         local de global",
        longe.len()
    );
    for i in longe {
        assert_eq!(
            base.faces()[i],
            m.faces()[i],
            "a face {i} mudou no lado OPOSTO da peca — o passe deixou de ser local"
        );
    }
    // ⭐⭐ **E a METADE que a cerca sozinha não afirma: QUANTO a região cresce.**
    // Sem ela, alguém sobe o `LONGE` no dia em que um chão novo o encostar e o
    // gate continua verde a medir cada vez menos. *O alcance é um NÚMERO e tem
    // de ser afirmado como número.*
    let mut alcance = 0.0f32;
    for (i, f) in base.faces().iter().enumerate() {
        if *f == m.faces()[i] {
            continue;
        }
        for v in f.verts() {
            let p = base.positions()[*v as usize];
            let d = [p[0] - centro[0], p[1] - centro[1], p[2] - centro[2]];
            alcance = alcance.max((d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt());
        }
    }
    assert!(
        alcance < LONGE,
        "o flip chegou a {alcance:.4} do centro do dab (raio {raio}, cerca \
         {LONGE}; medido 1,7254 nesta fixtura) — a regiao cresceu, e a cerca \
         acima passou a medir o vazio"
    );
}

/// A que distância do dab uma face deixa de poder ser tocada, nesta fixtura.
///
/// ⛔ **Ele não é escolhido: é o alcance MEDIDO com folga.** O passe chega a
/// `1,7254` (ver o corpo do gate) e a peça é uma esfera de raio `1`, logo o
/// antípoda está a `2` — a cerca fica no meio do que sobra.
const LONGE: f32 = 1.85;

/// ⭐⭐⭐ **A TROCA POR DIRECÇÃO NÃO COMPRA ALINHAMENTO COM VINCO.**
///
/// ⛔⛔⛔ **Ela nasceu de um report com FOTO** (2026-09-19: *«o resultado fica
/// pior que o original, com irregularidade a 90 graus da direcção do
/// movimento»*). A fixtura é um quad **NÃO PLANO** montado para o caso: a
/// diagonal antiga está a `45°` do traço — a direcção que a lei menos quer — e a
/// nova está **ao longo** dele, a que ela mais quer. O ganho de alinhamento é
/// `1,5` contra um limiar de `0,20`: *a troca é desejadíssima, e abriria o par*.
///
/// ⚠️ **As duas metades são obrigatórias:** sem o CONTROLO plano, um passe que
/// recusasse tudo ficava verde. Com ele, o gate afirma a lei inteira — *recusa o
/// que vinca, aceita o que não vinca*, com o alinhamento comprado a ser
/// EXACTAMENTE o mesmo nos dois.
///
/// ⚠️ **A fixtura é sintética porque o produto não a discrimina:** medida na
/// bola da `=49`, esta cerca move o vinco `p90` de `4,493` para `4,391` — `2 %`.
/// *Uma cerca de zero disparos mensuráveis ou ganha fixtura própria ou sai.*
#[test]
fn a_troca_por_direccao_nao_compra_alinhamento_com_vinco() {
    // `c—d` ao longo de `x` (o que a lei quer); `a—b` a `45°` (o que ela evita).
    // Com `h > 0` o par NOVO abre-se num telhado.
    let quad = |h: f32| {
        let pos = vec![
            [-0.7, -0.7, h],  // a = 0
            [0.7, 0.7, h],    // b = 1
            [-1.0, 0.0, 0.0], // c = 2
            [1.0, 0.0, 0.0],  // d = 3
        ];
        let faces = vec![Face::tri(0, 1, 2), Face::tri(1, 0, 3)];
        Mesh::from_parts(pos, faces).expect("o quad e' uma malha valida")
    };
    // A MESMA forma que o pente usa (`cos 4α` com o traço ao longo de `x`), e a
    // mesma que os gates vizinhos deste ficheiro escrevem.
    let pref = |u: [f32; 3]| {
        let c2 = u[0] * u[0];
        8.0 * c2 * c2 - 8.0 * c2 + 1.0
    };

    // (1) O CONTROLO: plano, a troca ACONTECE.
    let mut plano = quad(0.0);
    let n = crate::alinha_arestas(&mut plano, [0.0; 3], 4.0, &pref, &mut scratch());
    assert_eq!(
        n, 1,
        "o quad PLANO nao trocou: a fixtura nao contem o fenomeno, e a metade \
         de baixo passaria por vacuo"
    );

    // (2) E com o par que ela ABRIRIA, recusa.
    let mut dobrado = quad(0.45);
    let antes = dobrado.faces().to_vec();
    let n = crate::alinha_arestas(&mut dobrado, [0.0; 3], 4.0, &pref, &mut scratch());
    assert_eq!(
        n, 0,
        "a troca aceitou um par que ela ABRE — e' a ondulacao que o dono \
         fotografou em 19/09: o alinhamento sobe e a luz piora"
    );
    assert_eq!(
        dobrado.faces(),
        &antes[..],
        "a malha mudou com zero trocas declaradas"
    );
}
