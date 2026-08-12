//! Preferências de UTILIZADOR — cross-session, `~/.ph2d/prefs.txt`.
//!
//! O que é do ARTISTA (como o app se mexe, como ele se parece) contra o que é do DOCUMENTO (o que
//! o artista desenhou). Hoje: o **carácter** da UI viva (`ph2d_editor::motion::UiCharacter`) e o
//! **reduced motion**.
//!
//! ⚠️ **Isto NÃO entra nas `SavedSettings` do `ProjectFile` (v69), e a razão é o modo de falha:**
//! ali o gosto viajaria DENTRO do documento, e abrir o ficheiro de um colega mudaria como o **seu**
//! app se mexe. Escala do mundo, unidade e snaps são do documento; a velocidade com que um botão
//! acende é de quem olha para ele.
//!
//! ⚠️ **E não inventa um segundo lar.** O plano desta wave pedia `~/.config/ph2d/prefs.postcard`
//! com `PREFS_SCHEMA` próprio; a leitura do repo corrigiu-o: já existe um ficheiro de preferências
//! de utilizador nesta shell — o [`crate::palette_persist`], em `~/.ph2d/palettes.txt` — e abrir uma
//! segunda pasta ao lado seria duas casas para a mesma categoria de facto. Este módulo é irmão
//! daquele: mesma pasta, mesmo estilo (texto, std-only, best-effort, sem serde).
//!
//! ⚠️ **Sem número de schema, de propósito.** Num formato POSICIONAL (postcard) a versão é
//! obrigatória — o `ProjectFile` recusa e está certo. Num `chave=valor` a compatibilidade é grátis
//! nos dois sentidos: um build ANTIGO lê as chaves que conhece e ignora as que não conhece, que é
//! precisamente o que se quer de uma preferência. Uma versão aqui faria o build antigo **recusar** o
//! ficheiro que o novo escreveu — o trade errado para esta classe de dado.

use std::path::PathBuf;

use ph2d_editor::motion::UiCharacter;

/// O par que viaja. Os dois eixos do plano: o GOSTO e a GARANTIA.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Prefs {
    pub character: UiCharacter,
    pub reduced_motion: bool,
}

/// `~/.ph2d/prefs.txt`, ou `None` com `$HOME` por definir (a persistência é então saltada).
/// Espelha o [`crate::palette_persist`] linha a linha — mesma pasta, mesma degradação.
fn prefs_file() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(PathBuf::from(home).join(".ph2d").join("prefs.txt"))
}

/// Serializa para o formato `chave=valor`. Inverso de [`parse`].
#[must_use]
pub fn serialize(p: &Prefs) -> String {
    format!(
        "# PH2D prefs\nmotion_character={}\nreduced_motion={}\n",
        p.character.wire(),
        u8::from(p.reduced_motion),
    )
}

/// Lê o formato `chave=valor`. **Toda linha que não se entende é saltada e o campo fica no
/// default** — comentário, lixo, chave de um build mais novo, valor inválido. É esta tolerância
/// que dispensa o número de versão.
#[must_use]
pub fn parse(text: &str) -> Prefs {
    let mut p = Prefs::default();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key.trim() {
            // ⚠️ O nome vem de `UiCharacter::from_wire`, nunca de uma tabela local: uma segunda
            // tabela aqui divergiria da do escritor sem nada falhar.
            "motion_character" => {
                if let Some(c) = UiCharacter::from_wire(value.trim()) {
                    p.character = c;
                }
            }
            "reduced_motion" => p.reduced_motion = value.trim() == "1",
            _ => {}
        }
    }
    p
}

/// *Vale a pena gravar?* — a decisão do detector de mudança, extraída para ser **executável**.
///
/// ⚠️ **`None` é *ainda não observei*, e não *está tudo no default*.** A primeira observação de uma
/// sessão SEMEIA o espelho e **não** escreve: o que está em memória acabou de vir do disco, então
/// gravar seria reescrever o ficheiro sem ninguém ter tocado em nada — e, no caso em que ele estava
/// malformado, seria apagar por cima do que o artista pode querer ver e corrigir.
#[must_use]
pub fn should_save(previous: Option<Prefs>, now: Prefs) -> bool {
    previous.is_some_and(|p| p != now)
}

/// Escreve as preferências. Erro de IO é logado, **nunca fatal** — perder uma preferência é um
/// aborrecimento; recusar arrancar por causa dela seria um defeito.
pub fn save(p: &Prefs) {
    let Some(path) = prefs_file() else {
        return;
    };
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Err(e) = std::fs::write(&path, serialize(p)) {
        eprintln!("[ph2d] prefs save: {e}");
    }
}

/// Lê as preferências. **Default quando o ficheiro está ausente, ilegível ou malformado** — as
/// três degradam para o mesmo sítio, que é o app a abrir como abre hoje.
#[must_use]
pub fn load() -> Prefs {
    prefs_file()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .map(|t| parse(&t))
        .unwrap_or_default()
}

#[cfg(test)]
#[path = "prefs_tests.rs"]
mod tests;
