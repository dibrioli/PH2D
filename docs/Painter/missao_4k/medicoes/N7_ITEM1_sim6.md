# N.º 7 item 1 — a sim do Wet a 6 traços a 4K: a `assenta` NÃO é o que falta (13/11)

Ordem do dono 13/11 («1- tente»): mudar a aritmética da `assenta` para a sim a 6 traços chegar a ≥ 1,0 sim-s/s pela MÉDIA.
**Resultado: impossível só pela `assenta`.** Com a `assenta` a custo ZERO a média é 27,1 ms = 0,92 sim-s/s.
O produto fica ao bit (nenhuma mudança de física comitada). Por isso não há alarmes de pincel a correr nem fotos antes/depois.

## Régua

`crates/ph2d-wet-paint/examples/sonda_estagios_4k.rs`, 3840×2160, 6 faixas `Lei::Cobre`, 16 fios, `--profile smoke`,
rotas intercaladas no MESMO processo sobre o mesmo estado (clone do grid); média = a melhor ronda.

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && bash scripts/ph2d-run.sh cargo build -p ph2d-wet-paint --example sonda_estagios_4k --profile smoke && PH2D_SONDA_ROTAS=a,z bash scripts/ph2d-run.sh target/smoke/examples/sonda_estagios_4k 6 24 4
```

Rota nova `z` = `MEMO_ASSENTA = 3`: a secagem devolve `sc` sem conta (resultado ERRADO de propósito) = o tecto de qualquer
aritmética da `assenta`.

## Medido (6 traços)

| corrida | load | rota | média ms | sim-s/s | p90 | estágio 1 (secagem) máx | estágio 3 (advect) med | hash |
|---|---|---|---|---|---|---|---|---|
| ANTES (só `a`, 24×3) | 4,8→7,5 | produto | 32,4 | 0,77 | 70,0 | 39,7 | 18,5 | `a9bac64ac5b857c8` |
| A×B 1 (24×3) | 5,1→9,5 | `o` (antes) | 33,4 | 0,75 | 73,1 | 47,5 | 19,4 | — |
| | | `g4` cores na grelha de ¼ de nível | 33,0 | 0,76 | 71,6 | 42,4 | 18,9 | muda |
| | | `g1` grelha de 1 nível | 33,0 | 0,76 | 69,0 | 41,3 | 19,1 | muda |
| | | `t25` Newton até 0,25 de nível (era 0,01) | 32,4 | 0,77 | 73,1 | 45,3 | 20,5 | muda |
| | | `n` = `g4` + `t25` | 32,0 | 0,78 | 67,7 | 45,2 | 20,1 | muda |
| A×B 2 (24×4) | 7,5→6,1 | `o` (antes) | 30,5 | 0,82 | 66,9 | 38,7 | 16,8 | `a9bac64ac5b857c8` |
| | | **`z` tecto: `assenta` grátis** | **27,1** | **0,92** | 45,7 | 14,5 | 16,8 | errado |
| | | `n` | 29,8 | 0,84 | 62,0 | 32,6 | 16,8 | muda |

Leitura: a `assenta` vale ~3,4 ms da média (30,5 → 27,1); a melhor aritmética real tirou 0,7 ms (ruído de carga).
Sem nenhum `assenta` a média continua acima dos 25 ms: o resto é o advect (estágio 3, ~17 ms em TODO passo), o campo de
fluxo (estágio 2, até 10 ms, 1 em 4), o project (estágio 5, até 7 ms, 1 em 3) e o chão da secagem (14,5 ms, 1 em 6).

## ⛔ Recusas MEDIDAS (não reconstruir)

| ideia | medido | porquê |
|---|---|---|
| quantizar `sc`/`uc` do memo (¼ ou 1 nível) | 33,4 → 33,0 ms | os misses do memo não são o custo dominante a esta cena; a secagem fica 41–42 ms |
| Newton com tolerância 0,25 de nível | 33,4 → 32,4 ms | ruído |
| as duas juntas | 30,5 → 29,8 ms | 2 %, muda o pincel sem comprar o alvo |
| qualquer aritmética da `assenta` | tecto 27,1 ms = 0,92 | a `assenta` inteira grátis não chega a 1,0 |

## ABERTO

Para 1,0 sim-s/s a 6 traços faltam ≥ 2,1 ms da média FORA da `assenta`: o advect (16,8 ms todo passo; partes no handoff
n.º 5 §1: momentum 3,4 · núcleo fundido 12,6 · commit 2,8) é a única alavanca do tamanho certo. Não é física da
`assenta`: pede outra decisão (o núcleo do advect ao bit, ou a placa). Os ~10 % de `crossbeam_epoch` em espera ativa
(handoff n.º 5) também continuam não investigados.

## Advecção, cura 1 (13/11): o complemento da troca DENTRO dos blocos — `COMMIT_NO_BLOCO`

Na rota de troca do commit, cada bloco do núcleo fundido copia do grid para o rascunho o complemento das
suas linhas logo depois de as transportar; as linhas fora da faixa copiam-se ao lado do núcleo (`rayon::join`);
o commit vira só a troca dos três planos. O grid e o rascunho finais são os da rota de antes por construção.
Sonda: rota `q` = o commit à parte (o de antes). Carga alta (outro agente): só o A×B vale.

| corrida | load | rota | média ms | sim-s/s | mediana | advect mín (mom · núcleo · commit) | hash |
|---|---|---|---|---|---|---|---|
| 24×4 | 5,5→12 | `q` antes | 33,4 | 0,75 | 29,9 | 3,51 · 13,71 · 2,93 = 20,2 | `a9bac64ac5b857c8` |
| | | **`a` produto** | **32,4** | 0,77 | 29,0 | 3,57 · 14,93 · 0,00 = 18,5 | `a9bac64ac5b857c8` |
| 24×6 | 11,6→10 | `q` antes | 32,6 | 0,77 | 28,0 | 3,44 · 12,55 · 2,81 = 18,8 | `a9bac64ac5b857c8` |
| | | **`a` produto** | **30,7** | **0,82** | 25,2 | 3,44 · 13,65 · 0,00 = 17,1 | `a9bac64ac5b857c8` |

Ganho: ~1,7 ms do advect em todo passo (o commit de 2,8 ms some, o núcleo sobe ~1,1 ms), média −1,0 a −1,9 ms.
Gate por nome `o_complemento_nos_blocos_da_o_commit_a_parte_ao_bit` (troca forçada, blocos 1/3/7/32,
série/paralela/medida, `rf` 1 e 4, o grid escrito FORA da faixa entre passos). Controlos: sem a cauda da linha
e sem as pontas fora da faixa ⇒ vermelho; a primeira versão do gate (cena sem escrita fora da faixa) deixava
a mutação da cauda passar — o rascunho velho já igualava o grid.

### ABERTO para 1,0 (medido com carga alta; reconfirmar calmo)

Média ~30,7 ms ⇒ faltam ~5,7 ms. O resto do advect: momentum 3,4 (passe à parte; no bloco já recusado),
núcleo ~13,6. Perfil (`perf -k`, 6 traços): `transport_row` 13 %, espera do rayon `crossbeam_epoch` 10 %,
`outflow_block` 7,6 %, `flow_at_point` 4,2 % (símbolo próprio — o momentum não o inline), `prepare_row` 4 %.
Próximas a medir: `#[inline(always)]` no `flow_at_point` (ao bit, mesma ordem); os picos (secagem 1 em 6,
campo de fluxo 1 em 4, project 1 em 3) — a média é o relógio, os picos são ~metade do excesso.
Alarmes de pincel NÃO corridos nesta cura: a árvore tinha edições alheias em `ph2d-tool-painter` (outro agente)
que confundiriam a lista dos 77; o motor está provado ao bit pelo hash e pelos gates.
