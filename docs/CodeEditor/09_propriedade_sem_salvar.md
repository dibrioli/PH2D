# 09 — A propriedade aparece no Inspector sem salvar (1b)

> Ordem do dono (23/10, no smoke OK da cena 9): *«como no Godot, a variável exposta no painel deve aparecer
> mesmo sem salvar o script»*. Feito na `line/CodeEditor` em 23/10 (`d4b25c2d0b`, cena `a02f86b2a8`).

## 1. Oráculo — o editor do Godot 4.7.2 corrido (MIT)

[`ferramentas/oraculo_export_sem_salvar/`](ferramentas/oraculo_export_sem_salvar/saida.txt): numa Xvfb, com
teclas reais (`xdotool`, 120 ms por tecla) no script aberto e o nó dele escolhido. Uma sonda (EditorPlugin) lê a
cada 50 ms as propriedades de script que o Inspector mostra. Nada é salvo até ao passo 9. **Duas corridas,
mesmos resultados** (23/10, 23:32Z e 23:33Z).

| Passo | O Inspector |
|---|---|
| escrever `@export var velocidade := 3.0` | aparece **~1,4 s depois da última tecla** (o `idle_parse_delay` = 1,5 s), nunca a cada tecla |
| valor próprio 7, mudar o padrão 3.0 → 5.0 | fica **7** |
| erro de sintaxe numa linha nova | fica a **última lista boa** (o erro só na barra do editor) |
| nova `@export escudo` com o erro presente | não aparece |
| escolher outro nó e voltar, com o erro | a última lista boa |
| apagar o erro | `escudo` aparece **~0,5 s** depois |
| renomear `velocidade` → `rapidez` | `rapidez` com o padrão; o 7 perde-se |
| apagar a linha `escudo` | some na pausa |
| o ficheiro no disco antes do Ctrl+S | **igual ao original** |

## 2. A lei (nossa)

1. Cada aba colhe as declarações do SEU texto na pausa: 1,5 s, ou 0,5 s se a última colheita falhou. São os
   mesmos `IDLE_NS`/`IDLE_ERRORS_NS` do lint e das cores. A aba recém-aberta colhe logo.
2. A colheita **corre o topo** do texto na VM da casa (`ScriptHost::draft_decls`). É a mesma resposta da
   linguagem que a recarga dá (`module.rs`), sob o mesmo prazo. Sem console: o `print` do topo cala-se. As filas
   de escrita e de sinais são deitadas fora, e a cena não sabe.
3. Texto partido = fica a última lista boa. Um texto que nunca carregou não publica nada, e responde a cena.
4. A família do editor publica a lista no recurso `ScriptDrafts`, por script aberto ou estacionado com ●. Este
   recurso **não é documento**: o projeto guarda o texto salvo. O cartão lê `ScriptDrafts` primeiro; senão, a
   versão que os objectos CORREM (`SceneScripts::decls_in_use`).
5. **Defeito da mesma família, curado junto:** com o texto SALVO partido, o cartão ficava sem propriedades,
   embora os objectos corressem a versão anterior. Agora mostra a dessa versão.
6. O valor próprio continua no `LuauScript.own`, pelo nome. Um nome renomeado mostra o padrão, como no Godot. O
   valor antigo continua a ser contado como «guardado» (a lei dos órfãos, sem mudança).
7. **O jogo corre o texto SALVO.** O rascunho só muda o Inspector, como o placeholder do editor do Godot.

## 3. Prova

- Portões: `as_declaracoes_de_um_texto_por_salvar_nao_falam_nem_congelam` (ph2d-script) ·
  `draft::tests::*` (pausa, última boa, cura a 0,5 s, aba fechada, texto que nunca carregou) ·
  `com_o_texto_salvo_partido_o_cartao_mostra_a_versao_que_corre` ·
  `a_lista_do_texto_por_salvar_manda_sobre_a_da_cena` · `a_cena_9_do_editor_ensina_o_que_diz` (③ antes
  do Ctrl+S).
- Corrida conduzida da cena 9 ([`conduz_inspector/`](ferramentas/conduz_inspector/corre.sh)): a
  `foto_09_campo_door_sem_salvar` mostra o campo «door» com a aba ainda com ●.
