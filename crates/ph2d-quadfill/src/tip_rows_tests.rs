//! Os gates da [`super`] — e o que eles defendem é a pergunta *«QUAL ponta?»*.
//!
//! ⚠️ **O controlo de cada um é a régua AGREGADA na mesma fixtura**: ela diz *quantas*
//! pontas estão más e nunca *quais*, que é exactamente o buraco que o report do dono de
//! 2026-09-04 abriu (*«muitas pontas boas, uma ruim — muito estranho»*).

use super::tip_rows;
use crate::{TIP_GAP_MAX, tip_density, tip_deviation};
use ph2d_mesh::{Face, Mesh};

/// ⭐⭐⭐ **DOIS espinhos na mesma peça, e `amputa` corta SÓ O DE CIMA** — o fuso.
///
/// ⛔⛔ **A fixtura do cone (a de [`crate::tips`]) não serve a esta pergunta, e a 1.ª
/// redacção deste ficheiro usou-a:** ela tem **um** ápice — o anel da base empata em raio,
/// mas o filtro de forma ([`crate::CONE_MAX`], que chegou em 2026-09-02) lê-o como bossa e
/// deita-o fora. *Com uma ponta só, «a tabela diz qual» é uma afirmação vazia: a única linha
/// é sempre a acusada.* ⇒ aqui a peça tem **duas**, e uma delas fica intacta.
pub(crate) fn cone(amputa: bool) -> Mesh {
    cone_com(amputa, 8, 1.0 / 3.0)
}

/// ⭐⭐⭐ **O MESMO fuso com a AGULHA e a contagem à escolha** — e os dois eixos são
/// load-bearing, cada um para um gate diferente.
///
/// ⚠️ **O leque que fecha um bico tem aspecto `n / (2π k)`**, onde `k` é a inclinação do cone:
/// com `n = 8` e `k = ⅓` (a agulha) cada triângulo do bico abre **`15°`** e conta como face
/// péssima — é a fixtura certa para provar que o remate RECUSA endireitar uma lasca. Com
/// `n = 6` e `k = 0,75` o leque abre `43°` e o bico é uma ponta que a grade sustenta, que é o
/// caso do produto ([`crate::tip_snap`]). *Uma fixtura mais afiada que tudo o que o produto
/// entrega mede a fixtura.*
pub(crate) fn cone_com(amputa: bool, n: u32, k: f32) -> Mesh {
    let big_n = n;
    #[expect(non_snake_case, reason = "a fixtura fala da contagem do anel")]
    let N: u32 = big_n;
    const ALTURA: f32 = 3.0;
    const CORTE: f32 = 2.2;
    // De cima para baixo. O bico cortado troca os três anéis do topo por um só, ao nível do
    // corte, e a tampa fica chata — *o defeito exacto da foto do dono: curto e gordo.*
    let mut aneis: Vec<f32> = if amputa {
        vec![CORTE]
    } else {
        vec![2.9, 2.7, 2.5]
    };
    aneis.extend([
        2.0, 1.5, 1.0, 0.5, 0.0, -0.5, -1.0, -1.5, -2.0, -2.5, -2.7, -2.9,
    ]);
    let aneis = &aneis[..];
    let raio = |z: f32| k * (ALTURA - z.abs());
    let bico = if amputa { CORTE } else { ALTURA };
    let mut verts: Vec<[f32; 3]> = vec![[0.0, 0.0, bico], [0.0, 0.0, -ALTURA]];
    for &z in aneis {
        for k in 0..N {
            #[expect(clippy::cast_precision_loss, reason = "N = 8")]
            let a = core::f32::consts::TAU * k as f32 / N as f32;
            verts.push([raio(z) * a.cos(), raio(z) * a.sin(), z]);
        }
    }
    let anel = |i: u32, k: u32| 2 + i * N + k % N;
    let ultimo = u32::try_from(aneis.len() - 1).expect("poucos aneis");
    let mut faces: Vec<Face> = Vec::new();
    for k in 0..N {
        faces.push(Face::tri(0, anel(0, k), anel(0, k + 1)));
        for i in 0..ultimo {
            faces.push(Face::quad(
                anel(i, k),
                anel(i + 1, k),
                anel(i + 1, k + 1),
                anel(i, k + 1),
            ));
        }
        faces.push(Face::tri(1, anel(ultimo, k + 1), anel(ultimo, k)));
    }
    Mesh::from_parts(verts, faces).expect("a fixtura e' construida aqui")
}

