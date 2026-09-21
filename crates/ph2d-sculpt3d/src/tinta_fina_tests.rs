//! **A TINTA FINA, medida no barro** — irmão (`#[path]`) do [`super`].
//!
//! O corte é o de sempre: lá a lei, aqui o que ela promete.

use super::*;
use crate::Symmetry;
use crate::tinta_fina::TintaDoTraco;
use ph2d_mesh::{DEFAULT_COLOR, Mesh, shapes};
use ph2d_mesh_colors::Tinta;

const COR: [f32; 3] = [0.9, 0.2, 0.1];

/// ⭐ O plano de uma peça, **semeado do que ela já tem pintado** — é assim que
/// o produto o arma, e a diferença não é estética: a [`Tinta::nova`] nasce
/// branca e apagaria o trabalho do artista.
fn plano(mesh: &Mesh, nivel: u8) -> Tinta {
    let faces = || mesh.faces().iter().map(ph2d_mesh::Face::verts);
    match mesh.colors() {
        Some(c) => Tinta::semeada(c, faces(), nivel),
        None => Tinta::nova(mesh.vert_count(), faces(), nivel),
    }
}

fn pincel(verb: Verb) -> Brush {
    Brush {
        verb,
        radius: 0.45,
        strength: 1.0,
        color: COR,
        ..Brush::default()
    }
}

/// Um traço de dois dabs sobre a peça, com ou sem a tinta fina armada.
fn traco(mesh: &mut Mesh, verb: Verb, nivel: Option<u8>) -> Option<Tinta> {
    traco_em(mesh, verb, nivel, [0.0, 0.0, 1.0])
}

/// O mesmo, com o centro escolhido.
fn traco_em(mesh: &mut Mesh, verb: Verb, nivel: Option<u8>, centro: [f32; 3]) -> Option<Tinta> {
    let brush = pincel(verb);
    let mut s = SculptStroke::default();
    s.begin(mesh);
    s.tinta_fina = nivel.map(|n| TintaDoTraco::nova(plano(mesh, n)));
    let path = [0.06, 0.0, 0.0];
    for i in 0..2u8 {
        let c = [centro[0] + path[0] * f32::from(i), centro[1], centro[2]];
        let dab = Dab {
            path,
            ..Dab::at(c, 0.45, c)
        };
        s.dab(mesh, &brush, &dab, Symmetry::default());
    }
    s.tinta_fina.take().map(TintaDoTraco::entregar)
}

/// ⭐⭐⭐⭐ **A `lado = 1` A TINTA FINA PINTA O MESMO QUE A COR POR-VÉRTICE, AO
/// BIT — e é isto que faz a família nova entrar sem um degrau de formato.**
///
/// ⛔ Se algum bit divergir, a wave deixou de ser *«a mesma lei noutro
/// sítio»* e passou a ser um segundo pincel — e nenhum corpus de oráculo desta
/// casa o veria, porque eles entram todos pelo caminho por-vértice.
#[test]
fn a_lado_um_a_tinta_fina_pinta_o_mesmo_que_a_cor_por_vertice() {
    let mut a = shapes::uv_sphere(16, 24, 1.0);
    let mut b = a.clone();
    traco(&mut a, Verb::Paint, None);
    let fina = traco(&mut b, Verb::Paint, Some(0)).expect("o plano estava armado");
    let por_vertice = a.colors().expect("o traço pintou");
    assert_eq!(
        fina.amostras(),
        por_vertice,
        "o nível zero divergiu do caminho por-vértice"
    );
    // ⭐ O CONTROLO: o traço de facto pintou alguma coisa.
    assert!(
        por_vertice.iter().any(|c| *c != DEFAULT_COLOR),
        "a fixtura não contém o fenómeno — nada foi pintado"
    );
}

