//! atelier-ui: a design system for GPUI apps, Atelier's look. Tokens and motion follow beui.dev; the primary is amber
//! `#F9A825`. Components take plain data and know nothing about any agent: an agent's look reaches
//! them as an [`AgentLook`].

pub mod accessibility;
pub mod file_upload;
pub mod focus;
pub mod folder_picker;
pub mod agent_look;
pub mod action_swap;
pub mod animated_badge;
pub mod agent_text;
pub mod badge;
pub mod bloom_menu;
pub mod breadcrumb;
pub mod button;
pub mod button_group;
pub mod changed_file_tree;
pub mod changed_files;
pub mod checks_panel;
pub mod code_block;
pub mod checkbox;
pub mod code_editor;
pub mod atelier_mark;
pub mod layout_motion;
pub mod multi_select;
pub mod notification_stack;
pub mod toast_stack;
pub mod range_slider;
pub mod trace;
pub mod color_selector;
pub mod combobox;
pub mod command_item;
pub mod conversation;
pub mod comment_composer;
pub mod commits_summary;
pub mod court;
pub(crate) mod copy_feedback;
pub mod disclosure;
pub mod entrance;
pub mod file_diff;
pub mod file_icon;
pub mod finder;
pub mod fuzzy;
pub mod file_tree;
pub(crate) mod glimmer;
pub mod icon;
pub mod inline_review;
pub mod island;
pub mod design_preview; // design preview: remove after Alex picks
pub mod kbd;
pub mod keys;
pub mod line_comment;
pub mod markdown_edit;
pub mod merge;
pub mod merge_box;
pub mod merge_button;
pub mod message_bubble;
pub mod message_rail;
pub mod model_badge;
pub mod morph;
pub mod motion;
pub mod pane;
pub mod pr;
pub mod placement;
pub mod popover;
pub mod pr_card;
pub mod pr_chip;
pub mod pr_refs;
pub mod prompt_input;
pub mod rail_section;
pub mod scale;
pub(crate) mod reveal;
pub mod hover_tone;
pub mod review;
pub mod review_bar;
pub mod review_file_header;
pub mod row_map;
pub mod session_status;
pub mod sidebar_filter;
pub mod sidebar_layout;
pub mod sidebar_model;
pub mod panel_layout;
pub mod tab_order;
pub mod session_row;
pub mod icon_candidates;
pub mod soft_breaks;
pub mod stream_text;
pub mod icon_picker;
pub mod project_badge;
pub mod project_section;
pub mod sidebar;
pub mod panel_types;
pub mod agent_panels;
pub mod panel_strip;
pub mod panel_tabs;
pub mod task_model;
pub mod task_list_model;
pub mod task_keys;
pub mod task_edit;
pub mod task_board_model;
pub mod new_task_model;
pub mod task_marks;
pub mod task_row;
pub mod task_picker;
pub mod task_list;
pub mod task_view;
pub mod new_task;
pub mod task_card;
pub mod task_board;
pub mod menu;
pub mod number;
pub mod modal;
pub mod segmented;
pub mod roll;
pub mod select;
pub mod sprite;
pub mod spinner;
pub mod status_mark;
pub mod subagent_card;
pub mod syntax;
pub mod subagent_row;
pub mod subagent_strip;
pub mod theme;
pub mod theme_file;
pub mod theme_import;
pub mod theme_picker;
pub mod themes;
pub mod thinking;
pub mod todo_list;
pub mod switch;
pub mod tabs;
pub mod text_input;
pub mod tooltip;
pub mod tool_approval;
pub mod tool_preview;
pub mod tool_call;
pub mod typography;
pub mod verdict;
pub mod voice_input;
pub mod voice_setup;
pub mod voice_waves;
pub mod unsent;
pub(crate) mod scroll_chain;
pub(crate) mod wake;

