//! **O QUE O CARTÃO DIZ SOBRE A ESPÉCIE E O PAPEL DE UM NÓ** — a cor de um socket e o selo de
//! papel no cabeçalho, cortados do `paint.rs` no teto de LOC do painel (600).
//!
//! ⚠️ **O corte é por RESPONSABILIDADE:** o `paint.rs` responde *«como se desenha um cartão?»*
//! (corpo, cabeçalho, título, sockets, fios) e este responde *«o que é este nó, e o que é que
//! esta porta carrega?»* — duas perguntas que crescem por razões diferentes: aquela quando o
//! cartão ganha uma peça, esta quando o VOCABULÁRIO ganha uma espécie ou um papel.
//!
//! ⛔⛔⛔ **Os dois nasceram do mesmo achado** (estudo do Mini Cavalry,
//! [doc 99 §10](../../../docs/Motion%20Nodes/99_estudo_do_mini_cavalry_2026-09-02.md)): dos
//! cinco canais visuais que o grafo tem, **dois estavam mortos** — a cor do socket tomava um
//! valor só (`Instances` em 100% das 138 portas) e a silhueta era declarada por 132 nós,
//! transportada até ao pintor e **nunca lida**.

use crate::geom::{self, View};
use crate::snapshot::{GraphNodeView, PortView};
use ph2d_editor_core::paint::{fill_circle, fill_rounded_rect, resolve};
use ph2d_editor_core::paint_shapes::{fill_diamond, fill_polygon};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;
use ph2d_node_registry::NodeSilhouette;
use ph2d_nodegraph::port::{Clock, Dim, Domain};
use ph2d_tokens::{ColorToken, Theme};

use super::{TITLE_PAD_X, domain_token};

/// **As razões de FORMA de cada selo** — geometria de um glifo, não espaçamento de layout: é a
/// mesma classe do `r` do [`fill_diamond`], e por isso levam o marcador da casa em vez de um
/// token de design (não há token para *«quanto um trapézio estreita»*).
const TAPER: f32 = 0.55; // LITERAL-PX-OK: how much a trapezoid narrows (shape geometry)
const CIGAR_H: f32 = 0.6; // LITERAL-PX-OK: cigar half-height as a fraction of r
const CIGAR_SPAN: f32 = 1.2; // LITERAL-PX-OK: cigar full height (2 x CIGAR_H)
const TAB_DEPTH: f32 = 0.4; // LITERAL-PX-OK: how deep the I/O tab bites into the badge

/// Meia-diagonal do selo de papel, em px lógicos.
const ROLE_R: f32 = 5.0; // LITERAL-PX-OK: role badge half-size
/// O que o selo empurra o título para a direita — o seu diâmetro mais a folga.
const ROLE_GAP: f32 = 5.0; // LITERAL-PX-OK: gap between the role badge and the title

/// **Quanto o selo custa ao título**, ou `0` quando não há selo.
///
/// ⚠️ **`Rect` NÃO tem selo, e é a lei que torna isto barato:** o censo do registry
/// (`what_our_visual_channels_carry`) diz **`Rect` em 106 de 132 nós (80,3%)** — é o
/// *modificador genérico*, o default, e desenhar um rectanguinho em 106 cartões seria ruído a
/// competir com o nome do nó. É a mesma forma da lei do [`crate::PortLabel`]: *um rótulo
/// responde «qual delas?», e sem pergunta não há resposta.*
///
/// ⇒ **106 cartões saem byte a byte como antes**; os **26** que declaram um papel diferente
/// pagam `15 px` do orçamento do título (de `178` para `163`, `−8,4%`).
pub(super) fn role_inset_px(s: NodeSilhouette) -> f32 {
    match s {
        NodeSilhouette::Rect => 0.0,
        _ => 2.0 * ROLE_R + ROLE_GAP,
    }
}

