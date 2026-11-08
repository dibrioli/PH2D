//! ⭐ **A janela SLICES, o menu, as marcas e o `Transform`, do pedido ao documento** (P11b): o pedido é
//! o que o painel faz (o ponteiro até ao pedido prova-se em `the_pixel_slices_answers_a_real_pointer`);
//! o botão direito, o arrasto e as teclas vêm pela porta da shell.

use ph2d_app_pixel::panel;
use ph2d_panel_pixel_slices::canal::{self, Campo, Pedido};
use ph2d_pixel_core::{CelKey, FrameId, LayerId, Op, Rect, SliceId, frames, layers};
use winit::keyboard::KeyCode;

use crate::slices::{CTRL, Mesa, NADA, SHIFT, px};

impl Mesa {
    fn passos(&self) -> usize {
        self.app.doc(self.key).expect("doc").history.undo_len()
    }

    /// Um pedido como o painel o faz, e o quadro que o aplica.
    fn pede(&mut self, p: Pedido) {
        canal::pede(p);
        self.quadro();
    }

    /// Uma slice só, selecionada, com a ferramenta na mão.
    fn com_uma(&mut self, de: (i32, i32), a: (i32, i32)) -> SliceId {
        assert!(self.tecla(KeyCode::KeyC, SHIFT));
        self.arrasta(px(de.0, de.1), px(a.0, a.1), NADA);
        let id = self.sprite().slices().last().expect("nasceu").id;
        self.arrasta(px(de.0 + 1, de.1 + 1), px(de.0 + 1, de.1 + 1), NADA);
        assert_eq!(self.app.slices_selected(self.key), [id]);
        id
    }

    fn pinta(&mut self, key: CelKey, r: Rect, rgba: [u8; 4]) {
        let op = Op::Patch(ph2d_pixel_core::Patch {
            layer: key.0,
            frame: key.1,
            bbox: Some(r),
            rect: r,
            pixels: rgba.repeat(r.area()),
        });
        self.app.commit(self.key, op).expect("pinta");
    }

    /// **As duas camadas e os dois quadros que separam as hipóteses do `Transform`:** um quadrado de
    /// cor própria em cada cel, no mesmo sítio. Fica ativa a camada de cima, no quadro 0.
    fn duas_por_duas(&mut self) -> (LayerId, LayerId, FrameId, FrameId) {
        let d = self.app.doc(self.key).expect("doc");
        let (l0, f0) = (d.layer, d.frame);
        let d = self.app.foca(self.key).expect("doc");
        let (op, f1) = frames::new_frame(&mut d.sprite, f0, frames::NewFrame::Empty).expect("q");
        self.app.commit(self.key, op).expect("q");
        let d = self.app.foca(self.key).expect("doc");
        let (op, l1) = layers::new_layer(&mut d.sprite, l0).expect("c");
        self.app.commit(self.key, op).expect("c");
        let quadrado = Rect::new(8, 8, 4, 4);
        self.pinta((l0, f0), quadrado, [10, 0, 0, 255]);
        self.pinta((l0, f1), quadrado, [20, 0, 0, 255]);
        self.pinta((l1, f0), quadrado, [30, 0, 0, 255]);
        self.pinta((l1, f1), quadrado, [40, 0, 0, 255]);
        let d = self.app.foca(self.key).expect("doc");
        (d.layer, d.frame) = (l1, f0);
        (l0, l1, f0, f1)
    }

    fn cel(&self, key: CelKey) -> Vec<u8> {
        let (w, h) = (self.sprite().width(), self.sprite().height());
        self.sprite().read_rect(key, Rect::new(0, 0, w, h))
    }
}

