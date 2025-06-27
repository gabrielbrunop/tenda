# Guia para escrever testes

## Diretrizes

Para escrever testes, siga estas diretrizes:

- Siga as convenções de nomenclatura e organização dos testes já existentes.
- Para testes internos e unitários de módulos do projeto, crie um submódulo `tests` dentro do módulo que você está testando.
  - Para módulos grandes, se você tiver um módulo `foo.rs`, crie um submódulo `foo/tests.rs` e escreva seus testes lá.
  - Para módulos pequenos e triviais, você pode criar um submódulo `tests` diretamente no arquivo do módulo, com `#[cfg(test)] mod tests { ... }`.
  - Quando um módulo começar a crescer e ter muitos testes, considere movê-los para um arquivo separado `tests.rs` dentro do diretório do módulo.
- Para testes de integração que envolvem múltiplos módulos ou o sistema como um todo, utilize o _crate_ `tests` na raiz do projeto. Siga as convenções dos testes já existentes nesse diretório.

Também siga as diretrizes e recomendações oficiais do Rust:

- [Como escrever testes](https://doc.rust-lang.org/book/ch11-01-writing-tests.html)
- [Organização de testes](https://doc.rust-lang.org/book/ch11-03-test-organization.html)
