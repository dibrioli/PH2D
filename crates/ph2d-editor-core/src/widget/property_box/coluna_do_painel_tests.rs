//! Os gates da [`super::ColunaDoPainel`] — cada um nomeia UMA propriedade da lei, com o controlo
//! ao lado (a mesma pergunta respondida pela lei de SECÇÃO, que é o que ela era antes).

use super::super::row::{limites_da_seccao, property_label_col_w_for};
use super::ColunaDoPainel;

/// Uma linha de propriedade como o pintor a pede: `x`, largura, o nome mais largo, o controlo.
#[derive(Clone, Copy)]
struct Linha {
    x: f32,
    w: f32,
    nome: Option<f32>,
    precisa: Option<f32>,
}

const fn linha(x: f32, w: f32, nome: Option<f32>, precisa: Option<f32>) -> Linha {
    Linha {
        x,
        w,
        nome,
        precisa,
    }
}

/// Um quadro: devolve onde o VALOR de cada linha arranca (`x + coluna do nome`).
fn quadro(mem: &mut ColunaDoPainel, linhas: &[Linha]) -> Vec<f32> {
    mem.pintando(|| {
        linhas
            .iter()
            .map(|l| l.x + property_label_col_w_for(l.x, l.w, l.nome, l.precisa))
            .collect()
    })
}

/// O `x` de valor que a lei de SECÇÃO daria a uma linha sozinha — o controlo de todo gate.
fn sozinha(l: Linha) -> f32 {
    l.x + property_label_col_w_for(l.x, l.w, l.nome, l.precisa)
}

/// Três quadros — o painel converge no 3.º (o 1.º aprende a base, o 2.º os pedidos).
fn convergido(mem: &mut ColunaDoPainel, linhas: &[Linha]) -> Vec<f32> {
    quadro(mem, linhas);
    quadro(mem, linhas);
    quadro(mem, linhas)
}

fn todos_iguais(xs: &[f32]) -> bool {
    xs.windows(2).all(|p| (p[0] - p[1]).abs() < 1e-3)
}

/// A lei de sempre, escrita como estava antes da unificação — o ORÁCULO do gate de baixo.
fn antiga(x: f32, w: f32, desired: Option<f32>, control_need: Option<f32>) -> f32 {
    let (usable_w, _) = crate::widget::form_row_columns(x, w, 0.0, 0.0);
    let gap = ph2d_tokens::Spacing::Md.px();
    let control_min = crate::widget::NUMBER_INPUT_MIN_W_PX;
    let metade = w * 0.5 - gap;
    let tecto = (usable_w - gap - control_min).max(0.0);
    let quer = desired.unwrap_or(metade);
    let cede = (usable_w - gap - control_need.unwrap_or(control_min)).max(0.0);
    let base = quer.max(metade);
    let coluna = if cede >= quer { base.min(cede) } else { base };
    coluna.min(tecto).max(0.0)
}

/// ⭐⭐⭐ **Fora de um painel NADA muda — ao bit.** Partir a lei da secção em `(prefere, tecto)`
/// não pode mexer num único píxel de quem pinta fora do `ErasedPanel::paint` (as janelas
/// flutuantes, os arnêses, os gates).
#[test]
fn fora_de_um_painel_a_coluna_e_a_da_seccao_ao_bit() {
    let mut casos = 0usize;
    for w in (40..640).step_by(7).map(|v| v as f32) {
        for desired in std::iter::once(None).chain((0..320).step_by(13).map(|v| Some(v as f32))) {
            for need in std::iter::once(None).chain((0..420).step_by(17).map(|v| Some(v as f32))) {
                assert_eq!(
                    property_label_col_w_for(3.0, w, desired, need).to_bits(),
                    antiga(3.0, w, desired, need).to_bits(),
                    "w={w} desired={desired:?} need={need:?}"
                );
                casos += 1;
            }
        }
    }
    assert!(casos > 30_000, "o varrimento encolheu para {casos} casos");
}

/// ⭐⭐⭐ **Dentro de um painel, secções diferentes arrancam o valor no MESMO `x`.**
#[test]
fn o_valor_de_um_painel_arranca_num_x_so() {
    let linhas = [
        linha(0.0, 300.0, Some(40.0), Some(72.0)),
        linha(0.0, 300.0, Some(160.0), Some(72.0)),
        linha(0.0, 300.0, None, None),
        linha(0.0, 300.0, Some(60.0), Some(147.0)),
    ];
    // O CONTROLO: pela lei de secção estas quatro arrancam em sítios diferentes.
    let fora: Vec<f32> = linhas.iter().map(|l| sozinha(*l)).collect();
    assert!(
        !todos_iguais(&fora),
        "a fixtura não contém o degrau: {fora:?}"
    );
    let dentro = convergido(&mut ColunaDoPainel::default(), &linhas);
    assert!(todos_iguais(&dentro), "o painel não alinhou: {dentro:?}");
}

