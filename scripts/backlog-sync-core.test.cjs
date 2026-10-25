'use strict';

const test = require('node:test');
const assert = require('node:assert/strict');
const { preflightIssueSnapshot, planTaskAction, runWrite, START_MARKER, END_MARKER } = require('./backlog-sync-core.cjs');

const task = (id = 'M1-037', checked = false) => ({ id, title: 'Sample task', checked });
const issue = (number, title, body = '', state = 'open', extra = {}) =>
  ({ number, title, body, state, labels: [{ name: 'task' }, { name: 'backlog-sync' }], ...extra });
const marker = id => '<!-- moonships-task-id: ' + id + ' -->';
const managedBody = id => START_MARKER + '\n' + marker(id) + '\n' + END_MARKER;

test('rejects duplicate backlog task IDs before any writer can run', () => {
  assert.throws(() => preflightIssueSnapshot({ tasks: [task(), task()], issues: [] }), /Duplicate backlog task ID/);
});

test('rejects duplicate task markers across issues', () => {
  assert.throws(() => preflightIssueSnapshot({
    tasks: [task()], issues: [issue(1, '[M1-037] one', managedBody('M1-037')), issue(2, '[M1-037] two', managedBody('M1-037'))],
  }), /Multiple GitHub issues use task marker M1-037/);
});

test('rejects managed issue IDs missing from the backlog', () => {
  assert.throws(() => preflightIssueSnapshot({
    tasks: [task()], issues: [issue(1, '[M1-099] orphan', managedBody('M1-099'))],
  }), /absent from docs\/TASK_BACKLOG\.md/);
});

test('rejects unmarked title collisions', () => {
  assert.throws(() => preflightIssueSnapshot({
    tasks: [task()], issues: [issue(1, '[M1-037] unmanaged collision')],
  }), /lacks a sync marker/);
});

test('rejects malformed managed body markers during preflight', () => {
  assert.throws(() => preflightIssueSnapshot({
    tasks: [task()], issues: [issue(1, '[M1-037] malformed', START_MARKER + '\n' + marker('M1-037'))],
  }), /Malformed managed task markers/);
});

test('dry-run write gate never invokes the writer', async () => {
  let calls = 0;
  const result = await runWrite({ dryRun: true, operation: async () => { calls += 1; } });
  assert.deepEqual(result, { performed: false });
  assert.equal(calls, 0);
});

test('stable no-op plan is deterministic and produces no updates', () => {
  const linked = preflightIssueSnapshot({
    tasks: [task()], issues: [issue(12, '[M1-037] Sample task', managedBody('M1-037'))],
  });
  const args = { task: task(), issue: linked.get('M1-037'), body: managedBody('M1-037') };
  assert.deepEqual(planTaskAction(args), { type: 'noop', taskId: 'M1-037', issueNumber: 12 });
  assert.deepEqual(planTaskAction(args), planTaskAction(args));
});

test('plans only required changes and preserves existing labels', () => {
  const result = planTaskAction({
    task: task(), issue: issue(12, '[M1-037] Old title', managedBody('M1-037'), 'closed', { labels: [{ name: 'custom' }] }),
    body: managedBody('M1-037'),
  });
  assert.equal(result.type, 'update');
  assert.equal(result.updates.title, '[M1-037] Sample task');
  assert.equal(result.updates.state, 'open');
  assert.deepEqual(result.updates.labels, ['custom', 'task', 'backlog-sync']);
});
