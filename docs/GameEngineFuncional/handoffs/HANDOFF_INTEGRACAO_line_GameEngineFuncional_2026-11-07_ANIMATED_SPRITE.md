# HANDOFF de integração — `line/GameEngineFuncional`, 07/11 (GEF 23 Sprite + GEF 24 Animated Sprite)

Leitor: o agente integrador. HEAD `59ad2121e3`, **55 commits** sobre a base `bbfefaf393` = merge-base = ponta do `main` (`git rev-parse main` = `bbfefaf393…`, conferido 07/11: o `main` NÃO andou; `--ff-only` serve sem rebase). **O intervalo inteiro está fora do `main` e inclui a Sprite (GEF 23: [handoff 06/11](HANDOFF_INTEGRACAO_line_GameEngineFuncional_2026-11-06_SPRITE.md), fecho em [23 FECHADA](HANDOFF_CONTINUACAO_2026-11-06_23_SPRITE_FECHADA.md)) mais o Animated Sprite (GEF 24)**; a Sprite não se repete aqui. Nada integrado, nenhum push. **Suíte completa e `nextest-impacted` NÃO correram** (regra do dono, 23/10: só o integrador). `git diff --shortstat bbfefaf393..HEAD`: 291 files, +10224 −2735 (21 deles `docs/` ou `project-memory/`).

⛔ **A linha entrega SEM smoke do dono do Animated Sprite**: o dono smoka AMANHÃ, depois da integração (§9). O smoke da Sprite foi OK em 06/11. Ficha (`no smoke`): [`02_animated_sprite.md`](../componentes/02_animated_sprite.md) · onda: [continuação 07/11](HANDOFF_CONTINUACAO_2026-11-07_24_ANIMATED_SPRITE.md).

## 1. O que entra (GEF 24; o mecanismo de cada etapa está na ficha)

**E0** medir o que grava chave · **E1** cartão Animation (verbos no cabeçalho, More e Look dobrados) · **E2** Who Picks Auto · Script · Rules com menu **Starts with**, ▶ só prévia · **E3** editor: **Split by rows**, ◆ marcador, tira que rola · **E4** Add ▸ Animated Sprite vazio + «Try with the sample hero» + aviso de imagem trocada (**Cut again** / **Keep the cut**); Add pede a imagem sozinho · **E5** regra que ganhou acesa no Play, «Who Picks · LIVE» (campo novo `RegrasEmCurso::acesas`, `personagem.rs`; a física não foi tocada) · **E6** cena `PH2D_GAME_SMOKE=5`, censo, fotos, provas reais de Play e Ctrl+Z (a prova achou e curou: com a aba do editor à frente o Ctrl+Z não chegava à fila do projecto, `c283350cfb`). Também no intervalo: `PH2D_PIXEL_SMOKE=21` (cartão do Pixel Sprite).

## 2. Superfície de colisão (foundational) — por que é aditiva

