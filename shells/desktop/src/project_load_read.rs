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
        // ⭐ **O vetor de um ficheiro antigo lê-se pela forma do FIO dele** (F16c `22` até ao `189`;
        // W4.2 `23` até ao `199`): todo `VecPath` e `StrokeSpec` dentro do âmbito usa essa forma, e o
        // resto do ficheiro, o tipo do braço. Ver `ph2d_vec_scene::stroke_wire`.
        let fio = crate::project_schema::fio_do_projeto(ver);
        let _fio = fio.map(ph2d_vec_scene::WireLayout::enter);
        let (mut file, migrated_counter) = match ver {
            // ⭐ **193 → … → 197: as animações do personagem, o `TiledDraw`, a memória do uso da imagem, o
            // `ImageRegion`** — um componente novo cada, a mesma forma; falta-lhes o `jogo` (198): o `0` do `Vec` vazio.
            crate::project_schema::PROJECT_SCHEMA_SEM_MESA..=PROJECT_SCHEMA
            | crate::project_schema::PROJECT_SCHEMA_GEOMETRIA_V23
            | crate::project_schema::PROJECT_SCHEMA_SEM_CONSTRAINTS
            | crate::project_schema::PROJECT_SCHEMA_SEM_FORMATO_DO_JOGO
            | crate::project_schema::PROJECT_SCHEMA_SEM_REGION
            | crate::project_schema::PROJECT_SCHEMA_SEM_MEMORIA_DO_USO
            | crate::project_schema::PROJECT_SCHEMA_SEM_TILED_DRAW
            | crate::project_schema::PROJECT_SCHEMA_SEM_ANIMACOES_DO_PERSONAGEM => {
                // ⭐ do `204` (a `mesa` já lá está) para cima os bytes são os do tipo vivo; do `198` ao `203`
                // falta-lhes a `mesa` (um `0`); abaixo falta-lhes também o `jogo` (outro). Os itens (F30 §A1,
                // `205`) migram em `migra_depois_de_ler`: a forma do ficheiro é a mesma.
                let falta: &[u8] = if ver > crate::project_schema::PROJECT_SCHEMA_SEM_MESA {
                    &[]
                } else if ver >= crate::project_schema::PROJECT_SCHEMA_SEM_CONSTRAINTS {
                    &[0]
                } else {
                    &[0, 0]
                };
                match postcard::from_bytes::<(u32, ProjectFile)>(&[&bytes[..], falta].concat()) {
                    Ok((_, f)) => (f, None),
                    Err(e) => {
                        eprintln!("[proj] erro ao ler {path}: {e}");
                        return None;
                    }
                }
            }
            // ⛔ Um v97 e um v98 são RECUSADOS (sem tipo congelado não há forma honesta de os ler): o
            // porquê, verbatim, em `docs/archive/project-schema/bracos_97_e_98_recusados.md`.
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
                            "[proj] migrado v128 -> v{PROJECT_SCHEMA} ({} tabela(s) de accoes reescrita(s), {} ilegivel(eis); {} camera(s) do jogo com dolly 0, {} ilegivel(eis))",
                            m.actions.tables,
                            m.actions.unreadable,
                            m.cameras.tables,
                            m.cameras.unreadable
                        );
                        let chave = if m.sculpture_left_out {
                            "shell.project_load.project_migrated_from_128_without_sculpture"
                        } else {
                            "shell.project_load.project_migrated_from_128"
                        };
                        self.toast(ph2d_i18n::tr_with(
                            chave,
                            &[("PROJECT_SCHEMA", &PROJECT_SCHEMA)],
                        ));
                        (m.file, None)
                    }
                    Err(e) => {
                        eprintln!("[proj] v128 ilegivel: {e}");
                        self.toast_refused("shell.project_load.project_refused_format_128", &e);
                        return None;
                    }
                }
            }
            // ⭐ **190 → 191: os scripts no projeto** (ADR-0188). O campo novo é o ÚLTIMO, e um
            // `Vec` vazio no postcard é o byte `0` — o v190 (e o v189, que só difere no traço, já
            // coberto pelo âmbito acima) lê-se com o tipo vivo acrescentando-o. Os blobs do
            // `LuauScript` passam de v2 (o caminho) a v3; o load importa os caminhos.
            // ⭐ **192 → 193: as marcas do artista** (W9 das Macros): o campo novo é o ÚLTIMO, e o `0` do
            // `Vec` vazio acrescentado lê o v192 com o tipo vivo (os anteriores levam-no também). O `191`
            // tem a mesma forma; só o SIGNIFICADO da opacidade nas poses mudou (abaixo).
            // ⭐ **203 → 204: a mesa de som** (F30 §B2): mais um `Vec` vazio no fim — um `0` a mais.
            192 | crate::project_schema::PROJECT_SCHEMA_OPACIDADE_FORA_DOS_ESTADOS => {
                match postcard::from_bytes::<(u32, ProjectFile)>(&[&bytes[..], &[0, 0, 0]].concat()) {
                    Ok((_, f)) => (f, None),
                    Err(e) => {
                        eprintln!("[proj] v{ver} ilegivel: {e}");
                        self.toast_refused("shell.project_load.project_refused_format_191", &e);
                        return None;
                    }
                }
            }
            190 | crate::project_schema::PROJECT_SCHEMA_TRACO_V22 => {
                use ph2d_app_components::script_project::read_before_scripts;
                match read_before_scripts(
                    ver,
                    &[&bytes[..], &[0, 0, 0]].concat(),
                    |f: &mut ProjectFile| &mut f.state.world,
                    PROJECT_SCHEMA,
                ) {
                    Ok((f, aviso)) => {
                        self.toast(aviso);
                        (f, None)
                    }
                    Err(e) => {
                        self.toast_refused("shell.project_load.project_refused_format_189", &e);
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
                        let chave = if m.sculpture_left_out {
                            "shell.project_load.project_migrated_from_without_sculpture"
                        } else {
                            "shell.project_load.project_migrated_from"
                        };
                        self.toast(ph2d_i18n::tr_with(
                            chave,
                            &[("PROJECT_SCHEMA", &PROJECT_SCHEMA)],
                        ));
                        (m.file, Some(m.stable_id_counter))
                    }
                    Err(e) => {
                        eprintln!("[proj] v95 ilegivel: {e}");
                        self.toast_refused("shell.project_load.project_refused_format", &e);
                        return None;
                    }
                }
            }
            _ => {
                eprintln!("[proj] schema {ver} != {PROJECT_SCHEMA} — recusado");
                self.toast(ph2d_i18n::tr_with(
                    "shell.project_load.project_refused_file",
                    &[("ver", &ver), ("PROJECT_SCHEMA", &PROJECT_SCHEMA)],
                ));
                return None;
            }
        };
        migra_depois_de_ler(&mut file, ver, fio);
        Some((file, migrated_counter))
    }
}

