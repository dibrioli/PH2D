# Handoff de integração — `line/sculpt3d`, **W8.7: os canais no DOCUMENTO**

> **Data:** 2026-08-04 · **Branch:** `line/sculpt3d` · **Base:** `main` de 2026-08-02
> **Commits:** `5b31e9a1d` · `1bfeb7b7a` · `1584762b2` (+ o commit de docs)
> ⚠️ **PENDENTE DE SMOKE.** Integrar não é aprovar.
> Cobre **só a W8.7**; as waves W1..W8.6 já integraram (ver `..._W4-W8_2026-08-02.md`).

## 1. A wave, numa frase

Um objeto assado (`docs/3D/02.2`, **rota A**) sobrevive a **fechar o app**, e re-acende **sem o
módulo 3D no build**. A W8.6 deu o gesto; o que faltava era a promessa que a rota A de fato faz — *a
malha some do build, o objeto continua reluminável* —, que é uma frase sobre **compilação e
persistência**, não sobre pixels.

## 2. Os números MEDIDOS

Sonda `baked_form::probe` (headless):

| lado | base f32 | base RGBA8 | forma RGBA8 | doc postcard |
|---|---|---|---|---|
| 512 | 4,00 | 1,00 | 1,00 | **2,00 MiB** |
| 1024 | 16,00 | 4,00 | 4,00 | **8,00 MiB** |
| 2048 | 64,00 | 16,00 | 16,00 | **32,00 MiB** |

```
cargo test -p ph2d-host-desktop --release --bins baked_form::probe -- --ignored --nocapture
```

Guardar a `form` como `f32` custa **4×** o disco e move o pixel aceso em **≤ 3 de 255** (média
~0,25). É também o que a indústria shipa (Unity *Secondary Textures*, `CanvasTexture` do Godot,
Sprite DLight, Spine) — e **nenhum deles guarda a malha**.

## 3. A tabela de colisão

| item | antes | depois | nota |
|---|---|---|---|
| `PROJECT_SCHEMA` | 51 | **52** | ⚠️ **PROVISÓRIO — se CONTA contra o `main` do dia** |
| tripla do pin | `(51, 13, 14)` | **`(52, 13, 14)`** | `FLIP`/`VEC_SCENE` **não** se movem |
| registro `ph2d-ecs` | 46 | **47** | `ph2d::ecs::BakedForm` |
| espelho `ph2d-render` | 47 | **48** | ⚠️ o contador é **TRÊS** |
| espelho `ph2d-script` | 47 | **48** | idem |
| `SCULPT_DOC_VERSION` | 1 | **1** | intocado |
| cenas de smoke | `1..11` | **`1..12`** | |
| contrato congelado | — | **intocado** | 4/4 e 3/3, rodados |
| ADR | — | **nenhum novo** | tudo sob o **ADR-0150** |

**Dep externa: UMA** — `serde` na `ph2d-light` (`version = "1"`, `derive`). ⚠️ No DONO e não no
shell: o doc do `LightRig` sempre disse *"o rig inteiro, como o documento o guarda"*, e uma lista de
campos re-escrita no shell seria a segunda representação do mesmo rig.
**Crates novas: nenhuma.**

## 4. Onde o código mora, e por quê

| arquivo | gateado? | assunto |
|---|---|---|
| `crates/ph2d-ecs/src/baked_form.rs` | não | a identidade `BakedForm(u32)` |
| `shells/desktop/src/baked_form.rs` | **NÃO** | *o que um objeto assado É*: canais, carimbo, acendida, codec |
| `shells/desktop/src/baked_form_planes.rs` | **NÃO** | o que o passe exige (veio de `sculpt3d_bake_planes.rs`) |
| `shells/desktop/src/project_baked_form.rs` | não | o documento: collect / restore |
| `shells/desktop/src/sculpt3d_bake.rs` | **sim** | só o **GESTO**, que precisa da malha |
| `shells/desktop/src/sculpt3d_scripts.rs` | sim | os roteiros por-cena (split de LOC) |

⚠️ **Esse corte É a wave**, e o gate `the_relight_is_not_behind_the_sculpt_feature` é o que o torna
verificável. ⚠️ **`AppGfx.baked_forms`/`baked_light`/`next_baked_form`** — o mapa saiu da
`Sculpt3dScene`: um objeto assado sobrevive à peça, e agora sobrevive ao **módulo**. ⚠️
**`CLAY_SHINE`/`CLAY_EXPONENT` mudaram para `ph2d-light`** (a única peça não-removível) com
**re-export** na `ph2d-mesh-render` ⇒ **zero churn de chamada**, o precedente da W3.

## 5. Três coisas que o integrador NÃO deve "consertar"

1. ⛔ **Não mova os canais para o blob `ProjectFile.sculpt`.** Ele já guarda as malhas e parece
   óbvio — mas o parser dele é `#[cfg(feature = "sculpt3d")]`, e um objeto assado tem de ser legível
   **sem** o módulo. Eles são campo de **sprite**, ao lado do `painted`.
2. ⛔ **Não faça o `restore_baked_forms` acender.** Ele entrega `lit_with: None` e o passe faz o
   trabalho no primeiro frame. Uma acendida na persistência é a segunda porta, e a arte **SALTARIA**
   ao reabrir — na forma mais cruel: certa enquanto o app está aberto.
