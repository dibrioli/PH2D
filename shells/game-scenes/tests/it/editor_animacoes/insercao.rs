//! **A marca de inserção da tira** (dono 08/11): as células da folha entram ONDE a marca está, e ela anda para depois
//! delas. Filhos de [`super`] (a `Mesa`).

use super::*;

/// ⭐ A marca nasce no fim; arrastada para entre o 2.º e o 3.º quadro, o clique numa célula entra ALI, e o seguinte
/// entra logo depois. *Mutações: `spliced` que junta no fim ⇒ `[0..5, 20]`; não avançar a marca ⇒ o 21 entra antes
/// do 20; a marca que não segue o arrasto ⇒ fica no fim.*
#[test]
fn as_celulas_entram_onde_esta_a_marca_e_ela_anda_para_depois_delas() {
    let mut m = Mesa::new();
    m.paint();
    let l = m.doc().layout.clone();
    assert_eq!(m.doc().playhead_gap(), 6, "nasce no fim da idle (0..=5)");
    assert!(l.playhead.x > l.frames[5].0.x, "pintada depois do último");
    let entre = (l.frames[1].0.x + l.frames[1].0.w, centre(l.frames[1].0).1);
    m.pointer(PointerKind::Down, centre(l.playhead));
    m.drag_to(entre);
    m.pointer(PointerKind::Up, entre);
    assert_eq!(m.doc().insert_at, Some(2));
    let at = m.cell(20);
    m.click(at);
    assert_eq!(
        m.pending(),
        [
            AnimFieldEdit::Frames(0, fr(&[0, 1, 20, 2, 3, 4, 5])),
            AnimFieldEdit::SetStep(2)
        ]
    );
    assert_eq!(
        m.doc().frame_selected,
        Some(2),
        "o que entrou fica escolhido"
    );
    m.frame();
    m.paint();
    let at = m.cell(21);
    m.click(at);
    assert_eq!(
        m.pending(),
        [
            AnimFieldEdit::Frames(0, fr(&[0, 1, 20, 21, 2, 3, 4, 5])),
            AnimFieldEdit::SetStep(3)
        ],
        "o seguinte entra depois do anterior"
    );
}

/// Clicar num vão da tira põe a marca ali; apagar um quadro antes dela recua-a, e trocar de animação volta ao fim.
/// *Mutações: sem `playhead_follow_delete` ⇒ fica 3; sem o reset em `pick_row` ⇒ fica 2.*
#[test]
fn clicar_num_vao_poe_a_marca_e_ela_segue_o_apagar() {
    let mut m = Mesa::new();
    m.paint();
    let l = m.doc().layout.clone();
    let vao = (
        l.frames[2].0.x + l.frames[2].0.w + 1.0,
        centre(l.frames[2].0).1,
    ); // LITERAL-PX-OK: dentro do vão
    m.click(vao);
    assert_eq!(m.doc().insert_at, Some(3));
    m.paint();
    let x = m.doc().layout.frames[0].1;
    m.click(centre(x));
    assert_eq!(m.doc().insert_at, Some(2), "o quadro 1 saiu antes dela");
    m.frame();
    m.paint();
    let row = centre(m.doc().layout.list_rows[1]);
    m.click(row);
    assert_eq!(m.doc().insert_at, None, "outra animação: no fim");
}

/// O ◆ pertence ao desenho: células que entram antes dele empurram-no. *Mutação: sem `markers_follow_insert` ⇒ o ◆
/// fica no 3 e passa a marcar a célula nova.*
#[test]
fn o_marcador_depois_da_marca_anda_com_o_seu_quadro() {
    let mut m = Mesa::new();
    let r = m.doc().layout.frames[3].0;
    m.click(centre(r));
    m.frame();
    let r = m.doc().layout.mark_toggle;
    m.click(centre(r));
    m.key(Key::Enter, None);
    m.frame();
    m.paint();
    let l = m.doc().layout.clone();
    m.click((
        l.frames[0].0.x + l.frames[0].0.w + 1.0,
        centre(l.frames[0].0).1,
    )); // LITERAL-PX-OK: dentro do vão
    let at = m.cell(20);
    m.click(at);
    m.frame();
    let marcas = m
        .sim
        .world()
        .get::<ph2d_ecs::SpriteAnimMarkers>(Entity::from_bits(m.e))
        .map(|k| k.0.iter().map(|x| x.step).collect::<Vec<_>>());
    assert_eq!(marcas, Some(vec![4]));
}

