//! **A CENA DO PINCEL DE PLANO** (`=47`) — aparar, encher e achatar com um
//! pincel só.
//!
//! # ⚠️ Ela abre num CAMPO DE BOSSAS, e a escolha é a lição da cena
//!
//! Numa esfera lisa este pincel não tem o que mostrar: ele **pára sozinho quando
//! a superfície fica plana** (o auto-limite da espec §8, medido em `0` de `2 401`
//! vértices movidos numa superfície já plana), e o artista veria uma ferramenta
//! que *«não faz nada»*. O que ele apara é **relevo**, e é preciso haver relevo.
//!
//! ⭐⭐ **E as bossas são a peça certa para os DOIS tectos**, que é o que esta
//! cena existe para ensinar: sobre um campo com cristas **e** vales, `Height 1 /
//! Depth 0` corta as cristas sem encher os vales, `0 / 1` faz o oposto, e `1 / 1`
//! faz os dois — *a mesma mão, três ferramentas, dois números*.

/// `=47` — a cena do **PINCEL DE PLANO**.
///
/// ⚠️ **O número foi CONTADO no roteador** (`scenes::CENAS` estava em `46`) — o
/// gate `no_two_sculpt3d_scenes_claim_the_same_level` já apanhou uma cena a
/// reclamar um número tomado, e a segunda fica **inalcançável e muda**.
pub(crate) fn plano_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("47")
}

/// **Quantos triângulos a peça desta cena tem.**
///
/// ⚠️ **Ele nomeia o recurso, e o recurso aqui é a NITIDEZ do relevo**, não o
/// relógio: este pincel corre por dab como todos os outros (a amostragem é
/// `O(vértices na pegada)`), então a peça pode ser densa. O que ela não pode ser
/// é grossa — uma crista de seis triângulos não tem o que aparar.
pub(crate) const TRIANGULOS_DA_PECA: usize = 40_000;

/// A peça com que a `=47` abre: uma bola com **bossas**.
pub(crate) fn peca() -> ph2d_mesh::Mesh {
    let mut m = ph2d_mesh::shapes::sphere_with_triangles(TRIANGULOS_DA_PECA, 1.0);
    // ⚠️ **O relevo é ANALÍTICO e não esculpido**, pela razão que o corpus do
    // oráculo também segue: uma superfície escrita por uma fórmula é a mesma em
    // toda máquina e em toda corrida, logo o que o dono vê é o que o gate mede.
    let pos: Vec<[f32; 3]> = m
        .positions()
        .iter()
        .map(|p| {
            let n = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt().max(1e-6);
            let u = [p[0] / n, p[1] / n, p[2] / n];
            // Bossas em duas direcções: cristas E vales, que é o que os dois
            // tectos separam.
            let h = 1.0 + 0.09 * (u[0] * 9.0).sin() * (u[1] * 9.0).sin();
            [u[0] * h, u[1] * h, u[2] * h]
        })
        .collect();
    ph2d_mesh::Mesh::from_parts(pos, m.faces().to_vec()).unwrap_or_else(|_| {
        // A topologia não mudou, logo isto é inalcançável; devolver a bola lisa
        // é a única resposta finita se alguma vez deixar de o ser.
        std::mem::take(&mut m)
    })
}

/// O roteiro da `=47`.
pub(crate) fn announce() {
    if !plano_scene() {
        return;
    }
    eprintln!(
        "[sculpt3d] =47 PLANE -- aparar, encher e achatar ESFREGANDO, com um pincel so'\n\
         [sculpt3d]    Este e' o pincel que voce pediu: ele apara a forma esfregando, como\n\
         [sculpt3d]    massa de modelar. A bola abre com BOSSAS -- cristas e vales -- porque\n\
         [sculpt3d]    e' relevo que ele apara; numa bola lisa ele nao tem o que fazer.\n\
         [sculpt3d]\n\
         [sculpt3d]    (1) Abra o painel com a CRASE (`). Na fileira de ferramentas, no FIM,\n\
         [sculpt3d]        ha' um botao `Plane`. Carregue nele.\n\
         [sculpt3d]        -> Aparecem pistas novas: `Height`, `Depth` e `Hold Tilt`. Ele nasce\n\
         [sculpt3d]           com Height 1 e Depth 0 (a APARAR) e Hold Tilt 1.\n\
         [sculpt3d]    (2) Aumente o `Radius` (a pista, ou a tecla `]` varias vezes) ate' o\n\
         [sculpt3d]        circulo cobrir tres ou quatro bossas.\n\
         [sculpt3d]        -> O circulo pode crescer ate' ao tamanho da janela.\n\
         [sculpt3d]    (3) ESFREGUE por cima das bossas, de um lado ao outro, IDA E VOLTA,\n\
         [sculpt3d]        quatro a oito vezes, SEM LARGAR o botao.\n\
         [sculpt3d]        -> As cristas descem ate' perto do fundo dos vales e a faixa fica\n\
         [sculpt3d]           LISA. Continue: quando ja' esta' plano ele PARA sozinho.\n\
         [sculpt3d]        -> Largar e voltar a carregar a cada passagem apara PIOR: cada\n\
         [sculpt3d]           traco novo esquece a inclinacao que o anterior tinha fixado.\n\
         [sculpt3d]    (4) Carregue e SOLTE sem arrastar, no meio de uma bossa.\n\
         [sculpt3d]        -> NAO acontece nada, e esta' certo: este pincel so' trabalha a\n\
         [sculpt3d]           ESFREGAR.\n\
         [sculpt3d]    (5) Ponha `Hold Tilt` em 0 e esfregue do mesmo jeito noutro sitio.\n\
         [sculpt3d]        -> A faixa fica ONDULADA e afunda. E' a diferenca que o Hold Tilt\n\
         [sculpt3d]           faz: ele prende a inclinacao do plano durante o traco. Volte a 1.\n\
         [sculpt3d]    (6) Ponha `Depth` em 1 e `Height` em 0, e esfregue outra vez.\n\
         [sculpt3d]        -> Agora e' o contrario: ele ENCHE os vales e nao toca nas cristas.\n\
         [sculpt3d]    (7) Ponha os DOIS em 1 e esfregue.\n\
         [sculpt3d]        -> Ele ACHATA: corta as cristas e enche os vales ao mesmo tempo.\n\
         [sculpt3d]    (8) Ponha os DOIS em 0 e esfregue.\n\
         [sculpt3d]        -> Nao faz nada, de proposito. E' o `desligado` da ferramenta.\n\
         [sculpt3d]    (9) Volte a Height 1 / Depth 0 e segure o CTRL enquanto esfrega.\n\
         [sculpt3d]        -> Ele ENCHE enquanto a tecla estiver em baixo: aparar e encher na\n\
         [sculpt3d]           mesma mao. No fundo do painel, `Ctrl Does` -> `Push Away` faz o\n\
         [sculpt3d]           Ctrl AFASTAR do plano (levantar) em vez de encher.\n\
         [sculpt3d]\n\
         [sculpt3d]    DEU ERRADO SE: oito passagens com o pincel grande nao deixam a faixa\n\
         [sculpt3d]    lisa; se ele cavar um buraco quando ja' esta' plano; se `Hold Tilt` 0\n\
         [sculpt3d]    e 1 derem o mesmo resultado; ou se `Height 0 / Depth 0` ainda mexer no\n\
         [sculpt3d]    barro.\n"
    );
}