/// ⭐⭐ **O `x` é o que o nome MAIS LARGO pede, sem apertar o controlo de linha nenhuma.**
///
/// ⛔ A 1.ª redacção tomava o `min` das colunas das secções e o nome caía à METADE de uma linha
/// sem nome largo — `211` nomes cortados no Inspector estreito.
#[test]
fn o_nome_mais_largo_manda_ate_ao_tecto_do_controlo() {
    // ⚠️ A `300 px` a METADE já dá `142` — um nome só pede empréstimo acima dela.
    let linhas = [
        linha(0.0, 300.0, Some(180.0), Some(72.0)),
        linha(0.0, 300.0, None, None),
        linha(0.0, 300.0, Some(40.0), Some(90.0)),
    ];
    let esperado = {
        let lim: Vec<(f32, f32)> = linhas
            .iter()
            .map(|l| limites_da_seccao(l.x, l.w, l.nome, l.precisa))
            .collect();
        let nome = lim.iter().map(|l| l.0).fold(0.0, f32::max);
        let controlo = lim.iter().map(|l| l.1).fold(f32::INFINITY, f32::min);
        nome.min(controlo)
    };
    let dentro = convergido(&mut ColunaDoPainel::default(), &linhas);
    for x in &dentro {
        assert!((x - esperado).abs() < 1e-3, "{x} contra {esperado}");
    }
    // ⚠️ E ele é MAIS LARGO que a metade — a metade é piso, não resposta.
    assert!(
        esperado > sozinha(linhas[1]),
        "a fixtura não põe o nome acima da metade"
    );
}

/// ⭐⭐ **Um cartão recuado o mesmo dos dois lados põe o VALOR no `x` do painel.**
#[test]
fn um_cartao_recuado_alinha_o_valor_com_o_painel() {
    let linhas = [
        linha(0.0, 300.0, Some(170.0), Some(72.0)),
        linha(8.0, 284.0, Some(60.0), Some(72.0)),
        linha(0.0, 300.0, None, None),
    ];
    assert!(
        (sozinha(linhas[1]) - sozinha(linhas[0])).abs() > 1.0,
        "a fixtura não contém o degrau do cartão"
    );
    let dentro = convergido(&mut ColunaDoPainel::default(), &linhas);
    assert!(todos_iguais(&dentro), "o cartão não alinhou: {dentro:?}");
}

/// ⭐⭐ **O cartão só recua quando o campo dele deixaria de caber** — e só ELE recua.
///
/// ⚠️ Duas ordens do dono colidem no fim estreito do dock: tudo alinhado, e nenhum campo abaixo de
/// `72 px`. O tecto de um cartão fica `recuo` abaixo do das linhas de fora, e deixá-lo mandar no
/// painel punha o Inspector estreito a cortar `197` nomes.
#[test]
fn o_cartao_recua_so_quando_o_campo_dele_nao_caberia() {
    let (w, recuo) = (200.0, 6.0);
    let linhas = [
        linha(0.0, w, Some(200.0), Some(72.0)),
        linha(recuo, w - 2.0 * recuo, Some(200.0), Some(72.0)),
    ];
    let dentro = convergido(&mut ColunaDoPainel::default(), &linhas);
    let (fora_cheia, fora_cartao) = (sozinha(linhas[0]), sozinha(linhas[1]));
    // A linha de largura inteira NÃO é arrastada pelo cartão.
    assert!(
        (dentro[0] - fora_cheia).abs() < 1e-3,
        "{} contra {fora_cheia}",
        dentro[0]
    );
    // O cartão fica no tecto DELE — o campo guarda os 72 px.
    assert!(
        (dentro[1] - fora_cartao).abs() < 1e-3,
        "{} contra {fora_cartao}",
        dentro[1]
    );
    assert!(
        dentro[1] < dentro[0],
        "a fixtura não põe o tecto do cartão a morder"
    );
}

/// ⭐ **Uma linha que NÃO é simétrica responde por si** — metade de um par, uma célula de grelha.
#[test]
fn uma_linha_assimetrica_fica_com_a_dela() {
    // ⚠️ `200` e não `140`: a `140` o tecto do par já dava a resposta dele, e a mutação que desliga
    //    o teste de simetria sobrevivia — o `clamp` ao tecto salvava-o. Aqui a resposta dele (a
    //    metade, `92`) fica abaixo do tecto (`106`) e abaixo da coluna do painel.
    let par = linha(0.0, 200.0, Some(30.0), Some(72.0));
    let linhas = [
        linha(0.0, 300.0, Some(170.0), Some(72.0)),
        par,
        linha(0.0, 300.0, None, None),
    ];
    let dentro = convergido(&mut ColunaDoPainel::default(), &linhas);
    assert!(
        (dentro[1] - sozinha(par)).abs() < 1e-3,
        "o par foi puxado: {}",
        dentro[1]
    );
    assert!((dentro[0] - dentro[2]).abs() < 1e-3);
}

