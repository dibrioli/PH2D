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
        for (nome, side, got) in [
            ("esquerda", DockSide::Left, e),
            ("direita", DockSide::Right, d),
        ] {
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

/// ⛔⛔ **O ARRASTO PASSA PELA PORTA DO ARRASTO** — e nenhum gate media isto.
///
/// O gate de costura irmão (`the_border_gesture_reaches_the_panel`) procura `set_dock_width` no
/// `dock_seam_move`, e **`set_dock_width_from_drag` contém esse nome** ⇒ ele fica verde com a
/// regressão inteira dentro. *Uma agulha que é PREFIXO da cura não distingue a cura do defeito.*
///
/// A cura vive no sítio que conhece a JANELA (o `HeroLayout`), logo a metade que a defende tem de
/// ser lida no CHAMADOR — e a agulha é o **braço inteiro**, nunca o nome da função.
#[test]
fn o_arrasto_da_borda_passa_pela_porta_que_conhece_a_janela() {
    const RESIZE: &str = include_str!("../../src/dock_resize.rs");
    assert!(
        RESIZE.contains("ChromeBands::escolha_de_um_arrasto("),
        "o arrasto deixou de passar pela porta que conhece a janela — sem ela um gesto que aterra \
         na largura de FÁBRICA volta a gravá-la como ESCOLHA, e a coluna sai da lei da fracção \
         para sempre (o report de 2026-09-20, com `dock_w_right=220` no ficheiro de arrumação)"
    );
    // ⭐ E o CONTROLO: o que o arrasto escreve tem de ser a resposta DELA, nunca um `Some(w)`
    //   montado aqui — senão a decisão volta a ser tomada no sítio que não é a lei.
    assert!(
        RESIZE.contains("set_dock_width(drag.side, escolha)"),
        "o arrasto deixou de escrever a resposta da lei — se ele monta o `Some` sozinho, a \
         decisão saiu da porta e volta a ser invisível"
    );
}

/// Pinta quatro quadros numa janela de `w` px e devolve o `HeroScreen` com o layout publicado.
///
/// ⚠️ Irmã da [`colunas_pintadas_em`], e pelas mesmas duas razões: quatro quadros porque o
/// `DockSides::from_published` lê o ANTERIOR, e a visibilidade semeada do manifesto porque o
/// `HeroScreen::new` nasce sem painel nenhum aberto.
fn hero_pintado_em(w: f32) -> HeroScreen {
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
        h: 1052.0,
    };
    for _ in 0..4 {
        paint_hero_screen(&mut h, vp, &mut scene, &mut text);
    }
    h
}

