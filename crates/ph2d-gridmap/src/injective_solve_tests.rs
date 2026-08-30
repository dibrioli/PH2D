//! ⭐⭐⭐ **OS GATES DA RESOLUÇÃO INJECTIVA** — e o que eles defendem é a **mudança de
//! variável**: *a costura tem de sair exacta, e o mapa tem de ficar melhor.*

use super::{InjectiveReport, make_injective};
use crate::cut::CutMesh;
use crate::solve::{GridMap, Step};
use ph2d_mesh::{Face, Mesh};
use ph2d_untangle::Settings;

/// **DOIS retalhos** ligados por uma costura, com a malha plana e o mapa igual à geometria.
///
/// ⚠️ **Dois e não um, de propósito:** com um retalho só não há costura nenhuma, e o gate que
/// importa — *a costura sai exacta* — ficaria vácuo.
fn dois_retalhos() -> (Mesh, CutMesh, GridMap, crate::weld::Weld) {
    // Uma faixa `3 × 2` de vértices, partida ao meio em dois retalhos que partilham a coluna
    // do meio (duplicada).
    let mut pos: Vec<[f32; 3]> = Vec::new();
    for j in 0..2 {
        for i in 0..3 {
            #[expect(
                clippy::cast_precision_loss,
                reason = "indices pequenos convertidos para posicao"
            )]
            pos.push([i as f32, j as f32, 0.0]);
        }
    }
    // globais: 0 1 2 / 3 4 5   (linha de baixo, linha de cima)
    let faces = vec![
        Face::tri(0, 1, 4),
        Face::tri(0, 4, 3),
        Face::tri(1, 2, 5),
        Face::tri(1, 5, 4),
    ];
    let mesh = Mesh::from_parts(pos.clone(), faces).expect("a fixtura e' construida aqui");

    // Retalho 0: globais 0,1,3,4 (locais 0,1,2,3) · Retalho 1: globais 1,2,4,5.
    let cut = CutMesh {
        origin: vec![vec![0, 1, 3, 4], vec![1, 2, 4, 5]],
        tris: vec![vec![[0, 1, 3], [0, 3, 2]], vec![[0, 1, 3], [0, 3, 2]]],
        tri_face: vec![vec![0, 1], vec![2, 3]],
        seams: Vec::new(),
    };
    let uv0: Vec<[f32; 2]> = vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0]];
    let uv1: Vec<[f32; 2]> = vec![[1.0, 0.0], [2.0, 0.0], [1.0, 1.0], [2.0, 1.0]];
    let map = GridMap {
        uv: vec![uv0, uv1],
        shift: Vec::new(),
    };
    // ⚠️ A `Weld` desta fixtura é construída pela porta real, para o gate medir a lei e não
    // uma cópia dela.
    let combed = crate::comb::Combed::default();
    let (w, _) = crate::weld::weld(&cut, &combed);
    (mesh, cut, map, w)
}

/// ⭐⭐⭐ **GATE — a costura sai EXACTA, e é a mudança de variável que a garante.**
///
/// ⛔ **É a razão de esta obra existir.** A sonda que a antecedeu impunha a costura por
/// **projecção** e estagnava a oscilar, porque a projecção deitava fora o trabalho da descida.
/// Aqui a costura é a **variável**: cada cópia é `R^k · raiz + t` com `k` e `t` constantes, e
/// não há nada a desfazer.
#[test]
fn a_costura_sai_exacta_porque_ela_e_a_variavel() {
    let (mesh, cut, mut map, w) = dois_retalhos();
    // Dobra o retalho 0 movendo um canto para o outro lado.
    map.uv[0][3] = [-1.0, -1.0];
    let rep = make_injective(
        &mesh,
        &cut,
        &w,
        &mut map,
        Step::uniform(1.0),
        Settings::default(),
    );
    // ⛔ O CONTROLE vem do próprio relatório: a fixtura tem de ter contido o fenómeno, senão
    // este gate ficaria verde sobre um mapa que nunca esteve dobrado.
    assert!(
        rep.flipped_before > 0,
        "⛔ a fixtura tem de conter dobras, senao este gate nao prova nada"
    );
    assert!(
        rep.flipped_after < rep.flipped_before,
        "⛔ tem de melhorar: {} -> {}",
        rep.flipped_before,
        rep.flipped_after
    );

    // ⭐⭐ E a costura: toda cópia de uma classe tem de continuar a ser a imagem da raiz.
    for c in 0..w.classes() {
        let raiz = w.value_pub(&map, c);
        for ((p, l), k) in w.members_pub(c) {
            let esperado = match k.rem_euclid(4) {
                1 => [-raiz[1], raiz[0]],
                2 => [-raiz[0], -raiz[1]],
                3 => [raiz[1], -raiz[0]],
                _ => raiz,
            };
            let tem = map.uv[p as usize][l as usize];
            assert!(
                (tem[0] - esperado[0]).abs() < 1e-4 && (tem[1] - esperado[1]).abs() < 1e-4,
                "⛔ a copia ({p},{l}) da classe {c} deixou de ser a imagem da raiz: {tem:?} \
                 contra {esperado:?}"
            );
        }
    }
}

