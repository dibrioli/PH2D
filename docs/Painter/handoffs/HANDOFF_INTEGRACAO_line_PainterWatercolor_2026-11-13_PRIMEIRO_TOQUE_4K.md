# Handoff de integração — linha PainterWatercolor · 13/11 (onda n.º 6): o primeiro toque a 4K, Wet e Watercolor Basic

> A linha NÃO está integrada; quem o dono nomear integra. Leitor: o integrador e a próxima LLM da linha.
> Anteriores (continuam a valer): [n.º 5 ONDA5_VIDRO_E_MEDIA](HANDOFF_CONTINUACAO_2026-11-12_ONDA5_VIDRO_E_MEDIA.md) ·
> [n.º 4 SELECAO_CAMADAS_E_RITMO](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-12_SELECAO_CAMADAS_E_RITMO.md) ·
> [MISTURA_E_ENVIO](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-12_MISTURA_E_ENVIO.md) ·
> [LUZ_E_CAMADAS](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-12_LUZ_E_CAMADAS.md) ·
> [DESEMPENHO_TOTAL](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-12_DESEMPENHO_TOTAL.md).
> Backlog: [`../missao_4k/BACKLOG.md`](../missao_4k/BACKLOG.md) (B18.4 «Cont. 7», B19.10).

## 0. Superfície de colisão

- Base: `main` = `4f46a3573c`; a linha já está rebaseada nele; merge-base `4f46a3573c`. **Não rebaseie** salvo se `git log HEAD..main` não for vazio.
- Esta onda = 4 commits sobre `ff30c0bcf6`: `4e4486cb30` (Wet) · `9b810843af` (compositor) · `8627ecf68a` (Watercolor Basic) · `87888a8290` (clippy). 23 ficheiros, +537/−18, todos em `crates/`; **zero** em `shells/`, `CLAUDE.md` à parte do §5.1, schema, ADR.
- Ficheiros (`git diff --stat ff30c0bcf6..HEAD`):
  - `ph2d-tool-painter/src/compositor/`: `cache/cache_tests.rs` · `compose/{barreira_tests,resumo,resumo_tests,tiras_tests}.rs`
  - `ph2d-tool-painter/src/tool/paint.rs` · `paint/{plane_fork,state,state_default,watercolor_backdrop,watercolor_render,wetpaint}.rs`
  - novos: `paint/watercolor_fundo_cache_tests.rs` · `paint/wetpaint/captura_da_caixa_tests.rs`; `paint/wetpaint/{composite,sonda_primeiro_traco}.rs`
  - `ph2d-tool-painter/src/{undo_delta_journal,undo_journal,undo_planes,undo_window}.rs`
  - `ph2d-wet-paint/src/solver.rs` · `solver/chao_forca.rs` · novo `solver/chao_sem_real_tests.rs`
- Contadores (DELTA): `PROJECT_SCHEMA` 0 · ADR 0 · cenas de smoke 0 · tripla 0 · gates novos 4 (§1–§3) · chaves A×B novas 4 (`CAPTURA_PELA_CAIXA` ×2 pontos, `CAPTURA_PARTILHADA`, `CHAO_SEM_REAL_SALTA`, `FUNDO_EM_CACHE`).
- O que um merge pode partir: `undo_journal.rs` (o `whole` passou a `Arc<Vec<T>>`; quem mais escrever `UndoJournal` por baixo) · `paint/plane_fork.rs` · `watercolor_backdrop.rs`. Os testes do compositor que mexem em chaves globais tomam `trava_das_chaves()`: um teste novo que as vire sem a trava volta a ser flaky.
- Aviso do `main`, não desta linha: `shells/desktop/src/main.rs:941` tem padrão inalcançável (braço `WindowEvent::CursorLeft` duplicado, um tratador morto). Para o integrador.

## 1. Wet (`4e4486cb30`)

Sonda `sonda_primeiro_traco`, Wet Wash, 3840×2160, preparo, camada nova; `smoke`, mesmo binário A×B, load 3–6:

| evento | antes | depois |
|---|---|---|
| 1.º traço da camada nova, 3.º evento | 20,0–21,6 ms · 91,4 MB | 14,0–16,1 ms · 59,5 MB |
| 2.º traço, 1.º evento (pen-down) | 9,9–12,0 ms · 96,8 MB | 6,7–7,1 ms · 63,5 MB |
| composite do 2.º traço | 2,3–6 ms | 0,5 ms |

