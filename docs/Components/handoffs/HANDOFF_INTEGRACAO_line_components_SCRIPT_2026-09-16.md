# HANDOFF DE INTEGRAÇÃO — `line/components` · TOP-20 **#16 `ScriptProperties` + anexar um script pela UI** · 2026-09-16

> Plano e medições: [`13_plano_script_properties.md`](../13_plano_script_properties.md).
> ⚠️ Leia o **§6** (o que o diff inverte) e o **§7** (as premissas que a medição derrubou) antes de
> ler o diff.

---

## §1 — O que o ARTISTA consegue fazer agora

Escrever **um** ficheiro `.luau` que declara os seus números no topo (`ph2d.property("speed", 2)`),
anexá-lo a N objectos por **Add Component → Script → Browse**, dar a cada objecto **os seus próprios
números no Inspector** (campo · caixa · texto, pelo tipo do default), e carregar em Play: cada
objecto mexe-se pelo mesmo script com os números dele. Gravar o ficheiro num editor de texto muda a
cena **sem reiniciar** (e só nos objectos que não têm número próprio). Um script emite sinais
(`ph2d.emit`) que a tabela de acções do #5 já sabe ouvir, e ouve os da cena (`on_signal`).

---

## §2 — Os contadores, como DELTA (⛔ nunca o literal)

| contador | delta desta wave | nota |
|---|---|---|
| `PROJECT_SCHEMA` | **+1** (133 → 134 nesta árvore) | ⚠️ **nenhum tipo novo**: o `LuauScript` existia e o registador dele **não corria no boot** — entrar no `build_component_registry` é o que muda o ficheiro. Degrau na escada **e** a tripla actualizada (*a tripla não vê este degrau, 18.ª vez*) |
| registos `ph2d-ecs` / `ph2d-render` / `ph2d-script` | **0 / 0 / 0** | o espelho do `ph2d-script` **já** contava o `LuauScript` (`ecs + 1`) |
| registo do BOOT (shell) | **+1 registador** | `register_script_components` — e o mesmo no espelho das fixturas (`component_registry_for_tests`), que o gate dele exigiu |
| `LIVE_SECTIONS` | **+1** (26 → 27) | a secção SCRIPT, no MESMO commit |
| `any_live_section` | **+1** (`[bool; 21]`) | a pergunta «há secção?» |
| `SignalOrigin` | **+1 variante** | `Script { source }`, **append-only** |
| `preview_drive::Driver` / `Driven` | **+1 variante cada** | `ScriptPose` — ⛔ não o `SolverPose` (ver §6.4) |
| `EditorAction` | **+1 variante** | `InspectorScriptEdit` |
| catálogo | `LuauScript`: **`Machinery` → `Authored`** (`O::ANY`) | e o registo passa a `register_default` |
| dependências | `ph2d-script`: **−`ph2d-asset`**, +`postcard` (dev) · `ph2d-app-components`: **+`ph2d-script`** | o `mlua` já era compilado pela shell |

⚠️ **O `PROJECT_SCHEMA` desta LINHA está `+6` contra o `main` de hoje** (128 → 134), somando as waves
ainda não integradas. ⛔ *Conte o DELTA contra a árvore em que vai aterrar.*

⚠️ **Tecto da shell:** `196 245` antes desta wave, **`196 498`** depois (**+253**, 991 ficheiros; o
`main` está em `196 003`), contra `196 990` ⇒ **492 de folga**. O corpo mora em `ph2d-app-components`
(a ponte, o instantâneo, o dreno, a cena); a shell só compõe — o que ela ganhou é a canalização de
uma secção nova (o campo do dreno, o braço do barramento, a publicação do instantâneo), o bloco dos
scripts dentro do `fase_signal_outbox` e o prólogo da cena. ⚠️ **Folga que SOMA entre linhas:** outra
linha que feche no mesmo dia com +500 reprova o `the_shell_only_shrinks` na árvore combinada.

---

## §3 — ⛔⛔ O que o §1 do plano mediu: o levantamento estava errado nas TRÊS palavras

O levantamento dizia *«host, persistência e determinismo prontos, ZERO UI»*. Medido pela leitura da
crate e dos chamadores:

