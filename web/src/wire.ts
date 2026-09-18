// Generated from Rust; run scripts/generate-contracts.
export type CommandKind = "run" | "remember_project" | "edit_draft" | "start_work" | "retry_task" | "create_mission" | "pause" | "stop" | "cancel" | "resume" | "take" | "submit_human" | "grant_allowance" | "resolve_decision";;
export type Command = { schema_version: number, command_id: string, kind: CommandKind, target_id: string | null, expected_version: number | null, payload: unknown, };;
export type Goal = { goal: string, project_id: string, };;