| Onde | O que | Aditivo porque |
|---|---|---|
| `ph2d-ecs` (+88) | `sprite_anim_start.rs` (`SpriteAnimStart(String)`), `sprite_cut_notice.rs` (`SpriteCutNotice`), 2 `register_default` em `scene/registry.rs:571-572`, `registry_tests.rs`; `sampling.rs` (`ImagemAberta`, recurso NÃO registado, da Sprite) | **componentes IRMÃOS, não campo de `SpriteAnimator`**: o postcard é posicional e o tocador grava-se em blob, campo novo tornaria ilegível todo blob antigo; só entram no mundo quando escritos (molde de `SpriteAnimFrames`). Sem degrau de schema |
| `ph2d-component-desc` (+12) | `catalog/image.rs`: dois `D::machinery` (categoria Animation) | linhas de tabela; sem ruído na Hierarchy (`inicio_e_aviso_nao_viram_ruido_na_hierarquia`) |
| **`ph2d-sprite-anim-scene`** (crate NOVA, 261) | guião da cena 5 (`passos`, `Foto`, `OBJETOS`); a shell só o executa (`janela_do_jogo_animado.rs`, 123) | folha nova que cura as catracas da shell e de `ph2d-app-game`; os cabeçalhos de smoke foram para `docs/Sprite_projeto/smoke_anim_a_andar.md`, `smoke_dither.md`, `docs/Timeline/smoke_nest.md` |
| `ph2d-editor-core` (16) | `doc_surface.rs`: método novo `DocumentSurface::undo_is_the_projects` (default `false`; trait não congelado) + uso em `surface_input.rs`; `new_image_ops.rs` (novo, saiu de `chrome_ops`); `pixel_card_ops.rs`, `ids/*`, `gizmo/sprite_box.rs`, `hier_types.rs` (−23) | método com default: quem implementa o trait não precisa de braço |
| `ph2d-panel-inspector` (18) · `ph2d-app-sprite-anim` (17) · `ph2d-app-game` (8) | cartão (`anim_cartao`, `anim_rules`, `anim_markers`), editor, intents, `object_add.rs`, `personagem.rs` (+36), `smoke*.rs`; 4 ficheiros de teste foram para `shells/game-scenes` | consumidores; catracas (§3) |
| `ph2d-render` · `ph2d-timeline` · `ph2d-i18n` · `ph2d-tokens` | `atlas/hero_sheet.rs` (+32) e `registry.rs` (−109, movimento); `doc_path.rs` (`path_key_corner`); chaves; sai o token `obj-painted` | movimento / método novo / chaves |
| `shells/desktop` · `shells/presos` | `Cargo.toml` +1 cada, `autokey_pass.rs`, `autokey_previa_tests.rs`, `fase_timeline_drain.rs`, `undo*.rs`, `janela_do_jogo*.rs`, `losango.rs` | composição |

`Cargo.lock` **+14**: pacote novo `ph2d-sprite-anim-scene` (entra em `ph2d-app-game` e `ph2d-host-desktop`); `ph2d-sprite-edit` ganha `ph2d-anim` e `ph2d-timeline`; `shells/presos` ganha `ph2d-sprite-edit`. Conflito de lock = `cargo metadata`, nunca à mão. `git diff --stat bbfefaf393..HEAD -- scripts .config .github .claude` é vazio.