/// A migração de CONTEÚDO depois de o ficheiro lido (a opacidade das poses; o fio antigo do vetor).
fn migra_depois_de_ler(file: &mut ProjectFile, ver: u32, fio: Option<u32>) {
    // ⭐ **Até ao `191` a opacidade não era canal de estado** (F22b): cada pose gravada recebe a
    // da forma, e o Show desses projetos continua a não a mexer.
    if ver <= crate::project_schema::PROJECT_SCHEMA_OPACIDADE_FORA_DOS_ESTADOS {
        let n = ph2d_app_vec::ui_state_edit::opacity_from_scene(
            &mut file.state.ui_states,
            &file.state.vec,
        );
        eprintln!("[proj] v{ver}: {n} pose(s) de estado com a opacidade da forma");
    }
    // Os itens das listas (`<= 204`) e os filtros da forma (`<= 205`): cada componente pela sua.
    if ver <= crate::project_schema::PROJECT_SCHEMA_FILTROS_SEM_ID {
        let m = ph2d_ecs::item_id::migra(&mut file.state.world, ver);
        eprintln!(
            "[proj] v{ver}: {} lista(s) de itens migrada(s), {} ilegivel(eis)",
            m.migrados, m.ilegiveis
        );
    }
    if let Some(fio) = fio {
        // As formas guardadas DENTRO de componentes (Envelope, pele) são lidas mais tarde, fora
        // deste âmbito ⇒ regravam-se agora na forma viva.
        let w = &mut file.state.world;
        let n = ph2d_app_vec::envelope_live::migra_fio(w, fio)
            + ph2d_skeleton_live::skinned_mesh::migra_fio(w, fio);
        eprintln!("[proj] v{ver}: vetor v{fio} lido; {n} envelope(s)/pele(s) regravado(s)");
    }
}
