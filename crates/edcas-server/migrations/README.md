# Migrations

Database migrations are applied **automatically on server startup** by the embedded runner
in [`../src/migrations.rs`](../src/migrations.rs), tracked in the `schema_migrations` table
so each runs exactly once.

- The authoritative, full schema is [`../schema.sql`](../schema.sql), embedded as migration
  `0001_initial_schema`. It uses `CREATE … IF NOT EXISTS`, so it both initialises a fresh
  database and safely adopts an existing one (creating only what's missing).
- To add a change: create `NNNN_description.sql` here and append a matching
  `(version, include_str!("../migrations/NNNN_description.sql"))` entry to `MIGRATIONS` in
  `../src/migrations.rs`. Never edit or reorder entries that may already be applied in
  production.

## Legacy files

The dated `2026MMDD_*.sql` files predate the embedded runner and were applied by hand. They
are kept for historical reference only — their end state is already folded into `schema.sql`,
and they are **not** run automatically. New databases (and existing ones, on the next server
start) are brought fully up to date by the runner; you do not need to apply these manually.