/// ⚠️ **OS DOIS QUE LEEM O ANEL não são byte-idênticos a `lado = 1`, e a razão
/// é a ORDEM DA SOMA — não a lei.**
///
/// O caminho por-vértice soma o anel pela ordem da adjacência da malha; aqui a
/// soma corre pela ordem dos PARES da retícula. O conjunto é o mesmo (há gate
/// na [`ph2d_mesh_colors`] a prová-lo: a `lado = 1` os pares **são** as
/// arestas da malha), e somar `f32` noutra ordem dá outro último bit.
///
/// ⛔ **E há uma segunda diferença, na BORDA da pegada:** o anel de lá sai da
/// adjacência inteira da malha e o daqui só das faces que a consulta trouxe.
/// O peso ali é zero, logo a diferença é multiplicada por zero — *e é por isso
/// que a barra é sobre o que se ESCREVE e não sobre o alvo*.
#[test]
fn os_dois_que_leem_o_anel_concordam_com_o_caminho_por_vertice() {
    for verb in [Verb::Blur, Verb::SmearColor] {
        let mut a = shapes::uv_sphere(16, 24, 1.0);
        crate::canal_de_teste::semeia_cor(&mut a);
        let mut b = a.clone();
        traco(&mut a, verb, None);
        let base = shapes::uv_sphere(16, 24, 1.0);
        let mut semeada = base.clone();
        crate::canal_de_teste::semeia_cor(&mut semeada);
        let fina = traco(&mut b, verb, Some(0)).expect("armado");
        let alvo = a.colors().expect("pintou");
        let pior = alvo
            .iter()
            .zip(fina.amostras())
            .map(|(x, y)| (0..3).map(|k| (x[k] - y[k]).abs()).fold(0.0, f32::max))
            .fold(0.0, f32::max);
        assert!(
            pior <= 1e-5,
            "{verb:?}: o anel da retícula desviou {pior:e} do anel da malha"
        );
        // ⭐ O CONTROLO: a fixtura contém o fenómeno.
        let mexeu = alvo
            .iter()
            .zip(semeada.colors().expect("semeada"))
            .any(|(x, y)| x != y);
        assert!(mexeu, "{verb:?}: o traço não mudou uma cor");
    }
}

/// ⭐⭐⭐ **A MARCA CONVERGE: cada nível aproxima-se do limite, e o erro CAI.**
///
/// A régua não é um epsilon escolhido — é a **convergência**. A tinta de um
/// nível é comparada com a de um nível muito mais fino nos MESMOS pontos da
/// superfície, pela mesma porta que o shader vai ler ([`Tinta::cor_tri`]).
/// Uma lei que não ganhasse resolução daria a mesma coluna em todas as linhas.
#[test]
fn a_marca_fica_mais_fina_a_cada_nivel() {
    let base = shapes::uv_sphere(16, 24, 1.0);
    // ⚠️ **O dab vai ao PÓLO de propósito.** A leitura usa [`Tinta::cor_tri`],
    // logo os pontos têm de estar em faces TRIANGULARES — e numa esfera UV os
    // triângulos são exactamente o leque dos dois pólos. A 1.ª redacção pintou
    // no equador (tudo quads) e leu nos pólos: os quatro níveis deram `6e-7`,
    // que é ruído de `f32`. *Uma régua que lê onde a lei não age mede o nada e
    // chama-lhe empate.*
    const POLO: [f32; 3] = [0.0, 1.0, 0.0];
    let mut limite_mesh = base.clone();
    let limite = traco_em(&mut limite_mesh, Verb::Paint, Some(4), POLO).expect("armado");
    // Pontos de leitura: um leque baricêntrico dentro de cada face.
    let pontos: Vec<(usize, [f32; 3])> = (0..base.faces().len())
        .filter(|f| base.faces()[*f].verts().len() == 3)
        .flat_map(|f| {
            [
                [0.6, 0.3, 0.1],
                [0.2, 0.5, 0.3],
                [0.15, 0.15, 0.7],
                [0.34, 0.33, 0.33],
            ]
            .into_iter()
            .map(move |b| (f, b))
        })
        .collect();
    let erro_de = |t: &Tinta| -> f32 {
        pontos
            .iter()
            .map(|&(f, b)| {
                let cantos = base.faces()[f].verts();
                let x = t.cor_tri(f, cantos, b);
                let y = limite.cor_tri(f, cantos, b);
                (0..3).map(|k| (x[k] - y[k]).abs()).fold(0.0, f32::max)
            })
            .fold(0.0, f32::max)
    };
    // ⭐ O CONTROLO: a fixtura CONTÉM o fenómeno — o limite tem tinta nos
    // pontos que a régua lê. Sem ele, quatro zeros leem-se como empate.
    let pintado = pontos
        .iter()
        .any(|&(f, b)| limite.cor_tri(f, base.faces()[f].verts(), b) != DEFAULT_COLOR);
    assert!(
        pintado,
        "a régua lê onde a lei não age — nenhum ponto de leitura tem tinta"
    );
    let mut erros = Vec::new();
    for nivel in 0..=3u8 {
        let mut m = base.clone();
        let t = traco_em(&mut m, Verb::Paint, Some(nivel), POLO).expect("armado");
        erros.push(erro_de(&t));
    }
    for (n, par) in erros.windows(2).enumerate() {
        assert!(
            par[1] < par[0],
            "erros={erros:?}\n
             do nível {n} para o {}: o erro NÃO caiu ({:.4} → {:.4}) — \
             subir o nível não está a comprar resolução",
            n + 1,
            par[0],
            par[1]
        );
    }
    assert!(
        erros[3] * 3.0 < erros[0],
        "o nível 3 tinha de estar MUITO mais perto do limite que o 0: {:.4} contra {:.4}",
        erros[3],
        erros[0]
    );
}

