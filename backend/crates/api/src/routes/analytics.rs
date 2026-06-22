//! Analytics routes for dashboard charts and summaries.
//!
//! Endpoints:
//! - GET /analytics/summary - Total income, expenses, and balance
//! - GET /analytics/by-category - Spending breakdown by category
//! - GET /analytics/monthly - Monthly income/expense trends

use actix_web::{get, web, HttpResponse, Responder};
use bigdecimal::{BigDecimal, ToPrimitive};
use chrono::{Datelike, NaiveDate};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use shared::{error::AppError, DbPool};
use std::collections::HashMap;
use uuid::Uuid;

use crate::extractors::AuthUser;

// ============================================================================
// Response DTOs
// ============================================================================

/// Summary of user's financial data.
#[derive(Debug, Serialize)]
pub struct SummaryResponse {
    pub total_income: f64,
    pub total_expenses: f64,
    pub net_balance: f64,
    pub transaction_count: i64,
    pub days_in_period: i64,
    pub daily_average: f64,
    pub savings_rate: f64,
    pub largest_expense: f64,
}

/// Spending by category breakdown.
#[derive(Debug, Serialize)]
pub struct CategoryBreakdown {
    pub category_id: Option<Uuid>,
    pub category_name: String,
    pub category_key: Option<String>,
    pub category_color: Option<String>,
    pub total_amount: f64,
    pub transaction_count: i64,
    pub percentage: f64,
}

#[derive(Debug, Serialize)]
pub struct SpendingByCategoryResponse {
    pub categories: Vec<CategoryBreakdown>,
    pub total_spending: f64,
}

/// Monthly income/expense data point.
#[derive(Debug, Serialize)]
pub struct MonthlyDataPoint {
    pub year: i32,
    pub month: u32,
    pub month_name: String,
    pub income: f64,
    pub expenses: f64,
    pub net: f64,
}

#[derive(Debug, Serialize)]
pub struct MonthlyTrendsResponse {
    pub data: Vec<MonthlyDataPoint>,
}

