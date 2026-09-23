//! **A PORTA DE SAÍDA no app** — Ctrl+Shift+E escreve a cena num arquivo.
//!
//! Irmão do [`super::sculpt3d_import`], e o par dele: a W8.4 deu a entrada, a
//! W8.3 deu o documento, e sem esta a escultura **entra, salva e não sai**. Um
//! `.ph2d` só abre aqui; levar a malha ao Blender ou a uma impressora é o que
//! torna o módulo parte de um fluxo.
//!
//! ⚠️ **A EXTENSÃO decide o formato**, e o diálogo não tem um segundo seletor —
//! ver [`MeshFormat::from_extension`]. Duas portas para *"que formato é este?"*
//! divergem no primeiro `retrato.obj` salvo com "STL" escolhido ao lado.
//!
//! ⚠️ **O que o formato NÃO carrega é DITO no toast**, e a frase sai da mesma
//! tabela que o escritor consulta ([`MeshFormat::keeps_colour`] /
//! [`keeps_pieces`](MeshFormat::keeps_pieces)). Exportar em STL e perder a
//! pintura em silêncio seria a resposta errada com a confiança da certa — e a
//! MÁSCARA não sobrevive a nenhum dos três, então ela é dita sempre.

use ph2d_mesh::{ExportPiece, MeshFormat};

use super::Sculpt3dScene;

impl Sculpt3dScene {
    /// A cena inteira pronta para escrever — o nível **VIVO** de cada peça.
    ///
    /// ⚠️ **O nível vivo, não a base nem o topo:** o que está na tela é o que o
    /// artista acabou de julgar. Exportar a base entregaria um bloco liso a quem
    /// esculpiu detalhe; exportar o topo entregaria milhões de triângulos a quem
    /// desceu de propósito para trabalhar grosso.
    pub(crate) fn export_pieces(&self) -> Vec<ExportPiece<'_>> {
        self.objects
            .iter()
            .map(|o| ExportPiece {
                name: None,
                mesh: o.stack.mesh(),
                pose: o.pose,
            })
            .collect()
    }
}

