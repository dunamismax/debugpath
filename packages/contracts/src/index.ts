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

export const workspaceRoleSchema = z.enum(['owner', 'editor', 'viewer']);
export const investigationStatusSchema = z.enum(['draft', 'active', 'resolved', 'archived']);

export const investigationSummarySchema = z.object({
  id: z.string().uuid(),
  workspaceId: z.string().uuid(),
  title: z.string().min(1),
  status: investigationStatusSchema,
  severity: z.enum(['low', 'medium', 'high', 'critical']).nullable(),
  createdAt: z.string().datetime(),
  updatedAt: z.string().datetime(),
});

export const paginatedInvestigationsSchema = z.object({
  items: z.array(investigationSummarySchema),
  total: z.number().int().nonnegative(),
  page: z.number().int().positive(),
  pageSize: z.number().int().positive(),
});

export const healthPayloadSchema = z.object({
  service: z.literal('api'),
  version: z.literal(apiVersion),
  environment: z.string().min(1),
  uptimeSeconds: z.number().int().nonnegative(),
  timestamp: z.string().datetime(),
});

export const healthResponseSchema = okEnvelopeSchema(healthPayloadSchema);

export const createOkEnvelope = <T extends z.ZodTypeAny>(schema: T, data: z.input<T>) =>
  okEnvelopeSchema(schema).parse({
    ok: true,
    data,
    meta: {
      generatedAt: new Date().toISOString(),
    },
  });

export type ErrorEnvelope = z.infer<typeof errorEnvelopeSchema>;
export type HealthPayload = z.infer<typeof healthPayloadSchema>;
export type HealthResponse = z.infer<typeof healthResponseSchema>;
export type InvestigationSummary = z.infer<typeof investigationSummarySchema>;
export type PaginatedInvestigations = z.infer<typeof paginatedInvestigationsSchema>;
export type WorkspaceRole = z.infer<typeof workspaceRoleSchema>;
