//! ⭐⭐⭐ **A COLUNA PINTADA ENCOLHE COM A JANELA** — o elo que nenhum gate media.
//!
//! # Porque este gate teve de existir
//!
//! A lei da fracção ([`ChromeBands::default_dock_w`]) tinha **três** gates e nenhum media isto:
//! um lê a LEI, outro a PORTA do store, e o terceiro o **TEXTO** do `frame_layout`. ⛔ Nenhum
//! percorre a rota até ao fim e pergunta *«que rectângulo é que a coluna OCUPA?»* — que é a única
//! coisa que o artista vê, e é o que o report **«não diminuiu os painéis»** afirma.
//!
//! *Um gate que lê a lei e um gate que lê o texto do quadro deixam, entre eles, o elo que pinta.*
//!
//! # A tabela MEDIDA por este arnês (rect publicado, em px)
//!
//! | janela | esquerda | direita |
//! |---:|---:|---:|
//! | `1 930` | `308,0` | `304,0` |
//! | `1 600` | `308,0` | `304,0` |
//! | `1 366` | `308,0` | `304,0` |
//! | `1 194` | `269,2` | `265,7` |
//! | `1 133` | `255,5` | `252,1` |
//! | `1 024` | `230,9` | `227,9` |
//! | `900` | `220,0` | `220,0` |
//!
//! # ⚠️⚠️ E a tabela responde a uma pergunta de SMOKE que nenhum número dizia antes
//!
//! **A coluna só se mexe entre `976` e `1 366` px de janela** — acima disso a lei é inerte de
//! propósito (ela é um TECTO, não uma escala), e abaixo o **mínimo do painel** prende-a
//! (`220 × 1366 / 308 = 976` à esquerda, `989` à direita). ⛔ E a janela **abre em `1 024 px`**
//! ([`init.rs`](../../src/init.rs): `with_inner_size(1024, 768)`), que já está quase no chão ⇒
//! *num perfil novo o gesto que mostra a lei é ALARGAR a janela, e o roteiro que mandava
//! ESTREITAR apontava para a direcção onde não há nada para ver.*

use ph2d_editor_core::screens::hero::{HeroScreen, paint_hero_screen};
use ph2d_editor_core::screens::layout::{ChromeBands, DockSide};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;

/// Pinta quatro quadros numa janela de `w` px e devolve o rect mais largo publicado de cada lado.
///
/// ⚠️ **Quatro quadros porque o `DockSides::from_published` lê o quadro ANTERIOR** — num quadro só
/// nenhuma coluna está reservada e a medição apanhava o estado transitório.
///
/// ⚠️ E a visibilidade vem do **manifesto**: o `HeroScreen::new` nasce sem painel nenhum aberto, e
/// sem esta semente a sonda mede uma tela vazia (foi o que a 1.ª corrida leu — `0,0` em tudo).
fn colunas_pintadas_em(w: f32) -> (f32, f32) {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut h = HeroScreen::new(ph2d_editor_core::NodeId(1));
    ph2d_editor_core::panel::with_registry_ref(|reg| {
        for q in reg.panels() {
            h.panel_visibility
                .insert(q.manifest.id, q.manifest.default_visible);
        }
    });
    let mut scene = ph2d_vector::VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    let vp = Rect {
        x: 0.0,
        y: 0.0,
        w,
        h: 1024.0,
    };
    for _ in 0..4 {
        paint_hero_screen(&mut h, vp, &mut scene, &mut text);
    }
    let meio = w * 0.5;
    let (mut esq, mut dir) = (0.0f32, 0.0f32);
    for r in h.store.panel_rects() {
        if r.w <= 0.0 || r.w > w {
            continue;
        }
        if r.x + r.w * 0.5 < meio {
            esq = esq.max(r.w);
        } else {
            dir = dir.max(r.w);
        }
    }
    (esq, dir)
}

