//! Os gates do colisor DECLARADO — a ordem do dono de 2026-09-17 (doc 114 §12).
//!
//! ⚠️ **O primeiro é o mais importante e é o mais fácil de esquecer:** toda cena que já existe
//! não declara colisor nenhum, e tem de sair **AO BIT** como saía. Uma feature nova que muda em
//! silêncio o que já shipava não é uma feature nova, é uma regressão com nome bonito.

use crate::{declarado, push_apart};
use ph2d_nodegraph::attr::{
    COLLIDER_BOX_COLUMN, COLLIDER_COLUMN, COLLIDER_OFFSET_COLUMN, Column, Stream,
};

/// Duas peças a `d` uma da outra no eixo `x`, sem declaração nenhuma.
fn duas(d: f32) -> Stream {
    Stream::new(2).with("P", Column::Vec2(vec![[0.0, 0.0], [d, 0.0]]))
}

/// A mesma corrente, com uma CAIXA declarada por peça (meias extensões em geometria).
fn duas_com_caixa(d: f32, meia: [f32; 2]) -> Stream {
    duas(d)
        .with(COLLIDER_BOX_COLUMN, Column::Vec2(vec![meia, meia]))
        .with(COLLIDER_OFFSET_COLUMN, Column::Vec2(vec![[0.0, 0.0]; 2]))
}

fn pos(s: &Stream) -> Vec<[f32; 2]> {
    match s.get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    }
}

fn separa(
    s: &Stream,
    raios: &[f32],
    falloff: &[f32],
    strength: f32,
) -> Option<declarado::Separado> {
    let pesos = vec![1.0f32; s.count()];
    let p = pos(s);
    declarado::separa(s, &p, &pesos, raios, falloff, 8, strength)
}

/// ⭐⭐⭐ **SEM DECLARAÇÃO, A PORTA NEM ABRE** — e é isto que mantém o catálogo inteiro ao bit.
///
/// ⛔ O gate mede a PORTA a devolver `None`, e não «a saída é parecida»: uma saída parecida
/// passaria com o caminho novo a correr e a dar quase o mesmo.
#[test]
fn sem_colisor_declarado_a_porta_devolve_nada() {
    let s = duas(0.1);
    assert!(
        separa(&s, &[0.3, 0.3], &[1.0, 1.0], 1.0).is_none(),
        "uma corrente sem colunas de colisor nao pode entrar no caminho novo"
    );
}

/// ⭐⭐⭐ **UMA CAIXA NÃO É UM DISCO** — o pedido do dono, num número.
///
/// Duas caixas ALTAS (meia-extensão `[0,2 · 1,0]`) lado a lado separam-se até `0,4` — a largura
/// delas. O disco que o cartão pediria (`radius = 0,3`) pararia em `0,6`. *A forma declarada
/// decide, e o número do cartão não.*
///
/// ⛔⛔ **A 1.ª redacção deste gate usava caixas COMPRIDAS e esperava `2,0`, e estava errada:** o
/// teorema do eixo separador empurra pelo **caminho mais curto**, não pelo lado maior. Duas caixas
/// `2,0 × 0,2` sobrepostas a `0,1` em `x` têm `1,9` de sobreposição em `x` e `0,2` em `y` ⇒ elas
/// saem **verticalmente**, e o vão em `x` fica onde estava (`0,101`, medido). *Foi o gate que me
/// ensinou a lei do motor, e a fixtura é que fazia a pergunta errada.*
#[test]
fn a_caixa_declarada_separa_pela_forma_dela_e_nao_pelo_raio_do_cartao() {
    let s = duas_com_caixa(0.1, [0.2, 1.0]);
    let d = separa(&s, &[0.3, 0.3], &[1.0, 1.0], 1.0).expect("a caixa declara");
    let vao = (d.pos[1][0] - d.pos[0][0]).abs();
    assert!(
        (vao - 0.4).abs() < 0.05,
        "duas caixas de meia-largura 0,2 tem de separar ate' ~0,4 e separaram {vao}"
    );
    // ⚠️ **O CONTROLO é o disco do cartão na MESMA fixtura**, senão `2,0` não separa «a caixa
    // ganhou» de «o motor novo separa mais».
    let disco = duas(0.1);
    let p = pos(&disco);
    let sem = push_apart(&p, &[1.0, 1.0], &[0.3, 0.3], &[1.0, 1.0], 8, 1.0);
    let vao_disco = (sem[1][0] - sem[0][0]).abs();
    assert!(
        (vao_disco - 0.6).abs() < 0.05,
        "o disco do cartao separa ate' ~0,6 e separou {vao_disco}"
    );
}