/// ⛔⛔ **UMA AMOSTRA NÃO É PINTADA DUAS VEZES.**
///
/// Uma amostra de aresta pertence às duas faces que ali se tocam, e uma de
/// canto a todas as faces do anel. Sem o carimbo, a mistura corre duas ou seis
/// vezes no mesmo sítio — e como ela é assintótica, o resultado fica **mais
/// perto da cor do pincel exactamente nas arestas e nos vértices da malha**.
/// *O desenho da malha apareceria na pintura.*
///
/// A régua é a MONOTONIA ao longo de uma linha de amostras que atravessa uma
/// aresta: com o defeito, a amostra do vértice salta acima das vizinhas.
#[test]
fn uma_amostra_nao_e_pintada_duas_vezes() {
    let mut mesh = shapes::uv_sphere(16, 24, 1.0);
    let t = traco(&mut mesh, Verb::Paint, Some(2)).expect("armado");
    // A cor de um vértice pintado não pode estar MAIS perto do pincel do que
    // a amostra vizinha que está mais perto do centro do dab.
    let centro = [0.0, 0.0, 1.0];
    let lado = t.lado();
    let mut pior: f32 = 0.0;
    for (f, face) in mesh.faces().iter().enumerate() {
        let cantos = face.verts();
        if cantos.len() != 3 {
            continue;
        }
        let p: Vec<[f32; 3]> = cantos
            .iter()
            .map(|&v| mesh.positions()[v as usize])
            .collect();
        // o canto 0 e a amostra a um passo dele, na aresta 0.
        let canto = t.indice_tri(f, cantos, lado, 0, 0) as usize;
        let perto = t.indice_tri(f, cantos, lado - 1, 1, 0) as usize;
        let d = |q: [f32; 3]| {
            (0..3)
                .map(|k| (q[k] - centro[k]).powi(2))
                .sum::<f32>()
                .sqrt()
        };
        let a = [
            (p[0][0] * (lado - 1) as f32 + p[1][0]) / lado as f32,
            (p[0][1] * (lado - 1) as f32 + p[1][1]) / lado as f32,
            (p[0][2] * (lado - 1) as f32 + p[1][2]) / lado as f32,
        ];
        // Só onde a vizinha está MAIS PERTO do centro: aí a tinta dela tem de
        // ser pelo menos tão forte quanto a do canto.
        if d(a) < d(p[0]) {
            let forca = |i: usize| 1.0 - t.amostras()[i][1] / DEFAULT_COLOR[1];
            pior = pior.max(forca(canto) - forca(perto));
        }
    }
    assert!(
        pior <= 1e-6,
        "um canto ficou {pior:.4} MAIS pintado do que a vizinha mais perto do centro — \
         a amostra está a ser misturada uma vez por face"
    );
}