/// **O que o artista VÊ segue a lei** — medido no rect publicado, nunca na fórmula.
#[test]
fn a_coluna_pintada_encolhe_com_a_janela() {
    // ⭐ O CONTROLO vem primeiro: sem ele, um arnês que devolvesse `0,0` (nenhum painel aberto)
    //   passaria todas as desigualdades de «encolheu» por vácuo.
    let (e_ref, d_ref) = colunas_pintadas_em(1366.0);
    assert!(
        e_ref > 300.0 && d_ref > 300.0,
        "o arnês não pintou coluna nenhuma na referência ({e_ref}, {d_ref}) — ele mede uma tela \
         vazia e toda a asserção abaixo passaria por vácuo"
    );

    // Acima da referência a lei é INERTE: a coluna não cresce com o ecrã grande.
    for w in [1600.0f32, 1930.0] {
        let (e, d) = colunas_pintadas_em(w);
        assert!(
            (e - e_ref).abs() < 0.5 && (d - d_ref).abs() < 0.5,
            "janela {w:.0}: a coluna pintada foi ({e}, {d}) contra ({e_ref}, {d_ref}) na \
             referência — a lei é um TECTO e não uma escala, e ela cresceu"
        );
    }

    // Abaixo da referência ela ENCOLHE, e encolhe exactamente o que a lei diz.
    for w in [1194.0f32, 1133.0, 1024.0] {
        let (e, d) = colunas_pintadas_em(w);
        assert!(
            e < e_ref - 1.0 && d < d_ref - 1.0,
            "janela {w:.0}: a coluna pintada ({e}, {d}) NÃO encolheu contra ({e_ref}, {d_ref}) — \
             é o report «não diminuiu os painéis», e nenhum gate da lei o via"
        );
        for (nome, side, got) in [("esquerda", DockSide::Left, e), ("direita", DockSide::Right, d)]
        {
            let lei = ChromeBands::default_dock_w(side, w);
            assert!(
                (got - lei).abs() < 0.5,
                "janela {w:.0}, {nome}: o rect pintado é {got} e a lei diz {lei} — o fio entre a \
                 lei e o pixel está cortado, e os gates de cima não o veem"
            );
        }
    }

    // ⭐ E o CHÃO é o do painel: abaixo dele a coluna PARA, e é por isso que a faixa em que o
    //   artista vê alguma coisa acaba em `976` px.
    let (e_baixo, d_baixo) = colunas_pintadas_em(900.0);
    let min = ph2d_tokens::PANEL_MIN_W_PX;
    assert!(
        (e_baixo - min).abs() < 0.5 && (d_baixo - min).abs() < 0.5,
        "janela 900: a coluna pintada ({e_baixo}, {d_baixo}) devia parar no mínimo do painel \
         ({min}) — abaixo dele o cabeçalho e uma linha deixam de caber juntos"
    );
}

/// ⭐⭐ **O READOUT DIZ A COLUNA QUE DECIDE O REPORT** (`PH2D_DOCK_LOG=1`).
///
/// ⛔⛔ **DUAS rondas de report não chegaram a uma conclusão por falta desta coluna.** O log de
/// `resize` da shell diz a JANELA e cala o resto, e as duas causas possíveis — *a coluna está na
/// largura de FÁBRICA, que segue a janela* contra *está numa ESCOLHA gravada, que não segue* —
/// **leem-se exactamente iguais no ecrã**.
///
/// ⚠️ A agulha é o **par inteiro** e não o nome da variável: um readout que imprimisse só a
/// largura ficaria verde com um `grep` por `dock_width`, *que é precisamente o estado em que ele
/// não bissecta nada*.
#[test]
fn o_readout_das_colunas_diz_de_onde_vem_cada_largura() {
    const FASE: &str = include_str!("../../src/render_loop/fase_hero_paint.rs");
    for agulha in [
        "PH2D_DOCK_LOG",
        "dock_width_choice(side)",
        concat!("(escolha ", "{ce})"),
        concat!("(escolha ", "{cd})"),
    ] {
        assert!(
            FASE.contains(agulha),
            "o readout das colunas perdeu `{agulha}` — sem a coluna da ESCOLHA ele volta a não \
             distinguir «a largura de fábrica segue a janela» de «a escolha gravada não segue», \
             que é o que deixou duas rondas de report sem conclusão"
        );
    }
    // ⭐ E o CONTROLO: ela tem de sair por `eprintln!`, porque é essa a isenção do HR-15 aqui —
    //   montá-la num `format!` para uma variável perde-a **sem tirar o literal do binário**, e o
    //   censo de texto da shell reprovou a 1.ª redacção desta função exactamente assim.
    //
    // ⛔⛔ **A agulha NÃO pode exigir adjacência, e isso foi medido:** a 1.ª redacção procurava
    //   `format!("[dock]` colados e **uma mutação SOBREVIVEU** — no ficheiro real o `cargo fmt`
    //   põe a macro e o literal em linhas diferentes. *Uma agulha que depende da formatação é
    //   cega exactamente à formatação que o ficheiro tem.* ⇒ acha-se o literal e olha-se para
    //   TRÁS numa janela, que é layout-independente.
    let i = FASE
        .find(concat!("\"[dock] ", "janela="))
        .expect("o readout perdeu a linha `[dock] janela=` — ele deixou de existir");
    let antes = &FASE[i.saturating_sub(40)..i];
    assert!(
        antes.contains("eprintln!"),
        "a linha do readout não sai de um `eprintln!` (os 40 caracteres antes dela são {antes:?}) \
         — ela perde a isenção de terminal do HR-15 sem tirar o literal do binário, e o censo de \
         texto da shell reprova-a"
    );
}
