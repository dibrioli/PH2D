# Handoff de integração — line/PainterWatercolor, a tinta viva (07/11)

> Leitor: o integrador e a próxima LLM. Detalhe e mecanismo ficam nos docs ligados; aqui só o que o
> merge precisa. ⛔ **Dono 07/11: NÃO integrar sem ordem explícita dele; o smoke é DEPOIS da integração, mais tarde.**

Detalhe: [HANDOFF_CONTINUACAO onda 2](HANDOFF_CONTINUACAO_2026-11-06_tinta_viva_onda2.md) (a última seção, «Onda do undo, da mão leve e do fecho», tem o mecanismo do undo, da Borracha e dos 5 pontos — não repetido aqui) · [TINTA_VIVA_2026-11-06.md](../TINTA_VIVA_2026-11-06.md) (plano e medições) · [onda 1](HANDOFF_CONTINUACAO_2026-11-06_tinta_viva_onda1.md).

## 1. Identidade

`line/PainterWatercolor` · HEAD `6097032030` (último de código: `chore(painter): clippy do fecho da tinta viva`; este handoff vem em commit de docs por cima) · base = merge-base = `main` do dia `bbfefaf393` · 33 commits antes deste · 73 arquivos · **não integrada**.

## 2. Superfície de colisão (saída de `scripts/collision-surface.sh`, 07/11)

| Item | Linha | Base |
|---|---|---|
| `PROJECT_SCHEMA` / tripla | 217 / (217, 13, 29) | igual |
| `VEC_SCENE_SCHEMA` · `FLIP_SCHEMA` · `DOC_VERSION` | 29 · 13 · 25 | igual |
| Registro de componentes (ecs / render / script) | 118 / 119 / 119 | igual |
| Contrato §6 (`nodegraph/node.rs`, `editor-core/tool.rs`) | intocado | — |
| ADR | nenhum criado (próximo livre 0199) | — |
| Cargo.lock | `+ "zstd"` na lista de deps de `ph2d-tool-painter`; **nenhum pacote novo** (`zstd 0.14.3` já no lock, usado por `ph2d-pen`) | — |
| Marcadores de conflito · tetos de LOC | nenhum · nenhum passa | — |

Foundational/compartilhado tocado: só `Cargo.lock` (1 linha) e `crates/ph2d-tool-painter/Cargo.toml` (`zstd = { version = "0.14", default-features = false }`). Sem `PROJECT_SCHEMA`; nenhum contrato §6 (conferido por `git diff bbfefaf393..HEAD --name-only`). Todo o resto está em `ph2d-tool-painter`, `ph2d-wet-paint`, docs e `project-memory/`.

Aditivo em `ph2d-wet-paint`: `Engine::fundo_da_historia` (default 6), `solta_o_registo_mais_velho`; `enterra_celulas` passou a marcar **só a caixa** (antes a grade inteira). Arquivos novos: `grid/celula_bytes.rs`, `grid/snapshot/registo_bytes.rs`, `painter/historia_tests.rs`; em `ph2d-tool-painter` `wetpaint/undo_vivo.rs` (reescrito), `undo_auditoria_tests.rs`, `undo_tinta_morre_tests.rs`, `sonda_borracha_morta.rs`, `foto_em_tiles.rs`.

## 3. O que um merge pode partir

- **Outra linha que mexa em `wetpaint/undo_vivo.rs`, `session.rs`, `wetpaint_commit.rs`, `ph2d-wet-paint/src/painter/historia.rs` ou `grid/`**: textual. Quem chega depois deve voltar a correr `undo_auditoria undo_tinta foto_em_tiles`.
- **Semântica de `enterra_celulas`**: quem chamava esperando a grade inteira marcada agora recebe só a caixa. Confira outros chamadores (`git grep enterra_celulas`).
- **Tetos de memória**: `teto_da_historia` conta registos + fotos (6 × `history_budget_bytes`); o `bytes_retidos` conta mais do que antes — o limiar «adormecida vira morta» pode disparar mais cedo; **não medido**.
- `PH2D_PAPEL_SEM_DEGRAU` e os pincéis de fábrica: **inalterados** (alarmes 2/2, nenhum pincel mudou).