3. ⛔ **Não tire o `follow_live_rig`.** Parece redundante (o objeto já tem rig) e é o que faz
   `Q/E/R/F` continuarem re-acendendo: o objeto guarda uma **cópia**, e cópia que ninguém re-autora
   congela.

## 6. Gates

**Arch-gates** (`tests/a_baked_object_outlives_the_3d_module.rs`, 5): `the_relight_is_not_behind_the_sculpt_feature`
(**o da wave**) · `the_module_that_holds_the_channels_is_unconditional` ·
`reopening_leaves_the_lighting_to_the_one_door` · `the_document_carries_the_rig_it_was_baked_with` ·
`loading_forgets_the_baked_objects_of_the_previous_document`.

**Unidade (12):** round-trip da forma · arredondamento ao mais próximo · o carimbo do rig (3) · o
documento pelo postcard **com o rig** · o reattach que não redimensiona e devolve alfa **direto** ·
`stamp_identity` que não cunha id novo · os 3 herdados de `planes`.

**4 mutações, 4 sangram**, cada uma no gate que a possui:

| mutação | |
|---|---|
| a re-acendida volta para dentro do bloco gateado (onde ela morava) | **RED** |
| o restore semeia `LightRig::default()` em vez do rig do documento | **RED** |
| o load para de esquecer os objetos do documento anterior | **RED** |
| o restore carimba o objeto como já ACESO (a segunda porta) | **RED** |

## 7. O gate de fechamento

```bash
cargo test -p ph2d-host-desktop            # 91 suítes, verdes
cargo test -p ph2d-ecs -p ph2d-light -p ph2d-mesh-render -p ph2d-render -p ph2d-script
cargo clippy -p ph2d-host-desktop -p ph2d-ecs -p ph2d-light -p ph2d-mesh-render --all-targets
cargo machete && cargo fmt --check
cargo test -p ph2d-mesh-render --release -- --ignored   # GPU: sem adapter, skip NÃO é verde
```

⚠️ **Rode a suíte do Painter em DEBUG também** — a `line/Painter` tem precedente registrado disso.

## 8. O SMOKE

```bash
env PH2D_SCULPT3D_SMOKE=12 PH2D_PROJECT_PATH=/tmp/ph2d_w87.postcard \
    cargo run -p ph2d-host-desktop --release
```

⚠️ **O `PH2D_PROJECT_PATH` não é opcional:** sem ele o save cai no CWD, e quem roda o 2º comando de
outro diretório abre um projeto vazio e reprova uma feature que funciona.

A cena imprime o que montou e o roteiro. Em resumo: **Shift+B** assa · **D** mostra o sprite ·
**Ctrl+S** (⚠️ o log tem de dizer **~8 MB**; *alguns KB* = os canais não foram gravados) · **feche o
app**, rode de novo, **Ctrl+O** — *o sprite volta **ACESO**, com a **mesma luz*** (branco = os canais
não viajaram; outro ângulo = o rig não viajou) · **`Q/E/R/F`**: as sombras têm de **ANDAR**, e é isso
que separa isto de uma fotografia · opcional, e é a promessa inteira: rode **sem**
`PH2D_SCULPT3D_SMOKE` e dê Ctrl+O — o objeto continua aceso **sem cena 3D nenhuma**.

⚠️ **A cena `=11` teve o roteiro corrigido:** a última linha dizia *"o bake NÃO sobrevive a fechar o
app"*. Era verdade ontem e passou a mentir hoje.

## 9. Aberto, com o preço ao lado

- ⚠️ **O mapa é keyed por bits de entidade, e o undo global RESPAWNA.** Depois de um Ctrl+Z que
  refaça o mundo a entrada fica órfã: a tela **não muda** (o sprite ainda aponta para o slot aceso),
  mas a lâmpada e o save deixam de a alcançar. **O `PaintedDoc` do Painter tem a MESMA forma** — o
  `restore_painted_docs` só é chamado pelo LOAD, nunca pelo undo (conferido por grep). Curar um e não
  o outro seria **duas respostas** para *como um documento reencontra o seu objeto depois de um
  respawn*. Wave própria, do dono dos dois lados.
- **merge** e **isolate** seguem sendo o que resta da W8.7 original.
- O campo assado **não carrega cor, material nem a máscara** (a W8.6 já nomeava).
- O `restore` **não valida** `base.len() == w*h*4`: um documento corrompido chega ao passe, que o
  recusa (`check()`), e o objeto fica com o slot vazio — falha alta, sem mensagem dedicada.

## 10. Ordem de integração

Toca `ph2d-ecs` (registro), `ph2d-light` (constantes + `serde`), `ph2d-mesh-render` (re-export),
`ph2d-render`/`ph2d-script` (só o número) e o shell.

⚠️ **Colisões prováveis, por risco:**

1. **`PROJECT_SCHEMA`** — o 52 é provisório; se outra linha bumpou na mesma janela, **CONTE** a
   partir do `main` do dia ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]). ⚠️ E confira
   o `project_schema_tests.rs`: em 2026-08-01 o `project.rs` **não conflitou** porque as duas linhas
   escreveram o mesmo literal, e só o conflito da tripla denunciou.
2. **O contador do registro é TRÊS**, e cada um só roda na suíte da própria crate — a família que já
   ficou vermelho-latente três vezes.
3. **`ph2d-light/Cargo.toml`** — add/add trivial se outra linha acrescentou dep ali.
