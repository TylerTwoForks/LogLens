export interface paths {
    "/health": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["health"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/openapi.json": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["openapi_json"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/v1/me": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["me"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/v1/me/license": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch: operations["update_me_license"];
        trace?: never;
    };
    "/v1/orgs": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["list_orgs"];
        put?: never;
        post: operations["create_org"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/v1/orgs/{org_id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["get_org"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/v1/orgs/{org_id}/jobs": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["list_jobs"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/v1/orgs/{org_id}/jobs/{job_id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["get_job"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/v1/orgs/{org_id}/jobs/{job_id}/benchmarks": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["list_job_benchmarks"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/v1/orgs/{org_id}/jobs/{job_id}/events": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["list_job_events"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/v1/orgs/{org_id}/license": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch: operations["update_org_license"];
        trace?: never;
    };
    "/v1/orgs/{org_id}/members": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["list_org_members"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/v1/orgs/{org_id}/members/{member_user_id}/role": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch: operations["update_org_member_role"];
        trace?: never;
    };
    "/v1/orgs/{org_id}/uploads": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post: operations["upload_logs"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/version": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["version"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
}
export type webhooks = Record<string, never>;
export interface components {
    schemas: {
        BenchmarkSnapshotResponse: {
            /** Format: int64 */
            cpu_time_delta: number;
            /** Format: int64 */
            cpu_time_limit: number;
            /** Format: int64 */
            cpu_time_ms: number;
            /** Format: int64 */
            dml_statements: number;
            /** Format: int64 */
            dml_statements_limit: number;
            /** Format: int64 */
            heap_size_bytes_limit: number;
            /** Format: double */
            heap_size_delta: number;
            /** Format: double */
            heap_size_pct: number;
            label: string;
            /** Format: int64 */
            query_rows: number;
            /** Format: int64 */
            query_rows_delta: number;
            /** Format: int64 */
            query_rows_limit: number;
            /** Format: int32 */
            sequence: number;
            /** Format: int64 */
            soql_queries: number;
            /** Format: int64 */
            soql_queries_limit: number;
        };
        CreateOrgRequest: {
            name: string;
        };
        ErrorResponse: {
            error: string;
        };
        HealthResponse: {
            database: string;
            status: string;
        };
        /** @enum {string} */
        JobStatus: "queued" | "running" | "done" | "failed";
        LicenseSnapshot: {
            features: string[];
            status: components["schemas"]["LicenseStatus"];
            tier: components["schemas"]["LicenseTier"];
        };
        /** @enum {string} */
        LicenseStatus: "active" | "past_due" | "canceled";
        /** @enum {string} */
        LicenseTier: "free" | "pro" | "enterprise";
        ListBenchmarksResponse: {
            benchmarks: components["schemas"]["BenchmarkSnapshotResponse"][];
        };
        ListEventsQuery: {
            event_type?: string | null;
            /** Format: int64 */
            limit?: number | null;
            log_level?: string | null;
            /** Format: int64 */
            offset?: number | null;
            search?: string | null;
        };
        ListEventsResponse: {
            events: components["schemas"]["LogEventResponse"][];
            /** Format: int64 */
            total: number;
        };
        ListJobsQuery: {
            status?: string | null;
        };
        ListJobsResponse: {
            jobs: components["schemas"]["ParseJobResponse"][];
        };
        ListOrgsResponse: {
            orgs: components["schemas"]["OrgSummary"][];
        };
        LogEventResponse: {
            event_type: string;
            /** Format: int32 */
            line_index: number;
            /** Format: int32 */
            line_number?: number | null;
            log_level?: string | null;
            message: string;
            /** Format: int64 */
            nanos?: number | null;
            timestamp: string;
        };
        MeResponse: {
            auth_subject: string;
            email: string;
            individual_license: components["schemas"]["LicenseSnapshot"];
            /** Format: int64 */
            user_id: number;
        };
        MutationResponse: {
            message: string;
        };
        OrgMember: {
            email: string;
            role: components["schemas"]["OrgRole"];
            /** Format: int64 */
            user_id: number;
        };
        OrgMembersResponse: {
            members: components["schemas"]["OrgMember"][];
        };
        /** @enum {string} */
        OrgRole: "owner" | "admin" | "member" | "viewer";
        OrgSummary: {
            license: components["schemas"]["LicenseSnapshot"];
            name: string;
            /** Format: int64 */
            org_id: number;
            role: components["schemas"]["OrgRole"];
        };
        ParseJobResponse: {
            /** Format: int32 */
            benchmark_count: number;
            created_at: string;
            error_message?: string | null;
            file_name: string;
            finished_at?: string | null;
            /** Format: int64 */
            job_id: number;
            /** Format: int64 */
            org_id: number;
            /** Format: int64 */
            parsed_lines: number;
            started_at?: string | null;
            status: components["schemas"]["JobStatus"];
            /** Format: int64 */
            total_lines: number;
        };
        UpdateLicenseRequest: {
            status: components["schemas"]["LicenseStatus"];
            tier: components["schemas"]["LicenseTier"];
        };
        UpdateMemberRoleRequest: {
            role: components["schemas"]["OrgRole"];
        };
        UploadResponse: {
            jobs: components["schemas"]["ParseJobResponse"][];
        };
        VersionResponse: {
            version: string;
        };
    };
    responses: never;
    parameters: never;
    requestBodies: never;
    headers: never;
    pathItems: never;
}
export type $defs = Record<string, never>;
export interface operations {
    health: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Service healthy */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["HealthResponse"];
                };
            };
            /** @description Database unavailable */
            503: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["HealthResponse"];
                };
            };
        };
    };
    openapi_json: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description OpenAPI contract */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
        };
    };
    me: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Current authenticated user context */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["MeResponse"];
                };
            };
            /** @description Missing or invalid auth context */
            401: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ErrorResponse"];
                };
            };
        };
    };
    update_me_license: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["UpdateLicenseRequest"];
            };
        };
        responses: {
            /** @description Updated individual license */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["LicenseSnapshot"];
                };
            };
            /** @description Missing or invalid auth context */
            401: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ErrorResponse"];
                };
            };
        };
    };
    list_orgs: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Organizations for authenticated user */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ListOrgsResponse"];
                };
            };
            /** @description Missing or invalid auth context */
            401: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ErrorResponse"];
                };
            };
        };
    };
    create_org: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["CreateOrgRequest"];
            };
        };
        responses: {
            /** @description Created organization with owner membership */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["OrgSummary"];
                };
            };
            /** @description Invalid organization payload */
            400: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ErrorResponse"];
                };
            };
            /** @description Missing or invalid auth context */
            401: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ErrorResponse"];
                };
            };
        };
    };
    get_org: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                /** @description Organization identifier */
                org_id: number;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Organization summary for current member */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["OrgSummary"];
                };
            };
            /** @description Missing or invalid auth context */
            401: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ErrorResponse"];
                };
            };
            /** @description Cross-org access denied */
            403: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ErrorResponse"];
                };
            };
        };
    };
    list_jobs: {
        parameters: {
            query?: {
                /** @description Filter by job status */
                status?: string;
            };
            header?: never;
            path: {
                /** @description Organization identifier */
                org_id: number;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Parse jobs for this organization */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ListJobsResponse"];
                };
            };
            /** @description Missing or invalid auth context */
            401: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ErrorResponse"];
                };
            };
            /** @description Cross-org access denied */
            403: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ErrorResponse"];
                };
            };
        };
    };
    get_job: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                /** @description Organization identifier */
                org_id: number;
                /** @description Parse job identifier */
                job_id: number;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Parse job status */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ParseJobResponse"];
                };
            };
            /** @description Missing or invalid auth context */
            401: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ErrorResponse"];
                };
            };
            /** @description Cross-org access denied */
            403: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ErrorResponse"];
                };
            };
            /** @description Job not found */
            404: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ErrorResponse"];
                };
            };
        };
    };
    list_job_benchmarks: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                /** @description Organization identifier */
                org_id: number;
                /** @description Parse job identifier */
                job_id: number;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Benchmark snapshots for completed parse job */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ListBenchmarksResponse"];
                };
            };
            /** @description Missing or invalid auth context */
            401: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ErrorResponse"];
                };
            };
            /** @description Cross-org access denied */
            403: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ErrorResponse"];
                };
            };
            /** @description Job not found */
            404: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ErrorResponse"];
                };
            };
        };
    };
    list_job_events: {
        parameters: {
            query?: {
                /** @description Pagination offset */
                offset?: number;
                /** @description Pagination limit (max 500) */
                limit?: number;
                /** @description Filter by event type */
                event_type?: string;
                /** @description Filter by log level */
                log_level?: string;
                /** @description Search message text */
                search?: string;
            };
            header?: never;
            path: {
                /** @description Organization identifier */
                org_id: number;
                /** @description Parse job identifier */
                job_id: number;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Parsed log events */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ListEventsResponse"];
                };
            };
            /** @description Missing or invalid auth context */
            401: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ErrorResponse"];
                };
            };
            /** @description Cross-org access denied */
            403: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ErrorResponse"];
                };
            };
            /** @description Job not found */
            404: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ErrorResponse"];
                };
            };
        };
    };
    update_org_license: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                /** @description Organization identifier */
                org_id: number;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["UpdateLicenseRequest"];
            };
        };
        responses: {
            /** @description Updated organization license */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["LicenseSnapshot"];
                };
            };
            /** @description Missing or invalid auth context */
            401: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ErrorResponse"];
                };
            };
            /** @description Role is not allowed to manage billing */
            403: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ErrorResponse"];
                };
            };
        };
    };
    list_org_members: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                /** @description Organization identifier */
                org_id: number;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Organization members */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["OrgMembersResponse"];
                };
            };
            /** @description Missing or invalid auth context */
            401: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ErrorResponse"];
                };
            };
            /** @description Cross-org access denied */
            403: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ErrorResponse"];
                };
            };
        };
    };
    update_org_member_role: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                /** @description Organization identifier */
                org_id: number;
                /** @description Target member user identifier */
                member_user_id: number;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["UpdateMemberRoleRequest"];
            };
        };
        responses: {
            /** @description Updated member role */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["MutationResponse"];
                };
            };
            /** @description Missing or invalid auth context */
            401: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ErrorResponse"];
                };
            };
            /** @description Role is not allowed to manage members */
            403: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ErrorResponse"];
                };
            };
            /** @description Target member not found in organization */
            404: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ErrorResponse"];
                };
            };
        };
    };
    upload_logs: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                /** @description Organization identifier */
                org_id: number;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Log files accepted and parse jobs created */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["UploadResponse"];
                };
            };
            /** @description No valid log files in upload */
            400: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ErrorResponse"];
                };
            };
            /** @description Missing or invalid auth context */
            401: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ErrorResponse"];
                };
            };
            /** @description Cross-org access denied */
            403: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ErrorResponse"];
                };
            };
        };
    };
    version: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Service version */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["VersionResponse"];
                };
            };
        };
    };
}
