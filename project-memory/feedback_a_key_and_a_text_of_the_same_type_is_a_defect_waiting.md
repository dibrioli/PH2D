---
name: feedback_a_key_and_a_text_of_the_same_type_is_a_defect_waiting
description: "Tabela `const` que guarda CHAVES de i18n como `&str` deixa o consumidor esquecer a tradução e pintar a chave crua com a suíte verde — tipe a chave (ph2d_i18n::TextKey) e o esquecimento vira erro de compilação; foi o compilador que achou 13 rótulos que a régua não via"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: e990a2f5-7d16-405a-8ddf-54393edf203d
  modified: 2026-09-13T23:22:35.318Z
---

Medido 2026-09-13 (`line/UIUX`, migração do Inspector para o HR-15): metade do vocabulário de um
painel mora em tabelas `const` (`[&str; N]` de tipos de junta, cards, modos), onde `tr` não compila
(E0015). A cura óbvia — guardar a CHAVE na tabela e traduzir no consumidor — deixa chave e texto com
o **mesmo tipo**: quem se esqueça do `tr` compila, passa em todo teste que não leia o pixel, e pinta
`panel.inspector.joint.pin` no ecrã. Com um newtype (`ph2d_i18n::TextKey`, `const fn new` + `tr`),
40 tabelas trocaram de tipo e ~50 consumidores foram **obrigados** a `.map(TextKey::tr)`/`.tr()`.

⭐ E o efeito colateral foi o achado maior: treze rótulos `(m/s)` que a régua lexical descartava (ela
lia `/` como caminho) ficaram `&str` no meio de uma tabela `TextKey` — **quem os apanhou foi o
compilador**, não o censo.

**Why:** um gate de fonte mede o que consegue ver; um TIPO mede o que o programa faz. Onde os dois
se sobrepõem, o tipo é a régua que não tem ponto cego.

**How to apply:** ao migrar texto para uma tabela de strings, tabela `const` guarda o newtype da
chave, nunca `&str`. Vale para todo par «identificador × valor» que viaje junto (chave/texto,
id/rótulo, caminho/conteúdo). Ligado: [[reference_topic_measurement_discipline]],
[[feedback_a_declared_blind_spot_can_be_the_majority]], [[reference_topic_gate_discipline]].

---

## ⛔⛔ Quando um `&str` tem DOIS PAPÉIS, a cura não é traduzi-lo: é PARTI-LO (2026-09-19)

Ao pagar a dívida da fronteira dos motores, **duas de três** acusações não eram um rótulo cru — eram
um **IDENTIFICADOR a fazer de rótulo**, e traduzi-lo teria partido coisas em silêncio:

- **`PickStrategy::name()`** era o texto do selector `◀ nome ▶` **e** o que o `from_name()` lê de
  volta de um **MANIFESTO gravado**. Traduzido, todo ficheiro que já existe deixava de ser lido — e
  em silêncio, porque o `from_name` devolve `None` e o chamador cai no valor de omissão.
- **`Platform.name`** era o que a aba diz **e** o infixo do ficheiro exportado
  (`{stem}.mobile.ogg`, por `name.to_lowercase()`). Traduzido, o nome dos ficheiros que o artista
  exporta mudava com a **língua da interface**, e uma pipeline que os procura pelo nome deixava de
  os achar.

⇒ o identificador fica onde estava (com a razão escrita), e a palavra que o artista lê passa a ser
uma chave: `Platform::id` + `Platform::label_key`, `PickStrategy::name()` + `label_key()`.

⚠️⚠️ **E a régua NÃO separa esta família sozinha, o que tem de ir escrito na isenção:** um campo que
se chame `id` escapa-lhe (ela colhe `label`/`name`/`title`/`text`) e **uma `fn … -> &str` é SEMPRE
acusada**. A isenção legítima é a que diz **QUEM LÊ A STRING DE VOLTA** — um formato de ficheiro, um
manifesto, um nome de ficheiro exportado. Sem essa frase, a próxima fatia traduz e parte.

⭐ **A pergunta que separa em dois segundos:** *alguém faz o caminho INVERSO a partir desta string?*
Se sim, ela é dado; a palavra é outra coisa.
