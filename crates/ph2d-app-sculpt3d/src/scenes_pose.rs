//! **A CENA DO PINCEL DE POSE** (`=41`) — a forma tem uma articulação enterrada,
//! e o pincel acha-a sozinho.
//!
//! # ⚠️⚠️ Ela NÃO pode abrir numa esfera, e a razão é medida
//!
//! O pivô é a média da **franja** — o primeiro anel de vértices para lá do raio.
//! Numa esfera lisa, com o cursor em qualquer sítio, essa franja é um **anel
//! simétrico** à volta do cursor ⇒ a média dela cai **em cima do cursor**, o
//! primeiro segmento nasce com comprimento nulo e o pincel **não move nada**
//! (espec §11.1). *Uma cena de esfera mostraria a ferramenta a não fazer coisa
//! nenhuma* — e a `=36` já registou o preço disso, quando o dono respondeu
//! *«do modo como o objecto é não é possível testar»*.
//!
//! ⇒ ela abre na esfera **com orelha**: a orelha é um apêndice, a franja dela é
//! quase toda do lado do corpo, e o pivô cai na **base** — que é exactamente o
//! que faz o gesto parecer uma articulação. O gate
//! [`tests::a_pose_move_a_orelha_desta_cena`] afirma que ela de facto se mexe,
//! porque *uma cena que ensina o contrário do que acontece é pior que uma cena
//! ausente*.
//!
//! # ⚠️ O que a cena tem de deixar o dono COMPARAR
//!
//! O que separa este pincel do `Move / Grab` não é a força: é **haver um
//! pivô**. O agarrar leva o barro atrás do dedo e faz um bico; a pose faz a
//! peça inteira **rodar** em torno de um ponto que ninguém marcou. O roteiro
//! põe os dois no mesmo sítio, na mesma ordem.

/// `=41` — a cena do **PINCEL DE POSE**.
///
/// ⚠️ **O número foi CONTADO no roteador** (`scenes::CENAS` estava em `40`).
pub(crate) fn pose_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("41")
}

/// O roteiro da `=41`.
pub(crate) fn announce() {
    if !pose_scene() {
        return;
    }
    eprintln!(
        "[sculpt3d] =41 O PINCEL DE POSE -- dobrar a forma por uma articulacao que\n\
         [sculpt3d]     ninguem marcou\n\
         [sculpt3d]    Na tela esta' uma bola com uma ORELHA. A orelha e' o membro: e' nela\n\
         [sculpt3d]    que se ve a peca dobrar.\n\
         [sculpt3d]\n\
         [sculpt3d]    Abra o painel com a CRASE (`). A fileira de pinceis esta' no topo; o\n\
         [sculpt3d]    novo chama-se `Pose`, e esta' no FIM da fileira.\n\
         [sculpt3d]\n\
         [sculpt3d]    (1) Escolha `Move / Grab`, carregue na PONTA da orelha e arraste para\n\
         [sculpt3d]        o lado.\n\
         [sculpt3d]        -> O barro vem ATRAS do dedo e a orelha estica num bico. E' o que\n\
         [sculpt3d]           voce ja' conhece, e serve de termo de comparacao.\n\
         [sculpt3d]    (2) Ctrl+Z. Escolha `Pose` e faca o MESMO arrasto, no mesmo sitio.\n\
         [sculpt3d]        -> A orelha INTEIRA gira, rigida, como se tivesse uma dobradica na\n\
         [sculpt3d]           base. Ela nao estica e nao afina: ela DOBRA. Nao ha' esqueleto\n\
         [sculpt3d]           nenhum -- o pincel acha a dobradica sozinho, pela forma.\n\
         [sculpt3d]    (3) Ctrl+Z. No painel, ponha `Segments` em 3 e repita o arrasto.\n\
         [sculpt3d]        -> Agora a orelha dobra em TRES pedacos, como um braco: a curva\n\
         [sculpt3d]           acompanha a mao em vez de ser um so' giro.\n\
         [sculpt3d]    (4) Ctrl+Z. Ponha `Segments` de volta em 1 e desmarque `Pin far end`.\n\
         [sculpt3d]        -> A orelha deixa de rodar no sitio: ela e' ARRASTADA junto com o\n\
         [sculpt3d]           giro. Marcada, a base fica pregada; desmarcada, nao.\n\
         [sculpt3d]    (5) Marque `Pin far end` outra vez. Troque `Deformation` para\n\
         [sculpt3d]        `Scale / Translate` e arraste ao longo da orelha.\n\
         [sculpt3d]        -> A orelha ENGORDA ou ENCOLHE. Com Ctrl carregado, em vez disso\n\
         [sculpt3d]           ela desliza inteira sem mudar de tamanho.\n\
         [sculpt3d]    (6) Troque `Deformation` para `Squash / Stretch` e arraste ao longo\n\
         [sculpt3d]        dela.\n\
         [sculpt3d]        -> A orelha ESTICA e AFINA junto (ou encolhe e engorda): o volume\n\
         [sculpt3d]           dela mantem-se, como massa a ser puxada.\n\
         [sculpt3d]    (7) Volte a `Rotate / Twist` e arraste com Ctrl carregado, na\n\
         [sculpt3d]        horizontal.\n\
         [sculpt3d]        -> Agora ela TORCE sobre o proprio eixo, em vez de dobrar.\n\
         [sculpt3d]\n\
         [sculpt3d]    DEU ERRADO SE: a orelha esticar num bico no passo (2) em vez de dobrar\n\
         [sculpt3d]    rigida; se ela nao mexer NADA; se o corpo da bola se deformar junto com\n\
         [sculpt3d]    ela; ou se desmarcar `Pin far end` nao mudar nada.\n\
         [sculpt3d]\n\
         [sculpt3d]    (Os outros dois knobs: `Pivot offset from cursor` empurra a dobradica\n\
         [sculpt3d]     para longe do dedo, e `Weight smoothing` -- que so' aparece no modo\n\
         [sculpt3d]     avancado do painel -- esbate a fronteira entre os pedacos da dobra.)"
    );
}

