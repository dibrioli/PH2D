//! **AS RÉGUAS DO GESTO** — quanto um pixel de arrasto vale, e onde uma
//! grandeza deixa de existir.
//!
//! ⚠️ **Saiu da shell em 2026-09-11 (W2/L3-A4)**, primeiro habitante da `ph2d-app-sculpt3d`.
//! O corte por ASSUNTO que o criou continua a valer — a shell diz *o que a CENA é* (o
//! `Sculpt3dScene`, os módulos, o `Drag`) e este ficheiro *com que régua a mão fala com ela*.
//! ⛔ **A régua do FILTRO ficou na shell de propósito**: ela é um re-export de
//! `ph2d_sculpt3d::FILTER_DRAG_PER_PX`, e trazê-la daria a esta crate uma dependência que
//! quem não usa a escultura teria de pagar — é isso que a mantém não-opcional e barata.
//!
//! ⚠️ **Quase todo número aqui é decisão de SMOKE, não teto de recurso** — e a
//! distinção é a do §0 do `CLAUDE.md`: um limite legítimo diz **de que recurso
//! ele é** e traz a medição ao lado. Os que TÊM medição a carregam no próprio
//! doc-comment ([`DEFAULT_RADIUS_PX`], [`RADIUS_MIN_PX`],
//! [`RADIUS_MAX_FRAC_OF_DIAGONAL`]); os outros dizem, em vez de fingir, que quem
//! os escolheu foi o olho do Enio.
//!
//! ⭐ **A shell re-exporta tudo** (`use ph2d_app_sculpt3d::rulers::*` no `sculpt3d/mod.rs`),
//! então os filhos seguem lendo `super::ORBIT_RAD_PER_PX` como sempre leram — a travessia de
//! crate não move um caminho, tal como o corte de arquivo não movia.

/// Quantos radianos um pixel de arrasto vale.
///
/// Decisão de **smoke**, como a tolerância do RDP do Flip: 0,01 dá meia volta a
/// cada ~314 px, que é uma varredura confortável de trackpad. Não é um teto de
/// recurso, então não tem tabela de medição ao lado — tem o olho do Enio.
pub const ORBIT_RAD_PER_PX: f32 = 0.01;

/// O raio do pincel, em **pixels de tela**.
///
/// ⚠️ **Pixels, não fração do modelo** — o raio de MUNDO é derivado por dab
/// (`Camera3d::world_radius_for_screen_px`), então o pincel mantém o tamanho
/// aparente quando a câmera aproxima. É o `computeWorldRadius2` do SculptGL, e
/// é o que Blender e ZBrush entregam: aproximar É como se alcança detalhe fino,
/// e um raio ancorado no modelo tornava isso impossível (o pincel crescia junto
/// com a imagem).
///
/// **50 px é MEDIDO, não escolhido:** é o que reproduz o tamanho aparente do
/// default anterior (0,12 do span) na cena do smoke a 720p — ver
/// `ph2d-mesh-render/tests/it/measure_screen_radius.rs`.
pub const DEFAULT_RADIUS_PX: f32 = 50.0;

/// Passo das teclas de LUZ (`Q`/`E` giram, `R`/`F` sobem e descem), em graus
/// inteiros — que é a unidade em que o rig é autorado. Quinze graus porque o
/// gesto é *"ver a forma reacender"*, não afinar: um passo de 1° pediria vinte
/// toques para a mudança ficar óbvia.
pub const LIGHT_STEP_DEG: u16 = 15;

/// Passo do `[` / `]`. Multiplicativo pelo motivo do `dolly`: o gesto tem o
/// mesmo efeito *aparente* com pincel grande e pequeno.
pub const RADIUS_STEP: f32 = 1.15;

/// Quantos passos um clique de blur/sharpen dá.
///
/// ⚠️ **Número de SMOKE, não teto de recurso.** Um passo é pequeno de propósito
/// (`BLUR_MIX = 0,5`, para o gesto não apagar a própria borda de uma vez), então
/// o clique precisa de vários para o artista ver a diferença — e clicar de novo
/// borra mais, que é o que o gesto significa.
pub const MASK_OP_PASSES: u32 = 6;

