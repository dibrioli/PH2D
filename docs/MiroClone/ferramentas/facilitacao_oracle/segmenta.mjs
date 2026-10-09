// Passo 1 do oráculo: segmentação de palavras UAX#29 pelo ICU embutido no Node (Intl.Segmenter).
// Entrada: JSON [{id, text}] ; saída: JSON com, por nota, os segmentos word-like e o fold.
import { readFileSync } from "node:fs";
const notas = JSON.parse(readFileSync(process.argv[2], "utf8"));
const seg = new Intl.Segmenter("en", { granularity: "word" });
const out = {
  meta: { node: process.version, icu: process.versions.icu, unicode: process.versions.unicode,
          segmenter: "Intl.Segmenter('en',{granularity:'word'}) isWordLike" },
  notas: notas.map(n => {
    const words = [...seg.segment(n.text)].filter(s => s.isWordLike).map(s => s.segment);
    // fold de comparação: NFD -> toLowerCase -> NFD -> sem marcas Mn (aproxima o ph2d-label-fold;
    // o toLowerCase NÃO é case folding completo: ß fica ß)
    const fold = w => w.normalize("NFD").toLowerCase().normalize("NFD").replace(/\p{Mn}/gu, "");
    return { id: n.id, text: n.text, words, folded: words.map(fold) };
  }),
};
console.log(JSON.stringify(out, null, 1));
