//! Preferências de UTILIZADOR — cross-session, `~/.ph2d/prefs.txt`: o que é do ARTISTA e da
//! máquina dele, nunca do documento (abrir o ficheiro de um colega não muda o SEU app). Irmão do
//! [`crate::palette_persist`] (mesma pasta, texto, std-only, best-effort). ⚠️ Sem número de schema:
//! em `chave=valor` um build antigo ignora as chaves que não conhece, que é o que se quer aqui.

use std::path::PathBuf;

use ph2d_app_input_map::View;
use ph2d_editor_core::motion::UiCharacter;

/// O par que viaja. Os dois eixos do plano: o GOSTO e a GARANTIA.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Prefs {
    pub character: UiCharacter,
    pub reduced_motion: bool,
    /// **O SOM DE UI** (UI viva, D1). ⛔ Nasce DESLIGADO: som que ninguém pediu leva a feature.
    pub ui_sound: bool,
    /// ⭐ A fonte, o peso e o tamanho do texto da interface (2026-10-01).
    pub text: ph2d_tokens::UiTextStyle,
    /// ⭐ A escala da interface inteira (2026-10-02) — do artista, pela mesma razão.
    pub ui_scale: ph2d_tokens::UiScale,
    /// F5: **canhoto** (*Look ▸ Left-handed*) — da MÃO do artista, não do projecto.
    pub left_handed: bool,
    /// ⭐ A nitidez da luz 2D (Settings ▸ Light) — do artista e da máquina dele, nunca do projeto.
    pub luz2d: ph2d_editor_core::luz_edits::PreferenciaDaLuz,
    /// O intervalo da cópia de recuperação (`recovery.minutes`, O15) — do artista e da máquina dele.
    pub recovery: ph2d_recovery::Period,
    /// A última vista da aba Input Map (`input_map_view`); a de omissão (nunca escolhida) é o Keyboard.
    pub input_map_view: View,
    /// O cartão Performance do Painter (n.º 8): o palpite do Wet — da máquina do artista.
    pub wet_guess: bool,
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
        "# PH2D prefs\nmotion_character={}\nreduced_motion={}\nui_sound={}\nui_font={}\n\
         ui_font_weight={}\nui_text_size={}\nui_scale={}\nleft_handed={}\nlight_quality={}\nlight_auto={}\nrecovery.minutes={}\n\
         input_map_view={}\nwet_guess={}\n",
        p.character.wire(),
        u8::from(p.reduced_motion),
        u8::from(p.ui_sound),
        p.text.font.wire(),
        p.text.weight.wire(),
        p.text.size.wire(),
        p.ui_scale.wire(),
        u8::from(p.left_handed),
        p.luz2d.nivel.token(),
        u8::from(p.luz2d.automatica),
        p.recovery.to_minutes(),
        p.input_map_view.pref_key(),
        u8::from(p.wet_guess),
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
            "ui_sound" => p.ui_sound = value.trim() == "1",
            "recovery.minutes" => p.recovery = ph2d_recovery::Period::from_minutes(value),
            "left_handed" => p.left_handed = value.trim() == "1",
            "wet_guess" => p.wet_guess = value.trim() == "1",
            // ⚠️ Os nomes vêm de `from_wire` dos próprios tipos, pela razão do carácter acima.
            "ui_font" => {
                if let Some(f) = ph2d_tokens::UiFont::from_wire(value.trim()) {
                    p.text.font = f;
                }
            }
            "ui_font_weight" => {
                if let Some(w) = ph2d_tokens::UiWeight::from_wire(value.trim()) {
                    p.text.weight = w;
                }
            }
            "ui_text_size" => {
                if let Some(z) = ph2d_tokens::UiTextSize::from_wire(value.trim()) {
                    p.text.size = z;
                }
            }
            "light_quality" => {
                if let Some(n) = ph2d_editor_core::luz_edits::NivelDaLuz::de_token(value.trim()) {
                    p.luz2d.nivel = n;
                }
            }
            "light_auto" => p.luz2d.automatica = value.trim() == "1",
            "input_map_view" => p.input_map_view = View::from_pref_key(value).unwrap_or_default(),
            "ui_scale" => {
                if let Some(z) = ph2d_tokens::UiScale::from_wire(value.trim()) {
                    p.ui_scale = z;
                }
            }
            _ => {}
        }
    }
    p
}

/// *Vale a pena gravar?* — a decisão do detector de mudança, extraída para ser **executável**.
///
/// ⚠️ `None` é *ainda não observei*: a primeira observação SEMEIA o espelho e não escreve (o que
/// está em memória acabou de vir do disco; malformado, apagaria o que o artista quer corrigir).
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
