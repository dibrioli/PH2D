use super::*;
use std::time::Instant;

fn pede(ativos: &[Produtor], focada: bool) -> Pedido {
    quer_quadro(&Estado { ativos, focada })
}

/// G2 (a lei): cada produtor contínuo, sozinho, pede quadro já com a janela em foco; desfocada,
/// Documento e Isento continuam já e só a UI desce a 10 q/s.
#[test]
fn cada_produtor_continuo_pede_quadro() {
    for p in Produtor::TODOS {
        assert_eq!(pede(&[p], true), Pedido::Ja(p), "{p:?} em foco");
        let desfocada = match p.classe() {
            Classe::Documento | Classe::Isento => Pedido::Ja(p),
            Classe::Ui => Pedido::Ate {
                prazo: DESFOCADA,
                porque: Parada::Desfocada,
            },
        };
        assert_eq!(pede(&[p], false), desfocada, "{p:?} desfocada");
    }
}

/// ⛔ G3 (a lei): o papel molhado e o traço parado seguram o laço contra a UI desfocada à frente
/// na lista — o byte da secagem depende do passo, não pode dormir.
#[test]
fn o_papel_molhado_e_o_traco_parado_seguram_o_laco() {
    for doc in [Produtor::PapelMolhado, Produtor::TracoAberto] {
        for focada in [true, false] {
            assert_eq!(
                pede(&[Produtor::Toast, Produtor::Entrada, doc], focada),
                Pedido::Ja(doc)
            );
        }
    }
}

/// G9: sem ninguém, o laço dorme o [`PRAZO`] com a parada nomeada; nenhum sono passa do prazo.
#[test]
fn o_sono_e_parada_nomeada() {
    for focada in [true, false] {
        assert_eq!(
            pede(&[], focada),
            Pedido::Ate {
                prazo: PRAZO,
                porque: Parada::Ocioso
            }
        );
    }
    assert!(DESFOCADA > Duration::ZERO && DESFOCADA <= PRAZO);
    assert_eq!(Produtor::TODOS.len(), 14);
}

/// O laço: a entrada segura o quadro só o [`PRAZO`]; depois dorme com a parada nomeada.
#[test]
fn a_entrada_segura_o_laco_so_o_prazo() {
    let t0 = Instant::now();
    let mut l = Laco::novo(false);
    l.entrada(t0);
    let antes = t0 + PRAZO / 2;
    assert_eq!(l.pedido(&[], antes), Pedido::Ja(Produtor::Entrada));
    assert_eq!(l.acorda(&[], antes), None);
    let depois = t0 + PRAZO * 2;
    assert_eq!(l.acorda(&[], depois), Some(depois + PRAZO));
    l.foco(false);
    assert_eq!(l.acorda(&[], antes), Some(antes + DESFOCADA));
    assert_eq!(
        l.pedido(&[Produtor::PapelMolhado], depois),
        Pedido::Ja(Produtor::PapelMolhado)
    );
}

/// Um smoke (que conta quadros) nunca dorme, nem desfocado: `PH2D_EXIT_AFTER_FRAMES` não pendura.
#[test]
fn o_smoke_nao_dorme_nem_desfocado() {
    let mut l = Laco::novo(true);
    l.foco(false);
    assert_eq!(l.pedido(&[], Instant::now()), Pedido::Ja(Produtor::Smoke));
    for (k, v) in [
        ("PH2D_EXIT_AFTER_FRAMES", "120"),
        ("PH2D_COMPOSITE_SMOKE", "1"),
        ("PH2D_GPU_COOK_DEMO", "3"),
        ("PH2D_QUADRO_A_PEDIDO", "0"),
    ] {
        assert!(e_smoke(k, v), "{k}");
    }
    assert!(!e_smoke("PH2D_QUADRO_A_PEDIDO", "1"));
    assert!(!e_smoke("PH2D_CARDS", "0"));
}