/// ⛔⛔⛔ **O REPORT DE 2026-09-20 REPRODUZIDO: numa janela estreita, arrastar a borda era INERTE.**
///
/// > *«não funciona. pare de tentar. permita que manualmente o usuário consiga estreitar o
/// > painel.»* — Enio, 2026-09-20.
///
/// # A causa, medida
///
/// Numa janela estreita a lei da fracção **já entrega o mínimo**, e até esse dia o piso da
/// ESCRITA era o mesmo número ⇒ o gesto pedia `157` e o store devolvia `220`. *Um gesto que
/// existe, arma, segue o dedo e não muda um pixel lê-se como um gesto partido* — e nenhum gate
/// desta linha o via, porque todos mediam a largura de FÁBRICA.
///
/// # ⚠️ Ele percorre a CORRENTE do arrasto, não a fórmula
///
/// As quatro portas que o `App::dock_seam_move` encadeia: onde a costura está
/// ([`HeroLayout::dock_seam`]), que largura o `x` pede ([`HeroLayout::dock_width_for`]), se isso é
/// uma escolha ([`ChromeBands::escolha_de_um_arrasto`]) e o que fica gravado
/// (`WidgetStore::set_dock_width`). ⛔ O elo que falta — a shell chamar isto — é o
/// [`o_arrasto_da_borda_passa_pela_porta_que_conhece_a_janela`], que o lê por texto: o `App` pede
/// uma janela de verdade e não é alcançável daqui.
#[test]
fn numa_janela_estreita_o_arrasto_ainda_estreita_a_coluna() {
    use ph2d_editor_core::interaction::WidgetStore;

    // ⭐ A janela do report: aqui a lei da fracção já está no piso dela nos dois lados.
    const ESTREITA: f32 = 640.0;
    let mut h = hero_pintado_em(ESTREITA);
    let layout = h.last_layout.expect("o arnês não publicou layout nenhum");

    for side in [DockSide::Left, DockSide::Right] {
        let antes = h.store.dock_width(side, ESTREITA);
        // ⭐ O CONTROLO da fixtura: sem isto o gate podia estar a medir uma janela em que a lei
        //   ainda tem folga, e aí o arrasto já funcionava ANTES da cura — ela não conteria o
        //   fenómeno, que é como uma fixtura aprova o defeito que existe para apanhar.
        assert!(
            (antes - ph2d_tokens::PANEL_MIN_W_PX).abs() < 0.5,
            "{side:?}: a fixtura não contém o report — a `{ESTREITA}` a coluna mede {antes} e o \
             report acontece com ela NO piso de fábrica ({})",
            ph2d_tokens::PANEL_MIN_W_PX
        );

        let seam = layout.dock_seam(side);
        assert!(
            seam.w > 0.0,
            "{side:?}: não há costura para agarrar — o gesto nem chega a armar"
        );
        let px = seam.x + seam.w * 0.5;
        // Arrastar 60 px para DENTRO: à esquerda o `x` diminui, à direita aumenta.
        let alvo = match side {
            DockSide::Left => px - 60.0,
            DockSide::Right => px + 60.0,
        };
        let pedido = layout.dock_width_for(side, alvo);
        let escolha = ChromeBands::escolha_de_um_arrasto(side, pedido, layout.viewport.w);
        h.store.set_dock_width(side, escolha);
        let depois = h.store.dock_width(side, ESTREITA);

        assert!(
            depois < antes - 1.0,
            "{side:?}: a coluna mediu {antes} antes e {depois} depois de um arrasto de 60 px — a \
             borda é INERTE, que é o report de 2026-09-20 à letra"
        );
        // ⚠️ E aterra ONDE O DEDO PEDIU, não no piso — a 1.ª redacção deste gate exigia o piso e
        //    reprovou sobre a cura a funcionar (`220 → 157`, que é exactamente o gesto). *Um
        //    arrasto de 60 px pede 60 px; o piso só entra quando o pedido passa por baixo dele.*
        assert!(
            pedido > WidgetStore::DOCK_W_MIN,
            "{side:?}: o arrasto desta fixtura ({pedido}) já pede por baixo do piso ({}) — então \
             ela mede o clamp e não o gesto",
            WidgetStore::DOCK_W_MIN
        );
        assert!(
            (depois - pedido).abs() < 0.5,
            "{side:?}: o dedo pediu {pedido} e ficou gravado {depois} — entre a lei e o store há \
             um número a mudar de valor"
        );
    }
}