/// ⭐ **A JANELA DO DESFAZER enche, e ela guarda a cor de ANTES.**
#[test]
fn a_janela_do_desfazer_enche_com_a_cor_de_antes() {
    let mut mesh = shapes::uv_sphere(16, 24, 1.0);
    let brush = pincel(Verb::Paint);
    let mut s = SculptStroke::default();
    s.begin(&mesh);
    s.tinta_fina = Some(TintaDoTraco::nova(plano(&mesh, 2)));
    let c = [0.0, 0.0, 1.0];
    s.dab(&mut mesh, &brush, &Dab::at(c, 0.45, c), Symmetry::default());
    let fina = s.tinta_fina.take().expect("armado");
    let tocadas = fina.tocadas().to_vec();
    let base = fina.base().to_vec();
    assert!(!tocadas.is_empty(), "o traço não tocou amostra nenhuma");
    assert_eq!(tocadas.len(), base.len());
    assert!(
        base.iter().all(|c| *c == DEFAULT_COLOR),
        "a janela tinha de guardar a cor de ANTES, que numa peça virgem é o default"
    );
    // ⭐ E o CONTROLO: as amostras tocadas de facto mudaram.
    let t = fina.entregar();
    assert!(
        tocadas
            .iter()
            .any(|&i| t.amostras()[i as usize] != DEFAULT_COLOR),
        "nenhuma amostra tocada mudou de cor"
    );
}

/// ⛔⛔ **A PREMISSA da tinta fina, afirmada: os verbos de cor não movem barro.**
///
/// O peso de uma amostra é calculado sobre a malha VIVA (posição, normal,
/// máscara). Isso só é o mesmo que o congelado porque estes três verbos não
/// escrevem geometria. *No dia em que um verbo de cor mexer num vértice, esta
/// wave passa a ler o que ela própria acabou de escrever* — e este gate
/// reprova antes disso.
#[test]
fn os_verbos_de_cor_nao_movem_barro() {
    for verb in [Verb::Paint, Verb::Blur, Verb::SmearColor] {
        let antes = shapes::uv_sphere(16, 24, 1.0);
        let mut mesh = antes.clone();
        traco(&mut mesh, verb, None);
        assert_eq!(
            mesh.positions(),
            antes.positions(),
            "{verb:?} moveu um vértice"
        );
        assert_eq!(mesh.normals(), antes.normals(), "{verb:?} mudou uma normal");
        assert_eq!(
            mesh.masks().map(<[f32]>::to_vec),
            antes.masks().map(<[f32]>::to_vec),
            "{verb:?} escreveu a máscara"
        );
    }
}

