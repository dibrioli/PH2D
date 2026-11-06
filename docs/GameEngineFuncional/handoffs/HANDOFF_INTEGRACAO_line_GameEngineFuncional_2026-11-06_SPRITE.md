# HANDOFF de integração — `line/GameEngineFuncional`, 06/11 (GEF 23 Sprite)

Leitor: o agente integrador. HEAD `43af07339e` (37 commits sobre a base). **Base = merge-base = `c61f228b05` = ponta do `main`**: o `main` NÃO andou (`git log c61f228b05..main` vazio, conferido 06/11); `--ff-only` serve sem rebase. Ficha: [`componentes/01_sprite.md`](../componentes/01_sprite.md) · estado da onda: [continuação 05/11](HANDOFF_CONTINUACAO_2026-11-05_23_SPRITE_ONDA2.md). Anterior integrado: [04/11 (GEF 22)](HANDOFF_INTEGRACAO_line_GameEngineFuncional_2026-11-04.md).

## 1. O que entra

A **Sprite** da ficha: cartões Image / Look / Transform (dobrado) / Ordering / Advanced; **Pivot de 9 pontos** sobre o desenho visível (+ Custom X/Y); `Add ▸ Sprite` pede imagem; aviso de folha; `Keep old size` ao trocar a imagem; **Edit in Pixel / Edit in Painter** voltando a todo objecto que usa a imagem (com Undo); a imagem editada herda o nome (só na sessão, o `.ph2dproj` não grava nomes).

Também entra, fora da Sprite:
- **Bug do dono curado** (`521a883293`): dois Transform presos não eram independentes. `commit_transform_edit` escreve só os campos da mão; `abre_ensaio`/`fecha_ensaio` guardam todos os números do preso (NaN = «não escrito»). Prova: `o_bloco7_duas_copias.rs` (mutada nas duas pontas).
- **Alfinete em todo cartão de objecto** (só Empty sem), censo por gesto real (`c6d83ea30b`).
- **Transform do Vector em Draw** = o mesmo cartão do Object, **hóspede reordenável** da lista do Vector (`card_guest_ops.rs`), mais um cartão Shape (Size, Resize Box, Set Center).
- **Pares de eixos lado a lado** em qualquer largura (`par_de_eixos`, `66907c388f`).
- **Roda**: do painel por cima de superfície e de sub-região (`dona_do_ponto`, `remove_graph_canvas`, sub-região com painel dono).
- **Colisão no Pivot**: o Pivot não separa desenho e colisão (`Collider.offset` compensa, a chave do Pivot o leva, os filhos ficam); as sondas do Player partem do centro do collider (`origem_e_centro`, `player_probes.rs`). Prova: `com_o_pivo_no_pe_o_boneco_pisa_e_salta_igual`.

## 2. Superfície de colisão (foundational) — e por que é aditiva

`git diff --stat c61f228b05..HEAD`: **156 files changed, 8936 insertions(+), 781 deletions(-)**.

| Onde | O que | Aditivo porque |
|---|---|---|
| `ph2d-editor-core` | `interaction/state/card_guest_ops.rs` (novo, 266), `card_ensaio_ops.rs` (+NaN «não escrito»), `undo_route.rs` (novo, 184: a política do Ctrl+Z desceu da shell), `interaction/dispatch/scroll.rs` (roda), `image_use_edits.rs` (`SpritePivot` novo; `ImageUseEdit` ganha `EditInPixel`, `EditInPainter`, `KeepSize`, `Pivot`), `panel/cards_*.rs` (lista, rótulos, verbos, cartão), `paint_rounded.rs`, `screens/hero/panel_walk.rs`, 1 teste de arquitetura (`every_editable_field_is_a_channel`) | arquivos novos + variantes novas num enum de edição interno (não é §6); a compilação da workspace prova os `match` |
| `ph2d-physics-ecs` | `bridge/player.rs`, `player_marks.rs`, `player_probes.rs` (+15): a sonda parte do centro do collider | com `Collider.offset == 0` o centro é a origem: byte-igual ao de antes |
| `ph2d-render` | `sprite_emissive.rs` (novo, 229) + `pub mod` em `lib.rs`; **a shell perde** `render_loop/sprite_emissive.rs` (-233) | movimento shell → crate; a cura da catraca |
| `ph2d-timeline` | `channel_group.rs` (+47) e `lib.rs` | grupo de canal novo (chave do Pivot); não toca `TimelineDoc` nem versão |
| `ph2d-tool-painter` | `tool/persist.rs` (+25) | só leitura/escrita da volta do Edit in Painter |
| `ph2d-i18n` | `shell.rs`, `cartoes.rs` | chaves novas |
| `ph2d-panel-*`, `ph2d-app-*` | inspector, vector, registry-init, motion-graph, asset-browser; app-game, app-pixel, app-sprite-anim, app-components, app-vec | consumidores |

