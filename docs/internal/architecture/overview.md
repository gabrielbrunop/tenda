# Visão geral da arquitetura da Tenda

Este documento fornece uma visão geral da arquitetura da linguagem de programação Tenda, destacando seus principais componentes e como eles interagem para formar um sistema coeso.

## Visão Geral

A base de código da Tenda utiliza Cargo Workspaces para organizar o projeto em múltiplos pacotes, em uma estrutura modular. Cada pacote tem uma função específica, permitindo que o sistema seja facilmente extensível e mantenível.

## Filosofia

A arquitetura da Tenda é projetada para ser extremamente modular, com cada pacote servindo a um propósito específico. Isso permite que desenvolvedores possam facilmente entender, modificar e contribuir para o código.

Nos limites do pragmatismo, buscamos manter todos os componentes do projeto Tenda como _crates_ nesse repositório, com uma única pipeline de CI/CD para todo o projeto.

## Componentes Principais

### `core`

O pacote `core` reexporta os componentes da linguagem que podem ser utilizados por outros pacotes, projetos externos e a própria CLI da Tenda.

### `scanner`

O pacote `scanner` é responsável por analisar o código-fonte e gerar tokens. Ele implementa o scanner, que lê o código e identifica palavras-chave, identificadores, literais e outros elementos da linguagem.

### `parser`

O pacote `parser` constrói a árvore de sintaxe abstrata (AST) a partir dos tokens gerados pelo scanner.

### `loader`

O pacote `loader` utiliza o `scanner` e o `parser` para carregar arquivos de código-fonte a partir de um caminho ou uma string, no caso de código passado por linha de comando ou entrada padrão. Ele recursivamente carrega arquivos, de acordo com as importações encontradas no código.

### `runtime`

O pacote `runtime` é responsável por executar o código compilado. Ele implementa um _tree-walk interpreter_, que percorre a árvore de sintaxe abstrata e executa as instruções conforme necessário.

### `common`

O pacote `common` contém definições comuns que são compartilhados entre muitos pacotes.

### `os-platform`

O pacote `os-platform` contém definições específicas para cada plataforma operacional suportada, como Windows, macOS e Linux. Ele fornece abstrações para interações com o sistema operacional, como manipulação de arquivos e execução de comandos. Ele deve ser passado como dependência para o pacote `runtime`, que o utiliza para executar código específico da plataforma.

### `prelude`

O pacote `prelude` contém definições comuns que são automaticamente importadas em todos os arquivos de código Tenda. Ele inclui funções e tipos básicos que são frequentemente utilizados, como manipulação de strings, listas e outras estruturas de dados fundamentais.

### `reporting`

O pacote `reporting` é responsável por gerar relatórios de erros e avisos durante a análise e execução do código. Ele fornece uma interface para formatar mensagens de erro de forma amigável, incluindo informações sobre a localização do erro no código.

### `reporting-derive`

O pacote `reporting-derive` é um _crate_ auxiliar que fornece macros para derivar implementações de relatórios de erros e avisos. Ele facilita a criação de tipos personalizados que podem ser usados com o sistema de relatórios da Tenda.

### `tenda`

O pacote `tenda` é o ponto de entrada principal para a CLI da Tenda. Ele utiliza os outros pacotes exportados pelo `core` para fornecer uma interface de linha de comando que permite aos usuários executar código Tenda, carregar arquivos e interagir com o sistema.

## Outros Componentes

Além dos pacotes principais, a Tenda também inclui outros componentes que não fazem parte do núcleo da linguagem:

### `tenda-playground`

Aplicação similar ao pacote `tenda`, mas exportado para WASM/WASI e com um pequeno protocolo de comunicação para permitir a execução de código Tenda em um ambiente isolado. É utilizado no playground online da Tenda, permitindo que usuários executem código diretamente no navegador.

### `tenda-playground-platform`

Este pacote contém definições específicas para a plataforma do playground.
