//! **OS GATES DO PINCEL DE DENSIDADE** — o verbo que não move um vértice.
//!
//! Irmão (`#[path]`) do [`super`], e o corte é de ASSUNTO: lá o arnês e a cura
//! da máscara, aqui tudo o que o pincel de densidade promete. Ele nasceu do
//! tecto de LOC, em 2026-09-14, depois de **três** reports do dono no mesmo dia
//! — *«não vejo efeito»* · *«apenas no grosso vi alguma coisa acontecendo»* ·
//! *«a densidade da malha deve ser independente do zoom»* — e de cada um ter
//! deixado um gate.
//!
//! ⚠️ O arnês (`cena_com`, `um_dab`, `vertices`, `CENTRE`) vive no pai e chega
//! por `use super::*`: duplicá-lo aqui seria a segunda resposta a *«o que é um
//! dab neste teste?»*.

use super::*;

/// ⭐⭐⭐ **A DENSIDADE LEVA A MALHA AO ALVO NOS DOIS SENTIDOS.**
///
/// ⛔⛔ **Este gate nasceu a afirmar UMA CÉLULA AO CONTRÁRIO, e quem o desmentiu
/// foi o dono, pelo produto:** *«por que não pode aumentar a densidade
/// também?»* (2026-09-14). A redacção anterior chamava-se *«a densidade afina a
/// malha e NUNCA a engrossa»* e escrevia, com o comentário ao lado, que *«um
/// pincel que também subdividisse é outro produto»*.
///
/// **Não é.** A espec §3.2 diz que o pincel **ACRESCENTA a bandeira de colapso**
/// ao modo do passe — ele não **RETIRA** a de partir —, e mede as duas células
/// **na mesma malha grossa**: `81 → 81` com o ajuste da cena em «só colapsar» e
/// **`81 → 101`** com «partir + colapsar».
///
/// ⚠️ **E o ajuste que existiu entre os dois reports do mesmo dia foi RETIRADO
/// pelo dono:** *«não precisamos do modo Thin Only. Deve ser sempre Equalise.»*
/// ⇒ não há célula a escolher: ela faz as duas metades, sempre.
///
/// ⚠️ **A recusa medida da espec continua de pé e é OUTRA pergunta:** o pincel
/// não pode **FORÇAR** o partir onde o ajuste da cena o desliga. Nós não temos
/// esse ajuste, logo não há nada que ele possa forçar.
///
/// **Medido** (um dab, raio `160 px`):
///
/// | arranjo | verbo | vértices |
/// |---|---|---|
/// | malha fina (`48×72`), alvo grosso | `Density` | **`3 386 → 3 352`** — afina |
/// | malha grossa (`8×12`), alvo fino | `Density` | **cresce** — adensa |
/// | a MESMA, alvo fino | `Draw` | **`86 → 359`** — o controlo |
///
/// ⚠️ **A colheita da 1.ª linha é modesta (`34` vértices) e isso é a NOSSA lei,
/// não um defeito:** o nosso colapso tem quatro recusas
/// (`ph2d-mesh/src/collapse.rs`), entre elas *«algum dos quatro vértices está na
/// beira»* — mais dura que a do alvo, que em vez de recusar **escolhe o
/// sobrevivente** (espec §3.8, decisão de produto com duas frases). ⭐ A
/// magnitude que o dono vê está no gate irmão
/// [`a_densidade_tira_uma_fraccao_visivel_e_nao_um_punhado`].
#[test]
#[ignore]
fn a_densidade_leva_a_malha_ao_alvo_nos_dois_sentidos() {
    let gpu = gpu_or_skip!();

    // (a) MALHA FINA, alvo GROSSO — há aresta curta de sobra: ela AFINA.
    let fina = || ph2d_mesh::shapes::uv_sphere(48, 72, 1.0);
    let mut s = cena_com(&gpu.device, Verb::Density, fina(), 0.15, 160.0);
    let antes = vertices(&s);
    um_dab(&mut s);
    let depois = vertices(&s);
    assert!(
        depois < antes,
        "a densidade não afinou nada ({antes} -> {depois}) — todo o efeito dela \
         é sobre o passe de topologia"
    );

    // ⭐⭐ (b) **MALHA GROSSA, alvo FINO: ela ADENSA.** É a célula `81 → 101` da
    // espec, e é o report do dono.
    let grossa = || ph2d_mesh::shapes::uv_sphere(8, 12, 1.0);
    let mut s = cena_com(&gpu.device, Verb::Density, grossa(), 1.0, 160.0);
    let antes = vertices(&s);
    um_dab(&mut s);
    let adensada = vertices(&s);
    assert!(
        adensada > antes,
        "a densidade não adensou nada ({antes} -> {adensada}) — é exactamente o \
         report do dono: *«por que não pode aumentar a densidade também?»*"
    );

    // ⭐ **O controlo positivo:** o mesmo arranjo com um verbo que liga as duas
    // metades cresce também — sem ele, a metade (b) podia estar a medir um
    // arranjo em que qualquer coisa cresce.
    let mut s = cena_com(&gpu.device, Verb::Draw, grossa(), 1.0, 160.0);
    let antes = vertices(&s);
    um_dab(&mut s);
    let depois = vertices(&s);
    assert!(
        depois > antes,
        "o desenho não subdividiu a malha grossa ({antes} -> {depois}) — sem \
         isto a metade (b) não afirma nada"
    );
}

