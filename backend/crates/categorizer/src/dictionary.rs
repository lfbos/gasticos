//! Colombian merchant dictionary for transaction categorization.
//!
//! Maps merchant names/patterns to categories based on Colombian market.

use crate::SystemCategory;
use once_cell::sync::Lazy;
use std::collections::HashMap;

/// A merchant entry with category and optional aliases.
#[derive(Debug, Clone)]
pub struct MerchantEntry {
    pub category: SystemCategory,
    pub patterns: Vec<&'static str>,
}

/// Colombian merchant dictionary.
/// Maps uppercase patterns to categories.
pub static MERCHANT_DICTIONARY: Lazy<HashMap<&'static str, SystemCategory>> = Lazy::new(|| {
    let mut map = HashMap::new();

    // ========================================================================
    // GROCERIES (Mercado)
    // ========================================================================
    // Supermarkets
    map.insert("EXITO", SystemCategory::Groceries);
    map.insert("ALMACENES EXITO", SystemCategory::Groceries);
    map.insert("CARULLA", SystemCategory::Groceries);
    map.insert("JUMBO", SystemCategory::Groceries);
    map.insert("METRO", SystemCategory::Groceries);
    map.insert("OLIMPICA", SystemCategory::Groceries);
    map.insert("SUPERTIENDAS OLIMPICA", SystemCategory::Groceries);
    map.insert("SAO", SystemCategory::Groceries);
    map.insert("SURTIMAX", SystemCategory::Groceries);
    map.insert("MAKRO", SystemCategory::Groceries);
    map.insert("PRICESMART", SystemCategory::Groceries);
    map.insert("PRICE SMART", SystemCategory::Groceries);
    map.insert("ALKOSTO", SystemCategory::Groceries);
    map.insert("ARA", SystemCategory::Groceries);
    map.insert("TIENDAS ARA", SystemCategory::Groceries);
    map.insert("CENCOSUD", SystemCategory::Groceries);

    // Discount stores (D1, Justo & Bueno, etc.)
    map.insert("D1", SystemCategory::Groceries);
    map.insert("TIENDAS D1", SystemCategory::Groceries);
    map.insert("JUSTO Y BUENO", SystemCategory::Groceries);
    map.insert("JUSTO&BUENO", SystemCategory::Groceries);
    map.insert("ÍSIMO", SystemCategory::Groceries);
    map.insert("ISIMO", SystemCategory::Groceries);
    map.insert("EURO SUPERMERCADO", SystemCategory::Groceries);
    map.insert("CORATIENDAS", SystemCategory::Groceries);
    map.insert("MERCACENTRO", SystemCategory::Groceries);
    map.insert("SUPERMERCADO", SystemCategory::Groceries);
    map.insert("MINIMARKET", SystemCategory::Groceries);
    map.insert("FRUVER", SystemCategory::Groceries);
    map.insert("SURTIFRUVER", SystemCategory::Groceries);

    // ========================================================================
    // RESTAURANTS (Restaurantes)
    // ========================================================================
    // Delivery apps
    map.insert("RAPPI", SystemCategory::Restaurants);
    map.insert("RAPPI*", SystemCategory::Restaurants);
    map.insert("IFOOD", SystemCategory::Restaurants);
    map.insert("PEDIDOSYA", SystemCategory::Restaurants);
    map.insert("PEDIDOS YA", SystemCategory::Restaurants);
    map.insert("DOMICILIOS", SystemCategory::Restaurants);
    map.insert("UBER EATS", SystemCategory::Restaurants);
    map.insert("UBEREATS", SystemCategory::Restaurants);
    map.insert("DIDI FOOD", SystemCategory::Restaurants);

    // Fast food chains
    map.insert("MCDONALDS", SystemCategory::Restaurants);
    map.insert("MC DONALDS", SystemCategory::Restaurants);
    map.insert("BURGER KING", SystemCategory::Restaurants);
    map.insert("WENDYS", SystemCategory::Restaurants);
    map.insert("KFC", SystemCategory::Restaurants);
    map.insert("SUBWAY", SystemCategory::Restaurants);
    map.insert("DOMINOS", SystemCategory::Restaurants);
    map.insert("PIZZA HUT", SystemCategory::Restaurants);
    map.insert("PAPA JOHNS", SystemCategory::Restaurants);
    map.insert("FRISBY", SystemCategory::Restaurants);
    map.insert("KOKORIKO", SystemCategory::Restaurants);
    map.insert("PRESTO", SystemCategory::Restaurants);
    map.insert("EL CORRAL", SystemCategory::Restaurants);
    map.insert("CREPES", SystemCategory::Restaurants);
    map.insert("JENO'S PIZZA", SystemCategory::Restaurants);
    map.insert("JENOS PIZZA", SystemCategory::Restaurants);
    map.insert("ARCHIES", SystemCategory::Restaurants);
    map.insert("HAMBURGUESAS", SystemCategory::Restaurants);
    map.insert("WINGSTOP", SystemCategory::Restaurants);

    // Coffee shops
    map.insert("STARBUCKS", SystemCategory::Restaurants);
    map.insert("JUAN VALDEZ", SystemCategory::Restaurants);
    map.insert("OMAS", SystemCategory::Restaurants);
    map.insert("TOSTAO", SystemCategory::Restaurants);
    map.insert("DUNKIN", SystemCategory::Restaurants);

    // Generic
    map.insert("RESTAURANTE", SystemCategory::Restaurants);
    map.insert("PANADERIA", SystemCategory::Restaurants);
    map.insert("CAFETERIA", SystemCategory::Restaurants);
    map.insert("ASADERO", SystemCategory::Restaurants);

    // ========================================================================
    // TRANSPORTATION (Transporte)
    // ========================================================================
    // Ride-sharing
    map.insert("UBER", SystemCategory::Transportation);
    map.insert("UBER*", SystemCategory::Transportation);
    map.insert("DIDI", SystemCategory::Transportation);
    map.insert("DIDI*", SystemCategory::Transportation);
    map.insert("CABIFY", SystemCategory::Transportation);
    map.insert("BEAT", SystemCategory::Transportation);
    map.insert("INDRIVER", SystemCategory::Transportation);

    // Public transport
    map.insert("SITP", SystemCategory::Transportation);
    map.insert("TRANSMILENIO", SystemCategory::Transportation);
    map.insert("METRO DE MEDELLIN", SystemCategory::Transportation);
    map.insert("MIO", SystemCategory::Transportation);
    map.insert("MEGABUS", SystemCategory::Transportation);
    map.insert("MASIVO", SystemCategory::Transportation);
    map.insert("TULLAVE", SystemCategory::Transportation);
    map.insert("TU LLAVE", SystemCategory::Transportation);

    // Gas stations
    map.insert("TERPEL", SystemCategory::Transportation);
    map.insert("PRIMAX", SystemCategory::Transportation);
    map.insert("MOBIL", SystemCategory::Transportation);
    map.insert("TEXACO", SystemCategory::Transportation);
    map.insert("BIOMAX", SystemCategory::Transportation);
    map.insert("BRIO", SystemCategory::Transportation);
    map.insert("ESSO", SystemCategory::Transportation);
    map.insert("GASOLINA", SystemCategory::Transportation);
    map.insert("COMBUSTIBLE", SystemCategory::Transportation);
    map.insert("EDS", SystemCategory::Transportation);
    map.insert("ESTACION DE SERVICIO", SystemCategory::Transportation);

    // Tolls
    map.insert("PEAJE", SystemCategory::Transportation);
    map.insert("AUTOPISTA", SystemCategory::Transportation);

    // Parking
    map.insert("PARQUEADERO", SystemCategory::Transportation);
    map.insert("PARKING", SystemCategory::Transportation);

    // Airlines
    map.insert("AVIANCA", SystemCategory::Transportation);
    map.insert("LATAM", SystemCategory::Transportation);
    map.insert("VIVA AIR", SystemCategory::Transportation);
    map.insert("WINGO", SystemCategory::Transportation);
    map.insert("SATENA", SystemCategory::Transportation);
    map.insert("COPA AIRLINES", SystemCategory::Transportation);

    // ========================================================================
    // SUBSCRIPTIONS (Suscripciones)
    // ========================================================================
    // Streaming
    map.insert("NETFLIX", SystemCategory::Subscriptions);
    map.insert("SPOTIFY", SystemCategory::Subscriptions);
    map.insert("AMAZON PRIME", SystemCategory::Subscriptions);
    map.insert("DISNEY+", SystemCategory::Subscriptions);
    map.insert("DISNEY PLUS", SystemCategory::Subscriptions);
    map.insert("HBO MAX", SystemCategory::Subscriptions);
    map.insert("PARAMOUNT", SystemCategory::Subscriptions);
    map.insert("APPLE TV", SystemCategory::Subscriptions);
    map.insert("YOUTUBE PREMIUM", SystemCategory::Subscriptions);
    map.insert("CRUNCHYROLL", SystemCategory::Subscriptions);
    map.insert("DEEZER", SystemCategory::Subscriptions);
    map.insert("APPLE MUSIC", SystemCategory::Subscriptions);
    map.insert("AMAZON MUSIC", SystemCategory::Subscriptions);
    map.insert("STAR+", SystemCategory::Subscriptions);
    map.insert("STAR PLUS", SystemCategory::Subscriptions);

    // Cloud/Software
    map.insert("GOOGLE STORAGE", SystemCategory::Subscriptions);
    map.insert("GOOGLE ONE", SystemCategory::Subscriptions);
    map.insert("ICLOUD", SystemCategory::Subscriptions);
    map.insert("DROPBOX", SystemCategory::Subscriptions);
    map.insert("MICROSOFT 365", SystemCategory::Subscriptions);
    map.insert("OFFICE 365", SystemCategory::Subscriptions);
    map.insert("ADOBE", SystemCategory::Subscriptions);
    map.insert("CANVA", SystemCategory::Subscriptions);

    // Gaming
    map.insert("PLAYSTATION", SystemCategory::Subscriptions);
    map.insert("XBOX", SystemCategory::Subscriptions);
    map.insert("NINTENDO", SystemCategory::Subscriptions);
    map.insert("STEAM", SystemCategory::Subscriptions);
    map.insert("TWITCH", SystemCategory::Subscriptions);

    // ========================================================================
    // HEALTH (Salud)
    // ========================================================================
    // Pharmacies
    map.insert("DROGUERIA", SystemCategory::Health);
    map.insert("DROGUERIAS", SystemCategory::Health);
    map.insert("FARMACIA", SystemCategory::Health);
    map.insert("FARMATODO", SystemCategory::Health);
    map.insert("LA REBAJA", SystemCategory::Health);
    map.insert("CRUZ VERDE", SystemCategory::Health);
    map.insert("LOCATEL", SystemCategory::Health);
    map.insert("DROGAS", SystemCategory::Health);
    map.insert("COLSUBSIDIO DROG", SystemCategory::Health);
    map.insert("AUDIFARMA", SystemCategory::Health);
    map.insert("PASTEUR", SystemCategory::Health);

    // Health services
    map.insert("EPS", SystemCategory::Health);
    map.insert("CLINICA", SystemCategory::Health);
    map.insert("HOSPITAL", SystemCategory::Health);
    map.insert("LABORATORIO", SystemCategory::Health);
    map.insert("OPTICA", SystemCategory::Health);
    map.insert("ODONTOLOGIA", SystemCategory::Health);
    map.insert("CONSULTORIO", SystemCategory::Health);
    map.insert("MEDICO", SystemCategory::Health);
    map.insert("MEDICINA PREPAGADA", SystemCategory::Health);

    // ========================================================================
    // HOUSING (Vivienda)
    // ========================================================================
    // Utilities
    map.insert("EPM", SystemCategory::Housing);
    map.insert("CODENSA", SystemCategory::Housing);
    map.insert("ENEL", SystemCategory::Housing);
    map.insert("VANTI", SystemCategory::Housing);
    map.insert("GAS NATURAL", SystemCategory::Housing);
    map.insert("ACUEDUCTO", SystemCategory::Housing);
    map.insert("ETB", SystemCategory::Housing);
    map.insert("CLARO HOGAR", SystemCategory::Housing);
    map.insert("MOVISTAR HOGAR", SystemCategory::Housing);
    map.insert("TIGO HOGAR", SystemCategory::Housing);
    map.insert("SERVICIOS PUBLICOS", SystemCategory::Housing);
    map.insert("ENERGIA", SystemCategory::Housing);
    map.insert("AGUA", SystemCategory::Housing);

    // Rent/Admin
    map.insert("ARRIENDO", SystemCategory::Housing);
    map.insert("ADMINISTRACION", SystemCategory::Housing);
    map.insert("CONJUNTO", SystemCategory::Housing);
    map.insert("EDIFICIO", SystemCategory::Housing);
    map.insert("PROPIEDAD HORIZONTAL", SystemCategory::Housing);

    // Home improvement
    map.insert("HOMECENTER", SystemCategory::Housing);
    map.insert("HOME CENTER", SystemCategory::Housing);
    map.insert("EASY", SystemCategory::Housing);
    map.insert("CONSTRUCTOR", SystemCategory::Housing);
    map.insert("FERRETERIA", SystemCategory::Housing);
    map.insert("PINTUCO", SystemCategory::Housing);

    // ========================================================================
    // EDUCATION (Educacion)
    // ========================================================================
    map.insert("UNIVERSIDAD", SystemCategory::Education);
    map.insert("COLEGIO", SystemCategory::Education);
    map.insert("ESCUELA", SystemCategory::Education);
    map.insert("ICETEX", SystemCategory::Education);
    map.insert("PENSION", SystemCategory::Education);
    map.insert("MATRICULA", SystemCategory::Education);
    map.insert("SENA", SystemCategory::Education);
    map.insert("CURSO", SystemCategory::Education);
    map.insert("LIBRERIA", SystemCategory::Education);
    map.insert("PANAMERICANA", SystemCategory::Education);

    // Online education
    map.insert("PLATZI", SystemCategory::Education);
    map.insert("COURSERA", SystemCategory::Education);
    map.insert("UDEMY", SystemCategory::Education);
    map.insert("LINKEDIN LEARNING", SystemCategory::Education);
    map.insert("DUOLINGO", SystemCategory::Education);

    // ========================================================================
    // ENTERTAINMENT (Entretenimiento)
    // ========================================================================
    // Cinema
    map.insert("CINE COLOMBIA", SystemCategory::Entertainment);
    map.insert("CINEMARK", SystemCategory::Entertainment);
    map.insert("PROCINAL", SystemCategory::Entertainment);
    map.insert("ROYAL FILMS", SystemCategory::Entertainment);
    map.insert("CINEPOLIS", SystemCategory::Entertainment);

    // Events
    map.insert("TUBOLETA", SystemCategory::Entertainment);
    map.insert("TU BOLETA", SystemCategory::Entertainment);
    map.insert("TICKETMASTER", SystemCategory::Entertainment);
    map.insert("PRIMERA FILA", SystemCategory::Entertainment);
    map.insert("CONCIERTO", SystemCategory::Entertainment);
    map.insert("TEATRO", SystemCategory::Entertainment);

    // Recreation
    map.insert("SALITRE MAGICO", SystemCategory::Entertainment);
    map.insert("MUNDO AVENTURA", SystemCategory::Entertainment);
    map.insert("PARQUE", SystemCategory::Entertainment);
    map.insert("GIMNASIO", SystemCategory::Entertainment);
    map.insert("GYM", SystemCategory::Entertainment);
    map.insert("BODYTECH", SystemCategory::Entertainment);
    map.insert("SMARTFIT", SystemCategory::Entertainment);
    map.insert("SMART FIT", SystemCategory::Entertainment);
    map.insert("SPINNING CENTER", SystemCategory::Entertainment);

    // ========================================================================
    // CLOTHING (Ropa)
    // ========================================================================
    map.insert("ZARA", SystemCategory::Clothing);
    map.insert("H&M", SystemCategory::Clothing);
    map.insert("FALABELLA", SystemCategory::Clothing);
    map.insert("ARTURO CALLE", SystemCategory::Clothing);
    map.insert("GEF", SystemCategory::Clothing);
    map.insert("TENNIS", SystemCategory::Clothing);
    map.insert("STUDIO F", SystemCategory::Clothing);
    map.insert("KOAJ", SystemCategory::Clothing);
    map.insert("OFFCORSS", SystemCategory::Clothing);
    map.insert("BERSHKA", SystemCategory::Clothing);
    map.insert("PULL AND BEAR", SystemCategory::Clothing);
    map.insert("PULL&BEAR", SystemCategory::Clothing);
    map.insert("STRADIVARIUS", SystemCategory::Clothing);
    map.insert("AMERICANINO", SystemCategory::Clothing);
    map.insert("CHEVIGNON", SystemCategory::Clothing);
    map.insert("ADIDAS", SystemCategory::Clothing);
    map.insert("NIKE", SystemCategory::Clothing);
    map.insert("PUMA", SystemCategory::Clothing);
    map.insert("FOREVER 21", SystemCategory::Clothing);
    map.insert("ROSARIO", SystemCategory::Clothing);
    map.insert("VELEZ", SystemCategory::Clothing);
    map.insert("CUEROS", SystemCategory::Clothing);
    map.insert("CALZADO", SystemCategory::Clothing);
    map.insert("ZAPATERIA", SystemCategory::Clothing);

    // ========================================================================
    // TECHNOLOGY (Tecnologia)
    // ========================================================================
    // Stores
    map.insert("KTRONIX", SystemCategory::Technology);
    map.insert("ALKOSTO TECH", SystemCategory::Technology);
    map.insert("APPLE STORE", SystemCategory::Technology);
    map.insert("SAMSUNG", SystemCategory::Technology);
    map.insert("HUAWEI", SystemCategory::Technology);
    map.insert("XIAOMI", SystemCategory::Technology);
    map.insert("ISHOP", SystemCategory::Technology);
    map.insert("MERCADOLIBRE", SystemCategory::Technology);
    map.insert("MERCADO LIBRE", SystemCategory::Technology);
    map.insert("AMAZON", SystemCategory::Technology);
    map.insert("LINIO", SystemCategory::Technology);
    map.insert("FALABELLA.COM", SystemCategory::Technology);
    map.insert("EXITO.COM", SystemCategory::Technology);

    // Mobile
    map.insert("CLARO", SystemCategory::Technology);
    map.insert("MOVISTAR", SystemCategory::Technology);
    map.insert("TIGO", SystemCategory::Technology);
    map.insert("WOM", SystemCategory::Technology);
    map.insert("VIRGIN", SystemCategory::Technology);
    map.insert("ETB MOVIL", SystemCategory::Technology);
    map.insert("RECARGA", SystemCategory::Technology);
    map.insert("CELULAR", SystemCategory::Technology);

    // ========================================================================
    // FINANCIAL (Financiero)
    // ========================================================================
    // Banks
    map.insert("BANCOLOMBIA", SystemCategory::Financial);
    map.insert("DAVIVIENDA", SystemCategory::Financial);
    map.insert("BBVA", SystemCategory::Financial);
    map.insert("BANCO DE BOGOTA", SystemCategory::Financial);
    map.insert("BANCO POPULAR", SystemCategory::Financial);
    map.insert("BANCO DE OCCIDENTE", SystemCategory::Financial);
    map.insert("SCOTIABANK COLPATRIA", SystemCategory::Financial);
    map.insert("BANCO CAJA SOCIAL", SystemCategory::Financial);
    map.insert("BANCO AGRARIO", SystemCategory::Financial);
    map.insert("ITAU", SystemCategory::Financial);
    map.insert("AV VILLAS", SystemCategory::Financial);
    map.insert("NEQUI", SystemCategory::Financial);
    map.insert("DAVIPLATA", SystemCategory::Financial);
    map.insert("NU COLOMBIA", SystemCategory::Financial);
    map.insert("NUBANK", SystemCategory::Financial);
    map.insert("LULO BANK", SystemCategory::Financial);
    map.insert("UALÁ", SystemCategory::Financial);
    map.insert("UALA", SystemCategory::Financial);
    map.insert("RAPPIPAY", SystemCategory::Financial);
    map.insert("MOVII", SystemCategory::Financial);
    map.insert("DALE", SystemCategory::Financial);

    // Financial services
    map.insert("CUOTA MANEJO", SystemCategory::Financial);
    map.insert("COMISION", SystemCategory::Financial);
    map.insert("GMF", SystemCategory::Financial);
    map.insert("4X1000", SystemCategory::Financial);
    map.insert("4 X 1000", SystemCategory::Financial);
    map.insert("INTERES", SystemCategory::Financial);
    map.insert("SEGUROS", SystemCategory::Financial);
    map.insert("SEGURO", SystemCategory::Financial);
    map.insert("POLIZA", SystemCategory::Financial);
    map.insert("SURA", SystemCategory::Financial);
    map.insert("BOLIVAR", SystemCategory::Financial);
    map.insert("COLSEGUROS", SystemCategory::Financial);
    map.insert("CREDITO", SystemCategory::Financial);
    map.insert("PAGO TARJETA", SystemCategory::Financial);
    map.insert("TRANSFERENCIA", SystemCategory::Financial);
    map.insert("RETIRO", SystemCategory::Financial);
    map.insert("CAJERO", SystemCategory::Financial);
    map.insert("ATM", SystemCategory::Financial);
    map.insert("PSE", SystemCategory::Financial);

    map
});