/// O piso do raio, em pixels. **Quem aperta é a TELA, não a malha** — e isso é
/// medição, não herança: a régua antiga dizia *"menor que uma aresta não pega
/// vértice"*, e na cena do smoke um disco de **0,5 px já pega um vértice**
/// (`measure_screen_radius.rs`), porque a malha é densa. O que de fato quebra
/// abaixo de um pixel é o artista **ver onde está mirando**.
///
/// ⚠️ **Este doc-comment estava ORFANADO:** ele tinha escorregado para cima do
/// [`MASK_OP_PASSES`] (que passou a abrir descrevendo o piso do raio) e a const
/// ficara NUA — a mesma família que o split do `paint.rs` do Painter já pagou em
/// 2026-07-19. Um `mod` novo entre uma doc e o item dela não dá erro: ela apenas
/// passa a documentar o vizinho.
pub const RADIUS_MIN_PX: f32 = 1.0;

/// ⭐⭐ **O tecto do raio é a DIAGONAL da vista** — em fracção dela, e o recurso é o ECRÃ.
///
/// Com o cursor em qualquer ponto da vista, um anel deste raio contém a vista inteira: acima dele
/// o pincel só acrescenta geometria que o artista **não vê**, e a borda do que ele toca deixa de
/// caber no ecrã. ⚠️ **Fracção da tela e não pixels fixos**, porque um tecto fixo muda de
/// significado com a resolução (medido: 160 px cobriam 91 % da altura do modelo a 1280×720 e 45 %
/// a 2560×1440).
///
/// ⛔⛔ **O tecto antigo era `1/8` da ALTURA, e a razão escrita dele era uma opinião** — *«acima de
/// meio modelo o pincel é um deformador global, que é outra ferramenta»*. O dono desmentiu-a com o
/// uso (2026-09-16, pincel de plano): *«O radius máximo permitido é pouco»* e *«o melhor jeito de
/// ver o efeito é com o pincel bem grande, do tamanho da peça»*. Na vista de omissão a peça ocupa
/// 49 % da altura, logo `1/8` dava **metade do raio da peça** — nunca a peça.
///
/// ⭐ **E o custo não é o recurso que aperta, medido** (`mede_o_raio_do_pincel`, `--release`,
/// `load 5`): um dab custa ~0,14 µs por vértice tocado e **SATURA na malha** — acima do diâmetro
/// da peça não há mais vértice nenhum. A peça inteira custa `2,8 ms` na cena `=47` (20 k
/// vértices), `27 ms` a 200 k e `257 ms` a 1,5 M; e o espaçamento é proporcional ao raio, logo um
/// pincel oito vezes maior deposita oito vezes menos dabs. ⚠️ O preço de uma malha pesada é o da
/// MALHA (o filtro de malha inteira paga o mesmo), e a saída dele é o K1 (`docs/3D/03.5`), não um
/// tecto que esconda a peça do artista.
pub const RADIUS_MAX_FRAC_OF_DIAGONAL: f32 = 1.0;

/// O tecto do raio para uma vista `w × h` — ver [`RADIUS_MAX_FRAC_OF_DIAGONAL`].
#[must_use]
pub fn radius_ceiling_px(w: u32, h: u32) -> f32 {
    #[allow(clippy::cast_precision_loss)]
    let diagonal = (w.max(1) as f32).hypot(h.max(1) as f32);
    (RADIUS_MAX_FRAC_OF_DIAGONAL * diagonal).max(RADIUS_MIN_PX)
}

/// A zona morta do **Twist**, em pixels de tela.
///
/// ⚠️ **Ela não é conforto, é a fronteira onde a grandeza deixa de existir:**
/// perto da âncora a direção *âncora → cursor* é RUÍDO, e um tremor de um pixel
/// a um pixel de distância vale meio radiano. Trinta é o número do SculptGL
/// (`Twist.js:92`), e como todo número de gesto deste arquivo ele é decisão de
/// **smoke** — não é teto de recurso nenhum.
pub const TWIST_DEADZONE_PX: f32 = 30.0;

/// Quanto de escala vale um pixel de arrasto horizontal no **Local Scale**
/// (`+1` dobra o raio da pegada). Cem pixels dobram; decisão de smoke, como o
/// [`ORBIT_RAD_PER_PX`].
pub const SCALE_PER_PX: f32 = 0.01;

#[cfg(test)]
mod tests {
    use super::radius_ceiling_px;

