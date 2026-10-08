use super::{Draft, Submit, create, next_key};
use crate::{
    agent_look::AgentLook,
    task_model::{Activity, Assignee, Label, Priority, TaskData, TaskStatus},
    theme::Theme,
};

fn agent() -> Assignee {
    Assignee::agent("Claude", AgentLook::neutral(&Theme::dark()))
}

#[test]
fn a_draft_needs_a_title_with_something_in_it() {
    let mut draft = Draft::default();
    assert!(!draft.can_create());
    draft.title = "   \n ".into();
    assert!(!draft.can_create());
    draft.title = " Fix it ".into();
    assert!(draft.can_create());
}

#[test]
fn the_button_offers_a_session_only_when_the_assignee_is_an_agent() {
    let mut draft = Draft::default();
    assert_eq!(
        (draft.submit(), draft.submit_words()),
        (Submit::Create, "Create task")
    );
    draft.assignee = Some(Assignee::Person { name: "Ada".into() });
    assert_eq!(draft.submit(), Submit::Create);
    draft.assignee = Some(agent());
    assert_eq!(
        (draft.submit(), draft.submit_words()),
        (Submit::CreateAndStart, "Create and start a session")
    );
}

#[test]
fn the_next_key_follows_the_highest_number_of_the_prefix() {
    let mut list = vec![
        TaskData::new("1", "LAT-7", "x", TaskStatus::Todo),
        TaskData::new("2", "LAT-42", "y", TaskStatus::Todo),
        TaskData::new("3", "API-99", "z", TaskStatus::Todo),
    ];
    assert_eq!(next_key("LAT", &list).as_ref(), "LAT-43");
    assert_eq!(next_key("API", &list).as_ref(), "API-100");
    assert_eq!(next_key("NEW", &list).as_ref(), "NEW-1");
    list.push(TaskData::new("4", "LATE-500", "w", TaskStatus::Todo));
    list.push(TaskData::new("5", "LAT-x", "v", TaskStatus::Todo));
    assert_eq!(
        next_key("LAT", &list).as_ref(),
        "LAT-43",
        "another prefix or a key that is not a number does not count"
    );
    assert_eq!(next_key("LAT", &[]).as_ref(), "LAT-1");
}

#[test]
fn creating_makes_a_task_with_the_drafts_fields_and_a_note_of_its_creation() {
    let draft = Draft {
        title: "  Add the board  ".into(),
        description: "\nWith **drag**.\n".into(),
        status: TaskStatus::Backlog,
        priority: Priority::High,
        assignee: Some(agent()),
        labels: vec![Label::new("ui", 2)],
        project: Some("atelier".into()),
        parent: Some("parent".into()),
    };
    let task = create(&draft, "t9", "LAT-9".into(), "Ada", 1234).expect("a task");
    assert_eq!(
        (task.id.as_ref(), task.key.as_ref(), task.title.as_ref()),
        ("t9", "LAT-9", "Add the board")
    );
    assert_eq!(task.description.as_ref(), "With **drag**.");
    assert_eq!(
        (task.status, task.priority),
        (TaskStatus::Backlog, Priority::High)
    );
    assert_eq!((task.created_at, task.updated_at), (1234, 1234));
    assert!(task.assignee.as_ref().is_some_and(Assignee::is_agent));
    assert_eq!(
        (
            task.labels.len(),
            task.project.as_deref(),
            task.parent.as_deref()
        ),
        (1, Some("atelier"), Some("parent"))
    );
    assert_eq!(
        task.activity,
        vec![Activity::Created {
            by: "Ada".into(),
            at: 1234
        }]
    );
}

#[test]
fn a_draft_without_a_title_makes_nothing() {
    assert!(create(&Draft::default(), "t", "LAT-1".into(), "Ada", 1).is_none());
}