/// ⭐ **Uma célula fora da faixa não move a base** — o envelope movia-a `67 px` e punha todas as
/// linhas do quadro seguinte como assimétricas.
#[test]
fn uma_celula_fora_da_faixa_nao_move_a_base() {
    let linhas = [
        linha(0.0, 300.0, Some(90.0), Some(72.0)),
        linha(0.0, 300.0, None, None),
        linha(-67.0, 133.0, None, None),
        linha(0.0, 300.0, Some(40.0), Some(72.0)),
    ];
    let dentro = convergido(&mut ColunaDoPainel::default(), &linhas);
    let cheias = [dentro[0], dentro[1], dentro[3]];
    assert!(todos_iguais(&cheias), "a base moveu-se: {dentro:?}");
    assert!(
        (dentro[2] - sozinha(linhas[2])).abs() < 1e-3,
        "a célula foi puxada"
    );
}

/// ⭐ **Mais linhas DENTRO de cartões do que fora não põem a base num cartão** — a moda escolhia a
/// geometria mais frequente, e no degrau estreito ela é a de um cartão.
#[test]
fn a_base_e_a_linha_mais_larga_que_se_repete() {
    // ⚠️ O nome da linha cheia PEDE EMPRÉSTIMO (`170` > a metade `142`): sem ele toda linha
    //    simétrica arranca no mesmo `x` por construção (a metade é o centro menos o vão), e a
    //    mutação que devolve a base à MODA sobrevivia — medido.
    let mut linhas = vec![
        linha(0.0, 300.0, Some(170.0), Some(72.0)),
        linha(0.0, 300.0, None, None),
    ];
    linhas.extend((0..6).map(|i| linha(8.0, 284.0, Some(40.0 + i as f32), Some(72.0))));
    let dentro = convergido(&mut ColunaDoPainel::default(), &linhas);
    assert!(
        todos_iguais(&dentro),
        "a base foi para o cartão: {dentro:?}"
    );
}

/// ⭐ **A primeira linha pode ser um cartão** — o 1.º quadro erra, o 3.º não; e o 2.º já aprendeu
/// os pedidos de TODAS (medido no Inspector: `1` depois do 1.º quadro, `51` depois do 2.º).
#[test]
fn a_primeira_linha_pode_ser_um_cartao() {
    let linhas = [
        linha(8.0, 284.0, Some(60.0), Some(72.0)),
        linha(0.0, 300.0, Some(130.0), Some(72.0)),
        linha(0.0, 300.0, None, None),
    ];
    let mut mem = ColunaDoPainel::default();
    quadro(&mut mem, &linhas);
    let depois_do_1 = mem.quantos();
    quadro(&mut mem, &linhas);
    assert!(
        mem.quantos() > depois_do_1,
        "o 2.º quadro não aprendeu os pedidos"
    );
    let terceiro = quadro(&mut mem, &linhas);
    assert!(todos_iguais(&terceiro), "{terceiro:?}");
}

/// ⭐ **A primeira linha mudar de ESPÉCIE não desalinha o quadro** (outra selecção no Inspector).
#[test]
fn a_primeira_linha_muda_de_especie_sem_desalinhar() {
    // ⚠️ Empréstimo pela mesma razão do gate da base: sem ele a mutação que tira a folga
    //    simétrica sobrevivia (a metade põe toda linha simétrica no mesmo `x`).
    let cheia = linha(0.0, 300.0, Some(170.0), Some(72.0));
    let cartao = linha(8.0, 284.0, Some(60.0), Some(72.0));
    let mut mem = ColunaDoPainel::default();
    convergido(&mut mem, &[cheia, cartao, cheia]);
    let virado = quadro(&mut mem, &[cartao, cheia, cartao, cheia]);
    assert!(
        todos_iguais(&virado),
        "o quadro em que a primeira linha virou: {virado:?}"
    );
}

/// ⭐ **Arrastar o dock acerta NO MESMO QUADRO** — guardam-se os pedidos, não a coluna.
#[test]
fn arrastar_o_dock_acerta_no_mesmo_quadro() {
    let em = |x: f32, w: f32| {
        [
            linha(x, w, Some(120.0), Some(72.0)),
            linha(x, w, None, None),
            linha(x + 8.0, w - 16.0, Some(50.0), Some(72.0)),
        ]
    };
    let mut mem = ColunaDoPainel::default();
    let antes = convergido(&mut mem, &em(0.0, 300.0));
    let durante = quadro(&mut mem, &em(-40.0, 340.0));
    assert!(
        todos_iguais(&durante),
        "o quadro do arrasto desalinhou: {durante:?}"
    );
    assert!(
        (durante[0] - antes[0]).abs() > 1.0,
        "a fixtura não moveu a coluna"
    );
}

/// ⭐ **A memória só CRESCE** — fechar uma secção não mexe a coluna das outras.
#[test]
fn fechar_uma_seccao_nao_mexe_a_coluna() {
    let larga = linha(0.0, 300.0, Some(190.0), Some(72.0));
    let curta = linha(0.0, 300.0, Some(40.0), Some(72.0));
    let mut mem = ColunaDoPainel::default();
    let aberto = convergido(&mut mem, &[curta, larga, curta]);
    let fechado = quadro(&mut mem, &[curta, curta]);
    assert!(
        (fechado[0] - aberto[0]).abs() < 1e-3,
        "{fechado:?} contra {aberto:?}"
    );
    assert!(
        (aberto[0] - sozinha(curta)).abs() > 1.0,
        "a fixtura não contém o empréstimo"
    );
}
