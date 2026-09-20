//! **AS DUAS LEIS DE COR QUE LEEM O ANEL, medidas no barro** — irmão
//! (`#[path]`) do [`super`], e o corte é o ASSUNTO: lá a lei, aqui o que ela
//! promete.
//!
//! ⚠️ **A fixtura é uma FRONTEIRA e não ruído** (ver
//! [`crate::canal_de_teste::semeia_cor`]): uma média do anel tem de a esbater e
//! um transporte tem de a MOVER, e as duas coisas são indistinguíveis sobre uma
//! peça de cor uniforme — onde os dois verbos devolvem o que já lá estava, ao
//! bit.

use super::*;
use crate::Symmetry;
use crate::canal_de_teste::semeia_cor;
use ph2d_mesh::{DEFAULT_COLOR, Mesh, shapes};

/// A peça com a faixa de cor e a fronteira em `x = 0`.
fn peca() -> Mesh {
    let mut m = shapes::uv_sphere(24, 36, 1.0);
    semeia_cor(&mut m);
    m
}

/// O vértice mais perto de um ponto — por onde o dab entra.
fn perto(mesh: &Mesh, p: [f32; 3]) -> usize {
    (0..mesh.vert_count())
        .min_by(|&a, &b| {
            let d = |i: usize| {
                let q = mesh.positions()[i];
                (0..3).map(|k| (q[k] - p[k]).powi(2)).sum::<f32>()
            };
            d(a).total_cmp(&d(b))
        })
        .expect("a peça não tem vértices")
}

/// ⛔⛔ **UM TRAÇO DE DOIS DABS, e o segundo não é conforto:** o
/// [`Dab::path`] é a diferença entre os centros de dabs CONSECUTIVOS, logo num
/// traço de **um** dab ele é nulo — e o transporte com direcção nula é inerte
/// **por lei** (espec §5.3: com o cursor parado o pincel não faz nada). *Uma
/// régua corrida com um dab só mede o caso degenerado do verbo*, que é a
/// armadilha que o censo dos knobs da crate vizinha já pagou.
fn um_traco(verb: Verb, centro: [f32; 3], path: [f32; 3]) -> (Mesh, Vec<[f32; 3]>) {
    let (mut mesh, antes, brush, mut s) = arma(verb);
    for i in 0..2 {
        let c = [
            centro[0] + path[0] * i as f32,
            centro[1] + path[1] * i as f32,
            centro[2] + path[2] * i as f32,
        ];
        let dab = Dab {
            path,
            ..Dab::at(c, 0.45, c)
        };
        s.dab(&mut mesh, &brush, &dab, Symmetry::default());
    }
    (mesh, antes)
}

/// A peça, as cores de antes, o pincel e o traço aberto.
fn arma(verb: Verb) -> (Mesh, Vec<[f32; 3]>, Brush, SculptStroke) {
    let mesh = peca();
    let antes = mesh.colors().expect("a semente não pintou").to_vec();
    let brush = Brush {
        verb,
        radius: 0.45,
        strength: 1.0,
        ..Brush::default()
    };
    let mut s = SculptStroke::default();
    s.begin(&mesh);
    (mesh, antes, brush, s)
}

/// Um dab no ponto pedido, com o verbo pedido, e as cores de ANTES ao lado.
fn um_dab(verb: Verb, centro: [f32; 3], path: [f32; 3]) -> (Mesh, Vec<[f32; 3]>) {
    let mut mesh = peca();
    let antes = mesh.colors().expect("a semente não pintou").to_vec();
    let brush = Brush {
        verb,
        radius: 0.45,
        strength: 1.0,
        ..Brush::default()
    };
    let mut s = SculptStroke::default();
    s.begin(&mesh);
    let dab = Dab {
        path,
        // ⚠️ O OLHO aponta da peça para fora ao longo do próprio raio: a
        // pegada nasce virada ao artista, senão a máscara de faces recusa-a.
        ..Dab::at(centro, 0.45, centro)
    };
    s.dab(&mut mesh, &brush, &dab, Symmetry::default());
    (mesh, antes)
}

