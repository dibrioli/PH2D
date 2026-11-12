# Prompt de continuação — linha PainterWatercolor · 12/11 n.º 2: acabar o desempenho do Painter e a luz única do Óleo Real

`/pd-linha-assumir PainterWatercolor docs/Painter/handoffs/PROMPT_CONTINUACAO_2026-11-12_desempenho_2.md`

Worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor`, branch `line/PainterWatercolor`.
`cd` + `pwd` + `git branch --show-current` ANTES de ler. O main não andou (merge-base `bc4a6ce930`): não faça rebase.

## Ordem do dono (verbatim)

11–12/11: «Se esforce para resolver todos os problemas de performance do módulo a todo custo. Se necessário, auditoria
exaustiva. Não esqueça do handoff de integração ao final de tudo. Não integre. O integrador será estabelecido por mim.»
12/11: «A próxima janela deve ir até o fim da implementação sem parar. Ao final a próxima janela deve escrever o handoff de
integração pois outra linha vai integrar.» · 12/11 (Óleo Real): «a luz do traço (luz simplificada) tem melhor aparência que a
luz final (no mouse up). Podemos então deixar a luz do traço como a única luz?» → feito (memória
`project_real_oil_one_light_the_stroke_light_2026_11_12.md`).

## Leia primeiro (nesta ordem)

1. [HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-12_DESEMPENHO_TOTAL.md](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-12_DESEMPENHO_TOTAL.md)
   — o que a janela anterior fez (52 commits), a matriz final, a prova de fecho, §7 decisões do dono, **§8 ABERTO por ordem**.
2. [PLANO_DESEMPENHO_TOTAL_2026-11-12.md](../missao_4k/PLANO_DESEMPENHO_TOTAL_2026-11-12.md) — as famílias e o estado.
3. `git log --oneline f755413923..HEAD` — a luz única (depois do handoff): `055544c67e` (Simples em todo instante, sem a
   dívida Cheia), `ca013b8987` (B14.2 decidido), `ffe143a592`, `7359c0095b` (sai `QualidadeDaLuz` e o código da Cheia; hashes
   de 4 pincéis de óleo iguais), `baef79929a`.

## Estado da luz única (12/11, ~07:00)

Os portões de defeito da LUZ que mediam a Cheia passaram a medir a luz do traço (a do produto desde `055544c67e`) e
reprovam — NÃO afrouxados:
1. `o_papel_ao_pe_da_parede_perde_o_ceu_do_lado_dela` (wet-paint): Estúdio 0,0003 × Clássico 0,071 à direita da parede.
2. `a_orla_rasgada_nao_tem_pontinhos_escuros` (wet-paint): Estúdio, 1 pontinho escuro.
3. `a_vista_a_400_por_cento_tem_o_detalhe_que_a_ampliacao_nao_tem` (wet-paint): 2,99× contra os 3× do dono.
4. `o_papel_nos_vaos_entre_lascas_nao_fica_preto` (tool-painter): Clássico 0,11 % dos vãos abaixo de 70 (tecto 0 %).
Causa medida de 1 e 2: com 2 fatias o céu HDRI lê-se em só 4 cunhas de 1/8 — metade das direcções nunca é amostrada;
`CUNHA = 1/4` (oleo_horizonte.rs) pôs 1 e 2 verdes (revertido; muda os bits do HDRI). 3 e 4 sem cura achada.
**Classificação A/B/C dos vermelhos** (A = `f755413923` antes da luz única · B = `acb4e537f3` · C = `7836787a2d`):

| teste | A | B | C | 1.ª linha vermelha |
|---|---|---|---|---|
| `cada_campo_do_pincel_vai_para_o_preview_ou_diz_porque_nao` (censo do preview) | ✗ | ✗ | ✗ | `preview_censo_tests.rs:426` — já vermelho ANTES: classificar contra `4381ee6d4a` e o main |
| `o_impasto_nao_encaroca_em_tamanho_nenhum` | ✓ | ✗ | ✗ | `sonda_oleo_estria.rs:391` raio 16: 12 pretos (0) — **causado pela luz única** |
| `o_modo_blender_casa_com_o_cycles_e_o_clear_glass_e_o_de_antes` | ✓ | ✗ | ✗ | `medida_transparencia.rs:560` hash do Clear glass — mudança de lei (regravar com nota, como `a5fe97a911`) |
| `o_oleo_pouco_diluido_cruza_o_amarelo_no_azul_em_verde` | ✗ | ✗ | ✗ | decisão do dono (Filbert Dry, §7 do handoff) |
| `o_papel_nos_vaos_entre_lascas_nao_fica_preto` | ✓ | ✓ | ✗ | `medida_oleo_real_fenda.rs:211` (portão 4) |
| wet `o_papel_ao_pe_da_parede_perde_o_ceu_do_lado_dela` | ✓ | ✓ | ✗ | `oleo_real_tests.rs:579` (portão 1) |
| wet `a_orla_rasgada_nao_tem_pontinhos_escuros` | ✓ | ✓ | ✗ | `oleo_borda_w3b_tests.rs:137` (portão 2) |
| wet `a_vista_a_400_por_cento_tem_o_detalhe_que_a_ampliacao_nao_tem` | ✓ | ✓ | ✗ | `oleo_vista_tests.rs:59` (portão 3) |

Os portões 1–4 só ficam vermelhos em C porque até B eles construíam `OleoReal::default()` = Cheia; o PRODUTO já usava a
luz do traço desde B — o defeito que medem já está no app. O `o_impasto_nao_encaroca` (12 píxeis pretos a raio 16) é um
defeito VISUAL novo da luz única no produto: prioridade 1. Logs: scratch da janela anterior, `<teste>.<sha>.log`.

## O que fazer, por ordem (até ao fim, sem parar)

1. **A luz do traço sem defeitos:** amostrar o céu inteiro (cunha 1/4 ou o que a medição mostrar — padrão-ouro, o
   acabamento é técnico) e curar 3 e 4 com a medição; os vermelhos que esta mudança causou fora da luz (tabela acima) —
   regravar só o que for expectativa da lei antiga com nota, nunca afrouxar portão de defeito. Foto antes × depois do
   Óleo Real (gesto = repouso) para o smoke do dono.
2. **§8 do handoff, por ordem de impacto:** W5 (o depósito fora do quadro — fila de traços sobre o `offthread` do Wet; os
   cortes ao bit que restam) · Ctrl+Z do Wet/Óleo (`FotoDoMotor::de`, chunk esparso como tile) · a água com Óleo Real (o
   campo por linhas) · camadas (B18.1 cópia sob demanda, B18.2 salvar por empréstimo, B19.10) · B21.1 Mailbox · B27.1 fila
   da caneta · B42.3/B17.9 memória do undo · B20.6/B20.11. O piso por quadro da UI (a11y/BTreeMap/Quadro) é da linha da UI:
   só o handoff.
3. Cada família: medir antes, curar ao bit (ou com decisão do dono registada), medir depois no MESMO binário/target, comitar
   com a tabela. Pode delegar a agentes em worktrees próprias (`.ph2d-ag-<x>`, target próprio, prazo 45 min, cherry-pick -x),
   instruções como métrica quando a máquina está carregada (outra sessão integra: load até 220 em 12/11).
4. **Fecho:** bateria por nome (fmt/clippy `PH2D_SUITE=fecho` das crates tocadas, tectos LOC, `the_shell_only_shrinks`,
   alarmes de pincel — largura: Oil_Wash, Smear_Wet; miniatura: os 74 de sempre; nunca regravar sem o dono), uma matriz final
   base `4381ee6d4a` × HEAD (roteiro `matriz2.sh` descrito no handoff), **handoff de integração NOVO**
   `docs/Painter/handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-12_<TEMA>.md` (outra linha vai integrar: o leitor
   é o integrador), link no CLAUDE.md §5.1, `rm -rf target/*/incremental`, binário smoke compilado 2× (2.ª saída no handoff),
   limpar `.ph2d-ag-*` e targets. Não integre, não envie.

## Regras

RTX proibida nas medições (Radeon: `VK_DRIVER_FILES=/usr/share/vulkan/icd.d/radeon_icd.json VK_ICD_FILENAMES=…`); pesado por
`bash scripts/ph2d-run.sh`; testes por NOME; o `perf` aqui não grava pilha (paranoid 2) — binário com símbolos num target à
parte e `perf annotate -l`; laços de medição em ficheiro bash (zsh não parte `"a b"`). Resposta ao dono: curta, PT-BR, sem
jargão, smoke em passos numerados com o `cd`.