1. `composite.rs`: o `fork_canvas` do composite Wet recebe a caixa do `declare_wrote`; o journal guarda os tiles dela, não a tela (33 MB a 4K por traço). Chave `CAPTURA_PELA_CAIXA`.
2. `undo_journal.rs`: `capture_shared`/`capture_canvas_arc` partilham o buffer velho quando o fork copiaria de qualquer modo (strong>1) e no `replace_canvas` (o «+» passa de 33 MB copiados a 0); o commit usa o `Arc` em vez de `to_vec`. Chave `CAPTURA_PARTILHADA`.
3. `chao_forca::fotografa`: sem Real Oil em célula nenhuma o ganho Depth×Real Oil é 0 e o chão saía todo zero: não se calcula (33 MB, ~5 ms por traço). Chave `CHAO_SEM_REAL_SALTA`.

Gates: `o_desfazer_do_composite_wet_guarda_a_caixa_e_nao_a_tela` (512×384: pico antigo = tela 786 432 B, novo 98–196 KB; mesmos bytes; controlo: o antigo captura a tela) · `sem_real_oil_o_chao_nao_se_fotografa_e_sai_o_mesmo` (controlo com Real Oil, ao bit; mutação «salta sempre» → vermelho).

**O que resta do custo do 1.º traço Wet (decomposto, não curado):**
- fork da tela 6 ms · 33 MB: COW inerente contra a base da sessão Wet (strong=3);
- pousa do 1.º dab ~8 ms: primeiro toque às páginas da grade (`e.transfer` 4,4 ms, `marca_dab` 1,4 ms);
- pen-down de traço que continua: espera pelo motor (`bring_home`) 3,8 ms + `abre_o_passo` 2 ms. A espera só some adiando o pen-down inteiro (abertura do passo de undo + abertura do traço para o lote adiado): julgado arriscado, não feito.
- «+» a 4K: 55–65 ms = `add_raster_layer` 51 ms (`commit_structural_edit` 15–30, dos quais `UndoEntry::split` 14 ms a comparar a tela velha com a nova zerada; `flush_active_to_images` 6,5; `a_tinta_que_adormeceu_muda` 7,5) + preview 4,4. A cura mudaria a posse/contabilidade do undo (entradas `Whole` que seguram a tela viva, tecto de bytes do undo): troca visível ao produto, não feita.

## 2. Compositor (`9b810843af`)

- `cache_hit_skips_below_layers` estava VERMELHO desde `57b9cf12d5` (onda n.º 4): `com_resumos` chamava `layer_rgba` de toda camada de baixo antes de olhar o plano, e a fonte sem planos contava-o como leitura. Agora o plano primeiro; a imagem só com ele.
- `o_resumo_por_ladrilho_e_a_rota_sem_resumo_ao_bit_e_nunca_fica_velho` era flaky em paralelo: `SEM_RESUMO`, `SALTO_SO_NORMAL`, `LINHAS_DA_TIRA`, `FIOS_DA_REGIAO` são globais. `trava_das_chaves()` (`cfg(test)`) nos cinco testes que as mexem. `compositor::`: 60 verdes em 3 corridas seguidas.

## 3. Watercolor Basic (`8627ecf68a`)

Mesma sonda, WB, 3840×2160, preparo, camada nova; load 6–10:

| pen-down | antes | depois |
|---|---|---|
| base, 1.º traço | 36–39 ms | 27–31 ms |
| base, 2.º traço | 28–29 ms | 14,4–15,4 ms |
| camada nova, 2.º traço | 31–35 ms | 14,7–15,7 ms |
| camada nova, 1.º traço (depois do «+») | 45–50 ms | 45–50 ms (a pilha mudou: tem de recompor) |

1. `watercolor_render.rs`: o `fork_canvas` da aguada passa a região que o laço escreve (a do `declare_wrote`): o journal guarda os tiles dela (5,7 ms · 33 MB por pen-down a 4K). Chave `CAPTURA_PELA_CAIXA`.
2. `watercolor_backdrop.rs::fundo_de_baixo`: o `composite_below` das camadas de baixo (10–14 ms) fica em cache sob `ChaveDoFundo` = a pilha (`PartialEq`) + a identidade `Weak` do `rgba8` de cada imagem + o papel + o tamanho. Com o `Weak` vivo nenhuma escrita segura muda o plano no lugar (`make_mut` copia ou MOVE). O chão preto do vidro já se guardava pelo `Arc` do fundo: com o mesmo `Arc` deixa de recompor (8 ms). Máscara activa: compõe sempre. Chave `FUNDO_EM_CACHE`.