/// ⭐⭐⭐ **GATE — a tabela diz QUAL ponta, e a agregada só diz QUANTAS.**
///
/// ⛔ **É o gate do report de 2026-09-04.** Numa peça com dois espinhos e **um** deles
/// cortado, a [`tip_deviation`] devolve `cut = 1` e não há como saber de qual se trata; a
/// tabela aponta a linha, e o `apex` dela é o vértice `0` — o bico de cima.
#[test]
fn a_tabela_diz_qual_ponta_esta_partida_e_a_agregada_so_diz_quantas() {
    let entrada = cone(false);
    let saida = cone(true);
    let alvo = 0.2;

    // O CONTROLO: a régua que existia responde «uma», e mais nada.
    let agregada = tip_deviation(&entrada, &saida, alvo);
    assert_eq!(agregada.cut, 1, "{agregada:?}");

    let linhas = tip_rows(&entrada, &saida, alvo);
    assert!(
        linhas.len() >= 2,
        "a fixtura tem DOIS espinhos: {}",
        linhas.len()
    );
    let acusadas: Vec<usize> = linhas
        .iter()
        .filter(|r| r.gap > TIP_GAP_MAX)
        .map(|r| r.apex)
        .collect();
    assert_eq!(
        acusadas,
        vec![0],
        "⛔ a linha acusada tem de ser o BICO (vertice 0), e uma so'"
    );
    // ⭐ E as outras não são tocadas — *a régua é por ponta, e não borra o defeito de uma
    // por cima das vizinhas.*
    for r in linhas.iter().filter(|r| r.apex != 0) {
        assert!(
            r.gap <= TIP_GAP_MAX,
            "a ponta {} foi acusada: {r:?}",
            r.apex
        );
    }
}

/// ⭐⭐⭐ **GATE — as duas agregadas são DOBRAS da tabela, e não uma segunda contagem.**
///
/// ⚠️ *Uma régua nova que muda o veredito da anterior não é a mesma régua* — este gate dobra
/// a tabela **aqui**, com a lei escrita à mão, e exige o mesmo número que o produto lê.
#[test]
fn as_agregadas_concordam_com_a_dobra_feita_a_mao() {
    let entrada = cone(false);
    for saida in [cone(false), cone(true)] {
        let alvo = 0.2;
        let linhas = tip_rows(&entrada, &saida, alvo);
        let cortadas = linhas.iter().filter(|r| r.gap > TIP_GAP_MAX).count();
        let pior = linhas.iter().fold(0.0f32, |a, r| a.max(r.gap));
        let d = tip_deviation(&entrada, &saida, alvo);
        assert_eq!(d.cut, cortadas, "{d:?}");
        assert!((d.apex_max - pior).abs() < 1.0e-6, "{d:?} contra {pior}");
        assert_eq!(
            d.tips,
            linhas.iter().filter(|r| r.dev.is_some()).count(),
            "{d:?}"
        );

        let mut graus: Vec<f32> = linhas.iter().filter_map(|r| r.grade).collect();
        graus.sort_by(f32::total_cmp);
        let g = tip_density(&entrada, &saida, alvo);
        assert_eq!(g.tips, graus.len(), "{g:?}");
        if !graus.is_empty() {
            assert!(
                (g.worst - graus[graus.len() - 1]).abs() < 1.0e-6,
                "{g:?} contra {graus:?}"
            );
        }
    }
}

/// ⭐⭐⭐ **GATE — a ponta comida POR INTEIRO marca-se `blind`, e não desaparece.**
///
/// ⚠️ O valor que a linha traz é o **raio da busca**, que é *«mais longe do que eu olhei»* e
/// não a distância verdadeira — é por isso que a coluna existe: sem ela, um `gap` de `3,00`
/// lê-se como uma medida quando é um **piso**.
#[test]
fn a_ponta_comida_por_inteiro_diz_que_e_um_piso() {
    let entrada = cone(false);
    // Alvo pequeno ⇒ o raio de busca (`3 × alvo`) não alcança a tampa do cone cortado.
    let linhas = tip_rows(&entrada, &cone(true), 0.05);
    let bico = linhas
        .iter()
        .find(|r| r.apex == 0)
        .expect("o bico esta' na tabela");
    assert!(
        bico.blind,
        "⛔ sem superficie perto, a linha e' um PISO: {bico:?}"
    );
    assert!(
        (bico.gap - super::DEV_RADIUS).abs() < 1.0e-6,
        "o piso e' o raio da busca: {bico:?}"
    );
    // E o resto da tabela continua a ser medida de verdade.
    assert!(
        linhas.iter().filter(|r| !r.blind).count() >= 1,
        "as pontas sas nao podem ficar cegas: {linhas:?}"
    );
}

/// ⭐⭐ **GATE — «não medido» distingue-se de «perfeito», e a tabela é o sítio onde isso é
/// legível**: sem ápices ela vem **vazia**, e as agregadas dizem `tips = 0`.
#[test]
fn sem_apice_a_tabela_vem_vazia_e_as_agregadas_dizem_que_nao_mediram() {
    let entrada = cone(false);
    // ⛔ `unit` não positiva ⇒ não há cone, e uma lista sem o filtro de forma diria que a
    // peça tem dezenas de pontas — ver [`crate::apices`].
    assert!(tip_rows(&entrada, &entrada, 0.0).is_empty());
    let d = tip_deviation(&entrada, &entrada, 0.0);
    assert_eq!((d.tips, d.cut, d.over), (0, 0, 0), "{d:?}");
    let g = tip_density(&entrada, &entrada, 0.0);
    assert_eq!(g.tips, 0, "{g:?}");
}
