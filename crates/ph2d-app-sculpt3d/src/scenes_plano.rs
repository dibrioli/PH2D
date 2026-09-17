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

/// **SONDA — o que o `Plane Offset` de facto faz na peça desta cena.**
///
/// ```text
/// cargo test -p ph2d-app-sculpt3d --lib diag_o_deslocamento_do_plano -- --ignored --nocapture
/// ```
#[cfg(test)]
mod diag {
    use ph2d_mesh::Mesh;
    use ph2d_panel_sculpt3d::slots::VerbSlot;
    use ph2d_sculpt3d::{Dab, SculptStroke, Symmetry, Verb};

    fn corre(offset: f32, dabs: usize) -> (Mesh, Mesh) {
        corre_com(offset, dabs, None)
    }

    fn corre_com(offset: f32, dabs: usize, tectos: Option<(f32, f32)>) -> (Mesh, Mesh) {
        let base = super::peca();
        let mut m = base.clone();
        let mut b = VerbSlot::for_verb(Verb::Plane).brush;
        b.radius = 0.35;
        b.plane_offset = offset;
        if let Some((alt, prof)) = tectos {
            b.plano_altura = alt;
            b.plano_profundidade = prof;
        }
        let olho = [0.0, 0.0, -1.0];
        let mut s = SculptStroke::default();
        s.begin(&m);
        // ⚠️ **O centro tem de estar SOBRE a peça** — a 1.ª redacção pôs o cursor
        // em `z = 1,2`, fora de uma bola de raio `1,09`, e a sonda leu `0`
        // movidos em todas as linhas. *Uma sonda que não toca nada lê-se como um
        // knob morto.*
        for k in 0..dabs {
            let x = -0.25 + 0.5 * (k as f32) / ((dabs - 1).max(1) as f32);
            let alvo = [x, 0.0, 1.0];
            let centro = *m
                .positions()
                .iter()
                .min_by(|a, c| {
                    let d = |q: &[f32; 3]| {
                        (q[0] - alvo[0]).powi(2)
                            + (q[1] - alvo[1]).powi(2)
                            + (q[2] - alvo[2]).powi(2)
                    };
                    d(a).total_cmp(&d(c))
                })
                .expect("a peca tem vertices");
            s.dab(
                &mut m,
                &b,
                &Dab::at(centro, b.radius, olho),
                Symmetry::default(),
            );
        }
        (base, m)
    }

    #[test]
    #[ignore]
    fn diag_o_deslocamento_do_plano() {
        println!(
            "{:>8} {:>8} {:>10} {:>10} {:>10} {:>10}",
            "offset", "movidos", "max|d|", "medio|d|", "raio_toc", "raio/R"
        );
        for offset in [0.0f32, -0.5, -0.2, -0.1, -0.05, 0.05, 0.1, 0.2, 0.5] {
            let (base, m) = corre(offset, 8);
            let (mut movidos, mut maxd, mut soma) = (0usize, 0.0f32, 0.0f64);
            let mut raio_toc = 0.0f32;
            for (i, p) in m.positions().iter().enumerate() {
                let r = base.positions()[i];
                let d =
                    ((p[0] - r[0]).powi(2) + (p[1] - r[1]).powi(2) + (p[2] - r[2]).powi(2)).sqrt();
                if d > 0.0 {
                    movidos += 1;
                    maxd = maxd.max(d);
                    soma += f64::from(d);
                    // distancia do vertice ao EIXO do traco (que corre em x, z~1.2)
                    let lateral = (r[1] * r[1]).sqrt();
                    raio_toc = raio_toc.max(lateral);
                }
            }
            println!(
                "{offset:>8.2} {movidos:>8} {maxd:>10.4} {:>10.4} {raio_toc:>10.4} {:>10.2}",
                if movidos > 0 {
                    soma / movidos as f64
                } else {
                    0.0
                },
                raio_toc / 0.35
            );
        }

        // A MESMA varredura com os tectos BILATERAIS (o perfil *achatar*), para
        // separar «o knob e' assimetrico» de «os tectos de fabrica sao de UM
        // lado so'».
        println!("\n-- com tectos 1/1 (bilateral, o perfil *achatar*) --");
        for offset in [0.0f32, -0.5, 0.5] {
            let (base, m) = corre_com(offset, 8, Some((1.0, 1.0)));
            let (mut movidos, mut maxd) = (0usize, 0.0f32);
            for (i, q) in m.positions().iter().enumerate() {
                let r = base.positions()[i];
                let d =
                    ((q[0] - r[0]).powi(2) + (q[1] - r[1]).powi(2) + (q[2] - r[2]).powi(2)).sqrt();
                if d > 0.0 {
                    movidos += 1;
                    maxd = maxd.max(d);
                }
            }
            println!("{offset:>8.2} {movidos:>8} {maxd:>10.4}");
        }
    }
}
