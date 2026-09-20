//! ⭐⭐⭐⭐ **A RÉGUA DAS MANCHAS PRETAS** — a que separa *«a cor parte na CPU»*
//! de *«a cor parte a caminho do device»*, e que até 2026-09-20 não existia.
//!
//! # O report
//!
//! Dono, 2026-09-20, com foto: pintar com a **topologia dinâmica armada** deixa
//! **faces INTEIRAS a preto, de aresta dura**, dentro da zona pintada — e
//! *«acontece em painter, blur e Smear»*, ou seja nos três verbos que escrevem
//! o canal de cor.
//!
//! # ⛔⛔ Porque o gate que já existia não podia vê-lo
//!
//! O `ph2d-mesh :: the_new_vertices_carry_colour_and_mask` **promete a cor no
//! NOME e mede-a pelo COMPRIMENTO**:
//!
//! ```text
//! let colors = m.colors().expect("a cor também");
//! assert_eq!(colors.len(), m.vert_count());
//! ```
//!
//! A máscara, ao lado, tem asserção de VALOR (`(v − 0,25).abs() < 1e-5`). A cor
//! tem o comprimento e mais nada — e a fixtura dela é um xadrez por índice par
//! (`m_pos_z(i) = i.is_multiple_of(2)`), que é um campo sem valor esperado:
//! *ela foi construída de um jeito que torna a asserção de valor impossível de
//! escrever*. ⇒ um refino que fizesse todo vértice novo nascer PRETO deixa
//! aquele gate **verde**.
//!
//! ⚠️ **E a nota do `CLAUDE.md` herdou a promessa do nome** — ela dá o refino
//! como *«gateado e provavelmente ilibado»* e manda procurar no colapso. É a
//! forma que esta casa já regista: *um gate cujo nome promete duas metades e
//! cujo corpo mede uma é lido pelo nome.*
//!
//! # A lei que a régua usa, e porque ela não precisa de um valor esperado
//!
//! ⭐⭐⭐ **A COR UNIFORME É PONTO FIXO DE TODO O PASSE.** Se a peça inteira vale
//! `C` e o pincel deposita `C`, então:
//!
//! | operação | conta | resultado |
//! |---|---|---|
//! | refino (`mesh_splice`) | `(C + C)·0,5` | `C` |
//! | colapso (`mesh_shrink`, a média do merge) | `(C + C)·0,5` | `C` |
//! | colapso (a permutação do [`ph2d_mesh::Remap`]) | move `C` de casa | `C` |
//! | depósito (`Paint.js:129`) | `C·(1 − f) + C·f` | `C` |
//!
//! ⇒ **todo vértice tem de ler `C` no fim, seja qual for a topologia que o
//! passe produziu.** A régua não precisa de saber quantos vértices nasceram,
//! nem de onde, nem para onde foram — e é isso que a torna imune à cadeia de
//! renumeração que o §27 desta linha pagou: *uma permutação de valores iguais é
//! invisível, logo o que ela mede é só o que INVENTA um valor.*
//!
//! ⚠️ **E o VALOR do desvio nomeia a causa**, que é o que uma barra sozinha não
//! faz: `[0,0,0]` é um slot que nasceu no default errado ou que ninguém
//! escreveu; [`ph2d_mesh::DEFAULT_COLOR`] é o plano inteiro recriado; qualquer
//! outro é mistura ou permutação com um vizinho.
//!
//! # ⚠️ A folga é RUÍDO de `f32` e não tolerância de gosto
//!
//! `C·(1 − f) + C·f` só é `C` ao bit quando `(1 − f) + f` fecha em `1,0`; fora
//! disso o erro é de meia ULP por dab. A barra é `1e-5`, que é ~`10²` acima do
//! ruído acumulado de um traço e ~`10⁵` abaixo do defeito que se procura (uma
//! mancha preta sobre `C = 0,8` desvia `0,8`).
//!
//! # ⚠️ Ela percorre a ROTA e não as funções soltas
//!
//! A ordem aqui é a do [`super::Sculpt3dScene::refine_for_dab`], costura
//! incluída: [`passe_nos_motores`] → `shrink_with` no colapso → `grow_with` no
//! refino → `dab`. ⛔ O arnês do censo dos knobs chama `begin` entre os dabs, o
//! que **recongela o `base_color` a cada carimbo** — para aquele censo é
//! inofensivo (ele compara duas corridas do mesmo arnês), e aqui apagaria
//! exactamente o estado que o defeito poderia corromper. *Um gate que chama a
//! função em vez de percorrer a rota afirma que as leis existem, nunca que o
//! produto as usa* (§24 desta linha).