## 3. Contratos e schema (corrido em 07/11)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional
git diff --stat bbfefaf393..HEAD -- crates/ph2d-editor-core/src/tool.rs crates/ph2d-nodegraph/src/node.rs crates/ph2d-nodegraph/src/cook.rs crates/ph2d-imageio/src/color.rs crates/ph2d-vector-doc/src crates/ph2d-vector-traits/src/anim_value.rs   → (vazio)
git diff --stat bbfefaf393..HEAD -- 'shells/desktop/src/project_schema*.rs'                                  → (vazio)
git diff -U0 bbfefaf393..HEAD -- '*.rs' | grep -cE '^[+-].*PROJECT_SCHEMA'                                      → 0
```

**§6 intocado; `PROJECT_SCHEMA` sem degrau.** `.ph2dproj` v191–193 verdes; `o_projeto_antigo_abre_igual` (sem os dois componentes: abre, não os inventa, regrava os mesmos bytes). **Números que SOMAM entre linhas, o integrador reconta:**
- **Registo de componentes** (`ph2d-ecs`, `registry_tests.rs`): **DELTA +2** (`SpriteAnimStart`, `SpriteCutNotice`; aqui 118→120, o `main` já tem 118 do SFX Lab). Os espelhos `registers_sprite_alongside_ecs` e `registers_luau_script_alongside_ecs` estavam vermelhos (literal 121) e agora **derivam** (`ph2d-ecs` + 1); a varredura não achou outro espelho.
- **Cenas de smoke:** `PH2D_GAME_SMOKE=5` e `PH2D_PIXEL_SMOKE=21` são novas (outra linha com o mesmo número colide).
- **Catracas** (`crates/ph2d-editor-core/tests/it/architecture_the_shell_only_shrinks.rs`; tectos NÃO subidos). Medido por `cat | wc -l` (o gate reconta, difere por poucas linhas): shell 189 036 (tecto 189 041, folga 5) · `ph2d-app-game` 2 728 (2 728, **folga 0**) · `ph2d-app-sprite-anim` 8 348 (8 352, folga 4) · `ph2d-app-pixel` 11 360 (11 415) · `ph2d-app-components` 80 860 (80 869, folga 9) · `ph2d-app-vec` 93 635 (93 637, folga 2). A folga soma entre linhas; cura = MOVER, nunca subir.

## 4. Medido e provado

- **Censo** `o_censo_dos_campos_do_animated_em_uso`: Animated **17 à vista e 48 no total, moldura 22** (era 28 e 54; **meta 15 e 47 NÃO atingida**, o dono julga). Sprite: 13 e 31.
- **Mutações:** 16/16 mataram em E6 (lista na ficha). As curas posteriores **não tiveram rodada de mutação**; vermelho visto só em: espelhos do registo (121×119) · renomear preso (o nome ia à seleção; voltava a `idle`) · ▶ armado (sem a cura a prova via nascer uma chave) · Ctrl+Z da aba (a prova achou o defeito).
- **Portões do último fecho** (da janela principal, NÃO re-corridos por mim, salvo ✔): fmt limpo · `clippy --workspace --all-targets -D warnings` limpo · `seam_anim` 48 · `o_cartao_da_animacao` 20 · `o_cartao_da_sprite` 27 · `sprite_` 103 · `editor::tests` 36 · `autokey_pass` 63 · `file_loc_caps` 4 · `workspace_file_loc_cap` 2 · `the_shell_only_shrinks` 2 · `arch_safe_clamp_only` 2 · `censos-da-arvore-combinada.sh` 133/133. ✔ `architecture_claude_md_cabe_no_orcamento` e ✔ `o_indice_da_memoria_conta_o_que_aponta` (`cargo-test-narrow.sh ph2d-editor-core <nome>`, 07/11, depois das edições de CLAUDE.md e MEMORY.md; as entradas do §5.1 ficam ≤700 bytes).
- **2.ª build** (`bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`, 07/11): `Finished \`smoke\` profile [optimized] target(s) in 0.25s`, sem `Compiling`.
- `agent-loop-profile.sh`: paralelismo 1,86 ✓ · respostas 72 ✓ · `test:check` 5,0× ✗ · edições sem conferir 1,0 % ✓ · contexto 309 mil ✗ · inicial 67 mil ✓.

## 5. Auditoria (duas lentes, 07/11): achado · cura · teste

Os rótulos F1–F5/A1–A6 não estão no repositório; vai por commit.

| Achado | Cura | Teste |
|---|---|---|
| espelhos do registo vermelhos | contam `ph2d-ecs` + 1 (`915307b53f`) | `registers_sprite_alongside_ecs`, `registers_luau_script_alongside_ecs` |
| renomear no cartão preso ia à seleção; `poe_estados_presos` repunha o nome; campo abria em cartão alheio (`60652ca8d6`) | grava no dono do cartão | `clicar_fora_grava_no_objecto_do_cartao_preso`, `a_pressao_no_proprio_cartao_nao_apaga_o_que_se_digitou`, `o_campo_do_nome_so_abre_no_cartao_do_dono` |
| `PH2D_CARDS=0`: linha e ▶ não seguiam o que começa (mesmo commit) | idem | `sem_cartoes_a_linha_muda_com_o_que_comeca`, `sem_cartoes_clicar_na_linha_muda_com_o_que_comeca` |
| ▶ com Auto Key armado gravava chave (`1c94c17f7f`) | ▶ é só prévia | `o_play_armado_nao_grava_chave_e_o_jogo_entra_no_inicio`, `a_linha_armada_grava_chave_de_qual_toca`, `a_chave_do_artista_vence_o_starts_with` |
| a grade antiga ressuscitava o aviso; Cut again sem proposta o fechava; herói recusado calava (`2e536f7656`) | mantém / abre o editor / vira aviso | `voltar_a_grade_antiga_nao_ressuscita_o_aviso`, `cut_again_sem_proposta_abre_o_editor`, `a_recusa_do_heroi_vira_aviso` |
| reordenar regra no Play perdia o seguro; Play no quadro errado (`3aef3ea05d`) | leva o seguro; 1.º quadro do início | `reordenar_no_play_leva_o_seguro_junto`, `o_play_comeca_no_primeiro_quadro_mesmo_ja_a_tocar_o_inicio` |
| catracas de `app-game` e `app-sprite-anim` acima do tecto (`59ad2121e3`) | corta comentário | `the_shell_only_shrinks` |