/// ⭐⭐ **GATE — um mapa SEM dobras sai BYTE A BYTE igual.**
///
/// ⛔ *Esta obra não existe para melhorar um mapa bom* — e sem esta cerca ela mexeria toda peça
/// limpa do corpus, e todo golden desta cadeia mudaria de valor sem ninguém ter pedido.
#[test]
fn um_mapa_sem_dobras_sai_byte_a_byte_igual() {
    let (mesh, cut, mut map, w) = dois_retalhos();
    let antes = map.uv.clone();
    let rep = make_injective(
        &mesh,
        &cut,
        &w,
        &mut map,
        Step::uniform(1.0),
        Settings::default(),
    );
    assert_eq!(rep.flipped_before, 0);
    assert_eq!(rep.outer, 0, "⛔ nem uma iteracao: nao havia nada a fazer");
    assert!(map.uv == antes, "⛔ o mapa limpo tem de sair intacto");
}

/// ⭐⭐⭐ **GATE — a energia NÃO TEM OPINIÃO SOBRE A DENSIDADE.**
///
/// ⛔⛔⛔ **É o gate do defeito que o A/B ponta a ponta de 2026-08-30 apanhou** (tabela em
/// [`super::element_of`]): com o repouso em unidades do **mundo**, o termo `g(J)` puxava o mapa
/// para `det J = 1` em área do mundo, a peça saía com `2,3×` os quads pedidos e o enviesamento
/// mediano ia de `6,4°` para `30,3°`.
///
/// ⭐⭐ **A régua é a INVARIÂNCIA, não um número:** ampliar a malha `s×` **e** o passo `s×` é a
/// mesma pergunta em outras unidades, logo tem de dar o **mesmo `uv`**. *Um gate que fixasse um
/// valor teria de ser reescrito à mão a cada mudança da energia; este só se parte se a escala
/// voltar a entrar na conta.*
///
/// ⛔ **E ele tem CONTROLE:** o mesmo ensaio com o passo **não** escalado tem de divergir —
/// senão o gate estaria a medir uma função que ignora o argumento que ele testa.
#[test]
fn a_energia_nao_tem_opiniao_sobre_a_densidade() {
    // ⚠️ **POTÊNCIA DE DOIS, e é load-bearing.** Com `s = 7` este gate reprova, e não por
    // defeito do produto: `√(49a)` e `7·√a` diferem por um ULP, e uma descida a partir de um
    // estado **emaranhado** é caótica — um ULP no repouso vira `0,15` na saída. Com `s = 2^k`
    // o achatamento é **exactamente** escalado (a raiz de `4^k·a` só desloca o expoente), o
    // referencial de repouso sai bit a bit igual, e o gate volta a medir a LEI em vez da
    // aritmética. *Uma invariância exacta exige uma transformação exacta.*
    let s = 8.0f32;
    let corre = |escala: f32, passo: f32| {
        let (mesh, cut, mut map, w) = dois_retalhos();
        let pos: Vec<[f32; 3]> = mesh
            .positions()
            .iter()
            .map(|p| [p[0] * escala, p[1] * escala, p[2] * escala])
            .collect();
        let faces: Vec<Face> = mesh.faces().to_vec();
        let grande = Mesh::from_parts(pos, faces).expect("a fixtura escalada");
        map.uv[0][3] = [-1.0, -1.0];
        // ⚠️ O relatório não interessa aqui: o que este gate lê é o MAPA.
        let _ = make_injective(
            &grande,
            &cut,
            &w,
            &mut map,
            Step::uniform(passo),
            Settings::default(),
        );
        map.uv
    };
    let base = corre(1.0, 1.0);
    let ampliada = corre(s, s);
    for (p, (a, b)) in base.iter().zip(ampliada.iter()).enumerate() {
        for (l, (x, y)) in a.iter().zip(b.iter()).enumerate() {
            assert!(
                (x[0] - y[0]).abs() < 1e-3 && (x[1] - y[1]).abs() < 1e-3,
                "⛔ a copia ({p},{l}) mudou com a ESCALA: {x:?} contra {y:?} -- o repouso \
                 deixou de estar em unidades de celula"
            );
        }
    }
    // ⛔ O CONTROLE: com o passo por escalar, a mesma malha ampliada tem de dar OUTRA coisa.
    let torta = corre(s, 1.0);
    let mudou = base.iter().zip(torta.iter()).any(|(a, b)| {
        a.iter()
            .zip(b.iter())
            .any(|(x, y)| (x[0] - y[0]).abs() > 1e-3 || (x[1] - y[1]).abs() > 1e-3)
    });
    assert!(
        mudou,
        "⛔ sem escalar o passo o resultado tem de MUDAR -- senao este gate nao mede o passo"
    );
}

