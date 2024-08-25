use chrono::{Datelike, Days};

pub mod haproxy_index;
pub mod haproxy_index_entities;
pub mod haproxy_index_search_result_entities;
pub mod healthcheck_entities;
pub mod jdbc_index_entities;
pub mod jdbc_index_search_result;
pub mod missing_query;
pub mod replicasettings_entities;

pub fn today_index_name(ind_name: String) -> String {
    #[allow(unused)]
    let mut day_str = String::new();
    #[allow(unused)]
    let mut month_str = String::new();
    #[allow(unused)]
    let mut year_str = String::new();

    let tdy = chrono::Local::now();
    let day = tdy.day();
    if day < 10 {
        day_str = format!("{}{}", 0, day)
    } else {
        day_str = day.to_string()
    }
    let month = tdy.month();
    if month < 10 {
        month_str = format!("{}{}", 0, month)
    } else {
        month_str = month.to_string()
    }
    let year = tdy.year();

    let mut out = ind_name.to_string();
    if ind_name.contains("TODAY") {
        let date_str = format!("{}.{}.{}", year, month_str, day_str);
        out = ind_name.replace("TODAY", date_str.as_str());
    }
    out
}

pub fn yesterday_index_name(ind_name: String) -> String {
    #[allow(unused)]
    let mut day_str = String::new();
    #[allow(unused)]
    let mut month_str = String::new();
    #[allow(unused)]
    let mut year_str = String::new();

    let tdy = chrono::Local::now();
    let yest = tdy.checked_sub_days(Days::new(1)).unwrap();
    let day = yest.day();
    if day < 10 {
        day_str = format!("{}{}", 0, day)
    } else {
        day_str = day.to_string()
    }
    let month = tdy.month();
    if month < 10 {
        month_str = format!("{}{}", 0, month)
    } else {
        month_str = month.to_string()
    }
    let year = tdy.year();

    let mut out = ind_name.to_string();
    if ind_name.contains("YESTERDAY") {
        let date_str = format!("{}.{}.{}", year, month_str, day_str);
        out = ind_name.replace("YESTERDAY", date_str.as_str());
    }
    out
}

pub fn escape_special(input: String) -> String {
    // let output = String::new();

    // output
    escape_string::escape(input.as_str()).to_string()
}
