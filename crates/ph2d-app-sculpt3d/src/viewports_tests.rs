//! **OS QUATRO VIEWPORTS, medidos.**
//!
//! ⚠️ Precisam de um device (uma [`Sculpt3dScene`] não existe sem ele), mas
//! **não** de janela: tudo o que afirmam é geometria de câmera e de rectângulo.
//!
//! ```text
//! cargo test -p ph2d-host-desktop --bins sculpt3d::viewports
//! ```

use ph2d_viewport3d::views::Standard;
use ph2d_editor::zones::Rect;
use ph2d_mesh::shapes::uv_sphere;

const W: f32 = 960.0;
const H: f32 = 640.0;

/// Uma cena com uma esfera e a área do canvas publicada, ou nada a afirmar.
fn cena() -> Option<crate::Sculpt3dScene> {
    let gpu = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None).ok()?;
    let mut s = crate::Sculpt3dScene::new(&gpu.device, uv_sphere(16, 24, 1.0), 1.0);
    s.note_canvas(Rect::new(0.0, 0.0, W, H));
    Some(s)
}

macro_rules! cena_ou_sai {
    () => {
        match cena() {
            Some(s) => s,
            None => {
                eprintln!("no GPU adapter on this machine — nothing to assert");
                return;
            }
        }
    };
}

/// ⭐⭐⭐ **O CLIQUE CAI ONDE A PEÇA FOI DESENHADA — NOS QUATRO QUADRANTES.**
///
/// ⛔⛔ **É o gate desta wave inteira.** O desenho passou a acontecer num
/// sub-rectângulo do alvo e o pick continua a receber coordenadas de **JANELA**:
/// as duas metades só concordam se o gesto atravessar a porta
/// [`Sculpt3dScene::to_view`]. Sem ela, apontar o **centro** do quadrante de
/// baixo à direita — `(0,75·W, 0,75·H)` em janela — seria lido como
/// `(0,75·W, 0,75·H)` dentro de uma vista que mede `W/2 × H/2`, ou seja um NDC
/// de `(2, −2)`: **fora do écran**, e o raio erra a peça por completo.
///
/// ⚠️ **A régua é o ACERTO na malha, não a aritmética da conversão.** Comparar
/// `to_view` com `x − origem` seria a mesma linha escrita duas vezes; o que se
/// mede é a consequência que o artista vê.
#[test]
fn o_clique_cai_onde_a_peca_foi_desenhada_nos_quatro_quadrantes() {
    let mut s = cena_ou_sai!();
    assert!(s.toggle_split(), "a divisao nao abriu");
    assert_eq!(s.vp_count(), 4);
    for i in 0..4 {
        s.set_active_vp(i);
        let r = s.vp_rect(i).expect("o quadrante existe");
        let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
        let hit = s.pick_active(cx, cy).is_some();
        println!(
            "quadrante {i} rect ({:.0}, {:.0}, {:.0}, {:.0}) centro ({cx:.0}, {cy:.0}) -> {}",
            r.x,
            r.y,
            r.w,
            r.h,
            if hit { "ACERTOU" } else { "errou" }
        );
        assert!(
            hit,
            "o centro do quadrante {i} nao acerta a peca -- o pick esta' a ler coordenadas de \
             JANELA como se fossem da VISTA (falta a porta `to_view`)"
        );
    }
}

/// ⭐⭐ **UM PONTO FORA DO CANVAS NÃO É DE VIEWPORT NENHUM.**
///
/// ⚠️ **É uma RECUSA nova e ela é uma cura:** enquanto a peça era desenhada na
/// janela inteira, a cena engolia cliques sobre as réguas e sobre a faixa da
/// esquerda — havia malha ali, por baixo do chrome.
#[test]
fn um_ponto_fora_do_canvas_nao_e_de_viewport_nenhum() {
    let s = cena_ou_sai!();
    assert_eq!(
        s.vp_at(W * 0.5, H * 0.5),
        Some(0),
        "o centro e' do viewport unico"
    );
    for (x, y, onde) in [
        (-4.0, H * 0.5, "a` esquerda"),
        (W + 4.0, H * 0.5, "a` direita"),
        (W * 0.5, -4.0, "acima"),
        (W * 0.5, H + 4.0, "abaixo"),
    ] {
        assert_eq!(
            s.vp_at(x, y),
            None,
            "um ponto {onde} do canvas foi aceite como sendo da cena"
        );
    }
}

