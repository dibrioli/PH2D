//! **A CENA DO AGARRE ELÁSTICO** (`=28`) — o `l-mode` do Grab, de Goes & James
//! 2017.
//!
//! ⚠️ **Irmã das outras cenas e não parte delas**, pelo teto de LOC da shell e
//! pela mesma linha de corte que as separa: cada uma é a história de uma wave.
//!
//! ⚠️ **A cena NÃO arma o pincel, e isso é metade do smoke.** O que esta wave
//! entrega é um CHIP que passou a existir onde não existia — e uma cena que
//! escolhesse o verbo e o modo por baixo pularia exatamente a costura que ela
//! existe para provar (a cicatriz que o `impasto_smoke` do Painter 2D prega, e
//! que o `=27` desta pasta já herdou).

/// `=28` — a cena do **AGARRE QUE RESPONDE COMO MATÉRIA**.
pub(crate) fn elastic_grab_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("28")
}

/// **Os três números que tornam a `=28` julgável, e os TRÊS saem do motor.**
///
/// ⚠️ **Um roteiro com o número escrito à mão deixa de dizer a verdade no dia em
/// que a constante se move, e ninguém fica sabendo** — é a mesma razão pela qual
/// o `=26` e o `=27` derivam os deles.
///
/// ⚠️ **E o LUGAR onde a razão é medida é parte do número.** A primeira versão
/// media a meio raio da pegada (`r = 1,5·ε`) e anunciava **83,98×** — verdadeiro,
/// e inútil: ali o barro *ao lado* já praticamente não anda, então a razão
/// dispara sobre dois deslocamentos que o artista não distingue de zero. A um
/// `ε` do centro o barro ainda anda quase metade do que o bico anda, e é lá que
/// a diferença é uma coisa que se VÊ. *Uma razão sem o lugar em que foi medida
/// não é um número, é uma afirmação.*
///
/// Devolve `(razão à frente ÷ ao lado, quanto o barro à frente anda ali, quanto
/// sobra na BORDA da pegada)`, os três como fração do deslocamento do bico.
#[must_use]
pub(crate) fn elastic_grab_numbers() -> (f32, f32, f32) {
    use ph2d_sculpt3d::kelvinlet::{Scales, grab};
    let f = [1.0, 0.0, 0.0];
    let eps = 1.0 / ph2d_sculpt3d::KELVINLET_REACH;
    let len = |v: [f32; 3]| (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    // A UM `ε` do centro — um terço do raio da pegada.
    let ahead = len(grab([eps, 0.0, 0.0], eps, f, Scales::default()));
    let beside = len(grab([0.0, eps, 0.0], eps, f, Scales::default()));
    // Na borda exata da pegada, no pior caso (à frente).
    let rim = len(grab([1.0, 0.0, 0.0], eps, f, Scales::default()));
    (ahead / beside, ahead, rim)
}

/// O roteiro da `=28`.
pub(crate) fn announce() {
    if !elastic_grab_scene() {
        return;
    }
    let (aniso, ahead, rim) = elastic_grab_numbers();
    let (ahead_pct, rim_pct) = (ahead * 100.0, rim * 100.0);
    eprintln!(
        "[sculpt3d] =28 O AGARRE QUE RESPONDE COMO MATERIA (l-mode do Grab).\n\
         [sculpt3d]    A lei do s-mode e' 'gesto x escalar': TODO vertice da pegada anda na\n\
         [sculpt3d]    MESMA direcao, e o falloff so' decide quanto. Um Kelvinlet e' a solucao\n\
         [sculpt3d]    fundamental da elasticidade -- o barro a' FRENTE do puxao acompanha mais\n\
         [sculpt3d]    que o barro ao LADO dele, e uma curva nao tem como dizer isso.\n\
         [sculpt3d]    Abra o painel com a CRASE (`) e escolha o verbo Grab.\n\
         [sculpt3d]    (1) O CHIP. Com o Grab em maos, a linha de modo tem de mostrar DOIS\n\
         [sculpt3d]        botoes: S e L. Se o L nao estiver la', PARE.\n\
         [sculpt3d]    (2) O CONTROLE, e faca-o PRIMEIRO. Em S, puxe uma alca comprida de lado.\n\
         [sculpt3d]        Guarde a forma dela na cabeca (ou Ctrl+Z e repita).\n\
         [sculpt3d]    (3) Agora em L, o MESMO gesto. O vertice sob o cursor segue o dedo\n\
         [sculpt3d]        exatamente igual -- e' a normalizacao, e e' o que faz os dois modos\n\
         [sculpt3d]        comparaveis. O que muda e' a VIZINHANCA: a UM TERCO do raio -- onde\n\
         [sculpt3d]        o barro ainda anda {ahead_pct:.0}% do que o bico anda -- o que esta' a' FRENTE\n\
         [sculpt3d]        do puxao acompanha {aniso:.2}x o que esta' ao LADO. A alca sai parecida\n\
         [sculpt3d]        com material esticado, e nao com um cone.\n\
         [sculpt3d]    (4) A BORDA da pegada. Na borda ainda sobra {rim_pct:.1}% do puxao -- o campo\n\
         [sculpt3d]        tem suporte infinito e o cursor tem raio, e este e' o preco nomeado.\n\
         [sculpt3d]        Procure um DEGRAU no anel do cursor: ele tem de ser menor que a\n\
         [sculpt3d]        distancia entre vertices, ou seja invisivel. Se voce vir um degrau,\n\
         [sculpt3d]        reporte -- e' o unico numero desta wave que a malha pode desmentir.\n\
         [sculpt3d]    (5) O L NAO aparece em mais nenhum verbo de geometria (so' no Smooth, que\n\
         [sculpt3d]        e' o Taubin da wave anterior). Se ele aparecer no Draw ou no Clay,\n\
         [sculpt3d]        PARE: um chip que promete uma fonte e nao tem lei dela e' o defeito\n\
         [sculpt3d]        que o modo L existe para nao ter."
    );
}
