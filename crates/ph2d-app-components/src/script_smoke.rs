//! ⭐⭐⭐ **Smoke do SCRIPT DO ARTISTA** (TOP-20 #16, W4). `PH2D_SCRIPT_SMOKE=1`.
//!
//! # UM ficheiro, TRÊS objectos, três conjuntos de números
//!
//! A cena escreve **um** `bob.luau` (sobe e desce) e pendura-o em três bonecos lado a lado:
//!
//! | boneco | números PRÓPRIOS | o que se vê |
//! |---|---|---|
//! | **«Bob»** | nenhum — segue o ficheiro | sobe e desce devagar, pouco |
//! | **«Bob (tall)»** | `amplitude = 2` · e um `height = 9` que o script **não** declara | sobe e desce o dobro; o painel mostra o `height` como ÓRFÃO |
//! | **«Bob (fast)»** | `speed = 6` · `top_signal = "top"` | sobe e desce depressa e, a cada topo, **grita** `top` — e a tabela de acções dele acende e apaga a lâmpada |
//!
//! ⭐⭐⭐ **O que a cena ensina é o §0 do plano, à vista:** o mesmo script vira três comportamentos
//! pelos números de cada objecto, e o ficheiro é vivo — mudar o default da `amplitude` no editor de
//! texto e gravar muda o **«Bob»** e o **«Bob (fast)»** e deixa o **«Bob (tall)»** na dele (Q1/Q2
//! do oráculo). ⚠️ *«Próprio» é por NÚMERO, não por objecto* — o rápido tem `speed` próprio e
//! segue a `amplitude` do ficheiro.
//!
//! ⭐ **E o grito liga o script ao mundo pela porta que o levantamento prescreve** (sinais, nunca uma
//! referência directa a outro objecto): o script não sabe que a lâmpada existe.
//!
//! # ⛔ Cada boneco é um CORPO visível — a lição do #15
//!
//! O script mora num objecto que **se desenha**, porque o pick só vê quem emite uma sprite; um
//! boneco sem corpo seria impossível de escolher e a secção nunca apareceria (o report do dono sobre
//! a máquina de estados, 2026-09-15).
//!
//! # ⚠️ O ficheiro vive fora do repositório
//!
//! Em `~/.ph2d/smoke/bob.luau` — é para o dono o abrir e mexer. ⚠️ **A cena reescreve-o a cada
//! arranque**: um smoke começa sempre do mesmo sítio, e uma edição de ontem não pode fazer a cena de
//! hoje ensinar outra coisa.

use std::path::{Path, PathBuf};

use ph2d_core::Vec2;
use ph2d_ecs::{Name, SignalAction, SignalActions, SignalVerb, Transform, Visibility, World};
use ph2d_render::{Sprite, WHITE_TILE_KEY};
use ph2d_script::{LuauScript, ScriptValue};

/// ⭐⭐ **Quantas cenas este roteador serve** — contado do [`montar`] (uma, sem `match`).
pub const CENAS: u32 = 1;

/// O nome do ficheiro que a cena escreve.
pub const FILE_NAME: &str = "bob.luau";

/// ⭐ **O script da cena** — o que o dono abre no editor de texto.
///
/// ⚠️ **Os comentários são para o DONO**, em português, e dizem o que mexer.
pub const BOB_LUAU: &str = r#"-- bob.luau — sobe e desce.
--
-- Os números abaixo aparecem no painel (secção Script) de cada boneco.
-- Mude um default aqui e grave: o boneco que NÃO tem número próprio muda na hora;
-- os outros ficam com os números deles.

ph2d.property("amplitude", 1.0, { min = 0, max = 4 })  -- quanto sobe (metros)
ph2d.property("speed", 2.0, { min = 0, max = 10 })     -- quão depressa
ph2d.property("active", true)                          -- desligado = fica parado
ph2d.property("top_signal", "")                        -- o grito, a cada topo (vazio = calado)

function init(self)
  self.base_y = ph2d.get(self.id, "y")
  self.t = 0
  self.was_up = false
end

function update(self, dt)
  if not self.active then return end
  self.t = self.t + dt
  local s = math.sin(self.t * self.speed)
  ph2d.set(self.id, "y", self.base_y + s * self.amplitude)
  local up = s > 0.98
  if up and not self.was_up and self.top_signal ~= "" then
    ph2d.emit(self.top_signal)
  end
  self.was_up = up
end
"#;

const CHAO_RGBA: [f32; 4] = [0.16, 0.18, 0.22, 1.0];
const LAMPADA_RGBA: [f32; 4] = [0.98, 0.86, 0.30, 1.0];
/// As cores dos três bonecos — cada um a sua, para o dono os distinguir de longe.
const BOB_RGBA: [[f32; 4]; 3] = [
    [0.40, 0.66, 0.92, 1.0],
    [0.62, 0.48, 0.90, 1.0],
    [0.92, 0.52, 0.34, 1.0],
];