/// ⭐⭐⭐ **ABRIR A DIVISÃO PÕE O ARTISTA EM BAIXO À DIREITA E NOMEIA OS OUTROS
/// TRÊS** — a disposição do Blender, que é onde a mão dele já está.
#[test]
fn abrir_a_divisao_nomeia_tres_e_deixa_a_do_artista_no_canto_da_mao() {
    let mut s = cena_ou_sai!();
    // Uma vista LIVRE antes de abrir, para que «a do artista» seja reconhecível.
    s.camera.aim(0.9, 0.35);
    let antes = (s.camera.yaw, s.camera.pitch);
    assert!(s.toggle_split());
    let nomes: Vec<_> = (0..4)
        .map(|i| super::super::navball::named_view(&s.cam_of(i)))
        .collect();
    println!("quadrantes: {nomes:?}");
    assert_eq!(
        nomes[..3],
        [
            Some(Standard::Top),
            Some(Standard::Right),
            Some(Standard::Front)
        ],
        "a disposicao nao e' a do Blender (topo, direita, frente, artista)"
    );
    assert_eq!(
        nomes[3], None,
        "o quarto quadrante devia ser a vista LIVRE do artista"
    );
    assert_eq!(s.vp_active(), 3, "o activo devia ser o do artista");
    assert_eq!(
        (s.camera.yaw, s.camera.pitch),
        antes,
        "a camera do artista nao viajou com ele para o quadrante dele"
    );
}

/// ⭐⭐ **TROCAR DE QUADRANTE GUARDA A CÂMERA DO QUE SAI.**
///
/// ⛔⛔ **É a invariante do registo «em uso»**: a `Sculpt3dScene::camera` manda
/// para o activo e `vp_cams[activo]` está velha. Sem o `set_active_vp` guardar,
/// tocar noutro quadrante e voltar deitaria fora o enquadramento que o artista
/// escolheu — e o modo de falha seria *«a vista salta quando eu clico noutra»*.
#[test]
fn trocar_de_quadrante_guarda_a_camera_do_que_sai() {
    let mut s = cena_ou_sai!();
    assert!(s.toggle_split());
    s.set_active_vp(3);
    s.camera.aim(1.234, 0.321);
    let meu = (s.camera.yaw, s.camera.pitch);
    // Passeia pelos outros três e volta.
    for i in [0, 1, 2, 0] {
        s.set_active_vp(i);
    }
    s.set_active_vp(3);
    assert_eq!(
        (s.camera.yaw, s.camera.pitch),
        meu,
        "voltar ao quadrante do artista devolveu outra camera"
    );
    // E a de um quadrante nomeado continua a ser a dele.
    s.set_active_vp(0);
    assert_eq!(
        super::super::navball::named_view(&s.camera),
        Some(Standard::Top),
        "o quadrante 0 deixou de ser o Topo depois de o artista passear"
    );
}

/// ⭐⭐ **FECHAR A DIVISÃO FICA COM A VISTA ACTIVA**, nunca com «a primeira».
///
/// ⚠️ O artista fecha a olhar para o quadrante que lhe interessa; ficar com
/// outro seria desfazer-lhe o gesto.
#[test]
fn fechar_a_divisao_fica_com_a_vista_activa() {
    let mut s = cena_ou_sai!();
    assert!(s.toggle_split());
    s.set_active_vp(1);
    let direita = (s.camera.yaw, s.camera.pitch);
    assert!(!s.toggle_split(), "a divisao nao fechou");
    assert_eq!(s.vp_count(), 1);
    assert_eq!(
        (s.camera.yaw, s.camera.pitch),
        direita,
        "fechar a divisao trocou a camera -- ela devia ficar com a que estava ACTIVA"
    );
}