/// Os números da lei G6, pelo pedido: e um passo por confirmação.
#[test]
fn os_campos_seguem_a_lei_do_dialogo_um_passo_por_confirmacao() {
    let mut t = Mesa::nova();
    let id = t.com_uma((4, 4), (24, 14));
    let casos = [
        (Campo::H, "-7", Rect::new(4, 4, 20, 1)),
        (Campo::W, "abc", Rect::new(4, 4, 1, 1)),
        (Campo::W, "12abc", Rect::new(4, 4, 12, 1)),
        (Campo::W, "0", Rect::new(4, 4, 1, 1)),
        (Campo::W, "", Rect::new(4, 4, 1, 1)),
        (Campo::X, "1.5", Rect::new(1, 4, 1, 1)),
        (Campo::X, "-10", Rect::new(-10, 4, 1, 1)),
        (Campo::X, "", Rect::new(0, 4, 1, 1)),
    ];
    for (c, texto, esperado) in casos {
        let antes = t.passos();
        t.pede(Pedido::Campo(c, texto.to_owned()));
        assert_eq!(t.forma(id).bounds, esperado, "{c:?} {texto:?}");
        let mudou = t.passos() - antes;
        assert!(mudou <= 1, "{c:?} {texto:?} fez {mudou} passos");
    }
    t.pede(Pedido::Pivot(true));
    assert_eq!(t.forma(id).pivot, Some((0, 0)));
    t.pede(Pedido::Campo(Campo::PivotX, "-5".to_owned()));
    assert_eq!(t.forma(id).pivot, Some((-5, 0)));
    t.pede(Pedido::Campo(Campo::W, "20".to_owned()));
    t.pede(Pedido::Campo(Campo::H, "10".to_owned()));
    t.pede(Pedido::Nove(true));
    assert_eq!(t.forma(id).center, Some(Rect::new(1, 1, 18, 8)), "T3");
    t.pede(Pedido::Nove(false));
    assert_eq!(t.forma(id).center, None);
}

/// Com a key no quadro 0, editar no 1 cria key só lá; o nome, a cor e o texto valem para a slice toda
/// (G6), num passo cada, e com várias selecionadas valem para todas (§10).
#[test]
fn os_campos_editam_a_key_do_quadro_e_o_nome_a_slice_inteira() {
    let mut t = Mesa::nova();
    let (_, _, _, f1) = t.duas_por_duas();
    let id = t.com_uma((4, 4), (24, 14));
    let outra = t.com_uma((30, 4), (40, 14));
    t.pede(Pedido::Seleciona(id));
    t.app.foca(t.key).expect("doc").frame = f1;
    t.pede(Pedido::Campo(Campo::X, "9".to_owned()));
    let c = t.sprite().slice(id).expect("slice");
    assert_eq!(c.keys.len(), 2);
    assert_eq!(c.shape_at(0).expect("0").bounds.x, 4, "o quadro 0 fica");
    assert_eq!(c.shape_at(1).expect("1").bounds.x, 9);

    t.pede(Pedido::Todas);
    let antes = t.passos();
    t.pede(Pedido::Campo(Campo::Y, "2".to_owned()));
    assert_eq!(t.passos(), antes + 1, "várias: 1 passo");
    assert_eq!(t.forma(id).bounds.y, 2);
    assert_eq!(t.forma(outra).bounds.y, 2);
    assert_eq!(t.forma(id).bounds.x, 9, "só o campo escrito");
    assert_eq!(t.forma(outra).bounds.x, 30, "só o campo escrito");
    t.pede(Pedido::Nome("zzz".to_owned()));
    t.pede(Pedido::Cor([255, 0, 255, 255]));
    t.pede(Pedido::PassoFechou);
    t.pede(Pedido::Dados("hello".to_owned()));
    assert_eq!(t.passos(), antes + 4);
    for s in t.sprite().slices() {
        assert_eq!(s.name, "zzz");
        assert_eq!(s.color, Some([255, 0, 255, 255]));
        assert_eq!(s.data, "hello");
    }
    assert_eq!(t.sprite().slice(id).expect("slice").keys.len(), 2);
    assert!(t.app.undo(t.key));
    assert_eq!(t.sprite().slices()[0].data, "", "o desfazer volta o texto");
}

/// O seletor aberto reescreve o MESMO passo; fechado, a cor seguinte é outro.
#[test]
fn arrastar_no_seletor_de_cor_e_um_passo() {
    let mut t = Mesa::nova();
    let id = t.com_uma((4, 4), (24, 14));
    let antes = t.passos();
    for v in 1..5 {
        t.pede(Pedido::Cor([v, 0, 0, 255]));
    }
    assert_eq!(t.passos(), antes + 1);
    assert_eq!(
        t.sprite().slice(id).expect("slice").color,
        Some([4, 0, 0, 255])
    );
    t.pede(Pedido::PassoFechou);
    t.pede(Pedido::Cor([9, 0, 0, 255]));
    assert_eq!(t.passos(), antes + 2);
}