/// **Escreve a cena num arquivo escolhido pelo artista.**
///
/// ⚠️ Sem cena não há o que exportar, e o silêncio seria indistinguível de
/// um diálogo que falhou — o toast diz.
pub fn export(scene: Option<&Sculpt3dScene>, toasts: &mut ph2d_editor_core::ToastQueue) {
    // ⚠️ **A ausência de JANELA fica do lado da shell, e a de CENA fica aqui** — os dois casos
    // eram o mesmo `let ... else` e não são a mesma coisa: sem GPU não há gesto nenhum a
    // reportar, sem escultura há um artista que carregou num botão e merece a razão.
    let Some(scene) = scene else {
        crate::import::toast(
            toasts,
            ph2d_i18n::tr("app.sculpt3d.export.nothing_to_export_no_sculpture_open").into(),
        );
        return;
    };
    let n = scene.objects.len();

    // ⚠️ **UM FILTRO POR FORMATO, e o report do Enio é o motivo:** com um
    // filtro único listando as três extensões, o diálogo nativo (o portal
    // XDG, o GTK, o Windows) **completa o nome com a PRIMEIRA delas** — o
    // artista digitava `volta.ply` e o arquivo saía `volta.ply.obj`. Não é
    // uma segunda porta para *"que formato é este?"*: quem decide continua
    // sendo a extensão do caminho FINAL, e o filtro é só como o diálogo
    // ajuda a escrevê-la.
    let mut dialog = rfd::FileDialog::new();
    for f in MeshFormat::ALL {
        dialog = dialog.add_filter(f.extension().to_uppercase(), &[f.extension()]);
    }
    // ⛔⛔⛔ **PELA PORTA, nunca `dialog.save_file()` direto** (report do dono, 22/09:
    // *«passo 4 não mostra mensagem nenhuma no app»*). Um diálogo modal CONGELA o laço, e o
    // quadro seguinte traz um `wall_dt` do tamanho do tempo que o artista passou nele — o
    // `ToastQueue` anda `3 s` de relógio de PAREDE, logo a mensagem escrita aqui era apagada
    // no `tick` antes de chegar a ser pintada. A cura já existia desde 2026-08-22 (com as
    // palavras do dono de então: *«não vejo em nenhum lugar a mensagem»*) e mora no
    // [`ph2d_app_host::modal`], que desconta a parte parada; **este chamador é que estava
    // fora dela.** *Uma cura escrita para um chamador não é uma lei — só uma PORTA é.*
    let Some(path) = ph2d_app_host::modal::save_file(
        dialog.set_file_name(format!("sculpt.{}", MeshFormat::Obj.extension())),
    ) else {
        return;
    };
    // ⚠️ **Uma extensão que não reconhecemos NÃO vira OBJ em silêncio.** Um
    // default calado escreveria um arquivo OBJ com o nome `.fbx`, e o
    // primeiro programa a abri-lo diria que o ARQUIVO está corrompido —
    // apontando para o lugar errado.
    let Some(fmt) = path
        .extension()
        .and_then(|e| e.to_str())
        .and_then(MeshFormat::from_extension)
    else {
        crate::import::toast(
            toasts,
            ph2d_i18n::tr_with(
                "app.sculpt3d.export.unknown_extension_use",
                &[(
                    "join",
                    &(MeshFormat::ALL
                        .map(|f| format!(".{}", f.extension()))
                        .join(", ")),
                )],
            ),
        );
        return;
    };

    // O empréstimo das malhas termina aqui: `write` já produziu os bytes.
    let bytes = fmt.write(&scene.export_pieces());
    let size = bytes.len();
    match std::fs::write(&path, bytes) {
        Ok(()) => {
            let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("?");
            // ⛔⛔⛔ **DOIS balões, e a partição é MEDIDA** (report do dono, 22/09: *«as
            // mensagens estão cortadas com … não consigo ler tudo»*). O balão tem `300 px`
            // ≈ 48 caracteres e a frase única media `95`; ⚠️ **e a de antes desta wave já
            // media `60` — ela NUNCA foi legível**, o que torna isto pré-existente e não uma
            // dívida que a cláusula da tinta fina criou.
            //
            // ⭐⭐ **A lei da partição:** *a metade que TEM de ser lida não pode ter parte
            // variável.* Uma frase única cabe com `teste.obj` (`48`) e **estoura com um nome
            // real** (`retrato-da-personagem-v3.obj` ⇒ `62`) — e a elisão corta o FIM, que é
            // exactamente o aviso. ⇒ a CONFIRMAÇÃO leva o nome (elide, e o artista acabou de
            // o escrever; o balão do `text_elide` mostra-o ao passar o rato) e o AVISO fica
            // sozinho, **sem uma única parte variável**, medido a caber no pior caso.
            crate::import::toast(
                toasts,
                ph2d_i18n::tr_with(
                    "app.sculpt3d.export.exported_piece_s_kb",
                    &[("n", &n), ("size", &(size / 1024)), ("name", &name)],
                ),
            );
            crate::import::toast(toasts, lost_by(fmt, scene.alguma_peca_tem_tinta_fina()));
        }
        Err(e) => crate::import::toast(
            toasts,
            ph2d_i18n::tr_with("app.sculpt3d.export.export_failed", &[("e", &e)]),
        ),
    }
}

/// O que este formato deixa para trás, em palavras.
///
/// ⚠️⚠️ **A nota que aqui estava ENVELHECEU, e o código di-lo:** ela dizia
/// *«`pub(crate)` porque a modelagem 3D o CHAMA ([`crate::field3d_export`])»*, e
/// esse módulo **não existe nesta crate** desde que a família saiu da shell — a
/// modelagem 3D chama hoje o [`ph2d_mesh::lost_by`] **directamente**. *Uma nota
/// que justifica uma visibilidade por um chamador que mudou de casa lê-se como
/// medição e é um palpite.*
///
/// ⭐ **O que continua verdade é a lei, e é ela que o gate da shell afirma:** há
/// **uma** tabela, e os dois consumidores delegam nela. Copiá-la para lá era a
/// alternativa, e duas listas divergem — a que fica errada diz *"cor
/// preservada"* sobre um STL com a confiança da certa.
///
/// ⚠️ **Deriva da MESMA tabela que o escritor consulta.** Uma segunda lista aqui
/// diria *"cor preservada"* sobre um STL no dia em que alguém trocasse o
/// escritor — e um aviso errado é pior que aviso nenhum, porque o artista
/// confia nele e só descobre no outro programa.
///
/// ⚠️ A **máscara** é dita sempre: nenhum dos três formatos tem campo para ela, e
/// isso não é uma pergunta — é uma constante. Quem a preserva é o documento.
pub(crate) fn lost_by(fmt: MeshFormat, has_fine_paint: bool) -> String {
    // ⚠️ **UMA porta, e ela mudou-se para o dono** (W2): esta lei é puro `MeshFormat`
    // (`keeps_colour` / `keeps_pieces`), e o módulo de modelagem 3D passou a ser o **segundo**
    // consumidor dela ao sair da shell. Duas cópias divergiriam no dia do quarto formato.
    ph2d_mesh::lost_by(fmt, has_fine_paint)
}
