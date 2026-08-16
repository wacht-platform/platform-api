use super::core::AgentExecutor;
use dto::json::ProjectTaskBoardPromptItem;
use models::{AiTool, SchemaField};
use serde_json::json;

impl AgentExecutor {
    pub(crate) fn constrain_tool_input_schema(
        &self,
        tool: &AiTool,
        fields: &mut Vec<SchemaField>,
        active_board_item: Option<&ProjectTaskBoardPromptItem>,
    ) {
        let is_coordinator = self.effective_is_coordinator_thread();
        let is_conversation = self.is_conversation_thread && !is_coordinator;

        if is_coordinator
            && matches!(
                tool.name.as_str(),
                "assign_project_task" | "update_project_task"
            )
        {
            if let Some(active_board_item) = active_board_item {
                if let Some(task_key_field) =
                    fields.iter_mut().find(|field| field.name == "task_key")
                {
                    task_key_field.enum_values =
                        Some(vec![json!(active_board_item.task_key.clone())]);
                    task_key_field.description = Some(format!(
                        "Must be the active task key `{}` for this coordinator decision.",
                        active_board_item.task_key
                    ));
                }
            }
        }

        if tool.name == "update_project_task" {
            if is_conversation {
                fields.retain(|field| {
                    matches!(field.name.as_str(), "task_key" | "title" | "description")
                });
            } else if is_coordinator {
                if let Some(status_field) = fields.iter_mut().find(|field| field.name == "status") {
                    status_field.enum_values = Some(
                        crate::executor::project::status_machine::update_project_task_statuses()
                            .iter()
                            .map(|status| json!(status))
                            .collect(),
                    );
                    status_field.description = Some(
                        "Optional updated task status for coordinator decisions. Supported values: pending, in_progress, completed, blocked, cancelled, failed, waiting_for_children, or needs_clarification. Omit when status should stay unchanged.".to_string(),
                    );
                }
            }
        }
    }
}
