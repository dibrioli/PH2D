//! ⭐ **A PORTA DE SAÍDA da modelagem 3D** — o campo vira malha e vai para um arquivo.
//!
//! # ⚠️ Exportar é a primeira vez que este módulo PERDE informação de propósito
//!
//! Um campo tem resolução **infinita**: o filete é uma fórmula, e ampliar não revela serrilha. Uma
//! malha não — ela é uma escolha de quantos triângulos. Todo o resto do módulo foi construído para
//! **não** decidir isso cedo ([ADR-0161 §2]), e aqui a decisão é inevitável.
//!
//! Então ela é **explícita e medida**: os três níveis são resoluções de grade, e cada uma tem o
//! número ao lado ([`ExportLevel`]). O toast diz quantos quads e quantos triângulos saíram de facto
//! — não uma promessa, o resultado.
//!
//! # ⚠️ A malha é uma GRADE DE QUADS, e a tela continua a não passar por aqui
//!
//! O extrator é o da casa ([`ph2d_field_eval::extract`]), e ele fecha o que o spike deixou aberto:
//! a aresta viva sai **exata** (a W0 media `0/49` faixas capturadas, hoje são `116/116` com desvio
//! `0,00` de célula) e nenhuma face sai dobrada. Ainda assim a tela é o campo **traçado**: uma malha
//! é uma resolução escolhida, e o campo não tem nenhuma.
//!
//! # ⚠️ Nada de segunda tabela de formatos
//!
//! O diálogo, os três formatos e o aviso do que se perde vêm todos da
//! [`ph2d_mesh::MeshFormat`] e do `sculpt3d::lost_by`, que a escultura já tinha.
//! Uma cópia local diria *"cor preservada"* sobre um STL no dia em que alguém trocasse o escritor.
//!
//! [ADR-0161 §2]: ../../../docs/architecture/decisions/0161-3d-modeling-is-an-implicit-field-tree-and-what-the-artist-sees-is-the-traced-field.md

use ph2d_mesh::{ExportPiece, MeshFormat};

/// ⭐ **Quanta malha sai** — e cada nível traz o número que a medição deu.
///
/// ⚠️ **Não é um modo que se guarda**: os três são **ações**, e o rótulo diz o que sai. Um seletor
/// de qualidade guardado obrigaria o artista a lembrar em que ficou — e a resposta certa está na
/// peça que ele tem à frente, não numa preferência de ontem.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ExportLevel {
    Draft,
    Fine,
    Max,
}

impl ExportLevel {
    /// ⭐ **A fonte da contagem** — o painel deriva os botões daqui, como faz com `Mode::ALL`.
    pub(crate) const ALL: [ExportLevel; 3] =
        [ExportLevel::Draft, ExportLevel::Fine, ExportLevel::Max];

    /// A resolução da grade deste nível — **MEDIDA**, não escolhida.
    ///
    /// `measure_export_resolution` sobre a cena 1 (três cilindros com filete) e
    /// `measure_export_mesh_quality` sobre as quatro cenas, em release:
    ///
    /// | prof | quads | triângulos | ms | faces dobradas (pior cena) |
    /// |---|---|---|---|---|
    /// | 4 | 438 | 876 | 0,8 | — |
    /// | 5 | 1 830 | 3 660 | 1,9 | **68** ⛔ |
    /// | **6** | **7 446** | **14 892** | **7,9** | **0** ← Draft |
    /// | **7** | **29 502** | **59 004** | **35,5** | **0** ← Fine |
    /// | 8 | 117 198 | 234 396 | 214,6 | 0 |
    /// | **9** | **467 334** | **934 668** | **1 463** | **0** ← Max |
    ///
    /// ⚠️ **O Draft subiu de 5 para 6 por MEDIÇÃO, e o motivo não é o relógio.** Na prof. 5 a célula
    /// mede 0,0625 e a parede do vaso da cena 5 mede 0,06: a grade não consegue representar uma
    /// parede mais fina que a própria célula, e o resultado tem faces dobradas **e** 40 arestas
    /// não-manifold. A prof. 6 zera as duas coisas em todas as cenas e custa 6 ms a mais — *o degrau
    /// mais barato deste módulo*.
    ///
    /// ⭐ **A grade é uniforme, então a contagem quadruplica a cada degrau e o relógio segue.** Isto
    /// é diferente do que a tabela anterior (extrator da `fidget`) dizia: lá os triângulos saturavam
    /// porque o octree **colapsava** células, e `depth` era um teto e não uma resolução. Aqui os três
    /// níveis são o que dizem ser.
    ///
    /// Daí os três: **6** é instantâneo e já é uma malha sã, **7** é o dobro do detalhe ainda
    /// instantâneo, e **9** é onde se aceita esperar uma batida e meia por tudo o que a peça tem.
    ///
    /// ⚠️ **Esta tabela é só a EXTRAÇÃO.** Desde 2026-08-25 a exportação corre também a cadeia de
    /// quads, que custa ~4,6 s e **não** depende deste nível — ver [`meshes_for`].
    /// ⛔ A prof. 10 quadruplicaria para ~1,9 M quads e ~6 s — e a peça já não muda de forma.
    pub(crate) fn depth(self) -> u8 {
        match self {
            ExportLevel::Draft => 6,
            ExportLevel::Fine => 7,
            ExportLevel::Max => 9,
        }
    }