/// ⭐⭐⭐ **O SELO DE PAPEL — o que o nó É no grafo, no cabeçalho.**
///
/// ⛔⛔⛔ **A silhueta era um canal MORTO** (estudo do Mini Cavalry,
/// [doc 99 §4](../../../docs/Motion%20Nodes/99_estudo_do_mini_cavalry_2026-09-02.md)): o
/// `ph2d-node-registry` declara **sete** papéis, **132 nós preenchem-nos** com **seis** valores
/// distintos, a `GraphNodeView` transporta-a e **o pintor nunca a lia**. Declarada,
/// transportada, sem consumidor — a forma pura do knob morto que o
/// [doc 90](../../../docs/Motion%20Nodes/90_caca_aos_knobs_mortos.md) caça.
///
/// ⚠️ **E ela NÃO podia ser o contorno do CARTÃO**, que foi a primeira ideia: o corpo define os
/// rectângulos de hit e a posição de cada socket (`geom`/`hits`), então mudar a forma dele
/// arrastaria a geometria toda. *Chamei-lhe «grátis» antes de medir; o cabeçalho é a única
/// superfície onde de facto é.*
///
/// A tinta é a do TÍTULO, não uma cor nova: o selo é uma marca de leitura, não uma sexta cor a
/// competir com a categoria (que já pinta o cabeçalho) e com as três espécies de socket.
pub(super) fn role_glyph(ctx: &mut PaintCtx, n: &GraphNodeView, view: &View, theme: Theme) {
    if n.silhouette == NodeSilhouette::Rect {
        return; // o default: 80,3% dos nós, e nenhum pixel muda
    }
    let (sx, sy) = view.pt(n.x, n.y);
    let r = ROLE_R * view.zoom;
    let cx = sx + (TITLE_PAD_X + ROLE_R) * view.zoom;
    let cy = sy + geom::HEADER_H * 0.5 * view.zoom;
    let color = resolve(ColorToken::Text1, theme);
    match n.silhouette {
        // Já tratado acima; repetido para o `match` ser exaustivo sem um braço `_` — um
        // papel NOVO no registry passa a ser erro de compilação AQUI, que é o único sítio
        // onde alguém se lembraria de lhe dar forma.
        NodeSilhouette::Rect => {}
        // Terminal / sink: o ponto final de um fio.
        NodeSilhouette::Circle => fill_circle(ctx.scene, cx, cy, r, color),
        // Decisão / porta: o losango do fluxograma.
        NodeSilhouette::Diamond => fill_diamond(ctx.scene, cx, cy, r, color),
        // Junção: a cápsula que recebe vários e emite um.
        NodeSilhouette::Cigar => fill_rounded_rect(
            ctx.scene,
            Rect::new(cx - r, cy - r * CIGAR_H, 2.0 * r, r * CIGAR_SPAN),
            r * CIGAR_H,
            color,
        ),
        // Fonte: alarga para BAIXO — o dado nasce aqui e derrama para a cadeia.
        NodeSilhouette::TrapezoidDown => fill_polygon(
            ctx.scene,
            &[
                (cx - r * TAPER, cy - r),
                (cx + r * TAPER, cy - r),
                (cx + r, cy + r),
                (cx - r, cy + r),
            ],
            color,
        ),
        // Sink de efeito colateral: estreita para baixo — a cadeia entra e não sai.
        NodeSilhouette::TrapezoidUp => fill_polygon(
            ctx.scene,
            &[
                (cx - r, cy - r),
                (cx + r, cy - r),
                (cx + r * TAPER, cy + r),
                (cx - r * TAPER, cy + r),
            ],
            color,
        ),
        // I/O externo: um corpo com ABA — o que entra ou sai do documento.
        NodeSilhouette::Tabbed => fill_polygon(
            ctx.scene,
            &[
                (cx - r, cy - r * TAB_DEPTH),
                (cx, cy - r * TAB_DEPTH),
                (cx, cy - r),
                (cx + r, cy - r),
                (cx + r, cy + r),
                (cx - r, cy + r),
            ],
            color,
        ),
    }
}

