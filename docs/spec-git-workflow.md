# Git Workflow Specification

Version control conventions for this project.

---

## 1. Trunk

`master` is the integration branch. It must always build, pass all
tests, and have zero doc warnings. Never commit directly to master —
all changes go through feature branches and pull requests.

---

## 2. Branch Naming

`<type>/<short-description>`

| Type        | Purpose | Example                     |
|-------------|---|-----------------------------|
| `feature/`  | New functionality, new examples | `feature/timer-api`         |
| `fix/`      | Bug fixes | `fix/dropdown-dangling-ptr` |
| `refactor/` | Restructuring without behavior change | `refactor/module-cleanup`   |
| `docs/`     | Documentation only | `docs/example-porting-spec` |

Keep descriptions short and lowercase with hyphens.

---

## 3. Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>: <concise summary>

<optional body — what and why, not how>

Co-Authored-By: <agent> <email>
```

Types: `feat`, `fix`, `refactor`, `docs`, `test`, `ci`, `chore`.

Rules:
- Summary line: imperative mood, lowercase, no period, under 72 chars.
- Body: wrap at 72 chars. Explain motivation and trade-offs, not
  mechanics visible in the diff.
- AI-assisted commits include a `Co-Authored-By` trailer.

---

## 4. Pull Requests

- One PR per logical unit of work. Squash thematic groups of commits
  before opening the PR if the intermediate history is noisy, but keep
  logically distinct changes as separate commits — only squash into a
  single commit when the change is truly trivial.
- PR title follows the same `<type>: <summary>` format.
- PR description includes a summary, list of changes, and a test plan
  with checkboxes.
- Target `master`. For stacked work, target the predecessor branch and
  rebase after it merges.
- CI must pass before merge.

---

## 5. Rebase Policy

- Always start new branches from current `origin/master`.
- Rebase feature branches onto `master` before merging — no merge
  commits in the feature branch.
- Use `--force-with-lease` (never `--force`) when pushing rebased
  branches.
- After rebase, verify the build and run all tests before pushing.

---

## 6. Destructive Operations

Avoid unless explicitly needed:
- `git reset --hard` — prefer stash or a new branch.
- `git push --force` — use `--force-with-lease` instead.
- `git checkout .` / `git restore .` — check `git stash` first.
- `git branch -D` — only for branches confirmed merged or abandoned.
- Never force-push to `master`.

---

## 7. What Goes Into a Commit

- Each commit should leave the tree in a buildable, testable state.
- Separate concerns: don't mix refactoring with feature work in the
  same commit.
- Prefer small, focused commits over large monolithic ones.
- Undo changes that did not lead to proven success — do not leave dead
  experiments in the history.

---

## 8. What Does NOT Go Into the Repository

- IDE configuration (`.idea/`, `.vscode/` — already in `.gitignore`).
- Credentials, secrets, `.env` files.
- Build artifacts, `target/` directory.
- Large binaries not needed for the build (screenshots in
  `examples/doc/screenshots/` are an intentional exception — they serve
  as visual documentation).

---

## 9. CI Integration

CI runs on every push to `master` and on every pull request
(`.github/workflows/ci.yml`). Two parallel jobs:

- **host** — unit, doc, and integration tests on x86\_64 (headless
  SDL2).
- **firmware** — cross-compile for xtensa-esp32-none-elf.

A green CI is required before merge. If CI fails on a PR, fix the issue
and push — do not bypass hooks or skip verification.
