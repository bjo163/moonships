//! Cross-service Project/Task persistence checks on disposable SQLite databases.

use rusqlite::{params, Connection};

use crate::{labels, project_task_export, projects, run_migrations, task_dependencies, tasks};

const PERSONAL: &str = "00000000-0000-4000-8000-000000000001";

fn disposable_database() -> Connection {
    let mut connection = Connection::open_in_memory().expect("isolated in-memory database");
    connection
        .execute_batch("PRAGMA foreign_keys = ON;")
        .expect("enable fixture foreign keys");
    run_migrations(&mut connection).expect("migrate disposable database");
    connection
}

fn create_project(
    connection: &mut Connection,
    workspace_id: &str,
    name: &str,
) -> projects::Project {
    projects::create_project(
        connection,
        projects::CreateProject {
            workspace_id: workspace_id.to_string(),
            name: name.to_string(),
            project_key: None,
            description: None,
            goal: Some("Integration fixture goal".to_string()),
            status: None,
        },
    )
    .expect("create project")
}

fn create_task(
    connection: &mut Connection,
    workspace_id: &str,
    project_id: Option<String>,
    title: &str,
) -> tasks::Task {
    tasks::create_task(
        connection,
        tasks::CreateTask {
            workspace_id: workspace_id.to_string(),
            project_id,
            parent_task_id: None,
            title: title.to_string(),
            description: None,
            due_at: None,
            status: None,
            priority: None,
        },
    )
    .expect("create task")
}

