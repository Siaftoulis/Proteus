//! Declarative Hardware Profile Manifest in .pr Packages (Micro-task 25.1.2).
//! Allows `.pr` solution packages to declare required/optional physical and IoT peripherals
//! (scales, scanners, pole displays, relay boards) with Greek & English pairing instructions
//! and automatic host hardware compliance verification (Rule 5: Zero mock data).

use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use super::bus::PeripheralBus;
use super::types::{HardwareBusError, PeripheralDeviceRole, PeripheralProtocol, PeripheralStatus};

/// Single hardware peripheral requirement declared by a `.pr` package.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HardwareRequirement {
    pub requirement_key: String,
    pub name: String,
    pub role: PeripheralDeviceRole,
    pub is_mandatory: bool,
    pub supported_protocols: Vec<PeripheralProtocol>,
    pub baud_rate_hint: Option<u32>,
    pub pairing_instructions_el: String,
    pub pairing_instructions_en: String,
}

/// Complete declarative hardware profile manifest bundled inside a `.pr` package.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HardwareProfileManifest {
    pub profile_id: String,
    pub title: String,
    pub version: String,
    pub requirements: Vec<HardwareRequirement>,
    pub auto_bind_matching: bool,
    pub watchdog_timeout_secs: u32,
    pub created_at: String,
}

impl Default for HardwareProfileManifest {
    fn default() -> Self {
        Self {
            profile_id: "HW-PROFILE-DEFAULT".into(),
            title: "Standard POS Hardware Profile".into(),
            version: "1.0.0".into(),
            requirements: Vec::new(),
            auto_bind_matching: true,
            watchdog_timeout_secs: 15,
            created_at: Utc::now().to_rfc3339(),
        }
    }
}

/// Active binding mapping a package hardware requirement to a physical device on the bus.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HardwareBinding {
    pub requirement_key: String,
    pub device_id: String,
    pub role: PeripheralDeviceRole,
    pub bound_at: String,
}

/// Hardware compliance and readiness report for a mounted package profile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HardwareComplianceReport {
    pub profile_id: String,
    pub is_fully_operational: bool,
    pub satisfied_bindings: Vec<HardwareBinding>,
    pub missing_mandatory: Vec<HardwareRequirement>,
    pub missing_optional: Vec<HardwareRequirement>,
}

pub struct HardwareProfileEngine;

