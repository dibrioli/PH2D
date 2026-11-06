# HANDOFF DE INTEGRAÇÃO — linha Pixel Art (P8d + P9: Export e Import) — 2026-11-05

> Leitor: o agente integrador. A linha **fecha e PARA**: não integra, não faz push. Plano: [P9_plano.md](../P9_plano.md).
> Continuação (mecanismo por onda): [P8d](HANDOFF_CONTINUACAO_line_PixelArt_2026-11-05_P8d.md) ·
> [P9a](HANDOFF_CONTINUACAO_line_PixelArt_2026-11-05_P9a.md) · [P9b](HANDOFF_CONTINUACAO_line_PixelArt_2026-11-05_P9b.md).
> Molde/antecessor: [P8](HANDOFF_INTEGRACAO_line_PixelArt_2026-11-02_P8.md) (a P7+P8 vão no mesmo ramo).

## 1. Identidade

- Branch `line/PixelArt`; worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PixelArt`.
- Merge-base `c61f228b05`; HEAD `0c39ec3442`. `git diff --stat c61f228b05..HEAD`: **20 commits, 171 ficheiros, +15 198 / −959**.
- Ondas: P8d (`4f4a72456a`..`19c36ab8b8`) · plano P9 (`b87f026c2a`) · P9a (`9315def31b`..`0eeb2dce0c`) · P9b (`62bfd1fca3`..`0c39ec3442`).
- **Smoke do dono: `Smoke OK` em 05/11** — cena 20 inteira e a janela do Import; cena 19 aprovada antes. Integrar não é aprovar.
- Sem rebase sobre o `main` (ordem: não integrar); a árvore combinada é do integrador.

## 2. O que entrega

- **P8d**: filtros ao vivo com Apply/Cancel, giro que volta ao original, as 12 fontes do Text; os ajustes redesenham a tela em tempo real; um Pixel Sprite traz o painel dele.
- **P9a (motor)**: `.ase` lê e grava com o documento inteiro (conferido no LibreSprite); `ph2d-pixel-io` (render, folha, JSON, GIF, APNG, WebP, PNG indexado, abrir); lei do `edges-find` medida no Trial; pendências da P8 fechadas.
- **P9b (tela)**: menu File (Open, Import Sheet, Export As, Export Sprite Sheet, Repeat Last Export) + atalhos; janelas em cartões com prévia ao vivo (crate NOVA `ph2d-panel-pixel-export`); File > Import abre janela Pixel Sprite × Image (Painter); folha importada traz tempos e tags do JSON; `.ase` arrastado fica nítido (filtro Nearest); cena de smoke 20.

## 3. Superfície de colisão (aditiva; confira por `git diff --stat c61f228b05..HEAD -- <crate>`)

| Onde | O quê | Por que é aditivo |
|---|---|---|
| `ph2d-editor-core/src/ids/chrome/pixel.rs` | `PIXEL_EXPORT_PANEL` + 5 `CTX_MENU_PIXEL_*` (Open, Import Sheet, Export As, Export Sheet, Repeat Export) | ids novos, append-only; `pixel_filter.rs` +19/−? só ids de filtro |
| `pixel_panel/mod.rs` | `PixelRequest::File(PixelFileAction)`, `enum PixelFileAction`, campos novos em `PixelDialogs` (`file_request`, ficheiros do Import à espera) | variante/enum novos. `PixelDialogs` **PERDEU `Copy`** (carrega `Vec`): nenhum leitor dependia da cópia (compilou sem tocar em chamadores) |
| `interaction/state/{pixel_edit_ops,store_hierarchy}.rs`, `section_prefs_ops/tema_custom.rs` | portas de menu: `WidgetStore::pixel_file_action(id)`, `pixel_file_menu` | métodos novos; a porta pela `WidgetStore` cura o ciclo ids↔pixel_panel |
| `interaction/dispatch/pointer_down_menus.rs` | roteia o clique nas linhas `CTX_MENU_PIXEL_*` | +4/−? ramos novos |
| `screens/hero/{menu_tables,panel_ids,paint,pre_populate}.rs`, `chrome/io_menu.rs` | linhas do File, painel, pintura, registo de hit | tabelas append-only |
| `panel/cards_ferramenta.rs` | `pinta_lista_curta` nova; `pinta_lista` com o mesmo comportamento para quem já a chamava | função nova, a antiga delega |
| `ph2d-i18n` | `pixel_export.rs` NOVO (179 l), `chrome_menus.rs`, `lib.rs` (+2), `pixel.rs`, `pixel_dialogos.rs` | chaves novas |
| `ph2d-app-host/src/modal.rs` | `pick_files` respeita `responde_o_proximo` (como já faziam os outros diálogos) | só muda com resposta de smoke armada |
| `ph2d-panel-registry-init` | `Cargo.toml` (feature `panel-pixel-export`, deps `ph2d-pixel-core`/`-io`), `lib.rs` +2, testes | feature nova |
| `shells/desktop` | `Cargo.toml` (+2: feature), `fase_pixel_panel.rs` +1 linha, `image_edit_import.rs`, `ase_import.rs` (filtro `Nearest`), `ase_import_tests.rs` | |
| `ph2d-aseprite`, `ph2d-sprite-sheet/src/aseprite.rs`, `ph2d-imageio-{png,gif,apng,webp}` | leitor/gravador `.ase`, codecs | |
| Crate NOVA `crates/ph2d-panel-pixel-export` | glob `crates/*` | `Cargo.lock` **+1 pacote** (36 linhas no total): quando colidir, REGENERE |
| `ph2d-pixel-text/fonts/*` | 12 fontes + licenças | binários, sem colisão textual |

Colisões prováveis: `menu_tables.rs`, `pre_populate.rs`, `ph2d-i18n/src/lib.rs`, `registry-init` (Cargo + lib), `shells/desktop/Cargo.toml`, `Cargo.lock`, `hero/paint.rs`, e o tecto de `ph2d-app-pixel` em `architecture_the_shell_only_shrinks.rs`.

## 4. Contratos e schemas: NÃO tocados

`git diff --name-only c61f228b05..HEAD` (171 ficheiros) não lista nenhum ficheiro de `PROJECT_SCHEMA`, nem `ph2d-nodegraph`, nem `ph2d-tool-*`, nem `ph2d-vec-*`; os gates `architecture_contract_surface`, `architecture_tool_contract_surface` e `architecture_vector_contract_surface` não têm superfície aqui. Contratos §6 do CLAUDE.md intactos.

- **Formato gravado**: blob `PH2DPIXEL` foi de **MINOR 2 → 3** (`ph2d-pixel-core/src/codec.rs`): campo 11 opcional = «último export». Sem export o registo fica idêntico ao MINOR 2. Binário antigo ignora o campo e, ao regravar, perde só o «último export».

## 5. Números e tectos (DELTA)

| Item | Valor |
|---|---|
| Tecto `ph2d-app-pixel` (`architecture_the_shell_only_shrinks.rs`) | 13 424 → **11 415** (desceu; a linha mede 11 413, **folga de 2**). ⚠️ SOMA entre linhas: o integrador pode ter de recontar com `python3 scripts/recontar.py`, nunca subir à mão |
| M10 `montar` 1024²×256 | 2040 / 2026 / 1994 ms (nada / trim / merge); perfil smoke, mínimo de 3, load ~7 (relógio indicativo). Daí a prévia e a gravação irem em `Job` |
| Binário smoke | 2.ª build `Finished smoke in 0.37s`, zero `Compiling` (mexer só em `.md` não o invalida) |
| `Cargo.lock` | +1 pacote |

## 6. Prova de fecho (por nome; a suíte completa é do integrador)

- **Mutações**: 17 + variantes; 15 sangraram à primeira. As 2 sobreviventes (tag pelo nome em `de_quadros`; `esquece` sem limpar `trabalho`) ganharam `o_repeat_acha_a_tag_pelo_nome` e `esquecer_a_previa_larga_a_folha_a_montar`, vistos vermelhos sob a mutação.
- **Oráculo**: `o_godot_recorta_igual` (`--ignored`, Godot 4.7.2) corrido; LibreSprite lê o nosso `.ase`; fixtures `oracle_export.json`, `oracle_ase_nosso.json`.
- **Gate novo por ponteiro real**: `the_pixel_export_answers_a_real_pointer` (`ph2d-panel-registry-init`).
- **Verdes por nome**: clippy `-D warnings` em 19 pacotes · `ph2d-panel-registry-init` inteira 385 ok (antes dos últimos consertos; depois, por nome) · `ph2d-pixel-core` 176 · `ph2d-pixel-io` inteira · `ph2d-host-pixel-scenes` 253/254, a falha curada depois (`smoke_1_stages…` agora nível 21) e `exportar::` 19 ok · `file_loc_caps` 4 · `architecture_` da editor-core após curar o ciclo ids↔pixel_panel.
- ⚠️ `arch_safe_clamp_only` não existe na árvore: o filtro casa 0.
- ⚠️ **NÃO correram**: `nextest-impacted`, a suíte completa e `censos-da-arvore-combinada.sh` (regra do dono 23/10: são da integração).

## 7. Auditoria de 2 lentes (dados/formato · costura/concorrência) — corrigido

Prévia atrasada sobre o Import · nome com quebra de linha reescrevendo o preset · JSON de outra folha (`meta.size`) · escrita por `.tmp` + rename · cel corrompido vira `Aviso::CelCorrompido` · preset lembrado ao começar a gravar (cura a chave reciclada e o Repeat apagado).

**Deixado de propósito, com a razão**: sequência mais curta não apaga os PNG antigos (apagar ficheiro do artista seria pior) · PNG e JSON da folha não são um par atómico (cada um é atómico sozinho) · `Ctrl+I` com outra janela aberta troca a janela · `.ase` com nome acima de 65 535 bytes · clone do sprite na linha da tela não medido · parser `ph2d-aseprite/src/file.rs` e overflow de `celulas` com gap enorme NÃO auditados · Open/Import Sheet só em Draw é decisão do dono. Nitidez do `.ase` arrastado coberta pelo `Smoke OK` do dono.

## 8. Smoke do dono (já feito: `Smoke OK`)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PixelArt && PH2D_PIXEL_SMOKE=20 ./target/smoke/ph2d-host-desktop`
2. Menu File: Export As, Export Sprite Sheet, Import (janela com Pixel Sprite × Image), Repeat Last Export.
3. Cena 19 (filtros): `PH2D_PIXEL_SMOKE=19`. O aviso do filtro Edges fica.

## 9. Decisões do dono

File > Import abre janela Pixel Sprite × Image (Painter); arrastar não pergunta (a pergunta sobre mudar isso segue sem resposta); LibreSprite é referência, o Trial só abre e fotografa.

## 10. ABERTO

- `curve_a11y`/`range_a11y`: lacuna do app (ver P9a).
- P10 a seguir em [02_plano_modulo.md](../02_plano_modulo.md).
- Os «deixado de propósito» da §7.

## Abertos do dono (05/11, para o dia seguinte; não corrigidos, causa não investigada)

1. «O color picker do seletor de cor não funciona para Pixel.» Reproduzir antes de assumir qual controlo é: provavelmente o conta-gotas do seletor partilhado.
2. «Depois de entrar e sair de Pixel, o zoom do canvas fica limitado até escolher outro objeto.» Suspeita a conferir, não medida: estado do Draw que sobrevive à saída (`PixelApp::fit`, `set_family_owns_canvas`, degrau de zoom da família).

Não estão na P9 e não foram pedidos como bloqueio da integração: quem integra confirma com o dono se entram antes ou depois.
