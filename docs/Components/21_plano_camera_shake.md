# Suplente #25 — `CameraShake` + `ShakeEmitter`: **isto explodiu, e a vista tremeu**

> **O último dos suplentes** ([levantamento §7](00_levantamento_componentes.md), 21–25). A folha da
> pesquisa já escreve a entrega e a nota de desenho
> ([`sintese_visual_camera.md` §H.4](pesquisa/sintese_visual_camera.md)):
>
> * *«shake pelo modelo **trauma-com-decay** (amplitude/frequência — nunca brusco nem flutuante)»*;
> * *«**fontes posicionais**: um `ShakeEmitter` na explosão com raio interno/externo e falloff faz
>   tremer mais perto, **sem acoplamento explosão → achar câmera → Shake()**»*;
> * ⚠️ **a nota que governa a wave inteira:** *«shake é OFFSET pós-follow, nunca escreve no
>   `Transform` do alvo nem compete com o follow (o erro clássico)»*.

## §1 — O que a composição JÁ dá (medido em 2026-09-19)

Sonda: [`mede_o_que_a_composicao_ja_da_ao_abanao`](../../crates/ph2d-app-components/tests/it/mede_o_que_a_composicao_ja_da_ao_abanao.rs)
(`--ignored`, imprime). Ela mora na crate de **família** porque é a única que vê os três lados — a
câmera e o gerador (`ph2d-ecs`), a origem de um sinal (`ph2d-runtime`) e o concorrente
(`ph2d-tween`).

| a pergunta | a resposta MEDIDA | o que ela decide |
|---|---|---|
| **a tabela de acções sabe dizer «treme»?** | ⛔ **não** — `9` verbos, e nenhum é da câmera | um sinal chega a toda a parte e não sabe pedir um abanão |
| **um TWEEN sobre a própria câmera exprime-o?** | ⛔⛔ **não, e é o achado**: o tween **escreveu** (`Transform.x = 2,0000`) e o **centro da vista ficou em `[0,0]`** | a vista vem do `CameraRuntime`, que **não é um `Transform`** — nenhum dos `8` canais a alcança |
| **um sinal sabe QUEM gritou?** | ⭐ **sim** em `3` das `5` origens amostradas (contacto · morte · relógio) | ⇒ **a DISTÂNCIA é derivável**: o sujeito tem `Transform`, e o falloff é uma subtracção |
| **há gerador determinístico?** | ⭐ **sim** — *splitmix64*, e ele é **PRIVADO** da fábrica | *um gerador com um consumidor e sem porta é a segunda cópia à espera de ser escrita* |
| **rebobinar já renasce?** | ⭐ **sim** — o censo tocou `2` componentes e **apagou** a `CameraRuntime` | um estado vivo NOVO **tem** de entrar nele |

⇒ **O buraco é o que a nota de desenho já dizia, e agora tem número: nada escreve na VISTA.** O
concorrente mais forte — um tween de pose sobre a câmera — move um `Transform` que ninguém lê.

## §2 — O estado da arte (dos dossiês da própria pesquisa)

| engine | o que entrega | o que fica |
|---|---|---|
| **Cinemachine Impulse** | fonte + *listener*: a fonte levanta um impulso **num sítio**, o ouvinte integra os que lhe chegam | ⭐ **a arquitectura**: quem explode não procura a câmera |
| **Unreal** `UCameraShakeSourceComponent` | raio interno/externo + falloff (Linear/Quadratic) | ⭐ os **dois raios**, que é o vocabulário do artista |
| **Phaser** `camera.shake(duration, intensity)` | abanão **translacional**, global | a forma mínima — e é a que a nossa vista suporta (§5) |
| **Bevy** `bevy_trauma_shake` | trauma com decaimento, **cinco números** | ⭐ o modelo do decaimento, e o **expoente** |
| **Godot** | ⛔ **não tem** (buraco confesso) | — |

⭐ **O modelo de trauma é do Squirrel Eiserloh** (*«Juicing Your Cameras With Math»*): o abanão é
`amplitude × trauma^n × ruído(f·t)`, com o trauma a decair linearmente. ⚠️ **O `n` é o que faz a
cauda morrer** — com `n = 1` o abanão fica a pairar, que é metade do que a folha da pesquisa proíbe.

