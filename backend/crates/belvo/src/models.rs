//! Belvo API models and types.

use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// Widget Token
// ============================================================================

/// Response from widget token creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetToken {
    pub access: String,
    pub refresh: String,
}

/// Request to create a widget token.
#[derive(Debug, Serialize)]
pub struct CreateWidgetTokenRequest {
    pub id: String,
    pub password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub widget: Option<WidgetConfig>,
}

/// Widget configuration options.
#[derive(Debug, Serialize)]
pub struct WidgetConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branding: Option<WidgetBranding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callback_urls: Option<WidgetCallbackUrls>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub institutions: Option<Vec<String>>,
}

/// Widget branding options.
#[derive(Debug, Serialize)]
pub struct WidgetBranding {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company_logo: Option<String>,
}

/// Widget callback URLs.
#[derive(Debug, Serialize)]
pub struct WidgetCallbackUrls {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<String>,
}

// ============================================================================
// Link
// ============================================================================

/// Belvo Link representing a connection to a bank.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    pub id: Uuid,
    pub institution: String,
    #[serde(default)]
    pub institution_user_id: Option<String>,
    pub access_mode: AccessMode,
    pub status: LinkStatus,
    pub created_by: Option<Uuid>,
    pub external_id: Option<String>,
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub last_accessed_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub refresh_rate: Option<String>,
}

/// Link access mode.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AccessMode {
    #[default]
    Recurrent,
    Single,
}

/// Link status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkStatus {
    Valid,
    Invalid,
    Unconfirmed,
    TokenRequired,
}

/// Request to register a new link.
#[derive(Debug, Serialize)]
pub struct RegisterLinkRequest {
    pub institution: String,
    pub username: String,
    pub password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_mode: Option<AccessMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
}

// ============================================================================
// Institution
// ============================================================================

/// Belvo Institution (bank).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Institution {
    pub id: Option<i64>,
    pub name: String,
    #[serde(rename = "type")]
    pub institution_type: String,
    pub code: Option<String>,
    pub country_code: Option<String>,
    pub country_codes: Option<Vec<String>>,
    pub display_name: Option<String>,
    pub primary_color: Option<String>,
    pub logo: Option<String>,
    pub icon_logo: Option<String>,
    pub text_logo: Option<String>,
    pub form_fields: Option<Vec<FormField>>,
    pub features: Option<Vec<InstitutionFeature>>,
    pub resources: Option<Vec<String>>,
    pub integration_type: Option<String>,
    pub status: Option<String>,
}

/// Form field for institution authentication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormField {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
    pub label: Option<String>,
    pub validation: Option<String>,
    pub placeholder: Option<String>,
    pub validation_message: Option<String>,
}

/// Institution feature.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstitutionFeature {
    pub name: String,
    pub description: Option<String>,
}

// ============================================================================
// Account
// ============================================================================

/// Belvo Account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: Uuid,
    pub link: Uuid,
    pub institution: Option<Institution>,
    pub created_at: Option<DateTime<Utc>>,
    pub collected_at: Option<DateTime<Utc>>,
    pub category: AccountCategory,
    #[serde(rename = "type")]
    pub account_type: Option<String>,
    pub name: Option<String>,
    pub number: Option<String>,
    pub balance: AccountBalance,
    pub currency: String,
    pub public_identification_name: Option<String>,
    pub public_identification_value: Option<String>,
    pub last_accessed_at: Option<DateTime<Utc>>,
    pub credit_data: Option<CreditData>,
    pub loan_data: Option<LoanData>,
    pub funds_data: Option<Vec<FundsData>>,
    pub bank_product_id: Option<String>,
    pub internal_identification: Option<String>,
}

/// Account category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AccountCategory {
    CheckingAccount,
    SavingsAccount,
    CreditCard,
    LoanAccount,
    PensionFundAccount,
    InvestmentAccount,
    Uncategorized,
    #[serde(other)]
    Other,
}

/// Account balance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountBalance {
    pub current: Option<BigDecimal>,
    pub available: Option<BigDecimal>,
}

/// Credit card data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditData {
    pub credit_limit: Option<BigDecimal>,
    pub collected_at: Option<DateTime<Utc>>,
    pub cutting_date: Option<String>,
    pub next_payment_date: Option<String>,
    pub minimum_payment: Option<BigDecimal>,
    pub no_interest_payment: Option<BigDecimal>,
    pub interest_rate: Option<BigDecimal>,
}

/// Loan data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoanData {
    pub collected_at: Option<DateTime<Utc>>,
    pub contract_amount: Option<BigDecimal>,
    pub principal: Option<BigDecimal>,
    pub loan_type: Option<String>,
    pub payment_day: Option<String>,
    pub outstanding_principal: Option<BigDecimal>,
    pub outstanding_balance: Option<BigDecimal>,
    pub monthly_payment: Option<BigDecimal>,
    pub interest_rates: Option<Vec<InterestRate>>,
    pub fees: Option<Vec<LoanFee>>,
    pub number_of_installments_total: Option<i32>,
    pub number_of_installments_outstanding: Option<i32>,
    pub contract_start_date: Option<String>,
    pub contract_end_date: Option<String>,
    pub contract_number: Option<String>,
    pub credit_limit: Option<BigDecimal>,
    pub last_period_balance: Option<BigDecimal>,
    pub interest_rate: Option<BigDecimal>,
    pub limit_day: Option<String>,
    pub cutting_day: Option<String>,
    pub cutting_date: Option<String>,
    pub last_payment_date: Option<String>,
    pub no_interest_payment: Option<BigDecimal>,
}

