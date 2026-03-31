import { createHash } from 'node:crypto';
import { readdir } from 'node:fs/promises';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

import {
  type DatabaseExecutor,
  createDatabase,
  requireDatabaseUrl,
} from '../../apps/api/src/db/client';

export interface MigrationFile {
  name: string;
  checksum: string;
  sql: string;
}

export interface AppliedMigration {
  name: string;
  checksum: string;
  executedAt: string;
}

export interface MigrationResult {
  applied: string[];
  skipped: string[];
}

const defaultMigrationsDirectory = fileURLToPath(new URL('../migrations', import.meta.url));

const checksumFor = (content: string) => createHash('sha256').update(content).digest('hex');

export const loadMigrationFiles = async (directory = defaultMigrationsDirectory) => {
  const entries = await readdir(directory, { withFileTypes: true });

  const migrationNames = entries
    .filter((entry) => entry.isFile() && entry.name.endsWith('.sql'))
    .map((entry) => entry.name)
    .sort((left, right) => left.localeCompare(right));

  const migrations = await Promise.all(
    migrationNames.map(async (name) => {
      const sql = await Bun.file(join(directory, name)).text();

      return {
        name,
        checksum: checksumFor(sql),
        sql,
      } satisfies MigrationFile;
    })
  );

  return migrations;
};

const ensureSchemaMigrationsTable = async (database: ReturnType<typeof createDatabase>) => {
  await database`
    create table if not exists schema_migrations (
      name text primary key,
      checksum text not null,
      executed_at timestamptz not null default now()
    )
  `;
};

export const migrateDatabase = async ({
  databaseUrl = requireDatabaseUrl(),
  migrationsDirectory = defaultMigrationsDirectory,
  quiet = false,
}: {
  databaseUrl?: string;
  migrationsDirectory?: string;
  quiet?: boolean;
} = {}) => {
  const database = createDatabase(databaseUrl, 1);

  try {
    await ensureSchemaMigrationsTable(database);

    const appliedRows = await database<AppliedMigration[]>`
      select name, checksum, executed_at as "executedAt"
      from schema_migrations
      order by name asc
    `;

    const appliedByName = new Map(appliedRows.map((row) => [row.name, row]));
    const migrations = await loadMigrationFiles(migrationsDirectory);
    const result: MigrationResult = {
      applied: [],
      skipped: [],
    };

    for (const migration of migrations) {
      const existing = appliedByName.get(migration.name);

      if (existing) {
        if (existing.checksum !== migration.checksum) {
          throw new Error(
            `Migration ${migration.name} was already applied with checksum ${existing.checksum}, ` +
              `but the current file checksum is ${migration.checksum}.`
          );
        }

        result.skipped.push(migration.name);
        continue;
      }

      await database.begin(async (tx) => {
        const transaction = tx as unknown as DatabaseExecutor;

        await transaction.unsafe(migration.sql);
        await transaction`
          insert into schema_migrations (name, checksum)
          values (${migration.name}, ${migration.checksum})
        `;
      });

      result.applied.push(migration.name);
    }

    if (!quiet) {
      if (result.applied.length > 0) {
        console.log(`Applied migrations: ${result.applied.join(', ')}`);
      }

      if (result.skipped.length > 0) {
        console.log(`Already applied: ${result.skipped.join(', ')}`);
      }

      if (result.applied.length === 0 && result.skipped.length === 0) {
        console.log('No migrations found.');
      }
    }

    return result;
  } finally {
    await database.end({ timeout: 5 });
  }
};

if (import.meta.main) {
  await migrateDatabase();
}
