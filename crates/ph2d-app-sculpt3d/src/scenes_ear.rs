//! **A CENA DA ORELHA** (`=36`) — a retopologia sobre um vinco CÔNCAVO fundo.
//!
//! ⚠️ **Ela existe porque nenhuma das outras fixturas continha o fenómeno.** A
//! `=35` abre na esfera amassada (sulcos rasos), a do bico tem uma protuberância
//! esticada, a das cristas tem relevo convexo. O que o artista fotografou em
//! 2026-08-22 foi outra coisa: **uma borda saliente com um vinco fundo e côncavo
//! colado a ela** — a geometria de uma orelha.
//!
//! ⭐ **É a espécie de feição que quebra três coisas ao mesmo tempo**, e por isso
//! ela merece uma cena própria:
//!
//! | o quê | por que a orelha o expõe |
//! |---|---|
//! | a **projeção pelo ponto mais próximo** | dentro de um vinco côncavo o pé mais próximo pode estar do outro lado da dobra |
//! | a **remalha isotrópica** do F1 | a espessura da borda é menor que `α × diagonal` ⇒ a feição desaparece antes de o traçado a ver |
//! | o **campo cruzado** | as direções principais giram ao atravessar a crista |

/// `=36` — a cena da **ORELHA**.
pub(crate) fn ear_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("36")
}

/// O roteiro da `=36`.
pub(crate) fn announce() {
    if !ear_scene() {
        return;
    }
    eprintln!(
        "[sculpt3d] =36 A ORELHA -- a retopologia sobre um vinco FUNDO.\n\
         [sculpt3d]    Esta peca tem uma borda levantada com um vinco cavado colado a ela, que\n\
         [sculpt3d]    e' o formato que mais quebra uma retopologia. A =35 nao tinha nada assim.\n\
         [sculpt3d]    Abra o painel com a CRASE (`) e ache a secao Topology.\n\
         [sculpt3d]    (1) OLHE A PECA ANTES. Gire ate' ver a orelha de frente e repare no\n\
         [sculpt3d]        sulco fundo entre a borda e a concha. E' ele que tem de sobreviver.\n\
         [sculpt3d]    (2) Em `Engine` escolha `Even Grid`, ponha `Resolution Detail` no MAXIMO\n\
         [sculpt3d]        e clique em `Quad Retopology`. Pode levar alguns segundos.\n\
         [sculpt3d]    (3) O terminal imprime quantos quads sairam e QUANTAS FACES DOBRARAM.\n\
         [sculpt3d]        Se a linha de faces dobradas nao aparecer, nenhuma dobrou.\n\
         [sculpt3d]    (4) OLHE O SULCO. Ele tem de continuar la', com a mesma profundidade. Se\n\
         [sculpt3d]        a orelha virou um calombo liso, PARE e diga -- e' a 3a foto de 22/08.\n\
         [sculpt3d]    (5) PROCURE RASGOS junto a' borda: um risco escuro, faces esmagadas. E' a\n\
         [sculpt3d]        2a foto de 22/08. O Ctrl+Z devolve a peca de antes.\n\
         [sculpt3d]    (6) Repita com `Engine` = `Fast` para comparar os dois motores na MESMA\n\
         [sculpt3d]        peca."
    );
}
