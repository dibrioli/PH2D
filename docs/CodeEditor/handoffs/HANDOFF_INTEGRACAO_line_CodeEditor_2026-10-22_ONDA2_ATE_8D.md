# HANDOFF DE INTEGRAÇÃO — line/CodeEditor, Onda 2 inteira até à fase 8d (d4/d4b) · 22/10/2026

> Leitor: o agente integrador e a próxima LLM da linha. **Não integrado, não enviado.** Último CodeEditor no `main`:
> Onda 2 I0–I5 (17/10, [handoff](HANDOFF_INTEGRACAO_line_CodeEditor_2026-10-17_ONDA2_I5.md)). Este handoff **substitui**
> os de integração de 20/10 ([I6 a API](HANDOFF_INTEGRACAO_line_CodeEditor_2026-10-20_ONDA2_I6_A_API.md) e
> [I7 a posse](HANDOFF_INTEGRACAO_line_CodeEditor_2026-10-20_ONDA2_I7_A_POSSE.md), nunca integrados): as suas
> secções de superfície, portões e ABERTOS valem **só** onde este não diz o contrário — o mecanismo de cada item vive
> nos docs linkados em §3, não aqui.
> **Ordem do dono (22/10):** integrar esta linha no `main` **depois** da `line/Macros-Automação`; a
> `line/PainterWatercolor` (sessão ph2d-68) integra depois desta. **Smoke do dono: depois de integrado** (§9).

## 1. Identidade

- Worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor` · branch `line/CodeEditor` · HEAD
  `3e88dc7bc1` (+ o commit de docs deste handoff) · merge-base `100d34c8e0` · **131 commits** · 849 ficheiros
  (+69 683 / −4 513) · `main` medido a `af072d09d6` (259 commits à frente do merge-base).
- Tabelas de medição, oráculos e fotos de cada item: `docs/CodeEditor/ferramentas/`.

## 2. Superfície de colisão (`scripts/collision-surface.sh`, 22/10 — REFERÊNCIA, não evidência)

```
SUPERFÍCIE DE COLISÃO — line/CodeEditor contra main
  merge-base 100d34c8e0   ·   131 commit(s)   ·   849 arquivo(s)
