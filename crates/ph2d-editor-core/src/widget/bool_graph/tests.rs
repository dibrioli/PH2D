//! Gates da metade PURA do diagrama da booleana viva.
//!
//! ⚠️ O que estes gates defendem não é "a conta está certa" — é **quem PINTA e quem ACERTA leem o
//! mesmo mapa**. Toda posição sai de `node_center`/`link_points`, e é por isso que os testes de
//! clique constroem o ponto a partir dessas funções em vez de escreverem coordenadas à mão: um
//! número escrito à mão passaria a mentir no dia em que um espaçamento mudasse, e o gate diria
//! que está tudo bem enquanto o artista clica ao lado do que vê.

use super::{
    BoolGraphIntent, BoolGraphLink, BoolGraphNode, BoolGraphView, card_size, drop_intent, link_at,
    link_points, node_at, node_center, node_radius,
};
use crate::zones::Rect;

fn node(id: u64) -> BoolGraphNode {
    BoolGraphNode {
        id,
        label: format!("Forma {id}"),
        consumed: false,
    }
}

/// Três formas em z (1 é a mais ao fundo), sem ligações.
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

/// **O MAIS AO FUNDO DESENHA-SE EM BAIXO** — a única inversão do módulo, e ela tem de estar lá.
///
/// ⚠️ É a costura com a lei: `nodes` chega em ordem de z (fundo → topo), a MESMA que o resolvedor
/// consome. Se a coluna desenhasse na ordem crua, o diagrama diria "esta está por cima" sobre a
/// forma que está por baixo — e a ordem de dobra das ligações que chegam depende exatamente disso.
#[test]
fn o_mais_ao_fundo_desenha_se_em_baixo() {
    let v = view3();
    let r = card(&v);
    let fundo = node_center(r, &v, 0);
    let meio = node_center(r, &v, 1);
    let topo = node_center(r, &v, 2);
    assert!(
        fundo.1 > meio.1 && meio.1 > topo.1,
        "y: fundo {} / meio {} / topo {} -- a coluna não está invertida",
        fundo.1,
        meio.1,
        topo.1
    );
    // E a coluna é uma coluna: todos no mesmo x.
    assert!((fundo.0 - topo.0).abs() < f32::EPSILON);
}

/// **O CLIQUE ACERTA O CÍRCULO QUE SE VÊ** — o ponto do teste vem da MESMA função que o painter usa.
#[test]
fn clicar_no_centro_de_um_circulo_acerta_aquela_forma() {
    let v = view3();
    let r = card(&v);
    for i in 0..v.rows() {
        assert_eq!(node_at(r, &v, node_center(r, &v, i)), Some(i), "forma {i}");
    }
}

/// **O RÓTULO NÃO É ALVO.** Um ponto à direita do círculo, na mesma linha, não acerta nada — é
/// por lá que o arrasto de uma ligação passa, e engoli-lo tornaria o gesto impossível.
#[test]
fn o_rotulo_ao_lado_nao_e_alvo_do_clique() {
    let v = view3();
    let r = card(&v);
    let c = node_center(r, &v, 1);
    let ao_lado = (c.0 + node_radius() * 3.0, c.1);
    assert_eq!(node_at(r, &v, ao_lado), None);
}

/// **UM ARCO COMEÇA E ACABA NOS DOIS CENTROS** — o desenho liga os círculos que diz ligar.
#[test]
fn o_arco_liga_os_dois_centros() {
    let mut v = view3();
    v.links.push(BoolGraphLink {
        from: 3,
        to: 1,
        op: 1,
    });
    let r = card(&v);
    let pts = link_points(r, &v, v.links[0]);
    let a = node_center(r, &v, 2); // id 3 = índice 2
    let b = node_center(r, &v, 0); // id 1 = índice 0
    assert!((pts[0].0 - a.0).abs() < 1e-3 && (pts[0].1 - a.1).abs() < 1e-3);
    let last = *pts.last().unwrap();
    assert!((last.0 - b.0).abs() < 1e-3 && (last.1 - b.1).abs() < 1e-3);
}

