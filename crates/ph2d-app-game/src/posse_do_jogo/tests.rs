use super::*;
use ph2d_input::{Event, Key};

/// ⭐ Report do dono 25/10: no palco uma aba aberta no editor não tapa o jogo; no editor tapa.
#[test]
fn no_palco_a_aba_aberta_nao_tapa_o_jogo() {
    let aba = Tapa {
        aba: true,
        ..Tapa::default()
    };
    assert!(!aba.a_vista(), "no editor a aba tapa a cena");
    assert!(
        Tapa { palco: true, ..aba }.a_vista(),
        "no palco a aba fica atrás do jogo"
    );
    for tapa in [
        Tapa {
            palco: true,
            texto: true,
            ..Tapa::default()
        },
        Tapa {
            palco: true,
            modo: true,
            ..Tapa::default()
        },
    ] {
        assert!(
            !tapa.a_vista(),
            "{tapa:?}: texto e modos tapam até no palco"
        );
    }
    assert!(Tapa::default().a_vista());
}

/// Doc 17 item 11 (dono 10/11): com a Timeline a ANIMAR, tocar a agulha não dá a cena ao jogo — o
/// clique arrasta as chaves e as alças do canvas. *Mutação: `a_vista` sem o `animar` ⇒ o clique é do jogo.*
#[test]
fn a_previa_da_timeline_a_tocar_nao_da_o_rato_ao_jogo() {
    let animar = Tapa {
        animar: true,
        ..Tapa::default()
    };
    assert!(!animar.a_vista(), "a animar, a cena é do editor");
    let mut foco = FocoDoJogo::default();
    let cena = Cena {
        a_correr: true,
        a_vista: animar.a_vista(),
        palco: false,
    };
    let mut input = InputState::default();
    assert!(
        !foco.clique(
            &mut input,
            cena,
            ph2d_host::PointerButton::Primary,
            true,
            true
        ),
        "o clique na cena a tocar chega ao editor"
    );
}

#[test]
fn um_clique_fora_da_cena_nao_conta_e_o_largar_passa_sempre() {
    let mut input = InputState::new();
    input.begin_frame();
    game_mouse(&mut input, ph2d_host::PointerButton::Primary, true, false);
    assert!(
        !input.mouse.pressed(MouseButton::Left),
        "um clique num painel não é do jogo"
    );
    game_mouse(&mut input, ph2d_host::PointerButton::Primary, true, true);
    assert!(input.mouse.pressed(MouseButton::Left));
    input.begin_frame();
    game_mouse(&mut input, ph2d_host::PointerButton::Primary, false, false);
    assert!(
        input.mouse.released(MouseButton::Left),
        "o largar fora da cena passa"
    );
}

const CORRE: Cena = Cena {
    a_correr: true,
    a_vista: true,
    palco: false,
};
const PARADA: Cena = Cena {
    a_correr: false,
    a_vista: true,
    palco: false,
};
const SETA: Key = Key(0x25);

const NO_PALCO: Cena = Cena {
    a_correr: true,
    a_vista: true,
    palco: true,
};
const PAUSADA_NO_PALCO: Cena = Cena {
    a_correr: false,
    a_vista: true,
    palco: true,
};

/// ⭐ Pausado em tela cheia, Delete e G não mexem no editor escondido.
/// Mutação: tirar o `|=` de `tecla` ⇒ a tecla chega aos atalhos do editor.
#[test]
fn pausado_no_palco_a_tecla_nao_chega_ao_editor_nem_ao_jogo() {
    let mut input = InputState::default();
    let f = FocoDoJogo::default();
    let t = Tecla {
        desceu: true,
        acorde: false,
        shift: false,
        f5: false,
        repete: false,
    };
    let d = f.tecla(&mut input, PAUSADA_NO_PALCO, t, Some(SETA), false, &"G");
    assert!(d.consumida);
    assert!(!d.jogo);
}