/// ⭐⭐⭐ **A COR DE UM SOCKET — o que ele CARREGA, não só de que domínio é.**
///
/// ⛔⛔⛔ **O canal estava morto** (estudo do Mini Cavalry,
/// [doc 99 §2](../../../docs/Motion%20Nodes/99_estudo_do_mini_cavalry_2026-09-02.md)): medido
/// no registry, **`Domain::Instances` em 100% das 138 portas** do módulo. A cor não distinguia
/// **nada**, e a única distinção viva — ○ escalar contra ◇ vector — vivia num glifo que nada
/// no ecrã nomeia.
///
/// ⇒ é a causa medida do report de 2026-09-01: o `value.lfo` (escalar) e o `motion.oscillator`
/// (vector) tinham sockets do MESMO roxo, e o artista não tinha como saber que um conduz um
/// param e o outro não.
///
/// **As TRÊS espécies que o módulo de facto tem**, contadas (`what_the_socket_encoding_carries`):
///
/// | espécie | portas | token |
/// |---|---:|---|
/// | pulso (`Clock::Event`) | 8 (5,8%) | [`ColorToken::PortEvent`] |
/// | número (`Dim::Scalar`) | 45 (32,6%) | [`ColorToken::PortValue`] |
/// | corrente (vector) | 93 (67,4%) | [`ColorToken::PortInstances`] |
///
/// ⚠️ **A ordem das perguntas é a lei, e o RELÓGIO vem primeiro**: um pulso escalar é um pulso,
/// não um número — é o que um `pulse.*` emite, e tratá-lo como número apagaria a distinção que
/// mais salta à vista num grafo (o disparo contra o valor contínuo).
///
/// ⚠️ **Os tokens `PortEvent`/`PortStatic` já EXISTIAM e nunca tinham sido usados** — o próprio
/// comentário deles no `ph2d-tokens` dizia *«socket color = Domain (or Clock for
/// event/static)»*, e o código só lia o domínio. *Um token declarado que ninguém resolve é a
/// mesma espécie de morto que um knob.*
///
/// ⚠️ **Um domínio que NÃO é `Instances` mantém a cor do domínio** — fora do Motion (vector,
/// campo, sinal, controlo) a pergunta «de que mundo é este fio?» ainda é a primeira, e nenhum
/// desses aparece neste módulo hoje.
///
/// ⏳ **O FIO ainda não** — a `GraphEdgeView` só carrega `out_domain`, não o `PortType`
/// inteiro, então pintar o fio pela espécie pede um campo novo no snapshot. Nomeado, não feito.
pub(super) fn socket_token(p: &PortView) -> ColorToken {
    match (p.clock, p.domain, p.dim) {
        (Clock::Event, _, _) => ColorToken::PortEvent,
        (Clock::Static, _, _) => ColorToken::PortStatic,
        (_, Domain::Instances, Dim::Scalar) => ColorToken::PortValue,
        (_, d, _) => domain_token(d),
    }
}

#[cfg(test)]
mod role_badge_tests {
    use super::role_inset_px;
    use ph2d_node_registry::NodeSilhouette;

    /// ⭐⭐ **O DEFAULT NÃO PAGA NADA** — a metade que torna esta wave barata.
    ///
    /// O censo do registry diz `Rect` em **106 de 132 nós (80,3%)**: se o selo aparecesse
    /// neles, 80% dos cartões perderiam orçamento de título por um rectanguinho que diz
    /// *«sou um modificador genérico»* — o mesmo ruído que a lei do rótulo de porta já
    /// recusou («escrever *Out* em 294 cartões»).
    #[test]
    fn the_default_role_costs_the_title_nothing() {
        assert_eq!(
            role_inset_px(NodeSilhouette::Rect),
            0.0,
            "um modificador generico nao veste selo, e o cartao dele fica byte a byte igual"
        );
    }

    /// ⚠️ **E os SEIS papéis que não são o default pagam o MESMO** — um selo de largura
    /// variável faria o título começar em sítios diferentes de cartão para cartão, e o olho
    /// lê uma coluna, não seis.
    #[test]
    fn every_other_role_pays_the_same_and_it_is_measured() {
        let want = 2.0 * super::ROLE_R + super::ROLE_GAP;
        for s in [
            NodeSilhouette::Circle,
            NodeSilhouette::Diamond,
            NodeSilhouette::Cigar,
            NodeSilhouette::TrapezoidDown,
            NodeSilhouette::TrapezoidUp,
            NodeSilhouette::Tabbed,
        ] {
            assert_eq!(role_inset_px(s), want, "{s:?}");
        }
        // O preço, escrito: o orçamento do título vai de 178 a 163 px (−8,4%).
        assert!(
            (want - 15.0).abs() < 1e-6,
            "o selo custa 15 px; se mudar, a conta do doc muda com ele"
        );
        assert!(
            crate::geom::CARD_W - crate::paint::TITLE_INSET_R - want > 150.0,
            "o titulo tem de continuar a caber num nome de no' tipico"
        );
    }
}