/// Muitos quadros, tira rolada pela roda até ao fim.
fn tira_comprida(m: &mut Mesa) {
    m.doc()
        .pending
        .push(AnimFieldEdit::Frames(0, fr(&(0..16).collect::<Vec<_>>())));
    m.frame();
}

fn rola(m: &mut Mesa, dy: f32) {
    let e = m.e;
    let at = centre(m.doc().layout.strip);
    m.docs().wheel(e, at.0, at.1, dy, false);
    m.paint();
}

/// Bug do smoke 08/11: a roda parava antes do fim e o último quadro ficava cortado (ela media o passo com outro
/// intervalo que o da disposição). *Mutação: voltar ao `strip_pitch` ⇒ a borda do último passa da tira.*
#[test]
fn a_roda_chega_ao_fim_e_o_ultimo_quadro_cabe_inteiro() {
    let mut m = Mesa::new();
    tira_comprida(&mut m);
    rola(&mut m, -100_000.0);
    let l = m.doc().layout.clone();
    let fim = l.strip.x + l.strip.w;
    let ultimo = l.frames.last().unwrap().0;
    assert!(
        ultimo.x + ultimo.w <= fim + 0.5,
        "{} > {fim}",
        ultimo.x + ultimo.w
    );
    let borda = l.playhead.x + l.playhead.w;
    assert!(
        (borda - fim).abs() < 0.5,
        "a marca, no fim, encosta à borda: {borda} × {fim}"
    );
}

/// ⭐ O voo: a célula vira um cartão; o vão abre aos poucos com a mola e, assente, o quadro está no lugar inteiro.
/// *Mutações: `open` sempre 1 ⇒ o vão nasce aberto; `flying` sempre falso ⇒ idem; sem `launch` ⇒ não há voo.*
#[test]
fn a_celula_voa_ate_a_marca_e_o_vao_abre_aos_poucos() {
    let mut m = Mesa::new();
    m.paint();
    let l = m.doc().layout.clone();
    m.click((
        l.frames[1].0.x + l.frames[1].0.w + 1.0,
        centre(l.frames[1].0).1,
    )); // LITERAL-PX-OK: o vão
    let at = m.cell(20);
    m.click(at);
    m.paint();
    let cheio = m.doc().layout.frames[0].0.w;
    assert!(m.doc().flying(2), "a célula 20 voa para o lugar 2");
    assert_eq!(m.doc().layout.frames[2].0.w, 0.0, "o vão nasce fechado");
    let law = m.hero.motion.law(ph2d_editor_core::motion::Role::Travel);
    m.doc().tick(0.05, law);
    m.paint();
    let meio = m.doc().layout.frames[2].0.w;
    assert!(meio > 0.0 && meio < cheio, "a meio: {meio} de {cheio}");
    m.doc().tick(0.0, None);
    m.paint();
    assert_eq!(m.doc().layout.frames[2].0.w, cheio);
    m.doc().tick(0.0, None);
    assert!(
        m.doc().flight.is_none(),
        "assentou depois de uma disposição no lugar"
    );
}

