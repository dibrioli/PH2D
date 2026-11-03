# HANDOFF DE INTEGRAÇÃO — line/PainterWatercolor, 03/11/2026 (cobertura + molhado, Blend no círculo)

Leitor: o integrador. Mecanismo e números da medição: [`MEDICAO_P1_P4_LARGURA_2026-11-03.md`](../../DevOps/MEDICAO_P1_P4_LARGURA_2026-11-03.md) (não repetidos aqui).

## 1. Identidade e ordem

| Campo | Valor |
|---|---|
| Branch / HEAD | `line/PainterWatercolor` · `41c34fe2f6` |
| Merge-base com main | `3a3896a7ab` (leia o `main` no ficheiro; a coluna `base:` abaixo é este merge-base) |
| Commits | 10 (`git log --oneline 3a3896a7ab..HEAD`) · 47 ficheiros · +1026 −275 (31 deles são PNG de miniatura) |
| Ordem | O dono **MANDOU** integrar esta linha ao main (03/11: «SE tudo der certo pode integrar essa linha ao Main»). **Sem push**: a ordem não o pediu. |

Commits (do mais novo): `41c34fe2f6` 3 pinos de fingerprint regravados · `94d0e2848c` 6 vermelhos da varredura das crates · `91072c8bb4` aprovação do dono (miniaturas, larguras, 13 `sem_glaze`) · `613d6b1585` doc da medição P1/P4 · `a76326af2b` saem os 8 irmãos `a_largura_e_a_prometida_*` · `5bdaef3fa3` a lei da umidade · `795920b60f` memória (decisão do dono) · `949a9cc17f` alarme barato + relatório · `500cc8ce70` orla do misturador na grade inteira · `25640bb220` `sem_glaze` 13 cenas.

## 2. O que mudou

| Onde | O quê |
|---|---|
| `crates/ph2d-wet-paint/src/trail/deposit.rs` | A lei da umidade: rampa de 1 célula, a orla só molha (nunca seca o miolo). `DepositLaw::cell` ganhou o parâmetro `wet_atual: u8`; API interna `pub(super)`. Cura 62e7fc8505 (cobertura) mantida. |
| `crates/ph2d-tool-painter/tests/it/presets_de_fabrica/misturador.rs` + ramo novo do gerador em `portoes.rs` | Tabelas dos 4 Blender (Soft/Hard/Dots/Textured) pela mudança visível a 5 % em 14 raios. |
| Tabelas de Fan, Smear Shattered, Smear Wet | Voltaram às de `6cbcce804c` (as regeradas em `0dc6532492` mediam a lei que saiu). |
| `presets_de_fabrica/{alarme,relatorio,umidade}.rs`, `larguras_aprovadas.tsv` | Alarme barato + relatório do dono (P4/P5 do PLANO_ACELERACAO_2026-11). Aprovar: `PH2D_PINCEIS_APROVA=1`. |
| `.config/nextest.toml` | Retirado o override `a_largura_e_a_prometida_*` (−13 linhas); os 8 irmãos saíram em `a76326af2b`. |
| Pinos regravados | fingerprint (3) · `flow_step_bits` (route 1) · `a_secao_desligada` · `com_as_opcoes_desligadas` · `sem_glaze` (13 cenas) · 24 miniaturas · `larguras_aprovadas.tsv`. |
| Memória | `project-memory/MEMORY.md` +1 linha e o ficheiro `..._the_coverage_cure_must_keep_the_old_wetness_2026_11_03.md`. |

Em `crates/ph2d-tool-painter/src` só mudaram 3 ficheiros de testes de `tool/paint/wetpaint/` (`achatamento_tests`, `opcoes_tests`, `shape_color_tests`) — nenhum código de produção fora de `ph2d-wet-paint`.

## 3. Superfície de colisão (colada de `bash <worktree>/scripts/collision-surface.sh`, 03/11)

| Item | Valor | base |
|---|---|---|
| `PROJECT_SCHEMA` / tripla do gate | 215 / (215, 13, 28) | 215 / (215, 13, 28) |
| `VEC_SCENE_SCHEMA` · `FLIP_SCHEMA` · `DOC_VERSION` (timeline) | 28 · 13 · 25 | 28 · 13 · 25 |
| Registo de componentes (`ph2d-ecs` / `ph2d-render` / `ph2d-script`) | 116 / 117 / 117 | 116 / 117 / 117 |
| Contrato congelado (`nodegraph/node.rs`, `editor-core/tool.rs`) | intocado | — |
| ADR | a linha não cria; último 0197, próximo livre 0198 | — |
| `Cargo.lock` | nenhum pacote externo novo | — |
| Marcadores de conflito · tetos de LOC | nenhum · nenhum ficheiro acima do teto | — |