    pub(crate) fn key(self) -> &'static str {
        match self {
            ExportLevel::Draft => "panel.model3d.export.draft",
            ExportLevel::Fine => "panel.model3d.export.fine",
            ExportLevel::Max => "panel.model3d.export.max",
        }
    }
}

/// **Escreve a peça num arquivo escolhido pelo artista.**
///
/// ⚠️ **Sem peça não há o que exportar, e o toast diz** — o silêncio seria indistinguível de um
/// diálogo que falhou.
///
/// ⭐ **O TAMANHO DA PEÇA que saiu** — a caixa da malha escrita, por eixo.
///
/// # ⚠️ Por que NÃO é o bordo da grade
///
/// Há dois números disponíveis e eles não são o mesmo. O bordo
/// ([`ph2d_field_eval::bounds::bounding_ball`], W33) é o cubo que envolve a **esfera** que contém a
/// peça, mais 5 % de folga: ele é **andaime**, conservador por construção, e **cúbico** — numa peça
/// fina o eixo curto sai mais de uma ordem de grandeza maior do que a peça. Dizê-lo seria responder
/// *"que tamanho tem a caixa em que eu desenhei"* a quem perguntou *"que tamanho tem a peça"*.
///
/// O que se diz é a caixa da **malha que de facto foi escrita no arquivo** — o mesmo número que o
/// outro programa vai mostrar. Sonda: `measure_the_grid_box_against_the_real_piece`.
///
/// ⚠️ Uma malha **vazia** devolve zeros: o [`ph2d_mesh::Aabb::EMPTY`] é invertido de propósito, e
/// subtrair as pontas dele daria negativos — que num toast leem como um defeito da peça.
fn piece_size(mesh: &ph2d_mesh::Mesh) -> [f32; 3] {
    if mesh.positions().is_empty() {
        return [0.0; 3];
    }
    let b = mesh.bounds();
    [
        b.max[0] - b.min[0],
        b.max[1] - b.min[1],
        b.max[2] - b.min[2],
    ]
}

/// ⭐⭐ **ONDE a peça está — e ela só o diz quando a pergunta EXISTE.**
///
/// # ⚠️ O buraco
///
/// A malha sai em **mundo** (a pose já está cozida no campo, ver [`cook`]), então uma peça modelada
/// longe da origem aterra longe da origem no outro programa — **fora do enquadramento inicial**. O
/// artista abre o arquivo, vê o vazio, e conclui que a exportação falhou. O toast dizia o **tamanho**
/// desde a W36 e nunca disse o **sítio**.
///
/// # ⭐ E o limiar é DERIVADO, não escolhido
///
/// Dizer `(0,00, 0,00, 0,00)` em toda exportação centrada é ruído — a mesma lei do sufixo da
/// retopologia, três linhas acima: *silencioso quando não muda nada*. ⇒ ela fala quando **a origem
/// está FORA da caixa da peça**, que é exactamente a condição em que *"onde é que isto está?"* deixa
/// de ter resposta óbvia: com a origem dentro, o outro programa enquadra a peça sozinho.
///
/// ⚠️ **Nada de épsilon nem de número mágico:** o recurso é a **própria caixa da peça**. Uma peça de
/// `0,9` centrada em `0,05` está na origem; uma em `5,0` não está — e a régua é a mesma nas duas.
///
/// ⚠️ Uma malha **vazia** cala-se: o [`ph2d_mesh::Aabb::EMPTY`] é invertido de propósito, e um
/// centro tirado dele seria um sítio inventado.
fn piece_origin_note(mesh: &ph2d_mesh::Mesh) -> String {
    if mesh.positions().is_empty() {
        return String::new();
    }
    let b = mesh.bounds();
    let inside = (0..3).all(|k| b.min[k] <= 0.0 && 0.0 <= b.max[k]);
    if inside {
        return String::new();
    }
    let c = [
        f32::midpoint(b.min[0], b.max[0]),
        f32::midpoint(b.min[1], b.max[1]),
        f32::midpoint(b.min[2], b.max[2]),
    ];
    format!(" · at ({:.2}, {:.2}, {:.2})", c[0], c[1], c[2])
}