/// ⭐⭐ **A COSTURA ARRASTA, E O EIXO CERTO SE MOVE.**
///
/// ⚠️ **As duas metades são o gate.** A costura VERTICAL move o `tx` e a
/// HORIZONTAL move o `ty`; trocá-las compila, e o sintoma é arrastar a linha de
/// cima e ver a do lado mexer-se. Só a primeira metade (*«mexeu»*) deixaria a
/// troca passar.
#[test]
fn a_costura_arrasta_e_o_eixo_certo_se_move() {
    let mut s = cena_ou_sai!();
    assert!(s.toggle_split());
    let largura0 = s.vp_rect(0).expect("q0").w;
    let altura0 = s.vp_rect(0).expect("q0").h;
    // A costura VERTICAL está em x = W/2; puxa-a para a direita.
    assert!(
        s.seam_grab(W * 0.5, H * 0.5),
        "o cruzamento das costuras nao foi agarrado"
    );
    assert!(
        s.seam_at(W * 0.7, H * 0.5),
        "o arrasto da costura nao foi aceite"
    );
    assert!(s.seam_release());
    let q0 = s.vp_rect(0).expect("q0");
    println!("q0: {largura0} x {altura0} -> {} x {}", q0.w, q0.h);
    assert!(
        q0.w > largura0 + 1.0,
        "puxar a costura vertical para a direita nao alargou o quadrante da esquerda \
         ({largura0} -> {})",
        q0.w
    );
    assert!(
        (q0.h - altura0).abs() < 0.5,
        "puxar a costura VERTICAL mexeu na altura ({altura0} -> {}) -- os dois eixos estao trocados",
        q0.h
    );
}