pub use voice_input::{VoiceInput, VoiceInputEvent, VoiceMode};
pub use voice_setup::{SetupPhase, VoiceSetup};
pub use voice_waves::VoiceWaves;
pub use agent_look::{AgentLook, Mark, PhaseLabels};
pub use agent_text::{AgentText, AgentTextSource, AgentTextStatus};
pub use action_swap::{ActionSwapButton, SwapSize, SwapVariant};
pub use animated_badge::{AnimatedBadge, BadgeSize, BadgeStatus};
pub use badge::{Badge, Tone};
pub use breadcrumb::{Breadcrumb, Crumb};
pub use button::{Button, ButtonSize, ButtonVariant, dot};
pub use changed_file_tree::ChangedFileTree;
pub use changed_files::{ChangedFile, ChangedFiles, FileChange};
pub use code_block::{CodeBlock, CodeBlockStatus};
pub use code_editor::{CodeEditor, language_for};
pub use disclosure::Disclosure;
pub use entrance::{Entrance, EntranceList};
pub use file_diff::{DiffLine, DiffLineKind, FileDiff, FileDiffStatus};
pub use file_icon::FileIcon;
pub use finder::{Filter, Finder, FinderEvent, FinderItem};
pub use row_map::RowMap;
pub use icon::{Assets, Icon, IconName};
pub use inline_review::{Decision, InlineHunk, InlineReview, Resolve};
pub use island::{Island, IslandCounts, SessionsIsland, counts_of, most_urgent};
pub use kbd::Kbd;
pub use line_comment::{Comment, LineComment, LineComposer, LineComposerEvent};
pub use message_bubble::{
    MessageBubble, MessageBubbleAlign, MessageBubbleCollapsible, MessageBubbleGroupSpacing, MessageBubbleVariant,
    message_bubble_group,
};
pub use model_badge::{BrandMark, ModelBadge};
pub use pane::{pane_header_height, drag_space, pane_header};
pub use pr::{Checks, ChecksSummary, PrChipData, PrState, ReviewState};
pub use merge_box::{MergeBox, MergeBoxEvent};
pub use button_group::ButtonGroup;
pub use checkbox::Checkbox;
pub use file_upload::{FileUpload, FileUploadEvent, UploadItem, UploadStatus, UploadVariant};
pub use bloom_menu::{BloomEvent, BloomItem, BloomMenu};
pub use focus::PressStop;
pub use focus::Field;
pub use folder_picker::{FolderError, FolderPicker, FolderPickerEvent};
pub use atelier_mark::AtelierMark;
pub use multi_select::{MultiOption, MultiSelect, MultiSelectEvent};
pub use notification_stack::{NotificationEvent, NotificationItem, NotificationStack, Trailing, TrailingTone};
pub use toast_stack::{Toast, ToastEvent, ToastPatch, ToastPosition, ToastStack, ToastStatus};
pub use range_slider::RangeSlider;
pub use combobox::{ComboEntry, ComboList, ComboRow, ComboStyle};
pub use color_selector::{ColorSelector, Swatch};
pub use merge_button::MergeButton;
pub use pr_card::PrCard;
pub use pr_chip::PrChip;
pub use pr_refs::pr_refs;
pub use prompt_input::{PromptAction, PromptInput, PromptInputEvent, PromptModel};
pub use checks_panel::{CheckRun, CheckState, ChecksPanel, JobStep};
pub use comment_composer::{CommentComposer, CommentComposerEvent};
pub use commits_summary::{CommitData, CommitsSummary};
pub use conversation::{ConversationList, RemarkSummary, ThreadSummary};
pub use court::{Court, CourtItem, CourtList};
pub use review::{ReviewHandlers, ReviewProgress};
pub use unsent::UnsentComments;
pub use new_task::{NewTask, NewTaskEvent};
pub use task_board::{TaskBoard, TaskBoardEvent};
pub use task_list::{TaskList, TaskListEvent};
pub use task_marks::{PriorityMark, TaskStatusMark};
pub use task_model::{Activity, Assignee, Label, Priority, SessionLink, TaskData, TaskStatus};
pub use task_row::TaskRow;
pub use task_view::{TaskView, TaskViewEvent};
pub use verdict::{Decision as VerdictDecision, VerdictBox, VerdictEvent, Verb};
pub use review_bar::ReviewBar;
pub use review_file_header::ReviewFileHeader;
pub use menu::{Menu, MenuItem};
pub use number::Digits;
pub use modal::Modal;
pub use segmented::{Segment, Segmented};
pub use select::{Select, SelectOption};
pub use session_status::{Need, SessionStatus};
pub use project_section::ProjectSection;
pub use session_row::SessionRow;
pub use sidebar::{Sidebar, SidebarEvent};
pub use agent_panels::AgentPanels;
pub use panel_types::{Layout as PanelLayout, PanelContent, PanelData, PanelsEvent, PanelsState, ProjectLabel};
pub use sidebar_model::{Connection, Location, ProjectData, SessionData};
pub use sprite::{Sprite, Strip};
pub use subagent_card::SubagentCard;
pub use subagent_row::SubagentRow;
pub use subagent_strip::{STACK_GAP, SubagentStrip};
pub use spinner::Spinner;
pub use theme::{ActiveTheme, Appearance, StatusTone, Theme};
pub use morph::Morph;
pub use thinking::{Shimmer, Thinking, ThinkingPhase, ThinkingStyle};
pub use switch::Switch;
pub use tabs::{Tab, Tabs, TabsVariant};
pub use text_input::TextInput;
pub use tooltip::Tooltip;
pub use todo_list::{Todo, TodoList, TodoStatus};
pub use tool_approval::{ParamValue, ToolApproval, ToolApprovalStatus};
pub use tool_preview::{TextEdit, ToolPreview};
pub use tool_call::{ToolCall, ToolStatus};
pub use typography::{FONT_FAMILY, MONO_FONT_FAMILY, SEGMENT_GAP, TextSize};

