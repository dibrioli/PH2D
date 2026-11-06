use super::*;
use ph2d_host::Modifiers;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

const AREA: Rect = Rect::new(0.0, 0.0, 1000.0, 700.0);

fn objetos(n: u64) -> Vec<(u64, String)> {
    (0..n).map(|i| (i, format!("Object {i:02}"))).collect()
}

fn nomes() -> Vec<(u64, String)> {
    ["Big Crate", "Coin", "Crate", "Ears"]
        .iter()
        .enumerate()
        .map(|(i, n)| (i as u64, (*n).to_owned()))
        .collect()
}

fn pinta(j: &mut ObjectPicker) {
    let mut ts = TextSystem::without_system_fonts();
    let mut scene = VectorScene::new();
    let t = Textos {
        titulo: "Use in Game",
        subtitulo: "Pick",
        aviso: None,
    };
    j.paint(AREA, &t, &mut scene, &mut ts, Theme::Forge);
}

fn tecla(j: &mut ObjectPicker, key: Key, text: Option<&str>) -> Option<(Saida, Option<String>)> {
    j.tecla(&KeyInput {
        key,
        mods: Modifiers::default(),
        pressed: true,
        text,
        paste: None,
    })
}

fn escreve(j: &mut ObjectPicker, s: &str) {
    for c in s.chars() {
        tecla(j, Key::Char(c), Some(c.encode_utf8(&mut [0; 4])));
    }
}

fn centro(j: &ObjectPicker, a: Alvo) -> (f32, f32) {
    let (_, r) = j.alvos().find(|(x, _)| *x == a).expect("não foi pintado");
    (r.x + r.w / 2.0, r.y + r.h / 2.0)
}

fn clica(j: &mut ObjectPicker, a: Alvo) -> Saida {
    let (x, y) = centro(j, a);
    assert_eq!(j.ponteiro(PointerKind::Down, x, y), Saida::Nada);
    j.ponteiro(PointerKind::Up, x, y)
}

#[test]
fn a_busca_filtra_sem_ligar_para_maiuscula_e_quem_comeca_vem_primeiro() {
    let mut j = ObjectPicker::new(nomes(), None);
    assert_eq!(j.nomes_vistos().len(), 4);
    escreve(&mut j, "CRA");
    assert_eq!(j.nomes_vistos(), ["Crate", "Big Crate"]);
    assert_eq!(j.destacado(), Some(2), "o destaque não foi para o primeiro");
    tecla(&mut j, Key::Backspace, None);
    tecla(&mut j, Key::Backspace, None);
    tecla(&mut j, Key::Backspace, None);
    assert_eq!(j.nomes_vistos().len(), 4, "apagar não devolveu a lista");
    escreve(&mut j, "zzz");
    assert!(j.nomes_vistos().is_empty());
    assert_eq!(tecla(&mut j, Key::Enter, None), Some((Saida::Nada, None)));
}

#[test]
fn o_objeto_de_antes_nasce_destacado_e_enter_o_confirma() {
    let mut j = ObjectPicker::new(nomes(), Some(2));
    assert_eq!(
        tecla(&mut j, Key::Enter, None),
        Some((Saida::Escolheu(2), None))
    );
}

#[test]
fn as_setas_andam_e_a_lista_rola_para_mostrar_o_destaque() {
    let mut j = ObjectPicker::new(objetos(30), None);
    pinta(&mut j);
    for _ in 0..20 {
        tecla(&mut j, Key::Down, None);
    }
    assert_eq!(j.destacado(), Some(20));
    pinta(&mut j);
    assert!(
        j.alvos().any(|(a, _)| a == Alvo::Objeto(20)),
        "o destaque ficou fora da lista"
    );
    for _ in 0..40 {
        tecla(&mut j, Key::Up, None);
    }
    assert_eq!(j.destacado(), Some(0));
}

