# Fluxo de trabalho de desenvolvimento

Este documento descreve o fluxo de trabalho Git e CI/CD usado neste projeto.

## Colaboração externa

Colaboradores devem ler os demais documentos do projeto antes de contribuir, em especial o [guia de contribuição](/CONTRIBUTING.md) e o [código de conduta](/CODE_OF_CONDUCT.md). Pull requests de colaboradores externos são bem-vindos, mas é necessário seguir as diretrizes estabelecidas.

## Branches

| Branch     | Descrição                         |
| ---------- | --------------------------------- |
| `feat/*`   | Funcionalidades e correções       |
| `hotfix/*` | Correções críticas e emergenciais |
| `dev`      | Integração contínua               |
| `main`     | Última versão estável publicada   |

## Desenvolvimento

### Nova funcionalidade

Crie um novo branch a partir de `dev` para desenvolver uma nova funcionalidade. Para colaboradores externos, comece com um fork do repositório.

```bash
git switch dev
git pull
git switch -c feat/nova-funcionalidade
# Desenvolva e faça commits...
git push origin feat/nova-funcionalidade
```

Quando estiver pronto, abra um pull request para `dev`.

#### Correção crítica

Se for uma correção crítica, use `hotfix/*` em vez de `feat/*`, e abra o pull request diretamente para `main`.

### Revisão de código

Após abrir o pull request, aguarde a revisão dos mantenedores. Eles podem solicitar ajustes ou aprovar o PR. Veja as [diretrizes de revisão de código no guia de contribuição](/CONTRIBUTING.md#revisão-de-código) para mais detalhes.

### Preparação de release

No branch `dev`:

```bash
cargo set-version minor --workspace
git commit -am "chore: vX.Y.Z"
git push
```

Abra pull request para `main`. Após revisão e CI aprovados, faça merge.

### Gerando a versão oficial

No branch `main`:

```bash
git tag -a vX.Y.Z -m "Versão vX.Y.Z"
git push origin vX.Y.Z
```

## Fluxo resumido

```
feat/* (CI/Testes) → dev (CI/Testes) → main (CI/Testes) → tag vX.Y.Z → CI/CD Publicação
```

- `feat/*`: Desenvolvimento individual de funcionalidades e correções.
- `dev`: Branch para integração contínua dos recursos.
- `main`: Branch contendo a última versão estável.
- `tag vX.Y.Z`: Tag SemVer que dispara a publicação via CI/CD.