/// ⭐⭐⭐ **O *FIT* ENQUADRA PARA A VISTA, NÃO PARA A JANELA.**
///
/// ⛔⛔ **A wave dos viewports criou uma SEGUNDA resposta a «qual é o aspecto?»**
/// e este gate fecha-a: o desenho passou a usar o do rectângulo da vista e os
/// três chamadores do enquadramento continuavam a passar o da **janela**.
///
/// # ⛔ A PRIMEIRA fixtura não produzia o fenómeno
///
/// Ela media *«que fracção da LARGURA a peça ocupa»* numa vista quadrada contra
/// uma deitada — e leu `0,4862` contra `0,1621`, acusando código correcto. A
/// razão é geometria: o [`ph2d_mesh_render::Camera3d::frame`] toma o **máximo**
/// das duas restrições, e numa vista **deitada** quem manda é sempre a
/// **altura** (o `fov` é vertical). ⇒ a fracção da largura *tem* de diferir, e o
/// aspecto **nunca chega a morder**: com uma esfera (caixa simétrica) numa vista
/// larga, passar o aspecto certo ou o errado dá a **mesma distância**, e o
/// controlo lia `0,4862` contra `0,4862`.
///
/// ⇒ **duas correcções**: a régua passa a ser `max(fracção da largura, fracção
/// da altura)` — que é o que *enquadrado* quer dizer, «toca um dos bordos» —, e a
/// fixtura passa a incluir uma vista **ALTA**, que é onde a restrição horizontal
/// manda e onde o aspecto errado se vê. *Uma fixtura em que o knob não morde não
/// testa o knob.*
#[test]
fn o_fit_enquadra_para_a_vista_e_nao_para_a_janela() {
    /// Quanto da vista a peça ocupa — o maior dos dois eixos. Enquadrado ⇒ ~1.
    fn ocupacao(s: &crate::Sculpt3dScene) -> f32 {
        let (w, h) = s.viewport();
        let (mut x0, mut x1, mut y0, mut y1) = (
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::INFINITY,
            f32::NEG_INFINITY,
        );
        for p in s.mesh().positions() {
            if let Some((x, y)) = s.camera.project(*p, (w, h)) {
                x0 = x0.min(x);
                x1 = x1.max(x);
                y0 = y0.min(y);
                y1 = y1.max(y);
            }
        }
        ((x1 - x0) / w as f32).max((y1 - y0) / h as f32)
    }
    // ⚠️ O `fallback` é o aspecto da JANELA, e é ele que a porta tem de ignorar
    // quando conhece a vista.
    const JANELA: f32 = 3.0;
    let mut ocupacoes = Vec::new();
    for (w, h) in [(600.0, 600.0), (1200.0, 400.0), (400.0, 1200.0)] {
        let mut s = cena_ou_sai!();
        s.note_canvas(Rect::new(0.0, 0.0, w, h));
        s.frame_all(JANELA);
        ocupacoes.push(((w, h), ocupacao(&s)));
    }
    // ⚠️ **O CONTROLO**: a vista ALTA enquadrada com o aspecto da JANELA. É aqui
    // que a restrição horizontal manda, e é o único enquadramento das três em
    // que passar o aspecto errado muda a distância.
    let mut errada = cena_ou_sai!();
    errada.note_canvas(Rect::new(0.0, 0.0, 400.0, 1200.0));
    errada.camera.frame(errada.world_bounds(), JANELA);
    let f_errada = ocupacao(&errada);

    println!(
        "ocupacao por vista: {ocupacoes:?} | vista ALTA com o aspecto da JANELA: {f_errada:.4}"
    );
    // ⚠️⚠️ **A régua é «CABE», e não uma fracção fixa** — e a fracção fixa foi a
    // terceira redacção errada deste gate. O [`Camera3d::frame`] enquadra a
    // **CAIXA** da peça, não a peça: uma esfera dentro do cubo dela ocupa `0,49`
    // e não `0,87`, e com o cubo visto de um ângulo (`yaw 0,6`, `pitch 0,35`) as
    // extensões horizontal e vertical dele diferem — logo a ocupação **muda com
    // o aspecto** mesmo com o enquadramento perfeitamente correcto (`0,488` nas
    // vistas quadrada e deitada, `0,591` na alta). *Um invariante tem de ser uma
    // propriedade da lei, não um número que a fixtura calhou de dar.*
    //
    // ⇒ o que a lei promete é **caber**, com a peça a ocupar a vista de verdade.
    for ((w, h), f) in &ocupacoes {
        assert!(
            *f <= 1.0,
            "a vista {w}x{h} deixou a peca TRANSBORDAR ({f:.4} da vista) -- enquadrar quer dizer \
             caber, e o aspecto usado nao e' o dela"
        );
        assert!(
            *f > 0.3,
            "a vista {w}x{h} pos a peca a {f:.4} da vista -- ela cabe e nao esta' enquadrada, \
             que e' o outro modo de falhar de um `fit`"
        );
    }
    assert!(
        f_errada > 1.0,
        "o CONTROLO deu {f_errada:.4}: com o aspecto da JANELA (3,0) numa vista ALTA a peca tinha \
         de TRANSBORDAR, e sem essa diferenca este gate nao afirma nada"
    );
}