## 4. Novo desde o registo anterior (deltas, factos)

**Mutação: 32 de 34 com prova.** F7 e F8 (`retain <=` e `is_none_or` em `wet_enterra_como_passo`) **sobrevivem**, sem portão nem prova de equivalência. F3 e H2 fechadas por REMOÇÃO de código redundante (guarda de ponteiro; parâmetro `aplicou` trocado por `Arc::ptr_eq`). Portões novos: `para_la_do_tecto_os_passos_que_restam_ficam_esquecidos` · `a_foto_mais_velha_conta_inteira` · `o_alfa_de_onde_nada_se_tira_fica_igual` · `a_troca_de_camada_com_um_traco_aberto_nao_leva_a_tinta` · `a_tela_de_um_traco_aberto_so_se_enterra_quando_ele_acaba` · `uma_tela_de_outro_tamanho_acaba_a_sessao_sem_panico` · `a_orla_da_base_nao_toca_a_celula_tocada` · `a_orla_da_base_ignora_o_excesso_de_arredondamento` · `a_orla_da_base_poe_a_cobertura_que_a_tela_mostra` · `desfazer_a_camada_com_a_tinta_da_nova_viva_acorda_a_de_antes`.

**Auditoria de 2 lentes, commit `f6014599ca`:**
- A1 não reproduziu (`active_grid_mut` já chama `marca_tudo`); portões de regressão postos.
- A2 o tecto conta `Congelada`/`base_depois` por `Arc` distinto (`o_tecto_conta_o_que_cada_enterro_retem`).
- A3 planos só na caixa. 4K, 10 traços, mín/mediana ms: para lá do tecto 1224/1571 → 350/460; com foto, já enterrou 1417/1664 → 397/1451, carga ~15. **Acima dos 100 ms pretendidos; a régua não foi refeita sobre o ficheiro final.**
- A4 `toca_tudo` no refazer (`refazer_para_la_do_tecto_recusa_a_imagem_no_ar`).
- A5 invisível só a ≤ 32 células do que mudou (`para_la_do_tecto_a_tinta_longe_fica_ao_bit`).
- A6 reservas. A7 larga passos abaixo de `chave_do_topo − undo_depth` (`o_passo_que_o_documento_ja_largou_sai_do_tecto`). A8 `usize` no mapa. A9 `desde = None`.
- **Achado maior:** `enterra_celulas` copiava a grade inteira para o registo (via `active_grid_mut`); agora só a caixa (`historia_o_enterro_so_retem_os_tiles_dele`). O RED não foi refeito na régua corrigida.
- `RAYON_NUM_THREADS=1`: 5 testes, sem deadlock. 1.º dab a 4K: pen-down 13–57 ms.

**Incidente, a relatar:** `undo_vivo.rs` foi reconstruído À MÃO por um agente depois de um `git checkout` apagar edições não comitadas. Os portões e a bateria correram DEPOIS da reconstrução; mesmo assim, quem revisar este arquivo deve lê-lo inteiro, não só o diff.

**Correção de registro:** a mensagem do commit `325d15de9b` tem a causa errada para os 5 pontos (a certa está na onda 2, §3 da seção final).

## 5. Prova de fecho (07/11)

Árvore ANTES do lint `6097032030`; depois dele correram clippy limpo das 3 crates e `undo_auditoria undo_tinta foto_em_tiles` (27 verdes).

fmt ✓ · por nome `ph2d-tool-painter` 275 passam, 37 ignorados · `ph2d-wet-paint` 14/14 · alarmes de pincel 2/2 (nenhum pincel mudou) · `ph2d-editor-core` 13/13 (LOC, clamp, `shell_only_shrinks`, CLAUDE.md, índice da memória) · `file_loc_caps` 4/4 · `seam_wetpaint` 33/33 · censos 133 testes, 12/12. **Suíte completa NÃO correu: é da integração.**

Binário smoke: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke` — 2.ª saída `Finished \`smoke\` profile [optimized] target(s) in 0.42s`, zero `Compiling`. Binário: `target/smoke/ph2d-host-desktop`.