**Descartadas** (cada uma com teste, na ficha): regras no pai com tocador no filho · LIVE com `PH2D_CARDS=0` · corte no piso · início e aviso em instância e duplicar · desfazer do herói.

## 6. Decisões DELEGADAS, a confirmar no smoke

(a) ▶ com Auto Key armado **não grava chave**; clicar na LINHA grava `current`; a chave do artista vence Starts with enquanto existir curva. (b) `SpriteCutNotice` **fica no save** e reabre enquanto ninguém responder (tirá-lo perderia o desfazer). (c) «N components» **sobe 1** uma vez por `SpriteAnimStart`. (d) censo **17/48** em vez de 15/47. (e) **Frames/s, Cycle e Rules não gravam chave**, contra «Auto Key grava tudo» (19/10): o Coordenador não os construiu; o dono decide. (f) Look: resumo `Opacity 100%`.

## 7. Sem prova

Clique real na tela virtual (XTest não chega) · ▶ armado na janela real · toast do herói recusado na shell · **`animado_prova_desfaz` não recorrida** depois de perder o passo do Cut again sem proposta · remoção do aviso em `recut_users` e `inspector_commits_sprite` · Cut again COM proposta só pela foto do desfazer · ◆ com o nome do sinal na foto · a cena 5 sem tutorial em PDF. **Nome do objeto cortado no piso do painel («Start…»)**: chrome comum a todo objeto. **Abertos herdados da Sprite continuam**: ver [23 FECHADA](HANDOFF_CONTINUACAO_2026-11-06_23_SPRITE_FECHADA.md), com o que é «antes de integrar» (onda 3), e o §8 do [handoff 06/11](HANDOFF_INTEGRACAO_line_GameEngineFuncional_2026-11-06_SPRITE.md).

## 8. Riscos de integração

- **Primário:** `git -C /home/enio/Documentos/Projetos/PH2D status --short` ANTES. Em 07/11 tinha ` M project-memory/MEMORY.md` e 4 ficheiros novos de outras sessões (`project_sfx_w4_…`, `project_tilemap_rule_…`, `project_tiles_any_layer_…`, `project_two_image_objects_…`). **Nunca descartar**: funda o `MEMORY.md` por reconta (`o_indice_da_memoria_conta_o_que_aponta`, tecto 22 000 bytes), sem `checkout`.
- **Registo de componentes:** reconte o DELTA (§3). **`CLAUDE.md` §5.1:** entradas Game Engine e Pixel Art (`1..21`) podem colidir; o §5 inteiro tem tecto de 14 000 e aqui mede 13 994 (sem folga). `.config/testes-lentos.tsv` e `Cargo.lock`: regenerar. **Catracas** (§3): folga 0–9 em quatro crates.
- Existem: `scripts/collision-surface.sh` (caminho ABSOLUTO), `censos-da-arvore-combinada.sh`, `recontar.py`, `schema-recount.py`, `integrar-rodada.sh`.

## 9. SMOKE do dono (amanhã, depois da integração)

⚠️ O integrador constrói antes o binário do `main`: `cd /home/enio/Documentos/Projetos/PH2D && cargo build --profile smoke -p ph2d-host-desktop`. Fotos TEMPORÁRIAS (tiradas antes das últimas curas, que só mudaram balões): `/tmp/claude-1000/-home-enio-Documentos-Projetos-PH2D/6e8151aa-aa08-409b-aebb-10e09e79403f/scratchpad/fotos_smoke/` (`animado_vazio|uso|more|editor|inicio|aviso|regra|estreito`, `add_pede_a_imagem`, `jogo4_uso`, `prova_play`, `prova_desfaz`; `.png` + `.log`).