/// B23.3: oculta, a UI (a entrada incluída) dorme o [`PRAZO`]; o documento e o Play não param.
#[test]
fn a_janela_oculta_so_acorda_pelo_documento_e_pelo_play() {
    let t0 = Instant::now();
    let mut l = Laco::novo(false);
    l.oculta(true);
    l.entrada(t0);
    let oculta = Pedido::Ate {
        prazo: PRAZO,
        porque: Parada::Oculta,
    };
    assert_eq!(l.pedido(&[Produtor::UiAnima, Produtor::Toast], t0), oculta);
    for p in [Produtor::PapelMolhado, Produtor::Play, Produtor::Som] {
        assert_eq!(l.pedido(&[Produtor::Toast, p], t0), Pedido::Ja(p));
    }
    l.oculta(false);
    assert_eq!(l.pedido(&[], t0), Pedido::Ja(Produtor::Entrada));
}

/// B23.3: oculta, o quadro não apresenta (o documento avança na mesma); visível ou num smoke, apresenta.
#[test]
fn a_janela_oculta_nao_apresenta_salvo_o_smoke() {
    for smoke in [false, true] {
        let mut l = Laco::novo(smoke);
        assert!(l.apresenta(), "visível, smoke={smoke}");
        l.foco(false);
        assert!(l.apresenta(), "desfocada mas à vista, smoke={smoke}");
        l.oculta(true);
        assert_eq!(l.apresenta(), smoke, "oculta, smoke={smoke}");
        l.oculta(false);
        assert!(l.apresenta(), "de volta à vista, smoke={smoke}");
    }
    for (forca, apresenta) in [(Some(true), true), (Some(false), false)] {
        let mut l = Laco {
            forca_oculta: forca,
            ..Laco::novo(true)
        };
        l.oculta(true);
        assert_eq!(l.apresenta(), apresenta, "PH2D_OCULTA_APRESENTA={forca:?}");
    }
}

/// B23.3: oculta e sem present, o vsync já não segura o laço (medido: 190–256 q/s, CPU 97–100 % de um
/// núcleo × 58–83 q/s a apresentar); o Documento corre ao [`SEM_PRESENT`]. O laço da shell: `Some(t)`
/// dorme até `t` e desenha; `None` desenha já. Cada quadro sem present custa 4 ms.
#[test]
fn oculta_sem_present_o_documento_corre_a_60_e_nao_gira() {
    let doc = [Produtor::PapelMolhado];
    let quadros_num_segundo = |l: &mut Laco| {
        let t0 = Instant::now();
        let (mut agora, mut n) = (t0, 0);
        while agora < t0 + Duration::from_secs(1) {
            if let Some(t) = l.acorda(&doc, agora) {
                assert!(t > agora, "acordar no passado é girar");
                agora = t;
            }
            agora += Duration::from_millis(4);
            n += 1;
        }
        n
    };
    let mut l = Laco::novo(false);
    l.oculta(true);
    assert!(!l.apresenta());
    let n = quadros_num_segundo(&mut l);
    assert!((58..=61).contains(&n), "oculta sem present: {n} q/s");
    l.oculta(false);
    assert_eq!(
        l.acorda(&doc, Instant::now()),
        None,
        "à vista, o present segura o laço"
    );
    let mut smoke = Laco::novo(true);
    smoke.oculta(true);
    assert_eq!(
        quadros_num_segundo(&mut smoke),
        250,
        "o smoke apresenta: o present segura"
    );
}

/// B23.1: a linha `[quadro] quem:` conta o vencedor da lei e a parada, por janela, com as faltas por quadro.
#[test]
fn o_rastreio_conta_quem_pediu_o_quadro() {
    use rastreio::{JANELA, Rastreio};
    let mut r = Rastreio::default();
    let dorme = Pedido::Ate {
        prazo: PRAZO,
        porque: Parada::Ocioso,
    };
    assert_eq!(r.nota(&dorme, Some(1_000)), None);
    for i in 1..JANELA - 1 {
        assert_eq!(
            r.nota(&Pedido::Ja(Produtor::Smoke), Some(1_000 + u64::from(i))),
            None
        );
    }
    let linha = r
        .nota(&Pedido::Ja(Produtor::Entrada), Some(1_000 + 240))
        .expect("fim da janela");
    assert_eq!(
        linha,
        "[quadro] quem: Entrada=1 Smoke=118 dorme:Ocioso=1 | minflt/quadro=2.0 (fio da UI)"
    );
    let depois = r.nota(&dorme, Some(1_240));
    assert_eq!(depois, None, "a janela recomeça");
}