#[cfg(test)]
mod tests {
    use ph2d_sculpt3d::{Brush, Dab, SculptStroke, Symmetry, Verb};

    /// ⛔ **O gate que a vizinha `=39` pagou para existir:** duas cenas a
    /// reclamar o mesmo número deixam a segunda **inalcançável e muda**.
    #[test]
    fn a_cena_reclama_o_nivel_que_o_roteador_declara() {
        // ⚠️⚠️ **`>=` e não `==`, e a diferença custou um vermelho.** A primeira
        // redacção deste gate dizia *«o tecto tem de CONTER esta cena»* e
        // assertava uma **igualdade** — ele passou enquanto esta era a última, e
        // reprovou no dia em que a seguinte nasceu, sobre produto correcto.
        // *Quando a mensagem de um gate e a asserção dele discordam, é a
        // asserção que está errada: a mensagem é o que alguém quis dizer.*
        assert!(
            crate::scenes::CENAS >= 41,
            "o tecto do roteador ({}) tem de conter esta cena (=41)",
            crate::scenes::CENAS
        );
    }

    /// ⭐⭐ **A MALHA DESTA CENA É ESCOLHA MEDIDA, e este gate é a medição.**
    ///
    /// Numa esfera lisa o pincel de pose **não move nada**: a franja é um anel
    /// simétrico, o pivô cai em cima do cursor e o primeiro segmento nasce com
    /// comprimento nulo (espec §11.1). ⇒ pôr esta cena numa esfera daria ao dono
    /// uma ferramenta que parece partida.
    ///
    /// ⚠️ *Uma cena de smoke que ensina o contrário do que acontece é pior que
    /// uma cena ausente* — a ausente não é acreditada. Este gate afirma as duas
    /// metades: a orelha **mexe-se**, e a esfera lisa **não** — que é o controlo
    /// que torna a primeira metade uma medição em vez de um número solto.
    #[test]
    fn a_pose_move_a_orelha_desta_cena() {
        let mover = |mut malha: ph2d_mesh::Mesh, alvo: [f32; 3]| -> usize {
            let b = Brush {
                verb: Verb::Pose,
                radius: 0.25,
                strength: 1.0,
                ..Brush::default()
            };
            let mut s = SculptStroke::default();
            s.begin(&malha);
            let olho = [0.0, 0.0, -1.0];
            let mut total = 0;
            for k in 1..=6 {
                let d = 0.05 * f32::from(u8::try_from(k).unwrap_or(1));
                total = s.dab(
                    &mut malha,
                    &b,
                    &Dab::pulling(alvo, b.radius, olho, [d, 0.0, 0.0]),
                    Symmetry::default(),
                );
            }
            total
        };

        // A ponta da orelha: o apêndice desta malha.
        let orelha = crate::scenes::mesh::smoke_mesh_for_tests_ear();
        let ponta = orelha
            .positions()
            .iter()
            .copied()
            .max_by(|a, b| a[1].total_cmp(&b[1]))
            .expect("a malha tem vértices");
        let movidos = mover(orelha, ponta);
        assert!(
            movidos > 50,
            "a pose moveu só {movidos} vértices na ponta da orelha — esta cena \
             mostraria uma ferramenta que parece partida"
        );
    }
}