Gates: `o_desfazer_da_aguada_guarda_a_caixa_e_nao_a_tela` (512×384: tela 786 432 B → 114–229 KB; controlo: o antigo captura a tela) · `o_fundo_em_cache_e_o_composto_do_zero_e_so_se_recompoe_quando_algo_abaixo_muda` (oráculo = `FUNDO_EM_CACHE` desligado no mesmo processo, depois de pintar em cima, opacidade, visibilidade, pixels de baixo por traço, desfazer, só os pixels com a pilha igual, ordem; controlo: sem mudança o MESMO `Arc`). Mutações «sem a identidade das imagens» e «sem a pilha» → vermelhas.

**WB ainda aberto:** o COW do vidro por traço (`vidro_do_quadro` 11,5 ms · 115 MB no 2.º pen-down: o snapshot de undo guarda o plano do vidro) = undo em janela/tiles — mudança de desenho, pede ADR e a palavra do dono. Reserva do Wet Mix (`r.reserve` 4,4 ms · 33 MB no 1.º traço).

## 4. ⛔ Recusas MEDIDAS (não reconstruir)

| # | ideia | medição | porquê |
|---|---|---|---|
| a | camada nova toda a zero nascer zerada no fork em vez de copiar | varredura «é tudo zero?» 5,2 × cópia 5,4–6,4 ms | ler páginas nunca tocadas também dá falta de página |
| b | reserva de cobertura/cor/água entre sessões (zerar só a caixa suja) | vidro 8,4 → 14,6 ms; pen-down igual, 50,1 × 50,9 ms | as faltas só mudam para o buffer grande seguinte; a tela 4K tem 33,2 MB, logo abaixo dos 32 MiB do `mmap` do glibc: o reaproveitamento depende de quem largou o quê antes |
| c | journal partilhado com um dono só (sem a guarda strong>1) | moveu 34 MB para o 2.º traço (11 → 45 MB) | `make_mut` passa a mudar o ponteiro da tela, que o guarda de identidade do Wet lê; ficou só o partilhado com strong>1 e no `replace_canvas` |

Herdadas do n.º 5: momentum no bloco, «cura 1b» do `assenta`, tamanho do memo (handoff n.º 5 §1).

## 5. Alarmes de pincel (item 4: FEITO)

Corridos nesta linha e na cópia de segurança `backup/PainterWatercolor-antes-rebase-2026-11-12` (`8246bd897a`) numa worktree temporária (removida): as listas são **IDÊNTICAS** (largura: Oil Wash, Smear Wet; miniatura: a mesma lista, mesmo fnv). **Nada veio do rebase.** O handoff n.º 5 leu mal o «3»: o fecho de 11/11 já tinha 74 miniaturas vermelhas (`…_DESEMPENHO_TOTAL.md` e `…_MISTURA_E_ENVIO.md`), a onda n.º 4 somou 3 (WB, Brush Pen, Mapping Pen) = 77. Só o dono aprova regravar (`PH2D_PINCEIS_APROVA=1`); o relatório é `target/relatorio-pinceis/index.html`.

Outros vermelhos que NÃO vêm desta onda (iguais com as 4 chaves desligadas), origem não investigada (provável mesma família dos 74):
- `tool::paint::tests::watercolor_flow::o_classic_e_o_paper_edge_zero_sao_o_byte_de_hoje` (o byte de «ragged 24 Paper Rough» mudou);
- `tool::paint::watercolor_noise::tests::slot_paper_preset_tiles_seamlessly_under_tiling`;
- `tool::paint::substrate_relief_digital_tests::a_primeira_marca_do_wet_refaz_a_imagem_inteira`.

Defeito de ferramenta: `scripts/cargo-test-narrow.sh` imprime «0 passaram» quando algum teste falha (`cargo test` directo deu 248 passaram, 2 falharam, para `watercolor`).

## 6. ABERTO, por ordem