/// ⭐⭐ **ELA NÃO TEM LEI POR-VÉRTICE — e a régua disso é a JANELA DO TRAÇO.**
///
/// ⚠️⚠️ **Este gate chamava-se *«com o passe DESARMADO ela é inteiramente
/// inerte»*, e o SUJEITO dele mudou em 14/09** por ordem do dono: o pincel corre
/// sem o interruptor, logo *«desarmado»* deixou de ser um estado em que ele não
/// faz nada. ⛔ E ele continuava a **passar** — sobre outra coisa: a esfera de
/// fixtura é de quads, o passe passou a triangulá-la, e triangular não move um
/// vértice. *Um gate que passa pela razão errada é pior que um vermelho.*
///
/// ⇒ o que fica são as **duas** metades que a espec §3.1 de facto afirma:
///
/// 1. **A janela do traço é VAZIA.** O `dab_core` fotografa (`capture`) todo
///    vértice ao alcance **antes** de decidir o que fazer com ele, logo um
///    `touched` vazio prova que a cadeia de peso **nunca correu** — ainda que a
///    topologia mude à volta.
/// 2. **Onde o passe não pode agir, o `0` é EXACTO.** A recusa que sobra é
///    estrutural (a pilha de multiresolução montada), e ali a peça sai
///    byte-idêntica.
///
/// ⚠️ **A 1.ª metade é a que DISCRIMINA, e foi uma mutação que o provou:** apagar
/// o desvio do `stroke_symmetry` deixava um `assert` de posições **verde**,
/// porque o `stroke_target` tem um braço `Verb::Density => live` — a resposta
/// defensiva para *«e se alguém chegar aqui mesmo assim?»*. *Duas respostas à
/// mesma pergunta, e a de baixo mascarava a de cima.*
#[test]
#[ignore]
fn a_densidade_nunca_tem_lei_por_vertice() {
    let gpu = gpu_or_skip!();

    // (1) O caminho NORMAL: o passe corre, a topologia muda, e a janela do
    // traço continua vazia.
    let mut s = cena_com(
        &gpu.device,
        Verb::Density,
        ph2d_mesh::shapes::uv_sphere(48, 72, 1.0),
        0.15,
        160.0,
    );
    let antes = vertices(&s);
    um_dab(&mut s);
    assert!(
        vertices(&s) != antes,
        "o arranjo não mexeu na topologia ({antes} -> {}) — sem isso a metade \
         abaixo ficaria verde sobre um passe que nunca disparou",
        vertices(&s)
    );
    assert!(
        s.stroke.touched().is_empty(),
        "a densidade fotografou {} vértices — ela desvia ANTES do laço \
         por-vértice, e um `touched` não-vazio quer dizer que a cadeia de peso \
         correu à mesma",
        s.stroke.touched().len()
    );

    // (2) A recusa ESTRUTURAL que sobra: com a pilha montada, `0` exacto.
    let mut s = Sculpt3dScene::new(&gpu.device, ph2d_mesh::shapes::uv_sphere(48, 72, 1.0), 1.0);
    s.note_canvas(ph2d_editor_core::zones::Rect::new(0.0, 0.0, 900.0, 700.0));
    s.brush.verb = Verb::Density;
    s.subdivide();
    assert!(
        s.level_count() > 1,
        "a fixtura não montou a pilha — a recusa que este ramo mede não existe"
    );
    let antes = s.objects[s.active].stack.mesh().positions().to_vec();
    let faces = s.mesh().face_count();
    um_dab(&mut s);
    let depois = s.objects[s.active].stack.mesh().positions().to_vec();
    assert_eq!(antes.len(), depois.len(), "a contagem não pode mudar");
    assert_eq!(faces, s.mesh().face_count(), "as faces não podem mudar");
    let pior = antes
        .iter()
        .zip(&depois)
        .map(|(a, b)| {
            (a[0] - b[0])
                .abs()
                .max((a[1] - b[1]).abs())
                .max((a[2] - b[2]).abs())
        })
        .fold(0.0f32, f32::max);
    assert_eq!(
        pior, 0.0,
        "a densidade moveu barro com a pilha montada (pior delta {pior:e}) — \
         ela não tem lei por-vértice nenhuma"
    );
}

