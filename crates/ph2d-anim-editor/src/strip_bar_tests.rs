use super::*;

fn doc(content: f32) -> AnimDoc {
    let mut d = AnimDoc::new(1, "x".into());
    d.layout.area = Rect::new(0.0, 0.0, 1000.0, 1000.0);
    d.layout.strip = Rect::new(100.0, 100.0, 400.0, 80.0);
    d.layout.strip_bar = Rect::new(100.0, 190.0, 400.0, SCROLLBAR_W);
    d.layout.strip_content = content;
    d
}

/// O polegar mede a parte visível e anda de ponta a ponta. *Mutação: largura = trilho ⇒ 400; sem o `max` do fim ⇒
/// o polegar sai do trilho.*
#[test]
fn o_polegar_mede_a_parte_visivel_e_anda_de_ponta_a_ponta() {
    let track = Rect::new(0.0, 0.0, 400.0, SCROLLBAR_W);
    assert_eq!(
        thumb(track, 0.0, 400.0, 400.0),
        None,
        "cabe tudo: sem barra"
    );
    let t = thumb(track, 0.0, 800.0, 400.0).unwrap();
    assert_eq!((t.x, t.w), (0.0, 200.0));
    let t = thumb(track, 400.0, 800.0, 400.0).unwrap();
    assert_eq!(t.x + t.w, 400.0, "no fim, encosta à direita");
}

/// ⭐ Agarrar o polegar e arrastar rola a tira na mesma proporção; clicar no trilho salta para lá; soltar larga.
/// *Mutações: `bar_press` sempre `false` ⇒ o scroll fica 0; `release` sem largar ⇒ o grab fica.*
#[test]
fn arrastar_o_polegar_e_clicar_no_trilho_rolam_a_tira() {
    let mut d = doc(800.0);
    assert!(d.press(150.0, 195.0), "agarra o polegar (100..300)");
    assert!(d.motion(250.0, 195.0));
    assert!(
        (d.strip_scroll - 200.0).abs() < 0.01,
        "100 px de trilho = 200 px de tira"
    );
    d.release(250.0, 195.0);
    assert_eq!(d.bar_grab, None);
    // O trilho: o polegar centra-se no ponteiro, aqui no fim.
    assert!(d.press(495.0, 195.0));
    assert!((d.strip_scroll - 400.0).abs() < 0.01);
    d.release(495.0, 195.0);
    // CONTROLO: sem nada a rolar, a barra não pega o clique.
    let mut d = doc(400.0);
    d.bar_press(150.0, 195.0);
    assert_eq!((d.strip_scroll, d.bar_grab), (0.0, None));
}

/// O rótulo da ranhura num quadro baixo (janela 1366 × 768, `fotos_31`): «INSERT HERE» não cabe em 58 px, as duas palavras
/// cabem lado a lado nos 28 px do vão. *Mutação: sempre a frase inteira ⇒ vermelho.*
#[test]
fn o_rotulo_da_ranhura_parte_se_quando_nao_cabe() {
    let (inteira, palavras) = (
        ("INSERT HERE", 80.0),
        vec![("INSERT", 45.0), ("HERE", 36.0)],
    );
    let baixo = super::colunas_da_ranhura(inteira, palavras.clone(), (28.0, 57.8), 12.0);
    assert_eq!(baixo, palavras, "partido em duas colunas");
    let alto = super::colunas_da_ranhura(inteira, palavras.clone(), (28.0, 137.8), 12.0);
    assert_eq!(alto, vec![inteira], "cabe numa coluna");
    let estreito = super::colunas_da_ranhura(inteira, palavras, (20.0, 57.8), 12.0);
    assert_eq!(
        estreito,
        vec![inteira],
        "duas colunas não cabem: a frase, cortada"
    );
}
