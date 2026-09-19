# 13 — ⭐⭐⭐ O REBORDO CLARO DE UM PIXEL: o alfa era pré-multiplicado no espaço errado

> Report do dono, 2026-09-19, com foto: *«toda forma apresenta uma falsa outline branca de 1 pixel»*.

## §1 — O que estava errado, e é UMA LINHA em cada motor

A imagem do modelador é entregue ao compositor como **`Rgba8` + `AlphaPremultiplied`**
([`StableImage::from_rgba_premultiplied`](../../crates/ph2d-vector/src/scene.rs) → `VelloPass`), ou
seja bytes em **sRGB já multiplicados pela cobertura**: `byte = sRGB(C)·a`.

Os dois motores produziam outra coisa. Eles faziam a média das quatro sub-amostras da silhueta **em
LINEAR** (o `acc` do `shade_render` e o `pinta_bordas` do WGSL) e só **depois** codificavam ⇒
`byte = sRGB(C·a)`.

⚠️ **A curva sRGB é CÔNCAVA e passa pela origem**, logo `sRGB(C·a) ≥ sRGB(C)·a` para todo
`a ∈ [0,1]`, com igualdade **só** em `a = 0` e `a = 1`. ⇒ *todo pixel de cobertura parcial saía
claro demais, e nenhum saía escuro demais* — que é exactamente a forma de um rebordo claro à volta
de **toda** silhueta, clara ou escura, que é o que ele fotografou.

⭐ **A régua que o mostra é a invariante do FORMATO e não precisa de saber que lei produziu o
pixel:** num pré-multiplicado `rgb ≤ alfa` sempre, porque `C ∈ [0,1]`. Medido na cena `=36` a
`1920×1080`, na banda da silhueta (`8 432` px, `25 296` canais):

| | canais acima do alfa | pior excesso |
|---|---:|---:|
| a lei de ontem | **`1 182`** (`4,7 %`) | **`+73` bytes** |
| a lei de hoje | **`0`** | `0` |

## §2 — ⭐⭐ O CONSUMIDOR foi MEDIDO, e não deduzido

*Qual é a convenção certa é uma afirmação sobre o CONSUMIDOR*, e este repositório tem uma nota
**medida** que responde ao contrário: a
[`ph2d_render::premul::premultiply_rgba8_in_linear`](../../crates/ph2d-render/src/premul.rs)
documenta, por escrito, que pré-multiplicar **em linear** é o que cura um *«light halo at the
silhouette edge»*. ⚠️ Ela descreve o pipeline de **SPRITES**, que tem **dois** consumidores (o shader
com decode de hardware e o Vello) e por isso outra resposta. *Uma lei portada traz a premissa do alvo
sobre a disposição DELE* — a nota não foi acreditada nem contrariada de cabeça: mediu-se **este**
caminho.

[`o_pixel_de_meia_cobertura_aterra_entre_os_vizinhos`](../../crates/ph2d-render/tests/it/o_compositor_e_meia_cobertura.rs)
desenha, pelo `VelloPass` real e pela porta do produto (`draw_stable_image_transformed`), os três
pixels que uma silhueta produz — o fundo, a peça e um de **meia cobertura** — nas duas convenções:

| o que a imagem leva no pixel do meio | o que o compositor devolve (fundo `110` · meio · peça `255`) |
|---|---|
| `188` — `sRGB(C·a)`, a lei de ontem | `[110, **243**, 255]` — **fora** do intervalo dos vizinhos |
| `128` — `sRGB(C)·a` | `[110, **183**, 255]` — entre os dois |

⇒ o compositor faz **`img + fundo·(1−a)` em BYTES**, e `183 = 128 + 110·0,5` fecha a conta ao
inteiro. *Um pixel de meia cobertura tem de aterrar ENTRE os vizinhos; o que aterra fora deles é o
rebordo.*

## §3 — ⛔⛔⛔ A luz ADITIVA não é cobertura, e a primeira cura partiu o chão

A cura ingénua — dividir o `rgb` pelo alfa, codificar, multiplicar — **reprovou dois gates na
primeira corrida**, e os dois tinham razão:

* `a_luz_devolvida_atravessa_o_material_do_chao`: `0,004777` contra `0,011194` esperado.
* `a_cobertura_e_o_canal_mais_forte_e_nao_a_soma` (o brilho).