#[test]
fn project_and_task_services_persist_a_scoped_workflow_in_an_isolated_database() {
    let mut connection = disposable_database();
    let database_file: String = connection
        .query_row("PRAGMA database_list", [], |row| row.get(2))
        .expect("database path");
    assert!(
        database_file.is_empty(),
        "fixture must not open a user database"
    );

    let project = create_project(&mut connection, PERSONAL, "Integration project");
    let task = create_task(
        &mut connection,
        PERSONAL,
        Some(project.id.clone()),
        "Integration task",
    );
    let updated = tasks::update_task(
        &mut connection,
        tasks::UpdateTask {
            workspace_id: PERSONAL.to_string(),
            id: task.id.clone(),
            project_id: Some(project.id.clone()),
            title: task.title.clone(),
            description: Some("Persisted through the task service".to_string()),
            due_at: Some("2026-10-27T12:00:00+07:00".to_string()),
            status: "in_progress".to_string(),
            priority: "p1".to_string(),
        },
    )
    .expect("update task");

    assert_eq!(updated.project_id.as_deref(), Some(project.id.as_str()));
    assert_eq!(updated.status, "in_progress");
    assert_eq!(updated.priority, "p1");
    assert_eq!(updated.due_at.as_deref(), Some("2026-10-27T05:00:00Z"));
    let listed = tasks::list_tasks(&connection, PERSONAL, tasks::TaskArchiveFilter::Active)
        .expect("list active tasks");
    assert_eq!(listed, vec![updated]);

    let (projects_count, linked_tasks, activity_count): (i64, i64, i64) = connection
        .query_row(
            "SELECT (SELECT COUNT(*) FROM projects WHERE workspace_id = ?1),
                    (SELECT COUNT(*) FROM tasks WHERE workspace_id = ?1 AND project_id = ?2),
                    (SELECT COUNT(*) FROM activity_events WHERE workspace_id = ?1)",
            params![PERSONAL, project.id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .expect("persisted repository rows");
    assert_eq!(projects_count, 1);
    assert_eq!(linked_tasks, 1);
    assert_eq!(activity_count, 5);
}

#[test]
fn project_task_export_is_versioned_scoped_and_excludes_restricted_and_config_data() {
    let mut connection = disposable_database();
    connection
        .execute(
            "INSERT INTO workspaces (id, name) VALUES ('other-workspace', 'Other')",
            [],
        )
        .expect("other workspace");
    let project = create_project(&mut connection, PERSONAL, "Export project");
    let task = create_task(
        &mut connection,
        PERSONAL,
        Some(project.id.clone()),
        "Export task",
    );
    let other_project = create_project(&mut connection, "other-workspace", "Out of scope");
    let _other_task = create_task(
        &mut connection,
        "other-workspace",
        Some(other_project.id),
        "Out of scope task",
    );
    connection
        .execute(
            "UPDATE tasks SET archived_at = '2026-10-26T00:00:00Z' WHERE id = ?1",
            params![task.id],
        )
        .expect("archive fixture task");
    connection
        .execute(
            "UPDATE projects SET status = 'archived', archived_at = '2026-10-26T00:00:00Z',
                 color_token = 'signal-cyan', start_date = '2026-10-01', target_date = '2026-10-31'
             WHERE id = ?1",
            params![project.id],
        )
        .expect("archive fixture project");
    connection
        .execute(
            "UPDATE tasks SET estimate_minutes = 30, completed_at = '2026-10-25T00:00:00Z' WHERE id = ?1",
            params![task.id],
        )
        .expect("add fixture task fields");
    connection
        .execute(
            "INSERT INTO activity_events (
                id, workspace_id, actor_type, entity_type, entity_id, project_id, task_id,
                action, occurred_at, summary, sensitivity
             ) VALUES (
                'restricted-export-event', ?1, 'local', 'task', ?2, NULL, ?2,
                'updated', '2026-10-26T00:00:00Z', 'restricted export sentinel', 'restricted'
             )",
            params![PERSONAL, task.id],
        )
        .expect("insert restricted activity");
    connection
        .execute(
            "INSERT INTO app_metadata (key, value) VALUES ('test.secret', 'credential sentinel')",
            [],
        )
        .expect("insert unrelated secret metadata");

    let export = project_task_export::load_personal_export(&connection, "0.1.0-2")
        .expect("load Personal export");
    let json = export
        .to_json()
        .expect("serialize and validate export schema");
    let parsed: project_task_export::ProjectTaskExport =
        serde_json::from_str(&json).expect("strict export deserialization");

    assert_eq!(
        parsed.schema_version,
        project_task_export::EXPORT_SCHEMA_VERSION
    );
    assert_eq!(parsed.projects.len(), 1);
    assert_eq!(parsed.projects[0].id, project.id);
    assert_eq!(parsed.projects[0].status, "archived");
    assert_eq!(
        parsed.projects[0].color_token.as_deref(),
        Some("signal-cyan")
    );
    assert_eq!(
        parsed.projects[0].target_date.as_deref(),
        Some("2026-10-31")
    );
    assert_eq!(parsed.tasks.len(), 1);
    assert_eq!(parsed.tasks[0].id, task.id);
    assert!(parsed.tasks[0].archived_at.is_some());
    assert_eq!(parsed.tasks[0].estimate_minutes, Some(30));
    assert!(parsed.tasks[0].completed_at.is_some());
    assert!(!parsed.activity_events.is_empty());
    assert!(!json.contains("Out of scope"));
    assert!(!json.contains("restricted export sentinel"));
    assert!(!json.contains("credential sentinel"));
}

#[test]
fn labels_are_workspace_unique_and_task_relationships_are_scoped_and_idempotent() {
    let mut connection = disposable_database();
    connection
        .execute(
            "INSERT INTO workspaces (id, name) VALUES ('other-workspace', 'Other')",
            [],
        )
        .expect("other workspace");
    let task = create_task(&mut connection, PERSONAL, None, "Label me");
    let other_task = create_task(&mut connection, "other-workspace", None, "Other task");

    let label =
        labels::create_label(&mut connection, PERSONAL, "  Needs   Review ").expect("create label");
    assert_eq!(label.name, "Needs Review");
    assert_eq!(label.normalized_name, "needs review");
    assert_eq!(
        labels::create_label(&mut connection, PERSONAL, "needs review")
            .expect_err("case/whitespace duplicate is rejected"),
        "CONFLICT: label name already exists in this workspace"
    );
    let unicode_label =
        labels::create_label(&mut connection, PERSONAL, "CAFÉ").expect("create Unicode label");
    assert_eq!(unicode_label.normalized_name, "café");
    assert!(labels::create_label(&mut connection, PERSONAL, "café")
        .expect_err("Unicode lowercase duplicate is rejected")
        .starts_with("CONFLICT:"));
    let same_name_other_workspace =
        labels::create_label(&mut connection, "other-workspace", "needs review")
            .expect("same normalized name is allowed in another workspace");
    assert_eq!(labels::list_labels(&connection, PERSONAL).unwrap().len(), 2);
    assert_eq!(
        labels::list_labels(&connection, "other-workspace")
            .unwrap()
            .len(),
        1
    );

    labels::add_label_to_task(&connection, PERSONAL, &task.id, &label.id)
        .expect("link label and task");
    labels::add_label_to_task(&connection, PERSONAL, &task.id, &label.id)
        .expect("repeated link is idempotent");
    assert_eq!(
        labels::list_task_labels(&connection, PERSONAL, &task.id).expect("read linked labels"),
        vec![label.clone()]
    );
    assert!(labels::add_label_to_task(&connection, PERSONAL, &other_task.id, &label.id).is_err());
    assert!(labels::add_label_to_task(
        &connection,
        PERSONAL,
        &task.id,
        &same_name_other_workspace.id
    )
    .is_err());

    labels::remove_label_from_task(&connection, PERSONAL, &task.id, &label.id)
        .expect("remove link");
    labels::remove_label_from_task(&connection, PERSONAL, &task.id, &label.id)
        .expect("repeated remove is idempotent");
    assert!(labels::list_task_labels(&connection, PERSONAL, &task.id)
        .expect("read after remove")
        .is_empty());
}

#[test]
fn task_dependencies_reject_self_and_cross_workspace_links() {
    let mut connection = disposable_database();
    connection
        .execute(
            "INSERT INTO workspaces (id, name) VALUES ('other-workspace', 'Other')",
            [],
        )
        .expect("other workspace");
    let task = create_task(&mut connection, PERSONAL, None, "Blocked task");
    let prerequisite = create_task(&mut connection, PERSONAL, None, "Prerequisite task");
    let other_task = create_task(&mut connection, "other-workspace", None, "Other task");

    assert_eq!(
        task_dependencies::add_dependency(&connection, PERSONAL, &task.id, &task.id)
            .expect_err("self dependency is rejected"),
        "VALIDATION_FAILED: a Task cannot depend on itself"
    );
    assert!(
        task_dependencies::add_dependency(&connection, PERSONAL, &task.id, &other_task.id).is_err()
    );

    let dependency =
        task_dependencies::add_dependency(&connection, PERSONAL, &task.id, &prerequisite.id)
            .expect("create directed dependency");
    assert_eq!(dependency.task_id, task.id);
    assert_eq!(dependency.depends_on_task_id, prerequisite.id);
    assert_eq!(dependency.dependency_type, "blocked_by");
    assert_eq!(
        task_dependencies::add_dependency(&connection, PERSONAL, &task.id, &prerequisite.id)
            .expect("repeated dependency is idempotent"),
        dependency
    );
    assert_eq!(
        task_dependencies::list_dependencies_for_task(&connection, PERSONAL, &task.id)
            .expect("list Task dependencies"),
        vec![dependency]
    );

    assert!(connection
        .execute(
            "INSERT INTO task_dependencies (workspace_id, task_id, depends_on_task_id)
             VALUES (?1, ?2, ?2)",
            params![PERSONAL, task.id],
        )
        .is_err());
    task_dependencies::remove_dependency(&connection, PERSONAL, &task.id, &prerequisite.id)
        .expect("remove dependency");
    task_dependencies::remove_dependency(&connection, PERSONAL, &task.id, &prerequisite.id)
        .expect("repeated removal is idempotent");
    assert!(
        task_dependencies::list_dependencies_for_task(&connection, PERSONAL, &task.id)
            .expect("list after removal")
            .is_empty()
    );
}

#[test]
fn project_task_services_reject_cross_workspace_links_without_partial_writes() {
    let mut connection = disposable_database();
    connection
        .execute(
            "INSERT INTO workspaces (id, name) VALUES ('other-workspace', 'Other')",
            [],
        )
        .expect("create second workspace");
    let foreign_project = create_project(&mut connection, "other-workspace", "Foreign project");

    let error = tasks::create_task(
        &mut connection,
        tasks::CreateTask {
            workspace_id: PERSONAL.to_string(),
            project_id: Some(foreign_project.id.clone()),
            parent_task_id: None,
            title: "Must not be linked cross-workspace".to_string(),
            description: None,
            due_at: None,
            status: None,
            priority: None,
        },
    )
    .expect_err("cross-workspace project must be rejected");
    assert!(error.starts_with("NOT_FOUND:"));

    let task = create_task(&mut connection, PERSONAL, None, "Inbox task");
    let error = tasks::update_task(
        &mut connection,
        tasks::UpdateTask {
            workspace_id: PERSONAL.to_string(),
            id: task.id.clone(),
            project_id: Some(foreign_project.id),
            title: task.title.clone(),
            description: None,
            due_at: None,
            status: task.status.clone(),
            priority: task.priority.clone(),
        },
    )
    .expect_err("cross-workspace project update must be rejected");
    assert!(error.starts_with("NOT_FOUND:"));

    let (count, project_id): (i64, Option<String>) = connection
        .query_row(
            "SELECT COUNT(*), project_id FROM tasks WHERE workspace_id = ?1",
            params![PERSONAL],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("verify unchanged inbox task");
    assert_eq!(count, 1);
    assert_eq!(project_id, None);
}

#[test]
fn archiving_a_project_keeps_its_task_consistent_across_task_operations() {
    let mut connection = disposable_database();
    let project = create_project(&mut connection, PERSONAL, "Archived parent");
    let task = create_task(
        &mut connection,
        PERSONAL,
        Some(project.id.clone()),
        "Linked task",
    );
    projects::archive_project(&mut connection, PERSONAL, &project.id).expect("archive project");

    let active = tasks::list_tasks(&connection, PERSONAL, tasks::TaskArchiveFilter::Active)
        .expect("list task retained by project archive");
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].id, task.id);
    assert!(tasks::update_task(
        &mut connection,
        tasks::UpdateTask {
            workspace_id: PERSONAL.to_string(),
            id: task.id.clone(),
            project_id: Some(project.id.clone()),
            title: task.title.clone(),
            description: None,
            due_at: None,
            status: task.status.clone(),
            priority: task.priority.clone(),
        },
    )
    .unwrap_err()
    .starts_with("NOT_FOUND:"));

    tasks::archive_task(&mut connection, PERSONAL, &task.id).expect("archive task");
    assert!(tasks::restore_task(&mut connection, PERSONAL, &task.id)
        .unwrap_err()
        .starts_with("CONFLICT:"));
    let archived = tasks::list_tasks(&connection, PERSONAL, tasks::TaskArchiveFilter::Archived)
        .expect("task remains archived");
    assert_eq!(archived.len(), 1);
    assert!(archived[0].archived_at.is_some());
}

