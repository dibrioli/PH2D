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
/// ⚠️ **Recebe os TOASTS, e não o `App`.** Ela é chamada de dentro do quadro, onde o `gfx` já está
/// emprestado — e pedir `&mut self` ali é um empréstimo duplo. Pedir só o que se usa é o que a
/// deixa chamável de onde ela precisa de ser chamada.
pub(crate) fn field3d_export(level: ExportLevel, toasts: &mut ph2d_editor::ToastQueue) {
    let say =
        |toasts: &mut ph2d_editor::ToastQueue, m: String| toasts.push(ph2d_editor::Toast::info(m));
    {
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
        let Some(path) = dialog
            .set_file_name(format!("model.{}", MeshFormat::Obj.extension()))
            .save_file()
        else {
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

        let t0 = std::time::Instant::now();
        let reg = crate::field3d_smoke::sampled_registry();
        let mesh = match ph2d_field_eval::extract::extract(&doc, &reg, level.depth()) {
            Ok(m) => m,
            Err(e) => {
                say(toasts, format!("Meshing failed: {e:?}"));
                return;
            }
        };
        let ms = t0.elapsed().as_secs_f64() * 1000.0;
        // ⚠️ **A pose é a identidade, e isso não é um esquecimento.** O documento cozido já tem
        // toda a cadeia de poses dentro do campo (`cook` compõe, `place` aplica), então a malha sai
        // em MUNDO. Uma pose aqui aplicaria a transformação duas vezes.
        // ⚠️ **Quads e triângulos são contagens DIFERENTES, e o toast diz as duas.** A saída deste
        // extrator é uma grade de quads (`extract`), e `faces().len()` conta quads; um STL só sabe
        // triângulos, então o número que o artista vê no Blender é o dobro. Dizer "tris" sobre uma
        // contagem de quads era uma etiqueta a prometer o que o modelo não entrega.
        let quads = mesh.faces().len();
        let tris: usize = mesh.faces().iter().map(ph2d_mesh::Face::tri_count).sum();
        let bytes = fmt.write(&[ExportPiece {
            name: None,
            mesh: &mesh,
            pose: ph2d_mesh::Pose::IDENTITY,
        }]);
        let size = bytes.len();
        match std::fs::write(&path, bytes) {
            Ok(()) => {
                let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("?");
                // ⭐ O que saiu de FACTO — triângulos e KB —, não o que o nível prometia.
                say(
                    toasts,
                    format!(
                        "Exported {quads} quads = {tris} tris, {} KB in {ms:.0} ms -- {name} ({})",
                        size / 1024,
                        crate::sculpt3d::lost_by(fmt)
                    ),
                );
            }
            Err(e) => {
                say(toasts, format!("Export failed: {e}"));
            }
        }
    }
}