**Crates novas** (folhas, nasceram da catraca `the_shell_only_shrinks_and_every_app_crate_too`): `ph2d-sprite-edit` (Pivot e ida e volta da imagem; dependência nova de `shells/desktop`, `shells/game-scenes`), `ph2d-pixel-formas` (era `ph2d-app-pixel/src/formas.rs`, renomeado como `lib.rs`), `ph2d-game-dicas` (era `dicas.rs` da `ph2d-app-game`).

**Atenção do integrador:**
- `Cargo.lock`: **+36 linhas** (as três crates). Conflito é reconta com `cargo metadata`, nunca à mão.
- `shells/desktop/Cargo.toml`: +1 linha (`ph2d-sprite-edit = { path = "../../crates/ph2d-sprite-edit" }`).
- `the_shell_only_shrinks`: ver §4; o tecto soma entre linhas.
- `undo_route.rs` existe agora na shell (-192) **e** no `ph2d-editor-core` (novo): se outra linha tocou `shells/desktop/src/undo_route.rs`, o conflito é de movimento — o corpo vive na crate.

## 3. Contratos e schema

Refeito em 06/11 (no diff `c61f228b05..HEAD`):

```
git diff -U0 c61f228b05..HEAD -- '*.rs' '*.toml' | grep -cE '^[+-].*PROJECT_SCHEMA'          # 0
git diff -U0 c61f228b05..HEAD | grep -cE '^[+-].*enum (PanelEvent|NodeOp|VectorOp)'          # 0
git diff -U0 c61f228b05..HEAD -- '*.rs' | grep -E '^[+-].*\bregister_[a-z_]+\('              # só chamadas
```

- **`PROJECT_SCHEMA`: 0 linhas.** Escada e tripla intactas; nada a recontar.
- **§6:** nenhum trait/enum congelado alterado (`NodeOp`, `Tool`, `PanelEvent`, `VectorOp`); `PanelEvent` aparece só como USO (`SetValue`, `Click`).
- **`register_*`:** nenhum registrador novo; o grep acha `register_if_absent(` (chamada) e `register_all_panels()` (3 testes). Registo de painel/tool/node: nada.
- Sem `NodeId`, sem variante de `Tool`. O formato `.ph2dproj` não muda (por isso o nome herdado não viaja).

## 4. Contadores (DELTA e medidos)

Medidos com a MESMA regra do gate (`conta`: diretório da crate inteiro, `src` + `tests`, `.rs`, `lines().count()`, salta `target/`). ⚠️ O `wc` só sobre `src` dá menos (game 2 705 × 2 728 é o do gate; pixel `src` 11 082 contra 13 322 do gate).

| Catraca | Medido | Tecto | Folga |
|---|---|---|---|
| shell (`shells/desktop`) | 188 990 | 189 041 | **51** |
| `ph2d-app-game` | 2 705 | 2 728 | 23 |
| `ph2d-app-pixel` | 13 322 | 13 424 | 102 |
| `ph2d-app-sprite-anim` | 8 330 | 8 352 | 22 |
| `ph2d-app-components` | 80 859 | 80 869 | **10** |
| `ph2d-app-vec` | 93 635 | 93 637 | **2** |

