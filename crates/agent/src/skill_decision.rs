use blockcell_skills::manager::SkillExecution;
use serde_json::{json, Map, Value};

pub(crate) struct SkillDecisionEngine;

impl SkillDecisionEngine {
    pub(crate) fn build_structured_skill_schema(
        skill_name: &str,
        execution: &SkillExecution,
    ) -> Value {
        let actions = execution
            .actions
            .iter()
            .map(|action| {
                let arguments = action
                    .arguments
                    .iter()
                    .map(|(name, arg)| {
                        (
                            name.clone(),
                            json!({
                                "type": arg.kind,
                                "required": arg.required,
                                "description": arg.description,
                                "enum": arg.enum_values,
                            }),
                        )
                    })
                    .collect::<Map<String, Value>>();

                json!({
                    "name": action.name,
                    "description": action.description,
                    "triggers": action.triggers,
                    "arguments": arguments,
                    "argv": action.argv,
                })
            })
            .collect::<Vec<_>>();

        json!({
            "skill": skill_name,
            "runtime_kind": execution.normalized_kind(),
            "dispatch_kind": execution.effective_dispatch_kind(),
            "summary_mode": execution.effective_summary_mode(),
            "actions": actions,
        })
    }

    pub(crate) fn normalize_selected_skill_name(
        selected: &str,
        candidates: &[(String, String)],
    ) -> Option<String> {
        let selected = selected.trim();
        if selected.is_empty() {
            return None;
        }

        if let Some((name, _)) = candidates.iter().find(|(name, _)| name == selected) {
            return Some(name.clone());
        }

        candidates
            .iter()
            .find(|(name, _)| selected.contains(name.as_str()) || name.contains(selected))
            .map(|(name, _)| name.clone())
    }

    pub(crate) fn build_method_decision_prompt(
        user_input: &str,
        skill_name: &str,
        execution: &SkillExecution,
    ) -> String {
        let schema = Self::build_structured_skill_schema(skill_name, execution);
        format!(
            "你是技能调度器，请根据用户请求选择最合适的方法并构造参数。\n\n\
            技能结构化 schema：\n{}\n\n\
            用户请求：{}\n\n\
            请严格输出 JSON，不要包含任何额外说明：\n\
            {{\"method\": \"方法名\", \"arguments\": {{\"参数名\": \"参数值\"}}}}",
            schema, user_input
        )
    }
}
