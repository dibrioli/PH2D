//! ⭐ **A PORTA DE SAÍDA da modelagem 3D** — o campo vira malha e vai para um arquivo.
//!
//! # ⚠️ Exportar é a primeira vez que este módulo PERDE informação de propósito
//!
//! Um campo tem resolução **infinita**: o filete é uma fórmula, e ampliar não revela serrilha. Uma
//! malha não — ela é uma escolha de quantos triângulos. Todo o resto do módulo foi construído para
//! **não** decidir isso cedo ([ADR-0161 §2]), e aqui a decisão é inevitável.
//!
//! Então ela é **explícita e medida**: os três níveis são profundidades de octree, e cada uma tem o
//! número ao lado ([`ExportLevel`]). O toast diz quantos triângulos saíram de facto — não uma
//! promessa, o resultado.
//!
//! # ⚠️ A malha SERRILHA aresta viva, e é por isso que a tela não passa por aqui
//!
//! Medido no spike (`docs/3DModeling/01_resultados_spike.md` §2). É aceitável num artefato de
//! exportação e **não** no que o artista julga — a tela continua a ser o campo traçado.
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

    /// A profundidade de octree deste nível — **MEDIDA**, não escolhida.
    ///
    /// `measure_export_resolution` sobre a cena 1 (três cilindros com filete), em release:
    ///
    /// | prof | triângulos | ms |
    /// |---|---|---|
    /// | 4 | 1 752 | 3,1 |
    /// | **5** | **6 888** | **3,7** | ← Draft
    /// | 6 | 27 716 | 8,2 |
    /// | **7** | **61 540** | **17,9** | ← Fine
    /// | 8 | 91 710 | 46,0 |
    /// | **9** | **130 914** | **119,5** | ← Max
    ///
    /// ⭐ **Os triângulos SATURAM e o relógio não.** De 4 para 6 a contagem quadruplica por degrau;
    /// de 7 para 9 ela só duplica, enquanto o tempo multiplica por 6,7. A eficiência (triângulos por
    /// ms) cai de **1 861** no degrau 5 para **1 096** no 9 — a superfície é finita, e a partir de
    /// certo ponto paga-se tempo por pouco detalhe novo.
    ///
    /// Daí os três: **5** é instantâneo para ver, **7** é o dobro do detalhe ainda instantâneo, e
    /// **9** é onde se aceita esperar uma batida por tudo o que a peça tem. ⛔ Acima de 9 não há
    /// degrau que compense — e um nível que ninguém escolheria não é um nível.
    pub(crate) fn depth(self) -> u8 {
        match self {
            ExportLevel::Draft => 5,
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
        let mesh = match ph2d_field_eval::mesh(&doc, level.depth()) {
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
        let tris = mesh.faces().len();
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
                        "Exported {tris} tris, {} KB in {ms:.0} ms -- {name} ({})",
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
