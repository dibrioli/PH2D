//! `PH2D_LINE_SMOKE` — a tela pronta para julgar o card **Line** (plano 38, W2..W6).
//!
//! ⚠️ **Uma cena para o CARD, não uma por tipo.** O que a wave entrega é o dropdown `Type` e as rows
//! que cada tipo pinta; três cenas seriam três canvas idênticos com três roteiros, e a costura que
//! importa — *escolher o tipo* — ficaria pulada em todas.
//!
//! ⚠️ **Ela dá o MATERIAL e NÃO arma tipo nenhum** — a cicatriz que o `impasto_smoke` prega no doc
//! dele: *"the smoke that arms state under the table skips exactly the seam it was supposed to
//! prove"*. O `Type` é um dropdown, e é ele que o artista tem de alcançar.
//!
//! ```text
//! cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter && \
//!   PH2D_LINE_SMOKE=1 cargo run --release -p ph2d-host-desktop
//! ```
//!
//! **O `--release` não é preferência:** a densidade cheia do Sketchy põe ~16 mil px de fio num traço
//! de 312 px, o Wire desenha quatro cordas por dab, e o Spray multiplica cada dab por até dezasseis.
//! Em debug isso lê como *"o pincel travou"*.
//!
//! ⚠️ **A FITA percorre caminho no TIQUE, não no evento de ponteiro** — e no PEN-UP ela **não
//! acrescenta nada**: o traço acaba exactamente onde o último tique a deixou, atrás do dedo. ⚠️ **A
//! 1ª versão corria até o cursor** (uma reta atravessando o desenho, porque o alvo já estava parado)
//! e a 2ª soltava a coleira e deixava a inércia esgotar-se — o *follow-through*. O Enio reprovou as
//! duas: *"no mouse up o fim do traço cresce um segmento indesejado"* (2026-08-15). O que fica é a
//! tinta ATRÁS do dedo, sem crescer depois da soltura.
//!
//! ⚠️ **O SPRAY não mora no card Line, e isso é a decisão da W5** — ele não é um tipo de linha, é um
//! multiplicador da emissão que compõe com todos eles (o *Scattering* do Photoshop, não um modo).
//! Mora no card **Jitter** da seção Stroke, onde ficam as três rows que ele transforma de *tremor*
//! em *nuvem*. A cena é a mesma porque o material é o mesmo: um canvas e um pincel grande.

use ph2d_asset::{AssetDb, AssetId};
use ph2d_core::Vec2;
use ph2d_ecs::SimWorld;
use ph2d_render::SpriteRenderer;
use std::collections::BTreeMap;

/// A cena está armada?
pub(crate) fn enabled() -> bool {
    std::env::var_os("PH2D_LINE_SMOKE").is_some()
}