#[cfg(test)]
#[path = "field3d_export_tests.rs"]
mod tests;

/// ⭐⭐⭐ **A CADEIA DE QUADS ALINHADOS** (W61b) — a exportação oferece o que há de melhor, e
/// só o adopta quando ele é de facto melhor.
///
/// ⛔ **Medido** (`ph2d_field_eval::tests::the_scorecard_of_the_extracted_mesh` e a irmã): a
/// extração por *Dual Contouring* já entrega topologia **perfeita** (`0` não-manifold, `0`
/// bordo) e geometria quase exacta (`|f|` médio de `~0,005` célula), mas a **forma da face**
/// sai a `25–27°` de enviesamento contra os `4,8–7,1°` do oráculo de produção. ⭐ A cadeia
/// leva a esfera a **`1,08` de aspecto e `6,4°`** — a classe do oráculo.
///
/// ⚠️ **E ela NÃO é «sempre»:** numa peça de faces planas a grade dual já é a resposta certa
/// (o quad pousa na face e sai a `0°`), e a cadeia piora. Quem decide é o veto de
/// [`ph2d_quadchain::quads_or_keep_from`], que nunca devolve uma malha pior nem uma peça
/// aberta.
///
/// ⭐⭐⭐ **A GRADE QUE ALIMENTA A CADEIA É A DO `Draft`, E ISSO É UMA MEDIÇÃO.**
///
/// ⚠️ **Não é uma economia: é a resposta certa.** O `ph2d_remesh_iso::target_edge` é
/// `alpha · diagonal_da_caixa` — ele **não olha para a densidade da malha**. ⇒ a cadeia
/// remalha para o mesmo alvo venha a entrada de que profundidade vier, e tudo o que a grade
/// fina traz a mais é deitado fora pelo F1, **depois de pago**:
///
/// | prof | quads que entram | F1 ms | cadeia ms | o que sai |
/// |---|---|---|---|---|
/// | **6** | 17 550 | **632** | ⭐ **4 613** | 6,4° · 2 539 quads |
/// | 7 | 69 966 | 4 513 | 8 193 | 6,3° · 2 471 quads |
/// | 8 | 280 062 | — | 47 454 | ⛔ 55,5° (o veto recusa) |
/// | 9 | 1 120 158 | ⛔ **482 451** | ⛔ **495 244** | 6,4° · 2 436 quads |
///
/// ⭐ **Oito minutos e quinze segundos, 107× o preço, para a MESMA resposta a uma casa
/// decimal** — e 97 % disso é a fase zero a mastigar um milhão de faces até 2 436 quads.
/// ⛔ E não é só preço: nas profundidades 7 e 8 a fidelidade (medida no CAMPO, que é exacto)
/// **piora**, e na 8 a peça é destruída. *Uma grade mais fina não é mais informação para a
/// cadeia: é ruído que ela tem de mastigar e depois segue mal.*
///
/// ⭐⭐ **Medido de ponta a ponta pelo caminho do produto** (`measure_the_export_wall_clock`,
/// release, esfera): `Draft` **4 686 ms** · `Fine` **4 648 ms** · `Max` **6 435 ms** — contra os
/// `495 244 + 1 400 ms` que o `Max` pagava. **77×**, e os três saem **idênticos até à última
/// casa** (`1,0794725` de aspecto, `6,417694°`, 2 539 quads), porque comem a mesma grade.
///
/// ⚠️ **O que sobra do custo é do `ph2d-gridmap`**: no `Max` o G3/G5 é `3 322` dos `4 677 ms` da
/// cadeia — 71 %. A crate é da `line/quadextract`.
///
/// ⚠️ **A coincidência com o `Draft` não é coincidência**: `2^6 = 64` subdivisões da peça e
/// um `ALPHA` de 2 % da diagonal são a MESMA escala, os dois relativos ao bordo da peça
/// (W33). Por isso a razão `célula/alvo` fica em `0,47` e não deriva com o tamanho da peça.
/// Sonda: `ph2d_field_eval::tests::measure_what_the_chain_gains_from_a_finer_grid`.
///
/// # Errors
/// A extração pode recusar; ver [`ph2d_field_eval::MeshError`].
///
/// Devolve `(a grade que alimenta a cadeia, a malha do nível pedido)`. O primeiro é `None` quando
/// os dois coincidem — no nível `Draft` não há segunda extração a fazer.
fn meshes_for(
    doc: &ph2d_field::FieldDoc,
    reg: &ph2d_field_eval::hybrid::Registry,
    level: ExportLevel,
) -> Result<(Option<ph2d_mesh::Mesh>, ph2d_mesh::Mesh), ph2d_field_eval::MeshError> {
    let mesh = ph2d_field_eval::extract::extract(doc, reg, level.depth())?;
    let feed = if level.depth() == ExportLevel::Draft.depth() {
        None
    } else {
        Some(ph2d_field_eval::extract::extract(
            doc,
            reg,
            ExportLevel::Draft.depth(),
        )?)
    };
    Ok((feed, mesh))
}

