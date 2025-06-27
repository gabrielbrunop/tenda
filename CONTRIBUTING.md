# Guia de contribuição

## Missão do projeto

A missão da Tenda é tornar a programação mais acessível nos países lusófonos, utilizando uma linguagem de programação moderna e intuitiva, com palavras-chave em português e uma sintaxe próxima da linguagem natural. Queremos reduzir a barreira de entrada para iniciantes e educadores, promovendo o aprendizado de programação nas nações de língua portuguesa.

Se você está aqui, é porque acredita na nossa missão, e queremos agradecer por isso! Sua contribuição é fundamental para alcançarmos nossos objetivos e levar a programação a mais pessoas.

## Contribuindo para o projeto

Pull requests, issues e sugestões são bem-vindos! Antes de contribuir, por favor, leia este guia para entender como funciona o processo de contribuição para o projeto Tenda.

Leia também o [código de conduta](CODE_OF_CONDUCT.md) para garantir um ambiente respeitoso e colaborativo.

## Bug reports

Se você encontrou um bug, por favor, abra uma issue no repositório do GitHub. Inclua o máximo de detalhes possível, como:

- Descrição do problema
- Passos para reproduzir o bug
- Comportamento esperado
- Comportamento atual
- Versão da Tenda que você está usando
- Sistema operacional e versão

Também:

- Verifique se o bug já foi reportado anteriormente.
- Revise a documentação para garantir que não é um comportamento esperado.
- Use o [markdown do GitHub](https://docs.github.com/en/get-started/writing-on-github/getting-started-with-writing-and-formatting-on-github/basic-writing-and-formatting-syntax) ao seu favor para formatar a issue de forma clara e legível.

## Pedidos de funcionalidade

Pedidos de funcionalidade são bem-vindos! Nós vamos considerar todas as sugestões, mas não podemos garantir que todas serão implementadas. Queremos evitar que a linguagem se torne um "monstro de Frankenstein" com funcionalidades que não se encaixam bem. Sua ideia pode ser ótima, mas pode não se encaixar na visão geral da Tenda. Se a sugestão for aceita, ela será adicionada à nossa lista de funcionalidades futuras, mas não há garantia de que será implementada em breve. Porém, você é bem-vindo a abrir um pull request com a implementação da funcionalidade!

- Antes de abrir uma issue, verifique se a funcionalidade já foi sugerida anteriormente.
- Descreva a funcionalidade de forma clara e objetiva.
- Explique por que a funcionalidade é importante e como ela se encaixa na visão da Tenda.
- Se possível, forneça exemplos de como a funcionalidade poderia ser implementada ou utilizada.
- Use o [markdown do GitHub](https://docs.github.com/en/get-started/writing-on-github/getting-started-with-writing-and-formatting-on-github/basic-writing-and-formatting-syntax) para formatar a issue de forma clara e legível.
- Inclua detalhes de implementação, se possível.

Os mantenedores podem solicitar mais informações ou discutir a implementação da funcionalidade antes de aceitá-la, bem como incluir novas condições ou limitar o escopo da funcionalidade para garantir que ela se encaixe na visão do projeto.

## Pull requests

Pull requests são muito bem-vindos! Antes de abrir uma pull request para mudanças não triviais, é importante que a mudança seja discutida previamente com os mantenedores. Isso ajuda a evitar retrabalho e garante que a mudança esteja alinhada com a visão do projeto.

Todas as contribuições serão licenciadas sob a licença GPL-3.0, conforme o arquivo LICENSE no repositório.

### Antes de abrir uma pull request

- Veja o [workflow de desenvolvimento](docs/internal/processes/development-workflow.md).
- Se a mudança for não trivial ou demorada, avise os mantenedores para evitar retrabalho.
- Verifique se não há ninguém trabalhando na mesma funcionalidade ou correção.
- Para triviais e pequenas correções de erro, documentação ou melhorias, você pode abrir uma pull request diretamente. Porém, é recomendado discutir a mudança primeiro para garantir que ela esteja alinhada com a visão do projeto.
- Crie ou escolha uma issue para:
  - Contribuir com uma correção de bug ou documentação
  - Contribuir com adições à linguagem
  - Contribuir com o tooling
  - Contribuir com refatorações
  - Contribuir com melhorias de performance
- Para adições à linguagem, crie uma proposta de mudança e discuta com os mantenedores antes de abrir a pull request.
- Antes de criar uma issue, leia os [documentos internos do projeto](/docs/internal/README.md).
- Caso você queira desenvolver uma parte grande do tooling, como a LSP, formatter, depurador etc., crie uma issue para discutir a implementação, definir requisitos e escopo, e obter feedback dos mantenedores. Também verifique se há outros issues relacionadas que possam impactar sua implementação, ou se a literatura técnica da Tenda já possui conteúdo sobre o assunto.
- Para melhorias de performance, cheque as [diretrizes de performance no guia de estilo de código](/docs/internal/style/code-style.md#performance-1)] antes de criar uma issue ou pull request. Para melhorias não triviais, utilize profiling e dados concretos para justificar a mudança. Se você não tiver certeza se a mudança é trivial ou não, discuta com os mantenedores antes de abrir a pull request.

### Desenvolvendo

- Mantenha seu branch atualizado com o branch `dev` do repositório principal.
- Siga todas as diretrizes de estilo e qualidade do código do projeto.
- Siga o mesmo estilo de código utilizado no projeto, incluindo formatação, nomenclatura e organização do código. Escreva código de tal maneira que não fique perceptível a diferença entre o seu código e o código existente.
- Modifique a documentação interna do projeto, se necessário, para refletir as mudanças feitas.
- Resolva qualquer merge conflicts antes de abrir a pull request.
- Não refatore código fora do escopo da pull request. Se você identificar áreas que precisam de refatoração, crie uma issue separada para discutir e implementar essas mudanças.
- Preferimos um pull request longo e abrangente do que vários pequenos pull requests. Isso ajuda a manter todo o contexto da mudança em um único lugar. Porém, se a mudança for muito grande, prefira fazê-la gradualmente, adicionando revisões incrementais e significativas ao longo do tempo. Isso facilita a revisão e garante que o código esteja sempre em um estado funcional. Para saber mais, leia: [The advantage of large, long-running pull requests](https://world.hey.com/dhh/the-advantages-of-large-long-running-pull-requests-c33d913c).
- O pull request tem que cobrir um único escopo, mesmo que seja grande. Isso pode ser desde uma pequena correção de bug, mas não duas correções de bugs não relacionados, ou até mesmo uma ferramenta inteira, como um depurador ou um formatter, mas não os dois juntos. No último caso, lembre-se da importância de discutir a implementação com os mantenedores antes de abrir o pull request.

### Abrindo a pull request

Após concluir o desenvolvimento, abra uma pull request no repositório principal. Certifique-se de que o pull request:

- Esteja direcionada para o branch `dev`.
- Tenha um título claro e descritivo.
- Inclua uma descrição detalhada do que foi feito, por que foi feito e como testar a mudança.
- Referencie a issue relacionada, se houver, usando `#<número da issue>`.
- Inclua testes automatizados, se aplicável, e garanta que todos os testes existentes passem.

### Revisão de código

Após abrir a pull request, os mantenedores irão revisar o código. Eles podem solicitar ajustes ou aprovar a pull request. É importante que você esteja aberto a feedback e disposto a fazer as alterações necessárias.

Em geral, espere revisões longas e detalhadas, especialmente para mudanças significativas. Revisões são uma parte importante do processo de desenvolvimento e ajudam a garantir não só a qualidade do projeto, mas também a sua própria evolução como desenvolvedor.

#### Conduta de revisão

- **Revise o código, não o autor.** Seja cordial, respeitoso e construtivo nas suas críticas.
- **Seja específico.** Evite comentários vagos como "isso não está bom". Explique o que pode ser melhorado e por quê.
- **Você não é o seu código.** Não leve críticas ao seu código como críticas pessoais. Todos nós estamos aqui para aprender e melhorar. Não há ninguém que escreva código perfeito.
- **Faça seu melhor!** Tente escrever código de alta qualidade, seguindo as diretrizes do projeto. Se você não tem certeza se algo está correto, pergunte aos mantenedores ou a outros desenvolvedores do projeto.
- **Seja paciente.** Os mantenedores podem levar algum tempo para revisar o código. Eles estão fazendo o melhor que podem para garantir a qualidade do projeto.
- **Não hesite em pedir ajuda** ou esclarecimentos sobre o código. Se você não entende algo, é provável que outros também não entendam. Pergunte e busque entender o porquê das decisões tomadas.
- Com gentileza, **evite violações aos guias e diretrizes do projeto**. Todos nós adoramos desenvolvendo e estamos aqui para cumprir uma missão: levar a programação a mais pessoas.
- **Elogios são bem-vindos!** Se você gostou de algo no código, não hesite em dizer. Reconhecer o bom trabalho dos outros é uma ótima maneira de manter um ambiente colaborativo e motivador.
- **Dê os créditos devidos.** Se você se inspirou em outra pessoa, projeto ou artigo, mencione isso na pull request. Isso ajuda a construir uma comunidade mais colaborativa e respeitosa.

#### Boas práticas

- Registre decisões importantes e esclareça o porquê de certas escolhas no código.
- Seja comunicativo e compartilhe seu raciocínio com os revisores. Isso não só ajuda na revisão, mas também enriquece o entendimento do código para todos os envolvidos.
- Referencie a fonte de suas ideias, se aplicável. Artigos, livros, outras linguagens ou projetos, e até mesmo respostas de inteligência artificial podem ser fontes de inspiração. Isso ajuda a contextualizar suas decisões e pode ser útil para outros desenvolvedores no futuro.

#### Resolução de conflitos

Os mantenedores buscam sempre o diálogo e a colaboração, mas também possuem a palavra final sobre o que é ou não aceitável no projeto. Se você não concorda com uma decisão, sinta-se à vontade para discutir, mas lembre-se de que a decisão final cabe aos mantenedores.

### CI

Corrija todos os erros de CI antes de solicitar uma revisão. O CI é executado automaticamente para cada pull request e garante que o código esteja em um estado funcional. Se o CI falhar, a pull request não será mesclada até que todos os erros sejam corrigidos.

### Mensagens de commit

Use mensagens de commit claras e descritivas, seguindo o padrão de mensagens do projeto.

Veja:

- [Conventional Commits](https://www.conventionalcommits.org/pt-br/v1.0.0/)
- [How to Write a Git Commit Message](https://cbea.ms/git-commit/)

## Certificado de Origem do Desenvolvedor

_Certificado de Origem do Desenvolvedor 1.1_

Ao realizar uma contribuição para este projeto, eu certifico que:

> (a) A contribuição foi criada integralmente ou parcialmente por mim, e tenho o direito de submetê-la sob a licença open source indicada no arquivo; ou
>
> (b) A contribuição é baseada em um trabalho prévio que, até onde é de meu conhecimento, está coberto por uma licença open source apropriada e eu tenho o direito, sob essa licença, de submeter o trabalho com modificações, criadas integralmente ou parcialmente por mim, sob a mesma licença open source (a menos que eu tenha permissão para submeter sob uma licença diferente), conforme indicado no arquivo; ou
>
> (c) A contribuição me foi fornecida diretamente por outra pessoa que certificou (a), (b) ou (c), e eu não realizei modificações nela.
>
> (d) Entendo e concordo que este projeto e a contribuição são públicos e que um registro da contribuição (incluindo todas as informações pessoais que eu enviar com ela, inclusive minha assinatura) será mantido indefinidamente e poderá ser redistribuído em conformidade com este projeto ou com a(s) licença(s) open source envolvida(s).

## Agradecimentos

Esse documento foi inspirado em:

- [Contributing Guidelines by @jessesquires](https://github.com/jessesquires/.github/blob/main/CONTRIBUTING.md)
- [The advantage of large, long-running pull requests](https://world.hey.com/dhh/the-advantages-of-large-long-running-pull-requests-c33d913c)
