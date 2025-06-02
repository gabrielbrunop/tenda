# Workflow

Este documento descreve o fluxo de trabalho Git e CI/CD usado neste projeto.

## Visão geral

```
feat/* → dev → main → tag vX.Y.Z → Publicação
```

- `feat/*`: Desenvolvimento individual de funcionalidades e correções.
- `dev`: Branch para integração contínua dos recursos.
- `main`: Branch contendo a última versão estável.
- `tag vX.Y.Z`: Tag SemVer que dispara a publicação via CI/CD.

## Branches

| Branch   | Descrição                       |
| -------- | ------------------------------- |
| `feat/*` | Funcionalidades e correções     |
| `dev`    | Integração contínua             |
| `main`   | Última versão estável publicada |

## Desenvolvimento

### Nova funcionalidade

```bash
git switch dev
git pull
git switch -c feat/nova-funcionalidade
# Desenvolva e faça commits...
git push origin feat/nova-funcionalidade
```

Abra pull request para `dev`.

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

## Contribuições externas

Colaboradores externos normalmente fariam um fork e criariam um branch `feat/<sua-funcionalidade>` a partir de `dev`, para depois abrir um pull request; **no entanto, atualmente este projeto está fechado a contribuições externas**. Se você tiver sugestões ou encontrar bugs, por favor abra uma **issue** no repositório do GitHub.

## Fluxo resumido

```
feat/* (CI/Testes) → dev (CI/Testes) → main (CI/Testes) → tag vX.Y.Z → CI/CD Publicação
```
