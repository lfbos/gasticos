//! Rule-based categorization using regex patterns.

use crate::{CategorizationError, SystemCategory};
use once_cell::sync::Lazy;
use regex::Regex;

/// A categorization rule with a regex pattern.
pub struct CategorizationRule {
    pub name: &'static str,
    pub pattern: Regex,
    pub category: SystemCategory,
    pub priority: u8,
}

impl CategorizationRule {
    /// Create a new categorization rule.
    pub fn new(
        name: &'static str,
        pattern: &str,
        category: SystemCategory,
        priority: u8,
    ) -> Result<Self, CategorizationError> {
        let regex =
            Regex::new(pattern).map_err(|e| CategorizationError::InvalidPattern(e.to_string()))?;
        Ok(Self {
            name,
            pattern: regex,
            category,
            priority,
        })
    }

    /// Check if this rule matches the description.
    pub fn matches(&self, description: &str) -> bool {
        self.pattern.is_match(description)
    }
}

/// Pre-built categorization rules for Colombian transactions.
pub static CATEGORIZATION_RULES: Lazy<Vec<CategorizationRule>> = Lazy::new(|| {
    vec![
        // ====================================================================
        // FINANCIAL PATTERNS (High priority - specific patterns)
        // ====================================================================
        CategorizationRule::new(
            "4x1000_tax",
            r"(?i)(4\s*[xX]\s*1\.?000|GMF|GRAVAMEN)",
            SystemCategory::Financial,
            100,
        )
        .unwrap(),
        CategorizationRule::new(
            "bank_fee",
            r"(?i)(CUOTA\s+MANEJO|COMISI[OÓ]N|IVA\s+COMISI|COBRO\s+SERVICIO)",
            SystemCategory::Financial,
            100,
        )
        .unwrap(),
        CategorizationRule::new(
            "insurance",
            r"(?i)(SEGURO|P[OÓ]LIZA|PRIMA\s+SEGURO)",
            SystemCategory::Financial,
            90,
        )
        .unwrap(),
        CategorizationRule::new(
            "interest",
            r"(?i)(INTER[EÉ]S|INTERESES|RENDIMIENTO)",
            SystemCategory::Financial,
            90,
        )
        .unwrap(),
        CategorizationRule::new(
            "withdrawal",
            r"(?i)(RETIRO|CAJERO|ATM|AVANCE)",
            SystemCategory::Financial,
            80,
        )
        .unwrap(),
        CategorizationRule::new(
            "transfer",
            r"(?i)(TRANSFERENCIA|PSE|PAGO\s+PSE)",
            SystemCategory::Financial,
            70,
        )
        .unwrap(),

        // ====================================================================
        // SUBSCRIPTIONS (Specific patterns)
        // ====================================================================
        CategorizationRule::new(
            "streaming_subscription",
            r"(?i)(NETFLIX|SPOTIFY|DISNEY|HBO|PRIME\s+VIDEO|APPLE\s+TV|YOUTUBE\s+PREMIUM|DEEZER|PARAMOUNT|STAR\+)",
            SystemCategory::Subscriptions,
            100,
        )
        .unwrap(),
        CategorizationRule::new(
            "cloud_subscription",
            r"(?i)(GOOGLE\s+(ONE|STORAGE)|ICLOUD|DROPBOX|MICROSOFT\s*365|OFFICE\s*365|ADOBE)",
            SystemCategory::Subscriptions,
            100,
        )
        .unwrap(),
        CategorizationRule::new(
            "gaming_subscription",
            r"(?i)(PLAYSTATION|PS\s*PLUS|XBOX|NINTENDO|STEAM|TWITCH)",
            SystemCategory::Subscriptions,
            100,
        )
        .unwrap(),

        // ====================================================================
        // TRANSPORTATION
        // ====================================================================
        CategorizationRule::new(
            "ride_sharing",
            r"(?i)(UBER|DIDI|CABIFY|BEAT|INDRIVER)\s*\*?",
            SystemCategory::Transportation,
            90,
        )
        .unwrap(),
        CategorizationRule::new(
            "public_transport",
            r"(?i)(SITP|TRANSMILENIO|METRO\s+DE|MIO\s+CALI|MEGABUS|TULLAVE|TU\s+LLAVE|MASIVO)",
            SystemCategory::Transportation,
            90,
        )
        .unwrap(),
        CategorizationRule::new(
            "gas_station",
            r"(?i)(TERPEL|PRIMAX|MOBIL|TEXACO|BIOMAX|BRIO|ESSO|EDS\s|GASOLINA|COMBUSTIBLE|ESTACI[OÓ]N\s+SERVICIO)",
            SystemCategory::Transportation,
            90,
        )
        .unwrap(),
        CategorizationRule::new(
            "toll",
            r"(?i)(PEAJE|AUTOPISTA|CONCESI[OÓ]N)",
            SystemCategory::Transportation,
            90,
        )
        .unwrap(),
        CategorizationRule::new(
            "parking",
            r"(?i)(PARQUEADERO|PARKING|ESTACIONAMIENTO)",
            SystemCategory::Transportation,
            80,
        )
        .unwrap(),
        CategorizationRule::new(
            "airline",
            r"(?i)(AVIANCA|LATAM|VIVA\s*AIR|WINGO|SATENA|COPA\s+AIRLINES)",
            SystemCategory::Transportation,
            90,
        )
        .unwrap(),

        // ====================================================================
        // GROCERIES
        // ====================================================================
        CategorizationRule::new(
            "supermarket",
            r"(?i)(EXITO|CARULLA|JUMBO|OLIMPICA|SAO|SURTIMAX|MAKRO|PRICESMART|ALKOSTO|ARA|CENCOSUD)",
            SystemCategory::Groceries,
            90,
        )
        .unwrap(),
        CategorizationRule::new(
            "discount_store",
            r"(?i)(D1|TIENDAS\s+D1|JUSTO\s*[Y&]\s*BUENO|[IÍ]SIMO|EURO\s+SUPERMERCADO|CORATIENDAS)",
            SystemCategory::Groceries,
            90,
        )
        .unwrap(),
        CategorizationRule::new(
            "generic_grocery",
            r"(?i)(SUPERMERCADO|MINIMARKET|FRUVER|SURTIFRUVER|MERCADO|TIENDA|ABARROTES)",
            SystemCategory::Groceries,
            60,
        )
        .unwrap(),

        // ====================================================================
        // RESTAURANTS
        // ====================================================================
        CategorizationRule::new(
            "delivery_app",
            r"(?i)(RAPPI|IFOOD|PEDIDOS\s*YA|UBER\s*EATS|DIDI\s*FOOD|DOMICILIOS\.COM)",
            SystemCategory::Restaurants,
            90,
        )
        .unwrap(),
        CategorizationRule::new(
            "fast_food",
            r"(?i)(MCDONALDS|MC\s+DONALDS|BURGER\s+KING|KFC|SUBWAY|DOMINOS|PIZZA\s+HUT|PAPA\s+JOHNS|FRISBY|KOKORIKO|PRESTO|EL\s+CORRAL|CREPES|JENOS|ARCHIES|WINGSTOP)",
            SystemCategory::Restaurants,
            90,
        )
        .unwrap(),
        CategorizationRule::new(
            "coffee_shop",
            r"(?i)(STARBUCKS|JUAN\s+VALDEZ|OMAS|TOSTAO|DUNKIN)",
            SystemCategory::Restaurants,
            85,
        )
        .unwrap(),
        CategorizationRule::new(
            "generic_restaurant",
            r"(?i)(RESTAURANTE|PANADERIA|PANADER[IÍ]A|CAFETERIA|ASADERO|COMIDAS|ALMUERZO)",
            SystemCategory::Restaurants,
            50,
        )
        .unwrap(),

        // ====================================================================
        // HEALTH
        // ====================================================================
        CategorizationRule::new(
            "pharmacy",
            r"(?i)(DROGUERIA|FARMACIA|FARMATODO|LA\s+REBAJA|CRUZ\s+VERDE|LOCATEL|AUDIFARMA|PASTEUR|COLSUBSIDIO\s+DROG)",
            SystemCategory::Health,
            90,
        )
        .unwrap(),
        CategorizationRule::new(
            "health_services",
            r"(?i)(EPS|CL[IÍ]NICA|HOSPITAL|LABORATORIO|[OÓ]PTICA|ODONTOLOG[IÍ]A|CONSULTORIO|MEDICINA\s+PREPAGADA)",
            SystemCategory::Health,
            90,
        )
        .unwrap(),

        // ====================================================================
        // HOUSING
        // ====================================================================
        CategorizationRule::new(
            "utilities",
            r"(?i)(EPM|CODENSA|ENEL|VANTI|GAS\s+NATURAL|ACUEDUCTO|SERVICIOS\s+P[UÚ]BLICOS|ENERG[IÍ]A|AGUA)",
            SystemCategory::Housing,
            90,
        )
        .unwrap(),
        CategorizationRule::new(
            "rent_admin",
            r"(?i)(ARRIENDO|ADMINISTRACI[OÓ]N|CONJUNTO|PROPIEDAD\s+HORIZONTAL)",
            SystemCategory::Housing,
            85,
        )
        .unwrap(),
        CategorizationRule::new(
            "home_improvement",
            r"(?i)(HOMECENTER|HOME\s+CENTER|EASY|CONSTRUCTOR|FERRETER[IÍ]A|PINTUCO)",
            SystemCategory::Housing,
            80,
        )
        .unwrap(),

        // ====================================================================
        // EDUCATION
        // ====================================================================
        CategorizationRule::new(
            "education_institution",
            r"(?i)(UNIVERSIDAD|COLEGIO|ESCUELA|ICETEX|SENA|MATR[IÍ]CULA|PENSI[OÓ]N\s+ESCOLAR)",
            SystemCategory::Education,
            90,
        )
        .unwrap(),
        CategorizationRule::new(
            "online_education",
            r"(?i)(PLATZI|COURSERA|UDEMY|LINKEDIN\s+LEARNING|DUOLINGO)",
            SystemCategory::Education,
            90,
        )
        .unwrap(),
        CategorizationRule::new(
            "bookstore",
            r"(?i)(LIBRER[IÍ]A|PANAMERICANA|LIBROS)",
            SystemCategory::Education,
            70,
        )
        .unwrap(),

        // ====================================================================
        // ENTERTAINMENT
        // ====================================================================
        CategorizationRule::new(
            "cinema",
            r"(?i)(CINE\s+COLOMBIA|CINEMARK|PROCINAL|ROYAL\s+FILMS|CINEPOLIS)",
            SystemCategory::Entertainment,
            90,
        )
        .unwrap(),
        CategorizationRule::new(
            "events",
            r"(?i)(TUBOLETA|TU\s+BOLETA|TICKETMASTER|PRIMERA\s+FILA|CONCIERTO|TEATRO)",
            SystemCategory::Entertainment,
            90,
        )
        .unwrap(),
        CategorizationRule::new(
            "gym",
            r"(?i)(GIMNASIO|GYM|BODYTECH|SMART\s*FIT|SPINNING\s+CENTER)",
            SystemCategory::Entertainment,
            85,
        )
        .unwrap(),

        // ====================================================================
        // CLOTHING
        // ====================================================================
        CategorizationRule::new(
            "clothing_store",
            r"(?i)(ZARA|H&M|FALABELLA|ARTURO\s+CALLE|GEF|TENNIS|STUDIO\s+F|KOAJ|OFFCORSS|BERSHKA|PULL\s*&?\s*BEAR|STRADIVARIUS|AMERICANINO|CHEVIGNON|FOREVER\s*21|VELEZ)",
            SystemCategory::Clothing,
            90,
        )
        .unwrap(),
        CategorizationRule::new(
            "sportswear",
            r"(?i)(ADIDAS|NIKE|PUMA|REEBOK|UNDER\s+ARMOUR)",
            SystemCategory::Clothing,
            85,
        )
        .unwrap(),
        CategorizationRule::new(
            "generic_clothing",
            r"(?i)(CUEROS|CALZADO|ZAPATER[IÍ]A|ROPA|BOUTIQUE)",
            SystemCategory::Clothing,
            50,
        )
        .unwrap(),

        // ====================================================================
        // TECHNOLOGY
        // ====================================================================
        CategorizationRule::new(
            "tech_store",
            r"(?i)(KTRONIX|APPLE\s+STORE|SAMSUNG|HUAWEI|XIAOMI|ISHOP)",
            SystemCategory::Technology,
            90,
        )
        .unwrap(),
        CategorizationRule::new(
            "ecommerce",
            r"(?i)(MERCADO\s*LIBRE|AMAZON|LINIO|FALABELLA\.COM|EXITO\.COM)",
            SystemCategory::Technology,
            70,
        )
        .unwrap(),
        CategorizationRule::new(
            "mobile_carrier",
            r"(?i)(CLARO|MOVISTAR|TIGO|WOM|VIRGIN|ETB|RECARGA)",
            SystemCategory::Technology,
            80,
        )
        .unwrap(),
    ]
});

