#!/usr/bin/env node
/**
 * Validate the Moonships task backlog and relative Markdown links.
 * Uses only Node.js built-ins so it can run before dependencies are installed.
 */
import fs from "node:fs";
import path from "node:path";

const root = process.cwd();
const markdownFiles = [];
const ignoredDirectories = new Set([".git", "node_modules", "target", "dist", "coverage"]);

function walk(directory) {
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    if (entry.isDirectory()) {
      if (!ignoredDirectories.has(entry.name)) walk(path.join(directory, entry.name));
    } else if (entry.isFile() && entry.name.toLowerCase().endsWith(".md")) {
      markdownFiles.push(path.relative(root, path.join(directory, entry.name)).replaceAll(path.sep, "/"));
    }
  }
}

walk(root);

const errors = [];
const fileContents = new Map(markdownFiles.map((file) => [file, fs.readFileSync(path.join(root, file), "utf8")]));
const backlogPath = "docs/TASK_BACKLOG.md";
const milestonePath = "docs/MILESTONES.md";
const backlog = fileContents.get(backlogPath);
const milestones = fileContents.get(milestonePath);

if (!backlog) errors.push(`Missing required file: ${backlogPath}`);
if (!milestones) errors.push(`Missing required file: ${milestonePath}`);

const taskRegex = /^- \[([ xX])\] \*\*([A-Z]\d+-\d{3}) — ([^*]+)\*\*(.*)$/gm;
const tasks = [];
if (backlog) {
  let match;
  while ((match = taskRegex.exec(backlog)) !== null) {
    const lineNumber = backlog.slice(0, match.index).split("\n").length;
    const task = {
      checked: match[1].toLowerCase() === "x",
      id: match[2],
      title: match[3].trim(),
      tail: match[4].trim(),
      lineNumber,
    };
    if (!task.tail.includes("**AC:**")) {
      errors.push(`${backlogPath}:${lineNumber} ${task.id} has no **AC:** acceptance criterion.`);
    }
    tasks.push(task);
  }
}

if (tasks.length === 0) errors.push("No atomic task lines were found.");
const ids = new Set();
for (const task of tasks) {
  if (ids.has(task.id)) errors.push(`Duplicate task ID: ${task.id}`);
  ids.add(task.id);
}

const milestoneTitles = new Map();
if (milestones) {
  for (const match of milestones.matchAll(/^## (M\d+) — (.+)$/gm)) {
    milestoneTitles.set(match[1], match[2].trim());
  }
}

function expandDependencyToken(token, taskId) {
  const range = token.match(/^([A-Z]\d+)-(\d{3})\.\.([A-Z]\d+)-(\d{3})$/);
  if (range) {
    if (range[1] !== range[3]) {
      errors.push(`${taskId} has a cross-phase dependency range: ${token}`);
      return [];
    }
    const first = Number(range[2]);
    const last = Number(range[4]);
    if (last < first || last - first > 500) {
      errors.push(`${taskId} has an invalid dependency range: ${token}`);
      return [];
    }
    return Array.from({ length: last - first + 1 }, (_, index) =>
      `${range[1]}-${String(first + index).padStart(3, "0")}`,
    );
  }
  return [token];
}

const dependencyMap = new Map();
for (const task of tasks) {
  const match = task.tail.match(/\[depends:\s*([^\]]+)\]/i);
  const dependencies = match
    ? match[1].split(/[\s,]+/).filter(Boolean).flatMap((token) => expandDependencyToken(token, task.id))
    : [];
  dependencyMap.set(task.id, dependencies);
  for (const dependency of dependencies) {
    if (!ids.has(dependency)) errors.push(`${task.id} depends on missing task ${dependency}`);
    if (dependency === task.id) errors.push(`${task.id} depends on itself`);
  }
  const phase = task.id.match(/^M\d+/)?.[0];
  if (phase && !milestoneTitles.has(phase)) {
    errors.push(`${task.id} has no matching milestone heading in ${milestonePath}`);
  }
}

// Detect dependency cycles (including indirect cycles), which would block an agent forever.
const visiting = new Set();
const visited = new Set();
function visit(id, stack = []) {
  if (visited.has(id)) return;
  if (visiting.has(id)) {
    errors.push(`Dependency cycle detected: ${[...stack, id].join(" -> ")}`);
    return;
  }
  visiting.add(id);
  for (const dependency of dependencyMap.get(id) ?? []) {
    if (ids.has(dependency)) visit(dependency, [...stack, id]);
  }
  visiting.delete(id);
  visited.add(id);
}
for (const task of tasks) visit(task.id);

// Check relative Markdown links outside fenced code blocks.
for (const [sourceFile, original] of fileContents.entries()) {
  const lines = original.split("\n");
  let fenced = false;
  for (let index = 0; index < lines.length; index += 1) {
    const line = lines[index];
    if (/^\s*\x60{3,}|^\s*~{3,}/.test(line)) {
      fenced = !fenced;
      continue;
    }
    if (fenced) continue;
    const linkRegex = /!?\[[^\]]*\]\(([^)]+)\)/g;
    let match;
    while ((match = linkRegex.exec(line)) !== null) {
      const href = match[1].trim().split(/[?#]/, 1)[0].replace(/^<|>$/g, "");
      if (!href || /^(https?:|mailto:|tel:|data:)/i.test(href)) continue;
      const target = href.startsWith("/")
        ? href.slice(1)
        : path.posix.normalize(path.posix.join(path.posix.dirname(sourceFile), href));
      if (!fileContents.has(target) && !fs.existsSync(path.join(root, target))) {
        errors.push(`${sourceFile}:${index + 1} broken local link: ${href}`);
      }
    }
  }
}

if (backlog && !backlog.includes("docs/TASK_ISSUE_SYNC.md")) {
  errors.push("Task backlog does not document its GitHub Issues sync contract.");
}

if (errors.length > 0) {
  console.error("Moonships backlog validation FAILED:");
  for (const error of errors) console.error(`- ${error}`);
  process.exitCode = 1;
} else {
  const completed = tasks.filter((task) => task.checked).length;
  console.log("Moonships backlog validation PASSED");
  console.log(`Markdown files scanned: ${markdownFiles.length}`);
  console.log(`Tasks: ${tasks.length} total; ${completed} checked; ${tasks.length - completed} unchecked`);
  console.log(`Dependency edges: ${[...dependencyMap.values()].reduce((sum, list) => sum + list.length, 0)}`);
  console.log("Task IDs, acceptance criteria, milestone mapping, dependency graph and local Markdown links are valid.");
}
