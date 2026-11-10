# Briefing do pente fino (missão Painter 4K, §6) — todo auditor lê isto

Leia antes: [`../REGRAS_DO_AGENTE.md`](../REGRAS_DO_AGENTE.md) (formas proibidas, worktree, caminhos absolutos).
**Esta fase é SÓ LEITURA:** nada de cargo, nada de editar código, nada de commit (a máquina está a medir o «antes»;
compilar suja os números). Escreva APENAS o seu ficheiro de resultado, por `cat > … <<'EOF'`.

## O que fazer na sua partição

1. **Inventário:** liste TODOS os ficheiros `.rs` da partição (`find`), com linhas. Ficheiros só de teste/sonda/diag
   (`*test*`, `*probe*`, `diag_*`, `tests/`): uma linha «teste — não auditado a fundo», salvo se forem portões que
   MENTEM (régua enviesada, `#[ignore]` que esconde o vermelho).
2. **Esboço barato de cada ficheiro de produto:** `rg -n` por: laços sobre a tela inteira (`for y in 0..h`,
   `0..width*height`, `.chunks(` de plano inteiro), cópias (`.clone()` / `to_vec()` / `vec![0; w*h]` de planos ou
   imagens), alocação por quadro/dab, `par_`/`rayon`/`Mutex`/`RwLock`/`channel`/`recv`/`join` (espera bloqueante
   dentro de job do pool = deadlock, Bug #66), `HashMap` iterado em ordem (não determinístico), `f32` vs `f64` na mesma
   lei, `powf`/`exp`/`sin` fora do `det-math`, `unwrap`/`expect` no caminho do artista, `#[allow(dead_code)]`, ficheiro
   > 700 linhas, função > 150 linhas.
3. **Leitura a fundo** só dos ficheiros que o esboço acusa e dos que estão no caminho QUENTE (toque, traço por quadro,
   soltar, parado, Ctrl+Z, trocar camada, slider de luz/material, abrir doc 4K). O mapa por fase está em
   [`../fontes/RC_recusas_e_mapa.md`](../fontes/RC_recusas_e_mapa.md) §MAPA DO PIPELINE.
4. **Três lentes:**
   - **CORREÇÃO:** dois lugares que deviam concordar e discordam (prévia × motor, CPU × GPU, painel × tool, gravar ×
     ler, undo × redo, os dois lados de uma troca de documento/camada).
   - **COSTURA DE UI:** pintado ⇒ populado ⇒ clicável ⇒ o clique chega ao barramento ⇒ o leitor DECIDE (controlo morto,
     id órfão, chave gravada que ninguém lê, string fora do `ph2d-i18n`, hex/`f32` literal de UI).
   - **DESEMPENHO:** trabalho na tela inteira por toque/quadro/soltar, cópias, alocação por quadro, um núcleo onde cabe
     o pool, O(camadas × tela), recomputar o que não mudou.
5. **Antes de reportar um achado,** procure-o em [`../fontes/`](../fontes/) (`rg -n '<ficheiro ou função>' …/fontes`)
   e em [`../BACKLOG.md`](../BACKLOG.md) se já existir: se já está listado, NÃO o repita (escreva só «já em <id>» no
   inventário). Consulte as recusas (`RC_recusas_e_mapa.md` §RECUSAS) — não proponha o recusado.

## Formato do resultado

Ficheiro: `docs/Painter/missao_4k/auditoria/<ID>.md` na worktree. Cabeçalho: partição, HEAD (`git -C … rev-parse
--short HEAD`), data, nº de ficheiros e de linhas lidas.

```
## Inventário
| ficheiro | linhas | veredito (ok-esboço / ok-lido / achado AU-<ID>-<n> / já em <id> / teste) |

## Achados
### AU-<ID>-<n> — <título> (lente · gravidade alta/média/baixa · muda aparência ao curar? sim/não)
- Mecanismo: file:line → … → efeito observável.
- Reprodução: passos mínimos (motor ou app).
- Gate que faltava (red-first): o teste que ficaria vermelho.
- Porque o gate existente era verde.
- Cura proposta (1–2 frases) e o custo estimado.

## Refatoração
Ficheiros > 700 linhas ou que misturam responsabilidades: ficheiro → partição proposta (1 linha cada).
```

Seja denso. Achado sem file:line ou sem gate red-first = não é achado (DIRETIVA_IMPLEMENTACAO §3).

**Devolva (≤ 1 200 caracteres):** nº de ficheiros inventariados, nº de achados por gravidade, os 5 achados mais graves
(1 linha cada com o id) e as refatorações propostas (1 linha cada). Prazo: 40 minutos — se não couber, inventário
completo com «esboço» e achados só do que leu a fundo, e diga o que ficou por ler.