`agent-loop-profile`: test:check 5,0× e contexto 309 mil por passo — ambos ✗ (fora do alvo).

## 6. ABERTO, por ordem de importância

1. ⚠️ **Água a correr no traço desfeito para lá do tecto:** `tocada` enterra células longe dele (97 171 valores zerados na cena do portão), o que contraria «a tinta que sobrou continua viva» nesse caso de borda. A cura exige caixa dos dabs por passo, com risco de degrau: **não construída**.
2. **Wet Round, troca de modo:** trava 1080p 89/333 ms, 4K 245/1282 (main 381/409 e 1303/1382); não é regressão. Compor por linhas foi recusado por medição (patch em `material_2026-11-06/regua_censo_do_sujo.patch`). Opções: imagem atrasada, baixar a ótica, aceitar — **decisão do dono**.
3. **Veredito do dono:** `PISO` 0,30 da Borracha de mão leve (`material_2026-11-06/foto_borracha_pouca_pressao.png`; alternativa 0,15).
4. **Veredito do dono:** `foto_resto_da_borracha.png` (manchas, fio claro); nada foi mudado nisso.
5. Menores: F7/F8 sem prova · macro com desfazer do 1.º traço não testada (nenhum teste de `macro_track` grava desfazer com Wet) · `bytes_retidos` conta mais (ver §3) · Dry Canvas 1,3–1,45 s e registos 210 → 1191 MB (já acontecia) · traço que não muda píxel não enterra a orla · o refazer para lá do tecto devolve tinta morta, por desenho · registo em vida não comprimido (candidato) · «a foto 5 refeita» do dono não chegou.
6. **Depois de integrar:** opção «manter a tinta viva» ao salvar, ligada por padrão, sem perda primeiro (mapa na onda 2). Depois [PROXIMA_FASE_2026-11-06.md](../PROXIMA_FASE_2026-11-06.md).

## 7. Smoke do dono (só depois da integração, quando o Enio mandar)

Sem cena especial: abrir uma tela nova.

Comando: `cd /home/enio/Documentos/Projetos/PH2D && ./target/smoke/ph2d-host-desktop` (depois da integração o binário é o do `main`; antes, o do worktree: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && ./target/smoke/ph2d-host-desktop`). Abra o Painter e crie uma tela nova.

**A. Desfazer não mata a tinta**
1. Com o pincel **Dabber**, faça 8 traços, cada um sobre o anterior.
2. Aperte `Ctrl+Z` 5 vezes.
3. Pegue a **Relief Eraser** e passe nos 3 traços que sobraram. Certo: a tinta amassa e some aos poucos. Errado: corta seco, como se fosse uma borracha comum.
4. Aperte `Ctrl+Shift+Z` 5 vezes. Certo: os 5 traços voltam.

**B. Desfazer e refazer um traço só**
1. Tela nova. Um traço de Dabber.
2. `Ctrl+Z`, depois `Ctrl+Shift+Z`.
3. Passe a Relief Eraser nele. Certo: amassa (tinta viva). Errado: corta seco.

**C. Trocar de modo logo depois de soltar (veredito próprio)**
1. Tela grande, pincel de óleo (Dabber). Faça um traço e solte.
2. Imediatamente troque para outro modo (por exemplo Digital).
3. Certo: a troca é imediata, sem travar. Errado: a tela congela por mais de um segundo.
4. Repita com um pincel de aguada (Wet Round). Aqui ainda trava um pouco, como hoje: **diga se aguenta ou se quer que eu conserte**.

**D. Relief Eraser com caneta leve**
1. Em modo Digital, pinte um traço grosso.
2. Com a Relief Eraser e a caneta bem leve, passe 20 vezes por cima.
3. Certo: ao fim das 20 passadas chegou ao papel. Errado: sobra tinta que nunca sai.

**E. As duas fotos**
Abra e diga o que acha: [mão leve da Borracha](material_2026-11-06/foto_borracha_pouca_pressao.png) e [o resto da Borracha](material_2026-11-06/foto_resto_da_borracha.png).