**Contadores como DELTA: todos 0.** Nenhum contrato do §6, nenhum `PROJECT_SCHEMA`, nenhum símbolo/id/variant/token novo. Foundational: nada além do motor (`ph2d-wet-paint`) e dos assets `assets/brushes` (PNG). ⚠️ Prazo de validade: a tabela mede contra o merge-base `3a3896a7ab`; confirme com `git diff --stat 3a3896a7ab..HEAD` e leia o main no ficheiro.

## 4. O que um merge pode partir

- Outra linha que mexa em `trail/deposit.rs` ou chame `DepositLaw::cell` (assinatura nova, `wet_atual`).
- Outra linha que regrave miniaturas, `larguras_aprovadas.tsv` ou pinos hash (fingerprint, `flow_step_bits`, `sem_glaze`): o conflito em PNG/TSV resolve-se pelo lado da linha que tem a lei vigente, nunca por mistura — regravar depois da fusão com `PH2D_PINCEIS_APROVA=1` só com o dono.
- **Atribuição:** estes vermelhos físicos e hashes **já reprovavam no main** com a cura `62e7fc8505`, medidos com a lei que saiu; nenhum entrou na lista conhecida da rodada 02/11c. O main está vermelho neles e **esta linha os cura**.
- `.config/nextest.toml`: o override dos 8 irmãos saiu; se o main acrescentou termos vizinhos, o `scripts/nextest-filtros-casam.py` deve continuar a casar.

## 5. Prova de fecho (MEDIDA, a HEAD `94d0e2848c`, perfil `smoke`, crate inteira)

| Crate | Resultado |
|---|---|
| `ph2d-wet-paint` | 262 ok, 3 red (pinos de fingerprint → regravados em `41c34fe2f6`, 3/3 verdes por nome) |
| `ph2d-tool-painter` | 1974 ok, 1 red de carga (ver §7) |
| `ph2d-wet-gpu` | 3 ok · ignored de placa 8/8 verdes |
| `ph2d-painter-brush` | 559 ok |
| `ph2d-panel-wet-tuning` | 25 ok |
| `ph2d-app-painter` | 145 ok |
| `ph2d-panel-painter-layers` | 322 ok |

Fmt das 3 crates tocadas e clippy `-D warnings` em `ph2d-wet-paint` e `ph2d-tool-painter` verdes. `file_loc_caps` 4/4 · `arch_safe_clamp_only` 2/2 · `architecture_workspace_file_loc_cap` 2/2.

**NÃO corrido (regra do dono 23/10):** a suíte inteira e o `nextest-impacted` — correm AQUI, na integração. Os outros `#[ignore]` (700 em `ph2d-tool-painter`, 61 em `ph2d-wet-paint`, sondas e placa) não correram. ⚠️ A cena completa `ph2d-wet-paint` `tests/it` só correu ANTES da regravação dos 3 pinos; depois, só o filtro `fingerprint`.

## 6. Smoke do dono (03/11) — APROVADO

O dono viu Wet Round/Drip «praticamente idênticos» à cópia antes da cura; «Blend OK»; «Smoke OK». Só depois disso `PH2D_PINCEIS_APROVA=1` regravou miniaturas e `larguras_aprovadas.tsv`.

Para repetir (a cena: formas pintadas, não precisa de cena pronta):

1. `cd /home/enio/Documentos/Projetos/PH2D && cargo run --profile smoke -p ph2d-host-desktop` (o integrador roda no primário, depois da fusão).
2. Painter ▸ Wet Paint: pinte com **Wet Round** e **Wet Drip**; depois **Blender Soft / Hard / Dots / Textured** contra o círculo de referência; depois **Smear Wet** e **Smear Shattered**.
3. Esperado: Wet Round/Drip com o molhado de antes sem perder a cobertura; o Blend cabe no círculo; Smear contínuo, sem emenda.
4. Deu errado se: o Blend passa do círculo, a pincelada do Wet fica com miolo seco, ou o Smear quebra em degraus.
5. Relatório do alarme: `target/relatorio-pinceis/index.html`.

A cópia `Worktrees/painter-atrib-0311` foi removida; os rascunhos estão em `/home/enio/.ph2d-integ-2026-11-02c/painter-atrib-0311-rascunhos.tar.gz`.