/// ⭐⭐⭐ **O PISO DIZ DE QUE RECURSO É: ele é EXACTAMENTE a largura que o corpo precisa.**
///
/// ⛔⛔ **Abaixo do piso esta régua não consegue medir, e a razão é o próprio piso:** a única
/// porta que escreve a largura de uma coluna clampa nele, logo pedir `83` devolve `84`. *Uma
/// régua que quer ver o outro lado de uma cerca teria de derrubar a cerca* — e foi isso que a
/// 1.ª redacção deste gate tentou, reprovando com *«a coluna não chegou a 83»*.
///
/// ⇒ ela afirma a metade que se pode medir de DENTRO: **no piso, nada do corpo sai da coluna.**
/// Um piso mais apertado que o recurso reprova aqui (medido: a `60` saem dois controlos).
///
/// # ⛔⛔⛔ E a outra metade — *«o piso não está SOLTO»* — NÃO é gateável, com o mecanismo
///
/// Ela ficou registada como **mutação NOMEADA** (um piso de `120` sobrevive a esta suíte), e a
/// razão não é falta de vontade: **três** réguas foram construídas e as três medem o piso em vez
/// do recurso.
///
/// 1. *«alguma coisa toca a borda»* — quase toda fileira de painel é **ELÁSTICA** e enche a
///    coluna seja qual for a largura ⇒ verdadeira em todo número que se escreva ali.
/// 2. *«o controlo fixo mais largo»*, com o 2.º quadro em `piso + folga` — os dois quadros
///    **movem-se com a constante medida**, e um piso de `120` lê `118`. *Uma barra derivada da
///    constante que ela mede não pode medi-la.*
/// 3. A mesma, com a âncora **fixa** no `PANEL_MIN_W_PX` — os controlos de largura fixa do
///    cabeçalho são **alinhados à DIREITA**, logo o canto direito deles acompanha a coluna e
///    `(x − col.x) + w` volta a ler `piso − 2` em qualquer piso.
///
/// ⚠️ **O que falta para a fechar é uma porta que escreva uma largura ABAIXO do piso** — e a
/// única que existe é a que o piso guarda. *Uma régua que quer ver o outro lado de uma cerca
/// teria de derrubar a cerca*, e um `set` sem clamp só para teste seria a segunda porta pela
/// qual o defeito de 2026-09-20 voltava.
///
/// ⇒ o joelho (`84`) foi medido **uma vez**, baixando o piso à mão e varrendo pixel a pixel; a
/// tabela vive no doc do [`WidgetStore::DOCK_W_MIN`] e é a proveniência do número.
///
/// ⚠️ **A régua é o transbordo do CORPO e nunca o absoluto**, e isso foi medido: há um controlo
/// de `36 × 36` px que já sai da coluna **`7 px` na largura de fábrica de `220`**, e a posição
/// dele nem é monótona na largura (`x` lê `191` a `220`, `50` a `84` e `90` a `90`). *Uma régua
/// de transbordo absoluto neste app mede esse widget e não o piso* — ele fica NOMEADO e não é
/// desta wave, porque a essa largura o produto de hoje shipa igual. É por isso que o filtro
/// abaixo corta cromo de largura inteira e o que vive acima da coluna.
///
/// ⛔ E o piso **não** é o da faixa de abas: o `tab_plan` ainda entrega uma aba a `32 px`. O que
/// falha primeiro é o CORPO, e é dele que o número fala.
#[test]
fn o_piso_de_uma_escolha_e_onde_o_corpo_do_painel_ainda_cabe() {
    use ph2d_editor_core::interaction::WidgetStore;

    let janela = 1920.0f32;
    let piso = WidgetStore::DOCK_W_MIN;
    let mut h = hero_pintado_em(janela);
    h.store.set_dock_width(DockSide::Left, Some(piso));
    // ⚠️ Repintar: a largura só chega ao layout no quadro seguinte.
    let mut scene = ph2d_vector::VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    let vp = Rect {
        x: 0.0,
        y: 0.0,
        w: janela,
        h: 1052.0,
    };
    for _ in 0..4 {
        paint_hero_screen(&mut h, vp, &mut scene, &mut text);
    }
    let layout = h.last_layout.expect("sem layout");
    let (col, _) = layout.side_columns();

    // ⭐ O CONTROLO: a coluna tem de ter CHEGADO ao piso, senão tudo o que se segue mede outra
    //   largura. Foi esta linha que apanhou a primeira medição desta wave a correr contra um
    //   clamp que eu tinha restaurado — as quatro leituras liam `220` e pareciam um planalto.
    assert!(
        (col.w - piso).abs() < 0.5,
        "a coluna não chegou ao piso (pedi {piso}, mede {}) — esta régua mede o clamp",
        col.w
    );

    let borda = col.x + col.w;
    let (mut fora, mut pior) = (0usize, 0.0f32);
    for (_, r) in h.hit_index.iter_registrations() {
        // Só o que NASCE na faixa de `x` da coluna…
        if r.x < col.x - 0.5 || r.x > borda + 0.5 {
            continue;
        }
        // …⛔ e não é cromo de largura INTEIRA (a barra de menu, o canvas), que nasce em `x = 0`
        //    e atravessa o ecrã. Sem este corte a régua lê `1 700 px` de transbordo.
        if r.w > col.w * 2.0 + 8.0 {
            continue;
        }
        // …⛔ nem vive ACIMA da coluna (a barra de menu tem itens dentro da faixa de `x` dela).
        if r.y < col.y {
            continue;
        }
        if r.x + r.w > borda + 0.5 {
            fora += 1;
            pior = pior.max(r.x + r.w - borda);
        }
    }

    assert_eq!(
        fora, 0,
        "no piso ({piso}) o corpo de um painel já sai da coluna (pior {pior:.1} px) — o número \
         deixou de descrever o recurso de que ele fala, e a cura é SUBI-LO com a medição ao \
         lado, nunca deixá-lo mentir"
    );
}
