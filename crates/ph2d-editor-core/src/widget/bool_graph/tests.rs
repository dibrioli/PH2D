//! Gates da metade PURA do diagrama da booleana viva.
//!
//! ⚠️ O que estes gates defendem não é "a conta está certa" — é **quem PINTA e quem ACERTA leem o
//! mesmo mapa**. Toda posição sai de `node_center`/`link_points`, e é por isso que os testes de
//! clique constroem o ponto a partir dessas funções em vez de escreverem coordenadas à mão: um
//! número escrito à mão passaria a mentir no dia em que um espaçamento mudasse, e o gate diria
//! que está tudo bem enquanto o artista clica ao lado do que vê.

use super::{
    BoolGraphIntent, BoolGraphLink, BoolGraphNode, BoolGraphView, NodeZone, canvas_rect, card_size,
    clamp_to_plane, drop_intent, link_at, link_points, node_at, node_center, node_radius,
    ring_inner_radius,
};
use crate::zones::Rect;

fn node(id: u64) -> BoolGraphNode {
    BoolGraphNode {
        id,
        label: format!("Forma {id}"),
        consumed: false,
        at: None,
    }
}

/// Três formas sem posição guardada — elas caem no anel default.
fn view3() -> BoolGraphView {
    BoolGraphView {
        nodes: vec![node(1), node(2), node(3)],
        links: Vec::new(),
        cycle: false,
    }
}

fn card(view: &BoolGraphView) -> Rect {
    let (w, h) = card_size(view);
    Rect {
        x: 100.0,
        y: 50.0,
        w,
        h,
    }
}

/// **SEM POSIÇÃO GUARDADA, O ANEL ARRUMA — E NINGUÉM SE SOBREPÕE.**
///
/// ⚠️ É o que faz abrir a janela pela primeira vez mostrar algo legível. Sem o anel, todas as
/// formas nasceriam no mesmo ponto e o artista teria de as separar antes de poder ler o diagrama.
#[test]
fn sem_posicao_guardada_o_anel_arruma_sem_sobreposicao() {
    let v = view3();
    let r = card(&v);
    let cs: Vec<(f32, f32)> = (0..v.rows()).map(|i| node_center(r, &v, i)).collect();
    for i in 0..cs.len() {
        for j in (i + 1)..cs.len() {
            let d = (cs[i].0 - cs[j].0).hypot(cs[i].1 - cs[j].1);
            assert!(
                d > node_radius() * 2.0,
                "as formas {i} e {j} sobrepõem-se (distância {d})"
            );
        }
    }
    // E todas ficam dentro do plano.
    let plane = canvas_rect(r);
    for (i, c) in cs.iter().enumerate() {
        assert!(
            c.0 - node_radius() >= plane.x - 0.5 && c.0 + node_radius() <= plane.x + plane.w + 0.5,
            "a forma {i} saiu do plano em x"
        );
    }
}

/// **A POSIÇÃO GUARDADA MANDA SOBRE O ANEL.** É o gesto de arrastar a ter efeito.
#[test]
fn a_posicao_guardada_manda_sobre_o_anel() {
    let mut v = view3();
    v.nodes[1].at = Some([200.0, 120.0]);
    let r = card(&v);
    let plane = canvas_rect(r);
    assert_eq!(
        node_center(r, &v, 1),
        (plane.x + 200.0, plane.y + 120.0),
        "o anel ganhou de uma posição autorada"
    );
    // As outras continuam no anel.
    assert_ne!(node_center(r, &v, 0), (plane.x + 200.0, plane.y + 120.0));
}

/// **O NÚMERO DE Z É A ORDEM DA LISTA, COMEÇANDO EM 1.**
///
/// ⚠️ É o que sobrou da coluna, e é o essencial dela: quando várias ligações chegam ao mesmo nó,
/// elas dobram na ordem de z de quem opera. Sem este número no plano livre, o resultado dependeria
/// de uma coisa que o diagrama não mostra.
#[test]
fn o_numero_de_z_e_a_ordem_da_lista() {
    assert_eq!(BoolGraphNode::z_badge(0), 1, "o mais ao FUNDO é o número 1");
    assert_eq!(BoolGraphNode::z_badge(2), 3);
}

/// **O CÍRCULO TEM DUAS ZONAS, E ELAS SÃO GESTOS DIFERENTES.**
///
/// ⚠️ Sem a separação, arrastar para mover e arrastar para ligar seriam o mesmo gesto — e um deles
/// teria de ganhar, deixando o outro inexprimível.
#[test]
fn o_miolo_e_o_aro_sao_zonas_diferentes() {
    let v = view3();
    let r = card(&v);
    let c = node_center(r, &v, 0);
    assert_eq!(
        node_at(r, &v, c),
        Some((0, NodeZone::Core)),
        "o centro é miolo"
    );
    let no_aro = (c.0 + (ring_inner_radius() + node_radius()) * 0.5, c.1);
    assert_eq!(
        node_at(r, &v, no_aro),
        Some((0, NodeZone::Ring)),
        "a banda é aro"
    );
    // E fora do círculo não é nada.
    let fora = (c.0 + node_radius() * 2.0, c.1);
    assert_eq!(node_at(r, &v, fora), None);
}