⭐⭐ **A causa é que nem tudo o que está no `rgb` é cobertura.** A luz que a peça devolve ao chão
(`docs/Render3d/09`) **soma sem tapar**: ela não é `C·a`, e dividi-la por `a` inventa uma cobertura
que ela não tem. Enfiada no mesmo `vec4` que a cor que TAPA — que é o que a
`ground_shade::mais_luz` fazia — o empacotamento dividia-a pelo alfa da **sombra**.

⇒ **duas grandezas, dois argumentos** até ao byte
([`ph2d_field_render::premultiplicado::para_ecra`](../../crates/ph2d-field-render/src/premultiplicado.rs)):

```text
byte = sRGB(C)·a  +  sRGB(L)
```

A `mais_luz` **morreu** e a lei dela mudou de sítio, com a morte escrita onde ela vivia: *uma lei que
precisa de saber qual metade do seu resultado é o quê não cabe num `vec4`*.

⭐ **O alcance da cura é exactamente a população do defeito:** com `a = 255` a divisão e a
multiplicação cancelam-se, com `a = 0` só a luz passa, e com `L = 0` e `a` cheio nada muda ⇒ **a peça
opaca, o fundo limpo e o chão com luz devolvida não mudam um bit**. É isso que mantém de pé todas as
barras de paridade já pagas — e as **24** delas fecharam verdes sem serem tocadas.

## §4 — A prova visual, e porque a FOTO não serve de A/B aqui

⛔⛔ **Tentou-se, e está medido porque não serve:** a cena `=36` tem o prato a rodar, logo duas
fotografias apanham a peça em sítios diferentes; e um detector simples de rebordo apanha a **grelha
do canvas** (`104` sobre `98`) e o **texto do painel**. *Uma prova visual sobre uma cena que se move
não é uma prova.*

⇒ a sonda [`desenha_as_duas_leis_sobre_o_mesmo_cinzento`](../../crates/ph2d-app-field3d/src/premultiplicado_sondas.rs)
compõe o **mesmo quadro do produto** sobre o **mesmo cinzento**, pela lei que o compositor foi medido
a fazer, e centra o recorte no pixel onde o defeito era **pior**. Atravessando a borda de cima da
barra:

| | fundo | **a borda** | peça | peça |
|---|---:|---:|---:|---:|
| ontem | `110` | **`205`** | `214` | `216` |
| hoje | `110` | **`179`** | `214` | `216` |

*O pixel da transição estava a `95 %` do caminho para a peça e passa a estar a `66 %`.*

## §5 — ⚠️ O que esta wave ABRIU e NÃO fechou

⛔⛔⛔ **A premissa da wave do BRILHO (`12` §10) é falsa para este consumidor.** Ela diz que luz
acrescentada com cobertura zero *«é luz multiplicada por nada»* e por isso o halo precisava de
carregar cobertura. Medido agora no `VelloPass` real, um pixel `[128,128,128,0]` sobre um fundo `110`
devolve **`238`** — *ele SOMA*.

⇒ a cobertura que aquela wave acrescentou ao halo **não era necessária para o halo aparecer**, e o
que ela faz é o halo **TAPAR** a grelha em vez de a acender.

⚠️ **Fica como está, e é decisão do dono:** ele aprovou o smoke do brilho com esta lei, e trocá-la
muda o que ele viu. O que a nota do [`brilho.rs`](../../crates/ph2d-field-render/src/brilho.rs) passa
a ser é a **medição ao lado da escolha**, e não um mecanismo por confirmar.

⏳ E fica **por explicar** porque é que o halo era invisível antes daquela wave, já que a soma
funciona: a medição de então contou `2 738 165` canais acesos e a tela não mudou. *Duas medições
honestas que não podem ser as duas verdadeiras é o sinal de que falta um terceiro facto.*

## §6 — ⚠️ O que a prova de MUTAÇÃO apanhou

**5 de 5 sangram** — e a primeira redacção tinha **um sobrevivente**, que nomeou uma cegueira:

⛔⛔ **Devolver a codificação do lado da CPU à lei de ontem deixava o gate da invariante VERDE**,
porque ele corre o caminho do **pintor do dispositivo** (`paint_com`). *Um gate que mede um motor não
diz nada sobre o outro, mesmo quando a lei é a mesma nos dois.*

