'use strict';

const START_MARKER = '<!-- moonships-task-sync:start -->';
const END_MARKER = '<!-- moonships-task-sync:end -->';
const TASK_MARKER = /<!--\\s*moonships-task-id:\\s*(M\\d+-\\d{3})\\s*-->/g;
const TASK_TITLE = /^\\[([A-Z]\\d+-\\d{3})\\]/;

function preflightIssueSnapshot({ tasks, issues }) {
  const taskIds = new Set();
  for (const task of tasks) {
    if (taskIds.has(task.id)) throw new Error('Duplicate backlog task ID: ' + task.id);
    taskIds.add(task.id);
  }

  const linkedIssues = new Map();
  const collisions = [];
  for (const issue of issues) {
    if (issue.pull_request) continue;
    const body = issue.body || '';
    const startCount = body.split(START_MARKER).length - 1;
    const endCount = body.split(END_MARKER).length - 1;
    const start = body.indexOf(START_MARKER);
    const end = body.indexOf(END_MARKER);
    if (startCount !== endCount || startCount > 1 || endCount > 1 ||
        (startCount === 1 && (end < start || end < 0)) ||
        (startCount === 0 && endCount !== 0)) {
      throw new Error('Malformed managed task markers on issue #' + issue.number + '; reconcile the body before syncing.');
    }

    const markers = [...body.matchAll(TASK_MARKER)];
    if (markers.length > 1) {
      throw new Error('Multiple task ID markers on issue #' + issue.number + '; reconcile the body before syncing.');
    }
    const marker = markers[0];
    if (marker) {
      const id = marker[1];
      if (linkedIssues.has(id)) {
        throw new Error('Multiple GitHub issues use task marker ' + id + ': #' +
          linkedIssues.get(id).number + ' and #' + issue.number + '. Resolve duplicates before syncing.');
      }
      linkedIssues.set(id, issue);
      continue;
    }
    const titleMatch = (issue.title || '').match(TASK_TITLE);
    if (titleMatch && taskIds.has(titleMatch[1])) {
      collisions.push('#' + issue.number + ' (' + titleMatch[1] + ')');
    }
  }
  if (collisions.length) {
    throw new Error('Issue title collides with a backlog task ID but lacks a sync marker: ' +
      collisions.join(', ') + '. Add the correct marker or rename/reconcile the issue before syncing.');
  }

  const orphans = [...linkedIssues.entries()].filter(([id]) => !taskIds.has(id));
  if (orphans.length) {
    throw new Error('Managed task issue marker(s) are absent from docs/TASK_BACKLOG.md: ' +
      orphans.map(([id, issue]) => id + ' (#' + issue.number + ')').join(', ') +
      '. Restore the original task ID or explicitly reconcile these issues before syncing.');
  }
  return linkedIssues;
}

function planTaskAction({ task, issue, body, milestone, labelSet = ['task', 'backlog-sync'] }) {
  if (task.checked && !issue) return { type: 'skip', taskId: task.id };
  const title = '[' + task.id + '] ' + task.title;
  if (!issue) return { type: 'create', taskId: task.id, title, body, labels: labelSet, milestone };

  const desiredState = task.checked ? 'closed' : 'open';
  const updates = {};
  if (issue.title !== title) updates.title = title;
  if ((issue.body || '') !== body) updates.body = body;
  if (issue.state !== desiredState) {
    updates.state = desiredState;
    if (desiredState === 'closed') updates.state_reason = 'completed';
  }
  const currentLabels = new Set((issue.labels || []).map(label =>
    typeof label === 'string' ? label : label.name));
  const mergedLabels = [...new Set([...currentLabels, ...labelSet])];
  if (mergedLabels.length !== currentLabels.size) updates.labels = mergedLabels;
  if (milestone && (!issue.milestone || issue.milestone.number !== milestone.number)) {
    updates.milestone = milestone.number;
  }
  return Object.keys(updates).length
    ? { type: 'update', taskId: task.id, issueNumber: issue.number, updates }
    : { type: 'noop', taskId: task.id, issueNumber: issue.number };
}

async function runWrite({ dryRun, operation }) {
  if (dryRun) return { performed: false };
  return { performed: true, value: await operation() };
}

module.exports = { preflightIssueSnapshot, planTaskAction, runWrite, START_MARKER, END_MARKER };
