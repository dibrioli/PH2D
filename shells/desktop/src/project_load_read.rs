//! **QUE BYTES SÃO ESTES** — a metade do load que decide o TIPO com que desserializar, corre a
//! escada de migração e devolve o `ProjectFile` já no formato de hoje. Irmão de
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
        // ⭐ **A PRIMEIRA migração da história do repo** (ADR-0164 F1). Até esta wave a
        // política de facto era *"versão diferente = recusado"* — a auditoria de 21/08
        // registou-a como ambiguidade em aberto (§8 item 7: HR-14 exige `migrate_vN_to_vN+1`
        // e havia **zero**).
        //
        // ⚠️ A versão tem de ser lida **antes** do resto: o postcard é posicional, então
        // desserializar bytes v95 com o tipo v96 não dá erro — dá lixo. Por isso o `ver` sai
        // sozinho primeiro, e só depois se escolhe o tipo com que ler o corpo.
        let ver: u32 = match postcard::take_from_bytes::<u32>(&bytes) {
            Ok((v, _)) => v,
            Err(e) => {
                eprintln!("[proj] erro ao ler a versao de {path}: {e}");
                return None;
            }
        };
        let (file, migrated_counter) = match ver {
            PROJECT_SCHEMA => match postcard::from_bytes::<(u32, ProjectFile)>(&bytes) {
                Ok((_, f)) => (f, None),
                Err(e) => {
                    eprintln!("[proj] erro ao ler {path}: {e}");
                    return None;
                }
            },
            // ⭐ **98 → 99: o corte da `Sprite`** (ADR-0164 F1 passo 6). A forma do ficheiro é a
            // mesma, então ele lê-se com o tipo VIVO — o que muda são os bytes DENTRO do blob da
            // `Sprite`, que o parse atravessa sem olhar. Ver `crate::project_migrate_sprite`.
            //
            // ⚠️⚠️ **ESTE BRAÇO ERA `97 =>` E FOI RE-ENGATADO NA INTEGRAÇÃO de 2026-08-26.** Ele
            // nasceu quando o corte da `Sprite` era o degrau `98`; a `line/Vector` pôs outro degrau
            // no meio (o `morph_shape` do `ObjectPose`) e o corte passou a ser o `99`. ⛔ **Deixá-lo
            // em `97` seria o defeito que o bump existe para impedir:** este braço lê os bytes com o
            // tipo **VIVO**, e o tipo vivo já tem o campo da `line/Vector` — um v97 sairia lixo
            // bem-formado. *Um número que soma entre linhas move o CONSUMIDOR dele, não só a
            // declaração.*
            //
            // ⛔ **Um v97 é RECUSADO**, e é a decisão da `line/Vector` aplicada até ao fim: ela
            // escolheu, com o Enio, não congelar um `ProjectFileV97` (*"não há projetos salvos"*).
            // Sem tipo congelado não existe forma honesta de ler aqueles bytes. O `v95` continua a
            // subir a escada inteira, porque o tipo dele **está** congelado.
            // ⛔⛔ **O braço `98` MORREU em 2026-08-30, e a razão é a mesma que matou o `97`.**
            //
            // Ele lia os bytes com o tipo **VIVO**, e a premissa era *«a forma do ficheiro é a
            // mesma»*. O degrau `105` partiu-a de uma maneira que o `104` não partia: a biblioteca
            // entrou como **último campo do `ProjectState`**, que é o **primeiro campo do
            // `ProjectFile`** ⇒ no fluxo de bytes ela cai **no MEIO**. Um campo apendado no fim
            // fazia o postcard chegar ao fim dos bytes e **recusar em voz alta**; um campo no meio
            // faz-lhe ler *lixo bem-formado*.
            //
            // ⇒ um v98 cai no `_ =>` e é **recusado com o número na frase**, que é a decisão que a
            // `line/Vector` já tomou para o v97 e que o Enio confirmou (*«não há projetos
            // salvos»*). ⚠️ Quem um dia precisar dele **congela um `ProjectFileV98` primeiro** —
            // sem tipo congelado não há forma honesta de ler aqueles bytes.
            // ⭐⭐⭐ **128 → 129: as TAGS** (TOP-20 #9). Duas coisas mudaram e nenhuma é aditiva no
            // fio: a árvore entrou no `ProjectState` (um campo no MEIO do fluxo de bytes, porque o
            // `state` é o primeiro campo do ficheiro) e o `SignalAction` ganhou o alvo por TAG.
            //
            // ⚠️ **É por isso que este braço lê com um tipo CONGELADO** e não com o vivo, ao
            // contrário do que o braço do corte da `Sprite` fazia: ali a forma do ficheiro era a
            // mesma. Aqui, ler com o vivo daria lixo bem-formado a partir do campo novo.
            //
            // ⚠️ **E COM migração, ao contrário dos últimos onze degraus:** o `128` é o schema que o
            // `main` tem desde 10/09, com o `Timer` e o `SignalActions` dentro — um projecto gravado
            // desde então tem autoria que recusar apagaria.
            128 => {
                match postcard::from_bytes::<(u32, crate::project_migrate::ProjectFileV128)>(&bytes)
                {
                    Ok((_, old)) => {
                        let m = crate::project_migrate::migrate_v128_to_v129(old);
                        eprintln!(
                            "[proj] migrado v128 -> v{PROJECT_SCHEMA} ({} tabela(s) de accoes reescrita(s), {} ilegivel(eis))",
                            m.actions.tables, m.actions.unreadable
                        );
                        self.toast(format!(
                            "Project migrated from format 128 to {PROJECT_SCHEMA}"
                        ));
                        (m.file, None)
                    }
                    Err(e) => {
                        eprintln!("[proj] v128 ilegivel: {e}");
                        self.toast(format!(
                            "Project refused: format 128 file is unreadable ({e})"
                        ));
                        return None;
                    }
                }
            }
            95 => {
                match postcard::from_bytes::<(u32, crate::project_migrate::ProjectFileV95)>(&bytes)
                {
                    Ok((_, old)) => {
                        let mut m = crate::project_migrate::migrate_v95_to_v96(old);
                        // ⚠️ **A escada é ENCADEADA: um v95 sobe a 96 e depois passa pelo corte
                        // da 99.** O `migrate_v95_to_v96` não sabe da `Sprite`, e um v95 tem
                        // sprites v4 como qualquer outro — saltar este passo daria um ficheiro
                        // antigo que abre com todas as sprites a ler lixo bem-formado.
                        let split = crate::project_migrate_sprite::split_sprite_blobs(
                            &mut m.file.state.world,
                        );
                        eprintln!(
                            "[proj] migrado v95 -> v{PROJECT_SCHEMA} ({} objetos receberam identidade, {} sprites partidas)",
                            m.file.state.world.entities.len(),
                            split.sprites
                        );
                        self.toast(format!(
                            "Project migrated from format 95 to {PROJECT_SCHEMA}"
                        ));
                        (m.file, Some(m.stable_id_counter))
                    }
                    Err(e) => {
                        eprintln!("[proj] v95 ilegivel: {e}");
                        self.toast(format!(
                            "Project refused: format 95 file is unreadable ({e})"
                        ));
                        return None;
                    }
                }
            }
            _ => {
                eprintln!("[proj] schema {ver} != {PROJECT_SCHEMA} — recusado");
                self.toast(format!(
                    "Project refused: file format {ver}, this build reads {PROJECT_SCHEMA}"
                ));
                return None;
            }
        };
        Some((file, migrated_counter))
    }
}