use ph2d_mesh::{DEFAULT_COLOR, Mesh, shapes::uv_sphere};
use ph2d_sculpt3d::{Brush, Dab, SculptStroke, Symmetry, Verb};

use super::{Rascunho, passe_nos_motores};

/// A cor da fixtura — longe do PRETO (o sintoma) e longe do
/// [`DEFAULT_COLOR`] (o outro suspeito), para que o valor do desvio os separe.
const C: [f32; 3] = [0.8, 0.35, 0.15];

/// A barra, em unidades do canal. Ver o cabeçalho.
const RUIDO: f32 = 1e-5;

/// Quantos carimbos o traço tem. ⚠️ **Um dab só não chega:** o colapso come
/// arestas CURTAS, e num carimbo a malha ainda não tem nenhuma — *uma régua de
/// topologia corrida com um dab mede só o refino*.
const DABS: usize = 6;

/// A peça, pintada inteira de `C`.
fn peca() -> Mesh {
    let mut m = uv_sphere(24, 36, 1.0);
    m.colors_mut().fill(C);
    m
}

/// O pincel do report, com a cor da fixtura.
fn pincel(verb: Verb) -> Brush {
    Brush {
        verb,
        radius: 0.45,
        strength: 1.0,
        color: C,
        ..Brush::default()
    }
}

/// **UM TRAÇO PELA ROTA DO PRODUTO** — devolve a malha e a contagem de antes.
///
/// `passe` diz se a topologia dinâmica está armada: com `false` esta função é o
/// CONTROLO, e é o par das duas leituras que atribui o defeito.
fn traco(verb: Verb, passe: bool) -> (Mesh, usize) {
    let mut mesh = peca();
    let antes = mesh.vert_count();
    let brush = pincel(verb);
    let mut s = SculptStroke::default();
    // ⚠️ **O pen-down TRIANGULA**, como o `open_dyntopo_stroke` do produto: os
    // dois motores recusam um quad por geometria, e sem isto o passe seria um
    // no-op silencioso numa malha de quads.
    if passe {
        mesh.triangulate();
    }
    s.begin(&mesh);
    let (mut remap, mut births, mut region) = (
        ph2d_mesh::Remap::default(),
        Vec::new(),
        ph2d_mesh::RegionScratch::default(),
    );
    for k in 0..DABS {
        // O centro ANDA sobre a calota — é a diferença entre centros que vira o
        // [`Dab::path`], e sem ela o Smear é inerte por lei.
        let t = 0.06 * k as f32;
        let c = [t, 0.0, (1.0 - t * t).max(0.0).sqrt()];
        if passe {
            let alvo = ph2d_mesh::edge_target_for_mesh(&mesh, 1.0);
            let (cut, done, _) = passe_nos_motores(
                &mut mesh,
                verb,
                alvo,
                c,
                brush.radius,
                Rascunho {
                    remap: &mut remap,
                    births: &mut births,
                    region: &mut region,
                },
                // Sem pente: ele mudaria a topologia dos DOIS lados da
                // comparação, e o que está sob teste é o par colapso/refino.
                None,
            );
            // ⚠️ **A costura, na ordem do produto.** Ela é o que mantém o
            // `base_color` a descrever a malha de agora — e é precisamente aqui
            // que uma renumeração mal aplicada poria a cor de partida de um
            // vértice no slot de outro.
            if cut {
                s.shrink_with(&remap);
            }
            if done {
                s.grow_with(&mesh, &births);
            }
        }
        let dab = Dab {
            path: [0.06, 0.0, 0.0],
            ..Dab::at(c, brush.radius, c)
        };
        s.dab(&mut mesh, &brush, &dab, Symmetry::default());
    }
    (mesh, antes)
}

