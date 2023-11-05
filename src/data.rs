use yahoo_finance_api as yahoo;
use std::collections::HashMap;

pub struct Row {
    pub name: String,
    pub value: f64,
    pub target_percentage: f64,
    pub current_percentage: f64,
    pub is_category: bool,
    pub out_of_date: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Security {
    pub name: String,
    pub symbol: String,
    pub url: String,
    pub amount: u32,
    pub latest_value: f64,
    pub latest_value_time: u64,
    pub target_percentage: f64,
    pub category: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Category {
    pub name: String,
    pub target_percentage: f64,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Portfolio {
    pub categories: Vec<Category>,
    pub securities: Vec<Security>,
}

pub fn load_portfolio() -> Portfolio {
    let s = std::fs::read_to_string("yuri.toml").expect("missing toml");
    let p = toml::from_str(&s).expect("invalid toml");
    p
}
pub fn save_portfolio(p: &Portfolio) {
    let s = toml::to_string(p).expect("cannot serialize to toml");
    std::fs::write("yuri.toml", &s).expect("cannot save toml");
}

pub async fn populate_rows(p: &Portfolio) -> Vec<Row> {
    let mut groups = HashMap::new();
    let mut tot_value = 0.;
    for s in &p.securities {
        let entry = groups.entry(&s.category).or_insert_with(Vec::new);
        entry.push(s.clone());
        tot_value += s.latest_value;
    }
    let mut rows = Vec::new();
    for c in &p.categories {
        let securities = groups.get(&c.name).unwrap();
        let value = securities.iter().map(|s| s.latest_value).sum();
        rows.push(Row {
            name: c.name.clone(),
            value,
            target_percentage: c.target_percentage,
            current_percentage: value / tot_value,
            is_category: true,
            out_of_date: false,
        });
        for s in securities {
            rows.push(Row {
                name: s.name.clone(),
                value,
                target_percentage: s.target_percentage*c.target_percentage,
                current_percentage: s.latest_value / tot_value,
                is_category: false,
                out_of_date: false,
            });
        }
    }
    rows
}

pub async fn update_quotes(securities: &mut [Security]) -> bool {
    let mut updated = false;
    for s in securities {
        let provider = yahoo::YahooConnector::new();
        if let Ok(resp) = provider.get_quote_range(&s.symbol, "1m", "1d").await {
            if let Ok(last) = resp.last_quote() {
                updated = true;
                s.latest_value = last.close;
                s.latest_value_time = last.timestamp;
            }
        }
    }
    updated
}