/// Match a transaction description against all rules.
/// Returns the best matching category with its confidence.
pub fn match_rules(description: &str) -> Option<(SystemCategory, f32, &'static str)> {
    let upper = description.to_uppercase();

    let mut best_match: Option<(&CategorizationRule, f32)> = None;

    for rule in CATEGORIZATION_RULES.iter() {
        if rule.matches(&upper) {
            let confidence = rule.priority as f32 / 100.0;

            if best_match.is_none() || confidence > best_match.unwrap().1 {
                best_match = Some((rule, confidence));
            }
        }
    }

    best_match.map(|(rule, confidence)| (rule.category, confidence, rule.name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_financial_rules() {
        let (cat, conf, name) = match_rules("CUOTA MANEJO TC").unwrap();
        assert_eq!(cat, SystemCategory::Financial);
        assert!(conf >= 0.9);
        assert_eq!(name, "bank_fee");

        let (cat, _, _) = match_rules("GMF 4X1000").unwrap();
        assert_eq!(cat, SystemCategory::Financial);

        let (cat, _, _) = match_rules("RETIRO CAJERO").unwrap();
        assert_eq!(cat, SystemCategory::Financial);
    }

    #[test]
    fn test_match_subscription_rules() {
        let (cat, _, _) = match_rules("NETFLIX.COM").unwrap();
        assert_eq!(cat, SystemCategory::Subscriptions);

        let (cat, _, _) = match_rules("SPOTIFY PREMIUM").unwrap();
        assert_eq!(cat, SystemCategory::Subscriptions);

        let (cat, _, _) = match_rules("GOOGLE ONE").unwrap();
        assert_eq!(cat, SystemCategory::Subscriptions);
    }

    #[test]
    fn test_match_transport_rules() {
        let (cat, _, _) = match_rules("UBER *TRIP").unwrap();
        assert_eq!(cat, SystemCategory::Transportation);

        let (cat, _, _) = match_rules("SITP RECARGA").unwrap();
        assert_eq!(cat, SystemCategory::Transportation);

        let (cat, _, _) = match_rules("TERPEL EDS").unwrap();
        assert_eq!(cat, SystemCategory::Transportation);

        let (cat, _, _) = match_rules("PEAJE AUTOPISTA NORTE").unwrap();
        assert_eq!(cat, SystemCategory::Transportation);
    }

    #[test]
    fn test_match_groceries_rules() {
        let (cat, _, _) = match_rules("ALMACENES EXITO").unwrap();
        assert_eq!(cat, SystemCategory::Groceries);

        let (cat, _, _) = match_rules("TIENDAS D1").unwrap();
        assert_eq!(cat, SystemCategory::Groceries);

        let (cat, _, _) = match_rules("JUSTO Y BUENO").unwrap();
        assert_eq!(cat, SystemCategory::Groceries);
    }

    #[test]
    fn test_match_restaurant_rules() {
        let (cat, _, _) = match_rules("RAPPI*DOMICILIO").unwrap();
        assert_eq!(cat, SystemCategory::Restaurants);

        let (cat, _, _) = match_rules("MCDONALDS").unwrap();
        assert_eq!(cat, SystemCategory::Restaurants);

        let (cat, _, _) = match_rules("JUAN VALDEZ CAFE").unwrap();
        assert_eq!(cat, SystemCategory::Restaurants);
    }

    #[test]
    fn test_no_match() {
        assert!(match_rules("RANDOM TRANSACTION XYZ").is_none());
    }

    #[test]
    fn test_case_insensitive() {
        let (cat1, _, _) = match_rules("netflix").unwrap();
        let (cat2, _, _) = match_rules("NETFLIX").unwrap();
        let (cat3, _, _) = match_rules("Netflix").unwrap();
        assert_eq!(cat1, cat2);
        assert_eq!(cat2, cat3);
    }
}
