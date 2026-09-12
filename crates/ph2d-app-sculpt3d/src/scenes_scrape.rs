//! **A CENA DA LÂMINA EM V** (`=31`) — o `Verb::MultiplaneScrape`, o único verbo
//! com DOIS planos.
//!
//! ⚠️ **Irmã das outras cenas e não parte delas**, pelo teto de LOC da shell e
//! pela mesma linha de corte: cada arquivo é a história de uma wave.
//!
//! ⚠️ **A cena NÃO arma o pincel**, a mesma cicatriz que o `impasto_smoke` do
//! Painter 2D prega e que as cenas `=28` a `=30` já herdaram — a wave entrega
//! uma ferramenta NOVA na lista, e uma cena que a escolhesse por baixo do pano
//! pularia justamente a costura que ela existe para provar.

/// `=31` — a cena da **LÂMINA EM V**.
pub(crate) fn multiplane_scrape_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("31")
}

/// **Os números que tornam a `=31` julgável, e os DOIS saem do motor.**
///
/// ⚠️ **Um roteiro com o número escrito à mão deixa de dizer a verdade no dia em
/// que a constante se move, e ninguém fica sabendo** — a mesma razão das cenas
/// `=26` a `=30`.
///
/// Devolve `(o ângulo de fábrica, o teto do knob)`.
#[must_use]
pub(crate) fn scrape_numbers() -> (f32, f32) {
    (
        ph2d_sculpt3d::DEFAULT_MULTIPLANE_ANGLE_DEG,
        ph2d_sculpt3d::MULTIPLANE_ANGLE_MAX_DEG,
    )
}

/// O roteiro da `=31`.
pub(crate) fn announce() {
    if !multiplane_scrape_scene() {
        return;
    }
    let (deg, max) = scrape_numbers();
    eprintln!(
        "[sculpt3d] =31 A LAMINA EM V (Multiplane Scrape).\n\
         [sculpt3d]    E' o unico verbo com DOIS planos: em vez de raspar contra uma\n\
         [sculpt3d]    superficie, ele raspa contra um TELHADO, e o que sobra e' um sulco de\n\
         [sculpt3d]    duas facetas planas com uma ARESTA VIVA no meio -- e' assim que se faz\n\
         [sculpt3d]    um vinco duro. De fabrica o V abre {deg:.0} graus (o knob vai ate' {max:.0}).\n\
         [sculpt3d]    Abra o painel com a CRASE (`) e escolha o verbo Multiplane Scrape.\n\
         [sculpt3d]    (1) O CHIP. A lamina tem de estar na lista de ferramentas. Se nao\n\
         [sculpt3d]        estiver, PARE.\n\
         [sculpt3d]    (2) O CONTROLE, e faca-o PRIMEIRO. Com o SCRAPE, passe um traco longo e\n\
         [sculpt3d]        reto. Guarde a forma: ele deixa um CANAL de fundo chato.\n\
         [sculpt3d]    (3) Agora a lamina, o MESMO gesto. O fundo deixa de ser chato: nasce uma\n\
         [sculpt3d]        CRISTA correndo no meio do sulco, com uma faceta plana de cada lado.\n\
         [sculpt3d]        Se sair igual ao Scrape, reporte.\n\
         [sculpt3d]    (4) O ANGULO. Ponha 'Plane angle' em ZERO e repita: a ferramenta tem de\n\
         [sculpt3d]        fazer NADA. Nao e' bug -- com o V fechado os dois planos coincidem\n\
         [sculpt3d]        com o plano TANGENTE ao cursor, e acima dele nao ha' barro nenhum.\n\
         [sculpt3d]        Suba para {deg:.0} de novo e o sulco volta.\n\
         [sculpt3d]    (5) A DOBRADICA. Faca um traco na VERTICAL. A crista tem de correr na\n\
         [sculpt3d]        vertical tambem: o V abre ATRAVESSANDO o traco, e a dobradica e' o\n\
         [sculpt3d]        proprio caminho. Se ela sair atravessada, reporte.\n\
         [sculpt3d]    (6) O TOQUE. Um clique SEM arrastar tem de fazer NADA -- sem caminho nao\n\
         [sculpt3d]        ha' dobradica, e a referencia recusa pela mesma razao.\n\
         [sculpt3d]    (7) O CTRL. Segure Ctrl e repita: o telhado vira VALE. Ele nao fica mais\n\
         [sculpt3d]        fraco -- ele se inverte, e passa a ENCHER a dobra em vez de a cavar.\n\
         [sculpt3d]    (8) LER A SUPERFICIE. Marque 'Read the surface' e ponha o angulo em ZERO.\n\
         [sculpt3d]        Agora ele AINDA corta: a abertura do V esta' sendo lida da forma que\n\
         [sculpt3d]        esta' debaixo do pincel. Passe ao longo de um vinco que voce ja' tenha\n\
         [sculpt3d]        feito -- a lamina acompanha o vinco em vez de impor um proprio.\n\
         [sculpt3d]    (9) O CTRL, no modo dinamico, MUDA DE SIGNIFICADO: em vez de inverter, ele\n\
         [sculpt3d]        ACHATA (angulo zero). E' de proposito -- e' como se apara uma\n\
         [sculpt3d]        superficie plana sem trocar de ferramenta."
    );
}