## 7. ⚠️ Decisões a saber e o red de carga

Cada uma com a tabela no commit `94d0e2848c` ou no doc da medição.

- `o_rasto_do_smear_wet`: tolerância 1,0 → 1,75 px.
- Faixa 1,3–1,6 do Rebelle saiu da Heavy Edge; agora +0,2 relativo; o teste perdeu `_como_no_rebelle` do nome.
- Limite do miolo do Smooth 5 → 7 %.
- O controlo do escorrimento do S basta numa inclinação.
- A orla de baixo no Cold a meia está mais pesada que antes da cobertura (1,59 × 1,28); o dono foi avisado.
- Rampas 0,5 e 0,25 **recusadas por medição** (consulte a medição antes de propor outra).

**Red de carga:** `o_custo_no_caminho_do_app::o_deposito_de_um_pincel_pesado_de_fabrica_vai_pela_rota_que_paga` (`ph2d-tool-painter`, `tests/it/o_custo_no_caminho_do_app.rs:77`), asserção de relógio. Reprovou na crate inteira com load 5,2→13,6; sozinho 3/3 verde a load 12–15. Candidato à [FLAKES_DE_CARGA](../../DevOps/FLAKES_DE_CARGA.md) se reprovar na integração; **não foi promovido**. `the_cost_of_a_gated_stroke_follows_the_footprint_not_the_canvas` já está na tabela e passou sozinho. A família de carga está no §I do doc de medição.

## 8. ABERTO

| Item | O que fazer |
|---|---|
| `.config/testes-lentos.tsv` (gerado) ainda lista `a_largura_e_a_prometida` (linha 60) e o nome antigo da Heavy Edge | Regerar depois da suíte: `python3 scripts/testes-lentos.py --regerar` |
| Comentário histórico em `.config/nextest.toml` (~linha 95) cita `a_largura_e_a_prometida` | Só texto; o override já saiu |
| Wet Round/Drip a raio 7: ~11 % menos área que antes da cobertura | O dono julgou OK; não é defeito |
| Mensagem do commit `5bdaef3fa3` diz 40 presets; são 45 | Erro só na mensagem |
| P1 (reta + 5 pontos) passa 34/55 | Por isso a linha **não** trocou as tabelas dos outros pincéis |

## 9. Integração (03/11, janela `ph2d-ef`)

Rodada de uma linha: o `main` não andou desde `3a3896a7ab`, sem rebase nem conflito; contadores do `collision-surface.sh` com delta 0. Censos da árvore combinada 132/132 (12 de 12 censos correram). Sem push.

**Suíte inteira a `8f6c0ee266`** (`ci-test`, 1 644 s, load até ~100): 31 443 ok, 3 FAIL, 4 TIMEOUT.

| Vermelho | Causa | Cura |
|---|---|---|
| `o_claude_md_cabe_no_orcamento` | §5 com 14 009 B contra o tecto 14 000: a entrada do Painter cresceu | Entrada encurtada; defeito da linha |
| `a_memoria_do_dabber_cabe` | Régua instável: arenas do glibc, 1 673–2 053 MiB na mesma lei. **Já reprovava no `main`** (1 964) e reprova a load 12, logo não é carga | Filho com `MALLOC_ARENA_MAX=1`; tecto 340/1 950 → 322/1 690 (medido + 5 %), portão mais apertado. Tabela em `memoria.rs`, mecanismo em `FLAKES_DE_CARGA.md` |
| `com_pacotes_perdidos_chega_tudo_e_na_ordem` · `cada_interruptor_que_neutraliza_…` · `cada_knob_do_material_…` · `pular_a_pincelada_…` · `o_oleo_pouco_diluido_…` | Carga: passam por nome | Promovidos à faixa isolada (tabela + filtro); barras intactas |

**Prova depois das curas, por nome, SEM segunda suíte inteira** (as curas são teste, config e docs): os 5 da carga verdes em série na faixa nova (load 10–13) · `a_memoria_do_dabber_cabe` 12/12 em seis cópias paralelas (load até 84).

⚠️ **Não houve uma passada inteira verde.** A árvore final não está no registo de suítes verdes: o `ff` foi à mão, e o `ship` vai correr a suíte. `o_deposito_de_um_pincel_pesado_de_fabrica_…` (o red de carga do §7) passou nesta suíte.

**Continua aberto:** `.config/testes-lentos.tsv` **não** foi regerado. O `junit.xml` da suíte foi sobrescrito pelas corridas por nome, e regerar dele apagaria a lista; regera sozinho na próxima suíte verde.
