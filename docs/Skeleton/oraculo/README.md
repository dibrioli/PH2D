# O ORÁCULO DOS PESOS — a triagem que fechou a porta, medida

> **Pergunta, escrita antes de correr** (protocolo `_ComoInvestigarApps` §3.2):
> **«O Godot CALCULA os pesos da pele 2D, ou só os APLICA?»** — e, a seguir,
> **«os pesos automáticos da referência dobram menos que os nossos?»**
>
> ⚠️ Cada ficheiro aqui é a ENTRADA e a SAÍDA de uma corrida real, com a
> proveniência no cabeçalho. A prosa nunca é a fonte.

## Passo 1 — a TRIAGEM, e ela parou em duas portas fechadas

| app | licença | veredito **medido** |
|---|---|---|
| **Godot 4.7.2** | MIT (⭐ portar) | ⛔ **Não calcula pesos nenhuns.** O `Polygon2D` só tem `get/set_bone_weights`; o `Bone2D` só auto-calcula *comprimento e ângulo*; o `Skeleton2D` não tem nada. E a única acção do binário é **`Paint Bone Weights`** — o artista pinta-os à mão (censo: `strings` sobre `/usr/bin/godot`, zero acertos em `auto.?weight\|weight.*calc\|calc.*weight`). ⇒ ele está **onde nós estamos**, menos o automatismo. |
| **OpenToonz 1.8.0** | BSD-3 (⭐ portar) | ⚠️ **Tem a coisa e não tem porta.** A `libtnzext.so` traz `PlasticSkeletonDeformation` sobre uma malha de `PlasticSkeletonVertex`, com `tcg::Vertex<RigidPoint>` — o vocabulário da família *rigid/ARAP*. ⛔ Mas o app **não tem consola** (medido no arsenal: o `--help` abre a GUI), logo ele **não se corre sobre a nossa arte**. O valor dele é o FONTE, que é permissivo — outra wave, outro método. |
| **Blender 5.2.1** | GPL (⛔ parede para o fonte) | ⭐ **É o único que se CORRE.** Ele calcula pesos automáticos (difusão de calor) e a saída sobre a **nossa** malha é livre — posições e pesos de uma malha nossa não são obra baseada no programa. |

## Passo 2 — a corrida, sobre a NOSSA malha

`oraculo_pesos.py` constrói **a nossa** grelha (`48 × 48` sobre a caixa da
fixtura da régua da dobra: arco `4`, meia-altura `2,4`) e **o nosso** esqueleto
(dois ossos de `2` ao longo de `+x`), aplica *Parent → With Automatic Weights* e
despeja os pesos por vértice.

```
blender -b -X -P oraculo_pesos.py -- pesos_blender.json
python3 compara.py pesos_blender.json
```

⚠️ **`-X` (factory startup)**: sem ele o oráculo herda as preferências desta
máquina, e a corrida deixa de ser reproduzível noutra.

## Passo 3 — o resultado, os dois lados pelo MESMO código

`compara.py` deforma a **mesma** malha com a **mesma** lei de mistura, variando
**só os pesos**, e conta a área virada do avesso (o sinal da área de cada
triângulo contra o repouso):

| dobra | NOSSO (alcance = osso) | **BLENDER** (calor automático) | NOSSO (alcance `2,08 ×` a arte) |
|---:|---:|---:|---:|
| `60°` | `0,434 %` | **`0,000 %`** | **`0,000 %`** |
| `90°` | `0,651 %` | `5,273 %` | **`0,000 %`** |
| `120°` | `1,107 %` | `9,397 %` | **`0,000 %`** |
| `150°` | `2,300 %` | `8,876 %` | **`0,000 %`** |

⭐⭐⭐ **A cura que a nossa própria medição deu bate o oráculo em toda a escada.**

## Passo 4 — e o PASSO A PASSO diz porquê (§3.5)

O peso do 1.º osso ao longo da linha do meio da arte:

| `x` | BLENDER | NOSSO (alcance = osso) | NOSSO (`2,08 ×`) |
|---:|---:|---:|---:|
| `1,70` | `1,0000` | `0,5114` | `0,5018` |
| `2,00` (a junta) | `0,5000` | `0,5000` | `0,5000` |
| `2,30` | `0,0000` | `0,4886` | `0,4982` |

⇒ **os pesos do oráculo são uma FUNÇÃO ESCADA** nesta fixtura: `1` até à junta,
`0` depois. Maior gradiente na vertical: **`0,652`** (oráculo) · `10,000`
(nosso, com o salto do ponto órfão) · **`0,045`** (nosso, alcance curado).

⚠️⚠️ **E isto é uma afirmação sobre O NOSSO REGIME, não sobre a difusão de
calor.** Ela foi desenhada para uma superfície 3D que **envolve** o osso; aqui a
arte é uma folha plana e os ossos vivem **dentro do plano dela**, então cada osso
«vê» só a metade mais próxima e a partição sai dura. ⇒ *o método não é mau — ele
não é do nosso meio*, e é isso que fecha a porta.

## ⛔ Recusas MEDIDAS

| recusa | mecanismo |
|---|---|
| **Portar os pesos do Godot** | Não há nada para portar: ele não os calcula. |
| **Correr o OpenToonz como oráculo** | Não tem porta de consola (medido). O fonte é BSD-3 e continua aberto — mas isso é *portar*, não *medir*. |
| **Adoptar os pesos automáticos da referência** | Na nossa fixtura eles dobram **`5`–`9 %`** da arte acima de `60°`, contra `0,65`–`2,3 %` dos nossos e `0 %` do alcance curado. A causa está no passo 4: no nosso meio eles degeneram numa partição dura. |
