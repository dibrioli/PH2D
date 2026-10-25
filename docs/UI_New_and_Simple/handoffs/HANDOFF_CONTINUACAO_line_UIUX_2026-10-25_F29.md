# HANDOFF DE CONTINUAÇÃO — `line/UIUX`, 25/10 — F29: o composto do balde tem um z só, a booleana veste a cor, o Corte corta

- Branch `line/UIUX`, worktree `Worktrees/line-UIUX`. Base: `main` `c3662cd8e4`. A linha tem os 5 commits da F28 + os da F29, **NÃO integrados**.
  Anterior: [F28](HANDOFF_CONTINUACAO_line_UIUX_2026-10-24_F28.md) (§8 = os três reports que abriram esta onda).
- Mecanismo medido de cada item, recusas e cura: [`../F29_plano.md`](../F29_plano.md). **Não está copiado aqui.**
- Estado: construído, gates vistos vermelhos por mutação; **smoke do dono (§4) por fazer**. Próxima LLM: janela nova.

## 1 — Commits (`git log --oneline c3662cd8e4..HEAD`, só a F29)

| commit | o quê |
|---|---|
| `8cfb48bbcb` | `fix`: as curas (A composto/balde, B booleana, C Corte) + gates + plano |
| `433f57ad03` | `test`: cena `PH2D_OBJECT_MODE_SMOKE=38` + fotos `img_F29/` |
| `f4bd590983` | `chore`: fecho — fmt, clippy e as réguas que o diff partiu |
| (este) | este handoff |

Reports do dono: os três de F28 §8 + um a meio da onda (25/10): *«A tool Cut não exibe suas funções nem no painel nem na barra
flutuante, logo, não consigo cortar nada com ela. Vamos colocar na barra flutuante.»*

## 2 — O que mudou

### 2.1 (A) O composto weld+balde tem UM z; a tinta nasce filha do caminho

Causa: a tinta ia para o fundo da CENA (`arm_new_fills` → `ToBack`) e a rede do balde via as bordas de TODOS os objectos
(`contornos_mundo`). Cura:

- A tinta é FILHA do caminho das âncoras mais à frente, com `ShowBehindParent`. A pilha do Vetor
  (`zorder::emission_rank`, `ph2d-vec-entities/src/zorder.rs`) honra `ShowBehindParent` como o `sort_key` de sprites; o Arrange
  conta descendentes pela árvore (campo `arvore`).
- Cada tinta re-coze só contra as paredes do OBJECTO dela (pai + caminhos das âncoras; agrupadas entre tintas que partilham
  caminhos: `bucket::objectos`). A rede da cena inteira fica só para o realce (`BucketCache.realce`).
- A solda (`weld::soldar`, a porta que a shell chama) muda os filhos dos traços consumidos para o anfitrião
  (`transform::adopt_children`).
- A cura de F28 §7 (tinta órfã) continua necessária: tintas antigas (raiz) e âncora num 2.º caminho apagado.
- **Tintas de documentos anteriores à F29 NÃO são migradas** (um escritor por quadro faria passo de undo espúrio): ficam raiz, no
  fundo, com paredes = caminhos das âncoras.

### 2.2 (B) Booleana com operando transparente

`ph2d_vec_boolean` vestia o resultado sempre com o operando de CIMA. Agora `doador` (`crates/ph2d-vec-boolean/src/doador.rs`):
o de cima que não corta (verbo ≠ Subtract) e tem fill visível; senão o de cima que não corta; dá fill e stroke juntos.
**Muda o estilo de TODO Subtract** (veste o de trás, como o Minus Front do Illustrator); nenhum teste do motor (167) nem dos
consumidores reprovou.

### 2.3 (C) Corte (report 4)

A ④ (barra flutuante) sumia com o Corte (está em `PRECISE`); os verbos viviam só na ② e só com lâmina. E o clique do Corte
chamava o `on_press` comum da caneta (agarra nó / reabre ponta / insere nó no segmento da forma seleccionada antes de começar
caminho): clicar sobre a forma editava-a e nenhuma lâmina nascia. Cura: `PenTool::on_press_blade` (o mesmo `press`, agora em
`crates/ph2d-vec-edit/src/pen_press.rs`, com `lamina = true`); `bars_actions::cut_actions`: a ④ mostra [Cortar][Descartar] com
lâmina E seleção; saíram da ②.
⚠️ **O clique real nunca foi conduzido**: o XTest não chega à Xwayland da foto; só se provou por código. O smoke do dono é o
primeiro teste manual.

