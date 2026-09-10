# O corpus da câmera 2D — 45 corridas do **Godot 4.7.2 (MIT)**, sem interface

> **Atribuição.** A lei de câmera de `ph2d-ecs/src/camera_2d.rs` é portada do **Godot Engine**
> (`Camera2D`), licença **MIT**, medida como oráculo nesta máquina em 2026-09-09
> (`/usr/bin/godot`, pacote `godot 4.7.2-1.1`). Pela triagem do `CLAUDE.md §0.9` uma porta
> permissiva dispensa clean-room: **porta-se, com atribuição** — e porta-se **CORRENDO**, não lendo.

## Como se regera

```
cd <esta pasta>
env TRAJ=<trajetória> SMOOTH=0|1 SPEED=<1/s> DRAG=<fracção> LIMIT=<±L> \
    godot --headless --script oraculo.gd
```

O `LIM_L`/`LIM_R` substituem o `LIMIT` por uma caixa **assimétrica** (é assim que nasce a
`caixa_estreita`). A saída é CSV com **duas linhas de cabeçalho** — a segunda leva os parâmetros e o
`current=true`, e o gate `the_fixture_header_agrees_with_the_table` **confere-a contra a tabela do
teste**: um corpus regerado com outros números reprova em vez de testar, em silêncio, outra lei.

## As 9 configurações × 5 trajetórias

| configuração | o que ela isola |
|---|---|
| `livre` | a câmera cola no alvo (sem suavização, sem zona morta) |
| `suave2` · `suave5` · `suave12` | o amortecimento, em três velocidades |
| `zonamorta` | a janela morta sozinha |
| `zonamorta_suave` | ⭐ **a COMPOSIÇÃO** — foi ela que fixou a ordem das leis |
| `limites` | a janela presa à cerca |
| `tudo` | as três juntas |
| `caixa_estreita` | ⭐ a caixa **mais estreita que a janela** (pino no centro) |

Trajetórias: `degrau` (salto) · `rampa` (velocidade constante) · `vaivem` (⭐ **inverte** — foi ela
que revelou que a zona morta tem acumulador próprio) · `diagonal` (os dois eixos) · `parada`
(anda e pára).

## ⚠️ Três coisas que uma leitura rápida entende ao contrário

1. **Os números estão em PIXELS do oráculo** (janela `1152 × 648` ⇒ meia-janela `576 × 324`), e a
   nossa lei corre em metros. Não há conversão: a lei é **livre de escala** (a zona morta é uma
   fracção, o amortecimento é `1/s`), então o gate alimenta os pixels dele como se fossem metros.
2. **A recorrência é `cam[f] = passo(cam[f−1], mira = alvo[f])`** — o alvo da MESMA linha, não o da
   anterior. Foi medido, não suposto (o `15,400024` do degrau só sai assim).
3. **Nenhuma corrida tem `speed·dt > 1`**, e é de propósito: acima disso o oráculo **oscila e
   diverge**, e nós divergimos dele por decisão declarada — ver a tabela no doc do módulo e o gate
   `we_do_not_diverge_where_the_oracle_does`.
