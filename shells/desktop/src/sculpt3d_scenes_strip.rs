//! **A CENA DA FAIXA** (`=29`) — o `Verb::ClayStrips`, a primeira ferramenta
//! cujo dab não é um disco.
//!
//! ⚠️ **Irmã das outras cenas e não parte delas**, pelo teto de LOC da shell e
//! pela mesma linha de corte: cada arquivo é a história de uma wave.
//!
//! ⚠️ **A cena NÃO arma o pincel, e isso é metade do smoke.** O que esta wave
//! entrega é uma ferramenta NOVA na lista — uma cena que a escolhesse por baixo
//! do pano pularia exatamente a costura que ela existe para provar (a cicatriz
//! que o `impasto_smoke` do Painter 2D prega, e que a `=28` já herdou).

/// `=29` — a cena da **FAIXA**.
pub(crate) fn clay_strips_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("29")
}

/// **Os três números que tornam a `=29` julgável, e os TRÊS saem do motor.**
///
/// ⚠️ **Um roteiro com o número escrito à mão deixa de dizer a verdade no dia em
/// que a constante se move, e ninguém fica sabendo** — a mesma razão das cenas
/// `=26`, `=27` e `=28`.
///
/// Devolve `(quantos raios a faixa alcança na quina, onde o portão de
/// profundidade tem o pico em raios abaixo do plano, quanto o verbo ergue o
/// próprio plano em raios)`.
#[must_use]
pub(crate) fn strip_numbers() -> (f32, f32, f32) {
    let corner = ph2d_sculpt3d::Footprint::strip_query_factor(
        ph2d_sculpt3d::Brush::default().strip_length,
    );
    // O pico de `z·(1−z)` está em `z = 1/2`, e o lugar é o número: derivá-lo do
    // kernel em vez de o escrever mantém o roteiro honesto se a lei mudar.
    let peak = {
        let s = ph2d_sculpt3d::Strip::new(
            [0.0; 3],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 0.0],
            1.0,
            1.0,
            1.0,
        )
        .expect("há plano");
        let mut best = (0.0f32, 0.0f32);
        for i in 0..=100 {
            let z = -(i as f32) / 100.0;
            let g = s.at([0.0, 0.0, z]).1;
            if g > best.1 {
                best = (-z, g);
            }
        }
        best.0
    };
    (corner, peak, ph2d_sculpt3d::STRIP_PLANE_FRACTION)
}

/// O roteiro da `=29`.
pub(crate) fn announce() {
    if !clay_strips_scene() {
        return;
    }
    let (corner, peak, lift) = strip_numbers();
    eprintln!(
        "[sculpt3d] =29 O DAB QUE NAO E' UM DISCO (Clay Strips).\n\
         [sculpt3d]    Ate' aqui TODA ferramenta media 'distancia ao centro / raio': um disco,\n\
         [sculpt3d]    sempre, e por isso o catalogo inteiro tinha a mesma silhueta. A faixa\n\
         [sculpt3d]    tem miolo CHATO numa caixa arredondada, deitada na direcao em que a mao\n\
         [sculpt3d]    vai, e um portao parabolico na profundidade que a faz DEPOSITAR barro\n\
         [sculpt3d]    abaixo do plano em vez de levantar o que ja' esta' no lugar.\n\
         [sculpt3d]    Abra o painel com a CRASE (`) e escolha o verbo Clay Strips.\n\
         [sculpt3d]    (1) O CHIP. A faixa tem de estar na lista de ferramentas. Se nao\n\
         [sculpt3d]        estiver, PARE.\n\
         [sculpt3d]    (2) O CONTROLE, e faca-o PRIMEIRO. Com o Draw, passe uma vez pela\n\
         [sculpt3d]        esfera. Guarde a forma: um DOMO, redondo, mais alto no meio.\n\
         [sculpt3d]    (3) Agora a faixa, o MESMO gesto. O que ela deixa e' uma TIRA de barro\n\
         [sculpt3d]        com topo chato e ombro definido -- uma prancha, nao um monte.\n\
         [sculpt3d]    (4) A DIRECAO. Trace uma CURVA em S. A tira tem de acompanhar a curva:\n\
         [sculpt3d]        ela deita no caminho, e nao num angulo fixo. Se ela ficar sempre no\n\
         [sculpt3d]        mesmo angulo, reporte.\n\
         [sculpt3d]    (5) O TOQUE. Um clique SEM arrastar nao tem caminho -- e ali a ponta e'\n\
         [sculpt3d]        REDONDA de proposito, porque a ferramenta ainda nao sabe para onde\n\
         [sculpt3d]        voce ia. Um toque que saisse deitado num angulo arbitrario seria o\n\
         [sculpt3d]        bug; um toque redondo e' a decisao.\n\
         [sculpt3d]    (6) A ESPESSURA. A faixa deposita o que esta' ate' UM raio abaixo do\n\
         [sculpt3d]        plano, com o pico a {peak:.2} raio -- e ela ergue o proprio plano\n\
         [sculpt3d]        {lift:.2} raio, que e' exatamente o offset que poe esse pico na\n\
         [sculpt3d]        superficie em repouso. Passar duas vezes CONSTROI (o plano sobe com\n\
         [sculpt3d]        o barro, como no Clay); o que ela nao faz e' inchar sem limite num\n\
         [sculpt3d]        ponto so'.\n\
         [sculpt3d]    (7) AS QUINAS. A pegada alcanca {corner:.2} raios na quina, e a consulta\n\
         [sculpt3d]        cresce junto. Procure um CANTO COMIDO -- uma tira cujo fim parece\n\
         [sculpt3d]        cortado em arco em vez de reto. Se vir, reporte: e' o unico numero\n\
         [sculpt3d]        desta wave que a malha pode desmentir."
    );
}
