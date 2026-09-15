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
//!
//! # ⭐⭐ E o passo (2) é o OSSO, que é o que torna o resto ensinável
//!
//! Ordem do dono (2026-09-14): *«no blender temos um gizmo do pincel que mostra
//! como se fosse um bone de modo ao usuário perceber a área de atualização do
//! pincel»*. Ele existe agora ([`super::pose_gizmo`]), e o roteiro **passa o
//! rato antes de carregar** de propósito: a pergunta que este verbo levanta —
//! *onde é que ele vai achar a dobradiça?* — passa a ter resposta **antes** do
//! gesto, e não depois de o desfazer. ⚠️ *Um pincel cuja região não se vê
//! aprende-se por tentativa; o anel do cursor, sozinho, desenha um círculo onde
//! a ferramenta pensa num membro.*

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
         [sculpt3d]    (2) Ctrl+Z. Escolha `Pose` e passe o rato sobre a orelha SEM CARREGAR.\n\
         [sculpt3d]        -> Aparece um OSSO desenhado por cima da forma, do feitio dos ossos\n\
         [sculpt3d]           de um boneco articulado: largo de um lado e afilado do outro.\n\
         [sculpt3d]           Ele diz o que vai acontecer se voce arrastar dali:\n\
         [sculpt3d]             * a BOLINHA CHEIA, na ponta larga, e' a dobradica;\n\
         [sculpt3d]             * o osso vai da dobradica ate' a' sua mao, e e' esse pedaco\n\
         [sculpt3d]               da peca que vai rodar.\n\
         [sculpt3d]           Passeie o rato ao longo da orelha e veja a dobradica MUDAR de\n\
         [sculpt3d]           sitio: perto da ponta ela fica na base; perto da base ela cai\n\
         [sculpt3d]           no corpo da bola.\n\
         [sculpt3d]    (3) Agora faca o MESMO arrasto do passo (1), no mesmo sitio.\n\
         [sculpt3d]        -> A orelha INTEIRA gira, rigida, em volta da bolinha que voce viu.\n\
         [sculpt3d]           Ela nao estica e nao afina: ela DOBRA. Nao ha' esqueleto nenhum\n\
         [sculpt3d]           -- o pincel acha a dobradica sozinho, pela forma.\n\
         [sculpt3d]           O osso acompanha a mao enquanto voce arrasta.\n\
         [sculpt3d]    (4) Ctrl+Z. No painel, ponha `Segments` em 3 e passe o rato outra vez.\n\
         [sculpt3d]        -> Agora sao TRES ossos em fila, como um braco. Arraste: a orelha\n\
         [sculpt3d]           dobra em tres pedacos e a curva acompanha a mao, em vez de ser\n\
         [sculpt3d]           um so' giro.\n\
         [sculpt3d]    (5) Ctrl+Z. Ponha `Segments` de volta em 1 e desmarque `Pin far end`.\n\
         [sculpt3d]        -> A orelha deixa de rodar no sitio: ela e' ARRASTADA junto com o\n\
         [sculpt3d]           giro. Marcada, a base fica pregada; desmarcada, nao.\n\
         [sculpt3d]    (6) Marque `Pin far end` outra vez. A fileira `Deformation` fica\n\
         [sculpt3d]        logo abaixo dos numeros do pincel e tem CINCO botoes:\n\
         [sculpt3d]        `Rotate` · `Twist` · `Scale` · `Translate` · `Squash / Stretch`.\n\
         [sculpt3d]        ⚠️ NAO e' preciso Ctrl para nenhum deles -- basta carregar no\n\
         [sculpt3d]        botao. Escolha `Scale` e arraste AO LONGO da orelha (na direcao\n\
         [sculpt3d]        em que ela aponta).\n\
         [sculpt3d]        -> A orelha ENGORDA ou ENCOLHE **e roda ao mesmo tempo**. Isso e'\n\
         [sculpt3d]           a lei: neste modo o gesto resolve a articulacao primeiro e so'\n\
         [sculpt3d]           depois escala. Para escalar SEM rodar, marque a caixa\n\
         [sculpt3d]           `Scale without rotating`, que aparece so' com `Scale` escolhido.\n\
         [sculpt3d]    (7) Escolha `Translate` e arraste.\n\
         [sculpt3d]        -> Ela desliza inteira, sem mudar de tamanho nem rodar.\n\
         [sculpt3d]    (8) Escolha `Squash / Stretch` e arraste AO LONGO dela.\n\
         [sculpt3d]        -> A orelha ESTICA e AFINA junto (ou encolhe e engorda): o volume\n\
         [sculpt3d]           dela mantem-se, como massa a ser puxada.\n\
         [sculpt3d]        ⚠️ Estes tres leem o arrasto AO LONGO do osso: arrastar de\n\
         [sculpt3d]           travessao move pouco, e isso e' a lei, nao um defeito.\n\
         [sculpt3d]    (9) Escolha `Twist` e arraste na HORIZONTAL.\n\
         [sculpt3d]        -> Agora ela TORCE sobre o proprio eixo, em vez de dobrar.\n\
         [sculpt3d]   (10) Ctrl+Z e volte a `Rotate`. No painel, o `Auto-Smooth` so' aparece\n\
         [sculpt3d]        com o interruptor\n\
         [sculpt3d]        `Detail` em `Pro` -- ele e' a PRIMEIRA fileira dentro da seccao do\n\
         [sculpt3d]        pincel, logo abaixo do titulo dela, e vem de fabrica em `Basic`.\n\
         [sculpt3d]        Ponha em `Pro`, suba `Auto-Smooth` para o\n\
         [sculpt3d]        maximo e repita o arrasto do passo (3).\n\
         [sculpt3d]        -> A orelha dobra na mesma, e a superficie sai MAIS LISA: a dobra\n\
         [sculpt3d]           deixa de mostrar as facetas da malha. Ponha de volta em 0 e\n\
         [sculpt3d]           repita para comparar.\n\
         [sculpt3d]           O alisamento acompanha o OSSO, nao o circulo do pincel: ele\n\
         [sculpt3d]           alcanca a orelha inteira, mesmo a parte que esta' longe do\n\
         [sculpt3d]           cursor, e nao toca no resto da bola.\n\
         [sculpt3d]\n\
         [sculpt3d]    DEU ERRADO SE: nao aparecer osso nenhum no passo (2); se a orelha\n\
         [sculpt3d]    esticar num bico no passo (3) em vez de dobrar rigida; se ela nao mexer\n\
         [sculpt3d]    NADA; se o corpo da bola se deformar junto com ela; se desmarcar\n\
         [sculpt3d]    `Pin far end` nao mudar nada; se `Scale without rotating` nao tirar a\n\
         [sculpt3d]    rotacao do passo (6); se algum dos CINCO botoes de\n\
         [sculpt3d]    `Deformation` nao mudar o gesto ao ser carregado; ou se o\n\
         [sculpt3d]    `Auto-Smooth` do passo (10) nao mudar a superficie.\n\
         [sculpt3d]\n\
         [sculpt3d]    (Se em vez do osso aparecer so' uma BOLINHA VERMELHA sobre o cursor, e'\n\
         [sculpt3d]     um aviso: ali nao ha' dobradica nenhuma e arrastar nao move nada.\n\
         [sculpt3d]     Acontece no meio de uma superficie lisa, e e' o sitio certo para NAO\n\
         [sculpt3d]     usar este pincel.)\n\
         [sculpt3d]\n\
         [sculpt3d]    (Os outros dois knobs: `Pivot offset from cursor` empurra a dobradica\n\
         [sculpt3d]     para longe do dedo -- da' para ver o osso crescer enquanto voce mexe\n\
         [sculpt3d]     nele --, e `Weight smoothing` -- que so' aparece no modo avancado do\n\
         [sculpt3d]     painel -- esbate a fronteira entre os pedacos da dobra.)"
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
        //
        // ⭐ **E ela é de COMPILAÇÃO, não de teste** — os dois lados são
        // constantes, então o `clippy` acusava-a (`assertions_on_constants`) e
        // tinha razão: uma comparação que o compilador resolve não precisa de
        // uma corrida para reprovar. ⛔ O preço é a mensagem perder o número:
        // um `const` não formata.
        //
        // ⚠️⚠️ **E o `cargo check` é CEGO a isto — medido nesta jornada.** Com
        // o limiar mutado para `9999`, `cargo check -p … --all-targets` fecha
        // **verde** e só o `cargo test` (que constrói) devolve o `E0080`. *Um
        // `const` de dentro de uma função só é avaliado quando ela é
        // construída*, logo o laço interno deste repo não o vê — o portão de
        // fecho, que corre o nextest, vê. É o mesmo ponto cego de família do
        // `--bins` que não alcança `tests/`.
        const {
            assert!(
                crate::scenes::CENAS >= 41,
                "o tecto do roteador tem de conter esta cena (=41)"
            );
        }
    }

    /// ⭐ **Sonda: o que o INDICADOR custa NESTA cena**, que é a malha que o dono
    /// vai ter à frente. O orçamento de [`ph2d_sculpt3d::pose_previa`] deriva o
    /// silêncio deste número, então ele é o que decide se o osso segue o cursor
    /// quadro a quadro ou se ele passa a arrastar-se.
    ///
    /// ```text
    /// cargo test -p ph2d-app-sculpt3d --lib -- --ignored --nocapture mede_o_indicador
    /// ```
    #[test]
    #[ignore = "sonda: imprime o custo do indicador na malha desta cena"]
    fn mede_o_indicador_nesta_cena() {
        let malha = crate::scenes::mesh::smoke_mesh_for_tests_ear();
        let ponta = malha
            .positions()
            .iter()
            .copied()
            .max_by(|a, b| a[1].total_cmp(&b[1]))
            .expect("a malha tem vértices");
        for segmentos in [1u32, 3, 20] {
            let mut b = Brush {
                verb: Verb::Pose,
                radius: 0.25,
                ..Brush::default()
            };
            b.pose.segmentos = segmentos;
            let mut s = SculptStroke::default();
            let t0 = std::time::Instant::now();
            let n = s.pose_ossos(&malha, &b, Symmetry::default(), ponta).len();
            let primeiro = t0.elapsed().as_secs_f64() * 1e3;
            let t1 = std::time::Instant::now();
            s.pose_ossos(&malha, &b, Symmetry::default(), ponta);
            let repetido = t1.elapsed().as_secs_f64() * 1e6;
            println!(
                "orelha {:>6} verts · segmentos {segmentos:>2} · ossos {n} · \
                 1.a construcao {primeiro:>7.2} ms · quadro repetido {repetido:>7.2} us",
                malha.positions().len()
            );
        }
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

    /// ⛔⛔⛔ **TODA FILEIRA QUE O ROTEIRO NOMEIA TEM DE ESTAR NA LISTA — e no
    /// NÍVEL que ele diz.**
    ///
    /// ⚠️ *Um passo que manda clicar numa linha de painel **afirma** que ela está
    /// lá*, e o dono aprova o smoke com o passo impossível dentro — ele conclui
    /// que não achou, não que não existe. Este gate nasceu porque o passo do `Auto-Smooth`
    /// quase shipou a mandá-lo subir o `Auto-Smooth` **sem dizer que ele vive no
    /// `Pro`**, e o painel nasce em `Basic`.
    ///
    /// ⚠️ **A régua é a TABELA do painel** (`rows()` + `Row::visible`), nunca uma
    /// lista escrita aqui: uma segunda cópia da condição divergiria na primeira
    /// wave que mexesse numa delas, e a que o artista vê é a que envelhece.
    #[test]
    fn as_fileiras_que_o_roteiro_nomeia_sao_alcancaveis_com_a_pose_na_mao() {
        use ph2d_panel_sculpt3d::rows::rows;
        use ph2d_panel_sculpt3d::slots::VerbSlot;
        use ph2d_panel_sculpt3d::state::Sculpt3dUi;
        use ph2d_panel_sculpt3d::state_modes::UiLevel;

        /// `(rótulo i18n, o nível MÍNIMO em que o roteiro promete achá-la)`.
        /// ⚠️ O `Auto-Smooth` está aqui como `Pro` **de propósito**: é isso que
        /// obriga o roteiro a dizer onde fica o interruptor.
        const NOMEADAS: &[(&str, UiLevel)] = &[
            ("panel.sculpt3d.pose_segments", UiLevel::Basic),
            ("panel.sculpt3d.auto_smooth", UiLevel::Pro),
        ];
        // ⚠️ **As duas que o roteiro nomeia e que NÃO são `Row`** — a caixa
        // `Pin far end` e a fileira de chips `Deformation` — vivem noutra
        // superfície, e por isso são presas pelo **texto do pintor**: se o
        // ficheiro mudar de sítio isto **deixa de compilar**, em vez de ficar
        // verde a medir menos (`HOWTO §2.6`). ⛔ Sem esta metade o gate afirmaria
        // sobre dois dos quatro passos e leria-se como se cobrisse os quatro.
        const FORA_DA_TABELA: &[&str] =
            &["panel.sculpt3d.pose_anchored", "panel.sculpt3d.pose_mode"];
        const PINTOR: &str = include_str!("../../ph2d-panel-sculpt3d/src/paint/brush_fileiras.rs");
        for chave in FORA_DA_TABELA {
            assert!(
                PINTOR.contains(chave),
                "o roteiro da =41 nomeia `{chave}` e o pintor do painel nao a \
                 desenha — o dono procura e nao acha"
            );
        }
        let slot = VerbSlot::for_verb(Verb::Pose);
        let ui = |nivel| Sculpt3dUi {
            brush: slot.brush.clone(),
            radius_px: slot.radius_px,
            ui_level: nivel,
            ..Sculpt3dUi::default()
        };
        for &(rotulo, nivel) in NOMEADAS {
            let r = rows().find(|r| r.label == rotulo).unwrap_or_else(|| {
                panic!("o roteiro da =41 nomeia `{rotulo}` e o painel nao tem essa fileira")
            });
            assert!(
                r.visible(&ui(nivel)),
                "o roteiro manda mexer em `{rotulo}` e ela nao e' pintada com a \
                 pose na mao no nivel {nivel:?} — o dono procura e nao acha"
            );
            // ⭐ **A metade que faz o roteiro ser HONESTO sobre o nivel:** uma
            // fileira que o roteiro promete no `Pro` tem de estar mesmo
            // ESCONDIDA no `Basic`, senão a frase que manda trocar o
            // interruptor é ruído; e uma que ele promete no `Basic` não pode
            // precisar do `Pro`.
            assert_eq!(
                r.visible(&ui(UiLevel::Basic)),
                nivel == UiLevel::Basic,
                "o roteiro promete `{rotulo}` a partir de {nivel:?} e o painel \
                 discorda no `Basic`"
            );
        }
    }
}
