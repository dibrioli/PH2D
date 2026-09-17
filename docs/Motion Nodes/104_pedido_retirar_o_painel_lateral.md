# PEDIDO À `line/motion-value`: RETIRAR o painel lateral de params

> **Origem:** ordem do dono no smoke de 2026-09-17 — *«não temos mais o painel da direita.
> estamos retirando ele»*. Escrito pela `line/UIUX`, que tropeçou nele ao migrar o texto dos
> motores para o HR-15 e **não é a dona desta superfície**.
>
> **Estado:** pedido, não trabalho começado. Nenhuma linha deste documento foi executada.

---

## §1 — ⛔⛔⛔ LEIA ISTO ANTES DE APAGAR UMA LINHA: a crate é a CAIXA DE CORREIO DO CARTÃO

O `CLAUDE.md` §5.1 já avisa que *«o `publish` do painel é quem DRENA as intenções do CARTÃO —
saltá-lo pararia o cartão inteiro»*. **Isso está medido e é pior do que a frase sugere:** não é só
o `publish`; é o **tipo** e a **fila** que vivem dentro da crate do painel.

```
crates/ph2d-panel-motion-params/src/snapshot_channel.rs        ← 85 linhas
    pub enum MotionParamIntent { SetParam · SetTextParam · ResetParam · PickFile }
    pub fn push_param_intent(…)        ← quem ESCREVE
    pub fn drain_param_intents() -> …  ← quem LÊ
```

E quem escreve nela, **fora da crate do painel**, é o CARTÃO:

| Ficheiro | Sítios | O que empurra |
|---|---:|---|
| `ph2d-app-motion/src/motion_bridge_intents.rs` | **3** | os gestos do cartão (`SetParam`, `SetTextParam`, `PickFile`) |
| `ph2d-app-motion/src/motion_bridge_choices.rs` | **2** | a escolha feita na lista do cartão |

⇒ **A remoção não é «apagar 7 558 linhas». É MOVER 85 e depois apagar 7 473.** Quem apagar a
crate primeiro fica com o cartão mudo — e o modo de falha é o bom (erro de compilação nos 5
sítios), mas só porque o tipo viaja com ela; se alguém “curar” isso duplicando o enum, o cartão
compila e as edições dele caem num segundo balde que ninguém drena.

## §2 — O estado MEDIDO (2026-09-17), que não é o que a frase do dono sugere

| Facto | Valor |
|---|---|
| o painel está **desligado, não apagado** | `motion_bridge_surfaces::painel_lateral()`, lê `PH2D_MOTION_PANEL` — desde 07/09 |
| crate | `34` ficheiros · `7 558` linhas |
| dependentes declarados | `3` — `ph2d-app-motion` · `ph2d-panel-registry-init` · (ela própria) |
| ficheiros do `ph2d-app-motion` que a tocam | `36`, dos quais **`20` são de teste** |
| testes que morrem com a crate | **`65`** |
| gates de OUTRAS crates que a nomeiam | **`0`** |
| registo no menu *Window* | `ph2d-panel-registry-init/src/lib.rs:77` |

⚠️ **`ph2d-param-editors` NÃO depende dela** (o doc-comment dele cita `MotionParamIntent` por
prosa, e o `Cargo.toml` não a declara) — ele fala com o cartão por outra porta. *Uma citação num
comentário lê-se como uma dependência numa tabela de risco; esta não é.*

## §3 — A ordem que a medição prescreve

1. **Mover a caixa de correio para fora da crate do painel.** Ela é `85` linhas com zero
   dependências de UI — uma crate-folha ou um módulo do `ph2d-app-motion` servem. ⛔ Enquanto ela
   estiver lá dentro, todo o resto é impossível.
2. **Re-apontar os 5 sítios que empurram** (`motion_bridge_intents.rs` ×3,
   `motion_bridge_choices.rs` ×2) e o único que drena (`motion_bridge_params_edit.rs:89`).
3. **`publish` perde o desvio**, não a primeira metade: o `apply_param_edits` corre **antes** do
   `if !painel_lateral() { return; }` e é load-bearing (o próprio ficheiro o diz em maiúsculas).
4. Tirar o registo do menu *Window* e a linha do `ph2d-panel-registry-init`.
5. Apagar a crate, o `painel_lateral()` e a env `PH2D_MOTION_PANEL`.

## §4 — ⚠️ O que esta obra parte, e o que fica MUDO

- **Falha alto:** os 5 push + o 1 drain (erro de compilação, porque o tipo viaja com a crate).
- ⛔ **Fica MUDO:** os **`65`** testes da crate desaparecem com ela. Alguns deles são os únicos
  gates de propriedades que o cartão TAMBÉM tem — antes de apagar, corra
  `cargo nextest list -p ph2d-panel-motion-params` e pergunte, um a um, *«esta propriedade tem
  gate noutro sítio?»*. Um teste que se apaga junto com o seu sujeito é correcto; um que era a
  única testemunha de uma lei que SOBREVIVE é dívida silenciosa.
- ⚠️ **E a `line/UIUX` acabou de pôr lá dentro** um gate de tinta
  (`lib_enum_option_ink_tests.rs`) e **13 sítios de tradução** nos `motion_bridge_params*.rs`.
  ⭐ **Nada a desfazer antes:** os dois viajam com o que for apagado. A lei do HR-15 que interessa
  ao cartão está gateada **noutros dois sítios** que sobrevivem —
  `paint_card_params_tests::an_enum_row_shows_the_word_and_never_the_key` e
  `snapshot_choices_tests::a_static_choice_list_opens_with_words`, ambos em
  `ph2d-panel-motion-graph`.

## §5 — O que NÃO está decidido

⛔ **Se a coluna da direita INTEIRA sai, ou só este painel.** A frase do dono foi *«o painel da
direita»*, e com o lateral de params desligado quem ocupa aquela coluna é o **Inspector**
(`CLAUDE.md` §5.1: *«o Inspector volta à coluna da direita»*). ⇒ **pergunte-lhe qual das duas**
antes de começar: são duas obras de tamanhos muito diferentes, e uma delas não é do módulo Motion.
