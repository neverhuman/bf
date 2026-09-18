export type Project = {id: string; name: string; path: string; base_oid: string; mode: "fixture" | "live"};
export type Plan = {objective: string; acceptance: {id: string; statement: string; check_ids: string[]}[]; scope: string[]; account: string; model: string; limits: {write_invocations: number; runtime_seconds: number}; checks: string[]; base_oid: string; fixture_only: boolean};
export type Draft = {id: string; mission_id: string; project_id: string; revision: number; goal: string; plan: Plan | null; status: string; mission_version: number; paused: boolean};
export type WorkItem = {cursor: number; task_id: string; mission_id: string; version: number; title: string; phase: string; why: string; pr: {number: number; url: string; fixture_only: boolean} | null};
export type Detail = {conversation: {seq: number; role: string; body: string}[]; jobs: {id: string; purpose: string; lifecycle: string; occupancy: string; authority: string; generation: number}[]};
export type Operation = {id: string; status: string; result: {draft_id?: string; status?: string; revision?: number}};
import type {Command as WireCommand, Goal} from "./wire";
export type Command = Omit<WireCommand,"payload"> & {payload: Goal | Record<string, never>};