/// B23.1: cada marca fecha o troço desde a anterior; a média é por quadro desenhado (os `Fora`).
#[test]
fn o_rastreio_parte_as_faltas_por_fase() {
    use rastreio::{Fase, PorFase};
    let mut f = PorFase::default();
    f.marca(Fase::Present, 100); // o quadro anterior: só arma
    let quadro = [
        (Fase::Fora, 2),
        (Fase::Entrada, 4),
        (Fase::Cena, 0),
        (Fase::Sim, 6),
        (Fase::Desenho, 10),
        (Fase::Acquire, 0),
        (Fase::Present, 20),
    ];
    let mut m = 100;
    for _ in 0..2 {
        for (fase, d) in quadro {
            m += d;
            f.marca(fase, m);
        }
    }
    assert_eq!(
        f.resumo(),
        " | por quadro desenhado (2): fora=2.0 entrada=4.0 cena=0.0 sim=6.0 desenho=10.0 acquire=0.0 present=20.0"
    );
    assert_eq!(f.resumo(), "", "a janela recomeça vazia");
    f.marca(Fase::Fora, m + 3);
    assert!(
        f.resumo().contains("(1): fora=3.0 "),
        "o último lido sobrevive à janela"
    );
}

/// B23.2: o pedido feito DENTRO do quadro acorda já o laço uma vez e esvazia-se; o despertar pedido
/// encurta o sono (nunca o alonga) e também se esvazia.
#[test]
fn o_pedido_de_dentro_do_quadro_vale_um_acorda() {
    let agora = Instant::now();
    let mut l = Laco::novo(false);
    l.pede(Produtor::Animacao);
    assert_eq!(l.acorda(&[], agora), None, "pedido ⇒ desenha já");
    assert_eq!(
        l.acorda(&[], agora),
        Some(agora + PRAZO),
        "esvaziou ⇒ dorme"
    );
    let cedo = agora + Duration::from_millis(40);
    l.desperta_ate(agora + Duration::from_secs(9));
    l.desperta_ate(cedo);
    assert_eq!(l.acorda(&[], agora), Some(cedo), "o mais cedo encurta");
    l.desperta_ate(agora + Duration::from_secs(9));
    assert_eq!(l.acorda(&[], agora), Some(agora + PRAZO), "nunca alonga");
    assert_eq!(l.acorda(&[], agora), Some(agora + PRAZO));
}

/// B21.1: em Mailbox (ritmo = um refresco) o contínuo não passa do monitor e dorme até ao
/// próximo refresco; sem ritmo (Fifo) «já» é sempre já.
#[test]
fn no_modo_que_nao_bloqueia_o_quadro_segue_o_refresco() {
    let r = Some(Duration::from_millis(7));
    let t0 = Instant::now();
    let mut l = Laco::novo(false);
    let p = [Produtor::Animacao];
    assert_eq!(l.acorda_no_ritmo(&p, t0, r), None);
    let t1 = t0 + Duration::from_millis(2);
    assert_eq!(
        l.acorda_no_ritmo(&p, t1, r),
        Some(t0 + Duration::from_millis(7))
    );
    let t2 = t0 + Duration::from_millis(7);
    assert_eq!(l.acorda_no_ritmo(&p, t2, r), None);
    let mut f = Laco::novo(false);
    assert_eq!(f.acorda_no_ritmo(&p, t0, None), None);
    assert_eq!(f.acorda_no_ritmo(&p, t1, None), None);
}