/// ⭐⭐⭐ **O NOME DA VISTA É UM BOTÃO: ele abre a lista, e a lista troca a
/// câmera daquele quadrante.**
///
/// ⛔ **REPORT DO ENIO, 2026-09-08:** *«ao clicar nos nomes das views não
/// aparece a lista de view como no módulo de modelagem 3d»*.
///
/// ⚠️ **Os chips são publicados à mão nesta fixture**, e é honesto: a GEOMETRIA
/// deles é lei do módulo vizinho (`ph2d_viewport3d::view_menu::chip`, já gateada lá) e o
/// que esta wave acrescenta é a **fiação** — quem os guarda, quem os aponta, e
/// o que o clique faz. Que o pintor os publique de verdade é o censo irmão.
#[test]
fn o_nome_da_vista_abre_a_lista_e_a_lista_troca_a_camera() {
    let mut s = cena_ou_sai!();
    assert!(s.toggle_split());
    // Um chip por quadrante, no canto de cada um.
    let chips: Vec<_> = (0..4)
        .map(|i| {
            s.vp_rect(i)
                .map(|r| Rect::new(r.x + 4.0, r.y + 4.0, 60.0, 20.0))
        })
        .collect();
    s.note_view_labels(chips.clone());

    // (1) — o chip do quadrante 1 é apontável, e abre o menu DELE.
    let c = chips[1].expect("o chip existe");
    assert_eq!(s.chip_at(c.x + 2.0, c.y + 2.0), Some(1));
    s.set_active_vp(1);
    s.open_view_menu(1);
    assert_eq!(s.view_menu_open(), Some(1));

    // (2) — a escolha troca a câmera daquele quadrante.
    let menu = Rect::new(c.x, c.y + c.h, 120.0, 26.0 * 6.0 + 16.0);
    s.note_view_menu_rect(menu);
    // A linha do `Top` é a quinta da lista (`Standard::ALL`).
    let alvo = Standard::Top;
    let i = Standard::ALL
        .iter()
        .position(|v| *v == alvo)
        .expect("Top esta' na lista");
    let y = menu.y + 8.0 + 26.0 * i as f32 + 4.0;
    assert!(
        s.view_menu_click(menu.x + 10.0, y),
        "o clique no menu nao foi consumido"
    );
    assert_eq!(
        s.view_menu_open(),
        None,
        "o menu tinha de fechar ao escolher"
    );
    assert_eq!(
        super::super::navball::named_view(&s.cam_of(1)),
        Some(alvo),
        "escolher `{}` no menu do quadrante 1 nao lhe trocou a camera",
        alvo.key()
    );
}

/// ⭐⭐ **UM CLIQUE FORA DO MENU FECHA-O — e não faz mais nada.**
///
/// ⚠️ **A segunda metade é o gate.** Deixar o clique de fora passar orbitaria a
/// peça no mesmo gesto em que o artista só queria desistir do menu, e é o que
/// todo o chrome desta casa já faz.
#[test]
fn um_clique_fora_do_menu_fecha_o_e_nao_faz_mais_nada() {
    let mut s = cena_ou_sai!();
    assert!(s.toggle_split());
    s.set_active_vp(1);
    s.open_view_menu(1);
    let menu = Rect::new(500.0, 100.0, 120.0, 170.0);
    s.note_view_menu_rect(menu);
    let antes = super::super::navball::named_view(&s.cam_of(1));
    // Bem longe do menu.
    assert!(
        s.view_menu_click(20.0, 600.0),
        "o clique fora do menu tem de ser CONSUMIDO -- senao ele orbita a peca no mesmo gesto"
    );
    assert_eq!(s.view_menu_open(), None, "o menu tinha de fechar");
    assert_eq!(
        super::super::navball::named_view(&s.cam_of(1)),
        antes,
        "desistir do menu trocou a camera na mesma"
    );
}

/// ⭐ **`Escape` fecha o menu, e SÓ quando ele está aberto.**
///
/// ⚠️ A segunda metade é o que o mantém invisível: `Escape` é a tecla de
/// desistir de meio mundo, e um handler que a reclamasse sempre roubaria o
/// cancelar de quem vem a seguir no roteador.
#[test]
fn o_escape_fecha_o_menu_e_so_quando_ele_esta_aberto() {
    let mut s = cena_ou_sai!();
    assert!(
        !s.close_view_menu(),
        "sem menu aberto o `Escape` nao pode dizer que consumiu -- ele roubaria o cancelar de \
         quem vem a seguir"
    );
    s.open_view_menu(0);
    assert!(s.close_view_menu());
    assert_eq!(s.view_menu_open(), None);
}

/// ⭐⭐ **COM UMA VISTA SÓ NÃO HÁ CHIP** — e é a diferença entre *inalcançável* e
/// *invisível-mas-clicável*.
///
/// ⚠️ Com uma vista a pergunta *«qual é qual?»* não existe, então o rótulo não é
/// pintado. Um alvo de clique que sobrevivesse ao rótulo seria exactamente o
/// «controlo morto sob o dedo» que esta casa já caçou — só que ao contrário:
/// vivo sob o dedo e invisível ao olho.
#[test]
fn com_uma_vista_so_nao_ha_chip() {
    let mut s = cena_ou_sai!();
    assert_eq!(s.vp_count(), 1);
    s.note_view_labels(Vec::new());
    assert_eq!(s.chip_at(10.0, 10.0), None);
    assert_eq!(s.chip_at(W * 0.5, H * 0.5), None);
}

