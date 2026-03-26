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

- One PR per higher-level topic (e.g. "observer API"). A PR typically
  spans multiple units of work — core library, examples, tests, docs —
  each as its own commit. Clean the history before opening: squash
  fixups into their parent commits, remove dead-end experiments, but
  keep logically distinct changes as separate commits. The result
  should be a minimal sequence of single-topic commits — never a
  single mega-commit.
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

**One topic per commit.** Each commit addresses exactly one thematic
concern — a new type, a single example, a test batch, a bug fix. Never
mix unrelated changes (e.g. a new API + an unrelated example + test
fixes) in the same commit.

**No mega-squash.** Squashing an entire feature branch into one commit
destroys useful history. Keep logically distinct changes as separate
commits. "One PR = one commit" is only appropriate for truly trivial
changes.

**Clean history before PR.** During development, fixup commits are fine
(e.g. "fix build after rebase", "address review feedback"). Before
opening or updating a PR, squash these fixups into the commits they
correct — the final PR history should read as a clean sequence of
intentional, self-contained changes with no meandering. Use `git reset
--soft` + recommit-by-topic or `git rebase` with fixup/squash.

**Each commit must build and pass tests.** Do not create commits that
break the build with the intent of fixing them in a later commit.

**Undo dead ends.** Changes that did not lead to proven success (e.g.
an approach that was later reverted, examples that were added then
removed) must not appear in the final PR history. Squash them out.

### Commit history anti-patterns

| Anti-pattern | Correct approach |
|---|---|
| Single mega-commit with 20 files | Split by topic: API, example, tests, docs |
| `feat` → `fix: oops` → `fix: really fix` | Squash the fixes into the original feat |
| Add feature → remove feature → re-add differently | Only the final version appears in history |
| Mix library code + example + test in one commit | Separate: library commit, example commit, test commit |
| Intermediate "WIP" or "checkpoint" commits | Squash before PR |

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