/// **O ARCO SAI DA COLUNA** — a barriga vai para a DIREITA, senão ele passaria por cima dos
/// círculos que salta e ficaria inclicável.
#[test]
fn a_barriga_do_arco_sai_para_a_direita_da_coluna() {
    let mut v = view3();
    v.links.push(BoolGraphLink {
        from: 3,
        to: 1,
        op: 0,
    });
    let r = card(&v);
    let coluna = node_center(r, &v, 0).0;
    let pts = link_points(r, &v, v.links[0]);
    let maior_x = pts.iter().fold(f32::MIN, |m, p| m.max(p.0));
    assert!(
        maior_x > coluna + node_radius(),
        "a barriga chegou a {maior_x} e a coluna está em {coluna} -- o arco não saiu"
    );
}

/// **UM ARCO QUE SALTA MAIS FORMAS É MAIS LARGO** — é o que os faz aninhar em vez de se
/// sobreporem, e é a única coisa que distingue duas ligações que partem do mesmo círculo.
#[test]
fn quem_salta_mais_formas_faz_arco_mais_largo() {
    let mut v = BoolGraphView {
        nodes: vec![node(1), node(2), node(3), node(4)],
        links: vec![
            BoolGraphLink {
                from: 2,
                to: 1,
                op: 0,
            },
            BoolGraphLink {
                from: 4,
                to: 1,
                op: 0,
            },
        ],
        cycle: false,
    };
    let r = card(&v);
    let curto = link_points(r, &v, v.links[0])
        .iter()
        .fold(f32::MIN, |m, p| m.max(p.0));
    let longo = link_points(r, &v, v.links[1])
        .iter()
        .fold(f32::MIN, |m, p| m.max(p.0));
    assert!(
        longo > curto,
        "o arco de 3 saltos ({longo}) não é mais largo que o de 1 ({curto})"
    );
    v.links.clear();
}

/// **O CARD CABE O ARCO MAIS LARGO — E A RESERVA É O QUE O FAZ CABER.**
///
/// ⚠️ Este gate JÁ EXISTIU numa forma que não provava nada: com o arco a curvar a partir da
/// coluna, a folga dos rótulos sozinha já o continha, e apagar a reserva do `card_size` deixava-o
/// verde. Um mutante sobrevivente expôs isso. Agora o arco passa **à direita dos rótulos**, então
/// a reserva é a única coisa que o segura — e a segunda metade do gate mede exatamente isso.
#[test]
fn o_card_cabe_o_arco_mais_largo_e_a_reserva_e_o_que_o_segura() {
    let v = BoolGraphView {
        nodes: vec![node(1), node(2), node(3), node(4), node(5)],
        links: vec![BoolGraphLink {
            from: 5,
            to: 1,
            op: 0,
        }],
        cycle: false,
    };
    let r = card(&v);
    let maior_x = link_points(r, &v, v.links[0])
        .iter()
        .fold(f32::MIN, |m, p| m.max(p.0));
    assert!(
        maior_x <= r.x + r.w,
        "o arco chega a {maior_x} e o card acaba em {}",
        r.x + r.w
    );
    // ⚠️ **Controlo positivo:** sem a reserva o arco SAIRIA. Se esta metade falhar, a largura do
    // card é folgada por outro motivo e a primeira metade voltou a ser verde por acidente.
    let sem_ligacao = BoolGraphView {
        links: Vec::new(),
        ..v.clone()
    };
    let (w_sem, _) = card_size(&sem_ligacao);
    assert!(
        maior_x > r.x + w_sem,
        "o arco chega a {maior_x} e o card SEM reserva já acabava em {} -- a reserva não é o que \
         o segura, e o gate não prova nada",
        r.x + w_sem
    );
}

/// **O ARCO NÃO ATRAVESSA OS RÓTULOS.** O nome é como o artista sabe qual círculo é qual; uma
/// ligação riscada por cima dele torna dois círculos indistinguíveis exatamente quando há muitos.
#[test]
fn o_arco_passa_a_direita_dos_rotulos() {
    let v = BoolGraphView {
        nodes: vec![node(1), node(2), node(3), node(4)],
        links: vec![BoolGraphLink {
            from: 4,
            to: 1,
            op: 0,
        }],
        cycle: false,
    };
    let r = card(&v);
    let pts = link_points(r, &v, v.links[0]);
    // O topo do arco (o ponto mais à direita) tem de estar além da coluna de rótulos.
    let topo = pts.iter().fold(f32::MIN, |m, p| m.max(p.0));
    let fim_dos_rotulos = super::label_right(r);
    assert!(
        topo > fim_dos_rotulos,
        "o topo do arco está em {topo} e os rótulos acabam em {fim_dos_rotulos}"
    );
}