### 2.4 Gates (todos vistos VERMELHOS por mutação, agente `mutacao`, 9/9, diff restaurado byte a byte)

- `a_behind_child_draws_right_under_its_parent` (ph2d-vec-entities)
- `bucket_objeto_tests.rs` (ph2d-app-vec): `a_tinta_do_balde_nasce_filha_do_caminho_e_atras_dele` · `apagar_o_pai_leva_a_tinta_filha` ·
  `outro_objecto_sobre_o_composto_nao_corta_a_tinta` · `a_solda_leva_as_filhas_do_traco_consumido`
- `um_operando_transparente_nao_apaga_o_preenchimento` (ph2d-vec-boolean, 2 mutações)
- `o_clique_do_corte_comeca_a_lamina_mesmo_sobre_a_forma` (ph2d-vec-edit `blade_tests.rs`)
- `the_cut_chips_need_the_cut_tool_and_a_blade` (ph2d-panel-vector, reescrito para a ④)
- `apagar_a_forma_pintada_leva_a_tinta_do_balde` re-provado; cena: `a_cena_38_comeca_no_composto`.

### 2.5 Réguas que o diff partiu — curadas por corte, nunca subindo número

Catraca `the_shell_only_shrinks` (agulhas encurtadas, comentários velhos cortados) · três agulhas textuais da shell re-apontadas
(`the_network_is_cached_not_rebuilt_every_frame`, `the_upkeep_runs_in_every_tool_not_only_in_the_bucket`,
`the_cut_mode_draws_with_the_pen_and_owns_no_press_branch`) · isenção órfã `VECTOR_CUT_` apagada do censo de canais
(`architecture_every_editable_field_is_a_channel.rs`) · `panel_functions_under_loc_cap` (função `actions`) ·
`workspace_src_files_under_loc_cap` (os dois `lib.rs`) · `every_word_this_family_shows_comes_from_the_string_table` (a cena
renomeada `vector_mode_composto_smoke.rs`).

### 2.6 A cena `PH2D_OBJECT_MODE_SMOKE=38`

Roteiro do Vetor (`vector_mode_scenes.rs`, código em `vector_mode_composto_smoke.rs`; família das alças `handles = 24`, estágio 42).
Fotos: `img_F29/38_antes.png` (composto com as três tintas à direita do quadrado verde) e `img_F29/38_composto_booleana_corte.png`
(composto arrastado: metade sobre o quadrado, tintas inteiras por cima, nada cortado; Hierarquia Path 4/5/6 filhas de Path 1;
meio-disco vermelho; quadrado azul com lâmina tracejada e barra flutuante com faca e lixo). Linha impressa:
`[object-mode-smoke] 38: pais das tintas (rede #1): 4→Some(1) 5→Some(1) 6→Some(1) · z: quadrado < tinta#4 < tinta#5 < tinta#6 < rede#1 · área antes = depois · booleana fill=Solid(220,40,40,255) · ④ do Corte: 2 botões`.

## 3 — Superfície de colisão (um merge pode partir)

- `crates/ph2d-vec-entities/src/zorder.rs` (StackRow/keyed_stack: quem mexe na pilha do Vetor) e `transform.rs`.
- `crates/ph2d-app-vec/src/bucket.rs` (`BucketCache` mudou de forma; `arm_new_fills` ganhou `scene`) e `weld.rs`
  (`apply_vec_weld` devolve `Option<(u64, Vec<u64>)>`; porta nova `soldar`).
- `crates/ph2d-vec-boolean/src/lib.rs` + `doador.rs`: lei do estilo — afecta todo consumidor do motor.
- `crates/ph2d-vec-edit/src/lib.rs` + `pen_press.rs` (o `on_press` passou a delegar).
- `crates/ph2d-panel-vector/src/bars_actions.rs` + `bars_row.rs`.
- Shell: `render_loop/fase_entity_sync.rs`, `render_loop/fase_node_and_arrange_verbs.rs`,
  `input_dispatch/despacho_clique_vetor_premido.rs`; gates `the_bucket_tool_owns_its_gesture.rs` / `the_node_ops_are_wired.rs`.
- `vector_mode_scenes.rs` (número de cena 38 — **reconta-se na integração**) · `architecture_every_editable_field_is_a_channel.rs`
  (isenção apagada).