▸ SCHEMAS   PROJECT_SCHEMA 191 (base 191) · tripla (191, 13, 23) = base · VEC_SCENE 23 · FLIP 13 · DOC_VERSION 20 = base
▸ REGISTRO  ph2d-ecs 108 · ph2d-render (espelho) 109 · ph2d-script (espelho) 109  (todos = base)
▸ CONTRATO CONGELADO (§6)  node.rs intocado · tool.rs intocado
▸ ADR       último 0188, próximo 0189 — esta linha não cria ADR
▸ Cargo.lock  3 pacotes '+name' novos, todos INTERNOS: ph2d-code-explain · ph2d-code-format · ph2d-code-help
▸ MARCADORES DE CONFLITO  nenhum nos arquivos da linha
▸ TETOS DE LOC  ✗ 1059 / 600  shells/desktop/src/main.rs  (isenção pré-existente)
```

**Contadores como DELTA da linha:** `PROJECT_SCHEMA` 0 · tripla 0 · registos de componentes 0 · ADR 0 · contratos
§6 0 · crates novas desde o I5: `ph2d-code-explain` (I8), `ph2d-code-format` (I13), `ph2d-code-help` (ajuda) — o
`Cargo.lock` regenera-se, nunca à mão. ⚠️ O `main` já recontou o `PROJECT_SCHEMA` para **193** (Macros, mensagem de
`af072d09d6`): a linha não tem degrau de schema, por isso **não há recontagem a fazer por nós** — confirme com
`python3 scripts/schema-recount.py` na árvore combinada.

**Conflitos reais** (`git merge-tree HEAD main`, 22/10, contra `af072d09d6`; 9 hunks em 4 ficheiros — relê-se no
`main` do dia, a coluna `base:` é o merge-base):

| Ficheiro | Hunks | O que colide |
|---|---|---|
| `shells/desktop/src/undo.rs` | 5 | MiroClone W23 (Ctrl+Z das operações de abas) × o nosso `push_linked`/marcador do undo global (I10) e o `FieldDrive` (8d) |
| `shells/desktop/src/forwarding.rs` | 2 | MiroClone W23 (copiar link sem chave) × a posse do teclado (I7) |
| `crates/ph2d-preview-drive/src/lib.rs` | 1 | UIUX cortou o lib.rs (752 → 474, `facto.rs` irmão); nós acrescentamos `FieldDrive`/`fields.rs` e re-exportes — a união fica no `lib.rs` pequeno |
| `crates/ph2d-app-components/Cargo.toml` | 1 | `[dev-dependencies]` aditivas dos dois lados (nós: `ph2d-code-format`, `ph2d-code-syntax`) — somar as duas listas |

`crates/ph2d-editor-core/src/interaction/state/mod.rs`: **693** na linha, **697** no `main` (UIUX curou o 702/700,
`c6f0963ba1`); a fusão limpa dá **695** (medido no tree do merge-tree) — recontar na árvore combinada depois de
resolver, o tecto 700 soma entre linhas.

## 3. O que a linha entrega desde o I5 (um item por linha; mecanismo no doc linkado)

| Item | O que é | Onde |
|---|---|---|
| I6+I11 | Ir para a definição (Ctrl+clique/F12) e tokens semânticos (cores do assistente) | [handoff](HANDOFF_CONTINUACAO_line_CodeEditor_2026-10-17_ONDA2_I6_I11.md) |
| I7 | Alt+Enter: «Did you mean», «Add type», quickfix do `luau-lsp`; lint do `self.<campo>` | [handoff](HANDOFF_CONTINUACAO_line_CodeEditor_2026-10-17_ONDA2_I7.md) |
| I8 / I8b | Explicações escritas à mão por classe do corpus (`ph2d-code-explain`); correcções como botões no balão | [I8](HANDOFF_CONTINUACAO_line_CodeEditor_2026-10-17_ONDA2_I8.md) · [I8b](HANDOFF_CONTINUACAO_line_CodeEditor_2026-10-18_ONDA2_I8b.md) |
| I10 | Renomear (F2) e referências; **marcador do undo global** (`UndoLink`: um passo em duas histórias) | [handoff](HANDOFF_CONTINUACAO_line_CodeEditor_2026-10-18_ONDA2_I10.md) |
| I9+I12+I14 | Dicas de tipo (inlay) opcionais; esqueletos com buracos tipados; paleta `@` `:` `>`; `--!strict` por ficheiro | [handoff](HANDOFF_CONTINUACAO_line_CodeEditor_2026-10-18_ONDA2_I9_I12_I14.md) |
| I13 | Formatar ao salvar com o StyLua 2.5.2 fixado (`ph2d-code-format`) | [handoff](HANDOFF_CONTINUACAO_line_CodeEditor_2026-10-18_ONDA2_I13.md) |
| I15 | Cena `PH2D_CODE_SMOKE=3`, bancada completa, solver antigo × novo (fica o antigo) | [handoff](HANDOFF_CONTINUACAO_line_CodeEditor_2026-10-19_ONDA2_I15.md) |
| Acabamento | Lista de problemas (Ctrl+Shift+M, F8), Ctrl+rato = mão, cena 4 | [handoff](HANDOFF_CONTINUACAO_line_CodeEditor_2026-10-19_ONDA2_ACABAMENTO.md) · [03](../03_onda2.md) |
| Ajuda (item 5) | A página de ajuda (`ph2d-code-help`, 225 nomes em 12 páginas), Ctrl+clique/F12 abrem-na; cena 5 | [handoff](HANDOFF_CONTINUACAO_line_CodeEditor_2026-10-20_ONDA2_AJUDA.md) · [plano 04](../04_pagina_de_ajuda.md) |
| Item 6 | A API que funciona: `ph2d.wait`, `find_by_name`, `spawn/despawn/attach_script`, teclado, rato, `loadstring`; cena 6 | [handoff I6](HANDOFF_INTEGRACAO_line_CodeEditor_2026-10-20_ONDA2_I6_A_API.md) · [plano 05](../05_api_que_funciona.md) |
| Item 7 | Com a cena a correr, teclado e clique são do JOGO (F5 pausa, Shift+F5 rebobina; moldura); lei pura `posse_do_jogo` | [handoff](HANDOFF_INTEGRACAO_line_CodeEditor_2026-10-20_ONDA2_I7_A_POSSE.md) · [plano 06](../06_posse_do_teclado.md) |
| Item 8a | Arrastar um objecto da Hierarquia para o script (Ctrl = local); `ph2d.object(name)`; avisos de nome; renomear reescreve os scripts no mesmo passo de undo; cena 7 | [handoff](HANDOFF_CONTINUACAO_line_CodeEditor_2026-10-21_ITEM8A_ARRASTAR.md) · [plano 07](../07_arrastar_para_o_codigo.md) |
| Fase 8d (d1–d4b) | Os scripts tocam TODA propriedade do Inspector pelo nome (`ph2d.get/set("health.max", …)`); detalhe em §4 | [plano 08](../08_scripts_tocam_propriedades.md) §7 |

## 4. Fase 8d — o que mexe fora da família (foundational / partilhado)

- **d1–d3 (`d6a129ce0b` · `e83fdc775f` · `1af469bdff` · `3cb9cfb865` · `73539a3a32` · `3620da4c20` · `9b3dabfbca`):**
  `FieldDesc.name/key/view` + `ComponentDesc.code/row_of/hidden` no registo; `value_of/set_value/check_value` na
  vtable; lei `ph2d_ecs::scene::property` (`resolve/get/set/names_of`); `ph2d.get/set/properties/damage/heal`;
  ledger por campo `ph2d_preview_drive::FieldDrive` (`undo.rs` usa `substitute/restore`); 156/156 componentes com
  `code` ou `Hidden::…`; o cartão passa a pintar a chave do CATÁLOGO (31 chaves paralelas saíram); skew na pose.
  **Valor do script = o do cartão, ângulos sempre em radianos.** Schema/registo/formato gravado: **nada** mudou (o
  postcard é o mesmo; o infinito da Luz é `null` só nos formatos legíveis).
- **d4 (`6c0987b565`):** referências pelo NOME do alvo — `View::Named(Target{Name, NameHash, Stable, Tag, TagSet,
  Layer})`; campos de variante `top:MOLDE/rest`; o ledger grava o enum inteiro (`FieldDesc::stored_key`);
  `RefNames`/`ScriptDoors.tags`; `catalog/physics.rs` cortado em `physics_fields.rs`; comentários de
  `interaction/state/mod.rs` aparados (é o ficheiro no tecto).
- **d4b (`5d95dd7853`):** `sprite.image` pelo nome da biblioteca = `ImageOverride` só da corrida
  (`ph2d-app-components/src/script_image.rs`, fora do Ctrl+Z, do gravar e do `FieldDrive`; sobe uma vez por imagem na
  `fase_macros_do_painter`, antes do extract; devolve-se no rebobinar). Nomes de biblioteca únicos
  (`library_names::unique`: stem, depois ` (2)`, ` (3)`). `audio_source_2d.sound` nomeado. `Hidden::Reference`
  **apagada**; `source` fica `Hidden::ScriptDoor`.
- **Fecho (`aab733e875` gates do catálogo · `8bead11016` fmt · `e9a44b8d3f` clippy · `3e88dc7bc1` 8 falhas):** a lei da
  soltura desce para `ph2d-code-syntax/src/dropping.rs` e **sai a aresta de dev `ph2d-app-components → ph2d-app-code`**
  (família → família reprovava as camadas; fica `→ ph2d-code-syntax`, folha); o portão da Hierarquia publica o painel
  + teste de cura novo; a cena 5 ensina o `PropertyName`; `ph2d.set` do Luau volta a perguntar a `lane::is_named_property`
  como o `get` — **regressão real** de `73539a3a32`.

## 5. O que um merge pode partir

- Quem toque o `undo.rs`/`forwarding.rs` da shell (§2), o prólogo de `on_mouse_input` ou o bloco do teclado
  (`keyboard.rs`, `tecla_do_jogo`).
- Quem acrescente linhas à shell: a catraca `the_shell_only_shrinks` (a linha deixou 189 005 contra 189 041 no I7;
  **recontar na árvore combinada** — soma entre linhas) e `interaction/state/mod.rs` (695 na fusão; tecto 700).
- Quem leia o catálogo de componentes: o cartão passou a pintar a chave do catálogo; `FieldDesc` ganhou campos e a
  variante `Hidden::Reference` não existe. Um literal de `FieldDesc`/`ComponentDesc` noutra linha deixa de compilar.
- Quem escreva `ph2d_input`/`InputState` fora do jogo: só chega teclado enquanto o jogo o tem (I7).
- Quem some `[dev-dependencies]` em `ph2d-app-components` (§2) ou volte a pôr `ph2d-app-code` ali.

## 6. Contratos congelados

**Nenhum tocado** (§2). Sem ADR novo, sem degrau de schema, sem registo novo.

## 7. Portões do fecho (22/10, MEDIDOS — sobre `8bead11016`+`e9a44b8d3f`, antes de `3e88dc7bc1`)

| Portão | Resultado |
|---|---|
| `cargo fmt --check` · clippy `--workspace --all-targets -D warnings` | ✓ |
| `nextest-impacted` (fecho) | 20 068 de 20 079: **8 FAIL reais** (todos curados em `3e88dc7bc1`, cada um reexecutado POR NOME e verde) + **3 flakes de carga** a `loadavg` até 172 (`as_duas_barras_do_igual…pico_no_preset_de_fabrica`; `ph2d-app-flip` `pass::luz::testes::quem_nao_recebe_a_luz…` e `a_camada_do_flip_para_a_luz…`), todos verdes sozinhos a `load` 11–15 ([FLAKES_DE_CARGA.md](../../DevOps/FLAKES_DE_CARGA.md)) |
| Depois de `3e88dc7bc1` | os 8 por nome ✓; **a suíte inteira NÃO foi refeita nesta árvore** |

⚠️ **A suíte inteira tem de correr de novo na árvore COMBINADA** (`PH2D_SUITE=fecho bash scripts/ph2d-run.sh
scripts/nextest-impacted.sh`), mais `censos-da-arvore-combinada.sh`, `foundational-integrate.sh` e clippy `--all-targets`;
`rm -rf target/*/incremental` antes do binário do smoke.

**Provas de mutação:** I7 (15 + controlo, 13/15 na 1.ª e os 2 sobreviventes ganharam teste), I8, I10, I13 (16), I12/I14 —
nos handoffs de continuação. **PENDENTE:** a mutação das leis do 8d (d1–d4b: o nome, a vista, a linha no neutro, o
nome do alvo, o override da imagem) — vai com o `/pd-linha-fechar` depois do d5.

## 8. Binário do smoke

Depois da fusão e do gate, na árvore combinada: `rm -rf target/*/incremental` e, **duas vezes**,
`cd <árvore> && bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`; a 2.ª tem de dizer
`Finished \`smoke\` profile` em < 1 s.