/// ⭐⭐⭐ **ELA CORRE SEM O INTERRUPTOR — ordem do dono, e as TRÊS metades.**
///
/// *«Independente se Dynamic topology está ligado ou não, Density faz o seu
/// trabalho. Dynamic topology é para os outros pincéis.»* (14/09)
///
/// ⚠️ **DIVERGÊNCIA DECLARADA da referência** (espec §3.2, 1.ª linha: o modo de
/// detalhe em *Manual* desarma o passe inteiro, este pincel incluído).
///
/// ⛔⛔ **E a 2.ª metade é o que quase ficou por fazer:** os dois motores recusam
/// quads por geometria, e quem os triangulava era o interruptor. Sem isso o
/// pincel seria um **no-op silencioso** em toda peça que ainda é de quads — uma
/// primitiva acabada de nascer, ou a saída do botão de retopologia.
#[test]
#[ignore]
fn a_densidade_corre_com_o_interruptor_desligado() {
    let gpu = gpu_or_skip!();

    // (1) Com o modo DESLIGADO, sobre uma peça de QUADS: ela trabalha.
    let mut s = Sculpt3dScene::new(&gpu.device, ph2d_mesh::shapes::uv_sphere(24, 36, 1.0), 1.0);
    s.note_canvas(ph2d_editor_core::zones::Rect::new(0.0, 0.0, 900.0, 700.0));
    s.brush.verb = Verb::Density;
    s.radius_px = 160.0;
    assert!(
        !s.dyntopo.armed,
        "a fixtura tem de partir com o modo DESLIGADO — é isso que ela mede"
    );
    let quads = s
        .mesh()
        .faces()
        .iter()
        .filter(|f| f.verts().len() > 3)
        .count();
    assert!(
        quads > 0,
        "a fixtura não tem quads — a metade da triangulação mediria o nada"
    );
    let antes = vertices(&s);
    um_dab(&mut s);
    let depois = vertices(&s);
    assert!(
        depois != antes,
        "a densidade não fez nada com o modo desligado ({antes} -> {depois}) — \
         é a ordem do dono: *«independente se Dynamic topology está ligado ou \
         não, Density faz o seu trabalho»*"
    );
    assert!(
        s.mesh().faces().iter().all(|f| f.verts().len() == 3),
        "sobraram quads — os dois motores recusam-nos, e quem os triangulava \
         era o interruptor"
    );

    // ⭐ (2) **O CONTROLO: outro verbo, no MESMO arranjo, não mexe na
    // topologia.** Sem ele este gate ficaria verde sobre uma porta que ignora o
    // interruptor para TODA a gente — que é o oposto da ordem.
    let mut s = Sculpt3dScene::new(&gpu.device, ph2d_mesh::shapes::uv_sphere(24, 36, 1.0), 1.0);
    s.note_canvas(ph2d_editor_core::zones::Rect::new(0.0, 0.0, 900.0, 700.0));
    s.brush.verb = Verb::Draw;
    s.radius_px = 160.0;
    let antes = vertices(&s);
    um_dab(&mut s);
    assert_eq!(
        vertices(&s),
        antes,
        "o desenho mexeu na topologia com o modo desligado — o interruptor \
         deixou de valer para quem ele governa"
    );

    // ⭐⭐⭐ (3) **O CASO EM QUE ELA *SÓ* TRITURA** — e foi uma mutação
    // SOBREVIVENTE que o encomendou.
    //
    // ⛔⛔ A 1.ª redacção deste gate media o desfazer sobre um dab que **também**
    // mudava a contagem de vértices, logo reverter a metade `face_count` do
    // registo deixava-o **verde**: o `vert_count` já era diferente. *Um gate que
    // não contém o caso não afirma nada sobre ele.*
    //
    // ⇒ o arranjo sai de uma MEDIÇÃO desta linha: com a câmera perto, o pincel
    // de `160 px` vale `0,12` de mundo contra arestas de `~0,3` — ele fica
    // **menor que um triângulo** e não há aresta nenhuma na esfera dele. O dab
    // não parte nem funde, e a ÚNICA coisa que acontece é a triangulação do
    // pen-down. Ali `vert_count` fica **igual** e só as faces se movem.
    //
    // ⚠️ **Uma banda uniforme não serve para isto, e eu tentei:** numa esfera
    // UV as arestas encolhem para zero nos pólos, logo não existe posição do
    // slider em que nada parta E nada funda — a 1.ª redacção escolheu
    // `detail = 0,21` por aritmética sobre a aresta *média* e mediu `+21`
    // vértices. *O caso não era um número do slider; era a GEOMETRIA do
    // alcance.*
    let mut s = Sculpt3dScene::new(&gpu.device, ph2d_mesh::shapes::uv_sphere(10, 14, 1.0), 1.0);
    s.note_canvas(ph2d_editor_core::zones::Rect::new(0.0, 0.0, 900.0, 700.0));
    s.brush.verb = Verb::Density;
    s.radius_px = 160.0;
    s.camera.distance = 1.5;
    let (v0, f0) = (vertices(&s), s.mesh().face_count());
    um_dab(&mut s);
    assert_eq!(
        vertices(&s),
        v0,
        "a fixtura do «só tritura» mexeu nos VÉRTICES — ela deixou de conter o \
         caso que a mutação exige"
    );
    assert!(
        s.mesh().face_count() != f0,
        "a fixtura não triturou nada ({f0} faces) — o arranjo não contém o caso"
    );
    assert!(
        s.undo_stroke(),
        "um traço que só triturou não deixou nada para desfazer — o registo \
         pergunta só pelos VÉRTICES, e triangular não cria nenhum"
    );
    assert_eq!(
        (vertices(&s), s.mesh().face_count()),
        (v0, f0),
        "o Ctrl+Z não devolveu os quads"
    );

    // ⭐⭐ (4) **E o gesto inteiro desfaz num passo** — incluindo a
    // triangulação, que muda as FACES sem criar um vértice. *Era aqui que o
    // `Ctrl+Z` não devolvia os quads: o registo perguntava só pelos vértices.*
    let mut s = Sculpt3dScene::new(&gpu.device, ph2d_mesh::shapes::uv_sphere(24, 36, 1.0), 1.0);
    s.note_canvas(ph2d_editor_core::zones::Rect::new(0.0, 0.0, 900.0, 700.0));
    s.brush.verb = Verb::Density;
    s.radius_px = 160.0;
    let (v0, f0) = (vertices(&s), s.mesh().face_count());
    um_dab(&mut s);
    assert!(s.undo_stroke(), "o traço não deixou nada para desfazer");
    assert_eq!(
        (vertices(&s), s.mesh().face_count()),
        (v0, f0),
        "o Ctrl+Z não devolveu a malha de antes (vértices E faces)"
    );
}