/// ⭐ Arrastar a letra `W` é UM passo: cada valor reescreve o mesmo, soltar fecha-o, e um `Ctrl+Z` volta a
/// largura de antes. Com duas selecionadas o arrasto vale para as duas, no mesmo passo (§10).
#[test]
fn arrastar_a_letra_de_um_numero_e_um_passo_e_um_ctrl_z_o_desfaz() {
    let mut t = Mesa::nova();
    let id = t.com_uma((4, 4), (24, 14));
    let largura = t.forma(id).bounds.w;
    let antes = t.passos();
    for w in [largura + 1, largura + 3, largura + 7] {
        t.pede(Pedido::Arrasta(Campo::W, w.to_string()));
    }
    t.pede(Pedido::PassoFechou);
    assert_eq!(t.forma(id).bounds.w, largura + 7, "a largura seguiu a mão");
    assert_eq!(t.passos(), antes + 1, "o arrasto inteiro é um passo");
    assert!(t.tecla(KeyCode::KeyZ, CTRL));
    assert_eq!(t.forma(id).bounds.w, largura, "um Ctrl+Z volta a largura");

    t.pede(Pedido::Arrasta(Campo::W, "3".to_owned()));
    t.pede(Pedido::PassoFechou);
    t.pede(Pedido::Arrasta(Campo::W, "5".to_owned()));
    t.pede(Pedido::PassoFechou);
    assert_eq!(t.passos(), antes + 2, "dois arrastos são dois passos");

    let outra = t.com_uma((30, 4), (40, 14));
    t.pede(Pedido::Todas);
    let antes = t.passos();
    t.pede(Pedido::Arrasta(Campo::H, "0".to_owned()));
    t.pede(Pedido::Arrasta(Campo::H, "4".to_owned()));
    t.pede(Pedido::PassoFechou);
    assert_eq!((t.forma(id).bounds.h, t.forma(outra).bounds.h), (4, 4));
    assert_eq!(t.passos(), antes + 1);
}

/// O direito numa selecionada abre o menu; noutra, nada (T2). A linha da mudança só acende com key
/// própria neste quadro que não seja a única; `Duplicate Slice` seleciona a cópia e 1 desfazer a tira.
#[test]
fn o_botao_direito_abre_o_menu_so_na_selecionada_e_o_menu_age() {
    let mut t = Mesa::nova();
    let (_, _, _, f1) = t.duas_por_duas();
    let id = t.com_uma((4, 4), (24, 14));
    let outra = t.com_uma((26, 4), (31, 14));
    assert_eq!(t.app.slices_selected(t.key), [outra]);
    let _ = canal::take_menu();
    t.direito(px(6, 6));
    t.quadro();
    assert_eq!(canal::take_menu(), None, "T2: não selecionada não tem menu");
    assert_eq!(t.app.slices_selected(t.key), [outra], "nem muda a seleção");
    t.direito(px(30, 30));
    t.quadro();
    assert_eq!(canal::take_menu(), None, "fora de todas");
    t.direito(px(28, 6));
    t.quadro();
    let m = canal::take_menu().expect("sobre a selecionada, abre");
    assert!(!m.remover, "a única key não se remove");
    assert_eq!(
        t.forma(id).bounds,
        Rect::new(4, 4, 20, 10),
        "o direito não é gesto"
    );

    t.app.foca(t.key).expect("doc").frame = f1;
    t.quadro();
    t.direito(px(28, 6));
    t.quadro();
    assert!(
        !canal::take_menu().expect("menu").remover,
        "herdada: sem key própria"
    );
    t.pede(Pedido::Campo(Campo::X, "27".to_owned()));
    t.direito(px(28, 6));
    t.quadro();
    assert!(canal::take_menu().expect("menu").remover, "agora tem");
    let antes = t.passos();
    t.pede(Pedido::RemoveMudanca);
    assert_eq!(t.passos(), antes + 1);
    assert_eq!(
        t.forma(outra).bounds.x,
        26,
        "o valor anterior volta a valer"
    );

    t.pede(Pedido::Duplica);
    let copia = t.app.slices_selected(t.key)[0];
    assert_ne!(copia, outra);
    let nome = &t.sprite().slice(copia).expect("cópia").name;
    assert!(nome.ends_with(" Copy"), "{nome}");
    assert_eq!(t.forma(copia).bounds.x, 28, "+2");
    assert!(t.app.undo(t.key));
    t.quadro();
    assert!(t.sprite().slice(copia).is_none());

    t.pede(Pedido::Seleciona(outra));
    t.pede(Pedido::Apaga);
    assert!(t.sprite().slice(outra).is_none(), "G2");
    assert!(t.app.slices_selected(t.key).is_empty());
}

