pub fn filter_link(title: Option<String>, service: Option<String>) -> String {
    let mut encoded = form_urlencoded::Serializer::new(String::new());

    if title.is_some() {
        encoded.append_pair("title", &title.unwrap());
    }
    if service.is_some() {
        encoded.append_pair("service", &service.unwrap());
    }

    format!("/events?{}", encoded.finish())
}

pub fn color_class(n: u64) -> String{
    if n == 0 {
        "trend0".to_string()
    } else if n < 10 {
        "trend1".to_string()
    } else if n < 100 {
        "trend2".to_string()
    } else if n < 1000 {
        "trend3".to_string()
    } else {
        "trend4".to_string()
    }
}