/// **SONDA:** o percurso do dono na `=14` — adensar com o `Draw` e depois afinar
/// com o `Density`, dab a dab.
#[test]
#[ignore]
fn diag_o_percurso_do_dono() {
    let gpu = gpu_or_skip!();
    // A malha da própria cena `=14`.
    let malha = ph2d_mesh::shapes::uv_sphere(10, 14, 1.0);
    let mut s = Sculpt3dScene::new(&gpu.device, malha, 1.0);
    s.note_canvas(ph2d_editor_core::zones::Rect::new(0.0, 0.0, 900.0, 700.0));
    let (ligado, tri) = s.toggle_dyntopo();
    assert!(ligado);
    println!("cena =14: {} verts (triangulou {tri} faces)", vertices(&s));

    s.brush.verb = Verb::Draw;
    s.dyntopo.detail = 1.0;
    for i in 0..8 {
        um_dab(&mut s);
        println!("  Draw fino dab {i}: {} verts", vertices(&s));
    }
    s.brush.verb = Verb::Density;
    for detalhe in [0.15f32, 0.5] {
        s.dyntopo.detail = detalhe;
        for i in 0..10 {
            um_dab(&mut s);
            println!(
                "  Density detalhe {detalhe} dab {i}: {} verts",
                vertices(&s)
            );
        }
    }

    // ⭐⭐ **O GESTO NOVO — adensar com o próprio pincel de densidade**, que é a
    // metade que o report de 14/09 pediu (*«por que não pode aumentar a
    // densidade também?»*). Parte da malha CRUA da cena, sem `Draw` nenhum.
    let mut s = Sculpt3dScene::new(&gpu.device, ph2d_mesh::shapes::uv_sphere(10, 14, 1.0), 1.0);
    s.note_canvas(ph2d_editor_core::zones::Rect::new(0.0, 0.0, 900.0, 700.0));
    assert!(s.toggle_dyntopo().0);
    s.brush.verb = Verb::Density;
    s.dyntopo.detail = 1.0;
    println!("adensar com Density: {} verts (cru)", vertices(&s));
    for i in 0..6 {
        um_dab(&mut s);
        println!("  Density Equalise fino dab {i}: {} verts", vertices(&s));
    }
}