/// **O TRAÇO VAI DE BORDA A BORDA, NUNCA DE CENTRO A CENTRO.**
///
/// ⚠️ Uma linha que entra no círculo passaria por cima do nome que está lá dentro — e o nome é como
/// o artista sabe qual círculo é qual.
#[test]
fn o_traco_para_na_borda_dos_circulos() {
    let mut v = view3();
    v.links.push(BoolGraphLink {
        from: 3,
        to: 1,
        op: 1,
    });
    let r = card(&v);
    let pts = link_points(r, &v, v.links[0]);
    let a = node_center(r, &v, 2);
    let b = node_center(r, &v, 0);
    let d_a = (pts[0].0 - a.0).hypot(pts[0].1 - a.1);
    let d_b = (pts[1].0 - b.0).hypot(pts[1].1 - b.1);
    assert!(
        d_a > node_radius() * 0.9 && d_b > node_radius() * 0.9,
        "o traço entrou no círculo (distâncias {d_a} / {d_b}, raio {})",
        node_radius()
    );
}

/// **`A→B` E `B→A` SÃO DUAS LINHAS, NÃO UMA RISCADA DUAS VEZES.**
///
/// ⚠️ Sem o deslocamento lateral, um par que opera nos dois sentidos seria indistinguível de um que
/// opera num só — e clicar nele acertaria sempre a mesma das duas.
#[test]
fn os_dois_sentidos_de_um_par_sao_duas_linhas() {
    let mut v = view3();
    v.links.push(BoolGraphLink {
        from: 1,
        to: 2,
        op: 0,
    });
    v.links.push(BoolGraphLink {
        from: 2,
        to: 1,
        op: 1,
    });
    let r = card(&v);
    let ida = link_points(r, &v, v.links[0]);
    let volta = link_points(r, &v, v.links[1]);
    // O ponto médio de cada uma tem de estar separado do outro.
    let m = |p: &[(f32, f32)]| (p[0].0.midpoint(p[1].0), p[0].1.midpoint(p[1].1));
    let (a, b) = (m(&ida), m(&volta));
    let d = (a.0 - b.0).hypot(a.1 - b.1);
    assert!(d > 1.0, "as duas linhas do par coincidem (distância {d})");
}

/// **CLICAR NUM TRAÇO ACERTA A LIGAÇÃO** — e o ponto vem da mesma geometria que o painter desenha.
#[test]
fn clicar_no_traco_acerta_a_ligacao() {
    let mut v = view3();
    v.links.push(BoolGraphLink {
        from: 3,
        to: 1,
        op: 1,
    });
    let r = card(&v);
    let pts = link_points(r, &v, v.links[0]);
    let meio = (pts[0].0.midpoint(pts[1].0), pts[0].1.midpoint(pts[1].1));
    assert_eq!(link_at(r, &v, meio), Some(0));
    assert_eq!(link_at(r, &v, (meio.0, meio.1 + 200.0)), None);
}

/// **QUANDO DOIS TRAÇOS SE CRUZAM, GANHA O MAIS PRÓXIMO** — nunca o primeiro da lista.
///
/// ⚠️ A ordem em `links` é de armazenamento e não diz nada ao artista. Um acerto por *"o primeiro
/// que passa no raio"* faria o mesmo pixel selecionar coisas diferentes conforme a ordem em que as
/// ligações foram criadas — e essa ordem é invisível.
#[test]
fn quando_dois_tracos_se_cruzam_ganha_o_mais_proximo() {
    // Quatro formas no anel: 1 em cima, 2 à direita, 3 em baixo, 4 à esquerda. As duas DIAGONAIS
    // (1→3 e 2→4) cruzam-se perto do centro — é ali que a desambiguação de facto acontece.
    //
    // ⚠️ Os dois sentidos do MESMO par NÃO servem: eles ficam a `2 × LINK_OFFSET = 14` px um do
    // outro e a folga do clique é 9, então nenhum ponto está ao alcance dos dois. O controlo
    // positivo abaixo apanhou exatamente isso na primeira escrita deste gate.
    let v = BoolGraphView {
        nodes: vec![node(1), node(2), node(3), node(4)],
        links: vec![
            BoolGraphLink {
                from: 1,
                to: 3,
                op: 0,
            },
            BoolGraphLink {
                from: 2,
                to: 4,
                op: 0,
            },
        ],
        cycle: false,
    };
    let r = card(&v);
    let a = link_points(r, &v, v.links[0]);
    let b = link_points(r, &v, v.links[1]);

    // Procura, ao longo do SEGUNDO traço, um ponto que esteja ao alcance dos DOIS. ⚠️ Sem provar
    // que ele existe, "ganha o mais próximo" e "ganha o primeiro da lista" respondem igual, e o
    // gate fica verde sobre um acerto que nunca teve de escolher.
    /// Quantas amostras ao longo do traço a busca examina. CONTAGEM, não pixel.
    const SAMPLES: u16 = 100;
    let ambiguo = (0..=SAMPLES)
        .map(|k| {
            #[allow(clippy::cast_precision_loss)] // índice de amostra, não medida
            let t = f32::from(k) / f32::from(SAMPLES);
            (
                (b[1].0 - b[0].0).mul_add(t, b[0].0),
                (b[1].1 - b[0].1).mul_add(t, b[0].1),
            )
        })
        .find(|p| {
            let d0 = super::dist_to_segment(*p, a[0], a[1]);
            d0 > 0.5 && d0 <= super::LINK_GRAB
        })
        .expect("os dois traços têm de se cruzar -- senão este gate não desempata nada");

    assert_eq!(
        link_at(r, &v, ambiguo),
        Some(1),
        "ganhou o primeiro da lista em vez do mais próximo"
    );
}