/// ⭐ A marca fora da vista: a tira anda até ela enquanto o cartão desce. *Mutação: sem o `goal` ⇒ a rolagem fica 0
/// e a marca continua fora.*
#[test]
fn com_a_marca_fora_da_vista_a_tira_anda_ate_ela() {
    let mut m = Mesa::new();
    tira_comprida(&mut m);
    assert_eq!(m.doc().strip_scroll, 0.0);
    let at = m.cell(20);
    m.click(at);
    m.paint();
    m.doc().tick(0.0, None);
    m.paint();
    let l = m.doc().layout.clone();
    let fim = l.strip.x + l.strip.w;
    assert!(m.doc().strip_scroll > 0.0);
    assert!(l.playhead.x + l.playhead.w <= fim + 0.5 && l.playhead.x >= l.strip.x);
}

/// Achado da auditoria 08/11: a roda a meio do voo era sobrescrita (a disposição recalculava a rolagem do ponto de
/// partida do voo a cada quadro). A roda é do artista: o voo cai, como no clique. *Mutação: não largar o voo na roda
/// ⇒ a tira volta para onde o voo a levava.*
#[test]
fn a_roda_a_meio_do_voo_manda_na_tira() {
    let mut m = Mesa::new();
    tira_comprida(&mut m);
    let at = m.cell(20);
    m.click(at);
    m.paint();
    let law = m.hero.motion.law(ph2d_editor_core::motion::Role::Travel);
    m.doc().tick(0.05, law);
    m.paint();
    assert!(m.doc().flight.is_some() && m.doc().strip_scroll > 0.0);
    rola(&mut m, 100_000.0);
    m.paint();
    assert_eq!(m.doc().strip_scroll, 0.0, "a tira fica onde a roda a pôs");
    assert!(m.doc().flight.is_none(), "a roda larga o voo");
}

/// Outra edição muda a animação a meio do voo (o undo, apagar a folha): o voo cai, e o cartão não tapa um quadro que
/// já é outro. *Mutação: não largar o voo em `follow_flight` ⇒ ele continua e o lugar 2 fica fechado.*
#[test]
fn o_voo_cai_quando_outra_edicao_muda_a_animacao() {
    let mut m = Mesa::new();
    let at = m.cell(20);
    m.click(at);
    m.frame();
    assert!(
        m.doc().flight.as_ref().is_some_and(|f| f.seen),
        "a lista nova chegou"
    );
    m.doc()
        .pending
        .push(AnimFieldEdit::Frames(0, fr(&[9, 8, 7, 6, 5, 4, 3])));
    m.frame();
    assert!(m.doc().flight.is_none());
    assert!(m.doc().layout.frames.iter().all(|f| f.0.w > 0.0));
}

/// Clicar num quadro põe a marca logo à direita dele, e a célula seguinte entra ali. *Mutação: não mover a marca no
/// clique ⇒ fica no fim e a célula entra no fim.*
#[test]
fn clicar_num_quadro_poe_a_marca_a_direita_dele() {
    let mut m = Mesa::new();
    let r = m.doc().layout.frames[1].0;
    m.click(centre(r));
    assert_eq!(
        (m.doc().frame_selected, m.doc().insert_at),
        (Some(1), Some(2))
    );
    m.doc().pending.clear();
    let at = m.cell(20);
    m.click(at);
    assert_eq!(
        m.pending(),
        [
            AnimFieldEdit::Frames(0, fr(&[0, 1, 20, 2, 3, 4, 5])),
            AnimFieldEdit::SetStep(2)
        ]
    );
}