/// ⭐⭐ **UMA PEÇA SEM DECLARAÇÃO CAI NO `Radius`, e não fica inerte** (a decisão 2).
///
/// ⛔⛔ **Sem isto uma corrente MISTA deixa metade das peças caladas**, e um nó que separa umas e
/// não outras é pior que um que não separa nenhuma.
#[test]
fn numa_corrente_mista_a_peca_sem_declaracao_usa_o_raio_do_cartao() {
    // A peça 0 declara uma caixa; a 1 não declara nada (meia extensão nula ⇒ `declarado` diz não).
    let s = duas(0.1)
        .with(
            COLLIDER_BOX_COLUMN,
            Column::Vec2(vec![[0.5, 0.5], [0.0, 0.0]]),
        )
        .with(COLLIDER_OFFSET_COLUMN, Column::Vec2(vec![[0.0, 0.0]; 2]));
    let d = separa(&s, &[0.3, 0.3], &[1.0, 1.0], 1.0).expect("uma delas declara");
    let vao = (d.pos[1][0] - d.pos[0][0]).abs();
    assert!(
        vao > 0.5,
        "a peca sem declaracao ficou INERTE — as duas acabaram a {vao} uma da outra"
    );
}

/// ⛔⛔⛔ **O `Strength` NÃO É INERTE no caminho novo — e esta é a armadilha que a wave existiu
/// para não repetir.**
///
/// Escalar os pesos das duas peças de um par pelo `strength` faz o factor **cancelar** na
/// aritmética do PBD (`λ = pen/(k_a+k_b)`, `Δp = n·λ·w`), e o knob deixaria de responder. A mistura
/// no fim é o que o mantém vivo, e este gate mede-o no BARRO: metade da força, metade do vão.
#[test]
fn o_strength_responde_no_caminho_declarado() {
    let s = duas_com_caixa(0.1, [0.5, 0.5]);
    let cheia = separa(&s, &[0.3, 0.3], &[1.0, 1.0], 1.0).expect("declara");
    let meia = separa(&s, &[0.3, 0.3], &[1.0, 1.0], 0.5).expect("declara");
    let zero = separa(&s, &[0.3, 0.3], &[1.0, 1.0], 0.0).expect("declara");
    let (vc, vm, vz) = (
        (cheia.pos[1][0] - cheia.pos[0][0]).abs(),
        (meia.pos[1][0] - meia.pos[0][0]).abs(),
        (zero.pos[1][0] - zero.pos[0][0]).abs(),
    );
    assert!(
        (vz - 0.1).abs() < 1e-5,
        "`strength = 0` tem de deixar as pecas onde estavam, e o vao ficou {vz}"
    );
    // A meio curso o deslocamento tem de ser METADE do deslocamento da força cheia.
    let (dc, dm) = (vc - 0.1, vm - 0.1);
    assert!(
        dc > 0.1 && (dm / dc - 0.5).abs() < 0.02,
        "meia forca tem de mover metade: cheia {dc}, meia {dm}"
    );
}

/// ⭐ **E o `falloff` cala UMA peça sem calar a outra.**
#[test]
fn o_falloff_a_zero_prende_a_peca_no_caminho_declarado() {
    let s = duas_com_caixa(0.1, [0.5, 0.5]);
    let d = separa(&s, &[0.3, 0.3], &[0.0, 1.0], 1.0).expect("declara");
    assert!(
        (d.pos[0][0] - 0.0).abs() < 1e-5,
        "a peca de falloff zero moveu-se para {:?}",
        d.pos[0]
    );
    assert!(
        d.pos[1][0] > 0.2,
        "a outra peca tinha de continuar a ser empurrada, e foi para {:?}",
        d.pos[1]
    );
}

/// ⚠️ **O `rot` só nasce quando alguma peça de facto RODOU.**
///
/// ⛔ Escrevê-lo sempre acrescenta uma coluna a uma corrente que não a tinha, e um consumidor a
/// jusante que distingue *«sem rotação»* de *«rotação zero»* muda de resposta em silêncio.
#[test]
fn duas_caixas_de_frente_nao_inventam_uma_coluna_de_rotacao() {
    let s = duas_com_caixa(0.1, [0.5, 0.5]);
    let d = separa(&s, &[0.3, 0.3], &[1.0, 1.0], 1.0).expect("declara");
    assert!(
        d.rot.iter().all(|r| *r == 0.0),
        "duas caixas alinhadas de frente nao tem binario: {:?}",
        d.rot
    );
}

