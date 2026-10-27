# Prompt da janela nova — line/PainterWatercolor, 26/10: Óleo Real W1c (de «Eevee antigo» a «Cycles»)

```
Linha line/PainterWatercolor (Modo L) — onda W1c do Óleo Real no Wet Paint: o dono aprovou o rumo da W1b («sombras interessantes entre a tinta! Bom!») e perguntou «porque o render parece mais um Eevee antigo que um Cycles moderno?». Fechar essa distância, medida contra um ORÁCULO que se corre.
FASE 0: cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && pwd && git branch --show-current
Leia: CLAUDE.md (§0, §2), project-memory/MEMORY.md, docs/Painter/49_realshader/PROMPT_NOVA_JANELA_W1c.md e
docs/Painter/49_realshader/HANDOFF_CONTINUACAO_oleo_real_W1b_2026-10-26.md inteiros.
Delegue leitura e verificação.
```

## Estado ao abrir

- Árvore limpa depois do commit destes docs (confirme com `git log -3`). W1b feita: `a1eb3643eb`, `a3095f714f`.
- Captura do dono: `capturas/smoke_W1b_dono_2026-10-26.png`.

## O que separa o nosso render de um Cycles (hipóteses — cada uma MEDE-SE antes de virar lei)

1. **Sem luz indirecta:** uma lâmpada + céu constante. Nas fendas a luz não ricocheteia na tinta vizinha ⇒ preto (os pontos pretos da captura). Cycles: vários ricochetes, a fenda fica no tom da tinta.
2. **Sem ambiente:** o brilho é de UMA lâmpada pontual ⇒ linhas brancas finas e duras. Cycles: o reflexo é do ambiente inteiro, largo e macio.
3. **Sombra de penumbra constante:** uma lâmpada real tem tamanho; a penumbra alarga com a distância ao obstáculo (PCSS).
4. **Brilho cortado a 1** (`clamp`): sem transformação de vista (Filmic/AgX), o realce satura em branco chapado. Respeitar a lei de neutralidade (tinta lisa = a cor dela).

## O trabalho (ordem; série justificada: o oráculo muda a medida de todos os passos)

0. **Oráculo (CLAUDE.md §0.9):** Blender/Cycles corrido por script sobre ENTRADAS NOSSAS — exportar o campo de altura + a cor de uma folha (`medida_oleo_real_folha_png` já tem o campo) como plano com deslocamento, mesma luz (azimute/altitude) e câmara ortográfica; gravar o render como fixture com cabeçalho. **Passo 1 = triagem de licença por artefacto instalado** (`pacman -Qi blender` e bibliotecas). Comparar POR TERMO (difuso, sombra, fenda, brilho) com os mapas `t_*.pgm`.
1. Medir os pontos pretos e as linhas brancas da captura (onde aterra ≠ o que provoca).
2. Rebote: GTAO com multi-ricochete pelo albedo (Jimenez 2016) e um ricochete de cor da tinta vizinha — a fenda no tom, nunca preta.
3. Ambiente (sobe a W2 do plano): irradiância SH9 + especular pré-filtrado (split-sum), HDRI CC0 ou foto NOSSA, triagem por ficheiro.
4. Penumbra que cresce com a distância (lâmpada com tamanho).
5. Rolagem dos realces (tipo AgX) só no que passa de 1.
6. Depois: pirâmide de máximos + passe de placa (`ph2d-render`, paridade com a CPU, K/N por placa), facas e rebordo.

## Regras

- Inner loop `bash scripts/cargo-check-narrow.sh <crate>`; testes POR NOME; clippy só no fecho; suíte completa é do integrador.
- Comando pesado por `bash scripts/ph2d-run.sh …`; `--profile smoke`. Variantes no MESMO processo (`PH2D_OLEO_AF="campo=valor"` na folha; a sonda de acúmulo já corre variantes).
- Olhe a FOLHA (`medida_oleo_real_folha_png`, `magick` para PNG e zoom) antes de mandar o smoke — números sozinhos já aprovaram uma régua cega nesta linha.
- Nada do Rebelle se lê nem se copia; do Blender corre-se o binário, nunca se lê o fonte. Sem push.

## Smoke ao fim (para o dono; passos numerados)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && cargo run -p ph2d-host-desktop --profile smoke`
2. Meio Wet Paint; Oil → Dabber e Oil Bristle; pinceladas grossas empilhadas, uma cor saturada (vermelho) e uma clara.
3. Marque «Real Oil»; mexa Light angle / Light height / Shadow / Gloss / Depth.
4. Compare com `capturas/smoke_W1b_dono_2026-10-26.png` e com o render do oráculo. Errado se: pontos pretos nas fendas, linhas brancas duras, brilho chapado.

**Handoff e prompt seguinte só depois do veredito do smoke.**
