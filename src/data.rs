use anyhow::Result;
use std::collections::HashMap;
use yahoo_finance_api as yahoo;

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

impl Portfolio {
    pub fn load(profile: &str) -> Result<Portfolio> {
        let s = std::fs::read_to_string(format!("{profile}.toml"))?;
        let p = toml::from_str(&s)?;
        Ok(p)
    }
    pub fn save(&self, profile: &str) -> Result<()> {
        let s = toml::to_string(self)?;
        std::fs::write(&format!("{profile}.toml"), &s)?;
        Ok(())
    }
    pub async fn update_quotes(&mut self) -> bool {
        let mut updated = false;
        for s in &mut self.securities {
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

    pub fn update_amounts(&mut self, amounts: &[u32]) {
        for (s, &a) in self.securities.iter_mut().zip(amounts) {
            s.amount += a;
        }
    }
}

pub fn populate_rows(p: &Portfolio) -> Vec<Row> {
    let mut groups = HashMap::new();
    let mut tot_value = 0.;
    for s in &p.securities {
        let entry = groups.entry(&s.category).or_insert_with(Vec::new);
        entry.push(s.clone());
        tot_value += s.amount as f64 * s.latest_value;
    }
    let mut rows = Vec::new();
    for c in &p.categories {
        let securities = groups.get(&c.name).unwrap();
        let value = securities
            .iter()
            .map(|s| s.amount as f64 * s.latest_value)
            .sum();
        rows.push(Row {
            name: c.name.clone(),
            value,
            target_percentage: 100. * c.target_percentage,
            current_percentage: 100. * value / tot_value,
            is_category: true,
            out_of_date: false,
        });
        for s in securities {
            let value = s.amount as f64 * s.latest_value;
            rows.push(Row {
                name: s.name.clone(),
                value,
                target_percentage: 100. * s.target_percentage * c.target_percentage,
                current_percentage: 100. * value / tot_value,
                is_category: false,
                out_of_date: false,
            });
        }
    }
    rows.push(Row {
        name: "Total".to_owned(),
        value: tot_value,
        target_percentage: 100.,
        current_percentage: 100.,
        is_category: true,
        out_of_date: false,
    });
    rows
}