/// **A SONDA da escada** — ela IMPRIME o que o gate acima afirma, para o número
/// poder entrar num handoff sem alguém o estimar.
///
///     cargo test -p ph2d-sculpt3d --release --lib diag_a_escada_da_tinta \
///       -- --ignored --nocapture
#[test]
#[ignore = "sonda: imprime a escada, não afirma nada que o gate irmão não afirme"]
fn diag_a_escada_da_tinta() {
    let base = shapes::uv_sphere(16, 24, 1.0);
    const POLO: [f32; 3] = [0.0, 1.0, 0.0];
    let mut lm = base.clone();
    let limite = traco_em(&mut lm, Verb::Paint, Some(4), POLO).expect("armado");
    let pontos: Vec<(usize, [f32; 3])> = (0..base.faces().len())
        .filter(|f| base.faces()[*f].verts().len() == 3)
        .flat_map(|f| {
            [
                [0.6, 0.3, 0.1],
                [0.2, 0.5, 0.3],
                [0.15, 0.15, 0.7],
                [0.34, 0.33, 0.33],
            ]
            .into_iter()
            .map(move |b| (f, b))
        })
        .collect();
    println!(
        "{:>6} {:>10} {:>12} {:>10}",
        "nível", "lado", "amostras", "erro"
    );
    for nivel in 0..=4u8 {
        let mut m = base.clone();
        let t = traco_em(&mut m, Verb::Paint, Some(nivel), POLO).expect("armado");
        let e = pontos
            .iter()
            .map(|&(f, b)| {
                let c = base.faces()[f].verts();
                let (x, y) = (t.cor_tri(f, c, b), limite.cor_tri(f, c, b));
                (0..3).map(|k| (x[k] - y[k]).abs()).fold(0.0, f32::max)
            })
            .fold(0.0, f32::max);
        println!(
            "{nivel:>6} {:>10} {:>12} {e:>10.5}",
            t.lado(),
            t.amostras().len()
        );
    }
}

/// ⛔⛔ **NUMA PEÇA DE COR UNIFORME, O `Blur` NÃO MUDA UM BIT** — o gémeo do
/// `numa_peca_de_cor_uniforme_os_dois_sao_inertes_ao_bit` do caminho
/// por-vértice, e é esta a propriedade que obriga a DIVIDIR em vez de
/// multiplicar pelo recíproco.
///
/// ⚠️⚠️ **A cor é a de FÁBRICA, e isso está MEDIDO e não copiado do irmão.**
/// Com uma cor qualquer (`0,3 · 0,7 · 0,45`) as DUAS rotas mexem um ULP — o
/// caminho por-vértice em `4` vértices e a tinta fina em `103` amostras (que
/// são mais porque são mais) —, e a causa é a forma do aplicador que as duas
/// partilham (`b·(1−a) + t·a`, a que o [`crate::stroke_apply`] mede com
/// **53 315** divergências na coluna `t = b`). ⇒ *a inércia é uma promessa
/// sobre a cor de fábrica, e o gate diz qual é em vez de a herdar.* A sonda
/// que o mediu é a `diag_a_inercia_com_uma_cor_qualquer`, ao lado.
#[test]
fn o_blur_e_no_op_ao_bit_numa_peca_de_cor_uniforme() {
    let mut mesh = shapes::uv_sphere(16, 24, 1.0);
    const C: [f32; 3] = DEFAULT_COLOR;
    mesh.colors_mut().fill(C);
    let antes = plano(&mesh, 2);
    // ⛔⛔ **OS DOIS, e o esfregão é o que DISCRIMINA.** No `Blur` os pesos são
    // uns e o quociente `n/n` arredonda de volta a `1` mesmo escrito como
    // `n × (1/n)`; no esfregão o denominador é uma soma de pesos ARBITRÁRIOS, e
    // é ali que a forma do recíproco erra o último bit. *Correr só o `Blur`
    // deixava a mutação do recíproco viva* — e foi exactamente o que aconteceu
    // na primeira prova.
    for verb in [Verb::Blur, Verb::SmearColor] {
        let mut m = mesh.clone();
        let depois = traco(&mut m, verb, Some(2)).expect("armado");
        assert_eq!(
            depois.amostras(),
            antes.amostras(),
            "{verb:?} mexeu numa peça de cor uniforme"
        );
    }
    // ⭐ O CONTROLO: numa peça NÃO uniforme o mesmo traço mexe — senão este
    // gate ficaria verde sobre um pincel inerte.
    let mut m2 = shapes::uv_sphere(16, 24, 1.0);
    crate::canal_de_teste::semeia_cor(&mut m2);
    let semeado = plano(&m2, 2);
    let d2 = traco(&mut m2, Verb::Blur, Some(2)).expect("armado");
    assert_ne!(
        d2.amostras(),
        semeado.amostras(),
        "o controlo não contém o fenómeno: o Blur não mexeu nem na peça com fronteira"
    );
}

