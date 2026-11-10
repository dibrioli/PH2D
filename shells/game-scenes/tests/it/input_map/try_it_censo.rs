//! A prova «0 textos cortados, 0 números mortos» na vista C (plano §3): o mapa de fábrica e o de 11 acções (o
//! move_left numa manche), a 1600×1000 e 1280×720, com a confirmação aberta e fechada — só a acção da manche mostra
//! os 2 números.

use super::Mesa;
use super::lista::onze;
use ph2d_app_input_map::View;
use ph2d_app_input_map::view_try::state::TryHit;
use ph2d_editor_core::zones::Rect;
use ph2d_input::recipes::Recipe;

const SIZES: [(f32, f32); 2] = [(1600.0, 1000.0), (1280.0, 720.0)]; // LITERAL-PX-OK: as janelas do plano §8

#[test]
fn o_try_it_nao_corta_texto_nem_mostra_numero_morto() {
    super::em_cada_modo_de_texto(|| {
        for (label, map) in [
            ("fábrica", ph2d_input::InputMap::with_player_defaults()),
            ("onze", onze()),
        ] {
            for (w, h) in SIZES {
                let area = Rect::new(0.0, 0.0, w, h);
                let mut m = Mesa::aberta(View::TryIt);
                m.hero.input_map = map.clone();
                m.frame();
                for confirm in [false, true] {
                    if confirm {
                        m.click(m.at(TryHit::Recipe(Recipe::Shooter).id()));
                    }
                    m.paint_in(area);
                    let c = &m.tab().doc.census;
                    let at = format!("{label} {w}×{h} confirm={confirm}");
                    assert_eq!(c.elided(), Vec::<&str>::new(), "{at}: texto cortado");
                    let low = c
                        .texts
                        .iter()
                        .map(|t| t.0.y + t.0.h)
                        .fold(0.0_f32, f32::max);
                    let right = c
                        .texts
                        .iter()
                        .map(|t| t.0.x + t.0.w)
                        .fold(0.0_f32, f32::max);
                    assert!(right <= w, "{at}: texto fora da janela à direita ({right})");
                    assert!(low <= h, "{at}: texto abaixo da janela ({low} > {h})");
                    let stick = map.actions().iter().find(|a| a.is_analog()).map(|a| a.id);
                    for b in map.actions() {
                        let want = if Some(b.id) == stick { 2 } else { 0 };
                        assert_eq!(c.numbers_of(b.id), want, "{at}: números de {}", b.name);
                    }
                }
            }
        }
    });
}

/// As cenas C1..C4 armadas como a shell as arma (o mapa do dono, a aba pedida).
fn cena(c: &str) -> Mesa {
    let mut m = Mesa::new(View::Keyboard);
    let map =
        ph2d_app_game::smoke::input_map::arma(&mut m.tab_mut().doc, c).expect("cena do roteiro");
    m.hero.input_map = map;
    m.hero.store.request_input_map_tab();
    m.frame();
    m
}

/// ⛔ O «F1» do «Opens with» saía «…» na foto da C4 e o censo dizia que coube (ele comparava com folga, o pintor
/// não). O censo tem de dar o MESMO veredicto que o pintor a cada texto, e o pintor — que pinta também os chips e as
/// caixas do cartão — não corta nada nas cenas C1..C4.
#[test]
fn o_censo_do_try_it_ve_cada_corte_do_pintor() {
    super::em_cada_modo_de_texto(|| {
        use ph2d_editor_core::text_elide::elisao;
        for c in ["C1", "C2", "C3", "C4"] {
            for (w, h) in SIZES {
                let mut m = cena(c);
                let area = Rect::new(0.0, 0.0, w, h);
                let ((), medidos) = elisao::medindo(|| m.paint_in(area));
                let at = format!("{c} {w}×{h}");
                let cortados: Vec<String> = (medidos.iter())
                    .filter(|x| !x.coube())
                    .map(|x| format!("{:?} em {:.2} px ({})", x.texto, x.largura, x.onde))
                    .collect();
                assert_eq!(cortados, Vec::<String>::new(), "{at}: o pintor cortou");
                let census = &m.tab().doc.census;
                for (r, s, cortado) in &census.texts {
                    let pintor = medidos
                        .iter()
                        .find(|x| &x.texto == s && x.largura.to_bits() == r.w.to_bits());
                    if let Some(p) = pintor {
                        assert_eq!(
                            *cortado,
                            !p.coube(),
                            "{at}: o censo e o pintor discordam de {s:?}"
                        );
                    }
                }
            }
        }
    });
}