- **Semântica:** Subtract veste o de trás; tinta nova é filha; `ShowBehindParent` agora conta na pilha do Vetor.
- **Formato:** nenhum campo novo gravado; `PROJECT_SCHEMA` e registos não tocados por esta onda (o integrador reconta na mesma).
- **Delta de contadores:** cenas `PH2D_OBJECT_MODE_SMOKE` +1 (`..38`). Mais nada a recontar.

## 4 — Smoke do dono (em linguagem simples)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && PH2D_OBJECT_MODE_SMOKE=38 cargo run -p ph2d-host-desktop --profile smoke`
2. A cena monta-se sozinha em uns 15 segundos: primeiro um quadrado verde à esquerda e, à direita dele, um círculo cortado por um X
   com três fatias pintadas (amarelo, ciano, magenta); depois o círculo desliza para cima do quadrado.
3. Tem de acontecer: as três cores e as linhas brancas ficam TODAS por cima do quadrado verde, inteiras (nenhuma fatia some por
   baixo dele nem é cortada pela borda). Na lista à esquerda (Hierarchy) as três cores aparecem por baixo do círculo, como filhas
   dele. À direita há uma forma meio-círculo VERMELHA. Em baixo, um quadrado azul com uma linha tracejada e uma barrinha com dois
   botões: faca (Cortar) e lixeira (Descartar).
4. Clique na faca: o quadrado azul parte-se em dois pela linha.
5. Teste à mão do Corte: escolha a ferramenta Corte (tecla C ou o ícone da faca), clique EM CIMA de uma forma e depois do outro
   lado dela. A linha da lâmina nasce, a forma não ganha pontos novos e a barrinha com faca e lixeira aparece. Clique na faca e a
   forma é cortada.
6. Teste à mão do conjunto: arraste o objecto com as cores sobre outra forma e de volta; as cores andam juntas e nunca ficam por
   baixo da outra forma. Apague-o (Delete): as cores somem junto. Ctrl+Z traz tudo.
7. Deu errado se: alguma cor fica por baixo do quadrado ou cortada; a forma vermelha aparece sem cor; a barrinha do Corte não
   aparece depois de desenhar a lâmina; ou o clique do Corte em cima da forma mexe nos pontos dela.

## 5 — Verificação

- Clippy `-D warnings` verde em `ph2d-vec-entities`, `ph2d-vec-boolean`, `ph2d-vec-edit`, `ph2d-panel-vector`, `ph2d-app-vec`,
  `ph2d-host-desktop`; fmt limpo; binário `--profile smoke` compila.
- Testes por nome verdes: bucket 28 · weld 18 · bool_ 63 · shape_build 15 · cut 10 · zorder 14 · transform 11 · `ph2d-vec-boolean`
  inteira 167 · vec-edit tests 266 · `the_bars_carry_the_gesture` 22 · `the_bucket_tool_owns_its_gesture` 7 · `the_node_ops_are_wired` 14.
- Mutação: 9/9 sangraram (§2.4).
- ⚠️ **Suíte completa NÃO corrida** (dono 23/10: só antes da integração). Um `nextest-impacted` arrancou-a por engano do verificador
  e foi interrompido aos 17 002/19 707 (16 982 ok; as 4 falhas reais eram as réguas de §2.5, já curadas; 16 SIGTERM da
  interrupção). Não vale como corrida: é do integrador.

## 6 — ABERTO, por ordem

1. Smoke do dono `=38` e o teste manual do Corte (§4; o clique real nunca foi conduzido).
2. A suíte inteira antes da integração (do integrador).
3. O `CLAUDE.md` §5 diz `PH2D_OBJECT_MODE_SMOKE=1|2|4|6..37`: passa a `..38` no fecho da linha/integração (conta-se no roteador).
4. A semente (`VecBucketFill.seed`) é guardada em MUNDO e fica velha quando o composto se move; só entra quando nenhuma âncora
   resolve (refazer as linhas). Gap medido, por decidir (mudar o significado é formato gravado).
5. Tintas de documentos anteriores à F29 continuam raiz no fundo (não migradas).
6. Herdados de F28 §6: perf de `linhas::layout`; «gate de fecho da LINHA»; undo da timeline vs vetor; ponte texto→chave de
   `nomes_do_motor`; a lista FORA.
7. **NÃO integrar nem fazer push sem ordem do Enio.**
