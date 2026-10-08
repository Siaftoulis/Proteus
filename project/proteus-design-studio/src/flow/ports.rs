use super::nodes::FlowNodeKind;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PortDirection {
    Input,
    Output,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PortDataType {
    ExecutionFlow,
    Boolean,
    Record,
    Text,
    Number,
    Any,
}

impl PortDataType {
    pub fn is_compatible_with(&self, target: &PortDataType) -> bool {
        if *self == PortDataType::Any || *target == PortDataType::Any {
            return true;
        }
        self == target
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            PortDataType::ExecutionFlow => "Flow",
            PortDataType::Boolean => "Bool",
            PortDataType::Record => "Record",
            PortDataType::Text => "Text",
            PortDataType::Number => "Number",
            PortDataType::Any => "Any",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct NodePort {
    pub id: String,
    pub label: String,
    pub direction: PortDirection,
    pub data_type: PortDataType,
}

impl NodePort {
    pub fn input(id: impl Into<String>, label: impl Into<String>, data_type: PortDataType) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            direction: PortDirection::Input,
            data_type,
        }
    }

    pub fn output(id: impl Into<String>, label: impl Into<String>, data_type: PortDataType) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            direction: PortDirection::Output,
            data_type,
        }
    }
}

pub fn default_ports_for_kind(kind: &FlowNodeKind) -> (Vec<NodePort>, Vec<NodePort>) {
    match kind {
        FlowNodeKind::TriggerClick { .. } => (
            vec![],
            vec![NodePort::output("exec_out", "Trigger", PortDataType::ExecutionFlow)],
        ),
        FlowNodeKind::NavigateTo { .. } => (
            vec![NodePort::input("exec_in", "Enter", PortDataType::ExecutionFlow)],
            vec![NodePort::output("exec_out", "Next", PortDataType::ExecutionFlow)],
        ),
        FlowNodeKind::SaveToDatabase { .. } => (
            vec![
                NodePort::input("exec_in", "Save", PortDataType::ExecutionFlow),
                NodePort::input("data_in", "Record", PortDataType::Record),
            ],
            vec![
                NodePort::output("exec_out", "Saved", PortDataType::ExecutionFlow),
                NodePort::output("error_out", "Error", PortDataType::ExecutionFlow),
            ],
        ),
        FlowNodeKind::Condition { .. } => (
            vec![NodePort::input("exec_in", "Eval", PortDataType::ExecutionFlow)],
            vec![
                NodePort::output("branch_true", "True", PortDataType::ExecutionFlow),
                NodePort::output("branch_false", "False", PortDataType::ExecutionFlow),
            ],
        ),
        FlowNodeKind::ShowToast { .. } => (
            vec![NodePort::input("exec_in", "Show", PortDataType::ExecutionFlow)],
            vec![NodePort::output("exec_out", "Done", PortDataType::ExecutionFlow)],
        ),
        FlowNodeKind::FederationBridge { .. } => (
            vec![NodePort::input("exec_in", "Sync", PortDataType::ExecutionFlow)],
            vec![
                NodePort::output("exec_out", "Synced", PortDataType::ExecutionFlow),
                NodePort::output("error_out", "Failed", PortDataType::ExecutionFlow),
            ],
        ),
    }
}
