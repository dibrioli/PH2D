//! **Os 16 slots `timeline-*` são APELIDOS PUROS** — cada um resolve para a mesma cor que um
//! slot geral, em **todos** os quatro temas. 64 pares, 0 divergências (medido 2026-08-30).
//!
//! # Por que este gate existe em vez da fusão
//!
//! O censo ([`docs/UI_New_and_Simple/medicoes/03_o_censo_de_cor.md`]) mostrou que os 16
//! `timeline-*` são **exactamente** os 16 apelidos de todo o sistema: os 34 slots dos nós são
//! valores distintos, e nenhum outro módulo introduziu um apelido. Fundi-los levaria a paleta de
//! **83 para 67 slots por tema** sem mover um pixel — e este gate é a prova executável de que o
//! «sem mover um pixel» é verdade.
//!
//! ⚠️ **Mas equivalência não é desejabilidade, e é por isso que a fusão não foi feita aqui.** A
//! referência que licenciámos (Adobe Spectrum, Apache-2.0) mantém tokens de COMPONENTE a
//! apelidar os globais **de propósito**: é o que deixa um componente divergir depois sem tocar
//! nos consumidores. Fundir troca 16 nomes por 58 sítios de chamada directos, e transforma
//! *«a playhead passa a ter cor própria»* de uma linha de token em sete de código. Essa é uma
//! decisão de **design system**, do Enio — não uma dedução que este gate autorize.
//!
//! # O que significa este gate ficar VERMELHO
//!
//! Que alguém deu **valor próprio** a um slot do Timeline. Nesse instante o censo envelheceu e a
//! aritmética da fusão (83 → 67) deixou de valer: ou o valor novo é intencional — e então o par
//! sai desta lista, com o número do censo corrigido —, ou é um engano de tema. ⛔ Não afrouxe a
//! comparação: o que ela mede é uma IGUALDADE de bytes, e uma igualdade que passa a ser
//! aproximada não afirma nada.

use ph2d_tokens::{ColorToken as C, Theme};

const PAIRS: &[(C, C)] = &[
    (C::TimelineCurve, C::Accent),
    (C::TimelineHandle, C::Accent),
    (C::TimelineKeySelected, C::Accent),
    (C::TimelineLoopBrace, C::Accent),
    (C::TimelinePlayhead, C::Accent),
    (C::TimelineSummaryRing, C::Accent),
    (C::TimelineHandleLine, C::AccentSoft),
    (C::TimelineLoopRegion, C::AccentSoft),
    (C::TimelineRowAlt, C::Bg2),
    (C::TimelineRulerBg, C::Bg2),
    (C::TimelineMarker, C::Warn),
    (C::TimelineSummaryKey, C::Warn),
    (C::TimelineKeyActive, C::AccentPress),
    (C::TimelineMissing, C::Danger),
    (C::TimelineKey, C::Text1),
    (C::TimelineRulerTick, C::Text3),
];

#[test]
fn the_sixteen_timeline_slots_are_pure_aliases() {
    let themes = [
        ("forge", Theme::Forge),
        ("workshop", Theme::Workshop),
        ("sunstone", Theme::Sunstone),
        ("blueprint", Theme::Blueprint),
    ];
    let mut bad = 0;
    for (tn, t) in themes {
        for (alias, general) in PAIRS {
            let a = alias.resolve(t);
            let g = general.resolve(t);
            let ok = a == g;
            if !ok {
                bad += 1;
            }
            println!(
                "{tn:9} {alias:?} = #{:02X}{:02X}{:02X}{:02X}  vs  {general:?} = \
                 #{:02X}{:02X}{:02X}{:02X}  {}",
                a.r,
                a.g,
                a.b,
                a.a,
                g.r,
                g.g,
                g.b,
                g.a,
                if ok { "IGUAL" } else { "DIFERE" }
            );
        }
    }
    println!("pares comparados: {} ; divergentes: {bad}", PAIRS.len() * 4);
    assert_eq!(bad, 0, "{bad} pares divergem — fundir NAO seria zero-pixel");
}
