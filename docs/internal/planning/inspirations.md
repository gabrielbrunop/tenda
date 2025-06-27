# Inspirações

Esse documento reúne inspirações e referências que influenciam a linguagem Tenda.

## Filosofia: leitura narrativa

A leitura narrativa é um conceito central na Tenda, onde o código é escrito e lido como se fosse uma linha de raciocínio em linguagem natural. Mais do que isso, é uma maneira de combinar o mundo do código com o discurso lógico.

Isso quer dizer que o código vai ter símbolos e estruturas tais como outras linguagens de programação: blocos, tokens etc., mas de uma maneira que isso não impeça a leitura fluida do código como um raciocínio lógico.

Exemplo de código em Tenda:

```tenda
para cada índice em 1 até 5 faça
  Lista.insira(notas, leia_número(índice))
fim
```

Equivalente em C:

```c
for (int i = 1; i <= 5; i++) {
  notas[i] = ler_numero(i);
}
```

Estruturas e lógicas similares, mas com uma leitura mais fluida e compreensível.

## Semelhança com outras linguagens

Algumas linguagens de programação influenciaram a sintaxe e a estrutura da Tenda, e continuarão a influenciar seu desenvolvimento. Abaixo estão algumas das linguagens mais notáveis que serviram de inspiração:

### Python

A Tenda tenta ser tão alto nível quanto o Python. Como ambos são projetados para serem fáceis de ler e escrever, eles acabam sendo frequentemente comparados.

### Lua

Lua é uma linguagem de script leve e embutível, conhecida por sua simplicidade e flexibilidade. A sintaxe da Lua se assemelha muito a algumas partes da Tenda, como o uso de `do`...`end` e `if`...`then`...`else`.

### Linguagens funcionais

Linguagens funcionais como Haskell e OCaml influenciaram algumas partes da Tenda.

Corpo de função como expressão e expressões condicionais, por exemplo:

```tenda
seja fatorial(n) =
  se n <= 1
    então 1
    senão n * fatorial(n - 1)
```

O prelúdio da Tenda também inclui diversas funções de ordem superior que são comuns em linguagens funcionais.

### Lições de tooling com Rust e Go

O tooling da Tenda é inspirado em Rust e Go. Valorizamos muito o tooling, que deve receber atenção especial para garantir uma boa experiência de desenvolvimento e aprendizado.

## Leituras

O livro Crafting Interpreters de Robert Nystrom é uma excelence referência para linguagens dinâmicas de alto nível e é uma das principais inspirações para o design da Tenda.