use std::rc::Rc;

use gpui_kit::{App, ClickEvent, Context, Subscription, Window};

/// A click callback that components store and share across frames.
pub(crate) type ClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// Sets up fonts, text rendering, and the theme macOS uses now. Call once, before opening a window.
/// Pass [`Assets`] to the application so icons load.
pub fn init(cx: &mut App) {
    // Must run before the first glyph is drawn.
    typography::disable_font_smoothing();
    gpui_kit::init(cx);
    code_editor::bind_keys(cx);
    inline_review::bind_keys(cx);
    agent_panels::bind_keys(cx);
    review::bind_keys(cx);
    select::bind_keys(cx);
    new_task::bind_keys(cx);
    changed_file_tree::bind_keys(cx);
    // After the inline review's keys, so the composer's own win inside it.
    line_comment::bind_keys(cx);
    comment_composer::bind_keys(cx);
    accessibility::sync_reduce_motion(cx);
    typography::load_fonts(cx);
    theme::follow_system(cx);
}

/// Keeps Reduce Motion and light or dark in step with macOS while the app runs. Call from the root view
/// of each window and keep the subscriptions for the window's life.
pub fn watch_system<T: 'static>(window: &mut Window, cx: &mut Context<T>) -> [Subscription; 2] {
    [
        // macOS has no Reduce Motion event, so read it again whenever the user comes back to the window.
        cx.observe_window_activation(window, |_, window, cx| {
            if window.is_window_active() {
                accessibility::sync_reduce_motion(cx);
            }
        }),
        // GPUI also calls this when the window opens; act only on a real change, so a theme the user
        // picked in the app is not reset.
        {
            let mut last = theme::Appearance::of_system(window.appearance());
            cx.observe_window_appearance(window, move |_, window, cx| {
                let now = theme::Appearance::of_system(window.appearance());
                if now != last {
                    last = now;
                    theme::set_appearance(now, cx);
                }
            })
        },
    ]
}