/// Single expense transaction for top expenses list.
#[derive(Debug, Serialize)]
pub struct TopExpenseItem {
    pub id: Uuid,
    pub date: NaiveDate,
    pub description: String,
    pub amount: f64,
    pub category_name: Option<String>,
    pub category_color: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TopExpensesResponse {
    pub expenses: Vec<TopExpenseItem>,
}

/// Merchant/description spending aggregate.
#[derive(Debug, Serialize)]
pub struct MerchantSpending {
    pub merchant: String,
    pub total_amount: f64,
    pub transaction_count: i64,
    pub percentage: f64,
}

#[derive(Debug, Serialize)]
pub struct TopMerchantsResponse {
    pub merchants: Vec<MerchantSpending>,
    pub total_spending: f64,
}

// ============================================================================
// Query parameters
// ============================================================================

/// Query parameters for analytics endpoints.
#[derive(Debug, Deserialize)]
pub struct AnalyticsQuery {
    /// Start date filter (inclusive)
    pub start_date: Option<NaiveDate>,
    /// End date filter (inclusive)
    pub end_date: Option<NaiveDate>,
    /// Number of months for monthly trends (default: 6)
    pub months: Option<i32>,
    /// Filter by category IDs (comma-separated UUIDs)
    pub category_ids: Option<String>,
    /// Filter by transaction type (true = income, false = expense)
    pub is_income: Option<bool>,
}

// ============================================================================
// Handlers
// ============================================================================

/// Parse comma-separated UUIDs from query string.
fn parse_category_ids(ids: &Option<String>) -> Vec<Uuid> {
    ids.as_ref()
        .map(|s| {
            s.split(',')
                .filter_map(|id| Uuid::parse_str(id.trim()).ok())
                .collect()
        })
        .unwrap_or_default()
}

/// Get financial summary for the user.
///
/// GET /api/v1/analytics/summary
#[get("/analytics/summary")]
pub async fn get_summary(
    auth_user: AuthUser,
    query: web::Query<AnalyticsQuery>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    use shared::schema::transactions;

    let mut conn = pool.get().await?;

    // Build query with optional filters
    let mut query_builder = transactions::table
        .filter(transactions::user_id.eq(auth_user.user_id))
        .into_boxed();

    // Calculate date range for days_in_period
    let end_date = query
        .end_date
        .unwrap_or_else(|| chrono::Utc::now().date_naive());
    let start_date = query.start_date.unwrap_or(end_date);

    query_builder = query_builder.filter(transactions::date.ge(start_date));
    query_builder = query_builder.filter(transactions::date.le(end_date));

    // Filter by category IDs
    let category_ids = parse_category_ids(&query.category_ids);
    if !category_ids.is_empty() {
        query_builder =
            query_builder.filter(transactions::category_id.eq_any(category_ids.clone()));
    }

    // Filter by transaction type
    if let Some(is_income) = query.is_income {
        query_builder = query_builder.filter(transactions::is_income.eq(is_income));
    }

    // Get all transactions and calculate totals
    let results: Vec<(BigDecimal, bool)> = query_builder
        .select((transactions::amount, transactions::is_income))
        .load(&mut conn)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let mut total_income = 0.0;
    let mut total_expenses = 0.0;
    let mut largest_expense = 0.0;

    for (amount, is_income) in &results {
        let value = amount.to_f64().unwrap_or(0.0).abs();
        if *is_income {
            total_income += value;
        } else {
            total_expenses += value;
            if value > largest_expense {
                largest_expense = value;
            }
        }
    }

    // Calculate days in period (minimum 1)
    let days_in_period = (end_date - start_date).num_days().max(1);
    let daily_average = total_expenses / days_in_period as f64;
    let savings_rate = if total_income > 0.0 {
        ((total_income - total_expenses) / total_income) * 100.0
    } else {
        0.0
    };

    Ok(HttpResponse::Ok().json(SummaryResponse {
        total_income,
        total_expenses,
        net_balance: total_income - total_expenses,
        transaction_count: results.len() as i64,
        days_in_period,
        daily_average,
        savings_rate,
        largest_expense,
    }))
}

/// Get spending breakdown by category.
///
/// GET /api/v1/analytics/by-category
#[get("/analytics/by-category")]
pub async fn get_spending_by_category(
    auth_user: AuthUser,
    query: web::Query<AnalyticsQuery>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    use shared::schema::{categories, transactions};

    let mut conn = pool.get().await?;

    // Build query for expenses only (negative amounts) unless is_income filter is set
    let mut query_builder = transactions::table
        .filter(transactions::user_id.eq(auth_user.user_id))
        .into_boxed();

    // Default to expenses for category breakdown unless explicitly filtered
    if let Some(is_income) = query.is_income {
        query_builder = query_builder.filter(transactions::is_income.eq(is_income));
    } else {
        query_builder = query_builder.filter(transactions::is_income.eq(false));
    }

    if let Some(start) = query.start_date {
        query_builder = query_builder.filter(transactions::date.ge(start));
    }
    if let Some(end) = query.end_date {
        query_builder = query_builder.filter(transactions::date.le(end));
    }

    // Filter by category IDs
    let category_ids = parse_category_ids(&query.category_ids);
    if !category_ids.is_empty() {
        query_builder =
            query_builder.filter(transactions::category_id.eq_any(category_ids.clone()));
    }

    // Get transactions with category info
    let results: Vec<(Option<Uuid>, BigDecimal)> = query_builder
        .select((transactions::category_id, transactions::amount))
        .load(&mut conn)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Load all categories
    let cats: Vec<(Uuid, String, Option<String>, Option<String>)> = categories::table
        .select((
            categories::id,
            categories::name,
            categories::key,
            categories::color,
        ))
        .load(&mut conn)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let cat_map: HashMap<Uuid, (String, Option<String>, Option<String>)> = cats
        .into_iter()
        .map(|(id, name, key, color)| (id, (name, key, color)))
        .collect();

    // Aggregate by category
    let mut by_category: HashMap<Option<Uuid>, (f64, i64)> = HashMap::new();
    let mut total_spending = 0.0;

    for (cat_id, amount) in results {
        let value = amount.to_f64().unwrap_or(0.0).abs();
        total_spending += value;

        let entry = by_category.entry(cat_id).or_insert((0.0, 0));
        entry.0 += value;
        entry.1 += 1;
    }

    // Convert to response
    let mut categories: Vec<CategoryBreakdown> = by_category
        .into_iter()
        .map(|(cat_id, (total, count))| {
            let (name, key, color) = cat_id
                .and_then(|id| cat_map.get(&id))
                .map(|(n, k, c)| (n.clone(), k.clone(), c.clone()))
                .unwrap_or_else(|| ("Sin categoría".to_string(), None, None));

            CategoryBreakdown {
                category_id: cat_id,
                category_name: name,
                category_key: key,
                category_color: color,
                total_amount: total,
                transaction_count: count,
                percentage: if total_spending > 0.0 {
                    (total / total_spending) * 100.0
                } else {
                    0.0
                },
            }
        })
        .collect();

    // Sort by amount descending
    categories.sort_by(|a, b| b.total_amount.partial_cmp(&a.total_amount).unwrap());

    Ok(HttpResponse::Ok().json(SpendingByCategoryResponse {
        categories,
        total_spending,
    }))
}

/// Get monthly income/expense trends.
///
/// GET /api/v1/analytics/monthly
#[get("/analytics/monthly")]
pub async fn get_monthly_trends(
    auth_user: AuthUser,
    query: web::Query<AnalyticsQuery>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    use shared::schema::transactions;

    let mut conn = pool.get().await?;
    let months = query.months.unwrap_or(6);

    // Calculate date range
    let end_date = query
        .end_date
        .unwrap_or_else(|| chrono::Utc::now().date_naive());
    let start_date = query
        .start_date
        .unwrap_or_else(|| end_date - chrono::Duration::days(months as i64 * 30));

    // Build query with filters
    let mut query_builder = transactions::table
        .filter(transactions::user_id.eq(auth_user.user_id))
        .filter(transactions::date.ge(start_date))
        .filter(transactions::date.le(end_date))
        .into_boxed();

    // Filter by category IDs
    let category_ids = parse_category_ids(&query.category_ids);
    if !category_ids.is_empty() {
        query_builder =
            query_builder.filter(transactions::category_id.eq_any(category_ids.clone()));
    }

    // Filter by transaction type
    if let Some(is_income) = query.is_income {
        query_builder = query_builder.filter(transactions::is_income.eq(is_income));
    }

    // Get all transactions in range
    let results: Vec<(NaiveDate, BigDecimal, bool)> = query_builder
        .select((
            transactions::date,
            transactions::amount,
            transactions::is_income,
        ))
        .load(&mut conn)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Aggregate by month
    let mut monthly_data: HashMap<(i32, u32), (f64, f64)> = HashMap::new();

    for (date, amount, is_income) in results {
        let key = (date.year(), date.month());
        let value = amount.to_f64().unwrap_or(0.0).abs();

        let entry = monthly_data.entry(key).or_insert((0.0, 0.0));
        if is_income {
            entry.0 += value;
        } else {
            entry.1 += value;
        }
    }

    // Convert to sorted list
    let mut data: Vec<MonthlyDataPoint> = monthly_data
        .into_iter()
        .map(|((year, month), (income, expenses))| MonthlyDataPoint {
            year,
            month,
            month_name: get_month_name(month),
            income,
            expenses,
            net: income - expenses,
        })
        .collect();

    // Sort by date
    data.sort_by(|a, b| (a.year, a.month).cmp(&(b.year, b.month)));

    Ok(HttpResponse::Ok().json(MonthlyTrendsResponse { data }))
}

/// Get top N expense transactions.
///
/// GET /api/v1/analytics/top-expenses
#[get("/analytics/top-expenses")]
pub async fn get_top_expenses(
    auth_user: AuthUser,
    query: web::Query<AnalyticsQuery>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    use shared::schema::{categories, transactions};

    let mut conn = pool.get().await?;

    // Build query for expenses only
    let mut query_builder = transactions::table
        .filter(transactions::user_id.eq(auth_user.user_id))
        .filter(transactions::is_income.eq(false))
        .into_boxed();

    if let Some(start) = query.start_date {
        query_builder = query_builder.filter(transactions::date.ge(start));
    }
    if let Some(end) = query.end_date {
        query_builder = query_builder.filter(transactions::date.le(end));
    }

    // Filter by category IDs
    let category_ids = parse_category_ids(&query.category_ids);
    if !category_ids.is_empty() {
        query_builder =
            query_builder.filter(transactions::category_id.eq_any(category_ids.clone()));
    }

    // Get top 10 expenses ordered by amount descending
    let results: Vec<(Uuid, NaiveDate, String, BigDecimal, Option<Uuid>)> = query_builder
        .select((
            transactions::id,
            transactions::date,
            transactions::description,
            transactions::amount,
            transactions::category_id,
        ))
        .order(transactions::amount.asc()) // Expenses are negative, so asc = largest expenses
        .limit(10)
        .load(&mut conn)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Load categories for names
    let cats: Vec<(Uuid, String, Option<String>)> = categories::table
        .select((categories::id, categories::name, categories::color))
        .load(&mut conn)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let cat_map: HashMap<Uuid, (String, Option<String>)> = cats
        .into_iter()
        .map(|(id, name, color)| (id, (name, color)))
        .collect();

    let expenses: Vec<TopExpenseItem> = results
        .into_iter()
        .map(|(id, date, description, amount, cat_id)| {
            let (cat_name, cat_color) = cat_id
                .and_then(|cid| cat_map.get(&cid))
                .map(|(n, c)| (Some(n.clone()), c.clone()))
                .unwrap_or((None, None));

            TopExpenseItem {
                id,
                date,
                description,
                amount: amount.to_f64().unwrap_or(0.0).abs(),
                category_name: cat_name,
                category_color: cat_color,
            }
        })
        .collect();

    Ok(HttpResponse::Ok().json(TopExpensesResponse { expenses }))
}

/// Get top spending by merchant/description.
///
/// GET /api/v1/analytics/top-merchants
#[get("/analytics/top-merchants")]
pub async fn get_top_merchants(
    auth_user: AuthUser,
    query: web::Query<AnalyticsQuery>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    use shared::schema::transactions;

    let mut conn = pool.get().await?;

    // Build query for expenses only
    let mut query_builder = transactions::table
        .filter(transactions::user_id.eq(auth_user.user_id))
        .filter(transactions::is_income.eq(false))
        .into_boxed();

    if let Some(start) = query.start_date {
        query_builder = query_builder.filter(transactions::date.ge(start));
    }
    if let Some(end) = query.end_date {
        query_builder = query_builder.filter(transactions::date.le(end));
    }

    // Filter by category IDs
    let category_ids = parse_category_ids(&query.category_ids);
    if !category_ids.is_empty() {
        query_builder =
            query_builder.filter(transactions::category_id.eq_any(category_ids.clone()));
    }

    // Get all expense transactions
    let results: Vec<(String, BigDecimal)> = query_builder
        .select((transactions::description, transactions::amount))
        .load(&mut conn)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Aggregate by description (simplified merchant extraction)
    let mut by_merchant: HashMap<String, (f64, i64)> = HashMap::new();
    let mut total_spending = 0.0;

    for (description, amount) in results {
        let value = amount.to_f64().unwrap_or(0.0).abs();
        total_spending += value;

        // Extract merchant name (first few words or simplified description)
        let merchant = extract_merchant_name(&description);

        let entry = by_merchant.entry(merchant).or_insert((0.0, 0));
        entry.0 += value;
        entry.1 += 1;
    }

    // Convert to sorted list (top 10 by amount)
    let mut merchants: Vec<MerchantSpending> = by_merchant
        .into_iter()
        .map(|(merchant, (total, count))| MerchantSpending {
            merchant,
            total_amount: total,
            transaction_count: count,
            percentage: if total_spending > 0.0 {
                (total / total_spending) * 100.0
            } else {
                0.0
            },
        })
        .collect();

    merchants.sort_by(|a, b| b.total_amount.partial_cmp(&a.total_amount).unwrap());
    merchants.truncate(10);

    Ok(HttpResponse::Ok().json(TopMerchantsResponse {
        merchants,
        total_spending,
    }))
}

/// Extract a simplified merchant name from transaction description.
fn extract_merchant_name(description: &str) -> String {
    // Remove common prefixes and clean up
    let cleaned = description
        .trim()
        .to_uppercase()
        .replace("COMPRA EN ", "")
        .replace("COMPRA ", "")
        .replace("PAGO ", "")
        .replace("TRANSFERENCIA A ", "")
        .replace("PAGO PSE ", "");

    // Take first few words (up to 30 chars) as merchant identifier
    let words: Vec<&str> = cleaned.split_whitespace().take(3).collect();
    let merchant = words.join(" ");

    if merchant.len() > 30 {
        merchant[..30].to_string()
    } else if merchant.is_empty() {
        "OTROS".to_string()
    } else {
        merchant
    }
}

/// Get Spanish month name.
fn get_month_name(month: u32) -> String {
    match month {
        1 => "Ene",
        2 => "Feb",
        3 => "Mar",
        4 => "Abr",
        5 => "May",
        6 => "Jun",
        7 => "Jul",
        8 => "Ago",
        9 => "Sep",
        10 => "Oct",
        11 => "Nov",
        12 => "Dic",
        _ => "???",
    }
    .to_string()
}

/// Configure analytics routes.
pub fn analytics_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_summary)
        .service(get_spending_by_category)
        .service(get_monthly_trends)
        .service(get_top_expenses)
        .service(get_top_merchants);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_summary_response_serialization() {
        let response = SummaryResponse {
            total_income: 5000000.0,
            total_expenses: 3500000.0,
            net_balance: 1500000.0,
            transaction_count: 50,
            days_in_period: 30,
            daily_average: 116666.67,
            savings_rate: 30.0,
            largest_expense: 500000.0,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"total_income\":5000000.0"));
        assert!(json.contains("\"net_balance\":1500000.0"));
        assert!(json.contains("\"savings_rate\":30.0"));
    }

    #[test]
    fn test_category_breakdown_serialization() {
        let breakdown = CategoryBreakdown {
            category_id: Some(Uuid::nil()),
            category_name: "Mercado".to_string(),
            category_key: Some("groceries".to_string()),
            category_color: Some("#22c55e".to_string()),
            total_amount: 500000.0,
            transaction_count: 10,
            percentage: 25.5,
        };

        let json = serde_json::to_string(&breakdown).unwrap();
        assert!(json.contains("\"category_name\":\"Mercado\""));
        assert!(json.contains("\"percentage\":25.5"));
    }

    #[test]
    fn test_monthly_data_point_serialization() {
        let point = MonthlyDataPoint {
            year: 2026,
            month: 3,
            month_name: "Mar".to_string(),
            income: 4000000.0,
            expenses: 3000000.0,
            net: 1000000.0,
        };

        let json = serde_json::to_string(&point).unwrap();
        assert!(json.contains("\"month_name\":\"Mar\""));
        assert!(json.contains("\"net\":1000000.0"));
    }

    #[test]
    fn test_get_month_name() {
        assert_eq!(get_month_name(1), "Ene");
        assert_eq!(get_month_name(6), "Jun");
        assert_eq!(get_month_name(12), "Dic");
        assert_eq!(get_month_name(13), "???");
    }
}