/// O tamanho de um boneco — o corpo que o dedo apanha.
pub const BOB_SIZE: [f32; 2] = [1.6, 1.6];
/// Onde os três bonecos repousam: `x` de cada um, e o `y` comum.
///
/// ⚠️ **A cena cabe no ecrã do DONO, e foi FOTOGRAFADA para o saber**: a 1.ª redacção punha os
/// bonecos a `±6 m`, o alto a subir `3×` e a lâmpada a `y = 5` — numa tela de `1080` de altura com a
/// régua do tempo aberta, a lâmpada ficava acima do visível e o alto descia para baixo da régua.
pub const BOB_X: [f32; 3] = [-4.0, 0.0, 4.0];
/// O `y` de repouso dos bonecos.
pub const BOB_Y: f32 = 0.5;
/// A amplitude PRÓPRIA do «Bob (tall)» — o dobro do default do ficheiro.
pub const TALL_AMPLITUDE: f64 = 2.0;
/// Onde a lâmpada fica: logo acima do «Bob (fast)», fora do caminho dele.
pub const LAMP_Y: f32 = 3.4;

/// **Onde a cena escreve o ficheiro** — `~/.ph2d/smoke`, ou a pasta temporária sem `HOME`.
#[must_use]
pub fn default_dir() -> PathBuf {
    std::env::var_os("HOME").map_or_else(
        || std::env::temp_dir().join("ph2d-smoke"),
        |h| PathBuf::from(h).join(".ph2d").join("smoke"),
    )
}

fn boneco(world: &mut World, nome: &str, i: usize, script: LuauScript) -> ph2d_ecs::Entity {
    world
        .spawn((
            Name::new(nome),
            Sprite::atlas(WHITE_TILE_KEY, BOB_SIZE, BOB_RGBA[i]),
            Transform::from_translation(Vec2::new(BOB_X[i], BOB_Y)),
            script,
        ))
        .id()
}

/// **O que o [`montar`] devolve** — o nível montado e quem a cena ESCOLHE ao abrir.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Montada {
    /// O nível montado (≤ [`CENAS`]).
    pub nivel: u32,
    /// ⭐ **O «Bob (tall)» abre ESCOLHIDO** — é o que tem um número próprio e um órfão, logo o que
    /// mais ensina, e com ele escolhido a secção *Script* está no painel antes do primeiro clique
    /// (o precedente é a cena de física que escolhe o corpo dela).
    pub escolhido: u64,
}

fn cena_um(world: &mut World, path: &str) -> u64 {
    // ⚠️ O chão é a PRIMEIRA raiz — desenha atrás de tudo.
    world.spawn((
        Name::new("Floor"),
        Sprite::atlas(WHITE_TILE_KEY, [22.0, 14.0], CHAO_RGBA),
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));
    world.spawn((
        Name::new("Lamp"),
        Sprite::atlas(WHITE_TILE_KEY, [2.4, 0.8], LAMPADA_RGBA),
        Visibility { hidden: true },
        Transform::from_translation(Vec2::new(BOB_X[2], LAMP_Y)),
    ));

    boneco(world, "Bob", 0, LuauScript::at(path));

    let mut alto = LuauScript::at(path);
    alto.own
        .insert("amplitude".into(), ScriptValue::Number(TALL_AMPLITUDE));
    // ⭐ **O ÓRFÃO de propósito** — um número que o script não declara (a divergência D2): o painel
    // mostra-o com `Remove`, e é assim que o dono o conhece antes de renomear uma propriedade.
    alto.own.insert("height".into(), ScriptValue::Number(9.0));
    let escolhido = boneco(world, "Bob (tall)", 1, alto).to_bits();

    let mut rapido = LuauScript::at(path);
    rapido.own.insert("speed".into(), ScriptValue::Number(6.0));
    rapido
        .own
        .insert("top_signal".into(), ScriptValue::Text("top".into()));
    let e = boneco(world, "Bob (fast)", 2, rapido);
    // ⭐ **A tabela de acções mora no MESMO boneco** — o script grita, a tabela age. Nenhum dos dois
    // sabe do outro: o sinal é o contrato.
    world.entity_mut(e).insert(SignalActions(vec![SignalAction {
        on: "top".into(),
        target: "Lamp".into(),
        verb: SignalVerb::ToggleVisibility,
        arg: String::new(),
        target_by: ph2d_ecs::SignalTarget::default(),
        from: ph2d_ecs::SignalFrom::default(),
    }]));
    escolhido
}

/// Monta a cena pedida, escreve o `bob.luau` em `dir` e devolve o nível montado.
///
/// ⚠️ **Uma cena só, logo SEM `match`** (a razão está no irmão `statemachine_smoke::montar`).
///
/// # Errors
/// A escrita do ficheiro falhou — sem ele a cena ensinaria *«o script não faz nada»*.
pub fn montar(world: &mut World, _nivel: u32, dir: &Path) -> std::io::Result<Montada> {
    std::fs::create_dir_all(dir)?;
    let path = dir.join(FILE_NAME);
    std::fs::write(&path, BOB_LUAU)?;
    let path = path.to_string_lossy().into_owned();
    let escolhido = cena_um(world, &path);
    println!(
        "[script-smoke] =1 tres bonecos correm o MESMO script ({path}) com numeros diferentes: \
         «Bob» segue o ficheiro, «Bob (tall)» sobe o dobro, «Bob (fast)» e' rapido e acende a \
         lampada a cada topo. Carregue num boneco e veja a seccao Script do painel; abra o \
         ficheiro, mude o default da amplitude, grave — o «Bob» e o «Bob (fast)» mudam, o \
         «Bob (tall)» fica com a dele"
    );
    Ok(Montada {
        nivel: 1,
        escolhido,
    })
}

#[cfg(test)]
#[path = "script_smoke_tests.rs"]
mod tests;