## 9. Smoke para o DONO (depois de integrado — ordem de 22/10)

Cenas já fotografadas e com smoke OK do dono antes: 1–4 (Onda 2), 5 (ajuda, **fotos a refazer**, ver §10), 6 (API +
teclado, OK 21/10), 7 (arrastar, OK 22/10). Nada de novo para o dono ver da d4/d4b ainda — a cena `PH2D_CODE_SMOKE=8`
(d5) é o que a mostra. Para rever tudo no `main`:

1. `cd /home/enio/Documentos/Projetos/PH2D && PH2D_CODE_SMOKE=7 cargo run -p ph2d-host-desktop --profile smoke`
2. Arraste «Door» da Hierarquia para o script segurando Ctrl e solte na linha 10: escreve o local com o nome do objecto.
3. Dê Play: a porta mexe. Ctrl+Z: a soltura desfaz-se.
4. Errado se: o objecto é reparentado, a linha sai sem recuo, ou o Ctrl+Z não desfaz a soltura.

## 10. ABERTO

- **Fases do plano 08:** **d5** (cena de smoke `PH2D_CODE_SMOKE=8` que ensina nomes, vistas e imagens), **8b**, **8c**,
  depois `/pd-linha-fechar` com a mutação do 8d. A próxima cena livre é a 8.
