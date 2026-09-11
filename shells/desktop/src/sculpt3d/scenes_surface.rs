//! **A CENA DA SUPERFÍCIE LOCAL** (`=32`) — o `l-mode` dos quatro verbos de
//! plano (Alexa et al. 2003), a W7.
//!
//! ⚠️ **Irmã das outras cenas e não parte delas**, pelo teto de LOC da shell e
//! pela mesma linha de corte: cada arquivo é a história de uma wave.
//!
//! ⚠️ **A cena NÃO arma o modo, e isso é METADE do smoke** — a mesma cicatriz
//! que as `=28`/`=29`/`=30` herdaram do `impasto_smoke` do Painter 2D. A wave
//! entrega um chip novo no seletor de MODO, e uma cena que o escolhesse por
//! baixo do pano pularia exactamente a costura que ela existe para provar.

/// `=32` — a cena da **SUPERFÍCIE LOCAL**.
pub(crate) fn local_surface_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("32")
}

/// O roteiro da `=32`.
///
/// ⚠️ **A pergunta é de OLHO e ela NÃO é *"mexeu?"***: os dois modos achatam. O
/// que os separa é o que sobra da FORMA — medido, um dab de raio `0,4` numa
/// esfera de raio 1 põe a superfície local até **10,3% do raio do pincel** acima
/// do plano, e essa é a distância que o artista vê entre uma faceta e uma
/// carícia.
pub(crate) fn announce() {
    if !local_surface_scene() {
        return;
    }
    eprintln!(
        "[sculpt3d] =32 A SUPERFICIE LOCAL (o l-mode de Flatten/Fill/Scrape/Clay).\n\
         [sculpt3d]    Ate' hoje os quatro verbos de plano projetavam num PLANO: passar o\n\
         [sculpt3d]    Flatten numa superficie curva CORTAVA UMA FACETA. Sob o modo L eles\n\
         [sculpt3d]    projetam numa SUPERFICIE ajustada ao barro -- o detalhe sai, a forma\n\
         [sculpt3d]    fica. Medido, o alvo desloca-se ate' 10,3% do raio do pincel.\n\
         [sculpt3d]    Abra o painel com a CRASE (`), pegue o FLATTEN e ache o seletor de MODO.\n\
         [sculpt3d]    (1) O CHIP. O modo L tem de aparecer no Flatten, no Fill, no Scrape e no\n\
         [sculpt3d]        Clay. Se nao aparecer, PARE -- o resto nao diz nada.\n\
         [sculpt3d]    (2) O CONTROLE, e faca-o PRIMEIRO. Modo S, Flatten, um traco na BARRIGA\n\
         [sculpt3d]        da esfera (nao no polo). Ele corta um CHANFRO: uma faceta reta, com\n\
         [sculpt3d]        quinas onde ela encontra a curva. Guarde essa imagem.\n\
         [sculpt3d]    (3) Agora o modo L, o MESMO gesto ao lado. A esfera tem de continuar\n\
         [sculpt3d]        REDONDA ali -- sem faceta, sem quina. Se as duas passadas sairem\n\
         [sculpt3d]        iguais, o chip nao chegou ao alvo: reporte.\n\
         [sculpt3d]    (4) E ELE AINDA ACHATA -- este e' o passo que prova que a wave nao\n\
         [sculpt3d]        quebrou o verbo. Faca uma RUGA (Draw com dabs curtos, ou Noise) e\n\
         [sculpt3d]        passe o Flatten em L por cima. A ruga tem de SUMIR e a curva grande\n\
         [sculpt3d]        tem de FICAR. Se a ruga sobreviver, o alvo esta' a seguir o detalhe:\n\
         [sculpt3d]        reporte.\n\
         [sculpt3d]    (5) ONDE E' PLANO, OS DOIS SAO IGUAIS. Ache uma regiao ja' achatada (a\n\
         [sculpt3d]        faceta do passo 2 serve) e passe S e depois L. Ali eles tem de fazer\n\
         [sculpt3d]        a MESMA coisa -- e' a lei do modo: outra lei sobre a mesma superficie.\n\
         [sculpt3d]    (6) O OFFSET. Com o L em maos, mexa no Plane Offset. Ele tem de continuar\n\
         [sculpt3d]        a funcionar: negativo cava mais, positivo deixa de raspar. Se o knob\n\
         [sculpt3d]        ficar INERTE sob o L, reporte -- foi exatamente o defeito que uma\n\
         [sculpt3d]        mutacao pegou nesta wave.\n\
         [sculpt3d]    (7) OS OUTROS TRES. Repita (2) e (3) com Fill, Scrape e Clay. Os quatro\n\
         [sculpt3d]        tem a mesma lei; se um deles ignorar o chip, reporte qual."
    );
}