/// ⭐⭐⭐ **O DESPACHO PERGUNTA NA ORDEM CERTA: menu · costura · chip.**
///
/// ⛔⛔ **A precedência é do vizinho, e ele já a pagou:** o cabeçalho do quadrante
/// de baixo-direita nasce encostado ao cruzamento das costuras, então o menu que
/// ele abre cai **por cima da banda de agarrar o divisor** — metade das linhas
/// dele ficaria inalcançável. *Uma precedência escrita por analogia («a costura
/// ganha de tudo») deixa de valer quando nasce algo que é modal.*
///
/// ⚠️ E o **chip vem depois da costura**, porque ele vive **dentro** de um
/// viewport e ela vive **entre** eles.
#[test]
fn o_despacho_pergunta_menu_costura_chip_nesta_ordem() {
    let fonte = include_str!("input_down.rs");
    let onde = |agulha: &str| {
        fonte
            .find(agulha)
            .unwrap_or_else(|| panic!("controlo positivo: `{agulha}` sumiu do despacho"))
    };
    let (menu, costura, chip, vp) = (
        onde("scene.view_menu_open()"),
        onde("scene.seam_grab("),
        onde("scene.chip_at("),
        onde("scene.vp_at("),
    );
    println!("ordem: menu {menu} < costura {costura} < chip {chip} < viewport {vp}");
    assert!(
        menu < costura,
        "a costura e' perguntada antes do MENU -- o menu do quadrante de baixo-direita cai por \
         cima da banda do divisor, e metade das linhas dele fica inalcancavel"
    );
    assert!(
        costura < chip,
        "o chip e' perguntado antes da COSTURA -- ele vive DENTRO de um viewport e ela vive ENTRE \
         eles"
    );
    assert!(
        chip < vp,
        "o chip tem de ser perguntado antes do encaminhamento por viewport"
    );
}

/// ⭐⭐ **O QUADRO PUBLICA O QUE O PINTOR MEDIU** — o chip e o rectângulo do menu.
///
/// ⛔ **Sem isto o botão é invisível ao ponteiro:** a largura do chip é a do
/// TEXTO e só o pintor a mede, então descartar o retorno do `paint_view_label`
/// deixa a lista de alvos vazia — o nome aparece na tela e o clique atravessa-o.
/// *É a forma exacta do report, e nada além de um censo a apanha: o gate de
/// registo mede ids, e aqui não há id nenhum.*
#[test]
fn o_quadro_publica_o_chip_e_o_rectangulo_que_o_pintor_mediu() {
    // ⚠️ **`../` desde 2026-09-11 (W2/L3-A3)**: um `include_str!` é relativo ao directório do
    // ficheiro que o escreve, e este desceu um nível ao entrar em `src/sculpt3d/`. O
    // `render_loop/` fica na shell — ele **não** é da família, e é essa a razão de ainda se
    // alcançar por caminho e não por módulo.
    let fonte = include_str!("../render_loop/mod.rs");
    for (chamada, porta) in [
        ("paint_view_label(", "note_view_labels("),
        ("paint_view_menu(", "note_view_menu_rect("),
    ] {
        let at = fonte
            .find(chamada)
            .unwrap_or_else(|| panic!("controlo positivo: `{chamada}` sumiu do quadro"));
        // A publicação tem de vir depois da chamada, e perto dela.
        let depois = &fonte[at..];
        assert!(
            depois.find(porta).is_some_and(|d| d < 1500),
            "o quadro chama `{chamada}` e nao publica o resultado por `{porta}` -- o alvo do \
             clique fica vazio e o nome aparece na tela com o clique a atravessa'-lo"
        );
    }
}