/// **A malha vira os bytes do formato escolhido.**
///
/// ⚠️ Um OBJ de 934 k triângulos são dezenas de MB de texto, e a gravação em disco vem a seguir —
/// as duas correm **fora da thread que desenha** ([`crate::field3d_export_job`]), como o resto da
/// exportação.
///
/// ⚠️ **A pose é a identidade, e isso não é um esquecimento.** O documento cozido já tem toda a
/// cadeia de poses dentro do campo (`cook` compõe, `place` aplica), então a malha sai em MUNDO.
/// Uma pose aqui aplicaria a transformação duas vezes.
pub(crate) fn bytes_of(fmt: MeshFormat, mesh: &ph2d_mesh::Mesh) -> Vec<u8> {
    fmt.write(&[ExportPiece {
        name: None,
        mesh,
        pose: ph2d_mesh::Pose::IDENTITY,
    }])
}

/// ⭐⭐ **A METADE PESADA da exportação — a que não pode correr na thread que desenha.**
///
/// # ⚠️ Os DOIS reports do Enio, no mesmo dia, e são duas camadas
///
/// | report | mecanismo | cura |
/// |---|---|---|
/// | *"a mensagem não aparece"* | o quadro seguinte cobrava o congelamento ao relógio do chrome, e o toast morria | declarar o congelamento (`crate::modal`) |
/// | *"o linux fica cinza"* | com 12 s sem responder, o compositor declara a janela morta | ⭐ **não congelar** ([`crate::field3d_export_job`]) |
///
/// ⛔ **Declarar cura a MENSAGEM e não cura o congelamento** — e a segunda cura torna a primeira
/// desnecessária *neste caminho*: com a conta fora da thread não há nada a declarar, e um
/// `note_stall` num trabalhador escreveria num `thread_local` que ninguém lê. A porta do
/// `crate::modal` continua a ser a resposta certa para o **diálogo**, que é o que ela sempre foi.
///
/// # ⚠️ Por que ela é uma FUNÇÃO e não um bloco
///
/// O [`field3d_export`] abre um diálogo nativo e **não é alcançável de um teste**. Esta metade é
/// pura — e é ela, e não a que abre o diálogo, que os gates da grade alcançam.
pub(crate) fn cook(
    doc: &ph2d_field::FieldDoc,
    reg: &ph2d_field_eval::hybrid::Registry,
    level: ExportLevel,
) -> Result<(ph2d_mesh::Mesh, ph2d_quadchain::Verdict), ph2d_field_eval::MeshError> {
    let (feed, mesh) = meshes_for(doc, reg, level)?;
    let feed = feed.as_ref().unwrap_or(&mesh);
    // ⚠️ **O alvo sai da malha que ENTRA na cadeia** — é a mesma caixa, e é ela que a fase zero
    // usa para reproduzir este número. Tirá-lo da outra seria pedir uma escala e remalhar noutra.
    let target = ph2d_remesh_iso::target_edge(feed, ph2d_remesh_iso::ALPHA);
    // ⚠️ **Come uma malha, outra fica se ela perder**: quando o veto recusa, o artista tem de
    // levar a malha do nível que ele pediu, não a grade grossa que alimentou a cadeia.
    Ok(ph2d_quadchain::quads_or_keep_from(feed, &mesh, target))
}