/// O pior desvio de `C`, com o vértice e o valor que o produziu.
fn pior(mesh: &Mesh) -> (f32, usize, [f32; 3]) {
    let cores = mesh
        .colors()
        .expect("o traço apagou o plano de cor inteiro");
    assert_eq!(
        cores.len(),
        mesh.vert_count(),
        "o plano de cor deixou de medir a malha: {} cores para {} vértices",
        cores.len(),
        mesh.vert_count()
    );
    // ⛔⛔ **O `NaN` VEM PRIMEIRO, e a primeira redacção desta função era CEGA a
    // ele** — pela mesma aritmética que ela existe para medir: `d > pior.0` com
    // `d = NaN` é **sempre falso**, logo um máximo por comparação SALTA todo
    // `NaN` e devolve `0,0` sobre uma peça inteiramente envenenada. *Uma régua
    // escrita para achar um valor inventado não pode usar a ordem dos `f32`
    // para o achar, porque o valor que ela procura não está nessa ordem.*
    if let Some((v, c)) = cores
        .iter()
        .enumerate()
        .find(|(_, c)| c.iter().any(|x| x.is_nan()))
    {
        return (f32::INFINITY, v, *c);
    }
    let mut pior = (0.0f32, usize::MAX, C);
    for (v, c) in cores.iter().enumerate() {
        let d = (0..3).map(|k| (c[k] - C[k]).abs()).fold(0.0f32, f32::max);
        if d > pior.0 {
            pior = (d, v, *c);
        }
    }
    pior
}

/// O nome do valor que a régua encontrou — é isto que transforma um número numa
/// pista.
fn diagnostico(c: [f32; 3]) -> &'static str {
    let perto = |a: [f32; 3]| (0..3).all(|k| (c[k] - a[k]).abs() < 1e-4);
    if c.iter().any(|x| x.is_nan()) {
        "NaN — o canal foi ENVENENADO: um NaN num canto contamina a \
         interpolação da FACE inteira, e é isso que se lê como mancha preta \
         de aresta dura"
    } else if perto([0.0; 3]) {
        "PRETO EXACTO — um slot que nasceu no default errado ou que ninguém escreveu"
    } else if perto(DEFAULT_COLOR) {
        "DEFAULT_COLOR — o plano de cor foi recriado, não permutado"
    } else {
        "nem preto nem o default — mistura ou permutação com um vizinho"
    }
}

/// ⭐⭐⭐ **A COR UNIFORME É PONTO FIXO DO PASSE.**
///
/// As três metades são necessárias: sem o **piso de população** um
/// `assert!(desvio < barra)` ficaria verde num arranjo em que o passe nunca
/// dispara — *uma régua que não vê o fenómeno acontecer não prova que ele não
/// aconteceu* —, e sem o **controlo** um desvio não diria se a culpa é do passe
/// ou da lei de cor, que é a pergunta inteira do report.
#[test]
fn a_cor_uniforme_e_ponto_fixo_do_passe() {
    for verb in [Verb::Paint, Verb::Blur, Verb::SmearColor] {
        // ── O CONTROLO: o mesmo traço, sem topologia. ──
        let (limpo, _) = traco(verb, false);
        let (d0, _, _) = pior(&limpo);
        assert!(
            d0 <= RUIDO,
            "{verb:?}: a cor uniforme já não é ponto fixo SEM o passe (Δ{d0:e}) — \
             o defeito está na lei de cor, não na topologia"
        );

        // ── A RÉGUA. ──
        let (mesh, antes) = traco(verb, true);
        let depois = mesh.vert_count();
        assert!(
            depois != antes,
            "{verb:?}: o passe não mudou a topologia ({antes} -> {depois}) — \
             a fixtura não contém o fenómeno e a asserção abaixo não afirma nada"
        );
        let (d, v, cor) = pior(&mesh);
        assert!(
            d <= RUIDO,
            "{verb:?}: o passe INVENTOU cor. O vértice {v} de {depois} lê \
             {cor:?} onde a peça inteira e o pincel valem {C:?} (Δ{d:e}). \
             Diagnóstico pelo valor: {}. \
             A topologia foi de {antes} para {depois} vértices.",
            diagnostico(cor)
        );
    }
}

