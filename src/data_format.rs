use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct DataFormat {
    pub LC_TIME: LC_TIME,
    pub LC_NUMERIC: LC_NUMERIC,
    pub LC_MONETARY: LC_MONETARY,
    pub LC_COLLATE: LC_COLLATE,
    pub PLURAL_RULES: String,
}

#[derive(Deserialize, Debug)]
#[derive(Clone)]
pub struct LC_TIME {
    pub days: Vec<String>,
    pub months: Vec<String>,
    pub date_fmt: String,
}

#[derive(Deserialize, Debug)]
#[derive(Clone)]
pub struct LC_NUMERIC {
    pub decimal_point: String,
    pub thousands_sep: String,
    pub grouping: Vec<u8>,
}

#[derive(Deserialize, Debug)]
#[derive(Clone)]
pub struct LC_MONETARY {
    pub currency_symbol: String,
    pub int_curr_symbol: String,
    pub mon_decimal_point: String,
    pub mon_thousands_sep: String,
    pub positive_sign: String,
    pub negative_sign: String,
    pub frac_digits: u8,
}

#[derive(Deserialize, Debug)]
#[derive(Clone)]
pub struct LC_COLLATE {
    pub sort_order: String,
}