- **Fotos da cena 5 a refazer** (o texto agora diz `PropertyName`).
- **Perguntas de produto ao dono** (plano 08 §7): um script renomear um objecto durante o jogo (`Name` é `Editor`)?
  o cartão «Render Source» × o componente «Sprite» (código `sprite`) e se o cartão deve mostrar qual imagem da biblioteca
  o sprite usa? subtítulos por componente nos cartões compostos (Câmera, Paralaxe, HUD)?
- **Herdados do I7:** as «consequências declaradas» da posse do teclado (botão do meio e menu do botão direito do canvas
  são do jogo enquanto corre; Shift+letra vai ao jogo; teclas sem nome engolidas; uma cena que começa tapada precisa de
  um clique) — [I7 §9](HANDOFF_INTEGRACAO_line_CodeEditor_2026-10-20_ONDA2_I7_A_POSSE.md); e o que ficou no
  [I6 §9](HANDOFF_INTEGRACAO_line_CodeEditor_2026-10-20_ONDA2_I6_A_API.md).
- **Flakes de carga** da lista §7: se reprovarem na combinada, confirme com o `loadavg` ao lado antes de culpar o commit.
- **Integração:** a entrada do CLAUDE.md §5.1 (Editor de scripts) ainda aponta para o handoff do I7; no merge o integrador
  troca o link «Último» para ESTE e o «Smoke» para `1..7` (e `1..8` quando o d5 entrar). Régua do agente
  (`agent-loop-profile.sh`) por correr neste fecho.
