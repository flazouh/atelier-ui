//! The head of the sidebar, the one place its controls live: the switch between the project list and the priority list,
//! the filter of which sessions it lists, and the button that adds a project. They edit one value, the
//! [`SidebarLayout`](crate::sidebar_layout::SidebarLayout), which the [`Sidebar`](crate::sidebar::Sidebar) itself keeps and which also decides what its rows show; the app hands the sidebar its data and hears the choices as
//! events, and draws none of this.

mod impls;
mod structs;