## §3 — O desenho, com a porta ÚNICA de cada pergunta

| a pergunta | a porta, UMA | porquê |
|---|---|---|
| *quanto abano agora?* | `ph2d_shake::deslocamento` (crate-folha nova, **zero dependências**) | irmã da `ph2d-topdown` e da `ph2d-sweep`; livre de transcendentais (HR-5) |
| *o ruído* | **valor** interpolado com `3t² − 2t³`, sobre o *splitmix64* | ⭐ **contínuo** ⇒ o abanão é facto do TEMPO, nunca da taxa de quadros |
| *o gerador* | ⭐ `ph2d_shake::baralha` — e a **fábrica passa a chamá-lo** | ele já existia privado lá dentro; duas cópias divergem no dia em que uma ganhar um cuidado |
| *quanto chega daqui?* | `ph2d_shake::atenuacao(d, dentro, fora)` | os dois raios do Unreal, com `3t² − 2t³` entre eles |
| *quem gritou, e onde?* | `SignalOrigin::quem()` + o `Transform` dele | **já existe** (medido: 3 de 5) — a wave não inventa canal nenhum |
| *onde o abanão entra na vista* | a fase da câmera da shell, **depois** dos limites | ver §4 |

**Os dois componentes:**

```
CameraShake  { amplitude: f32, frequencia: f32, decaimento: f32, expoente: u8, semente: u64 }
ShakeEmitter { on: String, de: SignalFrom, forca: f32, dentro: f32, fora: f32 }
```

⛔ **E NÃO há um verbo `Shake` na tabela de acções**, com razão: o emissor é a porta, e um verbo
seria a **segunda** maneira de pedir a mesma coisa — sem a distância, que é o que a folha da
pesquisa pede. *Duas superfícies sobre um valor divergem no dia em que uma ganhar um cuidado.*

⭐ **O `de: SignalFrom` é o do suplente #24**, reutilizado: *«só quando fui EU a gritar»* é o que faz
dez bombas iguais não abanarem todas quando uma explode.

## §4 — ⚠️ Onde o abanão entra, e porque não é no `Transform`

A nota de desenho da pesquisa é a lei: **shake é OFFSET pós-follow.** Concretamente, na fase da
câmera da shell:

```
follow → zona morta → amortecimento → LIMITES → ⭐ + o offset do abanão → CameraView.center
```

⚠️ **DEPOIS dos limites, e é uma decisão declarada:** antes deles a cerca **comeria** o abanão na
borda do nível, e o que o artista veria é *«o abanão parou de funcionar aqui»*. ⇒ ele pode mostrar
um fio de fora do nível durante os `~200 ms` de um abanão, que é o que toda engine faz.

⛔ **E ele NÃO toca no `CameraRuntime`:** o estado do *follow* fica intacto, logo o abanão não
realimenta o amortecimento — o erro clássico que a folha nomeia.

## §5 — ⛔ O que fica FORA, com o motivo MEDIDO

| item | porquê |
|---|---|
| **abanão ANGULAR** (o *roll* do Cinemachine) | a `CameraView` tem **três** campos (`center`, `height_world`, `cull_mask`) e **nenhum é um ângulo** — a vista deste app não roda, e inventar a rotação aqui seria uma wave de renderer |
| um verbo `Shake` na tabela de acções | seria a segunda porta para a mesma coisa, **sem** a distância (§3) |
| perfis prontos (*recoil*, *bump*, *explosion*) | são açúcar sobre os cinco números; o tween (#22) já mostrou que um preset se acrescenta depois **sem tocar no motor** |
| o abanão sentir a DIRECÇÃO do impulso | o modelo de trauma é isotrópico por construção; direccioná-lo é outro modelo (o *Impulse* vectorial do Cinemachine) e pede um segundo corpus |

## §6 — Onde isto encosta

* **Contratos congelados (§6 do `CLAUDE.md`):** ⛔ **nenhum**.
* **`PROJECT_SCHEMA`: +1** · registo do `ph2d-ecs` **+2** e os **dois espelhos +2** cada.
* **`LIVE_SECTIONS`: +2** · `ComponentEdit`: **+2** variantes (append-only).
* ⛔ **Zero portas novas no `AppHost`**; ⛔ **zero verbos novos**.
