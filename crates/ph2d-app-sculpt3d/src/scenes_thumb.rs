//! **A CENA DO POLEGAR** (`=30`) — o `Verb::ClayThumb`, a primeira ferramenta
//! cujo resultado depende de **quantos dabs já passaram**.
//!
//! ⚠️ **Irmã das outras cenas e não parte delas**, pelo teto de LOC da shell e
//! pela mesma linha de corte: cada arquivo é a história de uma wave.
//!
//! ⚠️ **A cena NÃO arma o pincel, e isso é metade do smoke** — a mesma cicatriz
//! que o `impasto_smoke` do Painter 2D prega e que a `=28`/`=29` já herdaram: a
//! wave entrega uma ferramenta NOVA na lista, e uma cena que a escolhesse por
//! baixo do pano pularia justamente a costura que ela existe para provar.

/// `=30` — a cena do **POLEGAR**.
pub(crate) fn clay_thumb_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("30")
}

/// **Os números que tornam a `=30` julgável, e os TRÊS saem do motor.**
///
/// ⚠️ **Um roteiro com o número escrito à mão deixa de dizer a verdade no dia em
/// que a constante se move, e ninguém fica sabendo** — a mesma razão das cenas
/// `=26` a `=29`.
///
/// Devolve `(graus por dab, teto em graus, quantos dabs até o teto)`.
#[must_use]
pub(crate) fn thumb_numbers() -> (f32, f32, u32) {
    let step = ph2d_sculpt3d::CLAY_THUMB_TILT_STEP_DEG;
    let max = ph2d_sculpt3d::CLAY_THUMB_TILT_MAX_DEG;
    // ⚠️ **DERIVADO, e é o que torna o roteiro auditável:** o `75` é uma
    // consequência dos dois literais da referência, não um terceiro número. Se
    // alguém mover um deles, o roteiro passa a mandar o artista contar outro
    // tanto de dabs, sozinho.
    let dabs = (max / step).ceil() as u32;
    (step, max, dabs)
}

/// O roteiro da `=30`.
pub(crate) fn announce() {
    if !clay_thumb_scene() {
        return;
    }
    let (step, max, dabs) = thumb_numbers();
    eprintln!(
        "[sculpt3d] =30 O POLEGAR (Clay Thumb).\n\
         [sculpt3d]    E' a primeira ferramenta cujo resultado depende de QUANTOS dabs ja'\n\
         [sculpt3d]    passaram, e nao so' de onde este caiu: o plano em que ela achata vai\n\
         [sculpt3d]    se INCLINANDO ao longo do traco, {step:.1} grau por dab, ate' um teto\n\
         [sculpt3d]    de {max:.0} graus -- que chega por volta do {dabs}o. dab.\n\
         [sculpt3d]    Abra o painel com a CRASE (`) e escolha o verbo Clay Thumb.\n\
         [sculpt3d]    (1) O CHIP. O polegar tem de estar na lista de ferramentas. Se nao\n\
         [sculpt3d]        estiver, PARE.\n\
         [sculpt3d]    (2) O CONTROLE, e faca-o PRIMEIRO. Com o FLATTEN, passe uma vez, num\n\
         [sculpt3d]        traco LONGO e reto. Guarde a forma: ele achata, e o que ele deixa\n\
         [sculpt3d]        fica ali -- o corte nao vira ao longo do caminho.\n\
         [sculpt3d]    (3) Agora o polegar, o MESMO gesto. No comeco do traco ele parece o\n\
         [sculpt3d]        Flatten; conforme a mao anda, o corte vai DEITANDO na direcao do\n\
         [sculpt3d]        movimento -- e' o barro a ser empurrado por um polegar, nao raspado\n\
         [sculpt3d]        por uma espatula. Se o traco inteiro sair igual ao Flatten, reporte.\n\
         [sculpt3d]    (4) O TOQUE. Um clique SEM arrastar tem de fazer NADA. Nao e' bug: sem\n\
         [sculpt3d]        caminho nao ha' eixo para inclinar, e a referencia recusa pela mesma\n\
         [sculpt3d]        razao. Se um toque parado deformar, reporte.\n\
         [sculpt3d]    (5) O TETO. Continue o mesmo traco, bem longo, passando dos {dabs} dabs.\n\
         [sculpt3d]        A inclinacao tem de PARAR de crescer, e a superficie tem de parar de\n\
         [sculpt3d]        mudar -- projetar num plano e' auto-limitado. Se ela continuar a\n\
         [sculpt3d]        cavar sem fim, reporte.\n\
         [sculpt3d]    (6) O TRACO NOVO. Solte e comece OUTRO traco ao lado. Ele tem de nascer\n\
         [sculpt3d]        do zero -- suave no comeco, como o primeiro. Se o segundo ja' comecar\n\
         [sculpt3d]        deitado, a inclinacao vazou entre tracos: reporte.\n\
         [sculpt3d]    (7) O ESPELHO. Ligue a simetria e repita. Os dois lados tem de ficar\n\
         [sculpt3d]        IGUAIS entre si e iguais ao lado unico do passo (3) -- o espelho nao\n\
         [sculpt3d]        pode acelerar a inclinacao."
    );
}