/// ⭐ Smoke do dono 09/11: o «+» punha a folha inteira na tira e a marca ficava presa no começo. A animação nova
/// nasce SEM quadros (a tira vazia, a marca no 0); cada célula clicada entra e a marca anda para depois dela.
/// *Mutações: o «+» com o intervalo da folha ⇒ a tira nasce cheia; a marca presa a outra conta ⇒ fica no 0.*
#[test]
fn na_animacao_nova_a_tira_nasce_vazia_e_a_marca_anda_com_os_cliques() {
    let mut m = Mesa::new();
    let add = centre(m.doc().layout.add);
    m.click(add);
    m.frame();
    m.paint();
    assert_eq!(m.doc().selected, 2);
    assert!(m.doc().layout.frames.is_empty(), "nenhum quadro sem pedir");
    assert_eq!(m.world_frames("anim_0"), Some(Vec::new()));
    assert_eq!(m.doc().playhead_gap(), 0, "começo");
    for c in [7, 3] {
        let at = m.cell(c);
        m.click(at);
        m.frame();
    }
    m.paint();
    assert_eq!(m.world_frames("anim_0"), Some(vec![7, 3]));
    assert_eq!(m.doc().playhead_gap(), 2, "depois das duas");
    let l = m.doc().layout.clone();
    // O 2.º ainda abre o vão (o voo não andou): a marca fica depois da ponta direita dele.
    let fim = l.frames[1].0.x + l.frames[1].0.w;
    assert!(
        l.playhead.x >= fim && fim > l.frames[0].0.x,
        "pintada depois do 2.º quadro"
    );
    // Tirar os dois deixa-a sem quadros, nunca a folha inteira.
    for _ in 0..2 {
        let x = centre(m.doc().layout.frames[0].1);
        m.click(x);
        m.frame();
        m.paint();
    }
    assert_eq!(m.world_frames("anim_0"), Some(Vec::new()));
    assert!(m.doc().layout.frames.is_empty());
}

/// ⭐ A ranhura «INSERT HERE» nunca cobre um quadro (handoff GEF 09/11, ABERTO 3: com a tira rolada cobria a ponta do
/// último), a toda largura, rolada ao começo e ao fim, com a marca no fim e no meio.
#[test]
fn a_ranhura_nunca_cobre_um_quadro() {
    for w in [1200.0, 700.0, 560.0, 420.0, 360.0] {
        // LITERAL-PX-OK: a janela do teste
        for marca in [None, Some(2)] {
            for rolada in [0.0, f32::MAX] {
                let mut m = Mesa::new();
                m.doc().insert_at = marca;
                let area = Rect::new(0.0, 0.0, w, AREA.h);
                m.paint_in(area);
                m.doc().strip_scroll = rolada;
                m.paint_in(area);
                let l = m.doc().layout.clone();
                let r = l.playhead;
                for (i, (f, _)) in l.frames.iter().enumerate() {
                    let pisa = r.x < f.x + f.w && f.x < r.x + r.w;
                    assert!(
                        !pisa,
                        "a {w} px, marca {marca:?}, rolada {rolada}: a ranhura {r:?} pisa o quadro {i} {f:?}"
                    );
                }
            }
        }
    }
}

/// ⭐ Smoke do dono 10/11: com o botão ainda premido na célula, o quadro NÃO aparece no lugar da tira antes de o cartão
/// descer — o vão fica fechado e a tira não o pinta até o voo. *Mutação: `arriving` sem o ramo do `Drag::Cells` ⇒ o
/// lugar 2 tem a largura cheia enquanto o botão está premido.*
#[test]
fn com_o_botao_premido_o_quadro_ainda_nao_esta_na_tira() {
    let mut m = Mesa::new();
    m.paint();
    let l = m.doc().layout.clone();
    m.click((
        l.frames[1].0.x + l.frames[1].0.w + 1.0,
        centre(l.frames[1].0).1,
    )); // LITERAL-PX-OK: o vão
    let at = m.cell(20);
    m.pointer(PointerKind::Down, at);
    m.paint();
    assert!(m.doc().flight.is_none(), "o voo só parte ao soltar");
    assert!(m.doc().flying(2), "a célula 20 está a caminho do lugar 2");
    assert_eq!(m.doc().layout.frames[2].0.w, 0.0, "e o lugar dela fechado");
    m.pointer(PointerKind::Up, at);
    m.paint();
    assert!(m.doc().flight.is_some());
    assert_eq!(
        m.doc().layout.frames[2].0.w,
        0.0,
        "o voo parte do vão fechado"
    );
}
