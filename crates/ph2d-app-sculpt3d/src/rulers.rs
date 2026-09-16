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
}