/// ⭐⭐ **E NENHUMA COR É INVENTADA SOBRE UM CAMPO NÃO-UNIFORME** — a metade que
/// a fixtura uniforme é CEGA a ver.
///
/// ⚠️ **Uma permutação de valores iguais é invisível**, logo o gate acima não
/// distingue *«a renumeração está certa»* de *«a renumeração está errada e
/// todos os valores são iguais»*. Aqui a peça tem duas cores, e a lei é que toda
/// saída caia no ENVELOPE por canal das cores que existiam ∪ a do pincel: toda
/// operação do passe é uma média ou uma mudança de casa, e **nem uma média nem
/// uma mudança de casa saem do envelope do que entrou**.
///
/// ⛔ Ela não substitui a de cima: um passe que zerasse a cor de todo vértice
/// novo NUMA PEÇA PRETA passaria aqui e reprovaria lá. As duas medem a mesma
/// família por lados opostos.
#[test]
fn o_passe_nunca_inventa_uma_cor_fora_do_envelope() {
    const QUENTE: [f32; 3] = [0.9, 0.1, 0.1];
    const FRIA: [f32; 3] = [0.1, 0.2, 0.9];

    for verb in [Verb::Paint, Verb::Blur, Verb::SmearColor] {
        let mut mesh = uv_sphere(24, 36, 1.0);
        let xs: Vec<f32> = mesh.positions().iter().map(|p| p[0]).collect();
        for (c, x) in mesh.colors_mut().iter_mut().zip(&xs) {
            *c = if *x < 0.0 { FRIA } else { QUENTE };
        }
        let antes = mesh.vert_count();
        let brush = pincel(verb);
        let mut s = SculptStroke::default();
        mesh.triangulate();
        s.begin(&mesh);
        let (mut remap, mut births, mut region) = (
            ph2d_mesh::Remap::default(),
            Vec::new(),
            ph2d_mesh::RegionScratch::default(),
        );
        for k in 0..DABS {
            let t = 0.06 * k as f32;
            let c = [t, 0.0, (1.0 - t * t).max(0.0).sqrt()];
            let alvo = ph2d_mesh::edge_target_for_mesh(&mesh, 1.0);
            let (cut, done, _) = passe_nos_motores(
                &mut mesh,
                verb,
                alvo,
                c,
                brush.radius,
                Rascunho {
                    remap: &mut remap,
                    births: &mut births,
                    region: &mut region,
                },
                None,
            );
            if cut {
                s.shrink_with(&remap);
            }
            if done {
                s.grow_with(&mesh, &births);
            }
            let dab = Dab {
                path: [0.06, 0.0, 0.0],
                ..Dab::at(c, brush.radius, c)
            };
            s.dab(&mut mesh, &brush, &dab, Symmetry::default());
        }
        let depois = mesh.vert_count();
        assert!(
            depois != antes,
            "{verb:?}: o passe não mudou a topologia ({antes} -> {depois})"
        );
        // O envelope: o mínimo e o máximo por canal do que ENTROU na peça.
        let lo = |k: usize| QUENTE[k].min(FRIA[k]).min(C[k]) - RUIDO;
        let hi = |k: usize| QUENTE[k].max(FRIA[k]).max(C[k]) + RUIDO;
        let cores = mesh.colors().expect("o traço apagou o plano de cor");
        for (v, c) in cores.iter().enumerate() {
            for k in 0..3 {
                assert!(
                    c[k] >= lo(k) && c[k] <= hi(k),
                    "{verb:?}: o vértice {v} de {depois} lê {c:?}, e o canal {k} \
                     está fora do envelope [{:.4}, {:.4}] do que entrou na peça. \
                     Diagnóstico pelo valor: {}. \
                     A topologia foi de {antes} para {depois} vértices.",
                    lo(k),
                    hi(k),
                    diagnostico(*c)
                );
            }
        }
    }
}
