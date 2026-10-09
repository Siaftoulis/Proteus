use super::ports::{default_ports_for_kind, NodePort, PortDirection};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum FlowNodeKind {
    TriggerClick { target_node_id: String },
    NavigateTo { page_id: String },
    SaveToDatabase { entity: String },
    Condition {
        field: String,
        operator: String,
        target_value: String,
    },
    ShowToast { message: String },
    FederationBridge {
        endpoint: String,
        source_entity: String,
        target_entity: String,
        bidirectional: bool,
    },
    HardwareRelay {
        device_id: String,
        channel: u8,
        action: String,
        pulse_ms: u32,
    },
    HardwareSensor {
        device_id: String,
        metric: String,
        operator: String,
        threshold: f32,
    },
    HardwareScale {
        device_id: String,
        require_stable: bool,
    },
    HardwareDisplay {
        device_id: String,
        line1: String,
        line2: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FlowNode {
    pub id: String,
    pub kind: FlowNodeKind,
    pub position: (f32, f32),
    #[serde(default)]
    pub input_ports: Vec<NodePort>,
    #[serde(default)]
    pub output_ports: Vec<NodePort>,
}

impl FlowNode {
    pub fn new(id: impl Into<String>, kind: FlowNodeKind, position: (f32, f32)) -> Self {
        let (inputs, outputs) = default_ports_for_kind(&kind);
        Self {
            id: id.into(),
            kind,
            position,
            input_ports: inputs,
            output_ports: outputs,
        }
    }

    pub fn find_port(&self, port_id: &str) -> Option<&NodePort> {
        self.input_ports.iter().find(|p| p.id == port_id)
            .or_else(|| self.output_ports.iter().find(|p| p.id == port_id))
    }

    pub fn find_port_direction(&self, port_id: &str) -> Option<PortDirection> {
        self.find_port(port_id).map(|p| p.direction)
    }

    pub fn ensure_ports(&mut self) {
        if self.input_ports.is_empty() && self.output_ports.is_empty() {
            let (inputs, outputs) = default_ports_for_kind(&self.kind);
            self.input_ports = inputs;
            self.output_ports = outputs;
        }
    }
}

fn default_exec_out() -> String {
    "exec_out".to_string()
}

fn default_exec_in() -> String {
    "exec_in".to_string()
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FlowEdge {
    pub from_node: String,
    #[serde(default = "default_exec_out")]
    pub from_port: String,
    pub to_node: String,
    #[serde(default = "default_exec_in")]
    pub to_port: String,
    #[serde(default)]
    pub branch: Option<String>,
}

impl FlowEdge {
    pub fn new(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            from_node: from.into(),
            from_port: "exec_out".into(),
            to_node: to.into(),
            to_port: "exec_in".into(),
            branch: None,
        }
    }

    pub fn with_ports(
        from_node: impl Into<String>,
        from_port: impl Into<String>,
        to_node: impl Into<String>,
        to_port: impl Into<String>,
    ) -> Self {
        Self {
            from_node: from_node.into(),
            from_port: from_port.into(),
            to_node: to_node.into(),
            to_port: to_port.into(),
            branch: None,
        }
    }

    pub fn with_branch(from: impl Into<String>, to: impl Into<String>, branch: impl Into<String>) -> Self {
        let branch_str: String = branch.into();
        let from_port = if branch_str == "false" { "branch_false" } else { "branch_true" };
        Self {
            from_node: from.into(),
            from_port: from_port.into(),
            to_node: to.into(),
            to_port: "exec_in".into(),
            branch: Some(branch_str),
        }
    }
}