⭐ **Quem a mata é a PARIDADE** (`a_imagem_do_dispositivo_e_a_da_cpu`), e a cobertura fecha porque as
duas metades se completam: uma regressão **só na CPU** parte a paridade, uma **só no dispositivo**
parte a invariante, e uma **nos dois ao mesmo tempo** parte a invariante na mesma. ⇒ *a prova de
mutação não precisa de um terceiro gate, mas precisava de dizer qual gate testemunha qual motor.*

⚠️ **E o arnês conta a população HONESTA** (`passed + failed` do `test result:`): o `running N tests`
inclui os `#[ignore]`, logo um filtro que casa três ignorados sem `--include-ignored` imprime
`running 3 tests`, corre **zero** e lê-se exactamente como *«a mutação sobreviveu»*.

## §7 — ⛔⛔⛔ O portão do fecho, e a hora que ele custou

O conjunto `--ignored` desta crate fecha **`115` passados, `0` reprovados** em `--release` com a
máquina calma. Antes disso ele reprovou por **uma** cena no
`com_o_dispositivo_a_maioria_das_cenas_e_nitida_em_movimento`, e a leitura dessa vermelha é o que
vale registar — o mesmo commit, cinco vezes, no mesmo dia:

| corrida | perfil | contexto | veredito |
|---|---|---|---|
| conjunto inteiro | release | `load 47` (outra linha a compilar) | `11 de 22` ✗ |
| sozinha | **debug** | `ociosa 68 %` | `8 de 22` ✗ |
| sozinha | **debug** | `ociosa 81 %` | `10 de 22` ✗ |
| sozinha | **debug** | `ociosa 98 %` | `9 de 22` ✗ |
| conjunto inteiro, máquina calma | release | `load 7` | **`115` ✓** |
| sozinha, a seguir ao próprio build | release | `load 20,68` · `ociosa 92 %` | `10 de 22` ✗ |

⭐⭐⭐ **A leitura mais CALMA das três do meio foi a pior, e a causa era o perfil:** ao comando
faltava `--release`, e o `cargo test` recompila em **silêncio**. ⚠️ Pior: a tabela por cena imitava a
assinatura da flake do §5.0 — *a vítima muda entre corridas* —, porque uma cena de `93` instruções
mede `14 ms` em debug e `58` numa release contendida, e outra de `934` faz o contrário. **O que
separou as duas famílias foi o CABEÇALHO do log, não os números** (`Finished 'release' profile …`
contra `Compiling …`).

⇒ a régua de calma passou a ser uma **porta** (`contexto()`) que nomeia carga, ociosidade **e
perfil**, lida pelos **nove** sítios de três famílias de sondas que escreviam a linha à mão — e o
gate leva a tabela acima no doc-comment, com a lei que dela sai: *nenhuma das duas réguas de calma
discrimina sozinha (`ociosa 92 %` reprovou, `load 7` passou), porque o que atrasa é a **cauda** do
que acabou de correr, e só a `loadavg` ainda a vê.*

⚠️ **E o `-D warnings` apanhou um `HashSet` meu** num dos dois gates re-apontados pela §5 — tipo
**proibido** (HR-5 · ADR-0022). A cura é melhor do que a régua era: uma máscara `vec![bool]`
indexada pelo pixel, que responde sem procurar.

## ⛔ Recusas MEDIDAS

| o que foi recusado | porquê, com o número |
|---|---|
| acreditar na nota da `premultiply_rgba8_in_linear` | ela descreve o pipeline de SPRITES, com DOIS consumidores; medido neste caminho, a resposta é a oposta (§2) |
| pré-multiplicar o `rgb` inteiro pelo alfa | a luz devolvida ao chão SOMA e não tapa: o gate do chão leu `0,004777` contra `0,011194` (§3) |
| curar o brilho na mesma wave | a lei dele foi aprovada pelo dono um dia antes, e a premissa que ela contraria é agora uma decisão de produto e não um defeito (§5) |
| usar a FOTO como A/B do rebordo | o prato roda entre as duas corridas, e o detector simples apanha a grelha do canvas e o texto do painel (§4) |
| medir o rebordo desfazendo `sRGB(C·a)` | *a régua supunha a fórmula do defeito*: depois da cura ela continuou a imprimir `+15,8` sobre uma imagem correcta (§1) |