/// **CLICAR NO ARCO ACERTA A LIGAÇÃO** — e o ponto vem da mesma amostragem que o painter desenha.
#[test]
fn clicar_no_arco_acerta_a_ligacao() {
    let mut v = view3();
    v.links.push(BoolGraphLink {
        from: 3,
        to: 1,
        op: 1,
    });
    let r = card(&v);
    let pts = link_points(r, &v, v.links[0]);
    let meio = pts[pts.len() / 2];
    assert_eq!(link_at(r, &v, meio), Some(0));
    // E longe dele não acerta nada.
    assert_eq!(link_at(r, &v, (meio.0 + 200.0, meio.1)), None);
}

/// **QUANDO DOIS ARCOS SE CRUZAM, GANHA O MAIS PRÓXIMO** — nunca o primeiro da lista.
///
/// ⚠️ A ordem em `links` é de armazenamento e não diz nada ao artista. Um acerto por "o primeiro
/// que passa no raio" faria o mesmo pixel selecionar coisas diferentes conforme a ordem em que as
/// ligações foram criadas — e essa ordem é invisível.
#[test]
fn quando_dois_arcos_se_cruzam_ganha_o_mais_proximo() {
    // Duas ligações que CHEGAM ao mesmo círculo: perto dele os dois arcos convergem, e é aí que a
    // desambiguação de facto acontece. Arcos que não se aproximam não testam nada.
    let v = BoolGraphView {
        nodes: vec![node(1), node(2), node(3)],
        links: vec![
            BoolGraphLink {
                from: 3,
                to: 1,
                op: 0,
            },
            BoolGraphLink {
                from: 2,
                to: 1,
                op: 0,
            },
        ],
        cycle: false,
    };
    let r = card(&v);
    // Um ponto sobre o SEGUNDO arco, perto do destino comum.
    let pts = link_points(r, &v, v.links[1]);
    let sobre = pts[pts.len() - 3];

    // ⚠️ **Controlo positivo, e é ele que dá o gate.** Sem provar que o ponto está ao alcance dos
    // DOIS, "ganha o mais próximo" e "ganha o primeiro da lista" respondem igual, e o gate fica
    // verde sobre um acerto que nunca teve de escolher. Foi assim que um mutante sobreviveu.
    let dist = |i: usize| {
        link_points(r, &v, v.links[i])
            .iter()
            .fold(f32::INFINITY, |m, q| {
                let (dx, dy) = (sobre.0 - q.0, sobre.1 - q.1);
                m.min(dx.mul_add(dx, dy * dy))
            })
            .sqrt()
    };
    let (d0, d1) = (dist(0), dist(1));
    assert!(
        d0 <= super::LINK_GRAB,
        "o outro arco está a {d0} px e a folga é {} -- o ponto não é ambíguo, e o gate não tem \
         nada para desempatar",
        super::LINK_GRAB
    );
    assert!(d1 < d0, "o segundo arco ({d1}) não é o mais próximo ({d0})");

    assert_eq!(
        link_at(r, &v, sobre),
        Some(1),
        "ganhou o primeiro da lista em vez do mais próximo"
    );
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

/// **A FAIXA DE AVISO SÓ OCUPA ALTURA QUANDO HÁ AVISO** — o card não reserva espaço para um
/// estado que quase nunca acontece.
#[test]
fn o_aviso_de_ciclo_so_ocupa_altura_quando_existe() {
    let calmo = view3();
    let mut com_ciclo = view3();
    com_ciclo.cycle = true;
    let (w0, h0) = card_size(&calmo);
    let (w1, h1) = card_size(&com_ciclo);
    assert!((w0 - w1).abs() < f32::EPSILON, "a largura não muda");
    assert!(h1 > h0, "o aviso não abriu espaço: {h0} -> {h1}");
}

/// **UMA VISTA VAZIA AINDA TEM CARD** — abrir o diagrama sobre nada não pode dar um card de altura
/// zero (ele fica sem banda de título e o artista não consegue fechá-lo).
#[test]
fn uma_vista_vazia_ainda_tem_card_clicavel() {
    let v = BoolGraphView::default();
    let (w, h) = card_size(&v);
    assert!(w > 0.0 && h > super::TITLE_H, "card {w}x{h}");
}
