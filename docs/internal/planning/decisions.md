# Decisões de linguagem e estilo

Esse documento destaca algumas decisões de linguagem e estilo que diferenciam a Tenda de outras linguagens de programação baseadas em português estruturado.

## Funções estruturadas

A sintaxe de funções estruturadas da Tenda não é muito similar a de linguagens industriais, e muitos costumam comparar mais com pseudocódigo.

```tenda
seja soma(lista) =
  faça
    seja total = 0

    para cada número em lista faça
      total = total + número
    fim

    retorna total
  fim
```

Existe um motivo especial para a existência dessa sintaxe: ao aprender funções, os estudantes começam com funções simples, cujo corpo é somente uma expressão. Na Tenda, a sintaxe desse tipo de função é muito simples e lembra funções matemáticas:

```tenda
seja quadrado(x) = x * x
```

Mover para um bloco `faça`...`fim` em funções estruturadas não só transmite a ideia de que agora estamos construindo _procedimentos_ com mínimas mudanças de sintaxe, como também permite a _leitura narrativa_ sem ter duas sintaxes diferentes para funções.

A título de curiosidade, as funções estruturadas seguem oficialmente a [indentação do GNU](https://en.wikipedia.org/wiki/Indentation_style#GNU).

## Acentuação

A acentuação faz parte da língua portuguesa. Palavras não acentuadas estão gramaticalmente incorretas. Não há motivo para não acentuar palavras em português, e a Tenda acentua palavras-chave e identificadores.

Isso é incomum em outras linguagens baseadas em português estruturado, mas na Tenda é uma escolha imprescindível. Imagine a confusão que seria não distinguir `e` e `é`!

## Conjugação das palavras-chave e funções embutidas

Palavras-chave e funções embutidas verbais são conjugadas de acordo com a seguinte regra:

1. Em geral, utiliza-se a segunda pessoa do singular do imperativo quando ele é o mesmo da terceira pessoa do singular do presente do indicativo, como `retorna`, `transforma` etc.
2. Quando as conjugações são diferentes, utiliza-se a terceira pessoa do singular do imperativo, como `seja`, `faça`, `leia`, `escreva` etc.

As motivações para essa escolha são:

- Consistência: todas terminam com a mesma vogal, facilitando a leitura e escrita.
- Imperativo: a computação é imperativa, diferente da geometria, que é declarativa, e queremos que os estudantes reconheçam isso desde o início. Não obstante, ainda promovemos aspectos do paradigma funcional, usando a língua portuguesa de forma inteligente para distinguir construtos dos dois paradigmas.
- Português moderno: evitamos a segunda pessoa quando ela é arcaica, como em `sê`.

## Declaração de variáveis com `seja`

`seja` é uma tradução direta de "let" em inglês e é usada em contextos matemáticos e lógicos, motivando a sua escolha como palavra-chave para declarar variáveis.