⚠️ **`ph2d-app-vec` (2) e `ph2d-app-components` (10) estão com quase nenhuma folga, e a folga SOMA entre linhas**: qualquer outra linha que cresça uma dessas crates reprova na árvore combinada. A cura é MOVER para uma crate da família, nunca subir o número. Os tetos desta linha não foram subidos (conferido: o diff do gate não mexe em `TETO_LOC` nem nas linhas das crates). Todo `ph2d-app-*` novo precisa de linha em `TETOS_DAS_CRATES_APP`; as três crates novas não são `ph2d-app-*`.

Testes (por nome, nesta linha): `ph2d-sprite-edit` 31 · `ph2d-app-sprite-anim` 83 · `ph2d-panel-inspector` 623 · `ph2d-panel-vector` 529 (+3) · `ph2d-panel-registry-init --test it` 413 (+38) · censos 12 de 12 (132). Binário: `Finished smoke profile ... in 0.19s` no 2.º build. `agent-loop-profile`: 4 de 6 no alvo (`test:check` 6,2×; contexto 305 mil).

## 5. O que um merge pode partir

1. **Catraca da shell/app-vec/components** (§4): árvore combinada com outra linha da shell ⇒ recontar.
2. **Censo do Inspector por gesto** (`c6d83ea30b`): cartão novo de outra linha entra na população do censo do alfinete — se a linha nova não tem alfinete, o censo reprova (e deve).
3. **Altura de abertura do Inspector** e testes de largura: o Look abre só nos cartões e o Pivot muda a altura; a colisão de 04/11 (`1 362` × `1 507`) mostra que o número certo sai da árvore combinada.
4. **Controle da máscara da câmera** (`0b61e98d4a`): conta só a câmera; Flip e Pivot entraram na tabela.
5. **Gates que varrem `shells/desktop/src/render_loop/`**: `sprite_emissive.rs` saiu de lá (mover código parte censos por prefixo — HOWTO §2).
6. **`ImageUseEdit` com 4 variantes novas**: outra linha com `match` exaustivo sobre ele não compila até ganhar os braços.
7. **Cenas de smoke**: nenhuma cena nova numerada; `PH2D_GAME_SMOKE=4` ganhou fotos `sprite_*` e a «mão» automática (`smoke_mao.rs`). `PH2D_VEC_CARTOES_SMOKE=13` ganhou o Transform preso.

## 6. Prova de fecho

**Suíte completa e `nextest-impacted` NÃO correram** (dono 23/10: só o integrador). `arch_safe_clamp_only` não existe na árvore: o pedido do comando de fecho está velho. Rodaram por nome os testes das crates tocadas (§4), os 12 censos, mutações e a auditoria abaixo.

### Auditoria de fecho (2 lentes, leitura) — o que curou (`82fb9beb6b`, `834efaacd5`)

1. objecto de trabalho no undo · 2. load com edição aberta · 3. moldura solta · 4. edição morta pelo documento do Painter · 5. origem morta · 6. Inspector fechado (Transform com o Inspector fechado) · 7. marca do Painter · 8. grelha/Region · 9. eixo do Pivot (X/Y) · 10. cantos por objecto · 11. comentário · 12. literal do censo. Depois: `adc1a9e6c6` (tabela dos achatados para a folha; política do Ctrl+Z para o núcleo), `cec1e0a06f` (import morto), `43af07339e` (o portão do Show em voo lê os motivos onde moram).

⚠️ **Decisão 4 PENDE aprovação do dono:** quando o documento do Painter morre com a edição aberta, **os píxeis de fora vencem**, as camadas saem com a mensagem «Layers flattened», e o Undo devolve.

Verificado à parte: Inspector vazio depois do Ctrl+Z é lei antiga (`undo_app.rs:214`, `undo_selection.rs:72`); o ícone de Painted vem de `document_entities` e já aparece ao entrar no Draw; gravar e reabrir depois de desfazer dá **0 píxeis de diferença** (tabela 0/0/0; controle 1977).

### Mutações
Provadas: as duas pontas do Transform preso (sem o ensaio o outro cartão lia 100 × 900); a cena do quadro (`162dd0690e`: mutação sem colar a cena sangra). ⚠️ **Sobreviventes M8 e M9** (hóspede reordenável a meio da mola): continuam SEM prova — só o movimento; abertas.