| afirmação | o que havia |
|---|---|
| **persistência** | o registador **não era chamado no boot** ⇒ um `LuauScript` evaporava ao gravar e ao desfazer (o catálogo já o dizia) |
| **host** | **uma** VM a correr um *placeholder*; nenhum `init`/`update` por objecto; o `WriteQueue` **sem dreno** |
| **determinismo** | ⛔⛔ `lateral_key = entity.to_bits() ^ hash` — **bits de alocação DENTRO dos bytes de um componente**, o veneno do undo que o §5 proíbe. Só não mordeu porque o componente nunca foi gravado |

⇒ o componente muda de forma (`{ bytecode, lateral_key }` → **`{ source, own }`**) **sem degrau de
migração**: a forma velha nunca chegou a um ficheiro.

---

## §4 — O ORÁCULO (§0.9), as dez leis e as três divergências

**Godot 4.7.2 (MIT)**, `@export`, corrido sem interface por
[`godot_export_probe.gd`](../ferramentas/godot_export_probe.gd) — grava com o script v1, reescreve o
**ficheiro** do script, recarrega (`CACHE_MODE_IGNORE_DEEP`). **Controlo C0 verde.**

Portadas: **Q1** (sem valor próprio segue o default) · **Q2** (com valor próprio guarda-o) · **Q6** (a
faixa é pista de edição) · **Q7** (a ordem é a da declaração) · **Q9** (instâncias independentes) ·
**Q10** (script desconhecido ⇒ os valores ficam).

⛔ **Três divergências DECLARADAS**, cada uma contra uma perda silenciosa do alvo, cada uma com gate:

- **D1 (Q3)** — no alvo, um valor **igual ao default** não é gravado, logo segue o default seguinte.
  Aqui *próprio* é o que o artista **PÔS** (a lei da casa: o discriminador é quem pôs).
- **D2 (Q4b)** — no alvo, re-gravar a cena com a propriedade ausente **apaga** o valor, e quando ela
  volta lê-se o default. Aqui o órfão **fica**, nomeado, com `Remove`.
- **D3 (Q5b)** — no alvo, `"rapido"` numa propriedade que passou a número lê-se **`0`**. Aqui o do
  tipo errado não se aplica nem se converte.

---

## §5 — As provas