/// ⚠️ **Recebe os TOASTS, e não o `App`.** Ela é chamada de dentro do quadro, onde o `gfx` já está
/// emprestado — e pedir `&mut self` ali é um empréstimo duplo. Pedir só o que se usa é o que a
/// deixa chamável de onde ela precisa de ser chamada.
pub(crate) fn field3d_export(level: ExportLevel, toasts: &mut ph2d_editor::ToastQueue) {
    let say =
        |toasts: &mut ph2d_editor::ToastQueue, m: String| toasts.push(ph2d_editor::Toast::info(m));
    // ⚠️ **Uma de cada vez, e a recusa é EM ALTO** — ver [`crate::field3d_export_job`]. Recusar em
    // silêncio deixaria o artista a concluir que o botão está partido.
    if crate::field3d_export_job::is_running() {
        say(toasts, "An export is already running".into());
        return;
    }
    let Some(doc) = crate::field3d_smoke::with_smoke(|s| s.doc.clone()).flatten() else {
        say(toasts, "Nothing to export: the part is empty".into());
        return;
    };

    // ⚠️ **UM FILTRO POR FORMATO** — a lição que o export da escultura pagou: com um filtro
    // único listando as três extensões, o diálogo nativo completa o nome com a PRIMEIRA delas e
    // `volta.ply` sai `volta.ply.obj`.
    let mut dialog = rfd::FileDialog::new();
    for f in MeshFormat::ALL {
        dialog = dialog.add_filter(f.extension().to_uppercase(), &[f.extension()]);
    }
    // ⚠️ **Pela PORTA** (`crate::modal`), nunca `dialog.save_file()` direto: o diálogo congela o
    // loop, e o quadro seguinte cobrava esse congelamento ao relógio do chrome — matando este
    // mesmo toast antes de ele ser visto. Gate: `every_field3d_modal_goes_through_the_door`.
    //
    // ⚠️ **O diálogo FICA na thread que desenha**, e é a única metade que fica: ele é uma janela do
    // sistema, e o compositor sabe que o loop está parado por vontade dele. O que não podia ficar é
    // a CONTA — ver [`crate::field3d_export_job`].
    let Some(path) = crate::modal::save_file(
        dialog.set_file_name(format!("model.{}", MeshFormat::Obj.extension())),
    ) else {
        return;
    };
    // ⚠️ Uma extensão desconhecida **não vira OBJ em silêncio**: o arquivo abriria como
    // corrompido noutro programa, apontando para o lugar errado.
    let Some(fmt) = path
        .extension()
        .and_then(|e| e.to_str())
        .and_then(MeshFormat::from_extension)
    else {
        say(
            toasts,
            format!(
                "Unknown extension: use {}",
                MeshFormat::ALL
                    .map(|f| format!(".{}", f.extension()))
                    .join(", ")
            ),
        );
        return;
    };

    let reg = crate::field3d_smoke::sampled_registry();
    // ⭐ **O artista tem de saber que ALGUMA COISA começou.** Doze segundos de silêncio com o app
    // vivo leem-se como *"o botão não fez nada"* — o mesmo defeito que a janela cinza tinha, com
    // outra cara. ⛔ E o aviso **não promete um prazo**: ele depende da peça, e um número inventado
    // seria pior que nenhum.
    say(toasts, "Exporting... the file is being written".into());
    if !crate::field3d_export_job::spawn(move || export_to_file(level, &doc, &reg, &path, fmt)) {
        say(toasts, "Could not start the export".into());
    }
}