/// O cartão Controls da C4 cabe INTEIRO a 1600×1000 (a tela do jogo ao lado das caixas): o «Forget them» e cada
/// caixa do «Shown in the game» ficam acima do fundo da vista, sem rolar.
#[test]
fn o_cartao_controls_da_c4_cabe_inteiro_a_1600x1000() {
    let mut m = cena("C4");
    m.paint_in(Rect::new(0.0, 0.0, 1600.0, 1000.0)); // LITERAL-PX-OK: a janela do plano §8
    let doc = &m.tab().doc;
    let clip = doc.census.clip.expect("a vista rola dentro de um recorte");
    assert_eq!(
        doc.tryit.player_changed, 1,
        "a frase do jogador está à vista"
    );
    let mut alvos = vec![
        TryHit::Forget.id(),
        TryHit::OpenKey.id(),
        TryHit::PlayersCanChange.id(),
    ];
    alvos.extend(
        m.hero
            .input_map
            .actions()
            .iter()
            .map(|a| TryHit::Shown(a.id).id()),
    );
    for id in alvos {
        let r = doc.rect_of(id).expect("o alvo pintado");
        assert!(
            r.y + r.h <= clip.y + clip.h,
            "{id:?} abaixo do fundo ({} > {})",
            r.y + r.h,
            clip.y + clip.h
        );
    }
}

/// ⛔ A largura do chip do «Opens with» refeita por subtracção (`(t + inset·2) − inset·2`) perde o último bit de
/// metade das larguras, e o pintor corta a tecla em «…» (o «F1» da foto C4). Doze teclas = doze larguras: o pintor
/// não corta nenhuma e o censo concorda com ele em cada uma.
#[test]
fn a_tecla_que_abre_a_tela_nunca_sai_cortada() {
    use ph2d_editor_core::text_elide::elisao;
    use ph2d_tokens::TextRendering as R;
    let mut m = cena("C4");
    // ⚠️ No modo de fábrica do app (`CrispHeavyPlus`, com o −0,01 em de espaçamento) as larguras são inexactas em
    // f32; o `Default` dos testes dá-as exactas e escondia o corte.
    for modo in [R::Default, R::CrispHeavyPlus] {
        ph2d_text::set_active_text_rendering(modo);
        for n in 1..=12 {
            let k = ph2d_input::Key::named_in_script(&format!("f{n}"))
                .and_then(|v| v.first().copied())
                .expect("F1..F12");
            m.hero.input_map.controls_mut().open_key = k;
            m.frame();
            let ((), medidos) = elisao::medindo(|| m.paint_in(super::AREA));
            let label = k.label();
            let p = (medidos.iter())
                .find(|x| x.texto == label)
                .unwrap_or_else(|| panic!("{label}: a tecla pintada"));
            let at = format!("{modo:?} {label}");
            assert!(p.coube(), "{at} saiu {:?} em {} px", p.pintado, p.largura);
            let census = &m.tab().doc.census;
            let c = census
                .texts
                .iter()
                .find(|t| t.1 == label)
                .expect("no censo");
            assert!(!c.2, "{at}: o censo e o pintor concordam");
        }
    }
    ph2d_text::set_active_text_rendering(R::Default);
}