1. **Abrir:** `cd /home/enio/Documentos/Projetos/PH2D && env PH2D_GAME_SMOKE=5 ./target/smoke/ph2d-host-desktop`. Abre uma fase com cinco objetos novos em fila e o Player. Errado: não abre ou fecha sozinho.
2. **Objeto vazio.** Na lista da esquerda (Hierarchy) clique **Empty Sheet**. Na direita (Inspector), o cartão **Image** diz «Drop a sprite sheet or click to choose», com as imagens recentes e **Try with the sample hero**. Isto É um Animated Sprite sem imagem, por isso ainda não há cartão Animation. Clique o botão: nasce o herói com **Animation (6)**. **Ctrl+Z** volta ao vazio num só passo. Errado: o botão não faz nada ou o Ctrl+Z precisa de dois passos. O menu **Add ▸ Animated Sprite** deve abrir o painel «Choose the image for …».
3. **Cortar por fileiras.** Clique **Split Sheet**: o cartão **Animation (5)** tem `loop` (32 frames) e `row 1` a `row 4`. O cabeçalho do cartão tem **+** (nova), **−** (apagar), o ícone de filme (**Edit Animations**) e ↺. O filme abre a aba «Animations — Split Sheet»: lista, folha, fila e o botão **Split by rows** (uma animação por fileira que tem desenho). Clique um quadro da fila («Click a frame to mark it»): surge uma **◆**, o marcador que grita um sinal no jogo sem script. Errado: Split by rows duplica fileiras, a ◆ não surge, ou o Ctrl+Z com a aba à frente não desfaz.
4. **Com qual começa.** Clique **Starts With**. **Who Picks** tem **Auto · Script · Rules**; em **Auto** surge o menu **Starts with**. Clique o ▶ de OUTRA linha: é só prévia e **não muda** o Starts with. Escolha outra no menu e use **Run ▸ Play**: o jogo começa pela escolhida; ao parar, o objeto volta como estava. Errado: o ▶ muda o menu, ou o jogo começa por outra.
5. **Imagem trocada.** Clique **Swapped Image**: abaixo de **Use** diz «New image, old cut: 8 × 4 makes cells of 100 × 150 px, and these 6 animations were made for another image.» com **Cut again** (refaz o corte) e **Keep the cut** (aceita). Clique cada um; **Ctrl+Z** devolve o aviso. Errado: sem aviso, ou o aviso volta depois de respondido.
6. **Regra acesa.** Clique **Player**: **Who Picks · LIVE** em **Rules** é a lista «quando → toca». **Run ▸ Play**, ande e pare: a faixa azul passa de regra a regra (Standing still, Running, Going up…). Errado: nada acende ou a faixa fica presa. Se o Inspector não mostrar a faixa no Play, anote: foi provado por log, não por clique.
7. **Em uso.** Rode o mesmo comando com `PH2D_GAME_SMOKE=4`: moedas e bandeira são Animated Sprites; clique **Coin 1** e veja o cartão Animation e o **More** (`Hold · signals`).
8. **Decisões (§6):** com **Auto Key** e Timeline abertos, o ▶ não grava chave e clicar na linha grava (a); «24 entities · 230 components» sobe 1 por Starts with (c); 17 campos contra 15 do papel (d); Frames/s, Cycle e Rules sem losango (e); Look dobrado com `Opacity 100%` (f).
9. Diga o que deu errado em cada passo; aprovado, a ficha vira `fechado` e a fila anda para Add Object.

## ⛔ Recusas MEDIDAS (completas na ficha)

Folha ou tira dentro do cartão (27/10) · ensaio com teclas nesta rodada · aviso de regra morta · `SpriteCutNotice` fora do save (perderia o desfazer) · campo novo no `SpriteAnimator` para o início (blob antigo ilegível).