/// ⭐⭐ **GATE — só escreve de volta o que MELHOROU.**
///
/// ⚠️ A descida nunca sobe (a busca linear recusa), mas o retorno a `f32` pode. *Um passe que
/// só pode ajudar tem de o provar na régua do consumidor, e a régua dele é `f32`.*
#[test]
fn nao_escreve_de_volta_o_que_nao_melhorou() {
    let src = include_str!("injective_solve.rs");
    let at = src
        .find("if rep.flipped_after < rep.flipped_before {")
        .expect("⛔ a guarda do «so' escreve o que melhorou» desapareceu");
    assert!(
        src[at..at + 400].contains("w.set(map, c,"),
        "⛔ a escrita de volta tem de estar DENTRO da guarda, e passar pela porta da costura"
    );
}

/// ⭐⭐⭐ **GATE — nasce DESLIGADO, e a tabela da recusa fica ao lado do interruptor.**
///
/// ⛔ *Um default invertido em silêncio* é o que este gate impede, e *um default sem razão
/// escrita* é o que a segunda metade impede. Quem ligar isto tem de vir aqui, e a medição está
/// à distância de zero saltos. ⚠️ Os números vivem no doc de [`super::enabled`], não aqui — um
/// gate que os **repetisse** seria a segunda cópia a divergir da primeira.
#[test]
fn nasce_desligado_e_a_tabela_da_recusa_esta_ao_lado() {
    assert!(
        std::env::var("PH2D_GRIDMAP_INJECTIVE").is_err(),
        "⛔ este gate mede o DEFAULT; corre-o sem a env posta"
    );
    assert!(!super::enabled());

    let src = include_str!("injective_solve.rs");
    // ⚠️ Os DOIS lados da medição: o que ela entrega (o mapa zera) e o que ela custa (o
    // produto piora). Guardar só um faria a nota mentir por omissão.
    for numero in [
        "`0`",
        "`352 ms`",
        "`21,3°`",
        "`1 191`",
        "`415`",
        "`0,061 %`",
    ] {
        assert!(
            src.contains(numero),
            "⛔ a tabela da recusa perdeu {numero} -- o default ficaria sem razao escrita"
        );
    }
}

/// ⭐⭐⭐ **GATE — a porta do PRODUTO está atrás da env, e a chamada é a real.**
///
/// ⛔ Sem isto o [`nasce_desligado`] mediria só a função `enabled`, que ninguém é obrigado a
/// chamar. *Um interruptor que o caminho do produto não consulta é um interruptor decorativo* —
/// e a diferença lê-se no fonte, porque a alternativa (correr a cadeia inteira duas vezes) é
/// cara e não distingue «não ligou» de «ligou e não mudou nada».
#[test]
fn a_porta_do_produto_esta_atras_da_env() {
    let src = include_str!("weld_round.rs");
    let at = src
        .find("crate::injective_solve::enabled()")
        .expect("⛔ o caminho do produto deixou de consultar o interruptor");
    let bloco = &src[at..(at + 400).min(src.len())];
    assert!(
        bloco.contains("make_injective("),
        "⛔ a chamada tem de estar DENTRO da guarda"
    );
    // ⚠️ E ela tem de correr sobre o mapa **contínuo**: a escada constrói o relaxador dela
    // depois, e passar a estar do outro lado inverteria a fase que esta obra cura.
    let escada = src
        .find("WeldRelaxer::new(")
        .expect("⛔ a escada mudou de nome -- reconfira a ordem das fases");
    assert!(
        at < escada,
        "⛔ a resolucao injectiva tem de correr ANTES da escada: {at} contra {escada}"
    );
}

/// ⭐⭐ **GATE — o relatório distingue «não havia» de «não consegui».**
#[test]
fn o_relatorio_distingue_nao_havia_de_nao_consegui() {
    let vazio = InjectiveReport::default();
    assert!(!vazio.gave_up, "⛔ zero dobras nao e' desistencia");
    let (mesh, cut, mut map, w) = dois_retalhos();
    map.uv[0][3] = [-1.0, -1.0];
    let rep = make_injective(
        &mesh,
        &cut,
        &w,
        &mut map,
        Step::uniform(1.0),
        Settings {
            max_outer: 1,
            max_inner: 1,
            ..Settings::default()
        },
    );
    // ⚠️ Com um orçamento de UMA iteração ele pode não fechar — e o que o gate exige é que ele
    // **diga**, não que consiga.
    assert_eq!(
        rep.gave_up,
        rep.flipped_after > 0,
        "⛔ `gave_up` tem de ser exactamente «sobraram dobras»"
    );
}
