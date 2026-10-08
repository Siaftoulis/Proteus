// Proteus Core — Fiscal Hardware (ΦΗΜ) & EFT-POS Protocol Types
// 100% Original Implementation. Zero third-party boilerplate.
// Compliant with AADE Decision A.1155/2023 for Retail Cash Desk Interlocking.

use serde::{Deserialize, Serialize};

/// Type of Greek Fiscal Mechanism (Φορολογικός Ηλεκτρονικός Μηχανισμός).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FiscalDeviceType {
    /// Type A: Electronic Cash Register (ΦΗΜ - Ταμειακή Μηχανή: RBS, Casio, Diga, ICS).
    CashRegisterTypeA,
    /// Type B: Fiscal Signature Mechanism (ΕΑΦΔΣΣ / TaxSpooler / Algobox).
    SignatureMechanismTypeB,
    /// Type C: Modern Open Fiscal Box (ΦΗΜAS / TaxBox / Web REST & Socket).
    ModernTaxBoxTypeC,
    /// Direct Cloud AADE (Direct Software myDATA Provider with Digital QR stamp).
    DirectCloudAade,
}

impl Default for FiscalDeviceType {
    fn default() -> Self {
        FiscalDeviceType::CashRegisterTypeA
    }
}

/// Physical or Network Connection Interface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionInterface {
    /// Serial RS-232 / USB Virtual COM port (e.g. "COM3", 9600 or 115200 baud).
    SerialCom { port: String, baud_rate: u32 },
    /// Local Area Network TCP/IP Socket (e.g. "192.168.1.150:10009").
    TcpSocket { host: String, port: u16 },
    /// Windows Print Spooler RAW redirection.
    WindowsSpooler { printer_name: String },
}

impl Default for ConnectionInterface {
    fn default() -> Self {
        ConnectionInterface::TcpSocket {
            host: "127.0.0.1".to_string(),
            port: 10009,
        }
    }
}

/// Persistent configuration for Fiscal Devices and EFT-POS Interconnection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiscalHardwareConfig {
    pub device_type: FiscalDeviceType,
    pub connection: ConnectionInterface,
    pub timeout_ms: u64,
    /// Whether EFT-POS card terminal interlocking is mandatory (AADE A.1155/2023).
    pub eft_pos_interlock_enabled: bool,
    /// EFT-POS terminal IP address or COM port.
    pub eft_pos_host: String,
    pub eft_pos_port: u16,
    pub eft_pos_terminal_id: String,
    pub eft_pos_merchant_id: String,
    /// Shared secret key for A.1155 HMAC integrity signature verification.
    pub a1155_secret_key: String,
    pub auto_issue_daily_z: bool,
}

impl Default for FiscalHardwareConfig {
    fn default() -> Self {
        Self {
            device_type: FiscalDeviceType::CashRegisterTypeA,
            connection: ConnectionInterface::TcpSocket {
                host: "127.0.0.1".to_string(),
                port: 10009,
            },
            timeout_ms: 5000,
            eft_pos_interlock_enabled: true,
            eft_pos_host: "127.0.0.1".to_string(),
            eft_pos_port: 8080,
            eft_pos_terminal_id: "POS-001".to_string(),
            eft_pos_merchant_id: "MERCH-001".to_string(),
            a1155_secret_key: "PROTEUS_A1155_MASTER_KEY".to_string(),
            auto_issue_daily_z: false,
        }
    }
}

/// EFT-POS Card Transaction Operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PosTransactionType {
    Sale,
    Refund,
    Void,
}

/// Status of EFT-POS Handshake & Card Settlement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PosTransactionStatus {
    Approved,
    Declined,
    CancelledByUser,
    Timeout,
    CommunicationError,
}

/// Request payload dispatched to the Bank Card Terminal under A.1155/2023.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PosInitiateRequest {
    pub transaction_id: String,
    pub operation: PosTransactionType,
    /// Amount in cents (e.g. 1550 = €15.50).
    pub amount_cents: u64,
    /// ISO 4217 currency numeric code (978 for EUR).
    pub currency_code: u16,
    pub receipt_number: u32,
    pub invoice_type: String,
    pub cashier_id: String,
}

/// Certified response returned by the EFT-POS terminal upon card swipe/chip/NFC.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PosTerminalResponse {
    pub transaction_id: String,
    pub status: PosTransactionStatus,
    /// Retrieval Reference Number (12 digits, e.g. "410415123456").
    pub rrn: String,
    /// Bank authorization code (e.g. "042918").
    pub auth_code: String,
    pub terminal_id: String,
    pub merchant_id: String,
    pub card_brand: String,
    pub card_masked: String,
    /// System Trace Audit Number.
    pub stan: String,
    /// Cryptographic interconnection hash mandated by A.1155/2023.
    pub interconnection_signature: String,
    pub timestamp: i64,
    pub raw_response_code: String,
    pub response_message: String,
}

/// Individual item sold at the retail cash desk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiscalReceiptItem {
    pub name: String,
    pub quantity: f64,
    pub unit_price: f64,
    pub vat_rate: f64,
    pub department: u8,
}

/// Aggregated VAT bucket for the receipt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiscalVatBreakdown {
    pub vat_rate: f64,
    pub net_amount: f64,
    pub vat_amount: f64,
}

/// Full fiscal receipt dispatched to the cash register or fiscal box.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiscalReceiptPayload {
    pub receipt_number: u32,
    pub issue_date: String,
    pub items: Vec<FiscalReceiptItem>,
    pub vat_breakdowns: Vec<FiscalVatBreakdown>,
    pub total_net: f64,
    pub total_vat: f64,
    pub total_gross: f64,
    /// "CASH", "CARD", or "MIXED".
    pub payment_method: String,
    /// Card transaction response if paid by card (Mandatory under A.1155/2023).
    pub card_payment: Option<PosTerminalResponse>,
}

/// Result returned after fiscal hardware execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiscalExecutionResult {
    pub success: bool,
    pub receipt_number: u32,
    /// Fiscal signature from ΦΗΜ / ΕΑΦΔΣΣ (e.g. "EAFDSS_HASH_40_CHARS").
    pub fiscal_signature: String,
    /// Daily Z counter at the time of execution.
    pub z_report_counter: u32,
    pub error_message: Option<String>,
}

/// Summary of daily sales and tax breakdown from the cash register Z report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyZReport {
    pub z_counter: u32,
    pub report_date: String,
    pub total_gross_sales: f64,
    pub total_net_sales: f64,
    pub total_vat_collected: f64,
    pub total_cash_received: f64,
    pub total_card_settled: f64,
    pub total_receipts_count: u32,
    pub fiscal_block_hash: String,
}