/// ⭐ Pausado em tela cheia, o clique no jogo não seleciona nem abre nada no editor.
#[test]
fn pausado_no_palco_o_clique_nao_chega_ao_editor() {
    use ph2d_host::PointerButton as B;
    for botao in [B::Primary, B::Secondary, B::Middle] {
        let (mut input, mut f) = (InputState::default(), FocoDoJogo::default());
        assert!(
            f.clique(&mut input, PAUSADA_NO_PALCO, botao, true, true),
            "{botao:?}"
        );
    }
}

/// ⭐ Clicar um botão do jogo com o jogo a correr não tira o teclado ao jogo.
/// Mutação: voltar a `self.focado = na_cena` ⇒ o personagem deixa de responder depois do clique.
#[test]
fn um_clique_nos_botoes_do_palco_nao_tira_o_teclado() {
    let (mut input, mut f) = (InputState::default(), FocoDoJogo::default());
    f.quadro(&mut input, NO_PALCO);
    assert!(f.tem_o_teclado(NO_PALCO));
    assert!(!f.clique(
        &mut input,
        NO_PALCO,
        ph2d_host::PointerButton::Primary,
        true,
        false
    ));
    assert!(f.tem_o_teclado(NO_PALCO));
}

fn nua(desceu: bool) -> Tecla {
    Tecla {
        desceu,
        acorde: false,
        shift: false,
        f5: false,
        repete: false,
    }
}

fn presa(input: &mut InputState) {
    input.apply_event(Event::KeyDown(SETA));
    input.begin_frame();
}

#[test]
fn o_play_da_o_teclado_ao_jogo() {
    let mut f = FocoDoJogo::default();
    let mut input = InputState::new();
    f.quadro(&mut input, PARADA);
    assert!(!f.tem_o_teclado(PARADA));
    f.quadro(&mut input, CORRE);
    assert!(
        f.tem_o_teclado(CORRE),
        "a subida do relógio dá o foco, sem clique"
    );
}

/// ⭐ Report do dono 25/10: com o editor de animações aberto, as teclas só chegavam ao jogo depois de um
/// clique na janela dele. O Play sobe com o palco ainda por abrir (a janela abre depois do quadro) e a
/// aba tapa a cena nesse quadro; abrir o palco é o mesmo gesto do Play, e dá o foco.
#[test]
fn abrir_o_palco_da_o_teclado_ao_jogo() {
    let mut f = FocoDoJogo::default();
    let mut input = InputState::new();
    let por_abrir = Cena {
        a_correr: true,
        a_vista: false,
        palco: false,
    };
    f.quadro(&mut input, PARADA);
    f.quadro(&mut input, por_abrir);
    assert!(!f.tem_o_teclado(por_abrir), "no quadro do Play a aba tapa");
    let no_palco = Cena {
        palco: true,
        ..CORRE
    };
    f.quadro(&mut input, no_palco);
    assert!(
        f.tem_o_teclado(no_palco),
        "o palco abriu: o jogo tem o teclado, sem clique"
    );
    assert!(
        !f.premido(&mut input, no_palco, false),
        "um clique fora da cena não é do jogo"
    );
    f.quadro(&mut input, no_palco);
    // ⭐ Pausa padrão-ouro (dono 07/11): no palco não há editor a quem dar o foco — um clique nos
    // botões do jogo não lhe tira o teclado.
    assert!(
        f.tem_o_teclado(no_palco),
        "no palco um clique fora não tira o teclado ao jogo"
    );
    f.quadro(&mut input, por_abrir);
    f.quadro(&mut input, PARADA);
    f.quadro(&mut input, CORRE);
    assert!(
        f.tem_o_teclado(CORRE),
        "sem aba, o Play no editor continua a dá-lo"
    );
}

