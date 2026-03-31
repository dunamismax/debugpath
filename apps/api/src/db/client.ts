import postgres, { type Sql } from 'postgres';

const DEFAULT_MAX_CONNECTIONS = 10;
const DEFAULT_IDLE_TIMEOUT_SECONDS = 20;
const DEFAULT_CONNECT_TIMEOUT_SECONDS = 10;
const DEFAULT_DATABASE_URL = 'postgresql://debugpath:debugpath@localhost:5433/debugpath';

export type Database = Sql<Record<string, never>>;
export type DatabaseExecutor = Sql<Record<string, never>>;

export const requireDatabaseUrl = (databaseUrl = Bun.env.DATABASE_URL ?? DEFAULT_DATABASE_URL) => {
  if (!databaseUrl) {
    throw new Error('DATABASE_URL is required to use the DebugPath database layer.');
  }

  return databaseUrl;
};

export const createDatabase = (
  databaseUrl = requireDatabaseUrl(),
  max = DEFAULT_MAX_CONNECTIONS
): Database =>
  postgres(databaseUrl, {
    connect_timeout: DEFAULT_CONNECT_TIMEOUT_SECONDS,
    idle_timeout: DEFAULT_IDLE_TIMEOUT_SECONDS,
    max,
    onnotice: () => {},
    prepare: false,
  });

let database: Database | null = null;

export const getDatabase = () => {
  if (database) {
    return database;
  }

  database = createDatabase();
  return database;
};

export const closeDatabase = async () => {
  if (!database) {
    return;
  }

  const activeDatabase = database;
  database = null;
  await activeDatabase.end({ timeout: 5 });
};

export const withTransaction = <T>(db: Database, callback: (tx: DatabaseExecutor) => Promise<T>) =>
  db.begin((tx) => callback(tx as unknown as DatabaseExecutor));

export const expectOne = <T>(rows: T[], label: string) => {
  const [row] = rows;

  if (!row) {
    throw new Error(`Expected ${label} to return one row, but none were returned.`);
  }

  return row;
};

export const maybeOne = <T>(rows: T[]) => rows[0] ?? null;