- **Gates novos:** 20 na lei e na colheita (`props`, `module`) · 17 no executor (`scene`) · 2 no
  componente · 9 na ponte contra o ledger REAL (com a `settle` a cada quadro) · 9 no instantâneo e
  no dreno · 7 na cena · 5 de costura com o **gesto real** no painel · 2 de ordem/chamador na shell
  · 2 de «o painel pinta tudo o que o modelo aceita» (um deles **devido** pelo #15, ver §7).
- **Provas de mutação:** [`mutacao_script_2026-09-16.sh`](../ferramentas/mutacao_script_2026-09-16.sh) —
  **33 de 33 sangraram**, cobrindo a lei · a colheita · o executor (ordem, renascer, campo
  desconhecido, `NaN`, o PRAZO no gancho e no topo, a recarga, a re-aplicação só quando muda) · o
  host · a ponte (o `driven(agora, agora)`, o rebobinar) · o dreno · o `populate` · a semente · o
  evento · o pintor · e as **três costuras da shell** (o leitor próprio do outbox, o renascer no
  invariante do transporte, o `PROPS_MAX` contra a tabela de ids). O arnês tem **controlo sobre o
  próprio filtro** (`running N tests`, reprova em `N = 0`), âncoras com contagem, `timeout` nas
  provas do prazo (rc `124` conta como sangue) e `touch` depois de cada restauro.
- ⭐⭐ **A cena foi FOTOGRAFADA antes de ir ao dono**, numa tela virtual:
  [`fotografa_cena.sh`](../ferramentas/fotografa_cena.sh). A foto apanhou **três** defeitos que
  nenhum gate via (§7.5).

---

## §6 — As leituras que o diff inverte

1. **O `bytecode: AssetId` e o `lateral_key` SAÍRAM de propósito** (§3) — não é uma regressão do
   M14.2. ⚠️ O `ph2d.attach_script(entity, hex)` do spike **continua** a empurrar um `AssetId` para
   uma fila **sem dreno** (o gate `the_intent_drain_reaches_every_variant` di-lo): fica nomeado no §9.
2. **Um `ScriptHost` por app, UM AMBIENTE por script** — com a sandbox do Luau, dois
   `function update` escreviam no mesmo sítio (gate `dois_scripts_nao_se_pisam`).
3. **`update` é chamado UMA vez por PASSO FIXO**, nunca `ticks × dt` — ao contrário da lei do relógio,
   um script não tem laço de recuperação.
4. ⭐ **`Driver::ScriptPose` e não `SolverPose`** — a chave do ledger é `(entidade, driver)`; um
   objecto que seja também um corpo teria duas mãos na mesma entrada (a razão do `PrefabStage`).
5. ⭐⭐ **A ponte declara `driven(agora, agora)` nos quadros em que o script NÃO escreveu** — e com a
   corrida PAUSADA. É a linha que impede a `settle` de promover a pose da corrida a documento (um
   script que PAROU de mexer não ACABOU), e a que faz um **arrasto na pausa** ser adoptado como o
   novo autorado. Só quando **já havia entrada** (`still_driving` nunca cria).
6. **Recarregar NÃO renasce** — as funções mudam, cada `self` fica, e um script partido volta a
   correr. **Rebobinar** é que renasce (o `init` corre outra vez) e devolve a pose (`release_to_authored`).
7. **As escritas aplicam-se no FIM da passagem** e todos leem a fotografia do início (HR-8) — escrever
   e ler o próprio `x` no mesmo gancho lê o valor de antes.
8. **O bloco dos scripts vive DENTRO do `fase_signal_outbox`**, e não numa fase-filha: o outbox segura o
   `sim` do princípio ao fim, e uma fase-filha só cabia depois da tabela de acções (§7.3).
9. **O instantâneo da secção é publicado à parte** (ao lado do painel de tags), porque só ele precisa
   da VM e o `publish` do Inspector não a recebe.
10. **«Não sei o que o script declara» ≠ «não declara nada»** — com o ficheiro sumido ou partido os
    valores contam-se como **guardados** e não se oferecem para apagar.

---

## §7 — As premissas DESTE plano que a medição derrubou

1. ⛔ *«o Godot guarda o valor como o artista o escreveu»* — o Q3 desmentiu-a (daí o D1).
2. ⛔ *«o `SignalOrigin` e o `Driver` fecham por `match` fora das crates deles»* — medido: o `Driver`
   não tem `match` fora; o `SignalOrigin` tem dois, exaustivos, e o compilador apontou-os.
3. ⛔ *«a ponte é uma fase-filha própria»* — ver §6.8.
4. ⚠️ **O registo das fixturas da família é um espelho do boot** — o gate dele reprovou com os dois
   nomes lado a lado até o registador entrar lá.
5. ⛔⛔ **Três defeitos só a FOTO viu**: os botões `Reset`/`Remove` liam-se `…` (largura fixa em passos
   de espaçamento → hoje MEDIDA do rótulo); a linha de uma caixa tinha **dois** pontos (o
   `paint_checkbox` já reserva o dele); e a cena **não cabia** no ecrã do dono (bonecos a ±6 m, lâmpada
   a `y = 5` → hoje ±4 m, lâmpada logo acima do rápido, conferido a 2560 e a ~1930 de largura).
6. ⛔ **Um gate que uma afirmação MINHA prometia e não existia:** o cabeçalho dos ids do CÉREBRO (#15)
   dizia que *«um gate na shell»* amarrava `STATES_MAX` à tabela de linhas. Nasceu nesta wave
   (`o_painel_pinta_todo_o_modelo_aceita`), junto com o do script.
7. ⛔⛔ **Processo:** a 1.ª tentativa de foto usou o `spectacle`, que fala com o KWin pelo D-Bus da
   sessão REAL — fotografou o ecrã do dono (outra janela do PH2D e o editor). A imagem foi apagada sem
   ser usada; o instrumento **recusa** `DISPLAY=:0` e lê só a Xwayland da sessão virtual. ⚠️ E os
   eventos sintéticos (XTest) **não chegam** àquela Xwayland; o `ydotool` moveria o rato REAL — o
   clique prova-se no gate de costura.
8. ⚠️ **O portão de fecho apanhou TRÊS vermelhos meus que os `check` e as suítes das crates editadas
   não viam** — o terceiro só **depois** de curar o segundo, porque o clippy pára na crate que não
   compila e **não chega às que dependem dela** (`ph2d-app-components` depende da `ph2d-script`):
   um `assert!` sobre constantes da cena (os bonecos não se sobrepõem) ⇒ `const { assert!(..) }`,
   que é mais forte — uma disposição que os encoste deixa de compilar. Os outros dois: a `fase_inspector_commits` foi a **`203`** contra o tecto de `200` por **função** (o
   dreno do script com o diálogo inline — o gate `fn_loc_caps` vive em `shells/desktop/tests/it/`) ⇒
   **fase-filha** [`fase_script_commits.rs`](../../../shells/desktop/src/render_loop/fase_script_commits.rs),
   a sétima irmã do mesmo molde, ⛔ nunca uma entrada no `FN_OVERAGE_OK`; e o `clippy::map_entry` no
   nascimento de uma instância (`contains_key` + `insert` + `expect`) ⇒ `Entry::Vacant`, que também
   apaga o `expect`.
9. ⛔⛔ **A CENA dizia ao dono uma coisa FALSA, e o gate dela não a via:** a linha `[script-smoke]`
   mandava *«mude um default, grave — só o «Bob» muda»*. **«Próprio» é por NÚMERO, não por
   objecto:** o «Bob (fast)» tem `speed` e `top_signal` próprios e **segue** a `amplitude` do
   ficheiro — mudar essa muda **dois** bonecos. O gate da recarga media só o «Bob» e o «Bob (tall)»,
   logo era cego ao terceiro. Achado ao conferir os passos do smoke contra o código, **antes** de os
   enviar ⇒ a frase e o doc da cena dizem os dois, e o gate (`mudar_o_default_no_ficheiro_muda_quem_
   nao_tem_aquele_numero_proprio`) mede os **três**. *Uma cena que ensina o contrário é pior que uma
   ausente* (§5.0).

---

## §8 — O SMOKE (o que o dono corre) e o fecho

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_SCRIPT_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke
```

A cena escreve **`~/.ph2d/smoke/bob.luau`** (reescrito a cada arranque) e pendura-o em três bonecos:
**«Bob»** (segue o ficheiro), **«Bob (tall)»** (sobe o dobro, e traz um órfão `height` de
propósito), **«Bob (fast)»** (rápido, e acende/apaga a lâmpada a cada topo pela tabela de acções).
Abre com o **«Bob (tall)»** escolhido e o relógio a andar.

**O fecho (16/09):**

- `NO_FAIL_FAST=1 BASE=115065b58 scripts/nextest-impacted.sh` — **13 739 testes, 13 739 verdes**
  (11 743 fora do conjunto impactado), a `load 15–25` com a `line/3DModeling` a correr testes ao lado.
- `cargo clippy --workspace --all-targets -- -D warnings` — **zero**, depois de três curas (§7.8).
- `cargo fmt --all -- --check` — limpo.
- **Mutação:** 33/33 (§5).
- ⚠️ **A foto** é a do instrumento, numa tela virtual de ~1930×1040 e de 2560: a secção inteira
  cabe (o caminho, `Browse`, as quatro linhas, o `Reset` na `amplitude`, o órfão `height` com
  `Remove`) e os três bonecos e a lâmpada ficam dentro do ecrã. ⛔ O **clique** não se fotografa (§7.7).

---

## §9 — O que fica ABERTO

1. **O `ph2d.attach_script(entity, hex)` do spike** fala de um `AssetId` que o componente já não
   tem, para uma fila sem dreno. Quando alguém escrever o dreno, a ligação muda para um **caminho**.
2. **O projecto não embute o script** e mover o ficheiro parte a ligação — o preço do som do #4, com
   a mesma cura nomeada (o tipo no índice de assets).
3. **Um script e a física no MESMO corpo brigam** — o painel avisa; não é suportado.
4. **O `find_by_name` não é alimentado** — de propósito (a regra do levantamento: sinais, nunca
   referência directa). Um script lê e escreve só a própria pose.
5. **`math.sin` e companhia vêm da libm da plataforma** — um replay entre sistemas operativos pode
   divergir no último bit. O `physics_ecs_c9` não corre scripts.
6. **O carimbo de recarga é `(tamanho, data)`** — um editor que grave o mesmo tamanho dentro da mesma
   marca de tempo de um sistema de ficheiros grosseiro não seria visto (btrfs/tmpfs têm ns).
7. **Tipos `Vector2`/`Color`/enum/recurso** — wave própria, com consumidor.
8. **Não há gate de PINTURA-a-pixel** para as secções opcionais (o do #15 continua aberto); esta
   secção tem a **foto** e a costura com o gesto real.
