pub mod models;
pub mod forms;
pub mod storage;
pub mod images;
pub mod handlers_web;
pub mod handlers_api;
pub mod handlers_api_hocr;

#[allow(dead_code)]
fn parse_date(s: &str) -> Option<time::Date> {
    let mut parts = s.splitn(3, '-');
    let year: i32 = parts.next()?.parse().ok()?;
    let month: u8 = parts.next()?.parse().ok()?;
    let day: u8 = parts.next()?.parse().ok()?;
    let month = time::Month::try_from(month).ok()?;
    time::Date::from_calendar_date(year, month, day).ok()
}
