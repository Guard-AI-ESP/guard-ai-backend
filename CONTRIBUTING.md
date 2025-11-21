# Règles de développement Guard AI

## 1. Branches Git

- Branche stable : `main`
- Branche d'intégration : `dev`
- Une branche par issue

Format des branches :

`<issueId>_<type>/<team>/<short-feature-name>`

Exemples :

- `012_dev/ia/face-recognition-pipeline`
- `034_fix/backend/camera-stream-timeout`
- `051_spike/iot/test-mqtt-broker`

Types : `dev`, `fix`, `spike`, `doc`, `chore`  
Teams : `frontend`, `backend`, `ia`, `iot`, `infra`, `all`

## 2. Commits

On utilise Conventional Commits :

`<type>(<scope>): <message>`

Types : `feat`, `fix`, `refactor`, `docs`, `test`, `chore`, `style`

Exemples :

- `feat(ia): ajouter detection visage`
- `fix(api): corriger crash si camera est offline`

## 3. Issues

Chaque issue doit contenir :

- Un titre clair : `[IA] Détection visage caméra entrée`
- Des labels : `ia`, `feature`, etc.
- Un assignee

## 4. Pull Requests

- Toujours lier la PR à une issue (`Closes #...`)
- Au moins 1 review avant merge
- Taille raisonnable

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