    /// ⭐ **GATE — o pincel pode ter o tamanho da PEÇA** (report de 2026-09-16).
    ///
    /// Na vista de omissão a peça ocupa 49 % da altura, logo o raio dela é ~¼ da altura; o tecto
    /// tem de o conter com folga em toda janela, e em nenhuma pode ser a fracção da altura que o
    /// escondia. ⚠️ A outra metade: o anel do tecto contém a vista inteira a partir de qualquer
    /// ponto — é esse o recurso que ele diz ser.
    #[test]
    fn o_pincel_pode_ter_o_tamanho_da_peca() {
        for (w, h) in [(1280u32, 720u32), (1920, 1080), (2560, 1080), (1024, 768)] {
            let tecto = radius_ceiling_px(w, h);
            #[allow(clippy::cast_precision_loss)]
            let (wf, hf) = (w as f32, h as f32);
            assert!(
                tecto >= 0.49 * hf,
                "{w}x{h}: tecto {tecto} nao cobre a peca inteira"
            );
            assert!(
                tecto >= wf.hypot(hf) - 0.5,
                "{w}x{h}: de um canto, o anel do tecto ({tecto}) nao contem a vista"
            );
        }
        assert!(
            radius_ceiling_px(0, 0) >= super::RADIUS_MIN_PX,
            "a janela vazia nao tem piso"
        );
    }

    /// O que o ALVO oferece, em píxeis de **RAIO** — a espec `SPEC_pincel_de_plano.md` §14.3
    /// publica-os em **diâmetro** (`1 000` na pista, `10 000` digitados).
    const PISTA_DO_ALVO_PX: f32 = 500.0;
    const DIGITAVEL_DO_ALVO_PX: f32 = 5000.0;

    /// As vistas NOMEADAS, com a diagonal MEDIDA de cada uma — ver o [`G-20`].
    ///
    /// [`G-20`]: o_tecto_do_raio_passa_a_pista_do_alvo_e_nao_chega_ao_digitavel
    const VISTAS: [(u32, u32, f32); 6] = [
        (1024, 768, 1280.0),
        (1280, 720, 1468.6),
        (1920, 1080, 2202.9),
        (2560, 1080, 2778.5),
        (2560, 1440, 2937.2),
        (3840, 2160, 4405.8),
    ];

    /// ⭐⭐⭐ **G-20 — o tecto do raio PASSA a pista do alvo e NÃO chega ao digitável dele**, com a
    /// população nomeada (espec `SPEC_pincel_de_plano.md` §14.3 + errata **Q2**).
    ///
    /// A espec encomendou o gate numa frase só — *«o tecto do raio não fica abaixo do do alvo»* —
    /// com os dois números de interface dele. Medido, a frase é **verdadeira numa metade e falsa
    /// na outra**, e é por isso que este gate não tem o nome que ela lhe deu: *um gate cujo nome
    /// promete as duas metades mentiria sobre a segunda em toda corrida verde.*
    ///
    /// ⛔⛔ **A metade que falha é uma DIVERGÊNCIA DECLARADA, e o gate EXIGE que ela exista.** O
    /// nosso tecto não é um número: é a **DIAGONAL DA VISTA** ([`RADIUS_MAX_FRAC_OF_DIAGONAL`]), e
    /// o recurso que ele nomeia é o **ECRÃ** — acima disso o anel do pincel já contém a vista
    /// inteira a partir de qualquer ponto, e não há mais barro ao alcance. Um tecto de `5 000`
    /// fixo seria um número sem recurso, que é o que o `CLAUDE.md` §0.0 proíbe. ⚠️ **Sem a segunda
    /// asserção o tecto vira LICENÇA:** quem subisse a fracção apagaria a divergência em silêncio,
    /// e a declaração da espec ficaria a descrever um produto que já não existe.
    ///
    /// | vista | tecto MEDIDO | contra a pista do alvo (`500`) | contra o digitável (`5 000`) |
    /// |---|---|---|---|
    /// | `1024×768` | `1 280,0` | **`2,56×`** | `0,26×` |
    /// | `1280×720` | `1 468,6` | `2,94×` | `0,29×` |
    /// | `1920×1080` | `2 202,9` | `4,41×` | `0,44×` |
    /// | `2560×1080` | `2 778,5` | `5,56×` | `0,56×` |
    /// | `2560×1440` | `2 937,2` | `5,87×` | `0,59×` |
    /// | `3840×2160` | `4 405,8` | **`8,81×`** | **`0,88×`** |
    ///
    /// ⚠️ **A vista é o CANVAS e não a janela** — ela é o sub-rectângulo em que a escultura
    /// desenha, logo é sempre MENOR que os pares acima. *O erro é todo para o lado conservador:*
    /// a divergência real é **maior** que a que esta tabela mede, nunca menor.
    ///
    /// ⭐ **O `3840×2160` é o PISO da população e não uma linha a mais:** é ali que a segunda
    /// metade quase cai (`0,88×`), logo é a única vista que a torna difícil. *Uma lista sem ela
    /// afirmaria a divergência só onde ela é fácil.*
    #[test]
    fn o_tecto_do_raio_passa_a_pista_do_alvo_e_nao_chega_ao_digitavel() {
        assert!(
            VISTAS.iter().any(|&(w, h, _)| (w, h) == (3840, 2160)),
            "a vista 4K saiu da populacao — e' a unica em que a divergencia e' dificil"
        );
        for (w, h, diagonal) in VISTAS {
            let tecto = radius_ceiling_px(w, h);
            // ⚠️ **A ORDEM destas três é load-bearing.** As duas COMPARAÇÕES vêm primeiro
            // porque são o que a espec encomendou; a terceira — *«o tecto ainda é a
            // diagonal»* — é a mais apertada das três e, posta à frente, seria a única a
            // disparar em toda mutação da lei, deixando as outras duas **impossíveis de
            // matar**. *Uma asserção que nunca chega a ser a primeira a falhar é comentário
            // com sintaxe de código*, e a prova de mutação (M1/M2/M3) mede exactamente isso.
            assert!(
                tecto >= PISTA_DO_ALVO_PX,
                "{w}x{h}: o tecto {tecto} caiu abaixo da pista do alvo ({PISTA_DO_ALVO_PX}) — \
                 e' o report de 2026-09-16 a voltar"
            );
            assert!(
                tecto < DIGITAVEL_DO_ALVO_PX,
                "{w}x{h}: o tecto {tecto} alcancou o digitavel do alvo \
                 ({DIGITAVEL_DO_ALVO_PX}) — a divergencia declarada na \
                 `SPEC_pincel_de_plano.md` (errata Q2) deixou de existir: releia-a antes \
                 de apagar esta assercao"
            );
            assert!(
                (tecto - diagonal).abs() < 0.1,
                "{w}x{h}: o tecto {tecto} deixou de ser a diagonal medida ({diagonal}) — \
                 as duas comparacoes acima continuam a fechar, logo o que mudou foi a \
                 FORMA da lei e nao a folga dela"
            );
        }
    }

