# Guia de estilo de código

Esse documento estabelece as diretrizes de estilo que todos os colaboradores devem seguir para garantir a consistência e a qualidade do código.

## Inspirações

- [Tiger Style](https://github.com/tigerbeetle/tigerbeetle/blob/main/docs/TIGER_STYLE.md)
- [Commit to competence in this coming year](https://world.hey.com/dhh/commit-to-competence-in-this-coming-year-feb7d7c5)

## Essência

### Corretude

Correto é a qualidade do código que se comporta como esperado sob qualquer circunstância.

**Legibilidade** define a dificuldade de cumprir esta tarefa. Quanto mais legível o código, mais difícil é de cometer erros.

**Provar invariantes** é o ato de informar aos outros e a si mesmo todo o conjunto das suas intenções - não só o que o código faz, mas também o que ele não deve fazer.

### Experiência do desenvolvedor

Já dizia o velho ditado: "Faça funcionar. Refatore. Otimize." Esse documento faz questão de adicionar um passo a mais: "Faça bonito."

Acreditamos que um código bem escrito não é apenas funcional, apesar de esse ser o primeiro passo, mas também legível e agradável de se trabalhar.

### Performance

Performance não quer dizer que o programa precisa ser o mais rápido possível, mas sim que deve atender às expectativas de desempenho do usuário. Dessa forma, está associada à qualidade do produto e à experiência do usuário final.

## Diretrizes

### Corretude

#### Asserções e testes

- Asserções detectam erros do programador. Diferente dos erros operacionais, que são esperados e devem ser tratados, falhas de asserção são inesperadas. A única forma correta de lidar com código corrompido é parar a execução do programa. Não queremos que o usuário use um programa diferente, potencialmente perigoso, e asserções são uma forma de garantir isso.
  - Use `assert!` para verificar invariantes do programa.
  - Asserções são preferíveis a comentários.
  - Use `.expect()` como asserção. Prefira `.expect()` com uma mensagem de erro clara e descritiva ao invés de `.unwrap()`, que não fornece contexto sobre o erro.
- Escreva testes que comprovem o funcionamento do código. Siga as regras de escrita de testes.

#### Design

- Declare variáveis no menor escopo possível.
- Imutabilidade deve ser sempre a primeira opção. Mutabilidade somente quando a situação exigir.

#### Rust

- Prefira usar iteradores ao invés de loops com mutação sempre que for tranquilo (em outras palavras, quando o borrow checker permitir).
- Prefira o uso de [ADTs (Algebraic Data Types)](https://en.wikipedia.org/wiki/Algebraic_data_type) e tipos compostos.
- Utilize o sistema de tipos ao seu favor. Escreva código que o sistema de tipos possa verificar.
- Warnings devem ser tratados como erros. Não ignore-os.
- Evite tipos complexos.
- Evite lifetimes complexos.
- Evite o uso de `unsafe` a menos que seja absolutamente necessário. Se for necessário, documente claramente o motivo e as garantias que o código oferece.

### Experiência do desenvolvedor (DX)

#### Tooling

- Use `cargo fmt` para formatar o código.
- Use `cargo clippy` para verificar o código e aplicar sugestões de melhorias.

#### Ordem

- A ordem é importante. Tipos pequenos e importantes vêm primeiro, seguido de tipos maiores e importantes, e depois de tipos pequenos e menos importantes.

#### Comentários

- Evite o uso de comentários desnecessários. O código deve ser autoexplicativo. Quando for absolutamente necessário passar uma mensagem para o próximo desenvolver, ou você do futuro, sobre algo que não pode ser percebido apenas lendo o código, use comentários claros, explicativos e bem escritos.
- Os comentários tem que seguir a gramática da língua portuguesa, com pontuação e acentuação corretas, capitalização adequada, sem erros de digitação e com clareza. Comentários ruins são piores do que nenhum comentário.

#### Idioma

- Para qualquer texto visível ao usuário, use o português. O dialeto deve ser o [português brasileiro neutro](https://pt.wikipedia.org/wiki/Dialeto_neutro). Evite regionalismos.
- Para nomes de variáveis, funções, tipos, comentários e todo o código não visível ao usuário, use o inglês norte-americano. Utilizar o mesmo idioma e termos da linguagem hospedeira (Rust) e da literatura de compiladores e teoria de linguagens de programação ajuda a manter a consistência e facilita a compreensão, além de evitar confusões com termos técnicos e traduções difíceis.

#### Nomeando

- Use nomes descritivos e significativos para variáveis, funções e tipos.
- Use verbos para funções e métodos, e substantivos para variáveis e tipos.
- Não abrevie nomes a menos que seja algo amplamente reconhecido (como `stmt`, `expr`, `env` etc.).
- Para o resto, siga o padrão de nomenclatura do Rust.
- Não use o mesmo termo em contextos diferentes.

#### Espaçamento

Em geral: escreva código à prova de miopia! Prefira código bem espaçado em vez de compacto.

- Use espaços em branco para melhorar a legibilidade do código.
- Separe blocos de código com uma linha em branco.
- Não exagere. Não precisa haver uma linha em branco entre cada linha de código. Dê significado semântico ao espaçamento.

Exemplos de bom espaçamento:

```rust
fn create_function(
    &self,
    params: &[ast::FunctionParam],
    body: Box<ast::Stmt>,
    metadata: Option<FunctionRuntimeMetadata>,
) -> Function {
    let mut context = Environment::new();

    for env in self.store.get_current().into_iter() {
        for (name, value) in env {
            if params.iter().any(|param| param.name == *name) {
                continue;
            }

            if let ValueCell::Shared(value) = value {
                context.set_or_replace(name.clone(), ValueCell::Shared(value.clone()));
            }
        }
    }

    let mut func = Function::new(
        params.iter().map(|p| p.clone().into()).collect(),
        context,
        body,
    );

    if let Some(metadata) = metadata {
        func.set_metadata(metadata);
    }

    func
}
```

```rust
pub fn define(&mut self, name: String, value: ValueCell) -> Result<(), StackDefinitionError> {
    let scope = self.get_innermost_frame_mut().get_env_mut();

    if scope.has(&name) {
        return Err(StackDefinitionError::AlreadyDeclared);
    }

    scope.set_or_replace(name, value);

    Ok(())
}
```

Lembre-se de que, em muitos casos, há multiplas opções aceitáveis de espaçamento. Não tendo uma regra específica, use o bom senso e mantenha a consistência no seu código.

#### Código

- Evite cadeias de `else if`. Prefira código linear ou `match`.
- Evite criar macros desnecessários. A maioria dos casos pode ser resolvida com funções normais.
- Utilize o código já existente ao invés de reinventar a roda. Se você precisar de uma funcionalidade que já existe, use-a.
- Cada função deve ter uma única responsabilidade. Se uma função estiver fazendo mais de uma coisa, divida-a em funções menores.
- Evite alta carga cognitiva. Uma função deve ser tão difícil quanto o domínio com a qual ela lida, e não mais do que isso. Se uma função estiver lidando com muitos conceitos diferentes, divida-a em funções menores.
- Por via de regra, evite código duplicado. Porém, não tente eliminar toda duplicação a qualquer custo. Às vezes, a duplicação é necessária para manter o código legível e compreensível. Use o bom senso.
- Siga a filosofia de arquitetura e design da Tenda. Escreva código modular e reutilizável.

#### Código de terceiros

- Prefira a biblioteca padrão do Rust e suas funcionalidades ao invés de criar suas próprias implementações ou de usar bibliotecas externas para funcionalidades comuns.
- Se você pode fazer você mesmo em poucas linhas, faça. Não use bibliotecas externas para isso.
- Para funcionalidades não inclusas na biblioteca padrão e que não são triviais, use bibliotecas externas bem estabelecidas e confiáveis. Verifique a popularidade, manutenção e documentação da biblioteca antes de usá-la.

#### Commits

- Commits devem ser descritivos e claros. Eles devem explicar o que foi feito e por quê, não apenas o que foi alterado.
- Commits em inglês ou português brasileiro serão aceitos. Se estiver em dúvidas sobre qual idioma usar, prefira o inglês, pois é o idioma mais comum na comunidade de desenvolvimento.
- Use o tempo verbal imperativo no início da mensagem do commit, como se estivesse dando uma ordem.
- Siga conventional commits para mensagens de commit.

### Performance

Performance não significa sempre "o mais rápido possível", mas sim atender às expectativas de desempenho do usuário de forma consistente e sustentável. As diretrizes abaixo ajudam a garantir que as otimizações sejam eficazes, seguras e justificadas.

- Evite otimização prematura: siga a ordem "Faça funcionar. Refatore. Otimize." e só complexifique o código quando a melhoria de performance compensar a perda de legibilidade.
- Use ferramentas de profiling para identificar gargalos de performance antes de otimizar. Não otimize sem dados concretos.
- Prefira algoritmos e estruturas de dados eficientes. Escolha a solução mais adequada para o problema em questão, considerando o trade-off entre complexidade e performance.

#### Boas práticas de código

- Use iteradores zero-cost: iteradores são lazy, eliminam alocações desnecessárias e removem checagens de índice em loops quando o compilador consegue inferir limites.
- Minimize alocações dinâmicas: prefira reutilizar Vec e buffers prealocados; evite criar e descartar String ou coleções em cada iteração crítica.
- Evite dispatch dinâmico sempre que possível: substitua `Box<dyn Trait>` por generics monomorfizados para reduzir indireções e overhead de vtable.
- Avalie o uso de paralelismo quando houver tarefas independentes que possam ser executadas simultaneamente.

Para mais detalhes sobre performance, consulte a [Rust Performance Book](https://nnethercote.github.io/perf-book/introduction.html).

## Documentos externos

Esse documento é a autoridade final sobre o estilo de código da Tenda. No entanto, existem outros documentos que podem ser úteis para entender melhor as diretrizes e filosofias por trás do estilo de código:

- [Rust Style Guide](https://doc.rust-lang.org/style-guide/)
- [Rust Design Patterns](https://rust-unofficial.github.io/patterns/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)

## Palavras finais

Não se sinta pressionado a acertar tudo de primeira. Valorizamos a melhora e a revisão contínua. Ninguém escreve código perfeito. No processo de revisão de código, você pode receber feedback sobre o estilo e a qualidade do seu código. Use isso como uma oportunidade de aprendizado e crescimento.