/// ⭐⭐ **GRAVA POR INTEIRO OU NÃO GRAVA** — arquivo temporário ao lado, depois `rename`.
///
/// # ⚠️ Esta função nasceu de uma consequência da wave que a precede
///
/// Enquanto a exportação corria na thread que desenha, o artista **não conseguia** fechar o app a
/// meio dela: o loop estava congelado e o pedido de fechar ficava na fila. Tirar o trabalho da
/// thread ([`crate::field3d_export_job`]) devolveu-lhe essa capacidade — e com ela a janela de 12 s
/// em que um `std::fs::write` a meio deixa **meio arquivo** com o nome certo. ⛔ Um OBJ truncado
/// abre noutro programa como uma peça partida, e nada diz que ele está incompleto.
///
/// ⭐ *Uma cura pode abrir a porta que outra fechava — quem move o trabalho tem de reconferir o que
/// o congelamento estava a proteger.*
///
/// ⚠️ **O temporário nasce NA PASTA DO DESTINO**, nunca no `/tmp`: o `rename` só é atómico dentro
/// do mesmo sistema de arquivos, e um destino noutro disco faria a operação cair para uma cópia —
/// que é exactamente o que se está a evitar.
///
/// ⚠️ E se o `rename` falhar, o temporário é **removido**: deixá-lo seria semear a pasta do artista
/// com restos que ninguém sabe apagar.
fn write_atomically(path: &std::path::Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut tmp = path.to_path_buf();
    let stem = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("export");
    tmp.set_file_name(format!(".{stem}.ph2d-partial"));
    std::fs::write(&tmp, bytes)?;
    std::fs::rename(&tmp, path).inspect_err(|_| {
        let _ = std::fs::remove_file(&tmp);
    })
}

/// ⭐⭐ **A EXPORTAÇÃO INTEIRA, longe da thread que desenha** — e ela devolve a **mensagem pronta**.
///
/// ⚠️ **Montar a frase é trabalho puro** (contagens, caixa da malha, o que o formato perde), e
/// fazê-lo deste lado deixa o quadro com uma coisa só a fazer: mostrar. *A fronteira mais barata é
/// a que só carrega texto.*
fn export_to_file(
    level: ExportLevel,
    doc: &ph2d_field::FieldDoc,
    reg: &ph2d_field_eval::hybrid::Registry,
    path: &std::path::Path,
    fmt: MeshFormat,
) -> String {
    let t0 = std::time::Instant::now();
    let (mesh, verdict) = match cook(doc, reg, level) {
        Ok(pair) => pair,
        Err(e) => return format!("Meshing failed: {e:?}"),
    };
    // ⚠️ **EM INGLÊS, como todo o resto do que o artista lê** — a primeira redação desta
    // linha saiu em português e passou por todo o portão, porque nenhum deles le um `format!`.
    let quality = match &verdict {
        ph2d_quadchain::Verdict::Adopted(r) => {
            format!(" · retopology: {:.1}° skew", r.shape.skew_p50)
        }
        // ⚠️ **Silencioso quando não muda nada.** Um aviso a dizer *"a melhoria opcional não se
        // aplicou"* seria ruído sobre uma exportação que correu bem.
        _ => String::new(),
    };
    // ⚠️ **Quads e triângulos são contagens DIFERENTES, e o toast diz as duas.** A saída deste
    // extrator é uma grade de quads (`extract`), e `faces().len()` conta quads; um STL só sabe
    // triângulos, então o número que o artista vê no Blender é o dobro. Dizer "tris" sobre uma
    // contagem de quads era uma etiqueta a prometer o que o modelo não entrega.
    let quads = mesh.faces().len();
    let tris: usize = mesh.faces().iter().map(ph2d_mesh::Face::tri_count).sum();
    let bytes = bytes_of(fmt, &mesh);
    let size = bytes.len();
    let wrote = write_atomically(path, &bytes);
    // ⚠️ **O relógio pára DEPOIS de gravar, e isso é uma correcção.** Ele parava antes de
    // serializar — e o artista compara o número com a espera dele, não com a metade que a
    // função escolheu contar.
    let ms = t0.elapsed().as_secs_f64() * 1000.0;
    match wrote {
        Ok(()) => {
            let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("?");
            // ⭐ O que saiu de FACTO — triângulos e KB —, não o que o nível prometia.
            // ⭐ **E o TAMANHO** — a primeira pergunta de quem leva o arquivo para outro
            // programa, e a única que o toast não respondia. É a caixa da MALHA, não a do
            // andaime: ver [`piece_size`].
            let [sx, sy, sz] = piece_size(&mesh);
            let sitio = piece_origin_note(&mesh);
            format!(
                "Exported {quads} quads = {tris} tris, {sx:.2} x {sy:.2} x {sz:.2}, \
                 {} KB in {ms:.0} ms -- {name} ({}){sitio}{quality}",
                size / 1024,
                crate::sculpt3d::lost_by(fmt)
            )
        }
        Err(e) => format!("Export failed: {e}"),
    }
}