/// **UM CÍRCULO ARRASTADO PARA FORA É PRESO AO PLANO.**
///
/// ⚠️ É o que impede o gesto de criar um estado irreversível: um círculo largado fora do card
/// ficaria fora do alcance do ponteiro para sempre.
#[test]
fn um_circulo_arrastado_para_fora_e_preso_ao_plano() {
    let v = view3();
    let r = card(&v);
    let plane = canvas_rect(r);
    let longe = clamp_to_plane(r, (plane.x - 5000.0, plane.y + 5000.0));
    assert!(longe[0] >= node_radius(), "saiu pela esquerda: {longe:?}");
    assert!(
        longe[1] <= plane.h - node_radius() + 0.5,
        "saiu por baixo: {longe:?}"
    );
    // E um ponto legítimo passa sem ser mexido.
    let dentro = clamp_to_plane(r, (plane.x + 200.0, plane.y + 100.0));
    assert_eq!(dentro, [200.0, 100.0]);
}

/// **O CARD CRESCE PARA CABER O CÍRCULO MAIS DISTANTE.**
///
/// ⚠️ Sem o crescimento, arrastar um círculo para longe o poria fora do card — e o gesto teria
/// criado um estado de que não se pode voltar.
#[test]
fn o_card_cresce_para_caber_o_circulo_mais_distante() {
    let base = card_size(&view3());
    let mut v = view3();
    v.nodes[0].at = Some([base.0 + 300.0, 100.0]);
    let maior = card_size(&v);
    assert!(
        maior.0 > base.0,
        "a largura não cresceu: {base:?} -> {maior:?}"
    );
    // E o círculo cabe.
    let r = Rect::new(0.0, 0.0, maior.0, maior.1);
    let c = node_center(r, &v, 0);
    assert!(c.0 + node_radius() <= r.x + r.w, "o círculo saiu do card");
}

/// **O LAÇO DE UM NÓ CONSIGO MESMO É RECUSADO NO GESTO** — não só no resolvedor.
///
/// ⚠️ Um gesto que produz uma recusa é um gesto que não devia ter sido aceite: aceitá-lo faria a
/// arte inteira parar de cozinhar (o ciclo recusa o grafo todo) por causa de um arrasto que
/// escorregou de volta para o círculo de partida.
#[test]
fn soltar_no_mesmo_circulo_nao_e_uma_ligacao() {
    let v = view3();
    assert_eq!(drop_intent(&v, 2, 2, 0), None);
    assert_eq!(
        drop_intent(&v, 2, 1, 0),
        Some(BoolGraphIntent::Link {
            from: 2,
            to: 1,
            op: 0
        })
    );
}

/// **SOLTAR NUMA FORMA QUE NÃO ESTÁ NO DIAGRAMA NÃO É UMA LIGAÇÃO.**
#[test]
fn soltar_fora_do_diagrama_nao_e_uma_ligacao() {
    let v = view3();
    assert_eq!(drop_intent(&v, 2, 99, 0), None);
    assert_eq!(drop_intent(&v, 99, 2, 0), None);
}

/// **UMA VISTA VAZIA AINDA TEM CARD** — abrir o diagrama sobre nada não pode dar um card sem banda
/// de título, que o artista não conseguiria fechar.
#[test]
fn uma_vista_vazia_ainda_tem_card_clicavel() {
    let v = BoolGraphView::default();
    let (w, h) = card_size(&v);
    assert!(
        w > 0.0 && h > super::TITLE_H + super::FOOTER_H,
        "card {w}x{h}"
    );
}