#[test]
fn com_uma_aba_por_cima_o_play_nao_da_o_teclado() {
    let mut f = FocoDoJogo::default();
    let mut input = InputState::new();
    let tapada = Cena {
        a_correr: true,
        a_vista: false,
        palco: false,
    };
    f.quadro(&mut input, tapada);
    assert!(!f.tem_o_teclado(tapada));
    f.quadro(&mut input, CORRE);
    assert!(
        !f.tem_o_teclado(CORRE),
        "fechar a aba não devolve o foco: o Play deu-o a uma cena tapada"
    );
    assert!(
        f.premido(&mut input, CORRE, true),
        "um clique na cena devolve-o"
    );
    assert!(f.tem_o_teclado(CORRE));
}

#[test]
fn um_clique_fora_tira_o_teclado_e_larga_as_teclas() {
    let mut f = FocoDoJogo::default();
    let mut input = InputState::new();
    f.quadro(&mut input, CORRE);
    presa(&mut input);
    assert!(
        !f.premido(&mut input, CORRE, false),
        "o clique num painel é do editor"
    );
    assert!(!f.tem_o_teclado(CORRE));
    assert!(!input.keyboard.held(SETA), "a seta presa larga-se");
    assert!(input.keyboard.released(SETA), "e o jogo vê-a subir");
    assert!(
        f.premido(&mut input, CORRE, true),
        "o clique na cena é só do jogo"
    );
    assert!(f.tem_o_teclado(CORRE));
}

#[test]
fn pausar_larga_as_teclas_presas() {
    let mut f = FocoDoJogo::default();
    let mut input = InputState::new();
    f.quadro(&mut input, CORRE);
    presa(&mut input);
    f.quadro(&mut input, PARADA);
    assert!(!input.keyboard.held(SETA));
    f.quadro(&mut input, CORRE);
    assert!(
        f.tem_o_teclado(CORRE),
        "continuar devolve o teclado ao jogo"
    );
}

#[test]
fn tapar_a_cena_tira_o_foco_e_destapar_nao_o_devolve() {
    let mut f = FocoDoJogo::default();
    let mut input = InputState::new();
    f.quadro(&mut input, CORRE);
    presa(&mut input);
    let tapada = Cena {
        a_correr: true,
        a_vista: false,
        palco: false,
    };
    f.quadro(&mut input, tapada);
    assert!(
        !input.keyboard.held(SETA),
        "a paleta engolia o soltar: larga-se aqui"
    );
    f.quadro(&mut input, CORRE);
    assert!(
        !f.tem_o_teclado(CORRE),
        "sem clique nem Play, o foco não volta"
    );
}

#[test]
fn o_largar_de_um_clique_do_jogo_e_do_jogo() {
    use ph2d_host::PointerButton::{Middle, Primary};
    let mut f = FocoDoJogo::default();
    let mut input = InputState::new();
    f.quadro(&mut input, CORRE);
    assert!(f.clique(&mut input, CORRE, Primary, true, true));
    assert!(
        !f.clique(&mut input, CORRE, Middle, false, true),
        "outro botão"
    );
    assert!(
        f.clique(&mut input, CORRE, Primary, false, false),
        "largado fora da cena, ainda do jogo"
    );
    assert!(
        !f.clique(&mut input, CORRE, Primary, false, true),
        "uma vez só"
    );
    assert!(!f.clique(&mut input, CORRE, Primary, true, false));
    assert!(
        !f.clique(&mut input, CORRE, Primary, false, true),
        "o premido foi do editor"
    );
}

/// Report do dono 27/10: *«Pan não está funcionando quando a animação está em play»*. O botão do meio
/// é a NAVEGAÇÃO da vista (o pan) e fica do editor com o jogo a correr — o jogo vê-o na mesma.
#[test]
fn o_botao_do_meio_navega_a_vista_com_o_jogo_a_correr() {
    use ph2d_host::PointerButton::Middle;
    let mut f = FocoDoJogo::default();
    let mut input = InputState::new();
    f.quadro(&mut input, CORRE);
    assert!(!f.clique(&mut input, CORRE, Middle, true, true), "o pan");
    assert!(
        input.mouse.held(ph2d_input::MouseButton::Middle),
        "e o jogo vê-o"
    );
    assert!(!f.clique(&mut input, CORRE, Middle, false, true));
}