/// ⭐⭐⭐ **O ALVO DO BLUR SAI DAS CORES DE ANTES — a prova de que o buffer é
/// DUPLO**, e é a única régua desta wave que a distingue de um Gauss-Seidel.
///
/// ⚠️ **A régua é GEOMÉTRICA e não um número escolhido:** a lei escreve
/// `out = pre·(1−a) + alvo·a`, logo `out` tem de cair **no segmento** entre a
/// cor de antes do vértice e a média do anel calculada das cores de antes. Com
/// um buffer só, metade da pegada lê o que a outra metade acabou de escrever ⇒
/// o alvo é OUTRO e a saída sai do segmento. *Num canal de três dimensões, cair
/// numa recta é uma coincidência que não acontece por acaso.*
///
/// ⚠️ **E o vértice medido é o que MAIS se moveu**, porque perto da borda da
/// pegada `a ≈ 0` e ali a colinearidade é trivialmente verdadeira — *uma régua
/// lida onde o fenómeno não acontece não afirma nada*.
#[test]
fn o_alvo_do_blur_sai_das_cores_de_antes() {
    let centro = [0.0, 0.0, 1.0];
    let (mesh, antes) = um_dab(Verb::Blur, centro, [0.0, 0.0, 0.0]);
    let depois = mesh.colors().expect("o traço apagou o plano de cor");
    let mut pior = (0.0f32, usize::MAX);
    for v in 0..mesh.vert_count() {
        let d = (0..3)
            .map(|k| (depois[v][k] - antes[v][k]).abs())
            .fold(0.0f32, f32::max);
        if d > pior.0 {
            pior = (d, v);
        }
    }
    let (movimento, v) = pior;
    assert!(
        movimento > 1e-3,
        "o dab não esbateu nada (max {movimento:e}): a fixtura não contém o fenómeno"
    );
    // A média do anel, das cores de ANTES — a conta que o produto faz.
    let mut soma = antes[v];
    let mut n = 1.0f32;
    for &nb in mesh.adjacency().vert_verts.neighbours(v) {
        for k in 0..3 {
            soma[k] += antes[nb as usize][k];
        }
        n += 1.0;
    }
    let alvo = [soma[0] / n, soma[1] / n, soma[2] / n];
    // `out` no segmento: a fracção lida num canal tem de valer nos outros dois.
    let frac = |k: usize| (depois[v][k] - antes[v][k]) / (alvo[k] - antes[v][k]);
    let eixo = (0..3)
        .max_by(|&a, &b| {
            (alvo[a] - antes[v][a])
                .abs()
                .total_cmp(&(alvo[b] - antes[v][b]).abs())
        })
        .expect("três eixos");
    let a = frac(eixo);
    assert!(
        (0.0..=1.0).contains(&a),
        "a fracção saiu da faixa ({a}): a lei não está a interpolar para o anel"
    );
    for k in 0..3 {
        let esperado = antes[v][k] + (alvo[k] - antes[v][k]) * a;
        assert!(
            (depois[v][k] - esperado).abs() < 1e-6,
            "o canal {k} do vértice {v} leu {} contra {esperado} — o alvo NÃO é a \
             média das cores de ANTES (um buffer só lê o que a pegada já escreveu)",
            depois[v][k]
        );
    }
}

/// ⭐⭐⭐ **O TRANSPORTE LÊ SÓ O MONTANTE** — a lei que separa esfregar de
/// borrar, e o sinal é a ferramenta inteira.
///
/// O peso de um vizinho é `g = max(0, −(d̂·ê))`: **só quem está a montante
/// contribui**. Com `g` escrito como `|cos|` os dois lados contribuem e o
/// pincel **borra**.
///
/// ⛔⛔ **A régua é o ALVO e não o barro, e a razão é MEDIDA:** a 1.ª redacção
/// comparava dois TRAÇOS (um para cada lado) e a mutação `|cos|` **SOBREVIVEU**
/// — um traço tem dois dabs, e os dois dabs de uma ida caem noutro sítio que os
/// da volta, logo a comparação media a POSIÇÃO dos carimbos e não a lei. *Uma
/// régua confundida com o enquadramento passa com a lei apagada.*
///
/// ⚠️ **A metade de PRODUTO fica ao lado** (a tinta atravessa a fronteira num
/// traço de verdade): ela não discrimina as duas leis — mede que esta chega ao
/// barro.
#[test]
fn o_transporte_de_cor_e_assimetrico_no_sentido_do_gesto() {
    let mesh = peca();
    let antes = mesh.colors().expect("a semente não pintou");
    let centro = [0.0, 0.0, 1.0];
    let v = perto(&mesh, centro);
    let alvo = |dx: f32| {
        let brush = Brush {
            verb: Verb::SmearColor,
            radius: 0.45,
            strength: 1.0,
            ..Brush::default()
        };
        let dab = Dab {
            path: [dx, 0.0, 0.0],
            ..Dab::at(centro, 0.45, centro)
        };
        SculptStroke::transporte_da_cor(&mesh, &brush, &dab, v as u32)
    };
    let ida = alvo(0.15);
    let volta = alvo(-0.15);
    assert_ne!(
        ida, volta,
        "o alvo é o MESMO nos dois sentidos: a lei está a ler o cosseno em \
         MÓDULO, que é BORRAR — os dois lados da vizinhança a contribuir"
    );
    // O vermelho mora em `x < 0`: arrastar para `+x` põe-no a montante.
    //
    // ⚠️⚠️ **A magnitude mede-se entre os DOIS SENTIDOS e não contra a cor do
    // próprio vértice**, e a 1.ª redacção errou aí: *quão longe o alvo vai*
    // depende de que lado da fronteira o vértice caiu (a semente é uma faixa, e
    // o vértice mais perto do pólo pode estar todo dentro de um dos lados),
    // enquanto *quanto os dois sentidos discordam* é a lei — e é exactamente
    // esta coluna que o `|cos|` leva a ZERO.
    let rubor = |c: [f32; 3]| c[0] - c[2];
    assert!(
        rubor(ida) - rubor(volta) > 1e-2,
        "arrastar para +x devia trazer o VERMELHO (que está a montante) e os dois \
         sentidos entregam {ida:?} contra {volta:?} — a diferença é pequena de \
         mais para ser uma escolha de montante"
    );
    // ⛔⛔ **E O ALVO É UMA MISTURA, NUNCA UMA EXTRAPOLAÇÃO** — a metade que
    // uma mutação SOBREVIVENTE nomeou: tirar a cerca `g <= 0` deixa os vizinhos
    // a JUSANTE entrar com peso NEGATIVO, e um peso negativo é uma extrapolação
    // — a cor sai da faixa da vizinhança e o denominador pode encolher para
    // zero. ⚠️ A assimetria sozinha não o vê: com pesos assinados a saída
    // continua a depender do sentido.
    for c in [ida, volta] {
        for k in 0..3 {
            let mut lo = antes[v][k];
            let mut hi = antes[v][k];
            for &nb in mesh.adjacency().vert_verts.neighbours(v) {
                lo = lo.min(antes[nb as usize][k]);
                hi = hi.max(antes[nb as usize][k]);
            }
            assert!(
                c[k] >= lo - 1e-6 && c[k] <= hi + 1e-6,
                "o alvo {c:?} sai da faixa [{lo}, {hi}] do canal {k}: um peso \
                 negativo entrou na mistura"
            );
        }
    }
    // ⚠️ E o vértice tem de estar na FRONTEIRA, senão as duas leituras são do
    // mesmo lado e a régua mede o enquadramento.
    assert!(
        rubor(antes[v]).abs() > 0.5,
        "a cor de partida do vértice medido é {:?}: a fixtura não o pôs num dos \
         lados da faixa",
        antes[v]
    );
}

