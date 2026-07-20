# tools/

Curated, reusable maintenance helpers. Not a junk drawer — one-off scripts
belong in the gitignored `work/`, and a tool that stops being used should be
deleted rather than left to rot.

| Tool | Purpose |
|------|---------|
| [`regen-docsrs-bindings.sh`](regen-docsrs-bindings.sh) | Refresh `oxivgl-sys/bindings_docsrs.rs`, the pre-generated bindings docs.rs uses because it has no network access. **Run this whenever `oxivgl-sys/default-conf/lv_conf.h` changes.** A stale snapshot does not fail any build — it silently publishes documentation with modules missing. |