/// Dois cliques depressa na mesma slice mostram o cartão (T2); um só, ou em vazio, não.
#[test]
fn o_duplo_clique_mostra_o_cartao() {
    let mut t = Mesa::nova();
    let _id = t.com_uma((4, 4), (24, 14));
    let _ = canal::take_mostrar();
    t.arrasta(px(6, 6), px(6, 6), NADA);
    assert!(!canal::take_mostrar(), "um clique só");
    t.arrasta(px(6, 6), px(6, 6), NADA);
    assert!(canal::take_mostrar(), "o segundo");
    t.arrasta(px(50, 50), px(50, 50), NADA);
    t.arrasta(px(50, 50), px(50, 50), NADA);
    assert!(!canal::take_mostrar(), "em vazio, nada");
}

/// A janela abre e fecha com a ferramenta, e diz o que há.
#[test]
fn a_janela_segue_a_ferramenta() {
    let mut t = Mesa::nova();
    let aberta = |t: &Mesa| t.hero.panel_visibility.get("pixel_slices").copied();
    t.quadro();
    assert_eq!(aberta(&t), Some(false));
    assert!(!canal::contexto().na_mao);
    let id = t.com_uma((4, 4), (24, 14));
    t.quadro();
    assert_eq!(aberta(&t), Some(true));
    let x = canal::contexto();
    assert_eq!(x.linhas.len(), 1);
    assert_eq!(x.vista.expect("selecionada").bounds[2], Some(20));
    assert_eq!(x.linhas[0].id, id);
    assert!(t.tecla(KeyCode::KeyB, NADA));
    t.quadro();
    assert_eq!(aberta(&t), Some(false));
}

/// As marcas: os quadros com key das selecionadas, com a ferramenta na mão.
#[test]
fn as_marcas_dizem_os_quadros_com_key_das_selecionadas() {
    let mut t = Mesa::nova();
    let (_, _, _, f1) = t.duas_por_duas();
    let marcas = |t: &mut Mesa| {
        panel::frame(&mut t.app, t.sim.world(), &mut t.hero, &mut t.toasts).slice_keys
    };
    assert_eq!(marcas(&mut t), [] as [u32; 0]);
    let id = t.com_uma((4, 4), (24, 14));
    assert_eq!(marcas(&mut t), [0]);
    t.app.foca(t.key).expect("doc").frame = f1;
    t.pede(Pedido::Campo(Campo::X, "9".to_owned()));
    assert_eq!(marcas(&mut t), [0, 1]);
    t.pede(Pedido::Nenhuma);
    assert_eq!(marcas(&mut t), [] as [u32; 0], "sem seleção, sem marca");
    t.pede(Pedido::Seleciona(id));
    assert!(t.tecla(KeyCode::KeyB, NADA));
    assert_eq!(marcas(&mut t), [] as [u32; 0], "com o lápis, sem marca");
}

/// **`Transform`, mover (G5):** só a cel da camada ativa no quadro atual muda e fica o buraco; a outra
/// camada e o outro quadro ficam byte a byte; 1 desfazer volta slice e píxeis. Desligado, nada anda.
#[test]
fn transform_mover_leva_so_a_cel_ativa_do_quadro_atual() {
    let mut t = Mesa::nova();
    let (l0, l1, f0, f1) = t.duas_por_duas();
    let id = t.com_uma((6, 6), (16, 16));
    let imagem = |t: &Mesa| {
        [
            t.cel((l0, f0)),
            t.cel((l0, f1)),
            t.cel((l1, f0)),
            t.cel((l1, f1)),
        ]
    };
    let antes = imagem(&t);

    t.arrasta(px(7, 7), px(10, 9), NADA);
    assert_eq!(t.forma(id).bounds, Rect::new(9, 8, 10, 10));
    assert_eq!(imagem(&t), antes, "desligado: os píxeis ficam");
    assert!(t.app.undo(t.key));

    t.pede(Pedido::Transform);
    let passos = t.passos();
    t.arrasta(px(7, 7), px(10, 9), NADA);
    assert_eq!(t.passos(), passos + 1, "um passo só");
    assert_eq!(t.forma(id).bounds, Rect::new(9, 8, 10, 10));
    let depois = imagem(&t);
    assert_eq!(depois[0], antes[0], "a outra camada, mesmo quadro");
    assert_eq!(depois[1], antes[1], "a outra camada, outro quadro");
    assert_eq!(depois[3], antes[3], "a mesma camada, outro quadro");
    assert_ne!(depois[2], antes[2]);
    let cor = t.sprite().value_of_rgba([30, 0, 0, 255]);
    let vazio = t.sprite().transparent_value();
    assert_eq!(t.sprite().value_at((l1, f0), 8, 8), vazio, "o buraco");
    assert_eq!(
        t.sprite().value_at((l1, f0), 11, 10),
        cor,
        "o canto do quadrado andou +3,+2"
    );
    assert_eq!(t.sprite().value_at((l1, f0), 14, 13), cor);
    assert_eq!(t.sprite().value_at((l1, f0), 15, 14), vazio);

    assert!(t.app.undo(t.key));
    assert_eq!(imagem(&t), antes, "1 desfazer volta os píxeis");
    assert_eq!(t.forma(id).bounds, Rect::new(6, 6, 10, 10), "e a slice");
}

