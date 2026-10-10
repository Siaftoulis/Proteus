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
    HardwareRfidChute {
        device_id: String,
        target_container_code: String,
        auto_seal_on_count: Option<u32>,
    },
    DataBlock(super::data_blocks::DataBlockKind),
    TableSchema {
        table_name: String,
        columns: Vec<proteus_core::sql_ingest::SqlColumnMeta>,
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
    #[serde(default)]
    pub color_hue: Option<f32>,
}

impl FlowEdge {
    pub fn new(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            from_node: from.into(),
            from_port: "exec_out".into(),
            to_node: to.into(),
            to_port: "exec_in".into(),
            branch: None,
            color_hue: None,
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
            color_hue: None,
        }
    }

    pub fn with_color(mut self, hue: f32) -> Self {
        self.color_hue = Some(hue);
        self
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
            color_hue: None,
        }
    }

    /// Computes a deterministic hue [0.0..360.0] for a relation pair between two nodes.
    pub fn compute_relation_hue(from_node: &str, to_node: &str) -> f32 {
        let hash_src = format!("{}:{}", from_node, to_node);
        let hash_val: u32 = hash_src.bytes().fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));
        (hash_val % 360) as f32
    }
}
