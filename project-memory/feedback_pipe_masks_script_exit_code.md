---
name: feedback_pipe_masks_script_exit_code
description: "Pipar a saída de um script pelo grep faz o $? virar o do grep — o ship.sh/integrate falha e você lê exit 0; verifique o ESTADO, não o código de saída"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: ac6fba2f-c694-4c47-a142-9e06671dae88
---

**Nunca pipe um script de gate (`foundational-integrate.sh`, `ship.sh`) por `grep`/`sed`
para "limpar a saída".** O exit code que volta é o do ÚLTIMO comando do pipe (o grep), não
o do script. Sem `pipefail`, um gate VERMELHO reporta `exit 0`.

**Como mordeu (integração das 6 linhas, 2026-07-11):** rodei
`bash scripts/foundational-integrate.sh 2>&1 | grep -vE '^\s+(Checking|Compiling)'` para
cortar ruído de compilação. A `line/motion-value` falhou no `nextest-impacted`
(`file_loc_caps`, um arquivo do shell 26 LOC acima do teto) — e eu recebi **exit 0**. Só não
segui em frente porque conferi se o `main` tinha de fato avançado (`git status -sb` continuava
em "ahead 149", o mesmo número de antes). **O estado contradisse o código de saída.**

**Como aplicar:**
- Redirecione para arquivo e capture o código explicitamente:
  `./scripts/ship.sh > /tmp/ship.log 2>&1; echo "EXIT=$?"` — depois `grep` o arquivo.
- **Verifique o EFEITO, não o veredito:** integrou? o `main` andou (`git rev-list --count
  origin/main..main`)? a branch virou ancestral (`git merge-base --is-ancestor`)?
- O `ship.sh` **imprime `✗ NOT CI-clean` e ainda assim sai 0** — o log é a verdade, o `$?` não.

Corolário direto da regra-mãe da DIRETIVA (*verde-de-compilação vale zero*): **verde-de-exit-code
também vale zero**. Ver [[feedback_no_industrial_claims_without_verification]] e
[[project_integrator_ship_catches_latents_budget_iterations]].
