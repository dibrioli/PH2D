//! **QUE BYTES SÃO ESTES** — a metade do load que decide o TIPO com que desserializar, corre a
//! recusa tudo o que não é o `PROJECT_SCHEMA` de hoje (sem projetos antigos: dono 07/11) e devolve o `ProjectFile`. Irmão de
//! [`super::project_load`] pelo teto de LOC (HR-18, 600), e o corte é de RESPONSABILIDADE: aqui
//! *que ficheiro é este*; lá *o que a sessão faz com um ficheiro aceite*.
//!
//! ⚠️ **O corte nasceu de um tecto a curar OUTRO** (2026-09-13): o degrau `129` (as TAGS) levou o
//! `project_load_from` a `228 / 200` do tecto por FUNÇÃO, partir a função acrescentou seis linhas
//! ao ficheiro, e isso levou o `project_load.rs` a `606 / 600` do tecto por FICHEIRO. *Curar um
//! tecto pode acender o irmão, e os dois gates correm em suítes diferentes* — o da função no
//! `--bins`, o do ficheiro em `tests/it/`.
//!
//! ⛔ **Uma recusa daqui é DITA** (terminal + toast, com o número do formato na frase) e devolve
//! `None`: um load que falha em silêncio deixa o artista a olhar para a cena antiga a pensar que
//! gravou.
//!
//! Filho (`#[path]`) de [`super::project`] pela mesma razão do irmão: ele usa a forma do
//! `ProjectFile`, o `PROJECT_SCHEMA` e o `toast`, que o pai possui.

use super::*;

impl crate::App {
    /// ⭐⭐ **QUE BYTES SÃO ESTES, e como se leem** — a metade do load que escolhe o TIPO com que
    /// desserializar e devolve o ficheiro já migrado. `None` = recusado, e a recusa já foi dita
    /// (terminal + toast) com o número na frase.
    ///
    /// ⚠️ **Saiu do [`Self::project_load_from`] em 2026-09-13**, quando o degrau `129` (as TAGS)
    /// levou aquela função a `228 / 200` do tecto por função. O corte é de RESPONSABILIDADE, que é
    /// a cura que o gate manda: aqui *que ficheiro é este*; lá *o que a sessão faz com ele*.
    pub(super) fn read_project_file(&mut self, path: &str) -> Option<(ProjectFile, Option<u64>)> {
        let bytes = match std::fs::read(path) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("[proj] sem arquivo {path}: {e}");
                return None;
            }
        };
        // ⚠️ A versão lê-se ANTES do resto (ADR-0164 F1): o postcard é posicional — bytes v95 com o
        // tipo v96 não dão erro, dão lixo.
        let ver: u32 = match postcard::take_from_bytes::<u32>(&bytes) {
            Ok((v, _)) => v,
            Err(e) => {
                eprintln!("[proj] erro ao ler a versao de {path}: {e}");
                return None;
            }
        };
        if ver != PROJECT_SCHEMA {
            eprintln!("[proj] schema {ver} != {PROJECT_SCHEMA} — recusado");
            self.toast(ph2d_i18n::tr_with(
                "shell.project_load.project_refused_file",
                &[("ver", &ver), ("PROJECT_SCHEMA", &PROJECT_SCHEMA)],
            ));
            return None;
        }
        le_com_o_tipo_vivo(&bytes, path).map(|f| (f, None))
    }
}

/// Os bytes lidos com o tipo VIVO (`None` = ilegível, dito no terminal).
fn le_com_o_tipo_vivo(bytes: &[u8], path: &str) -> Option<ProjectFile> {
    match postcard::from_bytes::<(u32, ProjectFile)>(bytes) {
        Ok((_, f)) => Some(f),
        Err(e) => {
            eprintln!("[proj] erro ao ler {path}: {e}");
            None
        }
    }
}