#[cfg(test)]
mod socket_token_tests {
    use super::socket_token;
    use crate::snapshot::PortView;
    use ph2d_nodegraph::port::{Clock, Dim, Domain};
    use ph2d_tokens::ColorToken;

    fn port(domain: Domain, dim: Dim, clock: Clock) -> PortView {
        PortView {
            name: "p",
            domain,
            dim,
            clock,
        }
    }

    /// ⭐⭐⭐ **AS TRÊS ESPÉCIES TÊM TRÊS CORES** — o gate do estudo do Mini Cavalry (doc 99 §2).
    ///
    /// ⛔⛔ **O defeito que ele fecha:** a cor do socket vinha só do `Domain`, e o censo do
    /// registry diz `Instances` em **100% das 138 portas** — ou seja o canal existia e não
    /// distinguia nada. O `value.lfo` (escalar) e o `motion.oscillator` (vector) eram do MESMO
    /// roxo, que é a causa medida do report de 2026-09-01.
    #[test]
    fn the_three_kinds_of_socket_wear_three_colours() {
        let numero = socket_token(&port(Domain::Instances, Dim::Scalar, Clock::Frame));
        let corrente = socket_token(&port(Domain::Instances, Dim::Vec2, Clock::Frame));
        let pulso = socket_token(&port(Domain::Instances, Dim::Scalar, Clock::Event));
        assert_eq!(numero, ColorToken::PortValue, "um NUMERO");
        assert_eq!(corrente, ColorToken::PortInstances, "uma CORRENTE");
        assert_eq!(pulso, ColorToken::PortEvent, "um PULSO");
        assert_ne!(numero, corrente, "as duas que o report confundiu");
        assert_ne!(numero, pulso);
        assert_ne!(corrente, pulso);
    }

    /// ⚠️ **A CORRENTE mantém a cor que sempre teve** — 93 das 138 portas (67,4%) não mudam
    /// um pixel. Uma cura que repintasse tudo seria uma mudança de tema disfarçada de correcção.
    #[test]
    fn the_stream_keeps_the_colour_it_always_had() {
        for dim in [Dim::Vec2, Dim::Vec3, Dim::Vec4] {
            assert_eq!(
                socket_token(&port(Domain::Instances, dim, Clock::Frame)),
                ColorToken::PortInstances,
                "{dim:?}: a corrente e' o roxo de sempre"
            );
        }
    }

    /// ⚠️ **O RELÓGIO ganha ao domínio, e a ordem é a lei:** um pulso ESCALAR é um pulso, não
    /// um número. Trocar a ordem apagaria a distinção que mais salta à vista num grafo — o
    /// disparo contra o valor contínuo — e nenhum outro gate a mede.
    #[test]
    fn a_scalar_pulse_reads_as_a_pulse_and_not_as_a_number() {
        assert_eq!(
            socket_token(&port(Domain::Instances, Dim::Scalar, Clock::Event)),
            ColorToken::PortEvent,
            "o `Clock::Event` decide ANTES do `Dim::Scalar`"
        );
    }

    /// ⚠️ **Fora do Motion o DOMÍNIO ainda manda** — a cura não pode alargar-se a mundos onde
    /// «de que mundo é este fio?» continua a ser a primeira pergunta.
    #[test]
    fn another_domain_keeps_its_own_colour() {
        for (d, want) in [
            (Domain::Vector, ColorToken::PortVector),
            (Domain::Field, ColorToken::PortField),
            (Domain::Signal, ColorToken::PortSignal),
            (Domain::Control, ColorToken::PortControl),
        ] {
            assert_eq!(
                socket_token(&port(d, Dim::Scalar, Clock::Frame)),
                want,
                "{d:?} escalar mantem a cor do dominio"
            );
        }
    }
}