/// ⭐ **E A TINTA ATRAVESSA A FRONTEIRA NUM TRAÇO DE VERDADE** — a metade de
/// PRODUTO da lei de cima, pela porta que o gesto percorre.
///
/// ⚠️ **Ela NÃO discrimina esfregar de borrar** (ver o doc do irmão): o que ela
/// afirma é que a lei chega ao barro — *uma lei certa que o produto não chama é
/// uma lei que o artista não tem*.
#[test]
fn um_traco_de_esfregao_leva_tinta_para_o_lado_limpo() {
    let invasao = |m: &Mesh| -> f32 {
        let c = m.colors().expect("plano de cor");
        (0..m.vert_count())
            .filter(|&v| m.positions()[v][0] > 0.0)
            .map(|v| c[v][0] - c[v][2])
            .sum()
    };
    let base = invasao(&peca());
    let depois = invasao(&um_traco(Verb::SmearColor, [0.0, 0.0, 1.0], [0.15, 0.0, 0.0]).0);
    assert!(
        depois - base > 1e-3,
        "o traço não levou vermelho nenhum para o lado azul ({:e})",
        depois - base
    );
}

/// ⛔ **O CONTROLO, e sem ele os dois de cima passam com um pincel que pinta de
/// branco:** numa peça de cor UNIFORME as duas leis devolvem o que já lá
/// estava, **ao bit** — a média de uma vizinhança de uma cor só é essa cor, e o
/// transporte dela também.
///
/// ⚠️ É a mesma frase que a [`crate::canal_de_teste::semeia_cor`] carrega, aqui
/// do lado do PRODUTO: *um corpus no ponto neutro de um canal não testa esse
/// canal*.
#[test]
fn numa_peca_de_cor_uniforme_os_dois_sao_inertes_ao_bit() {
    for verb in [Verb::Blur, Verb::SmearColor] {
        let mut mesh = shapes::uv_sphere(24, 36, 1.0);
        // O plano existe e é uniforme — ⚠️ **não ausente**: com `colors()` a
        // devolver `None` o gate leria «inerte» sobre um caminho que nem corre.
        mesh.colors_mut().fill(DEFAULT_COLOR);
        let antes = mesh.colors().expect("plano").to_vec();
        let brush = Brush {
            verb,
            radius: 0.45,
            strength: 1.0,
            ..Brush::default()
        };
        let mut s = SculptStroke::default();
        s.begin(&mesh);
        // ⛔⛔ **DOIS dabs, e a razão é uma mutação que SOBREVIVEU:** com um
        // dab só o esfregão é inerte **por outro motivo** (o caminho de um
        // traço de um dab é nulo), logo esta metade do controlo passava sem
        // nunca correr a lei que ela existe para medir. *Um controlo que não
        // percorre o mesmo caminho da metade positiva afirma sobre código que
        // não corre.*
        let c = [0.0, 0.0, 1.0];
        for i in 0..2 {
            let onde = [0.15f32.mul_add(i as f32, c[0]), c[1], c[2]];
            let dab = Dab {
                path: [0.15, 0.0, 0.0],
                ..Dab::at(onde, 0.45, onde)
            };
            s.dab(&mut mesh, &brush, &dab, Symmetry::default());
        }
        assert_eq!(
            mesh.colors().expect("plano"),
            &antes[..],
            "{verb:?} mexeu numa peça de cor uniforme"
        );
    }
}