/// ⭐⭐⭐ **ELA TIRA UMA FRACÇÃO VISÍVEL, e não um punhado de vértices.**
///
/// ⛔⛔ **Este gate nasceu de um report do dono — *«não vejo efeito com
/// density»* — e o que falhou foi a RÉGUA, não o pincel.** O gate irmão
/// afirmava `depois < antes`: uma **direcção**. Com ele verde, a colheita
/// medida era de `34` vértices em `3 386` — **1 %**, que é invisível a olho nu.
/// *Uma régua que só vê o SINAL não vê a MAGNITUDE*, e é a mesma família do
/// `edge_max` cego ao quad fino e da contagem cega a QUAIS vértices se movem.
///
/// ⇒ a barra é uma **fracção**, e ela sai do percurso do próprio dono medido
/// dab a dab: com o detalhe em `grosso`, dez toques levam a peça de `822` para
/// `399` vértices (**−51 %**). A barra fica em `−25 %`, com margem de `2×`.
#[test]
#[ignore]
fn a_densidade_tira_uma_fraccao_visivel_e_nao_um_punhado() {
    let gpu = gpu_or_skip!();
    // O percurso do dono: a malha da `=14`, adensada com o `Draw` no fino.
    let mut s = Sculpt3dScene::new(&gpu.device, ph2d_mesh::shapes::uv_sphere(10, 14, 1.0), 1.0);
    s.note_canvas(ph2d_editor_core::zones::Rect::new(0.0, 0.0, 900.0, 700.0));
    assert!(s.toggle_dyntopo().0);
    s.brush.verb = Verb::Draw;
    s.dyntopo.detail = 1.0;
    for _ in 0..8 {
        um_dab(&mut s);
    }
    let adensada = vertices(&s);
    assert!(
        adensada > 700,
        "o arranjo não adensou o suficiente para a pergunta ter sentido: {adensada}"
    );

    // ⚠️ **E o detalhe em GROSSO é parte do gesto, não do arnês:** o alvo de
    // aresta é `raio × f(detalhe)`, logo num detalhe fino não há o que colapsar.
    // É exactamente isto que a queixa do passe diz ao artista.
    s.brush.verb = Verb::Density;
    s.dyntopo.detail = 0.15;
    for _ in 0..10 {
        um_dab(&mut s);
    }
    let afinada = vertices(&s);
    let fraccao = 1.0 - (afinada as f32) / (adensada as f32);
    assert!(
        fraccao >= 0.25,
        "a densidade tirou só {:.1} % ({adensada} -> {afinada}) — uma colheita \
         que o dono não vê é indistinguível de um pincel partido",
        fraccao * 100.0
    );
}