#[test]
fn project_to_task_status_flow_persists_after_database_reopen() {
    let app_data = std::env::temp_dir().join(format!(
        "moonships-project-task-smoke-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time")
            .as_nanos()
    ));

    let (created_project, updated_task) = {
        let mut connection = crate::open_database_in(&app_data).expect("open isolated database");
        run_migrations(&mut connection).expect("migrate isolated database");
        let project = create_project(&mut connection, PERSONAL, "Restart acceptance project");
        let task = create_task(
            &mut connection,
            PERSONAL,
            Some(project.id.clone()),
            "Complete persisted workflow",
        );
        let updated = tasks::update_task(
            &mut connection,
            tasks::UpdateTask {
                workspace_id: PERSONAL.to_string(),
                id: task.id,
                project_id: Some(project.id.clone()),
                title: task.title,
                description: Some("Updated before reopening the database".to_string()),
                due_at: None,
                status: "in_progress".to_string(),
                priority: "p1".to_string(),
            },
        )
        .expect("update task status in project workflow");
        (project, updated)
    };

    let mut reopened = crate::open_database_in(&app_data).expect("reopen isolated database");
    run_migrations(&mut reopened).expect("migrations remain idempotent after reopen");
    let listed_projects = projects::list_projects(&reopened, PERSONAL).expect("read projects");
    assert_eq!(listed_projects, vec![created_project.clone()]);
    let listed_tasks = tasks::list_tasks(&reopened, PERSONAL, tasks::TaskArchiveFilter::Active)
        .expect("read tasks after reopen");
    assert_eq!(listed_tasks, vec![updated_task.clone()]);
    let read_task = tasks::get_task(&reopened, PERSONAL, &updated_task.id).expect("read task");
    assert_eq!(
        read_task.project_id.as_deref(),
        Some(created_project.id.as_str())
    );
    assert_eq!(read_task.status, "in_progress");
    assert_eq!(read_task.priority, "p1");
    assert_eq!(
        read_task.description.as_deref(),
        Some("Updated before reopening the database")
    );
    drop(reopened);
    std::fs::remove_dir_all(app_data).expect("remove disposable workflow database");
}