/// **`Transform`, redimensionar (G5):** escala pelo vizinho mais próximo, dentro da caixa nova; só com
/// a borda a mexer. O 9-slice e um clique não mexem na caixa e não tocam nos píxeis.
#[test]
fn transform_redimensionar_escala_a_cel_ativa() {
    let mut t = Mesa::nova();
    let (l0, l1, f0, f1) = t.duas_por_duas();
    let id = t.com_uma((8, 8), (12, 12));
    let outras = |t: &Mesa| [t.cel((l0, f0)), t.cel((l0, f1)), t.cel((l1, f1))];
    let antes = outras(&t);
    t.pede(Pedido::Transform);
    let passos = t.passos();
    t.arrasta(px(9, 9), px(9, 9), NADA);
    assert_eq!(t.passos(), passos, "um clique não é passo");
    // O canto de baixo à direita da caixa 8,8,4,4 vai dobrar o lado.
    t.arrasta((12.0, 12.0), (16.0, 16.0), NADA);
    assert_eq!(t.forma(id).bounds, Rect::new(8, 8, 8, 8));
    assert_eq!(t.passos(), passos + 1);
    assert_eq!(outras(&t), antes);
    let cor = t.sprite().value_of_rgba([30, 0, 0, 255]);
    for (x, y) in [(8, 8), (15, 15), (15, 8), (8, 15)] {
        assert_eq!(t.sprite().value_at((l1, f0), x, y), cor, "({x},{y})");
    }
    assert_eq!(
        t.sprite().value_at((l1, f0), 16, 16),
        t.sprite().transparent_value()
    );
    assert!(t.app.undo(t.key));
    assert_eq!(t.forma(id).bounds, Rect::new(8, 8, 4, 4));
}

/// **`Transform` recusa e diz porquê:** camada trancada, escondida, grupo e tilemap; nem a slice anda.
/// Desligado, a mesma camada não recusa nada.
#[test]
fn transform_recusa_o_que_nao_se_pinta_e_diz() {
    for caso in ["locked", "hidden", "group", "tilemap"] {
        let mut t = Mesa::nova();
        let (_, l1, _, _) = t.duas_por_duas();
        let id = t.com_uma((6, 6), (16, 16));
        t.pede(Pedido::Transform);
        let d = t.app.foca(t.key).expect("doc");
        let op = match caso {
            "locked" => layers::edit(&d.sprite, l1, |l| l.locked = true),
            "hidden" => layers::edit(&d.sprite, l1, |l| l.visible = false),
            "group" => {
                let (op, g) = layers::new_group(&mut d.sprite, l1).expect("grupo");
                d.layer = g;
                Some(op)
            }
            _ => {
                let (op, l, _) = ph2d_pixel_core::tiles::new_tilemap_layer(&mut d.sprite, l1, 4, 4)
                    .expect("tilemap");
                d.layer = l;
                Some(op)
            }
        };
        t.app.commit(t.key, op.expect("muda")).expect("aplica");
        let _ = t.app.take_avisos();
        let passos = t.passos();
        t.arrasta(px(7, 7), px(10, 9), NADA);
        assert_eq!(t.forma(id).bounds, Rect::new(6, 6, 10, 10), "{caso}");
        assert_eq!(t.passos(), passos, "{caso}");
        let avisos = t.app.take_avisos();
        assert_eq!(avisos.len(), 1, "{caso}: {avisos:?}");
        assert!(avisos[0].0, "{caso}: é erro");
        assert!(!avisos[0].1.starts_with("pixel."), "{caso}: {avisos:?}");

        t.pede(Pedido::Transform);
        t.arrasta(px(7, 7), px(10, 9), NADA);
        assert_eq!(
            t.forma(id).bounds,
            Rect::new(9, 8, 10, 10),
            "{caso}: sem Transform move"
        );
    }
}
