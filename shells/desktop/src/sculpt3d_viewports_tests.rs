//! **OS QUATRO VIEWPORTS, medidos.**
//!
//! ⚠️ Precisam de um device (uma [`Sculpt3dScene`] não existe sem ele), mas
//! **não** de janela: tudo o que afirmam é geometria de câmera e de rectângulo.
//!
//! ```text
//! cargo test -p ph2d-host-desktop --bins sculpt3d::viewports
//! ```

use crate::field3d_views::Standard;
use ph2d_editor::zones::Rect;
use ph2d_mesh::shapes::uv_sphere;

const W: f32 = 960.0;
const H: f32 = 640.0;

/// Uma cena com uma esfera e a área do canvas publicada, ou nada a afirmar.
fn cena() -> Option<crate::sculpt3d::Sculpt3dScene> {
    let gpu = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None).ok()?;
    let mut s = crate::sculpt3d::Sculpt3dScene::new(&gpu.device, uv_sphere(16, 24, 1.0), 1.0);
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
    assert_eq!(s.vp_at(W * 0.5, H * 0.5), Some(0), "o centro e' do viewport unico");
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
    assert_eq!(nomes[3], None, "o quarto quadrante devia ser a vista LIVRE do artista");
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
    assert!(s.seam_at(W * 0.7, H * 0.5), "o arrasto da costura nao foi aceite");
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