impl HardwareProfileEngine {
    /// Initializes SQLite tables for hardware profile manifests and device bindings.
    pub fn init_schema(conn: &Connection) -> Result<(), HardwareBusError> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS system_package_hardware_profiles (
                bundle_id TEXT PRIMARY KEY,
                profile_id TEXT NOT NULL,
                manifest_json TEXT NOT NULL,
                created_at TEXT NOT NULL
            );",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS system_hardware_profile_bindings (
                bundle_id TEXT NOT NULL,
                requirement_key TEXT NOT NULL,
                device_id TEXT NOT NULL,
                role TEXT NOT NULL,
                bound_at TEXT NOT NULL,
                PRIMARY KEY (bundle_id, requirement_key)
            );",
            [],
        )?;

        Ok(())
    }

    /// Stores a hardware profile manifest linked to a mounted package bundle ID.
    pub fn save_profile(
        conn: &Connection,
        bundle_id: &str,
        profile: &HardwareProfileManifest,
    ) -> Result<(), HardwareBusError> {
        Self::init_schema(conn)?;
        let manifest_json = serde_json::to_string(profile)?;
        let now = Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO system_package_hardware_profiles (bundle_id, profile_id, manifest_json, created_at)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(bundle_id) DO UPDATE SET
                profile_id = excluded.profile_id,
                manifest_json = excluded.manifest_json",
            params![bundle_id, profile.profile_id, manifest_json, now],
        )?;

        Ok(())
    }

    /// Retrieves a hardware profile manifest by bundle ID.
    pub fn get_profile(
        conn: &Connection,
        bundle_id: &str,
    ) -> Result<Option<HardwareProfileManifest>, HardwareBusError> {
        Self::init_schema(conn)?;
        let mut stmt = conn.prepare(
            "SELECT manifest_json FROM system_package_hardware_profiles WHERE bundle_id = ?1",
        )?;

        let mut rows = stmt.query(params![bundle_id])?;
        if let Some(row) = rows.next()? {
            let json_str: String = row.get(0)?;
            let profile: HardwareProfileManifest = serde_json::from_str(&json_str)?;
            Ok(Some(profile))
        } else {
            Ok(None)
        }
    }

    /// Binds a specific registered peripheral device to a profile requirement.
    pub fn bind_peripheral(
        conn: &Connection,
        bundle_id: &str,
        req_key: &str,
        device_id: &str,
        role: PeripheralDeviceRole,
    ) -> Result<(), HardwareBusError> {
        Self::init_schema(conn)?;
        let role_str = serde_json::to_string(&role)?;
        let now = Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO system_hardware_profile_bindings (bundle_id, requirement_key, device_id, role, bound_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(bundle_id, requirement_key) DO UPDATE SET
                device_id = excluded.device_id,
                role = excluded.role,
                bound_at = excluded.bound_at",
            params![bundle_id, req_key, device_id, role_str, now],
        )?;

        Ok(())
    }

    /// Unbinds a peripheral from a package requirement.
    pub fn unbind_peripheral(
        conn: &Connection,
        bundle_id: &str,
        req_key: &str,
    ) -> Result<(), HardwareBusError> {
        Self::init_schema(conn)?;
        conn.execute(
            "DELETE FROM system_hardware_profile_bindings WHERE bundle_id = ?1 AND requirement_key = ?2",
            params![bundle_id, req_key],
        )?;
        Ok(())
    }

    /// Fetches all active bindings for a package.
    pub fn list_bindings(
        conn: &Connection,
        bundle_id: &str,
    ) -> Result<Vec<HardwareBinding>, HardwareBusError> {
        Self::init_schema(conn)?;
        let mut stmt = conn.prepare(
            "SELECT requirement_key, device_id, role, bound_at 
             FROM system_hardware_profile_bindings 
             WHERE bundle_id = ?1",
        )?;

        let rows = stmt.query_map(params![bundle_id], |r| {
            let role_raw: String = r.get(2)?;
            let role: PeripheralDeviceRole = serde_json::from_str(&role_raw)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(2, rusqlite::types::Type::Text, Box::new(e)))?;
            Ok(HardwareBinding {
                requirement_key: r.get(0)?,
                device_id: r.get(1)?,
                role,
                bound_at: r.get(3)?,
            })
        })?;

        let mut list = Vec::new();
        for row in rows {
            list.push(row?);
        }
        Ok(list)
    }

    /// Evaluates whether the host machine satisfies all declared hardware requirements.
    pub fn evaluate_compliance(
        conn: &Connection,
        bundle_id: &str,
        profile: &HardwareProfileManifest,
    ) -> Result<HardwareComplianceReport, HardwareBusError> {
        let existing_bindings = Self::list_bindings(conn, bundle_id)?;
        let registered_devices = PeripheralBus::list_devices(conn)?;

        let mut satisfied = Vec::new();
        let mut missing_mandatory = Vec::new();
        let mut missing_optional = Vec::new();

        for req in &profile.requirements {
            // Check if explicitly bound and the bound device is currently online/ready
            let explicit_binding = existing_bindings.iter().find(|b| b.requirement_key == req.requirement_key);

            let is_matched = if let Some(binding) = explicit_binding {
                registered_devices.iter().any(|d| {
                    d.device_id == binding.device_id
                        && d.role == req.role
                        && d.is_enabled
                        && d.status != PeripheralStatus::Disconnected
                })
            } else {
                // Check if any matching registered device of this role exists and is ready
                registered_devices.iter().any(|d| {
                    d.role == req.role && d.is_enabled && d.status != PeripheralStatus::Disconnected
                })
            };

            if is_matched {
                let dev_id = explicit_binding
                    .map(|b| b.device_id.clone())
                    .or_else(|| {
                        registered_devices
                            .iter()
                            .find(|d| d.role == req.role && d.is_enabled)
                            .map(|d| d.device_id.clone())
                    })
                    .unwrap_or_else(|| "AUTO_MATCH".into());

                satisfied.push(HardwareBinding {
                    requirement_key: req.requirement_key.clone(),
                    device_id: dev_id,
                    role: req.role,
                    bound_at: Utc::now().to_rfc3339(),
                });
            } else if req.is_mandatory {
                missing_mandatory.push(req.clone());
            } else {
                missing_optional.push(req.clone());
            }
        }

        let is_fully_operational = missing_mandatory.is_empty();

        Ok(HardwareComplianceReport {
            profile_id: profile.profile_id.clone(),
            is_fully_operational,
            satisfied_bindings: satisfied,
            missing_mandatory,
            missing_optional,
        })
    }

    /// Automatically binds available registered peripherals matching requirement roles.
    pub fn auto_bind_available(
        conn: &Connection,
        bundle_id: &str,
        profile: &HardwareProfileManifest,
    ) -> Result<usize, HardwareBusError> {
        let devices = PeripheralBus::list_devices(conn)?;
        let mut bound_count = 0;

        for req in &profile.requirements {
            if let Some(dev) = devices.iter().find(|d| d.role == req.role && d.is_enabled) {
                Self::bind_peripheral(conn, bundle_id, &req.requirement_key, &dev.device_id, req.role)?;
                bound_count += 1;
            }
        }

        Ok(bound_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hardware::types::{PeripheralDeviceRecord, PeripheralProtocol, SerialParity};

    #[test]
    fn test_profile_schema_init_and_persistence() {
        let conn = Connection::open_in_memory().unwrap();
        HardwareProfileEngine::init_schema(&conn).unwrap();

        let profile = HardwareProfileManifest {
            profile_id: "HW-BUTCHER-POS".into(),
            title: "Butcher Shop POS Scale & Display".into(),
            version: "1.0.0".into(),
            requirements: vec![
                HardwareRequirement {
                    requirement_key: "meat_scale".into(),
                    name: "Electronic Deli Scale".into(),
                    role: PeripheralDeviceRole::WeightScale,
                    is_mandatory: true,
                    supported_protocols: vec![PeripheralProtocol::SerialUart {
                        port_name: "COM3".into(),
                        baud_rate: 9600,
                        data_bits: 8,
                        stop_bits: 1,
                        parity: SerialParity::None,
                    }],
                    baud_rate_hint: Some(9600),
                    pairing_instructions_el: "Συνδέστε τη ζυγαριά στη θύρα COM3.".into(),
                    pairing_instructions_en: "Connect scale to COM3 port at 9600 baud.".into(),
                },
                HardwareRequirement {
                    requirement_key: "pole_display".into(),
                    name: "VFD Customer Pole Display".into(),
                    role: PeripheralDeviceRole::CustomerPoleDisplay,
                    is_mandatory: false,
                    supported_protocols: vec![],
                    baud_rate_hint: None,
                    pairing_instructions_el: "Προαιρετική οθόνη πελάτη VFD.".into(),
                    pairing_instructions_en: "Optional VFD customer pole display.".into(),
                },
            ],
            auto_bind_matching: true,
            watchdog_timeout_secs: 10,
            created_at: Utc::now().to_rfc3339(),
        };

        HardwareProfileEngine::save_profile(&conn, "PKG-BUTCHER-01", &profile).unwrap();
        let loaded = HardwareProfileEngine::get_profile(&conn, "PKG-BUTCHER-01").unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().requirements.len(), 2);
    }

    #[test]
    fn test_evaluate_compliance_with_missing_mandatory() {
        let conn = Connection::open_in_memory().unwrap();
        let profile = HardwareProfileManifest {
            profile_id: "HW-TEST".into(),
            requirements: vec![HardwareRequirement {
                requirement_key: "scale".into(),
                name: "Scale".into(),
                role: PeripheralDeviceRole::WeightScale,
                is_mandatory: true,
                supported_protocols: vec![],
                baud_rate_hint: None,
                pairing_instructions_el: "".into(),
                pairing_instructions_en: "".into(),
            }],
            ..Default::default()
        };

        let report = HardwareProfileEngine::evaluate_compliance(&conn, "PKG-01", &profile).unwrap();
        assert!(!report.is_fully_operational);
        assert_eq!(report.missing_mandatory.len(), 1);
        assert_eq!(report.satisfied_bindings.len(), 0);
    }

    #[test]
    fn test_auto_bind_matching_peripherals_success() {
        let conn = Connection::open_in_memory().unwrap();
        PeripheralBus::init_schema(&conn).unwrap();

        // Register a physical scale
        let dev = PeripheralDeviceRecord {
            device_id: "SCALE-PHYSICAL-01".into(),
            name: "CAS Deli Scale".into(),
            role: PeripheralDeviceRole::WeightScale,
            protocol: PeripheralProtocol::VirtualLoopback { channel_id: "VIRT-0".into() },
            status: PeripheralStatus::OnlineReady,
            is_enabled: true,
            last_seen_at: Some(Utc::now().to_rfc3339()),
            created_at: Utc::now().to_rfc3339(),
        };
        PeripheralBus::register_device(&conn, &dev).unwrap();

        let profile = HardwareProfileManifest {
            profile_id: "HW-RETAIL".into(),
            requirements: vec![HardwareRequirement {
                requirement_key: "retail_scale".into(),
                name: "Weight Scale".into(),
                role: PeripheralDeviceRole::WeightScale,
                is_mandatory: true,
                supported_protocols: vec![],
                baud_rate_hint: None,
                pairing_instructions_el: "".into(),
                pairing_instructions_en: "".into(),
            }],
            ..Default::default()
        };

        let bound = HardwareProfileEngine::auto_bind_available(&conn, "PKG-RETAIL", &profile).unwrap();
        assert_eq!(bound, 1);

        let report = HardwareProfileEngine::evaluate_compliance(&conn, "PKG-RETAIL", &profile).unwrap();
        assert!(report.is_fully_operational);
        assert_eq!(report.satisfied_bindings.len(), 1);
        assert_eq!(report.satisfied_bindings[0].device_id, "SCALE-PHYSICAL-01");
    }

    #[test]
    fn test_bind_and_unbind_device() {
        let conn = Connection::open_in_memory().unwrap();
        HardwareProfileEngine::bind_peripheral(
            &conn,
            "PKG-01",
            "gate_relay",
            "RELAY-BOARD-01",
            PeripheralDeviceRole::IndustrialRelayActuator,
        )
        .unwrap();

        let bindings = HardwareProfileEngine::list_bindings(&conn, "PKG-01").unwrap();
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].device_id, "RELAY-BOARD-01");

        HardwareProfileEngine::unbind_peripheral(&conn, "PKG-01", "gate_relay").unwrap();
        let bindings_after = HardwareProfileEngine::list_bindings(&conn, "PKG-01").unwrap();
        assert_eq!(bindings_after.len(), 0);
    }
}