## 7. Smoke (do Enio) — o que foi e o que não foi visto

O dono **aprovou as rodadas até `66907c388f`**; a rodada de **colisão e nomes** (`951b1fe11e`, `4af1522162`, `032b1fa4c8`, `891dcf8391`) está **por confirmar por ele**. ⚠️ **As curas da auditoria (`82fb9beb6b` em diante) NÃO tiveram smoke do dono.**

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional && cargo build --profile smoke -p ph2d-host-desktop && PH2D_GAME_SMOKE=4 ./target/smoke/ph2d-host-desktop`
2. Clique numa plataforma; no Inspector, abra o cartão **Look**: ali estão Tint, Opacity, Flip e o **Pivot** (9 pontos). Troque o pivô para o pé; o desenho e a colisão ficam parados.
3. No cartão **Image**, clique **Pixel**, pinte, saia (`Tab`): as duas plataformas com a mesma imagem mudam; Ctrl+Z fora do Draw volta as duas. Mesmo com **Painter**.
4. Deu errado se: a colisão anda com o pivô, só uma plataforma muda, o nome da imagem vira «… Low», ou algum botão fica cortado com o painel no mínimo.
5. Vetor: `PH2D_VEC_CARTOES_SMOKE=13 ./target/smoke/ph2d-host-desktop` (mesmo `cd`): o Transform em Draw é o do Object e pode ser arrastado na lista.

Fotos: `PH2D_GAME_SMOKE_FOTO=sprite_<nome>` (`sprite_advanced`, `sprite_custom`, `sprite_no_painter`, `sprite_no_pixel`, `sprite_painter_aplica`, `sprite_painter_sem_pintar`, `sprite_painter_volta`, `sprite_pivo_pe`, `sprite_pixel_desfaz`, `sprite_pixel_sem_pintar`, `sprite_pixel_volta`, `sprite_recibo`, `sprite_repetida`, `sprite_uso`, `sprite_vazio`) com `docs/Components/ferramentas/fotografa_cena.sh`; mão automática em `crates/ph2d-app-game/src/smoke_mao.rs` (imprime `[sprite-pixel]`).

## 8. ABERTO

Ordem do dono para o próximo dia (**não entra nesta integração, salvo ele mandar**):
1. **Título do cartão dobrado embolado com o resumo em painel estreito** (captura `Transform` com `850, 200 · 0° · 1 × 1` em cima do título).
2. **Espelhar:** pesquisar Godot e Unity e escolher; regra dele: se houver **dois modos úteis diferentes, os 4 botões ficam juntos no Look**. Hoje Transform e Flip diferem (medido, decisão 14 da ficha) e os botões do Transform ficam.
3. Roda «nem sempre» sem causa conhecida (curados: grafo fechado e sub-região; posição refutada na fonte do winit 0.30.13); falta relato do dono ou medição em Wayland nativo.
4. Não provados: **Undo e Auto Key do Transform em Draw** e **chave do Pivot tocada na timeline real** (filhos não entram na compensação nem na chave); M8/M9 (§6); degrau estreito sem foto.
5. O nome da imagem editada só vale na sessão (o formato não o grava).
6. Cartão alto troca de lugar com pouco arrasto (`slot_index` pelo centro): comportamento antigo de todos.

## ⛔ Recusas MEDIDAS (não reconstruir)

| Recusado | Por quê |
|---|---|
| Botão **Replace** | o dono: clicar na miniatura troca (hover com borda Accent e `RotateCw`) |
| Tira de prévia do **Repeated** | o dono; o tamanho vai entre parênteses |
| Botões com **dois ícones** | o dono; sempre `[Export] Pixel` · `[Export] Painter` |
| Lista **flutuante separada** para o hóspede | o Transform é o 1.º cartão DENTRO do painel (`1b630c7c33`) |
| `set_collapsed_if_unchosen` do Look em `gavetas`/`populate` | medido contra o painel de antes; o Look abre só nos cartões, em `shells/desktop/src/init.rs` antes do `load_sections` |
| Mover a colisão com o pivô | decisão 13 da ficha: a colisão fica parada com o desenho |
