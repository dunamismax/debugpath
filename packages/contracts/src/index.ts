import { z } from 'zod';

export const appName = 'DebugPath';
export const apiVersion = 'v1' as const;
export const apiBasePath = `/api/${apiVersion}` as const;

export const responseMetaSchema = z.object({
  generatedAt: z.string().datetime(),
});

export const okEnvelopeSchema = <T extends z.ZodTypeAny>(dataSchema: T) =>
  z.object({
    ok: z.literal(true),
    data: dataSchema,
    meta: responseMetaSchema,
  });

export const errorEnvelopeSchema = z.object({
  ok: z.literal(false),
  error: z.object({
    code: z.string().min(1),
    message: z.string().min(1),
  }),
  meta: responseMetaSchema,
});

const uuidSchema = z.string().uuid();
const emailSchema = z.string().trim().email();
const datetimeSchema = z.string().datetime();
const nullableDatetimeSchema = datetimeSchema.nullable();

export const workspaceRoleSchema = z.enum(['owner', 'editor', 'viewer']);
export const investigationStatusSchema = z.enum(['draft', 'active', 'resolved', 'archived']);
export const investigationSeveritySchema = z.enum(['low', 'medium', 'high', 'critical']).nullable();

export const sessionUserSchema = z.object({
  id: uuidSchema,
  email: emailSchema,
  displayName: z.string().nullable(),
});

export const workspaceSummarySchema = z.object({
  id: uuidSchema,
  slug: z.string().min(1),
  name: z.string().min(1),
  role: workspaceRoleSchema,
});

export const investigationSummarySchema = z.object({
  id: uuidSchema,
  workspaceId: uuidSchema,
  slug: z.string().min(1),
  title: z.string().min(1),
  summary: z.string().nullable(),
  status: investigationStatusSchema,
  severity: investigationSeveritySchema,
  archivedAt: nullableDatetimeSchema,
  createdAt: datetimeSchema,
  updatedAt: datetimeSchema,
});

export const investigationDetailSchema = investigationSummarySchema.extend({
  createdByUserId: uuidSchema,
});

export const paginatedInvestigationsSchema = z.object({
  items: z.array(investigationSummarySchema),
  total: z.number().int().nonnegative(),
  page: z.number().int().positive(),
  pageSize: z.number().int().positive(),
});

export const authStatusSchema = z.object({
  authenticated: z.boolean(),
  user: sessionUserSchema.nullable(),
  currentWorkspace: workspaceSummarySchema.nullable(),
});

export const appShellSchema = z.object({
  currentUser: sessionUserSchema,
  currentWorkspace: workspaceSummarySchema,
  workspaces: z.array(workspaceSummarySchema),
  investigations: z.array(investigationSummarySchema),
});

export const loginInputSchema = z.object({
  email: emailSchema,
  password: z.string().min(12).max(128),
});

export const registerInputSchema = loginInputSchema.extend({
  displayName: z.string().trim().min(1).max(80),
});

export const createInvestigationInputSchema = z.object({
  title: z.string().trim().min(3).max(140),
  summary: z.string().trim().max(4000).nullable().optional(),
  severity: investigationSeveritySchema.optional(),
});

export const updateInvestigationInputSchema = z.object({
  title: z.string().trim().min(3).max(140),
  summary: z.string().trim().max(4000).nullable().optional(),
  severity: investigationSeveritySchema.optional(),
  status: investigationStatusSchema.optional(),
});

export const healthPayloadSchema = z.object({
  service: z.literal('api'),
  version: z.literal(apiVersion),
  environment: z.string().min(1),
  uptimeSeconds: z.number().int().nonnegative(),
  timestamp: z.string().datetime(),
});

export const healthResponseSchema = okEnvelopeSchema(healthPayloadSchema);
export const authStatusResponseSchema = okEnvelopeSchema(authStatusSchema);
export const appShellResponseSchema = okEnvelopeSchema(appShellSchema);
export const investigationDetailResponseSchema = okEnvelopeSchema(investigationDetailSchema);

export const createOkEnvelope = <T extends z.ZodTypeAny>(schema: T, data: z.input<T>) =>
  okEnvelopeSchema(schema).parse({
    ok: true,
    data,
    meta: {
      generatedAt: new Date().toISOString(),
    },
  });

export const createErrorEnvelope = (code: string, message: string) =>
  errorEnvelopeSchema.parse({
    ok: false,
    error: {
      code,
      message,
    },
    meta: {
      generatedAt: new Date().toISOString(),
    },
  });

export type ErrorEnvelope = z.infer<typeof errorEnvelopeSchema>;
export type HealthPayload = z.infer<typeof healthPayloadSchema>;
export type HealthResponse = z.infer<typeof healthResponseSchema>;
export type InvestigationSummary = z.infer<typeof investigationSummarySchema>;
export type InvestigationDetail = z.infer<typeof investigationDetailSchema>;
export type PaginatedInvestigations = z.infer<typeof paginatedInvestigationsSchema>;
export type SessionUser = z.infer<typeof sessionUserSchema>;
export type WorkspaceRole = z.infer<typeof workspaceRoleSchema>;
export type WorkspaceSummary = z.infer<typeof workspaceSummarySchema>;
export type AuthStatus = z.infer<typeof authStatusSchema>;
export type AppShell = z.infer<typeof appShellSchema>;
export type LoginInput = z.infer<typeof loginInputSchema>;
export type RegisterInput = z.infer<typeof registerInputSchema>;
export type CreateInvestigationInput = z.infer<typeof createInvestigationInputSchema>;
export type UpdateInvestigationInput = z.infer<typeof updateInvestigationInputSchema>;