/// Look up a category for a merchant name.
/// Returns the category if found, None otherwise.
pub fn lookup_merchant(description: &str) -> Option<SystemCategory> {
    let upper = description.to_uppercase();

    // Try exact match first
    if let Some(&category) = MERCHANT_DICTIONARY.get(upper.as_str()) {
        return Some(category);
    }

    // Try partial match - prefer longer matching patterns for more specific matches
    let mut best_match: Option<(&str, SystemCategory)> = None;

    for (pattern, &category) in MERCHANT_DICTIONARY.iter() {
        if upper.contains(pattern) {
            match best_match {
                None => best_match = Some((pattern, category)),
                Some((prev_pattern, _)) if pattern.len() > prev_pattern.len() => {
                    best_match = Some((pattern, category));
                }
                _ => {}
            }
        }
    }

    best_match.map(|(_, cat)| cat)
}

/// Get all patterns for a given category.
pub fn patterns_for_category(category: SystemCategory) -> Vec<&'static str> {
    MERCHANT_DICTIONARY
        .iter()
        .filter_map(
            |(&pattern, &cat)| {
                if cat == category {
                    Some(pattern)
                } else {
                    None
                }
            },
        )
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_exact_match() {
        assert_eq!(lookup_merchant("D1"), Some(SystemCategory::Groceries));
        assert_eq!(
            lookup_merchant("NETFLIX"),
            Some(SystemCategory::Subscriptions)
        );
        assert_eq!(
            lookup_merchant("UBER"),
            Some(SystemCategory::Transportation)
        );
    }

    #[test]
    fn test_lookup_case_insensitive() {
        assert_eq!(
            lookup_merchant("netflix"),
            Some(SystemCategory::Subscriptions)
        );
        assert_eq!(
            lookup_merchant("Netflix"),
            Some(SystemCategory::Subscriptions)
        );
        assert_eq!(
            lookup_merchant("NETFLIX"),
            Some(SystemCategory::Subscriptions)
        );
    }

    #[test]
    fn test_lookup_partial_match() {
        assert_eq!(
            lookup_merchant("COMPRA TIENDAS D1 BOGOTA"),
            Some(SystemCategory::Groceries)
        );
        assert_eq!(
            lookup_merchant("PAGO NETFLIX.COM"),
            Some(SystemCategory::Subscriptions)
        );
        assert_eq!(
            lookup_merchant("UBER *TRIP"),
            Some(SystemCategory::Transportation)
        );
    }

    #[test]
    fn test_lookup_colombian_merchants() {
        assert_eq!(lookup_merchant("EXITO"), Some(SystemCategory::Groceries));
        assert_eq!(lookup_merchant("CARULLA"), Some(SystemCategory::Groceries));
        assert_eq!(lookup_merchant("RAPPI"), Some(SystemCategory::Restaurants));
        assert_eq!(
            lookup_merchant("SITP"),
            Some(SystemCategory::Transportation)
        );
        assert_eq!(
            lookup_merchant("TRANSMILENIO"),
            Some(SystemCategory::Transportation)
        );
        assert_eq!(
            lookup_merchant("TERPEL"),
            Some(SystemCategory::Transportation)
        );
        assert_eq!(lookup_merchant("EPM"), Some(SystemCategory::Housing));
        assert_eq!(lookup_merchant("FARMATODO"), Some(SystemCategory::Health));
    }

    #[test]
    fn test_lookup_unknown() {
        assert_eq!(lookup_merchant("RANDOM UNKNOWN MERCHANT"), None);
        assert_eq!(lookup_merchant("XYZABC123"), None);
    }

    #[test]
    fn test_patterns_for_category() {
        let grocery_patterns = patterns_for_category(SystemCategory::Groceries);
        assert!(grocery_patterns.contains(&"D1"));
        assert!(grocery_patterns.contains(&"EXITO"));
        assert!(grocery_patterns.contains(&"CARULLA"));

        let transport_patterns = patterns_for_category(SystemCategory::Transportation);
        assert!(transport_patterns.contains(&"UBER"));
        assert!(transport_patterns.contains(&"SITP"));
    }
}
