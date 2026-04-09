# Règles de développement Guard AI

## 1. Branches Git

### Branches permanentes

| Branche | Rôle | Push direct |
|---------|------|-------------|
| `main` | Production — stable, déployée | Non |
| `staging` | Intégration — recette avant prod | Non |

### Branches de travail

Format : `<type>/<scope>`

```
feat/<scope>    ← nouvelle fonctionnalité
fix/<scope>     ← correction de bug
chore/<scope>   ← maintenance, deps, config
docs/<scope>    ← documentation
```

Exemples :
- `feat/backend/websocket-auth`
- `fix/backend/event-pagination`
- `chore/backend/upgrade-sqlx`
- `docs/backend/api-endpoints`

### Flux

```
feat/* ──PR──► staging ──PR──► main
fix/*  ──PR──► staging ──PR──► main
```

## 2. Commits

On utilise [Conventional Commits](https://www.conventionalcommits.org/) :

`<type>(<scope>): <message>`

Types : `feat`, `fix`, `refactor`, `docs`, `test`, `chore`, `style`

Exemples :
- `feat(auth): add X-API-Key validation on websocket`
- `fix(events): correct pagination offset calculation`
- `chore(deps): upgrade axum to 0.8`

## 3. Issues

Chaque issue doit contenir :

- Un titre clair : `[backend] Timeout sur le stream WebSocket`
- Des labels : `backend`, `bug`, `feature`, etc.
- Un assignee

## 4. Pull Requests

- Toujours cibler `staging`, jamais `main` directement
- Toujours lier la PR à une issue (`Closes #...`)
- Au moins 1 review avant merge vers `staging`
- Au moins 1 review avant merge vers `main`
- Squash merge préféré pour garder un historique propre

Template résumé :
- Description
- Lié à
- Type de changement
- Comment tester
- Checklist (tests, logs, doc)

## 5. Qualité / CI

- Pas de merge si les tests ne passent pas
- Lint obligatoire sur les langages utilisés
- Objectif : ajouter des tests dès qu'on touche à du code critique

## 6. Contact / Rôles

- Lead dev : ...
- Référent IA : ...
- Référent IOT : ...
- Référent frontend : ...