    /// ⭐⭐ **G-20, a segunda metade — o que APERTA o raio é a VISTA, nunca o WIDGET.**
    ///
    /// A divergência do irmão acima só é honesta se o tecto for mesmo o recurso medido. Se a pista
    /// do painel parasse antes da diagonal, quem clampava era **a régua do widget** — e a
    /// declaração *«o tecto é o ecrã»* passaria a descrever a coisa errada, **com os dois gates
    /// verdes**. ⛔ Era exactamente esse o estado em 2026-09-16 (pista de `200` px, tecto em `1/8`
    /// da altura) e o report do dono foi *«o radius máximo permitido é pouco»*.
    ///
    /// ⇒ a pista oferece **o número digitável do alvo em cheio** (`5 000` px de raio) e, em toda
    /// vista nomeada, fica **estritamente acima** do tecto ⇒ o número que o dab usa vem sempre da
    /// vista.
    ///
    /// ⚠️ **E a TROCA é nomeada:** numa vista cuja diagonal passe `5 000` (16:9, ~`4358×2451` —
    /// acima de 4K) quem passa a apertar é a pista, e isso **não é defeito**: ali o tecto do alvo
    /// é que é o mais apertado dos dois. *Sem esta metade alguém leria «a vista ganha sempre» como
    /// lei, e escreveria a próxima cura contra ela.*
    #[test]
    fn o_que_aperta_o_raio_e_a_vista_nunca_o_widget() {
        let raio = ph2d_panel_sculpt3d::rows::row_for(ph2d_panel_sculpt3d::ids::SCULPT3D_RADIUS)
            .expect("a linha do raio saiu da tabela de rows");
        assert!(
            raio.max >= DIGITAVEL_DO_ALVO_PX,
            "a pista do painel para em {} e ja' nao oferece o digitavel do alvo ({}) — \
             quem clampa passou a ser o WIDGET, e a declaracao «o tecto e' a vista» \
             deixou de descrever o produto",
            raio.max,
            DIGITAVEL_DO_ALVO_PX
        );
        for (w, h, _) in VISTAS {
            let tecto = radius_ceiling_px(w, h);
            assert!(
                tecto < raio.max,
                "{w}x{h}: o tecto da vista ({tecto}) alcancou a pista ({}) — quem aperta \
                 deixou de ser a vista",
                raio.max
            );
        }
        // A troca, NOMEADA: acima de `5 000` de diagonal quem aperta passa a ser a pista.
        let (w, h) = (5120u32, 2880u32);
        assert!(
            radius_ceiling_px(w, h) > raio.max,
            "{w}x{h}: a vista devia passar a pista aqui (diagonal ~5 874) — a troca mudou \
             de sitio e a linha que a nomeia no doc envelheceu"
        );
    }
}