/// Interest rate details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterestRate {
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub rate_type: Option<String>,
    pub value: Option<BigDecimal>,
}

/// Loan fee details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoanFee {
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub fee_type: Option<String>,
    pub value: Option<BigDecimal>,
}

/// Funds data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundsData {
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub fund_type: Option<String>,
    pub public_identifications: Option<Vec<PublicIdentification>>,
    pub balance: Option<FundsBalance>,
    pub percentage: Option<BigDecimal>,
}

/// Public identification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicIdentification {
    pub name: Option<String>,
    pub value: Option<String>,
}

/// Funds balance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundsBalance {
    pub current: Option<BigDecimal>,
    pub available: Option<BigDecimal>,
}

/// Request to retrieve accounts.
#[derive(Debug, Serialize)]
pub struct RetrieveAccountsRequest {
    pub link: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub save_data: Option<bool>,
}

// ============================================================================
// Transaction
// ============================================================================

/// Belvo Transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub account: TransactionAccount,
    pub collected_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub value_date: Option<NaiveDate>,
    pub accounting_date: Option<NaiveDate>,
    pub amount: BigDecimal,
    pub balance: Option<BigDecimal>,
    pub currency: String,
    pub description: Option<String>,
    pub observations: Option<String>,
    pub merchant: Option<TransactionMerchant>,
    pub category: Option<String>,
    pub subcategory: Option<String>,
    pub reference: Option<String>,
    #[serde(rename = "type")]
    pub transaction_type: TransactionType,
    pub status: TransactionStatus,
    pub internal_identification: Option<String>,
}

/// Transaction account reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionAccount {
    pub id: Uuid,
    pub link: Option<Uuid>,
    pub name: Option<String>,
    pub number: Option<String>,
    pub category: Option<AccountCategory>,
}

/// Transaction merchant info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionMerchant {
    pub name: Option<String>,
    pub logo: Option<String>,
    pub website: Option<String>,
}

/// Transaction type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionType {
    Inflow,
    Outflow,
    #[serde(other)]
    Other,
}

/// Transaction status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionStatus {
    Pending,
    Processed,
    Uncategorized,
    #[serde(other)]
    Other,
}

/// Request to retrieve transactions.
#[derive(Debug, Serialize)]
pub struct RetrieveTransactionsRequest {
    pub link: Uuid,
    pub date_from: NaiveDate,
    pub date_to: NaiveDate,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub save_data: Option<bool>,
}

// ============================================================================
// Pagination
// ============================================================================

/// Paginated response wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub count: i64,
    pub next: Option<String>,
    pub previous: Option<String>,
    pub results: Vec<T>,
}

// ============================================================================
// Webhook
// ============================================================================

/// Webhook event types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebhookEventType {
    LinkCreated,
    LinkUpdated,
    LinkDeleted,
    LinkRefreshError,
    AccountsCreated,
    TransactionsCreated,
    HistoricalUpdate,
    #[serde(other)]
    Unknown,
}

/// Webhook payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    pub webhook_id: Option<String>,
    pub webhook_type: Option<String>,
    pub webhook_code: WebhookEventType,
    pub link_id: Option<Uuid>,
    pub external_id: Option<String>,
    pub request_id: Option<String>,
    pub data: Option<serde_json::Value>,
}

// ============================================================================
// Colombian Banks (Supported Institutions)
// ============================================================================

/// Colombian bank institution codes supported by Belvo.
pub mod institutions {
    // Banking institutions
    pub const BANCOLOMBIA: &str = "bancolombia_retail_co";
    pub const DAVIVIENDA: &str = "davivienda_retail_co";
    pub const BBVA_COLOMBIA: &str = "bbva_co";
    pub const BANCO_DE_BOGOTA: &str = "banco_de_bogota_co";
    pub const BANCO_FALABELLA: &str = "banco_falabella_co";

    // Digital wallets / Neobanks
    pub const NEQUI: &str = "nequi_co";
    pub const DAVIPLATA: &str = "daviplata_co";

    // Fiscal institution
    pub const DIAN: &str = "dian_co";

    /// List of all supported Colombian banking institutions.
    pub const COLOMBIA_INSTITUTIONS: &[&str] = &[
        BANCOLOMBIA,
        DAVIVIENDA,
        BBVA_COLOMBIA,
        BANCO_DE_BOGOTA,
        BANCO_FALABELLA,
        NEQUI,
        DAVIPLATA,
    ];

    /// Get display name for an institution code.
    pub fn display_name(code: &str) -> &'static str {
        match code {
            BANCOLOMBIA => "Bancolombia",
            DAVIVIENDA => "Davivienda",
            BBVA_COLOMBIA => "BBVA Colombia",
            BANCO_DE_BOGOTA => "Banco de Bogotá",
            BANCO_FALABELLA => "Banco Falabella",
            NEQUI => "Nequi",
            DAVIPLATA => "Daviplata",
            DIAN => "DIAN",
            _ => "Unknown Bank",
        }
    }
}
