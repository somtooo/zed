use std::ops::Range;
use std::rc::Rc;

use editor::{Anchor, Editor, SelectionCommentCallback};
use feature_flags::{FeatureFlagAppExt as _, SelectionCommentFeatureFlag};
use gpui::{App, Context, Entity, WeakEntity, Window};
use settings::Settings as _;
use workspace::Workspace;
use zed_actions::agent::AddSelectionCommentToThread;

use crate::agent_panel::AgentPanel;
use crate::completion_provider::AgentContextSelection;
use crate::conversation_view::ConversationView;

pub fn selection_comment(
    workspace: &mut Workspace,
    _: &AddSelectionCommentToThread,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    log::debug!("AddSelectionCommentToThread invoked");
    if !cx.has_flag::<SelectionCommentFeatureFlag>() {
        log::warn!("AddSelectionCommentToThread ignored: selection-comment flag is off");
        return;
    }

    let Some(active_editor) = workspace
        .active_item(cx)
        .and_then(|item| item.act_as::<Editor>(cx))
    else {
        log::debug!("AddSelectionCommentToThread ignored: no active editor");
        return;
    };

    // Both the visible marker and the queued pill derive from this one
    // newest selection, snapshotted now so later cursor moves cannot
    // redirect the queued content.
    let newest_selection = active_editor.read(cx).selections.newest_anchor();
    let anchor_range: Range<Anchor> = newest_selection.start.clone()..newest_selection.end.clone();
    let queued_ranges = active_editor
        .read(cx)
        .selection_comment_queue_ranges(&anchor_range, cx);
    if queued_ranges.is_empty() {
        log::debug!("AddSelectionCommentToThread ignored: selection resolves to no quotable lines");
        return;
    }
    let queued_selection = AgentContextSelection::Editor(queued_ranges);
    let use_modifier_to_send = agent_settings::AgentSettings::get_global(cx).use_modifier_to_send;

    let workspace_handle = cx.entity().downgrade();
    let callback: SelectionCommentCallback = Rc::new(
        move |comment_text: String, window: &mut Window, cx: &mut App| {
            let Some(workspace) = workspace_handle.upgrade() else {
                log::warn!("Selection comment confirmed but the workspace is gone; nothing queued");
                return;
            };
            let queued_selection = queued_selection.clone();
            let comment_text = comment_text.clone();
            workspace.update(cx, |_workspace, cx| {
                cx.defer_in(window, move |workspace, window, cx| {
                    let conversation_view = workspace
                        .panel::<AgentPanel>(cx)
                        .and_then(|panel| panel.read(cx).active_conversation_view().cloned());
                    let Some(conversation_view) = conversation_view else {
                        log::warn!("Selection comment confirmed but no agent thread is open; nothing queued");
                        return;
                    };
                    crate::selection_comments::queue_selection_comment(
                        &conversation_view,
                        queued_selection,
                        comment_text,
                        window,
                        cx,
                    );
                });
            });
        },
    );

    active_editor.update(cx, |editor, cx| {
        editor.show_selection_comment(anchor_range, callback, use_modifier_to_send, window, cx);
    });
}

/// Appends one queued selection pill plus the comment prose to the thread
/// draft. Kept panel-independent so the bridging is directly testable; the
/// action handler below only resolves which view to target.
pub(crate) fn queue_selection_comment(
    conversation_view: &Entity<ConversationView>,
    selection: AgentContextSelection,
    comment_text: String,
    window: &mut Window,
    cx: &mut App,
) {
    conversation_view.update(cx, |conversation_view, cx| {
        conversation_view.insert_selection(selection, window, cx);
        if let Some(thread) = conversation_view.active_thread() {
            let comment = format!("\n{comment_text}\n");
            thread.update(cx, |thread, cx| {
                thread.active_editor(cx).update(cx, |editor, cx| {
                    editor.insert_text(&comment, window, cx);
                });
            });
        }
    });
}

pub(crate) fn clear_all_selection_comment_blocks(workspace: &Entity<Workspace>, cx: &mut App) {
    let handles: Vec<WeakEntity<Workspace>> = workspace
        .read(cx)
        .app_state()
        .workspace_store
        .read(cx)
        .workspaces()
        .cloned()
        .collect();
    let mut editors = Vec::new();
    for weak in handles {
        if let Some(workspace) = weak.upgrade() {
            for pane in workspace.read(cx).panes().iter() {
                for item in pane.read(cx).items() {
                    if let Some(editor) = item.act_as::<Editor>(cx) {
                        editors.push(editor);
                    }
                }
            }
        }
    }
    for editor in editors {
        editor.update(cx, |editor, cx| {
            editor.clear_selection_comments(cx);
        });
    }
}