1. **Matriz do app com calma** (item 3 NÃO feito): `bash docs/Painter/ferramentas/mede_pen_up_4k.sh matriz 3840x2160 papel` e com `NOVA=1`. Load 11–17 a sessão toda, vindo de outras linhas; relógio acima de load ~5 não vale.
2. B19.10 ruído, B20.6 anel da placa (a placa), espera activa de ~10 % do rayon (item 5 NÃO feito; pede calma/placa). B23.2: a parte do Painter já está ligada; o resto (sim do Motion fora do Play) é outro módulo.
3. O «+» a 4K (§1): só com decisão sobre a contabilidade do undo.
4. O vidro por traço = undo em janela (§3): ADR + dono.
5. A espera do pen-down pelo motor (3,8 ms; §1).
6. A sim a 6 traços (0,77 sim-s/s pela MÉDIA): decisão do dono (muda a aritmética da `assenta`).
7. Os 3 vermelhos antigos e os 77 alarmes (§5): decisão do dono.

## 7. Regras aprendidas (novas)

- **«MB novos» por faltas de página (`/proc/self/stat`) contam também faltas de LEITURA da página-zero:** um plano só lido aparece como memória nova.
- **A 4K a tela (33,2 MB) fica logo abaixo do limiar de `mmap` do glibc (32 MiB):** as faltas migram de buffer conforme os `free` anteriores. Meça o gesto TOTAL, não uma fase.
- **Cache chaveada em `Weak` de cada imagem de camada + `PartialEq` da pilha é fresca por construção** (escrita segura não muda o plano no lugar).
- **Chave A×B global em teste precisa de trava:** testes em paralelo viram a rota um do outro.
- Herdadas: a régua do tempo real é a MÉDIA; `Weak` morre quando o `make_mut` MOVE; `cargo fmt --all` reformata ficheiros de outras linhas (formate só os da linha).

## 8. Smoke do dono (UI de cartões, `PH2D_CARDS` por omissão)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && ./target/smoke/ph2d-host-desktop`
2. No Painter, abra ou crie uma imagem de 3840×2160. Escolha o pincel «Watercolor Basic», pinte um traço e depois um segundo. Clique em «+» em Layers e pinte dois traços na camada nova. Repita tudo com «Wet Wash».
3. O que tem de acontecer: o segundo traço e os seguintes começam sem engasgo; Ctrl+Z tira exatamente o último traço; a tinta debaixo da camada nova continua igual.
4. Deu errado se: um traço começa com travada visível, se o Ctrl+Z tira mais ou menos que um traço, ou se a tinta de baixo muda de aspeto depois do «+».

## 9. Prova de fecho

- `clippy -D warnings` limpo em `ph2d-tool-painter` e `ph2d-wet-paint` (todos os alvos, `PH2D_SUITE=fecho`); `rustfmt` só nos ficheiros tocados.
- Verdes por nome: journal 30 · undo 179 · layer 123 · watercolor 248 (+2 vermelhos antigos, §5) · wash 29 · aguada 34 · vidro 19 · backdrop 8 · camada_nova 11 · undo_vivo 11 · mask_gate_copia 2 · chao 2 · ladeira 2 · `compositor::` 60.
- Mutações: «salta sempre» (chão) → vermelho; «sem a identidade das imagens» e «sem a pilha» (fundo em cache) → vermelhos.
- Binário de smoke `target/smoke/ph2d-host-desktop`; 2.ª compilação: `Finished \`smoke\` profile [optimized] target(s) in 0.25s`, zero `Compiling`.
- A suíte completa é do integrador (`scripts/integrar-rodada.sh`).

## 10. Matriz do app (item 3, corrida no fim da janela)

Correu com load 2,7 → 6–8 ([DEPOIS_N6.md](../missao_4k/medicoes/DEPOIS_N6.md)); a n.º 4 correu com load 9–90, logo a
comparação é indicativa. Watercolor Basic: toque 34,9 → 27,1 ms · traço p95 38,2 → 15,1 · soltar 57,4 → 22,9 · Ctrl+Z
36,6 → 15,9 · «+» camada 145,5 → 14,0 · trocar 92,7 → 70,8. Wet Liner: toque 124,2 → 30,0 · Ctrl+Z 100,0 → 48,9.
Filbert: toque 70,1 → 43,6. Continuam vermelhos: CPU parado 52–68 % (B23.2: o laço já dorme no Painter, falta o resto
da shell), trocar/duplicar/apagar camada 33–80 ms, Ctrl+Z do Wet 37–54 ms, «+» do Wet Liner/Express Oil ~110 ms. `NOVA=1`
não corrido.