/// ⭐⭐⭐ **DUAS TECLAS QUE PODEM CASAR O MESMO EVENTO: A SEGUNDA É MORTA.**
///
/// ⛔⛔ **Este gate nasceu de um defeito meu, escrito e apanhado no mesmo dia
/// (2026-09-08).** A divisão dos viewports foi ligada a `K::Backquote` **sem
/// modificador** — e a crase sozinha já tinha dono neste mesmo ficheiro (ela
/// abre o painel, e o roteiro da cena `=37` manda o artista usá-la). O braço do
/// painel corre antes e devolve `true`: a tecla nova **compilava, não dava
/// warning nenhum, e nunca corria**.
///
/// ⚠️ **Nenhuma sonda deste repo vê isto.** O censo de ids de painel mede
/// registo; os `seam_*` medem que o clique chega à ferramenta. *Um `match` de
/// teclado com dois braços que casam o mesmo evento é a espécie de controlo
/// morto que só a ORDEM de leitura revela.*
///
/// # ⛔⛔ E a PRIMEIRA redacção deste gate era fraca — a mutação SOBREVIVEU
///
/// Ela comparava as guardas como **texto**: `!ctrl && !shift` e `` (vazia,
/// sempre verdadeira) são strings diferentes, então tirar o `&& ctrl` do braço
/// novo — que é exactamente o defeito original — passava. *Uma régua que
/// pergunta «as guardas são iguais?» responde a outra pergunta que não a
/// «podem as duas ser verdadeiras ao mesmo tempo?».*
///
/// ⇒ cada guarda é **avaliada** sobre as quatro combinações de `(ctrl, shift)`,
/// e o gate exige que os conjuntos sejam **disjuntos**. Um símbolo que o censo
/// não saiba avaliar entra em pânico — *um censo que não entende o que lê tem
/// de dizê-lo, e não devolver «nada a acusar»*.
///
/// # ⛔ O PONTO CEGO, e ele é NOMEADO em vez de tapado
///
/// Uma guarda pode ser do **bloco que envolve** o braço, e não da linha dele: o
/// `K::KeyJ` aparece duas vezes com a mesma condição escrita, e não colidem
/// porque uma delas vive dentro de um `if shift { … }`. A primeira redacção
/// acusou-a — *um censo textual que não conhece o contexto acusa o vivo, e a
/// cura que ele manda aplicar é a errada*.
///
/// ⇒ ele compara **só arms no mesmo NÍVEL DE INDENTAÇÃO**, que é o proxy honesto
/// de *«no mesmo bloco»*. Ele deixa passar uma colisão entre blocos diferentes
/// com a mesma indentação, e isso está declarado aqui em vez de ser um silêncio.
#[test]
fn nenhuma_tecla_e_reivindicada_duas_vezes_com_a_mesma_guarda() {
    // ⚠️ `BTreeMap` e não `HashMap`: é a espinha do determinismo desta casa
    // (HR-5 + ADR-0022), e aqui ela dá de graça uma listagem ORDENADA quando o
    // censo imprime o que achou.
    use std::collections::BTreeMap;

    /// **O que este braço reivindica**: as teclas e a máscara de `(ctrl, shift)`.
    ///
    /// ⚠️ **Ele lê a condição INTEIRA, e não «a primeira tecla e o resto»** — um
    /// braço pode nomear várias teclas (`code == K::Comma || code == K::Period`),
    /// e as guardas de modificador são tokens soltos ligados por `&&`.
    /// ⛔ Qualquer palavra que não seja uma dessas entra em pânico: *um censo que
    /// não entende o que lê tem de dizê-lo, e não devolver «nada a acusar»*.
    fn reivindica(cond: &str) -> (Vec<String>, u8) {
        let mut teclas = Vec::new();
        let mut guardas: Vec<&str> = Vec::new();
        let limpo = cond
            .replace("&&", " ")
            .replace("||", " ")
            .replace(['(', ')'], " ");
        let mut it = limpo.split_whitespace().peekable();
        while let Some(t) = it.next() {
            match t {
                "code" => {
                    assert_eq!(it.next(), Some("=="), "forma inesperada depois de `code`");
                    let k = it.next().expect("falta a tecla depois de `==`");
                    teclas.push(
                        k.strip_prefix("K::")
                            .unwrap_or_else(|| panic!("`{k}` nao e' uma tecla `K::…`"))
                            .to_string(),
                    );
                }
                "ctrl" | "!ctrl" | "shift" | "!shift" => guardas.push(t),
                outro => panic!(
                    "o censo nao sabe avaliar `{outro}` na condicao `{cond}` -- ele nao pode \
                     devolver «nada a acusar» sobre o que nao entende"
                ),
            }
        }
        let mut mask = 0u8;
        for bit in 0..4u8 {
            let (ctrl, shift) = (bit & 1 != 0, bit & 2 != 0);
            let ok = guardas.iter().all(|t| match *t {
                "ctrl" => ctrl,
                "!ctrl" => !ctrl,
                "shift" => shift,
                _ => !shift,
            });
            if ok {
                mask |= 1 << bit;
            }
        }
        (teclas, mask)
    }

    let fonte = include_str!("sculpt3d_keys.rs");
    let mut vistos: BTreeMap<String, Vec<(String, u8)>> = BTreeMap::new();
    for linha in fonte.lines() {
        let l = linha.trim();
        // ⚠️ Só condições, nunca prosa: um comentário que cite `code == K::X`
        // não reivindica tecla nenhuma, e contar prosa é o defeito que todo
        // censo textual paga uma vez.
        if l.starts_with("//") || !l.starts_with("if code == K::") {
            continue;
        }
        let indent = linha.len() - linha.trim_start().len();
        let cond = l
            .trim_start_matches("if ")
            .trim_end()
            .trim_end_matches('{')
            .trim();
        let (teclas, mask) = reivindica(cond);
        for k in teclas {
            // ⚠️ **A indentação faz parte da chave** — ver o ponto cego no doc.
            vistos
                .entry(format!("{k}@{indent}"))
                .or_default()
                .push((cond.to_string(), mask));
        }
    }
    assert!(
        !vistos.is_empty(),
        "o censo nao achou braco de tecla nenhum -- ele ficou cego (o `sculpt3d_keys` mudou de \
         forma, e um censo cego le^-se como aprovado)"
    );
    let mut colisoes = Vec::new();
    for (tecla, arms) in &vistos {
        for i in 0..arms.len() {
            for j in (i + 1)..arms.len() {
                if arms[i].1 & arms[j].1 != 0 {
                    colisoes.push(format!(
                        "K::{tecla}: `{}` e `{}` aceitam o MESMO evento",
                        arms[i].0, arms[j].0
                    ));
                }
            }
        }
    }
    let mut multi: Vec<_> = vistos
        .iter()
        .filter(|(_, a)| a.len() > 1)
        .map(|(k, a)| (k.clone(), a.clone()))
        .collect();
    multi.sort();
    println!("teclas com mais de um braco: {multi:?}");
    assert!(
        colisoes.is_empty(),
        "{colisoes:?} -- o SEGUNDO braco nunca corre: o primeiro casa o mesmo evento e \
         devolve `true`. Uma tecla que compila e nunca corre nao da' warning nenhum."
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
    fn ocupacao(s: &crate::sculpt3d::Sculpt3dScene) -> f32 {
        let (w, h) = s.viewport();
        let (mut x0, mut x1, mut y0, mut y1) =
            (f32::INFINITY, f32::NEG_INFINITY, f32::INFINITY, f32::NEG_INFINITY);
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

    println!("ocupacao por vista: {ocupacoes:?} | vista ALTA com o aspecto da JANELA: {f_errada:.4}");
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