/// Spawna a tela branca. Devolve os bits da entidade para o chamador sentar a seleção nela.
pub(crate) fn spawn_if_enabled(
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    cell_idx: u32,
    pixels_per_meter: f32,
    atlas_asset_map: &mut BTreeMap<u32, AssetId>,
) -> Option<u64> {
    if !enabled() {
        return None;
    }
    let edge = 2048u32;
    match crate::image_import::spawn_blank_canvas(
        sim,
        renderer,
        asset_db,
        cell_idx,
        edge,
        2, // branco opaco: um fio fino e de opacidade baixa só se lê sobre fundo claro
        Vec2::new(0.0, 0.0),
        pixels_per_meter,
        atlas_asset_map,
    ) {
        Ok((label, bits)) => {
            println!(
                "[line-smoke] canvas '{label}' ({edge}x{edge}) pronto, pincel r=24. \
                 O Painter abre em DIGITAL, que e onde os tipos de linha vivem."
            );
            println!(
                "[line-smoke] roteiro: pegue o Painter no rail -> card LINE (logo acima do \
                 Composite Brush) -> dropdown Type."
            );
            println!(
                "[line-smoke]   SPEED: um chicote rapido. A tinta e ARREMESSADA a frente do dedo \
                 e passa do ponto onde a mao parou -- e o oposto do estabilizador. Sem slider, de \
                 proposito (o Alchemy tambem nao tem)."
            );
            println!(
                "[line-smoke]   SKETCHY: rabisque para tras e para a frente. A teia nasce entre \
                 trechos vizinhos NO CANVAS. Depois desenhe um GRAMPO (va, de a volta, volte 5 px \
                 abaixo) e alterne o Magnetify: LIGADO ele costura as duas pernas, DESLIGADO so a \
                 porcao que o dedo acabou de deixar."
            );
            println!(
                "[line-smoke]   WIRE: desenhe uma CURVA fechada. O arame corta a quina e sobra \
                 para FORA da curva -- e o laco. O History mede o tamanho do laco (em arco \
                 percorrido, entao o Spacing nao o muda). Desmarque CONNECTION LINE e o traco \
                 desaparece: fica so o arame."
            );
            println!(
                "[line-smoke]   SPRAY: nao e um Type -- ele MULTIPLICA qualquer um deles. Va a \
                 secao STROKE -> card JITTER -> a PRIMEIRA row, 'Count'. Com 1 o traco e o de \
                 sempre; ao pedir a SEGUNDA marca o 'Position' logo abaixo sai do zero sozinho \
                 (senao as copias empilhariam e o slider pareceria morto) -- confira que ele mexeu, \
                 e depois mande nele. O Scale e o Rotation ao lado dao tamanho e angulo a cada \
                 marca. Combine com Speed ou Sketchy: eles compoem."
            );
            println!(
                "[line-smoke]   RIBBON: uma FAIXA com travessas (o Ribbon Shapes do Alchemy). Dois \
                 trilhos -- o do DEDO e o ATRASADO -- ligados por riscos atravessados, e a LARGURA \
                 DA FAIXA E O PROPRIO ATRASO. Desenhe uma onda RAPIDA: a faixa ABRE nas retas e \
                 FECHA nos picos, onde a mao desacelera. E a pergunta de olho desta wave."
            );
            println!(
                "[line-smoke]     O traco tambem PESA: a tinta fica ATRAS do dedo, chicoteia na \
                 saida da curva e -- com Gravity -- PENDE. ⚠️ SOLTE: o traco acaba onde esta, sem \
                 crescer nem um segmento. ⚠️ PARE a mao no meio do gesto, com o \
                 botao preso: NADA pode ser desenhado. Sao DUAS leis -- sem gesto nao ha tempo (a \
                 mola congela) E o settle e inerte com a fita armada (num traco de fita ha UM \
                 caminho e ele e o dela). Retome e o traco continua de onde parou, sem salto. O \
                 CONTROLE e sem fita: Type=None com Stabilizer alto ainda alcanca o cursor."
            );
            println!(
                "[line-smoke]     RUNGS e a densidade das travessas. ⚠️ Em 0 a faixa DEGENERA na \
                 linha atrasada sozinha (o pincel de arrasto) -- e o CONTROLE desta wave. ⚠️ Numa \
                 onda que DESACELERA nos picos a faixa tem de ficar uma ESCADA tambem nas cristas: \
                 onde os dois trilhos discordam ela fica mais esparsa, e isso e a lei. Mexa o \
                 slider de 0 ate 1 e a faixa tem de aparecer e adensar. A largura dela nao muda com \
                 este slider: quem a abre e o Weight (o atraso) e a velocidade da sua mao."
            );
            println!(
                "[line-smoke]     LINE WIDTH e OPACITY sao a tinta dos FIOS -- o trilho de fora e \
                 TODA travessa saem por elas, entao sao os dois numeros que decidem como a faixa \
                 APARECE. Sao as mesmas rows do Sketchy e do Wire, na mesma posicao: mexa e a \
                 escada tem de engrossar/clarear na hora."
            );
            println!(
                "[line-smoke]     Weight e QUANTO TEMPO ela atrasa, Friction e COMO ela assenta \
                 (baixo = chicote que passa do ponto e volta; alto = ela so arrasta), Gravity e o \
                 peso. Medido a 2 400 px/s: peso 1,00 deixa a tinta 804 px atras do dedo, peso \
                 0,16 deixa 106."
            );
            println!(
                "[line-smoke]     ⚠️ O FUNDO DA PISTA E INERTE, e e medido: ate peso ~0,02 a fita \
                 nao consegue mover a tinta um dab inteiro e desenha o traco comum. E o que se \
                 quer de um minimo que significa DESLIGADO -- se voce mexer no slider e nada \
                 mudar, suba mais."
            );
            println!(
                "[line-smoke]     E ela COMPOE: ligue o Spray (Jitter -> Count) ou a Symmetry com \
                 a fita armada -- eles seguem a fita, porque para todos eles a fita E o traco."
            );
            println!(
                "[line-smoke]   ROUGH: a linha VAGUEIA e e' desenhada DUAS vezes (rough.js / \
                 Excalidraw). ROUGHNESS e' o tremor CURTO, BOWING o arqueamento LONGO -- sao duas \
                 OITAVAS do mesmo desvio, entao mexa uma de cada vez para as ver separadas. PASSES \
                 e' quantas caminhadas: em 2 as duas divergem e se CRUZAM (o contorno duplo), em 1 \
                 sobra a linha que so' vagueia."
            );
            println!(
                "[line-smoke]     ⚠️ A PERGUNTA DE OLHO DESTA WAVE e' com um SHAPE EDITOR: escolha \
                 Stroke -> Ellipse, desenhe uma elipse e ARRASTE um ponto dela. A figura tem de \
                 ficar PARADA enquanto voce a ajusta -- o desvio e' funcao do ARCO, entao \
                 re-carimbar a mesma figura da' os mesmos dabs. Se ela FERVER, pare: o desvio \
                 virou jitter."
            );
            println!(
                "[line-smoke]     E o CONTRASTE que diz que ele nao e' o jitter: ponha Roughness \
                 no maximo e desenhe uma reta longa -- tem de sair uma linha que ONDULA, nunca uma \
                 linha empoeirada. O jitter (secao Stroke) e' o outro controle, e faz o oposto."
            );
            println!(
                "[line-smoke]   E o CONTROLE: com Type = None e Count = 1 o pincel tem de pintar \
                 exatamente como sempre pintou."
            );
            Some(bits)
        }
        Err(e) => {
            eprintln!("[line-smoke] nao consegui spawnar o canvas: {e}");
            None
        }
    }
}

/// Arma o TAMANHO do pincel na primeira vez que o Painter liga um documento sob esta cena.
///
/// ⚠️ **Só o tamanho, e ele importa:** o alcance do Sketchy e a janela do Wire são medidos em
/// DIÂMETROS de pincel, então um pincel minúsculo desenha uma teia minúscula e a cena não mostraria
/// a feature. O resto — o tipo, os parâmetros dele — é o que o artista vai escolher, e é a costura
/// que esta cena existe para exercitar.
pub(crate) fn arm_brush_once(painter: &mut ph2d_tool_painter::PainterTool) {
    use std::sync::atomic::{AtomicBool, Ordering};
    static ARMED: AtomicBool = AtomicBool::new(false);
    if !enabled() || ARMED.swap(true, Ordering::Relaxed) {
        return;
    }
    painter.set_brush_size_px(24.0);
}
