import { beforeEach, describe, expect, it, vi } from "vitest";
import { listProjects } from "./projects-api";
import { listTasks } from "./tasks-api";
import { loadGlobalSearch } from "./global-search";

vi.mock("./projects-api", () => ({ listProjects: vi.fn() }));
vi.mock("./tasks-api", () => ({ listTasks: vi.fn() }));

const project = {
  id: "p-1",
  workspaceId: "personal",
  name: "Moon Base",
  projectKey: "MOON",
  description: "Habitat research",
  goal: "Design a lunar habitat",
  status: "active" as const,
  archivedAt: null,
  createdAt: "2026-10-01T00:00:00Z",
  updatedAt: "2026-10-02T00:00:00Z",
};

const task = {
  id: "t-1",
  workspaceId: "personal",
  projectId: "p-1",
  parentTaskId: null,
  title: "Build the habitat",
  description: "Review habitat materials",
  dueAt: null,
  status: "in_progress" as const,
  priority: "p1" as const,
  createdAt: "2026-10-01T00:00:00Z",
  updatedAt: "2026-10-02T00:00:00Z",
  archivedAt: null,
};

beforeEach(() => {
  vi.mocked(listProjects).mockResolvedValue([project]);
  vi.mocked(listTasks).mockResolvedValue([task]);
});

describe("M2-034 global search adapters", () => {
  it("loads typed, locally scoped Project, Task and registered command results", async () => {
    const { results, failedAdapters } = await loadGlobalSearch();
    expect(failedAdapters).toEqual([]);
    expect(results.find((result) => result.entityType === "project")).toMatchObject({
      id: "project:p-1",
      targetId: "p-1",
      title: "Moon Base",
      description: "active · Habitat research · Design a lunar habitat",
      category: "Project",
      risk: "read-only",
    });
    expect(results.find((result) => result.entityType === "task")).toMatchObject({
      id: "task:t-1",
      targetId: "t-1",
      title: "Build the habitat",
      description: "in progress · Review habitat materials",
      category: "Task",
      risk: "read-only",
    });
    expect(
      results.some((result) => result.entityType === "command" && result.id === "task.create"),
    ).toBe(true);
    expect(listProjects).toHaveBeenCalledOnce();
    expect(listTasks).toHaveBeenCalledOnce();
  });

  it("keeps available sources and registered commands when one local source fails", async () => {
    vi.mocked(listTasks).mockRejectedValueOnce(new Error("unavailable"));
    const { results, failedAdapters } = await loadGlobalSearch();
    expect(failedAdapters).toEqual(["task"]);
    expect(results.some((result) => result.entityType === "project")).toBe(true);
    expect(results.some((result) => result.entityType === "command")).toBe(true);
    expect(results.some((result) => result.entityType === "task")).toBe(false);
  });
});