/// ⚠️⚠️ **A prova do NÓ INTEIRO não mora aqui, e isso é uma decisão.** Os gates acima medem a
/// PORTA; um `eval` que nunca a chamasse passava-os todos — a quarta vez que esta casa paga a
/// diferença entre a função e o fio. A prova pela rota do produto (`source.shape(Collide) →
/// motion.clone → motion.collide`, cozida com as membranas) vive no `ph2d-app-motion`, que é onde
/// há um registo e um cozedor; ali ela é o `o_botao_collide_da_forma_separa_as_copias`.
///
/// ⚠️ A coluna do RAIO declarado também é lida (o `Collider Shape = Circle` da forma).
#[test]
fn a_forma_pode_declarar_um_disco_em_vez_de_uma_caixa() {
    let s = duas(0.1)
        .with(COLLIDER_COLUMN, Column::Scalar(vec![0.8, 0.8]))
        .with(COLLIDER_OFFSET_COLUMN, Column::Vec2(vec![[0.0, 0.0]; 2]));
    let d = separa(&s, &[0.3, 0.3], &[1.0, 1.0], 1.0).expect("o disco declara");
    let vao = (d.pos[1][0] - d.pos[0][0]).abs();
    assert!(
        (vao - 1.6).abs() < 0.05,
        "dois discos de raio 0,8 separam ate' ~1,6 e separaram {vao}"
    );
}

/// ⭐⭐⭐ **O PASSE AUTOMÁTICO E ESTE NÓ CONCORDAM NO `falloff`** (doc 115 §15.1) — e é este gate
/// que torna a W6 uma correcção e não uma lei nova.
///
/// A W5 tinha recusado o `falloff` ao passe por um **erro de categoria**: ela pôs o `Strength` (um
/// param de CARTÃO, que um passe não tem) e o `falloff` (uma COLUNA da corrente, que ~50 nós
/// escrevem) na mesma frase, e concluiu que nenhum dos dois podia lá estar. ⇒ o passe honra-o
/// agora, e a barra é **este nó**, não um número escolhido.
///
/// ⚠️ **Mede-se em TRÊS pontos do knob**, e cada um apanha um defeito diferente: `0` (o interruptor
/// — o §10.4), `0,5` (a MISTURA — um passe que tratasse o falloff como booleano passaria nos outros
/// dois) e `1` (o neutro — que tem de ser byte-idêntico ao que o passe já fazia).
///
/// ⚠️ E o `strength` do nó vale **`1`** aqui de propósito: é o valor que o passe tem por não ter
/// cartão, logo é a única célula em que os dois PODEM concordar.
#[test]
fn o_passe_automatico_concorda_com_este_no_em_todo_o_curso_do_falloff() {
    const MEIA: [f32; 2] = [0.5, 0.5];
    for k in [0.0f32, 0.5, 1.0] {
        // A corrente que o passe lê traz a atenuação como COLUNA; o nó recebe-a como argumento.
        let base = duas_com_caixa(0.5, MEIA).with("size", Column::Vec2(vec![[1.0, 1.0]; 2]));
        let do_no = separa(&base, &[0.0, 0.0], &[k, k], 1.0)
            .unwrap_or_else(|| panic!("o no' separa (k = {k})"));

        let com_coluna = base.clone().with("falloff", Column::Scalar(vec![k, k]));
        let do_passe = ph2d_contact::passe::separa_o_que_se_desenha(&com_coluna, 8);

        if k == 0.0 {
            // ⭐ Atenuação total: o passe não escreve corrente nenhuma (nada se mexeu), e o nó
            // devolve as posições de partida. As duas leituras dizem a MESMA coisa.
            assert!(
                do_passe.is_none(),
                "com `falloff = 0` nada se mexe, logo o passe nao escreve corrente nova"
            );
            for (i, q) in do_no.pos.iter().enumerate() {
                assert!(
                    (q[0] - pos(&base)[i][0]).abs() < 1e-6,
                    "e o no' tambem deixa a peca {i} onde estava"
                );
            }
            continue;
        }
        let do_passe = do_passe.unwrap_or_else(|| panic!("o passe separa (k = {k})"));
        let p = pos(&do_passe);
        for (i, (passe, no)) in p.iter().zip(&do_no.pos).enumerate() {
            assert!(
                (passe[0] - no[0]).abs() < 1e-6,
                "k = {k}, peca {i}: o passe deu {} e o no' deu {} — as duas leis divergiram",
                passe[0],
                no[0]
            );
        }
    }
}
