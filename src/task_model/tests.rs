use super::{Activity, Assignee, Priority, TaskData, TaskStatus, sub_tasks};
use crate::{agent_look::AgentLook, theme::Theme};

#[test]
fn a_list_orders_its_groups_from_the_work_closest_to_done_and_a_board_from_the_start() {
    let words = |order: [TaskStatus; 6]| order.map(TaskStatus::words);
    assert_eq!(
        words(TaskStatus::LIST_ORDER),
        [
            "In Progress",
            "In Review",
            "Todo",
            "Backlog",
            "Done",
            "Canceled"
        ]
    );
    assert_eq!(
        words(TaskStatus::BOARD_ORDER),
        [
            "Backlog",
            "Todo",
            "In Progress",
            "In Review",
            "Done",
            "Canceled"
        ]
    );
    for order in [TaskStatus::LIST_ORDER, TaskStatus::BOARD_ORDER] {
        let mut sorted = order.to_vec();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), 6, "every status once");
    }
}

#[test]
fn only_done_and_canceled_are_closed() {
    let open: Vec<_> = TaskStatus::BOARD_ORDER
        .iter()
        .filter(|s| s.is_open())
        .map(|s| s.words())
        .collect();
    assert_eq!(open, ["Backlog", "Todo", "In Progress", "In Review"]);
}

#[test]
fn a_status_says_how_much_is_done_so_its_mark_can_fill() {
    let progress: Vec<_> = TaskStatus::BOARD_ORDER
        .iter()
        .map(|s| s.progress())
        .collect();
    assert_eq!(progress, [0., 0., 0.5, 0.75, 1., 0.]);
}

#[test]
fn priorities_sort_urgent_first_and_none_last_and_show_bars() {
    let mut all = Priority::ALL;
    all.reverse();
    all.sort();
    assert_eq!(all, Priority::ALL);
    assert_eq!(Priority::ALL.map(Priority::bars), [0, 3, 2, 1, 0]);
    assert_eq!(Priority::None.words(), "No priority");
}

#[test]
fn an_assignee_is_named_by_its_name_and_an_agent_is_told_from_a_person() {
    let person = Assignee::Person {
        name: "ada lovelace".into(),
    };
    let agent = Assignee::agent("Claude", AgentLook::neutral(&Theme::dark()));
    assert_eq!(
        (
            person.initial().as_str(),
            person.name().as_ref(),
            person.is_agent()
        ),
        ("A", "ada lovelace", false)
    );
    assert!(agent.is_agent());
    assert_eq!(Assignee::Person { name: "".into() }.initial(), "");
    assert_eq!(
        Assignee::Person {
            name: "élan".into()
        }
        .initial(),
        "É"
    );
}

#[test]
fn every_kind_of_activity_reads_as_one_line() {
    let lines = [
        (
            Activity::Comment {
                author: "Ada".into(),
                text: "x".into(),
                at: 1,
            },
            "Ada commented",
        ),
        (
            Activity::StatusChanged {
                by: "Ada".into(),
                from: TaskStatus::Todo,
                to: TaskStatus::InProgress,
                at: 2,
            },
            "Ada changed the status from Todo to In Progress",
        ),
        (
            Activity::SessionStarted {
                agent: "Claude".into(),
                at: 3,
            },
            "Claude started a session",
        ),
        (
            Activity::PrOpened {
                number: 3344,
                at: 4,
            },
            "PR #3344 opened",
        ),
        (
            Activity::PrMerged {
                number: 3344,
                at: 5,
            },
            "PR #3344 merged",
        ),
        (
            Activity::Created {
                by: "Ada".into(),
                at: 6,
            },
            "Ada created the task",
        ),
        (
            Activity::Committed {
                by: "Ada".into(),
                sha: "abc1234".into(),
                subject: "Fix the scroll".into(),
                at: 7,
            },
            "abc1234 Fix the scroll",
        ),
    ];
    for (i, (activity, words)) in lines.iter().enumerate() {
        assert_eq!(activity.words(), *words);
        assert_eq!(activity.at(), i as u64 + 1);
    }
}

#[test]
fn sub_tasks_are_found_by_parent_and_counted_done() {
    let mut parent = TaskData::new("p", "LAT-1", "Parent", TaskStatus::InProgress);
    parent.updated_at = 1;
    let mut done = TaskData::new("a", "LAT-2", "A", TaskStatus::Done);
    done.parent = Some("p".into());
    let mut open = TaskData::new("b", "LAT-3", "B", TaskStatus::Todo);
    open.parent = Some("p".into());
    let stranger = TaskData::new("c", "LAT-4", "C", TaskStatus::Done);
    let tasks = vec![parent.clone(), done, open, stranger];
    let (subs, finished) = sub_tasks(&tasks, &parent);
    assert_eq!((subs.len(), finished), (2, 1));
    assert_eq!(sub_tasks(&tasks, &tasks[3]).0.len(), 0);
}

#[test]
fn the_neighbor_of_a_status_follows_the_flow_of_the_board() {
    assert_eq!(TaskStatus::Backlog.neighbor(false), None);
    assert_eq!(TaskStatus::Backlog.neighbor(true), Some(TaskStatus::Todo));
    assert_eq!(TaskStatus::InReview.neighbor(true), Some(TaskStatus::Done));
    assert_eq!(TaskStatus::Done.neighbor(false), Some(TaskStatus::InReview));
    assert_eq!(TaskStatus::Canceled.neighbor(true), None);
}