#[test]
fn a_roda_rola_sem_passar_do_fim() {
    let mut j = ObjectPicker::new(objetos(30), None);
    pinta(&mut j);
    let (x, y) = centro(&j, Alvo::Objeto(0));
    for _ in 0..100 {
        j.roda(x, y, -1.0);
    }
    pinta(&mut j);
    assert!(j.alvos().any(|(a, _)| a == Alvo::Objeto(29)));
    assert!(!j.alvos().any(|(a, _)| a == Alvo::Objeto(0)));
}

#[test]
fn clicar_escolhe_e_clicar_fora_ou_em_cancelar_fecha() {
    let mut j = ObjectPicker::new(nomes(), None);
    pinta(&mut j);
    assert_eq!(clica(&mut j, Alvo::Objeto(1)), Saida::Escolheu(1));
    assert_eq!(clica(&mut j, Alvo::Cancelar), Saida::Fecha);
    assert_eq!(clica(&mut j, Alvo::Busca), Saida::Nada);
    assert_eq!(j.ponteiro(PointerKind::Down, 1.0, 1.0), Saida::Fecha);
}

#[test]
fn esc_desarma_o_conta_gotas_antes_de_fechar() {
    let mut j = ObjectPicker::new(nomes(), None);
    pinta(&mut j);
    assert_eq!(clica(&mut j, Alvo::ContaGotas), Saida::Nada);
    assert!(j.armado());
    assert_eq!(tecla(&mut j, Key::Escape, None), Some((Saida::Nada, None)));
    assert!(!j.armado());
    assert_eq!(tecla(&mut j, Key::Escape, None), Some((Saida::Fecha, None)));
}

#[test]
fn espaco_e_delete_sao_do_campo_e_atalho_do_app_passa() {
    let mut j = ObjectPicker::new(nomes(), None);
    escreve(&mut j, "a b");
    assert_eq!(j.busca.text, "a b");
    tecla(&mut j, Key::Space, None);
    assert_eq!(j.busca.text, "a b ");
    tecla(&mut j, Key::Home, None);
    tecla(&mut j, Key::Delete, None);
    assert_eq!(j.busca.text, " b ");
    let ctrl_s = j.tecla(&KeyInput {
        key: Key::Char('s'),
        mods: Modifiers {
            ctrl: true,
            ..Modifiers::default()
        },
        pressed: true,
        text: None,
        paste: None,
    });
    assert_eq!(ctrl_s, None);
}

#[test]
fn a_letra_que_vem_pela_composicao_filtra() {
    let mut j = ObjectPicker::new(nomes(), None);
    assert!(j.compose(&Compose::Commit('E')));
    assert!(j.compose(&Compose::Commit('a')));
    assert_eq!(j.nomes_vistos(), ["Ears"]);
}

#[test]
fn a_cena_vazia_e_a_busca_sem_resultado_pintam_sem_alvo_de_objeto() {
    for mut j in [ObjectPicker::new(Vec::new(), None), {
        let mut j = ObjectPicker::new(nomes(), None);
        escreve(&mut j, "zzz");
        j
    }] {
        pinta(&mut j);
        assert!(!j.alvos().any(|(a, _)| matches!(a, Alvo::Objeto(_))));
        assert_eq!(j.destacado(), None);
    }
}

#[test]
fn numa_aba_baixa_a_janela_cabe_com_uma_linha_ao_menos() {
    let mut j = ObjectPicker::new(objetos(30), None);
    let mut ts = TextSystem::without_system_fonts();
    let mut scene = VectorScene::new();
    let t = Textos {
        titulo: "T",
        subtitulo: "S",
        aviso: Some("A"),
    };
    let baixa = Rect::new(0.0, 0.0, 300.0, 100.0);
    j.paint(baixa, &t, &mut scene, &mut ts, Theme::Forge);
    assert_eq!(j.linhas, 1);
    assert!(j.caixa.w <= baixa.w);
}