#[test]
fn a_tecla_do_jogo_entra_no_teclado_dele_e_so_ela() {
    let mut f = FocoDoJogo::default();
    let mut input = InputState::new();
    f.quadro(&mut input, CORRE);
    f.tecla(&mut input, CORRE, nua(true), Some(SETA), false, &"seta");
    assert!(input.keyboard.held(SETA) && input.keyboard.pressed(SETA));
    f.tecla(&mut input, CORRE, nua(false), Some(SETA), false, &"seta");
    assert!(!input.keyboard.held(SETA) && input.keyboard.released(SETA));
    let parada = FocoDoJogo::default();
    parada.tecla(&mut input, PARADA, nua(true), Some(Key(0x41)), false, &"a");
    assert!(
        !input.keyboard.held(Key(0x41)),
        "sem o jogo a ter o teclado nada desce nele"
    );
    input.begin_frame();
    f.tecla(&mut input, CORRE, nua(true), None, false, &"sem nome");
    assert_eq!(
        input.keyboard.iter_pressed().count(),
        0,
        "uma tecla sem nome nunca entra"
    );
}

#[test]
fn com_a_cena_tapada_um_clique_nao_e_so_do_jogo() {
    let mut f = FocoDoJogo::default();
    let mut input = InputState::new();
    let tapada = Cena {
        a_correr: true,
        a_vista: false,
        palco: false,
    };
    assert!(!f.premido(&mut input, tapada, true));
}

#[test]
fn com_a_cena_parada_um_clique_nao_e_do_jogo() {
    let mut f = FocoDoJogo::default();
    let mut input = InputState::new();
    assert!(!f.premido(&mut input, PARADA, true));
}

#[test]
fn com_o_jogo_a_tecla_nua_e_so_do_jogo() {
    let d = destino(nua(true), true, false);
    assert!(d.jogo && d.consumida && d.transporte.is_none());
}

#[test]
fn sem_o_jogo_nenhuma_tecla_desce_nele() {
    let d = destino(nua(true), false, false);
    assert!(!d.jogo && !d.consumida);
}

#[test]
fn a_previa_do_morph_continua_a_ler_o_teclado() {
    let d = destino(nua(true), false, true);
    assert!(d.jogo && !d.consumida, "o modal decide o resto, como antes");
}

#[test]
fn um_acorde_e_sempre_do_editor() {
    let t = Tecla {
        acorde: true,
        ..nua(true)
    };
    for (jogo_tem, previa) in [(true, false), (false, true), (true, true)] {
        let d = destino(t, jogo_tem, previa);
        assert!(!d.jogo && !d.consumida, "{jogo_tem} {previa}");
    }
}

#[test]
fn o_soltar_passa_sempre() {
    for t in [
        nua(false),
        Tecla {
            acorde: true,
            ..nua(false)
        },
    ] {
        for jogo_tem in [true, false] {
            let d = destino(t, jogo_tem, false);
            assert!(d.jogo && !d.consumida, "{t:?} {jogo_tem}");
        }
    }
}

#[test]
fn f5_e_shift_f5_sao_do_transporte_e_nunca_do_jogo() {
    let f5 = Tecla {
        f5: true,
        ..nua(true)
    };
    for jogo_tem in [true, false] {
        let d = destino(f5, jogo_tem, false);
        assert_eq!(d.transporte, Some(Transporte::Alterna));
        assert!(d.consumida && !d.jogo);
        let d = destino(Tecla { shift: true, ..f5 }, jogo_tem, true);
        assert_eq!(d.transporte, Some(Transporte::Para));
        assert!(d.consumida && !d.jogo);
        let d = destino(
            Tecla {
                desceu: false,
                ..f5
            },
            jogo_tem,
            false,
        );
        assert!(d.transporte.is_none() && !d.jogo);
        let d = destino(Tecla { repete: true, ..f5 }, jogo_tem, false);
        assert!(
            d.transporte.is_none() && d.consumida,
            "segurar o F5 não alterna em rajada"
        );
    }
}