/// ⚠️ **SONDA: de quem é o último bit?** O gate irmão do caminho por-vértice
/// afirma a inércia com a cor de fábrica (`1,1,1`); esta sonda corre as DUAS
/// rotas com uma cor qualquer, para o desvio ter dono.
///
///     cargo test -p ph2d-sculpt3d --lib diag_a_inercia -- --ignored --nocapture
#[test]
#[ignore = "sonda: imprime de quem é o último bit"]
fn diag_a_inercia_com_uma_cor_qualquer() {
    for c in [[1.0f32, 1.0, 1.0], [0.3, 0.7, 0.45]] {
        let mut a = shapes::uv_sphere(16, 24, 1.0);
        a.colors_mut().fill(c);
        let antes = a.colors().expect("plano").to_vec();
        traco(&mut a, Verb::Blur, None);
        let mexeu_v = a
            .colors()
            .expect("plano")
            .iter()
            .zip(&antes)
            .filter(|(x, y)| x != y)
            .count();
        let mut b = shapes::uv_sphere(16, 24, 1.0);
        b.colors_mut().fill(c);
        let base = plano(&b, 2);
        let fina = traco(&mut b, Verb::Blur, Some(2)).expect("armado");
        let mexeu_f = fina
            .amostras()
            .iter()
            .zip(base.amostras())
            .filter(|(x, y)| x != y)
            .count();
        println!("cor {c:?}: por-vértice mexeu {mexeu_v}, tinta fina mexeu {mexeu_f}");
    }
}

/// ⭐⭐ **AS DUAS CONSTANTES DO SENTINELA CONCORDAM** — o gate que o
/// doc-comment da [`ph2d_mesh_colors::topo::TRI`] promete, e que **não
/// existia**.
///
/// ⛔⛔ A [`ph2d-mesh-colors`] declara **zero dependências** de propósito (ela
/// é a lei da retícula e não sabe o que é uma `Mesh`), logo ela **não pode**
/// importar a [`ph2d_mesh::TRI`] para a comparar. ⇒ o único sítio onde as duas
/// são visíveis é uma crate que dependa das duas, e esta é a primeira.
///
/// ⚠️ **Isto é a família que esta linha curou em 13/09** — oito gates citados
/// em comentário que nunca tinham existido — e a crate nova reintroduziu-a: o
/// doc dizia *«é GATEADO do lado de lá»* e apontava para um nome que
/// `git log -S` não encontra. *Um gate citado e um gate escrito leem-se igual
/// num cabeçalho.*
///
/// ⚠️ E o censo que impede a recaída ([`crate::named_gates_census_tests`]) lê
/// **esta** crate, não as folhas que ela consome — a citação vivia fora do
/// alcance dele.
#[test]
fn o_sentinela_do_triangulo_e_o_mesmo_nas_duas_crates() {
    assert_eq!(
        ph2d_mesh_colors::topo::TRI,
        ph2d_mesh::TRI,
        "o sentinela do 4.º índice divergiu entre a malha e a lei da tinta"
    );
}