/// ⭐⭐⭐ **QUANDO ELA NÃO FAZ NADA, ELA DIZ PORQUÊ — e as razões passaram a DUAS.**
///
/// ⛔ Um verbo cujo efeito inteiro é sobre a topologia **parece partido** sempre
/// que o passe não corre, e o artista vê todas as razões **iguais**: nada
/// acontece. *Foi assim que o report de 14/09 nasceu.*
///
/// ⚠️⚠️ **Eram TRÊS e são DUAS desde a ordem do dono** (*«independente se
/// Dynamic topology está ligado ou não…»*): a razão *«o modo está desligado»*
/// deixou de existir para este pincel — ele corre à mesma. ⭐ *Curar um defeito
/// pode APAGAR uma queixa, e um censo que continue a contar três fica a mentir
/// para o lado seguro.*
///
/// ⚠️ **O controlo negativo está dentro:** um `Draw` no MESMO caminho tem de
/// ficar **calado** — senão o log enche-se e deixa de ser lido.
#[test]
#[ignore]
fn quando_a_densidade_nao_faz_nada_ela_diz_porque() {
    let gpu = gpu_or_skip!();

    // (a) A PILHA DE MULTIRESOLUÇÃO montada — a recusa estrutural que sobra.
    let mut s = Sculpt3dScene::new(&gpu.device, ph2d_mesh::shapes::uv_sphere(24, 36, 1.0), 1.0);
    s.note_canvas(ph2d_editor_core::zones::Rect::new(0.0, 0.0, 900.0, 700.0));
    s.brush.verb = Verb::Density;
    s.radius_px = 160.0;
    s.subdivide();
    assert!(s.level_count() > 1, "a fixtura não montou a pilha");
    um_dab(&mut s);
    assert!(
        s.dyn_queixa_dita,
        "a densidade ficou muda com a pilha montada — o artista vê um pincel \
         partido e não tem como saber que o `J` a reverte"
    );

    // (b) A MALHA JÁ NO PONTO que o slider pede — a razão mais difícil de
    // adivinhar de fora. O arranjo é construído a MARTELAR até assentar.
    let mut s = cena_com(
        &gpu.device,
        Verb::Density,
        ph2d_mesh::shapes::uv_sphere(24, 36, 1.0),
        0.5,
        160.0,
    );
    for _ in 0..40 {
        um_dab(&mut s);
    }
    let assentada = vertices(&s);
    s.dyn_queixa_dita = false;
    um_dab(&mut s);
    assert_eq!(
        vertices(&s),
        assentada,
        "a malha ainda não assentou ({assentada} -> {}) — o arranjo não contém \
         o fenómeno que esta metade mede",
        vertices(&s)
    );
    assert!(
        s.dyn_queixa_dita,
        "a densidade ficou muda sobre uma malha que já está no ponto pedido"
    );

    // ⭐ **O CONTROLO NEGATIVO: o desenho no MESMO caminho fica CALADO.**
    //
    // ⚠️⚠️ **A primeira redacção deste controlo era VÁCUO, e uma mutação
    // provou-o:** ela usava um arranjo onde o `Draw` **colapsa**, logo ele nunca
    // chegava ao caminho da queixa, e tornar a queixa universal não o reprovava.
    // *Um controlo negativo tem de percorrer o MESMO caminho que a metade
    // positiva*, senão está a afirmar sobre código que não corre.
    s.brush.verb = Verb::Draw;
    s.dyn_queixa_dita = false;
    um_dab(&mut s);
    assert!(
        !s.dyn_queixa_dita,
        "o desenho falou — a queixa é só de quem não tem outra forma de se \
         mostrar, e um log por dab é um log que ninguém lê"
    );
}

/// **DE ONDE SAI O ALVO** — ver [`ancora`].
///
/// ⚠️ **O corte é de RESPONSABILIDADE e foi forçado pelo tecto de 700 LOC:**
/// aqui *o que o pincel FAZ*, lá *de onde vem o número que ele persegue* — que
/// é o assunto das duas invariâncias (o zoom e o tamanho da peça) e da ordem do
/// dono de 14/09. ⛔ Curado por corte, nunca por uma entrada no
/// `FILE_OVERAGE_OK`.
#[path = "densidade_ancora_tests.rs"]
mod ancora;
