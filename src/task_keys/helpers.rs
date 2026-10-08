use super::types::TaskCommand;
use crate::keys::Press;

/// The command a press asks for, if any. `typing` is whether a text input has focus: then only a press
/// with a modifier, and Escape, can be a command.
pub fn read(press: &Press, typing: bool) -> Option<TaskCommand> {
    use TaskCommand::*;
    if press.secondary {
        return (press.key == "a" && !press.shift && !press.alt).then_some(SelectAll);
    }
    if press.alt {
        return None;
    }
    match press.key.as_str() {
        "escape" | "Escape" => return Some(Clear),
        "down" => return Some(Down),
        "up" => return Some(Up),
        "left" => return Some(Fold),
        "right" => return Some(Unfold),
        "enter" | "Enter" if !typing => return Some(Open),
        "home" => return Some(First),
        "end" => return Some(Last),
        _ => {}
    }
    if typing {
        return None;
    }
    Some(match (press.key.as_str(), press.shift) {
        ("j", false) => Down,
        ("k", false) => Up,
        ("g", false) => First,
        ("g", true) | ("G", _) => Last,
        ("x", false) => Select,
        ("s", false) => SetStatus,
        ("p", false) => SetPriority,
        ("a", false) => SetAssignee,
        ("i", false) => AssignToMe,
        ("l", false) => SetLabels,
        ("c", false) => NewTask,
        ("f", false) => Filter,
        ("]", _) => MoveNext,
        ("[", _) => MovePrev,
        _ => return None,
    })
}