/// ⭐⭐⭐⭐ **GATE — A JANELA DAS AMOSTRAS SUJAS é o que o traço ESCREVEU desde
/// o último upload**, e é ela que tira o custo do device de cima de `O(plano)`.
///
/// ⚠️ **As três metades medem três coisas diferentes, e a segunda é a que uma
/// implementação ingénua falha:**
///
/// 1. o que o dab escreveu ESTÁ na janela;
/// 2. **drenar LIMPA** — o quadro seguinte sem dabs não reescreve nada
///    (*«tocada uma vez»* e *«mudou desde o último upload»* são grandezas
///    diferentes, e um upload que confundisse as duas voltaria a ser
///    `O(pegada do TRAÇO)` em vez de `O(pegada do QUADRO)`);
/// 3. uma amostra RE-ESCRITA por um dab seguinte volta à janela — sem isto o
///    device ficaria com a cor do primeiro dab e o traço parecia parar.
///
/// ⛔⛔ **A metade (3) era VÁCUA e foi uma mutação sobrevivente que o disse**
/// (a `U1` do [`docs/3D/ferramentas/muta_o_upload_da_tinta.sh`], 2026-09-21):
/// ela afirmava `!sujas.is_empty()`, e um dab que ANDOU toca amostras NOVAS —
/// que nascem sujas — logo a janela nunca vinha vazia **mesmo com a marca por
/// escrita apagada**. *Uma régua que conta QUANTOS nunca vê QUAIS*, a forma que
/// esta casa já pagou no `edge_max` e no `χ`. Hoje ela mede a INTERSECÇÃO com o
/// conjunto do 1.º dab, que é a única população em que a lei é observável.
#[test]
fn a_janela_das_amostras_sujas_e_o_que_o_traco_escreveu() {
    let mut m = shapes::uv_sphere(16, 24, 1.0);
    let brush = pincel(Verb::Paint);
    let mut s = SculptStroke::default();
    s.begin(&m);
    s.tinta_fina = Some(TintaDoTraco::nova(plano(&m, 2)));

    let mut sujas = Vec::new();
    let dab_em = |s: &mut SculptStroke, m: &mut Mesh, x: f32| {
        let c = [x, 0.0, 1.0];
        let dab = Dab {
            path: [0.02, 0.0, 0.0],
            ..Dab::at(c, 0.45, c)
        };
        s.dab(m, &brush, &dab, Symmetry::default());
    };

    dab_em(&mut s, &mut m, 0.0);
    let fina = s.tinta_fina.as_mut().expect("armado");
    fina.drena_sujas(&mut sujas);
    let primeiro = sujas.len();
    // ⚠️ **O conjunto do 1.º dab é o que torna a metade (3) uma medição** —
    // ver o cabeçalho. `BTreeSet` e não `HashSet` (HR-5).
    let do_primeiro: std::collections::BTreeSet<u32> = sujas.iter().copied().collect();
    assert!(
        primeiro > 0,
        "o dab não sujou uma amostra: a fixtura não contém o fenómeno"
    );
    assert_eq!(
        primeiro,
        fina.tocadas().len(),
        "o 1.º dab tocou {} amostras e sujou {primeiro} -- na primeira escrita \
         as duas grandezas são a mesma",
        fina.tocadas().len()
    );

    // (2) Drenar LIMPA.
    fina.drena_sujas(&mut sujas);
    assert!(
        sujas.is_empty(),
        "a janela não foi limpa: {} amostras voltaram sem ninguém escrever",
        sujas.len()
    );

    // (3) Uma amostra RE-ESCRITA volta.
    dab_em(&mut s, &mut m, 0.01);
    let fina = s.tinta_fina.as_mut().expect("armado");
    fina.drena_sujas(&mut sujas);
    let revisitadas = sujas.iter().filter(|i| do_primeiro.contains(i)).count();
    assert!(
        revisitadas > 0,
        "o 2.º dab não devolveu à janela NENHUMA das {} amostras que o 1.º já tinha \
         escrito -- ele sujou {} e todas são NOVAS.\n\
         É o defeito que esta metade existe para apanhar: sem a marca em cada \
         escrita, o device fica com a cor do 1.º dab onde os dois se sobrepõem, \
         e o traço parece parar por baixo da mão.",
        do_primeiro.len(),
        sujas.len()
    );
    assert!(
        sujas.len() <= fina.tocadas().len(),
        "a janela do 2.º quadro ({}) é maior que o traço inteiro ({})",
        sujas.len(),
        fina.tocadas().len()
    );
}
